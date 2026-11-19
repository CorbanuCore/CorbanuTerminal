//! #2455 (D676-D678): `su` with a command, `sg` and `watch` need approval
//! whatever the allow rules say; `xargs` applies rules to its fixed argv and
//! asks whenever a stricter rule could match once stdin adds words.

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

async fn escalated(line: &str, rules: &str) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: Some(rules.to_string()),
        command: shlex::split(line).unwrap(),
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::workspace_write(),
        sandbox_permissions: SandboxPermissions::RequireEscalated,
        prefix_rule: Some(shlex::split(line).unwrap()),
    })
    .await
}

const ALLOW_RULES: &str = r#"prefix_rule(pattern=["su"], decision="allow")
prefix_rule(pattern=["/bin/su"], decision="allow")
prefix_rule(pattern=["sg"], decision="allow")
prefix_rule(pattern=["watch"], decision="allow")
prefix_rule(pattern=["xargs"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["nice"], decision="allow")
prefix_rule(pattern=["busybox"], decision="allow")
prefix_rule(pattern=["git"], decision="allow")"#;

fn asks(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        }
    )
}

#[tokio::test]
async fn issue2455c_su_commands_sg_and_watch_need_approval() {
    for command in [
        // D676: su with a command, in every position.
        "su -c 'git status'",
        "su root -c 'git status'",
        "su - root -c 'git status'",
        "su -l root -c 'git status'",
        "su --command='git status'",
        "su --command 'git status'",
        "su --comm='git status' root",
        "su --session-command='git status'",
        "su -lc 'git status'",
        "su -c'git status'",
        "su -m root -c 'git status'",
        "/bin/su -c 'git status'",
        "sudo su -c 'git status'",
        "nice su - root -c 'git status'",
        "busybox su -c 'git status'",
        // A chosen shell, or words the shell reads as a script and its args.
        "su -s /usr/bin/git root status",
        "su --shell=/bin/sh root",
        "su root script.sh",
        "su - root -- -c 'git status'",
        // D678: approval-only.
        "sg wheel -c 'git status'",
        "sg wheel git status",
        "sg",
        "watch git status",
        "watch -n 1 git status",
        "watch -x git status",
        "watch",
        "sudo watch -x git status",
    ] {
        for result in [
            requirement(command, ALLOW_RULES).await,
            escalated(command, ALLOW_RULES).await,
        ] {
            assert!(asks(&result), "{command}: {result:?}");
        }
    }
}

