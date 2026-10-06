use super::*;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::AgentMessageContentDeltaEvent;
use codex_protocol::protocol::ErrorEvent;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecOutputStream;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::protocol::TurnAbortedEvent;
use pretty_assertions::assert_eq;
use std::ffi::OsString;

// Synthetic canaries only.
const CANARY: &str = "pf28-core-canary-7Hq2Lw9X";

fn gate() -> OutputGate {
    let gate = OutputGate::new();
    gate.register("env:PF28_CANARY_API_KEY", SecretClass::Operational, CANARY)
        .expect("register");
    gate
}

fn labels(values: &Labeled) -> Vec<&str> {
    values.iter().map(|(label, _)| label.as_str()).collect()
}

#[test]
fn pf_28_s01_env_values_are_credentials_not_session_plumbing() {
    let vars = [
        ("PF28_CANARY_API_KEY", CANARY),
        ("GITHUB_TOKEN", "ghp_pf28syntheticsyntheticsynthetic0000"),
        ("XDG_SESSION_DESKTOP", "ubuntu-wayland"),
        ("DESKTOP_SESSION", "ubuntu-wayland"),
        ("SSH_AUTH_SOCK", "/tmp/ssh-pf28/agent.1"),
        ("ITERM_SESSION_ID", "short"),
        ("PATH", "/usr/bin:/bin"),
        (
            "DATABASE_URL",
            "postgres://app:pf28-db-password@db.local/app",
        ),
        (
            "ALERTS_WEBHOOK_URL",
            "https://hooks.example.com/in?channel=ops&token=pf28-query-token-5555",
        ),
    ]
    .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    let values = env_values(vars);
    assert_eq!(
        labels(&values),
        vec![
            "env:PF28_CANARY_API_KEY",
            "env:GITHUB_TOKEN",
            "env:DATABASE_URL:password",
            "env:ALERTS_WEBHOOK_URL:token",
        ]
    );
    assert_eq!(values[2].1.as_str(), "pf28-db-password");
}

#[test]
fn pf_28_s01_ordinary_settings_are_not_seeded() {
    let vars = [
        ("GIT_AUTHOR_NAME", "Pat Example"),
        ("GIT_AUTHOR_EMAIL", "pat@example.com"),
        ("VAULT_ADDR", "https://vault.example.com:8200"),
        ("SSH_KEY_PATH", "/home/pat/.ssh/id_ed25519"),
        ("DATABASE_URL", "postgres://postgres:postgres@localhost/app"),
        ("REDIS_URL", "redis://default:dev@localhost:6379"),
        ("MY_SERVICE_TOKEN", "development"),
    ]
    .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    assert!(labels(&env_values(vars)).is_empty());
}

#[test]
fn pf_28_s01_delivered_events_are_gated_once_more() {
    let gate = codex_secret_broker::output_gate::global();
    gate.register("env:PF28_CANARY_API_KEY", SecretClass::Operational, CANARY)
        .expect("register");
    gate.arm().expect("arm");
    // An event sent on a cloned sender (MCP startup, elicitation) never
    // passed `send_event`; delivery still gates it.
    let error = EventMsg::Error(ErrorEvent {
        message: format!("mcp server failed: --token {CANARY}"),
        codex_error_info: None,
    });
    let delivered = gate_delivered(error).expect("delivered");
    let serialized = serde_json::to_string(&delivered).expect("json");
    assert!(!serialized.contains(CANARY), "{serialized}");
    let delta = EventMsg::ExecCommandOutputDelta(ExecCommandOutputDeltaEvent {
        call_id: "call-pf28-delivered".to_string(),
        stream: ExecOutputStream::Stdout,
        chunk: format!("v={CANARY}\n").into_bytes(),
    });
    let Some(EventMsg::ExecCommandOutputDelta(delta)) = gate_delivered(delta) else {
        panic!("delta dropped");
    };
    assert_eq!(delta.chunk, b"v=[REDACTED:env:PF28_CANARY_API_KEY]\n");
}

