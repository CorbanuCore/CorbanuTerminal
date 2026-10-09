//! Per-physical-send intent under endpoint retries, with response-local evidence.
use super::Sampling;
use codex_api::AnthropicUsageObserver;
use codex_api::AnthropicUsagePatch;
use codex_api::ApiError;
use codex_api::InvalidAnthropicUsage;
use codex_http_client::HttpTransport;
use codex_http_client::Request;
use codex_http_client::Response;
use codex_http_client::StreamResponse;
use codex_http_client::TransportError;
use codex_state::accounting::Attempt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use uuid::Uuid;

pub(crate) struct ResponseEvidence {
    sampling: Arc<Sampling>,
    attempt: OnceLock<Attempt>,
    source: Uuid,
    excluded: AtomicBool,
}

impl ResponseEvidence {
    pub(crate) fn admitted(sampling: Arc<Sampling>, attempt: Attempt) -> Arc<Self> {
        Arc::new(Self {
            sampling,
            attempt: OnceLock::from(attempt),
            source: Uuid::new_v4(),
            excluded: AtomicBool::new(false),
        })
    }

    pub(crate) fn new(sampling: Arc<Sampling>) -> Arc<Self> {
        Arc::new(Self {
            sampling,
            attempt: OnceLock::new(),
            source: Uuid::new_v4(),
            excluded: AtomicBool::new(false),
        })
    }

    /// Records nothing for this request and lets it proceed unaccounted.
    ///
    /// Rejecting the turn's sampling instead would abort the turn at the next
    /// `check()`, and leaving the observers armed with no admitted attempt would
    /// fail the stream on the first usage event - after the request had already
    /// been sent and billed. Neither is acceptable for a request the product is
    /// happy to serve; it is only one the accounting cannot attribute.
    pub(super) fn exclude(&self) {
        self.excluded.store(true, Ordering::SeqCst);
    }

    fn is_excluded(&self) -> bool {
        self.excluded.load(Ordering::SeqCst)
    }

    /// Whether this response is still being recorded: accounting has not
    /// given up on it or on its sampling.
    pub(crate) fn is_recorded(&self) -> bool {
        !self.is_excluded() && !self.sampling.is_closed()
    }

    /// Accounting failed for this request: record nothing more for it or the
    /// rest of its turn, and let it proceed. The request was, or will be, sent
    /// and billed; failing it would lose the provider's answer as well.
    fn give_up(&self, step: &'static str, cause: impl std::fmt::Display) {
        self.sampling.reject();
        self.exclude();
        super::gap(step, cause);
    }
}

impl AnthropicUsageObserver for ResponseEvidence {
    fn observe(
        &self,
        position: i64,
        usage: Result<AnthropicUsagePatch, InvalidAnthropicUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            if self.is_excluded() {
                return Ok(());
            }
            let result = match (self.attempt.get(), usage) {
                (Some(attempt), Ok(usage)) => {
                    self.sampling
                        .observe(attempt, self.source, position, usage)
                        .await
                }
                (None, _) => Err(anyhow::anyhow!(
                    "usage arrived before its attempt was admitted"
                )),
                (_, Err(_)) => Err(anyhow::anyhow!("provider usage failed validation")),
            };
            if let Err(error) = result {
                self.give_up("observe usage", format_args!("{error:#}"));
            }
            Ok(())
        })
    }
}

impl codex_api::ResponsesUsageObserver for ResponseEvidence {
    fn observe(
        &self,
        position: i64,
        usage: Result<codex_api::ResponsesUsagePatch, codex_api::InvalidResponsesUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            if self.is_excluded() {
                return Ok(());
            }
            let result = async {
                let attempt = self.attempt.get().ok_or_else(|| {
                    anyhow::anyhow!("usage arrived before its attempt was admitted")
                })?;
                let usage =
                    usage.map_err(|_| anyhow::anyhow!("provider usage failed validation"))?;
                self.sampling
                    .observe_patch(
                        attempt,
                        self.source,
                        position,
                        super::responses::patch(usage, &self.sampling.provider)?,
                    )
                    .await
            }
            .await;
            if let Err(error) = result {
                self.give_up("observe usage", format_args!("{error:#}"));
            }
            Ok(())
        })
    }
}

impl codex_api::ChatUsageObserver for ResponseEvidence {
    fn observe(
        &self,
        position: i64,
        usage: Result<codex_api::ChatUsagePatch, codex_api::InvalidChatUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            if self.is_excluded() {
                return Ok(());
            }
            let result = async {
                let attempt = self.attempt.get().ok_or_else(|| {
                    anyhow::anyhow!("usage arrived before its attempt was admitted")
                })?;
                let usage =
                    usage.map_err(|_| anyhow::anyhow!("provider usage failed validation"))?;
                self.sampling
                    .observe_patch(
                        attempt,
                        self.source,
                        position,
                        super::chat::patch(usage, &self.sampling.provider)?,
                    )
                    .await
            }
            .await;
            if let Err(error) = result {
                self.give_up("observe usage", format_args!("{error:#}"));
            }
            Ok(())
        })
    }
}

