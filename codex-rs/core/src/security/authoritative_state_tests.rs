#[cfg(unix)]
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

#[cfg(unix)]
use codex_config::AuthoritativeSecurityState;
use codex_config::AuthoritativeStateOwner;
use codex_secret_broker::platform_contract::CONTRACT_VERSION;
use codex_secret_broker::platform_contract::Capability;
use codex_secret_broker::platform_contract::CapabilityResult;
use codex_secret_broker::platform_contract::CapabilityStatus;
use codex_secret_broker::platform_contract::FIXTURE_PROTOCOL_VERSION;
use codex_secret_broker::platform_contract::Observation;
use codex_secret_broker::platform_contract::PlatformReport;
#[cfg(unix)]
use codex_security_policy::SecurityLevel;
#[cfg(unix)]
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::authoritative_state::AuthoritativeStateAnchor;
use super::authoritative_state::AuthoritativeStateAnchorError;
use super::authoritative_state::AuthoritativeStateAnchorStore;
#[cfg(unix)]
use super::authoritative_state::AuthoritativeStateLoad;
use super::authoritative_state::AuthoritativeStateStore;
use super::authoritative_state::AuthoritativeStateStoreError;
use super::authoritative_state::TrustedControllerAuthorization;

const TARGET_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PROBE_ID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const NOW: u64 = 1_000;

const CAPABILITIES: [CapabilityResult<'static>; 10] = [
    supported(Capability::ProcessIdentity),
    supported(Capability::FilesystemBoundary),
    supported(Capability::ConfigBoundary),
    supported(Capability::InheritedHandles),
    supported(Capability::IpcPeerIdentity),
    supported(Capability::NetworkBoundary),
    supported(Capability::ProcessMemoryDebug),
    supported(Capability::SigningEntitlements),
    supported(Capability::ElevationBoundary),
    supported(Capability::ProtectedStore),
];

const fn supported(capability: Capability) -> CapabilityResult<'static> {
    CapabilityResult {
        capability,
        status: CapabilityStatus::Supported,
        observation: Observation::Verified,
        mechanism: "test-controller-boundary",
        detail_code: "verified",
    }
}

fn platform_report() -> PlatformReport<'static> {
    PlatformReport {
        contract_version: CONTRACT_VERSION,
        fixture_protocol: FIXTURE_PROTOCOL_VERSION,
        probe_sha256: PROBE_ID,
        target_id: TARGET_ID,
        measured_at_unix_seconds: 900,
        expires_at_unix_seconds: 1_100,
        capabilities: &CAPABILITIES,
        protected_mode_eligible: true,
    }
}

#[cfg(unix)]
fn authorization(owner_id: &str, owner_generation: u64) -> TrustedControllerAuthorization {
    TrustedControllerAuthorization::from_platform_report(
        &platform_report(),
        TARGET_ID,
        PROBE_ID,
        NOW,
        owner_id,
        owner_generation,
    )
    .expect("valid platform authorization")
}

#[cfg(unix)]
fn state(
    revision: u64,
    owner_id: &str,
    owner_generation: u64,
    level: SecurityLevel,
    generations: AuthorityGenerations,
    kill_switch_active: bool,
) -> AuthoritativeSecurityState {
    AuthoritativeSecurityState::new(
        revision,
        AuthoritativeStateOwner::new(TARGET_ID, owner_id, owner_generation).unwrap(),
        level,
        generations.grant,
        generations.revocation,
        generations.kill_switch,
        kill_switch_active,
    )
    .expect("valid state")
}

#[cfg(unix)]
#[derive(Clone, Copy)]
struct AuthorityGenerations {
    grant: u64,
    revocation: u64,
    kill_switch: u64,
}

#[cfg(unix)]
const fn generations(grant: u64, revocation: u64, kill_switch: u64) -> AuthorityGenerations {
    AuthorityGenerations {
        grant,
        revocation,
        kill_switch,
    }
}

fn store() -> (TempDir, AuthoritativeStateStore) {
    let (root, store, _) = store_with_anchor();
    (root, store)
}

fn store_with_anchor() -> (TempDir, AuthoritativeStateStore, Arc<MemoryAnchor>) {
    let root = tempfile::tempdir().expect("protected state root");
    set_private_directory(root.path());
    let anchor = Arc::new(MemoryAnchor::default());
    let store = AuthoritativeStateStore::new(root.path(), anchor.clone());
    (root, store, anchor)
}

