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
use super::run_credential_broker_main_with;
use crate::connect_policy::PinnedPeers;
use crate::credential_broker::CredentialBroker;
use crate::credential_broker::CredentialRouting;
use crate::credential_broker::ScopedCredentialInjectionError;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::ProviderRequestOperation;
use pretty_assertions::assert_eq;
use rama_core::Layer as _;
use rama_core::bytes::Bytes;
use rama_core::extensions::ExtensionsMut as _;
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
        run_credential_broker_main_with(Some(test_stored_key));
    }
}

/// PF-27-S05 stand-in for the vault resolver the binary supplies: reads
/// `<home>/pf27-store/<id>`; an `<id>.fail` file reports the store as
/// unavailable.
fn test_stored_key(home: &std::path::Path, id: &str) -> std::io::Result<Option<String>> {
    // PF-27-S09: `VAULT_<LABEL>` reads the real encrypted vault, as the
    // binary's resolver does.
    #[cfg(windows)]
    if let Some(label) = id.strip_prefix("VAULT_") {
        return codex_vault::Vault::new(home.to_path_buf())
            .reveal(&label.to_ascii_lowercase())
            .map(Some)
            .map_err(|error| std::io::Error::other(error.to_string()));
    }
    let dir = home.join("pf27-store");
    if dir.join(format!("{id}.fail")).exists() {
        return Err(std::io::Error::other("store unavailable"));
    }
    match std::fs::read_to_string(dir.join(id)) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Whether the broker's data endpoint still exists: its socket file, or on
/// Windows its named pipe (probed without connecting).
fn endpoint_exists(path: &std::path::Path) -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Pipes::WaitNamedPipeW;
        const ERROR_FILE_NOT_FOUND: i32 = 2;
        let wide: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: `wide` is NUL-terminated; waits at most 1 ms.
        let ready = unsafe { WaitNamedPipeW(wide.as_ptr(), 1) } != 0;
        ready || std::io::Error::last_os_error().raw_os_error() != Some(ERROR_FILE_NOT_FOUND)
    }
    #[cfg(unix)]
    {
        path.exists()
    }
}

/// PF-27-S05: the key the upstream's `/check` path expects.
static CHECK_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();

struct Upstream {
    port: u16,
    _ca_dir: TempDir,
    ca_path: PathBuf,
}

/// Local TLS upstream that echoes the Authorization header it received.
async fn start_upstream() -> Upstream {
    start_upstream_for(UPSTREAM_HOST).await
}

