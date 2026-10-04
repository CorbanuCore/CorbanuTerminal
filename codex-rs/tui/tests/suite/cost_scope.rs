//! acct-scope-62: `/cost` states its scope, the day's other conversations and
//! the next step for unpriced requests, driven through a real PTY.
//!
//! The ledger is seeded with synthetic attempts on today's UTC date in two
//! other conversations: two priced OpenAI requests and one request on a custom
//! provider with no price. The open conversation runs one turn against a mocked
//! Responses server, then opens `/cost`.

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use codex_protocol::ThreadId;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::Attempt;
use codex_state::accounting::Observation;
use codex_state::accounting::Snapshot;
use codex_utils_absolute_path::AbsolutePathBuf;
use core_test_support::responses;
use core_test_support::skip_if_no_network;
use serde_json::json;
use tempfile::tempdir;
use uuid::Uuid;
use wiremock::MockServer;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxServer;

const DAY_MS: i64 = 86_400_000;
const PROMPT: &str = "Say the cost scope sentinel.";
const REPLY: &str = "cost scope sentinel";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_cost_states_scope_other_conversations_and_missing_price_step() -> Result<()> {
    skip_if_no_network!(Ok(()));
    if !TmuxServer::should_run("/cost scope and missing-price step")? {
        return Ok(());
    }

    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let codex = tui_binary(&repo_root)?;
    let codex_home = tempdir()?;
    let server = MockServer::start().await;
    let _turn = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_response_created("resp-cost-scope"),
            responses::ev_assistant_message("msg-cost-scope", REPLY),
            responses::ev_completed_with_tokens("resp-cost-scope", /*total_tokens*/ 42),
        ]),
    )
    .await;
    write_test_config(codex_home.path(), &repo_root)?;
    seed_other_conversations(codex_home.path()).await?;

    let tmux = TmuxServer::start("tmux_cost_scope")?;
    tmux.register_artifact("config.toml", codex_home.path().join("config.toml"));
    tmux.register_artifact("codex-tui.log", codex_home.path().join("log/codex-tui.log"));
    let session = tmux.new_session(
        SessionSpec::new(
            "codex-cost-scope",
            TerminalSize::new(/*columns*/ 120, /*rows*/ 50),
            CommandSpec::new(codex)
                .env("CODEX_HOME", codex_home.path())
                .env("CODEX_SQLITE_HOME", codex_home.path())
                .env("OPENAI_API_KEY", "tmux-cost-scope-test")
                .env("RUST_LOG", "trace")
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

    pane.send_literal("/cost")?;
    pane.wait_stable_contains("/cost", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(
        "Other conversations on this day, not included above: 3 requests in 2 conversations.",
        Duration::from_secs(/*secs*/ 30),
    )?;
    // The first screen may be taller than the popup; scroll until every
    // checkpoint has been seen on a settled frame.
    let checkpoints = [
        "in this conversation",
        "OpenAI",
        "local-mock · mock-model — Pay per use. 1 request, 55 tokens. Estimated cost: no price available.",
        "Other conversations' estimated cost: at least $0.001420 (1 attempt had no price).",
        "To see those requests, open that conversation with /resume and run /cost there.",
        "Next step for requests with no price: check the bill from",
    ];
    let mut seen = String::new();
    for _ in 0..30 {
        seen.push_str(&pane.capture_viewport()?);
        if checkpoints.iter().all(|needle| seen.contains(needle)) {
            break;
        }
        pane.send_key(TmuxKey::Down)?;
        pane.wait_stable_until("settled frame", Duration::from_secs(/*secs*/ 5), |_| true)?;
    }
    for needle in checkpoints {
        assert!(seen.contains(needle), "missing {needle:?} in:\n{seen}");
    }
    // Evidence for qualification records (shown with --no-capture).
    eprintln!("cost_scope frames checked:\n{seen}");
    assert!(
        !seen.contains("Known subtotal exact USD: 0\n") && !seen.contains("in this day"),
        "the page implied a day-wide zero:\n{seen}"
    );

    // Esc closes the inspector from its first page; the composer is usable again.
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_until(
        "inspector closed",
        Duration::from_secs(/*secs*/ 10),
        |screen| !screen.contains("Other conversations on this day"),
    )?;
    pane.send_literal("/exit")?;
    pane.wait_stable_contains("/exit", Duration::from_secs(/*secs*/ 5))?;
    pane.send_key(TmuxKey::Enter)?;
    session.wait_for_exit(Duration::from_secs(/*secs*/ 15))?;
    Ok(())
}

/// `CORBANU_ACCOUNTING_CANDIDATE` names a full developer-accounting CLI, which
/// also records the open conversation's turn. Otherwise the `codex-tui` built
/// with this test: it carries `/cost` and reads the seeded ledger, but its turn
/// is not collected unless codex-core's feature is enabled too.
fn tui_binary(repo_root: &Path) -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("CORBANU_ACCOUNTING_CANDIDATE") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = codex_utils_cargo_bin::cargo_bin("codex-tui") {
        return Ok(path);
    }
    let fallback = repo_root.join("codex-rs/target/debug/codex-tui");
    anyhow::ensure!(
        fallback.is_file(),
        "build codex-tui with --features codex-core/developer-accounting,codex-tui/developer-accounting"
    );
    Ok(fallback)
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
    std::fs::write(codex_home.join("config.toml"), config).context("write /cost test config")?;
    std::fs::write(
        codex_home.join("auth.json"),
        r#"{"OPENAI_API_KEY":"tmux-cost-scope-test","tokens":null,"last_refresh":null}"#,
    )
    .context("write /cost test authentication")
}

/// Conversation A: two OpenAI requests priced at $0.00071 each (80 noncached
/// at $5/M, 20 cache read at $0.50/M, 10 output at $30/M). Conversation B: one
/// request on a custom provider whose tokens no published price covers.
async fn seed_other_conversations(codex_home: &Path) -> Result<()> {
    let runtime = StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(codex_home.to_path_buf())?),
        "openai".into(),
    )
    .await?;
    let now = chrono::Utc::now().timestamp_millis();
    let dispatched = (now - 1_000).max(now / DAY_MS * DAY_MS);
    let store = AccountingStore::open(&runtime, now).await?;
    let (a, b) = (ThreadId::new(), ThreadId::new());
    for thread in [a, b] {
        let mut builder = ThreadMetadataBuilder::new(
            thread,
            codex_home.join(format!("{thread}.jsonl")),
            chrono::Utc::now(),
            SessionSource::Cli,
        );
        builder.cwd = codex_home.to_path_buf();
        runtime.upsert_thread(&builder.build("openai")).await?;
    }
    let price: Snapshot = serde_json::from_value(json!({
        "id": Uuid::new_v4(), "provider": "openai", "model": "gpt-5.4", "scope": Uuid::nil(),
        "currency": "USD", "unit": "PerMillionTokens",
        "rates": {"noncached": "5", "read": "0.5", "write": null, "output": "30"},
        "source_reference": Uuid::new_v4(), "source_kind": "ProviderPublished",
        "observed_at_ms": 0, "approved_at_ms": 0, "effective_from_ms": 0, "effective_end_ms": null
    }))?;
    for (owner, provider, model, prices, patch) in [
        (
            a,
            "openai",
            "gpt-5.4",
            vec![price.clone()],
            json!({"input": 100, "read": 20, "write": 0, "output": 10}),
        ),
        (
            a,
            "openai",
            "gpt-5.4",
            vec![price],
            json!({"input": 100, "read": 20, "write": 0, "output": 10}),
        ),
        (
            b,
            "local-mock",
            "mock-model",
            Vec::new(),
            json!({"input": 50, "read": 0, "write": 0, "output": 5}),
        ),
    ] {
        let attempt: Attempt = serde_json::from_value(json!({
            "attempt_id": Uuid::new_v4(), "request_id": Uuid::new_v4(), "thread_id": owner,
            "turn": "seeded", "retry_of": null, "provider": provider, "model": model,
            "scope": Uuid::nil(), "dialect": "Inclusive", "dispatched_at_ms": dispatched
        }))?;
        let observation: Observation = serde_json::from_value(json!({
            "revision": 1, "source": attempt.attempt_id, "sequence": 1, "patch": patch
        }))?;
        store.admit(owner, &attempt, &prices, now).await?;
        store.observe(owner, &attempt, &[observation], now).await?;
    }
    runtime.close().await;
    Ok(())
}
