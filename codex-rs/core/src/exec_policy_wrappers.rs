//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::ExecPolicyCommands;
use super::commands_for_exec_policy;
use crate::security::tainted_action::policy_runs_shell;
use crate::security::tainted_action::policy_shell_script;
use crate::security::tainted_action::policy_wrapper_commands;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;
use codex_shell_command::bash::ScriptCommands;
use codex_shell_command::bash::read_script_commands;
use codex_shell_command::bash::shell_lc_script_commands;
use codex_shell_command::bash::shell_lc_script_inputs;

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
        || has_script_input(command)
        || partial_script_interprets_text(command)
        || script_builtin(command).first().is_some_and(|word| matches!(word.as_str(), "eval" | "." | "source"))
}

/// Unknown eval/source text must not acquire a remembered argv approval,
/// even when no current rule prompts. Retain ordinary non-interpreting argv.
fn partial_script_interprets_text(command: &[String]) -> bool {
    let commands = match shell_lc_script_commands(command) {
        Some(ScriptCommands::Partial(commands)) => commands,
        Some(ScriptCommands::Unreadable) => return true,
        Some(ScriptCommands::Plain(_)) | None => return false,
    };
    commands.into_iter().any(|words| {
        let argv: Vec<_> = words
            .into_iter()
            .map(|word| word.unwrap_or_else(|| UNKNOWN_WORD.to_string()))
            .collect();
        let wrappers = policy_wrapper_commands(&argv);
        wrappers.uncertain
            || wrappers.starts.iter().any(|start| {
                script_builtin(&argv[*start..]).first().is_some_and(|word| {
                    matches!(word.as_str(), "eval" | "." | "source" | UNKNOWN_WORD)
                        || policy_runs_shell(word)
                })
            })
    })
}

/// A shell receiving stdin must retain that input through argv lowering.
/// Unknown receiver words are already handled by partial-command matching.
pub(super) fn has_script_input(command: &[String]) -> bool {
    shell_input_scripts(command).next().is_some()
}

/// Known literal bodies become a shell -c argv, so the existing reader and
/// depth bound apply. Unsupported shells/options and producers remain unknown.
fn shell_input_scripts(command: &[String]) -> impl Iterator<Item = Option<Vec<String>>> {
    let eval = eval_script(command).map(|script| vec!["sh".to_string(), "-c".to_string(), script]);
    shell_lc_script_inputs(eval.as_deref().unwrap_or(command))
        .into_iter()
        .filter_map(|input| {
            let words = input.words.into_iter().collect::<Option<Vec<_>>>()?;
            let wrappers = policy_wrapper_commands(&words);
            let start = *wrappers.starts.last()?;
            let inner = &words[start..];
            // Eval may start a shell; command-less exec may redirect the
            // surrounding shell's stdin, including across an eval boundary.
            // Their input is not script text we can attribute to a receiver.
            if script_builtin(inner)
                .first()
                .is_some_and(|program| matches!(program.as_str(), "eval" | "exec"))
            {
                return Some(vec![None]);
            }
            if !inner
                .first()
                .is_some_and(|program| policy_runs_shell(program))
            {
                return None;
            }
            if runs_unread_script(inner) {
                // A -c script may pass its stdin on to another shell
                // (`echo text | bash -c bash`). Only discard that input
                // when every plain command is known not to interpret it.
                let respelled = policy_shell_script(inner);
                let independent =
                    match shell_lc_script_commands(respelled.as_deref().unwrap_or(inner)) {
                        Some(ScriptCommands::Plain(commands)) => commands.iter().all(|command| {
                            let wrappers = policy_wrapper_commands(command);
                            let target =
                                wrappers.starts.last().and_then(|start| command.get(*start));
                            !wrappers.uncertain
                                && wrappers.open.is_none()
                                && target.is_some_and(|program| {
                                    !policy_runs_shell(program)
                                        && !matches!(
                                            program.as_str(),
                                            "eval" | "." | "source" | "builtin"
                                        )
                                })
                        }),
                        _ => false,
                    };
                return (!independent).then_some(vec![None]);
            }
            let script = input
                .literal
                .filter(|_| {
                    !wrappers.uncertain
                        && wrappers.open.is_none()
                        && inner[1..]
                            .iter()
                            .all(|arg| matches!(arg.as_str(), "-s" | "--"))
                })
                .and_then(|script| {
                    let argv = vec![inner[0].clone(), "-c".to_string(), script];
                    let respelled = policy_shell_script(&argv).unwrap_or(argv);
                    let reading = shell_lc_script_commands(&respelled)?;
                    // Stdin scripts are read incrementally. An earlier command
                    // can consume later lines (even part of a quoted word).
                    // A single plain command consumes its whole syntax first.
                    let single =
                        matches!(reading, ScriptCommands::Plain(commands) if commands.len() <= 1);
                    Some((respelled, single))
                });
            Some(match script {
                Some((command, true)) => vec![Some(command)],
                // Still inspect the literal text for refusals, but do not
                // assume it is the text the shell will actually parse.
                Some((command, false)) => vec![Some(command), None],
                None => vec![None],
            })
        })
        .flatten()
}

