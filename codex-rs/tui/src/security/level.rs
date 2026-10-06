//! PF-24-S03: the human-chosen `/security` level behind the `security_levels`
//! flag.
//!
//! The level lives in `$CODEX_HOME/security_level.toml`, outside every config
//! layer, so no `-c` override, profile, project file or agent config write can
//! select or downgrade it. It takes effect when Corbanu Terminal starts: the
//! launch path turns a stored Aggressive level into existing controls
//! (see [`super::aggressive`]) and verifies them before anything is shown as
//! active. Unknown or corrupt state enforces Aggressive and is reported; it is
//! never read as Permissive.

use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const STATE_FILE: &str = "security_level.toml";
const STATE_VERSION: u32 = 1;
/// Exec-policy rules written only while Aggressive is stored.
pub(crate) const RULES_FILE: &str = "corbanu-security-aggressive.rules";
const RULES_DIR: &str = "rules";
pub(crate) const VAULT_PROGRAMS: [&str; 5] = [
    "corbanu",
    "codex",
    "pfterminal",
    "corbanu-debug",
    "pfterminal-debug",
];

/// A level a human can choose in this build. Moderate is not offered yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChosenLevel {
    Permissive,
    Aggressive,
}

impl ChosenLevel {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Permissive => "Permissive",
            Self::Aggressive => "Aggressive",
        }
    }

    fn key(self) -> &'static str {
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
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Refuse => "refuse",
            Self::Pass => "pass",
        }
    }

    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Refuse => Self::Pass,
            Self::Pass => Self::Refuse,
        }
    }
}

/// What the state file says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StoredLevel {
    /// No file: the feature was never used; today's behaviour.
    Absent,
    Chosen(ChosenLevel),
    /// Unreadable or unknown content. Enforced as Aggressive.
    Invalid(String),
}

impl StoredLevel {
    /// The level this state enforces at the next start.
    pub(crate) fn enforced(&self) -> ChosenLevel {
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

pub(crate) fn state_path(codex_home: &Path) -> PathBuf {
    codex_home.join(STATE_FILE)
}

pub(crate) fn rules_path(codex_home: &Path) -> PathBuf {
    codex_home.join(RULES_DIR).join(RULES_FILE)
}

pub(crate) fn load(codex_home: &Path) -> StoredLevel {
    load_state(codex_home).0
}

/// The stored level and nested-launch setting. Anything unreadable enforces
/// Aggressive and refuses nested launches.
pub(crate) fn load_state(codex_home: &Path) -> (StoredLevel, NestedAgents) {
    match load_file(codex_home) {
        Ok((level, nested)) => (StoredLevel::Chosen(level), nested),
        Err(stored) => (stored, NestedAgents::Refuse),
    }
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
pub(crate) fn save(codex_home: &Path, level: ChosenLevel, nested: NestedAgents) -> io::Result<()> {
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
    match load_state(codex_home) {
        (StoredLevel::Chosen(saved), saved_nested)
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
pub(crate) fn sync_rules(codex_home: &Path, level: ChosenLevel) -> io::Result<()> {
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

pub(crate) fn rules_contents() -> String {
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

/// What this process enforces, fixed at launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LevelContext {
    pub(crate) codex_home: PathBuf,
    /// The `security_levels` flag, or a stored non-Permissive level that the
    /// human must be able to see and change.
    pub(crate) picker_enabled: bool,
    /// Verified at launch; Aggressive only when every control was observed.
    pub(crate) active: ChosenLevel,
}

static CONTEXT: OnceLock<LevelContext> = OnceLock::new();

/// Set once by the launch path after the Aggressive controls are verified.
pub(crate) fn install_context(context: LevelContext) {
    let _ = CONTEXT.set(context);
}

pub(crate) fn context() -> Option<&'static LevelContext> {
    CONTEXT.get()
}

/// `/status` line for the flagged level: what is active now and, when it
/// differs, what the next start will enforce.
pub(crate) fn status_line(active: ChosenLevel, stored: &StoredLevel) -> String {
    let next = stored.enforced();
    match stored {
        StoredLevel::Invalid(_) => format!(
            "{} active; stored level unreadable, Aggressive enforced (/security)",
            active.name()
        ),
        StoredLevel::Absent | StoredLevel::Chosen(_) if next != active => format!(
            "{} active; {} saved for next start (/security)",
            active.name(),
            next.name()
        ),
        StoredLevel::Absent | StoredLevel::Chosen(_) => {
            format!("{} active (/security)", active.name())
        }
    }
}

/// Human permission changes are refused while a non-Permissive level is
/// stored or active: `/permissions` must not quietly undo Aggressive.
pub(crate) fn permission_change_block_reason() -> Option<String> {
    let context = context()?;
    let stored = load(&context.codex_home).enforced();
    (context.active == ChosenLevel::Aggressive || stored == ChosenLevel::Aggressive).then(|| {
        "Security level Aggressive manages permissions. Choose Permissive in /security and restart before changing them.".to_string()
    })
}

#[cfg(test)]
#[path = "level_tests.rs"]
mod tests;
