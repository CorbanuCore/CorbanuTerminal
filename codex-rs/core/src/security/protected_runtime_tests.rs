#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use codex_config::AuthoritativeSecurityState;
use codex_config::AuthoritativeStateOwner;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_security_audit::DispatchResolution;
use codex_security_audit::IntegrityCheckpoint;
use codex_security_audit::IntegrityRootError;
use codex_security_audit::IntegrityRootStore;
use codex_security_audit::JournalConfig;
use codex_security_audit::JournalOwner;
use codex_security_audit::RecoveryBlocker;
use codex_security_audit::RecoveryReport;
use codex_security_audit::RecoveryState;
use codex_security_audit::ReferenceJournal;
use codex_security_audit::UnknownOutcomeReason;
use codex_security_policy::ActionReceipt;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::MandateOutcome;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedActionMandate;
use codex_security_policy::ProtectedActionPreview;
use codex_security_policy::ProtectedDispatchStep;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

use super::EffectivePolicyInitialization;
use super::EffectivePolicyView;
use super::PersistedHumanSecurityState;
use super::TrustedSecurityController;
use super::effective_policy::EffectivePolicySnapshot;
use super::protected_runtime::CurrentProtectedRuntime;
use super::protected_runtime::MeasuredProtectionReadiness;
use super::protected_runtime::ProtectedAuthority;
use super::protected_runtime::ProtectedRouteKind;
use super::protected_runtime::ProtectedRouteRegistry;
use super::protected_runtime::ProtectedRuntime;
use super::protected_runtime::ProtectedRuntimeError;
use super::protected_runtime::ProtectionReadinessStatus;
use super::protected_runtime::ReadinessWindow;
use super::protected_runtime::RuntimeGenerationBinding;

const NOW: i64 = 10;
const RUN_GENERATION: u64 = 7;

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("bounded text")
}

fn principal(kind: PrincipalKind, id: &str) -> PolicyPrincipal {
    PolicyPrincipal::new(kind, id).expect("principal")
}

fn policy(
    level: SecurityLevel,
    revocations: RevocationState,
) -> (EffectivePolicyView, TrustedSecurityController, ThreadId) {
    let root = ThreadId::new();
    let view = EffectivePolicyView::default();
    let persisted = PersistedHumanSecurityState::new(
        SecuritySettings::new(level),
        principal(PrincipalKind::Human, "human-runtime-owner"),
        revocations,
    )
    .expect("persisted state");
    let controller = TrustedSecurityController::initialize(
        &view,
        persisted,
        root,
        SessionId::new(),
        EffectivePolicyInitialization::Root,
    )
    .expect("controller");
    (view, controller, root)
}

fn authoritative(
    level: SecurityLevel,
    revocation_generation: u64,
    kill_switch_active: bool,
) -> AuthoritativeSecurityState {
    AuthoritativeSecurityState::new(
        1,
        state_owner(),
        level,
        0,
        revocation_generation,
        u64::from(kill_switch_active),
        kill_switch_active,
    )
    .expect("authoritative state")
}

fn state_owner() -> AuthoritativeStateOwner {
    AuthoritativeStateOwner::new("a".repeat(64), "runtime-owner", 1).expect("state owner")
}

fn readiness(status: ProtectionReadinessStatus) -> MeasuredProtectionReadiness {
    MeasuredProtectionReadiness::new(
        "broker-backend-v1",
        "broker-identity-v1",
        principal(PrincipalKind::Service, "security-audit-producer"),
        state_owner(),
        status,
        RuntimeGenerationBinding {
            owner: 1,
            policy: 1,
            run: RUN_GENERATION,
            revocation: 0,
        },
        ReadinessWindow {
            measured_at_unix_seconds: 5,
            expires_at_unix_seconds: 20,
        },
    )
    .expect("readiness")
}

fn ready_recovery() -> RecoveryReport {
    RecoveryReport {
        state: RecoveryState::Empty,
        event_count: 0,
        checkpoint: None,
        pending_dispatches: Vec::new(),
    }
}

