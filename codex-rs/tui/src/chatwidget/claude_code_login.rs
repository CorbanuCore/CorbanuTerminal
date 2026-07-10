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
use crate::app_event_sender::AppEventSender;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::ViewCompletion;
use crate::render::renderable::Renderable;

const CLAUDE_CODE_LOGIN_VIEW_ID: &str = "claude-code-plan-login";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const MAX_LOGIN_LINE_BYTES: usize = 16 * 1024;

pub(crate) enum ClaudeCodeLoginInput {
    AuthorizationCode(String),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClaudeCodePlanStatus {
    Checking,
    SignedIn {
        email: Option<String>,
        subscription: Option<String>,
    },
    SignedOut,
    Unavailable,
    Error,
}

pub(crate) async fn current_status() -> ClaudeCodePlanStatus {
    read_status(Path::new("claude"))
        .await
        .unwrap_or_else(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                ClaudeCodePlanStatus::Unavailable
            } else {
                ClaudeCodePlanStatus::Error
            }
        })
}

pub(crate) fn start(app_event_tx: AppEventSender) -> mpsc::UnboundedSender<ClaudeCodeLoginInput> {
    start_with_executable(app_event_tx, Path::new("claude"))
}

fn start_with_executable(
    app_event_tx: AppEventSender,
    executable: &Path,
) -> mpsc::UnboundedSender<ClaudeCodeLoginInput> {
    let (input_tx, input_rx) = mpsc::unbounded_channel();
    let task_input_tx = input_tx.clone();
    let executable = executable.to_path_buf();
    tokio::spawn(async move {
        let result = run_login(&executable, app_event_tx.clone(), task_input_tx, input_rx).await;
        app_event_tx.send(AppEvent::ClaudeCodePlanLoginFinished { result });
    });
    input_tx
}

async fn run_login(
    executable: &Path,
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

    Some(verify_login(executable).await)
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

async fn verify_login(executable: &Path) -> Result<String, String> {
    match read_status(executable).await {
        Ok(ClaudeCodePlanStatus::SignedIn { .. }) => Ok(
            "Claude Code plan login complete. Choose a Claude Plan model from /model.".to_string(),
        ),
        Ok(_) => {
            Err("Claude Code is not signed in with a Claude subscription after login.".to_string())
        }
        Err(err) => Err(format!("Could not verify Claude Code login: {err}")),
    }
}

async fn read_status(executable: &Path) -> std::io::Result<ClaudeCodePlanStatus> {
    let output = Command::new(executable)
        .args(["auth", "status", "--json"])
        .output()
        .await?;
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
                self.add_error_message(format!("Claude Code plan login failed: {message}"))
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
