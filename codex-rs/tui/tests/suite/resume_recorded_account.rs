//! PF-84 #545: `corbanu resume` of a conversation recorded on a named account,
//! without `--account`, in a home with no default key for that provider. It
//! must resume on the recorded account instead of opening the API-key
//! onboarding screen, and name that account with the `corbanu account add`
//! recovery when it no longer exists.
//!
//! The conversation is a seeded rollout, so no model request is made.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use codex_protocol::ThreadId;
use serde_json::Value;
use serde_json::json;
use tempfile::tempdir;

use crate::support::chat_ready::session_configured;
use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxPane;
use crate::support::tmux::TmuxServer;

pub(super) const READY_TIMEOUT: Duration = Duration::from_secs(30);
pub(super) const PROMPT: &str = "pf84 545 recorded account prompt";
pub(super) const REPLY: &str = "pf84 545 recorded account reply";
pub(super) const ONBOARDING: &str = "Use your Z.AI API key";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_resume_uses_recorded_named_account_without_default_key() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #545 resume on the recorded account")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config(home.path(), &repo_root)?;
    add_account(&binary, home.path(), "work")?;
    let thread_id = seed_rollout(home.path(), &repo_root, "work")?;

    let tmux = TmuxServer::start("pf84_545_resume_recorded_account")?;
    tmux.register_artifact("config.toml", home.path().join("config.toml"));

    // `corbanu resume <id>`.
    let session = tmux.new_session(resume_spec(
        "pf84-545-resume-id",
        &binary,
        &repo_root,
        home.path(),
        &[thread_id.to_string()],
    ))?;
    let pane = session.primary_pane();
    wait_resumed_on_work(pane)?;
    exit_tui(pane)?;

    // The picker (`corbanu resume`) now comes before any onboarding.
    let session = tmux.new_session(resume_spec(
        "pf84-545-resume-picker",
        &binary,
        &repo_root,
        home.path(),
        &[],
    ))?;
    let pane = session.primary_pane();
    let picker = pane.wait_stable_contains(PROMPT, READY_TIMEOUT)?;
    ensure!(
        !picker.contains(ONBOARDING),
        "onboarding came before the picker:\n{picker}"
    );
    pane.send_key(TmuxKey::Enter)?;
    wait_resumed_on_work(pane)?;
    exit_tui(pane)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_resume_names_a_missing_recorded_account_with_recovery() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #545 missing recorded account")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config(home.path(), &repo_root)?;
    // Another account exists, but not the one the conversation ran on.
    add_account(&binary, home.path(), "personal")?;
    seed_rollout(home.path(), &repo_root, "work")?;

    let tmux = TmuxServer::start("pf84_545_missing_recorded_account")?;
    let session = tmux.new_session(resume_spec(
        "pf84-545-missing",
        &binary,
        &repo_root,
        home.path(),
        &["--last".to_string()],
    ))?;
    let pane = session.primary_pane();
    let screen = pane.wait_stable_contains("EXITED", READY_TIMEOUT)?;
    let screen = screen.replace('\n', "");
    for needle in [
        "This conversation ran on zai account `work`",
        "account `work` of provider `zai` is not configured",
        "corbanu account add zai work",
        "EXITED 1",
    ] {
        ensure!(screen.contains(needle), "missing {needle:?} in:\n{screen}");
    }
    ensure!(
        !screen.contains(ONBOARDING),
        "asked for a default key:\n{screen}"
    );
    Ok(())
}

/// PF-84 #555: `corbanu fork <id>` runs the fork on the account the
/// conversation recorded, and `/status` names it, rather than the
/// `[provider_accounts]` account.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_fork_runs_on_the_recorded_account() -> Result<()> {
    if !TmuxServer::should_run("PF-84 #555 fork on the recorded account")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = corbanu_binary(&repo_root)?;
    let home = tempdir()?;
    write_config_with(
        home.path(),
        &repo_root,
        "\n[provider_accounts]\nzai = \"personal\"\n",
    )?;
    add_account(&binary, home.path(), "work")?;
    add_account(&binary, home.path(), "personal")?;
    let thread_id = seed_rollout(home.path(), &repo_root, "work")?;

    let tmux = TmuxServer::start("pf84_555_fork_recorded_account")?;
    let session = tmux.new_session(launch_spec(
        "pf84-555-fork-id",
        &binary,
        &repo_root,
        home.path(),
        Some("fork"),
        &[thread_id.to_string()],
    ))?;
    let pane = session.primary_pane();
    wait_resumed_on_work(pane)?;
    exit_tui(pane)?;
    Ok(())
}

