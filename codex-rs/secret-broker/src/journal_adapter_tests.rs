use super::*;
use codex_security_audit::IntegrityCheckpoint;
use codex_security_audit::IntegrityRootError;
use codex_security_audit::IntegrityRootStore;
use codex_security_audit::JournalConfig;
use codex_security_audit::JournalOwner;
use codex_security_audit::RecoveryState;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::RevocationState;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Debug, Default)]
struct TestRoot {
    checkpoint: Mutex<Option<IntegrityCheckpoint>>,
    fail: AtomicBool,
}

impl IntegrityRootStore for TestRoot {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        Ok(self.checkpoint.lock().unwrap().clone())
    }

    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(IntegrityRootError::Timeout);
        }
        let mut checkpoint = self.checkpoint.lock().unwrap();
        if checkpoint.as_ref() != expected {
            return Err(IntegrityRootError::Conflict);
        }
        *checkpoint = Some(next.clone());
        Ok(())
    }
}

struct FixedClock;
impl BrokerJournalClock for FixedClock {
    fn now_unix_seconds(&self) -> Result<i64, BrokerAuditError> {
        Ok(20)
    }
}

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).unwrap()
}

fn principal(kind: PrincipalKind, id: &str) -> PolicyPrincipal {
    PolicyPrincipal::new(kind, id).unwrap()
}

fn binding() -> BrokerJournalBinding {
    BrokerJournalBinding {
        binding: BrokerBinding {
            controller_instance: "controller-1".into(),
            worker_instance: "worker-1".into(),
            session_id: "session-1".into(),
            task_id: "task-1".into(),
            run_id: "run-1".into(),
            run_generation: 1,
        },
        credential: CredentialReference::from_sha256_hex("a".repeat(64)).unwrap(),
        request: AuthorizationRequest::new(
            ActorChain::new(vec![
                principal(PrincipalKind::Human, "human-1"),
                principal(PrincipalKind::Agent, "agent-1"),
            ])
            .unwrap(),
            ProtectedResource::new(ResourceKind::VaultCredential, "synthetic-key").unwrap(),
            PolicyAction::Use,
            AuthorizationContext {
                now_unix_seconds: 10,
                session_id: text("session-1"),
                task_id: text("task-1"),
                purpose: text("private-purpose-canary"),
                operation: text("openai.responses.create"),
                destination: Some(text("https://api.openai.com:443")),
                quantity: None,
                grant_id: Some(text("grant-1")),
            },
        )
        .unwrap(),
        authority: AuthorityIdentity::Grant {
            grant_id: text("grant-1"),
        },
        operation: OpenAiResponsesOperation::new("/v1/responses-private-path-canary").unwrap(),
    }
}

fn intent() -> BrokerAuditIntent {
    let bound = binding();
    BrokerAuditIntent {
        controller_instance: bound.binding.controller_instance,
        session_id: bound.binding.session_id,
        task_id: bound.binding.task_id,
        run_id: bound.binding.run_id,
        run_generation: 1,
        sequence: 1,
        credential_reference: bound.credential,
        operation: "openai.responses.create",
        destination: "https://api.openai.com:443",
        path: bound.operation.path().into(),
    }
}

struct Fixture {
    temp: tempfile::TempDir,
    root: Arc<TestRoot>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            temp: tempfile::tempdir().unwrap(),
            root: Arc::new(TestRoot::default()),
        }
    }

    fn adapter(&self, recover: bool) -> JournalBrokerAudit<FixedClock> {
        let producer = principal(PrincipalKind::Service, "broker-1");
        let mut journal = ReferenceJournal::new(
            AbsolutePathBuf::from_absolute_path_checked(self.temp.path().join("journal")).unwrap(),
            JournalOwner::new(producer.clone(), 1, text("test-root")).unwrap(),
            self.root.clone(),
            JournalConfig::default(),
        );
        if recover {
            let _ = journal.recover(1, 1, &RevocationState::new());
        }
        JournalBrokerAudit::new(
            journal,
            binding(),
            EventContext::new(producer, 1, 1).unwrap(),
            FixedClock,
        )
        .unwrap()
    }
}

