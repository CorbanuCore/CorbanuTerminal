//! D50 (#1001): a run approved while a security transition was committed
//! was approved for a stale prompt. Nothing runs on that answer: the run is
//! decided again, the person asked again (the prompt says why), and refused
//! when that keeps happening, or at once when the transition switched
//! Aggressive on. A retry outside the sandbox approved (or due) before a
//! transition is not run.
//!
//! The transitions here move the session's policy marker; the live level the
//! model client reports stays Permissive unless a test sets it.

use super::*;
use crate::session::session::Session;
use crate::session::tests::make_session_and_context_with_rx;
use crate::tools::approvals::ASKED_AGAIN_NOTE;
use crate::tools::sandboxing::Approvable;
use codex_protocol::error::CodexErr;
use codex_protocol::error::SandboxErr;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;
use std::sync::Arc;

fn initialize_policy(session: &Session) -> crate::security::TrustedSecurityController {
    crate::security::TrustedSecurityController::initialize(
        &session.services.agent_control.effective_security_policy(),
        crate::security::PersistedHumanSecurityState::new(
            SecuritySettings::new(SecurityLevel::Permissive),
            PolicyPrincipal::new(PrincipalKind::Human, "human:issue-1001").unwrap(),
            RevocationState::new(),
        )
        .unwrap(),
        session.thread_id,
        codex_protocol::SessionId::from(session.thread_id),
        crate::security::EffectivePolicyInitialization::Root,
    )
    .expect("initialize the session's policy tree")
}

/// What happens while the probe is asked or runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Transition {
    /// A security transition is committed: the policy marker moves.
    Commit(SecurityLevel),
    /// One that also switches Aggressive on for the session's live level.
    SwitchAggressiveOn,
    /// A revocation (of a grant): the marker moves, the level stays.
    Revoke,
}

/// A tool that needs approval (or not). Each ask records the prompt's
/// reason, applies the next queued transition while the prompt is open (so
/// before the answer), and the person approves. Each run applies the next
/// queued run transition and may be denied by the sandbox.
struct Probe {
    session: Arc<Session>,
    controller: crate::security::TrustedSecurityController,
    requirement: ExecApprovalRequirement,
    while_asked: Vec<Transition>,
    while_running: Vec<Transition>,
    /// Whether each run, in order, is denied by the sandbox (later ones are
    /// not).
    sandbox_denies: Vec<bool>,
    asked: Vec<Option<String>>,
    runs: usize,
    revocations: RevocationState,
    level: SecurityLevel,
}

impl Probe {
    fn apply(&mut self, transition: Transition) {
        let level = match transition {
            Transition::Commit(level) => level,
            Transition::Revoke => {
                let event = codex_security_policy::RevocationEvent::new(
                    PolicyPrincipal::new(PrincipalKind::Human, "human:issue-1001").unwrap(),
                    codex_security_policy::RevocationTarget::Grant {
                        grant_id: codex_security_policy::BoundedText::new(format!(
                            "issue-1001-grant-{}",
                            self.revocations.generation
                        ))
                        .unwrap(),
                    },
                    codex_security_policy::RevocationReason::HumanRequest,
                    /*created_at_unix_seconds*/ 1,
                )
                .unwrap();
                self.revocations.apply(&event).unwrap();
                self.level
            }
            Transition::SwitchAggressiveOn => {
                let client = (*self.session.services.model_client())
                    .clone()
                    .with_ingress_level(SecurityLevel::Aggressive);
                self.session.services.replace_model_client(client);
                SecurityLevel::Aggressive
            }
        };
        self.level = level;
        let change = self
            .controller
            .confirm_level_change(level, self.revocations.clone())
            .unwrap();
        self.controller.apply_confirmed_change(change).unwrap();
    }
}

/// The request: the additional permissions it carries (as a handler adds
/// granted ones).
type Req = Option<codex_protocol::models::AdditionalPermissionProfile>;

impl Approvable<Req> for Probe {
    type ApprovalKey = String;

