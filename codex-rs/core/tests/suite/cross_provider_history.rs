//! #578: after a mid-session provider switch, history items another provider
//! produced reach the next provider in a form its wire accepts, never as
//! references to items it never stored.
//!
//! Each mock host checks what the real one rejects:
//! - Responses (`store: false`): a reasoning item without encrypted content
//!   is a reference: `404 Item with id '…' not found`. Encrypted content
//!   another host sealed fails to verify (400).
//! - Anthropic: a thinking block another host signed fails its signature
//!   check, and `tool_use` IDs must match `^[a-zA-Z0-9_-]+$` (400).
//! - Chat Completions: accepts the request; the test checks no encrypted
//!   or signed state reaches it.
#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use codex_features::Feature;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ThreadSettingsOverrides;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::chat_completions_tool_call_sse;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::start_websocket_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event_with_timeout;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

#[derive(Clone, Copy)]
enum Host {
    Responses,
    Chat,
    Anthropic,
}

/// One mock host: `seal` marks the encrypted reasoning or signature it
/// issues and is the only one it accepts.
#[derive(Clone)]
struct MockHost {
    host: Host,
    seal: &'static str,
    /// Responses only: item IDs it reports missing once, as a host that
    /// lost them would (the recovery path).
    forgotten: Arc<Mutex<Vec<String>>>,
    /// Chat only: the tool-call ID of its first reply, if it calls a tool.
    tool_call_id: Option<&'static str>,
    requests: Arc<Mutex<Vec<Value>>>,
}

impl MockHost {
    fn new(host: Host, seal: &'static str) -> Self {
        Self {
            host,
            seal,
            forgotten: Arc::default(),
            tool_call_id: None,
            requests: Arc::default(),
        }
    }

    fn bodies(&self) -> Vec<Value> {
        self.requests.lock().unwrap().clone()
    }

