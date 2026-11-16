#![cfg(windows)]
//! Windows versions of the two #2196 local stdio shutdown tests in `process_group_cleanup.rs`,
//! which is Unix-only. The server runs under a `cmd.exe` wrapper, so it is a descendant of the
//! launched process, as it is for `npx.cmd` and similar launchers. The launcher puts the wrapper
//! in a job object, and stopping that job must stop the server before `shutdown()` returns.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::FromRawHandle;
use std::os::windows::io::OwnedHandle;
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
use tracing_test::traced_test;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(
        desired_access: u32,
        inherit_handle: i32,
        process_id: u32,
    ) -> *mut std::ffi::c_void;
    fn WaitForSingleObject(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
}

const SYNCHRONIZE: u32 = 0x0010_0000;
const WAIT_OBJECT_0: u32 = 0;

/// Far past anything these tests wait for; a step that reaches it hung.
const HANG_WAIT: Duration = Duration::from_secs(300);

/// Bounds each step that must not hang.
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Opens the server before shutdown, so its exit can't be confused with a reused PID.
fn open_process(process_id: u32) -> Result<OwnedHandle> {
    let handle = unsafe {
        OpenProcess(SYNCHRONIZE, /*inherit_handle*/ 0, process_id)
    };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("failed to open MCP server process {process_id}"));
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(handle.cast()) })
}

fn has_exited(process: &OwnedHandle) -> bool {
    let state = unsafe {
        WaitForSingleObject(process.as_raw_handle().cast(), /*milliseconds*/ 0)
    };
    state == WAIT_OBJECT_0
}

fn init_params() -> InitializeRequestParams {
    InitializeRequestParams::new(
        ClientCapabilities::default(),
        Implementation::new("codex-test", "0.0.0-test").with_title("Codex rmcp shutdown test"),
    )
    .with_protocol_version(ProtocolVersion::V_2025_06_18)
}

/// An opted-in 2026-07-28 client on a local `test_mcp_2026_stdio_server` in `mode`, launched
/// through `launch.cmd`. The server logs its PID to `server.pids` and the requests it receives to
/// `requests.log`, both in `dir`.
async fn opted_in_stdio_client(dir: &Path, mode: &str) -> Result<RmcpClient> {
    let server = codex_utils_cargo_bin::cargo_bin("test_mcp_2026_stdio_server")?;
    fs::write(
        dir.join("launch.cmd"),
        "@echo off\r\n\"%MCP_TEST_SERVER_BINARY%\" %MCP_TEST_MODE%\r\n",
    )?;
    let client = RmcpClient::new_stdio_client_with_protocol_mode(
        OsString::from("cmd.exe"),
        ["/d", "/s", "/c", "call", ".\\launch.cmd"]
            .map(OsString::from)
            .to_vec(),
        Some(HashMap::from([
            (
                OsString::from("CODEX_MCP_PROTOCOL_VERSION"),
                OsString::from("2026-07-28"),
            ),
            (
                OsString::from("MCP_TEST_PID_LOG"),
                dir.join("server.pids").into_os_string(),
            ),
            (
                OsString::from("MCP_TEST_REQUEST_LOG"),
                dir.join("requests.log").into_os_string(),
            ),
            (
                OsString::from("MCP_TEST_SERVER_BINARY"),
                server.into_os_string(),
            ),
            (OsString::from("MCP_TEST_MODE"), OsString::from(mode)),
        ])),
        &[],
        Some(dir.to_string_lossy().into_owned()),
        Arc::new(LocalStdioServerLauncher::new(dir.to_path_buf())),
        McpProtocolMode::V20260728,
    )
    .await?;
    Ok(client)
}

fn server_pids(dir: &Path) -> Result<Vec<u32>> {
    fs::read_to_string(dir.join("server.pids"))?
        .lines()
        .map(|line| line.trim().parse::<u32>().map_err(Into::into))
        .collect()
}

async fn wait_for_file_containing(path: &Path, needle: &str) -> Result<()> {
    tokio::time::timeout(TEST_TIMEOUT, async {
        loop {
            match fs::read_to_string(path) {
                Ok(content) if content.contains(needle) => return Ok(()),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to read {}", path.display()));
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .with_context(|| format!("timed out waiting for {needle:?} in {}", path.display()))?
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

// #2196 review: the Windows version of
// `process_group_cleanup::opted_in_shutdown_during_the_discover_wait_stops_the_stdio_server`.
#[expect(
    clippy::disallowed_methods,
    reason = "traced_test installs an in-memory subscriber to check that shutdown logs no warning"
)]
#[traced_test]
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
    let server = open_process(server_pid)?;
    assert!(
        !has_exited(&server),
        "expected MCP server process {server_pid} to be running during the discover wait"
    );

    tokio::time::timeout(TEST_TIMEOUT, client.shutdown())
        .await
        .context("shutdown during the discover wait hung")?;
    assert!(
        has_exited(&server),
        "shutdown returned before the job stopped MCP server process {server_pid}"
    );
    let result = tokio::time::timeout(TEST_TIMEOUT, initializing)
        .await
        .context("initialize kept waiting for discover after shutdown")??;

    assert!(
        result.is_err(),
        "initialize must fail once the client is shut down"
    );
    assert!(!logs_contain(
        "failed to terminate MCP stdio server process"
    ));
    assert_eq!(
        server_pids(temp_dir.path())?,
        vec![server_pid],
        "shutdown must not start a fallback server"
    );
    Ok(())
}

// #2196 review: the Windows version of
// `process_group_cleanup::opted_in_shutdown_after_a_modern_start_stops_the_stdio_server`.
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
    let server = open_process(server_pid)?;
    assert!(
        !has_exited(&server),
        "expected MCP server process {server_pid} to be running before shutdown"
    );

    tokio::time::timeout(TEST_TIMEOUT, client.shutdown())
        .await
        .context("shutdown after a modern start hung")?;

    assert!(
        has_exited(&server),
        "shutdown returned before the job stopped MCP server process {server_pid}"
    );
    assert!(!logs_contain(
        "failed to terminate MCP stdio server process"
    ));
    assert_eq!(server_pids(temp_dir.path())?, vec![server_pid]);
    Ok(())
}
