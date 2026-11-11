//! #1039: `/review` inside `/side` sends nothing and says why. The refusal must
//! stay readable on the real terminal: it used to be written into the first row
//! of the side view and erased by the next draw, leaving only `■`.

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

const PROMPT: &str = "Reply with the agreed word.";
const REPLY: &str = "SIDE-REFUSAL-READY";
const REFUSAL: &str = "'/review' is unavailable in side conversations. Press Ctrl+C to return to the main thread first.";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_review_refusal_in_side_conversation_stays_visible() -> Result<()> {
    skip_if_no_network!(Ok(()));
    if !TmuxServer::should_run("/review refusal inside /side")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = codex_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let server = MockServer::start().await;
    let _turn = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_response_created("resp-side-refusal"),
            responses::ev_assistant_message("msg-side-refusal", REPLY),
            responses::ev_completed("resp-side-refusal"),
        ]),
    )
    .await;
    write_test_config(codex_home.path(), &repo_root)?;

    let tmux = TmuxServer::start("tmux_side_review_refusal")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", codex_home.path().join("log/codex-tui.log"));
    let session = tmux.new_session(
        SessionSpec::new(
            "codex-side-review-refusal",
            // Wide enough that the refusal takes one row, as in the #1039 run.
            TerminalSize::new(/*columns*/ 140, /*rows*/ 40),
            CommandSpec::new(codex)
                .env("CODEX_HOME", codex_home.path())
                .env("OPENAI_API_KEY", "tmux-side-refusal-test")
                .arg("-c")
                .arg("analytics.enabled=false")
                .arg("-c")
                .arg(format!("openai_base_url=\"{}/v1\"", server.uri()))
                .arg("--no-alt-screen")
                .arg("-C")
                .arg(&repo_root),
        )
        .current_dir(&repo_root),
    )?;
    let pane = session.primary_pane();

    pane.wait_stable_contains("Corbanu Terminal", Duration::from_secs(/*secs*/ 30))?;
    pane.send_literal(PROMPT)?;
    pane.wait_stable_contains(PROMPT, Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(REPLY, Duration::from_secs(/*secs*/ 30))?;

    pane.send_literal("/side")?;
    pane.wait_stable_contains("/side", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Side from main thread", Duration::from_secs(/*secs*/ 15))?;

    pane.send_literal("/review")?;
    pane.wait_stable_contains("/review", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    let screen = pane.wait_stable_contains(REFUSAL, Duration::from_secs(/*secs*/ 10))?;
    // Evidence for qualification records (shown with --no-capture).
    eprintln!("side refusal screen:\n{screen}");
    assert!(
        screen.contains("Side from main thread"),
        "still in the side conversation:\n{screen}"
    );
    // A settled frame shows it (it used to vanish on the next draw), and the
    // refused command sent nothing. Count at the server: the one-shot mock
    // stops recording once it has matched, so it could not see a second request.
    let model_requests = server
        .received_requests()
        .await
        .context("wiremock request recording")?
        .into_iter()
        .filter(|request| {
            request.method == wiremock::http::Method::POST
                && request.url.path().ends_with("/responses")
        })
        .count();
    assert_eq!(model_requests, 1, "only the first turn reached the model");

    pane.send_key(TmuxKey::CtrlC)?;
    pane.wait_stable_until(
        "back on the main thread",
        Duration::from_secs(/*secs*/ 15),
        |screen| !screen.contains("Side from main thread") && screen.contains(REPLY),
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
         check_for_update_on_startup = false\n\
         suppress_unstable_features_warning = true\n\n\
         [projects.\"{repo_root}\"]\ntrust_level = \"trusted\"\n"
    );
    std::fs::write(codex_home.join("config.toml"), config)
        .context("write side refusal test configuration")?;
    std::fs::write(
        codex_home.join("auth.json"),
        r#"{"OPENAI_API_KEY":"tmux-side-refusal-test","tokens":null,"last_refresh":null}"#,
    )
    .context("write side refusal test authentication")
}
