//! Collection for model clients an extension owns.
//!
//! Extensions build their own API clients - image generation is the one that
//! ships on by default - and those requests are billed to the operator like any
//! other inference. They have no `ModelClientSession` to attach to and no
//! `TurnContext` to read, so the host hands them this handle at thread start
//! and they wrap their own transport with it.
use crate::accounting::Sampling;
use crate::accounting::transport::AccountingTransport;
use crate::accounting::transport::ResponseEvidence;
use crate::config::AccountingMode;
use crate::session::session::Session;
use codex_http_client::HttpTransport;
use codex_http_client::Request;
use codex_http_client::Response;
use codex_http_client::StreamResponse;
use codex_http_client::TransportError;
use codex_model_provider_info::ModelProviderInfo;
use std::sync::Arc;
use std::sync::Weak;
use uuid::Uuid;

/// A host handle that records an extension's own model requests.
///
/// Cloneable and cheap: it holds the owning session weakly, so an extension
/// that outlives its thread records nothing rather than keeping the session
/// alive or failing.
#[derive(Clone)]
pub struct ExtensionAccounting {
    owner: Weak<Session>,
}

impl std::fmt::Debug for ExtensionAccounting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExtensionAccounting")
    }
}

impl ExtensionAccounting {
    pub(crate) fn new(owner: Weak<Session>) -> Self {
        Self { owner }
    }

    /// Wrap `transport` so the request it carries is recorded.
    ///
    /// `endpoint` is the base URL the caller's client will send to and `path`
    /// its endpoint underneath it - a request to anywhere else is refused at
    /// admission rather than attributed here.
    /// `label` names the kind of work; the recorded turn is `label:<uuid>`,
    /// because an extension's requests are their own units of work and not part
    /// of the turn that happened to trigger them.
    ///
    /// Collection is best effort: when accounting is off, the owner is gone, or
    /// this route collects nothing, the transport is returned unwrapped and the
    /// request proceeds unrecorded.
    pub async fn transport<T: HttpTransport>(
        &self,
        transport: T,
        provider: &ModelProviderInfo,
        endpoint: &str,
        model: &str,
        path: &str,
        label: &str,
    ) -> Accounted<T> {
        match self.evidence(provider, endpoint, path, label).await {
            Some(evidence) => Accounted {
                inner: AccountingTransport::new(transport, Some(evidence), model.to_string()),
            },
            None => Accounted {
                inner: AccountingTransport::new(transport, None, model.to_string()),
            },
        }
    }

    /// A transport that records nothing, for a host that supplies no handle.
    pub fn unrecorded<T: HttpTransport>(transport: T, model: &str) -> Accounted<T> {
        Accounted {
            inner: AccountingTransport::new(transport, None, model.to_string()),
        }
    }

    /// Nothing here is snapshotted. The mode and provider identity are read
    /// from the owner at call time, and the provider and `endpoint` are the
    /// caller's own - the very ones the request will use - because resolving
    /// the endpoint here from a different auth source than the caller used
    /// would pin a route the request never takes.
    async fn evidence(
        &self,
        provider: &ModelProviderInfo,
        endpoint: &str,
        path: &str,
        label: &str,
    ) -> Option<Arc<ResponseEvidence>> {
        let owner = self.owner.upgrade()?;
        let (accounting, provider_id) = owner.accounting_binding().await;
        let auth = owner.services.auth_manager.auth().await;
        let auth_mode = auth.as_ref().map(codex_login::CodexAuth::auth_mode);
        let mode = if matches!(accounting, AccountingMode::Provider { .. }) {
            crate::accounting::turn_mode(&accounting, &provider_id, provider, auth_mode, endpoint)
        } else {
            accounting
        };
        if !crate::accounting::collects(&mode, &provider_id, provider, provider.wire_api) {
            return None;
        }
        owner.try_ensure_rollout_materialized().await.ok()?;
        let runtime = owner.state_db()?;
        let sampling = Sampling::start_at_path(
            runtime,
            owner.thread_id,
            format!("{label}:{}", Uuid::new_v4()),
            &mode,
            path,
        )
        .await
        .ok()?;
        Some(ResponseEvidence::new(sampling))
    }

