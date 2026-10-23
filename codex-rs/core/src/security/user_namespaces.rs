//! #826 (Travis, 2026-10-15) and its #876 follow-up: on Linux, agent
//! commands under Moderate and Aggressive (also where only the launch
//! enforces Aggressive) cannot create user namespaces. [`rule_applies`] is
//! the orchestrator's decision; the rest names the rule where a command that
//! needed one failed (to the person, to the model and in the inspector's
//! recent denials) and before a person approves a command that needs one.
//!
//! Explanations only: nothing here changes what runs or lifts the rule,
//! but for D27 (a) (Travis, 2026-10-19, security sign-off): under Moderate,
//! a person's fresh approval of a command's request to run outside the
//! sandbox before untrusted content ([`allows_one_run`]) also lets that one
//! run create user namespaces, with [`one_run_warning`] in the prompt.
//! D48 (c) (Travis, 2026-10-20, security sign-off): after untrusted content,
//! the same, through a fresh prompt with the same warning, but only for a
//! command whose program is a known tool ([`known_tool::known_tool`]).
//! Never remembered; Aggressive and an Aggressive launch keep the rule.

use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::Mutex;

use codex_protocol::ThreadId;
use codex_protocol::permissions::FileSystemSandboxKind;
use codex_sandboxing::user_namespaces;
use codex_security_policy::SecurityLevel;

use crate::security::tainted_action::PostTaintState;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;

#[path = "user_namespaces_known_tool.rs"]
mod known_tool;

pub(crate) use known_tool::Lookup;
pub(crate) use known_tool::known_tool;

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

/// The rule that denies user namespaces to this turn's agent commands, if
/// one does: the orchestrator's decision for a profile that takes Corbanu's
/// sandbox rules (not an external or disabled sandbox), and under Moderate
/// whether a person may still allow it for one command ([`allows_one_run`]).
fn turn_rule(session: &Session, turn: &TurnContext) -> Option<user_namespaces::Rule> {
    turn_rule_at(session, turn).map(|(rule, _)| rule)
}

/// [`turn_rule`], with the session's taint generation it was decided at.
fn turn_rule_at(session: &Session, turn: &TurnContext) -> Option<(user_namespaces::Rule, u64)> {
    if !cfg!(target_os = "linux")
        || turn.config.permissions.file_system_sandbox_policy().kind
            != FileSystemSandboxKind::Restricted
    {
        return None;
    }
    let state = session
        .services
        .model_client()
        .post_taint_state()
        .filter(crate::security::tainted_action::PostTaintState::protected_paths_apply);
    let generation = state.as_ref().map_or(0, |state| state.taint_generation);
    let rule = match denying_level(
        state.as_ref().map(|state| state.level),
        turn.config.permissions.launch_enforces_aggressive(),
    )? {
        "Aggressive" => user_namespaces::Rule::Aggressive,
        _ => match state {
            Some(state) if one_command_way(&state) => user_namespaces::Rule::ModerateOneCommand,
            Some(state) if !kill_switch_on(&state) && state.moderate_bound() => {
                user_namespaces::Rule::ModerateAfterUntrusted
            }
            _ => user_namespaces::Rule::ModerateNotInForce,
        },
    };
    Some((rule, generation))
}

/// Under Moderate, whether a person's approval of a command's request to
/// run outside the sandbox may still let it create user namespaces: Moderate
/// configured and live, no untrusted content in the session (the state under
/// which that approval lifts Moderate's protected-file rules, #239), and the
/// kill switch off.
fn one_command_way(state: &PostTaintState) -> bool {
    still_offered(state, /*warned_at*/ 0)
}

/// Whether a person warned at taint generation `warned_at` may still allow
/// it: Moderate configured and live, no untrusted content since, and the
/// kill switch off.
fn still_offered(state: &PostTaintState, warned_at: u64) -> bool {
    state.human_approval_lifts_rules(Some(warned_at)) && !kill_switch_on(state)
}

fn kill_switch_on(state: &PostTaintState) -> bool {
    matches!(
        state.policy,
        crate::security::tainted_action::PolicyBinding::Bound {
            kill_switch_active: true,
            ..
        }
    )
}

/// D27 (a), D48 (c): whether a person's approval of this command lets its
/// run create user namespaces: on Linux, `approved_now` (a person approved
/// this command's request to run outside the sandbox when asked about it
/// now: not a hook, the automatic reviewer, the approval cache or a saved
/// rule) in a prompt that showed [`one_run_warning`] at taint generation
/// `warned_at`, Core's protected-path level (`level`) is Moderate, the
/// launch does not enforce Aggressive, and still, when it is about to run,
/// Moderate is configured and live, the kill switch is off and no untrusted
/// content has arrived since the warning.
pub(crate) fn allows_one_run(
    session: &Session,
    level: Option<SecurityLevel>,
    launch_enforces_aggressive: bool,
    approved_now: bool,
    warned_at: Option<u64>,
) -> bool {
    cfg!(target_os = "linux")
        && approved_now
        && level == Some(SecurityLevel::Moderate)
        && !launch_enforces_aggressive
        && warned_at.is_some_and(|warned_at| {
            session
                .services
                .model_client()
                .post_taint_state()
                .is_some_and(|state| still_offered(&state, warned_at))
        })
}

