//! Source-informed Issue822 regressions. No command or model request is executed.
use super::*;
use crate::session::tests::make_session_and_context_with_dynamic_tools_and_rx;
use crate::tools::events::ToolEmitter;
use crate::tools::events::ToolEventCtx;
use codex_protocol::exec_output::ExecToolCallOutput;
use codex_protocol::items::TurnItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandSource;
use std::time::Duration;

async fn assert_command_status(
    out: Result<ExecToolCallOutput, ToolError>,
    expected: &str,
    unified: bool,
) {
    let denied = out.is_err();
    let (session, turn, events) =
        make_session_and_context_with_dynamic_tools_and_rx(Vec::new()).await;
    let cwd = tempfile::tempdir().expect("synthetic workspace");
    let cwd = AbsolutePathBuf::try_from(cwd.path()).expect("absolute workspace");
    let command = vec!["echo".to_string(), "synthetic-only".to_string()];
    let emitter = if unified {
        ToolEmitter::unified_exec(
            &command,
            PathUri::from_abs_path(&cwd),
            ExecCommandSource::UnifiedExecStartup,
            /*process_id*/ None,
            /*plugin_attribution*/ None,
        )
    } else {
        ToolEmitter::shell(
            command,
            cwd,
            ExecCommandSource::Agent,
            /*plugin_attribution*/ None,
        )
    };
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
    for unified in [false, true] {
        assert_command_status(
            Err(ToolError::Rejected(
                crate::security::protected_additional::PROTECTED_LAUNCH_NO_ESCALATION.to_string(),
            )),
            "refused",
            unified,
        )
        .await;
    }
}

#[tokio::test]
async fn issue822_actual_user_decline_keeps_declined_status() {
    for unified in [false, true] {
        let error = ApprovalResolution {
            decision: ReviewDecision::denied("synthetic user refusal"),
            source: ApprovalResolutionSource::User,
        }
        .into_tool_result()
        .expect_err("human declined");
        assert_command_status(Err(error), "declined", unified).await;
    }
}

#[tokio::test]
async fn issue822_other_approval_resolutions_stay_denied() {
    for source in [
        ApprovalResolutionSource::Hook,
        ApprovalResolutionSource::Guardian,
        ApprovalResolutionSource::Remembered,
    ] {
        let error = ApprovalResolution {
            decision: ReviewDecision::denied("synthetic approval refusal"),
            source,
        }
        .into_tool_result()
        .expect_err("approval denied");
        assert_command_status(Err(error), "declined", /*unified*/ false).await;
    }
}

#[tokio::test]
async fn issue822_exit_success_and_failure_controls() {
    for unified in [false, true] {
        for (exit_code, status) in [(0, "completed"), (7, "failed")] {
            assert_command_status(
                Ok(ExecToolCallOutput {
                    exit_code,
                    ..Default::default()
                }),
                status,
                unified,
            )
            .await;
        }
    }
}