    fn additional_permissions<'a>(
        &self,
        req: &'a Req,
    ) -> Option<&'a codex_protocol::models::AdditionalPermissionProfile> {
        req.as_ref()
    }

    fn approval_keys(&self, _req: &Req) -> Vec<Self::ApprovalKey> {
        vec!["issue-1001-probe".to_string()]
    }

    fn exec_approval_requirement(&self, _req: &Req) -> Option<ExecApprovalRequirement> {
        Some(self.requirement.clone())
    }

    fn wants_no_sandbox_approval(&self, _policy: AskForApproval) -> bool {
        true
    }

    fn start_approval_async<'a>(
        &'a mut self,
        _req: &'a Req,
        ctx: ApprovalCtx<'a>,
    ) -> futures::future::BoxFuture<'a, ReviewDecision> {
        // A person is shown the prompt (not a remembered answer).
        crate::tools::sandboxing::note_person_asked();
        self.asked.push(ctx.retry_reason);
        if !self.while_asked.is_empty() {
            let transition = self.while_asked.remove(0);
            self.apply(transition);
        }
        Box::pin(async { ReviewDecision::Approved })
    }

    fn approval_action(
        &self,
        _req: &Req,
        ctx: &ApprovalCtx<'_>,
    ) -> std::io::Result<ApprovalAction> {
        Ok(ApprovalAction::Shell {
            id: ctx.call_id.to_string(),
            environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
            command: vec!["issue-1001-probe".to_string()],
            #[allow(deprecated)]
            cwd: PathUri::from_abs_path(&ctx.turn.cwd),
            sandbox_permissions: crate::sandboxing::SandboxPermissions::UseDefault,
            additional_permissions: None,
            justification: Some("the model's reason".to_string()),
        })
    }
}

impl crate::tools::sandboxing::Sandboxable for Probe {
    fn sandbox_preference(&self) -> codex_sandboxing::SandboxablePreference {
        codex_sandboxing::SandboxablePreference::Auto
    }
}

impl ToolRuntime<Req, ()> for Probe {
    fn workspace_roots<'a>(&self, _req: &'a Req) -> &'a [PathUri] {
        &[]
    }

    async fn run(
        &mut self,
        _req: &Req,
        _attempt: &SandboxAttempt<'_>,
        _ctx: &ToolCtx,
    ) -> Result<(), ToolError> {
        if !self.while_running.is_empty() {
            let transition = self.while_running.remove(0);
            self.apply(transition);
        }
        let denied = self.sandbox_denies.get(self.runs).copied().unwrap_or(false);
        self.runs += 1;
        if denied {
            return Err(ToolError::Codex(CodexErr::Sandbox(SandboxErr::Denied {
                output: Box::new(ExecToolCallOutput {
                    exit_code: 1,
                    ..Default::default()
                }),
                network_policy_decision: None,
            })));
        }
        Ok(())
    }
}

fn needs_approval() -> ExecApprovalRequirement {
    ExecApprovalRequirement::NeedsApproval {
        reason: None,
        proposed_execpolicy_amendment: None,
    }
}

struct Case {
    requirement: ExecApprovalRequirement,
    while_asked: Vec<Transition>,
    while_running: Vec<Transition>,
    sandbox_denies: Vec<bool>,
    approval_policy: AskForApproval,
    additional: Option<codex_protocol::models::AdditionalPermissionProfile>,
    /// #1057: the request is prepared as a tool handler does: a session
    /// grant of network access (when `granted`) is read into it, then
    /// `after_handler` is committed before the orchestrator starts.
    handler: Option<Handler>,
}

struct Handler {
    granted: bool,
    after_handler: Option<Transition>,
}

impl Default for Case {
    fn default() -> Self {
        Self {
            requirement: needs_approval(),
            while_asked: Vec::new(),
            while_running: Vec::new(),
            sandbox_denies: Vec::new(),
            approval_policy: AskForApproval::OnRequest,
            additional: None,
            handler: None,
        }
    }
}

