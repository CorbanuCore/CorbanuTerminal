//! Inline orchestration for Claude Code's subscription OAuth flow.

use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;

use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncRead;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::timeout;
use zeroize::Zeroizing;

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::app_event::ClaudeSubscriptionTokenSecret;
use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::ViewCompletion;
use crate::internal_cli_helper::internal_cli_helper_executable;
use crate::render::renderable::Renderable;
use codex_provider_auth::claude_account_flow::ClaudeCodeIdentityPolicy;
use codex_vault::ClaudeAuthSelection;
use codex_vault::ClaudeAuthSource;
use codex_vault::ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID;
use codex_vault::MANAGED_CLAUDE_AUTH_SOURCE_ID;
use codex_vault::Vault;
#[cfg(target_os = "macos")]
use codex_vault::claude_code_macos_keychain_service;
use codex_vault::claude_environment_token_authority_id;
use codex_vault::claude_login_authority_id;
use codex_vault::credentials_file_claude_auth_source_id;
#[cfg(target_os = "macos")]
use codex_vault::macos_keychain_claude_auth_source_id;

const CLAUDE_CODE_LOGIN_VIEW_ID: &str = "claude-code-plan-login";
const CLAUDE_AUTH_METHOD_VIEW_ID: &str = "claude-plan-auth-method";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const LOGIN_HEALTH_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_LOGIN_LINE_BYTES: usize = 16 * 1024;
const CLAUDE_STATUS_ENV_REMOVE: [&str; 8] = [
    "CLAUDE_CODE_OAUTH_TOKEN",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
    "ANTHROPIC_MODEL",
];
// Preserve `CLAUDE_CODE_CUSTOM_OAUTH_URL`: it is profile identity, not provider routing.
// Claude Code needs it to inspect the same custom-OAuth login/Keychain slot Corbanu selected.

pub(crate) enum ClaudeCodeLoginInput {
    AuthorizationCode(Zeroizing<String>),
    Cancel,
}

pub(crate) enum ClaudeCodeLoginBackendEvent {
    Ready {
        verification_url: String,
        input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    },
    Finished {
        result: ClaudeCodeLoginBackendResult,
    },
}

pub(crate) type ClaudeCodeLoginBackendResult = Option<Result<String, ClaudeCodeLoginBackendError>>;

pub(crate) enum ClaudeCodeLoginBackendError {
    IdentityConflict,
    TimedOut,
    Other(String),
}

pub(crate) enum ClaudeManagedEnrollmentError {
    Invalid(String),
    StorageUnavailable(String),
}

impl ClaudeCodeLoginBackendError {
    fn into_message(self) -> String {
        match self {
            Self::IdentityConflict => {
                "Claude Code returned a different selected account; the previous Corbanu selection was preserved."
                    .to_string()
            }
            Self::TimedOut => "Claude Code login timed out after 15 minutes.".to_string(),
            Self::Other(message) => message,
        }
    }
}