fn current<'a>(
    effective: &'a EffectivePolicySnapshot,
    authoritative: &'a AuthoritativeSecurityState,
    readiness: &'a MeasuredProtectionReadiness,
    recovery: &'a RecoveryReport,
) -> CurrentProtectedRuntime<'a> {
    current_at(effective, authoritative, readiness, recovery, NOW)
}

fn current_at<'a>(
    effective: &'a EffectivePolicySnapshot,
    authoritative: &'a AuthoritativeSecurityState,
    readiness: &'a MeasuredProtectionReadiness,
    recovery: &'a RecoveryReport,
    now_unix_seconds: i64,
) -> CurrentProtectedRuntime<'a> {
    CurrentProtectedRuntime {
        effective,
        authoritative,
        readiness,
        recovery,
        run_generation: RUN_GENERATION,
        now_unix_seconds,
    }
}

fn routes() -> ProtectedRouteRegistry {
    ProtectedRouteRegistry::new([
        (
            ProtectedRouteKind::Ingress,
            "screened-retrieval".to_string(),
        ),
        (ProtectedRouteKind::Egress, "brokered-https".to_string()),
    ])
    .expect("routes")
}

#[test]
fn protected_runtime_binds_configured_creator_and_effective_levels() {
    let revocations = Arc::new(RwLock::new(RevocationState::new()));
    let (view, _controller, root) = policy(SecurityLevel::Moderate, RevocationState::new());
    let child = ThreadId::new();
    let child_snapshot = view
        .inherit_child(root, child, "task:child", SecurityLevel::Aggressive)
        .expect("stricter child");
    let state = authoritative(SecurityLevel::Moderate, 0, false);
    let measured = readiness(ProtectionReadinessStatus::Ready);
    let recovery = ready_recovery();

    let runtime = ProtectedRuntime::compose(
        current(&child_snapshot, &state, &measured, &recovery),
        routes(),
        Arc::clone(&revocations),
    )
    .expect("runtime");

    assert_eq!(
        runtime.snapshot(),
        &super::protected_runtime::ProtectedRuntimeSnapshot {
            contract_version: 4,
            configured_level: SecurityLevel::Moderate,
            creator_required_level: SecurityLevel::Aggressive,
            effective_level: SecurityLevel::Aggressive,
            effective_policy_epoch: 0,
            actor_chain: child_snapshot.actor_chain.clone(),
            session_id: child_snapshot.session_id.clone(),
            task_id: child_snapshot.task_id.clone(),
            generations: RuntimeGenerationBinding {
                owner: 1,
                policy: 1,
                run: RUN_GENERATION,
                revocation: 0,
            },
            readiness_window: ReadinessWindow {
                measured_at_unix_seconds: 5,
                expires_at_unix_seconds: 20,
            },
            state_owner: state_owner(),
            backend_id: text("broker-backend-v1"),
            identity_id: text("broker-identity-v1"),
        }
    );

    let mut weakened = child_snapshot;
    weakened.level = SecurityLevel::Moderate;
    assert!(matches!(
        ProtectedRuntime::compose(
            current(&weakened, &state, &measured, &recovery),
            routes(),
            revocations,
        ),
        Err(ProtectedRuntimeError::EffectivePolicyMismatch)
    ));

    let root_snapshot = view.snapshot_for_agent(root).expect("root snapshot");
    assert!(matches!(
        runtime.authorize_route(
            current(&root_snapshot, &state, &measured, &recovery),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        ),
        Err(ProtectedRuntimeError::StaleRuntimeBinding)
    ));
}

