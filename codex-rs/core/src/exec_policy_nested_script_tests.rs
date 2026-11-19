//! #2531 (D553): rules see into shell scripts nested in shell scripts
//! (`bash -lc "bash -c 'git push'"`), also through launchers
//! (`sudo bash -c "env bash -c '…'"`), down to a bound; deeper scripts ask.

use super::wrappers::MAX_SCRIPT_DEPTH;
use super::*;
use pretty_assertions::assert_eq;

const FORBID: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const PROMPT: &str = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;

async fn requirement(command: &[String], rules: &str) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: command.to_vec(),
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
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

/// `script` as one shell word, in whichever quotes keep it shorter (each
/// level of nesting multiplies the quotes it must escape).
fn quoted(script: &str) -> String {
    let single = format!("'{}'", script.replace('\'', r#"'"'"'"#));
    let double = format!("\"{}\"", script.replace('"', r#""'"'""#));
    if single.len() <= double.len() {
        single
    } else {
        double
    }
}

/// `inner` run by `layers` of shells, outermost first:
/// `nested(&["bash -lc", "sh -c"], "git push")` is
/// `["bash", "-lc", "sh -c 'git push'"]`.
fn nested(layers: &[&str], inner: &str) -> Vec<String> {
    let (outer, rest) = layers.split_first().expect("a shell");
    let script = rest.iter().rev().fold(inner.to_string(), |script, layer| {
        format!("{layer} {}", quoted(&script))
    });
    let mut command: Vec<String> = outer.split_whitespace().map(str::to_string).collect();
    command.push(script);
    command
}

const FORMS: &[&[&str]] = &[
    &["bash -lc", "bash -c"],
    &["sh -c", "zsh -c"],
    &["zsh -lc", "sh -c"],
    &["/bin/bash -lc", "/bin/sh -c"],
    &["bash -lc", "sh -c", "bash -c"],
    &["bash -lc", "bash -lc", "zsh -c", "sh -c"],
    // Launcher and shell chains, either way round.
    &["sudo bash -c", "env bash -c"],
    &["bash -lc", "env FOO=1 bash -c", "bash -c"],
    &["bash -lc", "bash -c", "sudo -n bash -c"],
    &["env bash -c", "nice -n 5 sh -c", "timeout 5 bash -c"],
];

#[tokio::test]
async fn rules_apply_inside_nested_scripts() {
    for layers in FORMS {
        for inner in ["git push", "echo hi && git push origin main"] {
            let command = nested(layers, inner);
            let result = requirement(&command, FORBID).await;
            assert!(forbidden(&result), "{command:?}: {result:?}");
            let result = requirement(&command, PROMPT).await;
            assert!(by_policy(&result), "{command:?}: {result:?}");
        }
    }
    // A nested script among other commands.
    let command = nested(&["bash -lc"], "echo hi && bash -c 'git push' && echo done");
    assert!(
        forbidden(&requirement(&command, FORBID).await),
        "{command:?}"
    );
    assert!(
        by_policy(&requirement(&command, PROMPT).await),
        "{command:?}"
    );
}

/// #2510 (D688): where names fold, a respelled shell anywhere in the chain
/// is read too, and so are the scripts nested in it.
#[cfg(any(target_os = "macos", windows))]
#[tokio::test]
async fn respelled_shells_in_nested_chains_are_read() {
    let deep = vec!["BASH -c"; MAX_SCRIPT_DEPTH + 3];
    for layers in [
        &["bash -lc", "BASH -c", "sh -c"][..],
        &["Bash -lc", "bash -c", "ZSH -c"],
        &["sudo BASH -c", "env Sh -c"],
        // Respelled shells are read at any depth, as before #2531.
        &deep,
    ] {
        let command = nested(layers, "git push");
        let result = requirement(&command, FORBID).await;
        assert!(forbidden(&result), "{command:?}: {result:?}");
        let result = requirement(&command, PROMPT).await;
        assert!(by_policy(&result), "{command:?}: {result:?}");
    }
}

#[tokio::test]
async fn nested_scripts_without_a_matching_command_keep_their_decision() {
    for layers in FORMS {
        for inner in ["git status", "git pull"] {
            let command = nested(layers, inner);
            for rules in [FORBID, PROMPT] {
                let result = requirement(&command, rules).await;
                assert!(
                    !forbidden(&result) && !by_policy(&result),
                    "{command:?}: {result:?}"
                );
            }
        }
    }
    // Words that merely look like a script are arguments, not commands.
    for inner in [
        r#"echo "bash -c 'git push'""#,
        "printf %s 'git push'",
        "bash -c 'echo git push'",
    ] {
        let command = nested(&["bash -lc"], inner);
        for rules in [FORBID, PROMPT] {
            let result = requirement(&command, rules).await;
            assert!(
                !forbidden(&result) && !by_policy(&result),
                "{command:?}: {result:?}"
            );
        }
    }
}

#[tokio::test]
async fn scripts_nested_beyond_the_bound_ask() {
    // The outermost script is read before any bound applies; below it,
    // MAX_SCRIPT_DEPTH more are read.
    let deepest_read = MAX_SCRIPT_DEPTH + 1;
    let shells = |count: usize| {
        let mut layers = vec!["bash -lc"];
        layers.extend(std::iter::repeat_n("bash -c", count - 1));
        layers
    };
    for (rules, inner) in [(FORBID, "git push"), (FORBID, "echo hi"), (PROMPT, "true")] {
        let read = requirement(&nested(&shells(deepest_read), inner), rules).await;
        let unread = requirement(&nested(&shells(deepest_read + 1), inner), rules).await;
        if inner == "git push" {
            assert!(forbidden(&read), "{read:?}");
        } else {
            assert_eq!(
                read,
                ExecApprovalRequirement::Skip {
                    bypass_sandbox: false,
                    proposed_execpolicy_amendment: None,
                },
                "{inner}"
            );
        }
        // Sync (D691): a command nested past the bound is not read, so where
        // a forbidden rule could match it a person decides.
        assert!(
            matches!(
                unread,
                ExecApprovalRequirement::NeedsApproval {
                    proposed_execpolicy_amendment: None,
                    ..
                }
            ) || (rules == FORBID
                && matches!(unread, ExecApprovalRequirement::NeedsHumanApproval { .. })),
            "{inner}: {unread:?}"
        );
    }
    // Scripts behind a launcher are read at any depth, as before #2531, so
    // a refused command stays refused past the bound (it also asks there).
    for count in [deepest_read, deepest_read + 2] {
        let mut layers = vec!["bash -lc"];
        layers.extend(std::iter::repeat_n("sudo -n bash -c", count - 1));
        let result = requirement(&nested(&layers, "git push"), FORBID).await;
        assert!(forbidden(&result), "{count}: {result:?}");
        let result = requirement(&nested(&layers, "git push"), PROMPT).await;
        assert!(by_policy(&result), "{count}: {result:?}");
        let mut layers = vec!["sudo -n bash -c"; count];
        layers[0] = "sudo bash -c";
        let result = requirement(&nested(&layers, "git push"), FORBID).await;
        assert!(forbidden(&result), "{count}: {result:?}");
    }
}

#[tokio::test]
async fn nested_scripts_propose_no_amendment() {
    // Approving the outer script's literal argv cannot approve the commands
    // inside it, so no "always allow" is offered for it.
    let command = nested(&["bash -lc", "bash -c"], "cargo build");
    let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: None,
        command: command.clone(),
        approval_policy: AskForApproval::UnlessTrusted,
        permission_profile: PermissionProfile::read_only(),
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await;
    assert!(
        matches!(
            result,
            ExecApprovalRequirement::NeedsApproval {
                proposed_execpolicy_amendment: None,
                ..
            }
        ),
        "{command:?}: {result:?}"
    );
}

/// #2524: PowerShell scripts are read on Windows only
/// (`exec_policy_windows_tests`); elsewhere none is read, so a PowerShell
/// command line asks under a prompt or forbidden rule (#2601), and a
/// respelled shell this platform runs as PowerShell (`PWSH` on macOS,
/// which folds name case) is treated exactly like the lowercase one.
#[cfg(not(windows))]
#[tokio::test]
async fn respelled_powershell_matches_lowercase_off_windows() {
    for (respelled, lowercase) in [("PWSH", "pwsh"), ("PowerShell.EXE", "powershell.exe")] {
        for rules in [FORBID, PROMPT] {
            let line = |shell: &str| {
                vec![
                    shell.to_string(),
                    "-NoProfile".to_string(),
                    "-Command".to_string(),
                    "git push".to_string(),
                ]
            };
            let lowercase_result = requirement(&line(lowercase), rules).await;
            assert!(
                !matches!(lowercase_result, ExecApprovalRequirement::Skip { .. }),
                "{lowercase}: {lowercase_result:?}"
            );
            if crate::security::tainted_action::policy_runs_powershell(respelled) {
                // The same decision; only the literal argv in an amendment
                // differs.
                assert_eq!(
                    std::mem::discriminant(&requirement(&line(respelled), rules).await),
                    std::mem::discriminant(&lowercase_result),
                    "{respelled}"
                );
            }
        }
    }
}
