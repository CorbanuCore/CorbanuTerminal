use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use codex_exec_server_protocol::JSONRPCMessage;
use codex_exec_server_protocol::JSONRPCNotification;
use codex_exec_server_protocol::JSONRPCRequest;
use codex_exec_server_protocol::JSONRPCResponse;
use codex_exec_server_protocol::RequestId;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use codex_network_proxy::NetworkDecision;
use codex_network_proxy::NetworkPolicyDecider;
use codex_network_proxy::NetworkPolicyRequest;
<<<<<<< HEAD
use codex_network_proxy::NetworkProxyConfig;
use codex_network_proxy::RemoteNetworkProxyConfig;
use codex_network_proxy::RemoteNetworkProxyLaunchConfig;
||||||| 413492cd6c
=======
use codex_network_proxy::NetworkProxyAuditMetadata;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_utils_path_uri::PathUri;
<<<<<<< HEAD
use futures::StreamExt;
||||||| 413492cd6c
=======
use http::HeaderMap;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::InMemorySpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use pretty_assertions::assert_eq;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::timeout;
<<<<<<< HEAD
use tokio_tungstenite::tungstenite::Message;
||||||| 413492cd6c
=======
use tracing::instrument::WithSubscriber;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::prelude::*;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

use super::super::ExecServerError;
use super::super::LazyRemoteExecServerClient;
use super::super::NetworkPolicyAuditContext;
use super::super::NetworkPolicyDecisionController;
use super::super::SessionState;
use super::super::handle_server_notification;
use super::accept_websocket;
use super::complete_websocket_initialize;
use super::read_jsonrpc_websocket;
use super::write_jsonrpc_websocket;
use crate::ProcessId;
use crate::client_api::ExecServerTransportParams;
use crate::protocol::EXEC_METHOD;
use crate::protocol::EXEC_TERMINATE_METHOD;
use crate::protocol::ExecParams;
use crate::protocol::ExecServerNetworkPolicyDecision;
use crate::protocol::ExecServerNetworkPolicyRequest;
use crate::protocol::ExecServerNetworkProtocol;
use crate::protocol::NETWORK_POLICY_DECISION_METHOD;
use crate::protocol::NETWORK_POLICY_REQUEST_METHOD;
use crate::protocol::NetworkPolicyDecisionNotification;
use crate::protocol::NetworkPolicyRequestParams;
use crate::protocol::NetworkPolicyRequestResponse;
use crate::rpc_server_requests::MAX_IN_FLIGHT_SERVER_CALLS;

struct PendingDecisionGuard(mpsc::UnboundedSender<()>);

