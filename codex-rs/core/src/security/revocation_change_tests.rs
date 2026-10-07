use codex_protocol::SessionId;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use tempfile::TempDir;

use super::super::EffectivePolicyInitialization;
use super::super::EffectivePolicyView;
use super::super::PersistedHumanSecurityState;
use super::super::TrustedSecurityController;
use super::super::aggressive;
use super::super::recovery;
use super::super::tainted_action::PolicyBinding;
use super::super::tainted_action::PostTaintState;
use super::*;

use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;

/// The same second for every commit: Core orders kill-switch changes made
/// within one second itself.
static NOW: AtomicI64 = AtomicI64::new(1_000);

fn now() -> i64 {
    NOW.load(Ordering::Relaxed)
}

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

fn basis(home: &TempDir, thread: Option<ThreadId>) -> LevelBasis {
    LevelBasis::read(home.path(), SecurityLevel::Permissive, thread)
}

fn revoke(
    home: &TempDir,
    thread: Option<ThreadId>,
    choice: HumanRevocation,
) -> Result<RevocationReport, LevelChangeError> {
    commit_human_revocation(
        home.path(),
        SecurityLevel::Permissive,
        thread,
        &basis(home, thread),
        choice,
        now(),
    )
}

fn stored_kill_switch(home: &TempDir) -> Option<(SecurityLevel, bool)> {
    recovery::load(home.path())
        .ok()
        .flatten()
        .map(|state| (state.level, state.revocations.kill_switch_active))
}

/// A grant held by `thread` under an Aggressive policy, as the grant review
/// issues it.
fn hold_grant(thread: ThreadId, operation: &str) -> String {
    let human = PolicyPrincipal::new(PrincipalKind::Human, "human").unwrap();
    let chain = codex_security_policy::ActorChain::new(vec![human.clone()]).unwrap();
    let text = |value: &str| BoundedText::new(value).unwrap();
    let surface = aggressive::Surface::UnprotectedCommand;
    let scope = GrantScope::new(
        surface.resource(),
        [surface.action()],
        GrantContext::new(
            text(&thread.to_string()),
            text(&thread.to_string()),
            text(aggressive::PURPOSE),
            text(operation),
        ),
        /*destination*/ None,
        BTreeMap::new(),
    )
    .unwrap();
    let now = aggressive::now_unix_seconds();
    let grant =
        BoundedGrant::issue(human, chain.clone(), scope, now, now + 600, text(operation)).unwrap();
    let id = grant.grant_id.as_str().to_string();
    let state = PostTaintState {
        taint_generation: 0,
        policy: PolicyBinding::Bound {
            epoch: 0,
            revocation_generation: 0,
            kill_switch_active: false,
            level: SecurityLevel::Aggressive,
            actor_chain: chain,
        },
        level: SecurityLevel::Aggressive,
    };
    let command = operation.split(' ').map(str::to_string).collect();
    aggressive::issue_labelled(thread, &state, grant, command, now).unwrap();
    id
}

fn held_by(thread: ThreadId) -> Vec<String> {
    aggressive::held(aggressive::now_unix_seconds())
        .into_iter()
        .filter(|grant| grant.thread == thread)
        .map(|grant| grant.label)
        .collect()
}

