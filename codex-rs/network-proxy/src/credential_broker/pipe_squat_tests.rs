//! #390 probes: a hostile same-user process squats the Windows broker's
//! named pipes.
//!
//! The broker's pipe names carry a fresh 128-bit nonce, so a squatter cannot
//! claim them before the broker starts; these probes inject the nonce
//! (`TEST_NONCE_ENV`, read only in test builds) to measure that case anyway.
//! The squatter is this test binary re-executed (`ROLE_ENV=squat`): it creates
//! the names first (`FILE_FLAG_FIRST_PIPE_INSTANCE` or one instance at most),
//! or joins a live broker pipe, and reports every connection it gets with the
//! client's process id and the number of bytes the client sent.
//!
//! Measured: the broker never joins a squatted name (it reports the squat and
//! exits); Core refuses a pipe another process created first before writing
//! anything (the control hello, which carries the channel key, or a signed
//! frame), fails with `PipeSquatted` instead of hanging, and never sends a
//! request without the broker. Windows reports the first creator as the
//! server of every instance, so a squatter can add instances to a live data
//! pipe that pass the process-id check: those get only Core's random peer
//! challenge, never a request, because they cannot prove the channel key.
//! The control pipe allows one instance, so none can be added. The same
//! holds when a squatter claims the names a dead broker freed, and when the
//! broker restarts.
#![expect(
    clippy::print_stderr,
    reason = "each probe prints what it measured, for the gate evidence"
)]

use super::UPSTREAM_HOST;
use super::Upstream;
use super::launcher;
use super::options;
use super::start_upstream;
use crate::credential_broker::isolated::IsolatedBrokerClient;
use crate::credential_broker::isolated::IsolatedBrokerError;
use crate::credential_broker::isolated::IsolatedBrokerLauncher;
use crate::credential_broker::isolated::pipe;
use crate::credential_broker::isolated::protocol::BrokerBootstrap;
use crate::credential_broker::isolated::protocol::CONTROL_PROTOCOL_VERSION;
use crate::credential_broker::model_auth::MODEL_BROKER_FRAME_HEADER;
use crate::credential_broker::model_auth::ModelAuthHeader;
use crate::credential_broker::model_auth::ModelBrokerRequest;
use crate::credential_broker::model_auth::ModelCredential;
use crate::credential_broker::model_auth::ModelCredentialBinding;
use crate::credential_broker::model_auth::ModelCredentialBroker;
use crate::credential_broker::model_auth::ModelCredentialBrokerError;
use crate::credential_broker::model_auth::ModelCredentialBrokerOptions;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::ipc::PIPE_CHALLENGE_BYTES;
use pretty_assertions::assert_eq;
use rama_core::Service as _;
use rama_core::futures::StreamExt as _;
use std::ffi::OsString;
use std::io::BufRead as _;
use std::io::Write as _;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use tokio::io::AsyncReadExt as _;
use tokio::net::windows::named_pipe::PipeMode;
use tokio::net::windows::named_pipe::ServerOptions;

const ROLE_ENV: &str = "CODEX_SEC390_ROLE";
const NAMES_ENV: &str = "CODEX_SEC390_NAMES";
const MODE_ENV: &str = "CODEX_SEC390_MODE";
const WAIT_ENV: &str = "CODEX_SEC390_WAIT";
const CHILD_TEST: &str = "credential_broker::isolated::tests::sec_390::sec_390_child_entry";
const REPORT_PREFIX: &str = "sec390:";
const MODEL_KEY: &str = "sk-sec390SyntheticModelKey0000000000000000000";
/// Core's connect deadline is 5 s; a refusal must come well before this.
const NO_HANG: Duration = Duration::from_secs(20);

#[test]
fn sec_390_child_entry() {
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("squat") => squatter_main(),
        Ok("fake-broker") => fake_broker_main(),
        _ => {}
    }
}

