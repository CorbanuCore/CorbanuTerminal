use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use core_test_support::responses;
use sha2::Digest;
use sha2::Sha256;
use tempfile::tempdir;
use uuid::Uuid;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxPane;
use crate::support::tmux::TmuxServer;

const READY_TIMEOUT: Duration = Duration::from_secs(45);

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_configure_many_preserves_first_default_restart_and_request() -> Result<()> {
    if !TmuxServer::should_run("PF-53 configure-many default restart request")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = codex_binary(&repo_root)?;
    let home = tempdir()?;
    let server = MockServer::start().await;
    let _first = responses::mount_sse_once(&server, response("pf53 first request")).await;
    let _restart = responses::mount_sse_once(&server, response("pf53 restart request")).await;
    write_config(home.path(), &repo_root, &server.uri())?;

    let tmux = TmuxServer::start("pf53_configure_many_restart_request")?;
    register_evidence(&tmux, home.path(), &binary)?;
    let session = tmux.new_session(session_spec(
        "pf53-configure-many",
        &binary,
        &repo_root,
        home.path(),
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Provider: OpenAI Codex Account", READY_TIMEOUT)?;

    let loopback_canary = synthetic_canary("loopback");
    let zai_canary = synthetic_canary("zai");
    configure_api_key(
        pane,
        "Provider: PF53 Loopback PF53_LOOPBACK_API_KEY",
        &loopback_canary,
    )?;
    pane.wait_stable_contains("Configured · active · ready", READY_TIMEOUT)?;
    configure_api_key(pane, "Provider: Z.AI API Key", &zai_canary)?;
    pane.wait_stable_contains("Configured · active · ready", READY_TIMEOUT)?;
    select_label(pane, "Done")?;
    pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;

    let config = fs::read_to_string(home.path().join("config.toml"))?;
    ensure!(
        config.contains("model_provider = \"pf53-loopback\""),
        "first successful provider was not preserved as the default:\n{config}"
    );

    submit_and_wait(pane, "PF53 request one", "pf53 first request")?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;

    let resumed = tmux.new_session(session_spec(
        "pf53-configure-many-restart",
        &binary,
        &repo_root,
        home.path(),
    ))?;
    let resumed_pane = resumed.primary_pane();
    resumed_pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;
    ensure!(
        !resumed_pane
            .capture_viewport()?
            .contains("Choose a provider account"),
        "restart unexpectedly reopened onboarding"
    );
    submit_and_wait(
        resumed_pane,
        "PF53 request after restart",
        "pf53 restart request",
    )?;
    capture_success_evidence(
        "configure-many-restart-request",
        &binary,
        home.path(),
        resumed_pane,
        &server,
        &[loopback_canary.as_str(), zai_canary.as_str()],
    )
    .await?;
    exit_tui(resumed_pane)?;
    resumed.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_deferred_plan_cancel_with_fallback_continues_to_chat() -> Result<()> {
    if !TmuxServer::should_run("PF-53 deferred Plan fallback cancellation")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = codex_binary(&repo_root)?;
    let home = tempdir()?;
    let server = MockServer::start().await;
    write_config(home.path(), &repo_root, &server.uri())?;

    let tmux = TmuxServer::start("pf53_deferred_fallback_cancel")?;
    register_evidence(&tmux, home.path(), &binary)?;
    let session = tmux.new_session(session_spec(
        "pf53-deferred-fallback",
        &binary,
        &repo_root,
        home.path(),
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Provider: OpenAI Codex Account", READY_TIMEOUT)?;
    let fallback_canary = synthetic_canary("fallback");
    configure_api_key(pane, "Provider: Ambient API Key", &fallback_canary)?;
    pane.wait_stable_contains("Configured · active · ready", READY_TIMEOUT)?;
    select_label(pane, "Corbanu Plan")?;
    pane.wait_stable_contains("Corbanu Plan (queued)", READY_TIMEOUT)?;
    select_label(pane, "Done")?;
    pane.wait_stable_contains("Create Solana wallet", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;
    wait_chat_ready(pane)?;
    ensure!(
        !pane.capture_viewport()?.contains("Set up providers"),
        "fallback cancellation reopened provider setup despite a usable provider"
    );
    capture_success_evidence(
        "deferred-fallback-cancel",
        &binary,
        home.path(),
        pane,
        &server,
        &[fallback_canary.as_str()],
    )
    .await?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_only_plan_cancel_returns_to_shared_provider_list() -> Result<()> {
    if !TmuxServer::should_run("PF-53 only-Plan cancellation return")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = codex_binary(&repo_root)?;
    let home = tempdir()?;
    let server = MockServer::start().await;
    write_config(home.path(), &repo_root, &server.uri())?;

    let tmux = TmuxServer::start("pf53_only_plan_cancel")?;
    register_evidence(&tmux, home.path(), &binary)?;
    let session = tmux.new_session(session_spec(
        "pf53-only-plan",
        &binary,
        &repo_root,
        home.path(),
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Provider: OpenAI Codex Account", READY_TIMEOUT)?;

    select_label(pane, "Provider: Ambient API Key")?;
    pane.wait_stable_contains("Paste or type your API key below.", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Provider: Ambient API Key", READY_TIMEOUT)?;
    select_label(pane, "Corbanu Plan")?;
    pane.wait_stable_contains("Corbanu Plan (queued)", READY_TIMEOUT)?;
    select_label(pane, "Done")?;
    pane.wait_stable_contains("Create Solana wallet", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Set up providers", READY_TIMEOUT)?;
    pane.wait_stable_contains("Configure more than one provider", READY_TIMEOUT)?;
    capture_success_evidence("only-plan-return", &binary, home.path(), pane, &server, &[]).await?;
    // With no usable provider, cancellation intentionally remains in the shared setup host.
    // Let the harness-owned session drop perform deterministic process cleanup.
    drop(session);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_fresh_wallet_plan_success_preserves_existing_current_provider() -> Result<()> {
    if !TmuxServer::should_run("PF-53 fresh wallet Plan success")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = codex_binary(&repo_root)?;
    let home = tempdir()?;
    let server = MockServer::start().await;
    let plan_key = synthetic_canary("plan-key");
    mount_plan_gateway(&server, &plan_key, /*flaky_plans*/ false).await;
    write_config(home.path(), &repo_root, &server.uri())?;

    let tmux = TmuxServer::start("pf53_fresh_wallet_plan_success")?;
    register_evidence(&tmux, home.path(), &binary)?;
    let session = tmux.new_session(session_spec_with_gateway(
        "pf53-fresh-plan-success",
        &binary,
        &repo_root,
        home.path(),
        &server.uri(),
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Provider: OpenAI Codex Account", READY_TIMEOUT)?;
    let ambient_canary = synthetic_canary("existing-current");
    configure_api_key(pane, "Provider: Ambient API Key", &ambient_canary)?;
    select_label(pane, "Corbanu Plan")?;
    select_label(pane, "Done")?;

    let passphrase = synthetic_canary("wallet-passphrase");
    pane.wait_stable_contains("Create Solana wallet", READY_TIMEOUT)?;
    pane.send_secret_literal(&passphrase)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Confirm the value", READY_TIMEOUT)?;
    pane.send_secret_literal(&passphrase)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Wallet recovery — secure view", READY_TIMEOUT)?;
    wait_for_rpc_method(&server, "getTokenAccountsByOwner").await?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Unlock wallet", READY_TIMEOUT)?;
    pane.send_secret_literal(&passphrase)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("affordable with", READY_TIMEOUT)?;
    select_label(pane, "Starter — 1.00 USDC")?;
    pane.wait_stable_contains("Confirm Starter plan", READY_TIMEOUT)?;
    select_label(pane, "Pay 1.00 USDC")?;
    if let Err(error) = pane.wait_stable_contains("Payment confirmed", READY_TIMEOUT) {
        let transcript = redacted_request_transcript(&server).await;
        anyhow::bail!("{error}\nredacted loopback request transcript:\n{transcript}");
    }
    wait_for_request_path(&server, "/v1/account").await?;
    pane.wait_stable_contains(
        "Active 2026-09-01T00:00:00Z through 2026-10-01T00:00:00Z",
        READY_TIMEOUT,
    )?;
    select_label(pane, "Done")?;
    pane.wait_stable_contains("Receive", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)?;
    pane.wait_stable_contains("Payment confirmed:", READY_TIMEOUT)?;

    let config = fs::read_to_string(home.path().join("config.toml"))?;
    ensure!(
        config.contains("model_provider = \"ambient\""),
        "reconciled deferred Plan overrode the usable existing current provider:\n{config}"
    );
    ensure!(
        !pane
            .capture_scrollback_tail(4_000)?
            .contains("via Corbanu Plan standard"),
        "deferred fallback receipt reconciliation selected the Plan provider"
    );
    capture_success_evidence(
        "fresh-wallet-plan-success",
        &binary,
        home.path(),
        pane,
        &server,
        &[
            ambient_canary.as_str(),
            passphrase.as_str(),
            plan_key.as_str(),
        ],
    )
    .await?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_locked_wallet_plan_load_failure_retries_and_cancels() -> Result<()> {
    if !TmuxServer::should_run("PF-53 locked wallet failure retry")? {
        return Ok(());
    }
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let binary = codex_binary(&repo_root)?;
    let home = tempdir()?;
    let server = MockServer::start().await;
    let plan_key = synthetic_canary("unused-plan-key");
    mount_plan_gateway(&server, &plan_key, /*flaky_plans*/ true).await;
    write_config(home.path(), &repo_root, &server.uri())?;
    let passphrase = synthetic_canary("locked-wallet-passphrase");
    codex_wallet::Wallet::new(home.path().to_path_buf())
        .create(&passphrase, codex_wallet::Network::Mainnet)
        .context("create locked wallet fixture")?;

    let tmux = TmuxServer::start("pf53_locked_wallet_failure_retry")?;
    register_evidence(&tmux, home.path(), &binary)?;
    let session = tmux.new_session(session_spec_with_gateway(
        "pf53-locked-wallet-retry",
        &binary,
        &repo_root,
        home.path(),
        &server.uri(),
    ))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("Provider: OpenAI Codex Account", READY_TIMEOUT)?;
    let ambient_canary = synthetic_canary("locked-fallback");
    configure_api_key(pane, "Provider: Ambient API Key", &ambient_canary)?;
    select_label(pane, "Corbanu Plan")?;
    select_label(pane, "Done")?;
    pane.wait_stable_contains("Unlock wallet", READY_TIMEOUT)?;
    pane.send_secret_literal(&passphrase)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Retry loading plans", READY_TIMEOUT)?;
    select_label(pane, "Retry loading plans")?;
    pane.wait_stable_contains("Starter — 1.00 USDC", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Corbanu Terminal", READY_TIMEOUT)?;
    capture_success_evidence(
        "locked-wallet-failure-retry",
        &binary,
        home.path(),
        pane,
        &server,
        &[ambient_canary.as_str(), passphrase.as_str()],
    )
    .await?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

fn configure_api_key(pane: &TmuxPane<'_>, label: &str, secret: &str) -> Result<()> {
    select_label(pane, label)?;
    pane.wait_stable_contains("Paste or type your API key below.", READY_TIMEOUT)?;
    pane.send_secret_literal(secret)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("API key configured", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Choose a provider account", READY_TIMEOUT)?;
    Ok(())
}

fn select_label(pane: &TmuxPane<'_>, label: &str) -> Result<()> {
    for _ in 0..32 {
        let capture = pane.capture_viewport()?;
        let selected = selected_row(&capture);
        if selected
            .as_deref()
            .and_then(selected_title)
            .is_some_and(|title| selected_title_matches(title, label))
        {
            pane.send_key(TmuxKey::Enter)?;
            return Ok(());
        }
        pane.send_key(TmuxKey::Down)?;
        pane.wait_stable_until(
            "provider selection to redraw after Down",
            Duration::from_secs(5),
            |next| selected_row(next) != selected,
        )?;
    }
    anyhow::bail!(
        "could not select {label:?}; last capture:\n{}",
        pane.capture_viewport()?
    )
}

fn selected_row(capture: &str) -> Option<String> {
    capture
        .lines()
        .find(|line| strip_selection_cursor(line).is_some())
        .map(str::trim)
        .map(str::to_owned)
}

fn selected_title(row: &str) -> Option<&str> {
    let selected = strip_selection_cursor(row)?;
    if let Some((number, title)) = selected.split_once(". ")
        && number.chars().all(|character| character.is_ascii_digit())
    {
        return Some(title.trim());
    }
    Some(selected.trim())
}

fn strip_selection_cursor(row: &str) -> Option<&str> {
    let trimmed = row.trim();
    let remainder = trimmed
        .strip_prefix('>')
        .or_else(|| trimmed.strip_prefix('›'))?;
    remainder
        .chars()
        .next()
        .is_some_and(char::is_whitespace)
        .then(|| remainder.trim_start())
}

fn selected_title_matches(title: &str, requested: &str) -> bool {
    title == requested
        || title
            .strip_prefix(requested)
            .is_some_and(|inline| inline.starts_with("  "))
}

#[test]
fn selected_title_strips_only_the_cursor_and_numeric_prefix() {
    assert_eq!(
        selected_title("  > 12. Corbanu Plan  "),
        Some("Corbanu Plan")
    );
    assert_eq!(
        selected_title("  › 1. Starter — 1.00 USDC  1,000 tokens/week  "),
        Some("Starter — 1.00 USDC  1,000 tokens/week")
    );
    assert_eq!(
        selected_title("> 3. Provider: Corbanu Plan API Key"),
        Some("Provider: Corbanu Plan API Key")
    );
    assert_eq!(
        selected_title("› Cancel  Return to provider setup"),
        Some("Cancel  Return to provider setup")
    );
    assert_eq!(
        selected_title("> Pay 1.00 USDC  Purchase Starter"),
        Some("Pay 1.00 USDC  Purchase Starter")
    );
    assert_eq!(selected_title(">_ Corbanu Terminal"), None);
    assert_eq!(selected_title("  12. Corbanu Plan"), None);
    assert_eq!(selected_title("> Corbanu Plan"), Some("Corbanu Plan"));
}

#[test]
fn selected_row_accepts_provider_and_plan_cursors_but_rejects_header_marker() {
    assert_eq!(
        selected_row(">_ Corbanu Terminal\n> 2. Provider: Ambient API Key"),
        Some("> 2. Provider: Ambient API Key".to_string())
    );
    assert_eq!(
        selected_row(">_ Corbanu Terminal\n› 1. Starter — 1.00 USDC  1,000 tokens/week"),
        Some("› 1. Starter — 1.00 USDC  1,000 tokens/week".to_string())
    );
    assert_eq!(selected_row(">_ Corbanu Terminal"), None);
}

#[test]
fn selected_title_match_allows_only_exact_or_inline_column_metadata() {
    assert!(selected_title_matches("Corbanu Plan", "Corbanu Plan"));
    assert!(selected_title_matches(
        "Starter — 1.00 USDC  1,000 tokens/week",
        "Starter — 1.00 USDC",
    ));
    assert!(!selected_title_matches(
        "Provider: Corbanu Plan API Key",
        "Corbanu Plan",
    ));
    assert!(!selected_title_matches(
        "Starter — 1.00 USDC extra",
        "Starter — 1.00 USDC",
    ));
}

#[cfg(unix)]
#[test]
fn canary_tree_scan_skips_unix_sockets_and_scans_regular_files() -> Result<()> {
    use std::os::unix::net::UnixListener;

    let directory = tempdir()?;
    let canary = synthetic_canary("walker");
    let _socket = UnixListener::bind(directory.path().join("wallet-daemon.sock"))?;

    assert!(!tree_contains(directory.path(), canary.as_bytes())?);

    fs::write(directory.path().join("observable.log"), canary.as_bytes())?;
    assert!(tree_contains(directory.path(), canary.as_bytes())?);
    Ok(())
}

fn submit_and_wait(pane: &TmuxPane<'_>, prompt: &str, response: &str) -> Result<()> {
    wait_chat_ready(pane)?;
    pane.send_literal(prompt)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(response, READY_TIMEOUT)?;
    Ok(())
}

fn wait_chat_ready(pane: &TmuxPane<'_>) -> Result<()> {
    pane.wait_stable_until(
        "chat composer without a selection overlay",
        READY_TIMEOUT,
        |capture| {
            capture.contains("/model to change")
                && !capture.contains("Press enter to confirm or esc to go back")
        },
    )?;
    Ok(())
}

fn exit_tui(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/exit")?;
    pane.send_key(TmuxKey::Enter)?;
    Ok(())
}

fn session_spec(name: &str, binary: &Path, repo_root: &Path, home: &Path) -> SessionSpec {
    SessionSpec::new(
        name,
        TerminalSize::new(140, 44),
        CommandSpec::new(binary)
            .env("CODEX_HOME", home)
            .env("CORBANU_HOME", home)
            .env("PFTERMINAL_HOME", home)
            .env(
                "RUST_LOG",
                "warn,codex_tui=debug,codex_login=debug,codex_secrets=debug",
            )
            .arg("-c")
            .arg("analytics.enabled=false")
            .arg("-c")
            .arg("tui.animations=false")
            .arg("--no-alt-screen")
            .arg("-C")
            .arg(repo_root),
    )
    .current_dir(repo_root)
}

fn session_spec_with_gateway(
    name: &str,
    binary: &Path,
    repo_root: &Path,
    home: &Path,
    gateway: &str,
) -> SessionSpec {
    SessionSpec::new(
        name,
        TerminalSize::new(140, 44),
        CommandSpec::new(binary)
            .env("CODEX_HOME", home)
            .env("CORBANU_HOME", home)
            .env("PFTERMINAL_HOME", home)
            .env("PFTERMINAL_PLAN_GATEWAY_URL", gateway)
            .env(
                "RUST_LOG",
                "warn,codex_tui=debug,codex_login=debug,codex_secrets=debug",
            )
            .arg("-c")
            .arg("analytics.enabled=false")
            .arg("-c")
            .arg("tui.animations=false")
            .arg("--no-alt-screen")
            .arg("-C")
            .arg(repo_root),
    )
    .current_dir(repo_root)
}

async fn mount_plan_gateway(server: &MockServer, plan_key: &str, flaky_plans: bool) {
    let plan_requests = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path("/v1/plans"))
        .respond_with({
            let plan_requests = Arc::clone(&plan_requests);
            let uri = server.uri();
            move |_request: &wiremock::Request| {
                let attempt = plan_requests.fetch_add(1, Ordering::SeqCst);
                if flaky_plans && attempt < 1 {
                    return ResponseTemplate::new(503);
                }
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "plans": [{
                        "id": "starter",
                        "priceUsdc": "1.00",
                        "amountAtomic": "1000000",
                        "weeklyTokenLimit": 1000,
                        "monthlyTokenLimit": 4000
                    }],
                    "payment": {
                        "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                        "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                        "payTo": "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd",
                        "rpcUrl": format!("{uri}/rpc")
                    }
                }))
            }
        })
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(|request: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap_or_default();
            match body.get("method").and_then(serde_json::Value::as_str) {
                Some("getBalance") => ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"value":1_000_000_000u64}}),
                ),
                Some("getTokenAccountsByOwner") => {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "jsonrpc":"2.0","id":1,
                        "result":{"value":[{
                            "account":{"data":{"parsed":{"info":{
                                "tokenAmount":{"amount":"5000000"}
                            }}}}
                        }]}
                    }))
                }
                Some("getLatestBlockhash") => {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "jsonrpc":"2.0","id":1,
                        "result":{"value":{"blockhash":"11111111111111111111111111111111"}}
                    }))
                }
                _ => ResponseTemplate::new(400),
            }
        })
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/subscriptions/starter"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"transaction":"pf53-loopback-settlement"})),
        )
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/keys/challenge"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"challenge":"pf53-loopback-ownership"})),
        )
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/keys/wallet"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id":"pf53-loopback-key-id",
            "key":plan_key,
            "displayPrefix":"pf53..."
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/account"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "walletAddress":"pf53-loopback-wallet",
            "period":{
                "transaction":"pf53-loopback-settlement",
                "planId":"starter",
                "startsAt":"2026-09-01T00:00:00Z",
                "endsAt":"2026-10-01T00:00:00Z",
                "monthlyLimitTokens":4000,
                "monthlyUsedTokens":0,
                "monthlyReservedTokens":0
            },
            "weekly":{
                "endsAt":"2026-09-08T00:00:00Z",
                "limitTokens":1000,
                "usedTokens":0,
                "reservedTokens":0
            },
            "monthlyRemainingTokens":4000,
            "weeklyRemainingTokens":1000,
            "queuedPeriods":[]
        })))
        .mount(server)
        .await;
}

