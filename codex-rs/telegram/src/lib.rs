pub mod approvals;
pub mod auth;
mod bot;
mod bridge;
pub mod commands;
pub mod config;
pub mod conversation;
pub mod dedup;
pub mod error;
pub mod media;
mod model_selection;
pub mod outbound;
mod persistence;
mod polling;
pub mod render;
mod sandbox_preflight;
mod session;

#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;

#[cfg(test)]
#[path = "dedup_tests.rs"]
mod dedup_tests;

#[cfg(not(windows))]
use std::path::PathBuf;
#[cfg(not(windows))]
use std::process::Command;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use codex_app_server_client::DEFAULT_IN_PROCESS_CHANNEL_CAPACITY;
use codex_app_server_client::EnvironmentManager;
use codex_app_server_client::ExecServerRuntimePaths;
use codex_app_server_client::InProcessAppServerClient;
use codex_app_server_client::InProcessClientStartArgs;
use codex_arg0::Arg0DispatchPaths;
use codex_config::CloudConfigBundleLoader;
use codex_config::LoaderOverrides;
use codex_feedback::CodexFeedback;
use codex_login::AuthManager;
use codex_model_provider_info::canonical_catalog_provider;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use teloxide::Bot;
use teloxide::prelude::Requester;
use tracing::instrument;
use tracing::warn;

use crate::bot::run_bot;
use crate::bridge::BridgeHandle;
use crate::config::TelegramConfig;
use crate::config::TelegramMode;
use crate::error::TelegramError;
use crate::session::SessionStore;

/// Client-level ceiling for any single outbound Bot API HTTP request. The
/// poller runs on a separate client built for long-poll reads (see
/// `polling::polling_bot`), so this bound applies only to action-style calls.
/// It must sit below the inner retry-attempt timeout in `outbound.rs` so a
/// hung attempt aborts with a clean `Elapsed` instead of a mid-read socket
/// error, and a stalled call can never wedge a chat pipeline forever.
const OUTBOUND_CLIENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug, Parser, Clone)]
pub struct Cli {
    /// Error out when config.toml contains fields core does not recognize.
    #[arg(long = "strict-config", default_value_t = false)]
    pub strict_config: bool,

    /// Verify Telegram identity, authorization, state, workspace, and provider readiness, then exit.
    #[arg(long, default_value_t = false)]
    pub health: bool,

    /// Configure Telegram interactively without requiring a source checkout.
    #[arg(long, default_value_t = false, conflicts_with = "health")]
    pub setup: bool,
}

pub struct RunConfig {
    pub cli: Cli,
    pub arg0_paths: Arg0DispatchPaths,
    pub cli_overrides: Vec<(String, toml::Value)>,
    pub loader_overrides: LoaderOverrides,
}