/// Runs the probe; the error is its message, or "sandbox denied".
async fn run_probe(case: Case) -> (Probe, Result<(), String>) {
    let (session, turn, _events) = make_session_and_context_with_rx().await;
    let controller = initialize_policy(&session);
    let mut probe = Probe {
        session: Arc::clone(&session),
        controller,
        requirement: case.requirement,
        while_asked: case.while_asked,
        while_running: case.while_running,
        sandbox_denies: case.sandbox_denies,
        asked: Vec::new(),
        runs: 0,
        revocations: RevocationState::new(),
        level: SecurityLevel::Permissive,
    };
    let tool_ctx = ToolCtx {
        session: Arc::clone(&session),
        turn: Arc::clone(&turn),
        call_id: "call-1001".to_string(),
        tool_name: codex_tools::ToolName::plain("issue_1001_probe"),
    };
    let mut additional = case.additional;
    let mut handler_basis = None;
    if let Some(handler) = case.handler {
        if handler.granted {
            session
                .record_granted_request_permissions_for_turn(
                    &codex_protocol::request_permissions::RequestPermissionsResponse {
                        permissions:
                            codex_protocol::request_permissions::RequestPermissionProfile {
                                network: Some(codex_protocol::models::NetworkPermissions {
                                    enabled: Some(true),
                                }),
                                ..Default::default()
                            },
                        scope: codex_protocol::request_permissions::PermissionGrantScope::Session,
                        strict_auto_review: false,
                    },
                    codex_exec_server::LOCAL_ENVIRONMENT_ID,
                    /*originating_turn_state*/ None,
                    session.services.authority_marker(),
                )
                .await;
        }
        handler_basis = Some(HandlerBasis::now(&session));
        #[allow(deprecated)]
        let cwd = turn.cwd.clone();
        let effective = crate::tools::handlers::apply_granted_turn_permissions(
            &session,
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            cwd.as_path(),
            crate::sandboxing::SandboxPermissions::UseDefault,
            additional,
        )
        .await;
        assert_eq!(
            effective.permissions_preapproved, handler.granted,
            "the grant is read"
        );
        additional = effective.additional_permissions;
        // The gap between the handler's read and the orchestrator's start.
        if let Some(transition) = handler.after_handler {
            probe.apply(transition);
        }
    }
    let result = ToolOrchestrator::new()
        .with_handler_basis(handler_basis)
        .run(
            &mut probe,
            &additional,
            &tool_ctx,
            turn.as_ref(),
            case.approval_policy,
        )
        .await
        .map(|_| ())
        .map_err(|err| match err {
            ToolError::Rejected(message) | ToolError::Declined(message) => message,
            ToolError::Codex(err) => match err.details() {
                CodexErrorDetails::Sandbox(SandboxErr::Denied { .. }) => {
                    "sandbox denied".to_string()
                }
                _ => err.to_string(),
            },
        });
    (probe, result)
}

#[tokio::test]
async fn issue_1001_approval_before_a_transition_is_asked_again() {
    let (probe, result) = run_probe(Case {
        while_asked: vec![Transition::Commit(SecurityLevel::Moderate)],
        ..Case::default()
    })
    .await;
    assert_eq!(result, Ok(()));
    assert_eq!(
        probe.asked,
        vec![None, Some(format!("{ASKED_AGAIN_NOTE} the model's reason"))]
    );
    assert_eq!(probe.runs, 1, "ran once, on the second answer");
}

#[tokio::test]
async fn issue_1001_approval_without_a_transition_runs_as_before() {
    let (probe, result) = run_probe(Case::default()).await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.asked, vec![None]);
    assert_eq!(probe.runs, 1);
}

