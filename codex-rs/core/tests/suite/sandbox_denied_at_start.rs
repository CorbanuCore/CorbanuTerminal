//! Issue #443 (milestone AGG-04): a Corbanu-home read that the sandbox
//! stopped as it started ran without ever appearing on screen.
#![cfg(target_os = "macos")]

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_sandbox;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use serde_json::json;

/// The milestone's other AGG-04 observation: a Corbanu-home read that the
/// sandbox stopped as it started ran without ever appearing on screen. It is
/// now shown like any other command: begin, then end with its output.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_443_a_command_the_sandbox_stops_at_start_is_shown() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let server = start_mock_server().await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.security_level = SecurityLevel::Aggressive;
            config.use_experimental_unified_exec_tool = true;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            for feature in [Feature::SourceEnvelopes, Feature::UnifiedExec] {
                config
                    .features
                    .enable(feature)
                    .expect("test config should allow feature update");
            }
        })
        .build_with_auto_env(&server)
        .await?;
    let canary = test.codex_home_path().canonicalize()?.join(".env");
    std::fs::write(&canary, "CANARY=pf443\n")?;
    let call_id = "call-home-read";
    let command = format!("head -3 '{}'", canary.display());
    mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("read"),
                ev_function_call(
                    call_id,
                    "exec_command",
                    &json!({ "cmd": command, "yield_time_ms": 5_000 }).to_string(),
                ),
                ev_completed("read"),
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
                text: "read the Corbanu home file".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut began = false;
    let mut ended = None;
    wait_for_event(&test.codex, |event| match event {
        EventMsg::ExecCommandBegin(begin) if begin.call_id == call_id => {
            began = true;
            false
        }
        EventMsg::ExecCommandEnd(end) if end.call_id == call_id => {
            ended = Some(end.clone());
            false
        }
        EventMsg::TurnComplete(_) => true,
        _ => false,
    })
    .await;
    assert!(began, "the command is shown when it starts");
    let ended = ended.expect("the command is shown when it ends");
    assert_ne!(ended.exit_code, 0);
    assert!(
        ended.aggregated_output.contains("Operation not permitted"),
        "{}",
        ended.aggregated_output
    );
    assert!(!ended.aggregated_output.contains("pf443"));
    Ok(())
}
