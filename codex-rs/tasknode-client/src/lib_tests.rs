use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use codex_keyring_store::tests::MockKeyringStore;
use codex_vault::Vault;
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::Mock;
use wiremock::Respond;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::matchers::query_param;

use super::*;

struct SequenceResponder {
    calls: AtomicUsize,
    responses: Vec<ResponseTemplate>,
}

impl SequenceResponder {
    fn new(responses: Vec<ResponseTemplate>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            responses,
        }
    }
}

impl Respond for SequenceResponder {
    fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        self.responses
            .get(call)
            .cloned()
            .expect("sequence response should exist")
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_github_link_parses_camel_case_fields() {
    let server = wiremock::MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth/terminal/start/github"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "requestId": "req-123",
            "pollToken": "poll-456",
            "verificationUrl": "https://verify.example/link",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let origin = server.uri();
    let started = tokio::task::spawn_blocking(move || {
        TaskNodeClient::new_without_token_for_origin(origin).start_github_link()
    })
    .await
    .expect("worker should not panic")
    .expect("start should parse");

    assert_eq!(
        started,
        TerminalAuthStartResponse {
            request_id: "req-123".to_string(),
            poll_token: "poll-456".to_string(),
            verification_url: "https://verify.example/link".to_string(),
        }
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn poll_session_can_progress_from_pending_to_success() {
    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/auth/terminal/session"))
        .and(query_param("requestId", "req-123"))
        .and(query_param("pollToken", "poll-456"))
        .respond_with(SequenceResponder::new(vec![
            ResponseTemplate::new(202),
            ResponseTemplate::new(202),
            ResponseTemplate::new(200).set_body_json(json!({
                "accountId": "acct-1",
                "githubUsername": "octocat",
                "terminalToken": "terminal-secret",
                "expiresAt": "2026-07-08T12:00:00Z",
            })),
        ]))
        .expect(3)
        .mount(&server)
        .await;

    let origin = server.uri();
    let responses = tokio::task::spawn_blocking(move || {
        let client = TaskNodeClient::new_without_token_for_origin(origin);
        vec![
            client.poll_session("req-123", "poll-456"),
            client.poll_session("req-123", "poll-456"),
            client.poll_session("req-123", "poll-456"),
        ]
    })
    .await
    .expect("worker should not panic");

    assert!(matches!(responses[0], Err(TaskNodeClientError::Pending)));
    assert!(matches!(responses[1], Err(TaskNodeClientError::Pending)));
    assert_eq!(
        responses[2].as_ref().expect("third poll should succeed"),
        &TerminalSessionResponse {
            account_id: "acct-1".to_string(),
            github_username: Some("octocat".to_string()),
            terminal_token: "terminal-secret".to_string(),
            expires_at: Some("2026-07-08T12:00:00Z".to_string()),
        }
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn poll_session_returns_rejected_for_definitive_failures() {
    let server = wiremock::MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/auth/terminal/session"))
        .respond_with(SequenceResponder::new(vec![
            ResponseTemplate::new(202),
            ResponseTemplate::new(410).set_body_json(json!({
                "message": "link request expired",
            })),
        ]))
        .expect(2)
        .mount(&server)
        .await;

    let origin = server.uri();
    let responses = tokio::task::spawn_blocking(move || {
        let client = TaskNodeClient::new_without_token_for_origin(origin);
        vec![
            client.poll_session("req-123", "poll-456"),
            client.poll_session("req-123", "poll-456"),
        ]
    })
    .await
    .expect("worker should not panic");

    assert!(matches!(responses[0], Err(TaskNodeClientError::Pending)));
    assert!(matches!(
        &responses[1],
        Err(TaskNodeClientError::Rejected(message)) if message == "link request expired"
    ));
}

#[test]
fn session_round_trips_through_real_vault_with_mock_keyring() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let keyring = Arc::new(MockKeyringStore::default());
    let vault = Vault::new_with_keyring_store(codex_home.path().to_path_buf(), keyring.clone());
    let session = TaskNodeLocalSession {
        origin: "https://tasknode.example".to_string(),
        account_id: Some("acct-1".to_string()),
        github_username: Some("octocat".to_string()),
        terminal_token: Some("terminal-secret".to_string()),
        expires_at: Some("2026-07-08T12:00:00Z".to_string()),
        pending_request_id: None,
        pending_poll_token: None,
        pending_verification_url: None,
    };

    save_to_vault(&session, &vault).expect("save session");

    let vault = Vault::new_with_keyring_store(codex_home.path().to_path_buf(), keyring);
    assert_eq!(load_from_vault(&vault).expect("load session"), session);
}

#[test]
fn old_format_session_deserializes_and_resaves_all_fields() {
    let old = json!({
        "origin": "https://tasknode.example",
        "terminal_token": "terminal-secret",
        "pending_verification_url": "https://verify.example/link",
    });

    let session: TaskNodeLocalSession =
        serde_json::from_value(old).expect("old-format session should deserialize");
    assert_eq!(session.origin, "https://tasknode.example");
    assert_eq!(session.terminal_token.as_deref(), Some("terminal-secret"));
    assert_eq!(
        session.pending_verification_url.as_deref(),
        Some("https://verify.example/link")
    );
    assert_eq!(session.account_id, None);
    assert_eq!(session.github_username, None);
    assert_eq!(session.expires_at, None);
    assert_eq!(session.pending_request_id, None);
    assert_eq!(session.pending_poll_token, None);

    let encoded = serde_json::to_value(&session).expect("serialize full session");
    let object = encoded
        .as_object()
        .expect("session should serialize as object");
    assert_eq!(object.len(), 8);
    for key in [
        "origin",
        "account_id",
        "github_username",
        "terminal_token",
        "expires_at",
        "pending_request_id",
        "pending_poll_token",
        "pending_verification_url",
    ] {
        assert!(object.contains_key(key), "missing serialized key {key}");
    }
}

#[test]
fn session_debug_redacts_secret_token_values() {
    let session = TaskNodeLocalSession {
        origin: "https://tasknode.example".to_string(),
        account_id: Some("acct-1".to_string()),
        github_username: Some("octocat".to_string()),
        terminal_token: Some("terminal-secret".to_string()),
        expires_at: None,
        pending_request_id: Some("req-123".to_string()),
        pending_poll_token: Some("poll-secret".to_string()),
        pending_verification_url: Some("https://verify.example/link".to_string()),
    };

    let debug = format!("{session:?}");
    assert!(!debug.contains("terminal-secret"), "{debug}");
    assert!(!debug.contains("poll-secret"), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
}
