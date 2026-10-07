//! Agents started by agent commands while Aggressive is enforced (nested
//! launches).
//!
//! The level always comes from a stored `security_level.toml`, never from the
//! environment. A Corbanu home is the origin of a nested launch when it
//! enforces Aggressive and this process can neither write it nor read its
//! vault store, which is exactly what the Aggressive profile does to agent
//! commands; a person's own launch, and a Permissive sandbox, can read it. Candidate homes are the
//! one named by [`ORIGIN_ENV`] (which Aggressive puts into every agent
//! command's environment), this process's own home, the account's default
//! homes, and every home registered by an Aggressive launch. Only the first
//! two can be changed by a command; the registry sits outside the workspace,
//! where agent commands cannot write.

use std::io;
use std::path::Path;
use std::path::PathBuf;

use codex_config::types::ShellEnvironmentPolicyToml;
use sha2::Digest;
use sha2::Sha256;

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
    /// Runs an agent without a person (`exec`, `review`); held to Aggressive
    /// when nested launches pass.
    Agent,
    /// An interactive session. Its approval prompts would be answered by the
    /// agent that started it, so it is never allowed nested.
    Interactive,
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
    match account_home() {
        Ok(Some(account_home)) => {
            homes.push(account_home.join(".corbanu"));
            homes.push(account_home.join(".pfterminal"));
            homes.extend(registered_homes(&registry_dir(&account_home)));
        }
        Ok(None) => {}
        // An agent command can make the lookup fail; it is the only
        // launcher that sets the marker.
        Err(err) if marker.is_some() => {
            return NestedLaunch::Refuse(format!(
                "Could not check whether security level Aggressive applies to this launch ({err}), so it is refused."
            ));
        }
        Err(_) => {}
    }
    // The marker's home comes last, so a real origin found above is the one
    // the nested run protects.
    homes.extend(marker);
    decide(name, kind, &nested_origins(homes))
}

