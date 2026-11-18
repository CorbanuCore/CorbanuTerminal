//! D553: unread PowerShell scripts ask under strict rules in every mode.
use super::*;
use pretty_assertions::assert_eq;

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
                    assert_eq!(
                        result, ASKED,
                        "{command:?} {decision} {approval_policy:?} {permission_profile:?}"
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
                            approval_policy: AskForApproval::Never,
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
                        assert_eq!(result, ASKED, "{command:?}");
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn non_plain_parameters_keep_literal_restrictions() {
    let encoded = encode("git status");
    for tail in [
        vec!["-EncodedCommand", encoded.as_str()],
        vec!["-ExecutionPolicy", "Bypass", "-Command", "git status"],
        vec!["-WindowStyle", "Hidden", "-Command", "git status"],
        vec!["-ep", "Bypass", "-c", "git status"],
    ] {
        let mut line = vec!["powershell.exe"];
        line.extend(tail);
        let command = words(&line);
        for decision in ["forbidden", "prompt"] {
            let rules = format!(
                r#"prefix_rule(pattern=["powershell.exe", "{}"], decision="{decision}")"#,
                line[1]
            );
            let result = requirement(command.clone(), Some(&rules)).await;
            if decision == "forbidden" {
                assert!(
                    matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                    "{command:?}: {result:?}"
                );
            } else {
                assert_eq!(result, ASKED, "{command:?}");
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
    assert_eq!(
        requirement(
            words(&["powershell.exe", "-c", "& ('gi'+'t') push"]),
            Some(rules)
        )
        .await,
        ASKED
    );
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
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{command:?}: {result:?}"
        );
    }
}
