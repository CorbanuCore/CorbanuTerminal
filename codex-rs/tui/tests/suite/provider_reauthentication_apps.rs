//! An off-origin Apps endpoint must not receive the account credential.
use super::*;
use wiremock::matchers::path_regex;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_untrusted_apps_origin_preserves_account_and_healthy_model_route() -> Result<()> {
    if !TmuxServer::should_run("PF-58 untrusted Apps origin")? {
        return Ok(());
    }
    let fixture = Fixture::new("apps-origin-boundary", false).await?;
    let config_path = fixture.home.path().join("config.toml");
    let original = fs::read_to_string(&config_path)?;
    fs::write(
        &config_path,
        format!(
            "chatgpt_base_url = {:?}\n{original}",
            format!("{}/backend-api", fixture.server.uri())
        ),
    )?;
    let canary = "pf58-apps-account-origin-canary";
    app_test_support::write_chatgpt_auth(
        fixture.home.path(),
        app_test_support::ChatGptAuthFixture::new(canary)
            .account_id("pf58-apps-account")
            .chatgpt_account_id("pf58-apps-account")
            .email("pf58@example.invalid")
            .plan_type("pro"),
        codex_login::AuthCredentialsStoreMode::File,
    )?;
    let auth_before = fs::read(fixture.home.path().join("auth.json"))?;
    Mock::given(method("POST"))
        .and(path_regex(".*/ps/mcp/?$"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_json(serde_json::json!({"error":"synthetic untrusted Apps endpoint"})),
        )
        .mount(&fixture.server)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(".*/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            response("PF58 healthy model unchanged"),
            "text/event-stream",
        ))
        .mount(&fixture.server)
        .await;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf58-apps-origin"))?;
    let pane = session.primary_pane();
    pane.wait_stable_contains("MCP startup incomplete", READY_TIMEOUT)?;
    wait_chat_ready(pane)?;
    submit_and_wait(
        pane,
        "model remains usable despite off-origin apps failure",
        "PF58 healthy model unchanged",
    )?;
    open_manager(pane)?;
    focus_label(pane, "OpenAI")?;
    // This account was seeded before startup, not saved through setup; no
    // eligibility write is expected merely from inspecting its metadata.
    pane.wait_stable_until("account status unchanged", READY_TIMEOUT, |capture| {
        selected_row(capture)
            .is_some_and(|row| row.contains("OpenAI") && row.contains("Enabled · configured"))
    })?;
    close_manager(pane)?;
    let requests = fixture.server.received_requests().await.unwrap_or_default();
    let apps: Vec<_> = requests
        .iter()
        .filter(|request| request.url.path().ends_with("/ps/mcp"))
        .collect();
    ensure!(!apps.is_empty(), "Apps fixture was never reached");
    ensure!(
        apps.iter()
            .all(|request| !request.headers.contains_key("authorization")),
        "account auth escaped to an untrusted origin"
    );
    ensure!(
        fs::read(fixture.home.path().join("auth.json"))? == auth_before,
        "unrelated service failure changed account credentials"
    );
    let config: toml::Value = toml::from_str(&fs::read_to_string(config_path)?)?;
    ensure!(config["model_provider"].as_str() == Some(PRIMARY_PROVIDER));
    ensure!(config["model"].as_str() == Some("fixture-model"));
    capture_success("apps-origin-boundary", &fixture, pane, &[canary])?;
    let receipt: Vec<_> = apps.into_iter().map(|request| {
        let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap_or_default();
        serde_json::json!({"method":body["method"], "authorization_attached":request.headers.contains_key("authorization")})
    }).collect();
    fs::write(
        "target/tmux-artifacts/pf54-apps-origin-boundary/service-requests.json",
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}
