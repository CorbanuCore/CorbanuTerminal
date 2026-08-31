//! Inline orchestration for Claude Code's subscription OAuth flow.

use std::path::Path;
use std::process::Stdio;

use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::timeout;

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::app_event::ClaudeSubscriptionTokenSecret;
use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::ViewCompletion;
use crate::render::renderable::Renderable;
use codex_vault::ClaudeAuthSelection;
use codex_vault::ClaudeAuthSource;
use codex_vault::ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID;
use codex_vault::MANAGED_CLAUDE_AUTH_SOURCE_ID;
use codex_vault::Vault;
#[cfg(target_os = "macos")]
use codex_vault::claude_code_macos_keychain_service;
use codex_vault::credentials_file_claude_auth_source_id;
#[cfg(target_os = "macos")]
use codex_vault::macos_keychain_claude_auth_source_id;

const CLAUDE_CODE_LOGIN_VIEW_ID: &str = "claude-code-plan-login";
const CLAUDE_AUTH_METHOD_VIEW_ID: &str = "claude-plan-auth-method";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const LOGIN_HEALTH_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_LOGIN_LINE_BYTES: usize = 16 * 1024;

pub(crate) enum ClaudeCodeLoginInput {
    AuthorizationCode(String),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClaudeCodePlanStatus {
    Checking,
    ManagedToken {
        stored: bool,
    },
    EnvironmentToken {
        available: bool,
    },
    SelectionRequired {
        existing_source_detected: bool,
    },
    InvalidSelection,
    NeedsReauthorization,
    SignedIn {
        email: Option<String>,
        subscription: Option<String>,
    },
    SignedOut,
    Unavailable,
    Error,
}

pub(crate) async fn current_status_with_timeout(
    codex_home: &Path,
    timeout: Duration,
) -> ClaudeCodePlanStatus {
    current_status_with_executables(
        codex_home,
        timeout,
        Path::new("claude"),
        /*health_executable*/ None,
    )
    .await
}

async fn current_status_with_executables(
    codex_home: &Path,
    timeout: Duration,
    claude_executable: &Path,
    health_executable: Option<&Path>,
) -> ClaudeCodePlanStatus {
    let codex_home = codex_home.to_path_buf();
    let stored = tokio::task::spawn_blocking(move || {
        let vault = Vault::new(codex_home);
        let selection = vault.load_claude_auth_selection()?;
        let managed_stored = matches!(
            vault.managed_claude_subscription_token_status()?,
            codex_vault::ManagedClaudeTokenStatus::Stored { .. }
        );
        Ok::<_, codex_vault::VaultError>((selection, managed_stored))
    })
    .await;
    let Ok(Ok((selection, managed_stored))) = stored else {
        return ClaudeCodePlanStatus::Error;
    };
    if selection
        .as_ref()
        .is_some_and(|selection| !selection_source_id_is_current(selection))
    {
        return ClaudeCodePlanStatus::InvalidSelection;
    }
    match selection {
        Some(ClaudeAuthSelection {
            source: ClaudeAuthSource::ManagedSubscriptionToken,
            ..
        }) => ClaudeCodePlanStatus::ManagedToken {
            stored: managed_stored,
        },
        Some(ClaudeAuthSelection {
            source: ClaudeAuthSource::EnvironmentToken,
            ..
        }) => ClaudeCodePlanStatus::EnvironmentToken {
            available: std::env::var("CLAUDE_CODE_OAUTH_TOKEN")
                .ok()
                .is_some_and(|token| !token.trim().is_empty()),
        },
        Some(
            selection @ ClaudeAuthSelection {
                source: ClaudeAuthSource::ClaudeCodeLogin,
                ..
            },
        ) => {
            let status = status_with_timeout(claude_executable, timeout).await;
            if matches!(status, ClaudeCodePlanStatus::SignedIn { .. })
                && verify_current_platform_login_health(
                    health_executable,
                    timeout,
                    Some(&selection.source_id),
                )
                .await
                .is_err()
            {
                ClaudeCodePlanStatus::NeedsReauthorization
            } else {
                status
            }
        }
        None => {
            let login = status_with_timeout(claude_executable, timeout).await;
            let environment_available = std::env::var("CLAUDE_CODE_OAUTH_TOKEN")
                .ok()
                .is_some_and(|token| !token.trim().is_empty());
            ClaudeCodePlanStatus::SelectionRequired {
                existing_source_detected: managed_stored
                    || environment_available
                    || matches!(login, ClaudeCodePlanStatus::SignedIn { .. }),
            }
        }
    }
}

fn selection_source_id_is_current(selection: &ClaudeAuthSelection) -> bool {
    match selection.source {
        ClaudeAuthSource::ManagedSubscriptionToken => {
            selection.source_id == MANAGED_CLAUDE_AUTH_SOURCE_ID
        }
        ClaudeAuthSource::EnvironmentToken => {
            selection.source_id == ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID
        }
        ClaudeAuthSource::ClaudeCodeLogin => accepted_platform_login_source_ids()
            .is_ok_and(|expected| expected.contains(&selection.source_id)),
    }
}

fn accepted_platform_login_source_ids() -> Result<Vec<String>, String> {
    let mut source_ids = vec![current_platform_login_source_id()?];
    #[cfg(target_os = "macos")]
    {
        let config_dir = claude_config_dir_for_source_id()?;
        source_ids.push(
            credentials_file_claude_auth_source_id(&config_dir).map_err(|error| {
                format!("Could not identify legacy Claude credentials: {error}")
            })?,
        );
    }
    Ok(source_ids)
}

fn current_platform_login_source_id() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .ok_or_else(|| {
                "HOME is not set; cannot identify Claude Code's Keychain profile".to_string()
            })?;
        let configured = std::env::var_os("CLAUDE_CONFIG_DIR")
            .filter(|value| !value.is_empty())
            .map(std::path::PathBuf::from);
        let config_dir_overridden = configured.is_some();
        let config_dir = configured.unwrap_or_else(|| home.join(".claude"));
        let custom_oauth = std::env::var("CLAUDE_CODE_CUSTOM_OAUTH_URL")
            .ok()
            .is_some_and(|value| !value.trim().is_empty());
        let service =
            claude_code_macos_keychain_service(&config_dir, config_dir_overridden, custom_oauth);
        Ok(macos_keychain_claude_auth_source_id(&service))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let home = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .ok_or_else(|| {
                "HOME is not set; cannot identify Claude Code's credentials-file profile"
                    .to_string()
            })?;
        let config_dir = std::env::var_os("CLAUDE_CONFIG_DIR")
            .filter(|value| !value.is_empty())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| home.join(".claude"));
        credentials_file_claude_auth_source_id(&config_dir).map_err(|error| {
            format!("cannot identify Claude Code's credentials-file profile: {error}")
        })
    }
}

