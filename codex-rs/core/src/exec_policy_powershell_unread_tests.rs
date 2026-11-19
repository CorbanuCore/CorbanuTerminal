//! D553: unread PowerShell scripts ask under strict rules in every mode.
use super::*;

fn unread_forms() -> Vec<Vec<String>> {
    let mut commands = vec![
        words(&["powershell.exe", "-Command", "& ('gi'+'t') push"]),
        words(&["powershell.exe", "-Command", "& $cmd push"]),
        words(&["powershell.exe", "-Command", "git status; $x"]),
        words(&["powershell.exe", "-Command", "param($x) & $x"]),
        words(&["powershell.exe", "-Command", "git push; ("]),
        words(&["powershell.exe", "-Command", "& ('gi'+'t')", "push"]),
        words(&["powershell.exe", "-Command", "-"]),
        words(&["powershell.exe", "-File", "-"]),
        words(&["powershell.exe", "-NoProfile"]),
        words(&["powershell.exe"]),
        words(&["bash", "-c", "echo 'git push' | powershell.exe -NoProfile"]),
        words(&["powershell.exe", "-c", "powershell.exe -NoProfile"]),
    ];
    for script in ["& ('gi'+'t') push", "git status; $x", "git push; ("] {
        commands.push(words(&[
            "powershell.exe",
            "-EncodedCommand",
            &encode(script),
        ]));
    }
    let pwsh_commands: Vec<_> = commands
        .iter()
        .map(|command| {
            command
                .iter()
                .map(|word| word.replace("powershell.exe", "pwsh.exe"))
                .collect()
        })
        .collect();
    commands.extend(pwsh_commands);
    commands
}