/// The kill switch applies now to this session and the others of the
/// process on the home, is saved, and holds at the next start; turning it
/// off keeps the level.
#[test]
fn security_revocation_kill_switch_applies_now_saves_and_releases_without_lowering() {
    let home = TempDir::new().unwrap();
    let (view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let (other, _) = live_tree(&home, SecurityLevel::Aggressive);
    let grant = hold_grant(root, "cat x");

    let report = revoke(&home, Some(root), HumanRevocation::KillSwitchOn).unwrap();
    assert_eq!(
        report,
        RevocationReport {
            in_force: SecurityLevel::Aggressive,
            kill_switch_active: true,
            live: true,
            not_saved: None,
        }
    );
    assert!(basis(&home, Some(root)).kill_switch_active);
    assert!(view.authority_marker().unwrap().0 > 0);
    assert!(
        other.authority_marker().unwrap().0 > 0,
        "the other session took it"
    );
    assert_eq!(held_by(root), Vec::<String>::new(), "{grant} ended");
    // A revocation never stores a level of its own: this tree's Aggressive
    // came from its configuration, not from a saved level.
    assert_eq!(
        stored_kill_switch(&home),
        Some((SecurityLevel::Permissive, true))
    );
    // The next start reads it.
    assert!(basis(&home, /*thread*/ None).kill_switch_active);

    let report = revoke(&home, Some(root), HumanRevocation::KillSwitchOff).unwrap();
    assert_eq!(
        (report.in_force, report.kill_switch_active),
        (SecurityLevel::Aggressive, false)
    );
    assert_eq!(
        stored_kill_switch(&home),
        Some((SecurityLevel::Permissive, false))
    );
}

/// Only a kill switch the person saw can be turned off.
#[test]
fn security_revocation_cannot_release_a_kill_switch_that_is_off() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    assert!(revoke(&home, Some(root), HumanRevocation::KillSwitchOff).is_err());
    assert_eq!(stored_kill_switch(&home), None);
}

/// Without a session the kill switch is saved for the next start, and
/// released from the saved state.
#[test]
fn security_revocation_kill_switch_without_a_session_is_saved() {
    let home = TempDir::new().unwrap();
    let report = revoke(&home, None, HumanRevocation::KillSwitchOn).unwrap();
    assert!(report.kill_switch_active && !report.live);
    assert_eq!(
        stored_kill_switch(&home),
        Some((SecurityLevel::Permissive, true))
    );
    revoke(&home, None, HumanRevocation::KillSwitchOff).unwrap();
    assert_eq!(
        stored_kill_switch(&home),
        Some((SecurityLevel::Permissive, false))
    );
}

/// Revoking all active authority ends every grant now and is saved; the
/// kill switch stays off and the level stays.
#[test]
fn security_revocation_all_active_authority_ends_grants() {
    let home = TempDir::new().unwrap();
    let (view, root) = live_tree(&home, SecurityLevel::Aggressive);
    hold_grant(root, "cat a");
    hold_grant(root, "cat b");
    let before = view.authority_marker().unwrap();
    let report = revoke(&home, Some(root), HumanRevocation::AllActiveAuthority).unwrap();
    assert_eq!(
        (report.in_force, report.kill_switch_active, report.live),
        (SecurityLevel::Aggressive, false, true)
    );
    assert_eq!(held_by(root), Vec::<String>::new());
    assert_ne!(view.authority_marker().unwrap(), before, "epoch moved");
    assert_eq!(
        stored_kill_switch(&home),
        Some((SecurityLevel::Permissive, false))
    );
}

/// A state changed since the review (another session turned the kill
/// switch on) is refused: nothing changes.
#[test]
fn security_revocation_refuses_a_state_changed_since_review() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let reviewed = basis(&home, Some(root));
    revoke(&home, Some(root), HumanRevocation::KillSwitchOn).unwrap();
    let result = commit_human_revocation(
        home.path(),
        SecurityLevel::Permissive,
        Some(root),
        &reviewed,
        HumanRevocation::AllActiveAuthority,
        now(),
    );
    assert!(
        matches!(result, Err(LevelChangeError::Changed)),
        "{result:?}"
    );
}

/// One grant ends; the others stay.
#[test]
fn security_revocation_one_grant_ends_only_that_grant() {
    let thread = ThreadId::new();
    let first = hold_grant(thread, "cat one");
    hold_grant(thread, "cat two");
    assert!(revoke_grant(&first));
    assert!(!revoke_grant(&first), "already gone");
    assert_eq!(held_by(thread), vec!["cat two".to_string()]);
}

