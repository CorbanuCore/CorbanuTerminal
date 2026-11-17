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
    let result = requirement("ls x", AskForApproval::UnlessTrusted).await;
    assert!(
        matches!(result, ExecApprovalRequirement::Skip { .. }),
        "{result:?}"
    );
    for line in ["LS x", "Ls x"] {
        let result = requirement(line, AskForApproval::UnlessTrusted).await;
        // D696: Windows too, though its host-executable lookup reads a bare
        // name by its lower-cased basename.
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{line}: {result:?}"
        );
    }
}

const ALLOW_RULES: &str = r#"prefix_rule(pattern=["rm"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="allow")"#;

async fn allow_rules_requirement(command: &[&str]) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(ALLOW_RULES.to_string()),
        command: command.iter().map(ToString::to_string).collect(),
        // Asks for every command no rule allows.
        approval_policy: AskForApproval::UnlessTrusted,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

/// D696: allow rules approve only the program name as the rule spells it,
/// on every platform. Windows used to read `RM`, `GIT.EXE` and
/// `C:\…\git.exe` by their lower-cased, suffix-less basename, so an allow
/// rule for `rm` or `git push` approved them.
#[tokio::test]
async fn d696_allow_rules_need_the_exact_spelling() {
    // The exact spelling is still allowed (the control), bare or as a path.
    let git = if cfg!(windows) {
        r"C:\Program Files\Git\cmd\git"
    } else {
        "/usr/bin/git"
    };
    for command in [&["rm", "x"][..], &["git", "push"], &[git, "push"]] {
        let result = allow_rules_requirement(command).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{command:?}: {result:?}"
        );
    }
    for command in [
        &["RM", "x"][..],
        &["Rm", "x"],
        &["rm.exe", "x"],
        &["RM.EXE", "x"],
        &["Git", "push"],
        &["GIT", "push"],
        &["git.exe", "push"],
        &["GIT.EXE", "push"],
        &[r"C:\Program Files\Git\cmd\git.exe", "push"],
        &[r"C:\Program Files\Git\cmd\GIT.EXE", "push"],
        &[r"c:\program files\git\cmd\Git", "push"],
        &["/usr/bin/GIT", "push"],
        &["./RM", "x"],
    ] {
        let result = allow_rules_requirement(command).await;
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{command:?}: {result:?}"
        );
    }
}

/// D696: a path listed by `host_executable` is allowed as listed; the same
/// file spelled with another drive letter case or name case is not.
#[tokio::test]
async fn d696_listed_host_executable_needs_the_listed_spelling() {
    let (listed, other_drive_case, other_name_case) = if cfg!(windows) {
        (
            r"C:\Git\cmd\git.exe",
            r"c:\Git\cmd\git.exe",
            r"C:\Git\cmd\GIT.EXE",
        )
    } else {
        ("/usr/bin/git", "/usr/bin/git", "/usr/bin/GIT")
    };
    let policy_src = format!(
        "prefix_rule(pattern=[\"git\", \"push\"], decision=\"allow\")\n\
         host_executable(name=\"git\", paths=[\"{}\"])",
        listed.replace('\\', "\\\\")
    );
    let requirement = |program: &str| {
        exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(policy_src.clone()),
            command: vec![program.to_string(), "push".to_string()],
            approval_policy: AskForApproval::UnlessTrusted,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
    };
    let result = requirement(listed).await;
    assert!(
        matches!(result, ExecApprovalRequirement::Skip { .. }),
        "{result:?}"
    );
    // Off Windows `other_drive_case` is the listed path itself.
    let result = requirement(other_drive_case).await;
    assert_eq!(
        matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
        cfg!(windows),
        "{result:?}"
    );
    let result = requirement(other_name_case).await;
    assert!(
        matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
        "{result:?}"
    );
}

/// D696 keeps Windows' program-name normalisation for forbidden and prompt
/// rules: `RM.EXE` and `C:\…\RM.exe` are refused by a forbidden `rm` rule,
/// `GIT.EXE push` is asked about by a prompt `git push` rule.
#[tokio::test]
async fn d696_forbidden_and_prompt_rules_keep_windows_spellings() {
    for line in ["RM.EXE -rf x", "rm.exe x"] {
        let result = requirement(line, AskForApproval::OnRequest).await;
        assert_eq!(
            refused_by_rule(&result),
            cfg!(windows),
            "{line}: {result:?}"
        );
    }
    let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(RULES.to_string()),
        command: vec![r"C:\Windows\RM.exe".to_string(), "x".to_string()],
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await;
    assert_eq!(refused_by_rule(&result), cfg!(windows), "{result:?}");
    let result = requirement("GIT.EXE push", AskForApproval::OnRequest).await;
    assert_eq!(asks_by_rule(&result), cfg!(windows), "{result:?}");
}
