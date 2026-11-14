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

/// #2050 (D510): the same rule where the work actually runs. A remote
/// environment's executor reports its own platform sandbox; under Moderate or
/// Aggressive, work for a remote Windows machine without a Windows sandbox for
/// this session's `[windows] sandbox` (none, or the elevated one not set up
/// there) is refused, and so is work for an executor that cannot say (an
/// older one that does not clearly run on Linux or macOS, or one that does
/// not answer). Local environments keep the check above, and so does a
/// Windows client sending work to a remote Linux machine (#2049).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteGap {
    /// This computer asks the remote Windows machine for no Windows sandbox
    /// (no `[windows] sandbox` setting here); the machine uses that setting.
    WindowsSandboxNotSelected,
    /// The default sandbox is asked for but not set up on that machine.
    WindowsSandboxNotSetUp,
    Unreported,
}

/// How to set the sandbox up for a remote Windows machine.
const REMOTE_STEPS: &str = "To set it up, run Corbanu Terminal on that machine, open \
     /permissions, choose a mode that uses the sandbox (such as \"Ask for approval\") and pick \
     \"Set up default sandbox\" (it needs Administrator permission there), using the same \
     Corbanu home as its exec-server; then set `sandbox = \"elevated\"` under `[windows]` in \
     config.toml on this computer and start a new session with /new.";

/// How to ask a remote Windows machine for its sandbox.
const REMOTE_SELECT_STEPS: &str = "Set `sandbox = \"elevated\"` under `[windows]` in \
     config.toml on this computer and start a new session with /new. The default sandbox must \
     also be set up on that machine: if it isn't yet, run Corbanu Terminal there, open \
     /permissions, choose a mode that uses the sandbox (such as \"Ask for approval\") and pick \
     \"Set up default sandbox\" (it needs Administrator permission there), using the same \
     Corbanu home as its exec-server.";

/// How to get a report from an executor that gives none.
const UNREPORTED_STEPS: &str = "Update Corbanu Terminal on that machine to this version so its \
     exec-server reports its sandbox, and check that it is reachable. If it runs on Windows, \
     it also needs the default Windows sandbox set up there, with `sandbox = \"elevated\"` \
     under `[windows]` in config.toml on this computer.";

/// The way out of a protected level for remote work.
const REMOTE_PERMISSIVE: &str =
    "Or a person can switch to Permissive in /security (it applies after a restart).";

/// The gap of `environment` for work sent with `windows_sandbox_level`, if
/// it is remote and does not confine that work.
pub(crate) async fn remote_gap(
    environment: &codex_exec_server::Environment,
    windows_sandbox_level: WindowsSandboxLevel,
) -> Option<RemoteGap> {
    if !environment.is_remote() {
        return None;
    }
    match environment.info().await {
        Ok(info) => match info.host_sandbox_verdict(windows_sandbox_level) {
            codex_exec_server::HostSandboxVerdict::Confined => None,
            codex_exec_server::HostSandboxVerdict::WindowsWithoutSandbox => {
                Some(match windows_sandbox_level {
                    WindowsSandboxLevel::Disabled => RemoteGap::WindowsSandboxNotSelected,
                    _ => RemoteGap::WindowsSandboxNotSetUp,
                })
            }
            codex_exec_server::HostSandboxVerdict::Unreported => Some(RemoteGap::Unreported),
        },
        Err(err) => {
            tracing::warn!("remote environment did not report its sandbox: {err}");
            Some(RemoteGap::Unreported)
        }
    }
}

fn remote_reason(environment_id: &str, gap: RemoteGap) -> (String, &'static str) {
    match gap {
        RemoteGap::WindowsSandboxNotSelected => (
            format!(
                "a Windows sandbox, and this computer asks the remote Windows machine of \
                 environment `{environment_id}` for none (no `[windows] sandbox` setting here)"
            ),
            REMOTE_SELECT_STEPS,
        ),
        RemoteGap::WindowsSandboxNotSetUp => (
            format!(
                "the default Windows sandbox, which is not set up on the remote Windows machine \
                 of environment `{environment_id}`"
            ),
            REMOTE_STEPS,
        ),
        RemoteGap::Unreported => (
            format!(
                "a sandbox, and the remote machine of environment `{environment_id}` did not \
                 report one (its exec-server may be older than this version)"
            ),
            UNREPORTED_STEPS,
        ),
    }
}

