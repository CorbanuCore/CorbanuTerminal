#![allow(clippy::expect_used)]

use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

use codex_security_policy::ActionReceipt;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::MandateOutcome;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedActionMandate;
use codex_security_policy::ProtectedActionPreview;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_security_policy::permissive_decision;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use crate::AuditGapReason;
use crate::AuthorityIdentity;
use crate::DispatchResolution;
use crate::EventContext;
use crate::IntegrityCheckpoint;
use crate::IntegrityRootError;
use crate::IntegrityRootStore;
use crate::JournalConfig;
use crate::JournalError;
use crate::JournalOwner;
use crate::RecoveryBlocker;
use crate::RecoveryState;
use crate::ReferenceJournal;
use crate::SecurityEvent;
use crate::SecurityEventId;
use crate::apply_emergency_restriction;
use crate::journal::FaultPoint;
use crate::journal::InjectedFault;

#[derive(Debug, Default)]
struct MemoryRoot {
    checkpoint: Mutex<Option<IntegrityCheckpoint>>,
    load_error: Mutex<Option<IntegrityRootError>>,
    store_error: Mutex<Option<IntegrityRootError>>,
}

impl MemoryRoot {
    fn force_checkpoint(&self, checkpoint: Option<IntegrityCheckpoint>) {
        *self.checkpoint.lock().expect("checkpoint mutex") = checkpoint;
    }

    fn fail_load(&self, error: IntegrityRootError) {
        *self.load_error.lock().expect("load error mutex") = Some(error);
    }

    fn fail_store(&self, error: IntegrityRootError) {
        *self.store_error.lock().expect("store error mutex") = Some(error);
    }
}

impl IntegrityRootStore for MemoryRoot {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        if let Some(error) = self.load_error.lock().expect("load error mutex").take() {
            return Err(error);
        }
        Ok(self.checkpoint.lock().expect("checkpoint mutex").clone())
    }

    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        if let Some(error) = self.store_error.lock().expect("store error mutex").take() {
            return Err(error);
        }
        let mut checkpoint = self.checkpoint.lock().expect("checkpoint mutex");
        if checkpoint.as_ref() != expected {
            return Err(IntegrityRootError::Conflict);
        }
        *checkpoint = Some(next.clone());
        Ok(())
    }
}

struct Fixture {
    _temp: TempDir,
    root_path: AbsolutePathBuf,
    roots: Arc<MemoryRoot>,
    journal: ReferenceJournal,
    producer: PolicyPrincipal,
}

impl Fixture {
    fn new(config: JournalConfig) -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let root_path = AbsolutePathBuf::from_absolute_path_checked(temp.path().join("journal"))
            .expect("absolute journal path");
        let producer = principal(PrincipalKind::Service, "audit-producer");
        let owner =
            JournalOwner::new(producer.clone(), 1, text("integrity-key-1")).expect("journal owner");
        let roots = Arc::new(MemoryRoot::default());
        let mut journal = ReferenceJournal::new(root_path.clone(), owner, roots.clone(), config);
        assert_eq!(
            journal.recover(1, &RevocationState::new()).state,
            RecoveryState::Empty
        );
        Self {
            _temp: temp,
            root_path,
            roots,
            journal,
            producer,
        }
    }

    fn context(&self) -> EventContext {
        EventContext::new(self.producer.clone(), 1, 1).expect("event context")
    }

    fn context_at(&self, policy_generation: u64, run_generation: u64) -> EventContext {
        EventContext::new(self.producer.clone(), policy_generation, run_generation)
            .expect("event context")
    }

    fn restarted_journal(&self) -> ReferenceJournal {
        let owner = JournalOwner::new(self.producer.clone(), 1, text("integrity-key-1"))
            .expect("journal owner");
        ReferenceJournal::new(
            self.root_path.clone(),
            owner,
            self.roots.clone(),
            JournalConfig::default(),
        )
    }

    fn append_decision(&mut self) {
        let request = request();
        let decision = permissive_decision(&request).expect("decision");
        let event = SecurityEvent::decision(self.context(), None, &request, decision, 11)
            .expect("decision event");
        self.journal
            .record_decision(event)
            .expect("record decision");
    }
}

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("bounded text")
}

