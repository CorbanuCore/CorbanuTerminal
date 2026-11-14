use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use codex_exec_server::ExecBackend;
use codex_exec_server::ExecBackendFuture;
use codex_exec_server::ExecParams;
use codex_exec_server::ExecProcess;
use codex_exec_server::ExecProcessEventReceiver;
use codex_exec_server::ExecProcessFuture;
use codex_exec_server::ProcessId;
use codex_exec_server::ProcessSignal;
use codex_exec_server::ReadResponse;
use codex_exec_server::StartedExecProcess;
use codex_exec_server::WriteResponse;
use codex_exec_server::WriteStatus;
use futures::FutureExt;
use futures::future::BoxFuture;
use pretty_assertions::assert_eq;
use rmcp::model::ClientCapabilities;
use rmcp::model::Implementation;
use rmcp::model::InitializeRequestParams;
use rmcp::model::ProtocolVersion;
use rmcp::model::ServerPeerInfo;
use serde_json::Value;
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::DuplexStream;
use tokio::sync::watch;

use super::ElicitationResponse;
use super::RmcpClient;
use crate::ElicitationAction;
use crate::ExecutorStdioServerLauncher;
use crate::in_process_transport::InProcessTransportFactory;
use crate::protocol_mode::McpProtocolMode;
use crate::stdio_server_launcher::StdioServerProcessHandle;

#[path = "rmcp_client_terminate_tests.rs"]
mod terminate_tests;

/// Codex's default MCP startup timeout.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy)]
enum TestServer {
    /// Speaks 2026-07-28 and answers `server/discover` after `discover_delay`.
    Modern { discover_delay: Duration },
    /// Rejects `server/discover` as an unknown method and answers `initialize`.
    Legacy,
}

#[derive(Default)]
struct Connection {
    requests: StdMutex<Vec<String>>,
    /// `protocolVersion` of each `initialize` request.
    initialize_versions: StdMutex<Vec<String>>,
    closed: AtomicBool,
}

/// Opens one in-memory server per transport and records what each one received.
struct TestServerFactory {
    server: TestServer,
    connections: StdMutex<Vec<Arc<Connection>>>,
}

impl TestServerFactory {
    fn requests_per_connection(&self) -> Vec<Vec<String>> {
        self.connections
            .lock()
            .unwrap()
            .iter()
            .map(|connection| connection.requests.lock().unwrap().clone())
            .collect()
    }

    fn initialize_versions(&self) -> Vec<String> {
        self.connections
            .lock()
            .unwrap()
            .iter()
            .flat_map(|connection| connection.initialize_versions.lock().unwrap().clone())
            .collect()
    }

    fn opened_connections(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    fn open_connections(&self) -> usize {
        self.connections
            .lock()
            .unwrap()
            .iter()
            .filter(|connection| !connection.closed.load(Ordering::Acquire))
            .count()
    }
}

impl InProcessTransportFactory for TestServerFactory {
    fn open(&self) -> BoxFuture<'static, io::Result<DuplexStream>> {
        let (client, server) = tokio::io::duplex(64 * 1024);
        let connection = Arc::new(Connection::default());
        self.connections
            .lock()
            .unwrap()
            .push(Arc::clone(&connection));
        let test_server = self.server;
        tokio::spawn(async move {
            serve(server, test_server, &connection).await;
            connection.closed.store(true, Ordering::Release);
        });
        async move { Ok(client) }.boxed()
    }
}

