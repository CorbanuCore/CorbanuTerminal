use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_reauth_openai_environment_key_preserves_external_ownership() -> Result<()> {
    if !TmuxServer::should_run("PF-58 OpenAI environment ownership")? {
        return Ok(());
    }
    let fixture = Fixture::new("reauth-openai-env", /*openai_auth*/ false).await?;
    let tmux = fixture.tmux()?;
    let canary = synthetic_canary("codex-environment");
    let command = CommandSpec::new(&fixture.binary)
        .env("CODEX_HOME", fixture.home.path())
        .env("CORBANU_HOME", fixture.home.path())
        .env("PFTERMINAL_HOME", fixture.home.path())
        .env("CODEX_API_KEY", &canary)
        .env("PF54_PRIMARY_KEY", "pf58-primary-fixture")
        .env("RUST_LOG", "trace")
        .arg("--no-alt-screen")
        .arg("-C")
        .arg(&fixture.repo_root);
    let session = tmux.new_session(
        SessionSpec::new("pf58-openai-env", TerminalSize::new(140, 44), command)
            .current_dir(&fixture.repo_root),
    )?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;
    open_manager(pane)?;
    focus_label(pane, "OpenAI")?;
    pane.send_literal("r")?;
    let capture = pane.wait_stable_contains("A vault key cannot override it", READY_TIMEOUT)?;
    ensure!(!capture.contains("Replace the saved API key"));
    pane.send_key(TmuxKey::Escape)?;
    capture_success("reauth-openai-env", &fixture, pane, &[&canary])?;
    close_manager(pane)?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_reauth_openai_account_cancel_then_request_without_model_change() -> Result<()> {
    if !TmuxServer::should_run("PF-58 OpenAI account recovery")? {
        return Ok(());
    }
    let fixture = Fixture::new("reauth-openai", /*openai_auth*/ false).await?;
    let config_path = fixture.home.path().join("config.toml");
    let original = fs::read_to_string(&config_path)?.replacen(
        &format!("model_provider = \"{PRIMARY_PROVIDER}\""),
        "model_provider = \"openai\"",
        1,
    );
    fs::write(
        &config_path,
        format!(
            "openai_base_url = {:?}\nchatgpt_base_url = {:?}\n{original}",
            format!("{}/v1", fixture.server.uri()),
            format!("{}/backend-api", fixture.server.uri())
        ),
    )?;
    app_test_support::write_chatgpt_auth(
        fixture.home.path(),
        app_test_support::ChatGptAuthFixture::new("pf58-old-account-token")
            .account_id("pf58-account")
            .chatgpt_account_id("pf58-account")
            .email("pf58@example.invalid")
            .plan_type("pro"),
        codex_login::AuthCredentialsStoreMode::File,
    )?;
    let auth: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fixture.home.path().join("auth.json"))?)?;
    let rejection = Mock::given(method("POST"))
        .and(wiremock::matchers::path_regex(".*/responses"))
        .and(wiremock::matchers::header(
            "authorization",
            "Bearer pf58-old-account-token",
        ))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"error":{"code":"token_expired"}})),
        )
        .with_priority(1)
        .mount_as_scoped(&fixture.server)
        .await;
    let refresh_rejection = Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"error":{"code":"refresh_token_reused"}})),
        )
        .with_priority(2)
        .mount_as_scoped(&fixture.server)
        .await;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf58-openai-account"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;
    submit_and_wait(
        pane,
        "exercise synthetic account failure",
        "OpenAI (openai) credential was rejected",
    )?;
    open_manager(pane)?;
    focus_label(pane, "OpenAI")?;
    pane.wait_stable_contains("Credential needs attention", READY_TIMEOUT)?;
    pane.send_literal("r")?;
    select_label(pane, "Sign in to OpenAI again")?;
    pane.wait_stable_contains("OpenAI account login", READY_TIMEOUT)?;
    cancel_account_auth(pane)?;
    open_manager(pane)?;
    focus_label(pane, "OpenAI")?;
    pane.send_literal("r")?;
    select_label(pane, "Sign in to OpenAI again")?;
    pane.wait_stable_contains("OpenAI account login", READY_TIMEOUT)?;
    drop(rejection);
    drop(refresh_rejection);
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token":"pf58-new-account-token", "refresh_token":"pf58-new-refresh",
            "id_token":auth["tokens"]["id_token"], "token_type":"Bearer", "expires_in":3600
        })))
        // Scoped mock removal is asynchronous; the new exchange must win even
        // while the old refresh-rejection guard is being removed.
        .with_priority(1)
        .mount(&fixture.server)
        .await;
    Mock::given(method("POST")).and(path("/api/accounts/deviceauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "authorization_code":"pf58-code", "code_challenge":"pf58-challenge", "code_verifier":"pf58-verifier"
        }))).with_priority(1).mount(&fixture.server).await;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    wait_manager_row(pane, fixture.home.path(), "OpenAI", "Enabled · configured")?;
    close_manager(pane)?;
    let _response = responses::mount_sse_once(
        &fixture.server,
        response("PF58 account recovered without restart"),
    )
    .await;
    submit_and_wait(
        pane,
        "continue after account recovery",
        "PF58 account recovered without restart",
    )?;
    ensure_authorization_seen(&fixture.server, "pf58-new-account-token").await?;
    let config: toml::Value = toml::from_str(&fs::read_to_string(config_path)?)?;
    ensure!(config["model_provider"].as_str() == Some("openai"));
    ensure!(config["model"].as_str() == Some("fixture-model"));
    // A configured account must also offer a real, cancelable manual sign-in.
    fixture.server.reset().await;
    mock_openai_device_code(&fixture.server).await;
    open_manager(pane)?;
    focus_label(pane, "OpenAI")?;
    pane.send_literal("r")?;
    select_label(pane, "Sign in to OpenAI again")?;
    pane.wait_stable_contains("OpenAI account login", READY_TIMEOUT)?;
    cancel_account_auth(pane)?;
    capture_success(
        "reauth-openai",
        &fixture,
        pane,
        &[
            "pf58-old-account-token",
            "pf58-new-account-token",
            "pf58-new-refresh",
        ],
    )?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_reauth_api_key_failure_cancel_replace_and_same_session_request() -> Result<()> {
    if !TmuxServer::should_run("PF-58 credential recovery")? {
        return Ok(());
    }
    let fixture = Fixture::new("reauth-key", /*openai_auth*/ true).await?;
    let tmux = fixture.tmux()?;
    let old_key = synthetic_canary("old-key");
    let new_key = synthetic_canary("replacement-key");
    let first = tmux.new_session(fixture.session("pf58-setup"))?;
    wait_chat_ready(first.primary_pane())?;
    configure_managed_provider(first.primary_pane(), fixture.home.path(), &old_key)?;
    exit_tui(first.primary_pane())?;
    first.wait_for_exit(READY_TIMEOUT)?;
    select_config_provider(fixture.home.path(), MANAGED_PROVIDER)?;
    Mock::given(method("POST"))
        .and(wiremock::matchers::path_regex(".*/responses"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": { "code": "invalid_api_key", "message": "Synthetic credential rejected" }
        })))
        .mount(&fixture.server)
        .await;
    let second = tmux.new_session(fixture.session("pf58-recover"))?;
    let pane = second.primary_pane();
    submit_and_wait(
        pane,
        "exercise synthetic credential failure",
        "PF54 Managed (pf54-managed) credential was rejected",
    )?;
    open_manager(pane)?;
    focus_label(pane, "PF54 Managed")?;
    pane.wait_stable_contains("Credential needs attention", READY_TIMEOUT)?;
    pane.send_literal("r")?;
    select_label(pane, "Replace the saved API key")?;
    pane.wait_stable_contains("API key — masked", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    focus_label(pane, "PF54 Managed")?;
    pane.send_literal("r")?;
    select_label(pane, "Replace the saved API key")?;
    pane.wait_stable_contains("API key — masked", READY_TIMEOUT)?;
    pane.send_secret_literal(&new_key)?;
    pane.send_key(TmuxKey::Enter)?;
    wait_manager_row(
        pane,
        fixture.home.path(),
        "PF54 Managed",
        "Enabled · configured",
    )?;
    close_manager(pane)?;
    fixture.server.reset().await;
    let _response =
        responses::mount_sse_once(&fixture.server, response("PF58 recovered without restart"))
            .await;
    submit_and_wait(
        pane,
        "prove the replacement credential is used",
        "PF58 recovered without restart",
    )?;
    ensure_authorization_seen(&fixture.server, &new_key).await?;
    let config: toml::Value = toml::from_str(&fs::read_to_string(
        fixture.home.path().join("config.toml"),
    )?)?;
    ensure!(config["model_provider"].as_str() == Some(MANAGED_PROVIDER));
    ensure!(config["model"].as_str() == Some("fixture-model"));
    capture_success("reauth-key", &fixture, pane, &[&old_key, &new_key])?;
    exit_tui(pane)?;
    second.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_reauth_environment_guidance_does_not_offer_vault_override() -> Result<()> {
    if !TmuxServer::should_run("PF-58 external credential guidance")? {
        return Ok(());
    }
    let fixture = Fixture::new("reauth-environment", /*openai_auth*/ true).await?;
    let config_before = fs::read(fixture.home.path().join("config.toml"))?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf58-environment"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;
    open_manager(pane)?;
    focus_label(pane, "PF54 Environment")?;
    pane.send_literal("r")?;
    let capture = pane.wait_stable_contains("A vault key cannot override it", READY_TIMEOUT)?;
    ensure!(!capture.contains("Replace the saved API key"));
    pane.send_key(TmuxKey::Escape)?;
    ensure!(fs::read(fixture.home.path().join("config.toml"))? == config_before);
    capture_success("reauth-environment", &fixture, pane, &[])?;
    close_manager(pane)?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}
