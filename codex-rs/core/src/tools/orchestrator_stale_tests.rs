//! D50 (#1001): a run approved while a security transition was committed
//! was approved for a stale prompt. Nothing runs on that answer: the run is
//! decided again, the person asked again (the prompt says why), and refused
//! when that keeps happening, or at once when the transition switched
//! Aggressive on. A retry outside the sandbox approved (or due) before a
//! transition is not run.
//!
//! The transitions here move the session's policy marker; the live level the
//! model client reports stays Permissive unless a test sets it. The person is
//! a task answering the session's approval prompts (upstream's approval flow
//! asks through the session, not the tool).

use super::*;
use crate::session::session::Session;
use crate::session::turn_context::TurnEnvironment;
use crate::tools::approvals::ASKED_AGAIN_NOTE;
use crate::tools::sandboxing::Approvable;
use codex_protocol::error::CodexErr;
use codex_protocol::error::SandboxErr;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::protocol::EventMsg;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::Mutex;

/// Upper bound on one probe run: a prompt that is never sent (or never
/// answered) fails the test instead of hanging it.
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

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

/// Commits transitions on the session's policy tree.
struct Transitions {
    session: Arc<Session>,
    controller: crate::security::TrustedSecurityController,
    revocations: RevocationState,
    level: SecurityLevel,
}

impl Transitions {
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

/// A tool that needs approval (or not). Each run applies the next queued
/// run transition and may be denied by the sandbox.
struct Probe {
    transitions: Arc<Mutex<Transitions>>,
    requirement: ExecApprovalRequirement,
    while_running: Vec<Transition>,
    /// Whether each run, in order, is denied by the sandbox (later ones are
    /// not).
    sandbox_denies: Vec<bool>,
    /// D639(a) (#2808): as unified exec, an ordinary denial is final.
    keeps_denial_final: bool,
    runs: usize,
}

/// The request: the environment it runs in and the additional permissions
/// it carries (as a handler adds granted ones).
struct Req {
    environment: TurnEnvironment,
    additional: Option<codex_protocol::models::AdditionalPermissionProfile>,
}

impl Approvable<Req> for Probe {
    fn additional_permissions<'a>(
        &self,
        req: &'a Req,
    ) -> Option<&'a codex_protocol::models::AdditionalPermissionProfile> {
        req.additional.as_ref()
    }

    fn exec_approval_requirement(&self, _req: &Req) -> Option<ExecApprovalRequirement> {
        Some(self.requirement.clone())
    }

    fn wants_no_sandbox_approval(&self, _policy: AskForApproval) -> bool {
        true
    }

    fn approval_action(&self, req: &Req, call_id: &str) -> std::io::Result<ApprovalAction> {
        Ok(ApprovalAction::ExecCommand {
            id: call_id.to_string(),
            environment_id: req.environment.selection.environment_id.clone(),
            command: vec!["issue-1001-probe".to_string()],
            hook_command: "issue-1001-probe".to_string(),
            cwd: req.environment.cwd().clone(),
            sandbox_permissions: crate::sandboxing::SandboxPermissions::UseDefault,
            additional_permissions: req.additional.clone(),
            justification: Some("the model's reason".to_string()),
            tty: false,
            proposed_execpolicy_amendment: None,
        })
    }
}

impl crate::tools::sandboxing::Sandboxable for Probe {
    fn sandbox_preference(&self) -> codex_sandboxing::SandboxablePreference {
        codex_sandboxing::SandboxablePreference::Auto
    }

    fn keeps_sandbox_denial_final(&self) -> bool {
        self.keeps_denial_final
    }
}

impl ToolRuntime<Req, ()> for Probe {
    fn turn_environment<'a>(&self, req: &'a Req) -> &'a TurnEnvironment {
        &req.environment
    }

    async fn run(
        &mut self,
        _req: &Req,
        _attempt: &SandboxAttempt<'_>,
        _ctx: &ToolCtx,
    ) -> Result<(), ToolError> {
        if !self.while_running.is_empty() {
            let transition = self.while_running.remove(0);
            self.transitions
                .lock()
                .expect("transitions")
                .apply(transition);
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

/// What the probe run left behind: the reason each prompt showed, in order,
/// and how often the tool ran.
struct Outcome {
    asked: Vec<Option<String>>,
    runs: usize,
}

/// The reason the first prompt shows: the model's justification (upstream's
/// prompt carries it as the reason; only a prompt asked again adds the note).
fn first_prompt() -> Option<String> {
    Some("the model's reason".to_string())
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
    keeps_denial_final: bool,
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
            keeps_denial_final: false,
            approval_policy: AskForApproval::OnRequest,
            additional: None,
            handler: None,
        }
    }
}

