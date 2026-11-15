//! #1048: D48's explanation-only note belongs to an attempt, not a stale
//! approval round. The runtime below returns synthetic output: it never
//! executes the displayed command, starts a child or creates a namespace.
//! The person is a task answering the session's approval prompts (upstream's
//! approval flow asks through the session, not the tool).

use super::*;
use crate::session::turn_context::TurnEnvironment;
use crate::tools::sandboxing::Approvable;
use crate::tools::sandboxing::Sandboxable;
use codex_protocol::protocol::EventMsg;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

const REFUSED: &str = "unshare: unshare failed: Operation not permitted";

/// Upper bound on one probe run: a prompt that is never sent (or never
/// answered) fails the test instead of hanging it.
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

struct Probe {
    session: Arc<crate::session::session::Session>,
    controller: crate::security::TrustedSecurityController,
    /// Set by the person's first approval (when the fixture asks for a
    /// transition); the next sandbox-preference read commits it.
    transition_pending: Arc<AtomicBool>,
    /// Each prompt's reason and the taint generation it was shown at.
    asked: Arc<Mutex<Vec<(String, u64)>>>,
    explanations: Vec<Option<String>>,
}

impl Approvable<TurnEnvironment> for Probe {
    fn additional_permissions<'a>(
        &self,
        _req: &'a TurnEnvironment,
    ) -> Option<&'a codex_protocol::models::AdditionalPermissionProfile> {
        None
    }

    fn sandbox_permissions(&self, _req: &TurnEnvironment) -> crate::sandboxing::SandboxPermissions {
        crate::sandboxing::SandboxPermissions::RequireEscalated
    }

    fn exec_approval_requirement(&self, _req: &TurnEnvironment) -> Option<ExecApprovalRequirement> {
        Some(ExecApprovalRequirement::NeedsApproval {
            reason: None,
            proposed_execpolicy_amendment: None,
        })
    }

    fn approval_action(
        &self,
        req: &TurnEnvironment,
        call_id: &str,
    ) -> std::io::Result<ApprovalAction> {
        Ok(ApprovalAction::ExecCommand {
            id: call_id.to_string(),
            environment_id: req.selection.environment_id.clone(),
            // An absolute system-tool name exercises D48's actual resolver;
            // the Probe runtime never launches it.
            command: vec!["/usr/bin/unshare".to_string(), "-Ur".to_string()],
            hook_command: "/usr/bin/unshare -Ur".to_string(),
            cwd: req.cwd().clone(),
            sandbox_permissions: crate::sandboxing::SandboxPermissions::RequireEscalated,
            additional_permissions: None,
            justification: Some("synthetic namespace denial".to_string()),
            tty: false,
            proposed_execpolicy_amendment: None,
        })
    }
}

impl Sandboxable for Probe {
    fn sandbox_preference(&self) -> codex_sandboxing::SandboxablePreference {
        // This callback is also used before approval. Only the approved future
        // arms it, so the transition occurs after D48's allowance and before
        // the final D50 check, without a timer or production-only test hook.
        if self.transition_pending.swap(false, Ordering::SeqCst) {
            {
                let asked = self.asked.lock().expect("asked");
                assert_eq!(
                    asked.len(),
                    1,
                    "only the first approval arms the transition"
                );
                assert_eq!(asked[0].1, 1, "the first warning precedes the transition");
            }
            assert_eq!(
                self.session
                    .services
                    .model_client()
                    .post_taint_state()
                    .unwrap()
                    .taint_generation,
                1,
                "commit only after approval of the original generation",
            );
            let before = self
                .session
                .services
                .authority_marker()
                .expect("the fixture has an initialized policy");
            // Reapplying Moderate with empty revocations is a no-op. A real
            // synthetic grant revocation moves D50's marker without changing
            // the level whose D48 warning the second round must show.
            let mut revocations = RevocationState::new();
            let event = codex_security_policy::RevocationEvent::new(
                PolicyPrincipal::new(PrincipalKind::Human, "human:issue-1048").unwrap(),
                codex_security_policy::RevocationTarget::Grant {
                    grant_id: codex_security_policy::BoundedText::new("issue-1048-grant").unwrap(),
                },
                codex_security_policy::RevocationReason::HumanRequest,
                /*created_at_unix_seconds*/ 1,
            )
            .unwrap();
            assert!(
                revocations
                    .apply(&event)
                    .expect("apply synthetic revocation")
            );
            let change = self
                .controller
                .confirm_level_change(SecurityLevel::Moderate, revocations)
                .expect("confirm the synthetic transition");
            self.controller
                .apply_confirmed_change(change)
                .expect("commit transition");
            assert_eq!(
                self.session.services.authority_marker(),
                Some((before.0 + 1, before.1 + 1)),
                "the revocation must advance both parts of D50's marker",
            );
            assert!(self.session.services.transition_since(Some(before)));
            self.session
                .services
                .model_client()
                .note_unrecorded_input_for_taint();
        }
        codex_sandboxing::SandboxablePreference::Auto
    }
}

