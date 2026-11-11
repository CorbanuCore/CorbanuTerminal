//! #1976 (D491, option (a)): on Windows, Moderate and Aggressive keep agent
//! commands and file tools away from protected files only through the
//! default (elevated) Windows sandbox. Without any Windows sandbox, commands
//! Windows treats as read-only and in-process file tools would run with no
//! sandbox at all, so under those levels both are refused, saying which level
//! and how to set the sandbox up. The session still starts, so a person can
//! set the sandbox up or change the level. Permissive is unchanged, and so is
//! the non-admin sandbox, which already refuses the levels' deny-read
//! profiles (#300). The level is the session's live one: every config layer,
//! the stored level and the Aggressive recovery of an unreadable store
//! (`Config::security_level`), raised by the live policy.

use codex_protocol::config_types::WindowsSandboxLevel;
use codex_security_policy::SecurityLevel;

/// How to set the default Windows sandbox up or leave the level. Set up in
/// the TUI, the sandbox applies to the running session from its next turn
/// (#2000); a config.toml edit applies from the next session.
const STEPS: &str = "To set it up, open /permissions, choose a mode that uses the sandbox (such \
     as \"Ask for approval\") and pick \"Set up default sandbox\" (it needs Administrator \
     permission); this session uses it from the next message. Or set `sandbox = \"elevated\"` \
     under `[windows]` in config.toml and start a new session with /new. Or a person can switch \
     to Permissive in /security (it applies after a restart).";

/// Whether `level` refuses agent commands and file tools here because no
/// Windows sandbox is selected.
pub(crate) fn applies(level: SecurityLevel, windows_sandbox_level: WindowsSandboxLevel) -> bool {
    applies_on(cfg!(windows), level, windows_sandbox_level)
}

fn applies_on(
    windows: bool,
    level: SecurityLevel,
    windows_sandbox_level: WindowsSandboxLevel,
) -> bool {
    windows
        && level != SecurityLevel::Permissive
        && windows_sandbox_level == WindowsSandboxLevel::Disabled
}

fn name(level: SecurityLevel) -> &'static str {
    match level {
        SecurityLevel::Permissive => "Permissive",
        SecurityLevel::Moderate => "Moderate",
        SecurityLevel::Aggressive => "Aggressive",
    }
}

/// The refusal of an agent command.
pub(crate) fn command_refusal(level: SecurityLevel) -> String {
    let level = name(level);
    format!(
        "Security level {level} refused this command: {level} runs agent commands only inside \
         the default Windows sandbox, which is not set up on this computer, so nothing ran. \
         {STEPS}"
    )
}

/// The refusal of a file tool's access.
pub(crate) fn file_refusal(level: SecurityLevel) -> String {
    let level = name(level);
    format!(
        "Security level {level} refused this file access: {level} lets file tools read and \
         write only inside the default Windows sandbox, which is not set up on this computer. \
         {STEPS}"
    )
}

/// Shown once when a session starts under such a level.
pub(crate) fn startup_notice(level: SecurityLevel) -> String {
    let level = name(level);
    format!(
        "Security level {level} needs the default Windows sandbox, which is not set up on this \
         computer, so agent commands and file tools are refused until it is. {STEPS}"
    )
}

/// The refusal of an agent command in `session`'s turn, if any.
pub(crate) fn session_command_refusal(
    session: &crate::session::session::Session,
    windows_sandbox_level: WindowsSandboxLevel,
) -> Option<String> {
    let level = session.services.model_client().live_level();
    applies(level, windows_sandbox_level).then(|| command_refusal(level))
}

/// The refusal of typing into a running command in `session`'s turn, if
/// any. #2000: a command started before the session's Windows sandbox was
/// set up (`started_with`) still runs without it, so typing into it stays
/// refused.
pub(crate) fn session_typing_refusal(
    session: &crate::session::session::Session,
    windows_sandbox_level: WindowsSandboxLevel,
    started_with: Option<WindowsSandboxLevel>,
) -> Option<String> {
    typing_refusal_on(
        cfg!(windows),
        session.services.model_client().live_level(),
        windows_sandbox_level,
        started_with,
    )
}

fn typing_refusal_on(
    windows: bool,
    level: SecurityLevel,
    windows_sandbox_level: WindowsSandboxLevel,
    started_with: Option<WindowsSandboxLevel>,
) -> Option<String> {
    if applies_on(windows, level, windows_sandbox_level) {
        return Some(command_refusal(level));
    }
    (started_with == Some(WindowsSandboxLevel::Disabled)
        && applies_on(windows, level, WindowsSandboxLevel::Disabled))
    .then(|| started_unsandboxed_refusal(level))
}

/// The refusal of typing into a command that started before the Windows
/// sandbox was set up.
fn started_unsandboxed_refusal(level: SecurityLevel) -> String {
    let level = name(level);
    format!(
        "Security level {level} refused typing into this command: it started before the \
         Windows sandbox was set up, so it runs without it. Start the command again; new \
         commands run inside the sandbox."
    )
}

/// The refusal a file tool's sandbox context carries (see
/// `FileSystemSandboxContext::unsandboxed_refusal`), if any.
pub(crate) fn session_file_refusal(
    session: &crate::session::session::Session,
    windows_sandbox_level: WindowsSandboxLevel,
) -> Option<String> {
    let level = session.services.model_client().live_level();
    applies(level, windows_sandbox_level).then(|| file_refusal(level))
}

/// What the inspector lists for a refused command or file access.
pub(crate) const INSPECTED: &str = "an agent command or file tool without the Windows sandbox";

/// What the inspector lists for refused typing into a running command.
pub(crate) const INSPECTED_TYPING: &str =
    "typing into a running command without the Windows sandbox";

/// The inspector's outcome for `level`.
pub(crate) fn inspected_outcome(level: SecurityLevel) -> &'static str {
    match level {
        SecurityLevel::Aggressive => "refused: Aggressive needs the default Windows sandbox",
        _ => "refused: Moderate needs the default Windows sandbox",
    }
}

#[cfg(test)]
#[path = "windows_unsandboxed_tests.rs"]
mod tests;
