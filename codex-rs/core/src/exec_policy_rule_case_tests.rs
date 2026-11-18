//! D692 (#2510): on macOS and Windows, which find programs without regard
//! to case, forbidden and prompt rules match the program name in any case.
//! Allow rules keep the exact name, so they never approve more. Linux keeps
//! exact names.
//!
//! Sync: main's D696 cases that follow these in main's copy of this file live
//! in `exec_policy_d696_tests.rs` on this branch (#2574).

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

async fn escalated(line: &str, approval_policy: AskForApproval) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(RULES.to_string()),
        command: shlex::split(line).unwrap(),
        approval_policy,
        permission_profile: PermissionProfile::workspace_write(),
        sandbox_permissions: SandboxPermissions::RequireEscalated,
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
        "./RM x",
        "../../../../bin/RM -rf x",
        "\u{17f}\u{75}do -n RM -rf x",
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
async fn d692_case_folded_prompt_is_approved_as_the_fallback() {
    // Rule prompts allowed, sandbox approvals refused.
    let granular = AskForApproval::Granular(GranularApprovalConfig {
        sandbox_approval: false,
        rules: true,
        skill_approval: false,
        request_permissions: false,
        mcp_elicitations: false,
    });
    // As on main, the escalation the fallback asks for is refused: a prompt
    // rule matched only in another case does not make it a rule prompt.
    // (Windows: unchanged from main, whose host-executable lookup already
    // reads `GIT` by its lower-cased basename as an exact rule prompt.)
    let result = escalated("GIT push", granular).await;
    if cfg!(windows) {
        assert!(asks_by_rule(&result), "{result:?}");
    } else {
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{result:?}"
        );
    }
    // The exact name is a rule prompt, which these settings allow.
    let result = escalated("git push", granular).await;
    assert!(asks_by_rule(&result), "{result:?}");
}

/// Sync: a `canonical_matching` rule (upstream) matches a differently cased
/// name as an exact rule, so D692's folded copy of the same rule must not
/// turn its prompt into the fallback's.
#[tokio::test]
async fn d692_canonical_prompt_rule_stays_a_rule_prompt() {
    let granular = AskForApproval::Granular(GranularApprovalConfig {
        sandbox_approval: false,
        rules: true,
        skill_approval: false,
        request_permissions: false,
        mcp_elicitations: false,
    });
    let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(
            r#"prefix_rule(pattern=["git", "push"], canonical_matching=True, decision="prompt")"#
                .to_string(),
        ),
        command: vec!["GIT".to_string(), "push".to_string()],
        approval_policy: granular,
        permission_profile: PermissionProfile::workspace_write(),
        sandbox_permissions: SandboxPermissions::RequireEscalated,
        prefix_rule: None,
    })
    .await;
    assert!(asks_by_rule(&result), "{result:?}");
}

#[test]
fn d692_exact_rule_matches_exclude_case_folded_ones() {
    let prompt = |prefix: &[&str]| RuleMatch::PrefixRuleMatch {
        matched_prefix: prefix.iter().map(ToString::to_string).collect(),
        decision: Decision::Prompt,
        resolved_program: None,
        justification: None,
    };
    let heuristic = RuleMatch::HeuristicsRuleMatch {
        command: vec!["GIT".to_string()],
        decision: Decision::Prompt,
    };
    let folded = [prompt(&["git", "push"])];
    for (matched, exact) in [
        (vec![heuristic, prompt(&["git", "push"])], false),
        // The same rule matched exactly too (`git push && GIT push`).
        (
            vec![prompt(&["git", "push"]), prompt(&["git", "push"])],
            true,
        ),
        (vec![prompt(&["git", "push"]), prompt(&["rm"])], true),
    ] {
        assert_eq!(
            crate::exec_policy::wrappers::has_exact_rule_match(&matched, &folded, Decision::Prompt),
            exact,
            "{matched:?}"
        );
    }
}

#[tokio::test]
async fn d692_allow_rules_keep_the_exact_name() {
    // UnlessTrusted asks for commands no rule allows.
    // `bypass_sandbox` is true only when the allow rule approved the command.
    let result = requirement("ls x", AskForApproval::UnlessTrusted).await;
    assert!(
        matches!(
            result,
            ExecApprovalRequirement::Skip {
                bypass_sandbox: true,
                ..
            }
        ),
        "{result:?}"
    );
    for line in ["LS x", "Ls x"] {
        let result = requirement(line, AskForApproval::UnlessTrusted).await;
        // D696: the allow rule never approves it, Windows included. Sync:
        // upstream removed main's built-in Windows read-only list, so Windows
        // asks here too (as `exec_policy_d696_tests.rs` records).
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{line}: {result:?}"
        );
    }
}
