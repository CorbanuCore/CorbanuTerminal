//! Running Claude Code command plans and producing turn outputs.

use std::io::Write as _;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

#[cfg(unix)]
use std::collections::BTreeSet;
#[cfg(target_os = "linux")]
use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::Command;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroize;
use zeroize::Zeroizing;

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;

use super::approval;
use super::bridge::run_claude_bridge;
use super::output_parse::ParsedClaudeOutput;
use super::output_parse::parse_claude_output;
use super::pane::ClaudePaneTurnStatus;
use super::pane::ClaudePaneUsageStatus;
use super::progress::dedupe_tool_names;
use super::progress::elapsed_ms;
use super::progress::emit_claude_progress;
use super::progress::progress_key;
use super::progress::progresses_from_claude_value;
use super::progress::reasoning_events_from_stdout;
use super::progress::tool_events_from_stdout;
use super::progress::truncate_for_display;
use super::progress::turn_usage_summary_from_stdout;
use super::progress::unix_epoch_ms;
use super::progress::usage_status_from_summary;
use super::turn_types::ClaudeCommandPlan;
use super::turn_types::ClaudePaneTurnAudit;
use super::turn_types::ClaudePaneTurnOutput;
use super::turn_types::DeferredClaudePlanAuth;
use super::turn_types::PreparedClaudePaneTurn;

const CLAUDE_PANE_PROGRESS_HEARTBEAT: Duration = Duration::from_secs(30);
const CLAUDE_PLAN_AUTH_TIMEOUT: Duration = Duration::from_secs(60);
const CLAUDE_SECRET_REDACTION: &str = "[REDACTED_SECRET]";
const MIN_REDACTED_SECRET_LEN: usize = 8;
pub(crate) async fn run_prepared_claude_turn(
    prepared: PreparedClaudePaneTurn,
    progress_tx: Option<AppEventSender>,
) -> Result<ClaudePaneTurnOutput, String> {
    run_claude_command_plan(prepared.plan, prepared.cancel_token, progress_tx)
        .await
        .map_err(|err| format!("{err:#}"))
}