#[derive(Debug, Default)]
struct MemoryAnchor {
    value: Mutex<Option<AuthoritativeStateAnchor>>,
}

impl AuthoritativeStateAnchorStore for MemoryAnchor {
    fn load_anchor(
        &self,
    ) -> Result<Option<AuthoritativeStateAnchor>, AuthoritativeStateAnchorError> {
        Ok(self.value.lock().unwrap().clone())
    }

    fn compare_and_store_anchor(
        &self,
        expected: Option<&AuthoritativeStateAnchor>,
        next: &AuthoritativeStateAnchor,
    ) -> Result<(), AuthoritativeStateAnchorError> {
        let mut value = self.value.lock().unwrap();
        if value.as_ref() != expected {
            return Err(AuthoritativeStateAnchorError::Conflict);
        }
        *value = Some(next.clone());
        Ok(())
    }
}

#[cfg(unix)]
fn set_private_directory(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(not(unix))]
fn set_private_directory(_path: &std::path::Path) {}

#[cfg(unix)]
#[test]
fn empty_protected_root_is_the_only_legacy_first_install() {
    let (_root, store) = store();
    assert_eq!(
        store.load().unwrap(),
        AuthoritativeStateLoad::LegacyFirstInstall
    );

    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    store
        .compare_and_activate(0, &initial, &authorization("credential-owner-a", 1))
        .unwrap();
    assert_eq!(
        store.load().unwrap(),
        AuthoritativeStateLoad::Active(initial)
    );
}

#[test]
fn model_supplied_identity_cannot_replace_platform_authorization() {
    let err = TrustedControllerAuthorization::from_platform_report(
        &platform_report(),
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        PROBE_ID,
        NOW,
        "forged-owner",
        1,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        AuthoritativeStateStoreError::PlatformAuthorization(_)
    ));
}

#[test]
fn corrupt_external_anchor_fails_closed_before_record_classification() {
    let (_root, store, anchor) = store_with_anchor();
    *anchor.value.lock().unwrap() = Some(AuthoritativeStateAnchor {
        schema_version: 0,
        revision: 1,
        owner: AuthoritativeStateOwner::new(TARGET_ID, "credential-owner-a", 1).unwrap(),
        state_sha256: "0".repeat(64),
        commit_sha256: "0".repeat(64),
    });
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::Anchor(
            AuthoritativeStateAnchorError::Invalid
        ))
    ));
}

#[cfg(not(unix))]
#[test]
fn protected_persistence_is_an_explicit_platform_blocker() {
    let (_root, store) = store();
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::UnsupportedPlatform)
    ));
}

#[cfg(unix)]
#[test]
fn compare_and_activate_rejects_stale_revision_and_wrong_owner() {
    let (_root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    store
        .compare_and_activate(0, &initial, &authorization("credential-owner-a", 1))
        .unwrap();
    let successor = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    assert!(matches!(
        store.compare_and_activate(0, &successor, &authorization("credential-owner-a", 1)),
        Err(AuthoritativeStateStoreError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert!(matches!(
        store.compare_and_activate(1, &successor, &authorization("other-owner", 1)),
        Err(AuthoritativeStateStoreError::UnauthorizedOwner)
    ));
}

#[cfg(unix)]
#[test]
fn unanchored_pending_state_is_discarded_only_by_the_authorized_owner() {
    let (root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    let owner_authorization = authorization("credential-owner-a", 1);
    store
        .compare_and_activate(0, &initial, &owner_authorization)
        .unwrap();
    let next = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    let path = root.path().join("state-00000000000000000002.json");
    let mut bytes = serde_json::to_vec(&next).unwrap();
    bytes.push(b'\n');
    write_private(&path, &bytes);

    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::UnanchoredRecords { revision: 2 })
    ));
    let mut different = next.clone();
    different.grant_generation = 2;
    assert!(matches!(
        store.compare_and_activate(1, &different, &owner_authorization),
        Err(AuthoritativeStateStoreError::UnanchoredRecords { revision: 2 })
    ));
    assert!(matches!(
        store.discard_unanchored_suffix(2, &authorization("other-owner", 1)),
        Err(AuthoritativeStateStoreError::UnauthorizedOwner)
    ));
    store
        .discard_unanchored_suffix(2, &owner_authorization)
        .unwrap();
    assert_eq!(
        store
            .compare_and_activate(1, &next, &owner_authorization)
            .unwrap(),
        next
    );
}

#[cfg(unix)]
#[test]
fn mismatched_anchored_pending_is_discardable_and_exact_state_resumes() {
    let (root, store, _anchor) = store_with_anchor();
    let owner_authorization = authorization("credential-owner-a", 1);
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    let anchored_next = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    store
        .compare_and_activate(0, &initial, &owner_authorization)
        .unwrap();
    store
        .compare_and_activate(1, &anchored_next, &owner_authorization)
        .unwrap();
    for prefix in ["state", "intent", "commit"] {
        fs::remove_file(root.path().join(format!("{prefix}-{:020}.json", 2))).unwrap();
    }
    let attacker_state = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Permissive,
        generations(0, 0, 0),
        false,
    );
    let mut bytes = serde_json::to_vec(&attacker_state).unwrap();
    bytes.push(b'\n');
    write_private(&root.path().join("state-00000000000000000002.json"), &bytes);

    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::AnchorMismatch { revision: 2 })
    ));
    assert!(matches!(
        store.discard_unanchored_suffix(2, &authorization("other-owner", 1)),
        Err(AuthoritativeStateStoreError::UnauthorizedOwner)
    ));
    store
        .discard_unanchored_suffix(2, &owner_authorization)
        .unwrap();
    assert_eq!(
        store
            .compare_and_activate(1, &anchored_next, &owner_authorization)
            .unwrap(),
        anchored_next
    );
}

