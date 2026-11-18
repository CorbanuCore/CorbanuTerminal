//! #2594, #2597, #2601 (D553): the reproducers of parser-boundary issues
//! that #2593's security reviews found, with close variants. Each script is
//! handed to bash, zsh and sh, behind a launcher and nested in another
//! script. With a forbidden or prompt `git push` rule each asks (a refusal
//! where the rule is forbidden) in every permission mode, with no
//! amendment, and is refused under Aggressive with approvals off: none runs
//! unasked. The fail-closed floor that covers them is #2613's: a script
//! outside the minimal lexical set (any here-document or here-string), or
//! one using a builtin that defines or runs code (`hash`, `alias`, `trap`,
//! `enable`, `mapfile`), asks whenever a prompt or forbidden rule exists.
use super::*;

const FORBID: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const PROMPT: &str = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;
/// #2597: an exact allow rule for the command as the dropped argv would
/// read it must not approve what really runs.
const ALLOW_AND_FORBID: &str = r#"
prefix_rule(pattern=["git"], decision="allow")
prefix_rule(pattern=[["bash", "sh", "zsh"]], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")
"#;

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(ToString::to_string).collect()
}

/// `script` as each shell is handed it.
fn spellings(script: &str) -> Vec<Vec<String>> {
    let nested = format!("sh -c '{}'", script.replace('\'', r#"'"'"'"#));
    vec![
        argv(&["bash", "-lc", script]),
        argv(&["zsh", "-lc", script]),
        argv(&["sh", "-c", script]),
        argv(&["/bin/bash", "-c", script]),
        argv(&["env", "A=1", "bash", "-c", script]),
        argv(&["sudo", "zsh", "-c", script]),
        argv(&["bash", "-lc", &nested]),
    ]
}

/// Every mode with approvals on, then Aggressive with approvals off.
async fn requirements(command: &[String], rules: &str) -> Vec<ExecApprovalRequirement> {
    let mut results = Vec::new();
    for permission_profile in [
        PermissionProfile::Disabled,
        PermissionProfile::read_only(),
        PermissionProfile::workspace_write(),
    ] {
        for sandbox_permissions in [
            SandboxPermissions::UseDefault,
            SandboxPermissions::RequireEscalated,
        ] {
            results.push(
                exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                    policy_src: Some(rules.to_string()),
                    command: command.to_vec(),
                    approval_policy: AskForApproval::OnRequest,
                    permission_profile: permission_profile.clone(),
                    sandbox_permissions,
                    // An amendment could be derived from it: none may be.
                    prefix_rule: Some(command.to_vec()),
                })
                .await,
            );
        }
    }
    results.push(aggressive_never(command, rules).await);
    results
}

async fn aggressive_never(command: &[String], rules: &str) -> ExecApprovalRequirement {
    ExecPolicyManager::new(policy_from_src(Some(rules)))
        .create_exec_approval_requirement(
            ExecApprovalRequest {
                command,
                approval_policy: AskForApproval::Never,
                permission_profile: PermissionProfile::read_only(),
                environment_policy: None,
                windows_sandbox_level: if cfg!(windows) {
                    WindowsSandboxLevel::Elevated
                } else {
                    WindowsSandboxLevel::Disabled
                },
                sandbox_permissions: SandboxPermissions::UseDefault,
                prefix_rule: Some(command.to_vec()),
                allow_prefix_rules: AllowPrefixRules::Honor,
            },
            /*live_aggressive*/ true,
        )
        .await
}