async fn serve(stream: DuplexStream, server: TestServer, connection: &Connection) {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();
    let mut queued_line = None;
    loop {
        let line = match queued_line.take() {
            Some(line) => line,
            None => match lines.next_line().await {
                Ok(Some(line)) => line,
                Ok(None) | Err(_) => return,
            },
        };
        let message: Value = serde_json::from_str(&line).expect("client sent JSON");
        let (Some(method), Some(id)) = (message["method"].as_str(), message.get("id").cloned())
        else {
            continue;
        };
        connection.requests.lock().unwrap().push(method.to_string());
        if method == "initialize" {
            connection.initialize_versions.lock().unwrap().push(
                message["params"]["protocolVersion"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            );
        }
        let reply = match (server, method) {
            (TestServer::Modern { discover_delay }, "server/discover") => {
                let delay = tokio::time::sleep(discover_delay);
                tokio::pin!(delay);
                tokio::select! {
                    () = &mut delay => {}
                    next = lines.next_line() => match next {
                        // The client moved on without waiting; the reply still arrives late.
                        Ok(Some(line)) => {
                            delay.await;
                            queued_line = Some(line);
                        }
                        // The client closed this transport.
                        Ok(None) | Err(_) => return,
                    },
                }
                json!({"jsonrpc": "2.0", "id": id, "result": {
                    "resultType": "complete",
                    "supportedVersions": ["2026-07-28"],
                    "capabilities": {"tools": {}},
                    "_meta": {
                        "io.modelcontextprotocol/serverInfo": {
                            "name": "modern-test-server",
                            "version": "1.0.0",
                        },
                    },
                    "ttlMs": 0,
                    "cacheScope": "private",
                }})
            }
            (TestServer::Legacy, "server/discover") => {
                json!({"jsonrpc": "2.0", "id": id, "error": {
                    "code": -32601,
                    "message": "Method not found: server/discover",
                }})
            }
            (_, "initialize") => json!({"jsonrpc": "2.0", "id": id, "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "serverInfo": {"name": "legacy-test-server", "version": "1.0.0"},
            }}),
            (_, other) => json!({"jsonrpc": "2.0", "id": id, "error": {
                "code": -32601,
                "message": format!("unexpected method {other}"),
            }}),
        };
        let mut bytes = serde_json::to_vec(&reply).expect("serialize reply");
        bytes.push(b'\n');
        if writer.write_all(&bytes).await.is_err() {
            return;
        }
    }
}

struct Started {
    result: anyhow::Result<ServerPeerInfo>,
    requests: Vec<Vec<String>>,
    initialize_versions: Vec<String>,
    /// Test servers still connected once the handshake settled.
    open_connections: usize,
}

fn test_server_factory(server: TestServer) -> Arc<TestServerFactory> {
    Arc::new(TestServerFactory {
        server,
        connections: StdMutex::new(Vec::new()),
    })
}

async fn opted_in_client(
    factory: &Arc<TestServerFactory>,
    discover_timeout: Option<Duration>,
) -> RmcpClient {
    RmcpClient::new_in_process_client_with_protocol_mode(
        Arc::clone(factory) as Arc<dyn InProcessTransportFactory>,
        McpProtocolMode::V20260728,
    )
    .await
    .expect("in-process client")
    .with_discover_timeout(discover_timeout)
}

/// Client params without an explicit `protocolVersion`, so rmcp's default applies.
fn default_init_params() -> InitializeRequestParams {
    InitializeRequestParams::new(
        ClientCapabilities::default(),
        Implementation::new("lifecycle-test", "1.0.0"),
    )
}

async fn initialize(
    client: &RmcpClient,
    params: InitializeRequestParams,
) -> anyhow::Result<ServerPeerInfo> {
    client
        .initialize(
            params,
            Some(STARTUP_TIMEOUT),
            Box::new(|_, _| {
                async {
                    Ok(ElicitationResponse {
                        action: ElicitationAction::Decline,
                        content: None,
                        meta: None,
                    })
                }
                .boxed()
            }),
        )
        .await
}

/// Starts an opted-in 2026-07-28 client against `server`, with the params Codex sends.
async fn start(server: TestServer, discover_timeout: Option<Duration>) -> Started {
    start_with_params(
        server,
        discover_timeout,
        default_init_params().with_protocol_version(ProtocolVersion::V_2025_06_18),
    )
    .await
}

