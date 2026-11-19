//! #2548 / D553: text passed to eval or a shell's stdin is executable.
//! Scripts in `literal_scripts_table` are read precisely; those in
//! `unknown_scripts_table` hold a shape rules cannot read in full, so they
//! ask (or are refused where a command a rule refuses is still seen).
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
    // Sync (D691): where a forbidden rule could match, a fresh human
    // decision.
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        } | ExecApprovalRequirement::NeedsHumanApproval { .. }
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
        "builtin command eval 'git push'",
        "command builtin command eval 'git push'",
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
        // #2548b: here-strings, reserved prefixes and their combinations
        // are read precisely.
        "bash <<< 'git push'",
        "time ! eval 'git push'",
        "! time -p -- eval 'git push'",
        "time coproc eval 'git push'",
        "eval 'time ! git push'",
        "coproc bash <<'EOF'\ngit push\nEOF",
        "f() { eval 'git push'; }; f",
        "x=$(eval 'git push')",
        "sh -c \"bash <<'EOF'\ngit push\nEOF\"",
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
        // Review round 1 (#2613).
        "cat <<EOF\n`git push`\nEOF",
        "cat <<EOF > notes.md\n`git push`\nEOF",
        "[[ x =~ (`git push`) ]]",
        "[[ x == ?a`{git,push}` ]]",
        "case x in ?a`{git,push}`) ;; esac",
        "exec 3< <(echo 'git push'); bash <<<'echo hi' 0>&3",
        "printf 'git push\\n' > s; bash <<<'echo hi' 00<s",
        "exec 3< <(echo 'git push'); bash <<'EOF' 0>&3\necho hi\nEOF",
        "OPTIND='a[$(git push)]'",
        "for OPTIND in 'a[$(git push)]'; do :; done",
        "export OPTIND='a[$(git push)]'",
        "declare HISTCMD='a[$(git push)]'",
        "read OPTIND <<< 'a[$(git push)]'",
        "echo ${OPTIND:='a[$(git push)]'}",
        "env SECONDS='a[$(git push)]' bash -c true",
        "ksh <<'EOF'\nshift 'a[$(git push)]'\nEOF",
        "ksh <<'EOF'\nprintf '%d' 'a[$(git push)]'\nEOF",
        "zsh -c \"shift 'a[\\$(git push)]'\"",
        "zsh -c \"printf '%d' 'a[\\$(git push)]'\"",
        "zsh -c \"exit \\$x\"",
        "hash -p /usr/bin/git x; x push",
        "zsh -c 'hash x=/usr/bin/git; x push'",
        "jobs -x git push",
        "coproc OPTIND='a[$(git push)]'",
        "x=(a)#; git push",
        "echo a\\\n#b; git push",
        // Review round 2 (#2613).
        "git 2>&1 push origin",
        "git >/dev/null push",
        "echo a\\\r\ngit push",
        "git pu\\\nsh",
        "(echo == x; git push)",
        "test 1 == 1; git push ]",
        "echo $(echo =~ x; git push)",
        "echo ${x#{}; git push; echo }",
        "zsh -c \"[[ -t 'path[\\$(git push)]' ]]\"",
        "zsh -c \"[ -t 'path[\\$(git push)]' ]\"",
        "zsh -c \"test -t 'path[\\$(git push)]'\"",
        "zsh -c \"echo \\$HOME['\\$(git push)']\"",
        "zsh -c \"a=\\$HOME['\\$(git push)']\"",
        "zsh -c 'nocorrect X=1 git push'",
        "zsh -c \"nocorrect OPTIND='a[\\$(git push)]'\"",
        // Review round 3 (#2613).
        "cat <<'Q' &&\nQ\ngit push\nQ",
        "cat <<Q |\nQ\ngit push\nQ",
        "cat <<'Q' && echo $(\ncat <<EOF\n$(echo '\nEOF\ngit push\n#')\nEOF\nQ\n)\nQ",
        "[ x | sh$IFS-c$IFS'git push' ]",
        "zsh -c 'commands=(ls /usr/bin/git); ls push'",
        "zsh -c \"functions=(ls 'git push'); ls\"",
        "zsh -c 'echo \"ls /usr/bin/git\" | read -A commands; ls push'",
        "zsh -c 'coproc nocorrect X=1 git push'",
        "zsh -c \"printf '%2\\$d %1\\$s' 5 'path[\\$(git push)]'\"",
        "zsh -c 'X==(git push)'",
        "zsh -c \"echo /(e:'git push':)\"",
        "[\n! sh -c 'git push' ]",
        "[ -n x\n] && git push",
        // Review round 4 (#2613).
        "[ x != ab<(git$IFS'push') ]",
        "[[ x != ab<(git$IFS'push') ]]",
        "case x in ab<(git$IFS'push')) ;; esac",
        "cat <<'A' && bash <<'A'\necho hi\nA\ngit push\nA",
        "zsh -c \"[ x |=sh -c 'git push' ]\"",
        "zsh -c \"x='a[\\$(git push)]'; a=(1); cat <<Q\nz \\$#a[x]\nQ\"",
        "zsh -c \"echo \\\"\\$#a[x]\\\"\"",
        "zsh -c \"printf '%ld' 'path[\\$(git push)]'\"",
        "sh -c \"echo \\$'\\\\' ; git push ; echo ' #'\"",
        "sh -c \"[[ -z x || sh -c 'git push' ]]\"",
        "cat <<'Q' && echo $(case x in x) cat <<'Q'\nQ\ngit push\nesac)\nQ",
        // Review round 6 (#2613).
        "echo \"$$(\" '\")\"\ngit push\n'",
        "echo \"$${\" '\"}\"\ngit push\n'",
        "zsh -c \"echo \\$\\$'\\\\' ';git push;' \\$\\$'\\\\'\"",
        "echo $(echo $$'\\' ')\ngit push\n')",
        "echo $(echo x\\ #'\n)\ngit push\n')",
        "sh -c 'time -l git push'",
        // Review round 7 (#2613).
        "sh -c 'time -p -l git push'",
        "sh -c \"time '-l' git push\"",
        "sh -c 'time \\-l git push'",
        "zsh -c '\"\"=git push'",
        "zsh -c \"''=git push\"",
        "zsh -c 'nohup \"\"=git push'",
        "/usr/bin/jobs -x git push",
        "echo x | /usr/bin/read 'a[$(git push)]'",
        "command gi#t push",
        "git pu#sh",
        // Review round 8 (#2613).
        "echo \\ #; git push",
        "echo x\\\t#; git push",
        "eval 'echo \\ #; git push'",
        "LC_ALL=zh_TW.Big5\necho \"\u{4e01}\\\"\ngit push\necho \"",
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
                    let refused = rules == FORBID
                        && matches!(result, ExecApprovalRequirement::Forbidden { .. });
                    if !asks(&result) && !refused {
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
        "echo 'git push' | bash -c 'echo safe'",
        "shift 2; exit 0",
        // Review round 2: everyday job control, status and formats stay read.
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

/// `eval 'X'` reads as the script `X` would: the same refusals and prompts,
/// no more (here a launcher no rule knows runs `git push` as an argument).
#[tokio::test]
async fn eval_text_reads_like_the_same_script() {
    for script in [
        "unrecognized-prefix git push",
        "git status",
        "echo 'git push'",
        "git push",
        "sudo git push",
    ] {
        let eval = format!("eval '{script}'");
        for rules in [FORBID, PROMPT] {
            let direct = check(script, rules, /*mode*/ 0).await;
            let through_eval = check(&eval, rules, /*mode*/ 0).await;
            assert_eq!(
                matches!(direct, ExecApprovalRequirement::Skip { .. }),
                matches!(through_eval, ExecApprovalRequirement::Skip { .. }),
                "{rules} {script:?}: {direct:?} / {through_eval:?}"
            );
            assert_eq!(
                matches!(direct, ExecApprovalRequirement::Forbidden { .. }),
                matches!(through_eval, ExecApprovalRequirement::Forbidden { .. }),
                "{rules} {script:?}: {direct:?} / {through_eval:?}"
            );
        }
    }
}

/// #2548 (coordinator, review round 5): scripts outside the minimal
/// lexical set ask wherever a prompt or forbidden rule exists, even with no
/// rule for what they run; with none, nothing changes. #2593's here-document
/// negative controls are among them.
#[tokio::test]
async fn scripts_outside_the_minimal_set_ask_only_with_strict_rules() {
    for script in [
        "cat <<EOF\ngit push\nEOF",
        "bash <<EOF\necho safe\nEOF",
        "cat <<'EOF' > f.py && python3 f.py\nprint(1)\nEOF",
        "f() { return $?; }; f; exit $?",
        "true;#note",
        "[ -n \"$(git status --porcelain)\" ] || [ \"$(uname)\" = Darwin ]",
        "cmd >/dev/null 2>&1",
        // The literal list (review round 7): no `&`.
        "sleep 1 & pid=$!; kill $pid; wait $!",
    ] {
        let result = check(script, FORBID, /*mode*/ 0).await;
        assert!(asks(&result), "{script:?}: {result:?}");
        // Sync (D691), review r1 of #2640: where a forbidden rule could
        // match, every one (here-documents included) is a fresh human
        // decision; where only a prompt rule could, an ordinary prompt
        // with no amendment.
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
            "{script:?}: {result:?}"
        );
        let result = check(script, PROMPT, /*mode*/ 0).await;
        assert!(
            matches!(
                result,
                ExecApprovalRequirement::NeedsApproval {
                    proposed_execpolicy_amendment: None,
                    ..
                }
            ),
            "{script:?}: {result:?}"
        );
        // Sync: any here-document script asks, as before (S3b).
        if script.contains("<<") {
            continue;
        }
        let result = check(script, "", /*mode*/ 0).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{script:?}: {result:?}"
        );
    }
    // D713 reads parameter expansions as arguments: these run nothing a
    // rule matches (`printf` reads no number from them).
    for script in [
        "printf '%s\\n' \"$x\" | sort",
        "printf '%s: %d\\n' \"$name\" 3",
    ] {
        let result = check(script, FORBID, /*mode*/ 0).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{script:?}: {result:?}"
        );
    }
    let result = check("printf '%d\\n' \"$x\"", FORBID, /*mode*/ 0).await;
    assert!(asks(&result), "{result:?}");
}

