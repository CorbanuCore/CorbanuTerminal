use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::security::SecurityControlAction;
use codex_protocol::security::SecurityControlRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;

use super::super::EffectivePolicyInitialization;
use super::super::EffectivePolicyView;
use super::super::PersistedHumanSecurityState;
use super::super::TrustedSecurityController;
use super::*;

const NOW: i64 = 1_000;

#[derive(Default)]
struct MemoryStore {
    saved: Mutex<Vec<DurableSecurityState>>,
    fail: std::sync::atomic::AtomicBool,
}

impl MemoryStore {
    fn failing() -> Self {
        Self {
            fail: true.into(),
            ..Default::default()
        }
    }

    fn saved(&self) -> Vec<(SecurityLevel, u64, bool)> {
        self.saved
            .lock()
            .unwrap()
            .iter()
            .map(|state| {
                (
                    state.level,
                    state.revocations.generation,
                    state.revocations.kill_switch_active,
                )
            })
            .collect()
    }
}

impl TransitionStore for MemoryStore {
    fn home(&self) -> Option<&std::path::Path> {
        None
    }

    fn update(
        &self,
        _write: crate::security::recovery::TransitionWrite,
        merge: &mut dyn FnMut(
            Option<DurableSecurityState>,
        ) -> Result<DurableSecurityState, TransitionError>,
    ) -> Result<DurableSecurityState, TransitionError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(TransitionError::Persist("disk full".to_string()));
        }
        let mut saved = self.saved.lock().unwrap();
        let next = merge(saved.last().cloned())?;
        saved.push(next.clone());
        Ok(next)
    }
}

#[derive(Default)]
struct CountingSink(AtomicUsize);

