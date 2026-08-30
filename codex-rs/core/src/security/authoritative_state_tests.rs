use std::fs;

use codex_config::AuthoritativeSecurityState;
use codex_config::AuthoritativeStateOwner;
use codex_secret_broker::platform_contract::CONTRACT_VERSION;
use codex_secret_broker::platform_contract::Capability;
use codex_secret_broker::platform_contract::CapabilityResult;
use codex_secret_broker::platform_contract::CapabilityStatus;
use codex_secret_broker::platform_contract::FIXTURE_PROTOCOL_VERSION;
use codex_secret_broker::platform_contract::Observation;
use codex_secret_broker::platform_contract::PlatformReport;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

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

#[derive(Clone, Copy)]
struct AuthorityGenerations {
    grant: u64,
    revocation: u64,
    kill_switch: u64,
}

const fn generations(grant: u64, revocation: u64, kill_switch: u64) -> AuthorityGenerations {
    AuthorityGenerations {
        grant,
        revocation,
        kill_switch,
    }
}

fn store() -> (TempDir, AuthoritativeStateStore) {
    let root = tempfile::tempdir().expect("protected state root");
    set_private_directory(root.path());
    let store = AuthoritativeStateStore::new(root.path());
    (root, store)
}

#[cfg(unix)]
fn set_private_directory(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(not(unix))]
fn set_private_directory(_path: &std::path::Path) {}

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

#[test]
fn state_only_crash_resumes_only_the_identical_successor() {
    let (root, store) = store();
    let initial = state(
        1,
        "credential-owner-a",
        1,
        SecurityLevel::Moderate,
        generations(0, 0, 0),
        false,
    );
    let authorization = authorization("credential-owner-a", 1);
    store
        .compare_and_activate(0, &initial, &authorization)
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
        Err(AuthoritativeStateStoreError::InterruptedWrite { revision: 2 })
    ));
    let mut different = next.clone();
    different.grant_generation = 2;
    assert!(matches!(
        store.compare_and_activate(1, &different, &authorization),
        Err(AuthoritativeStateStoreError::InterruptedStateMismatch { revision: 2 })
    ));
    assert_eq!(
        store
            .compare_and_activate(1, &next, &authorization)
            .unwrap(),
        next
    );
}

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

#[cfg(not(unix))]
fn write_private(path: &std::path::Path, contents: &[u8]) {
    fs::write(path, contents).unwrap();
}