pub(crate) async fn run_claude_command_plan(
    mut plan: ClaudeCommandPlan,
    cancel_token: CancellationToken,
    progress_tx: Option<AppEventSender>,
) -> Result<ClaudePaneTurnOutput> {
    // Every Claude agent turn starts here, including restored and spawned panes.
    if let Some(reason) = crate::security::level::external_agent_block_reason() {
        return Err(anyhow!(reason));
    }
    // #218: a contained turn is checked and wrapped in the OS sandbox before
    // any credential is read or the bridge starts, so a launch the contract
    // refuses never touches the vault or the provider.
    let contained = match plan.containment.as_ref() {
        Some(containment) => {
            require_contained_claude_version(&plan.executable).await?;
            let bridge_port = plan
                .bridge
                .as_ref()
                .map(|bridge| bridge.bind_addr.port())
                .ok_or_else(|| anyhow!("a contained Claude pane needs its bridge"))?;
            Some(super::containment::contain(
                containment,
                &plan.executable,
                &plan.args,
                &plan.env,
                &plan.cwd,
                bridge_port,
            )?)
        }
        None => None,
    };
    let started_at = Instant::now();
    let started_at_unix_ms = unix_epoch_ms();
    let mut last_progress_elapsed_ms = Some(0);
    let claude_config_dir_override = plan
        .deferred_claude_plan_auth
        .as_ref()
        .and_then(|deferred| deferred.claude_config_dir_override.clone());
    let claude_plan_token = if let Some(deferred) = plan.deferred_claude_plan_auth.take() {
        let token = tokio::select! {
            _ = cancel_token.cancelled() => {
                let ended_at_unix_ms = unix_epoch_ms();
                let output = failed_turn_output(
                    &plan,
                    elapsed_ms(&started_at),
                    ClaudePaneTurnStatus::Interrupted,
                    Some("interrupted_during_auth".to_string()),
                    "Claude pane turn interrupted before authentication completed.".to_string(),
                );
                report_direct_turn(progress_tx.as_ref(), &output);
                write_turn_audit(
                    &plan,
                    &output,
                    started_at_unix_ms,
                    ended_at_unix_ms,
                    last_progress_elapsed_ms,
                )?;
                return Ok(output);
            }
            result = resolve_deferred_claude_plan_token(deferred) => result?,
        };
        Some(token)
    } else {
        None
    };
    if let Some(token) = claude_plan_token.as_deref() {
        let bridge = plan.bridge.as_mut().ok_or_else(|| {
            anyhow!("Claude Plan credential broker is missing its loopback bridge")
        })?;
        if bridge.upstream_api_key.is_some() || bridge.deferred_vault_secret.is_some() {
            return Err(anyhow!(
                "Claude Plan credential broker has conflicting credential sources"
            ));
        }
        bridge.upstream_api_key = Some(token.to_string());
    }
    if let Some(bridge) = plan.bridge.as_mut()
        && bridge.upstream_api_key.is_none()
    {
        let deferred = bridge
            .deferred_vault_secret
            .take()
            .ok_or_else(|| anyhow!("Claude bridge is missing provider credentials"))?;
        let secret = tokio::task::spawn_blocking(move || {
            super::command_plan::reveal_provider_secret(&deferred.codex_home, &deferred.label)
        })
        .await
        .context("Claude bridge credential task failed")??;
        bridge.upstream_api_key = Some(secret);
    }
    emit_claude_progress(
        &progress_tx,
        &plan,
        &started_at,
        "starting",
        "Claude pane starting.",
        Some(format!(
            "mode: {}; artifact: {}; audit: {}",
            plan.command_mode.label(),
            plan.artifact_path.display(),
            plan.audit_path.display()
        )),
    );
    let redactor = ClaudeSecretRedactor::from_plan(&plan, /*additional_secret*/ None);
    let mut command = match contained {
        Some(contained) => {
            let (program, args) = contained
                .argv
                .split_first()
                .ok_or_else(|| anyhow!("the sandboxed Claude command is empty"))?;
            let mut command = Command::new(program);
            command.args(args).env_clear().envs(&contained.env);
            #[cfg(unix)]
            if let Some(arg0) = contained.arg0.as_deref() {
                command.arg0(arg0);
            }
            command
        }
        None => {
            let mut command = Command::new(&plan.executable);
            for key in &plan.env_remove {
                command.env_remove(key);
            }
            if let Some(config_dir) = claude_config_dir_override {
                command.env("CLAUDE_CONFIG_DIR", config_dir);
            }
            command.args(&plan.args).envs(&plan.env);
            command
        }
    };
    let bridge_handle = plan
        .bridge
        .take()
        .map(|bridge| tokio::spawn(run_claude_bridge(bridge, progress_tx.clone())));
    command.kill_on_drop(true);
    #[cfg(unix)]
    {
        command.process_group(0);
    }
    let mut child = command
        .current_dir(&plan.cwd)
        .stdin(if plan.stdin_prompt.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run `{}`", plan.executable))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Claude stdout pipe was not available"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("Claude stderr pipe was not available"))?;
    // Contained turns (#218): the prompt and every tool approval go through
    // Claude Code's stdin; it is closed once the turn's result arrives.
    let stdin = ClaudeStdin::spawn(child.stdin.take());
    if let Some(prompt) = plan.stdin_prompt.as_deref() {
        stdin.send(approval::user_message(prompt));
    }
    // Requests still waiting for a person stop waiting, and their popups
    // close, when the turn ends however it ends.
    let mut approvals = TurnApprovals::new(cancel_token.child_token());
    let _approvals_end = approvals.turn.clone().drop_guard();
    let mut protocol_error: Option<String> = None;
    let mut results = 0usize;
    let stderr_task = tokio::spawn(async move {
        let mut stderr_reader = BufReader::new(stderr);
        let mut stderr_text = String::new();
        let _ = stderr_reader.read_to_string(&mut stderr_text).await;
        stderr_text
    });

    let mut artifact = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&plan.artifact_path)
        .with_context(|| {
            format!(
                "failed to open Claude pane artifact `{}`",
                plan.artifact_path.display()
            )
        })?;
    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stdout_text = String::new();
    let mut heartbeat = interval(CLAUDE_PANE_PROGRESS_HEARTBEAT);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    heartbeat.tick().await;
    let mut timed_out = false;
    let mut interrupted = false;
    let mut cleanup_error: Option<String> = None;
    let mut last_progress_key: Option<String> = None;
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                interrupted = true;
                if let Err(err) = stop_claude_child(&mut child).await {
                    cleanup_error = Some(err.to_string());
                }
                break;
            }
            line = stdout_lines.next_line() => {
                let Some(line) = line.context("failed to read Claude stdout")? else {
                    break;
                };
                let line = redactor.redact(&line);
                stdout_text.push_str(&line);
                stdout_text.push('\n');
                use std::io::Write as _;
                writeln!(artifact, "{line}").with_context(|| {
                    format!(
                        "failed to append Claude pane artifact `{}`",
                        plan.artifact_path.display()
                    )
                })?;
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    // The line was redacted before parsing, so a tool runs
                    // with the redacted input, which is also what the person
                    // was shown.
                    let message = approval::control_message(&value);
                    let is_control = message.is_some();
                    if let Some(message) = message {
                        if let Err(err) =
                            approvals.handle(message, &plan, &stdin, progress_tx.as_ref(), &redactor)
                        {
                            protocol_error = Some(err);
                        }
                    } else if value.get("type").and_then(Value::as_str) == Some("result") {
                        results += 1;
                        if results > 1 {
                            protocol_error =
                                Some("Claude Code reported a second result for one turn".to_string());
                        }
                        stdin.close();
                    }
                    if protocol_error.is_some() {
                        if let Err(err) = stop_claude_child(&mut child).await {
                            cleanup_error = Some(err.to_string());
                        }
                        break;
                    }
                    if is_control {
                        continue;
                    }
                    for progress in progresses_from_claude_value(&plan, &started_at, &value) {
                        last_progress_elapsed_ms = Some(progress.elapsed_ms);
                        let is_assistant_text = progress.phase == "assistant-text";
                        let key = progress_key(&progress);
                        if (is_assistant_text || last_progress_key.as_deref() != Some(key.as_str()))
                            && let Some(tx) = progress_tx.as_ref()
                        {
                            tx.send(AppEvent::ClaudePaneTurnProgress { progress });
                        }
                        if !is_assistant_text {
                            last_progress_key = Some(key);
                        }
                    }
                }
            }
            _ = heartbeat.tick() => {
                last_progress_elapsed_ms = Some(elapsed_ms(&started_at));
                emit_claude_progress(
                    &progress_tx,
                    &plan,
                    &started_at,
                    "waiting",
                    "Claude running.",
                    /*hint*/ None,
                );
            }
        }
    }
    approvals.turn.cancel();
    artifact.flush().with_context(|| {
        format!(
            "failed to flush Claude pane artifact `{}`",
            plan.artifact_path.display()
        )
    })?;

    let wait_result = if timed_out || interrupted || protocol_error.is_some() {
        None
    } else {
        Some(
            tokio::time::timeout(Duration::from_secs(5), child.wait())
                .await
                .unwrap_or_else(|_| {
                    timed_out = true;
                    Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Claude process did not exit after stdout closed",
                    ))
                }),
        )
    };
    if timed_out && let Err(err) = stop_claude_child(&mut child).await {
        cleanup_error = Some(err.to_string());
    }
    let stderr = if timed_out || interrupted || protocol_error.is_some() {
        stderr_task.abort();
        String::new()
    } else {
        redactor.redact(&stderr_task.await.unwrap_or_default())
    };
    if let Some(handle) = bridge_handle {
        handle.abort();
    }
    let duration_ms = elapsed_ms(&started_at);
    let ended_at_unix_ms = unix_epoch_ms();

    if let Some(err) = cleanup_error {
        let output = partial_failed_turn_output(
            &plan,
            duration_ms,
            ClaudePaneTurnStatus::ProviderError,
            Some("cleanup_failed".to_string()),
            format!(
                "Claude pane process cleanup failed after interrupt/timeout; the turn is not considered safely stopped: {err}"
            ),
            &stdout_text,
        );
        report_direct_turn(progress_tx.as_ref(), &output);
        write_turn_audit(
            &plan,
            &output,
            started_at_unix_ms,
            ended_at_unix_ms,
            last_progress_elapsed_ms,
        )?;
        return Ok(output);
    }

    if let Some(err) = protocol_error {
        let output = partial_failed_turn_output(
            &plan,
            duration_ms,
            ClaudePaneTurnStatus::ProviderError,
            Some("protocol_error".to_string()),
            format!("Claude pane turn stopped: {err}"),
            &stdout_text,
        );
        report_direct_turn(progress_tx.as_ref(), &output);
        write_turn_audit(
            &plan,
            &output,
            started_at_unix_ms,
            ended_at_unix_ms,
            last_progress_elapsed_ms,
        )?;
        return Ok(output);
    }

    if interrupted {
        let output = partial_failed_turn_output(
            &plan,
            duration_ms,
            ClaudePaneTurnStatus::Interrupted,
            Some("interrupted".to_string()),
            "Claude pane turn interrupted by user.".to_string(),
            &stdout_text,
        );
        report_direct_turn(progress_tx.as_ref(), &output);
        write_turn_audit(
            &plan,
            &output,
            started_at_unix_ms,
            ended_at_unix_ms,
            last_progress_elapsed_ms,
        )?;
        return Ok(output);
    }

    if timed_out {
        let output = partial_failed_turn_output(
            &plan,
            duration_ms,
            ClaudePaneTurnStatus::TimeoutPause,
            Some("process_wait_timeout".to_string()),
            "Claude stdout closed, but the Claude process did not exit within the cleanup grace period. Type `continue` in this pane to resume if a Claude session id was captured.".to_string(),
            &stdout_text,
        );
        report_direct_turn(progress_tx.as_ref(), &output);
        write_turn_audit(
            &plan,
            &output,
            started_at_unix_ms,
            ended_at_unix_ms,
            last_progress_elapsed_ms,
        )?;
        return Ok(output);
    }

    let status = match wait_result {
        Some(Ok(status)) => status,
        Some(Err(err)) => {
            let output = partial_failed_turn_output(
                &plan,
                duration_ms,
                ClaudePaneTurnStatus::ProviderError,
                Some("process_wait".to_string()),
                format!("failed to wait for Claude process: {err}"),
                &stdout_text,
            );
            report_direct_turn(progress_tx.as_ref(), &output);
            write_turn_audit(
                &plan,
                &output,
                started_at_unix_ms,
                ended_at_unix_ms,
                last_progress_elapsed_ms,
            )?;
            return Ok(output);
        }
        None => unreachable!("timeout branch returned earlier"),
    };

    if !stdout_text.trim().is_empty() {
        let output = match parse_claude_output(&stdout_text) {
            Ok(parsed) => turn_output_from_parsed(&plan, parsed, duration_ms),
            // A transcript this client cannot parse is still a turn that
            // spent money, and the line stating what it spent may be sitting
            // in that same stdout. Salvage it rather than drop the turn.
            Err(err) => partial_failed_turn_output(
                &plan,
                duration_ms,
                ClaudePaneTurnStatus::ParseFailure,
                Some("parse_failure".to_string()),
                format!("{err:#}"),
                &stdout_text,
            ),
        };
        report_direct_turn(progress_tx.as_ref(), &output);
        write_turn_audit(
            &plan,
            &output,
            started_at_unix_ms,
            ended_at_unix_ms,
            last_progress_elapsed_ms,
        )?;
        return Ok(output);
    }

    if !status.success() {
        let output = failed_turn_output(
            &plan,
            duration_ms,
            ClaudePaneTurnStatus::ProviderError,
            Some("process_exit".to_string()),
            format!(
                "Claude exited with status {}: {}",
                status,
                truncate_for_display(stderr.trim(), /*max_chars*/ 1_000)
            ),
        );
        report_direct_turn(progress_tx.as_ref(), &output);
        write_turn_audit(
            &plan,
            &output,
            started_at_unix_ms,
            ended_at_unix_ms,
            last_progress_elapsed_ms,
        )?;
        return Ok(output);
    }

    let output = failed_turn_output(
        &plan,
        duration_ms,
        ClaudePaneTurnStatus::ParseFailure,
        Some("empty_output".to_string()),
        "Claude returned empty output".to_string(),
    );
    report_direct_turn(progress_tx.as_ref(), &output);
    write_turn_audit(
        &plan,
        &output,
        started_at_unix_ms,
        ended_at_unix_ms,
        last_progress_elapsed_ms,
    )?;
    Ok(output)
}

