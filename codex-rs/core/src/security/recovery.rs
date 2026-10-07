//! PF-23-S03: the durable security state and its recovery at start.
//!
//! A confirmed transition (see `effective_policy::transition`) writes
//! `security_state.json` in the Corbanu home before it takes effect: the
//! level the next start enforces and the revocation state (its generation,
//! revoked ids and the kill switch). At config load and at every session
//! start the stricter of that level and the configured one applies, and the
//! revocation state comes back, so a restart never runs a weaker interval.
//! Unreadable or unknown content (also a transient read error) enforces
//! Aggressive with the kill switch on and says so; it is never read as
//! Permissive. A confirmed transition replaces such a file; the revocations
//! it held are then lost, which only matters for authority that is itself
//! never stored (grants live in memory).
//!
//! Limits: same-user rollback or deletion of the file needs the external
//! anchor of the authoritative store (PF-20); agent commands cannot read or
//! write the file once the protected-path rules apply, but under Permissive
//! before untrusted content they can. A transition in one process reaches
//! another process on the same home at its next session start.

use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use serde::Deserialize;
use serde::Serialize;

use super::transition::TransitionError;

/// The durable state file inside a Corbanu home.
pub(crate) const STATE_FILE: &str = "security_state.json";
/// Held while a transition reads, merges and writes [`STATE_FILE`].
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const LOCK_FILE: &str = "security_state.lock";
const STATE_VERSION: u32 = 1;
/// How long a transition waits for [`LOCK_FILE`].
#[cfg_attr(not(test), allow(dead_code))]
const LOCK_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// What the next start enforces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DurableSecurityState {
    version: u32,
    pub(crate) level: SecurityLevel,
    pub(crate) revocations: RevocationState,
}

impl DurableSecurityState {
    pub(crate) fn new(level: SecurityLevel, revocations: RevocationState) -> Self {
        Self {
            version: STATE_VERSION,
            level,
            revocations,
        }
    }
}

/// The policy a session starts from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Recovery {
    pub(crate) level: SecurityLevel,
    pub(crate) revocations: RevocationState,
    /// Set when the file could not be read: Aggressive and the kill switch
    /// apply until it is replaced.
    pub(crate) unreadable: Option<String>,
    /// The Corbanu home it was read from.
    pub(crate) home: Option<PathBuf>,
}

impl Recovery {
    /// The warning shown at start, if any.
    pub(crate) fn warning(&self) -> Option<String> {
        self.unreadable.as_ref().map(|reason| {
            format!(
                "Security state is unreadable ({reason}). Aggressive and the kill switch are \
                 enforced until the file is repaired or removed."
            )
        })
    }
}

/// The stricter of `configured` and the stored level, with the stored
/// revocation state.
pub(crate) fn recover(codex_home: &Path, configured: SecurityLevel) -> Recovery {
    let home = Some(codex_home.to_path_buf());
    match load(codex_home) {
        Ok(None) => Recovery {
            level: configured,
            revocations: RevocationState::new(),
            unreadable: None,
            home,
        },
        Ok(Some(state)) => Recovery {
            level: configured.max(state.level),
            revocations: state.revocations,
            unreadable: None,
            home,
        },
        Err(LoadError::Io(reason) | LoadError::Corrupt(reason)) => Recovery {
            level: SecurityLevel::Aggressive,
            revocations: RevocationState::new(),
            unreadable: Some(reason),
            home,
        },
    }
}

enum LoadError {
    /// The file could not be read now (a transient error is possible).
    Io(String),
    /// Its content is not a valid state.
    Corrupt(String),
}

fn load(codex_home: &Path) -> Result<Option<DurableSecurityState>, LoadError> {
    let path = codex_home.join(STATE_FILE);
    let contents = match std::fs::read(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(LoadError::Io(format!(
                "cannot read {}: {err}",
                path.display()
            )));
        }
    };
    let corrupt = |reason: String| LoadError::Corrupt(format!("{}: {reason}", path.display()));
    let state: DurableSecurityState =
        serde_json::from_slice(&contents).map_err(|err| corrupt(err.to_string()))?;
    if state.version != STATE_VERSION {
        return Err(corrupt(format!("unsupported version {}", state.version)));
    }
    state
        .revocations
        .validate()
        .map_err(|err| corrupt(err.to_string()))?;
    Ok(Some(state))
}