/// Collects every (command, rules, mode) that does not ask or refuse; the
/// last mode (Aggressive with approvals off) must refuse.
async fn failures_for(commands: Vec<Vec<String>>, rule_sets: &[&str]) -> Vec<String> {
    let mut failures = Vec::new();
    for command in commands {
        for &rules in rule_sets {
            let results = requirements(&command, rules).await;
            let last = results.len() - 1;
            for (mode, result) in results.into_iter().enumerate() {
                // Sync (D691): where a forbidden rule could match, a fresh
                // human decision; a prompt rule alone asks as on main.
                let ok = match &result {
                    ExecApprovalRequirement::Forbidden { .. } => rules != PROMPT || mode == last,
                    ExecApprovalRequirement::NeedsHumanApproval { .. } => {
                        rules != PROMPT && mode != last
                    }
                    ExecApprovalRequirement::NeedsApproval {
                        proposed_execpolicy_amendment: None,
                        ..
                    } => rules == PROMPT && mode != last,
                    _ => false,
                };
                if !ok {
                    failures.push(format!("{rules} mode {mode} {command:?}: {result:?}"));
                }
            }
        }
    }
    failures
}

async fn assert_scripts_ask(scripts: &[&str], rule_sets: &[&str]) {
    let commands = scripts
        .iter()
        .flat_map(|script| spellings(script))
        .collect();
    let failures = failures_for(commands, rule_sets).await;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// #2594: Bash removes quotes anywhere in a here-document delimiter
/// (`E"O"F` ends at `EOF`); the tree-sitter scanner only at the edges, so
/// it may read the body as ending later and hide `git push` in it.
#[tokio::test]
async fn issue_2594_mixed_quoted_heredoc_delimiters_ask() {
    assert_scripts_ask(
        &[
            // The reproducer, and the same body fed to a shell.
            "cat <<E\"O\"F\necho '\nEOF\ngit push\n'\nE\"O\"F",
            "bash <<E\"O\"F\necho '\nEOF\ngit push\n'\nE\"O\"F",
            "sh <<E\"O\"F\necho '\nEOF\ngit push\n'\nE\"O\"F",
            // Other quotings and escapes inside the delimiter.
            "cat <<E'O'F\necho '\nEOF\ngit push\n'\nE'O'F",
            "cat <<\"E\"OF\necho '\nEOF\ngit push\n'\n\"E\"OF",
            "cat <<E\"OF\"\necho '\nEOF\ngit push\n'\nE\"OF\"",
            "cat <<E\\OF\necho '\nEOF\ngit push\n'\nE\\OF",
            "cat <<$'EOF'\necho '\nEOF\ngit push\n'\n$'EOF'",
            "cat <<-E\"O\"F\n\techo '\n\tEOF\n\tgit push\n\t'\n\tE\"O\"F",
            "cat <<E\"O\"F | bash\necho '\nEOF\ngit push\n'\nE\"O\"F",
            // Terminators the scanner and the shells match differently: a
            // prefix, trailing text, indentation, a line continuation.
            "cat <<EOF\necho '\nEOFX\ngit push\n'\nEOF",
            "cat <<EOF\necho '\nEOF \ngit push\n'\nEOF",
            "cat <<EOF\necho '\n  EOF\ngit push\n'\nEOF",
            "cat <<EOF\nx\nEO\\\nF\ngit push\nEOF",
        ],
        &[FORBID, PROMPT],
    )
    .await;
}

/// #2597: words after a here-document marker are argv, which the syntax
/// tree stores under the redirect: `git <<A push` runs `git push`, and
/// `bash <<A -c 'git push'` runs the `-c` script, not the body.
#[tokio::test]
async fn issue_2597_argv_after_heredoc_marker_asks() {
    assert_scripts_ask(
        &[
            // The reproducers.
            "git <<'A' push --force\nA",
            "bash <<'A' -c 'git push'\necho safe\nA",
            // Close variants: other quotings, `<<-`, here-strings, a file
            // descriptor, two here-documents, launchers, a pipeline.
            "git <<A push\nA",
            "git <<\"A\" push\nA",
            "git <<-A push\n\tA",
            "git 0<<A push\nA",
            "git <<A <<B push\nA\nB",
            "git <<<'' push --force",
            "git <<<x push",
            "sh <<'A' -c 'git push'\nA",
            "zsh <<'A' -c 'git push'\necho safe\nA",
            "bash <<<'echo safe' -c 'git push'",
            "sudo git <<A push\nA",
            "env B=1 git <<A push\nA",
            "command git <<A push\nA",
            "exec git <<A push\nA",
            "time git <<A push\nA",
            "echo x | git <<A push\nA",
            "sudo bash <<'A' -c 'git push'\necho safe\nA",
            "echo safe; git <<A push\nA",
        ],
        &[FORBID, PROMPT, ALLOW_AND_FORBID],
    )
    .await;
}

/// #2601: a builtin that rebinds a command name (`hash -p`, an alias,
/// zsh's `hash name=path`) or runs text later (`trap`, `mapfile -C`,
/// `enable -f`) can hide which command runs, with or without stdin.
#[tokio::test]
async fn issue_2601_name_rebinding_and_callbacks_ask() {
    assert_scripts_ask(
        &[
            // The reproducers.
            "hash -p /bin/bash f; echo 'git push' | f",
            "shopt -s expand_aliases\nalias f=bash\necho 'git push' | f",
            "trap 'sh -s' EXIT; echo 'git push'",
            // Without stdin: the rebound name or callback runs it.
            "hash -p /usr/bin/git f; f push",
            "hash -p /bin/sh f\nf -c 'git push'",
            "builtin hash -p /bin/bash f; echo 'git push' | f",
            "command hash -p /bin/bash f; echo 'git push' | f",
            "'hash' -p /bin/bash f; echo 'git push' | f",
            "h\\ash -p /bin/bash f; echo 'git push' | f",
            "hash -p /bin/bash f && echo 'git push' | f",
            "hash f=/bin/bash; echo 'git push' | f",
            "hash f=/usr/bin/git; f push",
            "shopt -s expand_aliases; alias f=git\nf push",
            "alias f=bash\necho 'git push' | f",
            "alias f='git push'\nf",
            "set -o posix\nalias f=git\nf push",
            "builtin alias f=git\nf push",
            "trap 'git push' EXIT",
            "trap -- 'git push' EXIT",
            "trap 'git push' ERR; false",
            "trap 'git push' DEBUG; true",
            "builtin trap 'git push' EXIT",
            "command trap 'git push' EXIT",
            "echo 'git push' | trap 'bash' EXIT",
            "echo x | mapfile -c 1 -C 'git push' a",
            "echo x | readarray -C 'git push' -c 1 a",
            "enable -f ./lib.so f; f push",
            "enable -n git; git push",
            "autoload -U f; f push",
            "sched +0 git push",
        ],
        &[FORBID, PROMPT],
    )
    .await;
}

/// #2613 review follow-up: a shell running a named script file keeps an
/// exact allow rule for its argv (`script_file_runs_keep_allow_rules_and_
/// ask_with_strict_rules`), but the file is unread, so with a prompt or
/// forbidden rule it asks behind launchers, nested, respelled, with
/// options and with its input redirected too.
#[tokio::test]
async fn script_file_run_shapes_ask_with_strict_rules() {
    const ALLOW_FILE_AND_FORBID: &str = r#"
prefix_rule(pattern=["sh", "./build.sh"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")
"#;
    assert_scripts_ask(
        &[
            "sh ./build.sh",
            "sh ./build.sh arg && echo done",
            "sudo sh ./build.sh",
            "env A=1 sh ./build.sh",
            "timeout 5 sh ./build.sh",
            "nice sh ./build.sh",
            "command sh ./build.sh",
            "exec sh ./build.sh",
            "bash -c 'sh ./build.sh'",
            "eval 'sh ./build.sh'",
            "/bin/sh ./build.sh",
            "dash ./build.sh",
            "sh -e ./build.sh",
            "sh -- ./build.sh",
            "sh ./build.sh <<EOF\nx\nEOF",
            "sh ./build.sh <<< x",
            "echo x | sh ./build.sh",
            "sh ./build.sh < in",
            "fish ./x.fish",
        ],
        &[FORBID, PROMPT, ALLOW_FILE_AND_FORBID],
    )
    .await;
}

/// Controls: the fail-closed floor is not unconditional. Without a prompt
/// or forbidden rule nothing changes (the reproducers run as before), and
/// plain commands and `hash -r` stay read precisely.
#[tokio::test]
async fn parser_issue_controls_stay_precise() {
    for script in [
        "hash -p /bin/bash f; echo 'echo hi' | f",
        "git <<'A' status\nA",
        "trap 'echo hi' EXIT",
    ] {
        let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: None,
            command: argv(&["bash", "-lc", script]),
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{script:?}: {result:?}"
        );
    }
    for script in ["hash -r; git status", "git status && echo safe"] {
        let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(FORBID.to_string()),
            command: argv(&["bash", "-lc", script]),
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{script:?}: {result:?}"
        );
    }
}

/// #2601 in PowerShell: an alias, a function or a PowerShell drive named
/// by a literal argument can rebind a command name, and
/// `Invoke-Expression`, `Import-Module`, `Add-Type` and script-file jobs run
/// code that is not their argv. On Windows the trusted parser reads these
/// scripts as plain commands, so each such command fails closed as bash's
/// `alias` does (a partial list: #2661). Elsewhere no PowerShell script is
/// read, so each asks as an unread script.
#[tokio::test]
async fn issue_2601_powershell_name_rebinding_and_code_asks() {
    // The trusted parser reads the reproducer in full, as plain commands.
    #[cfg(windows)]
    {
        use codex_shell_command::powershell::PowershellScriptRead;
        use codex_shell_command::powershell::read_powershell_command_script;
        use pretty_assertions::assert_eq;
        assert_eq!(
            read_powershell_command_script(&argv(&[
                "powershell.exe",
                "-Command",
                "Set-Alias f git; f push"
            ])),
            Some(PowershellScriptRead::Commands(vec![
                argv(&["Set-Alias", "f", "git"]),
                argv(&["f", "push"]),
            ]))
        );
    }
    let scripts = [
        "Set-Alias f git; f push",
        "set-alias -Name f -Value git; f push",
        "Microsoft.PowerShell.Utility\\Set-Alias f git; f push",
        "New-Alias f git; f push",
        "sal f git; f push",
        "nal f git; f push",
        "Import-Alias ./aliases.csv; f push",
        "Set-Item alias:f git; f push",
        "New-Item -Path Alias:f -Value git; f push",
        "Set-Item -Path Microsoft.PowerShell.Core\\Alias::f -Value git; f push",
        "Set-Item function:f 'git push'; f",
        "Set-Location alias:; Set-Item f git; f push",
        "New-PSDrive -Name a -PSProvider Alias -Root ''; Set-Item a:f git; f push",
        "Invoke-Expression 'git push'",
        "iex 'git push'",
        "Import-Module ./m.psm1; f",
        "ipmo ./m.psm1; f",
        "Add-Type -Path ./m.dll",
        "Invoke-Command -FilePath ./x.ps1",
        "Start-Job -FilePath ./x.ps1",
        // .NET may fold these non-ASCII letters to ASCII (review round 1).
        "\u{131}ex 'git push'",
        "\u{131}nvoke-expression 'git push'",
        "\u{17f}al f git; f push",
        "Set-Item al\u{131}as:f git; f push",
        "\u{17f}et-location funct\u{131}on:; Set-Item f 'git push'; f",
    ];
    let mut commands = Vec::new();
    for script in scripts {
        for shell in ["powershell.exe", "pwsh.exe"] {
            commands.push(argv(&[shell, "-Command", script]));
        }
    }
    let failures = failures_for(commands, &[FORBID, PROMPT]).await;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
