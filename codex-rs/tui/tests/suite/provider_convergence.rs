use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use anyhow::ensure;
use core_test_support::responses;
use sha2::Digest;
use sha2::Sha256;
use tempfile::tempdir;
use uuid::Uuid;
use wiremock::MockServer;

use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxKey;
use crate::support::tmux::TmuxPane;
use crate::support::tmux::TmuxServer;

const READY_TIMEOUT: Duration = Duration::from_secs(45);
const MODEL: &str = "fixture-model";
const A: &str = "pf55-a";
const B: &str = "pf55-b";
const MANAGED: &str = "pf55-managed";
const ENV: &str = "pf55-env";
const COMMAND: &str = "pf55-command";
const DUP_A: &str = "pf55-duplicate-a";
const DUP_B: &str = "pf55-duplicate-b";

#[derive(Clone, Copy, Debug)]
enum Case {
    Upgrade,
    FreshFirstSuccess,
    Environment,
    ManagedRestart,
    InactiveNoncurrent,
    InactiveCurrentCancel,
    ExactReplacement,
    MissingCurrent,
    Resume,
    NativeSpawn,
    CommandAuth,
    DuplicateSlug,
}

macro_rules! tmux_cases {
    ($($name:ident => ($description:literal, $case:expr)),+ $(,)?) => {
        $(
            #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
            async fn $name() -> Result<()> {
                if !TmuxServer::should_run($description)? {
                    return Ok(());
                }
                run_case($case).await
            }
        )+
    };
}

tmux_cases! {
    tmux_existing_install_upgrade_preserves_current_and_request => ("PF-55 existing install upgrade", Case::Upgrade),
    tmux_fresh_a_then_b_preserves_first_success_across_restart => ("PF-55 first-success default", Case::FreshFirstSuccess),
    tmux_custom_environment_provider_converges_across_hosts_picker_and_request => ("PF-55 custom environment provider", Case::Environment),
    tmux_managed_custom_key_survives_restart_and_request => ("PF-55 managed custom provider", Case::ManagedRestart),
    tmux_inactive_noncurrent_is_managed_but_not_pickable_then_reactivates => ("PF-55 inactive noncurrent provider", Case::InactiveNoncurrent),
    tmux_inactive_current_cancel_blocks_request_without_switch => ("PF-55 inactive current cancellation", Case::InactiveCurrentCancel),
    tmux_exact_replacement_persists_restart_and_request => ("PF-55 exact provider replacement", Case::ExactReplacement),
    tmux_missing_profile_current_never_silently_switches => ("PF-55 missing profile current", Case::MissingCurrent),
    tmux_resumed_main_session_retains_exact_runtime_identity => ("PF-55 resumed session identity", Case::Resume),
    tmux_native_spawn_picker_and_parent_request_share_exact_custom_runtime => ("PF-55 native spawn provider", Case::NativeSpawn),
    tmux_command_auth_is_visible_validated_and_has_no_enrollment_ui => ("PF-55 command auth provider", Case::CommandAuth),
    tmux_duplicate_model_slug_uses_exact_provider_identity => ("PF-55 duplicate model slug identity", Case::DuplicateSlug),
}

async fn run_case(case: Case) -> Result<()> {
    let fixture = Fixture::new(case).await?;
    let tmux = fixture.tmux()?;
    match case {
        Case::FreshFirstSuccess => run_fresh(&fixture, &tmux).await,
        Case::ManagedRestart => run_managed_restart(&fixture, &tmux).await,
        Case::Resume => run_resume(&fixture, &tmux).await,
        Case::ExactReplacement => run_exact_replacement(&fixture, &tmux).await,
        _ => {
            let session = tmux.new_session(fixture.session("pf55-case", false))?;
            let pane = session.primary_pane();
            wait_chat_ready(pane)?;
            run_open_case(case, &fixture, pane).await?;
            capture(&fixture, pane)?;
            exit_tui(pane)?;
            session.wait_for_exit(READY_TIMEOUT)?;
            Ok(())
        }
    }
}

