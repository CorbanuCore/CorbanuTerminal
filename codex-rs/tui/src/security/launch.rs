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
    /// Set when an agent command under Aggressive started this process; the
    /// home whose level chose Aggressive.
    nested_origin: Option<PathBuf>,
    replaced_flags: Vec<&'static str>,
}

impl LaunchPlan {
    /// Read the stored level before any config is loaded, keep the vault rule
    /// file in step and add the Aggressive overrides. `nested_origin` forces
    /// Aggressive whatever this home stores.
    pub(crate) fn prepare(
        codex_home: &Path,
        nested_origin: Option<&Path>,
        cli_kv_overrides: &mut Vec<(String, toml::Value)>,
    ) -> Result<Self, String> {
        let stored = level::load(codex_home);
        let plan = Self {
            codex_home: codex_home.to_path_buf(),
            stored,
            nested_origin: nested_origin.map(Path::to_path_buf),
            replaced_flags: Vec::new(),
        };
        if plan.stored != StoredLevel::Absent || plan.aggressive() {
            level::sync_rules(codex_home, plan.enforced()).map_err(|err| {
                format!(
                    "Could not apply the stored security level ({}): {err}",
                    level::rules_path(codex_home).display()
                )
            })?;
        }
        if plan.aggressive() {
            for home in [codex_home, plan.origin()] {
                if home.to_str().is_none() {
                    return Err(format!(
                        "Security level Aggressive needs a UTF-8 Corbanu home path; {} is not.",
                        home.display()
                    ));
                }
            }
            cli_kv_overrides.extend(aggressive::base_overrides(codex_home, plan.origin()));
        }
        Ok(plan)
    }

    fn enforced(&self) -> ChosenLevel {
        if self.nested_origin.is_some() {
            ChosenLevel::Aggressive
        } else {
            self.stored.enforced()
        }
    }

    fn aggressive(&self) -> bool {
        self.enforced() == ChosenLevel::Aggressive
    }

    fn origin(&self) -> &Path {
        self.nested_origin.as_deref().unwrap_or(&self.codex_home)
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
    pub(crate) async fn finish(self, config: &mut Config) -> Result<(), String> {
        let launch_warning = match &self.stored {
            StoredLevel::Invalid(reason) => Some(reason.clone()),
            StoredLevel::Absent | StoredLevel::Chosen(_) => None,
        };
        if self.aggressive() {
            verify_aggressive(&self.codex_home, self.origin(), config).await?;
            if let Some(origin) = &self.nested_origin {
                config.startup_warnings.push(format!(
                    "Security level Aggressive is enforced because an agent command under Aggressive started this session ({}).",
                    origin.display()
                ));
            }
            if let Some(home) = std::env::var_os("HOME")
                && config.cwd.as_path() == Path::new(&home)
            {
                config.startup_warnings.push(
                    "Security level Aggressive: the current folder is your home directory, so agent commands can write anywhere under it.".to_string(),
                );
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
        let active = self.enforced();
        let origin = self.origin().to_path_buf();
        level::install_context(LevelContext {
            codex_home: self.codex_home,
            origin,
            picker_enabled: config.features.enabled(Feature::SecurityLevels)
                || !matches!(
                    self.stored,
                    StoredLevel::Absent | StoredLevel::Chosen(ChosenLevel::Permissive)
                ),
            active,
        });
        Ok(())
    }
}

pub(crate) async fn verify_aggressive(
    codex_home: &Path,
    origin: &Path,
    config: &Config,
) -> Result<(), String> {
    let rules_present = std::fs::read_to_string(level::rules_path(codex_home))
        .is_ok_and(|contents| contents == level::rules_contents());
    let mut failures = aggressive::verify(config, rules_present, origin);
    failures.extend(aggressive::verify_exec_policy(config).await);
    if failures.is_empty() {
        return Ok(());
    }
    Err(format!(
        "Security level Aggressive could not be fully applied:\n  {}\nFix the setting, or choose Permissive by editing {}.",
        failures.join("\n  "),
        level::state_path(codex_home).display()
    ))
}

/// Every later config build (onboarding reload, resume/fork, new cwd) must
/// pass the same checks while this process enforces Aggressive.
pub(crate) async fn verify_reloaded(config: &Config) -> Result<(), String> {
    match level::context() {
        Some(context) if context.active == ChosenLevel::Aggressive => {
            verify_aggressive(&context.codex_home, &context.origin, config).await
        }
        Some(_) | None => Ok(()),
    }
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
