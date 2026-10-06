//! Launch-time application of the stored `/security` level. With no state
//! file this is a no-op, so a flag-off install starts exactly as before.

use std::path::Path;
use std::path::PathBuf;

use codex_config::types::ShellEnvironmentPolicyToml;
use codex_features::Feature;

use super::aggressive;
use super::level;
use super::level::ChosenLevel;
use super::level::LevelContext;
use super::level::StoredLevel;
use crate::legacy_core::config::Config;
use crate::legacy_core::config::ConfigOverrides;

pub(crate) struct LaunchPlan {
    codex_home: PathBuf,
    stored: StoredLevel,
    replaced_flags: Vec<&'static str>,
}

impl LaunchPlan {
    /// Read the stored level before any config is loaded, keep the vault rule
    /// file in step and add the Aggressive overrides.
    pub(crate) fn prepare(
        codex_home: &Path,
        cli_kv_overrides: &mut Vec<(String, toml::Value)>,
    ) -> Result<Self, String> {
        let stored = level::load(codex_home);
        if stored != StoredLevel::Absent {
            level::sync_rules(codex_home, stored.enforced()).map_err(|err| {
                format!(
                    "Could not apply the stored security level ({}): {err}",
                    level::rules_path(codex_home).display()
                )
            })?;
        }
        let plan = Self {
            codex_home: codex_home.to_path_buf(),
            stored,
            replaced_flags: Vec::new(),
        };
        if plan.aggressive() {
            cli_kv_overrides.extend(aggressive::base_overrides(codex_home));
        }
        Ok(plan)
    }

    fn aggressive(&self) -> bool {
        self.stored.enforced() == ChosenLevel::Aggressive
    }

    /// A remote app server does not load these overrides; refuse rather than
    /// show Aggressive over a server that does not enforce it.
    pub(crate) fn check_target(&self, uses_remote_app_server: bool) -> Result<(), String> {
        if self.aggressive() && uses_remote_app_server {
            return Err(format!(
                "Security level Aggressive cannot be enforced on a remote app server. Start locally, or choose Permissive in /security first ({}).",
                level::state_path(&self.codex_home).display()
            ));
        }
        Ok(())
    }

    pub(crate) fn extend_env_overrides(
        &self,
        user_env: &ShellEnvironmentPolicyToml,
        cli_kv_overrides: &mut Vec<(String, toml::Value)>,
    ) {
        if self.aggressive() {
            cli_kv_overrides.extend(aggressive::env_overrides(user_env));
        }
    }

    pub(crate) fn apply_launch_overrides(&mut self, overrides: &mut ConfigOverrides) {
        if self.aggressive() {
            self.replaced_flags = aggressive::apply_launch_overrides(overrides);
        }
    }

    /// Verify the loaded config, record warnings and publish what is active.
    pub(crate) fn finish(self, config: &mut Config) -> Result<(), String> {
        let launch_warning = match &self.stored {
            StoredLevel::Invalid(reason) => Some(reason.clone()),
            StoredLevel::Absent | StoredLevel::Chosen(_) => None,
        };
        if self.aggressive() {
            let rules_present = std::fs::read_to_string(level::rules_path(&self.codex_home))
                .is_ok_and(|contents| contents == level::rules_contents());
            let failures = aggressive::verify(config, rules_present);
            if !failures.is_empty() {
                return Err(format!(
                    "Security level Aggressive could not be fully applied, so Corbanu Terminal did not start:\n  {}\nFix the setting, or choose Permissive by editing {}.",
                    failures.join("\n  "),
                    level::state_path(&self.codex_home).display()
                ));
            }
            if let Some(reason) = &launch_warning {
                config.startup_warnings.push(format!(
                    "Stored security level is unreadable ({reason}). Aggressive is enforced; choose a level in /security to repair it."
                ));
            }
            if !self.replaced_flags.is_empty() {
                config.startup_warnings.push(format!(
                    "Security level Aggressive ignored {}.",
                    self.replaced_flags.join(", ")
                ));
            }
        }
        level::install_context(LevelContext {
            codex_home: self.codex_home,
            picker_enabled: config.features.enabled(Feature::SecurityLevels)
                || !matches!(
                    self.stored,
                    StoredLevel::Absent | StoredLevel::Chosen(ChosenLevel::Permissive)
                ),
            active: self.stored.enforced(),
            launch_warning,
        });
        Ok(())
    }
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
