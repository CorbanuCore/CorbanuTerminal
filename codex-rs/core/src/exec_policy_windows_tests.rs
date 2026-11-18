use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn evaluates_powershell_inner_commands_against_prompt_rules() {
    assert_exec_approval_requirement_for_command(
        ExecApprovalRequirementScenario {
            policy_src: Some(r#"prefix_rule(pattern=["echo"], decision="prompt")"#.to_string()),
            command: vec![
                "powershell.exe".to_string(),
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "echo blocked".to_string(),
            ],
            approval_policy: AskForApproval::Never,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        },
        ExecApprovalRequirement::Forbidden {
            reason: PROMPT_CONFLICT_REASON.to_string(),
        },
    )
    .await;
}

#[tokio::test]
async fn evaluates_powershell_inner_commands_against_allow_rules() {
    assert_exec_approval_requirement_for_command(
        ExecApprovalRequirementScenario {
            policy_src: Some(r#"prefix_rule(pattern=["echo"], decision="allow")"#.to_string()),
            command: vec![
                "powershell.exe".to_string(),
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "echo blocked".to_string(),
            ],
            approval_policy: AskForApproval::UnlessTrusted,
            permission_profile: PermissionProfile::read_only(),
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        },
        ExecApprovalRequirement::Skip {
            bypass_sandbox: true,
            proposed_execpolicy_amendment: None,
        },
    )
    .await;
}

#[test]
fn commands_for_exec_policy_parses_powershell_shell_wrapper() {
    let command = vec![
        "powershell.exe".to_string(),
        "-NoProfile".to_string(),
        "-Command".to_string(),
        "echo blocked".to_string(),
    ];

    assert_eq!(
        commands_for_exec_policy(&command),
        ExecPolicyCommands {
            commands: vec![vec!["echo".to_string(), "blocked".to_string()]],
            command_origin: ExecPolicyCommandOrigin::PowerShellLiteral,
        }
    );
}

#[test]
fn unmatched_powershell_read_requires_approval_without_sandbox() {
    let command = vec!["Get-Content".to_string(), "Cargo.toml".to_string()];

    assert_eq!(
        Decision::Prompt,
        render_decision_for_unmatched_command(
            &command,
            UnmatchedCommandContext {
                approval_policy: AskForApproval::UnlessTrusted,
                permission_profile: &PermissionProfile::read_only(),
                windows_sandbox_level: WindowsSandboxLevel::Disabled,
                sandbox_permissions: SandboxPermissions::UseDefault,
                command_origin: ExecPolicyCommandOrigin::PowerShell,
            },
        )
    );
}

#[test]
fn read_only_windows_sandbox_runs_unmatched_commands_under_sandbox() {
    let command = vec!["cmd.exe".to_string(), "/c".to_string(), "dir".to_string()];

    for windows_sandbox_level in [
        WindowsSandboxLevel::RestrictedToken,
        WindowsSandboxLevel::Elevated,
    ] {
        assert_eq!(
            Decision::Allow,
            render_decision_for_unmatched_command(
                &command,
                UnmatchedCommandContext {
                    approval_policy: AskForApproval::Never,
                    permission_profile: &PermissionProfile::read_only(),
                    windows_sandbox_level,
                    sandbox_permissions: SandboxPermissions::UseDefault,
                    command_origin: ExecPolicyCommandOrigin::Generic,
                },
            )
        );
    }
}

#[test]
fn read_only_windows_policy_without_sandbox_backend_still_requires_approval() {
    let command = vec!["cmd.exe".to_string(), "/c".to_string(), "dir".to_string()];

    assert_eq!(
        Decision::Forbidden,
        render_decision_for_unmatched_command(
            &command,
            UnmatchedCommandContext {
                approval_policy: AskForApproval::Never,
                permission_profile: &PermissionProfile::read_only(),
                windows_sandbox_level: WindowsSandboxLevel::Disabled,
                sandbox_permissions: SandboxPermissions::UseDefault,
                command_origin: ExecPolicyCommandOrigin::Generic,
            },
        ),
        "command is forbidden because approval policy is never and there is no Windows sandbox to rely on"
    );
}

#[test]
fn writable_windows_policy_without_sandbox_backend_still_requires_approval() {
    let command = vec!["cmd.exe".to_string(), "/c".to_string(), "dir".to_string()];
    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
    ]);
    let permission_profile = PermissionProfile::from_runtime_permissions(
        &file_system_sandbox_policy,
        NetworkSandboxPolicy::Restricted,
    );

    assert_eq!(
        Decision::Forbidden,
        render_decision_for_unmatched_command(
            &command,
            UnmatchedCommandContext {
                approval_policy: AskForApproval::Never,
                permission_profile: &permission_profile,
                windows_sandbox_level: WindowsSandboxLevel::Disabled,
                sandbox_permissions: SandboxPermissions::UseDefault,
                command_origin: ExecPolicyCommandOrigin::Generic,
            },
        )
    );
}

