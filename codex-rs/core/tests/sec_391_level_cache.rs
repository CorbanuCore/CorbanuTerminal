//! #391 outside unit-test builds: the level `/security` saves while a process
//! runs takes effect at its next start, while a config layer that raises the
//! level applies on every load. Its own test binary, because the last load
//! marks the whole process brokered.

use codex_config::LoaderOverrides;
use codex_core::BrokerModelAuthOrigin;
use codex_core::config::Config;
use codex_core::config::ConfigBuilder;
use codex_core::config::ConfigOverrides;
use codex_features::Feature;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

async fn load(
    home: &TempDir,
    cwd: &TempDir,
    cli: Vec<(String, toml::Value)>,
) -> std::io::Result<Config> {
    ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .cli_overrides(cli)
        .harness_overrides(ConfigOverrides {
            cwd: Some(cwd.path().to_path_buf()),
            ..Default::default()
        })
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await
}

fn broker(config: &Config) -> (SecurityLevel, bool, BrokerModelAuthOrigin) {
    (
        config.security_level,
        config.features.enabled(Feature::BrokerModelAuth),
        config.broker_model_auth_origin,
    )
}

#[tokio::test]
async fn sec_391_saved_level_waits_for_restart_config_level_applies_now() -> std::io::Result<()> {
    let supported = cfg!(any(target_os = "macos", target_os = "linux", windows));
    let home = TempDir::new()?;
    let cwd = TempDir::new()?;
    assert_eq!(
        broker(&load(&home, &cwd, Vec::new()).await?),
        (
            SecurityLevel::Permissive,
            false,
            BrokerModelAuthOrigin::Config
        )
    );

    // `/security` saves Aggressive (its level file and Core's confirmed
    // record) while this process runs: a rebuilt config enforces the level
    // but does not make the running process brokered.
    std::fs::write(
        home.path().join("security_level.toml"),
        "version = 1\nlevel = \"aggressive\"\n",
    )?;
    std::fs::write(
        home.path().join("security_state.json"),
        r#"{"version":1,"level":"aggressive","revocations":{"schema_version":1,"generation":0,"kill_switch_active":false}}"#,
    )?;
    assert_eq!(
        broker(&load(&home, &cwd, Vec::new()).await?),
        (
            SecurityLevel::Aggressive,
            false,
            BrokerModelAuthOrigin::Config
        )
    );

    // A config layer that raises the level applies on this load.
    let raised = load(
        &home,
        &cwd,
        vec![(
            "security".to_string(),
            toml::toml! { version = 1
            level = "aggressive" }
            .into(),
        )],
    )
    .await?;
    assert_eq!(
        broker(&raised),
        (
            SecurityLevel::Aggressive,
            supported,
            if supported {
                BrokerModelAuthOrigin::AggressiveLevel
            } else {
                BrokerModelAuthOrigin::Config
            }
        )
    );
    Ok(())
}