/// Runs the probe; the error is its message, or "sandbox denied". Each
/// prompt records its reason, applies the next queued transition while it is
/// open (so before the answer), and the person approves.
async fn run_probe(case: Case) -> (Outcome, Result<(), String>) {
    let approval_policy = case.approval_policy;
    let (session, events) = crate::session::tests::make_session_with_config_and_rx(|config| {
        config
            .permissions
            .approval_policy
            .set(approval_policy)
            .expect("approval policy");
        // #1976: a Windows machine with its sandbox set up (the probe runs
        // nothing), so switching Aggressive on does not refuse the run up
        // front.
        config.permissions.windows_sandbox_mode =
            Some(codex_config::types::WindowsSandboxModeToml::Elevated);
    })
    .await
    .expect("session");
    let control = session
        .services
        .agent_control
        .local()
        .expect("the local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("policy");
    let controller = control.trusted_security_controller().expect("controller");
    let transitions = Arc::new(Mutex::new(Transitions {
        session: Arc::clone(&session),
        controller,
        revocations: RevocationState::new(),
        level: SecurityLevel::Permissive,
    }));
    *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
    // The person.
    let asked = Arc::new(Mutex::new(Vec::new()));
    let answerer = tokio::spawn({
        let session = Arc::clone(&session);
        let transitions = Arc::clone(&transitions);
        let asked = Arc::clone(&asked);
        let mut while_asked = case.while_asked;
        async move {
            while let Ok(event) = events.recv().await {
                let EventMsg::ExecApprovalRequest(request) = event.msg else {
                    continue;
                };
                asked.lock().expect("asked").push(request.reason.clone());
                if !while_asked.is_empty() {
                    let transition = while_asked.remove(0);
                    transitions.lock().expect("transitions").apply(transition);
                }
                let id = request.approval_id.unwrap_or(request.call_id);
                session.notify_approval(&id, ReviewDecision::Approved).await;
            }
        }
    });
    let turn = session.new_default_turn().await;
    let environment = turn
        .initial_environments
        .primary()
        .expect("a primary environment")
        .clone();
    let mut probe = Probe {
        transitions: Arc::clone(&transitions),
        requirement: case.requirement,
        while_running: case.while_running,
        sandbox_denies: case.sandbox_denies,
        keeps_denial_final: case.keeps_denial_final,
        runs: 0,
    };
    let tool_ctx = ToolCtx {
        session: Arc::clone(&session),
        step_context: crate::session::step_context::StepContext::for_test(Arc::clone(&turn)),
        cancellation_token: tokio_util::sync::CancellationToken::new(),
        call_id: "call-1001".to_string(),
        tool_name: codex_tools::ToolName::plain("issue_1001_probe"),
    };
    let mut additional = case.additional;
    let mut handler_basis = None;
    if let Some(handler) = case.handler {
        if handler.granted {
            assert!(
                session.record_granted_request_permissions_for_turn(
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
                    &environment.selection.environment_id,
                    turn.as_ref(),
                    session.services.authority_marker(),
                ),
                "the grant is recorded"
            );
        }
        handler_basis = Some(HandlerBasis::now(&session));
        let effective = crate::tools::handlers::apply_granted_turn_permissions(
            &session,
            &turn,
            &environment,
            environment.cwd(),
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
            transitions.lock().expect("transitions").apply(transition);
        }
    }
    let req = Req {
        environment,
        additional,
    };
    let result = tokio::time::timeout(
        PROBE_TIMEOUT,
        ToolOrchestrator::new()
            .with_handler_basis(handler_basis)
            .run(&mut probe, &req, &tool_ctx),
    )
    .await
    .expect("the probe run finished (a prompt that is never answered would hang it)");
    answerer.abort();
    let result = result.map(|_| ()).map_err(|err| match err {
        ToolError::Rejected(message) | ToolError::Declined(message) => message,
        ToolError::Codex(err) => match err.details() {
            CodexErrorDetails::Sandbox(SandboxErr::Denied { .. }) => "sandbox denied".to_string(),
            _ => err.to_string(),
        },
    });
    let asked = asked.lock().expect("asked").clone();
    (
        Outcome {
            asked,
            runs: probe.runs,
        },
        result,
    )
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
        vec![
            first_prompt(),
            Some(format!("{ASKED_AGAIN_NOTE} the model's reason"))
        ]
    );
    assert_eq!(probe.runs, 1, "ran once, on the second answer");
}

#[tokio::test]
async fn issue_1001_approval_without_a_transition_runs_as_before() {
    let (probe, result) = run_probe(Case::default()).await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.asked, vec![first_prompt()]);
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
    assert_eq!(
        probe.asked,
        vec![first_prompt()],
        "approved once, before it ran"
    );
    assert_eq!(probe.runs, 1, "not retried");

    // Without the transition the same retry runs unasked, as before.
    let (probe, result) = run_probe(Case {
        sandbox_denies: vec![true],
        approval_policy: AskForApproval::UnlessTrusted,
        ..Case::default()
    })
    .await;
    assert_eq!(result, Ok(()));
    assert_eq!(probe.asked, vec![first_prompt()]);
    assert_eq!(probe.runs, 2);
}

/// D639(a) (#2808): where an ordinary sandbox denial is final (unified
/// exec), the retry outside the sandbox that the first approval would have
/// let run unasked (above) is not run, nor offered to a run that skipped the
/// prompt: the denial stands.
#[tokio::test]
async fn issue_2808_a_final_denial_is_not_retried_outside_the_sandbox() {
    let (probe, result) = run_probe(Case {
        sandbox_denies: vec![true],
        keeps_denial_final: true,
        approval_policy: AskForApproval::UnlessTrusted,
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err("sandbox denied".to_string()));
    assert_eq!(
        probe.asked,
        vec![first_prompt()],
        "approved once, before it ran"
    );
    assert_eq!(probe.runs, 1, "not retried");

    let (probe, result) = run_probe(Case {
        requirement: ExecApprovalRequirement::Skip {
            bypass_sandbox: false,
            proposed_execpolicy_amendment: None,
        },
        sandbox_denies: vec![true],
        keeps_denial_final: true,
        approval_policy: AskForApproval::UnlessTrusted,
        ..Case::default()
    })
    .await;
    assert_eq!(result, Err("sandbox denied".to_string()));
    assert_eq!(
        probe.asked,
        Vec::<Option<String>>::new(),
        "no retry offered"
    );
    assert_eq!(probe.runs, 1, "not retried");
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
    assert_eq!(probe.asked, vec![first_prompt()]);
    assert_eq!(probe.runs, 1);
}