#[instrument(skip(run_config))]
pub async fn run(run_config: RunConfig) -> anyhow::Result<()> {
    let RunConfig {
        cli,
        arg0_paths,
        cli_overrides,
        loader_overrides,
    } = run_config;
    if cli.setup {
        return run_setup();
    }
    let codex_home = codex_core::config::find_codex_home()?;
    let telegram_config = TelegramConfig::load_from_codex_home(&codex_home)?;
    if !telegram_config.enabled {
        return Err(TelegramError::Disabled.into());
    }
    match telegram_config.mode {
        TelegramMode::Polling => {}
        TelegramMode::Webhook => return Err(TelegramError::WebhookUnsupported.into()),
    }

    let token = telegram_config.resolve_token(&codex_home)?;
    let allowlist = telegram_config.allowlist();
    if allowlist.is_empty() {
        warn!("Telegram connector started with an empty allowlist; all chats will be rejected");
    }
    for chat_id in telegram_config
        .allowed_chat_ids
        .iter()
        .copied()
        .filter(|chat_id| *chat_id < 0)
    {
        warn!(
            conversation = %crate::conversation::ConversationKey::from(teloxide::types::ChatId(chat_id)).redacted_id(),
            "Telegram allowlist includes a group or supergroup chat; only allowed_user_ids may drive and approve turns"
        );
    }

    let core_config = build_core_config(
        &telegram_config,
        &arg0_paths,
        cli_overrides.clone(),
        loader_overrides.clone(),
        cli.strict_config,
    )
    .await?;
    if cli.health {
        return run_health_check(&token, &telegram_config, &core_config).await;
    }
    let sandbox_policy = core_config.legacy_sandbox_policy();
    if matches!(
        core_config.permissions.approval_policy.value(),
        AskForApproval::Never
    ) {
        warn!(
            "Telegram connector approval_policy resolves to never; remote turns can run without interactive approval prompts"
        );
    }
    if matches!(sandbox_policy, SandboxPolicy::DangerFullAccess) {
        warn!(
            "Telegram connector sandbox resolves to danger-full-access; filesystem sandboxing is disabled for remote turns"
        );
    }
    sandbox_preflight::warn_if_sandbox_may_fail(&sandbox_policy);
    let state_db = codex_core::init_state_db(&core_config).await;
    let runtime_paths = ExecServerRuntimePaths::from_optional_paths(
        arg0_paths.codex_self_exe.clone(),
        arg0_paths.codex_linux_sandbox_exe.clone(),
    )?;
    let environment_manager = EnvironmentManager::from_codex_home(
        core_config.codex_home.clone(),
        Some(runtime_paths),
        core_config.http_client_factory(),
    )
    .await?;
    let start_args = InProcessClientStartArgs {
        arg0_paths,
        config: Arc::new(core_config.clone()),
        cli_overrides,
        loader_overrides,
        strict_config: cli.strict_config,
        cloud_config_bundle: CloudConfigBundleLoader::default(),
        feedback: CodexFeedback::new(),
        log_db: None,
        state_db,
        environment_manager: Arc::new(environment_manager),
        config_warnings: Vec::new(),
        session_source: SessionSource::Custom("telegram".to_string()),
        enable_codex_api_key_env: true,
        client_name: "codex_telegram".to_string(),
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        experimental_api: true,
        mcp_server_openai_form_elicitation: false,
        opt_out_notification_methods: Vec::new(),
        channel_capacity: DEFAULT_IN_PROCESS_CHANNEL_CAPACITY,
    };
    let client = InProcessAppServerClient::start(start_args)
        .await
        .context("failed to initialize in-process app-server client")?;
    let auth_manager =
        AuthManager::shared_from_config(&core_config, /*enable_codex_api_key_env*/ true).await;
    let sessions = SessionStore::load(&codex_home).await?;
    let outbound_client = teloxide::net::default_reqwest_settings()
        .timeout(OUTBOUND_CLIENT_TIMEOUT)
        .build()
        .context("failed to build Telegram outbound HTTP client")?;
    let bot = Bot::with_client(token.clone(), outbound_client);
    let polling = crate::polling::polling_bot(token)?;
    let media = crate::media::MediaStore::with_limits(
        &codex_home,
        telegram_config.max_attachment_bytes,
        telegram_config.media_retention_days,
        telegram_config.max_media_store_bytes,
    );
    media.cleanup_expired().await;
    let bridge = BridgeHandle::spawn(
        bot.clone(),
        client,
        Arc::new(core_config),
        sessions,
        auth_manager,
    );
    let result = run_bot(
        bot,
        polling,
        bridge.clone(),
        allowlist,
        media,
        codex_home.to_path_buf(),
        telegram_config.max_consecutive_polling_failures,
    )
    .await;
    if let Err(err) = bridge.shutdown().await {
        warn!("Telegram bridge shutdown failed: {err}");
    }
    result
}

fn run_setup() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        anyhow::bail!(
            "interactive Telegram setup is not yet available on Windows; configure [telegram], run `corbanu telegram --health`, then use the bundled install-telegram-task.ps1"
        );
    }

    #[cfg(not(windows))]
    {
        let script = locate_setup_script()?;
        let status = Command::new("bash")
            .arg(&script)
            .status()
            .with_context(|| format!("failed to launch {}", script.display()))?;
        if !status.success() {
            anyhow::bail!("Telegram setup exited with {status}");
        }
        Ok(())
    }
}

#[cfg(not(windows))]
fn locate_setup_script() -> anyhow::Result<PathBuf> {
    let executable =
        std::env::current_exe().context("resolve current Corbanu Terminal executable")?;
    if let Some(package_root) = executable.parent().and_then(|bin_dir| bin_dir.parent()) {
        let packaged = package_root
            .join("codex-resources")
            .join("telegram")
            .join("setup-telegram.sh");
        if packaged.is_file() {
            return Ok(packaged);
        }
    }

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("Telegram crate has no codex-rs parent")?
        .join("scripts")
        .join("setup-telegram.sh");
    if source.is_file() {
        return Ok(source);
    }
    anyhow::bail!(
        "Telegram setup resources are missing; reinstall Corbanu Terminal or configure [telegram] manually"
    )
}