/// Like [`start_upstream`], with a certificate for `tls_host`.
async fn start_upstream_for(tls_host: &str) -> Upstream {
    let (ca_pem, acceptor) =
        crate::certs::test_ca_and_host_acceptor(tls_host).expect("test TLS material");
    let ca_dir = tempfile::tempdir().expect("ca dir");
    let ca_path = ca_dir.path().join("ca.pem");
    std::fs::write(&ca_path, ca_pem).expect("write test CA");
    let listener = TcpListener::bind_address("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let port = listener.local_addr().expect("upstream addr").port();
    let service = TlsAcceptorLayer::new(acceptor).into_layer(HttpServer::http1().service(
        service_fn(|request: Request| async move {
            let echoed = if request.uri().path().ends_with("/xkey") {
                rama_http::header::HeaderName::from_static("x-api-key")
            } else {
                AUTHORIZATION
            };
            let authorization = request
                .headers()
                .get(echoed)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("<none>")
                .to_string();
            if request.uri().path().ends_with("/check") {
                // PF-27-S05: confirm the key arrived without echoing it.
                let expected = CHECK_KEY.get().map(|key| format!("Bearer {key}"));
                let verdict = if expected.as_deref() == Some(authorization.as_str()) {
                    "ok"
                } else {
                    "mismatch"
                };
                return Ok::<_, Infallible>(Response::new(Body::from(verdict)));
            }
            if request.uri().path().ends_with("/redirect") {
                let mut response = Response::new(Body::empty());
                *response.status_mut() = rama_http::StatusCode::FOUND;
                response.headers_mut().insert(
                    rama_http::header::LOCATION,
                    HeaderValue::from_static("https://elsewhere.invalid/v1/steal"),
                );
                return Ok::<_, Infallible>(response);
            }
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
    launcher_with_ca(upstream.ca_path.clone(), controller_pid_override)
}

fn launcher_with_ca(
    ca_path: PathBuf,
    controller_pid_override: Option<u32>,
) -> IsolatedBrokerLauncher {
    IsolatedBrokerLauncher::test_harness(
        /*program*/ None,
        [CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"]
            .into_iter()
            .map(OsString::from)
            .collect(),
        vec![
            (OsString::from(CHILD_ENV), OsString::from("1")),
            (OsString::from("SSL_CERT_FILE"), ca_path.into_os_string()),
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
        scrub_responses: false,
        pin_connections: false,
    }
}

fn isolated_broker(launcher: IsolatedBrokerLauncher) -> CredentialBroker {
    CredentialBroker::new_isolated_with_launcher(/*enabled*/ true, options(), launcher)
}

fn virtualized_dummy(broker: &CredentialBroker) -> String {
    virtualized_dummy_for(broker, UPSTREAM_HOST)
}

fn virtualized_dummy_for(broker: &CredentialBroker, host: &str) -> String {
    let mut env = HashMap::from([
        ("GH_HOST".to_string(), host.to_string()),
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

const CONTROLLER_ENV: &str = "CODEX_PF27S06_CONTROLLER_CHILD";
#[cfg(windows)]
const CONTROLLER_TEST: &str =
    "credential_broker::isolated::tests::pf_27_s06_controller_child_entry";

/// Stands in for Core: starts a broker, prints its process id, then waits to
/// be killed.
#[test]
fn pf_27_s06_controller_child_entry() {
    if std::env::var_os(CONTROLLER_ENV).is_none() {
        return;
    }
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    runtime.block_on(async {
        let upstream = start_upstream().await;
        let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
        let _ = virtualized_dummy(&broker);
        let pid = broker
            .current_isolated_client()
            .and_then(|client| client.pid_for_test())
            .expect("broker pid");
        use std::io::Write as _;
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "\npf27s06-broker-pid:{pid}");
        let _ = stdout.flush();
        std::future::pending::<()>().await;
    });
}

/// PF-27-S06: a broker whose controller is killed outright (no graceful
/// close) exits by itself instead of lingering with credentials.
#[cfg(windows)]
#[test]
fn pf_27_s06_broker_exits_when_its_controller_is_killed() {
    use std::io::BufRead as _;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::OpenProcess;
    use windows_sys::Win32::System::Threading::PROCESS_SYNCHRONIZE;
    use windows_sys::Win32::System::Threading::WaitForSingleObject;
    let mut controller = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            CONTROLLER_TEST,
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CONTROLLER_ENV, "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("controller");
    let stdout = controller.stdout.take().expect("controller stdout");
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
        {
            if let Some(pid) = line
                .split("pf27s06-broker-pid:")
                .nth(1)
                .and_then(|pid| pid.trim().parse::<u32>().ok())
            {
                let _ = sender.send(pid);
                break;
            }
        }
    });
    let broker_pid = receiver
        .recv_timeout(Duration::from_secs(60))
        .expect("controller reported its broker");
    // SAFETY: wait-only access; closed below.
    let broker = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, broker_pid) };
    assert_ne!(broker, 0, "the broker is running");
    controller.kill().expect("kill controller");
    let _ = controller.wait();
    // SAFETY: `broker` is open.
    let waited = unsafe { WaitForSingleObject(broker, 10_000) };
    // SAFETY: opened above.
    unsafe { CloseHandle(broker) };
    assert_eq!(
        waited, /*WAIT_OBJECT_0*/ 0,
        "broker outlived its controller"
    );
}

/// Disables `SeDebugPrivilege` in this process's token, if it is enabled.
#[cfg(windows)]
fn disable_debug_privilege() {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::LUID;
    use windows_sys::Win32::Security::AdjustTokenPrivileges;
    use windows_sys::Win32::Security::LUID_AND_ATTRIBUTES;
    use windows_sys::Win32::Security::LookupPrivilegeValueW;
    use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
    use windows_sys::Win32::Security::TOKEN_PRIVILEGES;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    let name: Vec<u16> = "SeDebugPrivilege"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut luid = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let mut token = 0;
    // SAFETY: valid pointers; the token is closed below.
    unsafe {
        if LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) == 0
            || OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token) == 0
        {
            return;
        }
        let disabled = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: 0,
            }],
        };
        AdjustTokenPrivileges(
            token,
            0,
            &disabled,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        CloseHandle(token);
    }
}

/// PF-27-S06: on Windows the broker runs over its named pipes, reports both
/// containment layers, and no other process of the user can open it to read
/// its memory or environment (opening this test process is the control).
#[cfg(windows)]
#[tokio::test(flavor = "multi_thread")]
async fn pf_27_s06_windows_broker_uses_pipes_and_cannot_be_read() {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::Threading::OpenProcess;
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION;
    use windows_sys::Win32::System::Threading::PROCESS_VM_READ;
    let open_for_reading = |pid: u32| -> Result<(), u32> {
        // SAFETY: plain OpenProcess; the handle is closed at once.
        unsafe {
            let handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, 0, pid);
            if handle == 0 {
                Err(GetLastError())
            } else {
                CloseHandle(handle);
                Ok(())
            }
        }
    };

    let upstream = start_upstream().await;
    let broker = isolated_broker(launcher(&upstream, /*controller_pid_override*/ None));
    let dummy = virtualized_dummy(&broker);
    let client = broker.current_isolated_client().expect("broker client");
    assert_eq!(client.containment(), "token+dacl+job");
    let pipe = client.socket_path().to_string_lossy().into_owned();
    assert!(
        super::pipe::valid_pipe_name(&pipe, /*control*/ false),
        "{pipe}"
    );

    let response = forward(&broker, upstream.port, "/echo", &dummy).await;
    assert_eq!(
        response.try_into_string().await.expect("body"),
        format!("Bearer {SYNTHETIC_TOKEN}")
    );

    assert_eq!(open_for_reading(std::process::id()), Ok(()));
    // CI runners run elevated with SeDebugPrivilege enabled, which opens any
    // process whatever its DACL; an ordinary process of the user has it off.
    disable_debug_privilege();
    let broker_pid = client.pid_for_test().expect("broker pid");
    assert_eq!(open_for_reading(broker_pid), Err(/*ERROR_ACCESS_DENIED*/ 5));
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
        Ok(CredentialRouting::Direct(None))
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
    assert!(!endpoint_exists(old_client.socket_path()));

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
        Ok(CredentialRouting::Direct(None))
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
    assert!(endpoint_exists(&socket_path));

    drop(broker);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while endpoint_exists(&socket_path) && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!endpoint_exists(&socket_path));
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

#[cfg(unix)]
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

#[cfg(unix)]
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

#[cfg(unix)]
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

