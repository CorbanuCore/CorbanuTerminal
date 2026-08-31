use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use tempfile::TempDir;
use tempfile::tempdir;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxPane;
use crate::support::tmux::TmuxServer;

const READY_TIMEOUT: Duration = Duration::from_secs(30);

#[test]
fn tmux_claude_auth_managed_success_cancel_failure_recovery_and_resume() -> Result<()> {
    if !TmuxServer::should_run("Claude managed-auth state machine")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let log_dir = codex_home.path().join("log");
    fs::create_dir_all(&log_dir)?;
    write_test_config(codex_home.path(), &repo_root)?;
    let fake = FakeClaude::new()?;
    let canary = synthetic_canary();

    let tmux = TmuxServer::start("tmux_claude_auth_managed")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", log_dir.join("codex-tui.log"));
    let session = tmux.new_session(session_spec(
        "codex-claude-auth-managed",
        &codex,
        &repo_root,
        codex_home.path(),
        &log_dir,
        &fake,
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;

    open_claude_auth_choice(pane)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_until("method picker cancelled", READY_TIMEOUT, |capture| {
        !capture.contains("Claude Plan authentication")
    })?;

    open_claude_auth_choice(pane)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Long-lived token — masked", READY_TIMEOUT)?;
    pane.send_secret_literal(&canary)?;
    pane.wait_stable_contains("••", Duration::from_secs(10))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("saved and selected", Duration::from_secs(45))?;
    assert_managed_resolver_returns(&codex, codex_home.path(), &canary)?;

    fs::write(&fake.failure_marker, b"fail setup-token")?;
    open_claude_auth_choice(pane)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Claude authentication needs attention", READY_TIMEOUT)?;
    pane.wait_stable_contains("No fallback occurred", READY_TIMEOUT)?;
    fs::remove_file(&fake.failure_marker)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Long-lived token — masked", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_until("masked replacement cancelled", READY_TIMEOUT, |capture| {
        !capture.contains("Long-lived token — masked")
    })?;
    assert_managed_resolver_returns(&codex, codex_home.path(), &canary)?;

    open_providers(pane)?;
    pane.wait_stable_contains(
        "Selected · long-lived subscription token",
        Duration::from_secs(45),
    )?;
    pane.send_key(TmuxKey::Escape)?;
    let first_viewport = pane.capture_viewport()?;
    let first_scrollback = pane.capture_scrollback_tail(2_000)?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;

    let resumed = tmux.new_session(session_spec(
        "codex-claude-auth-resumed",
        &codex,
        &repo_root,
        codex_home.path(),
        &log_dir,
        &fake,
    ))?;
    let resumed_pane = resumed.primary_pane();
    resumed_pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;
    open_providers(resumed_pane)?;
    let resumed_viewport = resumed_pane.wait_stable_contains(
        "Selected · long-lived subscription token",
        Duration::from_secs(45),
    )?;
    let resumed_scrollback = resumed_pane.capture_scrollback_tail(2_000)?;
    resumed_pane.send_key(TmuxKey::Escape)?;
    exit_tui(resumed_pane)?;
    resumed.wait_for_exit(READY_TIMEOUT)?;

    for surface in [
        first_viewport.as_str(),
        first_scrollback.as_str(),
        resumed_viewport.as_str(),
        resumed_scrollback.as_str(),
    ] {
        ensure!(
            !surface.contains(&canary),
            "secret canary appeared in a terminal capture"
        );
    }
    ensure!(
        !tree_contains(codex_home.path(), canary.as_bytes())?,
        "secret canary appeared in the isolated home or trace logs"
    );
    if tmux.artifact_dir().is_dir() {
        ensure!(
            !tree_contains(&tmux.artifact_dir(), canary.as_bytes())?,
            "secret canary appeared in tmux artifacts"
        );
    }
    Ok(())
}

