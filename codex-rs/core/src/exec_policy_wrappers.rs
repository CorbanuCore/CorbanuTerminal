//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::commands_for_exec_policy_for_platform;
use crate::security::tainted_action::policy_runs_shell;
use crate::security::tainted_action::policy_shell_script;
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
    let checked = check(
        policy, commands, /*read_own_scripts*/ true, fallback, options, platform,
    );
    (checked.evaluation, checked.folded)
}

/// #2543 (D553, D688): the prompt and refusal matches for `command`'s own
/// shell script when rules cannot read it as plain commands (redirections,
/// substitutions, words built at run time, a script the parser does not
/// read), with the case-folded ones among them (D692): the literal argv keeps
/// its own match and fallback, and this only adds checks. Empty for any other
/// command. Sync: the approval path does not read the own script of a
/// command line its split left whole (see [`check`]), so this pass reads it.
pub(crate) fn unread_script_matches(
    policy: &Policy,
    command: &[String],
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
) -> (Vec<RuleMatch>, Vec<RuleMatch>) {
    let respelled = policy_shell_script(command);
    let reading = shell_lc_script_commands(respelled.as_deref().unwrap_or(command));
    let plain = matches!(reading, Some(ScriptCommands::Plain(_))) && respelled.is_none();
    // A top-level heredoc script has its own pass
    // (`apply_heredoc_restrictive_matches`).
    if plain
        || (reading.is_none() && !runs_unread_script(command))
        || parse_shell_lc_single_command_prefix(command).is_some()
    {
        return (Vec::new(), Vec::new());
    }
    let checked = check(
        policy,
        std::slice::from_ref(&command.to_vec()),
        /*read_own_scripts*/ true,
        &|_: &[String]| Decision::Allow,
        options,
        platform,
    );
    let restrictive = |matches: Vec<RuleMatch>| {
        matches
            .into_iter()
            .filter(|rule_match| rule_match.decision() > Decision::Allow)
            .collect::<Vec<_>>()
    };
    (
        restrictive(checked.evaluation.matched_rules),
        restrictive(checked.folded),
    )
}

/// What [`check`] found.
pub(super) struct Checked {
    pub(super) evaluation: Evaluation,
    /// The case-folded rule matches (D692) it added to the evaluation.
    pub(super) folded: Vec<RuleMatch>,
    /// #2543: a script it could not read in full added a prompt or a
    /// refusal; an amendment for the outer argv cannot approve it.
    pub(super) unread_script_prompts: bool,
}

/// A command still to check, with how many scripts it is nested in, and
/// whether it comes from a script not read in full (#2543): then it can
/// only add prompts and refusals.
struct Pending {
    command: Vec<String>,
    depth: usize,
    restrict_only: bool,
}

/// Where a word is known only at run time (#2543), launchers read this.
const UNKNOWN_WORD: &str = "\0";

