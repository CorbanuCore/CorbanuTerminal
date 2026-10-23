//! PF-84-S06 (#595): a TUI reusing a running local app-server daemon leaves a
//! ruled-out default login in place exactly like one that starts its own app
//! server; a default session still enforces as before.
#![cfg(unix)]

use super::*;
use crate::legacy_core::config::ConfigBuilder;
use codex_app_server_client::RemoteAppServerEndpoint;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RuledOutDefaultLoginPolicy;
use codex_config::types::AuthCredentialsStoreMode;
use codex_features::Feature;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use codex_protocol::config_types::ForcedLoginMethod;
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::SinkExt;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio_tungstenite::tungstenite::Message;

struct Launch {
    _home: TempDir,
    config: Config,
    auth_file: PathBuf,
    login_before: Vec<u8>,
}

/// A home whose default login is an API key while the configuration
/// requires a ChatGPT login: the restrictions rule the default login out.
async fn launch(named_accounts: bool, named_session: bool) -> Launch {
    let home = TempDir::new().expect("home");
    let mut config = ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .expect("config");
    if named_accounts {
        config
            .features
            .enable(Feature::NamedAccounts)
            .expect("named accounts should be available in tests");
    }
    config.model_provider_id = "openai".to_string();
    config.model_provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
    if named_session {
        config.model_provider.account = Some(NamedProviderAccount {
            provider_id: "openai".to_string(),
            name: "work".to_string(),
        });
    }
    config.configured_forced_login_method = Some(ForcedLoginMethod::Chatgpt);
    config.forced_login_method = Some(ForcedLoginMethod::Chatgpt);
    config.cli_auth_credentials_store_mode = AuthCredentialsStoreMode::File;
    codex_login::login_with_api_key(
        &config.codex_home,
        "sk-default-595",
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    )
    .expect("seed the default login");
    let auth_file = config.codex_home.join("auth.json").to_path_buf();
    let login_before = std::fs::read(&auth_file).expect("default login");
    Launch {
        _home: home,
        config,
        auth_file,
        login_before,
    }
}

impl Launch {
    fn assert_default_login_unchanged(&self) {
        assert_eq!(
            std::fs::read(&self.auth_file).ok(),
            Some(self.login_before.clone()),
            "the default login must be left in place, unchanged"
        );
    }

    fn policy(&self) -> RuledOutDefaultLoginPolicy {
        RuledOutDefaultLoginPolicy {
            forced_login_method: self.config.configured_forced_login_method,
            forced_chatgpt_workspace_id: self.config.forced_chatgpt_workspace_id.clone(),
        }
    }
}

/// Unix socket paths are short (`SUN_LEN`); test temp dirs may not be.
fn socket_dir() -> TempDir {
    TempDir::new_in("/tmp").expect("socket dir")
}