impl Drop for PendingDecisionGuard {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

fn policy_request(request_id: i64, process_id: ProcessId, host: &str) -> JSONRPCMessage {
    JSONRPCMessage::Request(JSONRPCRequest {
        id: RequestId::Integer(request_id),
        method: NETWORK_POLICY_REQUEST_METHOD.to_string(),
        params: Some(
            serde_json::to_value(NetworkPolicyRequestParams {
                process_id,
                request: ExecServerNetworkPolicyRequest {
                    protocol: ExecServerNetworkProtocol::HttpsConnect,
                    host: host.to_string(),
                    port: 443,
                },
            })
            .expect("policy request should serialize"),
        ),
        trace: None,
    })
}

async fn read_decision(
    websocket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    request_id: i64,
) -> ExecServerNetworkPolicyDecision {
    let JSONRPCMessage::Response(response) = read_jsonrpc_websocket(websocket).await else {
        panic!("expected network policy response");
    };
    assert_eq!(response.id, RequestId::Integer(request_id));
    serde_json::from_value::<NetworkPolicyRequestResponse>(response.result)
        .expect("policy response should deserialize")
        .decision
}

#[tokio::test(flavor = "current_thread")]
async fn policy_decisions_reject_forged_process_and_use_trusted_controller_metadata() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!("ws://{}", listener.local_addr().expect("listener address"));
    let (release_tx, release_rx) = oneshot::channel();
    let (initialized_tx, initialized_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(
            &mut websocket,
            "audit-session",
            /*expected_resume_session_id*/ None,
        )
        .await;
        initialized_tx
            .send(())
            .expect("client should await completed WebSocket initialization");
        release_rx.await.expect("server should be released");
    });

    let logs = Arc::new(Mutex::new(Vec::new()));
    let writer_logs = Arc::clone(&logs);
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(move || AuditLogWriter(Arc::clone(&writer_logs))),
    );
    async move {
        let client = LazyRemoteExecServerClient::new(
            ExecServerTransportParams::websocket_url(websocket_url, Duration::from_secs(1)),
            HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
        )
        .get()
        .await
        .expect("client should connect");
        initialized_rx
            .await
            .expect("server should complete WebSocket initialization");
        let mut state = SessionState::new(/*recoverable*/ true);
        state.network_policy.audit = Some(NetworkPolicyAuditContext {
            metadata: NetworkProxyAuditMetadata {
                conversation_id: Some("trusted-conversation".to_string()),
                user_account_id: Some("trusted-account".to_string()),
                ..NetworkProxyAuditMetadata::default()
            },
            execution_id: Some("trusted-execution".to_string()),
        });
        client
            .inner
            .insert_session(&ProcessId::from("trusted-process"), Arc::new(state))
            .expect("trusted process should register");
        for (process_id, host) in [
            ("forged-process", "forged.example"),
            ("trusted-process", "trusted.example"),
        ] {
            handle_server_notification(
                &client.inner,
                JSONRPCNotification {
                    method: NETWORK_POLICY_DECISION_METHOD.to_string(),
                    params: Some(
                        serde_json::to_value(NetworkPolicyDecisionNotification {
                            process_id: ProcessId::from(process_id),
                            timestamp: "2026-08-11T12:00:00.000Z".to_string(),
                            scope: "domain".to_string(),
                            decision: "deny".to_string(),
                            source: "baseline_policy".to_string(),
                            reason: "not_allowed".to_string(),
                            protocol: ExecServerNetworkProtocol::HttpsConnect,
                            host: host.to_string(),
                            port: 443,
                            method: None,
                            client: None,
                            policy_override: false,
                        })
                        .expect("network policy decision should serialize"),
                    ),
                },
            )
            .await
            .expect("controller should handle network policy notification");
        }
        let output = String::from_utf8(
            logs.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        )
        .expect("audit log should be UTF-8");
        assert!(!output.contains("forged.example"));
        for expected in [
            "codex_otel.log_only",
            "trusted-conversation",
            "trusted-account",
            "trusted-execution",
        ] {
            assert!(
                output.contains(expected),
                "missing `{expected}` in {output}"
            );
        }
        release_tx.send(()).expect("server should be released");
    }
    .with_subscriber(subscriber)
    .await;
    server.await.expect("server should finish");
}

struct AuditLogWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for AuditLogWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn abandoned_process_start_unregisters_and_cleans_up() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!("ws://{}", listener.local_addr().expect("listener address"));
    let (start_seen_tx, start_seen_rx) = oneshot::channel();
    let (finish_start_tx, finish_start_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(&mut websocket, "p", Default::default()).await;
        let JSONRPCMessage::Request(start) = read_jsonrpc_websocket(&mut websocket).await else {
            panic!("expected process start request");
        };
        assert_eq!(start.method, EXEC_METHOD);
        start_seen_tx.send(()).expect("start should be observed");
        finish_start_rx.await.expect("start should be released");
        write_jsonrpc_websocket(
            &mut websocket,
            JSONRPCMessage::Response(JSONRPCResponse {
                id: start.id,
                result: serde_json::json!({"processId": "pending-start"}),
            }),
        )
        .await;
        let JSONRPCMessage::Request(terminate) = read_jsonrpc_websocket(&mut websocket).await
        else {
            panic!("expected process terminate request");
        };
        assert_eq!(terminate.method, EXEC_TERMINATE_METHOD);
        write_jsonrpc_websocket(
            &mut websocket,
            JSONRPCMessage::Response(JSONRPCResponse {
                id: terminate.id,
                result: serde_json::json!({"running": true}),
            }),
        )
        .await;
    });

    let client = LazyRemoteExecServerClient::new(
        ExecServerTransportParams::websocket_url(websocket_url, Duration::from_secs(1)),
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .get()
    .await
    .expect("client should connect");
    let process_id = ProcessId::from("pending-start");
    let start_client = client.clone();
    let start_process_id = process_id.clone();
    let start = tokio::spawn(async move {
        let params = ExecParams {
            metadata: Default::default(),
            process_id: start_process_id,
            argv: vec!["true".to_string()],
            cwd: PathUri::from_host_native_path(std::env::current_dir().expect("cwd"))
                .expect("cwd URI"),
            shell_snapshot: None,
            env_policy: None,
            env: Default::default(),
            tty: false,
            pipe_stdin: false,
            arg0: None,
            sandbox: None,
            enforce_managed_network: false,
            managed_network: None,
            network_proxy: None,
        };
        start_client
            .start_process(
                params, /*network_policy_decider*/ None, /*network_denial_observer*/ None,
            )
            .await
    });
    start_seen_rx.await.expect("start should be observed");
    let state = client
        .inner
        .get_session(&process_id)
        .expect("pending process should be registered");
    let decider: Arc<dyn NetworkPolicyDecider> =
        Arc::new(|_request: NetworkPolicyRequest| async { NetworkDecision::Allow });
    let decider_weak = Arc::downgrade(&decider);
    state
        .network_policy
        .controller
        .store(Some(Arc::new(NetworkPolicyDecisionController {
            decider,
            timeout: Duration::from_secs(30),
        })));

    start.abort();
    assert!(start.await.is_err_and(|error| error.is_cancelled()));
    assert!(state.network_policy.cancelled.is_cancelled());
    assert!(client.inner.get_session(&process_id).is_none());
    assert!(decider_weak.upgrade().is_none());

    finish_start_tx.send(()).expect("start should be released");
    server.await.expect("server task should finish");
}