#[test]
fn pf_28_s01_sign_in_and_config_values_are_managed() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("auth.json"),
        r#"{"OPENAI_API_KEY":"sk-pf28-auth-json-key-0000","tokens":{"access_token":"pf28-access-token-1111","account_id":"acct-pf28-2222"},"last_refresh":"2026-10-06"}"#,
    )
    .expect("auth.json");
    std::fs::write(
        dir.path().join("config.toml"),
        r#"
model = "gpt-5"
[tui.keymap]
quit = "ctrl-shift-q"
[mcp_servers.github]
bearer_token_env_var = "GITHUB_TOKEN"
[mcp_servers.github.http_headers]
Authorization = "Bearer pf28-header-token-3333"
[mcp_servers.local.env]
SERVICE_API_KEY = "pf28-mcp-env-key-4444"
"#,
    )
    .expect("config.toml");
    let auth = json_file_values(&dir.path().join("auth.json"), "auth.json");
    let auth_values: Vec<&str> = auth.iter().map(|(_, value)| value.as_str()).collect();
    assert_eq!(
        auth_values,
        vec![
            "sk-pf28-auth-json-key-0000",
            "pf28-access-token-1111",
            "acct-pf28-2222"
        ]
    );
    let config = toml_file_values(&dir.path().join("config.toml"));
    let config_values: Vec<&str> = config.iter().map(|(_, value)| value.as_str()).collect();
    assert_eq!(
        config_values,
        vec![
            "Bearer pf28-header-token-3333",
            "pf28-header-token-3333",
            "pf28-mcp-env-key-4444",
        ]
    );
    assert!(json_file_values(&dir.path().join("missing.json"), "missing").is_empty());
}

#[test]
fn pf_28_s01_tool_result_and_error_are_gated() {
    let gate = gate();
    let item = ResponseItem::FunctionCallOutput {
        id: None,
        call_id: "call-pf28".to_string(),
        output: FunctionCallOutputPayload::from_text(format!("token={CANARY}\nok")),
        internal_chat_message_metadata_passthrough: None,
    };
    let Some(gated) = gate_values_with(&gate, OutputSink::ToolResult, &[item]) else {
        panic!("tool result not gated");
    };
    let serialized = serde_json::to_string(&gated).expect("json");
    assert!(!serialized.contains(CANARY), "{serialized}");
    assert!(serialized.contains("token=[REDACTED:env:PF28_CANARY_API_KEY]"));

    let error = EventMsg::Error(ErrorEvent {
        message: format!("request failed: Authorization: Bearer {CANARY}"),
        codex_error_info: None,
    });
    let events = gate_event_with(&gate, "turn-pf28", error);
    let serialized = serde_json::to_string(&events).expect("json");
    assert_eq!(events.len(), 1);
    assert!(!serialized.contains(CANARY), "{serialized}");
}

#[test]
fn pf_28_s01_split_exec_output_is_held_back_and_released_at_end() {
    let gate = gate();
    let call_id = "call-pf28-exec-split";
    let full = format!("value: {CANARY} done\n");
    let (first, second) = full.as_bytes().split_at(14);
    let mut delivered = Vec::new();
    for chunk in [first, second] {
        let events = gate_event_with(
            &gate,
            "turn-pf28-exec",
            EventMsg::ExecCommandOutputDelta(ExecCommandOutputDeltaEvent {
                call_id: call_id.to_string(),
                stream: ExecOutputStream::Stdout,
                chunk: chunk.to_vec(),
            }),
        );
        for event in events {
            let EventMsg::ExecCommandOutputDelta(event) = event else {
                panic!("unexpected event");
            };
            delivered.extend(event.chunk);
        }
    }
    // Releasing happens on the command's end event.
    let tail = finish_streams(&gate, |_, stream| stream.item_id == call_id);
    for event in tail {
        let EventMsg::ExecCommandOutputDelta(event) = event else {
            panic!("unexpected event");
        };
        delivered.extend(event.chunk);
    }
    assert_eq!(
        String::from_utf8(delivered).expect("utf8"),
        "value: [REDACTED:env:PF28_CANARY_API_KEY] done\n"
    );
}

