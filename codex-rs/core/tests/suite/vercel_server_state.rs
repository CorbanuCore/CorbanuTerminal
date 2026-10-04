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

fn input_has_synthetic_continue(request: &ResponsesRequest) -> bool {
    input_items(request).iter().any(|item| {
        item.get("role").and_then(Value::as_str) == Some("user")
            && item
                .get("content")
                .and_then(Value::as_array)
                .is_some_and(|parts| {
                    parts
                        .iter()
                        .any(|part| part.get("text").and_then(Value::as_str) == Some("Continue."))
                })
    })
}

/// A visible assistant commentary message before a tool call is not a prefill: the request ends
/// in tool output. It must not gain a synthetic `Continue.` user turn, which models read as the
/// user interjecting after every tool result.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vercel_commentary_tool_followups_carry_no_synthetic_user_turn() -> Result<()> {
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
    for (index, call_id) in [(1, first_call_id), (2, second_call_id)] {
        assert!(input_has_function_call_output(&requests[index], call_id));
        assert!(
            !input_has_synthetic_continue(&requests[index]),
            "tool follow-up {index} must not carry a synthetic Continue. turn"
        );
        // Vercel requires a user message in a previous_response_id continuation, so a tool
        // follow-up without one is sent with full context.
        assert_eq!(previous_response_id(&requests[index]), None);
        assert!(input_has_user_message(&requests[index]));
    }
    Ok(())
}

/// A provider that still rejects a tool continuation ending in tool output after assistant
/// commentary gets one retry with the legacy synthetic turn, and the session keeps using it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vercel_rejected_tool_continuation_restores_synthetic_turn_for_session() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let sse_template = |events: Vec<Value>| {
        ResponseTemplate::new(200)
            .insert_header("content-type", "text/event-stream")
            .set_body_raw(sse(events), "text/event-stream")
    };
    let response_mock = mount_response_sequence(
        &server,
        vec![
            sse_template(vec![
                ev_response_created("resp1"),
                ev_assistant_message("msg-1", "I will inspect this first."),
                ev_function_call("call-a", "nonexistent_tool", "{}"),
                ev_completed("resp1"),
            ]),
            ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": {
                    "type": "invalid_request_error",
                    "message": "The conversation must end with a user message."
                }
            })),
            sse_template(vec![
                ev_response_created("resp2"),
                ev_function_call("call-b", "nonexistent_tool", "{}"),
                ev_completed("resp2"),
            ]),
            sse_template(vec![
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
    assert_eq!(requests.len(), 4, "only one rejected request may be spent");
    assert!(!input_has_synthetic_continue(&requests[1]));
    assert!(
        input_has_synthetic_continue(&requests[2]),
        "the retry restores the synthetic turn"
    );
    assert!(
        input_has_synthetic_continue(&requests[3]),
        "the session keeps the legacy shape after a rejection"
    );
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

/// Some gateway/model pairs (Kimi K3 on Vercel) reject every
/// `previous_response_id` continuation. After the first rejection the session
/// must stay on full-context requests instead of paying a failed round trip on
/// every later turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vercel_server_state_rejection_is_sticky_for_the_session() -> Result<()> {
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
                    "type": "AI_APICallError",
                    "message": "Kimi K3 tool messages need a resolvable tool name"
                }
            })),
            sse_ok("resp2", "recovered"),
            sse_ok("resp3", "still fine"),
        ],
    )
    .await;

    let mut builder = vercel_test_codex();
    let test = builder.build(&server).await?;
    test.submit_turn("hello").await?;
    test.submit_turn("continue please").await?;
    test.submit_turn("and once more").await?;

    let requests = response_mock.requests();
    assert_eq!(
        requests.len(),
        4,
        "only the first continuation may be rejected"
    );
    assert_eq!(previous_response_id(&requests[1]).as_deref(), Some("resp1"));
    assert_eq!(previous_response_id(&requests[2]), None);
    assert_eq!(
        previous_response_id(&requests[3]),
        None,
        "after a rejection the session must not retry server-state continuations"
    );
    assert!(input_has_user_message(&requests[3]));
    Ok(())
}
