use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use futures::future::BoxFuture;
use pretty_assertions::assert_eq;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use super::ExecServerClient;
use crate::ExecServerError;
use crate::NoiseChannelIdentity;
use crate::NoiseChannelPublicKey;
use crate::NoiseRendezvousConnectBundle;
use crate::NoiseRendezvousConnectProvider;

struct SequenceNoiseConnectProvider {
    bundles: Mutex<VecDeque<NoiseRendezvousConnectBundle>>,
    returned_urls: Mutex<Vec<String>>,
}

impl SequenceNoiseConnectProvider {
    fn new(bundles: Vec<NoiseRendezvousConnectBundle>) -> Self {
        Self {
            bundles: Mutex::new(bundles.into()),
            returned_urls: Mutex::new(Vec::new()),
        }
    }

    fn returned_urls(&self) -> Vec<String> {
        self.returned_urls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl NoiseRendezvousConnectProvider for SequenceNoiseConnectProvider {
    fn connect_bundle(
        &self,
        _: NoiseChannelPublicKey,
    ) -> BoxFuture<'_, Result<NoiseRendezvousConnectBundle, ExecServerError>> {
        let result = self
            .bundles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front()
            .ok_or_else(|| ExecServerError::Protocol("test Noise provider exhausted".to_string()));
        if let Ok(bundle) = &result {
            self.returned_urls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(bundle.websocket_url.clone());
        }
        Box::pin(async move { result })
    }
}

fn test_bundle(websocket_url: String) -> Result<NoiseRendezvousConnectBundle> {
    Ok(NoiseRendezvousConnectBundle {
        websocket_url,
        environment_id: "environment".to_string(),
        executor_registration_id: "registration".to_string(),
        executor_public_key: NoiseChannelIdentity::generate()?.public_key(),
        harness_key_authorization: "authorization".to_string(),
    })
}

#[tokio::test]
async fn initial_noise_connection_refreshes_bundle_after_unauthorized_handshake() -> Result<()> {
    let unauthorized_listener = TcpListener::bind("127.0.0.1:0").await?;
    let unauthorized_url = format!("ws://{}", unauthorized_listener.local_addr()?);
    let unauthorized_server = tokio::spawn(async move {
        let (mut socket, _) = unauthorized_listener.accept().await?;
        let mut request = [0_u8; 4096];
        let _ = socket.read(&mut request).await?;
        socket
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .await?;
        socket.shutdown().await?;
        anyhow::Ok(())
    });
    let accepted_listener = TcpListener::bind("127.0.0.1:0").await?;
    let accepted_url = format!("ws://{}", accepted_listener.local_addr()?);
    let accepted_server = tokio::spawn(async move {
        let (socket, _) = accepted_listener.accept().await?;
        let _websocket = accept_async(socket).await?;
        anyhow::Ok(())
    });
    let sequence = Arc::new(SequenceNoiseConnectProvider::new(vec![
        test_bundle(unauthorized_url.clone())?,
        test_bundle(accepted_url.clone())?,
    ]));
    let provider: Arc<dyn NoiseRendezvousConnectProvider> = sequence.clone();
    let identity = NoiseChannelIdentity::generate()?;

    let _connection = ExecServerClient::open_initial_noise_rendezvous_connection(
        &provider,
        &identity,
        codex_http_client::HttpClientFactory::new(
            codex_http_client::OutboundProxyPolicy::ReqwestDefault,
        ),
    )
    .await?;

    assert_eq!(
        sequence.returned_urls(),
        vec![unauthorized_url, accepted_url]
    );
    unauthorized_server.await??;
    accepted_server.await??;
    Ok(())
}

/// Runs `check` in a child test process with two host variables set, so arming
/// secretless launch (one-way, process-wide) never reaches the other tests.
fn in_secretless_launch_child(test: &str, check: impl FnOnce()) {
    const CHILD: &str = "CODEX_EXEC_SERVER_STDIO_SECRETLESS_CHILD";
    if std::env::var_os(CHILD).is_some() {
        check();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test, "--nocapture"])
        .env(CHILD, "1")
        .env("PF27_PROBE_API_TOKEN", "host-secret")
        .env("PF27_PROBE_PLAIN", "host-plain")
        .output()
        .expect("run child test");
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The environment a stdio executor started from `command`'s settings sees.
fn stdio_executor_env(
    env: std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    let (program, args) = if cfg!(windows) {
        ("cmd", vec!["/C".to_string(), "set".to_string()])
    } else {
        ("sh", vec!["-c".to_string(), "env".to_string()])
    };
    let mut command = super::stdio_command_process(&crate::client_api::StdioExecServerCommand {
        program: program.to_string(),
        args,
        env,
        cwd: None,
    });
    let output = command
        .as_std_mut()
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run stdio executor stand-in");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.to_string(), value.trim_end_matches('\r').to_string()))
        .collect()
}

/// PF-27-S02: once secretless launch is armed, an executor started over stdio
/// (`environments.toml`) gets only the launch allowlist from Corbanu's
/// environment plus its own configured `env` (without values that embed a URL
/// password); it is the base environment of
/// every command, hook and MCP server it runs.
#[test]
fn pf_27_s02_armed_stdio_executor_starts_with_the_launch_allowlist() {
    in_secretless_launch_child(
        "client_transport::tests::pf_27_s02_armed_stdio_executor_starts_with_the_launch_allowlist",
        || {
            let configured = std::collections::HashMap::from([
                ("EXECUTOR_OWN_TOKEN".to_string(), "configured".to_string()),
                (
                    "EXECUTOR_PROXY".to_string(),
                    "http://user:pw@proxy.invalid".to_string(),
                ),
            ]);
            let observe = |env: &std::collections::HashMap<String, String>| {
                (
                    env.get("PF27_PROBE_API_TOKEN").cloned(),
                    env.get("PF27_PROBE_PLAIN").cloned(),
                    env.get("EXECUTOR_OWN_TOKEN").cloned(),
                    env.contains_key("EXECUTOR_PROXY"),
                    env.keys().any(|name| name.eq_ignore_ascii_case("PATH")),
                )
            };
            let unarmed = stdio_executor_env(configured.clone());
            assert_eq!(
                observe(&unarmed),
                (
                    Some("host-secret".to_string()),
                    Some("host-plain".to_string()),
                    Some("configured".to_string()),
                    true,
                    true
                )
            );
            codex_protocol::secretless_launch::arm();
            let armed = stdio_executor_env(configured);
            assert_eq!(
                observe(&armed),
                (None, None, Some("configured".to_string()), false, true)
            );
        },
    );
}

/// A stand-in executor that answers `initialize` and then stays up.
fn fake_stdio_executor() -> crate::client_api::ExecServerTransportParams {
    let (program, args) = if cfg!(windows) {
        (
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "$null = [Console]::In.ReadLine(); [Console]::Out.WriteLine('{\"id\":1,\"result\":{\"sessionId\":\"stdio-test\"}}'); $null = [Console]::In.ReadLine(); Start-Sleep -Seconds 60".to_string(),
            ],
        )
    } else {
        (
            "sh",
            vec![
                "-c".to_string(),
                "read _line; printf '%s\\n' '{\"id\":1,\"result\":{\"sessionId\":\"stdio-test\"}}'; read _line; sleep 60".to_string(),
            ],
        )
    };
    crate::client_api::ExecServerTransportParams::StdioCommand {
        command: crate::client_api::StdioExecServerCommand {
            program: program.to_string(),
            args,
            env: std::collections::HashMap::new(),
            cwd: None,
        },
        initialize_timeout: crate::client_api::DEFAULT_REMOTE_EXEC_SERVER_INITIALIZE_TIMEOUT,
    }
}

