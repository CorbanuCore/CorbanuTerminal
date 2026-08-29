use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use core_test_support::responses;
use core_test_support::skip_if_no_network;
use tempfile::tempdir;
use wiremock::MockServer;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_output_text_delta_before_item_added_stays_live() -> Result<()> {
    skip_if_no_network!(Ok(()));
    if !TmuxServer::should_run("early output-text delta recovery")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let server = MockServer::start().await;
    let _response_mock = responses::mount_sse_once(&server, orphan_delta_sse()).await;
    let openai_base_url_config = format!("openai_base_url=\"{}/v1\"", server.uri());
    write_test_config(codex_home.path(), &repo_root)?;
    write_test_auth(codex_home.path())?;

    let tmux = TmuxServer::start("tmux_output_text_delta_before_item_added")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", codex_home.path().join("log/codex-tui.log"));
    let session = tmux.new_session(
        SessionSpec::new(
            "codex-orphan-delta-smoke",
            TerminalSize::new(/*columns*/ 120, /*rows*/ 40),
            CommandSpec::new(codex)
                .env("CODEX_HOME", codex_home.path())
                .env("OPENAI_API_KEY", "tmux-output-delta-test")
                .env("RUST_LOG", "trace")
                .arg("-c")
                .arg("analytics.enabled=false")
                .arg("-c")
                .arg(&openai_base_url_config)
                .arg("--no-alt-screen")
                .arg("-C")
                .arg(&repo_root),
        )
        .current_dir(&repo_root),
    )?;
    let pane = session.primary_pane();

    pane.wait_stable_contains("Corbanu Terminal", Duration::from_secs(/*secs*/ 30))?;
    pane.send_literal("Recover a delta that arrived before its item.")?;
    pane.wait_stable_contains(
        "Recover a delta that arrived before its item.",
        Duration::from_secs(/*secs*/ 5),
    )?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(
        "early delta recovery sentinel",
        Duration::from_secs(/*secs*/ 30),
    )?;

    pane.send_literal("/exit")?;
    pane.wait_stable_contains("/exit", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    session.wait_for_exit(Duration::from_secs(/*secs*/ 15))?;
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
    let log_dir = codex_home.join("log");
    let log_dir = log_dir.display();
    let config = format!(
        "model = \"gpt-5.4\"\nmodel_provider = \"openai\"\nlog_dir = \"{log_dir}\"\n\
         suppress_unstable_features_warning = true\n\n\
         [projects.\"{repo_root}\"]\ntrust_level = \"trusted\"\n"
    );
    std::fs::write(codex_home.join("config.toml"), config)
        .context("write output-text delta test configuration")
}

fn write_test_auth(codex_home: &Path) -> Result<()> {
    std::fs::write(
        codex_home.join("auth.json"),
        r#"{"OPENAI_API_KEY":"tmux-output-delta-test","tokens":null,"last_refresh":null}"#,
    )
    .context("write output-text delta test authentication")
}

fn orphan_delta_sse() -> String {
    responses::sse(vec![
        responses::ev_response_created("resp-orphan-delta"),
        responses::ev_output_text_delta_for_item("msg-orphan-delta", "early delta "),
        responses::ev_message_item_added("msg-orphan-delta", ""),
        responses::ev_output_text_delta_for_item("msg-orphan-delta", "recovery sentinel"),
        responses::ev_assistant_message("msg-orphan-delta", "early delta recovery sentinel"),
        responses::ev_completed("resp-orphan-delta"),
    ])
}