#[tokio::test]
async fn ordinary_arguments_are_not_execution_text() {
    for script in [
        "/tmp/corbanu exec --skip-git-repo-check 'nested run' 2>&1; echo \"exit=$?\"",
        "env FLAG=1 /tmp/corbanu exec arg 2>&1; echo \"exit=$?\"",
        "sudo /tmp/corbanu eval arg 2>&1; echo \"exit=$?\"",
        "unshare -Ur /tmp/corbanu exec arg 2>&1; echo \"exit=$?\"",
        "git log -- source",
        "git log -- source; echo \"exit=$?\"",
        "/tmp/app . arg 2>&1; echo \"exit=$?\"",
        "/tmp/app coproc arg 2>&1; echo \"exit=$?\"",
        "echo eval | cat",
        "echo source | cat",
        "printf '%s' . | cat",
        "xargs echo exec",
        "builtin command echo exec",
        "command builtin command echo source",
    ] {
        for script in [script.to_string(), nested(script)] {
            for mode in 0..3 {
                let baseline = check(&script, "", mode).await;
                for rules in [FORBID, PROMPT] {
                    assert_eq!(
                        check(&script, rules, mode).await,
                        baseline,
                        "{mode} {script:?}: ordinary arguments must not activate unrelated rules"
                    );
                }
            }
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

/// Review round 7: a builtin run through a wrapper program (macOS ships
/// `/usr/bin/jobs` and the like as `builtin jobs "$@"` scripts) is
/// classified even as the argv given, not only inside a script.
#[tokio::test]
async fn builtin_wrappers_are_classified_as_argv_too() {
    for command in [
        vec!["/usr/bin/jobs", "-x", "git", "push"],
        vec!["env", "jobs", "-x", "git", "push"],
    ] {
        let command: Vec<String> = command.into_iter().map(str::to_string).collect();
        let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(FORBID.to_string()),
            command: command.clone(),
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        assert!(asks(&result), "{command:?}: {result:?}");
    }
}

/// Sync, review r1 of #2640: inside the session shell's `bash -lc`, a
/// non-ASCII word lies outside the minimal set, so the script is not split
/// and `execpolicy_blocks_shell_invocation`'s reason may name the session
/// shell's own `bash`. Canonical folding stays pinned here with a program
/// that is not the session shell: only folding `lſ` to `ls` can match.
#[tokio::test]
async fn canonical_rule_still_folds_inside_an_unsplit_script() {
    let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(
            r#"prefix_rule(pattern=["ls"], canonical_matching=True, decision="forbidden")"#
                .to_string(),
        ),
        command: vec!["bash".into(), "-lc".into(), "lſ -a".into()],
        approval_policy: AskForApproval::Never,
        permission_profile: PermissionProfile::workspace_write(),
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await;
    let ExecApprovalRequirement::Forbidden { reason } = &result else {
        panic!("{result:?}");
    };
    assert!(
        reason.contains("policy forbids commands starting with `ls`")
            || reason.contains("policy forbids commands starting with `'lſ'`"),
        "{reason}"
    );
}

/// CI on #2613 (upstream's `allowed_escalated_shell_command_inherits_active_
/// permission_profile`): a shell running a named script file runs its argv,
/// as `./build.sh` would, so a person's allow rule for exactly that argv
/// still applies to the script it is wrapped in; the file's commands are
/// unread, so a prompt or forbidden rule that could match one still asks.
#[tokio::test]
async fn script_file_runs_keep_allow_rules_and_ask_with_strict_rules() {
    const ALLOW: &str = r#"prefix_rule(pattern=["sh", "./build.sh"], decision="allow")"#;
    for script in ["sh ./build.sh", "sh ./build.sh arg && echo done"] {
        let scenario = |rules: String| ExecApprovalRequirementScenario {
            policy_src: Some(rules),
            command: vec!["bash".into(), "-lc".into(), script.into()],
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::workspace_write(),
            sandbox_permissions: SandboxPermissions::RequireEscalated,
            prefix_rule: None,
        };
        let allowed = exec_approval_requirement_for_command(scenario(ALLOW.to_string())).await;
        if script == "sh ./build.sh" {
            assert!(
                matches!(allowed, ExecApprovalRequirement::Skip { .. }),
                "{script:?}: {allowed:?}"
            );
        }
        let strict =
            exec_approval_requirement_for_command(scenario(format!("{ALLOW}\n{FORBID}"))).await;
        assert!(asks(&strict), "{script:?}: {strict:?}");
    }
}
