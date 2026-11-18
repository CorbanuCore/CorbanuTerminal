//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::commands_for_exec_policy_for_platform;
use crate::security::tainted_action::policy_shell_script;
use crate::security::tainted_action::policy_wrapper_commands;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;
use codex_shell_command::bash::parse_shell_lc_single_command_prefix;
use codex_shell_command::is_dangerous_command::DangerousCommandPlatform;

/// #2531 (D553): how many shell scripts nested in one another
/// (`bash -lc "bash -c '…'"`, also through launchers such as
/// `sudo bash -c "env bash -c '…'"`) rules see into below each command they
/// are given. A script nested deeper asks for approval; one behind a
/// launcher or a respelled shell is still read too.
pub(crate) const MAX_SCRIPT_DEPTH: usize = 8;

/// The commands inside `command` when it is a shell script rules can read
/// (`bash -c 'git push'`), and whether only the heredoc prefix parser read
/// it (main's `used_complex_parsing`); `None` for any other command, which
/// parses as itself. Sync: split for `platform`, as the outer command was;
/// upstream's split keeps a heredoc script whole, so its first command is
/// taken here as main's split took it.
fn script_commands(
    command: &[String],
    platform: DangerousCommandPlatform,
) -> Option<(Vec<Vec<String>>, bool)> {
    let parsed = commands_for_exec_policy_for_platform(command, platform);
    if parsed.commands.len() != 1 || parsed.commands[0] != command {
        return Some((parsed.commands, false));
    }
    parse_shell_lc_single_command_prefix(command).map(|single| (vec![single], true))
}

/// `read_own_scripts` as for [`check`]: whether `command` came from a split
/// (sync), so a shell script it is itself counts like main's nested one.
pub(super) fn has_wrappers(
    command: &[String],
    platform: DangerousCommandPlatform,
    read_own_scripts: bool,
) -> bool {
    let wrappers = policy_wrapper_commands(command);
    wrappers.uncertain
        || wrappers.approval_only
        || wrappers.starts.len() > 1
        || wrappers.open.is_some()
        || policy_shell_script(command).is_some()
        // #2531: a script nested in the script; an amendment for its
        // literal argv cannot approve the commands inside.
        || (read_own_scripts && script_commands(command, platform).is_some())
}

/// D677/D686 (#2506), D691 and #2519: what a command gets for a rule that
/// could match what it runs (words an open `xargs` adds, or a wrapper whose
/// command cannot be read). Allow rules only approve, so they need no
/// prompt; a prompt-user rule keeps its fresh human decision, and so does a
/// forbidden one (D691): the rule may not apply, so it is not refused.
pub(super) fn possible_rule_decision(rule: Decision) -> Option<Decision> {
    match rule {
        Decision::Allow => None,
        Decision::Prompt => Some(Decision::Prompt),
        Decision::PromptUser | Decision::Forbidden => Some(Decision::PromptUser),
    }
}

/// Keep the stricter of the current heuristic and `decision` for `command`.
fn raise<'a>(
    heuristic: &mut Option<(&'a [String], Decision)>,
    command: &'a [String],
    decision: Option<Decision>,
) {
    if let Some(decision) = decision
        && heuristic.is_none_or(|(_, previous)| decision > previous)
    {
        *heuristic = Some((command, decision));
    }
}

/// D692: whether `matched` holds a rule match for `decision` other than the
/// case-folded ones in `folded` (which [`check`]
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

/// Apply `commands`' rules and fallback decisions through every wrapper
/// layer, also returning the case-folded rule matches (D692,
/// [`Policy::case_folded_matches`]) it added to the evaluation. `platform` is
/// the one the outer command was parsed for (sync: it may be a remote
/// executor's), so scripts behind a wrapper are split the same way. Sync:
/// main's `check_with_wrappers` (this without the folded matches) has no
/// caller left here; the approval path calls [`check`] directly.
pub(crate) fn check_with_wrappers_and_folded(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
) -> (Evaluation, Vec<RuleMatch>) {
    check(
        policy, commands, /*read_own_scripts*/ true, fallback, options, platform,
    )
}

/// [`check_with_wrappers_and_folded`], reading each given command's own
/// shell script only with `read_own_scripts`. Sync: the approval path passes
/// `false` for a command line it could not split, whose own heredoc script
/// (`bash -lc "python3 <<'PY' …"`) upstream keeps as shell argv and
/// `apply_heredoc_restrictive_matches` reads, as main's caller split it
/// first. Scripts behind a launcher or a respelled shell in it, and scripts
/// nested in those, are still read. Main uses the same switch for the
/// zsh-fork escalation path, which the sync branch no longer has.
pub(super) fn check(
    policy: &Policy,
    commands: &[Vec<String>],
    read_own_scripts: bool,
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
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
        let mut heuristic = (wrappers.uncertain || wrappers.approval_only)
            .then_some((command.as_slice(), Decision::Prompt));
        // #2455 (D677, D686): xargs adds words read from stdin. Ask whenever
        // a prompt, prompt-user or forbidden rule could match once they
        // arrive; a rule the fixed argv already matches applies below. Each
        // rule's level is mapped before taking the strictest (#2506).
        // Every rule counts where no word is known; scan them once.
        let mut any_rule = None;
        let mut could_complete = |known: &[String]| {
            if known.is_empty() {
                *any_rule.get_or_insert_with(|| {
                    policy.strictest_rule_matching_more_args_by(
                        known,
                        options,
                        possible_rule_decision,
                    )
                })
            } else {
                policy.strictest_rule_matching_more_args_by(known, options, possible_rule_decision)
            }
        };
        // #2506, #2519: an uncertain chain (a shell string as with `su -c`
        // or `watch`, unknown options, under xargs a launcher with no
        // command or a shell) runs a command no rule has seen: every rule
        // counts. Where the command is read, its rules apply as written.
        if wrappers.uncertain {
            raise(&mut heuristic, command, could_complete(&[]));
        }
        for &start in &wrappers.starts {
            let candidate = &command[start..];
            let found = policy.matches_for_command_with_options(
                candidate, /*heuristics_fallback*/ None, options,
            );
            if found.is_empty() {
                raise(&mut heuristic, candidate, Some(fallback(candidate)));
            }
            matches.extend(found);
            // D692: kept beside the fallback, so this only adds checks.
            let case_folded = policy.case_folded_matches(candidate, options);
            matches.extend(case_folded.iter().cloned());
            folded.extend(case_folded);
            if let Some(open) = &wrappers.open {
                let known = &command[start..open.known_end.max(start)];
                raise(&mut heuristic, candidate, could_complete(known));
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
                && let Some((inner_commands, used_complex_parsing)) =
                    script_commands(parsed_as, platform)
            {
                if used_complex_parsing || depth >= MAX_SCRIPT_DEPTH {
                    matches.push(RuleMatch::HeuristicsRuleMatch {
                        command: inner.to_vec(),
                        decision: Decision::Prompt,
                    });
                }
                // Past the bound, still read what is always read, so a
                // command a rule refuses stays refused, not only asked.
                if depth < MAX_SCRIPT_DEPTH || always_read {
                    pending.extend(
                        inner_commands
                            .into_iter()
                            .map(|command| (command, depth + 1)),
                    );
                }
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
