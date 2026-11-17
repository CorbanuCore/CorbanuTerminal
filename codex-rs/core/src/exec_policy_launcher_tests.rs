//! #2455 (D670): rules apply through unshare, nsenter, timeout and the other
//! launchers that exec their trailing argv, as through sudo, chroot and env.

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

/// The request the model makes after the sandbox refused `unshare` (#2455):
/// to run outside it, in a workspace-write sandbox, approvals on request.
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

const LAUNCHER_ALLOW_RULES: &str = r#"prefix_rule(pattern=["unshare"], decision="allow")
prefix_rule(pattern=["nsenter"], decision="allow")
prefix_rule(pattern=["setpriv"], decision="allow")
prefix_rule(pattern=["timeout"], decision="allow")
prefix_rule(pattern=["nice"], decision="allow")
prefix_rule(pattern=["nohup"], decision="allow")
prefix_rule(pattern=["stdbuf"], decision="allow")
prefix_rule(pattern=["setsid"], decision="allow")
prefix_rule(pattern=["ionice"], decision="allow")
prefix_rule(pattern=["chrt"], decision="allow")
prefix_rule(pattern=["taskset"], decision="allow")
prefix_rule(pattern=["flock"], decision="allow")
prefix_rule(pattern=["doas"], decision="allow")
prefix_rule(pattern=["runuser"], decision="allow")
prefix_rule(pattern=["sudo"], decision="allow")"#;

