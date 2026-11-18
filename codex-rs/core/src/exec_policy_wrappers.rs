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
use codex_shell_command::bash::CommandStdin;
use codex_shell_command::bash::ReadCommand;
use codex_shell_command::bash::ScriptRead;
use codex_shell_command::bash::read_script;
use codex_shell_command::bash::shell_lc_script_read;
use roles::Role;

#[path = "exec_policy_script_roles.rs"]
mod roles;

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
        // #2548: nor can one for a script rules read only in part. (One
        // outside the minimal lexical set may get an amendment when nothing
        // asked: it is the exact script, and once a prompt or forbidden rule
        // exists the script is still read and asks, which an allow rule for
        // its argv cannot override.)
        || shell_lc_script_read(command).is_some_and(|read| runs_more_than_argv(&read))
}

/// #2548: whether `read` holds code rules see only by reading more than
/// its commands' argv: a part not read, or a command whose program runs
/// text (`eval`, a shell's here-document). Commands with words known only
/// at run time ask where needed when checked.
pub(super) fn runs_more_than_argv(read: &ScriptRead) -> bool {
    read.unread
        || read.commands.iter().any(|read_command| {
            let ReadCommand { words, stdin, .. } = read_command;
            let Some(command) = words.iter().cloned().collect::<Option<Vec<_>>>() else {
                // `eval "$x"`: its program is known to run text.
                let argv: Vec<String> = words
                    .iter()
                    .map(|word| word.clone().unwrap_or_else(|| UNKNOWN_WORD.to_string()))
                    .collect();
                let role_words = role_words(read_command);
                return policy_wrapper_commands(&argv).starts.iter().any(|&start| {
                    words[start].as_deref().is_some_and(|program| {
                        roles::runs_more_than_argv(program, &role_words[start + 1..])
                    })
                });
            };
            let wrappers = policy_wrapper_commands(&command);
            let start = wrappers.starts.last().copied().unwrap_or(0);
            roles::role(&command[..start], &command[start..], stdin) != Role::Argv
        })
}

/// #2548: whether `command` is a shell script outside the minimal lexical
/// set that its caller checks as argv (the one command of a here-document
/// script), and a prompt or forbidden rule exists: it must ask, as a script
/// read in part does.
pub(super) fn asks_outside_minimal_set(
    policy: &Policy,
    command: &[String],
    options: &MatchOptions,
) -> bool {
    let respelled = policy_shell_script(command);
    shell_lc_script_read(respelled.as_deref().unwrap_or(command)).is_some_and(|read| !read.minimal)
        && any_strict_rule(policy, options)
}

fn inner_runs_shell(command: &[String]) -> bool {
    command
        .first()
        .is_some_and(|program| roles::runs_shell(program))
}

