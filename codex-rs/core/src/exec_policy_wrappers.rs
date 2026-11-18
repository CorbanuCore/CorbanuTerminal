//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::commands_for_exec_policy_for_platform;
use crate::security::tainted_action::policy_posix_shell_script;
use crate::security::tainted_action::policy_runs_shell;
use crate::security::tainted_action::policy_wrapper_commands;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;
use codex_shell_command::bash::ScriptCommands;
use codex_shell_command::bash::parse_shell_lc_single_command_prefix;
use codex_shell_command::bash::shell_lc_script_commands;
use codex_shell_command::is_dangerous_command::DangerousCommandPlatform;
use codex_shell_command::powershell::UnreadPowershell;
use codex_shell_command::powershell::unread_powershell_command_line;

pub(super) fn has_wrappers(command: &[String]) -> bool {
    let wrappers = policy_wrapper_commands(command);
    wrappers.uncertain
        || wrappers.approval_only
        || wrappers.starts.len() > 1
        || wrappers.open.is_some()
        // #2568: an amendment for a PowerShell command line whose script
        // rules cannot read cannot approve the commands inside.
        || unread_powershell_command_line(command).is_some()
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

/// `platform` is the one the outer command was parsed for (sync: it may be a
/// remote executor's), so scripts behind a wrapper are split the same way.
pub(crate) fn check_with_wrappers(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
) -> Evaluation {
    check(
        policy, commands, fallback, options, platform, /*read_own_scripts*/ false,
    )
}

/// #2543 (D553, D688): the prompt and refusal matches for `command`'s own
/// shell script when rules cannot read it as plain commands (redirections,
/// substitutions, words built at run time, a POSIX shell the parser does not
/// know, a script it does not read): the literal argv keeps its own match
/// and fallback, and this only adds checks. Empty for any other command.
pub(crate) fn unread_script_matches(
    policy: &Policy,
    command: &[String],
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
) -> Vec<RuleMatch> {
    let reading = script_reading(command);
    let plain_bash = matches!(reading, Some(ScriptCommands::Plain(_)))
        && policy_posix_shell_script(command).is_none();
    // A top-level heredoc script has its own pass
    // (`apply_heredoc_restrictive_matches`).
    if plain_bash
        || (reading.is_none() && !runs_unread_script(command))
        || parse_shell_lc_single_command_prefix(command).is_some()
    {
        return Vec::new();
    }
    check(
        policy,
        std::slice::from_ref(&command.to_vec()),
        &|_: &[String]| Decision::Allow,
        options,
        platform,
        /*read_own_scripts*/ true,
    )
    .matched_rules
    .into_iter()
    .filter(|rule_match| rule_match.decision() > Decision::Allow)
    .collect()
}

/// A command still to check, and whether it comes from a script not read
/// in full (#2543): then it can only add prompts and refusals.
struct Pending {
    command: Vec<String>,
    restrict_only: bool,
    /// How many scripts read only in part it is nested in.
    depth: usize,
}

/// #2543: how deep scripts read only in part are read through shells with
/// no launcher before them (`bash -lc "bash -c '…' > log"`); a script nested
/// deeper asks when a rule could match.
const MAX_PARTIAL_SCRIPT_DEPTH: usize = 8;

/// Where a word is known only at run time (#2543), launchers read this.
const UNKNOWN_WORD: &str = "\0";

/// The script of `[shell, -c|-lc, script]` as rules can read it, for the
/// shells the parser knows and (#2543) other POSIX shells, read like sh.
fn script_reading(command: &[String]) -> Option<ScriptCommands> {
    shell_lc_script_commands(command).or_else(|| {
        policy_posix_shell_script(command)
            .and_then(|respelled| shell_lc_script_commands(&respelled))
    })
}

fn check(
    policy: &Policy,
    commands: &[Vec<String>],
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
    read_own_scripts: bool,
) -> Evaluation {
    let mut matches = Vec::new();
    // #2543: what any prompt, prompt-user or forbidden rule could ask,
    // computed once.
    let mut any_rule_once = None;
    let mut any_rule = || {
        *any_rule_once.get_or_insert_with(|| {
            policy.strictest_rule_matching_more_args_by(&[], options, possible_rule_decision)
        })
    };
    // Inner shell scripts can themselves contain wrapped commands. Use a work
    // list rather than recursion; preserve every outer match along the way.
    let mut pending: Vec<Pending> = Vec::new();
    let mut next = commands.iter();
    loop {
        let owned = pending.pop();
        let (command, restrict_only, depth) = match &owned {
            Some(item) => (&item.command, item.restrict_only, item.depth),
            None => match next.next() {
                Some(command) => (command, false, 0),
                None => break,
            },
        };
        let wrappers = policy_wrapper_commands(command);
        // Keep only the strictest fallback for this chain. Copying the entire
        // remaining argv at every layer would grow quadratically with nesting.
        let mut heuristic = ((wrappers.uncertain || wrappers.approval_only) && !restrict_only)
            .then_some((command.as_slice(), Decision::Prompt));
        // #2455 (D677, D686): xargs adds words read from stdin. Ask whenever
        // a prompt, prompt-user or forbidden rule could match once they
        // arrive; a rule the fixed argv already matches applies below. Each
        // rule's level is mapped before taking the strictest (#2506).
        // Every rule counts where no word is known; scan them once.
        let could_complete = |known: &[String]| {
            policy.strictest_rule_matching_more_args_by(known, options, possible_rule_decision)
        };
        // #2506, #2519: an uncertain chain (a shell string as with `su -c`
        // or `watch`, unknown options, under xargs a launcher with no
        // command or a shell) runs a command no rule has seen: every rule
        // counts. Where the command is read, its rules apply as written.
        if wrappers.uncertain || (restrict_only && wrappers.approval_only) {
            raise(&mut heuristic, command, any_rule());
        }
        for &start in &wrappers.starts {
            let candidate = &command[start..];
            let mut found = policy.matches_for_command_with_options(
                candidate, /*heuristics_fallback*/ None, options,
            );
            if restrict_only {
                // #2543: the script around it decides the rest.
                found.retain(|rule_match| rule_match.decision() > Decision::Allow);
            } else if found.is_empty() {
                raise(&mut heuristic, candidate, Some(fallback(candidate)));
            }
            matches.extend(found);
            if let Some(open) = &wrappers.open {
                let known = &command[start..open.known_end.max(start)];
                let decision = if known.is_empty() {
                    any_rule()
                } else {
                    could_complete(known)
                };
                raise(&mut heuristic, candidate, decision);
            }
            // #2568 (D553): a PowerShell command line whose script rules
            // cannot read (an unknown parameter, a script from stdin,
            // `-File`): every rule counts. A `-File` script's file and
            // arguments are checked as a command too.
            if platform == DangerousCommandPlatform::Windows
                && let Some(unread) = unread_powershell_command_line(candidate)
            {
                raise(&mut heuristic, candidate, could_complete(&[]));
                if let UnreadPowershell::File(file) = unread {
                    pending.push(Pending {
                        command: file.to_vec(),
                        restrict_only,
                        depth,
                    });
                }
            }
        }
        if let Some((command, decision)) = heuristic {
            matches.push(RuleMatch::HeuristicsRuleMatch {
                command: command.to_vec(),
                decision,
            });
        }
        let Some(&start) = wrappers.starts.last() else {
            continue;
        };
        // Sync reads scripts behind a launcher; #2543 also its own (the
        // top-level pass) and those in a script read only in part.
        if start == 0 && !(read_own_scripts && owned.is_none()) && !restrict_only {
            continue;
        }
        if start == 0 && restrict_only && depth >= MAX_PARTIAL_SCRIPT_DEPTH {
            if script_reading(command).is_some() || runs_unread_script(command) {
                raise_match(&mut matches, command, any_rule());
            }
            continue;
        }
        let inner = &command[start..];
        let parsed = commands_for_exec_policy_for_platform(inner, platform);
        if parsed.commands.len() != 1 || parsed.commands[0] != inner {
            pending.extend(parsed.commands.into_iter().map(|command| Pending {
                command,
                restrict_only,
                depth: depth + usize::from(restrict_only),
            }));
        } else if let Some(single) = parse_shell_lc_single_command_prefix(inner) {
            // Sync: upstream's parser no longer reduces a heredoc script to
            // its first command (main's `used_complex_parsing`); do it here
            // so a wrapped heredoc keeps main's prompt and inner rules.
            if !restrict_only {
                matches.push(RuleMatch::HeuristicsRuleMatch {
                    command: inner.to_vec(),
                    decision: Decision::Prompt,
                });
            }
            pending.push(Pending {
                command: single,
                restrict_only,
                depth: depth + usize::from(restrict_only),
            });
        } else if let Some(reading) = script_reading(inner) {
            // #2543 (D553, D688): a script rules cannot read as plain
            // commands. The literal argv above keeps its fallback; the
            // commands inside can only add prompts and refusals, and where
            // a command cannot be known, any rule that could match it asks.
            let mut ask = None;
            let commands = match reading {
                ScriptCommands::Plain(commands) => commands
                    .into_iter()
                    .map(|words| words.into_iter().map(Some).collect())
                    .collect(),
                ScriptCommands::Partial(commands) => commands,
                ScriptCommands::Unreadable => {
                    ask = any_rule();
                    Vec::new()
                }
            };
            for words in commands {
                if let Some(command) = words.iter().cloned().collect::<Option<Vec<_>>>() {
                    pending.push(Pending {
                        command,
                        restrict_only: true,
                        depth: depth + 1,
                    });
                } else {
                    ask = ask.max(partial_command_decision(
                        policy,
                        &words,
                        options,
                        &mut any_rule,
                        &mut matches,
                    ));
                }
            }
            raise_match(&mut matches, inner, ask);
        } else if runs_unread_script(inner) {
            // #2543: a script the parser does not read
            // (`bash -c 'git push' x`, `bash -ec …`, `fish -c …`).
            raise_match(&mut matches, inner, any_rule());
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

/// #2543: ask `decision`, if any, for `command`.
fn raise_match(matches: &mut Vec<RuleMatch>, command: &[String], decision: Option<Decision>) {
    if let Some(decision) = decision {
        matches.push(RuleMatch::HeuristicsRuleMatch {
            command: command.to_vec(),
            decision,
        });
    }
}

/// #2543: whether `command` runs a shell given a script on its command line
/// (`-c` in an option cluster, `--command`). Approximate on the side of
/// yes.
fn runs_unread_script(command: &[String]) -> bool {
    let Some((program, args)) = command.split_first() else {
        return false;
    };
    policy_runs_shell(program)
        && args.iter().any(|arg| {
            arg == "--command"
                || arg.starts_with("--command=")
                || arg
                    .strip_prefix('-')
                    .is_some_and(|letters| !letters.starts_with('-') && letters.contains('c'))
        })
}

/// #2543: the prompt and forbidden rule matches of a script command with
/// words known only at run time (`None`), added to `matches`. Returns what
/// it must ask besides, where a rule could match once those words are
/// known. Launchers read an unknown word as a word of their own, so one
/// that may be the program (`sudo $cmd`, `env $x git`) is found as such.
fn partial_command_decision(
    policy: &Policy,
    words: &[Option<String>],
    options: &MatchOptions,
    any_rule: &mut impl FnMut() -> Option<Decision>,
    matches: &mut Vec<RuleMatch>,
) -> Option<Decision> {
    let argv: Vec<String> = words
        .iter()
        .map(|word| word.clone().unwrap_or_else(|| UNKNOWN_WORD.to_string()))
        .collect();
    let wrappers = policy_wrapper_commands(&argv);
    if wrappers.uncertain {
        return any_rule();
    }
    // Words after an unknown one may move: it may expand to no word or
    // several.
    let first_unknown = words
        .iter()
        .position(Option::is_none)
        .unwrap_or(words.len());
    for &start in &wrappers.starts {
        // A launcher read past an unknown word (`timeout $t git`, `sudo -u
        // $u git`): the program may be another word.
        if start > first_unknown {
            return any_rule();
        }
        let mut end = words[start..]
            .iter()
            .position(Option::is_none)
            .map_or(words.len(), |offset| start + offset);
        if let Some(open) = &wrappers.open {
            end = end.min(open.known_end.max(start));
        }
        let known = &argv[start..end];
        // An unknown program, or a shell given unknown words, runs anything.
        if known.is_empty() || (end < words.len() && policy_runs_shell(&known[0])) {
            return any_rule();
        }
        let mut found = policy
            .matches_for_command_with_options(known, /*heuristics_fallback*/ None, options);
        found.retain(|rule_match| rule_match.decision() > Decision::Allow);
        matches.extend(found);
        if end < words.len()
            && let Some(decision) =
                policy.strictest_rule_matching_more_args_by(known, options, possible_rule_decision)
        {
            return Some(decision);
        }
    }
    None
}
