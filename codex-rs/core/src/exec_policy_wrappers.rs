//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::ExecPolicyCommands;
use super::commands_for_exec_policy;
use crate::security::tainted_action::policy_shell_script;
use crate::security::tainted_action::policy_wrapper_commands;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;

/// #2531 (D553): how many shell scripts nested in one another
/// (`bash -lc "bash -c '…'"`, also through launchers such as
/// `sudo bash -c "env bash -c '…'"`) rules see into below each command they
/// are given. A script nested deeper asks for approval; one behind a
/// launcher or a respelled shell is still read too.
pub(crate) const MAX_SCRIPT_DEPTH: usize = 8;

/// The commands inside `command` when it is a shell script rules can read
/// (`bash -c 'git push'`); `None` for any other command, which parses as
/// itself.
fn script_commands(command: &[String]) -> Option<ExecPolicyCommands> {
    let parsed = commands_for_exec_policy(command);
    (parsed.commands.len() != 1 || parsed.commands[0] != command).then_some(parsed)
}

pub(super) fn has_wrappers(command: &[String]) -> bool {
    let wrappers = policy_wrapper_commands(command);
    wrappers.uncertain
        || wrappers.starts.len() > 1
        || wrappers.open.is_some()
        || policy_shell_script(command).is_some()
        // #2531: a script nested in the script; an amendment for its
        // literal argv cannot approve the commands inside.
        || script_commands(command).is_some()
        // #2546: nor those of a PowerShell script no trusted parser read.
        || powershell_script_unreadable(command)
}

/// #2546 (D553): whether `command` runs a PowerShell `-Command` script that
/// no trusted PowerShell could read (none found, or it could not run), so
/// rules cannot see its commands.
#[cfg(windows)]
fn powershell_script_unreadable(command: &[String]) -> bool {
    let respelled = crate::security::tainted_action::policy_powershell_script(command);
    codex_shell_command::powershell::read_powershell_command_script(
        respelled.as_deref().unwrap_or(command),
    ) == Some(codex_shell_command::powershell::PowershellScriptRead::Unreadable)
}

#[cfg(not(windows))]
fn powershell_script_unreadable(_command: &[String]) -> bool {
    false
}

pub(crate) fn check_with_wrappers(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
) -> Evaluation {
    check_with_wrappers_and_folded(policy, commands, fallback, options).0
}

/// D692: whether `matched` holds a rule match for `decision` other than the
/// case-folded ones in `folded` (which [`check_with_wrappers_and_folded`]
/// also put in `matched`). A decision only a differently cased rule makes is
/// approved and sandboxed as the fallback's would be, so folding only adds.
pub(crate) fn has_exact_rule_match(
    matched: &[RuleMatch],
    folded: &[RuleMatch],
    decision: Decision,
) -> bool {
    let count = |matches: &[RuleMatch]| {
        matches
            .iter()
            .filter(|rule_match| {
                matches!(rule_match, RuleMatch::PrefixRuleMatch { .. })
                    && rule_match.decision() == decision
            })
            .count()
    };
    count(matched) > count(folded)
}

/// [`check_with_wrappers`], also returning the case-folded rule matches
/// (D692, [`Policy::case_folded_matches`]) it added to the evaluation.
pub(crate) fn check_with_wrappers_and_folded(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
) -> (Evaluation, Vec<RuleMatch>) {
    check(
        policy, commands, /*read_own_scripts*/ true, fallback, options,
    )
}

/// [`check_with_wrappers_and_folded`] for an argv as it is executed, whose
/// own shell script (`/bin/zsh -lc '…'`) is not read: the escalation path
/// intercepts the script's commands as they run. Scripts behind a launcher
/// in it (`sudo bash -c '…'`) and scripts nested in those are still read.
#[cfg(unix)]
pub(crate) fn check_executed_with_wrappers_and_folded(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
) -> (Evaluation, Vec<RuleMatch>) {
    check(
        policy, commands, /*read_own_scripts*/ false, fallback, options,
    )
}

