//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::commands_for_exec_policy_for_platform;
use crate::security::tainted_action::policy_wrapper_commands;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;
use codex_shell_command::bash::parse_shell_lc_single_command_prefix;
use codex_shell_command::is_dangerous_command::DangerousCommandPlatform;

pub(super) fn has_wrappers(command: &[String]) -> bool {
    let wrappers = policy_wrapper_commands(command);
    wrappers.uncertain || wrappers.starts.len() > 1
}

/// `platform` is the one the outer command was parsed for (sync: it may be a
/// remote executor's), so scripts behind a wrapper are split the same way.
pub(crate) fn check_with_wrappers(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
) -> Evaluation {
    let mut matches = Vec::new();
    // Inner shell scripts can themselves contain wrapped commands. Use a work
    // list rather than recursion; preserve every outer match along the way.
    let mut pending = Vec::new();
    let mut next = commands.iter();
    loop {
        let owned = pending.pop();
        let command = match owned.as_ref().or_else(|| next.next()) {
            Some(command) => command,
            None => break,
        };
        let wrappers = policy_wrapper_commands(command);
        // Keep only the strictest fallback for this chain. Copying the entire
        // remaining argv at every layer would grow quadratically with nesting.
        let mut heuristic = wrappers
            .uncertain
            .then_some((command.as_slice(), Decision::Prompt));
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
        }
        if let Some((command, decision)) = heuristic {
            matches.push(RuleMatch::HeuristicsRuleMatch {
                command: command.to_vec(),
                decision,
            });
        }
        if let Some(&start) = wrappers.starts.last()
            && start > 0
        {
            let inner = &command[start..];
            let parsed = commands_for_exec_policy_for_platform(inner, platform);
            if parsed.commands.len() != 1 || parsed.commands[0] != inner {
                pending.extend(parsed.commands);
            } else if let Some(single) = parse_shell_lc_single_command_prefix(inner) {
                // Sync: upstream's parser no longer reduces a heredoc script to
                // its first command (main's `used_complex_parsing`); do it here
                // so a wrapped heredoc keeps main's prompt and inner rules.
                matches.push(RuleMatch::HeuristicsRuleMatch {
                    command: inner.to_vec(),
                    decision: Decision::Prompt,
                });
                pending.push(single);
            }
        }
    }
    Evaluation {
        decision: matches
            .iter()
            .map(RuleMatch::decision)
            .max()
            .unwrap_or(Decision::Prompt),
        matched_rules: matches,
    }
}
