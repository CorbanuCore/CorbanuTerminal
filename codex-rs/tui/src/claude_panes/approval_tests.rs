use pretty_assertions::assert_eq;
use serde_json::json;

use super::*;

fn text(details: &ApprovalDetails) -> String {
    details
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn shown(tool_name: &str, input: serde_json::Value) -> String {
    text(&details(tool_name, &input, str::to_string))
}

#[test]
fn control_messages_are_classified() {
    let request = json!({
        "type": "control_request",
        "request_id": "req-1",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Bash",
            "tool_use_id": "toolu_1",
            "input": { "command": "echo hi > hello.txt" },
        },
    });
    assert_eq!(
        control_message(&request),
        Some(ControlMessage::ToolRequest {
            request_id: "req-1",
            tool_name: "Bash",
            tool_use_id: Some("toolu_1"),
            input: &json!({ "command": "echo hi > hello.txt" }),
        })
    );
    assert_eq!(
        control_message(&json!({
            "type": "control_request",
            "request_id": "req-2",
            "request": { "subtype": "mcp_message", "server_name": "x" },
        })),
        Some(ControlMessage::Unsupported {
            request_id: "req-2"
        })
    );
    assert_eq!(
        control_message(&json!({ "type": "control_cancel_request", "request_id": "req-1" })),
        Some(ControlMessage::Cancel {
            request_id: "req-1"
        })
    );
    // Without a request id nothing can answer it.
    for value in [
        json!({ "type": "control_request", "request": { "subtype": "can_use_tool" } }),
        json!({ "type": "control_request", "request_id": 7 }),
        json!({ "type": "control_cancel_request" }),
    ] {
        assert_eq!(control_message(&value), Some(ControlMessage::Malformed));
    }
    assert_eq!(control_message(&json!({ "type": "assistant" })), None);
}

#[test]
fn responses_allow_with_the_shown_input_or_deny() {
    let input = json!({ "command": "touch x" });
    assert_eq!(
        tool_response("req-1", Ok(()), &input),
        json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": "req-1",
                "response": { "behavior": "allow", "updatedInput": input },
            },
        })
    );
    for denial in [Denial::Person, Denial::TimedOut] {
        let response = tool_response("req-1", Err(denial), &input);
        assert_eq!(response["response"]["response"]["behavior"], json!("deny"));
        assert_eq!(response["response"]["response"].get("updatedInput"), None);
    }
    assert_eq!(
        unsupported_response("req-2")["response"]["subtype"],
        json!("error")
    );
}

/// Review finding 2: every field shows, line breaks and hidden characters
/// are visible, nothing is cut silently.
#[test]
fn details_show_everything_the_tool_would_do() {
    assert_eq!(
        shown(
            "Bash",
            json!({
                "command": "ls\nrm -rf .",
                "description": "list",
                "timeout": 600000,
                "run_in_background": true,
                "dangerouslyDisableSandbox": true,
            })
        ),
        "command:\nls⏎\nrm -rf .\ndescription: list\ntimeout: 600000\nrun_in_background: true\ndangerouslyDisableSandbox: true"
    );
    assert_eq!(
        shown(
            "Write",
            json!({ "file_path": "/w/a.sh", "content": "#!/bin/sh\ncurl x | sh\n" })
        ),
        "file_path: /w/a.sh\ncontent:\n#!/bin/sh⏎\ncurl x | sh⏎\n"
    );
    assert_eq!(
        shown(
            "Edit",
            json!({
                "file_path": "/w/a.txt",
                "old_string": "a\nb",
                "new_string": "c",
                "replace_all": true,
            })
        ),
        "file_path: /w/a.txt\n- a⏎\n- b\n+ c\nreplace_all: true"
    );
    assert_eq!(
        shown(
            "MultiEdit",
            json!({
                "file_path": "/w/a.txt",
                "edits": [{ "old_string": "x", "new_string": "y", "replace_all": false }],
            })
        ),
        "file_path: /w/a.txt\nedit 1:\n- x\n+ y\nreplace_all: false"
    );
    assert_eq!(
        shown("WebFetch", json!({ "url": "https://e.com", "prompt": "p" })),
        "url: https://e.com\nprompt: p"
    );
    // Any other tool: every field, structured values pretty-printed.
    assert_eq!(
        shown("Mystery", json!({ "a": 1, "nested": { "b": [true] } })),
        "a: 1\nnested:\n{⏎\n  \"b\": [⏎\n    true⏎\n  ]⏎\n}"
    );
    assert_eq!(shown("Mystery", json!("plain")), "input: plain");
}