/// Apply `commands`' rules and fallback decisions through every wrapper
/// layer, reading each given command's own shell script only with
/// `read_own_scripts`. Sync: the approval path passes `false` for a command
/// line it could not split, whose own script upstream keeps as shell argv:
/// a heredoc (`apply_heredoc_restrictive_matches` reads it, as main's caller
/// split it first) or a script that is not plain ([`unread_script_matches`]
/// reads it). Scripts behind a launcher or a respelled shell in it, and
/// scripts nested in those, are still read. Main uses the same switch for
/// the zsh-fork escalation path, which the sync branch no longer has.
pub(super) fn check(
    policy: &Policy,
    commands: &[Vec<String>],
    read_own_scripts: bool,
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
    platform: DangerousCommandPlatform,
) -> Checked {
    let mut matches = Vec::new();
    let mut folded = Vec::new();
    let mut unread_script_prompts = false;
    // #2543: what any prompt, prompt-user or forbidden rule could ask,
    // computed once (sync: mapped by `possible_rule_decision`, D691).
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
        let (command, depth, restrict_only) = match &owned {
            Some(item) => (&item.command, item.depth, item.restrict_only),
            None => match next.next() {
                Some(command) => (command, 0, false),
                None => break,
            },
        };
        let matches_before = matches.len();
        let wrappers = policy_wrapper_commands(command);
        // Keep only the strictest fallback for this chain. Copying the entire
        // remaining argv at every layer would grow quadratically with nesting.
        let mut heuristic = ((wrappers.uncertain || wrappers.approval_only) && !restrict_only)
            .then_some((command.as_slice(), Decision::Prompt));
        // #2455 (D677, D686): xargs adds words read from stdin. Ask whenever
        // a prompt, prompt-user or forbidden rule could match once they
        // arrive; a rule the fixed argv already matches applies below. Each
        // rule's level is mapped before taking the strictest (#2506).
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
            // D692: kept beside the fallback, so this only adds checks.
            let case_folded = policy.case_folded_matches(candidate, options);
            matches.extend(case_folded.iter().cloned());
            folded.extend(case_folded);
            if let Some(open) = &wrappers.open {
                let known = &command[start..open.known_end.max(start)];
                let decision = if known.is_empty() {
                    any_rule()
                } else {
                    could_complete(known)
                };
                raise(&mut heuristic, candidate, decision);
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
            // (`BASH -lc`, `bash.exe -c`), or (#2543) another POSIX shell
            // (`dash -c`), unwraps like `bash -lc`/`sh -c`. The literal argv
            // above keeps its own match and fallback.
            let respelled = policy_shell_script(inner);
            let parsed_as = respelled.as_deref().unwrap_or(inner);
            // Scripts behind a launcher or a respelled shell are read at any
            // depth: each is a strict substring of the one around it.
            let always_read = start > 0 || respelled.is_some();
            // #2531: a script with no launcher before it too. Its commands
            // come from a script already (`bash -lc "bash -c 'git push'"`
            // gives `bash -c 'git push'`), or are those the caller gives.
            let reads_script = always_read || depth > 0 || read_own_scripts;
            let reads_past_bound = depth < MAX_SCRIPT_DEPTH || always_read;
            if reads_script
                && let Some((inner_commands, used_complex_parsing)) =
                    script_commands(parsed_as, platform)
            {
                // Sync (#2575): a heredoc in a script read only in part adds
                // no prompt of its own; its command's rules still apply.
                if used_complex_parsing && !restrict_only {
                    push_prompt(&mut matches, inner);
                }
                if !reads_past_bound {
                    // #2531: nested too deep to read; D691: a fresh human
                    // decision where a forbidden or prompt-user rule could
                    // match a command in it.
                    raise_match(&mut matches, inner, Some(Decision::Prompt).max(any_rule()));
                } else {
                    if depth >= MAX_SCRIPT_DEPTH {
                        // Past the bound, still read what is always read, so
                        // a command a rule refuses stays refused, not only
                        // asked.
                        push_prompt(&mut matches, inner);
                    }
                    pending.extend(inner_commands.into_iter().map(|command| Pending {
                        command,
                        depth: depth + 1,
                        restrict_only,
                    }));
                }
            } else if reads_script && let Some(reading) = shell_lc_script_commands(parsed_as) {
                // #2543 (D553, D688): a script rules cannot read as plain
                // commands. The literal argv above keeps its fallback; the
                // commands inside can only add prompts and refusals, and
                // where a command cannot be known, any rule that could match
                // it asks (sync: by D691's levels).
                let reading_from = matches.len();
                let mut ask = None;
                match reading {
                    ScriptCommands::Plain(_) => {}
                    ScriptCommands::Unreadable => ask = any_rule(),
                    ScriptCommands::Partial(commands) => {
                        for words in commands {
                            if let Some(command) = words.iter().cloned().collect::<Option<Vec<_>>>()
                            {
                                if reads_past_bound {
                                    pending.push(Pending {
                                        command,
                                        depth: depth + 1,
                                        restrict_only: true,
                                    });
                                } else {
                                    ask = ask.max(Some(Decision::Prompt)).max(any_rule());
                                }
                            } else {
                                ask = ask.max(partial_command_decision(
                                    policy,
                                    &words,
                                    options,
                                    &mut any_rule,
                                    &mut matches,
                                    &mut folded,
                                ));
                            }
                        }
                    }
                }
                raise_match(&mut matches, inner, ask);
                unread_script_prompts |= matches[reading_from..]
                    .iter()
                    .any(|rule_match| rule_match.decision() > Decision::Allow);
            } else if reads_script && runs_unread_script(parsed_as) {
                // #2543: a script the parser does not read
                // (`bash -c 'git push' x`, `bash -ec …`, `fish -c …`).
                let ask = any_rule();
                raise_match(&mut matches, inner, ask);
                unread_script_prompts |= ask.is_some();
            }
        }
        if restrict_only {
            unread_script_prompts |= matches[matches_before..]
                .iter()
                .any(|rule_match| rule_match.decision() > Decision::Allow);
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
    Checked {
        evaluation,
        folded,
        unread_script_prompts,
    }
}

/// Ask for approval of `command`'s script.
fn push_prompt(matches: &mut Vec<RuleMatch>, command: &[String]) {
    matches.push(RuleMatch::HeuristicsRuleMatch {
        command: command.to_vec(),
        decision: Decision::Prompt,
    });
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
/// words known only at run time (`None`), added to `matches` (the D692
/// case-folded ones also to `folded`). Returns what it must ask besides,
/// where a rule could match once those words are known. Launchers read an
/// unknown word as a word of their own, so one that may be the program
/// (`sudo $cmd`, `env $x git`) is found as such.
fn partial_command_decision(
    policy: &Policy,
    words: &[Option<String>],
    options: &MatchOptions,
    any_rule: &mut impl FnMut() -> Option<Decision>,
    matches: &mut Vec<RuleMatch>,
    folded: &mut Vec<RuleMatch>,
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
        let case_folded = policy.case_folded_matches(known, options);
        matches.extend(case_folded.iter().cloned());
        folded.extend(case_folded);
        if end < words.len()
            && let Some(decision) =
                policy.strictest_rule_matching_more_args_by(known, options, possible_rule_decision)
        {
            return Some(decision);
        }
    }
    None
}