#[test]
fn pf_27_s01_audit_commits_before_permit_and_minimizes_exact_semantics() {
    let fixture = Fixture::new();
    let audit = fixture.adapter(/*recover*/ true);
    let permit = audit.reserve(&intent()).unwrap();
    assert_eq!(
        fixture
            .root
            .checkpoint
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sequence,
        1
    );
    let record = std::fs::read_to_string(
        fixture
            .temp
            .path()
            .join("journal/segment-00000000000000000001/record-00000000000000000001.json"),
    )
    .unwrap();
    for private in [
        "private-purpose-canary",
        "private-path-canary",
        "https://api.openai.com",
    ] {
        assert!(!record.contains(private));
    }
    audit
        .resolve(permit, BrokerAuditResolution::Completed)
        .unwrap();
    assert_eq!(
        fixture
            .root
            .checkpoint
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .sequence,
        2
    );
    drop(audit);
    assert!(
        fixture
            .adapter(/*recover*/ true)
            .reserve(&intent())
            .is_err()
    );
}

#[test]
fn pf_27_s01_audit_rejects_binding_changes_before_journal_write() {
    let fixture = Fixture::new();
    let audit = fixture.adapter(/*recover*/ true);
    let mut wrong = intent();
    wrong.path = "/v1/other".into();
    assert!(audit.reserve(&wrong).is_err());
    wrong = intent();
    wrong.run_generation += 1;
    assert!(audit.reserve(&wrong).is_err());
    wrong = intent();
    wrong.credential_reference = CredentialReference::from_sha256_hex("b".repeat(64)).unwrap();
    assert!(audit.reserve(&wrong).is_err());
    assert_eq!(*fixture.root.checkpoint.lock().unwrap(), None);
}

#[test]
fn pf_27_s01_audit_unrecovered_or_ambiguous_root_returns_no_permit() {
    let fixture = Fixture::new();
    assert!(
        fixture
            .adapter(/*recover*/ false)
            .reserve(&intent())
            .is_err()
    );
    let audit = fixture.adapter(/*recover*/ true);
    fixture.root.fail.store(true, Ordering::SeqCst);
    assert_eq!(
        audit.reserve(&intent()).err(),
        Some(BrokerAuditError::CommitUnknown)
    );
    fixture.root.fail.store(false, Ordering::SeqCst);
    assert!(audit.reserve(&intent()).is_err());
}

#[test]
fn pf_27_s01_audit_crash_pending_intent_blocks_restart_replay() {
    let fixture = Fixture::new();
    let audit = fixture.adapter(/*recover*/ true);
    let _pending_permit = audit.reserve(&intent()).unwrap();
    drop(audit);
    let restarted = fixture.adapter(/*recover*/ true);
    assert!(restarted.reserve(&intent()).is_err());
    assert_ne!(
        restarted
            .journal
            .lock()
            .unwrap()
            .recover(1, 1, &RevocationState::new())
            .state,
        RecoveryState::Ready,
    );
}

#[test]
fn pf_27_s01_audit_all_terminal_outcomes_are_durable() {
    for outcome in [
        BrokerAuditResolution::Failed,
        BrokerAuditResolution::Cancelled,
        BrokerAuditResolution::Unknown,
    ] {
        let fixture = Fixture::new();
        let audit = fixture.adapter(/*recover*/ true);
        audit
            .resolve(audit.reserve(&intent()).unwrap(), outcome)
            .unwrap();
        assert_eq!(
            fixture
                .root
                .checkpoint
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .sequence,
            2
        );
    }
}

#[test]
fn pf_27_s01_audit_rejects_unsettleable_or_mismatched_authority() {
    for invalid in 0..3 {
        let fixture = Fixture::new();
        let audit = fixture.adapter(/*recover*/ true);
        let mut bound = binding();
        match invalid {
            0 => {
                bound.authority = AuthorityIdentity::Mandate {
                    mandate_id: text("mandate-1"),
                }
            }
            1 => {
                bound.authority = AuthorityIdentity::Grant {
                    grant_id: text("other-grant"),
                }
            }
            2 => bound.request.context.grant_id = None,
            _ => unreachable!(),
        }
        assert!(
            JournalBrokerAudit::new(
                audit.journal.into_inner().unwrap(),
                bound,
                audit.context,
                FixedClock,
            )
            .is_err()
        );
        assert_eq!(*fixture.root.checkpoint.lock().unwrap(), None);
    }
}
