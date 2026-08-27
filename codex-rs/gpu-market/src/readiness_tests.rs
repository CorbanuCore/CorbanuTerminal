use super::*;
use wiremock::Match;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::Request;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

const TOKEN: &str = "endpoint-test-secret";

#[derive(Debug)]
struct Authorization(&'static str);

impl Match for Authorization {
    fn matches(&self, request: &Request) -> bool {
        request
            .headers
            .get("authorization")
            .is_some_and(|value| value.to_str().ok() == Some(self.0))
    }
}

#[derive(Debug)]
struct PromptContains(&'static str);

impl Match for PromptContains {
    fn matches(&self, request: &Request) -> bool {
        request
            .body_json::<Value>()
            .ok()
            .and_then(|body| {
                body.pointer("/messages/0/content")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .is_some_and(|prompt| prompt.contains(self.0))
    }
}

async fn mount_endpoint(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(Authorization(format!("Bearer {TOKEN}").leak()))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "pinned/model"}]
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(401))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(PromptContains("exactly READY"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "READY"}}]
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(PromptContains("Stream two"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "data: {\"choices\":[{\"delta\":{\"content\":\"one\"}}]}\n\ndata: [DONE]\n\n",
        ))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(PromptContains("deliberately long"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(200))
                .set_body_string("data: [DONE]\n\n"),
        )
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(PromptContains("Call readiness_probe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"tool_calls": [{"function": {"name": "readiness_probe", "arguments": "{\"value\":\"ok\"}"}}]}}]
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn ready_requires_the_complete_authenticated_client_contract() {
    let server = MockServer::start().await;
    mount_endpoint(&server).await;
    let token = SecretValue::new(TOKEN.to_string()).expect("token");
    let report = GpuEndpointProber::new(Duration::from_secs(1), Duration::from_millis(50))
        .probe(
            format!("{}/v1", server.uri()).as_str(),
            "pinned/model",
            &token,
        )
        .await
        .expect("probe");

    assert!(report.ready(), "{report:?}");
    assert!(!format!("{report:?}").contains(TOKEN));
}

#[tokio::test]
async fn wrong_model_identity_never_becomes_ready() {
    let server = MockServer::start().await;
    mount_endpoint(&server).await;
    let token = SecretValue::new(TOKEN.to_string()).expect("token");
    let report = GpuEndpointProber::new(Duration::from_secs(1), Duration::from_millis(50))
        .probe(
            format!("{}/v1", server.uri()).as_str(),
            "different/model",
            &token,
        )
        .await
        .expect("probe");

    assert!(!report.model_identity_ok);
    assert!(!report.ready());
}

#[tokio::test]
async fn health_probe_checks_auth_and_identity_without_consuming_generation_capacity() {
    let server = MockServer::start().await;
    mount_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(503))
        .with_priority(1)
        .mount(&server)
        .await;
    let token = SecretValue::new(TOKEN.to_string()).expect("token");
    let prober = GpuEndpointProber::new(Duration::from_secs(1), Duration::from_millis(50));

    assert!(
        prober
            .probe_health(
                format!("{}/v1", server.uri()).as_str(),
                "pinned/model",
                &token,
            )
            .await
            .expect("health probe")
    );
}

#[tokio::test]
async fn cold_chat_can_outlive_normal_request_timeout_without_retry_injection() {
    let server = MockServer::start().await;
    mount_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(PromptContains("exactly READY"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(100))
                .set_body_json(serde_json::json!({
                    "choices": [{"message": {"content": "READY"}}]
                })),
        )
        .with_priority(1)
        .expect(1)
        .mount(&server)
        .await;
    let token = SecretValue::new(TOKEN.to_string()).expect("token");

    let report = GpuEndpointProber::new(Duration::from_millis(50), Duration::from_millis(20))
        .with_cold_chat_timeout(Duration::from_millis(250))
        .probe(
            format!("{}/v1", server.uri()).as_str(),
            "pinned/model",
            &token,
        )
        .await
        .expect("cold probe");

    assert!(report.ready(), "{report:?}");
}

#[tokio::test]
async fn a_fast_completed_response_does_not_masquerade_as_cancellation() {
    let server = MockServer::start().await;
    mount_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(PromptContains("deliberately long"))
        .respond_with(ResponseTemplate::new(200).set_body_string("data: [DONE]\n\n"))
        .with_priority(1)
        .mount(&server)
        .await;
    let token = SecretValue::new(TOKEN.to_string()).expect("token");
    let report = GpuEndpointProber::new(Duration::from_secs(1), Duration::from_millis(100))
        .probe(
            format!("{}/v1", server.uri()).as_str(),
            "pinned/model",
            &token,
        )
        .await
        .expect("probe");

    assert!(!report.cancellation_ok);
    assert!(!report.ready());
}

#[test]
fn public_plaintext_endpoint_is_rejected_before_network_access() {
    let token = SecretValue::new(TOKEN.to_string()).expect("token");
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let error = runtime
        .block_on(GpuEndpointProber::default().probe(
            "http://203.0.113.1:8000/v1",
            "pinned/model",
            &token,
        ))
        .expect_err("plaintext public endpoint");
    assert!(error.to_string().contains("HTTPS or loopback"));
}

#[test]
fn readiness_chat_budget_allows_reasoning_models_to_reach_final_content() {
    let body = chat_body(
        "pinned/model",
        "Reply with exactly READY.",
        /*stream*/ false,
    );

    assert_eq!(body["temperature"], 0);
    assert_eq!(body["max_tokens"], 4_096);

    let streaming = chat_body(
        "pinned/model",
        "Stream two short words.",
        /*stream*/ true,
    );
    assert_eq!(streaming["temperature"], 0);
    assert_eq!(streaming["max_tokens"], 256);
}
