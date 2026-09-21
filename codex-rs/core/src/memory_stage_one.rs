//! Thread-owned, denial-only dispatch for unscreened stage-one memory input.

#[cfg(test)]
#[path = "memory_stage_one_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "accounting_tests.rs"]
mod accounting_tests;

use crate::client::ModelClient;
use crate::client_common::Prompt;
use crate::client_common::ResponseEvent;
use crate::responses_metadata::CodexResponsesMetadata;
use crate::session::SessionLoopTermination;
use crate::session::memory_stage_one::MemoryStageOneConfiguration;
use crate::session::session::Session;
use codex_features::Feature;
use codex_http_client::HttpTransport;
use codex_http_client::Request;
use codex_http_client::ReqwestTransport;
use codex_http_client::Response;
use codex_http_client::StreamResponse;
use codex_http_client::TransportError;
use codex_login::auth::AgentIdentityAuthPolicy;
use codex_model_provider_info::ModelProviderInfo;
use codex_otel::SessionTelemetry;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::error::CodexErr;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::TokenUsage;
use codex_rollout_trace::InferenceTraceContext;
use codex_security_policy::SecurityLevel;
use futures::FutureExt;
use futures::StreamExt;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use thiserror::Error;

/// Stable reasons that never contain rollout text or provider credentials.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum StageOneMemoryDenial {
    #[error("protected stage-one memory input is unavailable")]
    ProtectedInputUnavailable,
    #[error("stage-one memory policy is unavailable")]
    PolicyUnavailable,
    #[error("stage-one memory owner does not match")]
    OwnerMismatch,
    #[error("stage-one memory owner terminated")]
    OwnerTerminated,
    #[error("stage-one memory provider changed")]
    ProviderChanged,
    #[error("stage-one memory is stopped by the security kill switch")]
    KillSwitchActive,
    #[error("stage-one memory was cancelled")]
    Cancelled,
}

#[derive(Debug, Error)]
pub enum StageOneMemoryError {
    #[error(transparent)]
    Denied(#[from] StageOneMemoryDenial),
    #[error(transparent)]
    Request(#[from] CodexErr),
}

/// Request data only: none of these fields convey source admission or authority.
pub struct StageOneMemoryRequest<'a> {
    pub prompt: &'a Prompt,
    pub model_info: &'a ModelInfo,
    pub session_telemetry: &'a SessionTelemetry,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub reasoning_summary: ReasoningSummary,
    pub service_tier: Option<String>,
    pub responses_metadata: &'a CodexResponsesMetadata,
}

pub struct StageOneMemoryOutput {
    pub text: String,
    pub token_usage: Option<TokenUsage>,
}

/// Opaque request owner. It exposes neither an inner client nor a policy setter.
pub struct StageOneMemoryClient {
    client: ModelClient,
    binding: Arc<StageOneMemoryBinding>,
}

pub(crate) struct StageOneMemoryBinding {
    owner: Weak<Session>,
    termination: SessionLoopTermination,
    owner_id: ThreadId,
    provider: ModelProviderInfo,
    floor: SecurityLevel,
    runtime_nonce: [u8; 16],
    session_id: String,
    denial: Mutex<Option<StageOneMemoryDenial>>,
}

impl std::fmt::Debug for StageOneMemoryBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StageOneMemoryBinding")
    }
}

impl StageOneMemoryBinding {
    /// The owner's current stage-one configuration, denied on any drift.
    async fn owner_configuration(
        &self,
        owner: &Arc<Session>,
    ) -> Result<MemoryStageOneConfiguration, StageOneMemoryDenial> {
        owner
            .memory_stage_one_configuration(self.owner_id, &self.provider)
            .await
    }

    async fn evaluate(&self) -> Result<(), StageOneMemoryDenial> {
        if self.termination.clone().now_or_never().is_some() {
            return Err(StageOneMemoryDenial::OwnerTerminated);
        }
        let owner = self
            .owner
            .upgrade()
            .ok_or(StageOneMemoryDenial::OwnerTerminated)?;
        let policy = owner
            .memory_stage_one_configuration(self.owner_id, &self.provider)
            .await?;
        if policy.runtime_nonce != self.runtime_nonce || policy.session_id != self.session_id {
            return Err(StageOneMemoryDenial::OwnerMismatch);
        }
        if policy.kill_switch_active {
            return Err(StageOneMemoryDenial::KillSwitchActive);
        }
        if self
            .floor
            .max(policy.config.security_level)
            .max(policy.level)
            != SecurityLevel::Permissive
        {
            return Err(StageOneMemoryDenial::ProtectedInputUnavailable);
        }
        Ok(())
    }

