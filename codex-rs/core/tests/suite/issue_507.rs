//! Issue #507 (from #445, AGG-22): turning the security kill switch on stops
//! what is already running, not only the next turn: a reply still streaming
//! is cancelled, the command it is running is stopped, a command waiting for
//! approval never runs, and background terminals end. The person is told why.
#![cfg(unix)]

use std::path::Path;
use std::time::Duration;

use codex_core::config::Constrained;
use codex_core::security_level_change::LevelBasis;
use codex_core::security_revocation::HumanRevocation;
use codex_core::security_revocation::KILL_SWITCH_STOPPED;
use codex_core::security_revocation::commit_human_revocation;
use codex_core::security_revocation::kill_switch_on;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_output_text_delta;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_wine_exec;
use core_test_support::streaming_sse::StreamingSseChunk;
use core_test_support::streaming_sse::start_streaming_sse_server;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::TestCodexBuilder;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::json;

/// How long a stopped command would have needed to leave its mark.
const COMMAND_SECONDS: u64 = 3;

/// The kill-switch tests check host processes and the host sandbox: their
/// sessions use the local executor on purpose, also in the remote lane (#708).
fn builder(approval: AskForApproval, unified: bool) -> TestCodexBuilder {
    test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = SecurityLevel::Aggressive;
            config.permissions.approval_policy = Constrained::allow_any(approval);
            // Full access, so the marker commands could write where they do:
            // only the kill switch keeps them from running.
            config
                .permissions
                .set_permission_profile(PermissionProfile::Disabled)
                .expect("full access");
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
            if unified {
                config.use_experimental_unified_exec_tool = true;
                config
                    .features
                    .enable(Feature::UnifiedExec)
                    .expect("enable unified exec");
            }
        })
}

/// The person confirms `choice` in `/security`.
fn revoke(test: &TestCodex, choice: HumanRevocation) -> anyhow::Result<()> {
    let home = test.codex_home_path();
    let thread = Some(test.session_configured.thread_id);
    let basis = LevelBasis::read(home, SecurityLevel::Aggressive, thread);
    assert!(basis.live, "the session's policy tree runs in this process");
    let report = commit_human_revocation(
        home,
        SecurityLevel::Aggressive,
        thread,
        &basis,
        choice,
        /*now_unix_seconds*/ 1_000,
    )?;
    assert_eq!(
        report.kill_switch_active,
        choice == HumanRevocation::KillSwitchOn
    );
    assert_eq!(
        kill_switch_on(home),
        choice == HumanRevocation::KillSwitchOn
    );
    Ok(())
}

/// The person turns the kill switch on in `/security`.
fn kill_switch_on_now(test: &TestCodex) -> anyhow::Result<()> {
    revoke(test, HumanRevocation::KillSwitchOn)
}

async fn submit(test: &TestCodex, text: &str) -> anyhow::Result<()> {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: text.into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    Ok(())
}

/// After the switch: the stop warning, then the turn ends as interrupted.
async fn expect_stopped(test: &TestCodex) {
    let warning = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::Warning(warning) if warning.message == KILL_SWITCH_STOPPED)
            || matches!(event, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_))
    })
    .await;
    assert!(
        matches!(warning, EventMsg::Warning(_)),
        "no stop warning first: {warning:?}"
    );
    let ended = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_))
    })
    .await;
    match ended {
        EventMsg::TurnAborted(aborted) => {
            assert_eq!(aborted.reason, TurnAbortReason::Interrupted);
        }
        other => panic!("the turn was not stopped: {other:?}"),
    }
}

/// Wait longer than the command needed, then check it left no mark.
async fn assert_never_ran(marker: &Path) {
    tokio::time::sleep(Duration::from_secs(COMMAND_SECONDS + 2)).await;
    assert!(!marker.exists(), "{} was written", marker.display());
}

fn shell_call(id: &str, command: &str) -> Vec<serde_json::Value> {
    vec![
        ev_response_created(id),
        ev_function_call(
            id,
            "shell_command",
            &json!({ "command": command }).to_string(),
        ),
        ev_completed(id),
    ]
}

