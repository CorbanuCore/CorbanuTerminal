use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

#[test]
fn tool_requests_are_parsed_and_answered() {
    let request = json!({
        "type": "control_request",
        "request_id": "req-1",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Bash",
            "input": { "command": "echo hi > hello.txt", "description": "write" },
        },
    });
    assert!(is_control_request(&request));
    let (request_id, tool_name, input) = tool_request(&request).expect("tool request");
    assert_eq!((request_id, tool_name), ("req-1", "Bash"));
    assert_eq!(summary(tool_name, input), "echo hi > hello.txt");
    assert_eq!(
        tool_response(request_id, /*allow*/ true, input),
        json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": "req-1",
                "response": { "behavior": "allow", "updatedInput": input },
            },
        })
    );
    assert_eq!(
        tool_response(request_id, /*allow*/ false, input)["response"]["response"]["behavior"],
        json!("deny")
    );
}

#[test]
fn other_control_requests_are_refused() {
    let request = json!({
        "type": "control_request",
        "request_id": "req-2",
        "request": { "subtype": "mcp_message", "server_name": "x" },
    });
    assert_eq!(tool_request(&request), None);
    assert_eq!(
        unsupported_response(&request).expect("response")["response"]["subtype"],
        json!("error")
    );
    assert!(!is_control_request(&json!({ "type": "assistant" })));
}

#[test]
fn summaries_show_the_target_and_stay_short() {
    assert_eq!(
        summary("Write", &json!({ "file_path": "/w/a.txt", "content": "x" })),
        "/w/a.txt"
    );
    assert_eq!(
        summary("WebFetch", &json!({ "url": "https://example.com" })),
        "https://example.com"
    );
    assert_eq!(summary("Task", &json!({ "a": 1 })), "{\"a\":1}");
    let long = summary("Bash", &json!({ "command": "x".repeat(1_000) }));
    assert_eq!(long.chars().count(), SUMMARY_MAX_CHARS + 1);
    assert!(long.ends_with('…'));
}

#[tokio::test]
async fn a_responder_answers_once() {
    let (responder, rx) = ApprovalResponder::new();
    responder.clone().respond(/*allow*/ true);
    responder.respond(/*allow*/ false);
    assert_eq!(rx.await, Ok(true));
}

#[test]
fn escape_denies() {
    let (responder, mut rx) = ApprovalResponder::new();
    let view = selection_view(ClaudeApprovalRequest {
        pane_title: "Claude Code - GLM 5.2 Z.AI".to_string(),
        tool_name: "Bash".to_string(),
        summary: "touch x".to_string(),
        responder,
    });
    assert_eq!(
        view.title.as_deref(),
        Some("Claude Code - GLM 5.2 Z.AI wants to use Bash")
    );
    assert!(!view.allow_number_shortcuts);
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let sender = crate::app_event_sender::AppEventSender::new(tx);
    (view.on_cancel.expect("cancel"))(&sender);
    assert_eq!(rx.try_recv(), Ok(false));
}
