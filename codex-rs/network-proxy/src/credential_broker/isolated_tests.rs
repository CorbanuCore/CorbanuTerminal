//! PF-27-S04 qualification of the isolated broker process.
//!
//! Each test re-executes this test binary as the broker (filtered to the
//! child-entry test below) so the process boundary is real.

use super::IsolatedBrokerError;
use super::IsolatedBrokerLauncher;
use super::IsolatedBrokerOptions;
use super::protocol::BROKER_ERROR_HEADER;
use super::protocol::HostBindingWire;
use super::protocol::ProviderId;
use super::run_credential_broker_main;
use crate::credential_broker::CredentialBroker;
use crate::credential_broker::CredentialRouting;
use crate::credential_broker::ScopedCredentialInjectionError;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::ProviderRequestOperation;
use pretty_assertions::assert_eq;
use rama_core::Layer as _;
use rama_core::bytes::Bytes;
use rama_core::service::service_fn;
use rama_http::Body;
use rama_http::BodyExtractExt as _;
use rama_http::HeaderMap;
use rama_http::HeaderValue;
use rama_http::Request;
use rama_http::Response;
use rama_http::header::AUTHORIZATION;
use rama_http_backend::server::HttpServer;
use rama_tcp::server::TcpListener;
use rama_tls_rustls::server::TlsAcceptorLayer;
use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

const CHILD_ENV: &str = "CODEX_PF27_ISOLATED_BROKER_CHILD";
const CHILD_TEST: &str =
    "credential_broker::isolated::tests::pf_27_s04_pf_27_s01_isolated_broker_child_entry";
const UPSTREAM_HOST: &str = "127.0.0.1";
const SYNTHETIC_TOKEN: &str = "ghp_pf27SyntheticCanary0000000000000000000000";
const STREAM_PATH: &str = "/stream";

#[test]
fn pf_27_s04_pf_27_s01_isolated_broker_child_entry() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_credential_broker_main();
    }
}

struct Upstream {
    port: u16,
    _ca_dir: TempDir,
    ca_path: PathBuf,
}

