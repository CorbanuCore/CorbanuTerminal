//! Apply command rules and fallback decisions to every known wrapper layer.
//! Keep the literal outer match, so adding a wrapper can only tighten policy.

use super::commands_for_exec_policy_for_platform;
use crate::security::tainted_action::policy_runs_shell;
use crate::security::tainted_action::policy_shell_script;
use crate::security::tainted_action::policy_windows_program_name;
use crate::security::tainted_action::policy_wrapper_commands;
use codex_execpolicy::Decision;
use codex_execpolicy::Evaluation;
use codex_execpolicy::MatchOptions;
use codex_execpolicy::Policy;
use codex_execpolicy::RuleMatch;
use codex_shell_command::bash::CommandStdin;
use codex_shell_command::bash::ReadCommand;
use codex_shell_command::bash::ScriptRead;
use codex_shell_command::bash::parse_shell_lc_single_command_prefix;
use codex_shell_command::bash::read_script;
use codex_shell_command::bash::shell_lc_script_read;
use codex_shell_command::is_dangerous_command::DangerousCommandPlatform;
use codex_shell_command::powershell::UnreadPowershell;
use codex_shell_command::powershell::unread_powershell_command_line;
use roles::Role;
use std::cell::Cell;
use std::marker::PhantomData;

#[path = "exec_policy_script_roles.rs"]
mod roles;

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
) -> Option<(Vec<Vec<String>>, bool, bool)> {
    let parsed = commands_for_exec_policy_for_platform(command, platform);
    if parsed.commands.len() != 1 || parsed.commands[0] != command {
        // #2704: a PowerShell script's commands, whose programs PowerShell
        // finds (`pnpm` as `pnpm.cmd`).
        let powershell = matches!(
            parsed.command_origin,
            super::ExecPolicyCommandOrigin::PowerShell
                | super::ExecPolicyCommandOrigin::PowerShellLiteral
        );
        return Some((parsed.commands, false, powershell));
    }
    // #2548: a here-document a shell reads as its script (or any script
    // whose commands run more than their argv) is read whole, input and all.
    if shell_lc_script_read(command).is_some_and(|read| runs_more_than_argv(&read)) {
        return None;
    }
    parse_shell_lc_single_command_prefix(command).map(|single| (vec![single], true, false))
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
        // #2548: nor can one for a script rules read only in part.
        || shell_lc_script_read(command).is_some_and(|read| runs_more_than_argv(&read))
        // #2568: an amendment for a PowerShell command line whose script
        // rules cannot read cannot approve the commands inside.
        || unread_powershell_line(command, platform).is_some()
        // #2601: nor one for a PowerShell script not lowered off Windows.
        || runs_powershell_unlowered(command, platform)
        // #2671: nor one for a launcher or shell no rule reads as one.
        || windows_name_unread(command, platform)
}

/// #2601 (D553): `command` runs PowerShell for a platform whose PowerShell
/// scripts are not lowered into commands (any but Windows; #2671: named as
/// macOS names it too, on any host).
fn runs_powershell_unlowered(command: &[String], platform: DangerousCommandPlatform) -> bool {
    platform != DangerousCommandPlatform::Windows
        && command.first().is_some_and(|program| {
            crate::security::tainted_action::policy_runs_powershell_on_posix(program)
        })
}

/// #2671 (sync): `command` for a Windows executor starts with a launcher, a
/// shell or PowerShell that rules do not read as one (see
/// `windows_names_unread`).
fn windows_name_unread(command: &[String], platform: DangerousCommandPlatform) -> bool {
    platform == DangerousCommandPlatform::Windows
        && crate::security::tainted_action::windows_names_unread(command)
}

/// #2699: `command` for a Windows executor starts with PowerShell spelled
/// so Windows may run another file, which no reader reads.
fn windows_powershell_unsure(command: &[String], platform: DangerousCommandPlatform) -> bool {
    platform == DangerousCommandPlatform::Windows
        && command.first().is_some_and(|program| {
            codex_shell_command::powershell::is_powershell_program(program)
                && codex_shell_command::shell_detect::windows_name_unsure(program)
        })
}