#[cfg(target_os = "macos")]
fn claude_config_dir_for_source_id() -> Result<std::path::PathBuf, String> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "HOME is not set; cannot identify Claude Code's profile".to_string())?;
    Ok(std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home.join(".claude")))
}

async fn status_with_timeout(executable: &Path, timeout: Duration) -> ClaudeCodePlanStatus {
    match tokio::time::timeout(timeout, read_status(executable)).await {
        Ok(Ok(status)) => status,
        Ok(Err(err)) if err.kind() == std::io::ErrorKind::NotFound => {
            ClaudeCodePlanStatus::Unavailable
        }
        Ok(Err(_)) | Err(_) => ClaudeCodePlanStatus::Error,
    }
}

pub(crate) fn start(
    app_event_tx: AppEventSender,
    codex_home: std::path::PathBuf,
) -> mpsc::UnboundedSender<ClaudeCodeLoginInput> {
    start_with_executable(
        app_event_tx,
        Path::new("claude"),
        /*health_executable*/ None,
        codex_home,
    )
}

fn start_with_executable(
    app_event_tx: AppEventSender,
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: std::path::PathBuf,
) -> mpsc::UnboundedSender<ClaudeCodeLoginInput> {
    let (input_tx, input_rx) = mpsc::unbounded_channel();
    let task_input_tx = input_tx.clone();
    let executable = executable.to_path_buf();
    let health_executable = health_executable.map(Path::to_path_buf);
    tokio::spawn(async move {
        let result = run_login(
            &executable,
            health_executable.as_deref(),
            &codex_home,
            app_event_tx.clone(),
            task_input_tx,
            input_rx,
        )
        .await;
        app_event_tx.send(AppEvent::ClaudeCodePlanLoginFinished { result });
    });
    input_tx
}