fn principal(kind: PrincipalKind, id: &str) -> PolicyPrincipal {
    PolicyPrincipal::new(kind, id).expect("principal")
}

fn request() -> AuthorizationRequest {
    AuthorizationRequest::new(
        ActorChain::new(vec![
            principal(PrincipalKind::Human, "human-1"),
            principal(PrincipalKind::Agent, "agent-1"),
        ])
        .expect("actor chain"),
        ProtectedResource::new(ResourceKind::FinancialAction, "account-1").expect("resource"),
        PolicyAction::Sign,
        AuthorizationContext {
            now_unix_seconds: 10,
            session_id: text("session-1"),
            task_id: text("task-1"),
            purpose: text("rebalance"),
            operation: text("sign"),
            destination: None,
            quantity: None,
            grant_id: None,
        },
    )
    .expect("request")
}

fn mandate(request: &AuthorizationRequest) -> (ProtectedActionMandate, ProtectedActionPreview) {
    let preview =
        ProtectedActionPreview::new(request.clone(), 100, text("nonce-1")).expect("preview");
    let mandate =
        ProtectedActionMandate::approve(&preview, principal(PrincipalKind::Human, "human-1"), 11)
            .expect("mandate");
    (mandate, preview)
}

fn kill_event() -> RevocationEvent {
    RevocationEvent::new(
        principal(PrincipalKind::Human, "human-1"),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::KillSwitch,
        20,
    )
    .expect("kill event")
}

fn healthy_recovery(fixture: &mut Fixture) {
    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(report.state, RecoveryState::Ready);
}

#[test]
fn duplicate_event_returns_the_original_durable_acknowledgement() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(fixture.context(), None, &request, decision, 11)
        .expect("decision event");

    let first = fixture
        .journal
        .record_decision(event.clone())
        .expect("first append");
    let duplicate = fixture
        .journal
        .record_decision(event)
        .expect("duplicate append");

    assert_eq!(first.sequence, duplicate.sequence);
    assert!(!first.duplicate);
    assert!(duplicate.duplicate);
    healthy_recovery(&mut fixture);
}

#[test]
fn mandate_intent_precedes_completed_receipt() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let request = request();
    let (mandate, preview) = mandate(&request);
    let authority = AuthorityIdentity::from_mandate(&mandate).expect("authority");
    let (permit, intent_ack) = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request,
            authority,
            text("dispatch-1"),
            12,
        )
        .expect("durable intent");
    let receipt =
        ActionReceipt::complete(&mandate, &preview, MandateOutcome::Executed, 13).expect("receipt");
    let completion = fixture
        .journal
        .resolve_dispatch(
            permit,
            fixture.context(),
            DispatchResolution::Completed {
                outcome: MandateOutcome::Executed,
                mandate_receipt: Some(receipt),
            },
            13,
        )
        .expect("durable completion");

    assert_eq!(intent_ack.sequence, 1);
    assert_eq!(completion.sequence, 2);
    healthy_recovery(&mut fixture);
}

#[test]
fn unknown_receipt_is_terminal_and_never_auto_replayed() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let request = request();
    let authority = AuthorityIdentity::Grant {
        grant_id: text("grant-1"),
    };
    let (permit, _) = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request,
            authority,
            text("dispatch-1"),
            12,
        )
        .expect("durable intent");
    fixture
        .journal
        .resolve_dispatch(
            permit,
            fixture.context(),
            DispatchResolution::Unknown {
                reason: crate::UnknownOutcomeReason::SettlementUncertain,
            },
            13,
        )
        .expect("unknown receipt");

    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request,
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("terminal reservation must never issue another permit");
    assert!(matches!(error, JournalError::AlreadyResolved));
}

#[test]
fn producer_mismatch_and_generation_rollback_fail_closed() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let foreign = EventContext::new(principal(PrincipalKind::Service, "other-producer"), 1, 1)
        .expect("foreign context");
    let foreign_event = SecurityEvent::decision(foreign, None, &request, decision.clone(), 11)
        .expect("foreign event");
    assert!(matches!(
        fixture.journal.record_decision(foreign_event),
        Err(JournalError::ProducerMismatch)
    ));

    fixture.append_decision();
    let rolled_back =
        EventContext::new(fixture.producer.clone(), 0, 1).expect("rolled-back context");
    let rollback_event =
        SecurityEvent::decision(rolled_back, None, &request, decision, 12).expect("rollback event");
    assert!(matches!(
        fixture.journal.record_decision(rollback_event),
        Err(JournalError::InvalidEventSequence)
    ));
}

