//! Nonsecret established-route provenance and sampling-only pump admission.
use super::FAILURE;
use super::Sampling;
use super::responses::DeferredResponsesSampling;
use codex_api::ApiError;
use codex_api::ResponsesUsageObserver;
use codex_api::ResponsesWebsocketAdmission;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::error::CodexErr;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Captured using already-resolved setup, including unaccounted prewarm.
/// Coupled to connection replacement; never contains credentials or fingerprints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Provenance {
    endpoint: Option<String>,
    api_key: bool,
}

impl Provenance {
    pub(crate) fn capture(
        provider: &ModelProviderInfo,
        auth: Option<&CodexAuth>,
        api: &codex_api::Provider,
        agent_identity: bool,
    ) -> Self {
        Self {
            endpoint: api
                .websocket_url_for_path("responses")
                .ok()
                .map(|url| url.to_string()),
            api_key: !agent_identity && super::responses::eligible(provider, auth),
        }
    }

    pub(crate) fn validate(
        &self,
        deferred: &DeferredResponsesSampling,
        cached: Option<&Self>,
    ) -> Result<bool, CodexErr> {
        let Some(expected) = deferred.websocket_endpoint()? else {
            return Ok(false);
        };
        // Unsupported current routes remain excluded until accounting has started.
        // resolve() performs the initialized-route check, without a new auth read.
        if !self.api_key {
            deferred.exclude()?;
            return Ok(false);
        }
        if self.endpoint.as_deref() != Some(expected.as_str())
            || cached.is_some_and(|old| old != self)
        {
            deferred.reject();
            return Err(CodexErr::Fatal(FAILURE.into()));
        }
        Ok(true)
    }
}

pub(super) fn endpoint(base: &str) -> Result<String, CodexErr> {
    let raw = format!("{}/responses", base.trim_end_matches('/'));
    let mut url = url::Url::parse(&raw).map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.scheme(), "http" | "https")
        || url.as_str() != raw
    {
        return Err(CodexErr::Fatal(FAILURE.into()));
    }
    let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
    url.set_scheme(scheme)
        .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
    Ok(url.to_string())
}

pub(crate) struct Admission {
    sampling: Arc<Sampling>,
    established: Provenance,
    expected: String,
    binding: Arc<std::sync::OnceLock<Arc<crate::memory_stage_one::StageOneMemoryBinding>>>,
}

impl Admission {
    pub(crate) fn new(
        sampling: Arc<Sampling>,
        established: Provenance,
        expected: String,
        binding: Arc<std::sync::OnceLock<Arc<crate::memory_stage_one::StageOneMemoryBinding>>>,
    ) -> Arc<dyn ResponsesWebsocketAdmission> {
        Arc::new(Self {
            sampling,
            established,
            expected,
            binding,
        })
    }
}

impl ResponsesWebsocketAdmission for Admission {
    fn check(&self) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            if let Some(binding) = self.binding.get()
                && binding.check_stream().is_err()
            {
                self.sampling.reject();
                return Err(ApiError::Stream(FAILURE.into()));
            }
            Ok(())
        })
    }

    fn admit(
        &self,
        model: String,
        tier: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Arc<dyn ResponsesUsageObserver>, ApiError>> + Send + '_>>
    {
        Box::pin(async move {
            let result = async {
                anyhow::ensure!(
                    self.established.api_key
                        && self.established.endpoint.as_deref() == Some(self.expected.as_str()),
                    FAILURE
                );
                self.sampling
                    .admit_with_tier(&model, &self.sampling.endpoint, tier.as_deref())
                    .await
            }
            .await;
            match result {
                Ok(attempt) => Ok(super::transport::ResponseEvidence::admitted(
                    self.sampling.clone(),
                    attempt,
                ) as Arc<dyn ResponsesUsageObserver>),
                Err(_) => {
                    self.sampling.reject();
                    Err(ApiError::Stream(FAILURE.into()))
                }
            }
        })
    }
}

#[cfg(test)]
#[path = "accounting_websocket_tests.rs"]
mod tests;
