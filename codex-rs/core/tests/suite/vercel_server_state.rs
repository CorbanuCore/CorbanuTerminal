//! Regression coverage for the Vercel Responses HTTP server-state
//! continuation path.
//!
//! The Vercel gateway rejects `previous_response_id` continuations whose
//! `input` contains no user-role message
//! (`invalid_request_error`: "At least one user message is required in the
//! input"). That shape occurs naturally on tool-call follow-ups, which used
//! to hard-fail spawned-agent turns after tool execution. See
//! `pfterminal_vercel_responses_tool_continuation_bug_proposal_20260707.md`.

use anyhow::Result;
use core_test_support::responses;
use core_test_support::responses::ResponsesRequest;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_sequence;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodexBuilder;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use serde_json::Value;
use wiremock::ResponseTemplate;

const VERCEL_PROVIDER_NAME: &str = "Vercel";

fn vercel_test_codex() -> TestCodexBuilder {
    test_codex().with_config(|config| {
        // `is_vercel()` is name-based; this opts the HTTP transport into the
        // server-state incremental continuation path while keeping the mock
        // server base URL.
        config.model_provider.name = VERCEL_PROVIDER_NAME.to_string();
    })
}

fn previous_response_id(request: &ResponsesRequest) -> Option<String> {
    request
        .body_json()
        .get("previous_response_id")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn input_items(request: &ResponsesRequest) -> Vec<Value> {
    request
        .body_json()
        .get("input")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn input_has_user_message(request: &ResponsesRequest) -> bool {
    input_items(request).iter().any(|item| {
        item.get("type").and_then(Value::as_str) == Some("message")
            && item.get("role").and_then(Value::as_str) == Some("user")
    })
}

fn input_has_function_call_output(request: &ResponsesRequest, call_id: &str) -> bool {
    input_items(request).iter().any(|item| {
        item.get("type").and_then(Value::as_str) == Some("function_call_output")
            && item.get("call_id").and_then(Value::as_str) == Some(call_id)
    })
}

/// Tool-call follow-ups must never be sent as `previous_response_id`
/// continuations without a user message; they fall back to full context for
/// that request. A later user-initiated turn must still get the incremental
/// continuation (state is not poisoned by the fallback).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vercel_tool_followup_falls_back_to_full_context_and_keeps_state() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let call_id = "call-vercel-tool";
    let response_mock = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp1"),
                ev_function_call(call_id, "nonexistent_tool", "{}"),
                ev_completed("resp1"),
            ]),
            sse(vec![
                ev_response_created("resp2"),
                ev_assistant_message("msg-1", "tool handled"),
                ev_completed("resp2"),
            ]),
            sse(vec![
                ev_response_created("resp3"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp3"),
            ]),
        ],
    )
    .await;

    let mut builder = vercel_test_codex();
    let test = builder.build(&server).await?;
    test.submit_turn("please run the tool").await?;
    test.submit_turn("thanks, wrap up").await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 3);

    // Initial request: full context, no continuation.
    assert_eq!(previous_response_id(&requests[0]), None);
    assert!(input_has_user_message(&requests[0]));

    // Tool follow-up: the incremental delta is tool output only, so the
    // request must fall back to full context instead of an incompatible
    // `previous_response_id` continuation.
    assert!(
        input_has_function_call_output(&requests[1], call_id),
        "tool follow-up request should carry the function_call_output"
    );
    assert_eq!(
        previous_response_id(&requests[1]),
        None,
        "tool-output-only follow-up must not use previous_response_id"
    );
    assert!(
        input_has_user_message(&requests[1]),
        "full-context fallback must include a user message"
    );

    // Next user-initiated turn: incremental continuation resumes; the
    // fallback did not poison the server-conversation state.
    assert_eq!(
        previous_response_id(&requests[2]).as_deref(),
        Some("resp2"),
        "user-initiated turn should continue incrementally from the last response"
    );
    assert!(input_has_user_message(&requests[2]));
    Ok(())
}

/// A visible assistant commentary message before a tool call requires a
/// synthetic user turn on Vercel's wire. That wire-only item must not enter
/// the canonical continuation baseline or force later full-context replays.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vercel_commentary_tool_followups_keep_incremental_server_state() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let first_call_id = "call-vercel-first";
    let second_call_id = "call-vercel-second";
    let response_mock = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp1"),
                ev_assistant_message("msg-1", "I will inspect this first."),
                ev_function_call(first_call_id, "nonexistent_tool", "{}"),
                ev_completed("resp1"),
            ]),
            sse(vec![
                ev_response_created("resp2"),
                ev_function_call(second_call_id, "nonexistent_tool", "{}"),
                ev_completed("resp2"),
            ]),
            sse(vec![
                ev_response_created("resp3"),
                ev_assistant_message("msg-2", "done"),
                ev_completed("resp3"),
            ]),
        ],
    )
    .await;

    let mut builder = vercel_test_codex();
    let test = builder.build(&server).await?;
    test.submit_turn("please inspect the repository").await?;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 3);
    assert_eq!(previous_response_id(&requests[0]), None);

    assert_eq!(previous_response_id(&requests[1]).as_deref(), Some("resp1"));
    assert!(input_has_function_call_output(&requests[1], first_call_id));
    assert!(input_has_user_message(&requests[1]));

    assert_eq!(previous_response_id(&requests[2]).as_deref(), Some("resp2"));
    assert!(input_has_function_call_output(&requests[2], second_call_id));
    assert!(input_has_user_message(&requests[2]));
    Ok(())
}

/// If the provider still rejects a server-state continuation with HTTP 400,
/// the client clears the stored state and retries once with full context
/// instead of failing the turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vercel_server_state_rejection_self_heals_with_full_context_retry() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let sse_ok = |response_id: &str, text: &str| {
        ResponseTemplate::new(200)
            .insert_header("content-type", "text/event-stream")
            .set_body_raw(
                sse(vec![
                    ev_response_created(response_id),
                    ev_assistant_message(&format!("msg-{response_id}"), text),
                    ev_completed(response_id),
                ]),
                "text/event-stream",
            )
    };
    let response_mock = mount_response_sequence(
        &server,
        vec![
            sse_ok("resp1", "hi"),
            ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": {
                    "type": "invalid_request_error",
                    "message": "At least one user message is required in the input"
                }
            })),
            sse_ok("resp2", "recovered"),
        ],
    )
    .await;

    let mut builder = vercel_test_codex();
    let test = builder.build(&server).await?;
    test.submit_turn("hello").await?;

    // Second turn: sent as a previous_response_id continuation, rejected with
    // 400, then retried once with full context. The turn must complete
    // without surfacing an error event.
    test.submit_turn("continue please").await?;
    let requests = response_mock.requests();
    assert_eq!(requests.len(), 3);

    assert_eq!(previous_response_id(&requests[0]), None);
    assert_eq!(
        previous_response_id(&requests[1]).as_deref(),
        Some("resp1"),
        "second turn should first attempt a server-state continuation"
    );
    assert_eq!(
        previous_response_id(&requests[2]),
        None,
        "the 400 retry must be a full-context request"
    );
    assert!(
        input_has_user_message(&requests[2]),
        "the retry must carry the user messages"
    );
    Ok(())
}
