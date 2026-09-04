use super::*;
use codex_uds::UnixListener;
use codex_uds::prepare_private_socket_directory;

async fn mock_daemon(
    home: &Path,
    protocol_response: &'static str,
) -> tokio::task::JoinHandle<Vec<serde_json::Value>> {
    tokio::fs::create_dir_all(home.join("wallet"))
        .await
        .unwrap();
    prepare_private_socket_directory(run_dir(home))
        .await
        .unwrap();
    let mut listener = UnixListener::bind(socket_path(home)).await.unwrap();
    tokio::spawn(async move {
        let mut requests = Vec::new();
        for response in ["{\"type\":\"pong\"}", protocol_response] {
            let stream = listener.accept().await.unwrap();
            let (read, mut write) = tokio::io::split(stream);
            let mut line = String::new();
            BufReader::new(read).read_line(&mut line).await.unwrap();
            requests.push(serde_json::from_str(&line).unwrap());
            write
                .write_all(format!("{response}\n").as_bytes())
                .await
                .unwrap();
        }
        requests
    })
}

#[tokio::test]
async fn legacy_and_incompatible_daemons_are_rejected_before_secrets_or_operations_are_sent() {
    for response in [
        r#"{"type":"error","code":"invalid_request","message":"request was malformed"}"#,
        r#"{"type":"protocol_version","version":0}"#,
        r#"{"type":"protocol_version","version":4294967295}"#,
    ] {
        for unlock in [true, false] {
            let home = tempfile::tempdir().unwrap();
            let server = mock_daemon(home.path(), response).await;
            let client = WalletDaemonClient::new(home.path().to_path_buf());
            let error = if unlock {
                client
                    .unlock("passcode-canary".into(), UnlockPolicy::OneAction)
                    .await
                    .unwrap_err()
            } else {
                client
                    .execute_corbanu_api_operation(
                        "capability-canary".into(),
                        "https://api.corbanu.example".into(),
                        CorbanuApiOperation::CreateKey,
                    )
                    .await
                    .unwrap_err()
            };
            assert!(
                matches!(&error, WalletDaemonError::Refused { code, .. } if code == "daemon_upgrade_required")
            );
            let message = error.to_string();
            assert!(message.contains("Let any existing payment finish"));
            assert!(message.contains("Restarting only the TUI does not stop the daemon"));
            assert!(!message.contains("canary"));
            assert_eq!(
                server.await.unwrap(),
                vec![
                    serde_json::json!({"type": "ping"}),
                    serde_json::json!({"type": "protocol_version"}),
                ]
            );
        }
    }
}

#[tokio::test]
async fn matching_daemon_protocol_is_accepted() {
    let home = tempfile::tempdir().unwrap();
    let server = mock_daemon(home.path(), r#"{"type":"protocol_version","version":1}"#).await;
    WalletDaemonClient::new(home.path().to_path_buf())
        .ensure_running()
        .await
        .unwrap();
    assert_eq!(server.await.unwrap().len(), 2);
}

#[tokio::test]
async fn legacy_daemon_can_still_be_locked_without_protocol_negotiation() {
    let home = tempfile::tempdir().unwrap();
    let server = mock_daemon(home.path(), r#"{"type":"locked"}"#).await;
    WalletDaemonClient::new(home.path().to_path_buf())
        .lock()
        .await
        .unwrap();
    assert_eq!(
        server.await.unwrap()[1],
        serde_json::json!({"type": "lock"})
    );
}