#[tokio::test]
async fn policy_requests_use_process_decider_and_cancel_on_unregister() {
    let span_exporter = InMemorySpanExporter::default();
    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(span_exporter.clone())
        .build();
    let subscriber = tracing_subscriber::registry().with(
        tracing_opentelemetry::layer()
            .with_tracer(tracer_provider.tracer("exec-server-test"))
            .with_filter(filter_fn(codex_otel::OtelProvider::trace_export_filter)),
    );
    let _subscriber = tracing::subscriber::set_default(subscriber);
    tracing::callsite::rebuild_interest_cache();

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!(
        "ws://{}",
        listener.local_addr().expect("listener should have address")
    );
    let process_id = ProcessId::from("policy-process");
    let server_process_id = process_id.clone();
    let (ready_tx, ready_rx) = oneshot::channel();
    let (overflow_checked_tx, overflow_checked_rx) = oneshot::channel();
    let (unregistered_tx, unregistered_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(
            &mut websocket,
            "policy-session",
            /*expected_resume_session_id*/ None,
        )
        .await;
        ready_rx.await.expect("process should be registered");

        for (request_id, host, expected) in [
            (0, "allowed.example", ExecServerNetworkPolicyDecision::Allow),
            (
                2,
                "denied.example",
                ExecServerNetworkPolicyDecision::Deny {
                    reason: "blocked".to_string(),
                },
            ),
            (
                3,
                "invalid host",
                ExecServerNetworkPolicyDecision::Deny {
                    reason: "not_allowed".to_string(),
                },
            ),
            // #722: a host the prompt would show disguised is refused.
            (
                4,
                "allowed\u{200b}.example",
                ExecServerNetworkPolicyDecision::Deny {
                    reason: "not_allowed".to_string(),
                },
            ),
            (
                5,
                "elpmaxe.\u{202e}allowed",
                ExecServerNetworkPolicyDecision::Deny {
                    reason: "not_allowed".to_string(),
                },
            ),
        ] {
            write_jsonrpc_websocket(
                &mut websocket,
                policy_request(request_id, server_process_id.clone(), host),
            )
            .await;
            assert_eq!(read_decision(&mut websocket, request_id).await, expected);
        }

        let first_pending_request_id = 100;
        for offset in 0..MAX_IN_FLIGHT_SERVER_CALLS {
            write_jsonrpc_websocket(
                &mut websocket,
                policy_request(
                    first_pending_request_id + offset as i64,
                    server_process_id.clone(),
                    "pending.example",
                ),
            )
            .await;
        }
        let overflow_request_id = first_pending_request_id + MAX_IN_FLIGHT_SERVER_CALLS as i64;
        write_jsonrpc_websocket(
            &mut websocket,
            policy_request(
                overflow_request_id,
                server_process_id.clone(),
                "pending.example",
            ),
        )
        .await;
        assert_eq!(
            read_decision(&mut websocket, overflow_request_id).await,
            ExecServerNetworkPolicyDecision::Deny {
                reason: "not_allowed".to_string(),
            }
        );
        overflow_checked_tx.send(()).expect("overflow observed");

        unregistered_rx
            .await
            .expect("process should be unregistered");

        let post_unregister_request_id = 900;
        write_jsonrpc_websocket(
            &mut websocket,
            policy_request(
                post_unregister_request_id,
                server_process_id,
                "allowed.example",
            ),
        )
        .await;
        assert_eq!(
            read_decision(&mut websocket, post_unregister_request_id).await,
            ExecServerNetworkPolicyDecision::Deny {
                reason: "not_allowed".to_string(),
            }
        );
    });

    let client = LazyRemoteExecServerClient::new(
        ExecServerTransportParams::WebSocketUrl {
            websocket_url,
            connect_timeout: Duration::from_secs(1),
            initialize_timeout: Duration::from_secs(1),
            http_headers: HeaderMap::new(),
        },
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .get()
    .await
    .expect("client should connect");
    let session = client
        .register_session(&process_id)
        .await
        .expect("session should register");
    let (started_tx, mut started_rx) = mpsc::unbounded_channel();
    let (dropped_tx, mut dropped_rx) = mpsc::unbounded_channel();
    let decider: Arc<dyn NetworkPolicyDecider> = Arc::new(move |request: NetworkPolicyRequest| {
        let started_tx = started_tx.clone();
        let dropped_tx = dropped_tx.clone();
        async move {
            assert_eq!(
                tracing::Span::current()
                    .metadata()
                    .map(tracing::Metadata::name),
                Some("codex.exec_server.request"),
                "network policy decisions must run inside the inbound request span"
            );
            match request.host.as_str() {
                "allowed.example" => NetworkDecision::Allow,
                "denied.example" => NetworkDecision::deny("blocked"),
                "pending.example" => {
                    started_tx.send(()).expect("decision should start");
                    let _drop_guard = PendingDecisionGuard(dropped_tx);
                    std::future::pending().await
                }
                host => panic!("unexpected policy host: {host}"),
            }
        }
    });
    session.state.network_policy.controller.store(Some(Arc::new(
        NetworkPolicyDecisionController {
            decider,
            timeout: Duration::from_secs(30),
        },
    )));
    ready_tx.send(()).expect("server should be waiting");
    timeout(Duration::from_secs(5), async {
        for _ in 0..MAX_IN_FLIGHT_SERVER_CALLS {
            started_rx
                .recv()
                .await
                .expect("pending decision should start");
        }
    })
    .await
    .expect("pending decisions should start");
    overflow_checked_rx
        .await
        .expect("overflow should be observed");
    session.unregister().await;
    timeout(Duration::from_secs(5), async {
        for _ in 0..MAX_IN_FLIGHT_SERVER_CALLS {
            dropped_rx
                .recv()
                .await
                .expect("unregistered decision should be dropped");
        }
    })
    .await
    .expect("unregistered decisions should be cancelled");
    unregistered_tx
        .send(())
        .expect("server should verify late responses");
    timeout(Duration::from_secs(2), server)
        .await
        .expect("policy routing should finish")
        .expect("server task should finish");

    tracer_provider.force_flush().expect("flush traces");
    let spans = span_exporter.get_finished_spans().expect("span export");
    let policy_spans = spans
        .iter()
        .filter(|span| span.name.as_ref() == NETWORK_POLICY_REQUEST_METHOD)
        .collect::<Vec<_>>();
    assert!(
        !policy_spans.is_empty(),
        "network policy requests should export server spans"
    );
    let outcomes = policy_spans
        .iter()
        .map(|span| {
            span.attributes
                .iter()
                .find(|attribute| attribute.key.as_str() == "result")
                .map(|attribute| attribute.value.as_str().into_owned())
        })
        .collect::<Vec<_>>();
    assert!(
        outcomes.iter().all(Option::is_some),
        "completed, rejected, and cancelled policy requests must all record an outcome"
    );
    assert!(
        outcomes
            .iter()
            .any(|outcome| outcome.as_deref() == Some("success")),
        "completed and capacity-rejected requests should record successful responses"
    );
    assert!(
        outcomes
            .iter()
            .any(|outcome| outcome.as_deref() == Some("disconnected")),
        "cancelled requests should record disconnection"
    );
}