// Only used for restrict-only script checks, never to grant an allow rule.
// Bash's builtin can invoke eval/source, including through another builtin.
fn script_builtin(mut command: &[String]) -> &[String] {
    while command.first().is_some_and(|word| word == "builtin") {
        command = &command[1..];
        if command.first().is_some_and(|word| word == "--") {
            command = &command[1..];
        }
    }
    command
}

/// Eval joins its static argv after its optional option terminator. The
/// partial-command path handles unknown words before this is called.
fn eval_script(command: &[String]) -> Option<String> {
    let (program, words) = script_builtin(command).split_first()?;
    if program != "eval" {
        return None;
    }
    let words = if words.first().is_some_and(|word| word == "--") {
        &words[1..]
    } else {
        words
    };
    Some(words.join(" "))
}

/// #2546: whether [`check`] asks for a PowerShell script in `commands` that
/// no trusted parser read; an amendment for its literal argv cannot approve
/// the commands inside. With no prompt or forbidden rule it does not ask.
pub(super) fn asks_for_unread_powershell(
    policy: &Policy,
    commands: &[Vec<String>],
    options: &MatchOptions,
) -> bool {
    any_strict_rule(policy, options)
        && commands
            .iter()
            .any(|command| powershell_script_unreadable(command))
}

/// Whether any prompt or forbidden rule exists, which could match a command
/// rules cannot see.
fn any_strict_rule(policy: &Policy, options: &MatchOptions) -> bool {
    policy
        .strictest_rule_matching_more_args(&[], options)
        .is_some_and(|decision| decision > Decision::Allow)
}

/// #2546 (D553): whether `command` runs a PowerShell script that no trusted
/// PowerShell could read (none found, or it could not run), so rules cannot
/// see its commands. #2568: also a command line PowerShell reads in a way
/// this reader cannot (unknown parameters, a script from stdin), and a
/// `-File` script, whose content is not read.
#[cfg(windows)]
fn powershell_script_unreadable(command: &[String]) -> bool {
    use codex_shell_command::powershell::PowershellScriptRead;
    let respelled = crate::security::tainted_action::policy_powershell_script(command);
    matches!(
        codex_shell_command::powershell::read_powershell_command_script(
            respelled.as_deref().unwrap_or(command),
        ),
        Some(PowershellScriptRead::Unreadable | PowershellScriptRead::File(_))
    )
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
    let checked = check(
        policy, commands, /*read_own_scripts*/ true, fallback, options,
    );
    (checked.evaluation, checked.folded)
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
    let checked = check(
        policy, commands, /*read_own_scripts*/ false, fallback, options,
    );
    (checked.evaluation, checked.folded)
}