pub(super) fn wait_resumed_on_work(pane: &TmuxPane<'_>) -> Result<()> {
    let screen = pane.wait_stable_until(
        "the resumed conversation, or onboarding",
        READY_TIMEOUT,
        |capture| {
            capture.contains(ONBOARDING) || (session_configured(capture) && capture.contains(REPLY))
        },
    )?;
    ensure!(
        !screen.contains(ONBOARDING),
        "resume opened the API-key onboarding screen:\n{screen}"
    );
    pane.send_literal("/status")?;
    pane.wait_stable_contains("/status", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("account work", READY_TIMEOUT)?;
    Ok(())
}

/// Exits through `/exit`; the wrapper shell reports the exit status. No Esc
/// first: Esc right before the text can arrive as Alt+`/`.
pub(super) fn exit_tui(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/exit")?;
    pane.wait_stable_contains("/exit", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("EXITED 0", READY_TIMEOUT)?;
    Ok(())
}

/// Runs `corbanu resume <args>` with no Z.AI key in the environment and keeps
/// the pane open after exit so the error and exit status stay visible.
pub(super) fn resume_spec(
    name: &str,
    binary: &Path,
    repo_root: &Path,
    home: &Path,
    args: &[String],
) -> SessionSpec {
    launch_spec(name, binary, repo_root, home, Some("resume"), args)
}

/// Runs `corbanu <subcommand> <args>` like [`resume_spec`].
pub(super) fn launch_spec(
    name: &str,
    binary: &Path,
    repo_root: &Path,
    home: &Path,
    subcommand: Option<&str>,
    args: &[String],
) -> SessionSpec {
    let mut command = CommandSpec::new("/bin/sh")
        .env("CODEX_HOME", home)
        .env("CORBANU_HOME", home)
        .env("PFTERMINAL_HOME", home)
        .arg("-c")
        .arg(
            "unset ZAI_API_KEY OPENAI_API_KEY CODEX_API_KEY; \"$0\" \"$@\"; echo \"EXITED $?\"; sleep 600",
        )
        .arg(binary);
    if let Some(subcommand) = subcommand {
        command = command.arg(subcommand);
    }
    for arg in args {
        command = command.arg(arg);
    }
    let command = command
        .arg("-c")
        .arg("analytics.enabled=false")
        .arg("-c")
        .arg("tui.animations=false")
        .arg("--no-alt-screen")
        .arg("-C")
        .arg(repo_root);
    SessionSpec::new(
        name,
        TerminalSize::new(/*columns*/ 140, /*rows*/ 44),
        command,
    )
    .current_dir(repo_root)
}

pub(super) fn write_config(home: &Path, repo_root: &Path) -> Result<()> {
    write_config_with(home, repo_root, "")
}

/// [`write_config`] plus `extra` TOML tables.
pub(super) fn write_config_with(home: &Path, repo_root: &Path, extra: &str) -> Result<()> {
    write_config_on(
        home,
        repo_root,
        "model = \"glm-5.3-flash\"\nmodel_provider = \"zai\"",
        extra,
    )
}

/// [`write_config_with`] for the model and provider `session` selects.
pub(super) fn write_config_on(
    home: &Path,
    repo_root: &Path,
    session: &str,
    extra: &str,
) -> Result<()> {
    fs::create_dir_all(home.join("log"))?;
    fs::write(
        home.join("config.toml"),
        format!(
            r#"{session}
suppress_unstable_features_warning = true
check_for_update_on_startup = false
log_dir = "{}"

[features]
named_accounts = true

[projects."{}"]
trust_level = "trusted"
{extra}"#,
            home.join("log").display(),
            repo_root.display(),
        ),
    )
    .context("write #545 config")
}

/// Enrols a named Z.AI account with a synthetic key, as a user would.
pub(super) fn add_account(binary: &Path, home: &Path, name: &str) -> Result<()> {
    let mut child = Command::new(binary)
        .args(["account", "add", "zai", name])
        .env("CODEX_HOME", home)
        .env("CORBANU_HOME", home)
        .env("PFTERMINAL_HOME", home)
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("run corbanu account add")?;
    child
        .stdin
        .take()
        .context("account add stdin")?
        .write_all(format!("pf84-545-synthetic-{name}-key").as_bytes())?;
    let output = child.wait_with_output()?;
    ensure!(
        output.status.success(),
        "corbanu account add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// A one-turn conversation whose turn context recorded `zai` account `account`.
pub(super) fn seed_rollout(home: &Path, repo_root: &Path, account: &str) -> Result<ThreadId> {
    let thread_id = ThreadId::new();
    let dir = home.join("sessions/2026/10/11");
    fs::create_dir_all(&dir)?;
    let line = |kind: &str, payload: Value| {
        json!({"timestamp": "2026-10-11T10:00:00.000Z", "type": kind, "payload": payload})
            .to_string()
    };
    let lines = [
        line(
            "session_meta",
            json!({
                "session_id": thread_id,
                "id": thread_id,
                "timestamp": "2026-10-11T10:00:00.000Z",
                "cwd": repo_root,
                "originator": "codex-tui",
                "cli_version": "0.0.0",
                "source": "cli",
                "model_provider": "zai",
            }),
        ),
        line(
            "turn_context",
            json!({
                "cwd": repo_root,
                "approval_policy": "on-request",
                "sandbox_policy": {"type": "read-only"},
                "model": "glm-5.3-flash",
                "model_provider": "zai",
                "provider_account": account,
                "summary": "auto",
            }),
        ),
        line(
            "response_item",
            json!({"type": "message", "role": "user",
                   "content": [{"type": "input_text", "text": PROMPT}]}),
        ),
        line(
            "event_msg",
            json!({"type": "user_message", "message": PROMPT, "images": [],
                   "local_images": [], "text_elements": []}),
        ),
        line(
            "event_msg",
            json!({"type": "agent_message", "message": REPLY}),
        ),
        line(
            "response_item",
            json!({"type": "message", "role": "assistant",
                   "content": [{"type": "output_text", "text": REPLY}]}),
        ),
    ];
    fs::write(
        dir.join(format!("rollout-2026-10-11T10-00-00-{thread_id}.jsonl")),
        lines.join("\n") + "\n",
    )?;
    Ok(thread_id)
}

pub(super) fn corbanu_binary(repo_root: &Path) -> Result<PathBuf> {
    for name in ["corbanu", "codex"] {
        if let Ok(binary) = codex_utils_cargo_bin::cargo_bin(name) {
            return Ok(binary);
        }
    }
    let binary = repo_root.join("codex-rs/target/debug/corbanu");
    ensure!(binary.is_file(), "build the corbanu binary first");
    Ok(binary)
}