fn denied_notification(process_id: ProcessId, host: &str, reason: &str) -> JSONRPCMessage {
    JSONRPCMessage::Notification(codex_exec_server_protocol::JSONRPCNotification {
        method: crate::protocol::NETWORK_DENIED_METHOD.to_string(),
        params: Some(
            serde_json::to_value(crate::protocol::NetworkDeniedNotification {
                process_id,
                protocol: "http".to_string(),
                host: host.to_string(),
                port: Some(80),
                reason: reason.to_string(),
            })
            .expect("denial should serialize"),
        ),
    })
}

/// #722 (D278): the executor's own denials reach the process's observer as
/// denials, with untrusted strings neutralized; other processes' are ignored.
#[tokio::test]
async fn executor_denials_reach_the_process_observer() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!("ws://{}", listener.local_addr().expect("listener address"));
    let process_id = ProcessId::from("denied-process");
    let server_process_id = process_id.clone();
    let (ready_tx, ready_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(&mut websocket, "denials", Default::default()).await;
        ready_rx.await.expect("process should be registered");
        for message in [
            denied_notification(ProcessId::from("other-process"), "other.example", "denied"),
            denied_notification(server_process_id.clone(), "blocked.example", "denied"),
            denied_notification(
                server_process_id.clone(),
                "bad host\u{1b}[2J",
                "bad reason!",
            ),
            denied_notification(
                server_process_id,
                "\u{202e}blocked\u{200b}.exa\u{2066}mple\u{feff}",
                "denied",
            ),
        ] {
            write_jsonrpc_websocket(&mut websocket, message).await;
        }
        // Keep the connection open until the client is done.
        let _ = read_jsonrpc_websocket(&mut websocket).await;
    });

    let client = LazyRemoteExecServerClient::new(
        ExecServerTransportParams::websocket_url(websocket_url, Duration::from_secs(1)),
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .get()
    .await
    .expect("client should connect");
    let session = client
        .register_session(&process_id)
        .await
        .expect("session should register");
    let (denied_tx, mut denied_rx) = mpsc::unbounded_channel();
    let observer: Arc<dyn codex_network_proxy::BlockedRequestObserver> =
        Arc::new(move |blocked: codex_network_proxy::BlockedRequest| {
            let denied_tx = denied_tx.clone();
            async move {
                let _ = denied_tx.send(blocked);
            }
        });
    session.state.network_denial_observer.store(Some(Arc::new(
        super::super::NetworkDenialObserver(observer),
    )));
    ready_tx.send(()).expect("server should be waiting");

    let mut denials = Vec::new();
    for _ in 0..3 {
        let blocked = timeout(Duration::from_secs(5), denied_rx.recv())
            .await
            .expect("denial should arrive")
            .expect("observer channel open");
        denials.push((blocked.host, blocked.reason, blocked.decision, blocked.port));
    }
    assert_eq!(
        denials,
        vec![
            (
                "blocked.example".to_string(),
                "denied".to_string(),
                Some("deny".to_string()),
                Some(80),
            ),
            (
                String::new(),
                "executor_denied".to_string(),
                Some("deny".to_string()),
                Some(80),
            ),
            // #722: bidi and zero-width characters are stripped.
            (
                "blocked.example".to_string(),
                "denied".to_string(),
                Some("deny".to_string()),
                Some(80),
            ),
        ]
    );
    assert!(denied_rx.try_recv().is_err(), "other processes are ignored");
    session.unregister().await;
    drop(client);
    server.abort();
}