impl RevocationSink for CountingSink {
    fn revoke(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

struct Fixture {
    view: EffectivePolicyView,
    controller: TrustedSecurityController,
    root: ThreadId,
    sink: Arc<CountingSink>,
}

fn fixture(level: SecurityLevel) -> Fixture {
    let view = EffectivePolicyView::default();
    let root = ThreadId::new();
    let controller = TrustedSecurityController::initialize(
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
    let sink = Arc::new(CountingSink::default());
    let weak = Arc::downgrade(&sink);
    view.register_revocation_sink(weak as _);
    Fixture {
        view,
        controller,
        root,
        sink,
    }
}

impl Fixture {
    fn prepare(
        &self,
        action: SecurityControlAction,
        probes: ProbeOutcome,
    ) -> Result<PreparedTransition, TransitionError> {
        let epoch = self
            .view
            .snapshot_for_agent(self.root)
            .unwrap()
            .authority_epoch()
            .unwrap();
        let request = SecurityControlRequest::new(epoch, action).unwrap();
        let confirmed = self
            .controller
            .confirm_security_request(request, NOW)
            .unwrap();
        self.controller.prepare_transition(confirmed, probes)
    }

    fn kill(&self, active: bool) -> PreparedTransition {
        self.revoking(RevocationTarget::KillSwitch { active })
    }

    fn revoking(&self, target: RevocationTarget) -> PreparedTransition {
        self.prepare(revoke(target), ProbeOutcome::Passed).unwrap()
    }

    fn set_level(&self, level: SecurityLevel) -> PreparedTransition {
        self.prepare(
            SecurityControlAction::SetLevel { level },
            ProbeOutcome::Passed,
        )
        .unwrap()
    }

    fn commit(
        &self,
        prepared: PreparedTransition,
        store: &MemoryStore,
    ) -> Result<CommittedTransition, TransitionError> {
        self.commit_with(prepared, store)
    }

    fn commit_with(
        &self,
        prepared: PreparedTransition,
        store: &dyn TransitionStore,
    ) -> Result<CommittedTransition, TransitionError> {
        self.controller.commit_transition(prepared, store, NOW)
    }

    /// (level in force, next start, epoch, generation, kill switch).
    fn now(&self) -> (SecurityLevel, SecurityLevel, u64, u64, bool) {
        let snapshot = self.view.snapshot_for_agent(self.root).unwrap();
        (
            snapshot.level,
            snapshot.next_start_level,
            snapshot.epoch,
            snapshot.revocation_generation,
            snapshot.kill_switch_active,
        )
    }

    fn revocations(&self) -> usize {
        self.sink.0.load(Ordering::SeqCst)
    }
}

fn revoke(target: RevocationTarget) -> SecurityControlAction {
    SecurityControlAction::Revoke {
        target,
        reason: RevocationReason::HumanRequest,
    }
}

#[test]
fn security_transition_restrictive_persists_first_and_applies_now() {
    let fixture = fixture(SecurityLevel::Permissive);
    let store = MemoryStore::default();
    let prepared = fixture.set_level(SecurityLevel::Aggressive);
    assert_eq!(prepared.kind(), TransitionKind::Restrictive);
    let committed = fixture.commit(prepared, &store).unwrap();
    assert_eq!(
        committed,
        CommittedTransition {
            kind: TransitionKind::Restrictive,
            epoch: 1,
            revocation_generation: 1,
            level: SecurityLevel::Aggressive,
            next_start_level: SecurityLevel::Aggressive,
            kill_switch_active: false,
            not_saved: None,
        }
    );
    assert_eq!(store.saved(), vec![(SecurityLevel::Aggressive, 1, false)]);
    assert_eq!(
        fixture.now(),
        (
            SecurityLevel::Aggressive,
            SecurityLevel::Aggressive,
            1,
            1,
            false
        )
    );
    assert_eq!(fixture.revocations(), 1);
}

#[test]
fn security_transition_cancel_changes_nothing() {
    let fixture = fixture(SecurityLevel::Moderate);
    let store = MemoryStore::default();
    let before = fixture.now();
    let prepared = fixture.set_level(SecurityLevel::Permissive);
    fixture.controller.cancel_transition(prepared);
    assert_eq!(fixture.now(), before);
    assert_eq!(store.saved(), Vec::new());
    assert_eq!(fixture.revocations(), 0);
}

/// A downgrade or kill-switch release that cannot be saved changes nothing;
/// an emergency stop applies anyway and says it will not survive a restart.
#[test]
fn security_transition_save_failure_blocks_only_widening_changes() {
    let fixture = fixture(SecurityLevel::Aggressive);
    let before = fixture.now();
    let error = fixture
        .commit(
            fixture.set_level(SecurityLevel::Permissive),
            &MemoryStore::failing(),
        )
        .unwrap_err();
    assert!(matches!(error, TransitionError::Persist(_)), "{error}");
    assert_eq!(fixture.now(), before);
    assert_eq!(fixture.revocations(), 0);

    let kill = fixture.kill(/*active*/ true);
    let committed = fixture.commit(kill, &MemoryStore::failing()).unwrap();
    assert!(
        committed
            .not_saved
            .as_deref()
            .is_some_and(|reason| reason.contains("disk full")),
        "{committed:?}"
    );
    assert_eq!(
        fixture.now(),
        (
            SecurityLevel::Aggressive,
            SecurityLevel::Aggressive,
            1,
            1,
            true
        )
    );
    assert_eq!(fixture.revocations(), 1);
    let release = fixture.kill(/*active*/ false);
    assert!(fixture.commit(release, &MemoryStore::failing()).is_err());
    assert!(fixture.now().4);
}

#[test]
fn security_transition_downgrade_applies_at_next_start_and_revokes_now() {
    let fixture = fixture(SecurityLevel::Aggressive);
    let store = MemoryStore::default();
    let prepared = fixture.set_level(SecurityLevel::Permissive);
    assert_eq!(
        (prepared.kind(), prepared.levels()),
        (
            TransitionKind::Downgrade,
            (SecurityLevel::Aggressive, SecurityLevel::Permissive)
        )
    );
    fixture.commit(prepared, &store).unwrap();
    assert_eq!(
        fixture.now(),
        (
            SecurityLevel::Aggressive,
            SecurityLevel::Permissive,
            1,
            1,
            false
        )
    );
    // Choosing the level in force again undoes the pending downgrade.
    let prepared = fixture.set_level(SecurityLevel::Aggressive);
    assert_eq!(prepared.kind(), TransitionKind::Unchanged);
    fixture.commit(prepared, &store).unwrap();
    assert_eq!(
        fixture.now(),
        (
            SecurityLevel::Aggressive,
            SecurityLevel::Aggressive,
            2,
            1,
            false
        )
    );
    assert_eq!(
        store.saved(),
        vec![
            (SecurityLevel::Permissive, 1, false),
            (SecurityLevel::Aggressive, 1, false),
        ]
    );
    // The downgrade's revocation closes the broker's channels now.
    assert_eq!(fixture.revocations(), 1);
}

#[test]
fn security_transition_blocked_until_probes_pass() {
    let fixture = fixture(SecurityLevel::Permissive);
    let blocked = ProbeOutcome::Blocked(vec!["isolated credential broker is off".to_string()]);
    let Err(TransitionError::Blocked(blockers)) = fixture.prepare(
        SecurityControlAction::SetLevel {
            level: SecurityLevel::Moderate,
        },
        blocked.clone(),
    ) else {
        panic!("a protected level must wait for its probes");
    };
    assert_eq!(blockers, vec!["isolated credential broker is off"]);
    assert_eq!(fixture.now().0, SecurityLevel::Permissive);
    // Leaving a level never waits for them.
    let strict = self::fixture(SecurityLevel::Aggressive);
    assert!(
        strict
            .prepare(
                SecurityControlAction::SetLevel {
                    level: SecurityLevel::Permissive,
                },
                blocked.clone(),
            )
            .is_ok()
    );
    // The kill switch never waits for probes.
    let kill = fixture
        .prepare(
            revoke(RevocationTarget::KillSwitch { active: true }),
            blocked,
        )
        .unwrap();
    fixture.commit(kill, &MemoryStore::default()).unwrap();
    assert!(fixture.now().4);
}

#[test]
fn security_transition_stale_or_concurrent_confirmation_is_refused() {
    let fixture = fixture(SecurityLevel::Permissive);
    let store = MemoryStore::default();
    let first = fixture.set_level(SecurityLevel::Moderate);
    let second = fixture.set_level(SecurityLevel::Aggressive);
    fixture.commit(second, &store).unwrap();
    let error = fixture.commit(first, &store).unwrap_err();
    assert!(matches!(error, TransitionError::Policy(_)), "{error}");
    assert_eq!(store.saved(), vec![(SecurityLevel::Aggressive, 1, false)]);
}

#[test]
fn security_transition_concurrent_commits_from_one_epoch_apply_once() {
    let fixture = Arc::new(fixture(SecurityLevel::Permissive));
    let store = Arc::new(MemoryStore::default());
    let prepared = [
        fixture.set_level(SecurityLevel::Moderate),
        fixture.set_level(SecurityLevel::Aggressive),
    ];
    let results: Vec<bool> = prepared
        .into_iter()
        .map(|prepared| {
            let fixture = Arc::clone(&fixture);
            let store = Arc::clone(&store);
            std::thread::spawn(move || fixture.commit(prepared, &store).is_ok())
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|ok| **ok).count(), 1);
    assert_eq!(store.saved().len(), 1);
    assert_eq!(fixture.now().2, 1);
}

#[test]
fn security_transition_fences_children_pending_decisions_and_grants() {
    let fixture = fixture(SecurityLevel::Permissive);
    let child = ThreadId::new();
    let child_before = fixture
        .view
        .inherit_child(fixture.root, child, "task:child", SecurityLevel::Permissive)
        .unwrap();
    let pending = fixture.view.snapshot_for_agent(fixture.root).unwrap();
    fixture
        .commit(
            fixture.set_level(SecurityLevel::Aggressive),
            &MemoryStore::default(),
        )
        .unwrap();
    // A resumed child reads the new level; anything bound to the old epoch
    // (a pending approval, a cached decision, a grant) no longer matches.
    let child_after = fixture.view.snapshot_for_agent(child).unwrap();
    assert_eq!(
        (child_before.level, child_after.level),
        (SecurityLevel::Permissive, SecurityLevel::Aggressive)
    );
    let now = fixture.view.snapshot_for_agent(fixture.root).unwrap();
    assert_ne!(now.authority_epoch(), pending.authority_epoch());
    assert_ne!(
        child_after.authority_epoch(),
        child_before.authority_epoch()
    );
}

#[test]
fn security_transition_kill_switch_and_release() {
    let fixture = fixture(SecurityLevel::Moderate);
    let store = MemoryStore::default();
    fixture
        .commit(fixture.kill(/*active*/ true), &store)
        .unwrap();
    assert_eq!(
        fixture.now(),
        (SecurityLevel::Moderate, SecurityLevel::Moderate, 1, 1, true)
    );
    assert_eq!(fixture.revocations(), 1);
    let release = fixture.kill(/*active*/ false);
    assert_eq!(release.kind(), TransitionKind::KillSwitchRelease);
    fixture.commit(release, &store).unwrap();
    assert_eq!(
        fixture.now(),
        (
            SecurityLevel::Moderate,
            SecurityLevel::Moderate,
            2,
            2,
            false
        )
    );
    assert_eq!(
        store.saved(),
        // A revocation stores no level of its own: the floor stays empty.
        vec![
            (SecurityLevel::Permissive, 1, true),
            (SecurityLevel::Permissive, 2, false),
        ]
    );
    // Releasing opens nothing up again on the broker side.
    assert_eq!(fixture.revocations(), 1);
}

#[test]
fn security_transition_single_grant_revocation_keeps_broker_channels() {
    let fixture = fixture(SecurityLevel::Aggressive);
    let target = RevocationTarget::Grant {
        grant_id: BoundedText::new("grant-1").unwrap(),
    };
    fixture
        .commit(fixture.revoking(target), &MemoryStore::default())
        .unwrap();
    assert_eq!(fixture.now().3, 1);
    assert_eq!(fixture.revocations(), 0);
}

#[test]
fn security_transition_grant_request_is_not_a_transition() {
    let fixture = fixture(SecurityLevel::Aggressive);
    let actor_chain = fixture
        .view
        .snapshot_for_agent(fixture.root)
        .unwrap()
        .actor_chain;
    let thread = fixture.root.to_string();
    let scope = codex_security_policy::GrantScope::new(
        crate::security::aggressive::Surface::UnprotectedCommand.resource(),
        [crate::security::aggressive::Surface::UnprotectedCommand.action()],
        codex_security_policy::GrantContext::new(
            BoundedText::new(format!("session:{thread}")).unwrap(),
            BoundedText::new(format!("task:{thread}")).unwrap(),
            BoundedText::new(crate::security::aggressive::PURPOSE).unwrap(),
            BoundedText::new("command:x").unwrap(),
        ),
        /*destination*/ None,
        Default::default(),
    )
    .unwrap();
    let result = fixture.prepare(
        SecurityControlAction::CreateGrant {
            actor_chain,
            scope,
            expires_at_unix_seconds: NOW + 60,
        },
        ProbeOutcome::Passed,
    );
    assert!(matches!(result, Err(TransitionError::NotATransition)));
}

#[test]
fn security_transition_drops_grants_cached_and_pending_approvals() {
    use crate::security::aggressive;
    use crate::security::tainted_action::PostTaintAction;
    use crate::security::tainted_action::ProtectedActionKind;
    use crate::tools::sandboxing::ApprovalStore;
    use codex_protocol::protocol::ReviewDecision;

    let fixture = fixture(SecurityLevel::Aggressive);
    let thread = fixture.root.to_string();
    let surface = aggressive::Surface::UnprotectedCommand;
    let operation = aggressive::command_operation(&["cat", "x"]);
    let grant = codex_security_policy::BoundedGrant::issue(
        PolicyPrincipal::new(PrincipalKind::Human, "human:test").unwrap(),
        fixture
            .view
            .snapshot_for_agent(fixture.root)
            .unwrap()
            .actor_chain,
        codex_security_policy::GrantScope::new(
            surface.resource(),
            [surface.action()],
            codex_security_policy::GrantContext::new(
                BoundedText::new(thread.clone()).unwrap(),
                BoundedText::new(thread).unwrap(),
                BoundedText::new(aggressive::PURPOSE).unwrap(),
                BoundedText::new(operation.clone()).unwrap(),
            ),
            /*destination*/ None,
            Default::default(),
        )
        .unwrap(),
        NOW - 10,
        NOW + 600,
        BoundedText::new("nonce").unwrap(),
    )
    .unwrap();
    aggressive::issue(fixture.root, &post_taint(&fixture), grant, NOW).unwrap();
    assert!(
        aggressive::admit(
            fixture.root,
            &post_taint(&fixture),
            surface,
            &operation,
            NOW
        )
        .is_some()
    );

    // An approval prompt open while the transition commits.
    let pending = PostTaintAction {
        kind: ProtectedActionKind::Credentials,
        state: post_taint(&fixture),
    };
    let mut cache = ApprovalStore::default();
    cache.fence(fixture.view.authority_marker());
    cache.put("rm -rf build", ReviewDecision::ApprovedForSession);

    // Re-choosing Aggressive (undoing nothing) still moves the epoch.
    fixture
        .commit(
            fixture.revoking(RevocationTarget::AllActiveAuthority),
            &MemoryStore::default(),
        )
        .unwrap();
    assert_eq!(
        aggressive::admit(
            fixture.root,
            &post_taint(&fixture),
            surface,
            &operation,
            NOW
        ),
        None
    );
    cache.fence(fixture.view.authority_marker());
    assert_eq!(cache.get(&"rm -rf build"), None);
    assert!(pending.recheck(Some(&post_taint(&fixture))).is_err());
}

fn post_taint(fixture: &Fixture) -> crate::security::tainted_action::PostTaintState {
    use crate::security::tainted_action::PolicyBinding;
    let snapshot = fixture.view.snapshot_for_agent(fixture.root).unwrap();
    crate::security::tainted_action::PostTaintState {
        taint_generation: 1,
        policy: PolicyBinding::Bound {
            epoch: snapshot.epoch,
            revocation_generation: snapshot.revocation_generation,
            kill_switch_active: snapshot.kill_switch_active,
            level: snapshot.level,
            actor_chain: snapshot.actor_chain,
        },
        level: snapshot.level,
    }
}

/// Two policy trees on one Corbanu home (`/new`, or another process): a
/// later commit in the older tree merges into what is stored instead of
/// overwriting it, and a restrictive commit reaches the other tree now.
#[test]
fn security_transition_trees_on_one_home_merge_and_propagate() {
    let home = tempfile::TempDir::new().unwrap();
    let store = crate::security::recovery::HomeTransitionStore::new(home.path());
    let first = fixture(SecurityLevel::Moderate);
    let second = fixture(SecurityLevel::Moderate);
    first.view.register_home(home.path());
    second.view.register_home(home.path());

    let kill = first.kill(/*active*/ true);
    // The second tree confirms a change before the kill switch lands.
    let unchanged = second.set_level(SecurityLevel::Moderate);
    first.commit_with(kill, &store).unwrap();
    // It reached the second tree at once (and closed its channels) ...
    assert!(second.now().4);
    assert_eq!(second.revocations(), 1);
    // ... so the second tree's older confirmation is stale.
    assert!(second.commit_with(unchanged, &store).is_err());
    let unchanged = second.set_level(SecurityLevel::Moderate);
    second.commit_with(unchanged, &store).unwrap();
    let recovery = crate::security::recovery::recover(home.path(), SecurityLevel::Permissive);
    assert_eq!(
        (
            recovery.level,
            recovery.revocations.kill_switch_active,
            recovery.revocations.generation
        ),
        (SecurityLevel::Moderate, true, 1)
    );

    // Another process (a tree this one cannot reach) at Moderate: its
    // downgrade is refused once a stricter level is stored, rather than
    // lowering it unseen.
    let other_process = fixture(SecurityLevel::Moderate);
    let stricter = first.set_level(SecurityLevel::Aggressive);
    first.commit_with(stricter, &store).unwrap();
    assert_eq!(second.now().0, SecurityLevel::Aggressive);
    let downgrade = other_process.set_level(SecurityLevel::Permissive);
    assert!(matches!(
        other_process.commit_with(downgrade, &store),
        Err(TransitionError::StoredLevelChanged(
            SecurityLevel::Aggressive
        ))
    ));
    assert_eq!(
        crate::security::recovery::recover(home.path(), SecurityLevel::Permissive).level,
        SecurityLevel::Aggressive
    );
}

#[test]
fn security_transition_revoking_an_actor_stops_its_agents() {
    let fixture = fixture(SecurityLevel::Aggressive);
    let child = ThreadId::new();
    fixture
        .view
        .inherit_child(fixture.root, child, "task:child", SecurityLevel::Aggressive)
        .unwrap();
    let actor_id = BoundedText::new(format!("agent:{child}")).unwrap();
    fixture
        .commit(
            fixture.revoking(RevocationTarget::Actor { actor_id }),
            &MemoryStore::default(),
        )
        .unwrap();
    let snapshot = |thread| {
        fixture
            .view
            .snapshot_for_agent(thread)
            .unwrap()
            .kill_switch_active
    };
    assert_eq!((snapshot(fixture.root), snapshot(child)), (false, true));
}

/// Round 2: an emergency stop applied while the disk failed is kept by the
/// next commit that does save (and it is saved then), and a commit with no
/// stored state does not store this tree's level as a floor for the user.
#[test]
fn security_transition_unsaved_kill_switch_survives_the_next_commit() {
    let fixture = fixture(SecurityLevel::Aggressive);
    let store = MemoryStore::failing();
    let kill = fixture.kill(/*active*/ true);
    assert!(fixture.commit(kill, &store).unwrap().not_saved.is_some());
    store.fail.store(false, Ordering::SeqCst);
    let grant = RevocationTarget::Grant {
        grant_id: BoundedText::new("grant-1").unwrap(),
    };
    fixture.commit(fixture.revoking(grant), &store).unwrap();
    assert!(fixture.now().4);
    // Saved with the kill switch, and with no level of its own.
    assert_eq!(store.saved(), vec![(SecurityLevel::Permissive, 2, true)]);
}

/// Round 2: a tree with a longer history (the file was removed since)
/// still takes another tree's kill switch, and a release confirmed before a
/// newer kill switch was stored is refused.
#[test]
fn security_transition_kill_switch_reaches_older_trees_and_stale_release_is_refused() {
    let home = tempfile::TempDir::new().unwrap();
    let store = crate::security::recovery::HomeTransitionStore::new(home.path());
    let older = fixture(SecurityLevel::Moderate);
    older.view.register_home(home.path());
    for id in ["a", "b", "c"] {
        let target = RevocationTarget::Grant {
            grant_id: BoundedText::new(id).unwrap(),
        };
        older.commit_with(older.revoking(target), &store).unwrap();
    }
    std::fs::remove_file(home.path().join("security_state.json")).unwrap();
    let newer = fixture(SecurityLevel::Moderate);
    newer.view.register_home(home.path());
    newer
        .commit_with(newer.kill(/*active*/ true), &store)
        .unwrap();
    assert!(older.now().4);
    assert_eq!(older.now().3, 4);

    // Another process (unreachable) released earlier; a newer kill switch is
    // stored now: the stale release is refused.
    // A session that never showed the kill switch cannot release it.
    let other_process = fixture(SecurityLevel::Moderate);
    assert!(matches!(
        other_process.prepare(
            revoke(RevocationTarget::KillSwitch { active: false }),
            ProbeOutcome::Passed,
        ),
        Err(TransitionError::StoredStateChanged)
    ));
    // One that showed an older kill switch cannot release the newer one.
    let stale = fixture(SecurityLevel::Moderate);
    stale
        .controller
        .commit_transition(
            stale.kill(/*active*/ true),
            &MemoryStore::default(),
            NOW - 100,
        )
        .unwrap();
    let stale_release = stale.kill(/*active*/ false);
    assert!(matches!(
        stale.commit_with(stale_release, &store),
        Err(TransitionError::StoredStateChanged)
    ));
    assert!(
        crate::security::recovery::recover(home.path(), SecurityLevel::Permissive)
            .revocations
            .kill_switch_active
    );
}

/// Round 2: a held state lock never blocks the kill switch: it applies in
/// memory and says it was not saved.
#[test]
fn security_transition_kill_switch_does_not_wait_for_a_held_lock() {
    let home = tempfile::TempDir::new().unwrap();
    let holder = std::fs::File::create(home.path().join("security_state.lock")).unwrap();
    holder.lock().unwrap();
    let fixture = fixture(SecurityLevel::Moderate);
    let started = std::time::Instant::now();
    let committed = fixture
        .commit_with(
            fixture.kill(/*active*/ true),
            &crate::security::recovery::HomeTransitionStore::new(home.path()),
        )
        .unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
    assert!(
        committed
            .not_saved
            .is_some_and(|reason| reason.contains("locked")),
    );
    assert!(fixture.now().4);
}
