//! #2548 / D553: text passed to eval or a shell's stdin is executable.
use super::*;
use pretty_assertions::assert_eq;

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
        "{ bash <<EOF\ngit push\nEOF\n} < /dev/null",
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

// Deterministic exhaustive prefix pairs provide a wider net than random samples.
fn forwarding_variants() -> impl Iterator<Item = String> {
    const PREFIXES: [&str; 4] = ["time", "time -p", "!", "coproc"];
    PREFIXES.into_iter().flat_map(|first| {
        PREFIXES.into_iter().flat_map(move |second| {
            [
                format!("f() {{ bash; }}; {first} {second} f <<'EOF'\ngit push\nEOF"),
                format!("{first} {second} eval 'f() {{ bash; }}'; f <<'EOF'\ngit push\nEOF"),
                format!("{first} {second} exec <<'EOF'\ngit push\nEOF\nbash"),
            ]
        })
    })
}

#[tokio::test]
async fn unknown_scripts_table() {
    let mut failures = Vec::new();
    for script in [
        "eval \"$script\"",
        "eval echo \"$arg\"",
        "eval 'time ! git push'",
        "eval 'unrecognized-prefix git push'",
        "eval 'echo git push | bash'",
        "echo 'git push' | eval bash",
        "echo 'git push' | bash -c bash",
        "echo 'git push' | sh -c 'env A=1 bash'",
        "bash -c bash <<EOF\ngit push\nEOF",
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
        // Security review: stdin consumers can shift the shell's parse.
        "bash <<'EOF'\nread -r _\necho '\ngit push\n'\nEOF",
        "bash <<'EOF'\nhead -c 1\n#git push\nEOF",
        "bash <<'EOF'\necho one\necho two\nEOF",
        "{ read -r _; bash; } <<'EOF'\necho '\ngit push\n'\nEOF",
        "( read -r _; bash ) <<'EOF'\necho '\ngit push\n'\nEOF",
        "while read -r _; do bash; break; done <<'EOF'\necho '\ngit push\n'\nEOF",
        "exec <<'EOF'\ngit push\nEOF\nbash",
        "command exec <<'EOF'\ngit push\nEOF\nbash",
        "builtin exec <<'EOF'\ngit push\nEOF\nbash",
        "f() { bash; }; f <<'EOF'\ngit push\nEOF",
        "f() { bash; }; time f <<'EOF'\ngit push\nEOF",
        "f() { bash; }; time -p -- f <<'EOF'\ngit push\nEOF",
        "time eval 'f() { bash; }'; f <<'EOF'\ngit push\nEOF",
        "time -p eval 'exec <<EOF\ngit push\nEOF'; bash",
        "function time { bash; }; 'time' <<'EOF'\ngit push\nEOF",
        "function command { bash; }; command <<'EOF'\ngit push\nEOF",
        "function builtin { bash; }; builtin <<'EOF'\ngit push\nEOF",
        "f() { bash; }; time ! f <<'EOF'\ngit push\nEOF",
        "f() { bash; }; time -p ! f <<'EOF'\ngit push\nEOF",
        "time ! eval 'f() { bash; }'; f <<'EOF'\ngit push\nEOF",
        "time ! exec <<'EOF'\ngit push\nEOF\nbash",
        "f() { bash; }; ! ! f <<'EOF'\ngit push\nEOF",
        "f() { bash; }; coproc f <<'EOF'\ngit push\nEOF",
        "coproc bash <<'EOF'\ngit push\nEOF",
        "f() { eval bash; }; f <<'EOF'\ngit push\nEOF",
        "eval 'f() { bash; }'; f <<EOF\ngit push\nEOF",
        "eval 'exec <<EOF\ngit push\nEOF'; bash",
        // The scanner does not apply shell quote removal to mixed delimiters.
        "bash <<E\"O\"F\necho '\nEOF\ngit push\n'\nE\"O\"F",
        "cat <<E\"O\"F\necho '\nEOF\ngit push\n'\nE\"O\"F",
        "cat <<EOF\nx\nEO\\\nF\ngit push\nEOF",
        "bash <<'A' -c 'git push'\necho safe\nA",
        "git <<'A' push --force\nA",
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
        "sudo sh -c 'source ./x.sh'",
        "xargs sh -c 'source ./x.sh'",
        "xargs -I{} bash <<EOF\ngit push\nEOF",
        "eval \"source ./x.sh\"",
        "bash <<'EOF'\nsource ./x.sh\nEOF",
    ]
    .into_iter()
    .map(str::to_string)
    .chain(forwarding_variants())
    {
        let inner = nested(&script);
        for script in [script, inner] {
            for rules in [FORBID, PROMPT] {
                for mode in 0..3 {
                    let result = check(&script, rules, mode).await;
                    if !asks(&result) {
                        failures.push(format!("{rules} {mode} {script:?}: {result:?}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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
        for mode in 1..3 {
            assert_eq!(
                check(script, FORBID, mode).await,
                check(script, "", mode).await,
                "{mode} {script:?}: an unrelated rule must not tighten the fallback"
            );
        }
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
        "cat <<E\"O\"F\necho safe\nEOF\nE\"O\"F",
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
