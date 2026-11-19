//! D713: the minimal lexical set (#2613) also reads parameter expansions as
//! arguments, constant arithmetic, `$(…)` and backquotes holding scripts of
//! the same set, and one-line `[ … ]`/`[[ … ]]` tests. Every reproducer of
//! #2548, #2585, #2594, #2597 and #2601, alone and combined with those
//! forms, still asks (a refusal where the rule is forbidden) in every
//! permission mode, with no amendment, and is refused under Aggressive with
//! approvals off; so does each attempt to run code through the new forms.
//! The forms themselves no longer ask when only an unrelated strict rule
//! exists.
use super::parser_issue_tests::FORBID;
use super::parser_issue_tests::PROMPT;
use super::parser_issue_tests::argv;
use super::parser_issue_tests::assert_scripts_ask;
use super::*;

/// The reproducers, each as a bash script.
const REPRODUCERS: &[&str] = &[
    // #2548.
    "eval 'git push'",
    "echo 'git push' | bash",
    "bash <<EOF\ngit push\nEOF",
    ". ./x.sh",
    "source ./x.sh",
    // #2585.
    "pwsh -Command \"$x\"",
    "powershell -Command \"$x\"",
    // #2594.
    "cat <<E\"O\"F\necho '\nEOF\ngit push\n'\nE\"O\"F",
    // #2597.
    "git <<'A' push --force\nA",
    "bash <<'A' -c 'git push'\necho safe\nA",
    // #2601.
    "hash -p /bin/bash f; echo 'git push' | f",
    "shopt -s expand_aliases\nalias f=bash\necho 'git push' | f",
    "trap 'sh -s' EXIT; echo 'git push'",
];

/// `script` beside or inside each form D713 reads.
fn combined(script: &str) -> Vec<String> {
    let mut scripts = vec![
        script.to_string(),
        format!("[[ -n $HOME ]] && {script}"),
        format!("[ -f x ] || {script}"),
        format!("x=$(date); {script}"),
        format!("echo $((1 + 2)); {script}"),
        format!("if [[ $x == y ]]; then {script}\nfi"),
        format!("for f in $(ls); do {script}\ndone"),
        format!("echo \"$(\n{script}\n)\""),
        format!("cd \"$HOME\" && {script} \"${{x:-y}}\""),
    ];
    if !script.contains(['`', '\\', '\'', '"']) {
        scripts.push(format!("echo `{script}`"));
    }
    scripts
}

#[tokio::test]
async fn reproducers_combined_with_read_forms_ask() {
    let scripts: Vec<String> = REPRODUCERS
        .iter()
        .flat_map(|script| combined(script))
        .collect();
    let scripts: Vec<&str> = scripts.iter().map(String::as_str).collect();
    assert_scripts_ask(&scripts, &[FORBID, PROMPT]).await;
}

/// What each new form could hide or run, and its spellings around a
/// `git push` a rule refuses or asks for.
#[tokio::test]
async fn code_through_read_forms_asks() {
    assert_scripts_ask(
        &[
            // Read precisely: the command a rule matches is found.
            "echo $(git push)",
            "echo \"$(git push)\"",
            "x=`git push`",
            "echo \"a `git push` b\"",
            "echo $(echo $(git push --force))",
            "[[ -n $x ]] && git push",
            "[ -f x ] || git push origin main",
            "x=$((1 + 2)) git push",
            // A word known only at run time may be `push`.
            "git ${x:-push}",
            "git \"$sub\" --force",
            "git push $remote",
            // An expansion names the command.
            "cmd=git; $cmd push",
            "${x:-git} push",
            "$(echo git) push",
            "`echo git` push",
            "x=sh; $x -c 'git push'",
            // Arithmetic evaluates a variable's value as an expression.
            "x='a[$(git push)]'; echo $((x))",
            "x='a[$(git push)]'; echo $(($x))",
            "x='a[$(git push)]'; [[ $x -eq 1 ]]",
            "n='a[$(git push)]'; [[ 1 -eq -n ]]",
            "x='a[$(git push)]'; [ 1 -eq \"$x\" ]",
            // `[` splits an expansion into words that become operators.
            "x='-v a[$(git push)]'; [ $x ]",
            "[ \"-v\" 'a[$(git push)]' ]",
            "x='-v'; y='a[$(git push)]'; [ \"$x\" \"$y\" ]",
            "x='a[$(git push)]'; [ \"$x\" -gt 0 ]",
            "[ \\-v 'a[$(git push)]' ]",
            // dash has no `[[`: it runs what follows `||`.
            "[[ -n x || git -nt push ]]",
            "[[ -n x || sh -c 'git push' ]]",
            // Shapes bash 3.2's scan of `$(…)` reads differently.
            "echo $(echo x\\ #'\n)\ngit push\n')",
            "echo $(echo a#b)\ngit push",
            "echo \"$$(\" '\")\"\ngit push\n'",
            "echo ${x:-$(git push)}",
            // tree-sitter-bash joins a line starting with `\` to the line
            // before; bash runs it (on main since #2613).
            "true\n\\git push",
            "x=1\n\\git push --force",
        ],
        &[FORBID, PROMPT],
    )
    .await;
}

/// The ask rate D713 lowers: these forms run without asking when only an
/// unrelated strict rule exists, and with a `git push` rule when nothing in
/// them could run it.
#[tokio::test]
async fn read_forms_do_not_ask_without_a_matching_rule() {
    const UNRELATED: &str = r#"prefix_rule(pattern=["zz-unrelated"], decision="forbidden")"#;
    let scripts = [
        "echo $HOME \"${XDG_CONFIG_HOME:-$HOME/.config}\" ${x#*/} ${#x}",
        "x=$(git rev-parse HEAD); echo \"$x\"",
        "cd \"$(dirname \"$0\")\" && make",
        "echo `pwd` \"`date +%s`\"",
        "echo $((1 + 2)) $(( $? * 3 ))",
        "[[ -n $x ]] && echo y",
        "[[ $x == *.rs ]] || echo n",
        "if [ -f x ]; then echo a; fi",
        "[ $? -eq 0 ] && echo ok",
        "[ -n \"$x\" ] && [ \"$a\" != \"$b\" ] && echo ok",
        "for f in *.rs; do wc -l \"$f\"; done",
        "git log -1 --format=%h $(git rev-parse HEAD)",
    ];
    for (rules, scripts) in [(UNRELATED, &scripts[..]), (FORBID, &scripts[..11])] {
        for script in scripts {
            let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
                policy_src: Some(rules.to_string()),
                command: argv(&["bash", "-lc", script]),
                approval_policy: AskForApproval::OnRequest,
                permission_profile: PermissionProfile::Disabled,
                sandbox_permissions: SandboxPermissions::UseDefault,
                prefix_rule: None,
            })
            .await;
            assert!(
                matches!(result, ExecApprovalRequirement::Skip { .. }),
                "{rules} {script:?}: {result:?}"
            );
        }
    }
}