    fn sse(body: String) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_raw(body, "text/event-stream")
    }

    fn responses(&self, body: &Value, turn: usize) -> ResponseTemplate {
        for item in body["input"].as_array().into_iter().flatten() {
            let id = item["id"].as_str().unwrap_or_default();
            if item["type"] == "reasoning" || item["type"] == "web_search_call" {
                let mut forgotten = self.forgotten.lock().unwrap();
                if let Some(index) = forgotten.iter().position(|lost| lost == id) {
                    forgotten.remove(index);
                    return not_found(id);
                }
            }
            if item["type"] == "compaction"
                && !item["encrypted_content"]
                    .as_str()
                    .is_some_and(|sealed| sealed.starts_with(self.seal))
            {
                return ResponseTemplate::new(400).set_body_json(json!({"error": {
                    "message": format!("The encrypted content for item {id} could not be verified."),
                    "type": "invalid_request_error",
                }}));
            }
            if item["type"] == "reasoning" {
                match item["encrypted_content"].as_str() {
                    None => return not_found(id),
                    Some(sealed) if !sealed.starts_with(self.seal) => {
                        return ResponseTemplate::new(400).set_body_json(json!({"error": {
                            "message": format!("The encrypted content for item {id} could not be verified."),
                            "type": "invalid_request_error",
                        }}));
                    }
                    Some(_) => {}
                }
            }
        }
        let events = [
            json!({"type": "response.created", "response": {"id": format!("resp_{turn}")}}),
            json!({"type": "response.output_item.done", "item": {
                "type": "reasoning", "id": format!("rs_{}_{turn}", self.seal),
                "summary": [], "encrypted_content": format!("{}:sealed", self.seal),
            }}),
            json!({"type": "response.output_item.done", "item": {
                "type": "message", "role": "assistant", "id": format!("msg_{}_{turn}", self.seal),
                "content": [{"type": "output_text", "text": "responses reply"}],
            }}),
            json!({"type": "response.completed", "response": {"id": format!("resp_{turn}"),
                "usage": {"input_tokens": 1, "input_tokens_details": null, "output_tokens": 1,
                    "output_tokens_details": null, "total_tokens": 2}}}),
        ];
        Self::sse(
            events
                .iter()
                .map(|event| {
                    format!(
                        "event: {}\ndata: {event}\n\n",
                        event["type"].as_str().unwrap()
                    )
                })
                .collect(),
        )
    }

    fn chat(&self, turn: usize) -> ResponseTemplate {
        if turn == 0
            && let Some(call_id) = self.tool_call_id
        {
            return Self::sse(chat_completions_tool_call_sse(
                call_id,
                "no_such_tool",
                "{}",
            ));
        }
        // Z.AI response IDs are timestamps; the adapter names reasoning `rs_<id>`.
        let id = format!("2026101213421902b1975497{turn}");
        let chunks = [
            json!({"id": id, "choices": [{"index": 0, "delta": {"role": "assistant", "reasoning_content": "glm thinking"}}]}),
            json!({"id": id, "choices": [{"index": 0, "delta": {"content": "chat reply"}, "finish_reason": "stop"}]}),
            json!({"id": id, "choices": [], "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}}),
        ];
        Self::sse(
            chunks
                .iter()
                .map(|chunk| format!("data: {chunk}\n\n"))
                .chain(["data: [DONE]\n\n".to_string()])
                .collect(),
        )
    }

    fn anthropic(&self, body: &Value) -> ResponseTemplate {
        for message in body["messages"].as_array().into_iter().flatten() {
            for block in message["content"].as_array().into_iter().flatten() {
                let invalid = match block["type"].as_str() {
                    Some("thinking") => block["signature"] != self.seal,
                    Some("tool_use") => !block["id"].as_str().is_some_and(|id| {
                        id.chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    }),
                    _ => false,
                };
                if invalid {
                    return ResponseTemplate::new(400).set_body_json(json!({
                        "type": "error",
                        "error": {"type": "invalid_request_error", "message": format!("invalid block {block}")},
                    }));
                }
            }
        }
        let events = [
            json!({"type": "message_start", "message": {"id": format!("msg_{}", self.seal), "model": "claude", "usage": {"input_tokens": 1, "output_tokens": 0}}}),
            json!({"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": ""}}),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "claude thinking"}}),
            json!({"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": self.seal}}),
            json!({"type": "content_block_stop", "index": 0}),
            json!({"type": "content_block_start", "index": 1, "content_block": {"type": "text", "text": ""}}),
            json!({"type": "content_block_delta", "index": 1, "delta": {"type": "text_delta", "text": "anthropic reply"}}),
            json!({"type": "content_block_stop", "index": 1}),
            json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 1}}),
            json!({"type": "message_stop"}),
        ];
        Self::sse(
            events
                .iter()
                .map(|event| {
                    format!(
                        "event: {}\ndata: {event}\n\n",
                        event["type"].as_str().unwrap()
                    )
                })
                .collect(),
        )
    }
}

fn not_found(id: &str) -> ResponseTemplate {
    ResponseTemplate::new(404).set_body_json(json!({"error": {
        "message": format!("Item with id '{id}' not found. Items are not persisted when `store` is set to false. Try again with `store` set to true, or remove this item from your input."),
        "type": "invalid_request_error",
        "param": "input",
        "code": null,
    }}))
}

impl Respond for MockHost {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap_or_default();
        let turn = {
            let mut requests = self.requests.lock().unwrap();
            requests.push(body.clone());
            requests.len() - 1
        };
        match self.host {
            Host::Responses => self.responses(&body, turn),
            Host::Chat => self.chat(turn),
            Host::Anthropic => self.anthropic(&body),
        }
    }
}

/// A provider served by `mock`, registered under `id`.
async fn mount(mock: &MockHost) -> MockServer {
    let server = MockServer::start().await;
    let route = match mock.host {
        Host::Responses => ".*/responses$",
        Host::Chat => ".*/chat/completions$",
        Host::Anthropic => ".*/messages$",
    };
    Mock::given(method("POST"))
        .and(path_regex(route))
        .respond_with(mock.clone())
        .mount(&server)
        .await;
    if let Host::Responses = mock.host {
        // Remote compaction: a summary sealed for this host.
        Mock::given(method("POST"))
            .and(path_regex(".*/responses/compact$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"output": [{
                "type": "compaction",
                "encrypted_content": format!("{}:summary", mock.seal),
            }]})))
            .mount(&server)
            .await;
    }
    server
}