/// Local TLS upstream that echoes the Authorization header it received.
async fn start_upstream() -> Upstream {
    let (ca_pem, acceptor) =
        crate::certs::test_ca_and_host_acceptor(UPSTREAM_HOST).expect("test TLS material");
    let ca_dir = tempfile::tempdir().expect("ca dir");
    let ca_path = ca_dir.path().join("ca.pem");
    std::fs::write(&ca_path, ca_pem).expect("write test CA");
    let listener = TcpListener::bind_address("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let port = listener.local_addr().expect("upstream addr").port();
    let service = TlsAcceptorLayer::new(acceptor).into_layer(HttpServer::http1().service(
        service_fn(|request: Request| async move {
            let authorization = request
                .headers()
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("<none>")
                .to_string();
            let body = if request.uri().path() == STREAM_PATH {
                let first = rama_core::futures::stream::iter([Ok::<_, Infallible>(Bytes::from(
                    "first-chunk\n",
                ))]);
                Body::from_stream(rama_core::futures::StreamExt::chain(
                    first,
                    rama_core::futures::stream::pending(),
                ))
            } else {
                Body::from(authorization)
            };
            Ok::<_, Infallible>(Response::new(body))
        }),
    ));
    tokio::spawn(listener.serve(service));
    Upstream {
        port,
        _ca_dir: ca_dir,
        ca_path,
    }
}

fn launcher(upstream: &Upstream, controller_pid_override: Option<u32>) -> IsolatedBrokerLauncher {
    IsolatedBrokerLauncher::test_harness(
        /*program*/ None,
        [CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"]
            .into_iter()
            .map(OsString::from)
            .collect(),
        vec![
            (OsString::from(CHILD_ENV), OsString::from("1")),
            (
                OsString::from("SSL_CERT_FILE"),
                upstream.ca_path.clone().into_os_string(),
            ),
        ],
        controller_pid_override,
    )
}

fn options() -> IsolatedBrokerOptions {
    IsolatedBrokerOptions {
        allow_local_binding: true,
        allow_upstream_proxy: false,
        runtime_dir: None,
        require_containment: true,
    }
}

fn isolated_broker(launcher: IsolatedBrokerLauncher) -> CredentialBroker {
    CredentialBroker::new_isolated_with_launcher(/*enabled*/ true, options(), launcher)
}

fn virtualized_dummy(broker: &CredentialBroker) -> String {
    let mut env = HashMap::from([
        ("GH_HOST".to_string(), UPSTREAM_HOST.to_string()),
        (
            "GH_ENTERPRISE_TOKEN".to_string(),
            SYNTHETIC_TOKEN.to_string(),
        ),
    ]);
    broker.virtualize_child_env(&mut env);
    let dummy = env["GH_ENTERPRISE_TOKEN"].clone();
    assert_ne!(dummy, SYNTHETIC_TOKEN);
    dummy
}

fn bearer(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {value}")).expect("bearer"),
    );
    headers
}

fn request(path: &str, dummy: &str) -> Request {
    Request::builder()
        .method("GET")
        .uri(path)
        .header(AUTHORIZATION, format!("Bearer {dummy}"))
        .body(Body::empty())
        .expect("request")
}

fn route(
    broker: &CredentialBroker,
    port: u16,
    path: &str,
    dummy: &str,
) -> Result<CredentialRouting, ScopedCredentialInjectionError> {
    broker.route_request_credentials(
        "https",
        UPSTREAM_HOST,
        port,
        "GET",
        path,
        &mut bearer(dummy),
    )
}

async fn forward(broker: &CredentialBroker, port: u16, path: &str, dummy: &str) -> Response {
    let Ok(CredentialRouting::Brokered(route)) = route(broker, port, path, dummy) else {
        panic!("expected a brokered route");
    };
    route
        .forward(request(path, dummy))
        .await
        .expect("broker response")
}

fn denial(response: &Response) -> Option<&str> {
    response
        .headers()
        .get(BROKER_ERROR_HEADER)
        .and_then(|value| value.to_str().ok())
}

fn operation(port: u16, path: &str) -> ProviderRequestOperation {
    ProviderRequestOperation::new(UPSTREAM_HOST, port, "GET", path).expect("operation")
}

fn registered_reference(broker: &CredentialBroker, port: u16, dummy: &str) -> CredentialReference {
    match route(broker, port, "/echo", dummy) {
        Ok(CredentialRouting::Brokered(route)) => route.reference,
        _ => panic!("expected a brokered route"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_raw_credential_is_substituted_only_inside_the_broker() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let dummy = virtualized_dummy(&broker);

    assert!(!broker.holds_raw_value(SYNTHETIC_TOKEN));
    let response = forward(&broker, upstream.port, "/echo", &dummy).await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.try_into_string().await.expect("body"),
        format!("Bearer {SYNTHETIC_TOKEN}")
    );
    assert!(!broker.holds_raw_value(SYNTHETIC_TOKEN));

    // An agent that spoke HTTP/2 to the proxy still reaches an HTTP/1.1-only origin.
    let Ok(CredentialRouting::Brokered(route)) = route(&broker, upstream.port, "/echo", &dummy)
    else {
        panic!("expected a brokered route");
    };
    let mut http2 = request("/echo", &dummy);
    *http2.version_mut() = rama_http::Version::HTTP_2;
    let response = route.forward(http2).await.expect("broker response");
    assert_eq!(response.status(), 200);

    // The in-process legacy path has nothing to inject: the request keeps its dummy.
    let mut headers = bearer(&dummy);
    broker
        .inject_request_headers_for_request(
            "https",
            UPSTREAM_HOST,
            upstream.port,
            "GET",
            "/echo",
            &mut headers,
        )
        .expect("legacy path");
    assert_eq!(headers, bearer(&dummy));
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_wrong_os_peer_is_disconnected_before_any_request() {
    let upstream = start_upstream().await;
    // The broker is told its controller is pid 1, so this process is a wrong peer.
    let broker = isolated_broker(launcher(
        &upstream,
        /*controller_pid_override*/ Some(1),
    ));
    let dummy = virtualized_dummy(&broker);
    let Ok(CredentialRouting::Brokered(route)) = route(&broker, upstream.port, "/echo", &dummy)
    else {
        panic!("expected a brokered route");
    };

    assert_eq!(
        route.forward(request("/echo", &dummy)).await.err(),
        Some(IsolatedBrokerError::Unavailable)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_forged_tampered_and_replayed_frames_are_denied() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let dummy = virtualized_dummy(&broker);
    let reference = registered_reference(&broker, upstream.port, &dummy);
    let client = broker.current_isolated_client().expect("broker client");
    let operation = operation(upstream.port, "/echo");

    let forged = CredentialReference::from_sha256_hex("b".repeat(64)).expect("forged");
    let frame = client.sign_frame(&forged, &operation).expect("frame");
    let response = client
        .send_with_frame(&operation, request("/echo", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("unknown_credential"));

    let response = client
        .send_with_frame(&operation, request("/echo", &dummy), "not-a-frame")
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("malformed_frame"));

    let wrong_key = BrokerChannelMac::from_secret([9; 32])
        .sign_provider_request(
            client.binding(client.run_generation()),
            /*sequence*/ 900,
            reference.clone(),
            operation.clone(),
        )
        .expect("wrong-key frame");
    let wrong_key = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        wrong_key.as_bytes(),
    );
    let response = client
        .send_with_frame(&operation, request("/echo", &dummy), &wrong_key)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("authentication_failed"));

    let frame = client.sign_frame(&reference, &operation).expect("frame");
    let first = client
        .send_with_frame(&operation, request("/echo", &dummy), &frame)
        .await
        .expect("first");
    assert_eq!(first.status(), 200);
    let replay = client
        .send_with_frame(&operation, request("/echo", &dummy), &frame)
        .await
        .expect("replay");
    assert_eq!(denial(&replay), Some("replay"));
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_cross_run_unbound_host_and_mismatched_requests_are_denied() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let dummy = virtualized_dummy(&broker);
    let reference = registered_reference(&broker, upstream.port, &dummy);
    let client = broker.current_isolated_client().expect("broker client");
    let operation = operation(upstream.port, "/echo");

    let mut other_run = client.binding(client.run_generation());
    other_run.run_id = "controller-other-run".to_string();
    let frame = client.sign_frame_with_binding(other_run, &reference, &operation);
    let response = client
        .send_with_frame(&operation, request("/echo", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("binding_mismatch"));

    let unbound = ProviderRequestOperation::new("127.0.0.2", upstream.port, "GET", "/echo")
        .expect("unbound operation");
    let frame = client.sign_frame(&reference, &unbound).expect("frame");
    let response = client
        .send_with_frame(&unbound, request("/echo", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("host_not_bound"));

    let frame = client.sign_frame(&reference, &operation).expect("frame");
    let mismatched = ProviderRequestOperation::new(UPSTREAM_HOST, upstream.port, "GET", "/other")
        .expect("mismatched operation");
    let response = client
        .send_with_frame(&mismatched, request("/other", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("request_mismatch"));
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_revocation_closes_open_channels_and_old_generations() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let dummy = virtualized_dummy(&broker);
    let reference = registered_reference(&broker, upstream.port, &dummy);
    let client = broker.current_isolated_client().expect("broker client");
    let old_generation = client.run_generation();

    let response = forward(&broker, upstream.port, STREAM_PATH, &dummy).await;
    assert_eq!(response.status(), 200);
    let reader = tokio::spawn(async move { response.try_into_string().await.is_err() });
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(broker.revoke_isolated_credentials());
    let stream_failed = tokio::time::timeout(Duration::from_secs(10), reader)
        .await
        .expect("revocation closes the open download")
        .expect("reader task");
    assert!(stream_failed);

    // Core forgot the reference, so the dummy no longer routes to the broker.
    assert!(matches!(
        route(&broker, upstream.port, "/echo", &dummy),
        Ok(CredentialRouting::Direct)
    ));
    // A frame from the old generation cannot reach the old credential.
    let operation = operation(upstream.port, "/echo");
    let frame =
        client.sign_frame_with_binding(client.binding(old_generation), &reference, &operation);
    let response = client
        .send_with_frame(&operation, request("/echo", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("stale_generation"));
    let frame = client.sign_frame(&reference, &operation).expect("frame");
    let response = client
        .send_with_frame(&operation, request("/echo", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("unknown_credential"));
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_broker_crash_fails_closed_and_restart_rejects_old_handles() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let old_dummy = virtualized_dummy(&broker);
    let old_reference = registered_reference(&broker, upstream.port, &old_dummy);
    let old_client = broker.current_isolated_client().expect("broker client");

    old_client.kill_for_test();
    assert_eq!(
        route(&broker, upstream.port, "/echo", &old_dummy).err(),
        Some(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
    );
    assert!(!old_client.socket_path().exists());

    // A dead broker is not replaced: later children still get only dummies and
    // nothing is injected until Core restarts.
    let after_crash = virtualized_dummy(&broker);
    assert_ne!(after_crash, old_dummy);
    assert!(!broker.holds_raw_value(SYNTHETIC_TOKEN));
    assert_eq!(
        route(&broker, upstream.port, "/echo", &after_crash).err(),
        Some(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
    );
    assert!(matches!(
        route(&broker, upstream.port, "/echo", &old_dummy),
        Ok(CredentialRouting::Direct)
    ));

    // A restarted controller gets a fresh broker that never honors old handles.
    let restarted = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let new_dummy = virtualized_dummy(&restarted);
    let new_client = restarted
        .current_isolated_client()
        .expect("restarted broker");
    assert_ne!(new_client.broker_instance(), old_client.broker_instance());
    let operation = operation(upstream.port, "/echo");
    let frame = new_client
        .sign_frame(&old_reference, &operation)
        .expect("frame");
    let response = new_client
        .send_with_frame(&operation, request("/echo", &new_dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("unknown_credential"));

    let response = forward(&restarted, upstream.port, "/echo", &new_dummy).await;
    assert_eq!(
        response.try_into_string().await.expect("body"),
        format!("Bearer {SYNTHETIC_TOKEN}")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_broker_resources_and_registrations_are_bounded() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let _ = virtualized_dummy(&broker);
    let client = broker.current_isolated_client().expect("broker client");
    let binding = HostBindingWire {
        exact_hosts: vec![UPSTREAM_HOST.to_string()],
        suffixes: Vec::new(),
    };

    for index in 1..super::server::MAX_BROKER_CREDENTIALS {
        client
            .register(
                ProviderId::Github,
                binding.clone(),
                &format!("ghp_bounded{index:040}"),
            )
            .expect("bounded registration");
    }
    assert_eq!(
        client
            .register(ProviderId::Github, binding.clone(), "ghp_overflow")
            .err(),
        Some(IsolatedBrokerError::Rejected)
    );
    assert_eq!(
        client
            .register(ProviderId::Github, binding, &"x".repeat(9 * 1024))
            .err(),
        Some(IsolatedBrokerError::Rejected)
    );
    let unbounded = HostBindingWire {
        exact_hosts: (0..17).map(|index| format!("h{index}.example")).collect(),
        suffixes: Vec::new(),
    };
    assert_eq!(
        client
            .register(ProviderId::Github, unbounded, "ghp_unbounded")
            .err(),
        Some(IsolatedBrokerError::Rejected)
    );
    assert!(client.is_alive());
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_controller_exit_stops_the_broker() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let _ = virtualized_dummy(&broker);
    let socket_path = broker
        .current_isolated_client()
        .expect("broker client")
        .socket_path()
        .to_path_buf();
    assert!(socket_path.exists());

    drop(broker);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while socket_path.exists() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!socket_path.exists());
}

#[test]
fn pf_27_s04_pf_27_s01_unavailable_broker_never_exposes_or_injects_raw_values() {
    let broker = isolated_broker(IsolatedBrokerLauncher::test_harness(
        Some(PathBuf::from("/nonexistent/corbanu-broker")),
        Vec::new(),
        Vec::new(),
        /*controller_pid_override*/ None,
    ));
    let dummy = virtualized_dummy(&broker);

    assert!(!broker.holds_raw_value(SYNTHETIC_TOKEN));
    assert_eq!(
        route(&broker, /*port*/ 443, "/echo", &dummy).err(),
        Some(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s04_pf_27_s01_externally_terminated_broker_is_detected_and_cleaned_up() {
    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let dummy = virtualized_dummy(&broker);
    let client = broker.current_isolated_client().expect("broker client");
    let pid = client.pid_for_test().expect("broker pid");
    let status = std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .expect("kill");
    assert!(status.success());
    // The broker removes its own socket directory on SIGTERM.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while client.socket_path().exists() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(!client.socket_path().exists());
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_eq!(
        route(&broker, upstream.port, "/echo", &dummy).err(),
        Some(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s02_broker_sockets_live_in_the_private_runtime_dir() {
    let upstream = start_upstream().await;
    let runtime = tempfile::Builder::new()
        .prefix("pf27s02-")
        .tempdir_in("/tmp")
        .expect("runtime dir");
    let runtime_dir = std::fs::canonicalize(runtime.path()).expect("canonical runtime dir");
    let broker = CredentialBroker::new_isolated_with_launcher(
        /*enabled*/ true,
        IsolatedBrokerOptions {
            runtime_dir: Some(runtime_dir.clone()),
            ..options()
        },
        launcher(&upstream, /*controller_pid_override*/ None),
    );
    let dummy = virtualized_dummy(&broker);
    let client = broker.current_isolated_client().expect("broker client");
    let broker_dir = client.socket_path().parent().expect("broker dir");

    assert_eq!(broker_dir.parent(), Some(runtime_dir.as_path()));
    // The control socket is unlinked once the controller is connected.
    assert!(!broker_dir.join("c.sock").exists());
    let mode = std::os::unix::fs::PermissionsExt::mode(
        &std::fs::metadata(broker_dir)
            .expect("broker dir")
            .permissions(),
    );
    assert_eq!(mode & 0o777, 0o700);
    if cfg!(target_os = "macos") {
        assert_eq!(client.containment(), "seatbelt");
    } else if cfg!(target_os = "linux") {
        assert!(
            client.containment().contains("seccomp"),
            "{}",
            client.containment()
        );
    }

    // Brokered requests still work from the contained broker.
    let response = forward(&broker, upstream.port, "/echo", &dummy).await;
    assert_eq!(
        response.try_into_string().await.expect("body"),
        format!("Bearer {SYNTHETIC_TOKEN}")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s02_a_spoofed_bootstrap_path_is_never_trusted() {
    // A "broker" that prints a control socket owned by someone else: the
    // controller refuses it because the socket's peer is not its child.
    let squatter_dir = tempfile::Builder::new()
        .prefix("cbk-")
        .tempdir_in("/tmp")
        .expect("squatter dir");
    let control = squatter_dir.path().join("c.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&control).expect("squatter socket");
    let line = format!(
        "{{\"protocol_version\":2,\"control_socket\":\"{}\"}}",
        control.display()
    );
    let broker = isolated_broker(IsolatedBrokerLauncher::test_harness(
        Some(PathBuf::from("/bin/sh")),
        vec![
            OsString::from("-c"),
            OsString::from(format!("echo '{line}'; sleep 5")),
        ],
        Vec::new(),
        /*controller_pid_override*/ None,
    ));
    let dummy = virtualized_dummy(&broker);
    assert!(broker.current_isolated_client().is_none());
    assert_eq!(
        route(&broker, /*port*/ 443, "/echo", &dummy).err(),
        Some(ScopedCredentialInjectionError::IsolatedBrokerUnavailable)
    );
}
