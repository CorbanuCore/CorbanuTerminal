//! Agents started by agent commands while Aggressive is enforced (nested
//! launches).
//!
//! An Aggressive session puts [`ORIGIN_ENV`] into every agent command's
//! environment. A command can drop that variable or change `CODEX_HOME`, so
//! the launching `corbanu` also checks its own and the account's default
//! Corbanu homes: a home that stores Aggressive but that this process cannot
//! write means it runs inside an Aggressive agent command's sandbox, which
//! makes the home read-only and cannot be left. Corbanu cannot run on a home
//! it cannot write, so a person's own launch never matches. The level always
//! comes from a stored `security_level.toml`, never from the environment.

use std::path::Path;
use std::path::PathBuf;

use codex_config::types::ShellEnvironmentPolicyToml;

use super::aggressive;
use super::launch;
use super::level;
use super::level::ChosenLevel;
use super::level::NestedAgents;
use crate::legacy_core::config::Config;
use crate::legacy_core::config::ConfigOverrides;

/// Set in agent command environments to the home that chose Aggressive.
pub const ORIGIN_ENV: &str = "CORBANU_SECURITY_ORIGIN";

/// What a `corbanu` subcommand does, for nested-launch decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NestedKind {
    /// Starts an agent this process holds to Aggressive itself.
    Agent,
    /// Serves agents to a client that chooses their sandbox and approvals,
    /// or runs an agent outside Corbanu's sandbox. Never allowed nested.
    Host,
    /// Reads stored credentials. Never allowed nested.
    Credentials,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NestedLaunch {
    NotNested,
    Refuse(String),
    /// Run with Aggressive enforced; the path is the home that chose it.
    EnforceAggressive(PathBuf),
}

/// Decide whether `corbanu <name>` may run here (`name` is empty for a new
/// interactive session).
pub fn nested_launch(name: &str, kind: NestedKind) -> NestedLaunch {
    let marker = std::env::var_os(ORIGIN_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let mut homes = Vec::new();
    homes.extend(
        codex_utils_home_dir::find_codex_home()
            .ok()
            .map(|home| home.to_path_buf()),
    );
    homes.extend(default_homes());
    decide(name, kind, aggressive_origin(marker, homes))
}

/// The marker's home, or else the first unwritable candidate home, whose
/// stored level enforces Aggressive. A forged marker can only make a launch
/// stricter.
pub(crate) fn aggressive_origin(
    marker: Option<PathBuf>,
    homes: Vec<PathBuf>,
) -> Option<(PathBuf, NestedAgents)> {
    let stored_aggressive = |home: PathBuf| {
        let (stored, nested) = level::load_state(&home);
        (stored.enforced() == ChosenLevel::Aggressive).then_some((home, nested))
    };
    marker.and_then(stored_aggressive).or_else(|| {
        homes
            .into_iter()
            .filter_map(stored_aggressive)
            .find(|(home, _)| tempfile::NamedTempFile::new_in(home).is_err())
    })
}

pub(crate) fn decide(
    name: &str,
    kind: NestedKind,
    origin: Option<(PathBuf, NestedAgents)>,
) -> NestedLaunch {
    let Some((origin, nested)) = origin else {
        return NestedLaunch::NotNested;
    };
    let command = if name.is_empty() {
        "`corbanu`".to_string()
    } else {
        format!("`corbanu {name}`")
    };
    let started = format!(
        "{command} was started by an agent command while security level Aggressive is enforced ({})",
        level::state_path(&origin).display()
    );
    match (kind, nested) {
        (NestedKind::Agent, NestedAgents::Pass) => NestedLaunch::EnforceAggressive(origin),
        (NestedKind::Agent, NestedAgents::Refuse) => NestedLaunch::Refuse(format!(
            "{started}. Nested agent launches are set to refuse; a person can set them to pass, with Aggressive enforced in the nested agent, in /security."
        )),
        (NestedKind::Host, _) => NestedLaunch::Refuse(format!(
            "{started}. It is never allowed there: its client chooses each agent's sandbox and approvals, or the agent runs outside Corbanu's sandbox, so Aggressive cannot be enforced."
        )),
        (NestedKind::Credentials, _) => NestedLaunch::Refuse(format!(
            "{started}. It reads stored credentials, which Aggressive keeps from agent commands."
        )),
    }
}

/// `-c` overrides for a nested `corbanu exec` held to Aggressive.
pub fn aggressive_cli_overrides(
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

/// Default Corbanu homes from the account database, not `$HOME`, which an
/// agent command can change.
fn default_homes() -> Vec<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let mut buffer = vec![0 as libc::c_char; 16 * 1024];
        // SAFETY: `passwd` is plain data that getpwuid_r overwrites.
        let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
        let mut result = std::ptr::null_mut();
        // SAFETY: every pointer is valid for the call and `buffer.len()` is
        // its real size.
        let status = unsafe {
            libc::getpwuid_r(
                libc::getuid(),
                &mut entry,
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut result,
            )
        };
        if status != 0 || result.is_null() || entry.pw_dir.is_null() {
            return Vec::new();
        }
        // SAFETY: getpwuid_r succeeded, so `pw_dir` is a NUL-terminated
        // string inside `buffer`, which is still alive.
        let dir = unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) };
        let home = PathBuf::from(std::ffi::OsStr::from_bytes(dir.to_bytes()));
        vec![home.join(".corbanu"), home.join(".pfterminal")]
    }
    #[cfg(not(unix))]
    {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "nested_tests.rs"]
mod tests;
