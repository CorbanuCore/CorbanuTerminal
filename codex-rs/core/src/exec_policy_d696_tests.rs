//! D696 (#2553): on Windows, allow rules approve only the program name as
//! they spell it; forbidden and prompt rules keep Windows' host lookup
//! (lower-cased, `.exe`/`.cmd`/`.bat`/`.com` dropped). Elsewhere nothing
//! changes.

use super::*;
use pretty_assertions::assert_eq;

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
async fn d696_allow_rules_keep_the_exact_name() {
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
