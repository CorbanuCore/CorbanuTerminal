use codex_model_provider_info::KIMI_CODE_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::built_in_model_providers;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::chat_completions_sse;
use core_test_support::responses::sse_response;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use wiremock::Mock;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

const PROGRESS_RESPONSE: &str = "I found the relevant code. Next I will add the regression tests.";
const FINAL_RESPONSE: &str = "Done. I repaired the boundary and added regression coverage.";

#[derive(Default)]
struct ToolThenFinalResponder {
    calls: AtomicUsize,
}

impl Respond for ToolThenFinalResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => sse_response(tool_call_sse()),
            1 => sse_response(chat_completions_sse("k3", FINAL_RESPONSE)),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

fn tool_call_sse() -> String {
    let tool_call = serde_json::json!({
        "id": "chatcmpl-tool",
        "model": "k3",
        "choices": [{
            "delta": {
                "tool_calls": [{
                    "index": 0,
                    "id": "call-lifecycle",
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

fn kimi_provider(server: &wiremock::MockServer) -> ModelProviderInfo {
    ModelProviderInfo {
        base_url: Some(format!("{}/coding/v1", server.uri())),
        env_key: Some("PATH".to_string()),
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(2_000),
        ..built_in_model_providers(/*openai_base_url*/ None)[KIMI_CODE_PROVIDER_ID].clone()
    }
}

async fn submit_turn(test: &core_test_support::test_codex::TestCodex) {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "Inspect the repository and finish the requested repair.".to_string(),
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
            EventMsg::Warning(warning) => panic!("unexpected warning: {}", warning.message),
            EventMsg::Error(error) => panic!("turn failed: {error:?}"),
            _ => {}
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_text_stop_is_terminal_without_extra_inference() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(sse_response(chat_completions_sse("k3", PROGRESS_RESPONSE)))
        .expect(1)
        .mount(&server)
        .await;

    let provider = kimi_provider(&server);
    let test = test_codex()
        .with_config(move |config| {
            config.model = Some("k3".to_string());
            config.model_provider_id = KIMI_CODE_PROVIDER_ID.to_string();
            config.model_provider = provider;
        })
        .build(&server)
        .await
        .expect("build test session");

    submit_turn(&test).await;

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_tool_call_then_final_uses_exactly_two_inference_requests() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(ToolThenFinalResponder::default())
        .expect(2)
        .mount(&server)
        .await;

    let provider = kimi_provider(&server);
    let test = test_codex()
        .with_config(move |config| {
            config.model = Some("k3".to_string());
            config.model_provider_id = KIMI_CODE_PROVIDER_ID.to_string();
            config.model_provider = provider;
        })
        .build(&server)
        .await
        .expect("build test session");

    submit_turn(&test).await;

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 2);
}
