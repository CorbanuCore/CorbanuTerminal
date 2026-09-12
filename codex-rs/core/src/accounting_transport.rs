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
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use uuid::Uuid;

pub(crate) struct ResponseEvidence {
    sampling: Arc<Sampling>,
    attempt: OnceLock<Attempt>,
    source: Uuid,
}

impl ResponseEvidence {
    pub(crate) fn new(sampling: Arc<Sampling>) -> Arc<Self> {
        Arc::new(Self {
            sampling,
            attempt: OnceLock::new(),
            source: Uuid::new_v4(),
        })
    }
}

impl AnthropicUsageObserver for ResponseEvidence {
    fn observe(
        &self,
        position: i64,
        usage: Result<AnthropicUsagePatch, InvalidAnthropicUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
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

pub(crate) struct AccountingTransport<T> {
    inner: T,
    evidence: Option<Arc<ResponseEvidence>>,
    model: String,
}

impl<T> AccountingTransport<T> {
    pub(crate) fn new(inner: T, evidence: Option<Arc<ResponseEvidence>>, model: String) -> Self {
        Self {
            inner,
            evidence,
            model,
        }
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
        let attempt = evidence
            .sampling
            .admit(&self.model, &request.url)
            .await
            .map_err(|_| {
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
