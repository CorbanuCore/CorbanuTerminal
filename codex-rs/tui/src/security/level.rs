//! PF-24-S03: the human-chosen `/security` level behind the `security_levels`
//! flag.
//!
//! The stored level itself (`security_level.toml`, the Aggressive rule file)
//! lives in [`codex_security_level::level`]. It takes effect when Corbanu
//! Terminal starts: the launch path turns a stored Aggressive level into
//! existing controls (see [`super::aggressive`]) and verifies them before
//! anything is shown as active.

use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

pub(crate) use codex_security_level::level::ChosenLevel;
pub use codex_security_level::level::NestedAgents;
#[cfg(test)]
pub(crate) use codex_security_level::level::RULES_DIR;
pub(crate) use codex_security_level::level::STATE_FILE;
pub(crate) use codex_security_level::level::StoredLevel;
pub(crate) use codex_security_level::level::VAULT_PROGRAMS;
pub(crate) use codex_security_level::level::load;
pub(crate) use codex_security_level::level::load_state;
pub(crate) use codex_security_level::level::rules_contents;
pub(crate) use codex_security_level::level::rules_path;
pub(crate) use codex_security_level::level::save;
pub(crate) use codex_security_level::level::state_path;
pub(crate) use codex_security_level::level::sync_rules;

/// What this process enforces, fixed at launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LevelContext {
    pub(crate) codex_home: PathBuf,
    /// The `security_levels` flag, or a stored non-Permissive level that the
    /// human must be able to see and change.
    pub(crate) picker_enabled: bool,
    /// Verified at launch; Aggressive only when every control was observed.
    pub(crate) active: ChosenLevel,
    /// The `protected_mode_preflight` flag (PF-29-S01).
    pub(crate) preflight_enabled: bool,
    /// What may be claimed about the protected boundary; `None` when no
    /// preflight applies (flag off or not Aggressive).
    pub(crate) boundary: Option<super::preflight::Boundary>,
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
/// restart that activates it. Contained panes (#218, feature
/// `contained_external_agents` with the secretless launch contract armed)
/// run sandboxed, behind the bridge and with a person's approval for every
/// command, file edit and web request, so they are allowed.
pub(crate) fn external_agent_block_reason() -> Option<String> {
    if crate::claude_panes::containment::contained_launch_ready() {
        return None;
    }
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
            "Claude panes are off under security level {}: Claude Code would run outside Corbanu's sandbox with your environment and network. Choose Permissive in /security and restart to use them, or turn on the contained_external_agents and secretless_agent_launch features to run them in the sandbox, asking you before every command, file edit and web request; /panes switches back to Main.",
            if active != ChosenLevel::Permissive { active } else { stored }.name()
        )
    })
}

#[cfg(test)]
#[path = "level_tests.rs"]
mod tests;
