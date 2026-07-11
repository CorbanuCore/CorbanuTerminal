pub mod approvals;
pub mod auth;
mod bot;
mod bridge;
pub mod commands;
pub mod config;
pub mod error;
pub mod media;
mod model_selection;
mod polling;
pub mod render;
mod sandbox_preflight;
mod session;

#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;

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
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use teloxide::Bot;
use tracing::instrument;
use tracing::warn;

use crate::bot::run_bot;
use crate::bridge::BridgeHandle;
use crate::config::TelegramConfig;
use crate::config::TelegramMode;
use crate::error::TelegramError;
use crate::session::SessionStore;

#[derive(Debug, Parser, Clone)]
pub struct Cli {
    /// Error out when config.toml contains fields core does not recognize.
    #[arg(long = "strict-config", default_value_t = false)]
    pub strict_config: bool,
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
            chat_id,
            "Telegram allowlist includes a group or supergroup chat; every member of that chat can drive and approve turns"
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
    let environment_manager =
        EnvironmentManager::from_codex_home(core_config.codex_home.clone(), Some(runtime_paths))
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
    let sessions = SessionStore::load(&codex_home).await?;
    let bot = Bot::new(token);
    let media = crate::media::MediaStore::new(&codex_home);
    let bridge = BridgeHandle::spawn(bot.clone(), client, Arc::new(core_config), sessions);
    let result = run_bot(
        bot,
        bridge.clone(),
        allowlist,
        media,
        telegram_config.max_consecutive_polling_failures,
    )
    .await;
    if let Err(err) = bridge.shutdown().await {
        warn!("Telegram bridge shutdown failed: {err}");
    }
    result
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
        cwd: telegram_config.default_cwd.clone(),
        approval_policy: telegram_config.approval_policy,
        codex_self_exe: arg0_paths.codex_self_exe.clone(),
        codex_linux_sandbox_exe: arg0_paths.codex_linux_sandbox_exe.clone(),
        main_execve_wrapper_exe: arg0_paths.main_execve_wrapper_exe.clone(),
        ..Default::default()
    };
    codex_core::config::ConfigBuilder::default()
        .cli_overrides(cli_overrides)
        .harness_overrides(overrides)
        .loader_overrides(loader_overrides)
        .strict_config(strict_config)
        .build()
        .await
        .context("failed to build PFTerminal config for Telegram")
}