/// Claude Code's stdin for a contained turn. One task owns the pipe and
/// writes messages in the order they are sent; `close` closes the pipe and
/// later messages are dropped.
#[derive(Clone)]
struct ClaudeStdin(tokio::sync::mpsc::UnboundedSender<Option<Value>>);

impl ClaudeStdin {
    fn spawn(writer: Option<ChildStdin>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Option<Value>>();
        if let Some(mut writer) = writer {
            tokio::spawn(async move {
                while let Some(Some(value)) = rx.recv().await {
                    if let Err(err) = write_claude_input(&mut writer, &value).await {
                        tracing::debug!(error = %err, "failed to write to Claude Code's stdin");
                        break;
                    }
                }
            });
        }
        Self(tx)
    }

    fn send(&self, value: Value) {
        let _ = self.0.send(Some(value));
    }

    fn close(&self) {
        let _ = self.0.send(None);
    }
}

async fn write_claude_input(writer: &mut ChildStdin, value: &Value) -> Result<()> {
    let mut line = serde_json::to_vec(value).context("failed to encode Claude input")?;
    line.push(b'\n');
    writer
        .write_all(&line)
        .await
        .context("failed to write Claude input")?;
    writer.flush().await.context("failed to flush Claude input")
}

/// How long a tool request waits for a person before it is denied.
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// The control requests of one turn. Each request id is answered at most
/// once: a repeated id (Claude Code never repeats one; anything else writing
/// to its stdout might) is dropped, so it cannot take over a pending
/// request's answer.
struct TurnApprovals {
    /// Cancelled when the turn ends; every waiting request stops.
    turn: CancellationToken,
    seen: std::collections::HashSet<String>,
    /// Pending requests, to stop on `control_cancel_request`.
    pending: std::collections::HashMap<String, CancellationToken>,
}