async fn run_login(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    app_event_tx: AppEventSender,
    input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    mut input_rx: mpsc::UnboundedReceiver<ClaudeCodeLoginInput>,
) -> Option<Result<String, String>> {
    let mut child = match Command::new(executable)
        .args(["auth", "login", "--claudeai"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            return Some(Err(format!(
                "Could not start `claude auth login --claudeai`: {err}. Install Claude Code and try again."
            )));
        }
    };

    let Some(stdout) = child.stdout.take() else {
        return Some(Err(
            "Claude Code login did not provide an output stream.".to_string()
        ));
    };
    let Some(mut stdin) = child.stdin.take() else {
        return Some(Err(
            "Claude Code login did not provide an input stream.".to_string()
        ));
    };
    let (output_tx, mut output_rx) = mpsc::unbounded_channel();
    spawn_output_reader(stdout, output_tx.clone());
    if let Some(stderr) = child.stderr.take() {
        spawn_output_reader(stderr, output_tx.clone());
    }
    drop(output_tx);

    let verification_url = loop {
        let line = tokio::select! {
            input = input_rx.recv() => {
                if matches!(input, Some(ClaudeCodeLoginInput::Cancel) | None) {
                    let _ = child.kill().await;
                    return None;
                }
                continue;
            }
            line = output_rx.recv() => line,
        };
        match line {
            None => {
                let status = child.wait().await.ok();
                return Some(Err(format!(
                    "Claude Code login exited before providing a browser URL{}.",
                    status
                        .map(|status| format!(" ({status})"))
                        .unwrap_or_default()
                )));
            }
            Some(line) => {
                if line.len() > MAX_LOGIN_LINE_BYTES {
                    let _ = child.kill().await;
                    return Some(Err(
                        "Claude Code login returned an unexpectedly large browser URL.".to_string(),
                    ));
                }
                if let Some(url) = extract_https_url(&line) {
                    break url;
                }
            }
        }
    };

    drop(output_rx);
    app_event_tx.send(AppEvent::ClaudeCodePlanLoginReady {
        verification_url,
        input_tx,
    });

    let authorization_code = match input_rx.recv().await {
        Some(ClaudeCodeLoginInput::AuthorizationCode(code)) => code,
        Some(ClaudeCodeLoginInput::Cancel) | None => {
            let _ = child.kill().await;
            return None;
        }
    };
    if let Err(err) = stdin
        .write_all(format!("{authorization_code}\n").as_bytes())
        .await
    {
        let _ = child.kill().await;
        return Some(Err(format!(
            "Could not submit the authorization code to Claude Code: {err}"
        )));
    }
    drop(stdin);

    let status = match timeout(LOGIN_TIMEOUT, child.wait()).await {
        Ok(Ok(status)) => status,
        Ok(Err(err)) => return Some(Err(format!("Claude Code login failed to finish: {err}"))),
        Err(_) => {
            let _ = child.kill().await;
            return Some(Err(
                "Claude Code login timed out after 15 minutes.".to_string()
            ));
        }
    };
    if !status.success() {
        return Some(Err(format!("Claude Code rejected the login ({status}).")));
    }

    Some(verify_login(executable, health_executable, codex_home).await)
}

fn spawn_output_reader(
    output: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    output_tx: mpsc::UnboundedSender<String>,
) {
    tokio::spawn(async move {
        let mut output = BufReader::new(output);
        loop {
            let mut line = String::new();
            match output.read_line(&mut line).await {
                Ok(0) | Err(_) => return,
                Ok(_) => {
                    if output_tx.send(line).is_err() {
                        let mut sink = tokio::io::sink();
                        let _ = tokio::io::copy(&mut output, &mut sink).await;
                        return;
                    }
                }
            }
        }
    });
}

async fn verify_login(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
) -> Result<String, String> {
    match read_status(executable).await {
        Ok(ClaudeCodePlanStatus::SignedIn { .. }) => {
            let source_id = verify_current_platform_login_health(
                health_executable,
                LOGIN_HEALTH_TIMEOUT,
                /*selected_source_id*/ None,
            )
            .await?;
            persist_claude_code_login_selection(codex_home, source_id).await?;
            Ok("Claude Code login selected. Retry the Claude Plan request or choose a model from /model.".to_string())
        }
        Ok(_) => {
            Err("Claude Code is not signed in with a Claude subscription after login.".to_string())
        }
        Err(err) => Err(format!("Could not verify Claude Code login: {err}")),
    }
}