#[test]
fn tmux_claude_auth_compatibility_selects_existing_login() -> Result<()> {
    if !TmuxServer::should_run("Claude compatibility-login migration")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let log_dir = codex_home.path().join("log");
    fs::create_dir_all(&log_dir)?;
    write_test_config(codex_home.path(), &repo_root)?;
    let fake = FakeClaude::new()?;

    let tmux = TmuxServer::start("tmux_claude_auth_compatibility")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", log_dir.join("codex-tui.log"));
    let session = tmux.new_session(session_spec(
        "codex-claude-auth-compatibility",
        &codex,
        &repo_root,
        codex_home.path(),
        &log_dir,
        &fake,
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;
    open_claude_auth_choice(pane)?;
    pane.send_key(TmuxKey::Down)?;
    pane.wait_stable_contains("› Claude Code login", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(
        "Existing Claude Code login selected",
        Duration::from_secs(45),
    )?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;

    let selection = codex_vault::Vault::new(codex_home.path().to_path_buf())
        .load_claude_auth_selection()?
        .context("compatibility selection was not persisted")?;
    ensure!(
        selection.source == codex_vault::ClaudeAuthSource::ClaudeCodeLogin,
        "compatibility flow selected an unexpected source"
    );
    Ok(())
}

fn open_claude_auth_choice(pane: &TmuxPane<'_>) -> Result<()> {
    open_providers(pane)?;
    pane.send_key(TmuxKey::Down)?;
    pane.wait_stable_contains("› Provider: Claude Code Plan", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Long-lived subscription token (Recommended)", READY_TIMEOUT)?;
    Ok(())
}

fn open_providers(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/providers")?;
    pane.wait_stable_contains("/providers", Duration::from_secs(10))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Provider: Claude Code Plan", READY_TIMEOUT)?;
    Ok(())
}

fn exit_tui(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/exit")?;
    pane.wait_stable_contains("/exit", Duration::from_secs(10))?;
    pane.send_key(TmuxKey::Enter)?;
    Ok(())
}

fn assert_managed_resolver_returns(codex: &Path, codex_home: &Path, canary: &str) -> Result<()> {
    let output = Command::new(codex)
        .env("CODEX_HOME", codex_home)
        .arg("internal-claude-oauth-token")
        .output()
        .context("run candidate managed-token resolver")?;
    ensure!(
        output.status.success(),
        "candidate managed-token resolver failed"
    );
    ensure!(
        output.stdout == canary.as_bytes(),
        "candidate resolver returned an unexpected token"
    );
    Ok(())
}

fn session_spec(
    name: &str,
    codex: &Path,
    repo_root: &Path,
    codex_home: &Path,
    log_dir: &Path,
    fake: &FakeClaude,
) -> SessionSpec {
    SessionSpec::new(
        name,
        TerminalSize::new(/*columns*/ 140, /*rows*/ 44),
        CommandSpec::new(codex.to_path_buf())
            .env("CODEX_HOME", codex_home)
            .env("OPENAI_API_KEY", "tmux-claude-auth-openai-fixture")
            .env("PATH", fake.path())
            .env("CLAUDE_SETUP_FAIL_MARKER", fake.failure_marker.as_os_str())
            .env("RUST_LOG", "trace")
            .arg("-c")
            .arg(format!("log_dir=\"{}\"", log_dir.display()))
            .arg("-c")
            .arg("analytics.enabled=false")
            .arg("--no-alt-screen")
            .arg("-C")
            .arg(repo_root),
    )
    .current_dir(repo_root)
}

struct FakeClaude {
    _directory: TempDir,
    bin_dir: PathBuf,
    failure_marker: PathBuf,
}

impl FakeClaude {
    fn new() -> Result<Self> {
        let directory = tempdir()?;
        let bin_dir = directory.path().join("bin");
        fs::create_dir_all(&bin_dir)?;
        let failure_marker = directory.path().join("fail-setup-token");
        let executable = bin_dir.join("claude");
        fs::write(
            &executable,
            r#"#!/bin/sh
if [ "$1" = "setup-token" ]; then
  if [ -f "$CLAUDE_SETUP_FAIL_MARKER" ]; then
    exit 7
  fi
  exit 0
fi
if [ "$1 $2 $3" = "auth status --json" ]; then
  printf '{"loggedIn":true,"authMethod":"claude.ai","email":"fixture@example.invalid","subscriptionType":"max"}\n'
  exit 0
fi
exit 2
"#,
        )?;
        let mut permissions = fs::metadata(&executable)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions)?;
        Ok(Self {
            _directory: directory,
            bin_dir,
            failure_marker,
        })
    }

    fn path(&self) -> OsString {
        let mut value = self.bin_dir.as_os_str().to_os_string();
        if let Some(existing) = std::env::var_os("PATH") {
            value.push(":");
            value.push(existing);
        }
        value
    }
}

fn synthetic_canary() -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("synthetic-claude-oauth-{}-{nonce}", std::process::id())
}

fn tree_contains(root: &Path, needle: &[u8]) -> Result<bool> {
    if !root.exists() {
        return Ok(false);
    }
    if root.is_file() {
        let bytes = fs::read(root)?;
        return Ok(bytes.windows(needle.len()).any(|window| window == needle));
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if tree_contains(&entry.path(), needle)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn codex_binary(repo_root: &Path) -> Result<PathBuf> {
    if let Ok(path) = codex_utils_cargo_bin::cargo_bin("codex") {
        return Ok(path);
    }
    if let Ok(path) = codex_utils_cargo_bin::cargo_bin("codex-tui") {
        return Ok(path);
    }
    for binary in ["codex", "codex-tui"] {
        let fallback = repo_root.join("codex-rs/target/debug").join(binary);
        if fallback.is_file() {
            return Ok(fallback);
        }
    }
    anyhow::bail!("Corbanu TUI binary is unavailable; build `codex` or `codex-tui` first")
}

fn write_test_config(codex_home: &Path, repo_root: &Path) -> Result<()> {
    let repo_root = repo_root.display();
    let config = format!(
        "model = \"gpt-5.6-terra\"\nmodel_provider = \"openai\"\n\
         suppress_unstable_features_warning = true\n\n\
         [projects.\"{repo_root}\"]\ntrust_level = \"trusted\"\n"
    );
    fs::write(codex_home.join("config.toml"), config)
        .context("write Claude-auth test configuration")?;
    fs::write(
        codex_home.join("auth.json"),
        r#"{"OPENAI_API_KEY":"tmux-claude-auth-openai-fixture","tokens":null,"last_refresh":null}"#,
    )
    .context("write Claude-auth test authentication")
}