/// What [`check`] found.
pub(crate) struct Checked {
    pub(crate) evaluation: Evaluation,
    /// The case-folded rule matches (D692) it added to the evaluation.
    pub(crate) folded: Vec<RuleMatch>,
    /// #2543: a script it could not read in full added a prompt or a
    /// refusal; an amendment for the outer argv cannot approve it.
    pub(crate) unread_script_prompts: bool,
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

pub(crate) fn check(
    policy: &Policy,
    commands: &[Vec<String>],
    read_own_scripts: bool,
    fallback: &impl Fn(&[String]) -> Decision,
    options: &MatchOptions,
) -> Checked {
    let mut matches = Vec::new();
    let mut folded = Vec::new();
    let mut unread_script_prompts = false;
    // #2543: whether any prompt or forbidden rule exists, computed once.
    let mut strict_rules = None;
    let mut any_strict_rule = || {
        // The function of that name: this closure is not in scope yet.
        *strict_rules.get_or_insert_with(|| any_strict_rule(policy, options))
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
        let mut heuristic = (wrappers.uncertain && (!restrict_only || any_strict_rule()))
            .then_some((command.as_slice(), Decision::Prompt));
        // Every rule counts where no word is known; scan them once.
        let mut any_rule = None;
        for &start in &wrappers.starts {
            let candidate = &command[start..];
            let mut found = policy.matches_for_command_with_options(
                candidate, /*heuristics_fallback*/ None, options,
            );
            if restrict_only {
                // #2543: the script around it decides the rest.
                found.retain(|rule_match| rule_match.decision() > Decision::Allow);
            } else if found.is_empty() {
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
            let reads_script = always_read || depth > 0 || read_own_scripts;
            if reads_script {
                for script in shell_input_scripts(parsed_as) {
                    if let Some(command) = script.filter(|_| depth < MAX_SCRIPT_DEPTH) {
                        pending.push(Pending {
                            command,
                            depth: depth + 1,
                            restrict_only: true,
                        });
                    } else if any_strict_rule() {
                        matches.push(RuleMatch::HeuristicsRuleMatch {
                            command: inner.to_vec(),
                            decision: Decision::Prompt,
                        });
                        unread_script_prompts = true;
                    }
                }
            }
            let reading = || {
                if let Some(script) = eval_script(inner) {
                    Some(if wrappers.open.is_some() {
                        ScriptCommands::Unreadable
                    } else {
                        read_script_commands(&script)
                    })
                } else {
                    shell_lc_script_commands(parsed_as)
                }
            };
            if reads_script && let Some(parsed) = script_commands(parsed_as) {
                if parsed.used_complex_parsing || depth >= MAX_SCRIPT_DEPTH {
                    matches.push(RuleMatch::HeuristicsRuleMatch {
                        command: inner.to_vec(),
                        decision: Decision::Prompt,
                    });
                }
                // Past the bound, still read what is always read, so a
                // command a rule refuses stays refused, not only asked.
                if depth < MAX_SCRIPT_DEPTH || always_read {
                    pending.extend(parsed.commands.into_iter().map(|command| Pending {
                        command,
                        depth: depth + 1,
                        restrict_only,
                    }));
                }
            } else if reads_script && let Some(reading) = reading() {
                // #2543 (D553, D688): a script rules cannot read as plain
                // commands. The literal argv above keeps its fallback; the
                // commands inside can only add prompts and refusals, and
                // where a command cannot be known, any prompt or forbidden
                // rule could match it.
                let reading_from = matches.len();
                let mut ask = false;
                let reading = match reading {
                    ScriptCommands::Plain(commands) => ScriptCommands::Partial(
                        commands
                            .into_iter()
                            .map(|words| words.into_iter().map(Some).collect())
                            .collect(),
                    ),
                    other => other,
                };
                match reading {
                    ScriptCommands::Plain(_) => unreachable!("plain reading converted above"),
                    ScriptCommands::Unreadable => ask = any_strict_rule(),
                    ScriptCommands::Partial(commands) => {
                        for words in commands {
                            if let Some(command) = words.iter().cloned().collect::<Option<Vec<_>>>()
                            {
                                if depth < MAX_SCRIPT_DEPTH || always_read {
                                    pending.push(Pending {
                                        command,
                                        depth: depth + 1,
                                        restrict_only: true,
                                    });
                                } else {
                                    ask = true;
                                }
                            } else {
                                ask |= partial_command_matches(
                                    policy,
                                    &words,
                                    options,
                                    &mut any_strict_rule,
                                    &mut matches,
                                    &mut folded,
                                );
                            }
                        }
                    }
                }
                if ask {
                    matches.push(RuleMatch::HeuristicsRuleMatch {
                        command: inner.to_vec(),
                        decision: Decision::Prompt,
                    });
                }
                unread_script_prompts |= matches[reading_from..]
                    .iter()
                    .any(|rule_match| rule_match.decision() > Decision::Allow);
            } else if reads_script
                && (runs_unread_script(parsed_as)
                    || script_builtin(inner)
                        .first()
                        .is_some_and(|word| matches!(word.as_str(), "." | "source")))
                && any_strict_rule()
            {
                // #2543: a script the parser does not read
                // (`bash -c 'git push' x`, `bash -ec …`, `fish -c …`).
                matches.push(RuleMatch::HeuristicsRuleMatch {
                    command: inner.to_vec(),
                    decision: Decision::Prompt,
                });
                unread_script_prompts = true;
            }
            // Not `else`: a `-File` script reads as its file and arguments
            // above, and asks too (#2568).
            if reads_script && any_strict_rule() && powershell_script_unreadable(inner) {
                // #2546 (D553): a PowerShell script no trusted parser read.
                // Any prompt or forbidden rule could match a command in it;
                // the literal argv above keeps its own match and fallback.
                matches.push(RuleMatch::HeuristicsRuleMatch {
                    command: inner.to_vec(),
                    decision: Decision::Prompt,
                });
                unread_script_prompts = true;
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
/// words known only at run time (`None`), added to `matches`. Returns
/// whether it must ask besides: a prompt or forbidden rule could match once
/// those words are known. Launchers read an unknown word as a word of
/// their own, so one that may be the program (`sudo $cmd`, `env $x git`)
/// is found as such.
fn partial_command_matches(
    policy: &Policy,
    words: &[Option<String>],
    options: &MatchOptions,
    any_strict_rule: &mut impl FnMut() -> bool,
    matches: &mut Vec<RuleMatch>,
    folded: &mut Vec<RuleMatch>,
) -> bool {
    let argv: Vec<String> = words
        .iter()
        .map(|word| word.clone().unwrap_or_else(|| UNKNOWN_WORD.to_string()))
        .collect();
    let wrappers = policy_wrapper_commands(&argv);
    if wrappers.uncertain {
        return any_strict_rule();
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
            return any_strict_rule();
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
        if known.is_empty()
            || (end < words.len()
                && (policy_runs_shell(&known[0])
                    || matches!(known[0].as_str(), "eval" | "." | "source" | "builtin")))
        {
            return any_strict_rule();
        }
        let mut found = policy
            .matches_for_command_with_options(known, /*heuristics_fallback*/ None, options);
        found.retain(|rule_match| rule_match.decision() > Decision::Allow);
        matches.extend(found);
        let case_folded = policy.case_folded_matches(known, options);
        matches.extend(case_folded.iter().cloned());
        folded.extend(case_folded);
        if end < words.len()
            && policy
                .strictest_rule_matching_more_args(known, options)
                .is_some_and(|decision| decision > Decision::Allow)
        {
            return true;
        }
    }
    false
}
