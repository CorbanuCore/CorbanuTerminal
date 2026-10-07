//! Agents started by agent commands while Aggressive is enforced (nested
//! launches). Detection lives in [`codex_security_level::nested`] so the
//! standalone binaries run it too; this module holds what a nested run
//! held to Aggressive needs from the TUI's security code.

use std::path::Path;

use codex_config::types::ShellEnvironmentPolicyToml;

use super::aggressive;
use super::launch;
use super::level;
use super::level::ChosenLevel;
use crate::legacy_core::config::Config;
use crate::legacy_core::config::ConfigOverrides;

pub use codex_security_level::nested::NestedKind;
pub use codex_security_level::nested::NestedLaunch;
pub use codex_security_level::nested::ORIGIN_ENV;
#[cfg(test)]
pub(crate) use codex_security_level::nested::account_home;
#[cfg(test)]
pub(crate) use codex_security_level::nested::decide;
pub use codex_security_level::nested::nested_launch;
#[cfg(test)]
pub(crate) use codex_security_level::nested::nested_origins;
pub(crate) use codex_security_level::nested::register_origin;
#[cfg(test)]
pub(crate) use codex_security_level::nested::registered_homes;
#[cfg(test)]
pub(crate) use codex_security_level::nested::registry_entry;

/// The Aggressive-homes registry for this account, which the Aggressive
/// profile keeps read-only. `None` when the account has no home entry.
pub(crate) fn origin_registry_dir() -> Option<std::path::PathBuf> {
    codex_security_level::nested::account_home()
        .ok()
        .flatten()
        .map(|home| codex_security_level::nested::registry_dir(&home))
}

/// `-c` overrides for a nested `corbanu exec` held to Aggressive. Writes the
/// Aggressive vault rule into `codex_home` when it is missing.
pub fn prepare_nested_exec(
    codex_home: &Path,
    origin: &Path,
) -> Result<Vec<(String, toml::Value)>, String> {
    for home in [codex_home, origin] {
        if home.to_str().is_none() {
            return Err(format!(
                "Security level Aggressive needs a UTF-8 Corbanu home path; {} is not.",
                home.display()
            ));
        }
    }
    level::sync_rules(codex_home, ChosenLevel::Aggressive).map_err(|err| {
        format!(
            "Could not write the Aggressive vault rule ({}): {err}",
            level::rules_path(codex_home).display()
        )
    })?;
    Ok(aggressive::base_overrides(codex_home, origin))
}

/// Environment overrides that extend the user's own policy.
pub fn aggressive_env_overrides(
    user_env: &ShellEnvironmentPolicyToml,
) -> Vec<(String, toml::Value)> {
    aggressive::env_overrides(user_env)
}

/// Replace launch flags that would weaken Aggressive; returns the flags replaced.
pub fn apply_aggressive_launch_overrides(overrides: &mut ConfigOverrides) -> Vec<&'static str> {
    aggressive::apply_launch_overrides(overrides)
}

/// Every Aggressive row must hold in `config`, as at an Aggressive start.
pub async fn verify_aggressive_config(
    codex_home: &Path,
    origin: &Path,
    config: &Config,
) -> Result<(), String> {
    launch::verify_aggressive(codex_home, origin, config).await
}

#[cfg(test)]
#[path = "nested_tests.rs"]
mod tests;