#[test]
fn pf_28_s01_split_agent_message_never_shows_a_fragment() {
    let gate = gate();
    let item_id = "msg-pf28-split";
    let text = format!("the key is {CANARY}.");
    let mut delivered = String::new();
    for piece in [&text[..15], &text[15..20], &text[20..]] {
        for event in gate_event_with(
            &gate,
            "turn-pf28-split",
            EventMsg::AgentMessageContentDelta(AgentMessageContentDeltaEvent {
                thread_id: "thread-pf28".to_string(),
                turn_id: "turn-pf28".to_string(),
                item_id: item_id.to_string(),
                delta: piece.to_string(),
            }),
        ) {
            let EventMsg::AgentMessageContentDelta(event) = event else {
                panic!("unexpected event");
            };
            assert!(!event.delta.contains("canary"), "{}", event.delta);
            delivered.push_str(&event.delta);
        }
    }
    // The turn is aborted before the item completes: the tail is still
    // released (gated), ahead of the abort event itself.
    let aborted = gate_event_with(
        &gate,
        "turn-pf28-split",
        EventMsg::TurnAborted(TurnAbortedEvent {
            turn_id: Some("turn-pf28-split".to_string()),
            reason: TurnAbortReason::Interrupted,
            started_at: None,
            completed_at: None,
            duration_ms: None,
        }),
    );
    assert!(matches!(aborted.last(), Some(EventMsg::TurnAborted(_))));
    for event in &aborted[..aborted.len() - 1] {
        let EventMsg::AgentMessageContentDelta(event) = event else {
            panic!("unexpected event");
        };
        delivered.push_str(&event.delta);
    }
    assert_eq!(delivered, "the key is [REDACTED:env:PF28_CANARY_API_KEY].");
}

#[test]
fn pf_28_s01_undecodable_value_is_withheld() {
    let gate = gate();
    // Bytes serialize as base64; a redaction inside them cannot be decoded,
    // so the raw-path event is dropped rather than delivered.
    let event = EventMsg::ExecCommandOutputDelta(ExecCommandOutputDeltaEvent {
        call_id: "call-pf28-raw".to_string(),
        stream: ExecOutputStream::Stderr,
        chunk: format!("xx{CANARY}xx").into_bytes(),
    });
    assert!(matches!(
        gate_value_with(&gate, OutputSink::Presentation, &event),
        Gated::Withheld
    ));
}

#[test]
fn pf_28_s01_model_request_is_gated() {
    let gate = gate();
    let mut prompt = crate::client_common::Prompt {
        input: vec![ResponseItem::FunctionCallOutput {
            id: None,
            call_id: "call-pf28-prompt".to_string(),
            output: FunctionCallOutputPayload::from_text(CANARY.to_string()),
            internal_chat_message_metadata_passthrough: None,
        }],
        ..Default::default()
    };
    prompt.base_instructions.text = format!("instructions {CANARY}");
    let gated = gate_prompt_with(&gate, &prompt).expect("gated");
    let serialized = serde_json::to_string(&gated.input).expect("json");
    assert!(!serialized.contains(CANARY));
    assert!(!gated.base_instructions.text.contains(CANARY));
    // Nothing to remove: the prompt is used as is.
    assert!(gate_prompt_with(&gate, &crate::client_common::Prompt::default()).is_none());
}

fn exec_delta(call_id: &str, chunk: &[u8]) -> EventMsg {
    EventMsg::ExecCommandOutputDelta(ExecCommandOutputDeltaEvent {
        call_id: call_id.to_string(),
        stream: ExecOutputStream::Stdout,
        chunk: chunk.to_vec(),
    })
}

fn chunks(events: Vec<EventMsg>) -> Vec<u8> {
    let mut out = Vec::new();
    for event in events {
        if let EventMsg::ExecCommandOutputDelta(event) = event {
            out.extend(event.chunk);
        }
    }
    out
}

fn exec_end(call_id: &str, turn_id: &str) -> EventMsg {
    EventMsg::ExecCommandEnd(codex_protocol::protocol::ExecCommandEndEvent {
        call_id: call_id.to_string(),
        plugin_id: None,
        script_path: None,
        process_id: None,
        turn_id: turn_id.to_string(),
        completed_at_ms: 0,
        command: vec!["sh".to_string()],
        cwd: codex_utils_path_uri::PathUri::parse("file:///tmp").expect("cwd"),
        parsed_cmd: Vec::new(),
        source: codex_protocol::protocol::ExecCommandSource::Agent,
        interaction_input: None,
        stdout: String::new(),
        stderr: String::new(),
        aggregated_output: String::new(),
        exit_code: 0,
        duration: std::time::Duration::ZERO,
        formatted_output: String::new(),
        status: codex_protocol::protocol::ExecCommandStatus::Completed,
    })
}

fn reasoning_delta(turn_id: &str, index: i64, text: &str) -> EventMsg {
    EventMsg::ReasoningContentDelta(
        serde_json::from_value(serde_json::json!({
            "thread_id": "thread-pf28",
            "turn_id": turn_id,
            "item_id": "rs-pf28-forwarded",
            "delta": text,
            "summary_index": index,
        }))
        .expect("delta"),
    )
}