fn check(
    policy: &Policy,
    commands: &[Vec<String>],
    read_own_scripts: bool,
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
) -> (Evaluation, Vec<RuleMatch>) {
    let mut matches = Vec::new();
    let mut folded = Vec::new();
    // Inner shell scripts can themselves contain wrapped commands. Use a work
    // list rather than recursion; preserve every outer match along the way.
    // Each entry carries how many scripts it is nested in.
    let mut pending: Vec<(Vec<String>, usize)> = Vec::new();
    let mut next = commands.iter();
    loop {
        let owned = pending.pop();
        let (command, depth) = match &owned {
            Some((command, depth)) => (command, *depth),
            None => match next.next() {
                Some(command) => (command, 0),
                None => break,
            },
        };
        let wrappers = policy_wrapper_commands(command);
        // Keep only the strictest fallback for this chain. Copying the entire
        // remaining argv at every layer would grow quadratically with nesting.
        let mut heuristic = wrappers
            .uncertain
            .then_some((command.as_slice(), Decision::Prompt));
        // Every rule counts where no word is known; scan them once.
        let mut any_rule = None;
        for &start in &wrappers.starts {
            let candidate = &command[start..];
            let found = policy.matches_for_command_with_options(
                candidate, /*heuristics_fallback*/ None, options,
            );
            if found.is_empty() {
                let decision = fallback(candidate);
                if heuristic.is_none_or(|(_, previous)| decision > previous) {
                    heuristic = Some((candidate, decision));
                }
            }
            matches.extend(found);
            // D692: kept beside the fallback, so this only adds checks.
            let case_folded = policy.case_folded_matches(candidate, options);
            matches.extend(case_folded.iter().cloned());
            folded.extend(case_folded);
            // #2455 (D677): xargs adds words read from stdin. Ask whenever a
            // prompt or forbidden rule could match once they arrive; a rule
            // the fixed argv already matches applies above. Allow rules only
            // approve, so they need no prompt.
            if let Some(open) = &wrappers.open
                && heuristic.is_none_or(|(_, previous)| previous < Decision::Prompt)
                && {
                    let known = &command[start..open.known_end.max(start)];
                    let strictest = if known.is_empty() {
                        *any_rule.get_or_insert_with(|| {
                            policy.strictest_rule_matching_more_args(known, options)
                        })
                    } else {
                        policy.strictest_rule_matching_more_args(known, options)
                    };
                    strictest.is_some_and(|decision| decision > Decision::Allow)
                }
            {
                heuristic = Some((candidate, Decision::Prompt));
            }
        }
        if let Some((command, decision)) = heuristic {
            matches.push(RuleMatch::HeuristicsRuleMatch {
                command: command.to_vec(),
                decision,
            });
        }
        if let Some(&start) = wrappers.starts.last() {
            let inner = &command[start..];
            // #2510 (D688): a shell spelled as this platform runs it
            // (`BASH -lc`, `bash.exe -c`) unwraps like `bash -lc`. The
            // literal argv above keeps its own match and fallback.
            let respelled = policy_shell_script(inner);
            let parsed_as = respelled.as_deref().unwrap_or(inner);
            // Scripts behind a launcher or a respelled shell are read at any
            // depth: each is a strict substring of the one around it.
            let always_read = start > 0 || respelled.is_some();
            // #2531: a script with no launcher before it too. Its commands
            // come from a script already (`bash -lc "bash -c 'git push'"`
            // gives `bash -c 'git push'`), or are those the caller gives.
            if (always_read || depth > 0 || read_own_scripts)
                && let Some(parsed) = script_commands(parsed_as)
            {
                if parsed.used_complex_parsing || depth >= MAX_SCRIPT_DEPTH {
                    matches.push(RuleMatch::HeuristicsRuleMatch {
                        command: inner.to_vec(),
                        decision: Decision::Prompt,
                    });
                }
                // Past the bound, still read what is always read, so a
                // command a rule refuses stays refused, not only asked.
                if depth < MAX_SCRIPT_DEPTH || always_read {
                    pending.extend(
                        parsed
                            .commands
                            .into_iter()
                            .map(|command| (command, depth + 1)),
                    );
                }
            } else if (always_read || depth > 0 || read_own_scripts)
                && policy
                    .strictest_rule_matching_more_args(&[], options)
                    .is_some_and(|decision| decision > Decision::Allow)
                && powershell_script_unreadable(inner)
            {
                // #2546 (D553): a PowerShell script no trusted parser read.
                // Any prompt or forbidden rule could match a command in it;
                // the literal argv above keeps its own match and fallback.
                matches.push(RuleMatch::HeuristicsRuleMatch {
                    command: inner.to_vec(),
                    decision: Decision::Prompt,
                });
            }
        }
    }
    let evaluation = Evaluation {
        decision: matches
            .iter()
            .map(RuleMatch::decision)
            .max()
            .unwrap_or(Decision::Prompt),
        matched_rules: matches,
    };
    (evaluation, folded)
}
