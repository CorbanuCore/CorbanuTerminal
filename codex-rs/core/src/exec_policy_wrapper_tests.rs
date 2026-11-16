use super::*;
use pretty_assertions::assert_eq;

async fn requirement(line: &str, rules: &str) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: shlex::split(line).unwrap(),
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

#[tokio::test]
async fn issue2165_env_dash_assignments_do_not_hide_forbidden_commands() {
    let rules = r#"prefix_rule(pattern=["env"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in [
        "env -- -X=1 git push",
        "env - -X=1 git push",
        "env -- - -X=1 git push",
        "env - -u HOME git push",
        "env - -u=HOME -i git push",
        "env - - git push",
        "sudo env - -u HOME git push",
        "sudo env -- -X=1 git push",
        "env FOO=1 -X=1 git push",
    ] {
        assert!(
            matches!(
                requirement(command, rules).await,
                ExecApprovalRequirement::Forbidden { .. }
            ),
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2165_sudo_command_names_containing_equals_are_checked() {
    for inner in ["/work/a=b", "=script"] {
        let rules = format!(
            r#"prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["{inner}"], decision="forbidden")
prefix_rule(pattern=["git", "push"], decision="allow")"#
        );
        let command = format!("sudo {inner} git push");
        assert!(
            matches!(
                requirement(&command, &rules).await,
                ExecApprovalRequirement::Forbidden { .. }
            ),
            "{command}"
        );
    }
    let rules = r#"prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["FOO=1"], decision="forbidden")
prefix_rule(pattern=["git", "push"], decision="allow")"#;
    for command in ["sudo -- FOO=1 git push", "sudo -u root -- FOO=1 git push"] {
        assert!(
            matches!(
                requirement(command, rules).await,
                ExecApprovalRequirement::Forbidden { .. }
            ),
            "{command}"
        );
    }
    assert!(matches!(
        requirement(
            "sudo /work/a=b git push",
            r#"
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="allow")"#
        )
        .await,
        ExecApprovalRequirement::Skip {
            bypass_sandbox: false,
            ..
        }
    ));
}

#[tokio::test]
async fn issue2165_env_without_command_preserves_ordinary_policy() {
    for line in [
        "env",
        "env -0",
        "env -i",
        "env -u HOME",
        "env FOO=1",
        "env -- -X=1",
        "bash -lc 'env | grep PATH'",
    ] {
        for approval_policy in [AskForApproval::OnRequest, AskForApproval::Never] {
            let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                policy_src: None,
                command: shlex::split(line).unwrap(),
                approval_policy,
                permission_profile: PermissionProfile::read_only(),
                sandbox_permissions: SandboxPermissions::UseDefault,
                prefix_rule: None,
            })
            .await;
            assert!(
                matches!(
                    result,
                    ExecApprovalRequirement::Skip {
                        bypass_sandbox: false,
                        ..
                    }
                ),
                "{line}: {result:?}"
            );
        }
    }
}

#[tokio::test]
async fn issue2165_wrappers_preserve_inner_rule_decisions() {
    let wrappers = [
        "sudo",
        "sudo -n -u root -E --",
        "sudo -Eu root",
        "sudo --user=root --preserve-env=PATH",
        "sudo --us root",
        "/usr/bin/sudo -u root",
        "chroot /",
        "chroot --userspec=a:b --groups c --skip-chdir -- /",
        "env FOO=1",
        "env -i -u HOME FOO=1",
        "env -iu HOME -- FOO=1",
        "env --ignore-environment --unset=HOME FOO=1",
        "sudo -n chroot /",
        "env FOO=1 sudo -u root -E -- chroot / env -i git=not-a-command",
        "chroot / chroot / sudo -- env -u HOME",
    ];
    for wrapper in wrappers {
        for decision in ["allow", "prompt", "forbidden"] {
            let rules = format!(
                r#"prefix_rule(pattern=["git", "push"], decision="{decision}", justification="inner rule")"#
            );
            let command = format!("{wrapper} git push");
            let result = requirement(&command, &rules).await;
            match decision {
                "allow" => assert!(
                    matches!(result, ExecApprovalRequirement::Skip { .. }),
                    "{command}: {result:?}"
                ),
                "prompt" => assert!(
                    matches!(
                        result,
                        ExecApprovalRequirement::NeedsApproval {
                            proposed_execpolicy_amendment: None,
                            ..
                        }
                    ),
                    "{command}: {result:?}"
                ),
                "forbidden" => assert!(
                    matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                    "{command}: {result:?}"
                ),
                _ => unreachable!(),
            }
        }
    }
}

