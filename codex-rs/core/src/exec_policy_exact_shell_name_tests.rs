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

fn asked(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        }
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

/// Windows: PowerShell named through another extension is not read.
#[cfg(windows)]
mod powershell {
    use super::*;
    use codex_shell_command::powershell::PowershellScriptRead;
    use codex_shell_command::powershell::read_powershell_command_script;
    use pretty_assertions::assert_eq;

    const NEAR_POWERSHELLS: &[&str] = &[
        "pwsh.evil.exe",
        "pwsh.x.exe",
        "pwsh.bat",
        "powershell.cmd",
        "pwsh.exe.exe",
        r"C:\PS\pwsh.evil.exe",
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.bat",
    ];

    fn trusted_pwsh_reads() -> bool {
        matches!(
            read_powershell_command_script(&words(&["pwsh.exe", "-c", "git push"])),
            Some(PowershellScriptRead::Commands(_))
        )
    }

    #[tokio::test]
    async fn near_powershell_scripts_ask_where_rules_exist() {
        for &shell in NEAR_POWERSHELLS {
            for script in ["git status", "git push"] {
                for command in [
                    words(&[shell, "-Command", script]),
                    words(&[shell, "-NoProfile", "-c", script]),
                    words(&["bash", "-lc", &format!("'{shell}' -Command '{script}'")]),
                    words(&[
                        "pwsh.exe",
                        "-Command",
                        &format!("& '{shell}' -Command '{script}'"),
                    ]),
                ] {
                    let result = requirement(&command, ALLOW_STATUS_FORBID_PUSH).await;
                    assert!(asked(&result), "{command:?}: {result:?}");
                }
            }
            let command = words(&[shell, "-Command", "git status"]);
            assert_eq!(
                commands_for_exec_policy(&command).commands,
                vec![command.clone()],
                "{shell}"
            );
        }
    }

    /// Windows trims trailing dots and spaces, so `powershell.exe.` runs
    /// `powershell.exe`: it is respelled and read (Windows PowerShell is
    /// always present). Not `powershell .exe`, which names another file.
    #[tokio::test]
    async fn trimmed_powershell_names_are_still_read() {
        let mut shells = vec!["powershell.exe.", "POWERSHELL.EXE", "powershell.exe "];
        if trusted_pwsh_reads() {
            shells.extend(["pwsh.exe.", "PWSH.EXE"]);
        }
        for shell in [r"C:\ws\powershell .exe", "powershell..exe", "pwsh .exe"] {
            let status = words(&[shell, "-NoProfile", "-Command", "git status"]);
            let result = requirement(&status, ALLOW_STATUS_FORBID_PUSH).await;
            assert!(asked(&result), "{status:?}: {result:?}");
        }
        for shell in shells {
            let status = words(&[shell, "-NoProfile", "-Command", "git status"]);
            let result = requirement(&status, ALLOW_STATUS_FORBID_PUSH).await;
            assert!(
                matches!(result, ExecApprovalRequirement::Skip { .. }),
                "{status:?}: {result:?}"
            );
            let push = words(&[shell, "-NoProfile", "-Command", "git push"]);
            let result = requirement(&push, ALLOW_STATUS_FORBID_PUSH).await;
            assert!(
                matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                "{push:?}: {result:?}"
            );
        }
    }
}
