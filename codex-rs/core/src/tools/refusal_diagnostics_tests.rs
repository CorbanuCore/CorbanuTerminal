//! Source-informed Issue822 regressions. No command or model request is executed.
//! Commands are `exec_command` (unified exec): upstream removed the shell tools.
use super::*;
use crate::session::tests::make_session_and_context_with_dynamic_tools_and_rx;
use crate::tools::events::ToolEmitter;
use crate::tools::events::ToolEventCtx;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::items::TurnItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandSource;
use std::time::Duration;

async fn assert_command_status(out: Result<ExecToolCallOutput, ToolError>, expected: &str) {
    let denied = out.is_err();
    let (session, turn, events) =
        make_session_and_context_with_dynamic_tools_and_rx(Vec::new()).await;
    let cwd = tempfile::tempdir().expect("synthetic workspace");
    let cwd = AbsolutePathBuf::try_from(cwd.path()).expect("absolute workspace");
    let command = vec!["echo".to_string(), "synthetic-only".to_string()];
    let emitter = ToolEmitter::unified_exec(
        &command,
        PathUri::from_abs_path(&cwd),
        ExecCommandSource::UnifiedExecStartup,
        /*process_id*/ None,
        /*plugin_attribution*/ None,
    );
    let result = emitter
        .finish(
            ToolEventCtx::new(
                session.as_ref(),
                turn.as_ref(),
                "issue822",
                /*turn_diff_tracker*/ None,
            ),
            out,
            /*applied_patch_delta*/ None,
        )
        .await;
    if denied {
        assert!(
            result.is_err(),
            "diagnostic classification must not allow a refusal"
        );
    }
    let event = tokio::time::timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("completed event timeout")
        .expect("completed event");
    let EventMsg::ItemCompleted(event) = event.msg else {
        panic!("expected completed item")
    };
    let TurnItem::CommandExecution(item) = event.item else {
        panic!("expected command item")
    };
    assert_eq!(serde_json::to_value(item.status).unwrap(), expected);
    if denied {
        assert_eq!(item.exit_code, Some(-1));
        assert_eq!(item.duration, Some(Duration::ZERO));
        assert!(item.aggregated_output.is_some_and(|text| !text.is_empty()));
    }
}

#[tokio::test]
async fn issue822_policy_refusal_is_not_an_approval_decline() {
    // Exact error returned before approval by ToolOrchestrator's protected-launch gate.
    assert_command_status(
        Err(ToolError::Rejected(
            crate::security::protected_additional::PROTECTED_LAUNCH_NO_ESCALATION.to_string(),
        )),
        "refused",
    )
    .await;
}

fn resolved(
    decision: ReviewDecision,
    source: ApprovalResolutionSource,
) -> Result<ReviewDecision, ToolError> {
    ApprovalResolution {
        decision,
        source,
        cancellation_reason: None,
    }
    .into_tool_result(&codex_models_manager::model_info::model_info_from_slug(
        "acting-model",
    ))
}

#[tokio::test]
async fn issue822_actual_user_decline_keeps_declined_status() {
    let error = resolved(
        ReviewDecision::denied("synthetic user refusal"),
        ApprovalResolutionSource::User,
    )
    .expect_err("human declined");
    assert_command_status(Err(error), "declined").await;
}

#[tokio::test]
async fn issue822_other_approval_resolutions_stay_denied() {
    for source in [
        ApprovalResolutionSource::Hook,
        ApprovalResolutionSource::Guardian,
        ApprovalResolutionSource::Remembered,
    ] {
        let error = resolved(ReviewDecision::denied("synthetic approval refusal"), source)
            .expect_err("approval denied");
        assert_command_status(Err(error), "declined").await;
    }
}

/// #873: an approval that timed out was still an approval request, so the
/// command reads as declined, never as refused by policy. An aborted one
/// ends the turn (upstream rust-v0.167.0).
#[tokio::test]
async fn issue873_timed_out_approvals_stay_declined_and_aborted_ones_end_the_turn() {
    let error = resolved(ReviewDecision::TimedOut, ApprovalResolutionSource::Guardian)
        .expect_err("no permission to proceed");
    assert!(matches!(error, ToolError::Declined(_)), "{error:?}");
    assert_command_status(Err(error), "declined").await;
    let error = resolved(ReviewDecision::Abort, ApprovalResolutionSource::User)
        .expect_err("no permission to proceed");
    assert!(
        matches!(&error, ToolError::Codex(CodexErr::TurnAborted)),
        "{error:?}"
    );
}

#[tokio::test]
async fn issue822_exit_success_and_failure_controls() {
    for (exit_code, status) in [(0, "completed"), (7, "failed")] {
        assert_command_status(
            Ok(ExecToolCallOutput {
                exit_code,
                ..Default::default()
            }),
            status,
        )
        .await;
    }
}

/// Actual post-taint reason through the existing native refusal event path.
/// This is not a tool-dispatch or PTY execution test.
#[tokio::test]
async fn issue941_actual_approvals_off_reason_survives_command_events() {
    use crate::security::tainted_action::PolicyBinding;
    use crate::security::tainted_action::PostTaintAction;
    use crate::security::tainted_action::PostTaintState;
    use crate::security::tainted_action::ProtectedActionKind;
    use codex_security_policy::SecurityLevel;

    let action = PostTaintAction {
        kind: ProtectedActionKind::Credentials,
        state: PostTaintState {
            taint_generation: 1,
            policy: PolicyBinding::Unbound,
            level: SecurityLevel::Moderate,
        },
    };
    let reason = action.approvals_off_rejection();
    {
        let (session, turn, events) =
            make_session_and_context_with_dynamic_tools_and_rx(Vec::new()).await;
        let workspace = tempfile::tempdir().expect("synthetic workspace");
        let cwd = AbsolutePathBuf::try_from(workspace.path()).expect("absolute workspace");
        let command = vec!["echo".to_string(), "synthetic-not-executed".to_string()];
        let emitter = ToolEmitter::unified_exec(
            &command,
            PathUri::from_abs_path(&cwd),
            ExecCommandSource::UnifiedExecStartup,
            /*process_id*/ None,
            /*plugin_attribution*/ None,
        );
        let result = emitter
            .finish(
                ToolEventCtx::new(
                    session.as_ref(),
                    turn.as_ref(),
                    "issue941",
                    /*turn_diff_tracker*/ None,
                ),
                Err(ToolError::Rejected(reason.clone())),
                /*applied_patch_delta*/ None,
            )
            .await;
        assert!(result.is_err());
        let event = tokio::time::timeout(Duration::from_secs(1), events.recv())
            .await
            .expect("refusal event timeout")
            .expect("refusal event");
        let EventMsg::ItemCompleted(completed) = event.msg else {
            panic!("expected completed item")
        };
        let TurnItem::CommandExecution(item) = completed.item else {
            panic!("expected command item")
        };
        assert_eq!(serde_json::to_value(item.status).unwrap(), "refused");
        assert_eq!(item.aggregated_output.as_deref(), Some(reason.as_str()));
        assert_eq!(item.exit_code, Some(-1));
        assert_eq!(item.duration, Some(Duration::ZERO));
    }
}
