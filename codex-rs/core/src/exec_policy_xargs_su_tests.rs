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
            assert!(asks(&result), "{command}: {result:?}");
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
    assert!(asks(&result), "{result:?}");
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

/// D686 / D691 (pending): a forbidden rule stdin could complete asks for
/// approval, as on main; only a prompt-user rule needs a human.
#[tokio::test]
async fn issue2506_xargs_forbidden_rule_stdin_could_complete_keeps_main_parity() {
    let rules = r#"prefix_rule(pattern=["xargs"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    let result = requirement("xargs git", rules).await;
    assert!(asks(&result), "{result:?}");
    for (rule, expected) in [
        (Decision::Allow, None),
        (Decision::Prompt, Some(Decision::Prompt)),
        (Decision::PromptUser, Some(Decision::PromptUser)),
        (Decision::Forbidden, Some(Decision::Prompt)),
    ] {
        assert_eq!(wrappers::open_xargs_decision(rule), expected, "{rule:?}");
    }
}