/// #2548: `command`'s words for deciding what its program runs: a word
/// known only at run time that is always a number (`$?`) reads as one.
fn role_words(command: &ReadCommand) -> Vec<Option<String>> {
    command
        .words
        .iter()
        .enumerate()
        .map(|(index, word)| match word {
            None if command.numbers.get(index) == Some(&true) => Some("0".to_string()),
            word => word.clone(),
        })
        .collect()
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
    /// #2548: where its input comes from, for a command from a script.
    stdin: CommandStdin,
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
        let (command, depth, restrict_only, stdin) = match &owned {
            Some(item) => (&item.command, item.depth, item.restrict_only, &item.stdin),
            None => match next.next() {
                Some(command) => (command, 0, false, &CommandStdin::Inherited),
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
            if reads_script && let Some(parsed) = script_commands(parsed_as) {
                if asks_outside_minimal_set(policy, parsed_as, options) {
                    matches.push(RuleMatch::HeuristicsRuleMatch {
                        command: inner.to_vec(),
                        decision: Decision::Prompt,
                    });
                    unread_script_prompts = true;
                }
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
                        stdin: CommandStdin::Inherited,
                    }));
                }
            } else if reads_script && let Some(read) = shell_lc_script_read(parsed_as) {
                // #2543 (D553, D688): a script rules cannot read as plain
                // commands. The literal argv above keeps its fallback; the
                // commands inside can only add prompts and refusals, and
                // where a command cannot be known, any prompt or forbidden
                // rule could match it.
                let reading_from = matches.len();
                let ask = queue_script(
                    read,
                    /*single*/ false,
                    depth,
                    /*past_bound*/ depth >= MAX_SCRIPT_DEPTH && !always_read,
                    policy,
                    options,
                    &mut any_strict_rule,
                    &mut matches,
                    &mut folded,
                    &mut pending,
                );
                if ask {
                    matches.push(RuleMatch::HeuristicsRuleMatch {
                        command: inner.to_vec(),
                        decision: Decision::Prompt,
                    });
                }
                unread_script_prompts |= matches[reading_from..]
                    .iter()
                    .any(|rule_match| rule_match.decision() > Decision::Allow);
            } else if reads_script && runs_unread_script(parsed_as) && any_strict_rule() {
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
        // #2548 (D553): a command from a script whose program runs more than
        // its argv: `eval`'s words, a shell's here-document are read as
        // scripts; code rules cannot read asks where a rule could match it.
        // At the top (depth 0) a command is argv as given: a shell there
        // reads the caller's input, as before; any other program (a builtin
        // wrapper such as `/usr/bin/jobs -x`) is still classified.
        if let Some(&start) = wrappers.starts.last()
            && (depth > 0 || !inner_runs_shell(&command[start..]))
        {
            let inner = &command[start..];
            let reading_from = matches.len();
            let ask = match roles::role(&command[..start], inner, stdin) {
                Role::Argv => false,
                Role::Unread => any_strict_rule(),
                Role::Script { text, single } => {
                    let past_bound = depth >= MAX_SCRIPT_DEPTH;
                    queue_script(
                        read_script(&text),
                        single,
                        depth,
                        past_bound,
                        policy,
                        options,
                        &mut any_strict_rule,
                        &mut matches,
                        &mut folded,
                        &mut pending,
                    ) || (past_bound && any_strict_rule())
                }
            };
            if ask {
                matches.push(RuleMatch::HeuristicsRuleMatch {
                    command: inner.to_vec(),
                    decision: Decision::Prompt,
                });
            }
            unread_script_prompts |= matches[reading_from..]
                .iter()
                .any(|rule_match| rule_match.decision() > Decision::Allow);
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

/// #2543, #2548: queues the commands of a script read in part, each to add
/// only prompts and refusals; returns whether it must ask besides: part of
/// it is not read, or `single` and it holds more than one command (a shell
/// reading its script from stdin shares that input with its commands, which
/// may consume script text).
#[allow(clippy::too_many_arguments)]
fn queue_script(
    read: ScriptRead,
    single: bool,
    depth: usize,
    past_bound: bool,
    policy: &Policy,
    options: &MatchOptions,
    any_strict_rule: &mut impl FnMut() -> bool,
    matches: &mut Vec<RuleMatch>,
    folded: &mut Vec<RuleMatch>,
    pending: &mut Vec<Pending>,
) -> bool {
    let mut ask = false;
    // Text after a stdin script's one command may become commands once it
    // reads some of that input (`head -c 1` then `#git push`).
    if read.fails_closed() || (single && (read.commands.len() > 1 || read.text_after_commands)) {
        ask |= any_strict_rule();
    }
    for read_command in read.commands {
        let role_words = role_words(&read_command);
        let ReadCommand { words, stdin, .. } = read_command;
        if let Some(command) = words.iter().cloned().collect::<Option<Vec<_>>>() {
            if past_bound {
                ask = true;
            } else {
                pending.push(Pending {
                    command,
                    depth: depth + 1,
                    restrict_only: true,
                    stdin,
                });
            }
        } else {
            ask |= partial_command_matches(
                policy,
                &words,
                &role_words,
                options,
                any_strict_rule,
                matches,
                folded,
            );
        }
    }
    ask
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
    role_words: &[Option<String>],
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
        // An unknown program, or a shell (#2585: PowerShell too), `eval` or
        // a builtin that runs code given unknown words, runs anything.
        if known.is_empty() || roles::runs_more_than_argv(&known[0], &role_words[start + 1..]) {
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
