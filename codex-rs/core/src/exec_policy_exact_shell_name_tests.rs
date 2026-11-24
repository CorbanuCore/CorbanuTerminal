//! #2734: rules read a shell's script only when the shell is named exactly
//! (`bash`, `/bin/bash`, `bash.exe`, `pwsh.exe`), or respelled as the
//! platform runs it (`BASH`, `PWSH.EXE`). A name read as a shell only
//! through another extension (`bash.sh`, `bash.foo.exe`, `pwsh.evil.exe`,
//! `pwsh.bat`, an NTFS stream) names another file, which may be one the
//! agent wrote: it stays one command, which asks wherever a prompt or
//! forbidden rule exists and offers no amendment.
use super::*;
use pretty_assertions::assert_eq;

const ALLOW_STATUS_FORBID_PUSH: &str = r#"prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")"#;

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(ToString::to_string).collect()
}

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

/// Asked with no amendment (sync, D691: a fresh human decision where a
/// prompt or forbidden rule could match).
fn asked(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        } | ExecApprovalRequirement::NeedsHumanApproval { .. }
    )
}

fn near_shells() -> Vec<&'static str> {
    let mut shells = vec![
        "bash.sh",
        "/tmp/bash.sh",
        "bash.foo.exe",
        "zsh.app",
        "sh.x.exe",
        "bash.exe.exe",
    ];
    if cfg!(windows) {
        shells.extend(["bash.exe:x.exe", r"C:\x\bash.foo.exe"]);
    }
    shells
}

#[test]
fn near_shell_scripts_are_not_read() {
    for shell in near_shells() {
        let command = words(&[shell, "-lc", "git status"]);
        assert_eq!(
            commands_for_exec_policy(&command).commands,
            vec![command.clone()],
            "{shell}"
        );
    }
    // Negative control: the exact names are read.
    for shell in ["bash", "/bin/bash", "bash.exe", "zsh", "sh"] {
        let command = words(&[shell, "-lc", "git status"]);
        assert_eq!(
            commands_for_exec_policy(&command).commands,
            vec![words(&["git", "status"])],
            "{shell}"
        );
    }
}

#[tokio::test]
async fn near_shell_scripts_ask_where_rules_exist() {
    for shell in near_shells() {
        for script in ["git status", "git push"] {
            let command = words(&[shell, "-lc", script]);
            let result = requirement(&command, ALLOW_STATUS_FORBID_PUSH).await;
            assert!(asked(&result), "{command:?}: {result:?}");
        }
        // Nested in a script and behind a launcher, too.
        for command in [
            words(&["bash", "-lc", &format!("'{shell}' -c 'git status'")]),
            words(&["env", shell, "-c", "git status"]),
        ] {
            let result = requirement(&command, ALLOW_STATUS_FORBID_PUSH).await;
            assert!(asked(&result), "{command:?}: {result:?}");
        }
    }
    // Negative control: an exact name reads the script, which the rules allow
    // or refuse.
    let result = requirement(
        &words(&["bash", "-lc", "git status"]),
        ALLOW_STATUS_FORBID_PUSH,
    )
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Skip { .. }),
        "{result:?}"
    );
    let result = requirement(
        &words(&["bash", "-lc", "git push"]),
        ALLOW_STATUS_FORBID_PUSH,
    )
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        "{result:?}"
    );
}

#[test]
fn near_shell_approval_keys_keep_their_argv() {
    for shell in near_shells() {
        let command = words(&[shell, "-lc", "git status"]);
        assert_eq!(
            crate::command_canonicalization::canonicalize_command_for_approval(&command),
            command,
            "{shell}"
        );
    }
}

/// Sync: `command` checked for a Windows executor on this host (any host),
/// in full access, in workspace-write, and under Aggressive with approvals
/// off.
async fn windows_requirements(command: &[String], rules: &str) -> Vec<ExecApprovalRequirement> {
    let mut results = Vec::new();
    for (approval_policy, permission_profile, live_aggressive) in [
        (
            AskForApproval::OnRequest,
            PermissionProfile::Disabled,
            false,
        ),
        (
            AskForApproval::OnRequest,
            PermissionProfile::workspace_write(),
            false,
        ),
        (AskForApproval::Never, PermissionProfile::read_only(), true),
    ] {
        results.push(
            ExecPolicyManager::new(policy_from_src(Some(rules)))
                .create_exec_approval_requirement_for_command_platform(
                    ExecApprovalRequest {
                        command,
                        approval_policy,
                        permission_profile,
                        environment_policy: None,
                        windows_sandbox_level: WindowsSandboxLevel::Disabled,
                        sandbox_permissions: SandboxPermissions::UseDefault,
                        prefix_rule: Some(command.to_vec()),
                        allow_prefix_rules: AllowPrefixRules::Honor,
                        cwd: None,
                        env: None,
                    },
                    DangerousCommandPlatform::Windows,
                    live_aggressive,
                )
                .await,
        );
    }
    results
}