async fn start_with_params(
    server: TestServer,
    discover_timeout: Option<Duration>,
    params: InitializeRequestParams,
) -> Started {
    let factory = test_server_factory(server);
    let client = opted_in_client(&factory, discover_timeout).await;
    let result = initialize(&client, params).await;
    // Let the test servers observe any transport the client dropped.
    tokio::time::sleep(Duration::from_millis(1)).await;
    let open_connections = factory.open_connections();
    client.shutdown().await;
    Started {
        result,
        requests: factory.requests_per_connection(),
        initialize_versions: factory.initialize_versions(),
        open_connections,
    }
}

fn server_name(info: &ServerPeerInfo) -> Option<&str> {
    info.server_info.as_ref().map(|info| info.name.as_str())
}

fn requests(per_connection: &[&[&str]]) -> Vec<Vec<String>> {
    per_connection
        .iter()
        .map(|requests| requests.iter().map(ToString::to_string).collect())
        .collect()
}

// #1950: rmcp's `Auto` lifecycle gives up on `server/discover` after a fixed
// 10 s and sends `initialize` on the same transport, where the late discover
// reply then fails startup with `ConflictInitResponseId`.
#[tokio::test(start_paused = true)]
async fn modern_server_answering_discover_after_ten_seconds_starts_modern() {
    let started = start(
        TestServer::Modern {
            discover_delay: Duration::from_secs(15),
        },
        /*discover_timeout*/ None,
    )
    .await;

    let info = started
        .result
        .expect("a discover reply inside the startup timeout must start the server");
    assert_eq!(info.protocol_version, ProtocolVersion::V_2026_07_28);
    assert_eq!(server_name(&info), Some("modern-test-server"));
    assert_eq!(started.requests, requests(&[&["server/discover"]]));
}

#[tokio::test(start_paused = true)]
async fn modern_server_answering_discover_promptly_starts_modern() {
    let started = start(
        TestServer::Modern {
            discover_delay: Duration::ZERO,
        },
        /*discover_timeout*/ None,
    )
    .await;

    let info = started.result.expect("a prompt modern server must start");
    assert_eq!(info.protocol_version, ProtocolVersion::V_2026_07_28);
    assert_eq!(server_name(&info), Some("modern-test-server"));
    assert_eq!(started.requests, requests(&[&["server/discover"]]));
}

#[tokio::test(start_paused = true)]
async fn legacy_server_rejecting_discover_falls_back_to_initialize() {
    let started = start(TestServer::Legacy, /*discover_timeout*/ None).await;

    let info = started
        .result
        .expect("a legacy server must start through the initialize fallback");
    assert_eq!(info.protocol_version, ProtocolVersion::V_2025_06_18);
    assert_eq!(server_name(&info), Some("legacy-test-server"));
    assert_eq!(
        started.requests.concat(),
        vec!["server/discover".to_string(), "initialize".to_string()]
    );
}

#[tokio::test(start_paused = true)]
async fn legacy_fallback_initializes_on_a_fresh_transport() {
    let started = start(TestServer::Legacy, /*discover_timeout*/ None).await;

    started
        .result
        .expect("a legacy server must start through the initialize fallback");
    assert_eq!(
        started.requests,
        requests(&[&["server/discover"], &["initialize"]])
    );
    assert_eq!(started.open_connections, 1);
}

#[tokio::test(start_paused = true)]
async fn discover_timeout_falls_back_on_a_fresh_transport() {
    let started = start(
        TestServer::Modern {
            discover_delay: Duration::from_secs(15),
        },
        Some(Duration::from_secs(5)),
    )
    .await;

    let info = started
        .result
        .expect("a server that misses the discover wait must start through initialize");
    assert_eq!(info.protocol_version, ProtocolVersion::V_2025_06_18);
    assert_eq!(
        started.requests,
        requests(&[&["server/discover"], &["initialize"]])
    );
    assert_eq!(started.open_connections, 1);
}

/// A server that accepts the transport but never answers `server/discover`.
const SILENT_MODERN: TestServer = TestServer::Modern {
    discover_delay: Duration::from_secs(3600),
};