struct Session {
    test: TestCodex,
    _servers: Vec<MockServer>,
}

/// A session whose first provider is `providers[0]`; the others can be
/// switched to by id.
async fn session(providers: &[(&'static str, &MockHost)]) -> anyhow::Result<Session> {
    session_named(providers, /*openai_name*/ false).await
}

/// With `openai_name`, every provider is named like the built-in OpenAI one,
/// so it compacts remotely (like two OpenAI accounts).
async fn session_named(
    providers: &[(&'static str, &MockHost)],
    openai_name: bool,
) -> anyhow::Result<Session> {
    let mut servers = Vec::new();
    let mut infos = Vec::new();
    for (id, mock) in providers {
        let server = mount(mock).await;
        let wire_api = match mock.host {
            Host::Responses => WireApi::Responses,
            Host::Chat => WireApi::Chat,
            Host::Anthropic => WireApi::Anthropic,
        };
        infos.push((id.to_string(), format!("{}/v1", server.uri()), wire_api));
        servers.push(server);
    }
    let first = infos[0].0.clone();
    let test = test_codex()
        .with_config(move |config| {
            for (id, base_url, wire_api) in &infos {
                let info = ModelProviderInfo {
                    name: if openai_name {
                        "OpenAI".to_string()
                    } else {
                        format!("fixture {id}")
                    },
                    base_url: Some(base_url.clone()),
                    wire_api: *wire_api,
                    request_max_retries: Some(0),
                    stream_max_retries: Some(0),
                    stream_idle_timeout_ms: Some(5_000),
                    supports_websockets: false,
                    ..config.model_provider.clone()
                };
                if *id == first {
                    config.model_provider = info.clone();
                }
                config.model_providers.insert(id.clone(), info);
            }
            config.model_provider_id = first;
            // One model for every wire; Anthropic requests need its catalogued
            // output limit.
            config.model = Some("claude-opus-5".into());
            config.model_catalog = Some(
                codex_models_manager::bundled_models_response().expect("bundled model catalog"),
            );
            // The `/responses/compact` endpoint.
            config
                .features
                .disable(Feature::RemoteCompactionV2)
                .expect("remote compaction v1");
        })
        .build(&servers[0])
        .await?;
    Ok(Session {
        test,
        _servers: servers,
    })
}

impl Session {
    async fn switch(&self, provider: &str) -> anyhow::Result<()> {
        self.test
            .codex
            .submit(Op::ThreadSettings {
                thread_settings: ThreadSettingsOverrides {
                    model_provider: Some(provider.into()),
                    ..Default::default()
                },
            })
            .await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            while self.test.codex.config_snapshot().await.model_provider_id != provider {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        Ok(())
    }

    /// Runs one turn; returns the errors it reported.
    async fn turn(&self, text: &str) -> anyhow::Result<Vec<String>> {
        turn(&self.test, text).await
    }

    /// Compacts the history; returns the errors it reported.
    async fn compact(&self) -> anyhow::Result<Vec<String>> {
        self.test.codex.submit(Op::Compact).await?;
        finish(&self.test).await
    }
}

async fn turn(test: &TestCodex, text: &str) -> anyhow::Result<Vec<String>> {
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
    finish(test).await
}

/// Waits for the running turn to end; returns the errors it reported.
async fn finish(test: &TestCodex) -> anyhow::Result<Vec<String>> {
    let mut errors = Vec::new();
    loop {
        match wait_for_event_with_timeout(&test.codex, |_| true, Duration::from_secs(20)).await {
            EventMsg::Error(error) => errors.push(error.message),
            EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_) => return Ok(errors),
            _ => {}
        }
    }
}

fn input_items(body: &Value) -> Vec<Value> {
    body["input"].as_array().cloned().unwrap_or_default()
}

fn has_reasoning_id(body: &Value, id: &str) -> bool {
    input_items(body)
        .iter()
        .any(|item| item["type"] == "reasoning" && item["id"] == id)
}

/// The issue's steps: OpenAI, then Z.AI GLM (Chat), then OpenAI again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chat_reasoning_is_not_sent_to_responses_as_a_reference() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let openai = MockHost::new(Host::Responses, "A");
    let zai = MockHost::new(Host::Chat, "Z");
    let session = session(&[("openai", &openai), ("zai", &zai)]).await?;
    assert_eq!(session.turn("first").await?, Vec::<String>::new());
    session.switch("zai").await?;
    assert_eq!(session.turn("second").await?, Vec::<String>::new());
    session.switch("openai").await?;
    assert_eq!(session.turn("third").await?, Vec::<String>::new());
    assert_eq!(session.turn("fourth").await?, Vec::<String>::new());

    let bodies = openai.bodies();
    assert_eq!(bodies.len(), 3);
    let last = &bodies[2];
    // OpenAI's own reasoning is still replayed as it was (same provider).
    assert!(has_reasoning_id(last, "rs_A_0"));
    assert!(has_reasoning_id(last, "rs_A_1"));
    // GLM's reasoning is not; its reply is sent inline, without its ID.
    let reasoning_ids: Vec<Value> = input_items(last)
        .iter()
        .filter(|item| item["type"] == "reasoning")
        .map(|item| item["id"].clone())
        .collect();
    assert_eq!(reasoning_ids, vec![json!("rs_A_0"), json!("rs_A_1")]);
    let chat_reply = input_items(last)
        .into_iter()
        .find(|item| item.to_string().contains("chat reply"))
        .expect("the GLM reply is kept");
    assert_eq!(chat_reply.get("id"), None);
    // Nothing sent to Z.AI carries OpenAI's encrypted reasoning.
    assert!(
        zai.bodies()
            .iter()
            .all(|body| !body.to_string().contains("A:sealed"))
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn anthropic_thinking_is_not_sent_to_responses() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let openai = MockHost::new(Host::Responses, "A");
    let anthropic = MockHost::new(Host::Anthropic, "sig-X");
    let session = session(&[("openai", &openai), ("anthropic", &anthropic)]).await?;
    assert_eq!(session.turn("first").await?, Vec::<String>::new());
    session.switch("anthropic").await?;
    assert_eq!(session.turn("second").await?, Vec::<String>::new());
    session.switch("openai").await?;
    assert_eq!(session.turn("third").await?, Vec::<String>::new());

    let last = openai.bodies().pop().unwrap();
    assert!(!last.to_string().contains("sig-X"));
    assert!(!last.to_string().contains("claude thinking"));
    // Anthropic never sees OpenAI's encrypted reasoning either.
    assert!(!anthropic.bodies()[0].to_string().contains("A:sealed"));
    Ok(())
}

/// Another Responses provider: its encrypted reasoning is sealed for it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn another_responses_providers_reasoning_is_dropped() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let first = MockHost::new(Host::Responses, "A");
    let second = MockHost::new(Host::Responses, "B");
    let session = session(&[("first", &first), ("second", &second)]).await?;
    assert_eq!(session.turn("first").await?, Vec::<String>::new());
    session.switch("second").await?;
    assert_eq!(session.turn("second").await?, Vec::<String>::new());
    session.switch("first").await?;
    assert_eq!(session.turn("third").await?, Vec::<String>::new());

    let to_second = second.bodies();
    assert!(!has_reasoning_id(&to_second[0], "rs_A_0"));
    // Back on the first provider, its own reasoning is replayed again.
    let back = first.bodies().pop().unwrap();
    assert!(has_reasoning_id(&back, "rs_A_0"));
    assert!(!has_reasoning_id(&back, "rs_B_0"));
    Ok(())
}

/// Another Anthropic-compatible host: its thinking signatures are its own.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn another_anthropic_hosts_thinking_is_dropped() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let first = MockHost::new(Host::Anthropic, "sig-X");
    let second = MockHost::new(Host::Anthropic, "sig-Y");
    let session = session(&[("first", &first), ("second", &second)]).await?;
    assert_eq!(session.turn("first").await?, Vec::<String>::new());
    assert_eq!(session.turn("again").await?, Vec::<String>::new());
    session.switch("second").await?;
    assert_eq!(session.turn("second").await?, Vec::<String>::new());

    // The same host gets its own signed thinking back.
    assert!(first.bodies()[1].to_string().contains("sig-X"));
    assert!(!second.bodies()[0].to_string().contains("sig-X"));
    Ok(())
}

/// A Chat host's tool-call IDs need not match Anthropic's ID alphabet.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn chat_tool_call_ids_reach_anthropic_in_its_alphabet() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let mut zai = MockHost::new(Host::Chat, "Z");
    zai.tool_call_id = Some("functions.no_such_tool:0");
    let anthropic = MockHost::new(Host::Anthropic, "sig-X");
    let session = session(&[("zai", &zai), ("anthropic", &anthropic)]).await?;
    assert_eq!(session.turn("call a tool").await?, Vec::<String>::new());
    session.switch("anthropic").await?;
    assert_eq!(session.turn("second").await?, Vec::<String>::new());

    let body = anthropic.bodies()[0].clone();
    let blocks: Vec<Value> = body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|message| message["content"].as_array().cloned().unwrap_or_default())
        .collect();
    let tool_use = blocks
        .iter()
        .find(|block| block["type"] == "tool_use")
        .expect("the call is replayed");
    let tool_result = blocks
        .iter()
        .find(|block| block["type"] == "tool_result")
        .expect("its result is replayed");
    let id = tool_use["id"].as_str().unwrap();
    assert!(id.starts_with("functions_no_such_tool_0_"), "{id}");
    assert_eq!(tool_result["tool_use_id"], tool_use["id"]);
    Ok(())
}

