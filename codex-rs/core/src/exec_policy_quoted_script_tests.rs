//! #2543 (D553, D688): rules see the commands of a shell script whatever
//! its quoting (`bash -lc "'git' push"` runs `git push`). A script rules
//! cannot read as plain commands is still read for prompt and forbidden
//! rules; where a command cannot be known before the script runs
//! (`$cmd push`), any such rule that could match asks for approval.

use super::*;

const FORBID: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const PROMPT: &str = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;
const ALLOW_GIT: &str = r#"prefix_rule(pattern=["git"], decision="allow")"#;
const FORBID_RM: &str = r#"prefix_rule(pattern=["rm"], decision="forbidden")"#;

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(ToString::to_string).collect()
}

/// The three settings of the issue's table.
#[derive(Clone, Copy, Debug)]
enum Mode {
    /// Full access, on-request approvals.
    FullAccess,
    WorkspaceWrite,
    /// Workspace-write, the command asking to run outside the sandbox.
    RequireEscalated,
}

const MODES: [Mode; 3] = [
    Mode::FullAccess,
    Mode::WorkspaceWrite,
    Mode::RequireEscalated,
];

async fn requirement_in(mode: Mode, command: &[String], rules: &str) -> ExecApprovalRequirement {
    let (permission_profile, sandbox_permissions) = match mode {
        Mode::FullAccess => (PermissionProfile::Disabled, SandboxPermissions::UseDefault),
        Mode::WorkspaceWrite => (
            PermissionProfile::workspace_write(),
            SandboxPermissions::UseDefault,
        ),
        Mode::RequireEscalated => (
            PermissionProfile::workspace_write(),
            SandboxPermissions::RequireEscalated,
        ),
    };
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: command.to_vec(),
        approval_policy: AskForApproval::OnRequest,
        permission_profile,
        sandbox_permissions,
        prefix_rule: None,
    })
    .await
}

async fn requirement(command: &[String], rules: &str) -> ExecApprovalRequirement {
    requirement_in(Mode::FullAccess, command, rules).await
}

fn forbidden(result: &ExecApprovalRequirement) -> bool {
    matches!(result, ExecApprovalRequirement::Forbidden { .. })
}

fn by_policy(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            reason: Some(reason),
            proposed_execpolicy_amendment: None,
        } if reason.ends_with("requires approval by policy")
    )
}

/// Asks, offering no amendment: one for the script's literal argv could
/// not approve the commands in it. Sync (D691): where only a forbidden rule
/// could match, a person decides.
fn asks(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        } | ExecApprovalRequirement::NeedsHumanApproval { .. }
    )
}

fn runs(result: &ExecApprovalRequirement) -> bool {
    matches!(result, ExecApprovalRequirement::Skip { .. })
}

/// Scripts that run `git push`, quoted in every way the issue lists, also
/// among other commands and in syntax rules did not read before.
const QUOTED_SCRIPTS: &[&str] = &[
    "'git' push",
    "\"git\" push",
    "g'i't push",
    "g\"i\"t pu'sh'",
    "echo hi; 'git' push",
    "echo hi && \"git\" push origin main",
    "g\\it push",
    "$'git' push",
    // Not plain commands: read for prompt and forbidden rules.
    "'git' push > /dev/null",
    "\"git\" push 2>&1 | tail -n 5",
    "x=$('git' push); echo \"$x\"",
    "for f in a b; do 'git' push; done",
    "A=1 g'i't push",
    "if true; then \"git\" push; fi",
    "echo \"$HOME\" && 'git' push",
    "sudo 'git' push",
    "env FOO=1 \"git\" push",
    "nice -n 5 g'i't push > log",
];

