//! #2455 follow-up (D670): rules apply through the shell builtins `command`
//! and `exec`, `time`, `caffeinate`, `systemd-run`, `pkexec`, `bwrap`,
//! `firejail`, `busybox`/`toybox` applets and zsh's precommand modifiers,
//! as through unshare and sudo.

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

/// As in #2455: a request to run outside a workspace-write sandbox.
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

const ALLOW_RULES: &str = r#"prefix_rule(pattern=["command"], decision="allow")
prefix_rule(pattern=["exec"], decision="allow")
prefix_rule(pattern=["time"], decision="allow")
prefix_rule(pattern=["caffeinate"], decision="allow")
prefix_rule(pattern=["systemd-run"], decision="allow")
prefix_rule(pattern=["pkexec"], decision="allow")
prefix_rule(pattern=["bwrap"], decision="allow")
prefix_rule(pattern=["firejail"], decision="allow")
prefix_rule(pattern=["busybox"], decision="allow")
prefix_rule(pattern=["toybox"], decision="allow")
prefix_rule(pattern=["builtin"], decision="allow")
prefix_rule(pattern=["noglob"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")
prefix_rule(pattern=["unshare"], decision="allow")"#;

#[tokio::test]
async fn issue2455b_rules_apply_behind_more_launchers() {
    // The #2455 table, through each launcher.
    let ws = "/work/ws";
    let forbidden = format!(
        r#"prefix_rule(pattern=["/bin/rm", "{ws}/docs/ordinary.txt"], decision="forbidden")"#
    );
    let prompt = format!(
        r#"prefix_rule(pattern=["/bin/rm", "{ws}/targets/target-one.txt"], decision="prompt")"#
    );
    for wrapper in [
        "command",
        "command -p",
        "exec",
        "exec -a rm",
        "time -p",
        "/usr/bin/time",
        "caffeinate -i",
        "systemd-run --scope",
        "pkexec",
        "bwrap --ro-bind / / --dev /dev",
        "busybox",
        "command exec time -p sudo -n unshare -r chroot /",
    ] {
        let command = format!("{wrapper} /bin/rm {ws}/docs/ordinary.txt");
        for result in [
            requirement(&command, &forbidden).await,
            escalated(&command, &forbidden).await,
        ] {
            let ExecApprovalRequirement::Forbidden { reason } = result else {
                panic!("{command}: {result:?}");
            };
            assert!(
                reason.contains(&format!(
                    "policy forbids commands starting with `/bin/rm {ws}/docs/ordinary.txt`"
                )),
                "{command}: {reason}"
            );
        }
        let command = format!("{wrapper} /bin/rm {ws}/targets/target-one.txt");
        for result in [
            requirement(&command, &prompt).await,
            escalated(&command, &prompt).await,
        ] {
            let ExecApprovalRequirement::NeedsApproval {
                reason: Some(reason),
                proposed_execpolicy_amendment: None,
            } = &result
            else {
                panic!("{command}: {result:?}");
            };
            assert!(
                reason.ends_with("requires approval by policy"),
                "{command}: {reason}"
            );
        }
    }
}

#[tokio::test]
async fn issue2455b_launchers_preserve_inner_rule_decisions() {
    let wrappers = [
        "command",
        "command -p",
        "command -p --",
        "command --",
        "exec",
        "exec -c",
        "exec -cl",
        "exec -a name",
        "exec -aname",
        "exec -c -a name --",
        "time",
        "time -p",
        "time -p --",
        "/usr/bin/time -v",
        "time -f %e -o /tmp/t -a",
        "time --format=%e --output /tmp/t --append",
        "time -q --portability",
        "time -l -h",
        "caffeinate",
        "caffeinate -dimsu",
        "caffeinate -t 60",
        "caffeinate -t60 -w 1 --",
        "systemd-run",
        "systemd-run --scope",
        "systemd-run --user --scope -q",
        "systemd-run -p MemoryMax=1G --wait -t",
        "systemd-run --property=CPUQuota=10% -E FOO=1 --uid=0",
        "systemd-run -u unit --slice=x --on-active=5",
        "systemd-run -tq --",
        "systemd-run --setenv FOO=1 -M machine -H host",
        "pkexec",
        "pkexec --user root",
        "pkexec -u root --keep-cwd",
        "pkexec --disable-internal-agent",
        "bwrap --ro-bind / / --dev /dev --unshare-all --",
        "bwrap --bind /a /b --setenv A B --chmod 0755 /x --overlay a b c --argv0 x",
        "bwrap --die-with-parent --proc /proc",
        "busybox",
        "/bin/busybox",
        "toybox",
        "busybox nice -n 5",
        "busybox timeout 5",
        "command exec time -p busybox",
        "sudo -n exec command -p",
        "systemd-run --scope unshare -r chroot /",
        // zsh precommand modifiers, which `exec` passes through.
        "-",
        "noglob",
        "nocorrect",
        "builtin",
        "builtin --",
        "builtin exec",
        "exec -",
        "exec -c -",
        "exec -- -",
        "exec noglob",
        "noglob -",
        "nocorrect - noglob",
        "sudo -n noglob",
        // zsh's `-` repeats, and follows `builtin` too.
        "builtin -",
        "builtin builtin -",
        "- - -",
        "noglob - -",
        "nocorrect - - -",
        "exec - -",
        "exec -c - -",
        "exec - - - noglob -",
        // Any busybox… or toybox… name is the multicall binary.
        "busybox.static",
        "/bin/busybox-static",
        "busybox busybox-x",
        "busybox /bin/busybox",
        "toybox toybox.foo",
    ];
    for wrapper in wrappers {
        for decision in ["allow", "prompt", "forbidden"] {
            let rules = format!(
                r#"prefix_rule(pattern=["git", "push"], decision="{decision}", justification="inner rule")"#
            );
            let command = format!("{wrapper} git push");
            let result = requirement(&command, &rules).await;
            match decision {
                "allow" => assert!(
                    matches!(result, ExecApprovalRequirement::Skip { .. }),
                    "{command}: {result:?}"
                ),
                "prompt" => assert!(
                    matches!(
                        &result,
                        ExecApprovalRequirement::NeedsApproval {
                            reason: Some(reason),
                            proposed_execpolicy_amendment: None,
                        } if reason.ends_with("requires approval: inner rule")
                    ),
                    "{command}: {result:?}"
                ),
                "forbidden" => assert!(
                    matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                    "{command}: {result:?}"
                ),
                _ => unreachable!(),
            }
        }
    }
}

#[tokio::test]
async fn issue2455b_shell_strings_apply_rules_through_builtins() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for script in [
        "command git push",
        "command -p git push",
        "exec git push",
        "exec -a x git push",
        "time git push",
        "time -p git push",
        "busybox git push",
    ] {
        let command = vec!["bash".to_string(), "-lc".to_string(), script.to_string()];
        let result = exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
            policy_src: Some(rules.to_string()),
            command,
            approval_policy: AskForApproval::OnRequest,
            permission_profile: PermissionProfile::Disabled,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: None,
        })
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{script}: {result:?}"
        );
    }
}