    pub(crate) async fn check(&self) -> Result<(), StageOneMemoryDenial> {
        let previous = *self
            .denial
            .lock()
            .map_err(|_| StageOneMemoryDenial::PolicyUnavailable)?;
        if let Some(reason) = previous {
            return Err(reason);
        }
        let result = self.evaluate().await;
        if let Err(reason) = result {
            *self
                .denial
                .lock()
                .map_err(|_| StageOneMemoryDenial::PolicyUnavailable)? = Some(reason);
        }
        result
    }

    /// Stream-time validation uses the published runtime provider and live policy view.
    /// The configured security floor was captured at construction and is session-static;
    /// runtime config refresh does not change it. No session-state lock or Config copy.
    pub(crate) fn check_stream(&self) -> Result<(), StageOneMemoryDenial> {
        let mut denial = self
            .denial
            .lock()
            .map_err(|_| StageOneMemoryDenial::PolicyUnavailable)?;
        if let Some(reason) = *denial {
            return Err(reason);
        }
        let result = (|| {
            if self.termination.clone().now_or_never().is_some() {
                return Err(StageOneMemoryDenial::OwnerTerminated);
            }
            let owner = self
                .owner
                .upgrade()
                .ok_or(StageOneMemoryDenial::OwnerTerminated)?;
            if owner.thread_id() != self.owner_id {
                return Err(StageOneMemoryDenial::OwnerMismatch);
            }
            if owner.services.model_client().provider_info() != &self.provider {
                return Err(StageOneMemoryDenial::ProviderChanged);
            }
            let policy = owner
                .services
                .agent_control
                .effective_security_policy()
                .snapshot_for_agent(self.owner_id)
                .map_err(|_| StageOneMemoryDenial::PolicyUnavailable)?;
            if policy.runtime_nonce != self.runtime_nonce
                || policy.session_id.as_str() != self.session_id
            {
                return Err(StageOneMemoryDenial::OwnerMismatch);
            }
            if policy.kill_switch_active {
                return Err(StageOneMemoryDenial::KillSwitchActive);
            }
            if self.floor.max(policy.level) != SecurityLevel::Permissive {
                return Err(StageOneMemoryDenial::ProtectedInputUnavailable);
            }
            Ok(())
        })();
        if let Err(reason) = result {
            *denial = Some(reason);
        }
        result
    }

    fn request_error(&self, error: CodexErr) -> StageOneMemoryError {
        match self.denial.lock() {
            Ok(reason) => reason.map_or(
                StageOneMemoryError::Request(error),
                StageOneMemoryError::Denied,
            ),
            Err(_) => StageOneMemoryDenial::PolicyUnavailable.into(),
        }
    }
}

impl StageOneMemoryClient {
    #[cfg(debug_assertions)]
    pub(crate) fn binding_for_fixture(
        &self,
        owner: ThreadId,
    ) -> Result<Arc<StageOneMemoryBinding>, StageOneMemoryDenial> {
        if self.binding.owner_id != owner {
            return Err(StageOneMemoryDenial::OwnerMismatch);
        }
        Ok(Arc::clone(&self.binding))
    }

    pub(crate) async fn new(
        owner: Weak<Session>,
        termination: SessionLoopTermination,
        expected_owner: ThreadId,
        expected_provider: &ModelProviderInfo,
    ) -> Result<Self, StageOneMemoryError> {
        let session = owner
            .upgrade()
            .ok_or(StageOneMemoryDenial::OwnerTerminated)?;
        let policy = session
            .memory_stage_one_configuration(expected_owner, expected_provider)
            .await?;
        let config = policy.config;
        let snapshot = policy.thread;
        let binding = Arc::new(StageOneMemoryBinding {
            owner,
            termination,
            owner_id: expected_owner,
            provider: expected_provider.clone(),
            floor: config.security_level,
            runtime_nonce: policy.runtime_nonce,
            session_id: policy.session_id,
            denial: Mutex::new(None),
        });
        binding.check().await?;
        let client = ModelClient::new(
            Some(Arc::clone(&session.services.auth_manager)),
            AgentIdentityAuthPolicy::JwtOnly,
            expected_owner,
            config.model_provider.clone(),
            snapshot.session_source,
            snapshot.originator,
            config.model_verbosity,
            config.features.enabled(Feature::EnableRequestCompression),
            config.features.enabled(Feature::RuntimeMetrics),
            /*beta_features_header*/ None,
            /*concurrent_reasoning_summaries_enabled*/ false,
            /*attestation_provider*/ None,
            config.http_client_factory(),
        );
        client.with_stage_one_memory_binding(Arc::clone(&binding))?;
        Ok(Self { client, binding })
    }

    pub async fn check_completion(&self) -> Result<(), StageOneMemoryError> {
        self.binding.check().await.map_err(Into::into)
    }