/// Where a transition is made durable before it takes effect.
pub(crate) trait TransitionStore {
    /// The Corbanu home, when the store is one.
    fn home(&self) -> Option<&Path>;

    /// Under one lock: read the stored state (`None` when absent or
    /// corrupt), let `merge` build the next one from it, and save that. A
    /// `merge` error writes nothing; a read or save error is
    /// [`TransitionError::Persist`]. `lowers_level` also makes a confirmed
    /// downgrade reach the configured level.
    fn update(
        &self,
        lowers_level: bool,
        merge: &mut dyn FnMut(
            Option<DurableSecurityState>,
        ) -> Result<DurableSecurityState, TransitionError>,
    ) -> Result<DurableSecurityState, TransitionError>;
}

/// The Corbanu home's state file, then for a downgrade the user's
/// `[security]` level in `config.toml`, so the stricter-of rule at start sees
/// it. The file is written first: a crash or a failed `config.toml` write
/// leaves the stricter level for the next start.
// The trusted `/security` confirmation (PF-24-S02) is the production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HomeTransitionStore {
    codex_home: PathBuf,
}

#[cfg_attr(not(test), allow(dead_code))]
impl HomeTransitionStore {
    pub(crate) fn new(codex_home: &Path) -> Self {
        Self {
            codex_home: codex_home.to_path_buf(),
        }
    }
}

impl TransitionStore for HomeTransitionStore {
    fn home(&self) -> Option<&Path> {
        Some(&self.codex_home)
    }

    fn update(
        &self,
        lowers_level: bool,
        merge: &mut dyn FnMut(
            Option<DurableSecurityState>,
        ) -> Result<DurableSecurityState, TransitionError>,
    ) -> Result<DurableSecurityState, TransitionError> {
        let persist = |err: io::Error| TransitionError::Persist(err.to_string());
        std::fs::create_dir_all(&self.codex_home).map_err(persist)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.codex_home.join(LOCK_FILE))
            .map_err(persist)?;
        // Never wait without limit: a lock held by another process (or by a
        // command) makes the save fail, and a restrictive transition then
        // applies in memory anyway.
        let deadline = std::time::Instant::now() + LOCK_WAIT;
        loop {
            match lock.try_lock() {
                Ok(()) => break,
                Err(std::fs::TryLockError::WouldBlock) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    return Err(TransitionError::Persist(
                        "the security state is locked by another process".to_string(),
                    ));
                }
                Err(std::fs::TryLockError::Error(err)) => return Err(persist(err)),
            }
        }
        let stored = match load(&self.codex_home) {
            Ok(stored) => stored,
            // A confirmed transition repairs a corrupt file.
            Err(LoadError::Corrupt(_)) => None,
            Err(LoadError::Io(reason)) => return Err(TransitionError::Persist(reason)),
        };
        let next = merge(stored)?;
        let contents = serde_json::to_vec_pretty(&next)
            .map_err(|err| TransitionError::Persist(err.to_string()))?;
        write_atomically(&self.codex_home.join(STATE_FILE), &contents).map_err(persist)?;
        if load(&self.codex_home).ok().flatten().as_ref() != Some(&next) {
            return Err(TransitionError::Persist(
                "the saved security state could not be verified".to_string(),
            ));
        }
        if lowers_level
            && let Err(err) = crate::config::edit::ConfigEditsBuilder::new(&self.codex_home)
                .set_security_level(next.level)
                .apply_blocking()
        {
            return Err(TransitionError::Persist(format!(
                "config.toml could not be updated ({err}), so the next start keeps the \
                 stricter level"
            )));
        }
        Ok(next)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn write_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("state path has no parent"))?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(contents)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|err| err.error)?;
    #[cfg(unix)]
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