#[test]
fn protected_runtime_rejects_unavailable_stale_and_expired_readiness() {
    let revocations = Arc::new(RwLock::new(RevocationState::new()));
    let (view, _controller, root) = policy(SecurityLevel::Moderate, RevocationState::new());
    let effective = view.snapshot_for_agent(root).expect("snapshot");
    let state = authoritative(SecurityLevel::Moderate, 0, false);
    let recovery = ready_recovery();
    for status in [
        ProtectionReadinessStatus::Unavailable,
        ProtectionReadinessStatus::Unsupported,
    ] {
        let unavailable = readiness(status);
        assert!(matches!(
            ProtectedRuntime::compose(
                current(&effective, &state, &unavailable, &recovery),
                routes(),
                Arc::clone(&revocations),
            ),
            Err(ProtectedRuntimeError::ReadinessUnavailable(found)) if found == status
        ));
    }

    let forged_state = authoritative(SecurityLevel::Aggressive, 0, false);
    let ready = readiness(ProtectionReadinessStatus::Ready);
    assert!(matches!(
        ProtectedRuntime::compose(
            current(&effective, &forged_state, &ready, &recovery),
            routes(),
            Arc::clone(&revocations),
        ),
        Err(ProtectedRuntimeError::EffectivePolicyMismatch)
    ));

    let stale = MeasuredProtectionReadiness::new(
        "broker-backend-v1",
        "broker-identity-v1",
        principal(PrincipalKind::Service, "security-audit-producer"),
        state_owner(),
        ProtectionReadinessStatus::Ready,
        RuntimeGenerationBinding {
            owner: 1,
            policy: 2,
            run: RUN_GENERATION,
            revocation: 0,
        },
        ReadinessWindow {
            measured_at_unix_seconds: 5,
            expires_at_unix_seconds: 20,
        },
    )
    .expect("stale readiness");
    assert!(matches!(
        ProtectedRuntime::compose(
            current(&effective, &state, &stale, &recovery),
            routes(),
            Arc::clone(&revocations),
        ),
        Err(ProtectedRuntimeError::StaleReadinessGeneration)
    ));

    let expired = CurrentProtectedRuntime {
        now_unix_seconds: 20,
        ..current(&effective, &state, &ready, &recovery)
    };
    assert!(matches!(
        ProtectedRuntime::compose(expired, routes(), revocations),
        Err(ProtectedRuntimeError::ExpiredReadiness)
    ));

    assert!(matches!(
        MeasuredProtectionReadiness::new(
            "broker-backend-v1",
            "broker-identity-v1",
            principal(PrincipalKind::Service, "security-audit-producer"),
            state_owner(),
            ProtectionReadinessStatus::Ready,
            RuntimeGenerationBinding {
                owner: 1,
                policy: 1,
                run: RUN_GENERATION,
                revocation: 0,
            },
            ReadinessWindow {
                measured_at_unix_seconds: 5,
                expires_at_unix_seconds: 306,
            },
        ),
        Err(ProtectedRuntimeError::InvalidReadinessMeasurement)
    ));

    let clock_runtime = ProtectedRuntime::compose(
        current(&effective, &state, &ready, &recovery),
        routes(),
        Arc::new(RwLock::new(RevocationState::new())),
    )
    .expect("clock-bound runtime");
    clock_runtime
        .authorize_route(
            current_at(&effective, &state, &ready, &recovery, 12),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        )
        .expect("advance runtime clock");
    assert!(matches!(
        clock_runtime.authorize_route(
            current_at(&effective, &state, &ready, &recovery, 20),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        ),
        Err(ProtectedRuntimeError::ExpiredReadiness)
    ));
    clock_runtime
        .authorize_route(
            current_at(&effective, &state, &ready, &recovery, 13),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        )
        .expect("invalid future timestamp must not poison the clock");
    assert!(matches!(
        clock_runtime.authorize_route(
            current_at(&effective, &state, &ready, &recovery, 12),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        ),
        Err(ProtectedRuntimeError::RuntimeClockRegression)
    ));

    let substituted_window = MeasuredProtectionReadiness::new(
        "broker-backend-v1",
        "broker-identity-v1",
        principal(PrincipalKind::Service, "security-audit-producer"),
        state_owner(),
        ProtectionReadinessStatus::Ready,
        RuntimeGenerationBinding {
            owner: 1,
            policy: 1,
            run: RUN_GENERATION,
            revocation: 0,
        },
        ReadinessWindow {
            measured_at_unix_seconds: 0,
            expires_at_unix_seconds: 15,
        },
    )
    .expect("substituted window");
    assert!(matches!(
        clock_runtime.authorize_route(
            current_at(&effective, &state, &substituted_window, &recovery, 12),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        ),
        Err(ProtectedRuntimeError::StaleRuntimeBinding)
    ));
}

