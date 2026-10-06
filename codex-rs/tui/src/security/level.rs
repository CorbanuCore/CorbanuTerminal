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
}

pub(crate) fn state_path(codex_home: &Path) -> PathBuf {
    codex_home.join(STATE_FILE)
}

pub(crate) fn rules_path(codex_home: &Path) -> PathBuf {
    codex_home.join(RULES_DIR).join(RULES_FILE)
}

pub(crate) fn load(codex_home: &Path) -> StoredLevel {
    let path = state_path(codex_home);
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        // The rule file exists only while Aggressive is stored: a missing state
        // file next to it was deleted, not never written.
        Err(err) if err.kind() == io::ErrorKind::NotFound && rules_path(codex_home).exists() => {
            return StoredLevel::Invalid(format!(
                "{} is missing but {} exists",
                path.display(),
                rules_path(codex_home).display()
            ));
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => return StoredLevel::Absent,
        Err(err) => return StoredLevel::Invalid(format!("cannot read {}: {err}", path.display())),
    };
    let invalid = |reason: String| StoredLevel::Invalid(format!("{}: {reason}", path.display()));
    let file: StateFile = match toml::from_str(&contents) {
        Ok(file) => file,
        Err(err) => return invalid(err.message().to_string()),
    };
    if file.version != STATE_VERSION {
        return invalid(format!("unsupported version {}", file.version));
    }
    match file.level.as_str() {
        "permissive" => StoredLevel::Chosen(ChosenLevel::Permissive),
        "aggressive" => StoredLevel::Chosen(ChosenLevel::Aggressive),
        other => invalid(format!("unknown level `{other}`")),
    }
}

/// Persist a human choice and read the result back. Only a verified
/// read-back counts as saved. Aggressive writes the vault rule now; the rule
/// is removed only at the next launch, because an Aggressive session that
/// saves Permissive stays Aggressive (and keeps starting threads) until then.
pub(crate) fn save(codex_home: &Path, level: ChosenLevel) -> io::Result<()> {
    let contents = toml::to_string(&StateFile {
        version: STATE_VERSION,
        level: level.key().to_string(),
    })
    .map_err(io::Error::other)?;
    write_atomically(&state_path(codex_home), &contents)?;
    if level == ChosenLevel::Aggressive {
        sync_rules(codex_home, level)?;
    }
    match load(codex_home) {
        StoredLevel::Chosen(saved) if saved == level => Ok(()),
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

/// Claude panes run Claude Code outside Corbanu's sandbox, with this
/// process's environment, network and Claude's own permission bypass, so no
/// protected level can contain them. Refused while a protected level is
/// active or saved: a pane could otherwise rewrite the saved level before the
/// restart that activates it.
pub(crate) fn external_agent_block_reason() -> Option<String> {
    #[cfg(test)]
    if let Some((active, stored)) = test_levels::LEVELS.get() {
        return external_agent_block_reason_in(active, stored);
    }
    let context = context()?;
    external_agent_block_reason_in(context.active, load(&context.codex_home).enforced())
}

/// For entry points that may have no launch context (`corbanu
/// claude-pane-smoke`): `codex_home`'s saved level, and the active level when
/// there is one.
pub(crate) fn external_agent_block_reason_for_home(codex_home: &Path) -> Option<String> {
    let active = context().map_or(ChosenLevel::Permissive, |context| context.active);
    external_agent_block_reason_in(active, load(codex_home).enforced())
}

/// Stand-in active and saved levels for the Claude pane gate tests, on the
/// current thread only.
#[cfg(test)]
pub(crate) mod test_levels {
    use std::cell::Cell;

    use super::ChosenLevel;

    thread_local! {
        pub(super) static LEVELS: Cell<Option<(ChosenLevel, ChosenLevel)>> =
            const { Cell::new(None) };
    }

    /// Sets the levels until dropped.
    pub(crate) struct Guard;

    pub(crate) fn set(active: ChosenLevel, stored: ChosenLevel) -> Guard {
        LEVELS.set(Some((active, stored)));
        Guard
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            LEVELS.set(None);
        }
    }
}

fn external_agent_block_reason_in(active: ChosenLevel, stored: ChosenLevel) -> Option<String> {
    (active != ChosenLevel::Permissive || stored != ChosenLevel::Permissive).then(|| {
        format!(
            "Claude panes are off under security level {}: Claude Code would run outside Corbanu's sandbox with your environment and network. Choose Permissive in /security and restart to use them; /panes switches back to Main.",
            if active != ChosenLevel::Permissive { active } else { stored }.name()
        )
    })
}

#[cfg(test)]
#[path = "level_tests.rs"]
mod tests;