/// The refusal of an agent command for a remote environment.
pub(crate) fn remote_command_refusal(
    level: SecurityLevel,
    environment_id: &str,
    gap: RemoteGap,
) -> String {
    let level = name(level);
    let (reason, steps) = remote_reason(environment_id, gap);
    format!(
        "Security level {level} refused this command: {level} runs agent commands only inside \
         {reason}, so nothing ran. {steps} {REMOTE_PERMISSIVE}"
    )
}

/// The refusal of typing into a running command of a remote environment.
pub(crate) fn remote_typing_refusal(
    level: SecurityLevel,
    environment_id: &str,
    gap: RemoteGap,
) -> String {
    let level = name(level);
    let (reason, steps) = remote_reason(environment_id, gap);
    format!(
        "Security level {level} refused typing into this command: {level} runs agent commands \
         only inside {reason}. {steps} {REMOTE_PERMISSIVE}"
    )
}

/// The refusal a file tool's sandbox context carries for a remote
/// environment. Its executor's report is checked when the file tool runs
/// (`RemoteFileSystem`), so this names both gaps.
pub(crate) fn remote_file_refusal(level: SecurityLevel, environment_id: &str) -> String {
    let level = name(level);
    format!(
        "Security level {level} refused this file access: {level} lets file tools read and \
         write only inside a sandbox, and the remote machine of environment `{environment_id}` \
         has none for this session (a Windows machine without the default Windows sandbox set \
         up, or an exec-server too old to report one). {REMOTE_STEPS} {UNREPORTED_STEPS} \
         {REMOTE_PERMISSIVE}"
    )
}

/// The refusal of an agent command for `environment` in `session`'s turn,
/// if it is remote and does not confine it.
pub(crate) async fn session_remote_command_refusal(
    session: &crate::session::session::Session,
    environment: &crate::session::turn_context::TurnEnvironment,
    windows_sandbox_level: WindowsSandboxLevel,
) -> Option<String> {
    let level = session.services.model_client().live_level();
    if level == SecurityLevel::Permissive {
        return None;
    }
    let gap = remote_gap(&environment.environment, windows_sandbox_level).await?;
    Some(remote_command_refusal(
        level,
        &environment.selection.environment_id,
        gap,
    ))
}

/// The refusal of typing into a process of a remote environment, judged by
/// the Windows sandbox the process started with.
pub(crate) async fn session_remote_typing_refusal(
    session: &crate::session::session::Session,
    environment_id: &str,
    environment: &codex_exec_server::Environment,
    started_with: WindowsSandboxLevel,
) -> Option<String> {
    let level = session.services.model_client().live_level();
    if level == SecurityLevel::Permissive {
        return None;
    }
    let gap = remote_gap(environment, started_with).await?;
    Some(remote_typing_refusal(level, environment_id, gap))
}

/// The refusal a file tool's sandbox context carries (see
/// `FileSystemSandboxContext::unsandboxed_refusal`) for `environment` under
/// `level`, if any: the local check, and for a remote environment, any
/// protected level (judged against the executor's report when the file tool
/// runs).
pub(crate) fn environment_file_refusal(
    level: SecurityLevel,
    windows_sandbox_level: WindowsSandboxLevel,
    environment: &crate::session::turn_context::TurnEnvironment,
) -> Option<String> {
    // A Windows client without its own sandbox keeps #1976's refusal and
    // wording for every environment (#2049).
    if applies(level, windows_sandbox_level) {
        return Some(file_refusal(level));
    }
    (environment.environment.is_remote() && level != SecurityLevel::Permissive)
        .then(|| remote_file_refusal(level, &environment.selection.environment_id))
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