/// Turning the switch off reaches the other sessions of the process that
/// hold the same switch.
#[test]
fn security_revocation_release_reaches_the_other_sessions() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let (_other_view, other) = live_tree(&home, SecurityLevel::Aggressive);
    revoke(&home, Some(root), HumanRevocation::KillSwitchOn).unwrap();
    assert!(basis(&home, Some(other)).kill_switch_active);
    revoke(&home, Some(root), HumanRevocation::KillSwitchOff).unwrap();
    assert!(!basis(&home, Some(other)).kill_switch_active);
    // And that session can turn it on again itself.
    revoke(&home, Some(other), HumanRevocation::KillSwitchOn).unwrap();
}

/// Without a session, a release is for the switch the person saw: one
/// turned off and on again elsewhere since the review is refused.
#[test]
fn security_revocation_release_is_for_the_switch_reviewed() {
    let home = TempDir::new().unwrap();
    revoke(&home, None, HumanRevocation::KillSwitchOn).unwrap();
    let reviewed = basis(&home, None);
    revoke(&home, None, HumanRevocation::KillSwitchOff).unwrap();
    revoke(&home, None, HumanRevocation::KillSwitchOn).unwrap();
    let result = commit_human_revocation(
        home.path(),
        SecurityLevel::Permissive,
        None,
        &reviewed,
        HumanRevocation::KillSwitchOff,
        now(),
    );
    assert!(
        matches!(result, Err(LevelChangeError::Changed)),
        "{result:?}"
    );
    assert_eq!(
        stored_kill_switch(&home),
        Some((SecurityLevel::Permissive, true))
    );
}

/// Without a session, a kill switch that cannot be saved changed nothing
/// and says so.
#[test]
fn security_revocation_unsaved_without_a_session_is_an_error() {
    let home = TempDir::new().unwrap();
    std::fs::create_dir(home.path().join("security_state.lock")).unwrap();
    let result = revoke(&home, None, HumanRevocation::KillSwitchOn);
    assert!(
        matches!(result, Err(LevelChangeError::NotSaved(_))),
        "{result:?}"
    );
}

/// Revoke all also ends grants of sessions outside this home's trees.
#[test]
fn security_revocation_all_ends_every_grant_of_the_process() {
    let home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    let elsewhere = ThreadId::new();
    hold_grant(elsewhere, "cat elsewhere");
    revoke(&home, Some(root), HumanRevocation::AllActiveAuthority).unwrap();
    assert_eq!(held_by(elsewhere), Vec::<String>::new());
}

/// A release reaches only the sessions that hold the switch released; a
/// session whose switch is another one keeps it (and its epoch).
#[test]
fn security_revocation_release_reaches_only_the_same_switch() {
    let home = TempDir::new().unwrap();
    let other_home = TempDir::new().unwrap();
    let (_view, root) = live_tree(&home, SecurityLevel::Aggressive);
    // A tree on another home with its own switch is never touched.
    let (other_view, other) = live_tree(&other_home, SecurityLevel::Aggressive);
    revoke(&other_home, Some(other), HumanRevocation::KillSwitchOn).unwrap();
    let before = other_view.authority_marker().unwrap();
    revoke(&home, Some(root), HumanRevocation::KillSwitchOn).unwrap();
    revoke(&home, Some(root), HumanRevocation::KillSwitchOff).unwrap();
    assert!(basis(&other_home, Some(other)).kill_switch_active);
    assert_eq!(other_view.authority_marker().unwrap(), before);
}

/// Without a session but with another live session on the home, an unsaved
/// kill switch applied there, and the report says so.
#[test]
fn security_revocation_unsaved_but_taken_by_another_session_is_reported() {
    let home = TempDir::new().unwrap();
    let (_view, other) = live_tree(&home, SecurityLevel::Aggressive);
    std::fs::create_dir(home.path().join("security_state.lock")).unwrap();
    let report = revoke(&home, None, HumanRevocation::KillSwitchOn).unwrap();
    assert!(report.live && report.not_saved.is_some(), "{report:?}");
    assert!(basis(&home, Some(other)).kill_switch_active);
}