// #1978 review round 1: the discover wait and the legacy `initialize` each get the
// startup timeout, so a silent server still falls back with the default wait.
#[tokio::test(start_paused = true)]
async fn server_never_answering_discover_falls_back_with_the_default_wait() {
    let started = start(SILENT_MODERN, /*discover_timeout*/ None).await;

    let info = started
        .result
        .expect("a server that never answers discover must start through initialize");
    assert_eq!(info.protocol_version, ProtocolVersion::V_2025_06_18);
    assert_eq!(
        started.requests,
        requests(&[&["server/discover"], &["initialize"]])
    );
    assert_eq!(started.open_connections, 1);
}

#[tokio::test(start_paused = true)]
async fn server_never_answering_discover_falls_back_with_the_wait_at_the_startup_timeout() {
    let started = start(SILENT_MODERN, Some(STARTUP_TIMEOUT)).await;

    let info = started
        .result
        .expect("a discover wait equal to the startup timeout must still leave time to fall back");
    assert_eq!(info.protocol_version, ProtocolVersion::V_2025_06_18);
    assert_eq!(
        started.requests,
        requests(&[&["server/discover"], &["initialize"]])
    );
    assert_eq!(started.open_connections, 1);
}

// `Auto` sends its fallback `initialize` with the legacy 2025-06-18 version whatever the
// caller passed; the fresh-transport fallback must do the same.
#[tokio::test(start_paused = true)]
async fn legacy_fallback_sends_the_legacy_protocol_version() {
    let started = start_with_params(
        TestServer::Legacy,
        /*discover_timeout*/ None,
        default_init_params(),
    )
    .await;

    started
        .result
        .expect("a legacy server must start through the initialize fallback");
    assert_eq!(started.initialize_versions, vec!["2025-06-18".to_string()]);
}

// A `shutdown()` during the discover wait must not let the fallback start another server.
#[tokio::test(start_paused = true)]
async fn shutdown_during_the_discover_wait_starts_no_fallback_server() {
    let factory = test_server_factory(SILENT_MODERN);
    let client = Arc::new(opted_in_client(&factory, Some(Duration::from_secs(5))).await);
    let initializing = tokio::spawn({
        let client = Arc::clone(&client);
        async move {
            initialize(
                &client,
                default_init_params().with_protocol_version(ProtocolVersion::V_2025_06_18),
            )
            .await
        }
    });
    tokio::time::sleep(Duration::from_secs(1)).await;
    client.shutdown().await;

    let error = initializing
        .await
        .expect("initialize task")
        .expect_err("initialize must fail once the client is shut down");
    assert!(
        error.to_string().contains("MCP client is shut down"),
        "unexpected error: {error:#}"
    );
    assert_eq!(factory.opened_connections(), 1);
}

/// Which executor call never completes during the legacy fallback restart.
#[derive(Clone, Copy)]
enum ExecutorHang {
    /// Terminating the first server process.
    Terminate,
    /// Starting the second server process.
    Restart,
    /// None: every executor call completes.
    Nothing,
}

/// A remote stdio server process that never answers `server/discover`.
struct SilentExecProcess {
    process_id: ProcessId,
    terminate_hangs: bool,
    /// Counts the terminate requests the executor received.
    terminate_requests: Arc<watch::Sender<usize>>,
}

impl ExecProcess for SilentExecProcess {
    fn process_id(&self) -> &ProcessId {
        &self.process_id
    }

    fn subscribe_wake(&self) -> watch::Receiver<u64> {
        watch::channel(0).1
    }

    fn subscribe_events(&self) -> ExecProcessEventReceiver {
        ExecProcessEventReceiver::empty()
    }