/// The warning for Moderate's prompt on a request to run `command` outside
/// the sandbox, where approving it lets that run create user namespaces
/// ([`allows_one_run`]), with the taint generation it is shown at: before
/// untrusted content for any command (D27 (a)), after it only where
/// `runs_known_tool` says the command's program is a known tool (D48 (c),
/// [`known_tool`]; asked only then). The orchestrator allows it only where
/// this was shown.
pub(crate) fn one_run_warning(
    session: &Session,
    turn: &TurnContext,
    runs_known_tool: &dyn Fn() -> bool,
) -> Option<(String, u64)> {
    match turn_rule_at(session, turn)? {
        (user_namespaces::Rule::ModerateOneCommand, generation) => Some(generation),
        (user_namespaces::Rule::ModerateAfterUntrusted, generation) if runs_known_tool() => {
            Some(generation)
        }
        _ => None,
    }
    .map(|generation| (user_namespaces::moderate_one_run_warning(), generation))
}

/// The commands (thread, turn and call) whose run a person let create user
/// namespaces ([`allows_one_run`]), so that a refusal reported there (by the
/// host, not the rule) is not put down to the rule. Taken when the command's
/// explanation is decided; a few recent ones are enough.
static ALLOWED_RUNS: LazyLock<Mutex<VecDeque<AllowedRun>>> = LazyLock::new(Default::default);
const ALLOWED_RUNS_KEPT: usize = 64;

#[derive(Debug, PartialEq, Eq)]
struct AllowedRun {
    thread: ThreadId,
    turn: String,
    call: String,
}

/// Notes that the run of `call_id` in `turn_id` may create user namespaces.
pub(crate) fn note_allowed_run(thread: ThreadId, turn_id: &str, call_id: &str) {
    let mut runs = ALLOWED_RUNS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if runs.len() >= ALLOWED_RUNS_KEPT {
        runs.pop_front();
    }
    runs.push_back(AllowedRun {
        thread,
        turn: turn_id.to_string(),
        call: call_id.to_string(),
    });
}

/// Whether the run was noted by [`note_allowed_run`]; forgets it.
fn take_allowed_run(thread: ThreadId, turn_id: &str, call_id: &str) -> bool {
    let mut runs = ALLOWED_RUNS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match runs
        .iter()
        .position(|run| run.thread == thread && run.turn == turn_id && run.call == call_id)
    {
        Some(at) => runs.remove(at).is_some(),
        None => false,
    }
}

/// The explanation for a command that just finished (`call_id`) where it
/// failed (`exit_code` known and not zero), its output reports a refused
/// user namespace and the rule applied to its run; listed under the
/// inspector's recent denials too (#622).
pub(crate) fn denial_explanation(
    session: &Session,
    turn: &TurnContext,
    call_id: &str,
    exit_code: Option<i32>,
    output: &str,
) -> Option<String> {
    let allowed = take_allowed_run(session.thread_id(), &turn.sub_id, call_id);
    let rule = turn_rule(session, turn).filter(|_| !allowed);
    explained(rule, exit_code, output).inspect(|_| {
        super::inspection::record_command_denial(
            session.thread_id(),
            INSPECTED,
            match rule {
                Some(
                    user_namespaces::Rule::ModerateOneCommand
                    | user_namespaces::Rule::ModerateAfterUntrusted,
                ) => {
                    "reported refused by the command; Moderate blocks it on Linux, but for one command whose request to run outside the sandbox a person approves when asked: before untrusted content any command, after it only one that runs unshare, bwrap, podman, buildah, flatpak or nix"
                }
                Some(user_namespaces::Rule::ModerateNotInForce) => {
                    "reported refused by the command; Moderate blocks it on Linux, and with the kill switch on (or Moderate not fully in force) no approval lifts it"
                }
                _ => {
                    "reported refused by the command; Aggressive blocks it on Linux, and no approval or grant lifts it"
                }
            },
        );
    })
}

/// What the inspector's recent denials say was refused.
pub(crate) const INSPECTED: &str = "creating a user namespace";

fn explained(
    rule: Option<user_namespaces::Rule>,
    exit_code: Option<i32>,
    output: &str,
) -> Option<String> {
    let rule = rule?;
    (matches!(exit_code, Some(code) if code != 0)
        && user_namespaces::reports_user_namespace_denial(output))
    .then(|| user_namespaces::denial_explanation(rule))
}

/// The approval prompt's note for `command` when it runs a program that
/// needs a user namespace (`unshare`, `bwrap`, rootless Podman, …) and the
/// rule applies.
pub(crate) fn approval_note(
    session: &Session,
    turn: &TurnContext,
    command: &[String],
) -> Option<String> {
    let rule = turn_rule(session, turn)?;
    super::protected_reads::runs_user_namespace_program(command)
        .then(|| user_namespaces::approval_note(rule))
}

#[cfg(test)]
#[path = "user_namespaces_tests.rs"]
mod tests;