fn done() -> Vec<serde_json::Value> {
    vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_kill_switch_stops_the_running_command_and_its_turn() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran-after-kill-switch");
    let command = format!("sleep {COMMAND_SECONDS}; touch '{}'", marker.display());
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, vec![sse(shell_call("call-1", &command))]).await;
    let test = builder(AskForApproval::Never, /*unified*/ false)
        .build(&server)
        .await?;
    submit(&test, "run the slow command").await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::ExecCommandBegin(_))
    })
    .await;

    kill_switch_on_now(&test)?;
    expect_stopped(&test).await;
    assert_never_ran(&marker).await;
    assert_eq!(
        captured.requests().len(),
        1,
        "the command's output never went back to the model"
    );
    Ok(())
}

/// Another Corbanu Terminal process on the same home turns the switch on or
/// off: it saves `security_state.json` there and reaches no session of this
/// process directly. Its save is made in a scratch home and moved into place
/// the way a commit writes it (an atomic replace).
fn other_process_switch(
    home: &Path,
    scratch: &Path,
    choice: HumanRevocation,
    now: i64,
) -> anyhow::Result<()> {
    if let Ok(saved) = std::fs::read(home.join("security_state.json")) {
        std::fs::write(scratch.join("security_state.json"), saved)?;
    }
    let basis = LevelBasis::read(scratch, SecurityLevel::Aggressive, /*thread*/ None);
    commit_human_revocation(
        scratch,
        SecurityLevel::Aggressive,
        /*thread*/ None,
        &basis,
        choice,
        now,
    )?;
    let staged = home.join("security_state.json.other-process");
    std::fs::copy(scratch.join("security_state.json"), &staged)?;
    std::fs::rename(&staged, home.join("security_state.json"))?;
    Ok(())
}

/// #507 (Travis, 2026-10-12): a kill switch another Corbanu Terminal process
/// on the same home turns on stops the command running here within the
/// stated latency (about a second, at most 2 s), and turning it off there
/// reaches this process too.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_kill_switch_from_another_process_stops_the_running_command() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran-after-kill-switch");
    let command = format!(
        "sleep {}; touch '{}'",
        COMMAND_SECONDS * 2,
        marker.display()
    );
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, vec![sse(shell_call("call-1", &command))]).await;
    let test = builder(AskForApproval::Never, /*unified*/ false)
        .build(&server)
        .await?;
    let home = test.codex_home_path().to_path_buf();
    submit(&test, "run the slow command").await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::ExecCommandBegin(_))
    })
    .await;

    let other = tempfile::TempDir::new()?;
    let started = std::time::Instant::now();
    other_process_switch(
        &home,
        other.path(),
        HumanRevocation::KillSwitchOn,
        /*now*/ 1_000,
    )?;
    expect_stopped(&test).await;
    let latency = started.elapsed();
    assert!(
        latency < Duration::from_secs(2),
        "stopped after {latency:?}"
    );
    assert!(kill_switch_on(&home));
    tokio::time::sleep(Duration::from_secs(COMMAND_SECONDS * 2 + 1)).await;
    assert!(!marker.exists(), "{} was written", marker.display());
    assert_eq!(captured.requests().len(), 1);

    other_process_switch(
        &home,
        other.path(),
        HumanRevocation::KillSwitchOff,
        /*now*/ 1_001,
    )?;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while kill_switch_on(&home) && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(!kill_switch_on(&home), "turning it off did not arrive");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_kill_switch_cancels_a_streaming_reply_and_its_tool_calls() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran-after-kill-switch");
    let (first_gate, first_gate_rx) = tokio::sync::oneshot::channel();
    let (rest_gate, rest_gate_rx) = tokio::sync::oneshot::channel();
    let first = core_test_support::responses::sse(vec![
        ev_response_created("resp-1"),
        ev_output_text_delta("Working on it"),
    ]);
    let rest = core_test_support::responses::sse(vec![
        ev_function_call(
            "call-late",
            "shell_command",
            &json!({ "command": format!("touch '{}'", marker.display()) }).to_string(),
        ),
        ev_completed("resp-1"),
    ]);
    let (server, _completions) = start_streaming_sse_server(vec![vec![
        StreamingSseChunk {
            gate: Some(first_gate_rx),
            body: first,
        },
        StreamingSseChunk {
            gate: Some(rest_gate_rx),
            body: rest,
        },
    ]])
    .await;
    let test = builder(AskForApproval::Never, /*unified*/ false)
        .build_with_streaming_server(&server)
        .await?;
    submit(&test, "start a reply").await?;
    let _ = first_gate.send(());
    server.wait_for_request_count(/*count*/ 1).await;
    // The reply has started streaming and waits for the rest.
    tokio::time::sleep(Duration::from_millis(500)).await;

    kill_switch_on_now(&test)?;
    expect_stopped(&test).await;
    // The rest of the reply, with its tool call, arrives too late to run.
    let _ = rest_gate.send(());
    assert_never_ran(&marker).await;
    assert_eq!(server.requests().await.len(), 1, "nothing more was sent");
    server.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_a_command_waiting_for_approval_never_runs() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran-after-kill-switch");
    let command = format!("touch '{}'", marker.display());
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, vec![sse(shell_call("call-1", &command))]).await;
    let test = builder(AskForApproval::UnlessTrusted, /*unified*/ false)
        .build(&server)
        .await?;
    submit(&test, "write the marker").await?;
    let approval = match wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    {
        EventMsg::ExecApprovalRequest(approval) => approval,
        other => panic!("no approval prompt: {other:?}"),
    };

    kill_switch_on_now(&test)?;
    expect_stopped(&test).await;
    // An approval given after the switch reaches nothing.
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    assert_never_ran(&marker).await;
    assert_eq!(captured.requests().len(), 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_kill_switch_ends_background_terminals() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran-after-kill-switch");
    let command = format!("sleep {COMMAND_SECONDS}; touch '{}'", marker.display());
    let server = start_mock_server().await;
    mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-bg"),
                ev_function_call(
                    "call-bg",
                    "exec_command",
                    &json!({ "cmd": command, "yield_time_ms": 250 }).to_string(),
                ),
                ev_completed("call-bg"),
            ]),
            sse(done()),
        ],
    )
    .await;
    let test = builder(AskForApproval::Never, /*unified*/ true)
        .build(&server)
        .await?;
    submit(&test, "start it in the background").await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    assert!(!marker.exists(), "the background command is still running");

    kill_switch_on_now(&test)?;
    assert_never_ran(&marker).await;
    Ok(())
}