#[test]
fn unregistered_ingress_and_egress_fail_closed() {
    let revocations = Arc::new(RwLock::new(RevocationState::new()));
    let (view, _controller, root) = policy(SecurityLevel::Moderate, RevocationState::new());
    let effective = view.snapshot_for_agent(root).expect("snapshot");
    let state = authoritative(SecurityLevel::Moderate, 0, false);
    let measured = readiness(ProtectionReadinessStatus::Ready);
    let recovery = ready_recovery();
    let runtime = ProtectedRuntime::compose(
        current(&effective, &state, &measured, &recovery),
        routes(),
        revocations,
    )
    .expect("runtime");

    runtime
        .authorize_route(
            current(&effective, &state, &measured, &recovery),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        )
        .expect("known ingress");
    for (kind, id) in [
        (ProtectedRouteKind::Ingress, "synthetic-new-ingress"),
        (ProtectedRouteKind::Egress, "synthetic-new-egress"),
    ] {
        assert!(matches!(
            runtime.authorize_route(current(&effective, &state, &measured, &recovery), kind, id,),
            Err(ProtectedRuntimeError::UnknownRoute { .. })
        ));
    }
}

#[test]
fn restart_recovery_and_live_generation_changes_block_reuse() {
    let revocations = Arc::new(RwLock::new(RevocationState::new()));
    let (view, _controller, root) = policy(SecurityLevel::Moderate, RevocationState::new());
    let effective = view.snapshot_for_agent(root).expect("snapshot");
    let state = authoritative(SecurityLevel::Moderate, 0, false);
    let measured = readiness(ProtectionReadinessStatus::Ready);
    let recovery = ready_recovery();
    let runtime = ProtectedRuntime::compose(
        current(&effective, &state, &measured, &recovery),
        routes(),
        Arc::clone(&revocations),
    )
    .expect("runtime");
    let blocked = RecoveryReport {
        state: RecoveryState::Blocked(RecoveryBlocker::InterruptedWrite),
        event_count: 0,
        checkpoint: None,
        pending_dispatches: Vec::new(),
    };
    assert!(matches!(
        runtime.authorize_route(
            current(&effective, &state, &measured, &blocked),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        ),
        Err(ProtectedRuntimeError::AuditRecoveryBlocked)
    ));

    let matching_checkpoint = IntegrityCheckpoint {
        schema_version: 1,
        sequence: 1,
        record_sha256: "0".repeat(64),
        producer: principal(PrincipalKind::Service, "security-audit-producer"),
        owner_generation: 1,
        integrity_key_id: text("integrity-key-v1"),
        policy_generation: 1,
        run_generation: RUN_GENERATION,
    };
    let matching_recovery = RecoveryReport {
        state: RecoveryState::Ready,
        event_count: 1,
        checkpoint: Some(matching_checkpoint.clone()),
        pending_dispatches: Vec::new(),
    };
    runtime
        .authorize_route(
            current(&effective, &state, &measured, &matching_recovery),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        )
        .expect("matching recovery identity");

    let mut wrong_producer = matching_checkpoint.clone();
    wrong_producer.producer = principal(PrincipalKind::Service, "other-audit-producer");
    let mut wrong_owner = matching_checkpoint.clone();
    wrong_owner.owner_generation = 2;
    let mut future_policy = matching_checkpoint.clone();
    future_policy.policy_generation = 2;
    let mut future_run = matching_checkpoint;
    future_run.run_generation = RUN_GENERATION + 1;
    for mismatched_checkpoint in [wrong_producer, wrong_owner, future_policy, future_run] {
        let mismatched_recovery = RecoveryReport {
            state: RecoveryState::Ready,
            event_count: 1,
            checkpoint: Some(mismatched_checkpoint),
            pending_dispatches: Vec::new(),
        };
        assert!(matches!(
            runtime.authorize_route(
                current(&effective, &state, &measured, &mismatched_recovery),
                ProtectedRouteKind::Ingress,
                "screened-retrieval",
            ),
            Err(ProtectedRuntimeError::AuditRecoveryBindingMismatch)
        ));
    }

    let event = RevocationEvent::new(
        principal(PrincipalKind::Human, "human-runtime-owner"),
        RevocationTarget::AllActiveAuthority,
        RevocationReason::HumanRequest,
        11,
    )
    .expect("revocation");
    revocations
        .write()
        .expect("revocation lock")
        .apply(&event)
        .expect("apply revocation");
    assert!(matches!(
        runtime.authorize_route(
            current(&effective, &state, &measured, &recovery),
            ProtectedRouteKind::Ingress,
            "screened-retrieval",
        ),
        Err(ProtectedRuntimeError::StaleRevocationState)
    ));
}