/// Asked (no amendment) with approvals on, refused with them off.
fn asked_or_refused_off(results: &[ExecApprovalRequirement]) -> bool {
    let (on, off) = results.split_at(results.len() - 1);
    on.iter().all(asked) && matches!(off[0], ExecApprovalRequirement::Forbidden { .. })
}

/// Windows executor, on any host: PowerShell named through another
/// extension (`pwsh.evil.exe`, `pwsh.bat`) and bash named so (`bash.foo.exe`,
/// the stream `bash.exe:x.exe`) are not read; each asks wherever a rule
/// exists. Trimmed spellings Windows runs as PowerShell (`pwsh.exe.`) are
/// still read.
#[tokio::test]
async fn near_shells_ask_for_a_windows_executor() {
    let mut failures = Vec::new();
    for shell in [
        "pwsh.evil.exe",
        "pwsh.x.exe",
        "pwsh.bat",
        "powershell.cmd",
        "pwsh.exe.exe",
        r"C:\PS\pwsh.evil.exe",
        "POWERS~1.EXE",
        // Windows trims only the end of a name: these run other files.
        "pwsh .exe",
        "pwsh..exe",
        r"C:\ws\powershell .exe",
    ] {
        for script in ["git status", "git push"] {
            for command in [
                words(&[shell, "-Command", script]),
                words(&[shell, "-NoProfile", "-c", script]),
                words(&["bash", "-lc", &format!("'{shell}' -Command '{script}'")]),
                words(&["env", shell, "-Command", script]),
            ] {
                let results = windows_requirements(&command, ALLOW_STATUS_FORBID_PUSH).await;
                if !asked_or_refused_off(&results) {
                    failures.push(format!("{command:?}: {results:?}"));
                }
            }
        }
    }
    for shell in [
        "bash.foo.exe",
        "bash.exe:x.exe",
        "bash.sh",
        "bash .exe",
        "BASH..EXE",
    ] {
        for script in ["git status", "git push"] {
            let command = words(&[shell, "-lc", script]);
            let results = windows_requirements(&command, ALLOW_STATUS_FORBID_PUSH).await;
            if !asked_or_refused_off(&results) {
                failures.push(format!("{command:?}: {results:?}"));
            }
        }
    }
    // Negative control: Windows runs `pwsh.exe.` as `pwsh.exe`, which reads.
    for shell in ["pwsh.exe.", "PWSH.EXE", "pwsh.exe"] {
        let push = words(&[shell, "-Command", "git push"]);
        let results = windows_requirements(&push, ALLOW_STATUS_FORBID_PUSH).await;
        if !results
            .iter()
            .all(|result| matches!(result, ExecApprovalRequirement::Forbidden { .. }))
        {
            failures.push(format!("{push:?}: {results:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// #2794: as main reads it on a Windows host, a bash script command for a
/// Windows executor with words known only at run time, whose program is a
/// PowerShell command that rebinds a name or runs code (or that names a
/// drive that does), asks wherever a rule exists; for this host's own
/// platform it is an ordinary program, as before.
#[tokio::test]
async fn run_time_powershell_code_words_ask_for_a_windows_executor() {
    const FORBID_UNRELATED: &str = r#"prefix_rule(pattern=["zz-unrelated"], decision="forbidden")"#;
    let mut failures = Vec::new();
    for script in [
        "Import-Module \"$m\"",
        "Set-Alias f \"$x\"",
        "iex \"$x\"",
        "Set-Item alias:f \"$x\"",
        "Get-Item \"$x\" function:",
        "Add-Type -Path \"$p\"",
    ] {
        let command = words(&["bash", "-lc", script]);
        let results = windows_requirements(&command, FORBID_UNRELATED).await;
        if !asked_or_refused_off(&results) {
            failures.push(format!("{command:?}: {results:?}"));
        }
        if !cfg!(windows) {
            let result = requirement(&command, FORBID_UNRELATED).await;
            if !matches!(result, ExecApprovalRequirement::Skip { .. }) {
                failures.push(format!("host {command:?}: {result:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
