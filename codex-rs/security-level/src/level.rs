//! The stored `/security` level. It lives in `$CODEX_HOME/security_level.toml`,
//! outside every config layer, so no `-c` override, profile, project file or
//! agent config write can select or downgrade it. Unknown or corrupt state
//! enforces Aggressive and is reported; it is never read as Permissive.

use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

/// The level file inside a Corbanu home.
pub const STATE_FILE: &str = "security_level.toml";
const STATE_VERSION: u32 = 1;
/// Core's confirmed-transition record (PF-23-S03), cross-checked on load.
pub const CONFIRMED_STATE_FILE: &str = "security_state.json";
/// Shared-locked by every process that enforces Aggressive.
pub const ACTIVE_LOCK_FILE: &str = "security_level.lock";
/// Exec-policy rules written only while Aggressive is stored.
pub const RULES_FILE: &str = "corbanu-security-aggressive.rules";
/// The permission profile an Aggressive launch defines and selects; core
/// reads it to follow the level Corbanu Terminal shows (#428).
pub const AGGRESSIVE_PROFILE_ID: &str = "corbanu-aggressive";
/// The exec-policy rules folder inside a Corbanu home.
pub const RULES_DIR: &str = "rules";
/// Program names whose `vault` subcommand the Aggressive rule forbids.
pub const VAULT_PROGRAMS: [&str; 5] = [
    "corbanu",
    "codex",
    "pfterminal",
    "corbanu-debug",
    "pfterminal-debug",
];

/// A level a human can choose in this build. Moderate is not offered yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChosenLevel {
    Permissive,
    Aggressive,
}

impl ChosenLevel {
    pub fn name(self) -> &'static str {
        match self {
            Self::Permissive => "Permissive",
            Self::Aggressive => "Aggressive",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Aggressive => "aggressive",
        }
    }
}

/// What an agent command may do when it starts `corbanu exec` or `review`
/// while Aggressive is enforced (a nested launch). Stored next to the level,
/// so only a person using `/security` can change it; saving Permissive resets
/// it to refuse.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NestedAgents {
    /// The nested launch is refused.
    #[default]
    Refuse,
    /// The nested agent runs with Aggressive enforced.
    Pass,
}

impl NestedAgents {
    pub fn name(self) -> &'static str {
        match self {
            Self::Refuse => "refuse",
            Self::Pass => "pass",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Refuse => Self::Pass,
            Self::Pass => Self::Refuse,
        }
    }
}

/// What the state file says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredLevel {
    /// No file: the feature was never used; today's behaviour.
    Absent,
    Chosen(ChosenLevel),
    /// Unreadable or unknown content. Enforced as Aggressive.
    Invalid(String),
}