async fn run_open_case(case: Case, fixture: &Fixture, pane: &TmuxPane<'_>) -> Result<()> {
    match case {
        Case::Upgrade => {
            submit_and_wait(pane, "upgrade request", "PF55 response")?;
            require_authorization(&fixture.server, &fixture.a_key).await?;
            ensure!(current_provider(fixture.home.path())? == A);
        }
        Case::Environment => {
            inspect_provider(pane, "PF55 Environment", "Active · current")?;
            open_model_picker(pane)?;
            pane.wait_stable_contains(MODEL, READY_TIMEOUT)?;
            pane.send_key(TmuxKey::Escape)?;
            submit_and_wait(pane, "environment request", "PF55 response")?;
            require_authorization(&fixture.server, &fixture.env_key).await?;
        }
        Case::InactiveNoncurrent => {
            open_manager(pane)?;
            select_label(pane, "PF55 B")?;
            select_label(pane, "Deactivate")?;
            pane.wait_stable_contains("Inactive", READY_TIMEOUT)?;
            pane.send_key(TmuxKey::Escape)?;
            wait_chat_ready(pane)?;
            open_model_picker(pane)?;
            ensure!(
                !pane.capture_viewport()?.contains("PF55 B"),
                "inactive provider remained in the model picker"
            );
            pane.send_key(TmuxKey::Escape)?;
            open_manager(pane)?;
            select_label(pane, "PF55 B")?;
            select_label(pane, "Reactivate")?;
            pane.wait_stable_contains("Active", READY_TIMEOUT)?;
            pane.send_key(TmuxKey::Escape)?;
            wait_chat_ready(pane)?;
        }
        Case::InactiveCurrentCancel => {
            open_manager(pane)?;
            inspect_provider_detail(pane, "PF55 A", "Inactive")?;
            pane.send_key(TmuxKey::Escape)?;
            wait_chat_ready(pane)?;
            pane.send_literal("must remain blocked")?;
            pane.send_key(TmuxKey::Enter)?;
            pane.wait_stable_contains(
                "current provider is unavailable or inactive",
                READY_TIMEOUT,
            )?;
            ensure!(current_provider(fixture.home.path())? == A);
            ensure!(response_request_count(&fixture.server).await == 0);
        }
        Case::MissingCurrent => {
            pane.send_literal("missing current must block")?;
            pane.send_key(TmuxKey::Enter)?;
            pane.wait_stable_contains(
                "current provider is unavailable or inactive",
                READY_TIMEOUT,
            )?;
            ensure!(current_provider(fixture.home.path())? == "pf55-missing");
            ensure!(response_request_count(&fixture.server).await == 0);
        }
        Case::NativeSpawn => {
            submit_and_wait(pane, "parent exact runtime", "PF55 response")?;
            let parent_requests = authorization_count(&fixture.server, &fixture.a_key).await;
            pane.send_literal("/spawn")?;
            pane.send_key(TmuxKey::Enter)?;
            pane.wait_stable_contains("Nazgul", READY_TIMEOUT)?;
            select_label(pane, "Nazgul")?;
            select_label(pane, "Create Nazgul pane")?;
            pane.wait_stable_contains(MODEL, READY_TIMEOUT)?;
            select_label(pane, MODEL)?;
            pane.wait_stable_contains("Spawned Corbanu Terminal Nazgul pane", READY_TIMEOUT)?;
            pane.send_literal("/agent")?;
            pane.send_key(TmuxKey::Enter)?;
            pane.wait_stable_contains("Subagents", READY_TIMEOUT)?;
            pane.send_key(TmuxKey::Down)?;
            pane.send_key(TmuxKey::Enter)?;
            wait_chat_ready(pane)?;
            submit_and_wait(pane, "child exact runtime", "PF55 response")?;
            ensure!(
                authorization_count(&fixture.server, &fixture.a_key).await > parent_requests,
                "child request did not use the parent's exact provider identity"
            );
        }
        Case::CommandAuth => {
            open_manager(pane)?;
            select_label(pane, "PF55 Command")?;
            pane.wait_stable_contains("Externally managed credential", READY_TIMEOUT)?;
            ensure!(
                !pane.capture_viewport()?.contains("Set up"),
                "command-auth provider invented enrollment UI"
            );
            pane.send_key(TmuxKey::Escape)?;
            pane.send_key(TmuxKey::Escape)?;
            wait_chat_ready(pane)?;
            submit_and_wait(pane, "command auth request", "PF55 response")?;
            require_authorization(&fixture.server, &fixture.command_key).await?;
        }
        Case::DuplicateSlug => {
            submit_and_wait(pane, "duplicate slug exact request", "PF55 response")?;
            require_authorization(&fixture.server, &fixture.dup_b_key).await?;
            ensure!(!authorization_seen(&fixture.server, &fixture.dup_a_key).await);
        }
        Case::FreshFirstSuccess | Case::ManagedRestart | Case::Resume | Case::ExactReplacement => {
            unreachable!()
        }
    }
    Ok(())
}

