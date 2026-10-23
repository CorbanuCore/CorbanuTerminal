//! #81: a pin on a retired model (Claude Opus 4.8) moves to the provider's
//! current default with a visible notice instead of failing every turn.

use super::Config;
use super::ConfigOverrides;
use codex_config::config_toml::ConfigToml;
use codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL;
use codex_model_provider_info::ANTHROPIC_LEGACY_OPUS_4_8_MODEL;
use codex_model_provider_info::ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::CLAUDE_FABLE_5_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID;
use core_test_support::TempDirExt;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

async fn load(config_toml: &str, overrides: ConfigOverrides) -> std::io::Result<Config> {
    let cfg = toml::from_str::<ConfigToml>(config_toml).expect("config should deserialize");
    Config::load_from_base_config_with_overrides(cfg, overrides, tempdir()?.abs()).await
}

fn assert_notice(config: &Config, retired: &str, replacement: &str) {
    assert!(
        config.startup_warnings.iter().any(|warning| {
            warning.contains(&format!("Model `{retired}` has been retired"))
                && warning.contains(&format!("this session uses `{replacement}`"))
        }),
        "{:?}",
        config.startup_warnings
    );
}

#[tokio::test]
async fn config_pin_on_retired_opus_4_8_plan_moves_to_the_plan_default() -> std::io::Result<()> {
    // With and without an explicit provider: both shapes are what an upgrade
    // from 0.1.20 left in `config.toml`.
    for (config_toml, retired) in [
        (
            format!("model = {CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL:?}\n"),
            CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL,
        ),
        (
            format!("model = {ANTHROPIC_LEGACY_OPUS_4_8_MODEL:?}\n"),
            ANTHROPIC_LEGACY_OPUS_4_8_MODEL,
        ),
        (
            format!(
                "model_provider = {CLAUDE_PLAN_PROVIDER_ID:?}\nmodel = {CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL:?}\n"
            ),
            CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL,
        ),
    ] {
        let config = load(&config_toml, ConfigOverrides::default()).await?;
        assert_eq!(
            config.model_provider_id, CLAUDE_PLAN_PROVIDER_ID,
            "{config_toml}"
        );
        assert_eq!(
            config.model.as_deref(),
            Some(CLAUDE_PLAN_MODEL),
            "{config_toml}"
        );
        assert_notice(&config, retired, CLAUDE_PLAN_MODEL);
    }
    Ok(())
}

/// A resumed thread or a connector's saved state passes the model as a
/// runtime override; it is moved on the same way.
#[tokio::test]
async fn override_pin_on_retired_opus_4_8_moves_to_the_provider_default() -> std::io::Result<()> {
    for (requested, provider, replacement) in [
        (
            CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL,
            CLAUDE_PLAN_PROVIDER_ID,
            CLAUDE_PLAN_MODEL,
        ),
        (
            ANTHROPIC_LEGACY_OPUS_4_8_MODEL,
            CLAUDE_PLAN_PROVIDER_ID,
            CLAUDE_PLAN_MODEL,
        ),
        (
            ANTHROPIC_LEGACY_OPUS_4_8_MODEL,
            ANTHROPIC_PROVIDER_ID,
            ANTHROPIC_DEFAULT_MODEL,
        ),
    ] {
        let config = load(
            "",
            ConfigOverrides {
                model: Some(requested.to_string()),
                model_provider: Some(provider.to_string()),
                ..ConfigOverrides::default()
            },
        )
        .await?;
        assert_eq!(config.model_provider_id, provider, "{requested}");
        assert_eq!(config.model.as_deref(), Some(replacement), "{requested}");
        assert_notice(&config, requested, replacement);
    }
    Ok(())
}

#[tokio::test]
async fn current_claude_plan_model_has_no_retirement_notice() -> std::io::Result<()> {
    let config = load(
        &format!("model_provider = {CLAUDE_PLAN_PROVIDER_ID:?}\nmodel = {CLAUDE_PLAN_MODEL:?}\n"),
        ConfigOverrides::default(),
    )
    .await?;
    assert_eq!(config.model.as_deref(), Some(CLAUDE_PLAN_MODEL));
    assert!(
        !config
            .startup_warnings
            .iter()
            .any(|warning| warning.contains("has been retired")),
        "{:?}",
        config.startup_warnings
    );
    Ok(())
}

/// Providers with their own catalogues are left alone, and a replacement that
/// is not the retirement's (the provider's own default) carries no notice.
#[tokio::test]
async fn other_providers_keep_their_handling_without_a_retirement_notice() -> std::io::Result<()> {
    let no_notice = |config: &Config| {
        assert!(
            !config
                .startup_warnings
                .iter()
                .any(|warning| warning.contains("has been retired")),
            "{:?}",
            config.startup_warnings
        );
    };
    let config = load(
        "",
        ConfigOverrides {
            model: Some(ANTHROPIC_LEGACY_OPUS_4_8_MODEL.to_string()),
            model_provider: Some(PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID.to_string()),
            ..ConfigOverrides::default()
        },
    )
    .await?;
    assert_eq!(
        config.model.as_deref(),
        Some(ANTHROPIC_LEGACY_OPUS_4_8_MODEL)
    );
    no_notice(&config);

    let config = load(
        &format!(
            "model_provider = {PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID:?}\nmodel = {ANTHROPIC_LEGACY_OPUS_4_8_MODEL:?}\n"
        ),
        ConfigOverrides::default(),
    )
    .await?;
    // That provider's own default, as before.
    assert_eq!(config.model.as_deref(), Some(CLAUDE_FABLE_5_MODEL));
    no_notice(&config);
    Ok(())
}