#[tokio::test(flavor = "multi_thread")]
async fn pf_28_s02_broker_scrubs_its_credential_from_the_response() {
    let upstream = start_upstream().await;
    let broker = CredentialBroker::new_isolated_with_launcher(
        /*enabled*/ true,
        IsolatedBrokerOptions {
            scrub_responses: true,
            ..options()
        },
        launcher(&upstream, /*controller_pid_override*/ None),
    );
    let dummy = virtualized_dummy(&broker);

    // The upstream echoes the Authorization header it received.
    let response = forward(&broker, upstream.port, "/echo", &dummy).await;

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.try_into_string().await.expect("body"),
        "Bearer [REDACTED:broker:credential]"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_28_s02_revocation_still_closes_a_scrubbed_stream() {
    let upstream = start_upstream().await;
    let broker = CredentialBroker::new_isolated_with_launcher(
        /*enabled*/ true,
        IsolatedBrokerOptions {
            scrub_responses: true,
            ..options()
        },
        launcher(&upstream, /*controller_pid_override*/ None),
    );
    let dummy = virtualized_dummy(&broker);

    let response = forward(&broker, upstream.port, STREAM_PATH, &dummy).await;
    assert_eq!(response.status(), 200);
    let reader = tokio::spawn(async move { response.try_into_string().await.is_err() });
    tokio::time::sleep(Duration::from_millis(100)).await;

    // The scrubber's held bytes go with the stream; revocation still ends it.
    assert!(broker.revoke_isolated_credentials());
    let stream_failed = tokio::time::timeout(Duration::from_secs(10), reader)
        .await
        .expect("revocation closes the scrubbed download")
        .expect("reader task");
    assert!(stream_failed);
}

/// PF-33-S02 fixture name that no resolver answers, so reaching the upstream
/// proves the broker dialled the pinned answer instead of resolving.
const PINNED_NAME: &str = "pinned.invalid";

fn pinned_broker(upstream: &Upstream, options: IsolatedBrokerOptions) -> CredentialBroker {
    CredentialBroker::new_isolated_with_launcher(
        /*enabled*/ true,
        options,
        launcher(upstream, /*controller_pid_override*/ None),
    )
}

fn pinned_route(
    broker: &CredentialBroker,
    port: u16,
    dummy: &str,
) -> super::super::BrokeredCredentialRoute {
    match broker.route_request_credentials(
        "https",
        PINNED_NAME,
        port,
        "GET",
        "/echo",
        &mut bearer(dummy),
    ) {
        Ok(CredentialRouting::Brokered(route)) => route,
        _ => panic!("expected a brokered route"),
    }
}

fn pinned_request(dummy: &str, pin: Option<PinnedPeers>) -> Request {
    let mut request = request("/echo", dummy);
    if let Some(pin) = pin {
        request.extensions_mut().insert(pin);
    }
    request
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_33_s02_brokered_request_dials_the_pinned_answer_without_dns() {
    let upstream = start_upstream_for(PINNED_NAME).await;
    let broker = pinned_broker(
        &upstream,
        IsolatedBrokerOptions {
            pin_connections: true,
            ..options()
        },
    );
    let dummy = virtualized_dummy_for(&broker, PINNED_NAME);
    let loopback = std::net::IpAddr::from([127, 0, 0, 1]);

    let pin = PinnedPeers::new(PINNED_NAME, upstream.port, [loopback]);
    let response = pinned_route(&broker, upstream.port, &dummy)
        .forward(pinned_request(&dummy, Some(pin)))
        .await
        .expect("pinned broker response");
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.try_into_string().await.expect("body"),
        format!("Bearer {SYNTHETIC_TOKEN}")
    );

    // No checked answers, or answers for another authority: Core refuses
    // before the broker is asked.
    let wrong_port = PinnedPeers::new(PINNED_NAME, upstream.port + 1, [loopback]);
    let wrong_host = PinnedPeers::new("other.invalid", upstream.port, [loopback]);
    for pin in [None, Some(wrong_port), Some(wrong_host)] {
        assert_eq!(
            pinned_route(&broker, upstream.port, &dummy)
                .forward(pinned_request(&dummy, pin))
                .await
                .err(),
            Some(IsolatedBrokerError::Unpinned)
        );
    }

    // The broker enforces it too: a signed but unpinned frame is refused.
    let route = pinned_route(&broker, upstream.port, &dummy);
    let client = broker.current_isolated_client().expect("broker client");
    let frame = client
        .sign_frame(&route.reference, &route.operation)
        .expect("frame");
    let response = client
        .send_with_frame(&route.operation, request("/echo", &dummy), &frame)
        .await
        .expect("response");
    assert_eq!(denial(&response), Some("unpinned"));
}

#[tokio::test(flavor = "multi_thread")]
async fn pf_33_s02_unpinned_broker_resolves_and_pins_keep_the_private_peer_check() {
    let upstream = start_upstream_for(PINNED_NAME).await;
    let loopback = std::net::IpAddr::from([127, 0, 0, 1]);

    // Control (guard off): the broker resolves the name itself, which fails.
    let broker = pinned_broker(&upstream, options());
    let dummy = virtualized_dummy_for(&broker, PINNED_NAME);
    let response = pinned_route(&broker, upstream.port, &dummy)
        .forward(pinned_request(&dummy, None))
        .await
        .expect("broker response");
    assert_eq!(denial(&response), Some("upstream_failed"));

    // A pin cannot grant a private peer the broker's policy does not allow.
    let broker = pinned_broker(
        &upstream,
        IsolatedBrokerOptions {
            allow_local_binding: false,
            pin_connections: true,
            ..options()
        },
    );
    let dummy = virtualized_dummy_for(&broker, PINNED_NAME);
    let pin = PinnedPeers::new(PINNED_NAME, upstream.port, [loopback]);
    let response = pinned_route(&broker, upstream.port, &dummy)
        .forward(pinned_request(&dummy, Some(pin)))
        .await
        .expect("broker response");
    assert_eq!(denial(&response), Some("upstream_failed"));
}

// #390: pipe squats of the Windows broker's named pipes.
#[cfg(windows)]
#[path = "pipe_squat_tests.rs"]
mod sec_390;

// PF-27-S05 model auth; on Windows since PF-27-S09 (named pipes).
#[cfg(any(unix, windows))]
mod pf_27_s05 {
    use super::Body;
    use super::CHECK_KEY;
    use super::CHILD_ENV;
    use super::Duration;
    use super::PathBuf;
    use super::Request;
    use super::Response;
    use super::UPSTREAM_HOST;
    use super::Upstream;
    use super::denial;
    use super::launcher;
    use super::launcher_with_ca;
    use super::start_upstream;
    use crate::credential_broker::memory_scan_tests;
    use crate::credential_broker::model_auth::MODEL_BROKER_FRAME_HEADER;
    use crate::credential_broker::model_auth::ModelAuthHeader;
    use crate::credential_broker::model_auth::ModelCredential;
    use crate::credential_broker::model_auth::ModelCredentialBinding;
    use crate::credential_broker::model_auth::ModelCredentialBroker;
    use crate::credential_broker::model_auth::ModelCredentialBrokerError;
    use crate::credential_broker::model_auth::ModelCredentialBrokerOptions;
    #[cfg(unix)]
    use crate::upstream::UpstreamClient;
    use pretty_assertions::assert_eq;
    use rama_core::Service as _;
    use rama_http::BodyExtractExt as _;

    const MODEL_KEY: &str = "sk-pf27s05SyntheticModelKey000000000000000000";

    fn model_broker(upstream: &Upstream) -> ModelCredentialBroker {
        ModelCredentialBroker::spawn_with_launcher(
            ModelCredentialBrokerOptions::default(),
            launcher(upstream, /*controller_pid_override*/ None),
        )
        .expect("model broker")
    }

    fn binding(port: u16, path_prefix: &str, header: ModelAuthHeader) -> ModelCredentialBinding {
        ModelCredentialBinding {
            host: UPSTREAM_HOST.to_string(),
            port,
            path_prefix: path_prefix.to_string(),
            header,
        }
    }

    async fn send(
        credential: &ModelCredential,
        method: &str,
        path: &str,
        frame: String,
    ) -> Response {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(rama_http::header::HOST, UPSTREAM_HOST)
            .header(MODEL_BROKER_FRAME_HEADER, frame)
            .body(Body::empty())
            .expect("request");
        #[cfg(unix)]
        let broker = UpstreamClient::unix_socket(&credential.socket_path().to_string_lossy());
        // PF-27-S09: the data pipe, checked to be served by the broker.
        #[cfg(windows)]
        let broker = credential.data_pipe_client_for_test();
        broker.serve(request).await.expect("broker response")
    }

    async fn signed(credential: &ModelCredential, port: u16, path: &str) -> Response {
        let frame = credential
            .sign("POST", UPSTREAM_HOST, port, path)
            .expect("signed frame");
        send(credential, "POST", path, frame).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s05_model_key_is_attached_only_inside_the_broker() {
        let upstream = start_upstream().await;
        let broker = model_broker(&upstream);
        let bearer = broker
            .register(
                binding(upstream.port, "/v1", ModelAuthHeader::Bearer),
                MODEL_KEY,
            )
            .expect("register bearer key");
        let x_api_key = broker
            .register(
                binding(upstream.port, "/v1", ModelAuthHeader::XApiKey),
                MODEL_KEY,
            )
            .expect("register x-api-key key");
        // Core's handles carry no copy of the key.
        assert!(!format!("{bearer:?}{x_api_key:?}{broker:?}").contains(MODEL_KEY));

        let response = signed(&bearer, upstream.port, "/v1/responses?stream=true").await;
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.try_into_string().await.expect("body"),
            format!("Bearer {MODEL_KEY}")
        );
        let response = signed(&x_api_key, upstream.port, "/v1/messages/xkey").await;
        assert_eq!(response.status(), 200);
        assert_eq!(response.try_into_string().await.expect("body"), MODEL_KEY);

        // Each frame is single-use.
        let frame = bearer
            .sign("POST", UPSTREAM_HOST, upstream.port, "/v1/responses")
            .expect("frame");
        let first = send(&bearer, "POST", "/v1/responses", frame.clone()).await;
        assert_eq!(first.status(), 200);
        let replay = send(&bearer, "POST", "/v1/responses", frame).await;
        assert_eq!(denial(&replay), Some("replay"));

        // A redirect comes back to Core unfollowed; the key goes nowhere else.
        let redirect = signed(&bearer, upstream.port, "/v1/redirect").await;
        assert_eq!(redirect.status(), 302);
        assert_eq!(
            redirect
                .headers()
                .get(rama_http::header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("https://elsewhere.invalid/v1/steal")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s05_model_key_is_bound_to_its_origin_and_path_prefix() {
        let upstream = start_upstream().await;
        let broker = model_broker(&upstream);
        let credential = broker
            .register(
                binding(upstream.port, "/v1", ModelAuthHeader::Bearer),
                MODEL_KEY,
            )
            .expect("register");

        // Core refuses to sign outside the binding ...
        for (host, port, path) in [
            ("api.example.com", upstream.port, "/v1/responses"),
            (UPSTREAM_HOST, upstream.port + 1, "/v1/responses"),
            (UPSTREAM_HOST, upstream.port, "/v2/responses"),
            (UPSTREAM_HOST, upstream.port, "/v1x/responses"),
            (UPSTREAM_HOST, upstream.port, "/v1/../admin"),
            (UPSTREAM_HOST, upstream.port, "/v1/%2e%2e/admin"),
        ] {
            assert_eq!(
                credential.sign("POST", host, port, path).err(),
                Some(ModelCredentialBrokerError::Rejected),
                "{host}:{port}{path}"
            );
        }
        // ... and the broker refuses such frames itself.
        for path in ["/v2/responses", "/v1x/responses", "/v1/../admin"] {
            let frame =
                credential.sign_unchecked_for_test("POST", UPSTREAM_HOST, upstream.port, path);
            let response = send(&credential, "POST", path, frame).await;
            assert_eq!(denial(&response), Some("host_not_bound"), "{path}");
        }
        let frame =
            credential.sign_unchecked_for_test("POST", UPSTREAM_HOST, upstream.port + 1, "/v1/x");
        let response = send(&credential, "POST", "/v1/x", frame).await;
        assert_eq!(denial(&response), Some("host_not_bound"));

        // Malformed bindings are not admitted.
        for (prefix, host) in [
            ("v1", UPSTREAM_HOST),
            ("/v1/", UPSTREAM_HOST),
            ("/v1/../x", UPSTREAM_HOST),
            ("/v1", "Bad.Host"),
        ] {
            let binding = ModelCredentialBinding {
                host: host.to_string(),
                ..binding(upstream.port, prefix, ModelAuthHeader::Bearer)
            };
            assert_eq!(
                broker.register(binding, MODEL_KEY).err(),
                Some(ModelCredentialBrokerError::Rejected),
                "{host} {prefix}"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s05_model_broker_death_fails_closed() {
        let upstream = start_upstream().await;
        let broker = model_broker(&upstream);
        let credential = broker
            .register(
                binding(upstream.port, "/", ModelAuthHeader::Bearer),
                MODEL_KEY,
            )
            .expect("register");
        assert_eq!(
            signed(&credential, upstream.port, "/v1/responses")
                .await
                .status(),
            200
        );

        broker.kill_for_test();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while credential.is_alive() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(!credential.is_alive());
        assert_eq!(
            credential
                .sign("POST", UPSTREAM_HOST, upstream.port, "/v1/responses")
                .err(),
            Some(ModelCredentialBrokerError::Unavailable)
        );
        assert_eq!(
            broker
                .register(
                    binding(upstream.port, "/", ModelAuthHeader::Bearer),
                    MODEL_KEY
                )
                .err(),
            Some(ModelCredentialBrokerError::Unavailable)
        );
    }

    fn store_broker(upstream: &Upstream, home: &std::path::Path) -> ModelCredentialBroker {
        ModelCredentialBroker::spawn_with_launcher(
            ModelCredentialBrokerOptions {
                store_home: Some(home.to_path_buf()),
                ..ModelCredentialBrokerOptions::default()
            },
            launcher(upstream, /*controller_pid_override*/ None),
        )
        .expect("model broker")
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s05_stored_key_is_read_inside_the_broker() {
        let upstream = start_upstream().await;
        let home = tempfile::tempdir().expect("home");
        let store = home.path().join("pf27-store");
        std::fs::create_dir(&store).expect("store");
        std::fs::write(store.join("ZAI_API_KEY"), MODEL_KEY).expect("stored key");
        std::fs::write(store.join("BROKEN_API_KEY.fail"), "").expect("broken store");
        let broker = store_broker(&upstream, home.path());
        let bound = binding(upstream.port, "/v1", ModelAuthHeader::Bearer);

        let credential = broker
            .register_stored(bound.clone(), "ZAI_API_KEY", &[])
            .expect("register")
            .expect("stored key");
        let response = signed(&credential, upstream.port, "/v1/responses").await;
        assert_eq!(
            response.try_into_string().await.expect("body"),
            format!("Bearer {MODEL_KEY}")
        );

        // Nothing stored, an unreadable store, a malformed id.
        assert!(
            broker
                .register_stored(bound.clone(), "MISSING_API_KEY", &[])
                .expect("register")
                .is_none()
        );
        assert_eq!(
            broker
                .register_stored(bound.clone(), "BROKEN_API_KEY", &[])
                .err(),
            Some(ModelCredentialBrokerError::StoreUnavailable)
        );
        assert_eq!(
            broker
                .register_stored(bound.clone(), "../ZAI_API_KEY", &[])
                .err(),
            Some(ModelCredentialBrokerError::Rejected)
        );
        // A broker started without a store home reads no stored keys.
        assert!(
            model_broker(&upstream)
                .register_stored(bound, "ZAI_API_KEY", &[])
                .expect("register")
                .is_none()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s05_env_keys_are_handed_over_removed_and_unregistered() {
        let upstream = start_upstream().await;
        let name = format!("PF27_S05_ENV_KEY_{}", std::process::id());
        // A value no other test holds, kept here only masked: on Windows the
        // hand-over overwrites every other copy of it in this process
        // (PF-27-S09), this test's included.
        let key = format!("sk-pf27s05-env-{:016x}", rand::random::<u64>());
        crate::credential_broker::env_scrub::set_env_var_for_test(&name, &key);
        let masked = memory_scan_tests::mask(key.as_bytes());
        drop(key);
        let broker = model_broker(&upstream);
        let taken = broker
            .take_env_keys(&[name.clone(), "PF27_S05_NEVER_SET_KEY".to_string()])
            .expect("take env keys");
        assert_eq!(taken, vec![name.clone()]);
        assert_eq!(std::env::var_os(&name), None);

        let credential = broker
            .register_stored(
                binding(upstream.port, "/v1", ModelAuthHeader::Bearer),
                "PF27_S05_NEVER_SET_KEY",
                &taken,
            )
            .expect("register")
            .expect("stashed key");
        let response = signed(&credential, upstream.port, "/v1/responses").await;
        let body = response.try_into_string().await.expect("body");
        let unmasked = body
            .strip_prefix("Bearer ")
            .map(|key| memory_scan_tests::mask(key.as_bytes()));
        assert_eq!(
            unmasked,
            Some(masked),
            "the broker attached the handed-over key"
        );

        // A replaced sign-in token drops the broker's copy.
        credential.unregister().expect("unregister");
        let response = signed(&credential, upstream.port, "/v1/responses").await;
        assert_eq!(denial(&response), Some("unknown_credential"));
    }

    /// The vault lock is created before containment (so a vault first
    /// created later in the session is still usable) and is never reached
    /// through a symlink.
    #[cfg(unix)]
    #[test]
    fn pf_27_s05_vault_lock_is_created_and_never_a_symlink() {
        use crate::credential_broker::isolated::server::prepare_vault_lock;
        let home = tempfile::tempdir().expect("home");
        let lock = prepare_vault_lock(home.path()).expect("lock created");
        assert_eq!(lock, home.path().join("secrets").join(".vault.lock"));
        assert!(lock.is_file());

        let elsewhere = tempfile::tempdir().expect("elsewhere");
        let target = elsewhere.path().join("target");
        std::fs::write(&target, b"").expect("target");
        let linked = tempfile::tempdir().expect("linked home");
        std::fs::create_dir(linked.path().join("secrets")).expect("secrets");
        std::os::unix::fs::symlink(&target, linked.path().join("secrets").join(".vault.lock"))
            .expect("symlink");
        assert_eq!(prepare_vault_lock(linked.path()), None);

        let linked_dir = tempfile::tempdir().expect("linked dir home");
        std::os::unix::fs::symlink(elsewhere.path(), linked_dir.path().join("secrets"))
            .expect("dir symlink");
        assert_eq!(prepare_vault_lock(linked_dir.path()), None);
    }

    const MEMORY_CHILD_ENV: &str = "CODEX_PF27_S05_MEMORY_CHILD";
    const MEMORY_KEY_ENV: &str = "PF27_S05_MEMORY_KEY";
    const MEMORY_MASKED_ENV: &str = "PF27_S05_MEMORY_MASKED";
    const MEMORY_CHILD_TEST: &str =
        "credential_broker::isolated::tests::pf_27_s05::pf_27_s05_core_memory_child_entry";

    fn decode_hex(value: &str) -> Vec<u8> {
        (0..value.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&value[index..index + 2], 16).expect("hex"))
            .collect()
    }

    /// Plays Core in a fresh process whose launch environment holds the key,
    /// as Core's does: hands the key to the broker, uses it, then scans its
    /// own writable memory (heap, stacks, launch environment) for it.
    ///
    /// Scope: the broker hand-over path (`take_env_keys`, `register_stored`,
    /// signed requests). It does not run Core's config loading, `.env`
    /// handling or telemetry, which could copy the key before hand-over.
    #[test]
    #[expect(clippy::print_stdout, reason = "the parent test reads the hit counts")]
    fn pf_27_s05_core_memory_child_entry() {
        let Some(port) = std::env::var_os(MEMORY_CHILD_ENV) else {
            return;
        };
        let port: u16 = port
            .to_str()
            .and_then(|port| port.parse().ok())
            .expect("port");
        let masked = decode_hex(&std::env::var(MEMORY_MASKED_ENV).expect("masked key"));
        let ca_path = PathBuf::from(std::env::var_os("SSL_CERT_FILE").expect("ca"));
        // PF-27-S09: Windows keeps the environment block in UTF-16 and the C
        // runtime's copy in the ANSI code page; both forms are scanned.
        #[cfg(windows)]
        let wide_masked: Vec<u8> = masked
            .iter()
            .flat_map(|byte| [*byte, memory_scan_tests::MASK])
            .collect();
        let count = |masked: &[u8]| {
            let hits = memory_scan_tests::count_in_writable_memory(masked);
            #[cfg(windows)]
            let hits = hits + memory_scan_tests::count_in_writable_memory(&wide_masked);
            hits
        };
        // Core grows its environment before the hand-over (arg0 prepends to
        // PATH, `.env` adds variables), which can move the environment block.
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths: Vec<PathBuf> = (0..256)
            .map(|index| std::env::temp_dir().join(format!("pf27s09-grow-{index}")))
            .collect();
        paths.extend(std::env::split_paths(&path));
        let grown = std::env::join_paths(paths).expect("PATH");
        crate::credential_broker::env_scrub::set_env_var_for_test("PATH", &grown.to_string_lossy());
        crate::credential_broker::env_scrub::set_env_var_for_test(
            "PF27_S09_GROWN",
            // Within Windows' 32,767-character limit for a value.
            &"x".repeat(30_000),
        );
        // Positive control: the launch environment still holds the key.
        let before = count(&masked);
        println!("PF27S05 memory hits_before={before}");
        assert!(before > 0, "the scanner must find the key before hand-over");
        #[cfg(windows)]
        assert!(
            memory_scan_tests::count_in_writable_memory(&wide_masked) > 0,
            "positive control: the UTF-16 environment block"
        );
        #[cfg(windows)]
        let environ_holds_key = || windows_environment_holds(&masked);
        #[cfg(windows)]
        assert!(
            environ_holds_key(),
            "positive control: the environment block"
        );
        #[cfg(target_os = "linux")]
        let environ_holds_key = || {
            // Compared masked and the copy wiped, so this check leaves no
            // plain key behind for the memory scan.
            let environ =
                zeroize::Zeroizing::new(std::fs::read("/proc/self/environ").unwrap_or_default());
            environ.windows(masked.len()).any(|window| {
                window
                    .iter()
                    .zip(&masked)
                    .all(|(byte, masked)| byte ^ memory_scan_tests::MASK == *masked)
            })
        };
        #[cfg(target_os = "linux")]
        assert!(environ_holds_key(), "positive control: /proc/self/environ");

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");
        // Borrowed: the Linux environment check below still needs `masked`.
        let scan_masked = masked.as_slice();
        let hits = runtime.block_on(async move {
            let broker = ModelCredentialBroker::spawn_with_launcher(
                ModelCredentialBrokerOptions {
                    withheld_env: vec![MEMORY_KEY_ENV.to_string()],
                    ..ModelCredentialBrokerOptions::default()
                },
                launcher_with_ca(ca_path, /*controller_pid_override*/ None),
            )
            .expect("model broker");
            let taken = broker
                .take_env_keys(&[MEMORY_KEY_ENV.to_string()])
                .expect("take env key");
            let credential = broker
                .register_stored(
                    binding(port, "/v1", ModelAuthHeader::Bearer),
                    MEMORY_KEY_ENV,
                    &taken,
                )
                .expect("register")
                .expect("stashed key");
            for _ in 0..3 {
                let response = signed(&credential, port, "/v1/check").await;
                assert_eq!(response.try_into_string().await.expect("body"), "ok");
            }
            // Scan while Core still holds its live handles.
            let hits = count(scan_masked);
            drop((credential, broker));
            hits
        });
        // What another same-user process reads as Core's environment.
        #[cfg(any(target_os = "linux", windows))]
        assert!(
            !environ_holds_key(),
            "the environment block still holds the key"
        );
        println!("PF27S05 memory hits_after={hits}");
        #[cfg(windows)]
        let places: Vec<String> = if hits > 0 {
            [("ascii", &masked), ("utf-16", &wide_masked)]
                .into_iter()
                .flat_map(|(form, needle)| {
                    memory_scan_tests::locate_in_writable_memory(needle)
                        .into_iter()
                        .map(move |place| format!("({form}) {place}"))
                })
                .collect()
        } else {
            Vec::new()
        };
        #[cfg(not(windows))]
        let places: Vec<String> = Vec::new();
        #[cfg(windows)]
        let parameters = format!(
            "; process parameters at {:#x?}",
            crate::credential_broker::env_scrub::process_parameters_allocation_for_test()
        );
        #[cfg(not(windows))]
        let parameters = String::new();
        assert_eq!(
            hits,
            0,
            "the raw key is still in Core's memory: {}{parameters}",
            places.join(" | ")
        );
    }

    /// PF-27-S09: whether this process's environment block (what another
    /// process reads as its environment) holds the masked key. Compared
    /// masked, and the private copy is wiped, so this check leaves no plain
    /// key behind for the memory scan.
    #[cfg(windows)]
    fn windows_environment_holds(masked: &[u8]) -> bool {
        use windows_sys::Win32::System::Environment::FreeEnvironmentStringsW;
        use windows_sys::Win32::System::Environment::GetEnvironmentStringsW;
        // SAFETY: a private copy of the block, wiped and freed below.
        let block = unsafe { GetEnvironmentStringsW() };
        assert!(!block.is_null(), "environment block");
        let mut len = 0;
        // SAFETY: the block ends with two NUL units.
        unsafe {
            while *block.add(len) != 0 || *block.add(len + 1) != 0 {
                len += 1;
            }
        }
        // SAFETY: `len` units (plus the terminator) belong to the block.
        let units = unsafe { std::slice::from_raw_parts_mut(block, len) };
        let found = units.windows(masked.len()).any(|window| {
            window
                .iter()
                .zip(masked)
                .all(|(unit, masked)| *unit == u16::from(masked ^ memory_scan_tests::MASK))
        });
        for unit in units.iter_mut() {
            // SAFETY: a valid element of the block.
            unsafe { std::ptr::write_volatile(unit, 0) };
        }
        // SAFETY: from GetEnvironmentStringsW above.
        unsafe { FreeEnvironmentStringsW(block) };
        found
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s05_raw_key_is_not_left_in_core_memory() {
        let key = format!(
            "sk-pf27mem-{:016x}{:016x}",
            rand::random::<u64>(),
            rand::random::<u64>()
        );
        CHECK_KEY.set(key.clone()).expect("check key set once");
        let upstream = start_upstream().await;
        let masked = super::super::protocol::encode_hex(&memory_scan_tests::mask(key.as_bytes()));
        let program = std::env::current_exe().expect("test binary");
        let port = upstream.port.to_string();
        let ca_path = upstream.ca_path.clone();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new(program)
                .args([
                    MEMORY_CHILD_TEST,
                    "--exact",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(MEMORY_CHILD_ENV, port)
                .env(MEMORY_KEY_ENV, key)
                .env(MEMORY_MASKED_ENV, masked)
                .env("SSL_CERT_FILE", ca_path)
                .env_remove(CHILD_ENV)
                .output()
        })
        .await
        .expect("join")
        .expect("run Core stand-in");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("PF27S05 memory hits_after=0"),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(stdout.contains("1 passed"), "{stdout}");
    }

    /// PF-27-S09: Core creates the vault lock for the Windows broker, never
    /// through a junction or symlink.
    #[cfg(windows)]
    #[test]
    #[expect(
        clippy::print_stdout,
        reason = "records a skipped privilege-dependent case"
    )]
    fn pf_27_s09_vault_lock_is_created_and_never_a_reparse_point() {
        use crate::credential_broker::isolated::prepare_vault_lock;
        let home = tempfile::tempdir().expect("home");
        let lock = prepare_vault_lock(home.path()).expect("lock created");
        assert_eq!(
            lock,
            std::fs::canonicalize(home.path())
                .expect("canonical home")
                .join("secrets")
                .join(".vault.lock")
        );
        assert!(lock.is_file());
        // Again: an existing lock is kept.
        assert_eq!(prepare_vault_lock(home.path()), Some(lock));

        // `secrets` as a junction (no privilege needed to create one).
        let elsewhere = tempfile::tempdir().expect("elsewhere");
        let junction_home = tempfile::tempdir().expect("junction home");
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(junction_home.path().join("secrets"))
            .arg(elsewhere.path())
            .stdout(std::process::Stdio::null())
            .status()
            .expect("mklink");
        assert!(status.success(), "junction");
        assert_eq!(prepare_vault_lock(junction_home.path()), None);
        assert!(!elsewhere.path().join(".vault.lock").exists());

        // The lock itself as a file symlink (needs a privilege a normal
        // session may not have).
        let target = elsewhere.path().join("target");
        std::fs::write(&target, b"").expect("target");
        let linked = tempfile::tempdir().expect("linked home");
        std::fs::create_dir(linked.path().join("secrets")).expect("secrets");
        match std::os::windows::fs::symlink_file(
            &target,
            linked.path().join("secrets").join(".vault.lock"),
        ) {
            Ok(()) => assert_eq!(prepare_vault_lock(linked.path()), None),
            Err(error) => println!("PF27S09 file symlink case skipped: {error}"),
        }
    }

    #[cfg(windows)]
    const LOCK_CHILD_ENV: &str = "CODEX_PF27_S09_LOCK_CHILD";

    /// Run under the broker token by the test below: opens the vault lock
    /// named in [`LOCK_CHILD_ENV`] for writing, then read-only and locks it.
    #[cfg(windows)]
    #[test]
    #[expect(clippy::print_stdout, reason = "the parent test reads the results")]
    fn pf_27_s09_lock_child_entry() {
        let Some(lock) = std::env::var_os(LOCK_CHILD_ENV) else {
            return;
        };
        let write = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&lock);
        println!(
            "pf27s09-lock write-open={}",
            match &write {
                Ok(_) => "ok".to_string(),
                Err(error) => format!("{:?}", error.kind()),
            }
        );
        let locked = std::fs::OpenOptions::new()
            .read(true)
            .open(&lock)
            .and_then(|file| file.lock().map(|()| file));
        println!(
            "pf27s09-lock read-only-lock={}",
            if locked.is_ok() { "ok" } else { "failed" }
        );
    }

    /// PF-27-S09: why the vault falls back to a read-only lock handle. Under
    /// the broker token the lock Core created cannot be opened for writing
    /// (access denied), but a read-only handle locks it exclusively. This
    /// process, which can open it for writing, is the control.
    #[cfg(windows)]
    #[test]
    fn pf_27_s09_broker_token_locks_the_vault_read_only() {
        use crate::credential_broker::isolated::prepare_vault_lock;
        use std::io::Read as _;
        let home = tempfile::tempdir().expect("home");
        let lock = prepare_vault_lock(home.path()).expect("lock");
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&lock)
            .expect("control: this process opens the lock for writing");
        let mut env: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
        env.push((LOCK_CHILD_ENV.into(), lock.into_os_string()));
        let args: Vec<std::ffi::OsString> = [
            "credential_broker::isolated::tests::pf_27_s05::pf_27_s09_lock_child_entry",
            "--exact",
            "--nocapture",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        let (mut child, mut stdout) = codex_process_hardening::spawn_protected(
            &std::env::current_exe().expect("test binary"),
            &args,
            &env,
        )
        .expect("start under the broker token");
        let mut output = String::new();
        stdout.read_to_string(&mut output).expect("child output");
        let _ = child.wait();
        assert!(
            output.contains("pf27s09-lock write-open=PermissionDenied"),
            "{output}"
        );
        assert!(
            output.contains("pf27s09-lock read-only-lock=ok"),
            "{output}"
        );
    }

    #[cfg(windows)]
    const VAULT_CHILD_ENV: &str = "CODEX_PF27_S09_VAULT_CHILD";
    #[cfg(windows)]
    const VAULT_CHILD_TEST: &str =
        "credential_broker::isolated::tests::pf_27_s05::pf_27_s09_vault_child_entry";

    /// The vault half of [`pf_27_s09_stored_key_is_read_from_the_vault_under_the_broker_token`],
    /// in a child whose debug keyring stays in the profile
    /// (`CORBANU_TEST_NO_NATIVE_KEYRING`), so no test touches Credential Manager.
    #[cfg(windows)]
    #[test]
    fn pf_27_s09_vault_child_entry() {
        if std::env::var_os(VAULT_CHILD_ENV).is_none() {
            return;
        }
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            let upstream = start_upstream().await;
            let home = tempfile::tempdir().expect("home");
            codex_vault::Vault::new(home.path().to_path_buf())
                .add(codex_vault::AddCredential {
                    label: "zai".to_string(),
                    credential_type: codex_vault::CredentialType::ApiKey,
                    provider: Some("zai".to_string()),
                    notes: None,
                    revocation_notes: None,
                    secret: MODEL_KEY.to_string(),
                })
                .expect("vault entry");
            // The vault made its lock; Core must make it for the broker.
            std::fs::remove_file(home.path().join("secrets").join(".vault.lock"))
                .expect("remove the vault's own lock");
            let broker = store_broker(&upstream, home.path());
            let bound = binding(upstream.port, "/v1", ModelAuthHeader::Bearer);
            // Core created the lock the broker token cannot create.
            assert!(home.path().join("secrets").join(".vault.lock").is_file());
            let credential = broker
                .register_stored(bound.clone(), "VAULT_ZAI", &[])
                .expect("the broker reads the vault")
                .expect("stored key");
            let response = signed(&credential, upstream.port, "/v1/responses").await;
            assert_eq!(
                response.try_into_string().await.expect("body"),
                format!("Bearer {MODEL_KEY}")
            );
            // A label the vault does not hold is reported as unavailable,
            // never as some other key.
            assert_eq!(
                broker.register_stored(bound, "VAULT_MISSING", &[]).err(),
                Some(ModelCredentialBrokerError::StoreUnavailable)
            );
        });
    }

    /// PF-27-S09: under the broker token (low integrity, write-restricted),
    /// the broker opens the encrypted vault, locks it through a read-only
    /// handle to the lock Core created, and decrypts the stored key itself.
    #[cfg(windows)]
    #[test]
    fn pf_27_s09_stored_key_is_read_from_the_vault_under_the_broker_token() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                VAULT_CHILD_TEST,
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(VAULT_CHILD_ENV, "1")
            .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
            .env_remove(CHILD_ENV)
            .output()
            .expect("run vault child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("1 passed"),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// PF-27-S09: the production send path on Windows. A signed request goes
    /// over the checked data pipe and its response streams back; an unsigned
    /// one is refused by the broker; after the broker dies nothing is sent.
    #[cfg(windows)]
    #[tokio::test(flavor = "multi_thread")]
    async fn pf_27_s09_send_goes_only_to_the_broker_and_streams() {
        use crate::credential_broker::model_auth::ModelBrokerRequest;
        use rama_core::futures::StreamExt as _;
        let upstream = start_upstream().await;
        let broker = model_broker(&upstream);
        let credential = broker
            .register(
                binding(upstream.port, "/", ModelAuthHeader::Bearer),
                MODEL_KEY,
            )
            .expect("register");
        let request = |path: &str, frame: Option<String>| {
            let mut headers = rama_http::HeaderMap::new();
            if let Some(frame) = frame {
                headers.insert(
                    MODEL_BROKER_FRAME_HEADER,
                    rama_http::HeaderValue::from_str(&frame).expect("frame"),
                );
            }
            ModelBrokerRequest {
                method: rama_http::Method::POST,
                host: UPSTREAM_HOST.to_string(),
                path_and_query: path.to_string(),
                headers,
                body: rama_core::bytes::Bytes::from_static(b"{}"),
            }
        };

        let frame = credential
            .sign("POST", UPSTREAM_HOST, upstream.port, "/v1/responses")
            .expect("frame");
        let response = broker
            .send(request("/v1/responses", Some(frame)))
            .await
            .expect("brokered response");
        assert_eq!(response.status, 200);
        let body: Vec<u8> = response
            .body
            .map(|chunk| chunk.expect("chunk").to_vec())
            .concat()
            .await;
        assert_eq!(body, format!("Bearer {MODEL_KEY}").into_bytes());

        // A streaming response arrives chunk by chunk (the rest never ends).
        let frame = credential
            .sign("POST", UPSTREAM_HOST, upstream.port, super::STREAM_PATH)
            .expect("frame");
        let mut response = broker
            .send(request(super::STREAM_PATH, Some(frame)))
            .await
            .expect("streamed response");
        let first = tokio::time::timeout(Duration::from_secs(10), response.body.next())
            .await
            .expect("first chunk in time")
            .expect("a chunk")
            .expect("chunk");
        assert_eq!(first.as_ref(), b"first-chunk\n");
        drop(response);

        let unsigned = broker
            .send(request("/v1/responses", None))
            .await
            .expect("broker answer");
        assert_eq!(
            unsigned
                .headers
                .get(super::BROKER_ERROR_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("missing_frame")
        );

        broker.kill_for_test();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while broker.is_alive() && std::time::Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(
            broker.send(request("/v1/responses", None)).await.err(),
            Some(ModelCredentialBrokerError::Unavailable)
        );
    }
}
