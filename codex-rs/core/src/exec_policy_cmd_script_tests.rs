//! #2704 (D553): scripts cmd.exe reads, which rules cannot see, ask under
//! strict rules in every mode, are refused when approvals are off, and never
//! offer a remembered approval.
use super::*;
use pretty_assertions::assert_eq;

fn argv(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_string()).collect()
}

fn unread_cmd_forms() -> Vec<Vec<String>> {
    let mut forms = vec![
        argv(&["cmd", "/c", "git push"]),
        argv(&["cmd", "/C", "git", "push"]),
        argv(&["cmd.exe", "/d", "/c", "git push"]),
        argv(&["CMD.EXE", "/Q", "/C", "git push"]),
        argv(&["cmd.exe", "/d/c", "git push"]),
        argv(&["cmd", "/k", "git push"]),
        argv(&[r"C:\Windows\System32\cmd.exe", "/c", "git push"]),
        argv(&["C:/Windows/System32/CMD.exe", "/c", "git push"]),
        argv(&["cmd.exe. ", "/c", "git push"]),
        // A script from its input.
        argv(&["cmd"]),
        argv(&["cmd.exe", "/d"]),
        // cmd.exe re-reads a batch file's command line.
        argv(&["pnpm.cmd", "&git push"]),
        argv(&["pnpm.cmd", "run", "a & git push"]),
        argv(&[r".\build.BAT", "x|git push"]),
        argv(&["tool.cmd", "%PATH%"]),
        argv(&["tool.cmd", "a\"b"]),
        argv(&["tool.bat", "a^b"]),
        argv(&["tool.cmd", "(x)"]),
        argv(&["tool.cmd", "!x!"]),
        argv(&["tool.cmd", "a\nb"]),
        argv(&["tool.cmd.", "a>b"]),
        // A drive-relative spelling.
        argv(&["c:cmd", "/c", "git push"]),
        // Inside a shell script, and behind a launcher.
        argv(&["bash", "-c", "cmd /c 'git push'"]),
        argv(&["env", "cmd", "/c", "git push"]),
    ];
    if cfg!(windows) {
        // A path that may be a batch file: in the command's folder, or a
        // file a script creates first.
        forms.extend([
            argv(&[r".\build", "&git push"]),
            argv(&[r"C:\p\x", "&git push"]),
        ]);
    }
    forms
}

async fn decide(
    rules: &str,
    command: &[String],
    approval_policy: AskForApproval,
    permission_profile: PermissionProfile,
) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: command.to_vec(),
        approval_policy,
        permission_profile,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: Some(command.to_vec()),
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