#[test]
fn disk_full_blocks_dispatch_before_a_permit_exists() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture
        .journal
        .inject_once(FaultPoint::BeforeRecordWrite, InjectedFault::DiskFull);
    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("disk full must not issue permit");
    assert!(matches!(error, JournalError::StorageUnavailable));
    assert!(matches!(
        fixture.journal.reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        ),
        Err(JournalError::RecoveryRequired)
    ));
    healthy_recovery(&mut fixture);
}

#[test]
fn expired_append_deadline_blocks_dispatch() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture
        .journal
        .inject_once(FaultPoint::BeforeRecordWrite, InjectedFault::Timeout);
    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("timeout must not issue permit");
    assert!(matches!(error, JournalError::DeadlineExceeded));
}

#[test]
fn crash_before_rename_is_cleaned_without_replay() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture
        .journal
        .inject_once(FaultPoint::AfterRecordSync, InjectedFault::Crash);
    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("crash must be uncertain");
    assert!(matches!(error, JournalError::CommitUnknown { .. }));
    healthy_recovery(&mut fixture);
}

#[test]
fn ambiguous_commit_after_rename_remains_fail_closed() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture
        .journal
        .inject_once(FaultPoint::AfterRecordRename, InjectedFault::Crash);
    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("ambiguous commit must fail");
    assert!(matches!(error, JournalError::CommitUnknown { .. }));
    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::RecordsAheadOfIntegrityRoot)
    );
}

#[test]
fn operator_can_reconcile_exactly_one_ambiguous_commit_without_replay() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture
        .journal
        .inject_once(FaultPoint::AfterRecordRename, InjectedFault::Crash);
    let event_id = match fixture.journal.reserve_dispatch(
        fixture.context(),
        None,
        &request(),
        AuthorityIdentity::Grant {
            grant_id: text("grant-1"),
        },
        text("dispatch-1"),
        12,
    ) {
        Err(JournalError::CommitUnknown { event_id }) => event_id,
        other => panic!("expected ambiguous commit, got {other:?}"),
    };
    let wrong_id = serde_json::from_str::<SecurityEventId>(&format!("\"{}\"", "0".repeat(64)))
        .expect("synthetic digest");
    let mismatch = fixture
        .journal
        .reconcile_ambiguous_commit(&wrong_id, 1, &RevocationState::new())
        .expect_err("operator must identify the exact failed event");
    assert!(matches!(mismatch, JournalError::AmbiguousCommitMismatch));

    let checkpoint = fixture
        .journal
        .reconcile_ambiguous_commit(&event_id, 1, &RevocationState::new())
        .expect("operator-selected record can advance the protected root");
    assert_eq!(checkpoint.sequence, 2);
    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(report.state, RecoveryState::ReconciliationRequired);
    assert_eq!(report.pending_dispatches.len(), 1);
    fixture
        .journal
        .reconcile_dispatch_as_unknown(
            &report.pending_dispatches[0],
            fixture.context(),
            crate::UnknownOutcomeReason::PersistenceUncertain,
            13,
        )
        .expect("external effect remains unknown and is never replayed");
    healthy_recovery(&mut fixture);
}

#[test]
fn lost_ack_retry_never_reissues_a_dispatch_permit() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture.journal.inject_once(
        FaultPoint::AfterRootCommit,
        InjectedFault::AcknowledgementLost,
    );
    let first = fixture.journal.reserve_dispatch(
        fixture.context(),
        None,
        &request(),
        AuthorityIdentity::Grant {
            grant_id: text("grant-1"),
        },
        text("dispatch-1"),
        12,
    );
    assert!(matches!(
        first,
        Err(JournalError::AcknowledgementLost { .. })
    ));

    let retry = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            13,
        )
        .expect_err("retry must reconcile instead of replaying");
    assert!(matches!(retry, JournalError::AlreadyReserved));
}

