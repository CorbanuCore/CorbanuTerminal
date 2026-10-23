//! PF-84-S06 (#595): with named accounts on, the app server (standalone, or
//! a local daemon a TUI reuses) leaves a default OpenAI login that the login
//! restrictions rule out in place, never logs it out, refuses a thread on it
//! with the reason, and says so at initialize. Named accounts off: unchanged.

#[cfg(unix)]
use anyhow::Context;
use anyhow::Result;
#[cfg(unix)]
use app_test_support::DISABLE_PLUGIN_STARTUP_TASKS_ARG;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use codex_app_server_protocol::ClientInfo;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ThreadStartParams;
use codex_config::types::AuthCredentialsStoreMode;
use codex_features::Feature;
use codex_login::AuthKeyringBackendKind;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_LOGIN_KEY: &str = "sk-default-595";

/// A home whose default login is an API key while the configuration
/// requires a ChatGPT login.
fn ruled_out_default_login_home(named_accounts: bool) -> Result<(TempDir, Vec<u8>)> {
    let codex_home = TempDir::new()?;
    let mut config = MockResponsesConfig::new("http://127.0.0.1:0")
        .disable_feature(Feature::ShellSnapshot)
        .with_provider_config("requires_openai_auth = true")
        .with_root_config("forced_login_method = \"chatgpt\"")
        .with_root_config("cli_auth_credentials_store = \"file\"");
    if named_accounts {
        config = config.enable_feature(Feature::NamedAccounts);
    }
    config.write(codex_home.path())?;
    codex_login::login_with_api_key(
        codex_home.path(),
        DEFAULT_LOGIN_KEY,
        AuthCredentialsStoreMode::File,
        AuthKeyringBackendKind::default(),
    )?;
    let login = std::fs::read(codex_home.path().join("auth.json"))?;
    Ok((codex_home, login))
}

fn assert_default_login_unchanged(codex_home: &Path, login: &[u8]) {
    assert_eq!(
        std::fs::read(codex_home.join("auth.json")).ok().as_deref(),
        Some(login),
        "the default login must be left in place, unchanged"
    );
}

async fn initialize(app: &mut TestAppServer) -> Result<Value> {
    let message = timeout(
        DEFAULT_TIMEOUT,
        app.initialize_with_client_info(ClientInfo {
            name: "codex-tui".to_string(),
            title: None,
            version: "0.0.0-test".to_string(),
        }),
    )
    .await??;
    let JSONRPCMessage::Response(response) = message else {
        anyhow::bail!("expected an initialize response, got {message:?}");
    };
    Ok(response.result)
}

fn default_thread_start() -> Result<Value> {
    Ok(serde_json::to_value(ThreadStartParams {
        model: Some("mock-model".to_string()),
        ..Default::default()
    })?)
}

#[tokio::test]
async fn named_accounts_server_leaves_a_ruled_out_default_login_and_refuses_it() -> Result<()> {
    let (codex_home, login) = ruled_out_default_login_home(/*named_accounts*/ true)?;
    let mut app = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .with_env_overrides(&[("OPENAI_API_KEY", None)])
        .build()
        .await?;

    let initialize = initialize(&mut app).await?;

    let request = app
        .send_raw_request("thread/start", Some(default_thread_start()?))
        .await?;
    // Before #595 the server started the thread on the ruled-out login.
    let error = timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_error_message(RequestId::Integer(request)),
    )
    .await??;
    assert!(
        error.error.message.contains("ChatGPT login is required")
            && error.error.message.contains("left unchanged"),
        "{}",
        error.error.message
    );
    assert_default_login_unchanged(codex_home.path(), &login);
    assert_eq!(
        initialize.get("ruledOutDefaultLogin"),
        Some(&json!({"forcedLoginMethod": "chatgpt", "forcedChatgptWorkspaceId": null}))
    );
    Ok(())
}

#[tokio::test]
async fn named_accounts_off_server_is_unchanged() -> Result<()> {
    let (codex_home, login) = ruled_out_default_login_home(/*named_accounts*/ false)?;
    let mut app = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .with_env_overrides(&[("OPENAI_API_KEY", None)])
        .build()
        .await?;

    let initialize = initialize(&mut app).await?;
    assert_eq!(initialize.get("ruledOutDefaultLogin"), None);

    let request = app
        .send_raw_request("thread/start", Some(default_thread_start()?))
        .await?;
    // Named accounts off: the thread starts as before.
    timeout(
        DEFAULT_TIMEOUT,
        app.read_stream_until_response_message(RequestId::Integer(request)),
    )
    .await??;
    assert_default_login_unchanged(codex_home.path(), &login);
    Ok(())
}