/// The other directions: Responses and Anthropic history reaching Chat, and
/// Responses and Chat history reaching Anthropic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn responses_and_anthropic_state_does_not_reach_chat_or_anthropic() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let openai = MockHost::new(Host::Responses, "A");
    let anthropic = MockHost::new(Host::Anthropic, "sig-X");
    let zai = MockHost::new(Host::Chat, "Z");
    let session = session(&[
        ("openai", &openai),
        ("anthropic", &anthropic),
        ("zai", &zai),
    ])
    .await?;
    assert_eq!(session.turn("on openai").await?, Vec::<String>::new());
    session.switch("anthropic").await?;
    assert_eq!(session.turn("on anthropic").await?, Vec::<String>::new());
    session.switch("zai").await?;
    assert_eq!(session.turn("on zai").await?, Vec::<String>::new());
    session.switch("anthropic").await?;
    assert_eq!(session.turn("anthropic again").await?, Vec::<String>::new());

    let to_chat = zai.bodies()[0].to_string();
    assert!(!to_chat.contains("A:sealed") && !to_chat.contains("sig-X"));
    let to_anthropic = anthropic.bodies()[0].to_string();
    assert!(!to_anthropic.contains("A:sealed"));
    // Its own signed thinking still comes back after the Chat turn.
    assert!(anthropic.bodies()[1].to_string().contains("sig-X"));
    Ok(())
}