#[test]
fn duplicate_dispatch_is_generation_independent() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let request = request();
    let authority = AuthorityIdentity::Grant {
        grant_id: text("grant-1"),
    };
    let (permit, _) = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request,
            authority.clone(),
            text("dispatch-1"),
            12,
        )
        .expect("first permit");

    let forward_policy = fixture
        .journal
        .reserve_dispatch(
            fixture.context_at(2, 1),
            None,
            &request,
            authority.clone(),
            text("dispatch-1"),
            13,
        )
        .expect_err("policy reload must not issue a second permit");
    assert!(matches!(forward_policy, JournalError::AlreadyReserved));

    fixture
        .journal
        .resolve_dispatch(
            permit,
            fixture.context_at(2, 2),
            DispatchResolution::Unknown {
                reason: crate::UnknownOutcomeReason::SettlementUncertain,
            },
            14,
        )
        .expect("terminal receipt");
    let forward_run = fixture
        .journal
        .reserve_dispatch(
            fixture.context_at(2, 2),
            None,
            &request,
            authority,
            text("dispatch-1"),
            15,
        )
        .expect_err("run reload must not issue a terminal duplicate");
    assert!(matches!(forward_run, JournalError::AlreadyResolved));
}

#[test]
fn validated_tail_cache_avoids_rescanning_until_recovery() {
    let mut fixture = Fixture::new(JournalConfig::default());
    assert_eq!(fixture.journal.scan_count(), 1);
    fixture.append_decision();
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(fixture.context(), None, &request, decision, 12)
        .expect("second decision");
    fixture
        .journal
        .record_decision(event)
        .expect("cached append");
    assert_eq!(fixture.journal.scan_count(), 1);

    assert_eq!(
        fixture.journal.recover(1, &RevocationState::new()).state,
        RecoveryState::Ready
    );
    assert_eq!(fixture.journal.scan_count(), 2);
}

#[test]
fn protected_root_change_invalidates_cached_tail() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture.roots.force_checkpoint(None);
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(fixture.context(), None, &request, decision, 12)
        .expect("second decision");
    let error = fixture
        .journal
        .record_decision(event)
        .expect_err("changed protected root must invalidate cache");
    assert!(matches!(error, JournalError::RecoveryRequired));
}

#[test]
fn unavailable_integrity_root_creates_an_ambiguous_commit() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture.roots.fail_store(IntegrityRootError::Unavailable);
    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("root failure must be ambiguous");
    assert!(matches!(error, JournalError::CommitUnknown { .. }));
}

#[test]
fn missing_integrity_key_blocks_recovery() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture.roots.fail_load(IntegrityRootError::MissingKey);
    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::MissingIntegrityKey)
    );
}

#[test]
fn truncation_is_detected_against_the_controller_root() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    let request = request();
    fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request,
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect("intent");
    fs::remove_file(
        fixture
            .root_path
            .join("segment-00000000000000000001")
            .join("record-00000000000000000002.json")
            .as_path(),
    )
    .expect("truncate record");

    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::TruncatedJournal)
    );
}

#[test]
fn rollback_or_record_mutation_is_detected() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    let record = fixture
        .root_path
        .join("segment-00000000000000000001")
        .join("record-00000000000000000001.json");
    let mut bytes = fs::read(record.as_path()).expect("read record");
    let offset = bytes
        .iter()
        .position(|byte| *byte == b'1')
        .expect("digit in record");
    bytes[offset] = b'2';
    fs::write(record.as_path(), bytes).expect("mutate record");

    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::InvalidRecord)
    );
}

#[test]
fn segment_rotation_preserves_chain_and_recovery() {
    let mut fixture = Fixture::new(JournalConfig::bounded(4, 1).expect("config"));
    fixture.append_decision();
    fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect("intent");
    assert!(
        fixture
            .root_path
            .join("segment-00000000000000000002")
            .as_path()
            .is_dir()
    );
    let report = fixture.journal.recover(1, &RevocationState::new());
    assert_eq!(report.state, RecoveryState::ReconciliationRequired);
    assert_eq!(report.pending_dispatches.len(), 1);
}

