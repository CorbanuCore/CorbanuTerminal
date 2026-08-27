use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use tempfile::tempdir;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxServer;

#[test]
fn tmux_smoke_single_enter_dispatches_slash_command_and_exits_cleanly() -> Result<()> {
    if !TmuxServer::should_run("slash dispatch smoke")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    write_test_config(codex_home.path(), &repo_root)?;

    let tmux = TmuxServer::start("tmux_smoke_single_enter")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", codex_home.path().join("log/codex-tui.log"));
    let session = tmux.new_session(
        SessionSpec::new(
            "codex-slash-dispatch-smoke",
            TerminalSize::new(/*columns*/ 120, /*rows*/ 40),
            CommandSpec::new(codex)
                .env("CODEX_HOME", codex_home.path())
                .env("OPENAI_API_KEY", "tmux-slash-test")
                .arg("-c")
                .arg("analytics.enabled=false")
                .arg("--no-alt-screen")
                .arg("-C")
                .arg(&repo_root),
        )
        .current_dir(&repo_root),
    )?;
    let pane = session.primary_pane();

    pane.wait_stable_contains("Corbanu Terminal", Duration::from_secs(/*secs*/ 15))?;
    pane.send_literal("/status")?;
    pane.wait_stable_contains("/status", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Permissions:", Duration::from_secs(/*secs*/ 15))?;

    pane.send_literal("/exit")?;
    pane.wait_stable_contains("/exit", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    session.wait_for_exit(Duration::from_secs(/*secs*/ 15))?;
    Ok(())
}

#[test]
fn tmux_gpu_menu_lists_glm_5_3_h200_and_b300_presets_then_cancels() -> Result<()> {
    if !TmuxServer::should_run("GLM-5.3 GPU menu")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let log_dir = tempdir()?;
    write_test_config(codex_home.path(), &repo_root)?;

    let tmux = TmuxServer::start("tmux_gpu_menu_glm_5_3_flash")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", log_dir.path().join("codex-tui.log"));
    let session = tmux.new_session(
        SessionSpec::new(
            "codex-gpu-menu-glm53",
            TerminalSize::new(/*columns*/ 140, /*rows*/ 44),
            CommandSpec::new(codex)
                .env("CODEX_HOME", codex_home.path())
                .env("OPENAI_API_KEY", "tmux-gpu-menu-test")
                .env("RUST_LOG", "trace")
                .arg("-c")
                .arg("analytics.enabled=false")
                .arg("-c")
                .arg(format!("log_dir={}", log_dir.path().display()))
                .arg("--no-alt-screen")
                .arg("-C")
                .arg(&repo_root),
        )
        .current_dir(&repo_root),
    )?;
    let pane = session.primary_pane();

    pane.wait_stable_contains("Corbanu Terminal", Duration::from_secs(/*secs*/ 15))?;
    pane.send_literal("/gpu")?;
    pane.wait_stable_contains("/gpu", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    let menu =
        pane.wait_stable_contains("2xb300-r4 · qualified", Duration::from_secs(/*secs*/ 15))?;
    assert!(menu.contains("Rent zai-org/GLM-5.3-Flash · 4× NVIDIA H200"));
    assert!(menu.contains("Rent zai-org/GLM-5.3-Flash · 2× NVIDIA B300"));
    assert!(menu.contains("glm-5.3-flash-4xh200"));
    assert!(menu.contains("glm-5.3-flash-fp8-2xb300"));
    assert!(!menu.contains("glm-5.3-flash-fp8-2xb300-experimental"));
    assert!(menu.contains("Rent Qwen/Qwen3.8-27B-FP8 · 2× NVIDIA H200"));
    assert!(menu.contains("qwen3.8-27b-fp8-2xh200-experimental"));
    assert!(menu.contains("experimental"));

    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_until(
        "GPU menu dismissal",
        Duration::from_secs(/*secs*/ 5),
        |capture| !capture.contains("Rent zai-org/GLM-5.3-Flash"),
    )?;
    Ok(())
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
    std::fs::write(codex_home.join("config.toml"), config)
        .context("write slash-dispatch test configuration")?;
    std::fs::write(
        codex_home.join("auth.json"),
        r#"{"OPENAI_API_KEY":"tmux-slash-test","tokens":null,"last_refresh":null}"#,
    )
    .context("write slash-dispatch test authentication")
}