impl TurnApprovals {
    fn new(turn: CancellationToken) -> Self {
        Self {
            turn,
            seen: std::collections::HashSet::new(),
            pending: std::collections::HashMap::new(),
        }
    }

    /// Ask a person about a `can_use_tool` request and answer it on stdin;
    /// refuse every other request. With nobody to ask, deny. `Err` means the
    /// turn must end: a request nothing can answer.
    fn handle(
        &mut self,
        message: approval::ControlMessage<'_>,
        plan: &ClaudeCommandPlan,
        stdin: &ClaudeStdin,
        progress_tx: Option<&AppEventSender>,
        redactor: &ClaudeSecretRedactor,
    ) -> Result<(), String> {
        let (request_id, tool) = match message {
            approval::ControlMessage::Malformed => {
                return Err("Claude Code sent a control request without a request id".to_string());
            }
            approval::ControlMessage::Cancel { request_id } => {
                if let Some(cancel) = self.pending.get(request_id) {
                    cancel.cancel();
                }
                return Ok(());
            }
            approval::ControlMessage::Unsupported { request_id } => (request_id, None),
            approval::ControlMessage::ToolRequest {
                request_id,
                tool_name,
                tool_use_id,
                input,
            } => (request_id, Some((tool_name, tool_use_id, input))),
        };
        if !self.seen.insert(request_id.to_string()) {
            tracing::warn!(
                request_id,
                "dropped a repeated Claude Code control request id"
            );
            return Ok(());
        }
        let Some((tool_name, tool_use_id, input)) = tool else {
            stdin.send(approval::unsupported_response(request_id));
            return Ok(());
        };
        let request_id = request_id.to_string();
        let input = input.clone();
        let Some(tx) = progress_tx.cloned() else {
            stdin.send(approval::tool_response(
                &request_id,
                Err(approval::Denial::Person),
                &input,
            ));
            return Ok(());
        };
        let (responder, decision) = approval::ApprovalResponder::new();
        tx.send(AppEvent::ClaudePaneApprovalRequested(Box::new(
            approval::ClaudeApprovalRequest {
                pane_id: plan.pane_id.clone(),
                pane_title: plan.pane_title.clone(),
                cwd: plan.cwd.clone(),
                tool_name: tool_name.to_string(),
                tool_use_id: tool_use_id.map(str::to_string),
                details: approval::details(tool_name, &input, |text| redactor.redact(text)),
                responder,
            },
        )));
        let cancel = self.turn.child_token();
        self.pending.insert(request_id.clone(), cancel.clone());
        let stdin = stdin.clone();
        tokio::spawn(async move {
            let answer = wait_for_decision(decision, &cancel, APPROVAL_TIMEOUT).await;
            if let Some(answer) = answer {
                stdin.send(approval::tool_response(&request_id, answer, &input));
            }
            if !matches!(answer, Some(Ok(()) | Err(approval::Denial::Person))) {
                // Nobody answered: close the popup.
                tx.send(AppEvent::ClaudePaneApprovalsSettled);
            }
        });
        Ok(())
    }
}

