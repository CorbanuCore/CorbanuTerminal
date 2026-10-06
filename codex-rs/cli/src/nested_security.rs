//! `corbanu exec` and `corbanu review` held to Aggressive when an agent
//! command under Aggressive started them and nested launches pass.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use codex_config::TomlValue;
use codex_config::types::ShellEnvironmentPolicyToml;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_exec::EnforcedSecurity;

pub(crate) struct NestedAggressive {
    codex_home: PathBuf,
    origin: PathBuf,
    cli_overrides: Vec<(String, TomlValue)>,
}

impl NestedAggressive {
    pub(crate) fn new(origin: PathBuf) -> anyhow::Result<Self> {
        let codex_home = codex_core::config::find_codex_home()?.to_path_buf();
        let cli_overrides = codex_tui::aggressive_cli_overrides(&codex_home, &origin)
            .map_err(anyhow::Error::msg)?;
        Ok(Self {
            codex_home,
            origin,
            cli_overrides,
        })
    }
}

impl EnforcedSecurity for NestedAggressive {
    fn notice(&self) -> String {
        format!(
            "Security level Aggressive is enforced because an agent command under Aggressive started this run ({}).",
            self.origin.display()
        )
    }

    fn cli_overrides(&self) -> Vec<(String, TomlValue)> {
        self.cli_overrides.clone()
    }

    fn env_overrides(&self, user_env: &ShellEnvironmentPolicyToml) -> Vec<(String, TomlValue)> {
        codex_tui::aggressive_env_overrides(user_env)
    }

    fn apply_overrides(&self, overrides: &mut ConfigOverrides) -> Vec<&'static str> {
        codex_tui::apply_aggressive_launch_overrides(overrides)
    }

    fn verify<'a>(
        &'a self,
        config: &'a Config,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'a>> {
        Box::pin(codex_tui::verify_aggressive_config(
            &self.codex_home,
            &self.origin,
            config,
        ))
    }
}