/// A running local daemon that answers `initialize`, advertising `policy`
/// as its `ruledOutDefaultLogin` (an older daemon, or one with named
/// accounts off, advertises nothing).
fn spawn_daemon(
    socket_dir: &Path,
    policy: Option<RuledOutDefaultLoginPolicy>,
) -> (AppServerTarget, tokio::task::JoinHandle<()>) {
    let socket_path = socket_dir.join("d.sock");
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind daemon socket");
    let daemon = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let policy = policy.clone();
            tokio::spawn(async move {
                let Ok(mut websocket) = tokio_tungstenite::accept_async(stream).await else {
                    return;
                };
                while let Some(Ok(frame)) = websocket.next().await {
                    let Message::Text(text) = frame else {
                        continue;
                    };
                    let Ok(JSONRPCMessage::Request(request)) =
                        serde_json::from_str::<JSONRPCMessage>(&text)
                    else {
                        continue;
                    };
                    if request.method != "initialize" {
                        continue;
                    }
                    let mut result = serde_json::json!({
                        "userAgent": "codex_cli_rs/0.0.0-test (Test OS; x86_64) rust",
                        "codexHome": "/daemon/home",
                        "platformFamily": "unix",
                        "platformOs": "test",
                    });
                    if let Some(policy) = &policy {
                        result["ruledOutDefaultLogin"] =
                            serde_json::to_value(policy).expect("policy");
                    }
                    let response = JSONRPCMessage::Response(JSONRPCResponse {
                        id: request.id,
                        result,
                    });
                    let text = serde_json::to_string(&response).expect("response");
                    if websocket.send(Message::Text(text.into())).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    let target = AppServerTarget::LocalDaemon {
        endpoint: RemoteAppServerEndpoint::UnixSocket {
            socket_path: AbsolutePathBuf::from_absolute_path(&socket_path).expect("socket path"),
        },
    };
    (target, daemon)
}

#[tokio::test]
async fn named_session_never_reuses_a_daemon_and_leaves_the_default_login_in_place() {
    let launch = launch(/*named_accounts*/ true, /*named_session*/ true).await;
    let socket_dir = socket_dir();
    let (daemon_target, daemon) = spawn_daemon(socket_dir.path(), Some(launch.policy()));

    let settled = settle_default_login_at_launch(
        &launch.config,
        /*launches_resume_or_fork*/ false,
        daemon_target,
    )
    .await
    .expect("launch");
    launch.assert_default_login_unchanged();

    assert!(settled.left_in_place);
    // #658: even a daemon that leaves the login in place is not reused; it
    // may serve another session on another account.
    assert_eq!(settled.app_server_target, AppServerTarget::Embedded);
    assert!(settled.daemon_not_reused.is_some());
    daemon.abort();
}

#[tokio::test]
async fn resume_never_reuses_a_daemon_and_leaves_the_default_login_in_place() {
    let launch = launch(/*named_accounts*/ true, /*named_session*/ false).await;
    let socket_dir = socket_dir();
    let (daemon_target, daemon) = spawn_daemon(socket_dir.path(), Some(launch.policy()));

    let settled = settle_default_login_at_launch(
        &launch.config,
        /*launches_resume_or_fork*/ true,
        daemon_target,
    )
    .await
    .expect("launch");
    launch.assert_default_login_unchanged();

    assert!(settled.left_in_place);
    // #658: even a daemon that leaves the login in place is not reused; it
    // may serve another session on another account.
    assert_eq!(settled.app_server_target, AppServerTarget::Embedded);
    assert!(settled.daemon_not_reused.is_some());
    daemon.abort();
}

/// A daemon that would use the ruled-out login (an older build, named
/// accounts off when it started, or other restrictions) is not reused: the
/// TUI starts its own app server, which leaves the login in place.
#[tokio::test]
async fn named_session_starts_its_own_server_when_the_daemon_would_use_the_login() {
    let launch = launch(/*named_accounts*/ true, /*named_session*/ true).await;
    let mut other_restrictions = launch.policy();
    other_restrictions.forced_login_method = None;
    for policy in [None, Some(other_restrictions)] {
        let socket_dir = socket_dir();
        let (daemon_target, daemon) = spawn_daemon(socket_dir.path(), policy.clone());

        let settled = settle_default_login_at_launch(
            &launch.config,
            /*launches_resume_or_fork*/ false,
            daemon_target,
        )
        .await
        .expect("launch");
        launch.assert_default_login_unchanged();

        assert!(settled.left_in_place, "{policy:?}");
        assert_eq!(settled.app_server_target, AppServerTarget::Embedded);
        assert!(settled.daemon_not_reused.is_some(), "{policy:?}");
        daemon.abort();
    }
}

#[tokio::test]
async fn named_session_starts_its_own_server_when_the_daemon_is_gone() {
    let launch = launch(/*named_accounts*/ true, /*named_session*/ true).await;
    let socket_dir = socket_dir();
    let gone = AppServerTarget::LocalDaemon {
        endpoint: RemoteAppServerEndpoint::UnixSocket {
            socket_path: AbsolutePathBuf::from_absolute_path(socket_dir.path().join("d.sock"))
                .expect("socket path"),
        },
    };

    let settled = settle_default_login_at_launch(
        &launch.config,
        /*launches_resume_or_fork*/ false,
        gone,
    )
    .await
    .expect("launch");
    launch.assert_default_login_unchanged();

    assert!(settled.left_in_place);
    assert_eq!(settled.app_server_target, AppServerTarget::Embedded);
    assert!(settled.daemon_not_reused.is_some());
    // A reloaded session config (resume, fork) carries the warning over.
    let mut config = launch.config.clone();
    assert_eq!(daemon_not_reused_warning(&config), None);
    config
        .startup_warnings
        .extend(settled.daemon_not_reused.clone());
    assert_eq!(
        daemon_not_reused_warning(&config),
        settled.daemon_not_reused
    );
}

/// Unchanged: a default session, or any launch with named accounts off,
/// still logs out a ruled-out default login at startup, daemon or not.
#[tokio::test]
async fn default_session_or_named_accounts_off_still_enforces_on_a_daemon() {
    for (named_accounts, resume) in [(true, false), (false, true)] {
        let launch = launch(named_accounts, /*named_session*/ false).await;
        let socket_dir = socket_dir();
        let (daemon_target, daemon) = spawn_daemon(socket_dir.path(), Some(launch.policy()));

        let settled = settle_default_login_at_launch(&launch.config, resume, daemon_target.clone())
            .await
            .expect("launch");

        let case = (named_accounts, resume);
        assert!(!settled.left_in_place, "{case:?}");
        assert_eq!(settled.app_server_target, daemon_target, "{case:?}");
        assert!(!launch.auth_file.exists(), "{case:?}: logged out as before");
        daemon.abort();
    }
}

#[tokio::test]
async fn named_session_with_the_embedded_server_leaves_the_default_login_in_place() {
    let launch = launch(/*named_accounts*/ true, /*named_session*/ true).await;

    let settled = settle_default_login_at_launch(
        &launch.config,
        /*launches_resume_or_fork*/ false,
        AppServerTarget::Embedded,
    )
    .await
    .expect("launch");
    launch.assert_default_login_unchanged();

    assert!(settled.left_in_place);
    assert_eq!(settled.app_server_target, AppServerTarget::Embedded);
}
