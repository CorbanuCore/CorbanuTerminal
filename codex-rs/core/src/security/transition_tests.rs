use std::io;
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
    fail: bool,
}

impl MemoryStore {
    fn failing() -> Self {
        Self {
            fail: true,
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
    fn persist(&self, state: &DurableSecurityState) -> io::Result<()> {
        if self.fail {
            return Err(io::Error::other("disk full"));
        }
        self.saved.lock().unwrap().push(state.clone());
        Ok(())
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

#[test]
fn security_transition_persistence_failure_changes_nothing() {
    let fixture = fixture(SecurityLevel::Permissive);
    let before = fixture.now();
    let prepared = fixture.set_level(SecurityLevel::Aggressive);
    let error = fixture
        .commit(prepared, &MemoryStore::failing())
        .unwrap_err();
    assert!(matches!(error, TransitionError::Persist(_)), "{error}");
    assert_eq!(fixture.now(), before);
    assert_eq!(fixture.revocations(), 0);
    // A fresh confirmation still works afterwards.
    let store = MemoryStore::default();
    fixture
        .commit(fixture.set_level(SecurityLevel::Aggressive), &store)
        .unwrap();
    assert_eq!(store.saved(), vec![(SecurityLevel::Aggressive, 1, false)]);
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
    // Neither closes the broker's channels.
    assert_eq!(fixture.revocations(), 0);
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
        .commit(
            fixture
                .prepare(
                    revoke(RevocationTarget::KillSwitch { active: true }),
                    ProbeOutcome::Passed,
                )
                .unwrap(),
            &store,
        )
        .unwrap();
    assert_eq!(
        fixture.now(),
        (SecurityLevel::Moderate, SecurityLevel::Moderate, 1, 1, true)
    );
    assert_eq!(fixture.revocations(), 1);
    let release = fixture
        .prepare(
            revoke(RevocationTarget::KillSwitch { active: false }),
            ProbeOutcome::Passed,
        )
        .unwrap();
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
        vec![
            (SecurityLevel::Moderate, 1, true),
            (SecurityLevel::Moderate, 2, false),
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
        .commit(
            fixture
                .prepare(revoke(target), ProbeOutcome::Passed)
                .unwrap(),
            &MemoryStore::default(),
        )
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
            fixture
                .prepare(
                    revoke(RevocationTarget::AllActiveAuthority),
                    ProbeOutcome::Passed,
                )
                .unwrap(),
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