    fn read(
        &self,
        _after_seq: Option<u64>,
        _max_bytes: Option<usize>,
        _wait_ms: Option<u64>,
    ) -> ExecProcessFuture<'_, ReadResponse> {
        Box::pin(std::future::pending())
    }

    fn write(&self, _chunk: Vec<u8>) -> ExecProcessFuture<'_, WriteResponse> {
        Box::pin(async {
            Ok(WriteResponse {
                status: WriteStatus::Accepted,
            })
        })
    }

    fn signal(&self, _signal: ProcessSignal) -> ExecProcessFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }

    fn terminate(&self) -> ExecProcessFuture<'_, ()> {
        self.terminate_requests
            .send_modify(|requests| *requests += 1);
        if self.terminate_hangs {
            Box::pin(std::future::pending())
        } else {
            Box::pin(async { Ok(()) })
        }
    }
}

/// An executor whose terminate or second start never completes.
struct HangingExecBackend {
    hang: ExecutorHang,
    starts: AtomicUsize,
}

impl ExecBackend for HangingExecBackend {
    fn start(&self, params: ExecParams) -> ExecBackendFuture<'_> {
        let earlier_starts = self.starts.fetch_add(1, Ordering::AcqRel);
        if earlier_starts > 0 && matches!(self.hang, ExecutorHang::Restart) {
            return Box::pin(std::future::pending());
        }
        let process: Arc<dyn ExecProcess> = Arc::new(SilentExecProcess {
            process_id: params.process_id,
            terminate_hangs: matches!(self.hang, ExecutorHang::Terminate),
            terminate_requests: Arc::new(watch::channel(0).0),
        });
        Box::pin(async move {
            Ok(StartedExecProcess {
                process,
                sandbox_type: None,
            })
        })
    }
}

/// How long the remote tests wait for `server/discover` before falling back.
const REMOTE_DISCOVER_WAIT: Duration = Duration::from_secs(5);

/// Far past the discover wait plus the startup timeout; reaching it means startup hung.
const HANG_DEADLINE: Duration = Duration::from_secs(3600);

struct RemoteStart {
    result: anyhow::Result<ServerPeerInfo>,
    elapsed: Duration,
    executor_starts: usize,
}

/// Starts an opted-in remote stdio server whose discover goes unanswered, so the client
/// falls back and restarts it through an executor that hangs on `hang`.
async fn start_remote_with_hanging_executor(hang: ExecutorHang) -> RemoteStart {
    let backend = hanging_exec_backend(hang);
    let client = opted_in_remote_client(&backend).await;

    let started_at = tokio::time::Instant::now();
    let result = tokio::time::timeout(
        HANG_DEADLINE,
        initialize(
            &client,
            default_init_params().with_protocol_version(ProtocolVersion::V_2025_06_18),
        ),
    )
    .await
    .expect("the remote legacy fallback restart must end within the startup timeout");
    let elapsed = started_at.elapsed();
    client.shutdown().await;
    RemoteStart {
        result,
        elapsed,
        executor_starts: backend.starts.load(Ordering::Acquire),
    }
}

fn hanging_exec_backend(hang: ExecutorHang) -> Arc<HangingExecBackend> {
    Arc::new(HangingExecBackend {
        hang,
        starts: AtomicUsize::new(0),
    })
}

/// An opted-in remote stdio client on `backend` that waits `REMOTE_DISCOVER_WAIT` for discover.
async fn opted_in_remote_client(backend: &Arc<HangingExecBackend>) -> RmcpClient {
    let env = HashMap::from([(
        OsString::from("CODEX_MCP_PROTOCOL_VERSION"),
        OsString::from("2026-07-28"),
    )]);
    RmcpClient::new_stdio_client_with_protocol_mode(
        OsString::from("remote-mcp-server"),
        Vec::new(),
        Some(env),
        &[],
        // Match the executor stdio integration tests: an absolute cwd with no trailing
        // separator on every platform.
        Some(
            std::env::current_dir()
                .expect("current dir")
                .to_string_lossy()
                .into_owned(),
        ),
        Arc::new(ExecutorStdioServerLauncher::new(
            Arc::clone(backend) as Arc<dyn ExecBackend>
        )),
        McpProtocolMode::V20260728,
    )
    .await
    .expect("remote stdio client")
    .with_discover_timeout(Some(REMOTE_DISCOVER_WAIT))
}