#[tokio::test]
async fn unread_scripts_ask_without_amendments_across_permission_modes() {
    for command in unread_forms() {
        for decision in ["prompt", "forbidden"] {
            // An unrelated rule could match the unknown runtime command.
            let rules = format!(r#"prefix_rule(pattern=["secret-tool"], decision="{decision}")"#);
            for permission_profile in [
                PermissionProfile::Disabled,
                PermissionProfile::read_only(),
                PermissionProfile::workspace_write(),
            ] {
                for approval_policy in [
                    AskForApproval::OnRequest,
                    AskForApproval::Never,
                    AskForApproval::UnlessTrusted,
                ] {
                    let result =
                        exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                            policy_src: Some(rules.clone()),
                            command: command.clone(),
                            approval_policy,
                            permission_profile: permission_profile.clone(),
                            sandbox_permissions: SandboxPermissions::UseDefault,
                            prefix_rule: Some(command.clone()),
                        })
                        .await;
                    let expected = if approval_policy == AskForApproval::Never {
                        matches!(&result, ExecApprovalRequirement::Forbidden { .. })
                    } else if decision == "forbidden" {
                        matches!(&result, ExecApprovalRequirement::NeedsHumanApproval { .. })
                    } else {
                        matches!(
                            &result,
                            ExecApprovalRequirement::NeedsApproval {
                                proposed_execpolicy_amendment: None,
                                ..
                            }
                        )
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
async fn unread_scripts_with_no_strict_rules_do_not_add_prompts() {
    for command in unread_forms() {
        for rules in [
            None,
            Some(r#"prefix_rule(pattern=["unrelated"], decision="allow")"#),
        ] {
            assert!(
                matches!(
                    requirement(command.clone(), rules).await,
                    ExecApprovalRequirement::Skip { .. }
                ),
                "{command:?} {rules:?}"
            );
        }
    }
}

#[tokio::test]
async fn known_commands_in_declined_scripts_still_refuse() {
    for script in [
        "git push; $x",
        "$x; git push",
        "param($x) git push",
        "if ($x) { git push }",
        "& ('wr'+'ite-output') x; git push",
        "git status | & $cmd; git push",
        "git push $remote",
        "git push --% $REMOTE",
        "git push > $path",
        "& 'git' push $remote",
        "1..3 | ForEach-Object { git push }",
        "Invoke-Command { git push }",
        "& { git push }",
        "Write-Output (git push)",
        "Write-Output $(git push)",
        ". git push $remote",
    ] {
        for command in [
            words(&["powershell.exe", "-Command", script]),
            words(&["powershell.exe", "-EncodedCommand", &encode(script)]),
            words(&["powershell.exe", "-Command", script, "; $y"]),
        ] {
            for decision in ["forbidden", "prompt"] {
                let rules =
                    format!(r#"prefix_rule(pattern=["git", "push"], decision="{decision}")"#);
                for permission_profile in [
                    PermissionProfile::Disabled,
                    PermissionProfile::read_only(),
                    PermissionProfile::workspace_write(),
                ] {
                    let result =
                        exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                            policy_src: Some(rules.clone()),
                            command: command.clone(),
                            approval_policy: AskForApproval::OnRequest,
                            permission_profile,
                            sandbox_permissions: SandboxPermissions::UseDefault,
                            prefix_rule: None,
                        })
                        .await;
                    if decision == "forbidden" {
                        assert!(
                            matches!(&result, ExecApprovalRequirement::Forbidden { reason } if reason.contains("policy forbids commands starting with `git push`")),
                            "{command:?}: {result:?}"
                        );
                    } else {
                        assert!(
                            matches!(
                                result,
                                ExecApprovalRequirement::NeedsApproval {
                                    reason: Some(_),
                                    proposed_execpolicy_amendment: None,
                                }
                            ),
                            "{command:?}: {result:?}"
                        );
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn non_plain_parameters_keep_literal_restrictions() {
    let encoded = encode("git status");
    let mut lines = vec![
        vec!["powershell.exe", "-EncodedCommand", encoded.as_str()],
        vec!["PowerShell.EXE", "-EncodedCommand", encoded.as_str()],
        vec![
            "powershell.exe",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "git status",
        ],
        vec![
            "powershell.exe",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "git status",
        ],
        vec!["powershell.exe", "-ep", "Bypass", "-c", "git status"],
    ];
    {
        lines.push(vec![
            "pwsh.exe",
            "-WorkingDirectory",
            "C:\\",
            "-c",
            "git status",
        ]);
    }
    for line in lines {
        let command = words(&line);
        for decision in ["forbidden", "prompt"] {
            let rules = format!(
                r#"prefix_rule(pattern=["{}", "{}"], decision="{decision}")"#,
                line[0], line[1]
            );
            for permission_profile in [
                PermissionProfile::Disabled,
                PermissionProfile::read_only(),
                PermissionProfile::workspace_write(),
            ] {
                let result =
                    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                        policy_src: Some(rules.clone()),
                        command: command.clone(),
                        approval_policy: AskForApproval::OnRequest,
                        permission_profile,
                        sandbox_permissions: SandboxPermissions::UseDefault,
                        prefix_rule: None,
                    })
                    .await;
                if decision == "forbidden" {
                    assert!(
                        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                        "{command:?}: {result:?}"
                    );
                } else {
                    assert!(
                        matches!(
                            result,
                            ExecApprovalRequirement::NeedsApproval {
                                reason: Some(_),
                                proposed_execpolicy_amendment: None,
                            }
                        ),
                        "{command:?}: {result:?}"
                    );
                }
            }
        }
        assert!(
            matches!(
                requirement(command.clone(), Some(FORBID_GIT_PUSH)).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command:?}"
        );
    }
}

#[tokio::test]
async fn plain_scripts_remain_precise_and_dynamic_allow_never_overrides_unread() {
    let rules = r#"
prefix_rule(pattern=["git", "push"], decision="forbidden")
prefix_rule(pattern=["powershell.exe"], decision="allow")
"#;
    assert!(matches!(
        requirement(
            words(&["powershell.exe", "-c", "& ('gi'+'t') push"]),
            Some(rules)
        )
        .await,
        ExecApprovalRequirement::NeedsHumanApproval { .. }
    ));
    assert_forbidden(words(&["powershell.exe", "-c", "git push"])).await;
    assert!(matches!(
        requirement(
            words(&["powershell.exe", "-c", "git status"]),
            Some(FORBID_GIT_PUSH)
        )
        .await,
        ExecApprovalRequirement::Skip { .. }
    ));
}

#[tokio::test]
async fn joined_dangerous_script_keeps_heuristic_prompt_without_rules() {
    for command in [
        words(&[
            "powershell.exe",
            "-Command",
            "Remove-Item -Recurse -Force C:\\fixture",
            "; $a",
        ]),
        words(&[
            "powershell.exe",
            "-Command",
            "$a;",
            "Remove-Item -Recurse -Force C:\\fixture",
        ]),
        words(&[
            "powershell.exe",
            "-comm",
            "Remove-Item",
            "-Force C:\\fixture; $a",
        ]),
    ] {
        let result = requirement(command.clone(), None).await;
        assert!(asked(&result), "{command:?}: {result:?}");
        let never = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: None,
            command: command.clone(),
            approval_policy: AskForApproval::Never,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        assert!(
            matches!(never, ExecApprovalRequirement::Forbidden { .. }),
            "{command:?}: {never:?}"
        );
    }
}