/// A host that reports an item it was sent as missing: the turn retries
/// without it, and later turns no longer send it.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_missing_item_404_recovers_the_turn() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let openai = MockHost::new(Host::Responses, "A");
    let session = session(&[("openai", &openai)]).await?;
    assert_eq!(session.turn("first").await?, Vec::<String>::new());
    openai.forgotten.lock().unwrap().push("rs_A_0".into());
    assert_eq!(session.turn("second").await?, Vec::<String>::new());
    assert_eq!(session.turn("third").await?, Vec::<String>::new());

    let bodies = openai.bodies();
    // First turn, the rejected request, its retry, then the third turn.
    assert_eq!(bodies.len(), 4);
    assert!(has_reasoning_id(&bodies[1], "rs_A_0"));
    assert!(!has_reasoning_id(&bodies[2], "rs_A_0"));
    assert!(!has_reasoning_id(&bodies[3], "rs_A_0"));
    assert!(has_reasoning_id(&bodies[3], "rs_A_2"));
    Ok(())
}

/// A remote compaction summary is sealed for the provider (or account) that
/// made it: another one does not get it, the same one gets it back. Two
/// providers named like OpenAI stand in for two OpenAI accounts (a named
/// account is its own producer, see the `provider_items` unit tests).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn another_providers_compaction_summary_is_dropped() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let first = MockHost::new(Host::Responses, "A");
    let second = MockHost::new(Host::Responses, "B");
    let session = session_named(
        &[("first", &first), ("second", &second)],
        /*openai_name*/ true,
    )
    .await?;
    assert_eq!(session.turn("first").await?, Vec::<String>::new());
    assert_eq!(session.compact().await?, Vec::<String>::new());
    session.switch("second").await?;
    assert_eq!(session.turn("second").await?, Vec::<String>::new());
    session.switch("first").await?;
    assert_eq!(session.turn("third").await?, Vec::<String>::new());

    let compaction = |body: &Value| {
        input_items(body)
            .iter()
            .any(|item| item["type"] == "compaction")
    };
    assert!(!compaction(&second.bodies()[0]));
    assert!(compaction(&first.bodies().pop().unwrap()));
    Ok(())
}