/// #722 (D278): a launch asking for denial reports must come with an
/// observer, and an observer with a launch that asks for them; anything else
/// is refused before a process is started.
#[tokio::test]
async fn denial_reports_must_match_the_observer() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!("ws://{}", listener.local_addr().expect("listener address"));
    let (done_tx, done_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(&mut websocket, "mismatch", Default::default()).await;
        done_rx.await.expect("client should finish");
        assert!(
            timeout(Duration::from_millis(200), websocket.next())
                .await
                .is_err(),
            "no process start should be sent"
        );
    });

    let client = LazyRemoteExecServerClient::new(
        ExecServerTransportParams::websocket_url(websocket_url, Duration::from_secs(1)),
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .get()
    .await
    .expect("client should connect");
    let process_id = ProcessId::from("mismatch-process");
    let observer: Arc<dyn codex_network_proxy::BlockedRequestObserver> =
        Arc::new(|_blocked: codex_network_proxy::BlockedRequest| async {});
    for (report_denials, observer) in [(true, None), (false, Some(observer))] {
        let mut launch = RemoteNetworkProxyLaunchConfig::new(
            RemoteNetworkProxyConfig::from_effective_config(&NetworkProxyConfig::default())
                .expect("default remote config"),
        );
        launch.report_denials = report_denials;
        let params = ExecParams {
            process_id: process_id.clone(),
            argv: vec!["true".to_string()],
            cwd: PathUri::from_host_native_path(std::env::current_dir().expect("cwd"))
                .expect("cwd URI"),
            env_policy: None,
            env: Default::default(),
            tty: false,
            pipe_stdin: false,
            arg0: None,
            sandbox: None,
            enforce_managed_network: false,
            managed_network: None,
            network_proxy: Some(launch),
        };
        let Err(ExecServerError::Protocol(message)) = client
            .start_process(params, /*network_policy_decider*/ None, observer)
            .await
        else {
            panic!("a mismatched launch should be refused (report_denials={report_denials})");
        };
        assert_eq!(
            message,
            "network denial reports must match the configured observer"
        );
        assert!(client.inner.get_session(&process_id).is_none());
    }
    done_tx.send(()).expect("server should be waiting");
    timeout(Duration::from_secs(2), server)
        .await
        .expect("server should finish")
        .expect("server task should finish");
}

