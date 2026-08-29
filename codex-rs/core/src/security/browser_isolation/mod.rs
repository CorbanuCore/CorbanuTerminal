//! PF-30 native policy/health adapter. No model tool, facade, or inspector event
//! routing is activated here; S02 owns that join and PF-29 owns source handling.
#![allow(dead_code)]

use codex_browser_isolation::BrowserError;
use codex_browser_isolation::BrowserRuntime;
use codex_browser_isolation::LiveBrowserAuthority;
use codex_protocol::ThreadId;
use codex_protocol::security::SecurityInspectorEvent;
use codex_security_policy::AuthorityEpoch;
use codex_security_policy::SecurityControlHealth;
use codex_security_policy::SecurityControlHealthSnapshot;
use codex_security_policy::SecurityDegradationReason;
use codex_security_policy::SecurityInspectorSnapshot;
use codex_security_policy::SecurityLevel;
use tokio_util::sync::CancellationToken;

use super::effective_policy::EffectivePolicyView;

/// Binds runtime operations to native live policy, including inherited floors,
/// revocation and fresh resume incarnation. It grants no policy mutation access.
pub(crate) struct BrowserAuthority {
    view: EffectivePolicyView,
    agent: ThreadId,
}

impl BrowserAuthority {
    pub(crate) fn new(view: EffectivePolicyView, agent: ThreadId) -> Self {
        Self { view, agent }
    }

    pub(crate) async fn observe_backend(
        &self,
        runtime: &BrowserRuntime,
        cancel: &CancellationToken,
    ) -> Result<SecurityInspectorEvent, BrowserError> {
        let health = runtime.health(self, cancel).await;
        let snapshot = self
            .view
            .snapshot_for_agent(self.agent)
            .map_err(|_| BrowserError::StaleAuthority)?;
        let epoch = snapshot
            .authority_epoch()
            .map_err(|_| BrowserError::StaleAuthority)?;
        let health = if snapshot.level == SecurityLevel::Permissive {
            SecurityControlHealth::Inactive {}
        } else if epoch != runtime.epoch() || snapshot.kill_switch_active {
            SecurityControlHealth::Degraded {
                reason: SecurityDegradationReason::PolicyMismatch,
            }
        } else {
            health
        };
        inspector(snapshot.requested_level, snapshot.level, epoch, health)
    }
}

impl LiveBrowserAuthority for BrowserAuthority {
    fn current(&self) -> Result<(SecurityLevel, AuthorityEpoch), BrowserError> {
        let snapshot = self
            .view
            .snapshot_for_agent(self.agent)
            .map_err(|_| BrowserError::StaleAuthority)?;
        if snapshot.kill_switch_active {
            return Err(BrowserError::DestinationDenied);
        }
        let epoch = snapshot
            .authority_epoch()
            .map_err(|_| BrowserError::StaleAuthority)?;
        Ok((snapshot.level, epoch))
    }
}

fn inspector(
    requested: SecurityLevel,
    effective: SecurityLevel,
    epoch: AuthorityEpoch,
    health: SecurityControlHealth,
) -> Result<SecurityInspectorEvent, BrowserError> {
    Ok(SecurityInspectorEvent {
        epoch,
        snapshot: SecurityInspectorSnapshot::new(
            requested,
            effective,
            SecurityControlHealthSnapshot {
                browser_isolation: health,
                // Browser readiness cannot establish content/action/confidentiality health.
                ..SecurityControlHealthSnapshot::default()
            },
        )
        .map_err(|_| BrowserError::StaleAuthority)?,
    })
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod tests;