async fn run_health_check(
    token: &str,
    telegram_config: &TelegramConfig,
    core_config: &codex_core::config::Config,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !telegram_config.allowed_chat_ids.is_empty(),
        "Telegram health failed: allowed_chat_ids is empty"
    );
    if telegram_config
        .allowed_chat_ids
        .iter()
        .any(|chat_id| *chat_id < 0)
    {
        anyhow::ensure!(
            !telegram_config.allowed_user_ids.is_empty(),
            "Telegram health failed: group chats require allowed_user_ids"
        );
    }

    let telegram_dir = core_config.codex_home.join("telegram");
    tokio::fs::create_dir_all(&telegram_dir).await?;
    let state_probe = telegram_dir.join(".health-write-probe");
    tokio::fs::write(&state_probe, b"ok")
        .await
        .context("Telegram health failed: state directory is not writable")?;
    tokio::fs::remove_file(&state_probe).await?;

    let workspace_probe = core_config.cwd.join(".corbanu-telegram-health-probe");
    tokio::fs::write(&workspace_probe, b"ok")
        .await
        .context("Telegram health failed: workspace is not writable")?;
    tokio::fs::remove_file(&workspace_probe).await?;

    if let Some(missing) = crate::model_selection::missing_provider_credential(
        &core_config.model_provider_id,
        &core_config.model_providers,
        core_config,
    ) {
        anyhow::bail!(
            "Telegram health failed: provider `{}` is missing `{}`",
            missing.provider,
            missing.env_key
        );
    }

    sandbox_preflight::ensure_sandbox_viable(&core_config.legacy_sandbox_policy())
        .context("Telegram health failed: sandbox preflight")?;

    let client = teloxide::net::default_reqwest_settings()
        .timeout(OUTBOUND_CLIENT_TIMEOUT)
        .build()
        .context("Telegram health failed: HTTP client initialization")?;
    let me = Bot::with_client(token.to_string(), client)
        .get_me()
        .await
        .context("Telegram health failed: Bot API identity check")?;
    println!(
        "healthy: bot=@{} id={} chats={} users={} workspace={}",
        me.username(),
        me.id.0,
        telegram_config.allowed_chat_ids.len(),
        telegram_config.allowed_user_ids.len(),
        core_config.cwd.display()
    );
    Ok(())
}

async fn build_core_config(
    telegram_config: &TelegramConfig,
    arg0_paths: &Arg0DispatchPaths,
    cli_overrides: Vec<(String, toml::Value)>,
    loader_overrides: LoaderOverrides,
    strict_config: bool,
) -> anyhow::Result<codex_core::config::Config> {
    let overrides = codex_core::config::ConfigOverrides {
        model: telegram_config.default_model.clone(),
        // `[telegram].default_model` is a scoped model selection, so pair it
        // with its canonical provider instead of inheriting an unrelated
        // top-level provider. Unknown/custom models continue to inherit.
        model_provider: telegram_default_model_provider(telegram_config),
        cwd: telegram_config.default_cwd.clone(),
        approval_policy: telegram_config.approval_policy,
        sandbox_mode: telegram_config.sandbox_mode,
        codex_self_exe: arg0_paths.codex_self_exe.clone(),
        codex_linux_sandbox_exe: arg0_paths.codex_linux_sandbox_exe.clone(),
        main_execve_wrapper_exe: arg0_paths.main_execve_wrapper_exe.clone(),
        ..Default::default()
    };
    let mut core_config = codex_core::config::ConfigBuilder::default()
        .cli_overrides(cli_overrides)
        .harness_overrides(overrides)
        .loader_overrides(loader_overrides)
        .strict_config(strict_config)
        .build()
        .await
        .context("failed to build Corbanu Terminal config for Telegram")?;
    if let Some(identity) = telegram_config.identity_instructions.as_deref() {
        let identity = identity.replace("<cwd>", &core_config.cwd.display().to_string());
        core_config.developer_instructions = Some(match core_config.developer_instructions.take()
        {
            Some(existing) => format!("{existing}\n\n{identity}"),
            None => identity,
        });
    }
    Ok(core_config)
}

fn telegram_default_model_provider(config: &TelegramConfig) -> Option<String> {
    config
        .default_model
        .as_deref()
        .and_then(canonical_catalog_provider)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telegram_default_model_uses_its_canonical_provider() {
        let mut config = TelegramConfig {
            default_model: Some(codex_model_provider_info::KIMI_CODE_K3_MODEL.to_string()),
            ..TelegramConfig::default()
        };
        assert_eq!(
            telegram_default_model_provider(&config).as_deref(),
            Some(codex_model_provider_info::KIMI_CODE_PROVIDER_ID)
        );

        config.default_model = Some("custom-model".to_string());
        assert_eq!(telegram_default_model_provider(&config), None);
    }
}
