//! #826 (Travis, 2026-10-15) and its #876 follow-up: on Linux, agent
//! commands under Moderate and Aggressive (also where only the launch
//! enforces Aggressive) cannot create user namespaces. [`rule_applies`] is
//! the orchestrator's decision; the rest names the rule where a command that
//! needed one failed (to the person, to the model and in the inspector's
//! recent denials) and before a person approves a command that needs one.
//!
//! Explanations only: nothing here changes what runs or lifts the rule.

use codex_protocol::permissions::FileSystemSandboxKind;
use codex_sandboxing::user_namespaces;
use codex_security_policy::SecurityLevel;

use crate::session::session::Session;
use crate::session::turn_context::TurnContext;

/// Whether agent commands get the user-namespace deny entry: on Linux,
/// where Core's protected-path level is Moderate or Aggressive, or the
/// launch enforces Aggressive. No approval or grant changes it.
pub(crate) fn rule_applies(level: Option<SecurityLevel>, launch_enforces_aggressive: bool) -> bool {
    cfg!(target_os = "linux") && denying_level(level, launch_enforces_aggressive).is_some()
}

/// The level named for the rule, on any platform (see [`rule_applies`]).
fn denying_level(
    level: Option<SecurityLevel>,
    launch_enforces_aggressive: bool,
) -> Option<&'static str> {
    match level {
        Some(SecurityLevel::Aggressive) => Some("Aggressive"),
        _ if launch_enforces_aggressive => Some("Aggressive"),
        Some(SecurityLevel::Moderate) => Some("Moderate"),
        Some(SecurityLevel::Permissive) | None => None,
    }
}

/// The level whose rule denies user namespaces to this turn's agent
/// commands, if it does: the orchestrator's decision for a profile that
/// takes Corbanu's sandbox rules (not an external or disabled sandbox).
fn turn_level(session: &Session, turn: &TurnContext) -> Option<&'static str> {
    if !cfg!(target_os = "linux")
        || turn.config.permissions.file_system_sandbox_policy().kind
            != FileSystemSandboxKind::Restricted
    {
        return None;
    }
    denying_level(
        super::protected_additional::protected_level(session),
        turn.config.permissions.launch_enforces_aggressive(),
    )
}

/// The explanation for a command that just failed (`exit_code` known and
/// not zero) where its output reports a refused user namespace and the rule
/// applies; listed under the inspector's recent denials too (#622).
pub(crate) fn denial_explanation(
    session: &Session,
    turn: &TurnContext,
    exit_code: Option<i32>,
    output: &str,
) -> Option<String> {
    explained(turn_level(session, turn), exit_code, output).inspect(|_| {
        super::inspection::record_command_denial(
            session.thread_id(),
            INSPECTED,
            "reported refused by the command; Moderate and Aggressive block it on Linux, and no approval or grant lifts it",
        );
    })
}

/// What the inspector's recent denials say was refused.
pub(crate) const INSPECTED: &str = "creating a user namespace";

fn explained(level: Option<&'static str>, exit_code: Option<i32>, output: &str) -> Option<String> {
    let level = level?;
    (matches!(exit_code, Some(code) if code != 0)
        && user_namespaces::reports_user_namespace_denial(output))
    .then(|| user_namespaces::denial_explanation(level))
}

/// The approval prompt's note for `command` when it runs a program that
/// needs a user namespace (`unshare`, `bwrap`, rootless Podman, …) and the
/// rule applies.
pub(crate) fn approval_note(
    session: &Session,
    turn: &TurnContext,
    command: &[String],
) -> Option<String> {
    let level = turn_level(session, turn)?;
    super::protected_reads::runs_user_namespace_program(command)
        .then(|| user_namespaces::approval_note(level))
}

#[cfg(test)]
#[path = "user_namespaces_tests.rs"]
mod tests;