#[derive(Debug, Default)]
struct MemoryRoot(Mutex<Option<IntegrityCheckpoint>>);

impl IntegrityRootStore for MemoryRoot {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        Ok(self.0.lock().expect("root lock").clone())
    }

    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        let mut root = self.0.lock().expect("root lock");
        if root.as_ref() != expected {
            return Err(IntegrityRootError::Conflict);
        }
        *root = Some(next.clone());
        Ok(())
    }
}

fn request_and_grant(snapshot: &EffectivePolicySnapshot) -> (AuthorizationRequest, BoundedGrant) {
    let resource = ProtectedResource::new(ResourceKind::NetworkDestination, "api.example.test")
        .expect("resource");
    let context = AuthorizationContext {
        now_unix_seconds: NOW,
        session_id: snapshot.session_id.clone(),
        task_id: snapshot.task_id.clone(),
        purpose: text("bounded-research"),
        operation: text("connect"),
        destination: Some(text("https://api.example.test")),
        quantity: None,
        grant_id: None,
    };
    let request = AuthorizationRequest::new(
        snapshot.actor_chain.clone(),
        resource.clone(),
        PolicyAction::Connect,
        context,
    )
    .expect("request");
    let scope = GrantScope::new(
        resource,
        [PolicyAction::Connect],
        GrantContext::new(
            snapshot.session_id.clone(),
            snapshot.task_id.clone(),
            text("bounded-research"),
            text("connect"),
        ),
        Some(text("https://api.example.test")),
        BTreeMap::new(),
    )
    .expect("scope");
    let grant = BoundedGrant::issue(
        principal(PrincipalKind::Human, "human-runtime-owner"),
        snapshot.actor_chain.clone(),
        scope,
        5,
        30,
        text("grant-nonce"),
    )
    .expect("grant");
    (request, grant)
}