    /// Record a model request a client outside this process already sent.
    ///
    /// The Claude panes bridge is the reason this exists: it is a loopback HTTP
    /// server inside the TUI, and `codex-tui` does not depend on this crate, so
    /// its upstream requests cannot be wrapped in a transport from here. The
    /// host reports the send instead, and the one implementation of the policy
    /// stays in this crate rather than being written a second time above it.
    ///
    /// Two things are true of this path that are not true of the transport, and
    /// neither is hidden:
    ///
    /// - The attempt is admitted when the send is reported, which is after it
    ///   happened. A request whose reporter dies mid-flight is not recorded at
    ///   all, rather than recorded with its outcome unknown.
    /// - The route is what the caller says it sent to. There is no request here
    ///   to check it against, so the caller's own boundary decides which routes
    ///   may be reported; this records them.
    ///
    /// No money is claimed. The credential is the reporting client's, not one
    /// this session can attribute to a catalogue account, so the turn records
    /// its tokens and states no economics.
    pub async fn record_sent_request(&self, request: SentModelRequest) -> bool {
        let Some(owner) = self.owner.upgrade() else {
            return false;
        };
        let (accounting, _) = owner.accounting_binding().await;
        // The session's scope, because this spend is the operator's and belongs
        // beside the rest of it. Collection off means nothing is recorded.
        let AccountingMode::Provider { scope, .. } = accounting else {
            return false;
        };
        let mode = AccountingMode::Provider {
            scope,
            provider_id: request.provider_id,
            wire_api: request.wire_api,
            approved_endpoint: request.endpoint.clone(),
            approved_query: None,
            pricing: crate::config::PriceAuthority::Unavailable,
        };
        if owner.try_ensure_rollout_materialized().await.is_err() {
            return false;
        }
        let Some(runtime) = owner.state_db() else {
            return false;
        };
        let Ok(sampling) = Sampling::start_at_path(
            runtime,
            owner.thread_id,
            format!("{}:{}", request.label, Uuid::new_v4()),
            &mode,
            &request.path,
        )
        .await
        else {
            return false;
        };
        let route = crate::accounting::pinned_route(&request.endpoint, None, &request.path);
        let Ok(attempt) = sampling.admit_with_tier(&request.model, &route, None).await else {
            return false;
        };
        let evidence = ResponseEvidence::admitted(sampling, attempt);
        // The numbers are read from the provider's own response body with the
        // parser this client already uses for that dialect, so a reporter
        // cannot invent a usage shape. No usage records the attempt with its
        // tokens unknown, which is what a provider that stated nothing said.
        let Some(usage) = request.usage else {
            return true;
        };
        let body = serde_json::json!({ "usage": usage }).to_string();
        match request.wire_api {
            codex_model_provider_info::WireApi::Chat => {
                match codex_api::chat_body_usage(body.as_bytes()) {
                    Ok(Some(usage)) => {
                        codex_api::ChatUsageObserver::observe(evidence.as_ref(), 1, Ok(usage))
                            .await
                            .is_ok()
                    }
                    Ok(None) => true,
                    Err(_) => false,
                }
            }
            codex_model_provider_info::WireApi::Responses => {
                match codex_api::responses_body_usage(body.as_bytes()) {
                    Ok(Some(usage)) => {
                        codex_api::ResponsesUsageObserver::observe(evidence.as_ref(), 1, Ok(usage))
                            .await
                            .is_ok()
                    }
                    Ok(None) => true,
                    Err(_) => false,
                }
            }
            // The Anthropic dialect reports its usage inside a stream this
            // client never sees on this path. The call is recorded; its tokens
            // are unknown rather than invented.
            codex_model_provider_info::WireApi::Anthropic => true,
        }
    }
}

/// One model request a client outside this process already sent.
pub struct SentModelRequest {
    /// The account that is billed for it.
    pub provider_id: String,
    /// The base URL it went to, and the endpoint underneath it.
    pub endpoint: String,
    pub path: String,
    pub wire_api: codex_model_provider_info::WireApi,
    pub model: String,
    /// Names the kind of work; the recorded turn is `label:<uuid>`.
    pub label: String,
    /// The `usage` object the provider's response carried, verbatim.
    pub usage: Option<serde_json::Value>,
}

/// A transport that records the request it carries, when the host is
/// collecting, and passes it straight through when it is not.
pub struct Accounted<T> {
    inner: AccountingTransport<T>,
}

impl<T: HttpTransport> HttpTransport for Accounted<T> {
    async fn execute(&self, request: Request) -> Result<Response, TransportError> {
        self.inner.execute(request).await
    }

    async fn stream(&self, request: Request) -> Result<StreamResponse, TransportError> {
        self.inner.stream(request).await
    }
}

#[cfg(test)]
#[path = "accounting_extensions_tests.rs"]
mod tests;
