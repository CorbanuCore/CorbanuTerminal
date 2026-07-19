use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::chat_completions_sse;
use core_test_support::responses::sse_response;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use serde_json::Value;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use wiremock::Mock;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

const OBJECTIVE: &str = "Inspect the repository and finish the requested repair.";
const PROGRESS_RESPONSE: &str = "I found the relevant code. Next I will add the regression tests.";
const FINAL_RESPONSE: &str = "Done. I repaired the boundary and added regression coverage.";

#[derive(Default)]
struct KimiCompletionResponder {
    calls: AtomicUsize,
}

#[derive(Default)]
struct AlwaysIncompleteCompletionResponder {
    calls: AtomicUsize,
}

impl Respond for KimiCompletionResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => sse_response(tool_call_sse()),
            1 => sse_response(chat_completions_sse("k3", PROGRESS_RESPONSE)),
            2 => sse_response(chat_completions_sse("k3", r#"{"decision":"incomplete"}"#)),
            3 => sse_response(chat_completions_sse("k3", FINAL_RESPONSE)),
            4 => sse_response(chat_completions_sse("k3", r#"{"decision":"complete"}"#)),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

impl Respond for AlwaysIncompleteCompletionResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => sse_response(tool_call_sse()),
            1 | 4 | 7 => sse_response(chat_completions_sse("k3", FINAL_RESPONSE)),
            2 | 5 | 8 => sse_response(chat_completions_sse("k3", r#"{"decision":"incomplete"}"#)),
            3 => sse_response(tool_call_sse_with_id("call-guard-2")),
            6 => sse_response(tool_call_sse_with_id("call-guard-3")),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

fn tool_call_sse() -> String {
    tool_call_sse_with_id("call-guard")
}

fn tool_call_sse_with_id(id: &str) -> String {
    let tool_call = serde_json::json!({
        "id": "chatcmpl-tool",
        "model": "k3",
        "choices": [{
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": "exec_command",
                        "arguments": "{\"cmd\":\"true\"}"
                    }
                }]
            }
        }]
    });
    format!("data: {tool_call}\n\ndata: [DONE]\n\n")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_progress_stop_after_tool_work_is_classified_and_continued() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(KimiCompletionResponder::default())
        .expect(5)
        .mount(&server)
        .await;

    let provider = ModelProviderInfo {
        name: "Kimi Code".to_string(),
        base_url: Some(format!("{}/coding/v1", server.uri())),
        env_key: Some("PATH".to_string()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Chat,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(2_000),
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    };

    let test = test_codex()
        .with_config(move |config| {
            config.model = Some("k3".to_string());
            config.model_provider_id = "kimi-code".to_string();
            config.model_provider = provider;
        })
        .build(&server)
        .await
        .expect("build test session");

    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: OBJECTIVE.to_string(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await
        .expect("submit turn");

    loop {
        let event = test.codex.next_event().await.expect("next event");
        match event.msg {
            EventMsg::TurnComplete(_) => break,
            EventMsg::Error(error) => panic!("turn failed: {error:?}"),
            _ => {}
        }
    }

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 5);
    let bodies = requests
        .iter()
        .map(|request| serde_json::from_slice::<Value>(&request.body).expect("request body"))
        .collect::<Vec<_>>();
    assert!(bodies[2].get("response_format").is_some());
    assert!(bodies[2].to_string().contains(PROGRESS_RESPONSE));
    assert!(
        bodies[3]
            .to_string()
            .contains("did not finish the requested work")
    );
    assert!(bodies[4].to_string().contains(FINAL_RESPONSE));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn completion_guard_stops_when_classifier_repeatedly_rejects_final_answers() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(AlwaysIncompleteCompletionResponder::default())
        .expect(9)
        .mount(&server)
        .await;

    let provider = ModelProviderInfo {
        name: "Kimi Code".to_string(),
        base_url: Some(format!("{}/coding/v1", server.uri())),
        env_key: Some("PATH".to_string()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Chat,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(2_000),
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
    };

    let test = test_codex()
        .with_config(move |config| {
            config.model = Some("k3".to_string());
            config.model_provider_id = "kimi-code".to_string();
            config.model_provider = provider;
        })
        .build(&server)
        .await
        .expect("build test session");

    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: OBJECTIVE.to_string(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await
        .expect("submit turn");

    let mut saw_bounded_warning = false;
    loop {
        let event = test.codex.next_event().await.expect("next event");
        match event.msg {
            EventMsg::Warning(warning) => {
                saw_bounded_warning |= warning
                    .message
                    .contains("stopped the automatic continuation loop");
            }
            EventMsg::TurnComplete(_) => break,
            EventMsg::Error(error) => panic!("turn failed: {error:?}"),
            _ => {}
        }
    }

    assert!(saw_bounded_warning);
    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 9);
    assert_eq!(
        requests
            .iter()
            .filter(|request| request
                .body
                .windows(b"did not finish the requested work".len())
                .any(|window| window == b"did not finish the requested work"))
            .count(),
        4
    );
}