#[cfg(unix)]
#[test]
fn committed_records_ahead_of_anchor_are_discarded_without_touching_anchor_history() {
    let (root, store, anchor) = store_with_anchor();
    let owner_authorization = authorization("credential-owner-a", 1);
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    let next = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    store
        .compare_and_activate(0, &initial, &owner_authorization)
        .unwrap();
    let anchored_first = anchor.value.lock().unwrap().clone();
    store
        .compare_and_activate(1, &next, &owner_authorization)
        .unwrap();
    *anchor.value.lock().unwrap() = anchored_first;

    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::UnanchoredRecords { revision: 2 })
    ));
    assert!(matches!(
        store.discard_unanchored_suffix(2, &authorization("other-owner", 1)),
        Err(AuthoritativeStateStoreError::UnauthorizedOwner)
    ));
    store
        .discard_unanchored_suffix(2, &owner_authorization)
        .unwrap();
    for prefix in ["state", "intent", "commit"] {
        assert!(
            root.path()
                .join(format!("{prefix}-{:020}.json", 1))
                .exists()
        );
        assert!(
            !root
                .path()
                .join(format!("{prefix}-{:020}.json", 2))
                .exists()
        );
    }
    assert_eq!(
        store.load().unwrap(),
        AuthoritativeStateLoad::Active(initial)
    );
}

#[cfg(unix)]
#[test]
fn missing_commit_resumes_without_activating_the_pending_state() {
    let (root, store) = store();
    let authorization = authorization("credential-owner-a", 1);
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    let next = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    store
        .compare_and_activate(0, &initial, &authorization)
        .unwrap();
    store
        .compare_and_activate(1, &next, &authorization)
        .unwrap();
    fs::remove_file(root.path().join("commit-00000000000000000002.json")).unwrap();

    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::InterruptedWrite { revision: 2 })
    ));
    assert_eq!(
        store
            .compare_and_activate(1, &next, &authorization)
            .unwrap(),
        next
    );
}

#[cfg(unix)]
#[test]
fn overwrite_delete_and_rename_do_not_fall_back_to_permissive() {
    let (root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(0, 1, 1),
        true,
    );
    store
        .compare_and_activate(0, &initial, &authorization("credential-owner-a", 1))
        .unwrap();
    let state_path = root.path().join("state-00000000000000000001.json");
    fs::write(&state_path, b"{}\n").unwrap();
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::StateDigestMismatch { revision: 1 })
    ));

    fs::remove_file(&state_path).unwrap();
    assert!(store.load().is_err());
    assert!(!matches!(
        store.load(),
        Ok(AuthoritativeStateLoad::LegacyFirstInstall)
    ));
}

#[cfg(unix)]
#[test]
fn deleting_all_records_does_not_recreate_a_legacy_first_install() {
    let (root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    store
        .compare_and_activate(0, &initial, &authorization("credential-owner-a", 1))
        .unwrap();
    for entry in fs::read_dir(root.path()).unwrap() {
        fs::remove_file(entry.unwrap().path()).unwrap();
    }
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::InterruptedWrite { revision: 1 })
    ));
    assert!(!matches!(
        store.load(),
        Ok(AuthoritativeStateLoad::LegacyFirstInstall)
    ));
}

