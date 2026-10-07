//! PF-23-S03: the durable security state and its recovery at start.
//!
//! A confirmed transition (the trusted controller's, next in this sprint)
//! writes `security_state.json` in the Corbanu home before it takes effect:
//! the level the next start enforces and the revocation state (its
//! generation, revoked ids and the kill switch). At config load and at every session
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

use std::io;
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
    // Written by the trusted controller's transitions (next in this sprint).
    #[cfg_attr(not(test), allow(dead_code))]
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
        Err(LoadError::Io(reason) | LoadError::Corrupt(reason)) => Recovery {
            level: SecurityLevel::Aggressive,
            revocations: RevocationState::new(),
            unreadable: Some(reason),
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

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