#[tokio::test]
async fn issue2455b_unknown_or_shell_forms_prompt_even_when_allowed() {
    let rules = format!(
        r#"{ALLOW_RULES}
prefix_rule(pattern=["git", "push"], decision="allow")"#
    );
    for command in [
        "command -x git push",
        "command --help git push",
        "exec -z git push",
        "exec -a",
        "time -z git push",
        "time --frob git push",
        "caffeinate -x git push",
        "systemd-run --future git push",
        "systemd-run -S",
        "systemd-run --shell git push",
        "systemd-run -p ExecStartPre=/bin/rm git push",
        "systemd-run --property ExecStopPost=/x git push",
        "systemd-run --socket-property=ExecStartPre=/x git push",
        "systemd-run",
        "systemd-run -u",
        // A floor stays: abbreviations and `$` expansion.
        "systemd-run --wai git push",
        "systemd-run git push '$X'",
        "pkexec",
        "pkexec -x git push",
        "pkexec --user=root git push",
        "pkexec -- git push",
        "pkexec --user",
        "bwrap --args 3 git push",
        "bwrap --frob git push",
        "bwrap -x git push",
        "bwrap --bind /a",
        "bwrap",
        "bwrap --ro-bind / /",
        // Some firejail versions run the program through a shell.
        "firejail git push",
        "firejail --noprofile git push",
        "firejail --private=/tmp -- git push",
        "firejail",
        "firejail -c 'git push'",
        "firejail --ls=x /tmp",
        "busybox -x git push",
        "busybox --install git push",
        "toybox -x git push",
        // The applet is the base name: a path need not match a rule.
        "busybox ./git push",
        "busybox /tmp/git push",
        "builtin -x git push",
        "noglob -x git push",
        "- -x git push",
        // bash may run `command - cmd`; zsh refuses it.
        "command - git push",
    ] {
        assert_eq!(
            requirement(command, &rules).await,
            ExecApprovalRequirement::NeedsApproval {
                reason: None,
                proposed_execpolicy_amendment: None,
            },
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2455b_uncertain_commands_still_refuse_forbidden_commands() {
    let rules = format!(
        r#"{ALLOW_RULES}
prefix_rule(pattern=["git", "push"], decision="forbidden")"#
    );
    for command in [
        "firejail git push",
        "firejail --private -- git push",
        "systemd-run --wai git push",
        "systemd-run --sco git push",
        "systemd-run git push '$X'",
        "command - git push",
    ] {
        assert!(
            matches!(
                requirement(command, &rules).await,
                ExecApprovalRequirement::Forbidden { .. }
            ),
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2455b_operands_and_queries_are_not_commands() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in [
        // Only print.
        "command -v git push",
        "command -V git push",
        "command -pv git push",
        "command",
        "exec",
        "time",
        "caffeinate",
        "pkexec --version git push",
        "pkexec --help",
        "bwrap --help git push",
        "bwrap --version",
        "busybox",
        "busybox --list",
        "toybox --long",
        "noglob",
        "builtin",
        // Option values and operands.
        "exec -a git echo push",
        "time -o git echo push",
        "time -f git echo push",
        "caffeinate -t git echo push",
        "systemd-run -u git echo push",
        "systemd-run --description git echo push",
        "pkexec -u git echo push",
        "bwrap --bind git push echo x",
        "bwrap --chmod 0 git echo push",
        // Not launchers.
        "commander git push",
        "executor git push",
        "timed git push",
        "mybusybox git push",
        // After a program, `-` is a program named `-`, not zsh's modifier.
        "nice - git push",
        "timeout 5 - git push",
        "sudo - git push",
    ] {
        assert!(
            matches!(
                requirement(command, rules).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
}
