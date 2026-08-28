use crate::BrowserError;
use crate::ContainerEngine;
use crate::EnginePreference;
use crate::QuarantinedArtifact;
use crate::broker::Broker;
use crate::broker::WorkerMessage;
use crate::broker::decode_body;
use crate::broker::read_message;
use crate::command::write_json;
use crate::container::OwnedContainer;
use codex_network_proxy::NetworkProxyState;
use codex_network_proxy::browser_policy::resolve_browser_destination;
use codex_security_policy::AuthorityEpoch;
use codex_security_policy::SecurityControlHealth;
use codex_security_policy::SecurityDegradationReason;
use codex_security_policy::SecurityLevel;
use std::time::Duration;
use tokio::io::BufReader;
use tokio_util::sync::CancellationToken;

/// Host-only source of live policy facts. Never implement this with worker/model
/// JSON or a cached inspector. S02 supplies the native session-policy adapter.
pub trait LiveBrowserAuthority: Sync {
    fn current(&self) -> Result<(SecurityLevel, AuthorityEpoch), BrowserError>;
}

/// Raw acquisition, not trusted/sanitized/model-visible content. PF-29/S02 must
/// wrap the bytes in the native source envelope and run ingress enforcement.
pub struct AcquiredPage {
    pub requested_url: String,
    pub final_url: Option<String>,
    pub untrusted_html: Option<Vec<u8>>,
    pub downloads: Vec<QuarantinedArtifact>,
}

pub struct BrowserRuntime {
    engine: ContainerEngine,
    image: String,
    level: SecurityLevel,
    epoch: AuthorityEpoch,
}

