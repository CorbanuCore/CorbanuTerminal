use super::super::ConnectionIdentity;
use super::test_model_client_with_provider;
use super::test_model_info;
use super::test_responses_metadata_for_client;
use super::test_session_telemetry;
use crate::Prompt;
use crate::test_support::TestCodexResponsesRequestKind;
use codex_api::AuthProvider;
use codex_api::ResponseEvent;
use codex_login::CodexAuth;
use codex_model_provider::BearerAuthProvider;
use codex_model_provider_info::WireApi;
use codex_model_provider_info::create_oss_provider_with_base_url;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::protocol::SessionSource;
use codex_rollout_trace::InferenceTraceContext;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::start_websocket_server;
use core_test_support::skip_if_no_network;
use futures::StreamExt;
use http::HeaderMap;
use http::HeaderValue;
use pretty_assertions::assert_eq;

struct HeaderAuth(HeaderMap);

impl AuthProvider for HeaderAuth {
    fn add_auth_headers(&self, headers: &mut HeaderMap) {
        headers.extend(self.0.clone());
    }
}

#[test]
fn api_only_identity_follows_resolved_credentials_without_exposing_them() {
    let old = BearerAuthProvider::new("synthetic-old-token".to_string());
    let same = BearerAuthProvider::new("synthetic-old-token".to_string());
    let new = BearerAuthProvider::new("synthetic-new-token".to_string());
    let anonymous = BearerAuthProvider::default();
    let identity = ConnectionIdentity::of(/*auth*/ None, &old);

    assert_eq!(identity, ConnectionIdentity::of(/*auth*/ None, &same));
    assert_ne!(identity, ConnectionIdentity::of(/*auth*/ None, &new));
    assert_ne!(identity, ConnectionIdentity::of(/*auth*/ None, &anonymous));
    assert_eq!(format!("{identity:?}"), "ConnectionIdentity(..)");
}

#[test]
fn api_only_identity_canonicalizes_header_names_but_preserves_value_bytes_and_order() {
    let mut headers = HeaderMap::new();
    headers.append("x-api-key", HeaderValue::from_static("synthetic-first"));
    headers.append("x-api-key", HeaderValue::from_static("synthetic-second"));
    headers.insert("x-account", HeaderValue::from_static("test-account"));
    let identity = ConnectionIdentity::of(/*auth*/ None, &HeaderAuth(headers.clone()));

    let mut reordered = HeaderMap::new();
    reordered.insert("x-account", HeaderValue::from_static("test-account"));
    reordered.append("x-api-key", HeaderValue::from_static("synthetic-first"));
    reordered.append("x-api-key", HeaderValue::from_static("synthetic-second"));
    assert_eq!(
        identity,
        ConnectionIdentity::of(/*auth*/ None, &HeaderAuth(reordered))
    );

    headers.remove("x-api-key");
    headers.append("x-api-key", HeaderValue::from_static("synthetic-second"));
    headers.append("x-api-key", HeaderValue::from_static("synthetic-first"));
    assert_ne!(
        identity,
        ConnectionIdentity::of(/*auth*/ None, &HeaderAuth(headers.clone()))
    );
    let reversed = ConnectionIdentity::of(/*auth*/ None, &HeaderAuth(headers.clone()));
    headers.append(
        "x-api-key",
        HeaderValue::from_bytes(b"synthetic-\xff").expect("header"),
    );
    assert_ne!(
        reversed,
        ConnectionIdentity::of(/*auth*/ None, &HeaderAuth(headers))
    );
}

#[test]
fn known_chatgpt_identity_keeps_the_connection_across_bearer_refreshes() {
    let auth = CodexAuth::create_dummy_chatgpt_auth_for_testing();
    let old = BearerAuthProvider::new("synthetic-old-token".to_string());
    let refreshed = BearerAuthProvider::new("synthetic-refreshed-token".to_string());

    assert_eq!(
        ConnectionIdentity::of(Some(&auth), &old),
        ConnectionIdentity::of(Some(&auth), &refreshed)
    );
}

/// Exercise the real preconnect and stream paths with resolved experimental
/// bearer credentials and no CodexAuth. Keep the cached socket while supplying
/// the next resolved client setup, as a credential rotation does.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn api_only_rotation_reconnects_a_prewarmed_socket_and_keeps_same_key_reuse() {
    skip_if_no_network!();
    let server = start_websocket_server(vec![
        vec![
            vec![ev_response_created("old-1"), ev_completed("old-1")],
            vec![ev_response_created("old-2"), ev_completed("old-2")],
            // A broken implementation can finish instead of timing out;
            // the handshake and connection assertions below then fail.
            vec![ev_response_created("stale-3"), ev_completed("stale-3")],
        ],
        vec![vec![ev_response_created("new-3"), ev_completed("new-3")]],
    ])
    .await;
    let thread_id = ThreadId::new();
    let client_with_token = |token: &str| {
        let mut provider =
            create_oss_provider_with_base_url(&format!("{}/v1", server.uri()), WireApi::Responses);
        provider.supports_websockets = true;
        provider.experimental_bearer_token = Some(token.to_string());
        test_model_client_with_provider(thread_id, SessionSource::Cli, provider)
    };
    let mut session = client_with_token("synthetic-old-token").new_session();
    let telemetry = test_session_telemetry();
    let model = test_model_info();
    let metadata = test_responses_metadata_for_client(
        &session.client,
        /*turn_id*/ None,
        "identity-test".to_string(),
        /*parent_thread_id*/ None,
        TestCodexResponsesRequestKind::WebsocketConnection,
    );
    assert!(
        session
            .client
            .current_client_setup()
            .await
            .expect("setup")
            .auth
            .is_none()
    );
    session
        .preconnect_websocket(&telemetry, &metadata)
        .await
        .expect("preconnect");

    let mut completions = Vec::new();
    for (turn, token) in [
        ("one", "synthetic-old-token"),
        ("two", "synthetic-old-token"),
        ("three", "synthetic-new-token"),
    ] {
        session.client = client_with_token(token);
        let metadata = test_responses_metadata_for_client(
            &session.client,
            Some(turn),
            "identity-test".to_string(),
            /*parent_thread_id*/ None,
            TestCodexResponsesRequestKind::Turn,
        );
        let mut stream = session
            .stream(
                &Prompt::default(),
                &model,
                &telemetry,
                /*effort*/ None,
                ReasoningSummary::None,
                /*service_tier*/ None,
                &metadata,
                &InferenceTraceContext::disabled(),
            )
            .await
            .expect("stream");
        while let Some(event) = stream.next().await {
            if let ResponseEvent::Completed { response_id, .. } = event.expect("event") {
                completions.push(response_id);
                break;
            }
        }
    }
    assert_eq!(completions, vec!["old-1", "old-2", "new-3"]);
    assert_eq!(
        server
            .handshakes()
            .iter()
            .map(|handshake| handshake.header("authorization"))
            .collect::<Vec<_>>(),
        vec![
            Some("Bearer synthetic-old-token".to_string()),
            Some("Bearer synthetic-new-token".to_string()),
        ]
    );
    assert_eq!(
        server
            .connections()
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    server.shutdown().await;
}