/// `script` run by each shell and launcher chain.
fn shells(script: &str) -> Vec<Vec<String>> {
    let mut commands: Vec<Vec<String>> = [
        &["bash", "-lc"][..],
        &["bash", "-c"],
        &["sh", "-c"],
        &["zsh", "-lc"],
        &["dash", "-c"],
        &["/bin/bash", "-lc"],
        &["sudo", "bash", "-c"],
        &["env", "FOO=1", "sh", "-c"],
        &["timeout", "5", "zsh", "-c"],
        &["nice", "-n", "5", "dash", "-c"],
    ]
    .iter()
    .map(|shell| {
        let mut command = words(shell);
        command.push(script.to_string());
        command
    })
    .collect();
    // A script nested in a script.
    let nested = format!("bash -c '{}'", script.replace('\'', r#"'"'"'"#));
    commands.push(words(&["sh", "-c", &nested]));
    commands
}

#[tokio::test]
async fn quoted_words_do_not_hide_commands_from_rules() {
    for script in QUOTED_SCRIPTS {
        for command in shells(script) {
            for mode in MODES {
                let result = requirement_in(mode, &command, FORBID).await;
                assert!(forbidden(&result), "{mode:?} {command:?}: {result:?}");
            }
            let result = requirement(&command, PROMPT).await;
            assert!(by_policy(&result), "{command:?}: {result:?}");
        }
    }
}

/// The issue's table, row by row: refused under every setting, like
/// `bash -lc 'git push'`.
#[tokio::test]
async fn issue_table() {
    for script in [
        "git push",
        "'git' push",
        "\"git\" push",
        "g'i't push",
        "'/usr/bin/git' push",
        "echo hi; 'git' push",
    ] {
        let command = words(&["bash", "-lc", script]);
        for mode in MODES {
            let result = requirement_in(mode, &command, FORBID).await;
            assert!(forbidden(&result), "{mode:?} {command:?}: {result:?}");
        }
    }
}

/// A command whose words are built as the script runs asks whenever a
/// prompt or forbidden rule could match it.
#[tokio::test]
async fn unknown_commands_ask_when_a_rule_could_match() {
    for script in [
        "$cmd push",
        "\"$cmd\" push",
        "\"$(echo git)\" push",
        "`echo git` push",
        "git \"$(echo push)\"",
        "git $sub",
        "git \"$(echo status)\" --short",
        "/usr/bin/gi? push",
        "/usr/bin/gi[t] push",
        "/usr/bin/g*t push",
        "~/bin/git push",
        "git {pu,}sh",
        "{git,} push",
        "sudo $cmd push",
        "sudo -u \"$user\" $cmd",
        "env $assignment git push",
        "nice -n \"$n\" $cmd push",
        // A launcher's operand may hold the program (review r1).
        "timeout $(echo 5 git push) origin",
        "nice -n $(echo 5 git push) origin",
        "sudo -u $u true",
        "env -u $v true",
        "chroot $d true",
        // The issue_766 nested-run test's script before it named its home
        // literally (cli/tests/nested_moderate.rs).
        "env -u CORBANU_HOME CODEX_HOME=\"$PWD/made-up\" corbanu exec hi 2>&1; echo \"exit=$?\"",
        // zsh's `=git` and ksh's `@(…)` are not what they read as.
        "=git push",
        "gi@(t) push",
        "bash -c \"$script\"",
        "sh -c 'git push' \"$x\"",
        "eval_me=1; $(printf git) push",
        "echo 'unterminated",
    ] {
        for command in shells(script) {
            for rules in [FORBID, PROMPT] {
                let result = requirement(&command, rules).await;
                assert!(asks(&result), "{command:?} {rules}: {result:?}");
            }
            // Refused where the policy allows no prompt.
            let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                policy_src: Some(FORBID.to_string()),
                command: command.clone(),
                approval_policy: AskForApproval::Never,
                permission_profile: PermissionProfile::Disabled,
                sandbox_permissions: SandboxPermissions::UseDefault,
                prefix_rule: None,
            })
            .await;
            assert!(forbidden(&result), "{command:?}: {result:?}");
        }
    }
    // A known prefix a rule matches is refused outright.
    for script in ["git push \"$(echo origin)\"", "git push $remote > log"] {
        let command = words(&["bash", "-lc", script]);
        let result = requirement(&command, FORBID).await;
        assert!(forbidden(&result), "{command:?}: {result:?}");
    }
}

/// A shell script rules do not read as `-c SCRIPT` asks when any prompt or
/// forbidden rule exists.
#[tokio::test]
async fn unread_shell_scripts_ask_when_rules_exist() {
    for command in [
        words(&["bash", "-c", "git push", "name"]),
        words(&["bash", "-ec", "git push"]),
        words(&["bash", "--norc", "-c", "git push"]),
        words(&["fish", "-c", "git push"]),
        words(&["bash", "-lc", "bash -c 'git push' name"]),
        words(&["bash", "-lc", "fish -c 'git push' > log"]),
    ] {
        let result = requirement(&command, FORBID).await;
        assert!(asks(&result), "{command:?}: {result:?}");
        // Without a prompt or forbidden rule, nothing changes.
        let result = requirement(&command, ALLOW_GIT).await;
        assert!(runs(&result), "{command:?}: {result:?}");
        // Sync review r1 (#2575): where the fallback already asks for the
        // same argv, still no amendment for it is offered.
        let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(PROMPT.to_string()),
            command: command.clone(),
            approval_policy: AskForApproval::UnlessTrusted,
            permission_profile: PermissionProfile::workspace_write(),
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        assert!(asks(&result), "{command:?}: {result:?}");
    }
}

