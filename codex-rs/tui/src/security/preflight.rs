//! PF-29-S01 activation preflight for the `/security` Aggressive level
//! (feature `protected_mode_preflight`).
//!
//! - The picker runs the Core preflight before Aggressive can be saved and
//!   refuses while a control is missing or a raw secret would still reach an
//!   agent or the model; it rechecks on confirm and refuses on drift.
//! - A passing save writes `$CODEX_HOME/security_preflight.toml` (finding IDs
//!   and times, never values). While it exists and Aggressive is stored,
//!   launch denies agent reads of every credential path the inventory finds
//!   (`~/.ssh` keys, CLI tokens, browser profiles, history) and verifies it.
//! - Launch re-audits: a boundary that is no longer clean is reported, never
//!   claimed. Conversations recorded before activation cannot be resumed or
//!   forked: they may hold secrets from before the boundary existed.

use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use codex_protocol::ThreadId;
use serde::Deserialize;
use serde::Serialize;

use super::level;
use super::level::ChosenLevel;
use crate::legacy_core::config::Config;
use crate::legacy_core::protected_preflight::CORBANU_HOME_STORES;
use crate::legacy_core::protected_preflight::InventorySources;
use crate::legacy_core::protected_preflight::Preflight;
use crate::legacy_core::protected_preflight::ReadinessFlags;
use crate::legacy_core::protected_preflight::database_glob;
use crate::legacy_core::protected_preflight::database_siblings;
use crate::legacy_core::protected_preflight::file_sources;
use crate::legacy_core::protected_preflight::has_glob_chars;
use crate::legacy_core::protected_preflight::is_database_file;

pub(crate) const RECEIPT_FILE: &str = "security_preflight.toml";
/// 2: `activated_at` in milliseconds. A version-1 receipt (seconds) is
/// unreadable, so it can never make old conversations look new.
const RECEIPT_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Receipt {
    version: u32,
    pub(crate) saved_at: i64,
    /// First verified protected launch after the save (Unix milliseconds).
    pub(crate) activated_at: Option<i64>,
    pub(crate) findings: Vec<String>,
}

/// What this process may claim about the protected boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Boundary {
    /// Preflight clean at this launch; conversations from before
    /// `activated_at` (Unix milliseconds) are contaminated.
    Clean {
        activated_at: i64,
    },
    NotClean {
        blockers: Vec<String>,
    },
    /// No usable receipt; nothing is claimed and no resume is allowed.
    Unverified(String),
}

impl Boundary {
    pub(crate) fn summary(&self) -> String {
        match self {
            Self::Clean { .. } => "protected boundary checked at launch".to_string(),
            Self::NotClean { blockers } => format!(
                "protected boundary not clean: {} blocker{}",
                blockers.len(),
                if blockers.len() == 1 { "" } else { "s" }
            ),
            Self::Unverified(_) => "protected boundary unverified".to_string(),
        }
    }
}

/// Inputs the picker needs to run (and rerun) the preflight.
#[derive(Clone, Debug)]
pub(crate) struct PreflightInput {
    pub(crate) sources: InventorySources,
    pub(crate) flags: ReadinessFlags,
}

impl PreflightInput {
    pub(crate) fn from_config(config: &Config) -> Self {
        Self {
            sources: crate::legacy_core::protected_preflight::sources_from_config(config),
            flags: ReadinessFlags::from_config(config),
        }
    }
}

pub(crate) fn receipt_path(codex_home: &Path) -> PathBuf {
    codex_home.join(RECEIPT_FILE)
}

/// `Ok(None)` when absent; `Err` when present but unreadable.
pub(crate) fn load_receipt(codex_home: &Path) -> Result<Option<Receipt>, String> {
    let path = receipt_path(codex_home);
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
    };
    let receipt: Receipt = toml::from_str(&contents)
        .map_err(|err| format!("{}: {}", path.display(), err.message()))?;
    if receipt.version != RECEIPT_VERSION {
        return Err(format!(
            "{}: unsupported version {}",
            path.display(),
            receipt.version
        ));
    }
    Ok(Some(receipt))
}

pub(crate) fn save_receipt(codex_home: &Path, preflight: &Preflight) -> io::Result<()> {
    write_receipt(
        codex_home,
        &Receipt {
            version: RECEIPT_VERSION,
            saved_at: now_ms(),
            activated_at: None,
            findings: preflight.inventory.finding_ids().into_iter().collect(),
        },
    )
}

pub(crate) fn remove_receipt(codex_home: &Path) -> io::Result<()> {
    match std::fs::remove_file(receipt_path(codex_home)) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        Ok(()) | Err(_) => Ok(()),
    }
}

fn write_receipt(codex_home: &Path, receipt: &Receipt) -> io::Result<()> {
    let contents = toml::to_string(receipt).map_err(io::Error::other)?;
    let path = receipt_path(codex_home);
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("receipt path has no parent"))?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(contents.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(&path).map_err(|err| err.error)?;
    Ok(())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}