#[tokio::test]
async fn issue_1001_approval_stale_every_time_is_refused() {
    let (probe, result) = run_probe(Case {
        while_asked: vec![
            Transition::Commit(SecurityLevel::Moderate),
            Transition::Revoke,
            Transition::Revoke,
        ],
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err(STALE_APPROVAL_REFUSAL.to_string()));
    assert_eq!(probe.asked.len(), MAX_STALE_APPROVAL_ROUNDS);
    assert_eq!(probe.runs, 0, "nothing ran");
}

/// The exec-policy decision a run carries was made for the level when it
/// started (under Aggressive, allow rules saved outside it are suspended and
/// vault commands forbidden), so an approval made stale by switching
/// Aggressive on is refused instead of asked again.
#[tokio::test]
async fn issue_1001_approval_before_aggressive_is_switched_on_is_refused() {
    let (probe, result) = run_probe(Case {
        while_asked: vec![Transition::SwitchAggressiveOn],
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err(STALE_AGGRESSIVE_REFUSAL.to_string()));
    assert_eq!(probe.asked.len(), 1);
    assert_eq!(probe.runs, 0, "nothing ran");
}

/// The retry outside the sandbox, approved after a transition: not run, and
/// the run is not decided again (its sandboxed attempt already ran). The
/// sandbox denial stands.
#[tokio::test]
async fn issue_1001_retry_approved_before_a_transition_is_not_run() {
    let (probe, result) = run_probe(Case {
        requirement: ExecApprovalRequirement::Skip {
            bypass_sandbox: false,
            proposed_execpolicy_amendment: None,
        },
        while_asked: vec![Transition::Commit(SecurityLevel::Moderate)],
        sandbox_denies: vec![true],
        approval_policy: AskForApproval::UnlessTrusted,
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err("sandbox denied".to_string()));
    assert_eq!(probe.asked.len(), 1, "the retry was asked about once");
    assert_eq!(probe.runs, 1, "no retry, no second sandboxed attempt");
}

/// A transition while the sandboxed attempt runs: the retry that the first
/// approval would have let run unasked is not run on that approval.
#[tokio::test]
async fn issue_1001_unprompted_retry_after_a_transition_is_not_run() {
    let (probe, result) = run_probe(Case {
        while_running: vec![Transition::Commit(SecurityLevel::Moderate)],
        sandbox_denies: vec![true],
        approval_policy: AskForApproval::UnlessTrusted,
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err("sandbox denied".to_string()));
    assert_eq!(probe.asked, vec![None], "approved once, before it ran");
    assert_eq!(probe.runs, 1, "not retried");

    // Without the transition the same retry runs unasked, as before.
    let (probe, result) = run_probe(Case {
        sandbox_denies: vec![true],
        approval_policy: AskForApproval::UnlessTrusted,
        ..Case::default()
    })
    .await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.asked, vec![None]);
    assert_eq!(probe.runs, 2);
}

/// D51 (#1001): a request carrying additional permissions (a handler adds
/// granted ones before the run is decided) approved before a transition is
/// refused instead of decided again: the transition ended those grants.
#[tokio::test]
async fn issue_1001_stale_approval_with_permissions_is_refused() {
    let network = codex_protocol::models::AdditionalPermissionProfile {
        network: Some(codex_protocol::models::NetworkPermissions {
            enabled: Some(true),
        }),
        ..Default::default()
    };
    let (probe, result) = run_probe(Case {
        while_asked: vec![Transition::Revoke],
        additional: Some(network.clone()),
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err(STALE_PERMISSIONS_REFUSAL.to_string()));
    assert_eq!(probe.asked.len(), 1);
    assert_eq!(probe.runs, 0, "nothing ran");

    // Without a transition it runs as before.
    let (probe, result) = run_probe(Case {
        additional: Some(network),
        ..Case::default()
    })
    .await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.runs, 1);
}

/// #1057: a preapproved request (a session grant covers what it adds), so
/// nobody is asked.
fn preapproved() -> ExecApprovalRequirement {
    ExecApprovalRequirement::Skip {
        bypass_sandbox: false,
        proposed_execpolicy_amendment: None,
    }
}

/// #1057: a transition committed after the handler read the grants but
/// before the orchestrator started ended those grants: the request carrying
/// them is refused, and nothing runs on them.
#[tokio::test]
async fn issue_1057_grants_read_before_a_transition_are_not_used() {
    for transition in [
        Transition::Revoke,
        Transition::Commit(SecurityLevel::Moderate),
    ] {
        let (probe, result) = run_probe(Case {
            requirement: preapproved(),
            handler: Some(Handler {
                granted: true,
                after_handler: Some(transition),
            }),
            ..Case::default()
        })
        .await;
        assert_eq!(
            result,
            Err(PREPARED_PERMISSIONS_REFUSAL.to_string()),
            "{transition:?}"
        );
        assert_eq!(probe.asked, Vec::<Option<String>>::new(), "nobody asked");
        assert_eq!(probe.runs, 0, "nothing ran on the ended grant");
    }
}

/// #1057: the exec-policy decision the handler made before Aggressive was
/// switched on is stale: refused, before anyone is asked.
#[tokio::test]
async fn issue_1057_aggressive_switched_on_after_the_handler_is_refused() {
    let (probe, result) = run_probe(Case {
        requirement: preapproved(),
        handler: Some(Handler {
            granted: false,
            after_handler: Some(Transition::SwitchAggressiveOn),
        }),
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err(PREPARED_BEFORE_AGGRESSIVE_REFUSAL.to_string()));
    assert_eq!(probe.asked, Vec::<Option<String>>::new());
    assert_eq!(probe.runs, 0);
}

/// #1057: without a transition the grant is used as before; with one but
/// no grant (nothing read went stale) the run is decided under the new
/// level, the person asked once and without the "asked again" note.
#[tokio::test]
async fn issue_1057_handler_window_controls() {
    let (probe, result) = run_probe(Case {
        requirement: preapproved(),
        handler: Some(Handler {
            granted: true,
            after_handler: None,
        }),
        ..Case::default()
    })
    .await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.asked, Vec::<Option<String>>::new());
    assert_eq!(probe.runs, 1);

    let (probe, result) = run_probe(Case {
        handler: Some(Handler {
            granted: false,
            after_handler: Some(Transition::Commit(SecurityLevel::Moderate)),
        }),
        ..Case::default()
    })
    .await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.asked, vec![None]);
    assert_eq!(probe.runs, 1);
}
