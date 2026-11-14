//! #2050 (D510): a file access under a protected level (its context carries
//! `unsandboxed_refusal`) reaches a remote executor only when that executor
//! reports a platform sandbox for it; a Windows executor without one, or an
//! older one that cannot say, is refused here, before any file request.

use std::sync::Mutex as StdMutex;

use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
use futures::SinkExt;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use super::*;
use crate::Environment;

const REFUSAL: &str = "refused for #2050";

/// A fake executor answering `environment/info` with `info` and every file
/// request with an empty file; returns its URL and the methods it was sent.
async fn fake_executor(info: Value) -> (String, Arc<StdMutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let url = format!("ws://{}", listener.local_addr().expect("addr"));
    let methods = Arc::new(StdMutex::new(Vec::new()));
    let seen = Arc::clone(&methods);
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut websocket = accept_async(stream).await.expect("handshake");
        while let Some(Ok(message)) = websocket.next().await {
            let request: Value = match message {
                Message::Text(text) => serde_json::from_str(text.as_ref()).expect("json"),
                Message::Binary(bytes) => serde_json::from_slice(bytes.as_ref()).expect("json"),
                _ => continue,
            };
            let method = request["method"].as_str().unwrap_or_default().to_string();
            seen.lock().expect("methods").push(method.clone());
            let Some(id) = request.get("id") else {
                continue;
            };
            let result = match method.as_str() {
                "initialize" => json!({ "sessionId": "fake" }),
                "environment/info" => info.clone(),
                _ => json!({ "dataBase64": "" }),
            };
            let reply = json!({ "id": id, "result": result }).to_string();
            if websocket.send(Message::Text(reply.into())).await.is_err() {
                break;
            }
        }
    });
    (url, methods)
}

fn file_requests(methods: &StdMutex<Vec<String>>) -> Vec<String> {
    methods
        .lock()
        .expect("methods")
        .iter()
        .filter(|method| method.starts_with("fs/"))
        .cloned()
        .collect()
}

async fn read(
    info: Value,
    level: WindowsSandboxLevel,
    refusal: bool,
) -> (io::Result<()>, Vec<String>) {
    let (url, methods) = fake_executor(info).await;
    let environment = Environment::create_for_tests(Some(url)).expect("environment");
    let mut context = FileSystemSandboxContext::from_permission_profile(
        PermissionProfile::Disabled,
        PathUri::parse("file:///C:/work").expect("cwd uri"),
    );
    context.windows_sandbox_selection = level.into();
    context.unsandboxed_refusal = refusal.then(|| REFUSAL.to_string());
    let path = PathUri::parse("file:///C:/work/secret.txt").expect("uri");
    let result = environment
        .get_filesystem()
        .read_file(&path, ReadFileOptions::default(), Some(&context))
        .await
        .map(|_| ());
    (result, file_requests(&methods))
}

fn windows(set_up: bool) -> Value {
    json!({
        "shell": { "name": "powershell", "path": "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe" },
        "cwd": "file:///C:/work",
        "hostSandbox": { "os": "windows", "windowsSandboxSetUp": set_up },
    })
}

fn assert_refused(result: io::Result<()>, requests: Vec<String>) {
    let error = result.expect_err("refused before reaching the executor");
    assert_eq!(
        (error.kind(), error.to_string(), requests),
        (
            io::ErrorKind::PermissionDenied,
            REFUSAL.to_string(),
            Vec::new()
        )
    );
}

#[tokio::test]
async fn issue_2050_windows_executor_without_its_sandbox_is_refused_here() {
    // No Windows sandbox for this session, or the elevated one not set up.
    let (result, requests) = read(
        windows(/*set_up*/ true),
        WindowsSandboxLevel::Disabled,
        /*refusal*/ true,
    )
    .await;
    assert_refused(result, requests);
    let (result, requests) = read(
        windows(/*set_up*/ false),
        WindowsSandboxLevel::Elevated,
        /*refusal*/ true,
    )
    .await;
    assert_refused(result, requests);
    // The elevated sandbox set up there, or the non-admin one: served there.
    for (info, level) in [
        (windows(/*set_up*/ true), WindowsSandboxLevel::Elevated),
        (
            windows(/*set_up*/ false),
            WindowsSandboxLevel::RestrictedToken,
        ),
    ] {
        let (result, requests) = read(info, level, /*refusal*/ true).await;
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(requests, vec!["fs/readFile".to_string()]);
    }
}

#[tokio::test]
async fn issue_2050_older_executors_fail_closed_unless_clearly_posix() {
    // An older Windows executor reports no sandbox: refused.
    let older_windows = json!({
        "shell": { "name": "powershell", "path": "powershell.exe" },
        "cwd": "file:///C:/work",
    });
    let (result, requests) = read(
        older_windows,
        WindowsSandboxLevel::Elevated,
        /*refusal*/ true,
    )
    .await;
    assert_refused(result, requests);
    // An older Linux executor keeps working (it re-checks on its side).
    let older_linux =
        json!({ "shell": { "name": "bash", "path": "/bin/bash" }, "cwd": "file:///home/me" });
    let (result, requests) = read(
        older_linux,
        WindowsSandboxLevel::Disabled,
        /*refusal*/ true,
    )
    .await;
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(requests, vec!["fs/readFile".to_string()]);
}

/// Permissive (no refusal in the context) is unchanged: no report is asked
/// for and the request goes through.
#[tokio::test]
async fn issue_2050_permissive_is_unchanged() {
    let (url, methods) = fake_executor(windows(/*set_up*/ false)).await;
    let environment = Environment::create_for_tests(Some(url)).expect("environment");
    let context = FileSystemSandboxContext::from_permission_profile(
        PermissionProfile::Disabled,
        PathUri::parse("file:///C:/work").expect("cwd uri"),
    );
    let path = PathUri::parse("file:///C:/work/secret.txt").expect("uri");
    environment
        .get_filesystem()
        .read_file(&path, ReadFileOptions::default(), Some(&context))
        .await
        .expect("served");
    let methods = methods.lock().expect("methods").clone();
    assert!(
        !methods.contains(&"environment/info".to_string()),
        "{methods:?}"
    );
    assert!(methods.contains(&"fs/readFile".to_string()), "{methods:?}");
}
