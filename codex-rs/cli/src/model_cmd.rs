//! Local recovery actions for the selected provider and model.

use anyhow::Context;
use codex_core::config::ConfigBuilder;
use codex_core::config::ConfigOverrides;
use codex_tui::Cli as TuiCli;
use codex_utils_cli::CliConfigOverrides;

#[derive(Debug, clap::Parser)]
pub(crate) struct ModelCommand {
    #[command(subcommand)]
    action: ModelAction,
}

#[derive(Debug, clap::Subcommand)]
enum ModelAction {
    /// Clear the local rate-limit cooldown for the selected provider, model and credential.
    ClearCooldown,
}

pub(crate) async fn run(
    command: ModelCommand,
    config_overrides: CliConfigOverrides,
    interactive: TuiCli,
) -> anyhow::Result<()> {
    let loader_overrides =
        crate::loader_overrides_for_profile(interactive.config_profile_v2.as_ref())?;
    let shared = interactive.shared.into_inner();
    let config = ConfigBuilder::default()
        .cli_overrides(
            config_overrides
                .parse_overrides()
                .map_err(anyhow::Error::msg)?,
        )
        .harness_overrides(ConfigOverrides {
            model: shared.model,
            cwd: shared.cwd,
            provider_account: interactive.account,
            ..Default::default()
        })
        .loader_overrides(loader_overrides)
        .build()
        .await?;
    match command.action {
        ModelAction::ClearCooldown => {
            let model = config.model.as_deref().context(
                "No model is configured. Select the affected model with `corbanu -m MODEL model clear-cooldown`.",
            )?;
            let state = codex_state::StateRuntime::init(
                config.sqlite_config().clone(),
                config.model_provider_id.clone(),
            )
            .await?;
            codex_core::clear_provider_cooldown(
                &state, &config, model, /*enable_codex_api_key_env*/ true,
            )
            .await?;
            println!(
                "Cleared the local cooldown for {}/{}. The next request will probe the provider.",
                config.model_provider_id, model
            );
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "model_cmd_tests.rs"]
mod tests;