/// The error a collected request's transport returns when the provider
/// redirected it and no ordinary transport was supplied to resend it on
/// (`AccountingTransport::with_redirect_fallback`). Accounting has given up
/// on the request; the caller may resend it.
pub(crate) const REDIRECTED_UNRECORDED: &str =
    "provider redirected a recorded request; send it again to follow the redirect";

/// Request-level routing hints can change the serving provider after selection.
/// Inspect only the routing keys; never retain or report the request payload.
///
/// The transport sees the body after preparation, which for this client can mean
/// zstd-compressed bytes. Reading those as plain JSON fails, and treating that
/// failure as "uninspectable" refused every compressed turn and collected
/// nothing, so the decoding lives with the body type that produced them.
pub(super) fn request_refusal(
    request: &Request,
    configured_routing: Option<&serde_json::Value>,
    configured_routing_options: Option<&serde_json::Value>,
    configured_plugins: Option<&serde_json::Value>,
) -> Option<&'static str> {
    let Some(documents) = request.inspectable_json_documents() else {
        return Some("uninspectable request body");
    };
    documents.iter().find_map(|value| {
        routing_refusal(
            value,
            configured_routing,
            configured_routing_options,
            configured_plugins,
        )
    })
}

/// The routing keys one JSON document carries that this provider did not ask
/// for, or asked for with a different value.
fn routing_refusal(
    value: &serde_json::Value,
    configured_routing: Option<&serde_json::Value>,
    configured_routing_options: Option<&serde_json::Value>,
    configured_plugins: Option<&serde_json::Value>,
) -> Option<&'static str> {
    // These are the names the wire actually carries. `provider_options` is
    // serialized as `providerOptions` by every request type that has it, and the
    // shipped gateway providers populate it to pin a different upstream vendor,
    // so matching only the snake_case form left Responses and Anthropic turns
    // attributable to the selected provider while the body said otherwise.
    ["provider", "providerOptions", "provider_options", "plugins"]
        .into_iter()
        .find(|key| {
            let expected = match *key {
                "provider" => configured_routing,
                "providerOptions" | "provider_options" => configured_routing_options,
                "plugins" => configured_plugins,
                _ => None,
            };
            match (value.get(*key), expected) {
                (None, _) => false,
                // Exactly what configuration says this provider sends. The gateway
                // that was selected is the account billed for the turn.
                (Some(found), Some(expected)) if found == expected => false,
                (Some(_), _) => true,
            }
        })
}

pub(crate) struct AccountingTransport<T> {
    inner: T,
    evidence: Option<Arc<ResponseEvidence>>,
    /// The routing objects this provider is configured to send, if any.
    ///
    /// OpenRouter-compatible routes put their configured preferences in the body
    /// as `provider`; the Vercel gateway puts a configured vendor pin in
    /// `providerOptions`. Both come from provider and model configuration rather
    /// than from the turn, and the selected gateway is the account that is billed,
    /// so a body carrying exactly the configured value stays attributable. A
    /// DIFFERENT value, or a key configuration did not ask for, is not.
    configured_routing: Option<serde_json::Value>,
    configured_routing_options: Option<serde_json::Value>,
    /// OpenRouter web search rides the request-level `plugins` field, emitted by
    /// this client from the provider and the session's tool set. Same principle:
    /// the value the client itself constructed stays attributable.
    configured_plugins: Option<serde_json::Value>,
    model: String,
    tier: Option<String>,
    /// The caller's ordinary, redirect-following transport. A collected
    /// request goes out on a client that never follows a redirect, so that a
    /// response from elsewhere is never attributed to the approved route. When
    /// the provider redirects, nothing has been served yet: accounting gives up
    /// on the request and sends it again here, exactly as it would have gone
    /// with accounting off.
    follow: Option<T>,
}

impl<T> AccountingTransport<T> {
    pub(crate) fn new(inner: T, evidence: Option<Arc<ResponseEvidence>>, model: String) -> Self {
        Self {
            inner,
            evidence,
            model,
            tier: None,
            follow: None,
            configured_routing: None,
            configured_routing_options: None,
            configured_plugins: None,
        }
    }

    pub(crate) fn with_redirect_fallback(mut self, follow: Option<T>) -> Self {
        self.follow = follow;
        self
    }

    pub(crate) fn with_tier(mut self, tier: Option<String>) -> Self {
        self.tier = tier;
        self
    }

    pub(crate) fn with_configured_routing(mut self, routing: Option<serde_json::Value>) -> Self {
        self.configured_routing = routing;
        self
    }

    pub(crate) fn with_configured_routing_options(
        mut self,
        options: Option<serde_json::Value>,
    ) -> Self {
        self.configured_routing_options = options;
        self
    }