struct UnreadPowershellCommands {
    commands: Vec<Vec<String>>,
    restrict_only: bool,
}

/// D553: retain restrictions on readable prefixes and ask for the unknown
/// remainder. D688: recognize Windows executable spelling as on main;
/// (#2671) for a Windows executor on any host.
fn unread_powershell_line(
    command: &[String],
    platform: DangerousCommandPlatform,
) -> Option<UnreadPowershellCommands> {
    let respelled = (cfg!(windows) || platform == DangerousCommandPlatform::Windows)
        .then(|| crate::security::tainted_action::policy_powershell_script(command))
        .flatten();
    unread_powershell_command_line(respelled.as_deref().unwrap_or(command)).map(|unread| {
        match unread {
            UnreadPowershell::File(file) => UnreadPowershellCommands {
                commands: vec![file.to_vec()],
                restrict_only: false,
            },
            UnreadPowershell::Partial(commands) => UnreadPowershellCommands {
                commands,
                restrict_only: true,
            },
            UnreadPowershell::Unreadable => UnreadPowershellCommands {
                commands: Vec::new(),
                restrict_only: true,
            },
        }
    })
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

/// D729 (A): whether any prompt, prompt-user or forbidden rule exists, which
/// a PowerShell script read only in part might hide a command from (D733:
/// otherwise scripts are read literally, as before D729).
pub(super) fn any_strict_rule(policy: &Policy, options: &MatchOptions) -> bool {
    policy
        .strictest_rule_matching_more_args_by(&[], options, possible_rule_decision)
        .is_some()
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
        policy, commands, /*read_own_scripts*/ true, /*from_powershell*/ false, fallback,
        options, platform,
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
    let reading = shell_lc_script_read(respelled.as_deref().unwrap_or(command));
    // #2548: a plain script the split lowered to its commands is read there,
    // unless one runs a script file, whose unread commands ask below.
    let plain = reading
        .as_ref()
        .is_some_and(|read| read.plain && !runs_more_than_argv(read) && !runs_script_file(read))
        && respelled.is_none();
    // A top-level heredoc script has its own pass
    // (`apply_heredoc_restrictive_matches`) unless (#2548) it runs more than
    // its one command's argv (a shell given the here-document), or lies
    // outside the minimal lexical set where a rule could match (it asks).
    let strict_rules = policy
        .strictest_rule_matching_more_args_by(&[], options, possible_rule_decision)
        .is_some();
    let heredoc_pass_suffices = parse_shell_lc_single_command_prefix(command).is_some()
        && reading.as_ref().is_none_or(|read| {
            !runs_more_than_argv(read) && (!read.fails_closed() || !strict_rules)
        });
    // #2704: a script cmd.exe reads is read here too (strict rules first:
    // finding a batch file looks at `PATH`).
    let unread = runs_unread_script(command)
        || (strict_rules && runs_unread_cmd_script(command, platform, /*powershell*/ false));
    if plain || (reading.is_none() && !unread) || heredoc_pass_suffices {
        return (Vec::new(), Vec::new());
    }
    let checked = check(
        policy,
        std::slice::from_ref(&command.to_vec()),
        /*read_own_scripts*/ true,
        /*from_powershell*/ false,
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
    /// #2548: where its input comes from, for a command from a script.
    stdin: CommandStdin,
    /// #2704: from a PowerShell script, so PowerShell finds its program
    /// (`pnpm` as `pnpm.cmd` on `PATH`).
    powershell: bool,
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
/// `from_powershell`: `commands` are a PowerShell script's (#2704: their
/// programs are found as PowerShell finds them).
pub(super) fn check(
    policy: &Policy,
    commands: &[Vec<String>],
    read_own_scripts: bool,
    from_powershell: bool,
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
        let (command, depth, restrict_only, stdin, powershell) = match &owned {
            Some(item) => (
                &item.command,
                item.depth,
                item.restrict_only,
                &item.stdin,
                item.powershell,
            ),
            None => match next.next() {
                Some(command) => (command, 0, false, &CommandStdin::Inherited, from_powershell),
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
            // #2568 (D553): a PowerShell command line whose script rules
            // cannot read (an unknown parameter, a script from stdin,
            // `-File`): every rule counts. A `-File` script's file and
            // arguments are checked as a command too.
            // #2601 (D553): off Windows no PowerShell script is lowered, so
            // any PowerShell command line runs commands rules cannot see.
            // #2671: for a Windows executor, a launcher, shell or PowerShell
            // spelled so rules do not read it as one: every rule counts.
            if runs_powershell_unlowered(candidate, platform)
                || windows_name_unread(candidate, platform)
            {
                raise(&mut heuristic, candidate, could_complete(&[]));
            }
            // #2699: for a Windows executor, a PowerShell spelled so Windows
            // may run another file (`pwsh.exe:x.exe`, an NTFS stream) is not
            // read, so its script is neither matched nor judged dangerous:
            // it asks, as an uncertain launcher does on a Windows host.
            if !restrict_only && windows_powershell_unsure(candidate, platform) {
                raise(&mut heuristic, candidate, Some(Decision::Prompt));
            }
            if platform == DangerousCommandPlatform::Windows
                && let Some(unread) = unread_powershell_line(candidate, platform)
            {
                raise(&mut heuristic, candidate, could_complete(&[]));
                if !unread.restrict_only {
                    // `-File`: its file and arguments, checked as before.
                    pending.extend(unread.commands.into_iter().map(|command| Pending {
                        command,
                        restrict_only,
                        depth,
                        stdin: CommandStdin::Inherited,
                        powershell: true,
                    }));
                } else if depth < MAX_SCRIPT_DEPTH || start > 0 {
                    // #2556, #2583 (D553): known literal prefixes of a script
                    // read only in part add prompts and refusals only; the
                    // unknown remainder asks above. Behind a launcher they
                    // are read at any depth, as main's `always_read`.
                    pending.extend(unread.commands.into_iter().map(|command| Pending {
                        command,
                        restrict_only: true,
                        depth: depth + 1,
                        stdin: CommandStdin::Inherited,
                        powershell: true,
                    }));
                }
            }
        }
        if let Some((command, decision)) = heuristic {
            matches.push(RuleMatch::HeuristicsRuleMatch {
                command: command.to_vec(),
                decision,
            });
        }
        // #2704: in a PowerShell script any program of the command, a
        // launcher (`.\env foo …`) or a shell spelled as a path (`.\bash -c
        // …`) too, may be a batch file cmd.exe re-reads.
        if powershell
            && let Some(ask) = any_rule()
            && wrappers.starts.iter().any(|&start| {
                runs_unread_cmd_script(&command[start..], platform, /*powershell*/ true)
            })
        {
            raise_match(&mut matches, command, Some(ask));
            unread_script_prompts = true;
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
                && let Some((inner_commands, used_complex_parsing, inner_powershell)) =
                    script_commands(parsed_as, platform)
            {
                // A heredoc asks, as main's `used_complex_parsing` does, also
                // in a script read only in part (#2575 had skipped that
                // prompt; review r1 of #2582 matched main, the stricter).
                if used_complex_parsing {
                    push_prompt(&mut matches, inner);
                }
                // #2548 (D553): a script outside the minimal lexical set (a
                // here-document) asks wherever a rule could match a command
                // in it, as main's `asks_outside_minimal_set`; D691: a fresh
                // human decision where a forbidden or prompt-user rule could.
                if shell_lc_script_read(parsed_as)
                    .is_some_and(|read| !read.minimal || runs_script_file(&read))
                {
                    let ask = any_rule();
                    raise_match(&mut matches, inner, ask);
                    unread_script_prompts |= ask.is_some();
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
                        stdin: CommandStdin::Inherited,
                        powershell: inner_powershell,
                    }));
                }
            } else if reads_script && let Some(read) = shell_lc_script_read(parsed_as) {
                // #2543 (D553, D688), #2548: a script rules cannot read as
                // plain commands. The literal argv above keeps its fallback;
                // the commands inside can only add prompts and refusals, and
                // where a command or part cannot be known, any rule that
                // could match it asks (sync: by D691's levels).
                let reading_from = matches.len();
                let ask = queue_script(
                    read,
                    /*single*/ false,
                    depth,
                    /*past_bound*/ !reads_past_bound,
                    policy,
                    options,
                    &mut any_rule,
                    &mut matches,
                    &mut folded,
                    &mut pending,
                );
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
            } else if reads_script
                && let Some(ask) = any_rule()
                && runs_unread_cmd_script(parsed_as, platform, powershell)
            {
                // #2704: a script cmd.exe reads (`cmd /c "git push"`). Rules
                // first: finding a batch file looks at `PATH`.
                raise_match(&mut matches, inner, Some(ask));
                unread_script_prompts = true;
            }
        }
        // #2548 (D553): a command whose program runs more than its argv:
        // `eval`'s words and a shell's here-document are read as scripts;
        // code rules cannot read asks where a rule could match it. At the
        // top (depth 0) a command is argv as given: a shell there reads the
        // caller's input, as before; any other program (a builtin wrapper
        // such as `/usr/bin/jobs -x`) is still classified.
        if let Some(&start) = wrappers.starts.last()
            && (depth > 0 || !inner_runs_shell(&command[start..]))
        {
            let inner = &command[start..];
            let reading_from = matches.len();
            // #2601 (D553): a PowerShell command lowered for Windows that
            // rebinds a name or runs code is unread, as bash's `alias` is;
            // only where nothing else reads or asks, so script text read
            // precisely (`eval git push alias:`) keeps its refusals.
            let role = match roles::role(&command[..start], inner, stdin) {
                Role::Argv
                    if platform == DangerousCommandPlatform::Windows
                        && roles::powershell_runs_code(inner) =>
                {
                    Role::Unread
                }
                role => role,
            };
            let ask = match role {
                Role::Argv => None,
                Role::Unread | Role::File => any_rule(),
                Role::Script { text, single } => {
                    let past_bound = depth >= MAX_SCRIPT_DEPTH;
                    let ask = queue_script(
                        read_script(&text),
                        single,
                        depth,
                        past_bound,
                        policy,
                        options,
                        &mut any_rule,
                        &mut matches,
                        &mut folded,
                        &mut pending,
                    );
                    if past_bound { ask.max(any_rule()) } else { ask }
                }
            };
            raise_match(&mut matches, inner, ask);
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
/// only prompts and refusals; returns what it must ask besides: part of it
/// is not read (or lies outside the minimal lexical set), or `single` and it
/// holds more than one command (a shell reading its script from stdin
/// shares that input with its commands, which may consume script text).
#[allow(clippy::too_many_arguments)]
fn queue_script(
    read: ScriptRead,
    single: bool,
    depth: usize,
    past_bound: bool,
    policy: &Policy,
    options: &MatchOptions,
    any_rule: &mut impl FnMut() -> Option<Decision>,
    matches: &mut Vec<RuleMatch>,
    folded: &mut Vec<RuleMatch>,
    pending: &mut Vec<Pending>,
) -> Option<Decision> {
    let mut ask = None;
    // Text after a stdin script's one command may become commands once it
    // reads some of that input (`head -c 1` then `#git push`).
    if read.fails_closed() || (single && (read.commands.len() > 1 || read.text_after_commands)) {
        ask = ask.max(any_rule());
    }
    for read_command in read.commands {
        let role_words = role_words(&read_command);
        let ReadCommand { words, stdin, .. } = read_command;
        if let Some(command) = words.iter().cloned().collect::<Option<Vec<_>>>() {
            if past_bound {
                ask = ask.max(Some(Decision::Prompt)).max(any_rule());
            } else {
                pending.push(Pending {
                    command,
                    depth: depth + 1,
                    restrict_only: true,
                    stdin,
                    powershell: false,
                });
            }
        } else {
            ask = ask.max(partial_command_decision(
                policy,
                &words,
                &role_words,
                options,
                any_rule,
                matches,
                folded,
            ));
        }
    }
    ask
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
            !matches!(
                roles::role(&command[..start], &command[start..], stdin),
                Role::Argv | Role::File
            )
        })
}

/// #2548: whether a command of `read` is a shell running a named script file
/// ([`Role::File`]): split like argv, but its commands are unread.
pub(super) fn runs_script_file(read: &ScriptRead) -> bool {
    read.commands.iter().any(|read_command| {
        let Some(command) = read_command
            .words
            .iter()
            .cloned()
            .collect::<Option<Vec<_>>>()
        else {
            return false;
        };
        let wrappers = policy_wrapper_commands(&command);
        let start = wrappers.starts.last().copied().unwrap_or(0);
        roles::role(&command[..start], &command[start..], &read_command.stdin) == Role::File
    })
}

fn inner_runs_shell(command: &[String]) -> bool {
    command
        .first()
        .is_some_and(|program| roles::runs_shell(program))
}

/// #2548: `command`'s words for deciding what its program runs: a word
/// known only at run time that is always a number (`$?`) reads as one,
/// [`roles::RUN_TIME_NUMBER`].
fn role_words(command: &ReadCommand) -> Vec<Option<String>> {
    command
        .words
        .iter()
        .enumerate()
        .map(|(index, word)| match word {
            None if command.numbers.get(index) == Some(&true) => {
                Some(roles::RUN_TIME_NUMBER.to_string())
            }
            word => word.clone(),
        })
        .collect()
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

/// #2704 (D553): whether `command` runs a script cmd.exe reads, which rules
/// cannot see: cmd.exe itself, whatever its arguments (`/c`, `/k`, `/d/c`,
/// or a script from its input), or a program that may be a batch file
/// (see [`may_be_batch_file`]) given an argument cmd.exe would read as more
/// than a word when it re-reads the batch file's command line
/// (`pnpm.cmd '&git push'`). Approximate on the side of yes.
///
/// `powershell`: the command comes from a PowerShell script, which finds its
/// program as a batch file on `PATH` or by a path without an extension;
/// another shell (bash) or a command line run as given does not.
fn runs_unread_cmd_script(
    command: &[String],
    platform: DangerousCommandPlatform,
    powershell: bool,
) -> bool {
    let Some((program, args)) = command.split_first() else {
        return false;
    };
    // As Windows finds it (`C:\Windows\System32\CMD.EXE.`, `C:cmd`).
    let name = policy_windows_program_name(program);
    if name == "cmd" {
        return true;
    }
    args.iter().any(|arg| cmd_rereads(arg))
        && may_be_batch_file(program, &name, platform, powershell)
}

/// Whether cmd.exe may read `arg` as more than one plain word: a command
/// separator, redirection, escape, variable expansion, quote, group or
/// line break.
fn cmd_rereads(arg: &str) -> bool {
    arg.contains(['&', '|', '<', '>', '^', '%', '!', '"', '(', ')', '\r', '\n'])
}

/// Whether `program` (Windows file name `name`) may run as a batch file:
/// named `.cmd` or `.bat`, or on Windows a path not named `.exe` or `.com`
/// (`.\build`, `C:\p\x`: the command's folder may hold a batch file, or
/// a script may create one first), or a bare name PowerShell finds on
/// `PATH` as a batch file (`pnpm` as `pnpm.cmd`).
fn may_be_batch_file(
    program: &str,
    name: &str,
    platform: DangerousCommandPlatform,
    powershell: bool,
) -> bool {
    name.ends_with(".cmd")
        || name.ends_with(".bat")
        || (powershell
            && platform == DangerousCommandPlatform::Windows
            && windows_batch_file(program))
}

thread_local! {
    /// Whether the command being checked on this thread runs on this host
    /// (see [`enter_local_executor`]).
    static LOCAL_EXECUTOR: Cell<bool> = const { Cell::new(false) };
}

/// While the returned guard lives, commands checked on this thread run on
/// this host when `local` (a local environment), so their PowerShell finds
/// programs on this host's `PATH`. Outside it, or for a remote environment,
/// a program's file is unknown (see [`windows_batch_file`]). The guard is not
/// `Send`, so it cannot be held across an `.await`.
#[must_use]
pub(super) fn enter_local_executor(local: bool) -> LocalExecutorGuard {
    LocalExecutorGuard {
        previous: LOCAL_EXECUTOR.with(|current| current.replace(local)),
        _not_send: PhantomData,
    }
}

pub(super) struct LocalExecutorGuard {
    previous: bool,
    _not_send: PhantomData<*const ()>,
}

impl Drop for LocalExecutorGuard {
    fn drop(&mut self) {
        LOCAL_EXECUTOR.with(|current| current.set(self.previous));
    }
}

/// Whether commands checked on this thread run on this host (tests).
#[cfg(test)]
pub(super) fn local_executor() -> bool {
    LOCAL_EXECUTOR.with(Cell::get)
}

/// See [`may_be_batch_file`]. A command running on this host, a Windows one
/// (#2729: the 6 sync-only Windows regressions #2711 caused): as on main, a bare name is looked up
/// on this host's `PATH` ([`windows_batch_file_in`]), so PowerShell's own
/// `echo` is not a batch file unless `PATH` holds one first. Sync: any other
/// executor may be another host (#2671; a remote environment reports only its
/// platform), whose `PATH` this host cannot read, so there any program not
/// named `.exe` or `.com` may be a batch file (`pnpm` as `pnpm.cmd`,
/// `.\build` in the command's folder or created by the script first).
fn windows_batch_file(program: &str) -> bool {
    if cfg!(windows) && LOCAL_EXECUTOR.with(Cell::get) {
        return windows_batch_file_in(
            program,
            std::env::var_os("PATH"),
            std::env::var("PATHEXT").ok(),
        );
    }
    let lower = program.trim_end_matches(['.', ' ']).to_ascii_lowercase();
    !(lower.ends_with(".exe") || lower.ends_with(".com"))
}

/// [`windows_batch_file`] for a command on this host, with `PATH` and
/// `PATHEXT` given (main's lookup). A bare name is looked for only in
/// absolute local `PATH` folders, never in a folder the command names, and
/// only the first match counts, as PowerShell takes it; a relative or network
/// `PATH` folder before a match leaves it unknown (yes). A bare name found
/// nowhere does not run unless a command first writes it into a `PATH` folder
/// or changes `PATH` (a PowerShell command on the `env:` drive asks under
/// strict rules, see `powershell_runs_code`). This is Corbanu's `PATH`: a
/// user's `shell_environment_policy` can give the shell another one. A name
/// named `.exe` or `.com` is not looked up: PowerShell may still find
/// `x.exe.cmd` when `x.exe` is missing, which a command creating it could as
/// well hold its commands in.
pub(super) fn windows_batch_file_in(
    program: &str,
    path: Option<std::ffi::OsString>,
    extensions: Option<String>,
) -> bool {
    let lower = program.trim_end_matches(['.', ' ']).to_ascii_lowercase();
    if lower.ends_with(".exe") || lower.ends_with(".com") {
        return false;
    }
    if program.contains(['/', '\\', ':', '.']) {
        return true;
    }
    let Some(path) = path else {
        return false;
    };
    let extensions = extensions.unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
    // An empty entry (`a;;b`, a trailing `;`) names no folder. One rooted
    // at the current drive (`\tools`, `/usr/bin`) depends on the command's
    // drive: unknown.
    for dir in std::env::split_paths(&path).filter(|dir| !dir.as_os_str().is_empty()) {
        if !dir.is_absolute() || dir.to_string_lossy().starts_with(['\\', '/']) {
            return true;
        }
        for extension in extensions
            .split(';')
            .filter(|extension| !extension.is_empty())
        {
            if dir.join(format!("{program}{extension}")).is_file() {
                return extension.eq_ignore_ascii_case(".cmd")
                    || extension.eq_ignore_ascii_case(".bat");
            }
        }
    }
    false
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
    role_words: &[Option<String>],
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
        // An unknown program, or a shell (#2585: PowerShell too), `eval` or
        // a builtin that runs code given unknown words, runs anything.
        if known.is_empty() || roles::runs_more_than_argv(&known[0], &role_words[start + 1..]) {
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