async fn redacted_request_transcript(server: &MockServer) -> String {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(index, request)| {
            let rpc_method = serde_json::from_slice::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|body| body.get("method").cloned())
                .and_then(|method| method.as_str().map(str::to_owned));
            match rpc_method {
                Some(rpc_method) => format!(
                    "{index}: {} {} rpc.method={rpc_method}",
                    request.method,
                    request.url.path()
                ),
                None => format!("{index}: {} {}", request.method, request.url.path()),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn wait_for_request_path(server: &MockServer, expected_path: &str) -> Result<()> {
    let deadline = std::time::Instant::now() + READY_TIMEOUT;
    while std::time::Instant::now() < deadline {
        let requests = server.received_requests().await.unwrap_or_default();
        if requests
            .iter()
            .any(|request| request.url.path() == expected_path)
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let transcript = redacted_request_transcript(server).await;
    anyhow::bail!(
        "timed out waiting for loopback request path {expected_path}\nredacted loopback request transcript:\n{transcript}"
    )
}

async fn wait_for_rpc_method(server: &MockServer, method_name: &str) -> Result<()> {
    let deadline = std::time::Instant::now() + READY_TIMEOUT;
    while std::time::Instant::now() < deadline {
        let requests = server.received_requests().await.unwrap_or_default();
        if requests.iter().any(|request| {
            serde_json::from_slice::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|body| body.get("method").cloned())
                .and_then(|method| method.as_str().map(str::to_owned))
                .as_deref()
                == Some(method_name)
        }) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    anyhow::bail!("timed out waiting for loopback RPC method {method_name}")
}

fn write_config(home: &Path, repo_root: &Path, server_uri: &str) -> Result<()> {
    fs::create_dir_all(home.join("log"))?;
    fs::write(
        home.join("config.toml"),
        format!(
            r#"model = "gpt-5.4"
model_provider = "openai"
suppress_unstable_features_warning = true
log_dir = "{}"

[model_providers.pf53-loopback]
name = "PF53 Loopback"
base_url = "{server_uri}/v1"
env_key = "PF53_LOOPBACK_API_KEY"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0

[projects."{}"]
trust_level = "trusted"
"#,
            home.join("log").display(),
            repo_root.display(),
        ),
    )
    .context("write PF-53 tmux config")
}

fn register_evidence(tmux: &TmuxServer, home: &Path, binary: &Path) -> Result<()> {
    let hash = format!("{:x}", Sha256::digest(fs::read(binary)?));
    fs::write(
        home.join("binary.sha256"),
        format!("{hash}  {}\n", binary.display()),
    )?;
    tmux.register_artifact("binary.sha256", home.join("binary.sha256"));
    tmux.register_artifact("config.toml", home.join("config.toml"));
    tmux.register_artifact("codex-tui.log", home.join("log/codex-tui.log"));
    Ok(())
}

async fn capture_success_evidence(
    scenario: &str,
    binary: &Path,
    home: &Path,
    pane: &TmuxPane<'_>,
    server: &MockServer,
    canaries: &[&str],
) -> Result<()> {
    let viewport = pane.capture_viewport()?;
    let scrollback = pane.capture_scrollback_tail(4_000)?;
    let directory = PathBuf::from("target/tmux-artifacts").join(format!("pf53-{scenario}"));
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("viewport.txt"), &viewport)?;
    fs::write(directory.join("scrollback.txt"), &scrollback)?;
    let binary_hash = format!("{:x}", Sha256::digest(fs::read(binary)?));
    fs::write(
        directory.join("binary.sha256"),
        format!("{binary_hash}  {}\n", binary.display()),
    )?;

    let requests = server.received_requests().await.unwrap_or_default();
    let request_bodies = requests
        .iter()
        .flat_map(|request| request.body.iter().copied())
        .collect::<Vec<_>>();
    for canary in canaries {
        let needle = canary.as_bytes();
        ensure!(
            !viewport.contains(canary) && !scrollback.contains(canary),
            "secret canary appeared in terminal evidence"
        );
        ensure!(
            !request_bodies
                .windows(needle.len())
                .any(|window| window == needle),
            "secret canary appeared in a loopback request body"
        );
        ensure!(
            !tree_contains_except_custody(home, needle)?,
            "secret canary appeared in observable isolated-home files"
        );
        ensure!(
            !tree_contains(&directory, needle)?,
            "secret canary appeared in emitted success artifacts"
        );
    }
    Ok(())
}

fn synthetic_canary(label: &str) -> String {
    format!("pf53-{label}-{}", Uuid::new_v4())
}

fn tree_contains_except_custody(root: &Path, needle: &[u8]) -> Result<bool> {
    if root.file_name().and_then(|name| name.to_str()) == Some("provider_auth.json") {
        return Ok(false);
    }
    tree_contains(root, needle)
}

fn tree_contains(root: &Path, needle: &[u8]) -> Result<bool> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Ok(false);
    }
    if file_type.is_file() {
        let bytes = fs::read(root)?;
        return Ok(bytes.windows(needle.len()).any(|window| window == needle));
    }
    if !file_type.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if tree_contains_except_custody(&entry.path(), needle)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn response(text: &str) -> String {
    responses::sse(vec![
        responses::ev_response_created("pf53-response"),
        responses::ev_assistant_message("pf53-message", text),
        responses::ev_completed("pf53-response"),
    ])
}

fn codex_binary(repo_root: &Path) -> Result<PathBuf> {
    for name in ["codex", "corbanu", "pfterminal"] {
        if let Ok(binary) = codex_utils_cargo_bin::cargo_bin(name) {
            return Ok(binary);
        }
    }
    for name in ["codex", "corbanu", "pfterminal"] {
        let binary = repo_root.join("codex-rs/target/debug").join(name);
        if binary.is_file() {
            return Ok(binary);
        }
    }
    anyhow::bail!("build the Corbanu CLI binary before running PF-53 tmux tests")
}