fn reasoning_text(events: &[EventMsg]) -> String {
    events
        .iter()
        .filter_map(|event| match event {
            EventMsg::ReasoningContentDelta(event) => Some(event.delta.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn pf_28_s01_forwarded_sub_session_stream_is_gated_twice_without_leaking() {
    let gate = gate();
    let (child, parent) = ("turn-pf28-child", "turn-pf28-parent");
    // Review mode: every event the child's turn emits is forwarded, as is,
    // into the parent's turn, which gates it again. Only `gate_event_with`
    // runs, as in production.
    let forward = |events: Vec<EventMsg>, out: &mut Vec<EventMsg>| {
        for event in events {
            out.extend(gate_event_with(&gate, parent, event));
        }
    };
    let mut delivered = Vec::new();

    // Command output with the value split across chunks, then its end.
    let call_id = "call-pf28-forwarded";
    let full = format!("hello {CANARY} done");
    let (first, second) = full.as_bytes().split_at(16);
    for chunk in [first, second] {
        forward(
            gate_event_with(&gate, child, exec_delta(call_id, chunk)),
            &mut delivered,
        );
    }
    forward(
        gate_event_with(&gate, child, exec_end(call_id, child)),
        &mut delivered,
    );
    assert_eq!(
        String::from_utf8(chunks(delivered)).expect("utf8"),
        "hello [REDACTED:env:PF28_CANARY_API_KEY] done"
    );

    // Reasoning with the value split across chunks: each forwarded delta
    // must not drain the child's held-back prefix.
    let mut delivered = Vec::new();
    let text = format!("think {CANARY} more");
    let (first, second) = text.split_at(12);
    for piece in [first, second] {
        forward(
            gate_event_with(&gate, child, reasoning_delta(child, 0, piece)),
            &mut delivered,
        );
    }
    forward(
        gate_event_with(&gate, child, reasoning_delta(child, 1, "next")),
        &mut delivered,
    );
    let aborted = |turn: &str| {
        EventMsg::TurnAborted(TurnAbortedEvent {
            turn_id: Some(turn.to_string()),
            reason: TurnAbortReason::Interrupted,
            started_at: None,
            completed_at: None,
            duration_ms: None,
        })
    };
    forward(
        gate_event_with(&gate, child, aborted(child)),
        &mut delivered,
    );
    delivered.extend(gate_event_with(&gate, parent, aborted(parent)));
    let shown = reasoning_text(&delivered);
    assert_eq!(shown, "think [REDACTED:env:PF28_CANARY_API_KEY] morenext");
}

#[test]
fn pf_28_s01_reasoning_section_break_releases_the_earlier_section_first() {
    let gate = gate();
    let delta = |index: i64, text: &str| {
        EventMsg::ReasoningContentDelta(
            serde_json::from_value(serde_json::json!({
                "thread_id": "thread-pf28",
                "turn_id": "turn-pf28-reasoning",
                "item_id": "rs-pf28",
                "delta": text,
                "summary_index": index,
            }))
            .expect("delta"),
        )
    };
    let tail = &CANARY[..4];
    let mut delivered = Vec::new();
    delivered.extend(gate_event_with(
        &gate,
        "turn-pf28-reasoning",
        delta(0, &format!("first {tail}")),
    ));
    let section_break = EventMsg::AgentReasoningSectionBreak(
        serde_json::from_value(serde_json::json!({"item_id": "rs-pf28", "summary_index": 1}))
            .expect("break"),
    );
    delivered.extend(gate_event_with(&gate, "turn-pf28-reasoning", section_break));
    delivered.extend(gate_event_with(
        &gate,
        "turn-pf28-reasoning",
        delta(1, "second"),
    ));
    delivered.extend(gate_event_with(
        &gate,
        "turn-pf28-reasoning",
        EventMsg::TurnAborted(TurnAbortedEvent {
            turn_id: Some("turn-pf28-reasoning".to_string()),
            reason: TurnAbortReason::Interrupted,
            started_at: None,
            completed_at: None,
            duration_ms: None,
        }),
    ));
    let position = |want: fn(&EventMsg) -> bool| {
        delivered
            .iter()
            .enumerate()
            .filter(|(_, event)| want(event))
            .map(|(index, _)| index)
            .collect::<Vec<_>>()
    };
    let section = |index: i64| {
        delivered
            .iter()
            .filter_map(|event| match event {
                EventMsg::ReasoningContentDelta(event) if event.summary_index == index => {
                    Some(event.delta.as_str())
                }
                _ => None,
            })
            .collect::<String>()
    };
    let first = position(
        |event| matches!(event, EventMsg::ReasoningContentDelta(event) if event.summary_index == 0),
    );
    let second = position(
        |event| matches!(event, EventMsg::ReasoningContentDelta(event) if event.summary_index == 1),
    );
    let breaks = position(|event| matches!(event, EventMsg::AgentReasoningSectionBreak(_)));
    // Section 0's held-back tail arrives before the break, section 1 after.
    assert_eq!(section(0), format!("first {tail}"));
    assert_eq!(section(1), "second");
    assert!(
        first.iter().all(|index| *index < breaks[0]),
        "{first:?} {breaks:?}"
    );
    assert!(
        second.iter().all(|index| *index > breaks[0]),
        "{second:?} {breaks:?}"
    );
}

#[test]
fn pf_28_s01_event_that_cannot_be_rebuilt_is_delivered_stripped() {
    let gate = gate();
    // Bytes serialize as an array; a marker cannot be put back into them.
    let delivered = present(
        &gate,
        exec_delta("call-pf28-strip", format!("xx{CANARY}xx").as_bytes()),
    );
    let Some(EventMsg::ExecCommandOutputDelta(event)) = delivered else {
        panic!("event dropped or replaced");
    };
    assert_eq!(event.call_id, "call-pf28-strip");
    assert!(!String::from_utf8_lossy(&event.chunk).contains(CANARY));
}

#[test]
fn pf_28_s01_value_in_an_object_key_or_number_is_never_delivered_raw() {
    let gate = gate();
    // MCP structured content can carry a value as a key.
    let content = serde_json::json!({ CANARY: "x", "other": "y" });
    let Gated::Changed(gated) = gate_value_with(&gate, OutputSink::Presentation, &content) else {
        panic!("key not gated");
    };
    let text = serde_json::to_string(&gated).expect("json");
    assert!(!text.contains(CANARY), "{text}");
    assert!(
        text.contains("[REDACTED:env:PF28_CANARY_API_KEY]"),
        "{text}"
    );
    assert_eq!(gated.as_object().map(serde_json::Map::len), Some(2));

    // A numeric value cannot be rewritten: it is withheld, not passed.
    let numeric = OutputGate::new();
    numeric
        .register("account", SecretClass::Operational, "987654321012")
        .expect("register");
    let value = serde_json::json!({ "balance": 987_654_321_012_u64 });
    assert!(matches!(
        gate_value_with(&numeric, OutputSink::Presentation, &value),
        Gated::Withheld
    ));
}

#[test]
fn pf_28_s01_clean_value_over_the_scan_limit_passes_unchanged() {
    let gate = gate();
    let big = "a".repeat(9 * 1024 * 1024);
    let value = serde_json::json!({ "first": big, "second": big, "count": 7 });
    assert!(matches!(
        gate_value_with(&gate, OutputSink::Presentation, &value),
        Gated::Unchanged
    ));
}

#[test]
fn pf_28_s01_stripped_delivery_replaces_keys_too() {
    let gate = gate();
    let value = serde_json::json!({ CANARY: [1, 2], "kept": "ok" });
    let stripped: serde_json::Value =
        strip_value(&gate, OutputSink::Presentation, &value).expect("stripped");
    assert_eq!(
        stripped,
        serde_json::json!({ "[WITHHELD]": [1, 2], "kept": "ok" })
    );
}

#[test]
fn pf_28_s01_escaped_value_inside_a_string_is_found_in_the_serialized_item() {
    let gate = OutputGate::new();
    gate.register("escaped", SecretClass::Operational, "pf28\"quoted\"pw-77")
        .expect("register");
    // `cat config.json` output: the text already holds the escaped form.
    let item = ResponseItem::FunctionCallOutput {
        id: None,
        call_id: "call-pf28-escaped".to_string(),
        output: FunctionCallOutputPayload::from_text(
            "{\"password\": \"pf28\\\"quoted\\\"pw-77\"}".to_string(),
        ),
        internal_chat_message_metadata_passthrough: None,
    };
    let gated = gate_values_with(&gate, OutputSink::ToolResult, &[item]).expect("gated");
    let text = serde_json::to_string(&gated).expect("json");
    assert!(!text.contains("pw-77"), "{text}");
    assert!(text.contains("[REDACTED:escaped]"), "{text}");
}
