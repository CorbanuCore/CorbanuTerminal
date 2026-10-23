#![cfg(target_os = "linux")]

use std::net::TcpListener;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use tempfile::TempDir;

const BWRAP_UNAVAILABLE_ERR: &str = "bubblewrap is unavailable";
const MANAGED_PROXY_PERMISSION_ERR_SNIPPETS: &[&str] = &[
    "loopback: Failed RTM_NEWADDR",
    "loopback: Failed RTM_NEWLINK",
    "setting up uid map: Permission denied",
    "No permissions to create a new namespace",
    "error isolating Linux network namespace for proxy mode",
];

#[test]
fn sandbox_with_network_proxy_blocks_direct_loopback_access() -> Result<()> {
    let codex_home = TempDir::new()?;
    let listener = TcpListener::bind("127.0.0.2:0")?;
    let port = listener.local_addr()?.port();
    std::fs::write(
        codex_home.path().join("config.toml"),
        r#"
default_permissions = "network-test"

[features]
network_proxy = true
use_legacy_landlock = true

[permissions.network-test]
extends = ":workspace"

[permissions.network-test.network]
enabled = true
mode = "full"
"#,
    )?;

    let url = format!("http://127.0.0.2:{port}/");
    let output = std::process::Command::new(codex_utils_cargo_bin::cargo_bin("codex")?)
        .env("CODEX_HOME", codex_home.path())
        .args([
            "sandbox",
            "--permission-profile",
            "network-test",
            "--",
            "curl",
            "--noproxy",
            "*",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "1",
            "--max-time",
            "2",
            url.as_str(),
        ])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains(BWRAP_UNAVAILABLE_ERR) {
        eprintln!("skipping network proxy sandbox test: bubblewrap is unavailable");
        return Ok(());
    }
    if MANAGED_PROXY_PERMISSION_ERR_SNIPPETS
        .iter()
        .any(|snippet| stderr.contains(snippet))
    {
        eprintln!("skipping network proxy sandbox test: bubblewrap lacks namespace permissions");
        return Ok(());
    }

    assert_eq!(
        output.status.code(),
        Some(7),
        "expected direct loopback access to be blocked; status={:?}; stdout={}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );

    Ok(())
}

#[test]
fn sandbox_with_network_proxy_allows_explicit_loopback_access() -> Result<()> {
    let codex_home = TempDir::new()?;
    let listener = TcpListener::bind("127.0.0.2:0")?;
    let port = listener.local_addr()?.port();
    listener.set_nonblocking(true)?;
    let server = std::thread::spawn(move || -> std::io::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    // Read the request before answering: closing a socket
                    // with unread data sends a reset, which can reach the
                    // proxy before the response and turn it into a 502.
                    stream.set_nonblocking(false)?;
                    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                    read_request_head(&mut stream)?;
                    std::io::Write::write_all(
                        &mut stream,
                        b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n",
                    )?;
                    return Ok(());
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "timed out waiting for allowlisted loopback request",
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(err) => return Err(err),
            }
        }
    });
    std::fs::write(
        codex_home.path().join("config.toml"),
        r#"
default_permissions = "network-test"

[features]
network_proxy = true
use_legacy_landlock = true

[permissions.network-test]
extends = ":workspace"

[permissions.network-test.network]
enabled = true
mode = "full"
allow_local_binding = false

[permissions.network-test.network.domains]
"127.0.0.2" = "allow"
"#,
    )?;

    let url = format!("http://127.0.0.2:{port}/");
    let output = std::process::Command::new(codex_utils_cargo_bin::cargo_bin("codex")?)
        .env("CODEX_HOME", codex_home.path())
        .args([
            "sandbox",
            "--permission-profile",
            "network-test",
            "--",
            "curl",
            "--fail",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "2",
            "--max-time",
            "4",
            url.as_str(),
        ])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains(BWRAP_UNAVAILABLE_ERR) {
        eprintln!("skipping network proxy sandbox test: bubblewrap is unavailable");
        return Ok(());
    }

    assert!(
        output.status.success(),
        "expected allowlisted loopback access to succeed; status={:?}; stdout={}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    server.join().expect("loopback server panicked")?;

    Ok(())
}

/// Reads an HTTP request up to the end of its head (the test's requests have
/// no body).
fn read_request_head(stream: &mut std::net::TcpStream) -> std::io::Result<()> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = std::io::Read::read(stream, &mut buffer)?;
        if read == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..read]);
    }
    Ok(())
}