/// A person's answer; a dropped popup denies. `None` when the request
/// stopped waiting (cancelled, or the turn ended) and needs no answer.
pub(super) async fn wait_for_decision(
    decision: tokio::sync::oneshot::Receiver<bool>,
    cancel: &CancellationToken,
    timeout: Duration,
) -> Option<Result<(), approval::Denial>> {
    tokio::select! {
        allow = decision => Some(if allow.unwrap_or(false) {
            Ok(())
        } else {
            Err(approval::Denial::Person)
        }),
        () = tokio::time::sleep(timeout) => Some(Err(approval::Denial::TimedOut)),
        () = cancel.cancelled() => None,
    }
}

/// The oldest Claude Code contained panes were checked with: it has stdio
/// permission prompts, `--safe-mode`, ignores `bypassPermissions` in
/// subagent definitions, and loads nothing a pane can plant once setting
/// sources are off (`claude_code_regression.py`).
const CONTAINED_CLAUDE_MIN_VERSION: (u64, u64, u64) = (2, 1, 292);

/// Refuses a contained launch with a Claude Code older than
/// [`CONTAINED_CLAUDE_MIN_VERSION`]. A version that passed is remembered.
async fn require_contained_claude_version(executable: &str) -> Result<()> {
    static PASSED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
    let passed = |executable: &str| {
        PASSED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(|known| known == executable)
    };
    if passed(executable) {
        return Ok(());
    }
    let mut command = Command::new(executable);
    command
        .arg("--version")
        .env_clear()
        .envs(
            ["PATH", "HOME"]
                .into_iter()
                .filter_map(|name| Some((name, std::env::var_os(name)?))),
        )
        .env("DISABLE_AUTOUPDATER", "1")
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .stdin(Stdio::null())
        .kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(30), command.output())
        .await
        .map_err(|_| anyhow!("`{executable} --version` did not finish"))?
        .with_context(|| format!("failed to run `{executable} --version`"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (major, minor, patch) = CONTAINED_CLAUDE_MIN_VERSION;
    match parse_claude_version(&text) {
        Some(version) if version >= CONTAINED_CLAUDE_MIN_VERSION => {
            PASSED
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(executable.to_string());
            Ok(())
        }
        found => Err(anyhow!(
            "contained Claude panes need Claude Code {major}.{minor}.{patch} or later; found {}. Update Claude Code and try again.",
            found.map_or_else(
                || "an unknown version".to_string(),
                |(a, b, c)| format!("{a}.{b}.{c}")
            )
        )),
    }
}

/// `2.1.292` from `2.1.292 (Claude Code)`.
pub(super) fn parse_claude_version(text: &str) -> Option<(u64, u64, u64)> {
    let token = text.split_whitespace().next()?;
    let mut parts = token.split('.').map(str::parse::<u64>);
    let version = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    );
    parts.next().is_none().then_some(version)
}

