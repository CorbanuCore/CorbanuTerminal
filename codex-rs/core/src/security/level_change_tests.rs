use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use super::*;

const NOW: i64 = 1_000;

fn stored(home: &TempDir) -> StoredSecurityState {
    StoredSecurityState::load(home.path())
}

fn commit(
    home: &TempDir,
    target: SecurityLevel,
    probes: Probes,
) -> Result<LevelChangeReport, LevelChangeError> {
    commit_in(home, None, target, probes)
}

fn commit_in(
    home: &TempDir,
    thread: Option<ThreadId>,
    target: SecurityLevel,
    probes: Probes,
) -> Result<LevelChangeReport, LevelChangeError> {
    let basis = LevelBasis::read(home.path(), SecurityLevel::Permissive, thread);
    commit_human_level_change(
        home.path(),
        SecurityLevel::Permissive,
        thread,
        &basis,
        target,
        probes,
        NOW,
    )
}

/// `security_state.json` as another process saves it.
fn saved_elsewhere(home: &TempDir, level: SecurityLevel) {
    std::fs::write(
        home.path().join(recovery::STATE_FILE),
        serde_json::to_vec(&recovery::DurableSecurityState::new(
            level,
            RevocationState::new(),
        ))
        .unwrap(),
    )
    .unwrap();
}

/// A live session tree on `home`, as a session start registers it, and its
/// root thread.
fn live_tree(home: &TempDir, level: SecurityLevel) -> (EffectivePolicyView, ThreadId) {
    let view = EffectivePolicyView::default();
    let root = ThreadId::new();
    TrustedSecurityController::initialize(
        &view,
        PersistedHumanSecurityState::new(
            SecuritySettings::new(level),
            PolicyPrincipal::new(PrincipalKind::Human, "human:test").unwrap(),
            RevocationState::new(),
        )
        .unwrap(),
        root,
        SessionId::from(root),
        EffectivePolicyInitialization::Root,
    )
    .unwrap();
    view.register_home(home.path());
    (view, root)
}

#[test]
fn security_transition_confirmed_stricter_level_is_saved_without_a_session() {
    let home = TempDir::new().unwrap();
    let report = commit(&home, SecurityLevel::Aggressive, Probes::Passed).unwrap();
    assert_eq!(
        (
            report.kind,
            report.next_start,
            report.live,
            report.not_saved
        ),
        (
            LevelChangeKind::Stricter,
            SecurityLevel::Aggressive,
            false,
            None
        )
    );
    assert_eq!(
        stored(&home),
        StoredSecurityState::Level(SecurityLevel::Aggressive)
    );
}

#[test]
fn security_transition_blocked_preflight_changes_nothing() {
    let home = TempDir::new().unwrap();
    let result = commit(
        &home,
        SecurityLevel::Aggressive,
        Probes::Blocked(vec!["Vault: damaged".to_string()]),
    );
    assert!(
        matches!(&result, Err(LevelChangeError::Refused(reason)) if reason.contains("Vault: damaged")),
        "{result:?}"
    );
    assert_eq!(stored(&home), StoredSecurityState::Absent);
}

#[test]
fn security_transition_stricter_level_applies_to_the_live_session_now() {
    let home = TempDir::new().unwrap();
    let (view, root) = live_tree(&home, SecurityLevel::Permissive);
    // Another session of this process on the same home.
    let (other, _) = live_tree(&home, SecurityLevel::Permissive);
    let report = commit_in(&home, Some(root), SecurityLevel::Aggressive, Probes::Passed).unwrap();
    assert!(report.live);
    assert_eq!(report.in_force, SecurityLevel::Aggressive);
    let marker = view.authority_marker().unwrap();
    assert_eq!(marker.0, 1, "the commit moved the live tree's epoch");
    assert_eq!(
        other.authority_marker().unwrap().0,
        1,
        "and reached the other tree"
    );
    assert_eq!(
        LevelBasis::read(home.path(), SecurityLevel::Permissive, Some(root)).in_force,
        SecurityLevel::Aggressive
    );
}

/// A downgrade is saved now and enforced from the next start; the session
/// keeps the stricter level until then.
#[test]
fn security_transition_downgrade_waits_for_the_next_start() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Permissive);
    commit_in(&home, Some(root), SecurityLevel::Aggressive, Probes::Passed).unwrap();
    let report = commit_in(&home, Some(root), SecurityLevel::Permissive, Probes::Passed).unwrap();
    assert_eq!(
        (report.kind, report.in_force, report.next_start),
        (
            LevelChangeKind::Downgrade,
            SecurityLevel::Aggressive,
            SecurityLevel::Permissive
        )
    );
    assert_eq!(
        stored(&home),
        StoredSecurityState::Level(SecurityLevel::Permissive)
    );
    // A config.toml that sets no level is left alone.
    assert!(!home.path().join("config.toml").exists());
}

#[test]
fn security_transition_downgrade_rewrites_a_stricter_user_config_level_only() {
    let home = TempDir::new().unwrap();
    let config = home.path().join("config.toml");
    std::fs::write(
        &config,
        "model = \"m\"\n\n[security]\nversion = 1\nlevel = \"aggressive\"\n",
    )
    .unwrap();
    commit(&home, SecurityLevel::Aggressive, Probes::Passed).unwrap();
    let basis = LevelBasis::read(home.path(), SecurityLevel::Aggressive, None);
    commit_human_level_change(
        home.path(),
        SecurityLevel::Aggressive,
        None,
        &basis,
        SecurityLevel::Permissive,
        Probes::Passed,
        NOW,
    )
    .unwrap();
    let contents = std::fs::read_to_string(&config).unwrap();
    assert!(contents.contains("level = \"permissive\""), "{contents}");
    assert!(contents.contains("model = \"m\""), "{contents}");
}

