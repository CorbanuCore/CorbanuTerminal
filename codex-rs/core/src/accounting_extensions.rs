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
    accounting: AccountingMode,
    provider_id: String,
}

impl std::fmt::Debug for ExtensionAccounting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExtensionAccounting")
    }
}

impl ExtensionAccounting {
    pub(crate) fn new(
        owner: Weak<Session>,
        accounting: AccountingMode,
        provider_id: String,
    ) -> Self {
        Self {
            owner,
            accounting,
            provider_id,
        }
    }

    /// Wrap `transport` so the request it carries is recorded.
    ///
    /// `path` is the endpoint under the provider's approved route - a request
    /// to anywhere else is refused at admission rather than attributed here.
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
        model: &str,
        path: &str,
        label: &str,
    ) -> Accounted<T> {
        match self.evidence(provider, path, label).await {
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

    /// The provider is the caller's own, live one, not a snapshot taken when
    /// this handle was made. `turn_mode` then binds the route the request
    /// actually takes, exactly as it does for a turn, so a session whose
    /// provider changed records against the new route rather than against a
    /// stale one - or, if that route collects nothing, records nothing.
    async fn evidence(
        &self,
        provider: &ModelProviderInfo,
        path: &str,
        label: &str,
    ) -> Option<Arc<ResponseEvidence>> {
        let owner = self.owner.upgrade()?;
        let auth = owner.services.auth_manager.auth().await;
        let auth_mode = auth.as_ref().map(codex_login::CodexAuth::auth_mode);
        let api = provider.to_api_provider(auth_mode).ok()?;
        let mode = if matches!(self.accounting, AccountingMode::Provider { .. }) {
            crate::accounting::turn_mode(
                &self.accounting,
                &self.provider_id,
                provider,
                auth_mode,
                &api.base_url,
            )
        } else {
            self.accounting.clone()
        };
        if !crate::accounting::collects(&mode, &self.provider_id, provider, provider.wire_api) {
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