#[tokio::test]
async fn issue2455c_su_login_shells_keep_their_rules() {
    // Only a login or interactive shell: the allow rule still applies.
    for command in [
        "su",
        "su -",
        "su root",
        "su - root",
        "su -l root",
        "su --login root",
    ] {
        assert!(
            matches!(
                requirement(command, ALLOW_RULES).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
}

const XARGS_RULES: &str = r#"prefix_rule(pattern=["xargs"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;

#[tokio::test]
async fn issue2455c_xargs_refuses_forbidden_fixed_commands() {
    for command in [
        "xargs git push",
        "xargs -0 git push",
        "xargs -n 1 git push",
        "xargs -n1 -P4 git push",
        "xargs -0rn1 git push",
        "xargs -r -L 1 -t git push",
        "xargs -d '\\n' git push",
        "xargs -a list git push",
        "xargs --arg-file=list git push",
        "xargs --max-args 1 --null git push",
        "xargs --max-a=1 git push",
        "xargs -E eof -s 100 -x -p git push",
        "xargs -e git push",
        "xargs -l git push",
        "xargs -l1 git push",
        "xargs -I{} git push {}",
        "xargs -I {} git push origin {}",
        "xargs -i git push {}",
        "xargs --replace=@ git push @",
        "xargs -J % git push %",
        "xargs -R 2 -S 255 -I % git push %",
        "xargs -- git push",
        "sudo xargs git push",
        "xargs sudo -n git push",
        "xargs xargs git push",
        "/usr/bin/xargs git push",
        "gxargs -0 git push",
        "xargs env FOO=1 git push",
    ] {
        for result in [
            requirement(command, XARGS_RULES).await,
            escalated(command, XARGS_RULES).await,
        ] {
            let ExecApprovalRequirement::Forbidden { reason } = &result else {
                panic!("{command}: {result:?}");
            };
            assert!(
                reason.contains("policy forbids commands starting with `git push`"),
                "{command}: {reason}"
            );
        }
    }
    // A prompt rule the fixed argv matches keeps its reason.
    let result = requirement("xargs -0 git fetch", XARGS_RULES).await;
    let ExecApprovalRequirement::NeedsApproval {
        reason: Some(reason),
        proposed_execpolicy_amendment: None,
    } = &result
    else {
        panic!("{result:?}");
    };
    assert!(reason.ends_with("requires approval by policy"), "{reason}");
}

#[tokio::test]
async fn issue2455c_xargs_asks_when_stdin_could_complete_a_rule() {
    for command in [
        "xargs git",
        "xargs -0 git",
        "xargs -n 1 git",
        "xargs -- git",
        "sudo xargs git",
        "xargs sudo git",
        "xargs xargs git",
        // Replacement strings: the word and all after it are unknown.
        "xargs -I{} git {}",
        "xargs -I{} git {} --tags",
        "xargs -I {} git x{}",
        "xargs -i git {}",
        "xargs --replace git {}",
        "xargs -J % git %",
        "xargs -I{} {} push",
        "xargs -I{} sudo -u {} git status",
        "xargs xargs -I{} git {}",
        // No command of its own, or a shell: stdin supplies the command,
        // a script or su's `-c`.
        "xargs env",
        "xargs env -i",
        "xargs env FOO=1",
        "xargs nice",
        "xargs time",
        "xargs command",
        "xargs caffeinate",
        "xargs busybox",
        "xargs flock lockfile",
        "xargs chroot /",
        "xargs xargs",
        "xargs -0 xargs -0",
        "xargs su",
        "xargs su root",
        "xargs -I% su %",
        "xargs -0 sh -c",
        "xargs bash -c 'git status'",
        "xargs busybox sh",
        // Options stdin could add to a permuting launcher.
        "xargs doas git status",
        "xargs runuser -u root git status",
        // Unknown or malformed options.
        "xargs --frobnicate git status",
        "xargs -q git status",
        "xargs --help",
        "xargs --null=1 git status",
        "xargs -n",
    ] {
        for result in [
            requirement(command, XARGS_RULES).await,
            escalated(command, XARGS_RULES).await,
        ] {
            // D691: the forbidden `git push` rule could match, so a human
            // decides.
            assert!(
                matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
                "{command}: {result:?}"
            );
        }
    }
    // A prompt rule alone can be completed too.
    let rules = r#"prefix_rule(pattern=["xargs"], decision="allow")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;
    let result = requirement("xargs git", rules).await;
    assert!(asks(&result), "{result:?}");
    // Through a rule for xargs itself.
    let rules = r#"prefix_rule(pattern=["xargs", "git", "push"], decision="forbidden")"#;
    let result = requirement("xargs git", rules).await;
    assert!(
        matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
        "{result:?}"
    );
}

#[tokio::test]
async fn issue2455c_xargs_operands_and_other_commands_are_not_matched() {
    for command in [
        "xargs git status",
        "xargs -n1 git status --short",
        "xargs ls",
        "xargs echo push",
        "xargs",
        "xargs -0 -n 1",
        // Option values, not commands.
        "xargs -E git echo push",
        "xargs -a git echo push",
        "xargs -d git echo",
        "xargs -I git echo git",
        "xargs grep 'git push'",
        // Not these programs.
        "xargsx git push",
        "sux -c 'git push'",
        "watcher git push",
        "sgx git push",
    ] {
        assert!(
            matches!(
                requirement(command, XARGS_RULES).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
    // An allow rule the fixed argv could reach is no reason to ask.
    let rules = r#"prefix_rule(pattern=["git", "status"], decision="allow")"#;
    assert_eq!(
        requirement("xargs git", rules).await,
        requirement("xargs ls", rules).await
    );
}

/// `rules` plus managed requirements (`requirements.toml` `[rules]`), merged
/// as `load_exec_policy` merges them.
async fn managed_requirement(line: &str, rules: &str, managed: &str) -> ExecApprovalRequirement {
    let managed: codex_config::RequirementsExecPolicyToml =
        toml::from_str(managed).expect("parse managed rules");
    let policy =
        policy_from_src(Some(rules)).merge_overlay(&managed.to_policy().expect("managed policy"));
    let command = shlex::split(line).unwrap();
    ExecPolicyManager::new(Arc::new(policy))
        .create_exec_approval_requirement_for_command(ExecApprovalRequest {
            command: &command,
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            environment_policy: None,
            windows_sandbox_level: WindowsSandboxLevel::RestrictedToken,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
            allow_prefix_rules: AllowPrefixRules::Honor,
            cwd: None,
            env: None,
        })
        .await
}

const MANAGED_PROMPT_USER_PUSH: &str = r#"
[[prefix_rules]]
pattern = [{ token = "git" }, { token = "push" }]
decision = "prompt-user"
"#;

const MANAGED_FORBIDDEN_FORCE_PUSH: &str = r#"
[[prefix_rules]]
pattern = [{ token = "git" }, { token = "push" }]
decision = "prompt-user"

[[prefix_rules]]
pattern = [{ token = "git" }, { token = "push" }, { token = "--force" }]
decision = "forbidden"
"#;

/// #2506: an xargs command that stdin could complete into a managed
/// prompt-user rule needs a fresh human decision, never the configured
/// reviewer's plain approval (`printf push | xargs git`).
#[tokio::test]
async fn issue2506_xargs_keeps_prompt_user_for_a_rule_stdin_could_complete() {
    let rules = r#"prefix_rule(pattern=["xargs"], decision="allow")
prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;
    for managed in [MANAGED_PROMPT_USER_PUSH, MANAGED_FORBIDDEN_FORCE_PUSH] {
        for command in [
            "xargs git",
            "xargs -n 1 git",
            "xargs sudo git",
            "xargs -I{} git {}",
            // Every rule counts where stdin supplies the command.
            "xargs env",
            "xargs -0 sh -c",
            "xargs su",
            // Options xargs cannot read: nothing about its argv is known.
            "xargs --frobnicate git status",
            "xargs -n",
            "sudo xargs -q ls",
        ] {
            let result = managed_requirement(command, rules, managed).await;
            assert!(
                matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
                "{command}: {result:?}"
            );
        }
        // Fixed words no prompt-user rule can follow keep plain approval.
        let result = managed_requirement("xargs git fetch", rules, managed).await;
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{result:?}"
        );
        assert!(matches!(
            managed_requirement("xargs git status", rules, managed).await,
            ExecApprovalRequirement::Skip { .. }
        ));
    }
    // A rule the fixed words already match applies as usual.
    let result = managed_requirement("xargs git push", rules, MANAGED_PROMPT_USER_PUSH).await;
    assert!(
        matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
        "{result:?}"
    );
}

/// D691 (accepted 2026-11-17): a forbidden rule stdin could complete needs
/// a fresh human decision; one the fixed argv already matches still refuses.
#[tokio::test]
async fn d691_xargs_forbidden_rule_stdin_could_complete_needs_a_human() {
    let rules = r#"prefix_rule(pattern=["xargs"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in ["xargs git", "xargs -n 1 git", "xargs sh -c"] {
        let result = requirement(command, rules).await;
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
            "{command}: {result:?}"
        );
    }
    let result = requirement("xargs git push", rules).await;
    assert!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        "{result:?}"
    );
    for (rule, expected) in [
        (Decision::Allow, None),
        (Decision::Prompt, Some(Decision::Prompt)),
        (Decision::PromptUser, Some(Decision::PromptUser)),
        (Decision::Forbidden, Some(Decision::PromptUser)),
    ] {
        assert_eq!(wrappers::possible_rule_decision(rule), expected, "{rule:?}");
    }
}

const WRAPPER_RULES: &str = r#"prefix_rule(pattern=["su"], decision="allow")
prefix_rule(pattern=["sg"], decision="allow")
prefix_rule(pattern=["watch"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["env"], decision="allow")
prefix_rule(pattern=["chroot"], decision="allow")
prefix_rule(pattern=["timeout"], decision="allow")
prefix_rule(pattern=["nice"], decision="allow")
prefix_rule(pattern=["unshare"], decision="allow")
prefix_rule(pattern=["nsenter"], decision="allow")
prefix_rule(pattern=["flock"], decision="allow")
prefix_rule(pattern=["runuser"], decision="allow")
prefix_rule(pattern=["firejail"], decision="allow")
prefix_rule(pattern=["systemd-run"], decision="allow")
prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;

/// #2519 (D670): where a wrapper's command can be read, prompt-user and
/// forbidden rules apply to it exactly as to the direct command.
#[tokio::test]
async fn issue2519_read_wrappers_apply_rules_as_the_direct_command() {
    for managed in [MANAGED_PROMPT_USER_PUSH, MANAGED_FORBIDDEN_FORCE_PUSH] {
        for inner in ["git push", "git push --force", "git fetch", "git status"] {
            let direct = managed_requirement(inner, WRAPPER_RULES, managed).await;
            for wrapper in [
                "sudo",
                "sudo -u root",
                "env A=1",
                "env -i",
                "chroot /",
                "timeout 5",
                "nice -n 1",
                "unshare -r",
                "sudo timeout 5 env A=1",
            ] {
                let command = format!("{wrapper} {inner}");
                let result = managed_requirement(&command, WRAPPER_RULES, managed).await;
                assert_eq!(
                    std::mem::discriminant(&result),
                    std::mem::discriminant(&direct),
                    "{command}: {result:?} vs {direct:?}"
                );
            }
        }
    }
}

/// #2519 (D678): `watch -x` stays approval-only but runs its argv, so the
/// command's own rules apply: a prompt-user rule needs a human, a forbidden
/// one refuses, and other commands keep plain approval.
#[tokio::test]
async fn issue2519_watch_exec_applies_the_command_rules() {
    let human = |result: &ExecApprovalRequirement| {
        matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. })
    };
    for watch in [
        "watch -x",
        "watch -bx",
        "watch -x --",
        "watch -n 1 -x",
        "watch -n1 -tx",
        "watch -d -x",
        "watch -dpermanent -x",
        "watch --interval 1 --exec",
        "watch --interval=1 --differences=permanent --no-title -x",
        "sudo watch -x",
        "watch -x sudo",
    ] {
        for (inner, managed, expect_human) in [
            ("git push", MANAGED_PROMPT_USER_PUSH, true),
            ("git push --force", MANAGED_FORBIDDEN_FORCE_PUSH, false),
            ("git status", MANAGED_PROMPT_USER_PUSH, false),
            ("git fetch", MANAGED_FORBIDDEN_FORCE_PUSH, false),
        ] {
            let command = format!("{watch} {inner}");
            let result = managed_requirement(&command, WRAPPER_RULES, managed).await;
            if inner == "git push --force" {
                assert!(
                    matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                    "{command}: {result:?}"
                );
            } else if expect_human {
                assert!(human(&result), "{command}: {result:?}");
            } else {
                assert!(
                    matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
                    "{command}: {result:?}"
                );
            }
        }
    }
}

/// #2519: where a wrapper's command cannot be read (a shell string, unknown
/// options, a form D676-D678 keep approval-only), any prompt-user or
/// forbidden rule could match what it runs: a human decides, never the
/// configured reviewer's plain approval.
#[tokio::test]
async fn issue2519_unread_wrappers_need_a_human_when_a_restrictive_rule_exists() {
    let commands = [
        // D678: watch without -x runs `sh -c` on its words.
        "watch git push",
        "watch -n 1 git push",
        "watch --ex git push",
        "watch -q 1 -x git push",
        "watch -nx git push",
        "watch -dx git push",
        "watch -x -n",
        "watch -x",
        // D676: su with a command.
        "su -c 'git push'",
        "su root -c 'git status'",
        "su - root -c 'git push'",
        "sudo su -c 'git push'",
        // D678: sg runs a shell string.
        "sg wheel -c 'git push'",
        "sg wheel git push",
        // Options the wrapper tables do not know.
        "env -S 'git push'",
        "sudo --frobnicate git push",
        "chroot --frobnicate / git push",
        // D670 launchers: unknown options, shell strings, permutation.
        "timeout --frobnicate 5 git push",
        "nsenter --frobnicate git push",
        "flock /tmp/lock -c 'git push'",
        "runuser -u root git status -x",
        "firejail git status",
        "systemd-run --shell",
        // Under xargs, a replacement inside watch's options.
        "xargs -I{} watch -n{} -x git status",
    ];
    for managed in [MANAGED_PROMPT_USER_PUSH, MANAGED_FORBIDDEN_FORCE_PUSH] {
        for command in commands {
            let result = managed_requirement(command, WRAPPER_RULES, managed).await;
            let ExecApprovalRequirement::NeedsHumanApproval { reason } = &result else {
                panic!("{command}: {result:?}");
            };
            assert!(reason.is_some(), "{command}: {result:?}");
        }
    }
    // Without a prompt-user or forbidden rule they keep plain approval.
    for command in commands {
        let result = requirement(command, WRAPPER_RULES).await;
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{command}: {result:?}"
        );
    }
}

/// #2519: under xargs, `watch -x git` reads `git` as the program (stdin only
/// appends), so a prompt-user rule stdin could complete needs a human; a
/// read command marked uncertain (permutation) still refuses on an inner
/// forbidden rule.
#[tokio::test]
async fn issue2519_xargs_watch_exec_and_uncertain_read_commands() {
    let rules = format!("{WRAPPER_RULES}\nprefix_rule(pattern=[\"xargs\"], decision=\"allow\")");
    let result = managed_requirement("xargs watch -x git", &rules, MANAGED_PROMPT_USER_PUSH).await;
    assert!(
        matches!(result, ExecApprovalRequirement::NeedsHumanApproval { .. }),
        "{result:?}"
    );
    for command in [
        "runuser -u root git push --force -x",
        "firejail git push --force",
    ] {
        let result = managed_requirement(command, &rules, MANAGED_FORBIDDEN_FORCE_PUSH).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{command}: {result:?}"
        );
    }
}