#[tokio::test]
async fn cmd_scripts_ask_under_strict_rules_without_amendments() {
    for command in unread_cmd_forms() {
        for decision in ["prompt", "forbidden"] {
            // An unrelated rule could match a command cmd.exe runs.
            let rules = format!(r#"prefix_rule(pattern=["secret-tool"], decision="{decision}")"#);
            for permission_profile in [
                PermissionProfile::Disabled,
                PermissionProfile::read_only(),
                PermissionProfile::workspace_write(),
            ] {
                for approval_policy in [
                    AskForApproval::OnRequest,
                    AskForApproval::UnlessTrusted,
                    AskForApproval::Never,
                ] {
                    let result = decide(
                        &rules,
                        &command,
                        approval_policy,
                        permission_profile.clone(),
                    )
                    .await;
                    let expected = if approval_policy == AskForApproval::Never {
                        matches!(result, ExecApprovalRequirement::Forbidden { .. })
                    } else {
                        asked(&result)
                    };
                    assert!(
                        expected,
                        "{command:?} {decision} {approval_policy:?} {permission_profile:?}: {result:?}"
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn cmd_script_under_forbidden_git_push_asks() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in [
        argv(&["cmd", "/c", "git push"]),
        argv(&["cmd.exe", "/d", "/c", "git push"]),
        argv(&["pnpm.cmd", "x&git push"]),
    ] {
        let result = decide(
            rules,
            &command,
            AskForApproval::OnRequest,
            PermissionProfile::Disabled,
        )
        .await;
        assert!(asked(&result), "{command:?}: {result:?}");
    }
}

#[tokio::test]
async fn cmd_scripts_are_refused_under_aggressive_with_approvals_off() {
    // Aggressive always holds a forbidden vault rule.
    let manager = ExecPolicyManager::new(Arc::new(Policy::empty()));
    for command in unread_cmd_forms() {
        for approval_policy in [AskForApproval::Never, AskForApproval::OnRequest] {
            let result = manager
                .create_exec_approval_requirement(
                    ExecApprovalRequest {
                        command: &command,
                        approval_policy,
                        permission_profile: PermissionProfile::workspace_write(),
                        windows_sandbox_level: WindowsSandboxLevel::RestrictedToken,
                        sandbox_permissions: SandboxPermissions::UseDefault,
                        prefix_rule: Some(command.clone()),
                        cwd: None,
                        env: None,
                    },
                    /*live_aggressive*/ true,
                )
                .await;
            let expected = if approval_policy == AskForApproval::Never {
                matches!(result, ExecApprovalRequirement::Forbidden { .. })
            } else {
                asked(&result)
            };
            assert!(expected, "{command:?} {approval_policy:?}: {result:?}");
        }
    }
}

/// The kind of a requirement and whether it offers an amendment.
fn shape(result: &ExecApprovalRequirement) -> (&'static str, bool) {
    match result {
        ExecApprovalRequirement::Skip {
            proposed_execpolicy_amendment,
            ..
        } => ("skip", proposed_execpolicy_amendment.is_some()),
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment,
            ..
        } => ("ask", proposed_execpolicy_amendment.is_some()),
        ExecApprovalRequirement::Forbidden { .. } => ("refuse", false),
    }
}

#[tokio::test]
async fn cmd_scripts_with_no_strict_rules_are_unchanged() {
    // Without a prompt or forbidden rule the unread-script check stays off:
    // sandboxed, these run; with every command asked, an amendment is offered.
    for command in unread_cmd_forms() {
        for rules in [
            None,
            Some(r#"prefix_rule(pattern=["unrelated"], decision="allow")"#),
        ] {
            for (approval_policy, expected) in [
                (AskForApproval::OnRequest, ("skip", false)),
                (AskForApproval::UnlessTrusted, ("ask", true)),
            ] {
                let result =
                    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                        policy_src: rules.map(str::to_string),
                        command: command.clone(),
                        approval_policy,
                        permission_profile: PermissionProfile::workspace_write(),
                        sandbox_permissions: SandboxPermissions::UseDefault,
                        prefix_rule: None,
                    })
                    .await;
                assert_eq!(
                    shape(&result).0,
                    expected.0,
                    "{command:?} {rules:?}: {result:?}"
                );
                // A launcher's argv is never offered as an amendment.
                if expected.1 && command[0] != "env" {
                    assert!(shape(&result).1, "{command:?} {rules:?}: {result:?}");
                }
            }
        }
    }
}

#[tokio::test]
async fn batch_files_with_plain_arguments_read_as_argv() {
    // A batch file given plain words is a program with arguments, as before.
    let rules = r#"prefix_rule(pattern=["secret-tool"], decision="forbidden")"#;
    for (batch, exe) in [
        (
            argv(&["pnpm.cmd", "install", "--frozen-lockfile"]),
            argv(&["pnpm.exe", "install", "--frozen-lockfile"]),
        ),
        (
            argv(&["build.bat", "release", "x86_64", "a=b", "c,d"]),
            argv(&["build.exe", "release", "x86_64", "a=b", "c,d"]),
        ),
        // A program named `.exe` given cmd.exe's special characters is argv.
        (
            argv(&["git.exe", "commit", "-m", "a & b (c) %d%"]),
            argv(&["git.exe", "commit", "-m", "a b c d"]),
        ),
    ] {
        let profile = PermissionProfile::Disabled;
        for approval_policy in [AskForApproval::OnRequest, AskForApproval::UnlessTrusted] {
            let batch_result = decide(rules, &batch, approval_policy, profile.clone()).await;
            let exe_result = decide(rules, &exe, approval_policy, profile.clone()).await;
            assert_eq!(shape(&batch_result), shape(&exe_result), "{batch:?}");
        }
    }
}

#[cfg(windows)]
#[tokio::test]
async fn powershell_scripts_running_cmd_ask_under_forbidden_git_push() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for script in [
        r#"cmd /c "git push""#,
        "cmd.exe /d /c 'git push'",
        "cmd /k git push",
        r"C:\Windows\System32\cmd.exe /c git push",
        r".\tool.cmd '&git push'",
        // A script that changes `PATH` or `PATHEXT` first changes which file
        // a bare name runs (`gradlew` as `C:\repo\gradlew.bat`).
        r"Set-Item env:PATH C:\repo; gradlew '&git push'",
        r"New-Item env:Path -Value C:\repo -Force; gradlew '&git push'",
        r"Set-Location env:; Set-Item PATH C:\repo; gradlew '&git push'",
        r"Set-Item Env:PATHEXT .BAT; gradlew '&git push'",
        r"Set-Item Environment::PATH C:\repo; gradlew '&git push'",
        r"Set-Item Microsoft.PowerShell.Core\Environment::PATH C:\repo; gradlew '&git push'",
    ] {
        let command = argv(&["powershell.exe", "-NoProfile", "-Command", script]);
        for profile in [
            PermissionProfile::Disabled,
            PermissionProfile::workspace_write(),
        ] {
            let result = decide(rules, &command, AskForApproval::OnRequest, profile.clone()).await;
            assert!(asked(&result), "{script} {profile:?}: {result:?}");
            let result = decide(rules, &command, AskForApproval::Never, profile).await;
            assert!(
                matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                "{script}: {result:?}"
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn bare_names_are_batch_files_as_powershell_finds_them_on_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("zz-tool.cmd"), "@echo %*").expect("write");
    std::fs::write(dir.path().join("zz-exe.exe"), "").expect("write");
    let other = tempfile::tempdir().expect("tempdir");
    std::fs::write(other.path().join("zz-exe.cmd"), "@echo %*").expect("write");
    let path = |dirs: &[&Path]| Some(std::env::join_paths(dirs).expect("PATH"));
    let found =
        |program: &str, dirs: &[&Path]| wrappers::windows_batch_file_in(program, path(dirs), None);
    // The first match on `PATH` counts.
    assert!(found("zz-tool", &[dir.path()]));
    assert!(!found("zz-exe", &[dir.path(), other.path()]));
    assert!(found("zz-exe", &[other.path(), dir.path()]));
    // Not found: it cannot run unless written into a `PATH` folder first.
    assert!(!found("zz-missing", &[dir.path()]));
    // A relative or network folder before a match: unknown.
    assert!(found("zz-exe", &[Path::new("bin"), dir.path()]));
    assert!(found("zz-exe", &[Path::new(r"\\server\share"), dir.path()]));
    // Named or given as a path.
    assert!(!found("tool.exe", &[]));
    assert!(found(r".\zz-exe", &[dir.path()]));
    assert!(found(r"C:\p\x", &[]));
}