pub(crate) async fn select_existing_claude_code_login(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    status_timeout: Duration,
) -> Result<bool, String> {
    match status_with_timeout(executable, status_timeout).await {
        ClaudeCodePlanStatus::SignedIn { .. } => {
            let source_id = verify_current_platform_login_health(
                health_executable,
                LOGIN_HEALTH_TIMEOUT,
                /*selected_source_id*/ None,
            )
            .await?;
            persist_claude_code_login_selection(codex_home, source_id).await?;
            Ok(true)
        }
        ClaudeCodePlanStatus::SignedOut
        | ClaudeCodePlanStatus::Unavailable
        | ClaudeCodePlanStatus::Error
        | ClaudeCodePlanStatus::ManagedToken { .. }
        | ClaudeCodePlanStatus::EnvironmentToken { .. }
        | ClaudeCodePlanStatus::SelectionRequired { .. }
        | ClaudeCodePlanStatus::InvalidSelection
        | ClaudeCodePlanStatus::NeedsReauthorization
        | ClaudeCodePlanStatus::Checking => Ok(false),
    }
}

async fn verify_current_platform_login_health(
    executable: Option<&Path>,
    health_timeout: Duration,
    selected_source_id: Option<&str>,
) -> Result<String, String> {
    let current_executable;
    let executable = match executable {
        Some(executable) => executable,
        None => {
            current_executable = std::env::current_exe().map_err(|error| {
                format!("Could not locate Corbanu to verify Claude Code credentials: {error}")
            })?;
            current_executable.as_path()
        }
    };
    let mut command = Command::new(executable);
    command.arg("internal-claude-login-health");
    if let Some(source_id) = selected_source_id {
        command.arg("--source-id").arg(source_id);
    }
    let output = timeout(
        health_timeout,
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| "Timed out while verifying Claude Code's stored credentials".to_string())?
    .map_err(|error| format!("Could not verify Claude Code's stored credentials: {error}"))?;
    if !output.status.success() {
        return Err(
            "Claude Code reports a login, but its stored credential needs reauthorization. Run `claude auth login` again; your previous Corbanu method remains selected."
                .to_string(),
        );
    }
    let source_id = String::from_utf8(output.stdout)
        .map_err(|_| "Claude credential verification returned invalid metadata".to_string())?
        .trim()
        .to_string();
    let accepted = accepted_platform_login_source_ids()?;
    if !accepted.contains(&source_id)
        || selected_source_id.is_some_and(|selected| selected != source_id)
    {
        return Err(
            "Claude credential verification returned a different platform profile; the previous method remains selected."
                .to_string(),
        );
    }
    Ok(source_id)
}

async fn persist_claude_code_login_selection(
    codex_home: &Path,
    source_id: String,
) -> Result<(), String> {
    let codex_home = codex_home.to_path_buf();
    tokio::task::spawn_blocking(move || {
        persist_claude_code_login_selection_blocking(&codex_home, source_id)
    })
        .await
        .map_err(|error| {
            format!(
                "Claude Code login succeeded, but Corbanu could not finish selecting it: {error}. Your previous method remains selected; retry from Providers."
            )
        })?
}

fn persist_claude_code_login_selection_blocking(
    codex_home: &Path,
    source_id: String,
) -> Result<(), String> {
    let selection = ClaudeAuthSelection::new(ClaudeAuthSource::ClaudeCodeLogin, source_id)
        .map_err(|error| format!("Could not select the current Claude Code login: {error}"))?;
    Vault::new(codex_home.to_path_buf())
        .save_claude_auth_selection(&selection)
        .map_err(|error| {
            format!(
                "Claude Code login succeeded, but Corbanu could not select it: {error}. Your previous method remains selected; retry from Providers."
            )
        })
}

fn auth_method_choice_params() -> SelectionViewParams {
    SelectionViewParams {
        view_id: Some(CLAUDE_AUTH_METHOD_VIEW_ID),
        title: Some("Claude Plan authentication".to_string()),
        subtitle: Some(
            "Choose one source; Corbanu never falls back to another account.".to_string(),
        ),
        footer_note: Some(Line::from(
            "Your account and billing path change only after success. Esc keeps the current method."
                .dim(),
        )),
        items: vec![
            SelectionItem {
                name: "Long-lived subscription token (Recommended)".to_string(),
                description: Some(
                    "Run `claude setup-token` in a private terminal, then paste its approximately one-year token here (Pro, Max, Team, or Enterprise)."
                        .to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::RunClaudeSetupToken))],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Claude Code login".to_string(),
                description: Some(
                    "Use Claude Code's rotating login state; reauthorization may be needed more often."
                        .to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::UseClaudeCodePlanLogin))],
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        initial_selected_idx: Some(0),
        allow_number_shortcuts: false,
        ..Default::default()
    }
}

