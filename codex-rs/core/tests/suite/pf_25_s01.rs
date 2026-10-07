//! PF-25-S01: an Aggressive grant confirmed for a command approval lifts the
//! protected-path rules for that run only, end to end through the
//! orchestrator (offer key = the approval id the TUI answers, digest at
//! offer = digest at run).
#![cfg(target_os = "macos")]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_grant;
use codex_core::security_grant::GrantUses;
use codex_features::Feature;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecApprovalRequestEvent;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ResponseMock;
use core_test_support::responses::ResponsesRequest;
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
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

const CANARY: &str = "pf25-canary-notes";

type Step = Vec<Value>;

fn shell_step(call_id: &str, command: &str) -> Step {
    let arguments = json!({ "command": command }).to_string();
    vec![
        ev_response_created(call_id),
        ev_function_call(call_id, "shell_command", &arguments),
        ev_completed(call_id),
    ]
}

fn done_step() -> Step {
    vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]
}

/// A file in the Corbanu home, and a command that reads it without naming
/// the home folder.
fn canary_home() -> anyhow::Result<(Arc<TempDir>, String)> {
    let home = Arc::new(TempDir::new()?);
    let path = home.path().canonicalize()?.join("pf25-notes.txt");
    std::fs::write(&path, CANARY)?;
    let path = path.to_string_lossy().into_owned();
    let folder = path.len() - "/pf25-notes.txt".len();
    let (head, tail) = (&path[..folder - 2], &path[folder - 2..folder]);
    Ok((
        home,
        format!("cat \"$(printf '{head}%s/pf25-notes.txt' {tail})\""),
    ))
}

async fn start(home: Arc<TempDir>, steps: Vec<Step>) -> anyhow::Result<(TestCodex, ResponseMock)> {
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.security_level = SecurityLevel::Aggressive;
            config.permissions.approval_policy =
                Constrained::allow_any(AskForApproval::UnlessTrusted);
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
        .build_with_auto_env(&server)
        .await?;
    std::mem::forget(server);
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "read the notes".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    Ok((test, captured))
}

async fn next_approval(test: &TestCodex) -> Option<ExecApprovalRequestEvent> {
    match wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    {
        EventMsg::ExecApprovalRequest(approval) => Some(approval),
        _ => None,
    }
}

async fn approve(test: &TestCodex, approval: &ExecApprovalRequestEvent) -> anyhow::Result<()> {
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::Approved,
        })
        .await?;
    Ok(())
}

fn output_text(request: &ResponsesRequest, call_id: &str) -> String {
    request
        .function_call_output_text(call_id)
        .unwrap_or_default()
}

/// The first run, granted "1 run" in the review and approved, reads the
/// file; the same command again, only approved, gets the rules.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_25_s01_confirmed_grant_lifts_the_rules_for_the_approved_run_only() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (home, read) = canary_home()?;
    let (test, captured) = start(
        home,
        vec![
            shell_step("call-granted", &read),
            shell_step("call-again", &read),
            done_step(),
        ],
    )
    .await?;
    let thread = test.session_configured.thread_id;

    let first = next_approval(&test).await.expect("the read asks first");
    let offer = security_grant::offer(thread, &first.effective_approval_id())
        .expect("Core offers a grant for the approval the TUI answers");
    assert_eq!(offer.command, first.command);
    security_grant::confirm(&offer, GrantUses::Once)?;
    approve(&test, &first).await?;

    let second = next_approval(&test)
        .await
        .expect("the same read asks again");
    assert!(security_grant::offer(thread, &second.effective_approval_id()).is_some());
    approve(&test, &second).await?;
    assert!(next_approval(&test).await.is_none());

    let requests = captured.requests();
    let granted = output_text(&requests[requests.len() - 2], "call-granted");
    assert!(granted.contains(CANARY), "{granted}");
    let again = output_text(&requests[requests.len() - 1], "call-again");
    assert!(!again.contains(CANARY), "{again}");
    assert!(again.contains("Operation not permitted"), "{again}");
    assert!(
        security_grant::held_grants()
            .iter()
            .all(|grant| grant.thread != thread),
        "a 1-run grant is never held"
    );
    Ok(())
}

/// A grant confirmed in the review but declined in the approval does not
/// apply: the command does not run, and nothing is held.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_25_s01_declined_approval_drops_the_confirmed_grant() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let (home, read) = canary_home()?;
    let (test, captured) = start(home, vec![shell_step("call-read", &read), done_step()]).await?;
    let thread = test.session_configured.thread_id;
    let approval = next_approval(&test).await.expect("asks first");
    let offer = security_grant::offer(thread, &approval.effective_approval_id()).expect("offer");
    security_grant::confirm(&offer, GrantUses::UntilExpiry)?;
    test.codex
        .submit(Op::ExecApproval {
            id: approval.effective_approval_id(),
            turn_id: None,
            decision: ReviewDecision::denied("declined"),
        })
        .await?;
    assert!(next_approval(&test).await.is_none());
    assert!(security_grant::offer(thread, &approval.effective_approval_id()).is_none());
    assert!(
        security_grant::held_grants()
            .iter()
            .all(|grant| grant.thread != thread)
    );
    let requests = captured.requests();
    let output = output_text(&requests[requests.len() - 1], "call-read");
    assert!(!output.contains(CANARY), "{output}");
    Ok(())
}