    pub(crate) fn with_configured_plugins(mut self, plugins: Option<serde_json::Value>) -> Self {
        self.configured_plugins = plugins;
        self
    }
}

impl<T: HttpTransport> AccountingTransport<T> {
    /// Admit this send, or decide it goes unrecorded. Never refuses it.
    async fn admit(&self, evidence: &ResponseEvidence, request: &Request) -> Option<Attempt> {
        if evidence.is_excluded() {
            return None;
        }
        if evidence.attempt.get().is_some() {
            evidence.give_up("admit attempt", "a second send on one admitted response");
            return None;
        }
        // A body that can re-route the serving provider must not be attributed to
        // the selected one, but it also must not kill the user's turn: Chat
        // declines such a request before a collector exists, and Responses and
        // Anthropic have no typed body check. Decline to sample and let it
        // through unrecorded.
        if let Some(reason) = request_refusal(
            request,
            self.configured_routing.as_ref(),
            self.configured_routing_options.as_ref(),
            self.configured_plugins.as_ref(),
        ) {
            // Say so once. An excluded request is still billed by the provider,
            // and silence would make it indistinguishable from a turn that never
            // sent anything. The key name is routing metadata, not payload.
            tracing::warn!(
                accounting.excluded = reason,
                "accounting: request not attributable to the selected provider; serving it unrecorded"
            );
            evidence.exclude();
            return None;
        }
        let admission =
            if evidence.sampling.dialect == codex_state::accounting::Dialect::NativeAnthropic {
                evidence.sampling.admit(&self.model, &request.url).await
            } else {
                evidence
                    .sampling
                    .admit_with_tier(&self.model, &request.url, self.tier.as_deref())
                    .await
            };
        match admission {
            Ok(attempt) => Some(attempt),
            Err(error) => {
                evidence.give_up("admit attempt", format_args!("{error:#}"));
                None
            }
        }
    }

    /// A redirected send: give up recording it and resend it on the ordinary
    /// transport, or hand the caller `REDIRECTED_UNRECORDED` when there is none.
    fn redirected(&self, evidence: &ResponseEvidence, status: http::StatusCode) -> Option<&T> {
        evidence.give_up(
            "send",
            format_args!("provider redirected the request ({status}); resending it unrecorded"),
        );
        self.follow.as_ref()
    }

    fn bind(evidence: &ResponseEvidence, attempt: Option<Attempt>) {
        if let Some(attempt) = attempt
            && evidence.attempt.set(attempt).is_err()
        {
            evidence.give_up("bind attempt", "response already bound");
        }
    }
}

impl<T: HttpTransport> HttpTransport for AccountingTransport<T> {
    /// One request, one response, no stream: the compaction endpoint.
    ///
    /// It is inference the operator paid for like any other, so it admits an
    /// attempt before the send and records whatever numbers the body carries.
    /// A body with no usage records the attempt with usage unknown, which is
    /// what the provider actually said - not zero.
    async fn execute(&self, request: Request) -> Result<Response, TransportError> {
        let Some(evidence) = &self.evidence else {
            return self.inner.execute(request).await;
        };
        let attempt = self.admit(evidence, &request).await;
        let resend = self.follow.is_some().then(|| request.clone());
        let response = match self.inner.execute(request).await {
            Err(TransportError::Http { status, .. }) if status.is_redirection() => {
                return match (self.redirected(evidence, status), resend) {
                    (Some(follow), Some(request)) => follow.execute(request).await,
                    _ => Err(TransportError::Build(REDIRECTED_UNRECORDED.into())),
                };
            }
            result => result?,
        };
        Self::bind(evidence, attempt);
        // One body, one observation: revisions are positive, and this response
        // has exactly one. A failed observation is logged and leaves the
        // response alone.
        let usage = if evidence.sampling.image_generation {
            codex_api::images_body_usage(&response.body)
        } else {
            codex_api::responses_body_usage(&response.body)
        };
        if let Ok(Some(usage)) = usage {
            let _ = codex_api::ResponsesUsageObserver::observe(
                evidence.as_ref(),
                /*position*/ 1,
                Ok(usage),
            )
            .await;
        }
        Ok(response)
    }

    async fn stream(&self, request: Request) -> Result<StreamResponse, TransportError> {
        let Some(evidence) = &self.evidence else {
            return self.inner.stream(request).await;
        };
        let attempt = self.admit(evidence, &request).await;
        let resend = self.follow.is_some().then(|| request.clone());
        let response = match self.inner.stream(request).await {
            Err(TransportError::Http { status, .. }) if status.is_redirection() => {
                return match (self.redirected(evidence, status), resend) {
                    (Some(follow), Some(request)) => follow.stream(request).await,
                    _ => Err(TransportError::Build(REDIRECTED_UNRECORDED.into())),
                };
            }
            result => result?,
        };
        Self::bind(evidence, attempt);
        Ok(response)
    }
}
