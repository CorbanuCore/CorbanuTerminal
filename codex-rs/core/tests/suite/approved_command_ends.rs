//! Issue #466: a review (the person's prompt or the automatic reviewer)
//! shows its command as running, so an approved command must end on screen
//! even when it then fails to start; a declined one is ended by the
//! review's owner (the app server), not here.
#![cfg(unix)]

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandEndEvent;
use codex_protocol::protocol::ExecCommandStatus;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_sandbox;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use serde_json::json;

const CALL_ID: &str = "call-approved";

/// Runs one `exec_command` that needs approval and answers the prompt with
/// `decision`; returns how many begin events came and the end events.
async fn run_approved(
    arguments: serde_json::Value,
    decision: ReviewDecision,
) -> anyhow::Result<(usize, Vec<ExecCommandEndEvent>)> {
    let server = start_mock_server().await;
    let test: TestCodex = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.use_experimental_unified_exec_tool = true;
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            config
                .features
                .enable(Feature::UnifiedExec)
                .expect("test config should allow feature update");
        })
        .build_with_auto_env(&server)
        .await?;
    mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("run"),
                ev_function_call(CALL_ID, "exec_command", &arguments.to_string()),
                ev_completed("run"),
            ]),
            sse(vec![
                ev_response_created("done"),
                ev_assistant_message("msg-done", "done"),
                ev_completed("done"),
            ]),
        ],
    )
    .await;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "run it".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let EventMsg::ExecApprovalRequest(approval) = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    else {
        anyhow::bail!("the command was not sent for approval");
    };
    assert_eq!(approval.call_id, CALL_ID);
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision,
        })
        .await?;
    let mut began = 0;
    let mut ended = Vec::new();
    wait_for_event(&test.codex, |event| match event {
        EventMsg::ExecCommandBegin(begin) if begin.call_id == CALL_ID => {
            began += 1;
            false
        }
        EventMsg::ExecCommandEnd(end) if end.call_id == CALL_ID => {
            ended.push(end.clone());
            false
        }
        EventMsg::TurnComplete(_) => true,
        _ => false,
    })
    .await;
    Ok((began, ended))
}

/// Approved, then refused before it started (its folder is gone): it ends
/// as failed with the reason instead of staying "Running". Without a TTY the
/// spawn fails on the missing folder (a PTY could fall back to another one).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_466_an_approved_command_that_cannot_start_ends() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (began, ended) = run_approved(
        json!({
            "cmd": "touch approved.txt",
            "workdir": "missing-folder",
            "tty": false,
            "yield_time_ms": 5_000,
        }),
        ReviewDecision::Approved,
    )
    .await?;
    assert_eq!(began, 1, "the approved command is shown once");
    let [ended] = ended.as_slice() else {
        panic!("the approved command that could not start ends once: {ended:?}");
    };
    assert_ne!(ended.exit_code, 0);
    // #873: a spawn error is a failure, not a policy refusal.
    assert_eq!(ended.status, ExecCommandStatus::Failed);
    assert!(!ended.aggregated_output.is_empty(), "the reason is shown");
    Ok(())
}

/// Declined: the prompt's owner ends it; Core starts and ends nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_466_a_declined_command_gets_no_second_end() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (began, ended) = run_approved(
        json!({ "cmd": "touch declined.txt", "yield_time_ms": 5_000 }),
        ReviewDecision::Denied {
            rejection: "no".to_string(),
        },
    )
    .await?;
    assert_eq!(began, 0, "a declined command does not start");
    assert!(ended.is_empty(), "{ended:?}");
    Ok(())
}