fn auth_recovery_params(message: String) -> SelectionViewParams {
    SelectionViewParams {
        title: Some("Claude authentication needs attention".to_string()),
        subtitle: Some(message),
        footer_note: Some(Line::from(
            "No fallback occurred. Esc closes this without choosing another method.".dim(),
        )),
        items: vec![
            SelectionItem {
                name: "Retry long-lived token setup".to_string(),
                description: Some(
                    "Run `claude setup-token` in a private terminal, then return to masked entry."
                        .to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::RunClaudeSetupToken))],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Choose authentication method".to_string(),
                description: Some("Return to the explicit source picker.".to_string()),
                actions: vec![Box::new(|tx| tx.send(AppEvent::OpenClaudeCodePlanLogin))],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Use CLAUDE_CODE_OAUTH_TOKEN (legacy)".to_string(),
                description: Some(
                    "Persist this exact legacy source only if the environment variable is currently set and nonblank."
                        .to_string(),
                ),
                actions: vec![Box::new(|tx| {
                    tx.send(AppEvent::UseLegacyClaudeEnvironmentToken)
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Keep current method".to_string(),
                description: Some("Close this message without changing credentials.".to_string()),
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        initial_selected_idx: Some(0),
        allow_number_shortcuts: false,
        ..Default::default()
    }
}

async fn read_status(executable: &Path) -> std::io::Result<ClaudeCodePlanStatus> {
    let mut command = Command::new(executable);
    command.kill_on_drop(true);
    let output = command.args(["auth", "status", "--json"]).output().await?;
    if !output.status.success() {
        return Ok(ClaudeCodePlanStatus::SignedOut);
    }
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Claude Code returned invalid authentication status: {err}"),
        )
    })?;
    let logged_in = status.get("loggedIn").and_then(serde_json::Value::as_bool);
    let auth_method = status.get("authMethod").and_then(serde_json::Value::as_str);
    if logged_in != Some(true) || auth_method != Some("claude.ai") {
        return Ok(ClaudeCodePlanStatus::SignedOut);
    }
    Ok(ClaudeCodePlanStatus::SignedIn {
        email: status
            .get("email")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        subscription: status
            .get("subscriptionType")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    })
}

fn extract_https_url(line: &str) -> Option<String> {
    let start = line.find("https://")?;
    let remainder = &line[start..];
    let end = remainder
        .find(|character: char| {
            character.is_whitespace() || matches!(character, '\u{7}' | '\u{1b}')
        })
        .unwrap_or(remainder.len());
    Some(remainder[..end].to_string())
}

impl ChatWidget {
    pub(crate) fn open_claude_auth_method_choice(&mut self) {
        self.show_selection_view(auth_method_choice_params());
    }

    pub(crate) fn open_claude_auth_recovery(&mut self, message: String) {
        self.show_selection_view(auth_recovery_params(message));
    }