async fn run_fresh(fixture: &Fixture, tmux: &TmuxServer) -> Result<()> {
    let first = tmux.new_session(fixture.session("pf55-fresh", false))?;
    let pane = first.primary_pane();
    pane.wait_stable_contains("Choose a provider account", READY_TIMEOUT)?;
    configure_onboarding_key(pane, "Provider: PF55 A PF55_A_API_KEY", &fixture.a_key)?;
    configure_onboarding_key(pane, "Provider: PF55 B PF55_B_API_KEY", &fixture.b_key)?;
    select_label(pane, "Done")?;
    wait_chat_ready(pane)?;
    ensure!(current_provider(fixture.home.path())? == A);
    exit_tui(pane)?;
    first.wait_for_exit(READY_TIMEOUT)?;

    let second = tmux.new_session(fixture.session("pf55-fresh-restart", false))?;
    let pane = second.primary_pane();
    wait_chat_ready(pane)?;
    submit_and_wait(pane, "first success request", "PF55 response")?;
    require_authorization(&fixture.server, &fixture.a_key).await?;
    capture(fixture, pane)?;
    exit_tui(pane)?;
    second.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

async fn run_managed_restart(fixture: &Fixture, tmux: &TmuxServer) -> Result<()> {
    for (index, name) in ["pf55-managed-first", "pf55-managed-restart"]
        .into_iter()
        .enumerate()
    {
        let session = tmux.new_session(fixture.session(name, false))?;
        let pane = session.primary_pane();
        wait_chat_ready(pane)?;
        inspect_provider(pane, "PF55 Managed", "Active · current")?;
        if index == 1 {
            submit_and_wait(pane, "managed restart request", "PF55 response")?;
            require_authorization(&fixture.server, &fixture.managed_key).await?;
            capture(fixture, pane)?;
        }
        exit_tui(pane)?;
        session.wait_for_exit(READY_TIMEOUT)?;
    }
    Ok(())
}

async fn run_resume(fixture: &Fixture, tmux: &TmuxServer) -> Result<()> {
    let first = tmux.new_session(fixture.session("pf55-resume-source", false))?;
    let pane = first.primary_pane();
    wait_chat_ready(pane)?;
    submit_and_wait(pane, "resume source request", "PF55 response")?;
    exit_tui(pane)?;
    first.wait_for_exit(READY_TIMEOUT)?;

    let resumed = tmux.new_session(fixture.session("pf55-resumed", true))?;
    let pane = resumed.primary_pane();
    wait_chat_ready(pane)?;
    submit_and_wait(pane, "resumed exact request", "PF55 response")?;
    require_authorization(&fixture.server, &fixture.a_key).await?;
    ensure!(current_provider(fixture.home.path())? == A);
    capture(fixture, pane)?;
    exit_tui(pane)?;
    resumed.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

async fn run_exact_replacement(fixture: &Fixture, tmux: &TmuxServer) -> Result<()> {
    let first = tmux.new_session(fixture.session("pf55-replace", false))?;
    let pane = first.primary_pane();
    wait_chat_ready(pane)?;
    open_manager(pane)?;
    select_label(pane, "PF55 A")?;
    select_label(pane, "Deactivate")?;
    pane.wait_stable_contains("Choose replacement", READY_TIMEOUT)?;
    select_label(pane, "PF55 B — fixture-model")?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    ensure!(current_provider(fixture.home.path())? == B);
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)?;
    exit_tui(pane)?;
    first.wait_for_exit(READY_TIMEOUT)?;

    let second = tmux.new_session(fixture.session("pf55-replace-restart", false))?;
    let pane = second.primary_pane();
    wait_chat_ready(pane)?;
    submit_and_wait(pane, "replacement request", "PF55 response")?;
    require_authorization(&fixture.server, &fixture.b_key).await?;
    capture(fixture, pane)?;
    exit_tui(pane)?;
    second.wait_for_exit(READY_TIMEOUT)?;
    Ok(())
}

struct Fixture {
    repo_root: PathBuf,
    binary: PathBuf,
    home: tempfile::TempDir,
    server: MockServer,
    scenario: String,
    case: Case,
    a_key: String,
    b_key: String,
    managed_key: String,
    env_key: String,
    command_key: String,
    dup_a_key: String,
    dup_b_key: String,
}

impl Fixture {
    async fn new(case: Case) -> Result<Self> {
        let repo_root = codex_utils_cargo_bin::repo_root()?;
        let binary = repo_root.join("codex-rs/target/debug/codex");
        ensure!(
            binary.is_file(),
            "build target/debug/codex before PF-55 TMUX"
        );
        let home = tempdir()?;
        let server = MockServer::start().await;
        responses::mount_sse_repeating(&server, response("PF55 response")).await;
        let keys = ["a", "b", "managed", "env", "cmd", "dup-a", "dup-b"].map(canary);
        write_command(home.path(), &keys[4])?;
        write_config(home.path(), &repo_root, &server.uri(), case)?;
        if matches!(case, Case::ManagedRestart) {
            fs::write(
                home.path().join("provider_auth.json"),
                format!(r#"{{"api_keys":{{"PF55_MANAGED_API_KEY":"{}"}}}}"#, keys[2]),
            )?;
        }
        if matches!(case, Case::InactiveCurrentCancel) {
            fs::write(
                home.path().join("provider-eligibility.json"),
                "{\n  \"version\": 1,\n  \"inactive_identities\": [\n    \"credential-env:PF55_A_API_KEY\"\n  ]\n}\n",
            )?;
        }
        Ok(Self {
            repo_root,
            binary,
            home,
            server,
            scenario: format!("{case:?}").to_ascii_lowercase(),
            case,
            a_key: keys[0].clone(),
            b_key: keys[1].clone(),
            managed_key: keys[2].clone(),
            env_key: keys[3].clone(),
            command_key: keys[4].clone(),
            dup_a_key: keys[5].clone(),
            dup_b_key: keys[6].clone(),
        })
    }

    fn tmux(&self) -> Result<TmuxServer> {
        let tmux = TmuxServer::start(&format!("pf55_{}", self.scenario))?;
        register_evidence(&tmux, self.home.path(), &self.binary)?;
        Ok(tmux)
    }

    fn session(&self, name: &str, resume: bool) -> SessionSpec {
        let mut command = CommandSpec::new(&self.binary)
            .env("CODEX_HOME", self.home.path())
            .env("CORBANU_HOME", self.home.path())
            .env("PFTERMINAL_HOME", self.home.path())
            .env("RUST_LOG", "warn,codex_tui=debug,codex_core=debug")
            .arg("-c")
            .arg("analytics.enabled=false")
            .arg("-c")
            .arg("tui.animations=false")
            .arg("--no-alt-screen")
            .arg("-C")
            .arg(&self.repo_root);
        if !matches!(self.case, Case::FreshFirstSuccess) {
            command = command
                .env("PF55_A_API_KEY", &self.a_key)
                .env("PF55_B_API_KEY", &self.b_key)
                .env("PF55_ENV_API_KEY", &self.env_key)
                .env("PF55_DUP_A_API_KEY", &self.dup_a_key)
                .env("PF55_DUP_B_API_KEY", &self.dup_b_key)
                .env("PF55_UNUSED_API_KEY", "pf55-adjacent-usable");
        }
        if resume {
            command = command.arg("resume").arg("--last");
        }
        SessionSpec::new(name, TerminalSize::new(140, 44), command).current_dir(&self.repo_root)
    }
}

fn write_config(home: &Path, repo: &Path, server: &str, case: Case) -> Result<()> {
    fs::create_dir_all(home.join("log"))?;
    let current = match case {
        Case::Environment => ENV,
        Case::ManagedRestart => MANAGED,
        Case::MissingCurrent => "pf55-missing",
        Case::CommandAuth => COMMAND,
        Case::DuplicateSlug => DUP_B,
        _ => A,
    };
    let mut config = format!(
        "model = \"{MODEL}\"\nmodel_provider = \"{current}\"\nsuppress_unstable_features_warning = true\nlog_dir = \"{}\"\n",
        home.join("log").display()
    );
    for (id, name, env_key) in [
        (A, "PF55 A", "PF55_A_API_KEY"),
        (B, "PF55 B", "PF55_B_API_KEY"),
        (MANAGED, "PF55 Managed", "PF55_MANAGED_API_KEY"),
        (ENV, "PF55 Environment", "PF55_ENV_API_KEY"),
        ("pf55-missing", "PF55 Missing", "PF55_MISSING_API_KEY"),
        (DUP_A, "PF55 Duplicate A", "PF55_DUP_A_API_KEY"),
        (DUP_B, "PF55 Duplicate B", "PF55_DUP_B_API_KEY"),
    ] {
        config.push_str(&format!(
            "\n[model_providers.{id}]\nname = \"{name}\"\nbase_url = \"{server}/v1\"\nenv_key = \"{env_key}\"\nwire_api = \"responses\"\nrequest_max_retries = 0\nstream_max_retries = 0\n"
        ));
    }
    config.push_str(&format!(
        "\n[model_providers.{COMMAND}]\nname = \"PF55 Command\"\nbase_url = \"{server}/v1\"\nwire_api = \"responses\"\n\n[model_providers.{COMMAND}.auth]\ncommand = \"{}\"\nrefresh_interval_ms = 600000\n\n[projects.\"{}\"]\ntrust_level = \"trusted\"\n",
        home.join("pf55-auth-command.sh").display(),
        repo.display()
    ));
    fs::write(home.join("config.toml"), config)?;
    Ok(())
}

fn write_command(home: &Path, token: &str) -> Result<()> {
    let path = home.join("pf55-auth-command.sh");
    fs::write(
        &path,
        format!("#!/bin/sh\n[ \"$PF55_A_API_KEY\" ] && printf '{token}'\n"),
    )?;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn configure_onboarding_key(pane: &TmuxPane<'_>, label: &str, secret: &str) -> Result<()> {
    select_label(pane, label)?;
    pane.wait_stable_contains("never adds it to chat.", READY_TIMEOUT)?;
    pane.send_secret_literal(secret)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("API key configured", READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Choose a provider account", READY_TIMEOUT)?;
    Ok(())
}

fn inspect_provider(pane: &TmuxPane<'_>, label: &str, status: &str) -> Result<()> {
    open_manager(pane)?;
    inspect_provider_detail(pane, label, status)?;
    pane.send_key(TmuxKey::Escape)?;
    wait_chat_ready(pane)
}

fn inspect_provider_detail(pane: &TmuxPane<'_>, label: &str, status: &str) -> Result<()> {
    select_label(pane, label)?;
    pane.wait_stable_contains(status, READY_TIMEOUT)?;
    pane.send_key(TmuxKey::Escape)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    Ok(())
}

fn open_manager(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/providers")?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Configure providers and control", READY_TIMEOUT)?;
    Ok(())
}

fn open_model_picker(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/model")?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("Select Model", READY_TIMEOUT)?;
    Ok(())
}

fn select_label(pane: &TmuxPane<'_>, label: &str) -> Result<()> {
    for _ in 0..80 {
        let before = pane.capture_viewport()?;
        if selected_title(&before).is_some_and(|title| {
            title == label
                || title.strip_prefix(label).is_some_and(|suffix| {
                    suffix.starts_with("  ") || suffix.starts_with(" (current)")
                })
        }) {
            pane.send_key(TmuxKey::Enter)?;
            return Ok(());
        }
        pane.send_key(TmuxKey::Down)?;
        pane.wait_stable_until("selection redraw", Duration::from_secs(5), |next| {
            selected_title(next) != selected_title(&before)
        })?;
    }
    anyhow::bail!("could not select {label:?}:\n{}", pane.capture_viewport()?)
}

fn selected_title(capture: &str) -> Option<String> {
    let line = capture.lines().rev().find(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("> ") || trimmed.starts_with("› ")
    })?;
    let selected = line.trim().trim_start_matches(&['>', '›'][..]).trim();
    let title = selected
        .split_once(". ")
        .filter(|(prefix, _)| prefix.chars().all(|ch| ch.is_ascii_digit()))
        .map_or(selected, |(_, title)| title);
    Some(title.trim().to_string())
}

fn wait_chat_ready(pane: &TmuxPane<'_>) -> Result<()> {
    pane.wait_stable_until("chat ready", READY_TIMEOUT, |capture| {
        capture.contains("/model to change")
            && !capture.contains("Press enter to confirm or esc to go back")
    })?;
    Ok(())
}

fn submit_and_wait(pane: &TmuxPane<'_>, prompt: &str, expected: &str) -> Result<()> {
    wait_chat_ready(pane)?;
    pane.send_literal(prompt)?;
    pane.wait_stable_contains(prompt, READY_TIMEOUT)?;
    std::thread::sleep(Duration::from_millis(25));
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains(expected, READY_TIMEOUT)?;
    Ok(())
}

fn exit_tui(pane: &TmuxPane<'_>) -> Result<()> {
    pane.send_literal("/exit")?;
    pane.send_key(TmuxKey::Enter)?;
    Ok(())
}

fn current_provider(home: &Path) -> Result<String> {
    let config = fs::read_to_string(home.join("config.toml"))?;
    config
        .lines()
        .find_map(|line| line.strip_prefix("model_provider = \""))
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("current model_provider missing"))
}

async fn require_authorization(server: &MockServer, token: &str) -> Result<()> {
    ensure!(
        authorization_seen(server, token).await,
        "expected exact bearer authorization was not observed"
    );
    Ok(())
}

async fn authorization_seen(server: &MockServer, token: &str) -> bool {
    authorization_count(server, token).await > 0
}

async fn authorization_count(server: &MockServer, token: &str) -> usize {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .filter(|request| {
            request.headers.get("authorization").is_some_and(|header| {
                header
                    .to_str()
                    .is_ok_and(|value| value == format!("Bearer {token}"))
            })
        })
        .count()
}

async fn response_request_count(server: &MockServer) -> usize {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .filter(|request| request.url.path() == "/v1/responses")
        .count()
}

fn register_evidence(tmux: &TmuxServer, home: &Path, binary: &Path) -> Result<()> {
    let metadata = fs::metadata(binary)?;
    let modified = metadata.modified()?.duration_since(UNIX_EPOCH)?;
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
    tmux.register_artifact("codex-tui.log", home.join("log/codex-tui.log"));
    Ok(())
}

fn capture(fixture: &Fixture, pane: &TmuxPane<'_>) -> Result<()> {
    let viewport = pane.capture_viewport()?;
    let scrollback = pane.capture_scrollback_tail(4_000)?;
    let directory =
        PathBuf::from("target/tmux-artifacts").join(format!("pf55-{}", fixture.scenario));
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
    for secret in fixture.secrets() {
        ensure!(
            !viewport.contains(secret) && !scrollback.contains(secret),
            "secret canary appeared in terminal evidence"
        );
        ensure!(
            !tree_contains_except_custody(fixture.home.path(), secret.as_bytes())?,
            "secret canary appeared outside credential custody"
        );
        ensure!(
            !tree_contains(&directory, secret.as_bytes())?,
            "secret canary appeared in success artifacts"
        );
    }
    Ok(())
}

impl Fixture {
    fn secrets(&self) -> [&str; 7] {
        [
            &self.a_key,
            &self.b_key,
            &self.managed_key,
            &self.env_key,
            &self.command_key,
            &self.dup_a_key,
            &self.dup_b_key,
        ]
    }
}

fn tree_contains_except_custody(root: &Path, needle: &[u8]) -> Result<bool> {
    if root.file_name().and_then(|name| name.to_str()) == Some("provider_auth.json")
        || root.file_name().and_then(|name| name.to_str()) == Some("pf55-auth-command.sh")
    {
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
    if metadata.file_type().is_symlink() {
        return Ok(false);
    }
    if metadata.is_file() {
        let bytes = fs::read(root)?;
        return Ok(bytes.windows(needle.len()).any(|window| window == needle));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(root)? {
            if tree_contains_except_custody(&entry?.path(), needle)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn canary(label: &str) -> String {
    format!("pf55-{label}-{}", Uuid::new_v4())
}

fn response(text: &str) -> String {
    responses::sse(vec![
        responses::ev_response_created("pf55-response"),
        responses::ev_assistant_message("pf55-message", text),
        responses::ev_completed("pf55-response"),
    ])
}
