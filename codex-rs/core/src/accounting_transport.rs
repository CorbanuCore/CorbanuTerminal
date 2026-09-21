//! Per-physical-send intent under endpoint retries, with response-local evidence.
use super::FAILURE;
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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use uuid::Uuid;

pub(crate) struct ResponseEvidence {
    sampling: Arc<Sampling>,
    attempt: OnceLock<Attempt>,
    source: Uuid,
    excluded: AtomicBool,
}

impl ResponseEvidence {
    pub(super) fn admitted(sampling: Arc<Sampling>, attempt: Attempt) -> Arc<Self> {
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
                _ => Err(anyhow::anyhow!("invalid or unbound accounting evidence")),
            };
            result.map_err(|_| {
                self.sampling.reject();
                ApiError::Stream(FAILURE.into())
            })
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
                let attempt = self.attempt.get().ok_or_else(|| anyhow::anyhow!(FAILURE))?;
                let usage = usage.map_err(|_| anyhow::anyhow!(FAILURE))?;
                self.sampling
                    .observe_patch(
                        attempt,
                        self.source,
                        position,
                        super::responses::patch(usage)?,
                    )
                    .await
            }
            .await;
            result.map_err(|_| {
                self.sampling.reject();
                ApiError::Stream(FAILURE.into())
            })
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
                let attempt = self.attempt.get().ok_or_else(|| anyhow::anyhow!(FAILURE))?;
                let usage = usage.map_err(|_| anyhow::anyhow!(FAILURE))?;
                self.sampling
                    .observe_patch(attempt, self.source, position, super::chat::patch(usage)?)
                    .await
            }
            .await;
            result.map_err(|_| {
                self.sampling.reject();
                ApiError::Stream(FAILURE.into())
            })
        })
    }
}

/// Request-level routing hints can change the serving provider after selection.
/// Inspect only the routing keys; never retain or report the request payload.
///
/// The transport sees the body after preparation, which for this client can mean
/// zstd-compressed bytes. Reading those as plain JSON fails, and treating that
/// failure as "uninspectable" refused every compressed turn and collected
/// nothing, so the decoding lives with the body type that produced them.
pub(super) fn request_refusal(request: &Request) -> Option<&'static str> {
    let Some(value) = request.body.as_ref()?.inspectable_json() else {
        return Some("uninspectable request body");
    };
    // These are the names the wire actually carries. `provider_options` is
    // serialized as `providerOptions` by every request type that has it, and the
    // shipped gateway providers populate it to pin a different upstream vendor,
    // so matching only the snake_case form left Responses and Anthropic turns
    // attributable to the selected provider while the body said otherwise.
    ["provider", "providerOptions", "provider_options", "plugins"]
        .into_iter()
        .find(|key| value.get(*key).is_some())
}

pub(crate) struct AccountingTransport<T> {
    inner: T,
    evidence: Option<Arc<ResponseEvidence>>,
    model: String,
    tier: Option<String>,
}

impl<T> AccountingTransport<T> {
    pub(crate) fn new(inner: T, evidence: Option<Arc<ResponseEvidence>>, model: String) -> Self {
        Self {
            inner,
            evidence,
            model,
            tier: None,
        }
    }

    pub(crate) fn with_tier(mut self, tier: Option<String>) -> Self {
        self.tier = tier;
        self
    }
}

impl<T: HttpTransport> HttpTransport for AccountingTransport<T> {
    async fn execute(&self, request: Request) -> Result<Response, TransportError> {
        self.inner.execute(request).await
    }

    async fn stream(&self, request: Request) -> Result<StreamResponse, TransportError> {
        let Some(evidence) = &self.evidence else {
            return self.inner.stream(request).await;
        };
        if evidence.attempt.get().is_some() {
            evidence.sampling.reject();
            return Err(TransportError::Build(FAILURE.into()));
        }
        // A body that can re-route the serving provider must not be attributed to
        // the selected one, but it also must not kill the user's turn: Chat
        // declines such a request before a collector exists, and Responses and
        // Anthropic have no typed body check, so failing closed here would end the
        // turn for a request the product is happy to send. Decline to sample and
        // let it through unrecorded.
        if request_refusal(&request).is_some() {
            evidence.exclude();
            return self.inner.stream(request).await;
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
        let attempt = admission.map_err(|_| {
            evidence.sampling.reject();
            TransportError::Build(FAILURE.into())
        })?;
        let response = match self.inner.stream(request).await {
            Err(TransportError::Http { status, .. }) if status.is_redirection() => {
                evidence.sampling.reject();
                return Err(TransportError::Build(FAILURE.into()));
            }
            result => result?,
        };
        evidence.attempt.set(attempt).map_err(|_| {
            evidence.sampling.reject();
            TransportError::Build(FAILURE.into())
        })?;
        Ok(response)
    }
}
