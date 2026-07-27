use codex_model_provider_info::KIMI_CODE_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::built_in_model_providers;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::sse_response;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use serde_json::Value;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use wiremock::Mock;
use wiremock::Request;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path_regex;

const OBJECTIVE: &str = "Inspect the repository and finish the requested repair.";
const PROGRESS_RESPONSE: &str = "I found the relevant code. Next I will add the regression tests.";
const FINAL_RESPONSE: &str = "Done. I repaired the boundary and added regression coverage.";
const CONTINUE_INSTRUCTION_FRAGMENT: &str = "previous response was a progress checkpoint";
const SUBAGENT_FINAL_REQUEST_FRAGMENT: &str = "final model request allowed for this sub-agent turn";

#[derive(Default)]
struct ProgressToolFinalResponder {
    calls: AtomicUsize,
}

impl Respond for ProgressToolFinalResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => text_response(PROGRESS_RESPONSE),
            1 => assessment_response("incomplete", "announces unperformed work"),
            2 => sse_response(tool_call_sse("call-lifecycle")),
            3 => text_response(FINAL_RESPONSE),
            4 => assessment_response("complete", "requested repair was delivered"),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

#[derive(Default)]
struct ToolThenFinalResponder {
    calls: AtomicUsize,
}

impl Respond for ToolThenFinalResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => sse_response(tool_call_sse("call-tool-final")),
            1 => text_response(FINAL_RESPONSE),
            2 => assessment_response("complete", "requested repair was delivered"),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

#[derive(Default)]
struct RepeatedCheckpointResponder {
    calls: AtomicUsize,
}

impl Respond for RepeatedCheckpointResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 | 2 => text_response(PROGRESS_RESPONSE),
            1 | 3 => assessment_response("incomplete", "work remains"),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

#[derive(Default)]
struct RepeatedMalformedAssessmentResponder {
    calls: AtomicUsize,
}

impl Respond for RepeatedMalformedAssessmentResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => sse_response(tool_call_sse("call-malformed-one")),
            1 => text_response(PROGRESS_RESPONSE),
            2 => text_response("not-json"),
            3 => sse_response(tool_call_sse("call-malformed-two")),
            4 => text_response("The second repair is underway."),
            5 => text_response("still-not-json"),
            6 => text_response(FINAL_RESPONSE),
            7 => assessment_response("complete", "requested repair was delivered"),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

#[derive(Default)]
struct MalformedAssessmentWithoutProgressResponder {
    calls: AtomicUsize,
}

#[derive(Default)]
struct EndlessToolResponder {
    calls: AtomicUsize,
}

impl Respond for EndlessToolResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        sse_response(tool_call_sse(format!("call-budget-{call}").as_str()))
    }
}

#[derive(Default)]
struct DelayedAssessmentResponder {
    calls: AtomicUsize,
}

impl Respond for DelayedAssessmentResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => sse_response(tool_call_sse("call-before-delayed-assessment")),
            1 => text_response(FINAL_RESPONSE),
            2 => assessment_response("complete", "requested repair was delivered")
                .set_delay(Duration::from_secs(7)),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

impl Respond for MalformedAssessmentWithoutProgressResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        match self.calls.fetch_add(1, Ordering::SeqCst) {
            0 => text_response(PROGRESS_RESPONSE),
            1 => text_response("not-json"),
            call => panic!("unexpected Kimi request {call}"),
        }
    }
}