/// Every candidate home that is the origin of a nested launch, with its
/// setting.
pub(crate) fn nested_origins(homes: Vec<PathBuf>) -> Vec<(PathBuf, NestedAgents)> {
    let mut origins: Vec<(PathBuf, NestedAgents)> = Vec::new();
    for home in homes {
        if origins.iter().any(|(seen, _)| *seen == home) {
            continue;
        }
        let (stored, nested) = level::load_state(&home);
        // An Aggressive session that saved Permissive stays Aggressive, with
        // its rule file, until it restarts.
        let aggressive = stored.enforced() == ChosenLevel::Aggressive
            || std::fs::read_to_string(level::rules_path(&home))
                .is_ok_and(|contents| contents == level::rules_contents());
        if aggressive && sandboxed_away_from(&home) {
            let nested = match stored {
                level::StoredLevel::Chosen(ChosenLevel::Aggressive) => nested,
                _ => NestedAgents::Refuse,
            };
            origins.push((home, nested));
        }
    }
    origins
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Probe {
    Allowed,
    Denied,
    /// Another error, such as no free file descriptors, which a command can
    /// cause. Counted as denied.
    Unknown,
}

fn probe<T>(result: io::Result<T>) -> Probe {
    match result {
        Ok(_) => Probe::Allowed,
        Err(err)
            if matches!(
                err.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem
            ) =>
        {
            Probe::Denied
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Probe::Allowed,
        Err(_) => Probe::Unknown,
    }
}

/// Whether this process can neither write `home` nor read its vault store,
/// as inside an Aggressive agent command. A person's own launch and a
/// Permissive sandbox can read the store; Aggressive launches create its
/// folder so the denial is observable.
fn sandboxed_away_from(home: &Path) -> bool {
    probe(tempfile::NamedTempFile::new_in(home)) != Probe::Allowed
        && probe(std::fs::read_dir(home.join("secrets"))) != Probe::Allowed
}

/// Refuse wins when several origins apply.
pub(crate) fn decide(
    name: &str,
    kind: NestedKind,
    origins: &[(PathBuf, NestedAgents)],
) -> NestedLaunch {
    let Some((origin, nested)) = origins
        .iter()
        .find(|(_, nested)| *nested == NestedAgents::Refuse)
        .or_else(|| origins.first())
    else {
        return NestedLaunch::NotNested;
    };
    let command = if name.is_empty() {
        "`corbanu`".to_string()
    } else {
        format!("`corbanu {name}`")
    };
    let started = format!(
        "{command} was started by an agent command while security level Aggressive is enforced ({})",
        level::state_path(origin).display()
    );
    match (kind, nested) {
        (NestedKind::Agent, NestedAgents::Pass) => NestedLaunch::EnforceAggressive(origin.clone()),
        (NestedKind::Agent, NestedAgents::Refuse) => NestedLaunch::Refuse(format!(
            "{started}. Nested agent launches are set to refuse; a person can let `corbanu exec` and `corbanu review` run with Aggressive enforced in /security."
        )),
        (NestedKind::Interactive, _) => NestedLaunch::Refuse(format!(
            "{started}. Interactive sessions are never allowed there: the agent that started one would answer its approval prompts."
        )),
        (NestedKind::Host, _) => NestedLaunch::Refuse(format!(
            "{started}. It is never allowed there: its client chooses each agent's sandbox and approvals, or the agent runs outside Corbanu's sandbox, so Aggressive cannot be enforced."
        )),
        (NestedKind::Credentials, _) => NestedLaunch::Refuse(format!(
            "{started}. It reads stored credentials, which Aggressive keeps from agent commands."
        )),
    }
}

/// Record or forget `codex_home` as an Aggressive origin, so nested launches
/// that point `CODEX_HOME` elsewhere still find it, and drop entries for
/// homes that no longer store a level.
pub(crate) fn register_origin(codex_home: &Path, aggressive: bool) -> io::Result<()> {
    let Some(account_home) = account_home()? else {
        return Ok(());
    };
    let registry = registry_dir(&account_home);
    registry_entry(&registry, codex_home, aggressive)?;
    for entry in std::fs::read_dir(&registry).into_iter().flatten().flatten() {
        let stale = std::fs::read_to_string(entry.path())
            .map(PathBuf::from)
            .is_ok_and(|home| !level::state_path(&home).exists());
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}

pub(crate) fn registry_entry(
    registry: &Path,
    codex_home: &Path,
    aggressive: bool,
) -> io::Result<()> {
    let digest = Sha256::digest(codex_home.as_os_str().as_encoded_bytes());
    let name = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let entry = registry.join(name);
    if !aggressive {
        return match std::fs::remove_file(&entry) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        };
    }
    let path = codex_home
        .to_str()
        .ok_or_else(|| io::Error::other("Corbanu home path is not UTF-8"))?;
    if std::fs::read_to_string(&entry).ok().as_deref() == Some(path) {
        return Ok(());
    }
    std::fs::create_dir_all(registry)?;
    std::fs::write(entry, path)
}

pub(crate) fn registered_homes(registry: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(registry)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .collect()
}

fn registry_dir(account_home: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        account_home.join("Library/Application Support/Corbanu/aggressive-homes")
    } else {
        account_home.join(".local/state/corbanu/aggressive-homes")
    }
}

/// The Aggressive-homes registry for this account, which the Aggressive
/// profile keeps read-only. `None` when the account has no home entry.
pub(crate) fn origin_registry_dir() -> Option<PathBuf> {
    account_home()
        .ok()
        .flatten()
        .map(|home| registry_dir(&home))
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

/// The account's home directory from the account database, not `$HOME`,
/// which an agent command can change. `Ok(None)` when the account has no
/// entry. Debug builds honour `CORBANU_TEST_ACCOUNT_HOME` so tests never read
/// the operator's profile.
fn account_home() -> io::Result<Option<PathBuf>> {
    if cfg!(debug_assertions)
        && let Some(home) = std::env::var_os("CORBANU_TEST_ACCOUNT_HOME")
    {
        return Ok(Some(PathBuf::from(home)));
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        let mut size = 16 * 1024;
        loop {
            let mut buffer = vec![0 as libc::c_char; size];
            // SAFETY: `passwd` is plain data that getpwuid_r overwrites.
            let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
            let mut result = std::ptr::null_mut();
            // SAFETY: every pointer is valid for the call and `buffer.len()`
            // is its real size.
            let status = unsafe {
                libc::getpwuid_r(
                    libc::getuid(),
                    &mut entry,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    &mut result,
                )
            };
            if status == libc::ERANGE && size < 4 * 1024 * 1024 {
                size *= 4;
                continue;
            }
            if status != 0 {
                return Err(io::Error::from_raw_os_error(status));
            }
            if result.is_null() || entry.pw_dir.is_null() {
                return Ok(None);
            }
            // SAFETY: getpwuid_r succeeded, so `pw_dir` is a NUL-terminated
            // string inside `buffer`, which is still alive.
            let dir = unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) };
            let home = PathBuf::from(std::ffi::OsStr::from_bytes(dir.to_bytes()));
            return Ok(home.is_absolute().then_some(home));
        }
    }
    #[cfg(not(unix))]
    {
        Ok(None)
    }
}

#[cfg(test)]
#[path = "nested_tests.rs"]
mod tests;