impl BrowserRuntime {
    /// Installation/OS elevation is deliberately not performed by this backend.
    /// S03 owns consent. A missing/stopped engine returns a safe setup requirement.
    pub async fn prepare(
        preference: EnginePreference,
        authority: &dyn LiveBrowserAuthority,
        cancel: &CancellationToken,
    ) -> Result<Self, BrowserError> {
        let (level, epoch) = authority.current()?;
        if level == SecurityLevel::Permissive {
            return Err(BrowserError::Inactive);
        }
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err(BrowserError::Cancelled),
            result = async {
                let engine = ContainerEngine::discover(preference).await?;
                let image = engine.prepare_image().await?;
                let runtime = Self { engine, image, level, epoch };
                runtime.check_current(authority)?;
                Ok(runtime)
            } => result,
        }
    }

    pub fn image_id(&self) -> &str {
        &self.image
    }
    pub fn engine_kind(&self) -> crate::EngineKind {
        self.engine.kind()
    }
    pub fn epoch(&self) -> AuthorityEpoch {
        self.epoch
    }

    fn check_current(&self, authority: &dyn LiveBrowserAuthority) -> Result<(), BrowserError> {
        if authority.current()? != (self.level, self.epoch) {
            return Err(BrowserError::StaleAuthority);
        }
        Ok(())
    }

    /// A fresh backend observation, not an authority cache. Every acquisition
    /// creates and probes another owned container before accepting any input.
    pub async fn health(
        &self,
        authority: &dyn LiveBrowserAuthority,
        cancel: &CancellationToken,
    ) -> SecurityControlHealth {
        if authority
            .current()
            .is_ok_and(|(level, _)| level == SecurityLevel::Permissive)
        {
            return SecurityControlHealth::Inactive {};
        }
        let mut container = OwnedContainer::new(self.engine.clone(), self.image.clone());
        let result = self
            .protected_operation(authority, cancel, async {
                container.create().await?;
                container.ensure_ready().await?;
                self.check_current(authority)
            })
            .await;
        let cleanup = container.close().await;
        health_for(result.and(cleanup))
    }

    pub async fn acquire(
        &self,
        url: &str,
        policy: &NetworkProxyState,
        authority: &dyn LiveBrowserAuthority,
        cancel: &CancellationToken,
    ) -> Result<AcquiredPage, BrowserError> {
        let mut container = OwnedContainer::new(self.engine.clone(), self.image.clone());
        let result = self
            .protected_operation(authority, cancel, async {
                let destination = resolve_browser_destination(url, policy)
                    .await
                    .map_err(|_| BrowserError::DestinationDenied)?;
                let url = destination.url().to_string();
                container.create().await?;
                container.ensure_ready().await?;
                self.check_current(authority)?;
                let mut child = container.acquire_worker().await?;
                let mut stdin = child
                    .stdin
                    .take()
                    .ok_or(BrowserError::InvalidWorkerResponse)?;
                let mut stdout = BufReader::new(
                    child
                        .stdout
                        .take()
                        .ok_or(BrowserError::InvalidWorkerResponse)?,
                );
                write_json(&mut stdin, &serde_json::json!({"url":url})).await?;
                let mut broker = Broker::new(policy, self.epoch);
                let document = loop {
                    self.check_current(authority)?;
                    match read_message(&mut stdout).await? {
                        WorkerMessage::Request { id, url } => {
                            self.check_current(authority)?;
                            let reply = broker
                                .request(id, &url, || self.check_current(authority))
                                .await?;
                            self.check_current(authority)?;
                            write_json(&mut stdin, &reply).await?;
                        }
                        WorkerMessage::Result { url, body } => {
                            // A worker cannot invent a successful source URL that
                            // the host never fetched. Content remains untrusted.
                            let mut final_url = url::Url::parse(&url)
                                .map_err(|_| BrowserError::InvalidWorkerResponse)?;
                            final_url.set_fragment(None);
                            if !broker
                                .visited
                                .iter()
                                .any(|visited| visited == final_url.as_str())
                            {
                                return Err(BrowserError::InvalidWorkerResponse);
                            }
                            break Some((final_url.to_string(), decode_body(&body)?));
                        }
                        WorkerMessage::Failed {} if !broker.artifacts.is_empty() => break None,
                        WorkerMessage::Failed {} => return Err(BrowserError::FetchFailed),
                    }
                };
                drop(stdin);
                let status = child
                    .wait()
                    .await
                    .map_err(|_| BrowserError::InvalidWorkerResponse)?;
                if document.is_some() && !status.success() {
                    return Err(BrowserError::FetchFailed);
                }
                self.check_current(authority)?;
                let (final_url, untrusted_html) = match document {
                    Some((url, bytes)) => (Some(url), Some(bytes)),
                    None => (None, None),
                };
                Ok(AcquiredPage {
                    requested_url: url,
                    final_url,
                    untrusted_html,
                    downloads: broker.artifacts,
                })
            })
            .await;
        let cleanup = container.close().await;
        let page = result?;
        cleanup?;
        self.check_current(authority)?;
        Ok(page)
    }

    async fn protected_operation<T>(
        &self,
        authority: &dyn LiveBrowserAuthority,
        cancel: &CancellationToken,
        operation: impl std::future::Future<Output = Result<T, BrowserError>>,
    ) -> Result<T, BrowserError> {
        self.check_current(authority)?;
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err(BrowserError::Cancelled),
            result = tokio::time::timeout(Duration::from_secs(75), operation) => result.map_err(|_| BrowserError::ResourceLimit)?,
        }
    }
}

pub fn health_for(result: Result<(), BrowserError>) -> SecurityControlHealth {
    let reason = match result {
        Ok(()) => return SecurityControlHealth::Enforcing {},
        Err(BrowserError::Inactive) => return SecurityControlHealth::Inactive {},
        Err(BrowserError::UnsupportedRuntime) => SecurityDegradationReason::UnsupportedPlatform,
        Err(
            BrowserError::StaleAuthority
            | BrowserError::DestinationDenied
            | BrowserError::PromotionDenied,
        ) => SecurityDegradationReason::PolicyMismatch,
        Err(BrowserError::ResourceLimit) => SecurityDegradationReason::ResourceLimit,
        Err(
            BrowserError::ContainerMismatch
            | BrowserError::HealthCheckFailed
            | BrowserError::InvalidWorkerResponse,
        ) => SecurityDegradationReason::HealthCheckFailed,
        Err(_) => SecurityDegradationReason::BackendUnavailable,
    };
    SecurityControlHealth::Degraded { reason }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