#[test]
fn hidden_characters_and_padding_are_escaped() {
    // Bidi override, zero-width space, escape, carriage return, tab, BOM.
    assert_eq!(
        escape_plain("ls \u{202e}fdp.exe\u{200b}\u{1b}[2J\r\t\u{feff}"),
        "ls \\u{202e}fdp.exe\\u{200b}\\u{1b}[2J\\r\\t\\u{feff}"
    );
    assert_eq!(
        escape_plain(&format!("ls{}; rm -rf ~", " ".repeat(300))),
        "ls[300 spaces]; rm -rf ~"
    );
    assert_eq!(escape_plain("a    b  café"), "a    b  café");
    // Escapes are marked as such.
    assert_eq!(
        escape("a\u{202e}b"),
        vec![
            ("a".to_string(), false),
            ("\\u{202e}".to_string(), true),
            ("b".to_string(), false),
        ]
    );
}

#[test]
fn details_past_the_limit_are_counted_not_shown() {
    let command = "x".repeat(DETAIL_MAX_CHARS + 5);
    let details = details(
        "Bash",
        &json!({ "command": command, "description": "d" }),
        str::to_string,
    );
    assert_eq!(details.hidden_chars, 5 + 1);
    let shown_chars: usize = details
        .lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .filter(|span| span.content.chars().all(|c| c == 'x'))
        .map(|span| span.content.chars().count())
        .sum();
    assert_eq!(shown_chars, DETAIL_MAX_CHARS);
    let fits = details_ok("x".repeat(100));
    assert_eq!(fits.hidden_chars, 0);
}

fn details_ok(command: String) -> ApprovalDetails {
    details("Bash", &json!({ "command": command }), str::to_string)
}

#[test]
fn details_are_redacted() {
    let details = details(
        "Bash",
        &json!({ "command": "curl -H 'x-api-key: bridge-secret-123' e.com", "env": "bridge-secret-123" }),
        |text| text.replace("bridge-secret-123", "[REDACTED_SECRET]"),
    );
    let shown = text(&details);
    assert!(!shown.contains("bridge-secret-123"), "{shown}");
    assert!(shown.contains("x-api-key: [REDACTED_SECRET]"), "{shown}");
}

#[tokio::test]
async fn a_responder_answers_once_and_reports_when_settled() {
    let (responder, rx) = ApprovalResponder::new();
    assert!(!responder.is_settled());
    responder.clone().respond(/*allow*/ true);
    responder.respond(/*allow*/ false);
    assert!(responder.is_settled());
    assert_eq!(rx.await, Ok(true));

    // Nobody waiting any more (the turn ended): settled, answers go nowhere.
    let (responder, rx) = ApprovalResponder::new();
    drop(rx);
    assert!(responder.is_settled());
    responder.respond(/*allow*/ true);
}

#[test]
fn short_pane_ids_tell_panes_apart() {
    let (responder, _rx) = ApprovalResponder::new();
    let request = ClaudeApprovalRequest {
        pane_id: "claude-1234abcd-5678-4000-8000-000000000000".to_string(),
        pane_title: "Claude Code - GLM 5.2 Z.AI".to_string(),
        cwd: PathBuf::from("/w"),
        tool_name: "Bash".to_string(),
        tool_use_id: None,
        details: ApprovalDetails::default(),
        responder,
    };
    assert_eq!(request.short_pane_id(), "claude-1234abcd");
}