/// #722 (D278): a `network/denied` notification that isn't valid JSON for
/// the protocol (a port above 65535) is a protocol error: nothing is
/// recorded and the connection is dropped, like any malformed notification.
#[tokio::test]
async fn malformed_denial_drops_the_connection() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!("ws://{}", listener.local_addr().expect("listener address"));
    let process_id = ProcessId::from("malformed-process");
    let server_process_id = process_id.clone();
    let (ready_tx, ready_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(&mut websocket, "malformed", Default::default()).await;
        ready_rx.await.expect("process should be registered");
        write_jsonrpc_websocket(
            &mut websocket,
            JSONRPCMessage::Notification(codex_exec_server_protocol::JSONRPCNotification {
                method: crate::protocol::NETWORK_DENIED_METHOD.to_string(),
                params: Some(serde_json::json!({
                    "processId": server_process_id,
                    "protocol": "http",
                    "host": "blocked.example",
                    "port": 70000,
                    "reason": "denied",
                })),
            }),
        )
        .await;
        loop {
            match timeout(Duration::from_secs(5), websocket.next())
                .await
                .expect("the client should drop the connection")
            {
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                Some(Ok(Message::Text(text))) => panic!("unexpected message: {text}"),
                Some(Ok(Message::Binary(_))) => panic!("unexpected binary message"),
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(Message::Frame(_))) => {}
            }
        }
    });

    let client = LazyRemoteExecServerClient::new(
        ExecServerTransportParams::websocket_url(websocket_url, Duration::from_secs(1)),
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .get()
    .await
    .expect("client should connect");
    let session = client
        .register_session(&process_id)
        .await
        .expect("session should register");
    let (denied_tx, mut denied_rx) = mpsc::unbounded_channel();
    let observer: Arc<dyn codex_network_proxy::BlockedRequestObserver> =
        Arc::new(move |blocked: codex_network_proxy::BlockedRequest| {
            let denied_tx = denied_tx.clone();
            async move {
                let _ = denied_tx.send(blocked);
            }
        });
    session.state.network_denial_observer.store(Some(Arc::new(
        super::super::NetworkDenialObserver(observer),
    )));
    ready_tx.send(()).expect("server should be waiting");

    timeout(Duration::from_secs(6), server)
        .await
        .expect("server should see the connection close")
        .expect("server task should finish");
    assert!(
        denied_rx.try_recv().is_err(),
        "a malformed denial is not recorded"
    );
    session.unregister().await;
}
