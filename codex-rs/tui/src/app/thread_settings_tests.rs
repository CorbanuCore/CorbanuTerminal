use super::*;
use crate::app::tests::make_test_app_with_channels;
use crate::app_event::AppEvent;
use futures::SinkExt;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::json;

/// A remote server that can reject the confirmed request yet accept an
/// unconfirmed retry, reproducing the hidden-refusal path without a race.
async fn reply_server(
    reply: serde_json::Value,
) -> color_eyre::Result<(
    AppServerSession,
    tokio::task::JoinHandle<Vec<serde_json::Value>>,
)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let task = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut socket = tokio_tungstenite::accept_async(stream)
            .await
            .expect("websocket");
        let mut requests = Vec::new();
        while let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) =
            socket.next().await
        {
            let request: serde_json::Value = serde_json::from_str(&text).expect("request");
            if request.get("id").is_none() {
                continue;
            }
            let mut response = if request["method"] == "initialize" {
                json!({"result": {
                    "userAgent": "issue-2062",
                    "codexHome": std::env::temp_dir(),
                    "platformFamily": std::env::consts::FAMILY,
                    "platformOs": std::env::consts::OS
                }})
            } else {
                assert_eq!(request["method"], "thread/settings/update");
                requests.push(request["params"].clone());
                if request["params"]["confirm"] == true {
                    reply.clone()
                } else {
                    json!({"result": {}})
                }
            };
            response["id"] = request["id"].clone();
            socket
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    response.to_string().into(),
                ))
                .await
                .expect("response");
        }
        requests
    });
    let client = crate::connect_remote_app_server(
        codex_app_server_client::RemoteAppServerEndpoint::WebSocket {
            websocket_url: format!("ws://{addr}"),
            auth_token: None,
        },
    )
    .await?;
    Ok((
        AppServerSession::new(client, crate::app_server_session::ThreadParamsMode::Remote),
        task,
    ))
}

#[tokio::test]
async fn issue_2062_sandbox_replies_require_confirmation_and_never_retry() -> color_eyre::Result<()>
{
    for (case, reply, expected) in [
        (
            "applied",
            json!({"result": {"outcome": "applied"}}),
            SettingsUpdateOutcome::Confirmed,
        ),
        (
            "rejected",
            json!({"error": {"code": -32600, "message": "Core rejected weaker sandbox"}}),
            SettingsUpdateOutcome::Failed,
        ),
        (
            "uncertain",
            json!({"result": {"outcome": "uncertain"}}),
            SettingsUpdateOutcome::Accepted,
        ),
        (
            "accepted",
            json!({"result": {}}),
            SettingsUpdateOutcome::Accepted,
        ),
        (
            "unsupported",
            json!({"error": {"code": -32601, "message": "unknown method"}}),
            SettingsUpdateOutcome::Accepted,
        ),
        (
            "transport failure",
            json!({"error": {"code": -32603, "message": "lost completion"}}),
            SettingsUpdateOutcome::Failed,
        ),
    ] {
        let (mut app, _rx, _ops) = Box::pin(make_test_app_with_channels()).await;
        let (mut server, task) = reply_server(reply).await?;
        let outcome = app
            .send_thread_settings_update(
                &mut server,
                ThreadSettingsUpdateParams {
                    thread_id: ThreadId::new().to_string(),
                    refresh_windows_sandbox: true,
                    confirm: true,
                    ..Default::default()
                },
            )
            .await;
        server.shutdown().await?;
        let requests = task.await?;
        assert_eq!(outcome, expected, "{case}");
        assert_eq!(requests.len(), 1, "{case}: must not retry");
        assert_eq!(requests[0]["confirm"], true);
    }
    Ok(())
}

#[tokio::test]
async fn issue_2062_sandbox_confirmation_failure_from_embedded_server_is_preserved()
-> color_eyre::Result<()> {
    let (mut app, _rx, _ops) = Box::pin(make_test_app_with_channels()).await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let started = server.start_thread(&app.config).await?;
    let thread_id = started.session.thread_id;
    server.thread_unsubscribe(thread_id).await?;
    // Unsubscribe alone leaves the server listener alive. Use a nonexistent
    // thread to exercise a genuine rejection from the embedded server; the
    // controlled remote-server cases above also verify that it is not retried.
    let missing_thread_id = ThreadId::new();
    let outcome = app
        .send_thread_settings_update(
            &mut server,
            ThreadSettingsUpdateParams {
                thread_id: missing_thread_id.to_string(),
                refresh_windows_sandbox: true,
                confirm: true,
                ..Default::default()
            },
        )
        .await;
    server.shutdown().await?;
    assert_eq!(outcome, SettingsUpdateOutcome::Failed);
    Ok(())
}

