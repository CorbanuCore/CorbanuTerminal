use super::authoritative_state::AuthoritativeStateAnchor;
use super::authoritative_state::AuthoritativeStateAnchorError;
use super::authoritative_state::AuthoritativeStateAnchorStore;
use super::authoritative_state_anchor::NativeAuthoritativeStateAnchor;
use codex_config::AuthoritativeStateOwner;
use codex_protected_state::PolicyCheckpoint;
use codex_protected_state::PolicyRootStore;
use codex_protected_state::RootError;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Default)]
struct NativeContract(Mutex<Option<PolicyCheckpoint>>);
impl PolicyRootStore for NativeContract {
    fn load_policy(&self) -> Result<Option<PolicyCheckpoint>, RootError> {
        Ok(self.0.lock().map_err(|_| RootError::Unavailable)?.clone())
    }
    fn compare_policy(
        &self,
        expected: Option<&PolicyCheckpoint>,
        next: &PolicyCheckpoint,
    ) -> Result<(), RootError> {
        let mut value = self.0.lock().map_err(|_| RootError::Unavailable)?;
        if value.as_ref() != expected {
            return Err(RootError::Conflict);
        }
        *value = Some(next.clone());
        Ok(())
    }
}

#[test]
fn pf20_s03_core_adapter_preserves_exact_anchor_and_compare() {
    let storage = Arc::new(NativeContract::default());
    let adapter = NativeAuthoritativeStateAnchor(storage.clone());
    let anchor = AuthoritativeStateAnchor {
        schema_version: 1,
        revision: 1,
        owner: AuthoritativeStateOwner::new("a".repeat(64), "controller", 1).unwrap(),
        state_sha256: "b".repeat(64),
        commit_sha256: "c".repeat(64),
    };
    adapter.compare_and_store_anchor(None, &anchor).unwrap();
    assert_eq!(adapter.load_anchor().unwrap(), Some(anchor.clone()));
    assert_eq!(
        storage.load_policy().unwrap(),
        Some(PolicyCheckpoint::from(&anchor))
    );
    assert_eq!(
        adapter.compare_and_store_anchor(None, &anchor),
        Err(AuthoritativeStateAnchorError::Conflict)
    );
}

#[test]
fn pf20_s03_core_policy_anchor_first_recovery_survives_data_loss() {
    use super::authoritative_state::AuthoritativeStateStore;
    use super::authoritative_state::AuthoritativeStateStoreError;
    use super::authoritative_state::TrustedControllerAuthorization;
    use codex_config::AuthoritativeSecurityState;
    use codex_secret_broker::platform_contract::*;
    use codex_security_policy::SecurityLevel;
    let capabilities: Vec<_> = REQUIRED_CAPABILITIES
        .iter()
        .map(|capability| CapabilityResult {
            capability: *capability,
            status: CapabilityStatus::Supported,
            observation: Observation::Verified,
            mechanism: "synthetic-contract-only",
            detail_code: "fixture",
        })
        .collect();
    let target = "a".repeat(64);
    let probe = "b".repeat(64);
    let report = PlatformReport {
        contract_version: CONTRACT_VERSION,
        fixture_protocol: FIXTURE_PROTOCOL_VERSION,
        probe_sha256: &probe,
        target_id: &target,
        measured_at_unix_seconds: 1,
        expires_at_unix_seconds: 100,
        capabilities: &capabilities,
        protected_mode_eligible: true,
    };
    // Synthetic report proves only the adapter's existing Core ordering. It is
    // not a native readiness witness and is never used in product construction.
    let authorization = TrustedControllerAuthorization::from_platform_report(
        &report,
        &target,
        &probe,
        2,
        "controller",
        1,
    )
    .unwrap();
    let owner = AuthoritativeStateOwner::new(&target, "controller", 1).unwrap();
    let state =
        AuthoritativeSecurityState::new(1, owner, SecurityLevel::Moderate, 1, 1, 0, false).unwrap();
    let root = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let storage = Arc::new(NativeContract::default());
    let store = AuthoritativeStateStore::new(
        root.path(),
        Arc::new(NativeAuthoritativeStateAnchor(storage)),
    );
    assert_eq!(
        store
            .compare_and_activate(0, &state, &authorization)
            .unwrap(),
        state
    );
    for entry in std::fs::read_dir(root.path()).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::InterruptedWrite { revision: 1 })
    ));
    assert_eq!(
        store
            .compare_and_activate(0, &state, &authorization)
            .unwrap(),
        state
    );
}
