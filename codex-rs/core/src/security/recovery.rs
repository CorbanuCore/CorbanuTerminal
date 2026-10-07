//! PF-23-S03: the durable security state and its recovery at start.
//!
//! A confirmed transition (see `effective_policy::transition`) writes
//! `security_state.json` in the Corbanu home before anything changes in
//! memory: the level the next start enforces and the revocation state (its
//! generation and the kill switch). At start the stricter of that level and
//! the configured one applies, and the revocation state comes back, so a
//! restart never runs a weaker interval and never brings back a revoked
//! generation. Unreadable or unknown content enforces Aggressive with the
//! kill switch on and says so; it is never read as Permissive.
//!
//! Same-user rollback of the file to an older copy needs the external anchor
//! of the authoritative store (PF-20); until that is active, agent commands
//! cannot read or write the file once the protected-path rules apply.

use std::io;
use std::io::Write;
use std::path::Path;

use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use serde::Deserialize;
use serde::Serialize;

/// The durable state file inside a Corbanu home.
pub(crate) const STATE_FILE: &str = "security_state.json";
const STATE_VERSION: u32 = 1;

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
    /// apply until a human confirms a new level.
    pub(crate) unreadable: Option<String>,
}

impl Recovery {
    /// The warning shown at start, if any.
    pub(crate) fn warning(&self) -> Option<String> {
        self.unreadable.as_ref().map(|reason| {
            format!(
                "Security state is unreadable ({reason}). Aggressive and the kill switch are \
                 enforced; confirm a level in /security and restart to repair it."
            )
        })
    }
}

/// The stricter of `configured` and the stored level, with the stored
/// revocation state.
pub(crate) fn recover(codex_home: &Path, configured: SecurityLevel) -> Recovery {
    match load(codex_home) {
        Ok(None) => Recovery {
            level: configured,
            revocations: RevocationState::new(),
            unreadable: None,
        },
        Ok(Some(state)) => Recovery {
            level: configured.max(state.level),
            revocations: state.revocations,
            unreadable: None,
        },
        Err(reason) => Recovery {
            level: SecurityLevel::Aggressive,
            revocations: RevocationState::new(),
            unreadable: Some(reason),
        },
    }
}

fn load(codex_home: &Path) -> Result<Option<DurableSecurityState>, String> {
    let path = codex_home.join(STATE_FILE);
    let contents = match std::fs::read(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
    };
    let state: DurableSecurityState =
        serde_json::from_slice(&contents).map_err(|err| format!("{}: {err}", path.display()))?;
    if state.version != STATE_VERSION {
        return Err(format!(
            "{}: unsupported version {}",
            path.display(),
            state.version
        ));
    }
    state
        .revocations
        .validate()
        .map_err(|err| format!("{}: {err}", path.display()))?;
    Ok(Some(state))
}

/// Where a transition is made durable before it takes effect.
pub(crate) trait TransitionStore {
    fn persist(&self, state: &DurableSecurityState) -> io::Result<()>;
}

/// The Corbanu home's state file, then the user's `[security]` level in
/// `config.toml`, so the stricter-of rule at start sees the confirmed level.
/// Either order of a crash leaves the stricter level for the next start: the
/// file is written first, and only a downgrade lowers `config.toml`.
// The trusted `/security` confirmation (PF-24-S02) is the production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HomeTransitionStore {
    codex_home: std::path::PathBuf,
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
    fn persist(&self, state: &DurableSecurityState) -> io::Result<()> {
        let contents = serde_json::to_vec_pretty(state).map_err(io::Error::other)?;
        write_atomically(&self.codex_home.join(STATE_FILE), &contents)?;
        if load(&self.codex_home).map_err(io::Error::other)?.as_ref() != Some(state) {
            return Err(io::Error::other(
                "saved security state could not be verified",
            ));
        }
        crate::config::edit::ConfigEditsBuilder::new(&self.codex_home)
            .set_security_level(state.level)
            .apply_blocking()
            .map_err(io::Error::other)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn write_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("state path has no parent"))?;
    std::fs::create_dir_all(parent)?;
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