impl StoredLevel {
    /// The level this state enforces at the next start.
    pub fn enforced(&self) -> ChosenLevel {
        match self {
            Self::Absent | Self::Chosen(ChosenLevel::Permissive) => ChosenLevel::Permissive,
            Self::Chosen(ChosenLevel::Aggressive) | Self::Invalid(_) => ChosenLevel::Aggressive,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StateFile {
    version: u32,
    level: String,
    /// Absent means refuse, so files written before this setting read the same.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nested_agents: Option<String>,
}

pub fn state_path(codex_home: &Path) -> PathBuf {
    codex_home.join(STATE_FILE)
}

pub fn rules_path(codex_home: &Path) -> PathBuf {
    codex_home.join(RULES_DIR).join(RULES_FILE)
}

pub fn load(codex_home: &Path) -> StoredLevel {
    load_state(codex_home).0
}

/// The stored level and nested-launch setting. Anything unreadable enforces
/// Aggressive and refuses nested launches.
///
/// PF-24-S02 tamper check: a confirmed level change is also recorded in
/// Core's `security_state.json`. A level file weaker than that record (edited
/// or deleted outside `/security`) reads as invalid, so Aggressive stays
/// enforced and the person is told. Changing both files the same way is not
/// detected (that needs the PF-20 external anchor).
pub fn load_state(codex_home: &Path) -> (StoredLevel, NestedAgents) {
    let (stored, nested) = match load_file(codex_home) {
        Ok((level, nested)) => (StoredLevel::Chosen(level), nested),
        Err(stored) => (stored, NestedAgents::Refuse),
    };
    match reconcile(codex_home, stored) {
        stored @ StoredLevel::Invalid(_) => (stored, NestedAgents::Refuse),
        stored => (stored, nested),
    }
}

/// What Core's confirmed-transition record says the next start enforces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmedLevel {
    Absent,
    /// `permissive`, `moderate` or `aggressive`.
    Level(String),
    /// Present with content that is not a valid record.
    Unreadable(String),
    /// Present but cannot be read now (for example inside a sandbox that
    /// denies it). Not evidence of tampering: deleting the file is not
    /// detected either.
    Inaccessible(String),
}

#[derive(Deserialize)]
struct ConfirmedFile {
    version: u32,
    level: String,
}

pub fn confirmed_state_path(codex_home: &Path) -> PathBuf {
    codex_home.join(CONFIRMED_STATE_FILE)
}

/// A light read of Core's record: its version and level only.
pub fn load_confirmed(codex_home: &Path) -> ConfirmedLevel {
    let path = confirmed_state_path(codex_home);
    let contents = match std::fs::read(&path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return ConfirmedLevel::Absent,
        Err(err) => {
            return ConfirmedLevel::Inaccessible(format!("cannot read {}: {err}", path.display()));
        }
    };
    match serde_json::from_slice::<ConfirmedFile>(&contents) {
        Ok(file)
            if file.version == 1
                && matches!(
                    file.level.as_str(),
                    "permissive" | "moderate" | "aggressive"
                ) =>
        {
            ConfirmedLevel::Level(file.level)
        }
        Ok(_) => {
            ConfirmedLevel::Unreadable(format!("{}: unknown version or level", path.display()))
        }
        Err(err) => ConfirmedLevel::Unreadable(format!("{}: {err}", path.display())),
    }
}

fn reconcile(codex_home: &Path, stored: StoredLevel) -> StoredLevel {
    if stored.enforced() == ChosenLevel::Aggressive {
        return stored;
    }
    match load_confirmed(codex_home) {
        ConfirmedLevel::Absent | ConfirmedLevel::Inaccessible(_) => stored,
        ConfirmedLevel::Level(level) if level != "aggressive" => stored,
        ConfirmedLevel::Level(_) => StoredLevel::Invalid(format!(
            "{} says {} but the confirmed level in {} is Aggressive; it may have been changed outside /security",
            state_path(codex_home).display(),
            match stored {
                StoredLevel::Absent => "nothing",
                StoredLevel::Chosen(_) | StoredLevel::Invalid(_) => "Permissive",
            },
            confirmed_state_path(codex_home).display()
        )),
        ConfirmedLevel::Unreadable(reason) => StoredLevel::Invalid(reason),
    }
}

/// Held for the life of a process that enforces Aggressive. A Permissive
/// launch of the same home leaves the Aggressive rule file and registry
/// entry in place while it is held (PF-24-S02).
pub fn hold_aggressive_lock(codex_home: &Path) -> io::Result<std::fs::File> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(codex_home.join(ACTIVE_LOCK_FILE))?;
    file.lock_shared()?;
    Ok(file)
}

/// Whether another process enforcing Aggressive runs on this home.
pub fn aggressive_running(codex_home: &Path) -> bool {
    let Ok(file) = std::fs::OpenOptions::new()
        .write(true)
        .open(codex_home.join(ACTIVE_LOCK_FILE))
    else {
        return false;
    };
    matches!(file.try_lock(), Err(std::fs::TryLockError::WouldBlock))
}

fn load_file(codex_home: &Path) -> Result<(ChosenLevel, NestedAgents), StoredLevel> {
    let path = state_path(codex_home);
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        // The rule file exists only while Aggressive is stored: a missing state
        // file next to it was deleted, not never written.
        Err(err) if err.kind() == io::ErrorKind::NotFound && rules_path(codex_home).exists() => {
            return Err(StoredLevel::Invalid(format!(
                "{} is missing but {} exists",
                path.display(),
                rules_path(codex_home).display()
            )));
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Err(StoredLevel::Absent),
        Err(err) => {
            return Err(StoredLevel::Invalid(format!(
                "cannot read {}: {err}",
                path.display()
            )));
        }
    };
    let invalid = |reason: String| StoredLevel::Invalid(format!("{}: {reason}", path.display()));
    let file: StateFile =
        toml::from_str(&contents).map_err(|err| invalid(err.message().to_string()))?;
    if file.version != STATE_VERSION {
        return Err(invalid(format!("unsupported version {}", file.version)));
    }
    let level = match file.level.as_str() {
        "permissive" => ChosenLevel::Permissive,
        "aggressive" => ChosenLevel::Aggressive,
        other => return Err(invalid(format!("unknown level `{other}`"))),
    };
    let nested = match file.nested_agents.as_deref() {
        None | Some("refuse") => NestedAgents::Refuse,
        Some("pass") => NestedAgents::Pass,
        Some(other) => return Err(invalid(format!("unknown nested_agents `{other}`"))),
    };
    Ok((level, nested))
}

/// Persist a human choice and read the result back. Only a verified
/// read-back counts as saved. Aggressive writes the vault rule now; the rule
/// is removed only at the next launch, because an Aggressive session that
/// saves Permissive stays Aggressive (and keeps starting threads) until then.
pub fn save(codex_home: &Path, level: ChosenLevel, nested: NestedAgents) -> io::Result<()> {
    let contents = toml::to_string(&StateFile {
        version: STATE_VERSION,
        level: level.key().to_string(),
        nested_agents: (level == ChosenLevel::Aggressive && nested == NestedAgents::Pass)
            .then(|| nested.name().to_string()),
    })
    .map_err(io::Error::other)?;
    write_atomically(&state_path(codex_home), &contents)?;
    if level == ChosenLevel::Aggressive {
        sync_rules(codex_home, level)?;
    }
    // The file itself, before the tamper check: during a downgrade Core's
    // record still says Aggressive until it is committed next.
    match load_file(codex_home) {
        Ok((saved, saved_nested))
            if saved == level && (saved_nested == nested || level == ChosenLevel::Permissive) =>
        {
            Ok(())
        }
        other => Err(io::Error::other(format!(
            "saved level could not be verified: {other:?}"
        ))),
    }
}

/// Aggressive needs the vault rule present; Permissive needs it absent so the
/// user's own exec policy applies exactly as before.
pub fn sync_rules(codex_home: &Path, level: ChosenLevel) -> io::Result<()> {
    let path = rules_path(codex_home);
    match level {
        ChosenLevel::Aggressive => {
            if std::fs::read_to_string(&path).ok().as_deref() != Some(rules_contents().as_str()) {
                write_atomically(&path, &rules_contents())?;
            }
            Ok(())
        }
        ChosenLevel::Permissive => match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        },
    }
}

pub fn rules_contents() -> String {
    let mut contents = String::from(
        "# Managed by Corbanu Terminal /security (Aggressive). Choosing Permissive removes this file.\n",
    );
    for program in VAULT_PROGRAMS {
        contents.push_str(&format!(
            "prefix_rule(pattern = [\"{program}\", \"vault\"], decision = \"forbidden\", justification = \"Security level Aggressive: agents cannot use the Corbanu vault.\")\n"
        ));
    }
    contents
}

fn write_atomically(path: &Path, contents: &str) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("state path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(contents.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|err| err.error)?;
    // Make the rename itself durable.
    #[cfg(unix)]
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
#[path = "level_tests.rs"]
mod tests;