/// The same recovery when the 404 arrives over the Responses WebSocket
/// (OpenAI's default transport).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_missing_item_404_over_websocket_recovers_the_turn() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let reasoning = json!({"type": "response.output_item.done", "item": {
        "type": "reasoning", "id": "rs_ws_1", "summary": [], "encrypted_content": "sealed",
    }});
    let not_found = json!({
        "type": "error",
        "status": 404,
        "error": {
            "type": "invalid_request_error",
            "message": "Item with id 'rs_ws_1' not found. Items are not persisted when `store` is set to false.",
        },
    });
    let answer = |id: &str| {
        vec![
            ev_response_created(id),
            ev_assistant_message(&format!("msg_{id}"), "websocket reply"),
            ev_completed(id),
        ]
    };
    let server = start_websocket_server(vec![
        vec![
            vec![ev_response_created("warm"), ev_completed("warm")],
            vec![
                ev_response_created("one"),
                reasoning,
                ev_assistant_message("msg_one", "websocket reply"),
                ev_completed("one"),
            ],
            vec![not_found],
            answer("two"),
            answer("three"),
        ],
        vec![answer("two"), answer("three")],
    ])
    .await;
    let test = test_codex()
        .with_config(|config| {
            config.model_provider.request_max_retries = Some(0);
            config.model_provider.stream_max_retries = Some(0);
        })
        .build_with_websocket_server(&server)
        .await?;
    assert_eq!(turn(&test, "ws-turn-one").await?, Vec::<String>::new());
    assert_eq!(turn(&test, "ws-turn-two").await?, Vec::<String>::new());

    let requests: Vec<Value> = server
        .connections()
        .into_iter()
        .flatten()
        .map(|request| request.body_json())
        .collect();
    let rejected = requests
        .iter()
        .position(|body| body.to_string().contains("ws-turn-two"))
        .expect("the second turn was sent");
    let after: Vec<&Value> = requests[rejected + 1..].iter().collect();
    assert!(!after.is_empty(), "the second turn was retried");
    // The rejected request continued the first response; without the item the
    // history no longer extends it, so the retry sends it in full, without
    // the missing item.
    let retry = after[0];
    assert!(
        retry["input"].to_string().contains("ws-turn-one"),
        "{retry}"
    );
    assert!(
        retry.get("previous_response_id").is_none_or(Value::is_null),
        "{retry}"
    );
    assert!(
        after
            .iter()
            .all(|body| !body["input"].to_string().contains("rs_ws_1")),
        "{after:?}"
    );
    server.shutdown().await;
    Ok(())
}
