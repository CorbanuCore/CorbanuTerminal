//! #2510 (D553/D670/D688): rules see inside shell scripts and behind
//! launchers however the platform spells the program, and fail closed where
//! the spelling cannot be read.

use super::*;
use pretty_assertions::assert_eq;

const CASE_INSENSITIVE: bool = cfg!(any(target_os = "macos", windows));

const RULES: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;

async fn requirement(line: &str) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(RULES.to_string()),
        command: shlex::split(line).unwrap(),
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

async fn escalated(line: &str) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(RULES.to_string()),
        command: shlex::split(line).unwrap(),
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::workspace_write(),
        sandbox_permissions: SandboxPermissions::RequireEscalated,
        prefix_rule: Some(shlex::split(line).unwrap()),
    })
    .await
}

fn refused(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::Forbidden { reason }
            if reason.contains("policy forbids commands starting with `git push`")
    )
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

/// Asks with no remembered approval on offer. Sync: where a forbidden or
/// prompt-user rule could match a command the check cannot read (an open
/// `xargs`, an unreadable launcher name), D691/#2519 ask a human instead
/// (`NeedsHumanApproval`, which offers no remembered approval either).
fn asks(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        } | ExecApprovalRequirement::NeedsHumanApproval { .. }
    )
}

/// Shells spelled as macOS and Windows run them, with their flag. Only
/// spellings the shell parser does not read already (`detect_shell_type`
/// reads `zsh.exe` and `sh.exe.` by their file stem), so the list cannot
/// drift into names that unwrap without this change.
fn shells() -> Vec<String> {
    let mut shells = [
        "BASH -lc",
        "Bash -c",
        "ZSH -c",
        "SH -c",
        "/bin/Sh -c",
        "/BIN/BASH -lc",
        "\u{17f}h -c",
        "Z\u{17f}h -lc",
    ]
    .map(str::to_string)
    .to_vec();
    if cfg!(windows) {
        shells.extend([
            "Bash.exe -c".to_string(),
            "BASH.EXE -lc".to_string(),
            "SH.EXE. -c".to_string(),
            // Quoted: shlex reads an unquoted `\` as an escape.
            "'C:\\Tools\\Zsh.exe' -c".to_string(),
            "'C:\\Tools\\zsh.exe' -c".to_string(),
            "sh.exe. -c".to_string(),
        ]);
    }
    shells.retain(|shell| {
        let program = shlex::split(shell).unwrap().remove(0);
        codex_shell_command::shell_detect::detect_shell_type(std::path::Path::new(&program))
            .is_none()
    });
    assert!(shells.len() >= 8, "{shells:?}");
    shells
}

#[tokio::test]
async fn issue2510_shell_scripts_unwrap_in_any_case() {
    // Lower case unwraps everywhere (the control).
    for shell in ["bash -lc", "/bin/sh -c", "zsh -c"] {
        assert!(refused(&requirement(&format!("{shell} 'git push'")).await));
    }
    for shell in shells() {
        for script in ["git push", "echo hi && git push", "git status; git push"] {
            let command = format!("{shell} '{script}'");
            for result in [requirement(&command).await, escalated(&command).await] {
                assert_eq!(refused(&result), CASE_INSENSITIVE, "{command}: {result:?}");
            }
        }
        let command = format!("{shell} 'git fetch'");
        for result in [requirement(&command).await, escalated(&command).await] {
            assert_eq!(
                by_policy(&result),
                CASE_INSENSITIVE,
                "{command}: {result:?}"
            );
        }
        // Behind a launcher in either spelling.
        for launcher in ["sudo -n", "SUDO", "env FOO=1"] {
            let command = format!("{launcher} {shell} 'git push'");
            let result = requirement(&command).await;
            assert_eq!(refused(&result), CASE_INSENSITIVE, "{command}: {result:?}");
        }
        // Nested inside a lower-case script. (Its plain-command parser
        // does not read a quoted drive path such as `'C:\Tools\Zsh.exe'`.)
        if !shell.contains(':') {
            let command = format!("bash -lc \"{shell} 'git push'\"");
            let result = requirement(&command).await;
            assert_eq!(refused(&result), CASE_INSENSITIVE, "{command}: {result:?}");
        }
    }
}

#[tokio::test]
async fn issue2510_respelled_shell_offers_no_remembered_approval() {
    // The literal argv still gets its own fallback; an amendment for it
    // could not approve the script's commands.
    for shell in shells() {
        let command = format!("{shell} 'git status'");
        let result = escalated(&command).await;
        if CASE_INSENSITIVE {
            assert!(asks(&result), "{command}: {result:?}");
        }
    }
}

#[tokio::test]
async fn issue2510_unreadable_launcher_names_ask() {
    let unsure: &[&str] = if cfg!(target_os = "macos") {
        &[
            "su\u{200d}do -n",
            "\u{200b}env",
            "/usr/bin/xarg\u{feff}s",
            "BA\u{200c}SH -c",
        ]
    } else if cfg!(windows) {
        &[
            "BUSYBO~1.EXE",
            "SUDO~1 -n",
            "sudo::$DATA -n",
            "env.exe:s",
            "'C:\\Tools\\BASH~1.EXE' -c",
        ]
    } else {
        &[]
    };
    for launcher in unsure {
        let command = format!("{launcher} 'git push'");
        for result in [requirement(&command).await, escalated(&command).await] {
            assert!(asks(&result), "{command}: {result:?}");
        }
    }
    // Elsewhere these are other programs, as before.
    for launcher in [
        "su\u{200d}do -n",
        "BUSYBO~1.EXE",
        "SUDO~1 -n",
        "sudo::$DATA -n",
    ] {
        if unsure.contains(&launcher) {
            continue;
        }
        let command = format!("{launcher} git push");
        assert!(
            matches!(
                requirement(&command).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2510_chroot_named_like_an_assignment_reads_its_own_argv() {
    // argv runs its first word as a program: `a=b/chroot` is chroot in a
    // directory named `a=b`, not an assignment, and `#x/chroot` no comment.
    for command in [
        "a=b/chroot / git push",
        "'#x/chroot' / git push",
        "A=B/CHROOT --userspec=u / git push",
        "nohup a=b/chroot / git push",
    ] {
        let result = requirement(command).await;
        let expected = CASE_INSENSITIVE || !command.contains("CHROOT");
        assert_eq!(refused(&result), expected, "{command}: {result:?}");
    }
    // The command is the word after the root, not a later chroot's.
    let result = requirement("a=b/chroot chroot git push").await;
    assert!(refused(&result), "{result:?}");
    // sudo reads such a word as an assignment and runs `/` (as before).
    let result = requirement("sudo a=b/chroot / git push").await;
    assert!(!refused(&result), "{result:?}");
}

#[tokio::test]
async fn issue2510_windows_folds_the_dotless_i() {
    // NTFS uppercases `ı` to `I`, so `tımeout` runs timeout there.
    for command in ["t\u{131}meout 5 git push", "N\u{131}CE -n 5 git push"] {
        let result = requirement(command).await;
        assert_eq!(refused(&result), cfg!(windows), "{command}: {result:?}");
    }
}