#[test]
fn queue_saturation_blocks_new_dispatch() {
    let mut fixture = Fixture::new(JournalConfig::bounded(1, 1).expect("config"));
    fixture.append_decision();
    let error = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect_err("saturated journal must block");
    assert!(matches!(error, JournalError::CapacityExceeded));
}

#[test]
fn concurrent_writer_lock_blocks_append() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fs::create_dir_all(fixture.root_path.as_path()).expect("journal root");
    let mut lock = fslock::LockFile::open(fixture.root_path.join(".writer.lock").as_path())
        .expect("writer lock");
    assert!(lock.try_lock().expect("acquire writer lock"));
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(fixture.context(), None, &request, decision, 11)
        .expect("decision event");

    let error = fixture
        .journal
        .record_decision(event)
        .expect_err("concurrent writer must block");
    assert!(matches!(error, JournalError::ConcurrentWriter));
}

#[test]
fn emergency_restriction_fences_immediately_and_exposes_audit_gap() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fixture
        .journal
        .inject_once(FaultPoint::BeforeRecordWrite, InjectedFault::DiskFull);
    let mut state = RevocationState::new();
    let context = fixture.context();
    let result = apply_emergency_restriction(
        &mut state,
        &kill_event(),
        &mut fixture.journal,
        context,
        None,
    )
    .expect("restriction applies");

    assert!(state.kill_switch_active);
    assert_eq!(result.gap, Some(AuditGapReason::StorageUnavailable));
    assert_eq!(
        result.application.audit_status,
        codex_security_policy::RestrictionAuditStatus::Unavailable
    );
    let report = fixture.journal.recover(1, &state);
    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::RestrictionAuditGap)
    );
}

#[test]
fn recorded_restriction_recovers_with_the_controller_state() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    let mut state = RevocationState::new();
    let context = fixture.context();
    let result = apply_emergency_restriction(
        &mut state,
        &kill_event(),
        &mut fixture.journal,
        context,
        None,
    )
    .expect("restriction applies");

    assert_eq!(result.gap, None);
    assert!(result.audit_event_id.is_some());
    let report = fixture.journal.recover(1, &state);
    assert_eq!(report.state, RecoveryState::Ready);
}

#[test]
fn owner_rotation_cannot_reuse_an_old_integrity_root() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    let old_root = fixture
        .roots
        .checkpoint
        .lock()
        .expect("checkpoint mutex")
        .clone();
    fixture.roots.force_checkpoint(old_root);
    let owner = JournalOwner::new(fixture.producer.clone(), 2, text("integrity-key-2"))
        .expect("rotated owner");
    let mut rotated = ReferenceJournal::new(
        fixture.root_path.clone(),
        owner,
        fixture.roots.clone(),
        JournalConfig::default(),
    );
    let report = rotated.recover(1, &RevocationState::new());
    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::OwnerMismatch)
    );
}

#[test]
fn recovery_never_deletes_an_unrecognized_temporary_file() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    let unexpected = fixture
        .root_path
        .join("segment-00000000000000000001")
        .join("operator-notes.tmp");
    fs::write(unexpected.as_path(), b"not a journal temporary record").expect("write marker");

    let report = fixture.journal.recover(1, &RevocationState::new());

    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::InvalidRecord)
    );
    assert!(unexpected.as_path().exists());
}

#[test]
fn malformed_segment_names_fail_recovery_closed() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    fs::create_dir(fixture.root_path.join("segment-next").as_path()).expect("malformed segment");

    let report = fixture.journal.recover(1, &RevocationState::new());

    assert_eq!(
        report.state,
        RecoveryState::Blocked(RecoveryBlocker::InvalidRecord)
    );
}

#[test]
fn fresh_journal_requires_recovery_before_any_append() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture.append_decision();
    let mut restarted = fixture.restarted_journal();
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(fixture.context(), None, &request, decision, 12)
        .expect("decision event");

    let error = restarted
        .record_decision(event)
        .expect_err("unrecovered journal must fail closed");

    assert!(matches!(error, JournalError::RecoveryRequired));
}