#[tokio::test]
async fn issue2455_rules_apply_behind_unshare_chroot() {
    // The table in #2455: rules written for the direct command only.
    let ws = "/work/ws";
    let forbidden = format!(
        r#"prefix_rule(pattern=["/bin/rm", "{ws}/docs/ordinary.txt"], decision="forbidden")"#
    );
    let prompt = format!(
        r#"prefix_rule(pattern=["/bin/rm", "{ws}/targets/target-one.txt"], decision="prompt")"#
    );
    for wrapper in [
        "chroot /",
        "sudo -n chroot /",
        "unshare -r chroot /",
        "unshare -r",
        "unshare --map-root-user chroot /",
        "sudo -n unshare -r chroot /",
        "unshare -r sudo -n chroot /",
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
async fn issue2455_launchers_preserve_inner_rule_decisions() {
    let wrappers = [
        // unshare: short, long, combined, optional and valued options.
        "unshare -r",
        "unshare -U",
        "unshare -m",
        "unshare -p -f --mount-proc",
        "unshare -pf --mount-proc=/proc",
        "unshare -Urmpf",
        // Short namespace options take no value (only `--mount=…` does).
        "unshare -mR /",
        "unshare -UrR /",
        "unshare -nw /tmp",
        "unshare -mS 0",
        "unshare -iG0",
        "unshare -Cl /lib/ld.so",
        "unshare -r --",
        "unshare --map-root",
        "unshare --map-root-user --map-auto",
        "unshare -R / -w /tmp",
        "unshare -R/ -w/tmp",
        "unshare -rR /",
        "unshare --root / --wd=/tmp",
        "unshare --setgroups deny -U --mount=/run/ns --kill-child",
        "unshare --map-user 0 --map-group=0 -S 0 -G0",
        "/usr/bin/unshare -r",
        "sudo unshare -r",
        "env FOO=1 unshare -r chroot /",
        "unshare -r sudo -n",
        // Other launchers.
        "nsenter -t 1 -m -u -i -n -p",
        "nsenter -t1 -a",
        "nsenter --target=1 --mount=/proc/1/ns/mnt",
        "nsenter -m/proc/1/ns/mnt -S 0 -G 0",
        "nsenter -t 1 -r -w --",
        "nsenter --target 1 --preserve-credentials -F",
        "setpriv --reuid=1000 --regid 1000 --init-groups",
        "setpriv --no-new-privs --reset-env",
        "setpriv --nnp --",
        "timeout 5",
        "timeout -s KILL 5",
        "timeout -sKILL -k 1 5",
        "timeout --signal=TERM --kill-after 1 --preserve-status 5",
        "timeout -v -- 5",
        "nice",
        "nice -n 10",
        "nice -n10",
        "nice -10",
        "nice --5",
        "nice --adjustment=5",
        "nice --adj 5",
        "nohup",
        "nohup --",
        "stdbuf -oL",
        "stdbuf -o L -e 0",
        "stdbuf --output=L",
        "setsid",
        "setsid -f",
        "setsid --wait --fork",
        "ionice -c3",
        "ionice -c 2 -n 7",
        "ionice --class=idle -t",
        "chrt 10",
        "chrt -f 10",
        "chrt --rr 1",
        "chrt --idle",
        // Any integer `strtoimax` reads whole is the priority.
        "chrt -o +0",
        "chrt -o -- -0",
        "chrt -o ' 0'",
        "chrt -T 1 -P 2 -D 3 --deadline 0",
        "taskset 0x3",
        "taskset -c 0-3",
        "taskset --cpu-list 0,2",
        "flock /tmp/lock",
        "flock -n -w 5 /tmp/lock",
        "flock --timeout=5 -E 3 /tmp/lock",
        "flock -x -- /tmp/lock",
        "doas",
        "doas -n -u root",
        "doas -nu root",
        "runuser -u nobody --",
        "runuser -u nobody",
        "runuser --user=nobody -g users --",
        "timeout 5 nice -n 5 unshare -r chroot / sudo -n",
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
async fn issue2455_unknown_or_shell_launcher_forms_prompt_even_when_allowed() {
    let rules = format!(
        r#"{LAUNCHER_ALLOW_RULES}
prefix_rule(pattern=["git", "push"], decision="allow")"#
    );
    for command in [
        "unshare --future-option git push",
        "unshare -X git push",
        "unshare -rX git push",
        "unshare --m git push",
        "unshare --map-root-user=1 git push",
        "unshare -m/run/ns git push",
        "unshare -R",
        "unshare -r",
        "unshare",
        "sudo unshare --future-option git push",
        "nsenter -t 1 -W /x git push",
        "nsenter --wdns=/x git push",
        "nsenter -t 1",
        "setpriv --reuid=0",
        "setpriv --future git push",
        "timeout 5",
        "timeout --frob 5 git push",
        "nohup -x git push",
        "stdbuf -o",
        "nice -x git push",
        "chrt -x 5 git push",
        "taskset",
        "flock /tmp/l -c 'git push'",
        "flock /tmp/l --command 'git push'",
        "runuser nobody -c 'git push'",
        "runuser -l nobody",
        "runuser -u nobody -c 'git push'",
        "doas -s",
        "doas",
        // getopt may permute: a later dash word could be the launcher's.
        "doas git push -f",
        "runuser -u nobody git push -f",
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
async fn issue2455_possibly_permuted_launchers_still_refuse_forbidden_commands() {
    let rules = format!(
        r#"{LAUNCHER_ALLOW_RULES}
prefix_rule(pattern=["git", "push"], decision="forbidden")"#
    );
    for command in [
        "doas git push -f",
        "runuser -u nobody git push -f",
        "doas -u root -- git push -f",
        "runuser -u nobody -- git push -f",
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
async fn issue2455_operands_ids_and_queries_are_not_commands() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in [
        // Option values and operands.
        "unshare -R git echo push",
        "unshare --wd git echo push",
        "nsenter -t git echo push",
        "timeout -s git 5 echo push",
        "timeout git echo push",
        "nice -n git echo push",
        "stdbuf -o git echo push",
        "ionice -c git echo push",
        "chrt 5 echo git push",
        "taskset git echo push",
        "flock git echo push",
        "doas -u git echo push",
        "runuser -u git echo push",
        // Modes that change other processes or only print.
        "ionice -p 1 git push",
        "ionice -c3 -P 1 git push",
        "chrt -p 5 git push",
        "chrt -m git push",
        "taskset -p 3 git push",
        "nice",
        "ionice",
        "flock 9",
        "setpriv --dump",
        "unshare --help",
        "doas -L",
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