    pub(crate) fn open_claude_subscription_token_entry(&mut self) {
        let submit_tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret_with_cancel(
            "claude-subscription-token".to_string(),
            "Save Claude subscription token".to_string(),
            "Long-lived token — masked".to_string(),
            "In a separate private terminal, run `claude setup-token`. Paste its token here; Corbanu never captures the command output or adds the token to chat."
                .to_string(),
            Box::new(move |_label, token| {
                submit_tx.send(AppEvent::SaveClaudeManagedSubscriptionToken {
                    token: ClaudeSubscriptionTokenSecret::new(token),
                });
            }),
            Box::new(|| {}),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn on_claude_managed_subscription_token_saved(
        &mut self,
        result: Result<String, String>,
    ) {
        match result {
            Ok(message) => self.add_info_message(message, /*hint*/ None),
            Err(message) => {
                self.add_error_message(message.clone());
                self.open_claude_auth_recovery(message);
            }
        }
    }

    pub(crate) fn open_claude_code_plan_login_pending(
        &mut self,
        input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    ) {
        self.bottom_pane
            .show_view(Box::new(ClaudeCodePlanLoginView::pending(
                self.app_event_tx.clone(),
                input_tx,
            )));
    }

    pub(crate) fn open_claude_code_plan_login_ready(
        &mut self,
        verification_url: String,
        input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    ) {
        self.bottom_pane.replace_active_view_by_id(
            CLAUDE_CODE_LOGIN_VIEW_ID,
            Box::new(ClaudeCodePlanLoginView::ready(
                self.app_event_tx.clone(),
                verification_url,
                input_tx,
            )),
        );
    }

    pub(crate) fn open_claude_code_plan_login_code_entry(
        &mut self,
        input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    ) {
        let submit_tx = input_tx.clone();
        let cancel_tx = input_tx;
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret_with_cancel(
            "claude-code-oauth-code".to_string(),
            "Claude Code Plan Login".to_string(),
            "Authorization code — masked".to_string(),
            "Paste the one-time code from the browser".to_string(),
            Box::new(move |_label, code| {
                let _ = submit_tx.send(ClaudeCodeLoginInput::AuthorizationCode(code));
            }),
            Box::new(move || {
                let _ = cancel_tx.send(ClaudeCodeLoginInput::Cancel);
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn on_claude_code_plan_login_finished(
        &mut self,
        result: Option<Result<String, String>>,
    ) {
        self.bottom_pane
            .dismiss_view_by_id(CLAUDE_CODE_LOGIN_VIEW_ID);
        match result {
            Some(Ok(message)) => self.add_info_message(message, /*hint*/ None),
            Some(Err(message)) => {
                let message = format!("Claude Code plan login failed: {message}");
                self.add_error_message(message.clone());
                self.open_claude_auth_recovery(message);
            }
            None => {}
        }
    }
}

struct ClaudeCodePlanLoginView {
    app_event_tx: AppEventSender,
    input_tx: Option<mpsc::UnboundedSender<ClaudeCodeLoginInput>>,
    verification_url: Option<String>,
    completion: Option<ViewCompletion>,
}

impl ClaudeCodePlanLoginView {
    fn pending(
        app_event_tx: AppEventSender,
        input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    ) -> Self {
        Self {
            app_event_tx,
            input_tx: Some(input_tx),
            verification_url: None,
            completion: None,
        }
    }

    fn ready(
        app_event_tx: AppEventSender,
        verification_url: String,
        input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    ) -> Self {
        Self {
            app_event_tx,
            input_tx: Some(input_tx),
            verification_url: Some(verification_url),
            completion: None,
        }
    }

    fn cancel(&mut self) {
        if self.completion.is_some() {
            return;
        }
        if let Some(input_tx) = self.input_tx.take() {
            let _ = input_tx.send(ClaudeCodeLoginInput::Cancel);
        }
        self.completion = Some(ViewCompletion::Cancelled);
    }

    fn accept(&mut self) {
        let Some(input_tx) = self.input_tx.take() else {
            return;
        };
        if self.verification_url.is_none() {
            self.input_tx = Some(input_tx);
            return;
        }
        self.app_event_tx
            .send(AppEvent::OpenClaudeCodePlanLoginCodeEntry { input_tx });
        self.completion = Some(ViewCompletion::Accepted);
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec!["Claude Code Plan".bold().into(), Line::from("")];
        let Some(verification_url) = &self.verification_url else {
            lines.push("Starting Claude Code login...".dim().into());
            lines.push(Line::from(""));
            lines.push("Press Esc to cancel".dim().into());
            return lines;
        };
        lines.push("Sign in with your Claude subscription in the browser:".into());
        lines.push(Line::from(""));
        let wrap_width = usize::from(width.saturating_sub(2).max(1));
        lines.extend(
            textwrap::wrap(verification_url, wrap_width)
                .into_iter()
                .map(|part| part.into_owned().cyan().underlined().into()),
        );
        lines.push(Line::from(""));
        lines.push(
            "Press Enter to paste the one-time code, or Esc to cancel"
                .dim()
                .into(),
        );
        lines
    }
}

impl BottomPaneView for ClaudeCodePlanLoginView {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.cancel(),
            KeyCode::Enter => self.accept(),
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(CLAUDE_CODE_LOGIN_VIEW_ID)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.cancel();
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }
}

impl Renderable for ClaudeCodePlanLoginView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines(area.width)).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len().min(usize::from(u16::MAX)) as u16
    }
}

#[cfg(test)]
#[path = "claude_code_login_tests.rs"]
mod tests;