/// PF-27-S02: a stdio executor started before secretless launch was armed (a
/// security level changed during the run) still holds Corbanu's full
/// environment and cannot be restarted in place, so it is refused from then
/// on; one started after arming is used normally.
#[test]
fn pf_27_s02_stdio_executor_started_before_arming_is_refused_after_it() {
    in_secretless_launch_child(
        "client_transport::tests::pf_27_s02_stdio_executor_started_before_arming_is_refused_after_it",
        || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");
            let http = || {
                codex_http_client::HttpClientFactory::new(
                    codex_http_client::OutboundProxyPolicy::ReqwestDefault,
                )
            };
            let early =
                crate::client::LazyRemoteExecServerClient::new(fake_stdio_executor(), http());
            runtime
                .block_on(early.get())
                .expect("an executor started unarmed is used while unarmed");
            codex_protocol::secretless_launch::arm();
            let Err(error) = runtime.block_on(early.get()) else {
                panic!("an executor started unarmed is refused once armed");
            };
            assert!(
                error.to_string().contains("restart Corbanu"),
                "unexpected error: {error}"
            );
            let late =
                crate::client::LazyRemoteExecServerClient::new(fake_stdio_executor(), http());
            runtime
                .block_on(late.get())
                .expect("an executor started armed is used");
        },
    );
}
