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
use super::preflight;
use crate::legacy_core::config::Config;
use crate::legacy_core::config::ConfigOverrides;

pub(crate) struct LaunchPlan {
    codex_home: PathBuf,
    stored: StoredLevel,
    replaced_flags: Vec<&'static str>,
    /// PF-29-S01: credential paths denied to agent commands.
    isolated: Vec<PathBuf>,
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
        let mut plan = Self {
            codex_home: codex_home.to_path_buf(),
            stored,
            replaced_flags: Vec::new(),
            isolated: Vec::new(),
        };
        if plan.aggressive() && codex_home.to_str().is_none() {
            return Err(format!(
                "Security level Aggressive needs a UTF-8 Corbanu home path; {} is not.",
                codex_home.display()
            ));
        }
        if plan.aggressive() {
            cli_kv_overrides.extend(aggressive::base_overrides(codex_home));
            // Only after a preflight (its receipt exists, even if unreadable);
            // denying more paths never weakens Aggressive.
            let cwd = std::env::current_dir().ok();
            plan.isolated = preflight::isolation_paths(
                codex_home,
                crate::legacy_core::protected_preflight::home_dir().as_deref(),
                cwd.as_deref(),
            );
            aggressive::deny_reads(cli_kv_overrides, &plan.isolated);
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
    pub(crate) async fn finish(self, config: &mut Config) -> Result<(), String> {
        let launch_warning = match &self.stored {
            StoredLevel::Invalid(reason) => Some(reason.clone()),
            StoredLevel::Absent | StoredLevel::Chosen(_) => None,
        };
        let preflight_enabled = config.features.enabled(Feature::ProtectedModePreflight);
        let mut boundary = None;
        if self.aggressive() {
            verify_aggressive(&self.codex_home, config).await?;
            let unisolated = preflight::verify_isolation(config, &self.isolated);
            if !unisolated.is_empty() {
                return Err(format!(
                    "Security level Aggressive could not be fully applied:\n  {}\nChoose Permissive by editing {}.",
                    unisolated.join("\n  "),
                    level::state_path(&self.codex_home).display()
                ));
            }
            if preflight_enabled {
                let audited = preflight::audit_at_launch(&self.codex_home, config);
                match &audited {
                    preflight::Boundary::Clean { .. } => {}
                    preflight::Boundary::NotClean { blockers } => {
                        config.startup_warnings.push(format!(
                            "Security level Aggressive is enforced, but the protected boundary is not clean ({} blocker{}). Earlier conversations cannot be resumed. See /security:\n  {}",
                            blockers.len(),
                            if blockers.len() == 1 { "" } else { "s" },
                            blockers.join("\n  ")
                        ));
                    }
                    preflight::Boundary::Unverified(reason) => {
                        config.startup_warnings.push(format!(
                            "Security level Aggressive is enforced, but the protected boundary is unverified ({reason}). Earlier conversations cannot be resumed."
                        ));
                    }
                }
                boundary = Some(audited);
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
        level::install_context(LevelContext {
            codex_home: self.codex_home,
            picker_enabled: config.features.enabled(Feature::SecurityLevels)
                || !matches!(
                    self.stored,
                    StoredLevel::Absent | StoredLevel::Chosen(ChosenLevel::Permissive)
                ),
            active: self.stored.enforced(),
            preflight_enabled,
            boundary,
        });
        Ok(())
    }
}

async fn verify_aggressive(codex_home: &Path, config: &Config) -> Result<(), String> {
    let rules_present = std::fs::read_to_string(level::rules_path(codex_home))
        .is_ok_and(|contents| contents == level::rules_contents());
    let mut failures = aggressive::verify(config, rules_present);
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
            verify_aggressive(&context.codex_home, config).await
        }
        Some(_) | None => Ok(()),
    }
}

#[cfg(test)]
#[path = "launch_tests.rs"]
mod tests;
