//! Issue #380: `RUST_LOG=trace` never writes the provider key to
//! `corbanu exec`'s stderr or to its home (logs database included), on the
//! Responses websocket or over HTTP.
#![cfg(not(target_os = "windows"))]

use std::process::Output;

use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::responses::start_websocket_server;
use core_test_support::test_codex_exec::TestCodexExecBuilder;
use core_test_support::test_codex_exec::test_codex_exec;
use pretty_assertions::assert_eq;

const KEY_ENV: &str = "CORBANU_380_PROVIDER_KEY";
/// A user's `RUST_LOG` that names the HTTP and websocket libraries too.
const RUST_LOG_TRACE: &str =
    "trace,tungstenite=trace,tokio_tungstenite=trace,hyper=trace,h2=trace,reqwest=trace";

fn run_exec(test: &TestCodexExecBuilder, base_url: &str, key: &str, websockets: bool) -> Output {
    test.cmd()
        .env("RUST_LOG", RUST_LOG_TRACE)
        .env(KEY_ENV, key)
        .arg("--skip-git-repo-check")
        .arg("-c")
        .arg(format!(
            "model_providers.trace_mock={{name=\"trace mock\",base_url={base_url:?},wire_api=\"responses\",env_key=\"{KEY_ENV}\",supports_websockets={websockets}}}"
        ))
        .arg("-c")
        .arg("model_provider=\"trace_mock\"")
        .arg("say hi")
        .output()
        .expect("run codex-exec")
}

/// Fails if `key` is in stderr or in any file under the exec home.
fn assert_key_not_logged(test: &TestCodexExecBuilder, output: &Output, key: &str) {
    assert!(output.status.success(), "exec failed: {output:?}");
    let contains = |bytes: &[u8]| {
        bytes
            .windows(key.len())
            .any(|window| window == key.as_bytes())
    };
    assert!(!contains(&output.stderr), "provider key leaked into stderr");
    assert!(!contains(&output.stdout), "provider key leaked into stdout");
    for entry in walkdir::WalkDir::new(test.home_path()) {
        let entry = entry.expect("walk exec home");
        if entry.file_type().is_file() {
            let bytes = std::fs::read(entry.path()).expect("read home file");
            assert!(
                !contains(&bytes),
                "provider key leaked into {}",
                entry.path().display()
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn websocket_turn_keeps_the_key_out_of_trace_logs() {
    const KEY: &str = "fake-exec-ws-key-380-0003-91c4d2ab";
    let turn = vec![
        ev_response_created("resp-1"),
        ev_assistant_message("msg-1", "hi"),
        ev_completed("resp-1"),
    ];
    // A startup prewarm, if any, then the turn.
    let server = start_websocket_server(vec![vec![turn.clone(), turn]]).await;
    let test = test_codex_exec();
    let output = run_exec(&test, &format!("{}/v1", server.uri()), KEY, true);

    assert_eq!(
        server
            .handshakes()
            .iter()
            .map(|handshake| handshake.header("authorization"))
            .collect::<Vec<_>>(),
        vec![Some(format!("Bearer {KEY}"))]
    );
    server.shutdown().await;
    assert_key_not_logged(&test, &output, KEY);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // tungstenite's records reach stderr; only its TRACE is dropped.
    assert!(
        stderr.contains("Client handshake done."),
        "tungstenite's DEBUG records should reach stderr"
    );
    assert!(
        !stderr
            .lines()
            .any(|line| line.contains("tungstenite::handshake::client") && line.contains("Request")),
        "tungstenite's handshake request reached stderr"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_turn_keeps_the_key_out_of_trace_logs() {
    const KEY: &str = "fake-exec-http-key-380-0004-5f7e18c0";
    let server = start_mock_server().await;
    let response = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_assistant_message("msg-1", "hi"),
            ev_completed("resp-1"),
        ]),
    )
    .await;
    let test = test_codex_exec();
    let output = run_exec(&test, &format!("{}/v1", server.uri()), KEY, false);

    assert_eq!(
        response.single_request().header("authorization"),
        Some(format!("Bearer {KEY}"))
    );
    assert_key_not_logged(&test, &output, KEY);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("reqwest::"),
        "reqwest's records should reach stderr"
    );
}