#[tokio::test]
async fn issue2165_strictest_outer_middle_or_inner_rule_wins() {
    for outer in ["allow", "prompt", "forbidden"] {
        for inner in ["allow", "prompt", "forbidden"] {
            let rules = format!(
                r#"prefix_rule(pattern=["sudo"], decision="{outer}")
prefix_rule(pattern=["git", "push"], decision="{inner}")"#
            );
            let result = requirement("sudo chroot / env FOO=1 git push", &rules).await;
            if outer == "forbidden" || inner == "forbidden" {
                assert!(
                    matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                    "{rules}: {result:?}"
                );
            } else if outer == "prompt" || inner == "prompt" {
                assert!(
                    matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
                    "{rules}: {result:?}"
                );
            } else {
                assert!(
                    matches!(
                        result,
                        ExecApprovalRequirement::Skip {
                            bypass_sandbox: false,
                            ..
                        }
                    ),
                    "{rules}: {result:?}"
                );
            }
        }
    }
    assert!(matches!(
        requirement(
            "sudo chroot / env FOO=1 git push",
            r#"prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["chroot"], decision="forbidden")
prefix_rule(pattern=["git", "push"], decision="allow")"#
        )
        .await,
        ExecApprovalRequirement::Forbidden { .. }
    ));
}

#[tokio::test]
async fn issue2165_unknown_or_incomplete_wrappers_prompt_even_when_allowed() {
    let rules = r#"prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["env"], decision="allow")
prefix_rule(pattern=["chroot"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="allow")"#;
    for command in [
        "sudo --future-option value git push",
        "sudo -X git push",
        "sudo -Xu root git push",
        "sudo --p git push",
        "sudo -u",
        "sudo",
        "sudo -s",
        "env -S 'git push'",
        "env - -S 'git push'",
        "env --future-option git push",
        "env -u",
        "chroot /",
        "chroot --future-option / git push",
        "sudo env -S 'git push'",
    ] {
        assert_eq!(
            requirement(command, rules).await,
            ExecApprovalRequirement::NeedsApproval {
                reason: None,
                proposed_execpolicy_amendment: None,
            },
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2165_wrapped_forbidden_rm_and_shell_commands_are_rejected() {
    let rules = r#"prefix_rule(pattern=["rm", "-rf"], decision="forbidden")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["env"], decision="allow")"#;
    for command in [
        "sudo chroot / rm -rf x",
        "sudo -u rm chroot / rm -rf x",
        "bash -lc 'sudo -n chroot / rm -rf x'",
        "bash -lc 'echo ok && env FOO=1 rm -rf x'",
        "env FOO=1 bash -lc 'rm -rf x'",
    ] {
        assert!(
            matches!(
                requirement(command, rules).await,
                ExecApprovalRequirement::Forbidden { .. }
            ),
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2165_wrapper_allow_does_not_override_inner_heuristics() {
    let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(r#"prefix_rule(pattern=["sudo"], decision="allow")"#.to_string()),
        command: shlex::split("sudo rm -rf /").unwrap(),
        approval_policy: AskForApproval::Never,
        permission_profile: PermissionProfile::read_only(),
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        "{result:?}"
    );
}

#[tokio::test]
async fn issue2165_option_operands_and_ordinary_arguments_are_not_commands() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in [
        "echo sudo git push",
        "env -u git echo push",
        "sudo -u git echo push",
        "chroot git echo push",
        "env -- -u HOME git push",
        "env - -- - git push",
    ] {
        assert!(
            matches!(
                requirement(command, rules).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
}

/// Sync: upstream no longer reduces a top-level heredoc script to its first
/// command, so the S3b heredoc rule pass must look through wrappers as main's
/// parser-plus-`check_with_wrappers` path did (sync-main7 review r1).
#[tokio::test]
async fn issue2165_top_level_heredoc_checks_wrapped_rules() {
    let rules = r#"prefix_rule(pattern=["env"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["rm"], decision="forbidden")
prefix_rule(pattern=["git", "push"], decision="allow")"#;
    let heredoc = |script: &str| format!("bash -lc {}", shlex::try_quote(script).unwrap());
    for script in [
        "env FOO=1 rm -rf x <<'EOF'\nEOF",
        "sudo -u root rm -rf x <<'EOF'\nEOF",
        "sudo env FOO=1 rm -rf x <<'EOF'\nEOF",
    ] {
        assert!(
            matches!(
                requirement(&heredoc(script), rules).await,
                ExecApprovalRequirement::Forbidden { .. }
            ),
            "{script}"
        );
    }
    let uncertain = heredoc("sudo --future-option value git push <<'EOF'\nEOF");
    assert!(
        matches!(
            requirement(&uncertain, rules).await,
            ExecApprovalRequirement::NeedsApproval { .. }
        ),
        "{uncertain}"
    );
}
