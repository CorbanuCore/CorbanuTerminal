//! D692 (#2510): on macOS and Windows, which find programs without regard
//! to case, forbidden and prompt rules match the program name in any case.
//! Allow rules keep the exact name, so they never approve more. Linux keeps
//! exact names.

use super::*;
use pretty_assertions::assert_eq;

const CASE_INSENSITIVE: bool = cfg!(any(target_os = "macos", windows));

const RULES: &str = r#"prefix_rule(pattern=["rm"], decision="forbidden")
prefix_rule(pattern=["git", "push"], decision="prompt")
prefix_rule(pattern=["ls"], decision="allow")"#;

async fn requirement(line: &str, approval_policy: AskForApproval) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(RULES.to_string()),
        command: shlex::split(line).unwrap(),
        approval_policy,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

fn refused_by_rule(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::Forbidden { reason }
            if reason.contains("policy forbids commands starting with `rm`")
    )
}

fn asks_by_rule(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            reason: Some(reason),
            proposed_execpolicy_amendment: None,
        } if reason.ends_with("requires approval by policy")
    )
}

#[tokio::test]
async fn d692_forbidden_rules_match_any_case() {
    // The exact name is refused everywhere (the control).
    assert!(refused_by_rule(
        &requirement("rm -rf x", AskForApproval::OnRequest).await
    ));
    for line in [
        "RM -rf x",
        "Rm x",
        "/bin/RM -rf x",
        "\u{17f}udo -n RM -rf x",
        "sudo RM -rf x",
        "SUDO Rm x",
        "bash -lc 'RM -rf x'",
        "env FOO=1 rM x",
    ] {
        for approval_policy in [AskForApproval::OnRequest, AskForApproval::Never] {
            let result = requirement(line, approval_policy).await;
            assert_eq!(
                refused_by_rule(&result),
                CASE_INSENSITIVE,
                "{line}: {result:?}"
            );
        }
    }
}

#[tokio::test]
async fn d692_prompt_rules_match_any_case() {
    for line in ["GIT push", "Git push origin", "sudo -n GIT push"] {
        let result = requirement(line, AskForApproval::OnRequest).await;
        assert_eq!(
            asks_by_rule(&result),
            CASE_INSENSITIVE,
            "{line}: {result:?}"
        );
    }
    // Arguments compare exactly.
    let result = requirement("GIT PUSH", AskForApproval::OnRequest).await;
    assert!(!asks_by_rule(&result), "{result:?}");
    // xargs (D677): stdin could complete a prompt rule in another case.
    let result = requirement("xargs GIT", AskForApproval::OnRequest).await;
    assert_eq!(
        matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
        CASE_INSENSITIVE,
        "{result:?}"
    );
}

#[tokio::test]
async fn d692_allow_rules_keep_the_exact_name() {
    // UnlessTrusted asks for commands no rule allows.
    let result = requirement("ls x", AskForApproval::UnlessTrusted).await;
    assert!(
        matches!(result, ExecApprovalRequirement::Skip { .. }),
        "{result:?}"
    );
    for line in ["LS x", "Ls x"] {
        let result = requirement(line, AskForApproval::UnlessTrusted).await;
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{line}: {result:?}"
        );
    }
}