#[test]
fn recovery_accepts_first_install_and_forward_policy_generation() {
    let mut fixture = Fixture::new(JournalConfig::default());
    assert_eq!(
        fixture.journal.recover(7, &RevocationState::new()).state,
        RecoveryState::Empty
    );
    let first_request = request();
    let decision = permissive_decision(&first_request).expect("decision");
    let event =
        SecurityEvent::decision(fixture.context_at(7, 1), None, &first_request, decision, 12)
            .expect("generation-seven decision");
    fixture
        .journal
        .record_decision(event)
        .expect("first-install generation append");
    assert_eq!(
        fixture.journal.recover(8, &RevocationState::new()).state,
        RecoveryState::Ready
    );
    let second_request = request();
    let decision = permissive_decision(&second_request).expect("decision");
    let event = SecurityEvent::decision(
        fixture.context_at(8, 1),
        None,
        &second_request,
        decision,
        13,
    )
    .expect("generation-eight decision");
    fixture
        .journal
        .record_decision(event)
        .expect("forward generation append");

    let rollback = fixture.journal.recover(7, &RevocationState::new());
    assert_eq!(
        rollback.state,
        RecoveryState::Blocked(RecoveryBlocker::PolicyGenerationMismatch)
    );
}

#[test]
fn recovered_pending_intent_is_visible_and_reconciled_unknown() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let (_, intent) = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect("durable intent");
    assert_eq!(intent.sequence, 1);

    let mut restarted = fixture.restarted_journal();
    let report = restarted.recover(1, &RevocationState::new());
    assert_eq!(report.state, RecoveryState::ReconciliationRequired);
    assert_eq!(report.pending_dispatches.len(), 1);
    assert!(!report.permits_protected_dispatch());
    let reserve_error = restarted
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-2"),
            },
            text("dispatch-2"),
            13,
        )
        .expect_err("pending recovery must block new dispatch");
    assert!(matches!(
        reserve_error,
        JournalError::ReconciliationRequired
    ));

    restarted
        .reconcile_dispatch_as_unknown(
            &report.pending_dispatches[0],
            fixture.context(),
            crate::UnknownOutcomeReason::PersistenceUncertain,
            13,
        )
        .expect("unknown reconciliation");
    let ready = restarted.recover(1, &RevocationState::new());
    assert!(ready.pending_dispatches.is_empty());
    assert!(ready.permits_protected_dispatch());
}

#[test]
fn resolution_uses_live_generation_after_policy_advances() {
    let mut fixture = Fixture::new(JournalConfig::default());
    let (permit, _) = fixture
        .journal
        .reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        )
        .expect("durable intent");
    let request = request();
    let decision = permissive_decision(&request).expect("decision");
    let generation_two = fixture.context_at(2, 1);
    let event = SecurityEvent::decision(generation_two.clone(), None, &request, decision, 13)
        .expect("generation-two decision");
    fixture
        .journal
        .record_decision(event)
        .expect("advance generation");

    fixture
        .journal
        .resolve_dispatch(
            permit,
            generation_two,
            DispatchResolution::Unknown {
                reason: crate::UnknownOutcomeReason::TransportLost,
            },
            14,
        )
        .expect("terminal receipt at current generation");
    assert_eq!(
        fixture.journal.recover(2, &RevocationState::new()).state,
        RecoveryState::Ready
    );
}

#[test]
fn directory_sync_failure_after_publish_is_ambiguous_and_blocks() {
    let mut fixture = Fixture::new(JournalConfig::default());
    fixture
        .journal
        .inject_once(FaultPoint::BeforeDirectorySync, InjectedFault::Crash);
    let decision_request = request();
    let decision = permissive_decision(&decision_request).expect("decision");
    let event = SecurityEvent::decision(fixture.context(), None, &decision_request, decision, 11)
        .expect("decision event");

    let error = fixture
        .journal
        .record_decision(event)
        .expect_err("published record without directory sync is ambiguous");
    assert!(matches!(error, JournalError::CommitUnknown { .. }));
    assert!(matches!(
        fixture.journal.reserve_dispatch(
            fixture.context(),
            None,
            &request(),
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("dispatch-1"),
            12,
        ),
        Err(JournalError::RecoveryRequired)
    ));
}