async fn resolve_deferred_claude_plan_token(
    deferred: DeferredClaudePlanAuth,
) -> Result<Zeroizing<String>> {
    let mut command = Command::new(&deferred.helper_executable);
    command
        .arg("internal-claude-oauth-token")
        .env("CORBANU_HOME", &deferred.codex_home)
        .env("PFTERMINAL_HOME", &deferred.codex_home)
        .env("CODEX_HOME", &deferred.codex_home)
        .env_remove("CLAUDE_CONFIG_DIR")
        .current_dir(&deferred.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if let Some(config_dir) = deferred.claude_config_dir_override {
        command.env("CLAUDE_CONFIG_DIR", config_dir);
    }
    let output = tokio::time::timeout(CLAUDE_PLAN_AUTH_TIMEOUT, command.output())
        .await
        .map_err(|_| anyhow!("selected Claude Plan authentication timed out"))?
        .context("failed to start selected Claude Plan authentication helper")?;
    let mut stdout = output.stdout;
    if !output.status.success() {
        stdout.zeroize();
        return Err(anyhow!(
            "selected Claude Plan authentication is unavailable; open Providers to recover"
        ));
    }
    let token = match std::str::from_utf8(&stdout) {
        Ok(token) if !token.trim().is_empty() => Zeroizing::new(token.to_string()),
        Ok(_) => {
            stdout.zeroize();
            return Err(anyhow!(
                "selected Claude Plan authentication returned no credential; open Providers to recover"
            ));
        }
        Err(_) => {
            stdout.zeroize();
            return Err(anyhow!(
                "selected Claude Plan authentication returned an invalid credential"
            ));
        }
    };
    stdout.zeroize();
    Ok(token)
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ClaudeSecretRedactor {
    secrets: Vec<Zeroizing<String>>,
}

impl ClaudeSecretRedactor {
    pub(crate) fn from_plan(plan: &ClaudeCommandPlan, additional_secret: Option<&str>) -> Self {
        let mut secrets = plan
            .bridge
            .as_ref()
            .into_iter()
            .flat_map(|bridge| {
                [
                    Some(bridge.client_auth_token.as_str()),
                    bridge.upstream_api_key.as_deref(),
                ]
            })
            .flatten()
            .filter(|secret| secret.len() >= MIN_REDACTED_SECRET_LEN)
            .map(|secret| Zeroizing::new(secret.to_string()))
            .collect::<Vec<_>>();
        if let Some(secret) = additional_secret.filter(|secret| !secret.is_empty()) {
            secrets.push(Zeroizing::new(secret.to_string()));
        }
        Self { secrets }
    }

    pub(crate) fn redact(&self, text: &str) -> String {
        let mut redacted = text.to_string();
        for secret in &self.secrets {
            redacted = redacted.replace(secret.as_str(), CLAUDE_SECRET_REDACTION);
        }
        redacted
    }
}

pub(crate) async fn stop_claude_child(child: &mut Child) -> Result<()> {
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        kill_claude_process_tree(pid as libc::pid_t)?;
    }

    if let Err(err) = child.start_kill()
        && err.kind() != std::io::ErrorKind::InvalidInput
    {
        return Err(anyhow!("failed to kill Claude process: {err}"));
    }

    match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(err)) => Err(anyhow!(
            "failed to wait for Claude process after kill: {err}"
        )),
        Err(_) => Err(anyhow!(
            "Claude process did not exit after SIGKILL within cleanup timeout"
        )),
    }
}