#[tokio::test]
async fn unmatched_dangerous_powershell_inner_commands_require_approval() {
    let inner_command = vec![
        "Remove-Item".to_string(),
        "test".to_string(),
        "-Force".to_string(),
    ];

    assert_exec_approval_requirement_for_command(
        ExecApprovalRequirementScenario {
            policy_src: None,
            command: vec![
                "powershell.exe".to_string(),
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Remove-Item test -Force".to_string(),
            ],
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        },
        ExecApprovalRequirement::NeedsApproval {
            reason: None,
            proposed_execpolicy_amendment: Some(ExecPolicyAmendment::new(inner_command)),
        },
    )
    .await;
}

/// #2524 (D688): Windows runs PowerShell under any spelling of its name, so
/// its script is read like that of `pwsh`/`powershell.exe`.
const RESPELLED_POWERSHELL: &[&str] = &[
    "PWSH",
    "Pwsh.Exe",
    "PowerShell",
    "PowerShell.EXE",
    "POWERSHELL.EXE. ",
    r"C:\Windows\System32\WindowsPowerShell\v1.0\POWERSHELL.EXE",
];

async fn respelled_requirement(
    shell: &str,
    flag: &str,
    script: &str,
    rules: &str,
) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: vec![
            shell.to_string(),
            "-NoProfile".to_string(),
            flag.to_string(),
            script.to_string(),
        ],
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

#[tokio::test]
async fn rules_apply_inside_respelled_powershell_scripts() {
    let forbid = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    let prompt = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;
    for shell in RESPELLED_POWERSHELL {
        for flag in ["-Command", "-COMMAND", "-c", "-C"] {
            let result = respelled_requirement(shell, flag, "git push", forbid).await;
            assert!(
                matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                "{shell} {flag}: {result:?}"
            );
            let result = respelled_requirement(shell, flag, "git push", prompt).await;
            assert!(
                matches!(
                    &result,
                    ExecApprovalRequirement::NeedsApproval {
                        reason: Some(reason),
                        proposed_execpolicy_amendment: None,
                    } if reason.ends_with("requires approval by policy")
                ),
                "{shell} {flag}: {result:?}"
            );
            // Negative control: another command in the same script.
            let result = respelled_requirement(shell, flag, "git status", forbid).await;
            assert!(
                !matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                "{shell} {flag}: {result:?}"
            );
        }
    }
    // #2531: also nested in another (respelled) PowerShell script.
    let result = respelled_requirement(
        "PowerShell.EXE",
        "-Command",
        "POWERSHELL -NoProfile -Command 'git push'",
        forbid,
    )
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        "{result:?}"
    );
}

#[test]
fn commands_for_exec_policy_parses_respelled_powershell() {
    for shell in RESPELLED_POWERSHELL {
        let command = vec![
            shell.to_string(),
            "-NoProfile".to_string(),
            "-Command".to_string(),
            "echo blocked".to_string(),
        ];
        assert_eq!(
            commands_for_exec_policy(&command),
            ExecPolicyCommands {
                commands: vec![vec!["echo".to_string(), "blocked".to_string()]],
                // Sync: upstream's in-process reader and literal origin, as
                // in `commands_for_exec_policy_parses_powershell_shell_wrapper`.
                command_origin: ExecPolicyCommandOrigin::PowerShellLiteral,
            },
            "{shell}"
        );
    }
}