#[tokio::test]
async fn issue_2062_disabled_sandbox_confirmation_does_not_claim_readiness()
-> color_eyre::Result<()> {
    let (mut app, mut rx, _ops) = Box::pin(make_test_app_with_channels()).await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let started = server.start_thread(&app.config).await?;
    let thread_id = started.session.thread_id;
    // A confirmed refresh can leave sandboxing disabled: it must not claim
    // that a sandbox is ready merely because the request was acknowledged.
    let op = sandbox_refresh_op(codex_protocol::config_types::WindowsSandboxLevel::Disabled);
    while rx.try_recv().is_ok() {}
    app.sync_override_turn_context_settings(&mut server, thread_id, &op)
        .await;
    server.shutdown().await?;
    let text = history_text(&mut rx);
    assert!(!text.contains("Sandbox ready"), "{text}");
    assert!(text.contains("settings confirmed"), "{text}");
    insta::assert_snapshot!("issue_2062_sandbox_disabled", text);
    Ok(())
}

#[tokio::test]
async fn issue_2062_sandbox_success_waits_for_confirmation() -> color_eyre::Result<()> {
    let (mut app, mut rx, _ops) = Box::pin(make_test_app_with_channels()).await;
    let (mut server, task) = reply_server(json!({"result": {"outcome": "applied"}})).await?;
    while rx.try_recv().is_ok() {}
    app.sync_override_turn_context_settings(
        &mut server,
        ThreadId::new(),
        &sandbox_refresh_op(codex_protocol::config_types::WindowsSandboxLevel::Elevated),
    )
    .await;
    server.shutdown().await?;
    assert_eq!(task.await?.len(), 1);
    let text = history_text(&mut rx);
    assert!(text.contains("Sandbox ready"), "{text}");
    insta::assert_snapshot!("issue_2062_sandbox_confirmed", text);
    Ok(())
}

fn sandbox_refresh_op(level: codex_protocol::config_types::WindowsSandboxLevel) -> AppCommand {
    AppCommand::OverrideTurnContext {
        cwd: None,
        approval_policy: None,
        approvals_reviewer: None,
        permission_profile: None,
        active_permission_profile: None,
        windows_sandbox_level: Some(level),
        model: None,
        effort: None,
        summary: None,
        service_tier: None,
        collaboration_mode: None,
        personality: None,
    }
}

fn history_text(rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>) -> String {
    let mut text = String::new();
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            for line in cell.display_lines(/*width*/ 100) {
                text.push_str(&line.to_string());
                text.push('\n');
            }
        }
    }
    text
}

#[tokio::test]
async fn issue_2062_unconfirmed_sandbox_has_a_persistent_recovery_message() -> color_eyre::Result<()>
{
    for (snapshot, reply) in [
        (
            "issue_2062_sandbox_uncertain",
            json!({"result": {"outcome": "uncertain"}}),
        ),
        (
            "issue_2062_sandbox_rejected",
            json!({"error": {"code": -32600, "message": "sandbox update rejected"}}),
        ),
    ] {
        let (mut app, mut rx, _ops) = Box::pin(make_test_app_with_channels()).await;
        let (mut server, task) = reply_server(reply).await?;
        while rx.try_recv().is_ok() {}
        app.sync_override_turn_context_settings(
            &mut server,
            ThreadId::new(),
            &sandbox_refresh_op(codex_protocol::config_types::WindowsSandboxLevel::Elevated),
        )
        .await;
        server.shutdown().await?;
        assert_eq!(task.await?.len(), 1);
        let text = history_text(&mut rx);
        assert!(!text.contains("Sandbox ready"), "{text}");
        assert!(text.contains("could not be confirmed"), "{text}");
        assert!(text.contains("/new"), "{text}");
        if snapshot == "issue_2062_sandbox_rejected" {
            assert!(text.contains("sandbox update rejected"), "{text}");
        }
        insta::assert_snapshot!(snapshot, text);
    }
    Ok(())
}