fn tool_call_sse(id: &str) -> String {
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

fn text_response(text: &str) -> ResponseTemplate {
    let id = "chatcmpl-test";
    let delta = serde_json::json!({
        "id": id,
        "model": "k3",
        "choices": [{"delta": {"role": "assistant", "content": text}}],
    });
    let stop = serde_json::json!({
        "id": id,
        "model": "k3",
        "choices": [{"delta": {}, "finish_reason": "stop"}],
    });
    let usage = serde_json::json!({
        "id": id,
        "choices": [],
        "usage": {
            "prompt_tokens": 1,
            "completion_tokens": 1,
            "total_tokens": 2
        },
    });
    sse_response(format!(
        "data: {delta}\n\ndata: {stop}\n\ndata: {usage}\n\ndata: [DONE]\n\n"
    ))
}

fn assessment_response(state: &str, reason: &str) -> ResponseTemplate {
    text_response(
        &serde_json::json!({
            "state": state,
            "reason": reason,
        })
        .to_string(),
    )
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

fn reliable_provider(server: &wiremock::MockServer) -> ModelProviderInfo {
    ModelProviderInfo {
        name: "Reliable Chat Provider".to_string(),
        base_url: Some(format!("{}/v1", server.uri())),
        env_key: Some("PATH".to_string()),
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        stream_idle_timeout_ms: Some(2_000),
        ..built_in_model_providers(/*openai_base_url*/ None)[KIMI_CODE_PROVIDER_ID].clone()
    }
}

async fn submit_turn(test: &core_test_support::test_codex::TestCodex) -> Vec<EventMsg> {
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

    let mut events = Vec::new();
    loop {
        let event = test.codex.next_event().await.expect("next event");
        let terminal = matches!(event.msg, EventMsg::TurnComplete(_) | EventMsg::Error(_));
        events.push(event.msg);
        if terminal {
            break;
        }
    }
    events
}

fn request_bodies(requests: &[wiremock::Request]) -> Vec<Value> {
    requests
        .iter()
        .map(|request| serde_json::from_slice(&request.body).expect("request body"))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_progress_stop_continues_through_tool_and_final_result() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(ProgressToolFinalResponder::default())
        .expect(5)
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

    let events = submit_turn(&test).await;
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, EventMsg::TurnComplete(_)))
            .count(),
        1
    );
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, EventMsg::Error(_)))
    );

    let requests = server.received_requests().await.expect("recorded requests");
    let bodies = request_bodies(&requests);
    assert!(bodies[1].get("response_format").is_some());
    assert!(bodies[1].to_string().contains(PROGRESS_RESPONSE));
    assert!(
        bodies[2]
            .to_string()
            .contains(CONTINUE_INSTRUCTION_FRAGMENT)
    );
    assert_eq!(
        bodies
            .iter()
            .filter(|body| body.to_string().contains(CONTINUE_INSTRUCTION_FRAGMENT))
            .count(),
        1,
        "request-local continuation must not persist into later requests"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_tool_call_then_final_does_not_start_work_again() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(ToolThenFinalResponder::default())
        .expect(3)
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

    let events = submit_turn(&test).await;
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, EventMsg::Warning(_) | EventMsg::Error(_)))
    );

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|request| {
        !String::from_utf8_lossy(&request.body).contains(CONTINUE_INSTRUCTION_FRAGMENT)
    }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_repeated_no_progress_stops_once_with_warning() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(RepeatedCheckpointResponder::default())
        .expect(4)
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

    let events = submit_turn(&test).await;
    let warnings = events
        .iter()
        .filter_map(|event| match event {
            EventMsg::Warning(warning) => Some(warning.message.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        warnings,
        vec![
            "PfTerminal stopped automatic continuation because the model repeatedly ended without measurable progress. Review the result or continue manually."
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repeated_assessment_failure_continues_while_tools_make_progress() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(RepeatedMalformedAssessmentResponder::default())
        .expect(8)
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

    let events = submit_turn(&test).await;
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, EventMsg::Warning(_)))
    );

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(
        requests
            .iter()
            .filter(|request| String::from_utf8_lossy(&request.body)
                .contains(CONTINUE_INSTRUCTION_FRAGMENT))
            .count(),
        2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subagent_turn_hard_stops_after_configured_model_request_limit() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(EndlessToolResponder::default())
        .expect(3)
        .mount(&server)
        .await;

    let provider = kimi_provider(&server);
    let test = test_codex()
        .with_session_source(SessionSource::SubAgent(SubAgentSource::Other(
            "bounded-agent-test".to_string(),
        )))
        .with_config(move |config| {
            config.model = Some("k3".to_string());
            config.model_provider_id = KIMI_CODE_PROVIDER_ID.to_string();
            config.model_provider = provider;
            config.multi_agent_v2.max_subagent_model_requests_per_turn = 3;
        })
        .build(&server)
        .await
        .expect("build bounded subagent test session");

    let events = submit_turn(&test).await;
    assert!(events.iter().any(|event| {
        matches!(
            event,
            EventMsg::Warning(warning)
                if warning.message
                    == "Sub-agent turn stopped after the configured limit of 3 model requests. Review the last reported progress and send a focused follow-up task if more work is required."
        )
    }));

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 3);
    assert!(
        String::from_utf8_lossy(&requests[2].body).contains(SUBAGENT_FINAL_REQUEST_FRAGMENT),
        "the final allowed request must be told to return a result instead of calling more tools"
    );
    assert!(
        requests[..2].iter().all(|request| {
            !String::from_utf8_lossy(&request.body).contains(SUBAGENT_FINAL_REQUEST_FRAGMENT)
        }),
        "finalization guidance must not be injected before the last allowed request"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn assessment_failure_without_tool_progress_stops_with_warning() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(MalformedAssessmentWithoutProgressResponder::default())
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

    let events = submit_turn(&test).await;
    let warnings = events
        .iter()
        .filter_map(|event| match event {
            EventMsg::Warning(warning) => Some(warning.message.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        warnings,
        vec![
            "PfTerminal could not verify whether this action was complete; review the result before relying on it."
        ]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kimi_completion_assessment_accepts_observed_seven_second_tail_latency() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(DelayedAssessmentResponder::default())
        .expect(3)
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

    let events = submit_turn(&test).await;
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, EventMsg::Warning(_) | EventMsg::Error(_)))
    );
    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reliable_chat_provider_keeps_single_request_terminal_behavior() {
    skip_if_no_network!();

    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(".*/chat/completions$"))
        .respond_with(text_response(PROGRESS_RESPONSE))
        .expect(1)
        .mount(&server)
        .await;

    let provider = reliable_provider(&server);
    let test = test_codex()
        .with_config(move |config| {
            config.model = Some("k3".to_string());
            config.model_provider_id = "reliable-chat".to_string();
            config.model_provider = provider;
        })
        .build(&server)
        .await
        .expect("build test session");

    let events = submit_turn(&test).await;
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, EventMsg::Warning(_) | EventMsg::Error(_)))
    );
    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 1);
}