fn assert_fallback_timed_out(started: &RemoteStart) {
    let Err(error) = &started.result else {
        panic!("a hung restart must fail startup");
    };
    // The retry deadline leaves the fallback a hair under the startup timeout ("29.99…s").
    let message = error.to_string();
    assert!(
        message.starts_with("timed out handshaking with MCP server after ")
            && message.ends_with(" during the legacy initialize fallback"),
        "unexpected error: {error:#}"
    );
    // The discover wait, then one startup timeout for the restart and `initialize`.
    let expected = REMOTE_DISCOVER_WAIT + STARTUP_TIMEOUT;
    assert!(
        started.elapsed > expected - Duration::from_secs(1) && started.elapsed <= expected,
        "startup gave up after {:?}, expected about {expected:?}",
        started.elapsed
    );
}

// #2019: on the remote executor the restart (terminate, then spawn) ran outside any
// timeout, so an executor that never finished either step hung startup.
#[tokio::test(start_paused = true)]
async fn remote_fallback_times_out_when_the_executor_never_terminates_the_first_server() {
    let started = start_remote_with_hanging_executor(ExecutorHang::Terminate).await;

    assert_fallback_timed_out(&started);
    assert_eq!(started.executor_starts, 1);
}

#[tokio::test(start_paused = true)]
async fn remote_fallback_times_out_when_the_executor_never_starts_the_second_server() {
    let started = start_remote_with_hanging_executor(ExecutorHang::Restart).await;

    assert_fallback_timed_out(&started);
    assert_eq!(started.executor_starts, 2);
}

// #1978's shutdown race on the remote path: the executor launcher checks the client's process
// slot, as the local launcher does, so a `shutdown()` during the discover wait keeps the
// fallback from starting a second server.
#[tokio::test(start_paused = true)]
async fn remote_shutdown_during_the_discover_wait_starts_no_fallback_server() {
    let backend = hanging_exec_backend(ExecutorHang::Nothing);
    let client = Arc::new(opted_in_remote_client(&backend).await);
    let initializing = tokio::spawn({
        let client = Arc::clone(&client);
        async move {
            initialize(
                &client,
                default_init_params().with_protocol_version(ProtocolVersion::V_2025_06_18),
            )
            .await
        }
    });
    tokio::time::sleep(Duration::from_secs(1)).await;
    client.shutdown().await;

    let error = initializing
        .await
        .expect("initialize task")
        .expect_err("initialize must fail once the client is shut down");
    assert!(
        format!("{error:#}").contains("MCP client is shut down"),
        "unexpected error: {error:#}"
    );
    assert_eq!(backend.starts.load(Ordering::Acquire), 1);
}

// #2041: when the fallback's startup timeout cancelled a terminate the executor never
// answered, the handle stayed marked terminated, so dropping it sent no terminate.
#[tokio::test(start_paused = true)]
async fn dropping_a_handle_whose_terminate_was_cancelled_terminates_again() {
    let (terminate_requests, mut requests) = watch::channel(0);
    // Held here too, so the counter outlives the process the handle drops.
    let terminate_requests = Arc::new(terminate_requests);
    let handle = StdioServerProcessHandle::executor(
        "remote-mcp-server".to_string(),
        Arc::new(SilentExecProcess {
            process_id: ProcessId::from("remote-mcp-server"),
            terminate_hangs: true,
            terminate_requests: Arc::clone(&terminate_requests),
        }),
    );

    tokio::time::timeout(STARTUP_TIMEOUT, handle.terminate())
        .await
        .expect_err("the executor never answers the terminate");
    assert_eq!(*requests.borrow(), 1);

    drop(handle);
    tokio::time::timeout(HANG_DEADLINE, requests.wait_for(|requests| *requests == 2))
        .await
        .expect("dropping the handle must send the terminate again")
        .expect("the test holds the request counter");
}