/// The local daemon a TUI reuses: started first and listening on a control
/// socket; a client connecting later sees the same behaviour.
#[cfg(unix)]
fn request(id: i64, method: &str, params: Value) -> tokio_tungstenite::tungstenite::Message {
    tokio_tungstenite::tungstenite::Message::Text(
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
            .to_string()
            .into(),
    )
}

#[cfg(unix)]
async fn read_response(
    websocket: &mut tokio_tungstenite::WebSocketStream<tokio::net::UnixStream>,
    id: i64,
) -> Result<Value> {
    use futures::StreamExt;
    use tokio_tungstenite::tungstenite::Message;
    loop {
        let frame = timeout(DEFAULT_TIMEOUT, websocket.next())
            .await?
            .context("the daemon closed the connection")??;
        let Message::Text(text) = frame else {
            continue;
        };
        let message: Value = serde_json::from_str(&text)?;
        if message.get("id") == Some(&json!(id)) {
            return Ok(message);
        }
    }
}

#[cfg(unix)]
#[tokio::test]
async fn reused_local_daemon_leaves_a_ruled_out_default_login_and_refuses_it() -> Result<()> {
    use futures::SinkExt;
    use tokio::process::Command;
    use tokio_tungstenite::tungstenite::Message;

    let (codex_home, login) = ruled_out_default_login_home(/*named_accounts*/ true)?;
    // Unix socket paths are short (`SUN_LEN`); test temp dirs may not be.
    let socket_dir = TempDir::new_in("/tmp")?;
    let socket_path = socket_dir.path().join("d.sock");
    let program = codex_utils_cargo_bin::cargo_bin("codex-app-server")
        .context("should find app-server binary")?;
    let _daemon = Command::new(program)
        .arg("--listen")
        .arg(format!("unix://{}", socket_path.display()))
        .arg(DISABLE_PLUGIN_STARTUP_TASKS_ARG)
        .current_dir(codex_home.path())
        // The same isolation `TestAppServer` applies: every profile alias
        // pinned to the fixture, no native keyring, no host managed config.
        .env("CODEX_HOME", codex_home.path())
        .env("CORBANU_HOME", codex_home.path())
        .env("PFTERMINAL_HOME", codex_home.path())
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .env(
            "CODEX_APP_SERVER_MANAGED_CONFIG_PATH",
            codex_home.path().join("managed_config.toml"),
        )
        .env_remove("OPENAI_API_KEY")
        .env_remove("CODEX_API_KEY")
        .env("RUST_LOG", "warn")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("spawn the daemon")?;

    let stream = timeout(DEFAULT_TIMEOUT, async {
        loop {
            if let Ok(stream) = tokio::net::UnixStream::connect(&socket_path).await {
                return stream;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .context("the daemon should listen")?;
    let (mut websocket, _) = tokio_tungstenite::client_async("ws://localhost/", stream).await?;

    websocket
        .send(request(
            /*id*/ 1,
            "initialize",
            json!({"clientInfo": {"name": "codex-tui", "version": "0.0.0-test"},
                   "capabilities": {"experimentalApi": true}}),
        ))
        .await?;
    let initialize = read_response(&mut websocket, /*id*/ 1).await?;
    websocket
        .send(Message::Text(
            json!({"jsonrpc": "2.0", "method": "initialized"})
                .to_string()
                .into(),
        ))
        .await?;

    websocket
        .send(request(
            /*id*/ 2,
            "thread/start",
            default_thread_start()?,
        ))
        .await?;
    let refused = read_response(&mut websocket, /*id*/ 2).await?;
    let message = refused["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("ChatGPT login is required") && message.contains("left unchanged"),
        "a thread on the ruled-out default login must be refused, got {refused}"
    );
    assert_default_login_unchanged(codex_home.path(), &login);
    assert_eq!(
        initialize["result"].get("ruledOutDefaultLogin"),
        Some(&json!({"forcedLoginMethod": "chatgpt", "forcedChatgptWorkspaceId": null}))
    );
    Ok(())
}
