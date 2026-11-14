#![cfg(unix)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use codex_rmcp_client::ElicitationAction;
use codex_rmcp_client::ElicitationResponse;
use codex_rmcp_client::LocalStdioServerLauncher;
use codex_rmcp_client::McpProtocolMode;
use codex_rmcp_client::RmcpClient;
use futures::FutureExt as _;
use rmcp::model::ClientCapabilities;
use rmcp::model::Implementation;
use rmcp::model::InitializeRequestParams;
use rmcp::model::ProtocolVersion;
use serde_json::json;
use tracing_test::traced_test;

fn stdio_server_bin() -> Result<std::path::PathBuf> {
    codex_utils_cargo_bin::cargo_bin("test_stdio_server").map_err(Into::into)
}

fn init_params() -> InitializeRequestParams {
    InitializeRequestParams::new(
        ClientCapabilities::default(),
        Implementation::new("codex-test", "0.0.0-test").with_title("Codex rmcp shutdown test"),
    )
    .with_protocol_version(ProtocolVersion::V_2025_06_18)
}

fn process_exists(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

async fn wait_for_pid_file(path: &Path) -> Result<u32> {
    for _ in 0..50 {
        match fs::read_to_string(path) {
            Ok(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                let pid = trimmed
                    .parse::<u32>()
                    .with_context(|| format!("failed to parse pid from {}", path.display()))?;
                return Ok(pid);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => {
                return Err(error).with_context(|| format!("failed to read {}", path.display()));
            }
        }
    }

    anyhow::bail!("timed out waiting for child pid file at {}", path.display());
}

async fn wait_for_process_exit(pid: u32) -> Result<()> {
    for _ in 0..50 {
        if !process_exists(pid) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("process {pid} still running after timeout");
}

async fn wait_for_file_containing(path: &Path, needle: &str) -> Result<()> {
    for _ in 0..50 {
        match fs::read_to_string(path) {
            Ok(content) if content.contains(needle) => return Ok(()),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("failed to read {}", path.display()));
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("timed out waiting for {needle:?} in {}", path.display());
}

/// Far past anything these tests wait for; a step that reaches it hung.
const HANG_WAIT: Duration = Duration::from_secs(300);

/// Bounds each step that must not hang.
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// An opted-in 2026-07-28 client on a local `test_mcp_2026_stdio_server` in `mode`. The wrapper
/// logs its PID (the server's process group) to `server.pids` and the requests the server
/// receives to `requests.log`, both in `dir`.
async fn opted_in_stdio_client(dir: &Path, mode: &str) -> Result<RmcpClient> {
    let server = codex_utils_cargo_bin::cargo_bin("test_mcp_2026_stdio_server")?;
    let client = RmcpClient::new_stdio_client_with_protocol_mode(
        OsString::from("/bin/sh"),
        vec![
            OsString::from("-c"),
            OsString::from(
                "echo $$ >> \"$PID_LOG\"; tee -a \"$REQUEST_LOG\" | \"$SERVER\" \"$MODE\"",
            ),
        ],
        Some(HashMap::from([
            (
                OsString::from("CODEX_MCP_PROTOCOL_VERSION"),
                OsString::from("2026-07-28"),
            ),
            (
                OsString::from("PID_LOG"),
                dir.join("server.pids").into_os_string(),
            ),
            (
                OsString::from("REQUEST_LOG"),
                dir.join("requests.log").into_os_string(),
            ),
            (OsString::from("SERVER"), server.into_os_string()),
            (OsString::from("MODE"), OsString::from(mode)),
        ])),
        &[],
        /*cwd*/ None,
        Arc::new(LocalStdioServerLauncher::new(std::env::current_dir()?)),
        McpProtocolMode::V20260728,
    )
    .await?;
    Ok(client)
}

fn server_pids(dir: &Path) -> Result<Vec<u32>> {
    fs::read_to_string(dir.join("server.pids"))?
        .lines()
        .map(|line| line.parse::<u32>().map_err(Into::into))
        .collect()
}

async fn initialize_declining_elicitations(
    client: &RmcpClient,
    timeout: Duration,
) -> Result<rmcp::model::ServerPeerInfo> {
    client
        .initialize(
            init_params(),
            Some(timeout),
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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn drop_kills_wrapper_process_group() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let child_pid_file = temp_dir.path().join("child.pid");
    let child_pid_file_str = child_pid_file.to_string_lossy().into_owned();
    let wrapper = temp_dir.path().join("wrapper");
    codex_utils_cargo_bin::write_executable(
        &wrapper,
        "#!/bin/sh\nsleep 300 & child_pid=$!; echo \"$child_pid\" > \"$CHILD_PID_FILE\"; cat >/dev/null\n",
    )?;

    let client = RmcpClient::new_stdio_client(
        OsString::from("./wrapper"),
        vec![],
        Some(HashMap::from([(
            OsString::from("CHILD_PID_FILE"),
            OsString::from(child_pid_file_str),
        )])),
        &[],
        Some(temp_dir.path().to_string_lossy().into_owned()),
        Arc::new(LocalStdioServerLauncher::new(std::env::current_dir()?)),
    )
    .await?;

    let grandchild_pid = wait_for_pid_file(&child_pid_file).await?;
    assert!(
        process_exists(grandchild_pid),
        "expected grandchild process {grandchild_pid} to be running before dropping client"
    );

    drop(client);

    wait_for_process_exit(grandchild_pid).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_kills_initialized_stdio_server_with_in_flight_operation() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let server_pid_file = temp_dir.path().join("server.pid");
    let server_pid_file_str = server_pid_file.to_string_lossy().into_owned();

    let client = Arc::new(
        RmcpClient::new_stdio_client(
            stdio_server_bin()?.into(),
            Vec::<OsString>::new(),
            Some(HashMap::from([(
                OsString::from("MCP_TEST_PID_FILE"),
                OsString::from(server_pid_file_str),
            )])),
            &[],
            /*cwd*/ None,
            Arc::new(LocalStdioServerLauncher::new(std::env::current_dir()?)),
        )
        .await?,
    );

    client
        .initialize(
            init_params(),
            Some(Duration::from_secs(5)),
            Box::new(|_, _| {
                async {
                    Ok(ElicitationResponse {
                        action: ElicitationAction::Accept,
                        content: Some(json!({})),
                        meta: None,
                    })
                }
                .boxed()
            }),
        )
        .await?;

    let server_pid = wait_for_pid_file(&server_pid_file).await?;
    assert!(
        process_exists(server_pid),
        "expected MCP server process {server_pid} to be running before shutdown"
    );

    let call_client = Arc::clone(&client);
    let call_task = tokio::spawn(async move {
        call_client
            .call_tool(
                "sync".to_string(),
                Some(json!({ "sleep_after_ms": 300_000 })),
                |_| Ok(None),
                Some(Duration::from_secs(300)),
            )
            .await
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    client.shutdown().await;

    wait_for_process_exit(server_pid).await?;
    let _ = tokio::time::timeout(Duration::from_secs(5), call_task).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn opted_in_legacy_fallback_leaves_one_stdio_server_process() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let pid_log = temp_dir.path().join("server.pids");
    let overlap_log = temp_dir.path().join("overlap.pids");
    let server = codex_utils_cargo_bin::cargo_bin("test_mcp_2026_stdio_server")?;

    let client = RmcpClient::new_stdio_client_with_protocol_mode(
        OsString::from("/bin/sh"),
        vec![
            OsString::from("-c"),
            OsString::from(
                "if [ -s \"$PID_LOG\" ] && kill -0 \"$(head -n 1 \"$PID_LOG\")\" 2>/dev/null; \
                 then head -n 1 \"$PID_LOG\" >> \"$OVERLAP_LOG\"; fi; \
                 echo $$ >> \"$PID_LOG\"; exec \"$SERVER\" legacy-fallback",
            ),
        ],
        Some(HashMap::from([
            (
                OsString::from("CODEX_MCP_PROTOCOL_VERSION"),
                OsString::from("2026-07-28"),
            ),
            (OsString::from("PID_LOG"), pid_log.clone().into_os_string()),
            (
                OsString::from("OVERLAP_LOG"),
                overlap_log.clone().into_os_string(),
            ),
            (OsString::from("SERVER"), server.into_os_string()),
        ])),
        &[],
        /*cwd*/ None,
        Arc::new(LocalStdioServerLauncher::new(std::env::current_dir()?)),
        McpProtocolMode::V20260728,
    )
    .await?;

    client
        .initialize(
            init_params(),
            Some(Duration::from_secs(5)),
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
        .await?;

    let pids = fs::read_to_string(&pid_log)?
        .lines()
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(
        pids.len(),
        2,
        "discover and the legacy initialize each start one server: {pids:?}"
    );
    assert!(
        !overlap_log.exists(),
        "the fallback server started while the first server was still running: {:?}",
        fs::read_to_string(&overlap_log)
    );
    wait_for_process_exit(pids[0]).await?;
    assert!(
        process_exists(pids[1]),
        "expected the legacy MCP server process {} to keep running",
        pids[1]
    );

    client.shutdown().await;
    wait_for_process_exit(pids[1]).await
}

// #2101 review: a `shutdown()` while a local server hasn't answered `server/discover` stops that
// server, ends startup without waiting out the discover timeout, and starts no fallback server.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn opted_in_shutdown_during_the_discover_wait_stops_the_stdio_server() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    // The server never answers discover; startup could only end on its own after `HANG_WAIT`.
    let client = Arc::new(
        opted_in_stdio_client(temp_dir.path(), "legacy-silent")
            .await?
            .with_discover_timeout(Some(HANG_WAIT)),
    );
    let initializing = tokio::spawn({
        let client = Arc::clone(&client);
        async move { initialize_declining_elicitations(&client, HANG_WAIT).await }
    });
    wait_for_file_containing(&temp_dir.path().join("requests.log"), "server/discover").await?;
    let server_pid = server_pids(temp_dir.path())?[0];
    assert!(
        process_exists(server_pid),
        "expected MCP server process {server_pid} to be running during the discover wait"
    );

    tokio::time::timeout(TEST_TIMEOUT, client.shutdown())
        .await
        .context("shutdown during the discover wait hung")?;
    let result = tokio::time::timeout(TEST_TIMEOUT, initializing)
        .await
        .context("initialize kept waiting for discover after shutdown")??;

    assert!(
        result.is_err(),
        "initialize must fail once the client is shut down"
    );
    wait_for_process_exit(server_pid).await?;
    assert_eq!(
        server_pids(temp_dir.path())?,
        vec![server_pid],
        "shutdown must not start a fallback server"
    );
    Ok(())
}

// #2101 review: a local 2026-07-28 server is stopped by its discovery cleanup, not by the
// transport. After a successful modern start, `shutdown()` must still stop it, without a warning.
#[expect(
    clippy::disallowed_methods,
    reason = "traced_test installs an in-memory subscriber to check that shutdown logs no warning"
)]
#[traced_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn opted_in_shutdown_after_a_modern_start_stops_the_stdio_server() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let client = opted_in_stdio_client(temp_dir.path(), "modern").await?;
    let info = tokio::time::timeout(
        TEST_TIMEOUT,
        initialize_declining_elicitations(&client, TEST_TIMEOUT),
    )
    .await
    .context("the modern start hung")??;
    assert_eq!(
        info.server_info.as_ref().map(|info| info.name.as_str()),
        Some("strict-stdio-test")
    );
    let server_pid = server_pids(temp_dir.path())?[0];
    assert!(
        process_exists(server_pid),
        "expected MCP server process {server_pid} to be running before shutdown"
    );

    tokio::time::timeout(TEST_TIMEOUT, client.shutdown())
        .await
        .context("shutdown after a modern start hung")?;

    wait_for_process_exit(server_pid).await?;
    assert!(!logs_contain(
        "failed to terminate MCP stdio server process"
    ));
    assert_eq!(server_pids(temp_dir.path())?, vec![server_pid]);
    Ok(())
}