#[cfg(unix)]
fn kill_claude_process_tree(root_pid: libc::pid_t) -> Result<()> {
    // Claude launches some tool commands in their own session/process group. Killing only the
    // Claude process group therefore leaves those detached tools alive after the pane reports an
    // interrupt. Snapshot the Linux descendant tree before killing the root, and terminate every
    // discovered process group plus each concrete descendant as a race-resistant fallback.
    #[cfg(target_os = "linux")]
    let descendants = linux_descendant_pids(root_pid);
    #[cfg(not(target_os = "linux"))]
    let descendants: Vec<libc::pid_t> = Vec::new();

    let current_process_group = unsafe { libc::getpgrp() };
    let mut process_groups = BTreeSet::new();
    for pid in std::iter::once(root_pid).chain(descendants.iter().copied()) {
        let process_group = unsafe { libc::getpgid(pid) };
        if process_group > 0 {
            if process_group == current_process_group {
                return Err(anyhow!(
                    "refusing to kill Claude process tree {root_pid}: descendant {pid} shares Corbanu Terminal process group {current_process_group}"
                ));
            }
            process_groups.insert(process_group);
        } else {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ESRCH) {
                return Err(anyhow!(
                    "failed to inspect process group for Claude descendant {pid}: {err}"
                ));
            }
        }
    }

    for process_group in process_groups {
        send_sigkill(
            -process_group,
            &format!("Claude descendant process group {process_group}"),
        )?;
    }
    for pid in descendants.into_iter().rev() {
        send_sigkill(pid, &format!("Claude descendant process {pid}"))?;
    }
    Ok(())
}