/// Control for the tests above: without the kill switch, the same background
/// command does leave its mark, so a missing mark means it was stopped.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_control_background_command_runs_without_the_kill_switch() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran");
    let command = format!("sleep {COMMAND_SECONDS}; touch '{}'", marker.display());
    let server = start_mock_server().await;
    mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-bg"),
                ev_function_call(
                    "call-bg",
                    "exec_command",
                    &json!({ "cmd": command, "yield_time_ms": 250 }).to_string(),
                ),
                ev_completed("call-bg"),
            ]),
            sse(done()),
        ],
    )
    .await;
    let test = builder(AskForApproval::Never, /*unified*/ true)
        .build(&server)
        .await?;
    submit(&test, "start it in the background").await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    tokio::time::sleep(Duration::from_secs(COMMAND_SECONDS + 2)).await;
    assert!(marker.exists(), "the control command did not run");
    Ok(())
}

/// "Revoke all" ends grants and approvals but leaves running work alone, as
/// its review says; only the kill switch stops it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_revoke_all_leaves_the_running_command_alone() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_wine_exec!(
        Ok(()),
        "this security test runs host shell commands; Wine-exec routes them to a Windows executor with a non-native cwd"
    );
    let marker_dir = tempfile::TempDir::new()?;
    let marker = marker_dir.path().join("ran");
    let command = format!("sleep {COMMAND_SECONDS}; touch '{}'", marker.display());
    let server = start_mock_server().await;
    mount_sse_sequence(
        &server,
        vec![sse(shell_call("call-1", &command)), sse(done())],
    )
    .await;
    let test = builder(AskForApproval::Never, /*unified*/ false)
        .build(&server)
        .await?;
    submit(&test, "run the slow command").await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::ExecCommandBegin(_))
    })
    .await;

    revoke(&test, HumanRevocation::AllActiveAuthority)?;
    let ended = wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_))
            || matches!(event, EventMsg::Warning(warning) if warning.message == KILL_SWITCH_STOPPED)
    })
    .await;
    assert!(matches!(ended, EventMsg::TurnComplete(_)), "{ended:?}");
    assert!(marker.exists(), "the command finished");
    Ok(())
}
