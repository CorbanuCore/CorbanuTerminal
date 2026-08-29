use super::*;
use pretty_assertions::assert_eq;

struct Authority(SecurityLevel, AuthorityEpoch);
impl LiveBrowserAuthority for Authority {
    fn current(&self) -> Result<(SecurityLevel, AuthorityEpoch), BrowserError> {
        Ok((self.0, self.1))
    }
}

#[tokio::test]
async fn permissive_and_pre_cancelled_requests_never_discover_or_install_engines() {
    let epoch = AuthorityEpoch::new([1; 16], 0, 0).unwrap();
    let cancel = CancellationToken::new();
    assert!(matches!(
        BrowserRuntime::prepare(
            EnginePreference::Discover,
            &Authority(SecurityLevel::Permissive, epoch),
            &cancel
        )
        .await,
        Err(BrowserError::Inactive)
    ));
    cancel.cancel();
    assert!(matches!(
        BrowserRuntime::prepare(
            EnginePreference::Discover,
            &Authority(SecurityLevel::Moderate, epoch),
            &cancel
        )
        .await,
        Err(BrowserError::Cancelled)
    ));
}

#[test]
fn unavailable_failed_or_stale_backends_never_claim_enforcing() {
    assert_eq!(
        health_for(Err(BrowserError::Inactive)),
        SecurityControlHealth::Inactive {}
    );
    assert_eq!(
        health_for(Err(BrowserError::RuntimeMissing)),
        SecurityControlHealth::Degraded {
            reason: SecurityDegradationReason::BackendUnavailable
        }
    );
    assert_eq!(
        health_for(Err(BrowserError::StaleAuthority)),
        SecurityControlHealth::Degraded {
            reason: SecurityDegradationReason::PolicyMismatch
        }
    );
    assert_eq!(
        health_for(Err(BrowserError::HealthCheckFailed)),
        SecurityControlHealth::Degraded {
            reason: SecurityDegradationReason::HealthCheckFailed
        }
    );
    assert_eq!(health_for(Ok(())), SecurityControlHealth::Enforcing {});
}