impl From<String> for ClaudeCodeLoginBackendError {
    fn from(message: String) -> Self {
        Self::Other(message)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ExpectedClaudeCodeIdentity {
    source_id: String,
    authority_id: String,
}

fn remove_line_breaks(mut token: String) -> String {
    token.retain(|character| !matches!(character, '\r' | '\n'));
    token
}

pub(crate) async fn enroll_managed_subscription_token(
    codex_home: std::path::PathBuf,
    token: ClaudeSubscriptionTokenSecret,
) -> Result<String, String> {
    enroll_managed_subscription_token_typed(codex_home, token)
        .await
        .map_err(|error| match error {
            ClaudeManagedEnrollmentError::Invalid(message) => {
                format!("Claude subscription token was rejected: {message}. No fallback was attempted; inspect Providers and retry.")
            }
            ClaudeManagedEnrollmentError::StorageUnavailable(message) => {
                format!("Claude subscription token was not saved: {message}. No fallback was attempted; inspect Providers and retry.")
            }
        })?;
    Ok(
        "Long-lived Claude subscription token saved and selected. Retry the interrupted request or choose a Claude Plan model from /model."
            .to_string(),
    )
}

pub(crate) async fn enroll_managed_subscription_token_typed(
    codex_home: std::path::PathBuf,
    token: ClaudeSubscriptionTokenSecret,
) -> Result<(), ClaudeManagedEnrollmentError> {
    let token = remove_line_breaks(token.into_inner());
    tokio::task::spawn_blocking(move || {
        Vault::new(codex_home).enroll_managed_claude_subscription_token(token)
    })
    .await
    .map_err(|error| ClaudeManagedEnrollmentError::StorageUnavailable(error.to_string()))?
    .map(|_| ())
    .map_err(|error| match error {
        codex_vault::ClaudeSubscriptionTokenError::Empty
        | codex_vault::ClaudeSubscriptionTokenError::InvalidFormat => {
            ClaudeManagedEnrollmentError::Invalid(error.to_string())
        }
        codex_vault::ClaudeSubscriptionTokenError::Vault(_) => {
            ClaudeManagedEnrollmentError::StorageUnavailable(error.to_string())
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PlatformLoginHealthCheckError {
    NeedsReauthorization(String),
    IdentityMismatch(String),
    Undetermined(String),
}

impl std::fmt::Display for PlatformLoginHealthCheckError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeedsReauthorization(message)
            | Self::IdentityMismatch(message)
            | Self::Undetermined(message) => formatter.write_str(message),
        }
    }
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
        organization_id: Option<String>,
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
    tokio::time::timeout(
        timeout,
        current_status_with_executables(
            codex_home,
            timeout,
            Path::new("claude"),
            /*health_executable*/ None,
        ),
    )
    .await
    .unwrap_or(ClaudeCodePlanStatus::Error)
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
        Some(
            selection @ ClaudeAuthSelection {
                source: ClaudeAuthSource::EnvironmentToken,
                ..
            },
        ) => ClaudeCodePlanStatus::EnvironmentToken {
            available: environment_token_matches_selection(&selection),
        },
        Some(
            selection @ ClaudeAuthSelection {
                source: ClaudeAuthSource::ClaudeCodeLogin,
                ..
            },
        ) => {
            match verify_current_platform_login_health(
                health_executable,
                timeout,
                Some(&selection.source_id),
            )
            .await
            {
                Ok(_) => {}
                Err(PlatformLoginHealthCheckError::NeedsReauthorization(_)) => {
                    return ClaudeCodePlanStatus::NeedsReauthorization;
                }
                Err(PlatformLoginHealthCheckError::IdentityMismatch(_)) => {
                    return ClaudeCodePlanStatus::NeedsReauthorization;
                }
                Err(PlatformLoginHealthCheckError::Undetermined(_)) => {
                    return ClaudeCodePlanStatus::Error;
                }
            }
            let status = status_with_timeout(claude_executable, timeout).await;
            match &status {
                ClaudeCodePlanStatus::SignedIn { .. } => match status_authority_id(&status) {
                    Ok(authority_id)
                        if selection.authority_id.as_deref() == Some(authority_id.as_str()) =>
                    {
                        status
                    }
                    _ => ClaudeCodePlanStatus::NeedsReauthorization,
                },
                ClaudeCodePlanStatus::SignedOut => ClaudeCodePlanStatus::NeedsReauthorization,
                ClaudeCodePlanStatus::Unavailable => ClaudeCodePlanStatus::Unavailable,
                ClaudeCodePlanStatus::Error => ClaudeCodePlanStatus::Error,
                _ => ClaudeCodePlanStatus::Error,
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

fn environment_token_matches_selection(selection: &ClaudeAuthSelection) -> bool {
    environment_token_matches_selection_value(
        selection,
        std::env::var("CLAUDE_CODE_OAUTH_TOKEN").ok().as_deref(),
    )
}

fn environment_token_matches_selection_value(
    selection: &ClaudeAuthSelection,
    token: Option<&str>,
) -> bool {
    let Some(token) = token.map(str::trim).filter(|token| !token.is_empty()) else {
        return false;
    };
    selection.authority_id.as_deref() == Some(claude_environment_token_authority_id(token).as_str())
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
    #[allow(unused_mut)] // macOS also accepts the legacy credentials-file identity.
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
        let home = dirs::home_dir().ok_or_else(|| {
            "cannot determine the home directory for Claude Code's Keychain profile".to_string()
        })?;
        let configured = std::env::var_os("CLAUDE_CONFIG_DIR")
            .filter(|value| !value.is_empty())
            .map(std::path::PathBuf::from);
        let custom_oauth = std::env::var("CLAUDE_CODE_CUSTOM_OAUTH_URL")
            .ok()
            .is_some_and(|value| !value.trim().is_empty());
        macos_platform_login_source_id(&home, configured, custom_oauth)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let home = dirs::home_dir().ok_or_else(|| {
            "cannot determine the home directory for Claude Code's credentials-file profile"
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
fn macos_platform_login_source_id(
    home: &std::path::Path,
    configured: Option<std::path::PathBuf>,
    custom_oauth: bool,
) -> Result<String, String> {
    let config_dir_overridden = configured.is_some();
    let config_dir = match configured {
        Some(path) => std::path::absolute(&path).map_err(|error| {
            format!(
                "cannot identify Claude Code's Keychain profile at {}: {error}",
                path.display()
            )
        })?,
        None => home.join(".claude"),
    };
    let service =
        claude_code_macos_keychain_service(&config_dir, config_dir_overridden, custom_oauth);
    Ok(macos_keychain_claude_auth_source_id(&service))
}

#[cfg(target_os = "macos")]
fn claude_config_dir_for_source_id() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| {
        "cannot determine the home directory for Claude Code's profile".to_string()
    })?;
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
    let callback = Arc::new(move |event| match event {
        ClaudeCodeLoginBackendEvent::Ready {
            verification_url,
            input_tx,
        } => app_event_tx.send(AppEvent::ClaudeCodePlanLoginReady {
            verification_url,
            input_tx,
        }),
        ClaudeCodeLoginBackendEvent::Finished { result } => {
            app_event_tx.send(AppEvent::ClaudeCodePlanLoginFinished {
                result: result
                    .map(|result| result.map_err(ClaudeCodeLoginBackendError::into_message)),
            });
        }
    });
    start_with_callback_and_executable(
        executable,
        health_executable,
        codex_home,
        ClaudeCodeIdentityPolicy::AllowExplicitChange,
        callback,
    )
}

#[allow(dead_code)] // Consumed by the hidden PF-52 adapter before PF-53/PF-54 host adoption.
pub(crate) fn start_with_callback(
    codex_home: std::path::PathBuf,
    identity_policy: ClaudeCodeIdentityPolicy,
    callback: Arc<dyn Fn(ClaudeCodeLoginBackendEvent) + Send + Sync>,
) -> mpsc::UnboundedSender<ClaudeCodeLoginInput> {
    start_with_callback_and_executable(
        Path::new("claude"),
        /*health_executable*/ None,
        codex_home,
        identity_policy,
        callback,
    )
}

fn start_with_callback_and_executable(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: std::path::PathBuf,
    identity_policy: ClaudeCodeIdentityPolicy,
    callback: Arc<dyn Fn(ClaudeCodeLoginBackendEvent) + Send + Sync>,
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
            identity_policy,
            callback.clone(),
            task_input_tx,
            input_rx,
        )
        .await;
        callback(ClaudeCodeLoginBackendEvent::Finished { result });
    });
    input_tx
}

async fn load_expected_claude_code_identity(
    codex_home: &Path,
    identity_policy: ClaudeCodeIdentityPolicy,
) -> Result<Option<ExpectedClaudeCodeIdentity>, ClaudeCodeLoginBackendError> {
    if identity_policy == ClaudeCodeIdentityPolicy::AllowExplicitChange {
        return Ok(None);
    }
    let codex_home = codex_home.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let selection = Vault::new(codex_home)
            .load_claude_auth_selection()
            .map_err(|error| {
                ClaudeCodeLoginBackendError::Other(format!(
                    "Could not load the selected Claude Code login: {error}"
                ))
            })?
            .ok_or(ClaudeCodeLoginBackendError::IdentityConflict)?;
        if selection.source != ClaudeAuthSource::ClaudeCodeLogin {
            return Err(ClaudeCodeLoginBackendError::IdentityConflict);
        }
        let authority_id = selection
            .authority_id
            .ok_or(ClaudeCodeLoginBackendError::IdentityConflict)?;
        Ok(Some(ExpectedClaudeCodeIdentity {
            source_id: selection.source_id,
            authority_id,
        }))
    })
    .await
    .map_err(|error| {
        ClaudeCodeLoginBackendError::Other(format!(
            "Could not prepare Claude Code reauthorization: {error}"
        ))
    })?
}

async fn run_login(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    identity_policy: ClaudeCodeIdentityPolicy,
    callback: Arc<dyn Fn(ClaudeCodeLoginBackendEvent) + Send + Sync>,
    input_tx: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
    mut input_rx: mpsc::UnboundedReceiver<ClaudeCodeLoginInput>,
) -> ClaudeCodeLoginBackendResult {
    let expected_identity =
        match load_expected_claude_code_identity(codex_home, identity_policy).await {
            Ok(expected) => expected,
            Err(error) => return Some(Err(error)),
        };
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
            )
            .into()));
        }
    };

    let Some(stdout) = child.stdout.take() else {
        return Some(Err("Claude Code login did not provide an output stream."
            .to_string()
            .into()));
    };
    let Some(mut stdin) = child.stdin.take() else {
        return Some(Err("Claude Code login did not provide an input stream."
            .to_string()
            .into()));
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
                )
                .into()));
            }
            Some(line) => {
                if line.len() > MAX_LOGIN_LINE_BYTES {
                    let _ = child.kill().await;
                    return Some(Err(
                        "Claude Code login returned an unexpectedly large browser URL."
                            .to_string()
                            .into(),
                    ));
                }
                if let Some(url) = extract_https_url(&line) {
                    break url;
                }
            }
        }
    };

    drop(output_rx);
    callback(ClaudeCodeLoginBackendEvent::Ready {
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
    let submit_result = async {
        stdin.write_all(authorization_code.as_bytes()).await?;
        stdin.write_all(b"\n").await
    }
    .await;
    if let Err(err) = submit_result {
        let _ = child.kill().await;
        return Some(Err(format!(
            "Could not submit the authorization code to Claude Code: {err}"
        )
        .into()));
    }
    drop(stdin);

    let status = match timeout(LOGIN_TIMEOUT, async {
        loop {
            tokio::select! {
                biased;
                input = input_rx.recv() => match input {
                    Some(ClaudeCodeLoginInput::Cancel) | None => return None,
                    Some(ClaudeCodeLoginInput::AuthorizationCode(_)) => continue,
                },
                status = child.wait() => return Some(status),
            }
        }
    })
    .await
    {
        Ok(Some(Ok(status))) => status,
        Ok(Some(Err(err))) => {
            return Some(Err(
                format!("Claude Code login failed to finish: {err}").into()
            ));
        }
        Ok(None) => {
            let _ = child.kill().await;
            return None;
        }
        Err(_) => {
            let _ = child.kill().await;
            return Some(Err(ClaudeCodeLoginBackendError::TimedOut));
        }
    };
    if !status.success() {
        return Some(Err(
            format!("Claude Code rejected the login ({status}).").into()
        ));
    }

    let verification = verify_login(
        executable,
        health_executable,
        codex_home,
        expected_identity.as_ref(),
    );
    tokio::pin!(verification);
    loop {
        tokio::select! {
            biased;
            input = input_rx.recv() => match input {
                Some(ClaudeCodeLoginInput::Cancel) | None => return None,
                Some(ClaudeCodeLoginInput::AuthorizationCode(_)) => continue,
            },
            result = &mut verification => return Some(result),
        }
    }
}

fn spawn_output_reader(
    output: impl AsyncRead + Unpin + Send + 'static,
    output_tx: mpsc::UnboundedSender<String>,
) {
    tokio::spawn(async move {
        let mut output = BufReader::new(output);
        loop {
            match read_bounded_output_line(&mut output).await {
                Ok(None) | Err(_) => return,
                Ok(Some(line)) => {
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

async fn read_bounded_output_line<R: AsyncRead + Unpin>(
    output: &mut BufReader<R>,
) -> std::io::Result<Option<String>> {
    let mut bytes = Vec::with_capacity(1024);
    loop {
        let available = output.fill_buf().await?;
        if available.is_empty() {
            if bytes.is_empty() {
                return Ok(None);
            }
            break;
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let chunk_len = newline.map_or(available.len(), |index| index + 1);
        let remaining = (MAX_LOGIN_LINE_BYTES + 1).saturating_sub(bytes.len());
        let copied = chunk_len.min(remaining);
        bytes.extend_from_slice(&available[..copied]);
        output.consume(copied);

        if bytes.len() > MAX_LOGIN_LINE_BYTES || newline.is_some() {
            break;
        }
    }

    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}

async fn verify_login(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    expected_identity: Option<&ExpectedClaudeCodeIdentity>,
) -> Result<String, ClaudeCodeLoginBackendError> {
    verify_login_with_timeout(
        executable,
        health_executable,
        codex_home,
        LOGIN_HEALTH_TIMEOUT,
        expected_identity,
    )
    .await
}

async fn verify_login_with_timeout(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    verification_timeout: Duration,
    expected_identity: Option<&ExpectedClaudeCodeIdentity>,
) -> Result<String, ClaudeCodeLoginBackendError> {
    tokio::time::timeout(
        verification_timeout,
        verify_login_inner(
            executable,
            health_executable,
            codex_home,
            verification_timeout,
            expected_identity,
        ),
    )
    .await
    .map_err(|_| {
        ClaudeCodeLoginBackendError::Other(
            "Could not verify Claude Code login before the health check timed out.".to_string(),
        )
    })?
}

async fn verify_login_inner(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    verification_timeout: Duration,
    expected_identity: Option<&ExpectedClaudeCodeIdentity>,
) -> Result<String, ClaudeCodeLoginBackendError> {
    match status_with_timeout(executable, verification_timeout).await {
        status @ ClaudeCodePlanStatus::SignedIn { .. } => {
            let authority_id =
                status_authority_id(&status).map_err(ClaudeCodeLoginBackendError::from)?;
            let source_id = verify_current_platform_login_health(
                health_executable,
                verification_timeout,
                /*selected_source_id*/ None,
            )
            .await
            .map_err(|error| ClaudeCodeLoginBackendError::Other(error.to_string()))?;
            if expected_identity.is_some_and(|expected| {
                expected.source_id != source_id || expected.authority_id != authority_id
            }) {
                return Err(ClaudeCodeLoginBackendError::IdentityConflict);
            }
            persist_claude_code_login_selection(
                codex_home,
                source_id,
                authority_id,
                expected_identity,
            )
            .await?;
            Ok("Claude Code login selected. Retry the Claude Plan request or choose a model from /model.".to_string())
        }
        ClaudeCodePlanStatus::Checking
        | ClaudeCodePlanStatus::ManagedToken { .. }
        | ClaudeCodePlanStatus::EnvironmentToken { .. }
        | ClaudeCodePlanStatus::SelectionRequired { .. }
        | ClaudeCodePlanStatus::InvalidSelection
        | ClaudeCodePlanStatus::NeedsReauthorization
        | ClaudeCodePlanStatus::SignedOut
        | ClaudeCodePlanStatus::Unavailable => Err(ClaudeCodeLoginBackendError::Other(
            "Claude Code is not signed in with a Claude subscription after login.".to_string(),
        )),
        ClaudeCodePlanStatus::Error => Err(ClaudeCodeLoginBackendError::Other(
            "Could not verify Claude Code login status before the health check timed out."
                .to_string(),
        )),
    }
}

pub(crate) async fn select_existing_claude_code_login(
    executable: &Path,
    health_executable: Option<&Path>,
    codex_home: &Path,
    status_timeout: Duration,
) -> Result<bool, String> {
    match status_with_timeout(executable, status_timeout).await {
        status @ ClaudeCodePlanStatus::SignedIn { .. } => {
            let authority_id = status_authority_id(&status)?;
            let source_id = verify_current_platform_login_health(
                health_executable,
                LOGIN_HEALTH_TIMEOUT,
                /*selected_source_id*/ None,
            )
            .await
            .map_err(|error| error.to_string())?;
            persist_claude_code_login_selection(
                codex_home,
                source_id,
                authority_id,
                /*expected_identity*/ None,
            )
            .await
            .map_err(ClaudeCodeLoginBackendError::into_message)?;
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
) -> Result<String, PlatformLoginHealthCheckError> {
    let resolved_executable;
    let executable = match executable {
        Some(executable) => executable,
        None => {
            resolved_executable = internal_cli_helper_executable().map_err(|error| {
                PlatformLoginHealthCheckError::Undetermined(format!(
                    "Could not locate Corbanu to verify Claude Code credentials: {error}"
                ))
            })?;
            resolved_executable.as_path()
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
    .map_err(|_| {
        PlatformLoginHealthCheckError::Undetermined(
            "Timed out while verifying Claude Code's stored credentials".to_string(),
        )
    })?
    .map_err(|error| {
        PlatformLoginHealthCheckError::Undetermined(format!(
            "Could not verify Claude Code's stored credentials: {error}"
        ))
    })?;
    if !output.status.success() {
        return Err(PlatformLoginHealthCheckError::NeedsReauthorization(
            "Claude Code reports a login, but its stored credential needs reauthorization. Run `claude auth login` again; your previous Corbanu method remains selected."
                .to_string(),
        ));
    }
    let source_id = String::from_utf8(output.stdout)
        .map_err(|_| {
            PlatformLoginHealthCheckError::Undetermined(
                "Claude credential verification returned invalid metadata".to_string(),
            )
        })?
        .trim()
        .to_string();
    let accepted = accepted_platform_login_source_ids()
        .map_err(PlatformLoginHealthCheckError::Undetermined)?;
    if selected_source_id.is_some_and(|selected| selected != source_id) {
        return Err(PlatformLoginHealthCheckError::IdentityMismatch(
            "Claude credential verification returned a different platform profile; the previous method remains selected."
                .to_string(),
        ));
    }
    if !accepted.contains(&source_id) {
        return Err(PlatformLoginHealthCheckError::NeedsReauthorization(
            "Claude credential verification returned a different platform profile; the previous method remains selected."
                .to_string(),
        ));
    }
    Ok(source_id)
}

pub(crate) async fn selected_claude_recovery_source(
    codex_home: std::path::PathBuf,
) -> codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource {
    tokio::task::spawn_blocking(move || {
        Vault::new(codex_home)
            .load_claude_auth_selection()
            .ok()
            .flatten()
            .map(|selection| match selection.source {
                ClaudeAuthSource::ManagedSubscriptionToken => {
                    codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource::ManagedToken
                }
                ClaudeAuthSource::EnvironmentToken => {
                    codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource::Environment
                }
                ClaudeAuthSource::ClaudeCodeLogin => {
                    codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource::ClaudeCodeLogin
                }
            })
            .unwrap_or(
                codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource::Unknown,
            )
    })
    .await
    .unwrap_or(
        codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource::Unknown,
    )
}

async fn persist_claude_code_login_selection(
    codex_home: &Path,
    source_id: String,
    authority_id: String,
    expected_identity: Option<&ExpectedClaudeCodeIdentity>,
) -> Result<(), ClaudeCodeLoginBackendError> {
    let codex_home = codex_home.to_path_buf();
    let expected_identity = expected_identity.cloned();
    tokio::task::spawn_blocking(move || {
        persist_claude_code_login_selection_blocking(
            &codex_home,
            source_id,
            authority_id,
            expected_identity.as_ref(),
        )
    })
        .await
        .map_err(|error| {
            ClaudeCodeLoginBackendError::Other(format!(
                "Claude Code login succeeded, but Corbanu could not finish selecting it: {error}. Your previous method remains selected; retry from Providers."
            ))
        })?
}

fn persist_claude_code_login_selection_blocking(
    codex_home: &Path,
    source_id: String,
    authority_id: String,
    expected_identity: Option<&ExpectedClaudeCodeIdentity>,
) -> Result<(), ClaudeCodeLoginBackendError> {
    let vault = Vault::new(codex_home.to_path_buf());
    if let Some(expected) = expected_identity {
        let current = vault.load_claude_auth_selection().map_err(|error| {
            ClaudeCodeLoginBackendError::Other(format!(
                "Could not recheck the selected Claude Code login: {error}"
            ))
        })?;
        let still_selected = current.as_ref().is_some_and(|selection| {
            selection.source == ClaudeAuthSource::ClaudeCodeLogin
                && selection.source_id == expected.source_id
                && selection.authority_id.as_deref() == Some(expected.authority_id.as_str())
        });
        if !still_selected {
            return Err(ClaudeCodeLoginBackendError::IdentityConflict);
        }
    }
    let selection =
        ClaudeAuthSelection::new_claude_code_login(source_id, authority_id).map_err(|error| {
            ClaudeCodeLoginBackendError::Other(format!(
                "Could not select the current Claude Code login: {error}"
            ))
        })?;
    vault
        .save_claude_auth_selection(&selection)
        .map_err(|error| {
            ClaudeCodeLoginBackendError::Other(format!(
                "Claude Code login succeeded, but Corbanu could not select it: {error}. Your previous method remains selected; retry from Providers."
            ))
        })
}

fn auth_method_choice_params() -> SelectionViewParams {
    let mut params = SelectionViewParams {
        view_id: Some(CLAUDE_AUTH_METHOD_VIEW_ID),
        items: vec![
            SelectionItem {
                name: super::claude_auth_presentation::MANAGED_TOKEN_METHOD_NAME.to_string(),
                description: Some(
                    super::claude_auth_presentation::MANAGED_TOKEN_METHOD_DESCRIPTION.to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::RunClaudeSetupToken))],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: super::claude_auth_presentation::CLAUDE_CODE_LOGIN_METHOD_NAME.to_string(),
                description: Some(
                    super::claude_auth_presentation::CLAUDE_CODE_LOGIN_METHOD_DESCRIPTION
                        .to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::UseClaudeCodePlanLogin))],
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    super::claude_auth_presentation::apply_method_choice_copy(&mut params);
    params
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
    let config_dir_override = std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from)
        .map(std::path::absolute)
        .transpose()?;
    let custom_oauth_url = std::env::var("CLAUDE_CODE_CUSTOM_OAUTH_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(std::ffi::OsString::from);
    read_status_with_profile(
        executable,
        config_dir_override.as_deref(),
        custom_oauth_url.as_deref(),
    )
    .await
}

async fn read_status_with_profile(
    executable: &Path,
    config_dir_override: Option<&Path>,
    custom_oauth_url: Option<&std::ffi::OsStr>,
) -> std::io::Result<ClaudeCodePlanStatus> {
    let neutral_cwd = tempfile::tempdir()?;
    let mut command = Command::new(executable);
    command
        .current_dir(neutral_cwd.path())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if let Some(config_dir) = config_dir_override {
        command.env("CLAUDE_CONFIG_DIR", config_dir);
    } else {
        command.env_remove("CLAUDE_CONFIG_DIR");
    }
    for name in CLAUDE_STATUS_ENV_REMOVE {
        command.env_remove(name);
    }
    command.env_remove("CLAUDE_CODE_CUSTOM_OAUTH_URL");
    if let Some(custom_oauth_url) = custom_oauth_url {
        command.env("CLAUDE_CODE_CUSTOM_OAUTH_URL", custom_oauth_url);
    }
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
        organization_id: status
            .get("orgId")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        subscription: status
            .get("subscriptionType")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    })
}

fn status_authority_id(status: &ClaudeCodePlanStatus) -> Result<String, String> {
    let ClaudeCodePlanStatus::SignedIn {
        email: Some(email),
        organization_id,
        subscription,
    } = status
    else {
        return Err(
            "Claude Code did not report a complete account identity; run `claude auth login` again."
                .to_string(),
        );
    };
    claude_login_authority_id(email, organization_id.as_deref(), subscription.as_deref())
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
            super::claude_auth_presentation::MANAGED_TOKEN_ENTRY_TITLE.to_string(),
            super::claude_auth_presentation::MANAGED_TOKEN_ENTRY_LABEL.to_string(),
            super::claude_auth_presentation::MANAGED_TOKEN_ENTRY_GUIDANCE.to_string(),
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
                let _ = submit_tx.send(ClaudeCodeLoginInput::AuthorizationCode(
                    Zeroizing::new(code),
                ));
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