/// Credential paths agent commands must not read while Aggressive is stored
/// after a preflight. Computed before config loads, from files only.
pub(crate) fn isolation_paths(
    codex_home: &Path,
    home: Option<&Path>,
    cwd: Option<&Path>,
) -> Vec<PathBuf> {
    if !receipt_path(codex_home).exists() {
        return Vec::new();
    }
    // Folders for Corbanu's stores exist from the start, so a sandbox that
    // can only mask existing paths (bubblewrap) covers them too.
    for name in [
        "sessions",
        "archived_sessions",
        "shell_snapshots",
        "log",
        "wallet",
    ] {
        if let Err(err) = std::fs::create_dir_all(codex_home.join(name)) {
            tracing::warn!("could not create {name} in the Corbanu home: {err}");
        }
    }
    let found = Preflight::run(
        &file_sources(codex_home, home, cwd),
        ReadinessFlags::default(),
    )
    .inventory
    .isolation_paths();
    let mut paths = CORBANU_HOME_STORES
        .iter()
        .map(|name| codex_home.join(name))
        .collect::<Vec<_>>();
    if has_glob_chars(codex_home) {
        // The home cannot prefix a glob: deny each database and its
        // siblings by exact path instead.
        paths.extend(found.into_iter().flat_map(|path| {
            if is_database_file(codex_home, &path) {
                database_siblings(&path)
            } else {
                vec![path]
            }
        }));
    } else {
        // One glob covers every database, including files created later.
        paths.extend(
            found
                .into_iter()
                .filter(|path| !is_database_file(codex_home, path)),
        );
        paths.push(database_glob(codex_home));
    }
    paths.sort();
    paths.dedup();
    paths
}

/// Every isolated path must be unreadable to agent commands.
pub(crate) fn verify_isolation(config: &Config, paths: &[PathBuf]) -> Vec<String> {
    let policy = config.permissions.file_system_sandbox_policy();
    let cwd = config.cwd.as_path();
    let codex_home = config.codex_home.as_path();
    // The path check does not evaluate globs; the database glob must be in
    // the policy for the files it covers to count as denied.
    let glob = database_glob(codex_home).to_string_lossy().into_owned();
    let databases_denied = policy.get_unreadable_globs_with_cwd(cwd).contains(&glob);
    paths
        .iter()
        .filter(|path| {
            let covered = if path.as_os_str().to_string_lossy() == glob {
                databases_denied
            } else {
                (databases_denied && is_database_file(codex_home, path))
                    || !policy.can_read_path_with_cwd(path, cwd)
            };
            !covered
        })
        .map(|path| {
            format!(
                "Isolation: agent commands can still read {}",
                path.display()
            )
        })
        .collect()
}

/// Launch-time re-audit. Writes `activated_at` on the first clean launch.
pub(crate) fn audit_at_launch(codex_home: &Path, config: &Config) -> Boundary {
    let receipt = match load_receipt(codex_home) {
        Ok(Some(receipt)) => receipt,
        Ok(None) => {
            return Boundary::Unverified(
                "Aggressive was saved without a preflight; choose it again in /security"
                    .to_string(),
            );
        }
        Err(reason) => return Boundary::Unverified(reason),
    };
    let preflight = Preflight::run(
        &crate::legacy_core::protected_preflight::sources_from_config(config),
        ReadinessFlags::from_config(config),
    );
    let mut blockers = preflight.blockers();
    // Found now (another folder, a new file) but not denied at launch.
    blockers.extend(
        verify_isolation(config, &preflight.inventory.isolation_paths())
            .into_iter()
            .map(|line| {
                format!(
                    "{line}; it was not found when Corbanu Terminal started (another folder or config layer, or a new file). Remove it, or restart from the folder that holds it"
                )
            }),
    );
    if !blockers.is_empty() {
        return Boundary::NotClean { blockers };
    }
    let activated_at = match receipt.activated_at {
        Some(activated_at) => activated_at,
        None => {
            let activated_at = now_ms();
            let updated = Receipt {
                activated_at: Some(activated_at),
                ..receipt
            };
            if let Err(err) = write_receipt(codex_home, &updated) {
                return Boundary::Unverified(format!("cannot record activation: {err}"));
            }
            activated_at
        }
    };
    Boundary::Clean { activated_at }
}

/// Why resuming or forking `thread_id` is refused, if it is.
pub(crate) fn resume_refusal(thread_id: &ThreadId) -> Option<String> {
    refusal_for(level::context()?, thread_id)
}

fn refusal_for(context: &level::LevelContext, thread_id: &ThreadId) -> Option<String> {
    if context.active != ChosenLevel::Aggressive {
        return None;
    }
    let boundary = context.boundary.as_ref()?;
    let activated_at = match boundary {
        Boundary::Clean { activated_at } => *activated_at,
        Boundary::NotClean { .. } | Boundary::Unverified(_) => {
            return Some(format!(
                "Security level Aggressive: earlier conversations cannot be resumed while the {}. Start a new session, or fix it in /security and restart.",
                boundary.summary()
            ));
        }
    };
    let created = thread_created_at(thread_id);
    (created.is_none_or(|created| created < activated_at)).then(|| {
        "Security level Aggressive: this conversation was recorded before protected mode was activated and may contain secrets. Start a new session instead.".to_string()
    })
}

/// Thread IDs are UUIDv7: the creation time (milliseconds) is part of the ID.
fn thread_created_at(thread_id: &ThreadId) -> Option<i64> {
    let uuid = uuid::Uuid::parse_str(&thread_id.to_string()).ok()?;
    let (seconds, nanos) = uuid.get_timestamp()?.to_unix();
    i64::try_from(seconds)
        .ok()?
        .checked_mul(1000)?
        .checked_add(i64::from(nanos / 1_000_000))
}

#[cfg(test)]
#[path = "preflight_tests.rs"]
mod tests;
