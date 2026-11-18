//! #2548 / D553: text passed to eval or a shell's stdin is executable.
use super::*;

const FORBID: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const PROMPT: &str = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;

async fn check(script: &str, rules: &str, mode: usize) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: vec!["bash".into(), "-lc".into(), script.into()],
        approval_policy: AskForApproval::OnRequest,
        permission_profile: if mode == 0 {
            PermissionProfile::Disabled
        } else {
            PermissionProfile::workspace_write()
        },
        sandbox_permissions: if mode == 2 {
            SandboxPermissions::RequireEscalated
        } else {
            SandboxPermissions::UseDefault
        },
        prefix_rule: None,
    })
    .await
}

fn asks(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        }
    )
}

fn nested(script: &str) -> String {
    format!("sh -c '{}'", script.replace('\'', r#"'"'"'"#))
}

#[tokio::test]
async fn literal_scripts_table() {
    for script in [
        "eval 'git push'",
        "eval git push",
        "eval -- 'git push'",
        "builtin eval 'git push'",
        "command eval 'git push'",
        "builtin -- builtin eval 'git push'",
        "eval 'bash <<EOF\ngit push\nEOF'",
        "eval 'git' 'push'",
        "e'v'al 'g'\\'i\\'t push",
        "eval \"eval 'git push'\"",
        "sudo bash -c \"eval 'git push'\"",
        "env A=1 sh -c \"eval 'git push'\"",
        "xargs -I{} bash -c \"eval 'git push'\"",
        "bash <<EOF\ngit push\nEOF",
        "bash <<'EOF'\ngit push\nEOF",
        "bash <<\"EOF\"\ngit push\nEOF",
        "bash <<\\EOF\ngit push\nEOF",
        "bash <<-EOF\n\tgit push\n\tEOF",
        "sudo bash <<EOF\ngit push\nEOF",
        "env A=1 sh <<EOF\ngit push\nEOF",
        "bash -s <<EOF\ngit push\nEOF",
        "bash <<'EOF'\neval 'git push'\nEOF",
        "bash <<EOF\nenv A=1 sudo git push\nEOF",
        "bash <<EOF\nxargs git push\nEOF",
    ] {
        for script in [script.to_string(), nested(script)] {
            for mode in 0..3 {
                let result = check(&script, FORBID, mode).await;
                assert!(
                    matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                    "{mode} {script:?}: {result:?}"
                );
                let result = check(&script, PROMPT, mode).await;
                assert!(asks(&result), "{mode} {script:?}: {result:?}");
            }
        }
    }
}

#[tokio::test]
async fn unknown_scripts_table() {
    for script in [
        "eval \"$script\"",
        "eval echo \"$arg\"",
        "eval 'echo git push | bash'",
        "echo 'git push' | eval bash",
        "eval 'git ' \"$verb\"",
        "eval $(printf 'git push')",
        "echo 'git push' | bash",
        "echo 'git push' | sudo bash",
        "echo 'git push' | env A=1 sh",
        "echo 'git push' | xargs -I{} bash",
        "echo 'git push' | ( bash )",
        "cat <<EOF | bash\ngit push\nEOF",
        "echo 'git push' | echo $(bash)",
        "echo 'git push' | xargs sh",
        "bash <<EOF\n$script\nEOF",
        "bash <<EOF\ngit $verb\nEOF",
        "bash <<< 'git push'",
        "bash < ./x.sh",
        "bash <<EOF < ./x.sh\necho safe\nEOF",
        "bash <<EOF\necho safe\nEOF\n. ./x.sh",
        "fish <<EOF\ngit push\nEOF",
        ". ./x.sh",
        "source ./x.sh",
        "builtin source ./x.sh",
        "builtin eval \"$script\"",
        ". \"$file\"",
        "env A=1 bash -c 'source ./x.sh'",
        "eval \"source ./x.sh\"",
        "bash <<'EOF'\nsource ./x.sh\nEOF",
    ] {
        for script in [script.to_string(), nested(script)] {
            for rules in [FORBID, PROMPT] {
                for mode in 0..3 {
                    let result = check(&script, rules, mode).await;
                    assert!(asks(&result), "{mode} {script:?}: {result:?}");
                }
            }
        }
    }
}

#[tokio::test]
async fn negative_controls_and_no_rule() {
    for script in [
        "eval echo safe",
        "eval 'git status'",
        "eval",
        "echo 'git push' | cat",
        "cat <<EOF\ngit push\nEOF",
        "bash <<EOF\necho safe\nEOF",
        "echo 'git push' | bash -c 'echo safe'",
    ] {
        let result = check(script, FORBID, /*mode*/ 0).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{script:?}: {result:?}"
        );
    }
    for script in [
        "eval \"$script\"",
        "echo 'git push' | bash",
        ". ./x.sh",
        "source ./x.sh",
        "builtin source ./x.sh",
        "builtin eval \"$script\"",
        "bash <<EOF\n$script\nEOF",
    ] {
        for rules in ["", r#"prefix_rule(pattern=["git"], decision="allow")"#] {
            let result = check(script, rules, /*mode*/ 0).await;
            assert!(
                matches!(result, ExecApprovalRequirement::Skip { .. }),
                "{script:?}: {result:?}"
            );
        }
    }
}

#[tokio::test]
async fn allow_rules_cannot_hide_script_rules_or_offer_amendments() {
    for script in [
        "eval 'git push'",
        "bash <<EOF\ngit push\nEOF",
        "echo 'git push' | bash",
        "source ./x.sh",
        "builtin source ./x.sh",
        "builtin eval \"$script\"",
    ] {
        let rules = format!(
            "{FORBID}\nprefix_rule(pattern=[\"bash\"], decision=\"allow\")\nprefix_rule(pattern=[\"eval\"], decision=\"allow\")"
        );
        let result = check(script, &rules, /*mode*/ 0).await;
        assert!(
            asks(&result) || matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{script:?}: {result:?}"
        );
        let result = check(script, "", /*mode*/ 2).await;
        assert!(asks(&result), "{script:?}: {result:?}");
    }
}

#[tokio::test]
async fn eval_and_stdin_depth_bound() {
    let mut script = "git push".to_string();
    for _ in 0..super::super::wrappers::MAX_SCRIPT_DEPTH + 2 {
        script = format!("eval '{}'", script.replace('\'', r#"'"'"'"#));
    }
    let result = check(&script, FORBID, /*mode*/ 0).await;
    assert!(asks(&result), "{result:?}");
    let mut script = "git push".to_string();
    for depth in 0..super::super::wrappers::MAX_SCRIPT_DEPTH + 2 {
        script = format!("bash <<'END{depth}'\n{script}\nEND{depth}");
    }
    let result = check(&script, FORBID, /*mode*/ 0).await;
    assert!(asks(&result), "{result:?}");
}