#[cfg(unix)]
fn send_sigkill(target: libc::pid_t, label: &str) -> Result<()> {
    let kill_result = unsafe { libc::kill(target, libc::SIGKILL) };
    if kill_result == -1 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() != Some(libc::ESRCH) {
            return Err(anyhow!("failed to send SIGKILL to {label}: {err}"));
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_descendant_pids(root_pid: libc::pid_t) -> Vec<libc::pid_t> {
    let mut descendants = Vec::new();
    let mut pending = vec![root_pid];
    let mut seen = HashSet::from([root_pid]);

    while let Some(parent_pid) = pending.pop() {
        let children_path = format!("/proc/{parent_pid}/task/{parent_pid}/children");
        let Ok(children) = std::fs::read_to_string(children_path) else {
            continue;
        };
        for child_pid in children
            .split_whitespace()
            .filter_map(|value| value.parse::<libc::pid_t>().ok())
            .filter(|pid| *pid > 0)
        {
            if seen.insert(child_pid) {
                descendants.push(child_pid);
                pending.push(child_pid);
            }
        }
    }

    descendants
}

fn turn_output_from_parsed(
    plan: &ClaudeCommandPlan,
    parsed: ParsedClaudeOutput,
    duration_ms: i64,
) -> ClaudePaneTurnOutput {
    ClaudePaneTurnOutput {
        text: parsed.text,
        status: parsed.status,
        session_id: parsed
            .session_id
            .or_else(|| Some(plan.command_session_id.clone())),
        usage_status: usage_status_from_summary(parsed.usage_summary.as_deref()),
        usage_summary: parsed.usage_summary,
        turn_usage_summary: parsed.turn_usage_summary,
        direct_accounting: plan.direct_accounting.clone(),
        artifact_path: plan.artifact_path.clone(),
        audit_path: plan.audit_path.clone(),
        duration_ms,
        terminal_reason: parsed.terminal_reason,
        error_summary: parsed.error_summary,
        tool_names: parsed.tool_names,
        tool_events: parsed.tool_events,
        reasoning_events: parsed.reasoning_events,
        command_mode: plan.command_mode,
    }
}

pub(crate) fn failed_turn_output(
    plan: &ClaudeCommandPlan,
    duration_ms: i64,
    status: ClaudePaneTurnStatus,
    terminal_reason: Option<String>,
    error_summary: String,
) -> ClaudePaneTurnOutput {
    ClaudePaneTurnOutput {
        text: String::new(),
        status,
        session_id: None,
        usage_summary: None,
        turn_usage_summary: None,
        usage_status: ClaudePaneUsageStatus::Missing,
        // Nothing stated, nothing recorded. A partial turn that did state a
        // total gets its accounting back below, where the statement is read.
        direct_accounting: None,
        artifact_path: plan.artifact_path.clone(),
        audit_path: plan.audit_path.clone(),
        duration_ms,
        terminal_reason,
        error_summary: Some(error_summary),
        tool_names: Vec::new(),
        tool_events: Vec::new(),
        reasoning_events: Vec::new(),
        command_mode: plan.command_mode,
    }
}

pub(crate) fn partial_failed_turn_output(
    plan: &ClaudeCommandPlan,
    duration_ms: i64,
    status: ClaudePaneTurnStatus,
    terminal_reason: Option<String>,
    error_summary: String,
    stdout: &str,
) -> ClaudePaneTurnOutput {
    let mut output = failed_turn_output(plan, duration_ms, status, terminal_reason, error_summary);
    if let Ok(parsed) = parse_claude_output(stdout) {
        if !parsed.text.trim().is_empty() {
            output.text = parsed.text;
        }
        output.session_id = parsed
            .session_id
            .or_else(|| Some(plan.command_session_id.clone()));
        output.usage_status = usage_status_from_summary(parsed.usage_summary.as_deref());
        output.usage_summary = parsed.usage_summary;
        // An interrupted or timed-out turn still spent whatever it spent. If
        // the pane managed to state the turn's total before it stopped, that
        // statement is as good as any other and the turn is recorded from it.
        output.turn_usage_summary = parsed.turn_usage_summary;
        if output.turn_usage_summary.is_some() {
            output.direct_accounting = plan.direct_accounting.clone();
        }
        if output.terminal_reason.is_none() {
            output.terminal_reason = parsed.terminal_reason;
        }
        output.tool_names = parsed.tool_names;
        output.tool_events = parsed.tool_events;
        output.reasoning_events = parsed.reasoning_events;
    } else {
        output.turn_usage_summary = turn_usage_summary_from_stdout(stdout);
        if output.turn_usage_summary.is_some() {
            output.direct_accounting = plan.direct_accounting.clone();
        }
        output.tool_events = tool_events_from_stdout(stdout);
        output.tool_names = dedupe_tool_names(
            output
                .tool_events
                .iter()
                .map(|event| event.name.clone())
                .collect(),
        );
        output.reasoning_events = reasoning_events_from_stdout(stdout);
        if matches!(
            status,
            ClaudePaneTurnStatus::Interrupted | ClaudePaneTurnStatus::TimeoutPause
        ) {
            output.session_id =
                session_id_from_stdout(stdout).or_else(|| Some(plan.command_session_id.clone()));
        }
    }
    output
}

pub(crate) fn session_id_from_stdout(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|value| {
            value
                .get("session_id")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

/// Report a direct pane turn's spend, before anything that can fail.
///
/// The turn has already been paid for by the time any of this runs. Reporting
/// it from the turn's `Result` would mean a failed audit write - a full disk -
/// dropped a real charge from the operator's ledger, so it is sent from here,
/// where the output exists and nothing downstream can discard it. Recording is
/// idempotent per turn because this runs once per audit write, which is once
/// per turn.
fn report_direct_turn(progress_tx: Option<&AppEventSender>, output: &ClaudePaneTurnOutput) {
    let (Some(progress_tx), Some((accounting, usage))) = (progress_tx, output.direct_turn_record())
    else {
        return;
    };
    progress_tx.send(AppEvent::PaneBridgeModelRequestSent {
        provider_id: accounting.provider_id,
        base_url: accounting.base_url,
        path: "messages".to_string(),
        model: accounting.model,
        usage: Some(usage),
    });
}

pub(crate) fn write_turn_audit(
    plan: &ClaudeCommandPlan,
    output: &ClaudePaneTurnOutput,
    started_at_unix_ms: u128,
    ended_at_unix_ms: u128,
    last_progress_elapsed_ms: Option<i64>,
) -> Result<()> {
    let audit = ClaudePaneTurnAudit {
        pane_id: plan.pane_id.clone(),
        pane_title: plan.pane_title.clone(),
        provider: plan.profile_title.clone(),
        model: plan.provider_model.clone(),
        session_id: output.session_id.clone(),
        turn_index: plan.turn_index,
        command_mode: plan.command_mode,
        max_turns: plan.max_turns.clone(),
        artifact_path: output.artifact_path.clone(),
        audit_path: output.audit_path.clone(),
        timeout_ms: plan.timeout_ms,
        started_at_unix_ms,
        ended_at_unix_ms,
        last_progress_elapsed_ms,
        duration_ms: output.duration_ms,
        usage: output
            .usage_summary
            .as_deref()
            .and_then(|usage| serde_json::from_str::<Value>(usage).ok()),
        // The ledger records the turn's total; the audit is the operator's
        // evidence for the same turn and states the same number, beside the
        // first-request figure the display uses.
        turn_usage: output
            .turn_usage_summary
            .as_deref()
            .and_then(|usage| serde_json::from_str::<Value>(usage).ok()),
        usage_status: output.usage_status,
        terminal_reason: output.terminal_reason.clone(),
        status: output.status,
        error_summary: output.error_summary.clone(),
        reasoning_event_count: output.reasoning_events.len(),
        reasoning_events: output.reasoning_events.clone(),
        tool_use_count: output.tool_events.len(),
        tool_names: output.tool_names.clone(),
        tool_events: output.tool_events.clone(),
    };
    let bytes =
        serde_json::to_vec_pretty(&audit).context("failed to serialize Claude turn audit")?;
    std::fs::write(&plan.audit_path, bytes).with_context(|| {
        format!(
            "failed to write Claude pane audit `{}`",
            plan.audit_path.display()
        )
    })
}