impl ToolRuntime<TurnEnvironment, ()> for Probe {
    fn turn_environment<'a>(&self, req: &'a TurnEnvironment) -> &'a TurnEnvironment {
        req
    }

    async fn run(
        &mut self,
        _req: &TurnEnvironment,
        _attempt: &SandboxAttempt<'_>,
        ctx: &ToolCtx,
    ) -> Result<(), ToolError> {
        self.explanations
            .push(crate::security::user_namespaces::denial_explanation(
                &ctx.session,
                &ctx.step_context.turn,
                &ctx.call_id,
                Some(1),
                REFUSED,
            ));
        Ok(())
    }
}

struct Fixture {
    probe: Probe,
    environment: TurnEnvironment,
    ctx: ToolCtx,
    answerer: tokio::task::JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.answerer.abort();
    }
}

impl Fixture {
    async fn new(transition_requested: bool) -> Self {
        let (session, events) = crate::session::tests::make_session_with_config_and_rx(|config| {
            config.security_level = SecurityLevel::Moderate;
            config
                .features
                .enable(codex_features::Feature::SourceEnvelopes)
                .expect("source envelopes");
            config
                .permissions
                .approval_policy
                .set(AskForApproval::OnRequest)
                .expect("approval policy");
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
                SecurityLevel::Moderate,
                session.thread_id,
                /*inherits_from_spawn_parent*/ false,
            )
            .expect("policy");
        let controller = control.trusted_security_controller().expect("controller");
        session
            .services
            .model_client()
            .note_unrecorded_input_for_taint();
        *session.active_turn.lock().await = Some(crate::state::ActiveTurn::default());
        let transition_pending = Arc::new(AtomicBool::new(false));
        let asked = Arc::new(Mutex::new(Vec::new()));
        // The person: records each prompt, arms the transition on the first
        // approval only (earlier preference reads, including the
        // pre-approval Git check, cannot commit it), then approves.
        let answerer = tokio::spawn({
            let session = Arc::clone(&session);
            let transition_pending = Arc::clone(&transition_pending);
            let asked = Arc::clone(&asked);
            let mut transition_requested = transition_requested;
            async move {
                while let Ok(event) = events.recv().await {
                    let EventMsg::ExecApprovalRequest(request) = event.msg else {
                        continue;
                    };
                    let state = session
                        .services
                        .model_client()
                        .post_taint_state()
                        .expect("post-taint state");
                    asked.lock().expect("asked").push((
                        request.reason.clone().expect("the prompt has a reason"),
                        state.taint_generation,
                    ));
                    if std::mem::take(&mut transition_requested) {
                        transition_pending.store(true, Ordering::SeqCst);
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
        let ctx = ToolCtx {
            session: Arc::clone(&session),
            step_context: crate::session::step_context::StepContext::for_test(Arc::clone(&turn)),
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            call_id: codex_protocol::ThreadId::new().to_string(),
            tool_name: codex_tools::ToolName::plain("issue_1048_probe"),
        };
        Self {
            probe: Probe {
                session,
                controller,
                transition_pending,
                asked,
                explanations: Vec::new(),
            },
            environment,
            ctx,
            answerer,
        }
    }

    fn asked(&self) -> Vec<(String, u64)> {
        self.probe.asked.lock().expect("asked").clone()
    }

    fn assert_warning(&self, generations: &[u64]) {
        let asked = self.asked();
        assert_eq!(
            asked.iter().map(|(_, at)| *at).collect::<Vec<_>>(),
            generations,
            "each prompt uses the current taint generation",
        );
        let warning = codex_sandboxing::user_namespaces::moderate_one_run_warning();
        for (reason, _) in &asked {
            assert!(reason.contains(&warning), "D48 warning is shown: {reason}");
        }
    }

    fn assert_no_leftover_note(&self) {
        let explanation = crate::security::user_namespaces::denial_explanation(
            &self.ctx.session,
            &self.ctx.step_context.turn,
            &self.ctx.call_id,
            Some(1),
            REFUSED,
        );
        assert!(
            explanation.is_some_and(|text| text.starts_with("Security level Moderate")),
            "a stale or already-consumed allowance must not hide a later denial",
        );
    }
}

#[tokio::test]
async fn issue_1048_stale_round_leaves_no_allowed_run_note() {
    let mut fixture = Fixture::new(/*transition_requested*/ true).await;
    let marker = fixture.ctx.session.services.authority_marker();
    let turn = Arc::clone(&fixture.ctx.step_context.turn);
    let result = tokio::time::timeout(
        PROBE_TIMEOUT,
        ToolOrchestrator::new().run_round(
            &mut fixture.probe,
            &fixture.environment,
            &fixture.ctx,
            &turn,
            AskForApproval::OnRequest,
            /*asked_again*/ false,
            marker,
        ),
    )
    .await
    .expect("the probe run finished")
    .expect("stale round is not an execution error");
    assert!(matches!(result, Round::Stale { person_asked: true }));
    assert_eq!(
        fixture.probe.explanations,
        Vec::<Option<String>>::new(),
        "nothing ran"
    );
    fixture.assert_warning(&[1]);
    fixture.assert_no_leftover_note();
}

#[tokio::test]
async fn issue_1048_redecided_round_warns_again_and_records_only_the_run() {
    let mut fixture = Fixture::new(/*transition_requested*/ true).await;
    tokio::time::timeout(
        PROBE_TIMEOUT,
        ToolOrchestrator::new().run(&mut fixture.probe, &fixture.environment, &fixture.ctx),
    )
    .await
    .expect("the probe run finished")
    .expect("freshly approved second round runs");
    fixture.assert_warning(&[1, 2]);
    assert!(
        fixture.asked()[1]
            .0
            .contains(crate::tools::approvals::ASKED_AGAIN_NOTE)
    );
    assert_eq!(
        fixture.probe.explanations,
        vec![None],
        "one authorized run's host denial"
    );
    fixture.assert_no_leftover_note();
}

#[tokio::test]
async fn issue_1048_unchanged_round_consumes_its_one_allowed_run_note() {
    let mut fixture = Fixture::new(/*transition_requested*/ false).await;
    tokio::time::timeout(
        PROBE_TIMEOUT,
        ToolOrchestrator::new().run(&mut fixture.probe, &fixture.environment, &fixture.ctx),
    )
    .await
    .expect("the probe run finished")
    .expect("the unchanged round runs");
    fixture.assert_warning(&[1]);
    assert_eq!(
        fixture.probe.explanations,
        vec![None],
        "the host denial is not blamed on policy"
    );
    fixture.assert_no_leftover_note();
}