#[test]
fn durable_intent_and_live_fence_precede_the_effect() {
    let revocations = Arc::new(RwLock::new(RevocationState::new()));
    let (view, _controller, root) = policy(SecurityLevel::Moderate, RevocationState::new());
    let effective = view.snapshot_for_agent(root).expect("snapshot");
    let state = authoritative(SecurityLevel::Moderate, 0, false);
    let measured = readiness(ProtectionReadinessStatus::Ready);
    let temp = tempfile::tempdir().expect("journal tempdir");
    let journal_root = AbsolutePathBuf::from_absolute_path_checked(temp.path().join("journal"))
        .expect("absolute journal root");
    let roots = Arc::new(MemoryRoot::default());
    let owner = JournalOwner::new(
        principal(PrincipalKind::Service, "security-audit-producer"),
        1,
        text("integrity-key-v1"),
    )
    .expect("journal owner");
    let mut journal = ReferenceJournal::new(journal_root, owner, roots, JournalConfig::default());
    let recovery = journal.recover(1, RUN_GENERATION, &RevocationState::new());
    let runtime = ProtectedRuntime::compose(
        current(&effective, &state, &measured, &recovery),
        routes(),
        Arc::clone(&revocations),
    )
    .expect("runtime");
    let other_runtime = ProtectedRuntime::compose(
        current(&effective, &state, &measured, &recovery),
        routes(),
        Arc::new(RwLock::new(RevocationState::new())),
    )
    .expect("distinct runtime instance");
    let (request, grant) = request_and_grant(&effective);
    let mut wrong_session_request = request.clone();
    wrong_session_request.context.session_id = text("session:other");
    assert!(matches!(
        runtime.reserve_dispatch(
            current(&effective, &state, &measured, &recovery),
            "brokered-https",
            &mut journal,
            &wrong_session_request,
            ProtectedAuthority::Grant(&grant),
            text("run-7"),
            text("wrong-runtime-session"),
        ),
        Err(ProtectedRuntimeError::RequestRuntimeMismatch)
    ));
    let short_lived_grant = BoundedGrant::issue(
        grant.issuer.clone(),
        grant.actor_chain.clone(),
        grant.scope.clone(),
        5,
        11,
        text("short-lived-grant-nonce"),
    )
    .expect("short-lived grant");
    assert!(matches!(
        runtime.reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 12),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Grant(&short_lived_grant),
            text("run-7"),
            text("expired-live-clock"),
        ),
        Err(ProtectedRuntimeError::Revocation(
            codex_security_policy::RevocationError::AuthorityOutsideValidityWindow
        ))
    ));
    let mut substituted_request = request.clone();
    substituted_request.context.destination = Some(text("https://other.example.test"));
    assert!(matches!(
        runtime.reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 12),
            "brokered-https",
            &mut journal,
            &substituted_request,
            ProtectedAuthority::Grant(&grant),
            text("run-7"),
            text("substituted-grant-effect"),
        ),
        Err(ProtectedRuntimeError::AuthorityRequestMismatch)
    ));

    let preview = ProtectedActionPreview::new(request.clone(), 30, text("preview-nonce"))
        .expect("action preview");
    let mandate = ProtectedActionMandate::approve(
        &preview,
        principal(PrincipalKind::Human, "human-runtime-owner"),
        11,
    )
    .expect("action mandate");
    assert!(matches!(
        runtime.reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 12),
            "brokered-https",
            &mut journal,
            &substituted_request,
            ProtectedAuthority::Mandate {
                mandate: &mandate,
                preview: &preview,
            },
            text("run-7"),
            text("substituted-mandate-effect"),
        ),
        Err(ProtectedRuntimeError::AuthorityRequestMismatch)
    ));

    let mut dispatch = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 12),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Grant(&grant),
            text("run-7"),
            text("effect-1"),
        )
        .expect("durable dispatch intent");

    let fence_is_held = dispatch
        .authorize(
            &other_runtime,
            current_at(&effective, &state, &measured, &recovery, 12),
            ProtectedAuthority::Grant(&grant),
            ProtectedDispatchStep::Admit,
            || (),
        )
        .expect_err("another runtime cannot authorize the dispatch");
    assert!(matches!(
        fence_is_held,
        ProtectedRuntimeError::RuntimeInstanceMismatch
    ));
    let fence_is_held = dispatch
        .authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 12),
            ProtectedAuthority::Grant(&grant),
            ProtectedDispatchStep::Admit,
            || revocations.try_write().is_err(),
        )
        .expect("admit dispatch");
    assert!(
        fence_is_held,
        "the revocation read guard must span the effect"
    );
    let completion = dispatch
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Completed {
                outcome: MandateOutcome::Executed,
                mandate_receipt: None,
            },
            12,
        )
        .expect("durable terminal receipt");
    assert_eq!(completion.sequence, 2);

    let mut unknown_dispatch = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 12),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Grant(&grant),
            text("run-7"),
            text("effect-unknown"),
        )
        .expect("durable unknown intent");
    unknown_dispatch
        .authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 12),
            ProtectedAuthority::Grant(&grant),
            ProtectedDispatchStep::Admit,
            || (),
        )
        .expect("admit unknown dispatch");
    let unknown_completion = unknown_dispatch
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Unknown {
                reason: UnknownOutcomeReason::TransportLost,
            },
            12,
        )
        .expect("durable unknown receipt");
    assert_eq!(unknown_completion.sequence, 4);

    let mut mandate_dispatch = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 12),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Mandate {
                mandate: &mandate,
                preview: &preview,
            },
            text("run-7"),
            text("effect-2"),
        )
        .expect("durable mandate intent");
    mandate_dispatch
        .authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 12),
            ProtectedAuthority::Mandate {
                mandate: &mandate,
                preview: &preview,
            },
            ProtectedDispatchStep::Admit,
            || (),
        )
        .expect("admit mandate dispatch");
    let receipt = ActionReceipt::complete(&mandate, &preview, MandateOutcome::Executed, 12)
        .expect("mandate receipt");
    let mandate_completion = mandate_dispatch
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Completed {
                outcome: MandateOutcome::Executed,
                mandate_receipt: Some(receipt),
            },
            12,
        )
        .expect("durable mandate receipt");
    assert_eq!(mandate_completion.sequence, 6);

    assert!(matches!(
        runtime.reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 13),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Mandate {
                mandate: &mandate,
                preview: &preview,
            },
            text("run-7"),
            text("caller-selected-different-key"),
        ),
        Err(ProtectedRuntimeError::Journal(
            codex_security_audit::JournalError::AlreadyResolved { .. }
        ))
    ));

    let reapproved_mandate = ProtectedActionMandate::approve(
        &preview,
        principal(PrincipalKind::Human, "human-runtime-owner"),
        12,
    )
    .expect("reapproved mandate");
    assert_ne!(reapproved_mandate.mandate_id, mandate.mandate_id);
    assert!(matches!(
        runtime.reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 13),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Mandate {
                mandate: &reapproved_mandate,
                preview: &preview,
            },
            text("run-7"),
            text("caller-key-for-reapproval"),
        ),
        Err(ProtectedRuntimeError::Journal(
            codex_security_audit::JournalError::AlreadyResolved { .. }
        ))
    ));

    let expires_before_admission = BoundedGrant::issue(
        grant.issuer.clone(),
        grant.actor_chain.clone(),
        grant.scope.clone(),
        5,
        14,
        text("expires-before-admission"),
    )
    .expect("expiring grant");
    let mut never_admitted = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 13),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Grant(&expires_before_admission),
            text("run-7"),
            text("never-admitted"),
        )
        .expect("durable never-admitted intent");
    assert!(matches!(
        never_admitted.authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 15),
            ProtectedAuthority::Grant(&expires_before_admission),
            ProtectedDispatchStep::Admit,
            || (),
        ),
        Err(ProtectedRuntimeError::Revocation(
            codex_security_policy::RevocationError::AuthorityOutsideValidityWindow
        ))
    ));
    let cancelled = never_admitted
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Completed {
                outcome: MandateOutcome::Cancelled,
                mandate_receipt: None,
            },
            15,
        )
        .expect("durable cancellation before admission");
    assert_eq!(cancelled.sequence, 8);

    let expiring_preview =
        ProtectedActionPreview::new(request.clone(), 17, text("expiring-mandate-preview"))
            .expect("expiring preview");
    let expiring_mandate = ProtectedActionMandate::approve(
        &expiring_preview,
        principal(PrincipalKind::Human, "human-runtime-owner"),
        15,
    )
    .expect("expiring mandate");
    let mut never_admitted_mandate = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 15),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Mandate {
                mandate: &expiring_mandate,
                preview: &expiring_preview,
            },
            text("run-7"),
            text("ignored-mandate-key"),
        )
        .expect("durable never-admitted mandate intent");
    assert!(matches!(
        never_admitted_mandate.authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 18),
            ProtectedAuthority::Mandate {
                mandate: &expiring_mandate,
                preview: &expiring_preview,
            },
            ProtectedDispatchStep::Admit,
            || (),
        ),
        Err(ProtectedRuntimeError::Revocation(
            codex_security_policy::RevocationError::AuthorityOutsideValidityWindow
        ))
    ));
    let conservative_unknown = never_admitted_mandate
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Unknown {
                reason: UnknownOutcomeReason::PersistenceUncertain,
            },
            18,
        )
        .expect("expired mandate closes conservatively as unknown");
    assert_eq!(conservative_unknown.sequence, 10);

    let collision_preview =
        ProtectedActionPreview::new(request.clone(), 19, text("dedup-domain-preview"))
            .expect("domain-separation preview");
    let collision_mandate = ProtectedActionMandate::approve(
        &collision_preview,
        principal(PrincipalKind::Human, "human-runtime-owner"),
        18,
    )
    .expect("domain-separation mandate");
    let colliding_caller_key = BoundedText::new(format!(
        "mandate-preview:{}",
        collision_mandate.preview_digest
    ))
    .expect("caller-selected collision key");
    let mut collision_grant = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 18),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Grant(&grant),
            text("run-7"),
            colliding_caller_key,
        )
        .expect("domain-separated grant intent");
    collision_grant
        .authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 18),
            ProtectedAuthority::Grant(&grant),
            ProtectedDispatchStep::Admit,
            || (),
        )
        .expect("admit collision grant");
    collision_grant
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Completed {
                outcome: MandateOutcome::Cancelled,
                mandate_receipt: None,
            },
            18,
        )
        .expect("cancel collision grant");
    let collision_mandate_dispatch = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 18),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Mandate {
                mandate: &collision_mandate,
                preview: &collision_preview,
            },
            text("run-7"),
            text("ignored-second-caller-key"),
        )
        .expect("grant key cannot pre-burn mandate namespace");
    let collision_receipt = collision_mandate_dispatch
        .resolve(
            &runtime,
            &mut journal,
            DispatchResolution::Unknown {
                reason: UnknownOutcomeReason::PersistenceUncertain,
            },
            18,
        )
        .expect("close domain-separation mandate intent");
    assert_eq!(collision_receipt.sequence, 14);

    let mut wrong_runtime_resolution = runtime
        .reserve_dispatch(
            current_at(&effective, &state, &measured, &recovery, 18),
            "brokered-https",
            &mut journal,
            &request,
            ProtectedAuthority::Grant(&grant),
            text("run-7"),
            text("wrong-runtime-resolution"),
        )
        .expect("runtime-bound resolution intent");
    wrong_runtime_resolution
        .authorize(
            &runtime,
            current_at(&effective, &state, &measured, &recovery, 18),
            ProtectedAuthority::Grant(&grant),
            ProtectedDispatchStep::Admit,
            || (),
        )
        .expect("admit runtime-bound resolution");
    assert!(matches!(
        wrong_runtime_resolution.resolve(
            &other_runtime,
            &mut journal,
            DispatchResolution::Completed {
                outcome: MandateOutcome::Cancelled,
                mandate_receipt: None,
            },
            18,
        ),
        Err(ProtectedRuntimeError::RuntimeInstanceMismatch)
    ));
}

#[test]
fn auxiliary_and_nested_children_cannot_weaken_creator_requirements() {
    let (view, _controller, root) = policy(SecurityLevel::Moderate, RevocationState::new());
    let auxiliary = ThreadId::new();
    let auxiliary_snapshot = view
        .inherit_auxiliary_agent(auxiliary, "task:guardian", SecurityLevel::Aggressive)
        .expect("auxiliary binding");
    let nested = view
        .inherit_child(
            auxiliary,
            ThreadId::new(),
            "task:nested",
            SecurityLevel::Permissive,
        )
        .expect("nested binding");

    assert_eq!(
        auxiliary_snapshot.creator_required_level,
        SecurityLevel::Aggressive
    );
    assert_eq!(nested.creator_required_level, SecurityLevel::Aggressive);
    assert_eq!(nested.level, SecurityLevel::Aggressive);
    assert_eq!(
        view.snapshot_for_agent(root).expect("root").level,
        SecurityLevel::Moderate
    );
}