    /// Collection for one extraction, read from the owner's own configuration.
    ///
    /// The owner is held weakly, and the pipeline runs after a turn: a session
    /// that has gone away records nothing rather than failing the extraction.
    async fn attach_accounting(
        &self,
        session: &crate::client::ModelClientSession,
    ) -> anyhow::Result<Option<crate::accounting::TurnScopes>> {
        let Some(owner) = self.binding.owner.upgrade() else {
            return Ok(None);
        };
        let policy = self
            .binding
            .owner_configuration(&owner)
            .await
            .map_err(|denial| anyhow::anyhow!(denial.to_string()))?;
        // Collect only when this request really goes to the route the owner's
        // configuration approved. Accounting must never be the reason a
        // stage-one request fails, and a request bound elsewhere would fail the
        // route check at admission instead of simply going unrecorded.
        if self.client.provider_info() != &policy.config.model_provider {
            return Ok(None);
        }
        let auth = owner.services.auth_manager.auth().await;
        let auth_mode = auth.as_ref().map(codex_login::CodexAuth::auth_mode);
        let endpoint = policy
            .config
            .model_provider
            .to_api_provider(auth_mode)
            .map(|api| api.base_url)
            .unwrap_or_default();
        Ok(Some(
            crate::accounting::attach_scopes(
                &owner,
                &policy.config.accounting,
                &policy.config.model_provider_id,
                &policy.config.model_provider,
                auth_mode,
                &endpoint,
                session,
                crate::accounting::memory_turn_label(),
            )
            .await?,
        ))
    }

    pub async fn extract(
        &mut self,
        request: StageOneMemoryRequest<'_>,
    ) -> Result<StageOneMemoryOutput, StageOneMemoryError> {
        self.check_completion().await?;
        let mut session = self.client.new_session();
        // Extraction is a model request the operator paid for, on a session of
        // its own, so it is collected like any other - under its own `memory:`
        // turn. Best effort, exactly as compaction and prewarm are: an
        // extraction that cannot be recorded still runs, because accounting is
        // an observer here and not a gate on the memory pipeline.
        let _accounting = match self.attach_accounting(&session).await {
            Ok(scopes) => scopes,
            Err(error) => {
                tracing::warn!(%error, "accounting: stage-one memory proceeding unrecorded");
                None
            }
        };
        let trace = InferenceTraceContext::disabled();
        let mut stream = tokio::select! {
            _ = self.binding.termination.clone() => return Err(StageOneMemoryDenial::OwnerTerminated.into()),
            result = session.stream(request.prompt, request.model_info, request.session_telemetry,
                request.reasoning_effort, request.reasoning_summary, request.service_tier,
                request.responses_metadata, &trace) => result.map_err(|error| self.binding.request_error(error))?,
        };
        let mut text = String::new();
        loop {
            self.check_completion().await?;
            let event = tokio::select! {
                _ = self.binding.termination.clone() => return Err(StageOneMemoryDenial::OwnerTerminated.into()),
                event = stream.next() => event,
            };
            match event
                .transpose()
                .map_err(|error| self.binding.request_error(error))?
            {
                Some(ResponseEvent::OutputTextDelta { delta, .. }) => text.push_str(&delta),
                Some(ResponseEvent::OutputItemDone(
                    codex_protocol::models::ResponseItem::Message { content, .. },
                )) if text.is_empty() => {
                    if let Some(output) = crate::content_items_to_text(&content) {
                        text.push_str(&output);
                    }
                }
                Some(ResponseEvent::Completed { token_usage, .. }) => {
                    self.check_completion().await?;
                    return Ok(StageOneMemoryOutput { text, token_usage });
                }
                None => {
                    return Err(CodexErr::Stream(
                        "stage-one memory stream ended before completion".into(),
                    )
                    .into());
                }
                _ => {}
            }
        }
    }
}

/// Checks below endpoint retries, after async auth and before transport dispatch.
#[derive(Clone, Debug)]
pub(crate) struct StageOneGuardedTransport<T = ReqwestTransport> {
    inner: T,
    binding: Option<Arc<StageOneMemoryBinding>>,
}

impl<T> StageOneGuardedTransport<T> {
    pub(crate) fn new(inner: T, binding: Option<Arc<StageOneMemoryBinding>>) -> Self {
        Self { inner, binding }
    }

    pub(crate) fn map_inner<U>(self, map: impl FnOnce(T) -> U) -> StageOneGuardedTransport<U> {
        StageOneGuardedTransport {
            inner: map(self.inner),
            binding: self.binding,
        }
    }

    async fn check(&self) -> Result<(), TransportError> {
        if let Some(binding) = &self.binding {
            binding
                .check()
                .await
                .map_err(|reason| TransportError::Build(reason.to_string()))?;
        }
        Ok(())
    }
}

impl<T: HttpTransport> HttpTransport for StageOneGuardedTransport<T> {
    async fn execute(&self, request: Request) -> Result<Response, TransportError> {
        self.check().await?;
        self.inner.execute(request).await
    }

    async fn stream(&self, request: Request) -> Result<StreamResponse, TransportError> {
        self.check().await?;
        self.inner.stream(request).await
    }
}
