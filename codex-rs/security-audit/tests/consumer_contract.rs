use std::sync::Arc;
use std::sync::Mutex;

use codex_security_audit::AuthorityIdentity;
use codex_security_audit::DispatchResolution;
use codex_security_audit::EventContext;
use codex_security_audit::IntegrityCheckpoint;
use codex_security_audit::IntegrityRootError;
use codex_security_audit::IntegrityRootStore;
use codex_security_audit::JournalConfig;
use codex_security_audit::JournalOwner;
use codex_security_audit::RecoveryState;
use codex_security_audit::ReferenceJournal;
use codex_security_audit::UnknownOutcomeReason;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::RevocationState;
use codex_utils_absolute_path::AbsolutePathBuf;

#[derive(Debug, Default)]
struct ControllerRoot(Mutex<Option<IntegrityCheckpoint>>);

impl IntegrityRootStore for ControllerRoot {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        Ok(self.0.lock().expect("controller root").clone())
    }

    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        let mut root = self.0.lock().expect("controller root");
        if root.as_ref() != expected {
            return Err(IntegrityRootError::Conflict);
        }
        *root = Some(next.clone());
        Ok(())
    }
}

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("bounded text")
}

fn principal(kind: PrincipalKind, id: &str) -> PolicyPrincipal {
    PolicyPrincipal::new(kind, id).expect("principal")
}

#[test]
fn public_consumer_contract_requires_intent_then_terminal_receipt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root_path = AbsolutePathBuf::from_absolute_path_checked(temp.path().join("journal"))
        .expect("absolute journal path");
    let producer = principal(PrincipalKind::Service, "producer-1");
    let owner = JournalOwner::new(producer.clone(), 1, text("pf20-key-1")).expect("journal owner");
    let mut journal = ReferenceJournal::new(
        root_path,
        owner,
        Arc::new(ControllerRoot::default()),
        JournalConfig::default(),
    );
    assert_eq!(
        journal.recover(0, &RevocationState::new()).state,
        RecoveryState::Empty
    );
    let request = AuthorizationRequest::new(
        ActorChain::new(vec![
            principal(PrincipalKind::Human, "human-1"),
            principal(PrincipalKind::Agent, "agent-1"),
        ])
        .expect("actor chain"),
        ProtectedResource::new(ResourceKind::Tool, "broker-action").expect("resource"),
        PolicyAction::Execute,
        AuthorizationContext {
            now_unix_seconds: 10,
            session_id: text("session-1"),
            task_id: text("task-1"),
            purpose: text("protected-action"),
            operation: text("dispatch"),
            destination: None,
            quantity: None,
            grant_id: None,
        },
    )
    .expect("request");
    let context = EventContext::new(producer, 0, 1).expect("event context");
    let (permit, intent) = journal
        .reserve_dispatch(
            context.clone(),
            None,
            &request,
            AuthorityIdentity::Grant {
                grant_id: text("grant-1"),
            },
            text("attempt-1"),
            11,
        )
        .expect("durable intent");
    let receipt = journal
        .resolve_dispatch(
            permit,
            context,
            DispatchResolution::Unknown {
                reason: UnknownOutcomeReason::TransportLost,
            },
            12,
        )
        .expect("terminal unknown receipt");

    assert_eq!(intent.sequence, 1);
    assert_eq!(receipt.sequence, 2);
    assert_eq!(
        journal.recover(0, &RevocationState::new()).state,
        RecoveryState::Ready
    );
}