/// How the squatter creates its instances.
#[derive(Clone, Copy, Debug)]
enum Mode {
    /// `FILE_FLAG_FIRST_PIPE_INSTANCE`: fails if the name exists.
    First,
    /// At most one instance: nobody can add another.
    MaxOne,
    /// An extra instance of an existing pipe (the DACL grants the user).
    Join,
    /// [`Mode::Join`], serving one connection and then no more.
    JoinOnce,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Mode::First => "first",
            Mode::MaxOne => "max1",
            Mode::Join => "join",
            Mode::JoinOnce => "join-once",
        }
    }
}

/// One connection a squatter received.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Connection {
    pid: u32,
    bytes: usize,
}

/// A squatter process and what it reported.
struct Squatter {
    child: std::process::Child,
    lines: Arc<Mutex<Vec<String>>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl Squatter {
    fn start(names: &[&str], mode: Mode, wait: bool) -> Self {
        let mut child = Command::new(std::env::current_exe().expect("test binary"))
            .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
            .env(ROLE_ENV, "squat")
            .env(NAMES_ENV, names.join(","))
            .env(MODE_ENV, mode.as_str())
            .env(WAIT_ENV, if wait { "1" } else { "0" })
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("start squatter");
        let stdout = child.stdout.take().expect("squatter stdout");
        let lines = Arc::new(Mutex::new(Vec::new()));
        let reader = std::thread::spawn({
            let lines = lines.clone();
            move || {
                for line in std::io::BufReader::new(stdout).lines() {
                    let Ok(line) = line else { break };
                    if let Some(start) = line.find(REPORT_PREFIX) {
                        let report = line[start + REPORT_PREFIX.len()..].trim().to_string();
                        lines.lock().expect("lines").push(report);
                    }
                }
            }
        });
        let squatter = Self {
            child,
            lines,
            reader: Some(reader),
        };
        let deadline = Instant::now() + Duration::from_secs(60);
        while !squatter.reports().iter().any(|line| line == "ready") {
            assert!(
                Instant::now() < deadline,
                "squatter never got ready: {:?}",
                squatter.reports()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        squatter
    }

    fn reports(&self) -> Vec<String> {
        self.lines.lock().expect("lines").clone()
    }

    /// Whether each name's instance was created, in order.
    fn created(&self) -> Vec<bool> {
        self.reports()
            .iter()
            .filter_map(|line| line.strip_prefix("created "))
            .map(|rest| rest.ends_with(" ok"))
            .collect()
    }

    /// Stops the squatter once its pending reports are in.
    fn stop(mut self) -> Vec<Connection> {
        // A connection is reported once its client closes (at once, for a
        // refused one) or after the squatter's 2 s read deadline.
        std::thread::sleep(Duration::from_millis(2_500));
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        self.reports()
            .iter()
            .filter_map(|line| line.strip_prefix("conn "))
            .map(|rest| {
                let field = |name: &str| -> usize {
                    rest.split(' ')
                        .find_map(|part| part.strip_prefix(name))
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(|| panic!("bad report: {rest}"))
                };
                Connection {
                    pid: u32::try_from(field("pid=")).expect("pid"),
                    bytes: field("bytes="),
                }
            })
            .collect()
    }
}

impl Drop for Squatter {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn report(line: &str) {
    let mut stdout = std::io::stdout();
    // Own line: libtest prints the test name without a newline first.
    let _ = writeln!(stdout, "\n{REPORT_PREFIX}{line}");
    let _ = stdout.flush();
}

/// Creates the instances, reports `ready`, then serves every connection:
/// records the client's process id and every byte it sends, sends nothing.
fn squatter_main() {
    let names: Vec<String> = std::env::var(NAMES_ENV)
        .expect("names")
        .split(',')
        .map(str::to_string)
        .collect();
    let mode = std::env::var(MODE_ENV).expect("mode");
    let wait = std::env::var(WAIT_ENV).as_deref() == Ok("1");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async move {
        let mut servers = Vec::new();
        for (index, name) in names.iter().enumerate() {
            let mut options = ServerOptions::new();
            options
                .access_inbound(true)
                .access_outbound(true)
                .pipe_mode(PipeMode::Byte);
            match mode.as_str() {
                "first" => {
                    options.first_pipe_instance(true);
                }
                "max1" => {
                    options.max_instances(1);
                }
                _ => {}
            }
            // With WAIT, retry while a dead broker's names are still held.
            let deadline = Instant::now() + Duration::from_secs(if wait { 15 } else { 0 });
            let created = loop {
                match options.create(name) {
                    Ok(server) => break Ok(server),
                    Err(error) if Instant::now() < deadline => {
                        let _ = error;
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    Err(error) => break Err(error),
                }
            };
            match created {
                Ok(server) => {
                    report(&format!("created {index} ok"));
                    servers.push((name.clone(), server));
                }
                Err(error) => report(&format!(
                    "created {index} err={}",
                    error.raw_os_error().unwrap_or(-1)
                )),
            }
        }
        report("ready");
        let once = mode == "join-once";
        for (name, server) in servers {
            tokio::spawn(serve_squat(server, name, mode == "max1", once));
        }
        std::future::pending::<()>().await;
    });
}

/// Serves connections on `server`, each on a fresh instance where the pipe
/// allows one: a reused instance can pass data read ahead from one client
/// into the next one's report.
async fn serve_squat(
    mut server: tokio::net::windows::named_pipe::NamedPipeServer,
    name: String,
    max_one: bool,
    once: bool,
) {
    loop {
        let connected = server.connect().await;
        // A client that closed before the connect completed is still known.
        let pid = pipe::client_pid(&server);
        if connected.is_ok() || pid.is_some() {
            let mut bytes = 0;
            let mut buffer = [0_u8; 4096];
            loop {
                match tokio::time::timeout(Duration::from_secs(2), server.read(&mut buffer)).await {
                    Ok(Ok(0)) | Ok(Err(_)) | Err(_) => break,
                    Ok(Ok(read)) => bytes += read,
                }
            }
            report(&format!("conn pid={} bytes={bytes}", pid.unwrap_or(0)));
        }
        if once {
            // Keep the instance, never listening again.
            std::future::pending::<()>().await;
        }
        // The next instance first, so the name never disappears; a pipe of
        // one instance at most can only reuse its instance.
        let mut options = ServerOptions::new();
        options
            .access_inbound(true)
            .access_outbound(true)
            .pipe_mode(PipeMode::Byte);
        if max_one {
            options.max_instances(1);
        }
        match options.create(&name) {
            Ok(next) => server = next,
            Err(_) => {
                let _ = server.disconnect();
            }
        }
    }
}

/// A broker stand-in that names the squatted pipe as its control pipe (as a
/// broker that joined a squatted name would), then waits to be killed.
fn fake_broker_main() {
    let name = std::env::var(NAMES_ENV).expect("name");
    let bootstrap = BrokerBootstrap {
        protocol_version: CONTROL_PROTOCOL_VERSION,
        control_socket: name,
        pipe_taken: false,
    };
    let mut stdout = std::io::stdout();
    let _ = writeln!(
        stdout,
        "{}",
        serde_json::to_string(&bootstrap).expect("bootstrap")
    );
    let _ = stdout.flush();
    std::thread::sleep(Duration::from_secs(120));
}

fn random_nonce() -> String {
    rand::random::<[u8; 16]>()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn names_for(nonce: &str) -> (String, String) {
    (
        format!("{}{nonce}-c", pipe::PIPE_PREFIX),
        format!("{}{nonce}-b", pipe::PIPE_PREFIX),
    )
}

fn nonce_of(data_name: &str) -> String {
    data_name
        .strip_prefix(pipe::PIPE_PREFIX)
        .and_then(|rest| rest.strip_suffix("-b"))
        .expect("broker data pipe name")
        .to_string()
}

fn launcher_with_nonce(upstream: &Upstream, nonce: Option<&str>) -> IsolatedBrokerLauncher {
    let launcher = launcher(upstream, /*controller_pid_override*/ None);
    match nonce {
        Some(nonce) => launcher.with_env(pipe::TEST_NONCE_ENV, nonce),
        None => launcher,
    }
}

fn model_broker(
    upstream: &Upstream,
    nonce: Option<&str>,
) -> Result<ModelCredentialBroker, ModelCredentialBrokerError> {
    ModelCredentialBroker::spawn_with_launcher(
        ModelCredentialBrokerOptions::default(),
        launcher_with_nonce(upstream, nonce),
    )
}

fn register(broker: &ModelCredentialBroker, upstream: &Upstream) -> ModelCredential {
    broker
        .register(
            ModelCredentialBinding {
                host: UPSTREAM_HOST.to_string(),
                port: upstream.port,
                path_prefix: "/".to_string(),
                header: ModelAuthHeader::Bearer,
            },
            MODEL_KEY,
        )
        .expect("register")
}

/// A signed request on the production send path (`ModelCredentialBroker::send`).
async fn send_signed(
    broker: &ModelCredentialBroker,
    frame: String,
) -> Result<u16, ModelCredentialBrokerError> {
    let mut headers = rama_http::HeaderMap::new();
    headers.insert(
        MODEL_BROKER_FRAME_HEADER,
        rama_http::HeaderValue::from_str(&frame).expect("frame"),
    );
    let response = broker
        .send(ModelBrokerRequest {
            method: rama_http::Method::POST,
            host: UPSTREAM_HOST.to_string(),
            path_and_query: "/v1/responses".to_string(),
            headers,
            body: rama_core::bytes::Bytes::from_static(b"{}"),
        })
        .await?;
    let status = response.status.as_u16();
    let body: Vec<u8> = response
        .body
        .map(|chunk| chunk.expect("chunk").to_vec())
        .concat()
        .await;
    // The broker attached the key (compared, never printed).
    assert!(
        body == format!("Bearer {MODEL_KEY}").into_bytes(),
        "the upstream did not get the brokered key"
    );
    Ok(status)
}

fn frame(credential: &ModelCredential, upstream: &Upstream) -> String {
    credential
        .sign("POST", UPSTREAM_HOST, upstream.port, "/v1/responses")
        .expect("frame")
}

fn assert_nothing_sent(connections: &[Connection]) {
    assert!(
        connections.iter().all(|connection| connection.bytes == 0),
        "a squatter received bytes: {connections:?}"
    );
}

/// What a squatter that joined a live data pipe may receive from Core: its
/// random peer challenge, never a request, frame or key.
fn assert_only_challenges(connections: &[Connection]) {
    assert!(
        connections
            .iter()
            .all(|connection| matches!(connection.bytes, 0 | PIPE_CHALLENGE_BYTES)),
        "a squatter received more than a challenge: {connections:?}"
    );
}

fn test_mac() -> BrokerChannelMac {
    BrokerChannelMac::from_secret([5; 32])
}

/// The squatter was reached by this process (Core) and refused.
fn reached_by_core(connections: &[Connection]) -> bool {
    connections
        .iter()
        .any(|connection| connection.pid == std::process::id())
}

/// Exclusive squat before the broker starts, both ways the issue names
/// (`FILE_FLAG_FIRST_PIPE_INSTANCE`, one instance at most): the broker does
/// not join the squatter's pipe, Core fails with `PipeSquatted` at once and
/// never connects to the squatter. Positive control: the same names, once
/// free, start a broker that answers through the key it holds.
#[tokio::test(flavor = "multi_thread")]
async fn sec_390_exclusive_squat_before_start_fails_closed() {
    let upstream = start_upstream().await;
    for mode in [Mode::First, Mode::MaxOne] {
        let nonce = random_nonce();
        let (control, data) = names_for(&nonce);
        let squatter = Squatter::start(&[&control, &data], mode, /*wait*/ false);
        assert_eq!(squatter.created(), vec![true, true], "{mode:?}");

        let started = Instant::now();
        let spawned = model_broker(&upstream, Some(&nonce)).err();
        let elapsed = started.elapsed();
        assert_eq!(
            spawned,
            Some(ModelCredentialBrokerError::PipeSquatted),
            "{mode:?}"
        );
        assert!(elapsed < NO_HANG, "{mode:?}: took {elapsed:?}");
        let connections = squatter.stop();
        assert_eq!(
            connections,
            Vec::new(),
            "{mode:?}: Core reached the squatter"
        );
        eprintln!("sec_390 {mode:?}: broker refused in {elapsed:?}; squatter got no connection");

        // Positive control: the names are free now; the injected nonce works.
        let broker = model_broker(&upstream, Some(&nonce)).expect("control broker");
        assert_eq!(nonce_of(&broker.socket_path().to_string_lossy()), nonce);
        let credential = register(&broker, &upstream);
        assert_eq!(
            send_signed(&broker, frame(&credential, &upstream)).await,
            Ok(200)
        );
    }
}

/// A broker that reports a control pipe some other process serves (what a
/// broker joining a squatted name would do): Core's server process-id check
/// refuses it before the hello (which carries the channel key) and fails with
/// `PipeSquatted` within the connect deadline.
#[tokio::test(flavor = "multi_thread")]
async fn sec_390_control_pipe_served_by_a_squatter_is_refused() {
    for mode in [Mode::First, Mode::MaxOne] {
        let (control, _) = names_for(&random_nonce());
        let squatter = Squatter::start(&[&control], mode, /*wait*/ false);
        assert_eq!(squatter.created(), vec![true], "{mode:?}");
        let fake = IsolatedBrokerLauncher::test_harness(
            /*program*/ None,
            [CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"]
                .into_iter()
                .map(OsString::from)
                .collect(),
            vec![
                (OsString::from(ROLE_ENV), OsString::from("fake-broker")),
                (OsString::from(NAMES_ENV), OsString::from(&control)),
            ],
            /*controller_pid_override*/ None,
        );
        let started = Instant::now();
        let spawned = IsolatedBrokerClient::spawn(&fake, options()).err();
        let elapsed = started.elapsed();
        assert_eq!(spawned, Some(IsolatedBrokerError::PipeSquatted), "{mode:?}");
        assert!(elapsed < NO_HANG, "{mode:?}: took {elapsed:?}");
        let connections = squatter.stop();
        assert!(reached_by_core(&connections), "{mode:?}: {connections:?}");
        assert_nothing_sent(&connections);
        eprintln!(
            "sec_390 {mode:?}: control pipe refused in {elapsed:?}; squatter saw {} connection(s), 0 bytes",
            connections.len()
        );
    }
}

/// Core's pipe connects against exclusive squatters: refused before any
/// byte, `squatted_pipe_error` once the 5 s deadline passes, and the HTTP
/// connector reports the squat (the HTTP client's error does not).
#[tokio::test(flavor = "multi_thread")]
async fn sec_390_client_refuses_an_exclusive_squatter_and_sends_nothing() {
    let own_pid = std::process::id();
    for mode in [Mode::First, Mode::MaxOne] {
        let (control, data) = names_for(&random_nonce());
        let squatter = Squatter::start(&[&control, &data], mode, /*wait*/ false);
        assert_eq!(squatter.created(), vec![true, true], "{mode:?}");

        let started = Instant::now();
        let control_result = tokio::task::spawn_blocking({
            let control = control.clone();
            move || pipe::connect_control(&control, own_pid).map(|_| ())
        })
        .await
        .expect("connect");
        let control_elapsed = started.elapsed();
        let error = control_result.expect_err("control squatter accepted");
        assert!(pipe::is_squatted_pipe_error(&error), "{mode:?}: {error}");
        assert!(control_elapsed < NO_HANG, "{mode:?}: {control_elapsed:?}");

        let error = pipe::connect_data(&data, own_pid, &test_mac())
            .await
            .expect_err("data squatter accepted");
        assert!(pipe::is_squatted_pipe_error(&error), "{mode:?}: {error}");

        let squatted = Arc::new(AtomicBool::new(false));
        let client = crate::upstream::UpstreamClient::named_pipe(
            &data,
            own_pid,
            Arc::new(test_mac()),
            squatted.clone(),
        );
        let request = rama_http::Request::builder()
            .method("POST")
            .uri("/v1/responses")
            .header(rama_http::header::HOST, UPSTREAM_HOST)
            .header(MODEL_BROKER_FRAME_HEADER, "sec390-frame-must-not-leave")
            .body(rama_http::Body::from("sec390-body-must-not-leave"))
            .expect("request");
        assert!(client.serve(request).await.is_err(), "{mode:?}");
        assert!(squatted.load(Ordering::Acquire), "{mode:?}");

        let connections = squatter.stop();
        assert!(reached_by_core(&connections), "{mode:?}: {connections:?}");
        assert_nothing_sent(&connections);
        eprintln!(
            "sec_390 {mode:?}: control refused in {control_elapsed:?}; squatter saw {} connection(s), 0 bytes",
            connections.len()
        );
    }
}

/// The gap #390's probes measured: Windows reports the first instance's
/// creator as the server of every instance, so an instance a squatter adds
/// to a live pipe passes Core's process-id check.
#[tokio::test(flavor = "multi_thread")]
async fn sec_390_an_added_instance_reports_the_first_creator() {
    let own_pid = std::process::id();
    let (_, data) = names_for(&random_nonce());
    let _listener = pipe::PipeListener::bind(&data).expect("bind");
    // Our only instance is taken, so the squatter's is the one listening.
    let _occupant = open_client(&data).expect("occupy");
    let squatter = Squatter::start(&[&data], Mode::Join, /*wait*/ false);
    assert_eq!(squatter.created(), vec![true]);
    let reached = open_client(&data).expect("open the squatter's instance");
    assert_eq!(pipe::server_pid(&reached), Some(own_pid));
    drop(reached);
    let connections = squatter.stop();
    assert_eq!(
        connections,
        vec![Connection {
            pid: own_pid,
            bytes: 0
        }],
        "the squatter served it"
    );
}

/// So the control pipe allows one instance (nobody can add one, however
/// they create it), and on the data pipe Core writes its request only after
/// the server proves the channel key: a squatter's added instance gets only
/// the random challenge, and Core goes on to its broker's own instance.
#[tokio::test(flavor = "multi_thread")]
async fn sec_390_joined_squatter_gets_only_a_challenge() {
    let own_pid = std::process::id();
    let (control, data) = names_for(&random_nonce());
    let control_listener = pipe::PipeListener::bind_single(&control).expect("bind control");
    for mode in [Mode::Join, Mode::MaxOne, Mode::First] {
        let squatter = Squatter::start(&[&control], mode, /*wait*/ false);
        assert_eq!(
            squatter.created(),
            vec![false],
            "{mode:?} added a control instance"
        );
        assert_eq!(squatter.stop(), Vec::new());
    }
    drop(control_listener);

    let mut listener = pipe::PipeListener::bind(&data).expect("bind data");
    let occupant = open_client(&data).expect("occupy");
    let squatter = Squatter::start(&[&data], Mode::JoinOnce, /*wait*/ false);
    assert_eq!(squatter.created(), vec![true]);
    let started = Instant::now();
    let connect = {
        let data = data.clone();
        tokio::spawn(async move { pipe::connect_data(&data, own_pid, &test_mac()).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    // Accepting the occupant creates our next listening instance.
    let occupied = listener.accept(own_pid).await.expect("accept occupant");
    let mut served = listener.accept(own_pid).await.expect("accept Core");
    pipe::answer_peer_challenge(&mut served, &test_mac(), own_pid)
        .await
        .expect("answer the challenge");
    let result = connect.await.expect("connect task");
    assert!(result.is_ok(), "{result:?}");
    let elapsed = started.elapsed();
    drop((occupied, occupant, served, result));

    let connections = squatter.stop();
    assert!(reached_by_core(&connections), "{connections:?}");
    assert_only_challenges(&connections);
    eprintln!(
        "sec_390 joined squatter: Core reached its broker in {elapsed:?}; squatter saw {connections:?} (challenge bytes only)"
    );
}

fn open_client(name: &str) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name)
}

/// The broker dies and a squatter claims the data pipe name it freed: Core
/// sends nothing there (a dead broker is refused, and the pipe's server is
/// not the broker); a restart under the same names fails with `PipeSquatted`.
/// A restart under fresh names works, and a squatter racing in on them can
/// only join (it cannot claim them) and still receives nothing.
#[tokio::test(flavor = "multi_thread")]
async fn sec_390_squat_after_broker_death_and_restart() {
    let upstream = start_upstream().await;
    let broker = model_broker(&upstream, None).expect("broker");
    let credential = register(&broker, &upstream);
    assert_eq!(
        send_signed(&broker, frame(&credential, &upstream)).await,
        Ok(200)
    );
    let data = broker.socket_path().to_string_lossy().into_owned();
    let nonce = nonce_of(&data);
    let broker_pid = broker.pid_for_test().expect("broker pid");
    let stale_frame = frame(&credential, &upstream);

    broker.kill_for_test();
    let deadline = Instant::now() + Duration::from_secs(5);
    while broker.is_alive() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(!broker.is_alive());
    let squatter = Squatter::start(&[&data], Mode::First, /*wait*/ true);
    assert_eq!(
        squatter.created(),
        vec![true],
        "the dead broker's name is free"
    );

    assert_eq!(
        send_signed(&broker, stale_frame).await,
        Err(ModelCredentialBrokerError::Unavailable)
    );
    // Before Core notices the death, the process-id check still refuses it.
    let error = pipe::connect_data(&data, broker_pid, &test_mac())
        .await
        .expect_err("squatter accepted as the dead broker");
    assert!(pipe::is_squatted_pipe_error(&error), "{error}");

    // Core restarts (this process's old broker client goes) and its new
    // broker gets the same names: the squatter's name is not joined.
    drop((credential, broker));
    tokio::time::sleep(Duration::from_millis(500)).await;
    let started = Instant::now();
    assert_eq!(
        model_broker(&upstream, Some(&nonce)).err(),
        Some(ModelCredentialBrokerError::PipeSquatted)
    );
    assert!(started.elapsed() < NO_HANG, "{:?}", started.elapsed());
    let connections = squatter.stop();
    assert!(reached_by_core(&connections), "{connections:?}");
    assert_nothing_sent(&connections);

    // Restart under fresh names; a squatter races in on the live pipe.
    let restarted = model_broker(&upstream, None).expect("restarted broker");
    let credential = register(&restarted, &upstream);
    let data = restarted.socket_path().to_string_lossy().into_owned();
    assert_ne!(nonce_of(&data), nonce);
    // The first-instance flag cannot claim a live name.
    let squatter = Squatter::start(&[&data], Mode::First, /*wait*/ false);
    assert_eq!(squatter.created(), vec![false], "claimed a live pipe");
    assert_eq!(squatter.stop(), Vec::new());
    // Measured: Windows keeps the first instance's limit, so a one-instance
    // create adds an instance to the live data pipe like any other. Each
    // gets only Core's challenges; requests reach the broker.
    for mode in [Mode::MaxOne, Mode::Join] {
        let squatter = Squatter::start(&[&data], mode, /*wait*/ false);
        assert_eq!(squatter.created(), vec![true], "{mode:?}");
        let (mut served, mut refused) = (0, 0);
        for _ in 0..10 {
            let started = Instant::now();
            match send_signed(&restarted, frame(&credential, &upstream)).await {
                Ok(200) => served += 1,
                // Fail closed when the squatter kept winning the race.
                Err(ModelCredentialBrokerError::PipeSquatted) => refused += 1,
                other => panic!("{mode:?}: unexpected outcome: {other:?}"),
            }
            assert!(started.elapsed() < NO_HANG, "{:?}", started.elapsed());
        }
        assert!(served > 0, "{mode:?}: no request reached the broker");
        let connections = squatter.stop();
        assert_only_challenges(&connections);
        eprintln!(
            "sec_390 restart race ({mode:?}): {served} served, {refused} refused as squatted; joined squatter saw {} connection(s), challenge bytes only",
            connections.len()
        );
    }
}