#[test]
fn security_transition_refuses_a_state_that_changed_after_review() {
    let home = TempDir::new().unwrap();
    let basis = LevelBasis::read(home.path(), SecurityLevel::Permissive, None);
    // Another process stores Aggressive after the review was shown.
    commit(&home, SecurityLevel::Aggressive, Probes::Passed).unwrap();
    let result = commit_human_level_change(
        home.path(),
        SecurityLevel::Permissive,
        None,
        &basis,
        SecurityLevel::Permissive,
        Probes::Passed,
        NOW,
    );
    assert!(
        matches!(result, Err(LevelChangeError::Changed)),
        "{result:?}"
    );
    assert_eq!(
        stored(&home),
        StoredSecurityState::Level(SecurityLevel::Aggressive)
    );
}

#[test]
fn security_transition_a_confirmed_level_repairs_an_unreadable_state() {
    let home = TempDir::new().unwrap();
    std::fs::write(home.path().join(recovery::STATE_FILE), "{").unwrap();
    assert!(matches!(stored(&home), StoredSecurityState::Unreadable(_)));
    let report = commit(&home, SecurityLevel::Permissive, Probes::Passed).unwrap();
    assert_eq!(report.next_start, SecurityLevel::Permissive);
    assert_eq!(
        stored(&home),
        StoredSecurityState::Level(SecurityLevel::Permissive)
    );
}

/// Review 1, finding 2: this session is Permissive but another process saved
/// Aggressive. Choosing Permissive lowers the saved record (a downgrade), so
/// the level file and Core's record agree at the next start.
#[test]
fn security_transition_permissive_lowers_a_stricter_record_saved_elsewhere() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Permissive);
    // Another process saved Aggressive (it does not reach this tree).
    saved_elsewhere(&home, SecurityLevel::Aggressive);
    let report = commit_in(&home, Some(root), SecurityLevel::Permissive, Probes::Passed).unwrap();
    // The session is not raised first (review 2, finding 2).
    assert_eq!(
        (report.kind, report.in_force, report.next_start),
        (
            LevelChangeKind::Downgrade,
            SecurityLevel::Permissive,
            SecurityLevel::Permissive
        )
    );
    assert_eq!(
        stored(&home),
        StoredSecurityState::Level(SecurityLevel::Permissive)
    );
}

/// Review 2, finding 3: a level chosen again that raises the saved record
/// raises the other sessions below it and ends their "for session"
/// approvals (their sinks are told).
#[test]
fn security_transition_unchanged_raise_notifies_the_other_sessions() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    struct Sink(AtomicUsize);
    impl super::super::transition::RevocationSink for Sink {
        fn revoke(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let (other, _) = live_tree(&home, SecurityLevel::Permissive);
    let sink = std::sync::Arc::new(Sink(AtomicUsize::new(0)));
    other.register_revocation_sink(std::sync::Arc::downgrade(&sink) as _);
    let report = commit_in(&home, Some(root), SecurityLevel::Aggressive, Probes::Passed).unwrap();
    assert_eq!(report.kind, LevelChangeKind::Unchanged);
    assert_eq!(sink.0.load(Ordering::SeqCst), 1);
    assert_eq!(other.authority_marker().unwrap().0, 1);
}

/// Review 1, finding 7: a commit in this tree after the review was shown
/// (another confirmation in this process) makes the review stale.
#[test]
fn security_transition_review_is_bound_to_the_tree_epoch() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Permissive);
    let basis = LevelBasis::read(home.path(), SecurityLevel::Permissive, Some(root));
    commit_in(&home, Some(root), SecurityLevel::Permissive, Probes::Passed).unwrap();
    let result = commit_human_level_change(
        home.path(),
        SecurityLevel::Permissive,
        Some(root),
        &basis,
        SecurityLevel::Aggressive,
        Probes::Passed,
        NOW,
    );
    assert!(
        matches!(result, Err(LevelChangeError::Changed)),
        "{result:?}"
    );
}

/// Review 3, finding 1: a downgrade of the saved record that keeps the
/// level in force still sets what the next start enforces.
#[test]
fn security_transition_downgrade_of_the_record_sets_the_next_start() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Permissive);
    saved_elsewhere(&home, SecurityLevel::Aggressive);
    // An unchanged commit that merges the stricter saved record (as in a
    // race with the other process): the next start becomes Aggressive while
    // Permissive stays in force.
    let controller = live_controller(home.path(), root).unwrap();
    let request = SecurityControlRequest::new(
        controller.authority_epoch().unwrap(),
        SecurityControlAction::SetLevel {
            level: SecurityLevel::Permissive,
        },
    )
    .unwrap();
    let confirmed = controller.confirm_security_request(request, NOW).unwrap();
    let prepared = controller
        .prepare_transition(confirmed, ProbeOutcome::Passed)
        .unwrap();
    let committed = controller
        .commit_transition(prepared, &HomeTransitionStore::new(home.path()), NOW)
        .unwrap();
    assert_eq!(
        (committed.level, committed.next_start_level),
        (SecurityLevel::Permissive, SecurityLevel::Aggressive)
    );
    let report = commit_in(&home, Some(root), SecurityLevel::Permissive, Probes::Passed).unwrap();
    assert_eq!(
        (report.kind, report.next_start),
        (LevelChangeKind::Downgrade, SecurityLevel::Permissive)
    );
}
