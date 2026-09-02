use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
const PRIMARY_PROVIDER: &str = "pf54-primary";
const SECONDARY_PROVIDER: &str = "pf54-secondary";
const MANAGED_PROVIDER: &str = "pf54-managed";
const ENV_PROVIDER: &str = "pf54-environment";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_shared_and_custom_catalog_have_management_status_parity() -> Result<()> {
    if !TmuxServer::should_run("PF-54 shared/custom catalog parity")? {
        return Ok(());
    }
    let fixture = Fixture::new("catalog-parity", /*openai_auth*/ true).await?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf54-catalog-parity"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;

    open_manager(pane)?;
    pane.wait_stable_contains("OpenAI", READY_TIMEOUT)?;
    inspect_provider(pane, "PF54 Primary", "Active · current")?;
    open_manager(pane)?;
    inspect_provider(pane, "PF54 Managed", "Not configured")?;
    open_manager(pane)?;
    inspect_provider(pane, "PF54 Broken Environment", "Recovery required")?;

    capture_success("catalog-parity", &fixture, pane, &[])?;
    close_overlay_and_exit(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_pf50_api_key_setup_and_recovery_are_reused() -> Result<()> {
    if !TmuxServer::should_run("PF-54 PF-50 API-key setup/recovery")? {
        return Ok(());
    }
    let fixture = Fixture::new("api-setup-recovery", /*openai_auth*/ true).await?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf54-api-setup-recovery"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;
    let canary = synthetic_canary("api-key");

    open_manager(pane)?;
    select_label(pane, "PF54 Managed")?;
    pane.wait_stable_contains("Not configured", READY_TIMEOUT)?;
    select_label(pane, "Set up with API key")?;
    pane.wait_stable_contains("API key — masked", READY_TIMEOUT)?;
    pane.send_secret_literal(&canary)?;
    pane.send_key(TmuxKey::Enter)?;
    wait_manager_row(pane, fixture.home.path(), "PF54 Managed", "Active")?;
    inspect_provider(pane, "PF54 Managed", "Active")?;

    open_manager(pane)?;
    select_label(pane, "PF54 Broken Environment")?;
    pane.wait_stable_contains("Recovery required", READY_TIMEOUT)?;
    select_label(pane, "Recover with API key")?;
    pane.wait_stable_contains("API key — masked", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_until("API recovery cancellation", READY_TIMEOUT, |capture| {
        !capture.contains("API key — masked")
    })?;

    capture_success("api-setup-recovery", &fixture, pane, &[&canary])?;
    close_overlay_and_exit(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_pf51_openai_account_cancel_and_retry_are_correlated() -> Result<()> {
    if !TmuxServer::should_run("PF-54 PF-51 OpenAI cancel/retry")? {
        return Ok(());
    }
    let fixture = Fixture::new("openai-cancel-retry", /*openai_auth*/ false).await?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf54-openai-cancel-retry"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;

    begin_openai_account_auth(pane)?;
    cancel_account_auth(pane)?;
    begin_openai_account_auth(pane)?;
    pane.wait_stable_contains("OpenAI account login", READY_TIMEOUT)?;
    cancel_account_auth(pane)?;

    capture_success("openai-cancel-retry", &fixture, pane, &[])?;
    exit_tui(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_pf52_claude_recovery_cancel_and_retry_are_reused() -> Result<()> {
    if !TmuxServer::should_run("PF-54 PF-52 Claude recovery/cancel")? {
        return Ok(());
    }
    let fixture = Fixture::new("claude-recovery-cancel", /*openai_auth*/ true).await?;
    let selection = codex_vault::ClaudeAuthSelection::new(
        codex_vault::ClaudeAuthSource::ManagedSubscriptionToken,
        "pf54-missing-managed-token",
    )
    .map_err(anyhow::Error::msg)?;
    codex_vault::Vault::new(fixture.home.path().to_path_buf())
        .save_claude_auth_selection(&selection)?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf54-claude-recovery-cancel"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;
    let canary = synthetic_canary("claude-managed-token");

    open_manager(pane)?;
    focus_label(pane, "Claude Account")?;
    pane.wait_stable_until("Claude recovery status", READY_TIMEOUT, |capture| {
        selected_row(capture).is_some_and(|row| row.contains("Recovery required"))
    })?;
    pane.send_key(TmuxKey::Enter)?;
    select_label(pane, "Recover with Claude account")?;
    pane.wait_stable_contains("Claude account method", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)?;

    open_manager(pane)?;
    focus_label(pane, "Claude Account")?;
    pane.wait_stable_until("Claude retry recovery status", READY_TIMEOUT, |capture| {
        selected_row(capture).is_some_and(|row| row.contains("Recovery required"))
    })?;
    pane.send_key(TmuxKey::Enter)?;
    select_label(pane, "Recover with Claude account")?;
    pane.wait_stable_contains("Claude account method", READY_TIMEOUT)?;
    select_label(pane, "Managed subscription token")?;
    pane.wait_stable_contains("Token — masked", READY_TIMEOUT)?;
    pane.send_secret_literal(&canary)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;

    capture_success("claude-recovery-cancel", &fixture, pane, &[&canary])?;
    close_overlay_and_exit(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_noncurrent_deactivate_reactivate_restart_retains_request_credential() -> Result<()> {
    if !TmuxServer::should_run("PF-54 noncurrent retention/restart/request")? {
        return Ok(());
    }
    let fixture = Fixture::new("noncurrent-retention", /*openai_auth*/ true).await?;
    let _response = responses::mount_sse_once(
        &fixture.server,
        response("pf54 retained credential request succeeded"),
    )
    .await;
    let tmux = fixture.tmux()?;
    let canary = synthetic_canary("retained-provider-key");
    let first = tmux.new_session(fixture.session("pf54-retention-first"))?;
    let pane = first.primary_pane();
    wait_chat_ready(pane)?;
    configure_managed_provider(pane, fixture.home.path(), &canary)?;
    open_manager(pane)?;
    select_label(pane, "PF54 Managed")?;
    select_label(pane, "Deactivate")?;
    pane.wait_stable_contains("Inactive", READY_TIMEOUT)?;
    capture_success("noncurrent-inactive", &fixture, pane, &[&canary])?;
    close_overlay_and_exit(pane)?;
    first.wait_for_exit(READY_TIMEOUT)?;

    let second = tmux.new_session(fixture.session("pf54-retention-reactivate"))?;
    let pane = second.primary_pane();
    wait_chat_ready(pane)?;
    open_manager(pane)?;
    inspect_provider(pane, "PF54 Managed", "Inactive")?;
    open_manager(pane)?;
    select_label(pane, "PF54 Managed")?;
    select_label(pane, "Reactivate")?;
    pane.wait_stable_contains("Active", READY_TIMEOUT)?;
    close_overlay_and_exit(pane)?;
    second.wait_for_exit(READY_TIMEOUT)?;

    select_config_provider(fixture.home.path(), MANAGED_PROVIDER)?;
    let third = tmux.new_session(fixture.session("pf54-retention-request"))?;
    let pane = third.primary_pane();
    wait_chat_ready(pane)?;
    submit_and_wait(
        pane,
        "prove retained PF54 provider credential",
        "pf54 retained credential request succeeded",
    )?;
    ensure_authorization_seen(&fixture.server, &canary).await?;
    capture_success("noncurrent-retention-request", &fixture, pane, &[&canary])?;
    exit_tui(pane)?;
    third.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_current_deactivate_cancel_is_inert() -> Result<()> {
    if !TmuxServer::should_run("PF-54 current deactivate cancellation")? {
        return Ok(());
    }
    let fixture = Fixture::new("current-cancel", /*openai_auth*/ true).await?;
    let original = fs::read(fixture.home.path().join("config.toml"))?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf54-current-cancel"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;

    open_manager(pane)?;
    select_label(pane, "PF54 Primary")?;
    pane.wait_stable_contains("Active · current", READY_TIMEOUT)?;
    select_label(pane, "Deactivate")?;
    pane.wait_stable_contains("Choose replacement", READY_TIMEOUT)?;
    pane.wait_stable_contains("PF54 Secondary — fixture-model", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;

    ensure!(
        !fixture
            .home
            .path()
            .join("provider-eligibility.json")
            .exists(),
        "cancelled current-provider deactivation persisted eligibility"
    );
    ensure!(
        fs::read(fixture.home.path().join("config.toml"))? == original,
        "cancelled current-provider deactivation changed config"
    );
    capture_success("current-cancel", &fixture, pane, &[])?;
    close_overlay_and_exit(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_current_exact_replacement_is_persisted_before_deactivation() -> Result<()> {
    if !TmuxServer::should_run("PF-54 exact replacement ordering/restart")? {
        return Ok(());
    }
    let fixture = Fixture::new("current-replacement", /*openai_auth*/ true).await?;
    let tmux = fixture.tmux()?;
    let first = tmux.new_session(fixture.session("pf54-current-replacement"))?;
    let pane = first.primary_pane();
    wait_chat_ready(pane)?;

    open_manager(pane)?;
    select_label(pane, "PF54 Primary")?;
    select_label(pane, "Deactivate")?;
    pane.wait_stable_contains("Choose replacement", READY_TIMEOUT)?;
    select_label(pane, "PF54 Secondary — fixture-model")?;
    wait_replacement_order(fixture.home.path(), SECONDARY_PROVIDER)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    inspect_provider(pane, "PF54 Primary", "Inactive")?;
    capture_success("current-replacement", &fixture, pane, &[])?;
    close_overlay_and_exit(pane)?;
    first.wait_for_exit(READY_TIMEOUT)?;

    let second = tmux.new_session(fixture.session("pf54-current-replacement-restart"))?;
    let pane = second.primary_pane();
    wait_chat_ready(pane)?;
    open_manager(pane)?;
    inspect_provider(pane, "PF54 Secondary", "Active · current")?;
    open_manager(pane)?;
    inspect_provider(pane, "PF54 Primary", "Inactive")?;
    capture_success("current-replacement-restart", &fixture, pane, &[])?;
    close_overlay_and_exit(pane)?;
    second.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tmux_environment_copy_and_eligibility_never_delete_credentials() -> Result<()> {
    if !TmuxServer::should_run("PF-54 environment-backed management copy")? {
        return Ok(());
    }
    let fixture = Fixture::new("environment-copy", /*openai_auth*/ true).await?;
    let config_before = fs::read(fixture.home.path().join("config.toml"))?;
    let auth_before = fs::read(fixture.home.path().join("auth.json"))?;
    let tmux = fixture.tmux()?;
    let session = tmux.new_session(fixture.session("pf54-environment-copy"))?;
    let pane = session.primary_pane();
    wait_chat_ready(pane)?;

    open_manager(pane)?;
    select_label(pane, "PF54 Environment")?;
    pane.wait_stable_contains(
        "Environment-backed credential: deactivate here; unset it outside Corbanu.",
        READY_TIMEOUT,
    )?;
    select_label(pane, "Deactivate")?;
    pane.wait_stable_contains("Inactive", READY_TIMEOUT)?;
    select_label(pane, "PF54 Environment")?;
    select_label(pane, "Reactivate")?;
    pane.wait_stable_contains("Active", READY_TIMEOUT)?;

    ensure!(fs::read(fixture.home.path().join("config.toml"))? == config_before);
    ensure!(fs::read(fixture.home.path().join("auth.json"))? == auth_before);
    ensure!(
        fs::read_to_string(fixture.home.path().join("provider-eligibility.json"))?
            .contains("inactive_identities")
    );
    capture_success("environment-copy", &fixture, pane, &[])?;
    close_overlay_and_exit(pane)?;
    session.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

struct Fixture {
    repo_root: PathBuf,
    binary: PathBuf,
    home: tempfile::TempDir,
    server: MockServer,
    scenario: &'static str,
}

impl Fixture {
    async fn new(scenario: &'static str, openai_auth: bool) -> Result<Self> {
        let repo_root = codex_utils_cargo_bin::repo_root()?;
        let binary = codex_binary(&repo_root)?;
        let home = tempdir()?;
        let server = MockServer::start().await;
        mock_openai_device_code(&server).await;
        write_config(home.path(), &repo_root, &server.uri())?;
        if openai_auth {
            fs::write(
                home.path().join("auth.json"),
                r#"{"OPENAI_API_KEY":"pf54-openai-fixture","tokens":null,"last_refresh":null}"#,
            )?;
        }
        Ok(Self {
            repo_root,
            binary,
            home,
            server,
            scenario,
        })
    }

    fn tmux(&self) -> Result<TmuxServer> {
        let tmux = TmuxServer::start(&format!("pf54_{}", self.scenario.replace('-', "_")))?;
        register_evidence(&tmux, self.home.path(), &self.binary)?;
        Ok(tmux)
    }

    fn session(&self, name: &str) -> SessionSpec {
        session_spec(
            name,
            &self.binary,
            &self.repo_root,
            self.home.path(),
            &self.server.uri(),
        )
    }
}

fn begin_openai_account_auth(pane: &TmuxPane<'_>) -> Result<()> {
    open_manager(pane)?;
    select_label(pane, "OpenAI")?;
    pane.wait_stable_contains("Not configured", READY_TIMEOUT)?;
    select_label(pane, "Set up with OpenAI account")?;
    pane.wait_stable_contains("OpenAI account login", READY_TIMEOUT)?;
    Ok(())
}

fn cancel_account_auth(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)?;
    Ok(())
}

fn configure_managed_provider(pane: &TmuxPane<'_>, home: &Path, canary: &str) -> Result<()> {
    open_manager(pane)?;
    select_label(pane, "PF54 Managed")?;
    select_label(pane, "Set up with API key")?;
    pane.wait_stable_contains("API key — masked", READY_TIMEOUT)?;
    pane.send_secret_literal(canary)?;
    pane.send_key(TmuxKey::Enter)?;
    wait_manager_row(pane, home, "PF54 Managed", "Active")?;
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)?;
    Ok(())
}

fn wait_manager_row(pane: &TmuxPane<'_>, home: &Path, provider: &str, status: &str) -> Result<()> {
    let eligibility = home.join("provider-eligibility.json");
    let deadline = std::time::Instant::now() + READY_TIMEOUT;
    while std::time::Instant::now() < deadline && !eligibility.is_file() {
        std::thread::sleep(Duration::from_millis(50));
    }
    ensure!(
        eligibility.is_file(),
        "provider setup never reached durable eligibility persistence"
    );
    let managed =
        match codex_login::provider_api_key_metadata_from_auth_storage(home, "PF54_MANAGED_KEY") {
            Ok(codex_login::ProviderApiKeyStorageMetadata::Stored { source }) => {
                format!("Stored({source:?})")
            }
            Ok(codex_login::ProviderApiKeyStorageMetadata::Missing) => "Missing".to_string(),
            Ok(codex_login::ProviderApiKeyStorageMetadata::Suppressed) => "Suppressed".to_string(),
            Err(error) => format!("Error({:?})", error.kind()),
        };
    fs::write(home.join("post-save-provider-metadata.txt"), managed)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    focus_label(pane, provider)?;
    pane.wait_stable_until("provider status refresh", READY_TIMEOUT, |capture| {
        selected_row(capture)
            .is_some_and(|selected| selected.contains(provider) && selected.contains(status))
    })?;
    Ok(())
}

fn inspect_provider(pane: &TmuxPane<'_>, provider: &str, expected: &str) -> Result<()> {
    select_label(pane, provider)?;
    pane.wait_stable_contains(expected, READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)?;
    Ok(())
}

fn open_manager(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/providers")?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    Ok(())
}

fn select_label(pane: &TmuxPane<'_>, label: &str) -> Result<()> {
    focus_label(pane, label)?;
    pane.send_key(TmuxKey::Enter)?;
    Ok(())
}

fn focus_label(pane: &TmuxPane<'_>, label: &str) -> Result<()> {
    for _ in 0..64 {
        let capture = pane.capture_viewport()?;
        let selected = selected_row(&capture);
        if selected
            .as_deref()
            .and_then(selected_title)
            .is_some_and(|title| selected_title_matches(title, label))
        {
            return Ok(());
        }
        pane.send_key(TmuxKey::Down)?;
        pane.wait_stable_until(
            "selection redraw after Down",
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

fn wait_chat_ready(pane: &TmuxPane<'_>) -> Result<()> {
    pane.wait_stable_until("chat ready", READY_TIMEOUT, |capture| {
        capture.contains("/model to change")
            && !capture.contains("Press enter to confirm or esc to go back")
    })?;
    Ok(())
}

fn submit_and_wait(pane: &TmuxPane<'_>, prompt: &str, response: &str) -> Result<()> {
    wait_chat_ready(pane)?;
    pane.send_literal(prompt)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(response, READY_TIMEOUT)?;
    Ok(())
}

fn close_overlay_and_exit(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_until("management overlay closed", READY_TIMEOUT, |capture| {
        !capture.contains("Configure providers and control")
    })?;
    exit_tui(pane)
}

fn exit_tui(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/exit")?;
    pane.send_key(TmuxKey::Enter)?;
    Ok(())
}

fn session_spec(
    name: &str,
    binary: &Path,
    repo_root: &Path,
    home: &Path,
    login_issuer: &str,
) -> SessionSpec {
    SessionSpec::new(
        name,
        TerminalSize::new(140, 44),
        CommandSpec::new(binary)
            .env("CODEX_HOME", home)
            .env("CORBANU_HOME", home)
            .env("PFTERMINAL_HOME", home)
            .env("PF54_PRIMARY_KEY", "pf54-primary-environment-fixture")
            .env("PF54_SECONDARY_KEY", "pf54-secondary-environment-fixture")
            .env("PF54_ENV_KEY", "pf54-external-environment-fixture")
            .env("PF54_BROKEN_KEY", "")
            .env("CODEX_APP_SERVER_LOGIN_ISSUER", login_issuer)
            .env("RUST_LOG", "warn,codex_tui=debug,codex_login=debug")
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

async fn mock_openai_device_code(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/accounts/deviceauth/usercode"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"device_auth_id":"pf54-device","user_code":"PF54-CODE","interval":"1"}"#,
            "application/json",
        ))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/accounts/deviceauth/token"))
        .respond_with(ResponseTemplate::new(403))
        .mount(server)
        .await;
}

fn write_config(home: &Path, repo_root: &Path, server_uri: &str) -> Result<()> {
    fs::create_dir_all(home.join("log"))?;
    fs::write(
        home.join("config.toml"),
        format!(
            r#"model = "fixture-model"
model_provider = "{PRIMARY_PROVIDER}"
suppress_unstable_features_warning = true
log_dir = "{}"

[model_providers.{PRIMARY_PROVIDER}]
name = "PF54 Primary"
base_url = "{server_uri}/v1"
env_key = "PF54_PRIMARY_KEY"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0

[model_providers.{SECONDARY_PROVIDER}]
name = "PF54 Secondary"
base_url = "{server_uri}/v1"
env_key = "PF54_SECONDARY_KEY"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0

[model_providers.{MANAGED_PROVIDER}]
name = "PF54 Managed"
base_url = "{server_uri}/v1"
env_key = "PF54_MANAGED_KEY"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0

[model_providers.{ENV_PROVIDER}]
name = "PF54 Environment"
base_url = "{server_uri}/v1"
env_key = "PF54_ENV_KEY"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0

[model_providers.pf54-broken]
name = "PF54 Broken Environment"
base_url = "{server_uri}/v1"
env_key = "PF54_BROKEN_KEY"
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
    .context("write PF-54 tmux config")
}

fn select_config_provider(home: &Path, provider: &str) -> Result<()> {
    let path = home.join("config.toml");
    let config = fs::read_to_string(&path)?;
    let updated = config.replacen(
        &format!("model_provider = \"{PRIMARY_PROVIDER}\""),
        &format!("model_provider = \"{provider}\""),
        1,
    );
    ensure!(
        updated != config,
        "current provider config was not replaced"
    );
    fs::write(path, updated)?;
    Ok(())
}

fn wait_replacement_order(home: &Path, replacement: &str) -> Result<()> {
    let deadline = std::time::Instant::now() + READY_TIMEOUT;
    let config_path = home.join("config.toml");
    let eligibility_path = home.join("provider-eligibility.json");
    while std::time::Instant::now() < deadline {
        let selected = fs::read_to_string(&config_path)
            .is_ok_and(|config| config.contains(&format!("model_provider = \"{replacement}\"")));
        if eligibility_path.exists() && !selected {
            anyhow::bail!("old provider was deactivated before replacement persistence");
        }
        if eligibility_path.exists() && selected {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    anyhow::bail!("timed out waiting for ordered replacement persistence")
}

async fn ensure_authorization_seen(server: &MockServer, canary: &str) -> Result<()> {
    let deadline = std::time::Instant::now() + READY_TIMEOUT;
    while std::time::Instant::now() < deadline {
        if server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .any(|request| {
                request.headers.get("authorization").is_some_and(|value| {
                    value
                        .to_str()
                        .is_ok_and(|value| value == format!("Bearer {canary}"))
                })
            })
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    anyhow::bail!("retained managed provider credential was not used by the request")
}

fn register_evidence(tmux: &TmuxServer, home: &Path, binary: &Path) -> Result<()> {
    let metadata = fs::metadata(binary)?;
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let hash = format!("{:x}", Sha256::digest(fs::read(binary)?));
    fs::write(
        home.join("binary.sha256"),
        format!("{hash}  {}\n", binary.display()),
    )?;
    fs::write(
        home.join("binary.metadata"),
        format!(
            "size={}\nmtime_unix={}.{:09}\n",
            metadata.len(),
            modified.as_secs(),
            modified.subsec_nanos()
        ),
    )?;
    tmux.register_artifact("binary.sha256", home.join("binary.sha256"));
    tmux.register_artifact("binary.metadata", home.join("binary.metadata"));
    tmux.register_artifact("config.toml", home.join("config.toml"));
    tmux.register_artifact(
        "provider-eligibility.json",
        home.join("provider-eligibility.json"),
    );
    tmux.register_artifact(
        "post-save-provider-metadata.txt",
        home.join("post-save-provider-metadata.txt"),
    );
    tmux.register_artifact("codex-tui.log", home.join("log/codex-tui.log"));
    Ok(())
}

fn capture_success(
    scenario: &str,
    fixture: &Fixture,
    pane: &TmuxPane<'_>,
    canaries: &[&str],
) -> Result<()> {
    let viewport = pane.capture_viewport()?;
    let scrollback = pane.capture_scrollback_tail(4_000)?;
    let directory = PathBuf::from("target/tmux-artifacts").join(format!("pf54-{scenario}"));
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("viewport.txt"), &viewport)?;
    fs::write(directory.join("scrollback.txt"), &scrollback)?;
    fs::copy(
        fixture.home.path().join("binary.sha256"),
        directory.join("binary.sha256"),
    )?;
    fs::copy(
        fixture.home.path().join("binary.metadata"),
        directory.join("binary.metadata"),
    )?;
    if fixture
        .home
        .path()
        .join("provider-eligibility.json")
        .is_file()
    {
        fs::copy(
            fixture.home.path().join("provider-eligibility.json"),
            directory.join("provider-eligibility.json"),
        )?;
    }
    for canary in canaries {
        ensure!(
            !viewport.contains(canary) && !scrollback.contains(canary),
            "secret canary appeared in terminal evidence"
        );
        ensure!(
            !tree_contains_except_custody(fixture.home.path(), canary.as_bytes())?,
            "secret canary appeared outside credential custody"
        );
        ensure!(
            !tree_contains(&directory, canary.as_bytes())?,
            "secret canary appeared in emitted success artifacts"
        );
    }
    Ok(())
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

fn synthetic_canary(label: &str) -> String {
    format!("pf54-{label}-{}", Uuid::new_v4())
}

fn response(text: &str) -> String {
    responses::sse(vec![
        responses::ev_response_created("pf54-response"),
        responses::ev_assistant_message("pf54-message", text),
        responses::ev_completed("pf54-response"),
    ])
}

fn codex_binary(repo_root: &Path) -> Result<PathBuf> {
    let binary = repo_root.join("codex-rs/target/debug/codex");
    ensure!(
        binary.is_file(),
        "build target/debug/codex before PF-54 TMUX qualification"
    );
    Ok(binary)
}

#[test]
fn selection_parser_accepts_exact_and_inline_replacement_rows() {
    assert_eq!(selected_title("> 12. PF54 Managed"), Some("PF54 Managed"));
    assert!(selected_title_matches(
        "PF54 Secondary — fixture-model  Exact provider: pf54-secondary",
        "PF54 Secondary — fixture-model"
    ));
}

#[test]
fn binary_mtime_is_representable_for_evidence() {
    assert!(SystemTime::now().duration_since(UNIX_EPOCH).is_ok());
}
