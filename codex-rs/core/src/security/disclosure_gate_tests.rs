use super::*;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::AgentMessageContentDeltaEvent;
use codex_protocol::protocol::ErrorEvent;
use codex_protocol::protocol::ExecCommandOutputDeltaEvent;
use codex_protocol::protocol::ExecOutputStream;
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
    ]
    .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    let values = env_values(vars);
    assert_eq!(
        labels(&values),
        vec![
            "env:PF28_CANARY_API_KEY",
            "env:GITHUB_TOKEN",
            "env:DATABASE_URL:password",
        ]
    );
    assert_eq!(values[2].1.as_str(), "pf28-db-password");
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
    let events = gate_event_with(&gate, error);
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
    let tail = finish_streams(&gate, call_id);
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
    for event in finish_streams(&gate, item_id) {
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
    let mut prompt = crate::client_common::Prompt::default();
    prompt.input = vec![ResponseItem::FunctionCallOutput {
        id: None,
        call_id: "call-pf28-prompt".to_string(),
        output: FunctionCallOutputPayload::from_text(CANARY.to_string()),
        internal_chat_message_metadata_passthrough: None,
    }];
    prompt.base_instructions.text = format!("instructions {CANARY}");
    let gated = gate_prompt_with(&gate, &prompt).expect("gated");
    let serialized = serde_json::to_string(&gated.input).expect("json");
    assert!(!serialized.contains(CANARY));
    assert!(!gated.base_instructions.text.contains(CANARY));
    // Nothing to remove: the prompt is used as is.
    assert!(gate_prompt_with(&gate, &crate::client_common::Prompt::default()).is_none());
}