/// Negative controls: scripts that cannot run what a rule matches keep the
/// decision they had.
#[tokio::test]
async fn scripts_no_rule_could_match_keep_their_decision() {
    for script in [
        "ls > /dev/null",
        "cargo build 2>&1 | tail -n 5",
        "'echo' hi; \"ls\" -la",
        "git status > /dev/null",
        "git log --format='%H' | head -n 1",
        "echo 'git push'",
        "printf '%s\\n' \"git\" push",
    ] {
        for command in shells(script) {
            for rules in [FORBID, PROMPT] {
                let result = requirement(&command, rules).await;
                assert!(runs(&result), "{command:?} {rules}: {result:?}");
            }
        }
    }
    // #2548 (coordinator, review round 7) put `$name`, `${…}` and `$(…)`
    // outside the minimal lexical set; D713 reads them again as arguments
    // and as scripts of the same set, so these run as before.
    for script in [
        "echo \"$HOME\"",
        "for f in *.rs; do wc -l \"$f\"; done",
        "x=$(git log -1); echo \"$x\"",
    ] {
        for command in shells(script) {
            for rules in [FORBID, PROMPT, ""] {
                let result = requirement(&command, rules).await;
                assert!(runs(&result), "{command:?} {rules}: {result:?}");
            }
        }
    }
    // A command built at run time asks only where a prompt or forbidden
    // rule exists at all.
    for script in ["$cmd push", "echo 'unterminated"] {
        let command = words(&["bash", "-lc", script]);
        let result = requirement(&command, ALLOW_GIT).await;
        assert!(runs(&result), "{command:?}: {result:?}");
        let result = requirement(&command, "").await;
        assert!(runs(&result), "{command:?}: {result:?}");
        let result = requirement(&command, FORBID_RM).await;
        assert!(asks(&result), "{command:?}: {result:?}");
    }
}

/// Allow rules never approve a script read only for prompts and refusals.
#[tokio::test]
async fn allow_rules_do_not_approve_scripts_read_in_part() {
    let command = words(&["bash", "-lc", "'git' status > /dev/null"]);
    let result = requirement_in(Mode::RequireEscalated, &command, ALLOW_GIT).await;
    assert!(
        matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
        "{result:?}"
    );
    // A plain script with a quoted name is read in full, so it is.
    let command = words(&["bash", "-lc", "'git' status"]);
    let result = requirement_in(Mode::RequireEscalated, &command, ALLOW_GIT).await;
    assert!(runs(&result), "{result:?}");
}

/// D696 and D692 hold through quoted script words: an allow rule approves
/// only the exact spelling (`'GIT' push` is not `git`), on every platform;
/// where programs are found without regard to case (macOS, Windows), a
/// forbidden rule refuses it.
#[tokio::test]
async fn quoted_words_keep_rule_case_handling() {
    let unless_trusted = async |command: &[String], rules: &str| {
        exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(rules.to_string()),
            command: command.to_vec(),
            // Asks for every command no rule allows.
            approval_policy: AskForApproval::UnlessTrusted,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await
    };
    // The control: the exact spelling is allowed.
    let command = words(&["bash", "-lc", "'git' push"]);
    let result = unless_trusted(&command, ALLOW_GIT).await;
    assert!(runs(&result), "{command:?}: {result:?}");
    for script in [
        "'GIT' push",
        "\"Git\" push",
        "G'I'T push",
        "echo hi; 'GIT' push",
    ] {
        let command = words(&["bash", "-lc", script]);
        let result = unless_trusted(&command, ALLOW_GIT).await;
        assert!(
            matches!(
                result,
                ExecApprovalRequirement::NeedsApproval { .. }
                    | ExecApprovalRequirement::NeedsHumanApproval { .. }
            ),
            "{command:?}: {result:?}"
        );
        let result = requirement(&command, FORBID).await;
        if codex_execpolicy::PROGRAM_NAMES_FOLD_CASE {
            assert!(forbidden(&result), "{command:?}: {result:?}");
        } else {
            assert!(runs(&result), "{command:?}: {result:?}");
        }
    }
}

/// #2510: a quoted drive path names the shell Windows runs.
#[cfg(windows)]
#[tokio::test]
async fn quoted_windows_shell_paths_are_read() {
    for script in [
        r"'C:\Tools\Zsh.exe' -c 'git push'",
        r#""C:\Program Files\Git\bin\BASH.EXE" -lc "'git' push""#,
    ] {
        let command = words(&["bash", "-lc", script]);
        let result = requirement(&command, FORBID).await;
        assert!(forbidden(&result), "{command:?}: {result:?}");
    }
}