#[cfg(unix)]
#[test]
fn suffix_truncation_is_rejected_against_the_external_high_water_mark() {
    let (root, store) = store();
    let authorization = authorization("credential-owner-a", 1);
    for revision in 1..=4 {
        let next = state(
            revision,
            "credential-owner-a",
            1,
            if revision >= 3 {
                SecurityLevel::Aggressive
            } else {
                SecurityLevel::Moderate
            },
            generations(revision, revision, revision),
            revision >= 3,
        );
        store
            .compare_and_activate(revision - 1, &next, &authorization)
            .unwrap();
    }
    for revision in 3..=4 {
        for prefix in ["state", "intent", "commit"] {
            fs::remove_file(root.path().join(format!("{prefix}-{revision:020}.json"))).unwrap();
        }
    }
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::AnchorAheadOfRecords {
            anchor_revision: 4,
            record_revision: 2
        })
    ));
}

#[cfg(unix)]
#[test]
fn clearing_kill_switch_requires_a_new_generation() {
    let (_root, store) = store();
    let authorization = authorization("credential-owner-a", 1);
    let active = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        true,
    );
    store
        .compare_and_activate(0, &active, &authorization)
        .unwrap();
    let invalid = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 1),
        false,
    );
    assert!(matches!(
        store.compare_and_activate(1, &invalid, &authorization),
        Err(AuthoritativeStateStoreError::Validation(
            codex_config::AuthoritativeStateValidationError::KillSwitchClearedWithoutGeneration
        ))
    ));
}

#[cfg(unix)]
#[test]
fn symlink_and_permission_weakening_fail_closed() {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::fs::symlink;

    let (root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    store
        .compare_and_activate(0, &initial, &authorization("credential-owner-a", 1))
        .unwrap();
    let state_path = root.path().join("state-00000000000000000001.json");
    let replacement = root.path().join("replacement");
    write_private(&replacement, b"{}\n");
    fs::remove_file(&state_path).unwrap();
    symlink(&replacement, &state_path).unwrap();
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::SymlinkRejected { .. })
    ));

    fs::remove_file(&state_path).unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(
        store.load(),
        Err(AuthoritativeStateStoreError::PermissionsTooOpen { .. })
    ));
}

#[cfg(unix)]
#[test]
fn owner_rotation_blocks_stale_owner_recovery() {
    let (_root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(1, 1, 0),
        false,
    );
    store
        .compare_and_activate(0, &initial, &authorization("credential-owner-a", 1))
        .unwrap();
    let rotated = state(
        2,
        "provenance-owner-b",
        2,
        SecurityLevel::Aggressive,
        generations(2, 2, 1),
        true,
    );
    let rotated_authorization = authorization("provenance-owner-b", 2);
    store
        .compare_and_activate(1, &rotated, &rotated_authorization)
        .unwrap();
    assert!(matches!(
        store.recover_from_revision(2, 1, &rotated_authorization),
        Err(AuthoritativeStateStoreError::Validation(
            codex_config::AuthoritativeStateValidationError::RecoveryOwnerMismatch
        ))
    ));
}

#[cfg(unix)]
#[test]
fn recovery_is_forward_only_and_preserves_restrictions() {
    let (_root, store) = store();
    let authorization = authorization("credential-owner-a", 1);
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Aggressive,
        generations(1, 1, 0),
        false,
    );
    let current = state(
        2,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(5, 7, 7),
        true,
    );
    store
        .compare_and_activate(0, &initial, &authorization)
        .unwrap();
    store
        .compare_and_activate(1, &current, &authorization)
        .unwrap();

    let recovered = store.recover_from_revision(2, 1, &authorization).unwrap();
    assert_eq!(
        recovered,
        AuthoritativeSecurityState {
            schema_version: 1,
            revision: 3,
            owner: current.owner,
            level: SecurityLevel::Aggressive,
            grant_generation: 5,
            revocation_generation: 7,
            kill_switch_generation: 7,
            kill_switch_active: true,
            recovered_from_revision: Some(1),
        }
    );
}

#[cfg(unix)]
fn write_private(path: &std::path::Path, contents: &[u8]) {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(contents).unwrap();
    file.sync_all().unwrap();
}
