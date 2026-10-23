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
#[serial_test::serial(aggressive_grants)]
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
            reached_other_trees: false,
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
#[serial_test::serial(aggressive_grants)]
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
#[serial_test::serial(aggressive_grants)]
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
#[serial_test::serial(aggressive_grants)]
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
#[serial_test::serial(aggressive_grants)]
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
#[serial_test::serial(aggressive_grants)]
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
#[serial_test::serial(aggressive_grants)]
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
#[serial_test::serial(aggressive_grants)]
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

/// Another process saving to the same home: the file changes, but no tree
/// of this process is reached by the commit itself.
struct OtherProcessStore(crate::security::recovery::HomeTransitionStore);

impl TransitionStore for OtherProcessStore {
    fn home(&self) -> Option<&std::path::Path> {
        None
    }

    fn update(
        &self,
        write: crate::security::recovery::TransitionWrite,
        merge: &mut dyn FnMut(
            Option<DurableSecurityState>,
        ) -> Result<DurableSecurityState, TransitionError>,
    ) -> Result<DurableSecurityState, TransitionError> {
        self.0.update(write, merge)
    }
}

/// Waits up to `limit` for `done`.
fn within(limit: std::time::Duration, done: impl Fn() -> bool) -> bool {
    let deadline = std::time::Instant::now() + limit;
    while std::time::Instant::now() < deadline {
        if done() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    done()
}

/// #507 (Travis, 2026-10-12): a kill switch another Corbanu Terminal process
/// turns on for the home reaches the trees of this process while they run
/// (their sinks stop the running work), within the stated latency, and so
/// does turning it off. Levels are not taken from the file.
#[test]
#[serial_test::serial(aggressive_grants)]
fn security_transition_kill_switch_from_another_process_reaches_running_trees() {
    let latency = std::time::Duration::from_secs(2);
    let home = tempfile::TempDir::new().unwrap();
    let here = fixture(SecurityLevel::Moderate);
    here.view.register_home(home.path());
    let mut watched = kill_switch_watch::start_for_test(home.path());
    assert!(!*watched.borrow_and_update());
    let other = fixture(SecurityLevel::Moderate);
    let store = OtherProcessStore(crate::security::recovery::HomeTransitionStore::new(
        home.path(),
    ));

    let committed = other
        .commit_with(other.kill(/*active*/ true), &store)
        .unwrap();
    assert!(!committed.reached_other_trees);
    // #931: subscribers are told after the watcher pass has adopted the
    // switch and run the sinks; the tree's state alone changes first.
    assert!(
        within(latency, || *watched.borrow()),
        "the kill switch did not arrive"
    );
    assert!(here.now().4);
    assert!(here.revocations() >= 1, "the running work was not stopped");
    // A stricter level saved by the other process is not taken while running
    // (only the kill switch and the revocations saved with it are).
    assert_eq!(here.now().0, SecurityLevel::Moderate);

    let release = other.kill(/*active*/ false);
    other
        .controller
        .commit_transition(release, &store, NOW + 1)
        .unwrap();
    assert!(
        within(latency, || !here.now().4),
        "turning it off did not arrive"
    );
    assert!(within(latency, || !*watched.borrow()));
}

/// #507: a switch this tree turned on later than the one saved is kept; a
/// release saved by another process does not end a switch it never saw (one
/// this process could not save); an unreadable file changes nothing while
/// running.
#[test]
#[serial_test::serial(aggressive_grants)]
fn security_transition_kill_switch_watch_keeps_unseen_switches_and_ignores_unreadable() {
    let home = tempfile::TempDir::new().unwrap();
    let here = fixture(SecurityLevel::Moderate);
    here.view.register_home(home.path());
    let other = fixture(SecurityLevel::Moderate);
    let store = OtherProcessStore(crate::security::recovery::HomeTransitionStore::new(
        home.path(),
    ));
    other
        .commit_with(other.kill(/*active*/ true), &store)
        .unwrap();
    // This tree turns its own switch on later, and cannot save it.
    here.controller
        .commit_transition(here.kill(/*active*/ true), &MemoryStore::failing(), NOW + 5)
        .unwrap();
    let held = here.controller.kill_switch_event_id();
    assert!(held.is_some());
    assert_eq!(kill_switch_watch::apply(home.path()), 0);
    assert_eq!(here.controller.kill_switch_event_id(), held);

    // The other process releases its own switch, later still: this tree's
    // unseen switch stays on.
    other
        .controller
        .commit_transition(other.kill(/*active*/ false), &store, NOW + 9)
        .unwrap();
    assert_eq!(kill_switch_watch::apply(home.path()), 0);
    assert!(here.now().4);
    assert_eq!(here.controller.kill_switch_event_id(), held);

    std::fs::write(home.path().join("security_state.json"), b"{not json").unwrap();
    assert_eq!(kill_switch_watch::apply(home.path()), 0);
    assert!(here.now().4);
}

/// #507: a switch another process turned on and off again before this one
/// looked (one wake-up) changes nothing here: nothing to stop or end.
#[test]
#[serial_test::serial(aggressive_grants)]
fn security_transition_kill_switch_watch_on_and_off_within_one_look_changes_nothing() {
    let home = tempfile::TempDir::new().unwrap();
    let here = fixture(SecurityLevel::Moderate);
    here.view.register_home(home.path());
    let before = here.now();
    let other = fixture(SecurityLevel::Moderate);
    let store = OtherProcessStore(crate::security::recovery::HomeTransitionStore::new(
        home.path(),
    ));
    other
        .commit_with(other.kill(/*active*/ true), &store)
        .unwrap();
    other
        .controller
        .commit_transition(other.kill(/*active*/ false), &store, NOW + 1)
        .unwrap();
    assert_eq!(kill_switch_watch::apply(home.path()), 0);
    assert_eq!(here.now(), before);
    assert_eq!(here.revocations(), 0);
}

/// #507: a switch this process saves itself updates the subscribers' value
/// without signalling a change made elsewhere (the confirming screen reports
/// it); one saved by another process is signalled.
#[test]
#[serial_test::serial(aggressive_grants)]
fn security_transition_kill_switch_watch_signals_only_other_processes() {
    let home = tempfile::TempDir::new().unwrap();
    let here = fixture(SecurityLevel::Moderate);
    here.view.register_home(home.path());
    let mut watched = kill_switch_watch::start_for_test(home.path());
    let _ = watched.borrow_and_update();
    let local = crate::security::recovery::HomeTransitionStore::new(home.path());
    here.commit_with(here.kill(/*active*/ true), &local)
        .unwrap();
    assert!(within(std::time::Duration::from_secs(2), || *watched.borrow()));
    std::thread::sleep(kill_switch_watch::POLL);
    assert!(
        !watched.has_changed().unwrap(),
        "a local change was signalled"
    );

    let other = fixture(SecurityLevel::Moderate);
    let store = OtherProcessStore(crate::security::recovery::HomeTransitionStore::new(
        home.path(),
    ));
    // The other process saw this switch (it read the saved state) and
    // releases it.
    let seen = crate::security::recovery::recover(home.path(), SecurityLevel::Moderate);
    other.controller.catch_up(&seen);
    other
        .controller
        .commit_transition(other.kill(/*active*/ false), &store, NOW + 3)
        .unwrap();
    assert!(within(std::time::Duration::from_secs(2), || {
        watched.has_changed().unwrap_or(false)
    }));
    assert!(!*watched.borrow_and_update());
    assert!(!here.now().4);
}

/// #842 (Travis, 2026-10-15): Moderate confirmed in another Corbanu Terminal
/// process on the home raises this process's trees below it while they run,
/// as a confirmation raises the other sessions of its own process: Core's
/// level now (the next start stays Moderate), grants and "for session"
/// approvals end (the epoch moves) and the sinks revoke; subscribers are
/// told. A tree already at Moderate or above is left alone, and a lower
/// level saved elsewhere is not taken while running (Aggressive is, #919).
#[test]
#[serial_test::serial(aggressive_grants)]
fn issue_842_moderate_from_another_process_raises_running_trees() {
    let latency = std::time::Duration::from_secs(2);
    let home = tempfile::TempDir::new().unwrap();
    let here = fixture(SecurityLevel::Permissive);
    here.view.register_home(home.path());
    let already = fixture(SecurityLevel::Aggressive);
    already.view.register_home(home.path());
    let mut raised = kill_switch_watch::subscribe_level_raised(home.path());
    assert_eq!(*raised.borrow_and_update(), None);
    let (_, _, epoch, _, _) = here.now();
    let other = fixture(SecurityLevel::Permissive);
    let store = OtherProcessStore(crate::security::recovery::HomeTransitionStore::new(
        home.path(),
    ));

    let committed = other
        .commit_with(other.set_level(SecurityLevel::Moderate), &store)
        .unwrap();
    assert!(!committed.reached_other_trees);
    // #931: subscribers are told after the watcher pass has raised the tree
    // and run its sinks; the level alone becomes visible first.
    assert!(
        within(latency, || raised.has_changed().unwrap_or(false)),
        "Moderate did not arrive"
    );
    assert_eq!(*raised.borrow_and_update(), Some(SecurityLevel::Moderate));
    let (level, next_start, after, _, kill_switch) = here.now();
    assert_eq!(
        (level, next_start, kill_switch),
        (SecurityLevel::Moderate, SecurityLevel::Moderate, false)
    );
    assert!(after > epoch, "grants and approvals did not end");
    assert_eq!(here.revocations(), 1);
    // Above it: unchanged.
    assert_eq!(already.now().0, SecurityLevel::Aggressive);
    assert_eq!(already.revocations(), 0);
    // Seen once: nothing more to take.
    assert_eq!(kill_switch_watch::apply_saved(home.path()).raised, None);

    // A downgrade saved elsewhere (Permissive) raises nothing. The tree is
    // registered after it is saved: one registered while Moderate is still
    // saved rises to it on the watch's next read (the downgrade's own write).
    let downgrading = fixture(SecurityLevel::Moderate);
    downgrading
        .commit_with(downgrading.set_level(SecurityLevel::Permissive), &store)
        .unwrap();
    let lower = fixture(SecurityLevel::Permissive);
    lower.view.register_home(home.path());
    assert_eq!(kill_switch_watch::apply_saved(home.path()).raised, None);
    assert_eq!(lower.now().0, SecurityLevel::Permissive);
    assert_eq!(here.now().0, SecurityLevel::Moderate);
}

/// #919 (Travis, 2026-10-16, D20): Aggressive confirmed in another Corbanu
/// Terminal process on the home raises this process's trees below it while
/// they run (Permissive and Moderate alike), as a confirmation raises the
/// other sessions of its own process: Core's level now (the next start stays
/// Aggressive), grants and "for session" approvals end (the epoch moves),
/// the sinks revoke, and this process's sessions take approval for every
/// command (#505); subscribers are told. A tree already at Aggressive is
/// left alone, and nothing saved later lowers a tree while it runs. Before
/// #919 Aggressive saved elsewhere was not taken until a restart.
#[test]
#[serial_test::serial(aggressive_grants)]
fn issue_919_aggressive_from_another_process_raises_running_trees() {
    let latency = std::time::Duration::from_secs(5);
    let home = tempfile::TempDir::new().unwrap();
    let permissive = fixture(SecurityLevel::Permissive);
    permissive.view.register_home(home.path());
    let moderate = fixture(SecurityLevel::Moderate);
    moderate.view.register_home(home.path());
    let already = fixture(SecurityLevel::Aggressive);
    already.view.register_home(home.path());
    let mut raised = kill_switch_watch::subscribe_level_raised(home.path());
    assert_eq!(*raised.borrow_and_update(), None);
    let (_, _, permissive_epoch, _, _) = permissive.now();
    let (_, _, moderate_epoch, _, _) = moderate.now();
    assert!(!crate::security::confirmed_approvals::confirmed_here(
        home.path()
    ));
    let other = fixture(SecurityLevel::Permissive);
    let store = OtherProcessStore(crate::security::recovery::HomeTransitionStore::new(
        home.path(),
    ));

    let committed = other
        .commit_with(other.set_level(SecurityLevel::Aggressive), &store)
        .unwrap();
    assert!(!committed.reached_other_trees);
    assert!(
        within(latency, || raised.has_changed().unwrap_or(false)),
        "Aggressive did not arrive"
    );
    assert_eq!(*raised.borrow_and_update(), Some(SecurityLevel::Aggressive));
    for (tree, epoch) in [(&permissive, permissive_epoch), (&moderate, moderate_epoch)] {
        let (level, next_start, after, _, kill_switch) = tree.now();
        assert_eq!(
            (level, next_start, kill_switch),
            (SecurityLevel::Aggressive, SecurityLevel::Aggressive, false)
        );
        assert!(after > epoch, "grants and approvals did not end");
        assert_eq!(tree.revocations(), 1);
    }
    // #505: approval for every command, in this process's sessions on the
    // home and those it starts later (signalled after it applied).
    assert!(crate::security::confirmed_approvals::confirmed_here(
        home.path()
    ));
    // Already Aggressive: unchanged.
    assert_eq!(already.now().0, SecurityLevel::Aggressive);
    assert_eq!(already.revocations(), 0);
    // Seen once: nothing more to take.
    assert_eq!(kill_switch_watch::apply_saved(home.path()).raised, None);

    // Never lowered while running: Moderate, then Permissive, saved
    // elsewhere (downgrades there) leave these trees Aggressive.
    for lower in [SecurityLevel::Moderate, SecurityLevel::Permissive] {
        let downgrading = fixture(SecurityLevel::Aggressive);
        downgrading
            .commit_with(downgrading.set_level(lower), &store)
            .unwrap();
        assert_eq!(
            crate::security::recovery::load(home.path())
                .ok()
                .flatten()
                .map(|state| state.level),
            Some(lower)
        );
        assert_eq!(kill_switch_watch::apply_saved(home.path()).raised, None);
        for tree in [&permissive, &moderate, &already] {
            assert_eq!(tree.now().0, SecurityLevel::Aggressive);
        }
    }
    assert_eq!(permissive.revocations(), 1);
}

/// #919: the same across two real processes on one home. A child process
/// (this test binary again) confirms Aggressive through its own policy tree
/// and the home's transition store, then exits; the running tree of this
/// process rises to Aggressive within about a second, and its sessions take
/// approval for every command.
#[test]
#[serial_test::serial(aggressive_grants)]
fn issue_919_aggressive_confirmed_in_a_second_process_reaches_this_one() {
    const CHILD_ENV: &str = "CODEX_CORE_ISSUE_919_CHILD_HOME";
    const TEST: &str = "security::effective_policy::transition::tests::issue_919_aggressive_confirmed_in_a_second_process_reaches_this_one";
    if let Some(home) = std::env::var_os(CHILD_ENV) {
        // The other Corbanu Terminal: a person confirms Aggressive there.
        let home = std::path::PathBuf::from(home);
        let there = fixture(SecurityLevel::Permissive);
        there.view.register_home(&home);
        let committed = there
            .commit_with(
                there.set_level(SecurityLevel::Aggressive),
                &crate::security::recovery::HomeTransitionStore::new(&home),
            )
            .unwrap();
        assert_eq!(committed.level, SecurityLevel::Aggressive);
        assert_eq!(committed.not_saved, None);
        return;
    }
    let home = tempfile::TempDir::new().unwrap();
    let here = fixture(SecurityLevel::Permissive);
    here.view.register_home(home.path());
    let mut raised = kill_switch_watch::subscribe_level_raised(home.path());
    assert_eq!(*raised.borrow_and_update(), None);
    let (_, _, epoch, _, _) = here.now();

    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, home.path())
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .output()
        .expect("run the other process");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        within(std::time::Duration::from_secs(5), || {
            raised.has_changed().unwrap_or(false)
        }),
        "Aggressive from the other process did not arrive"
    );
    assert_eq!(*raised.borrow_and_update(), Some(SecurityLevel::Aggressive));
    let (level, next_start, after, _, _) = here.now();
    assert_eq!(
        (level, next_start),
        (SecurityLevel::Aggressive, SecurityLevel::Aggressive)
    );
    assert!(after > epoch, "grants and approvals did not end");
    assert_eq!(here.revocations(), 1);
    assert!(crate::security::confirmed_approvals::confirmed_here(
        home.path()
    ));
}
