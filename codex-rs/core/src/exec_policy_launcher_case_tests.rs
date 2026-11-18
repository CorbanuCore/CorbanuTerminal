//! #2503 (D688): on macOS and Windows, program names resolve without regard
//! to case, so `SUDO`, `/usr/bin/Env` or `XARGS` run the launcher and rules
//! for the command they launch still apply. Linux keeps exact names.

use super::*;
use pretty_assertions::assert_eq;

const CASE_INSENSITIVE: bool = cfg!(any(target_os = "macos", windows));

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

fn asks(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        }
    )
}

fn by_policy(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            reason: Some(reason),
            proposed_execpolicy_amendment: None,
        } if reason.ends_with("requires approval by policy")
    )
}

/// Launchers that run their trailing argv, written with their options.
const LAUNCHERS: &[&str] = &[
    "sudo -n",
    "env FOO=1",
    "chroot /",
    "unshare -r",
    "nsenter -t 1 -m",
    "setpriv --nnp",
    "timeout 5",
    "nice -n 10",
    "nohup",
    "stdbuf -oL",
    "setsid",
    "ionice -c 3",
    "chrt 0",
    "taskset 1",
    "flock /tmp/lock",
    "doas --",
    "runuser -u root --",
    "command",
    "exec",
    "time -p",
    "caffeinate -i",
    "systemd-run --scope",
    "pkexec",
    "bwrap --ro-bind / / --dev /dev",
    "busybox",
    "toybox",
    "noglob",
    "nocorrect",
    "builtin",
    "xargs",
    "gxargs -0",
];

/// `launcher` with its program name respelled: upper case, capitalized,
/// with a directory, and with the non-ASCII letters APFS folds to ASCII.
fn spellings(launcher: &str) -> Vec<String> {
    let (name, rest) = launcher.split_once(' ').unwrap_or((launcher, ""));
    let upper = name.to_ascii_uppercase();
    let mut names = vec![
        upper.clone(),
        format!("{}{}", &upper[..1], &name[1..]),
        format!("/usr/bin/{upper}"),
    ];
    for (from, to) in [("s", "\u{17f}"), ("k", "\u{212a}"), ("fi", "\u{fb01}")] {
        if name.contains(from) {
            names.push(name.replace(from, to));
        }
    }
    if cfg!(windows) {
        names.push(format!("{upper}.EXE"));
        // Quoted: shlex reads an unquoted `\` as an escape.
        names.push(format!("'C:\\Tools\\{name}.exe'"));
        names.push(format!("{name}.exe."));
    }
    names
        .into_iter()
        .map(|name| format!("{name} {rest}").trim_end().to_string())
        .collect()
}

#[tokio::test]
async fn issue2503_rules_apply_behind_launchers_in_any_case() {
    let ws = "/work/ws";
    let forbidden = format!(
        r#"prefix_rule(pattern=["/bin/rm", "{ws}/docs/ordinary.txt"], decision="forbidden")"#
    );
    let prompt = format!(
        r#"prefix_rule(pattern=["/bin/rm", "{ws}/targets/target-one.txt"], decision="prompt")"#
    );
    let mut checked = 0;
    for launcher in LAUNCHERS {
        // The canonical spelling first: these must unwrap everywhere.
        for (spelled, unwraps) in std::iter::once((launcher.to_string(), true)).chain(
            spellings(launcher)
                .into_iter()
                .map(|spelled| (spelled, CASE_INSENSITIVE)),
        ) {
            checked += 1;
            let command = format!("{spelled} /bin/rm {ws}/docs/ordinary.txt");
            for result in [
                requirement(&command, &forbidden).await,
                escalated(&command, &forbidden).await,
            ] {
                let refused = matches!(
                    &result,
                    ExecApprovalRequirement::Forbidden { reason }
                        if reason.contains(&format!(
                            "policy forbids commands starting with `/bin/rm {ws}/docs/ordinary.txt`"
                        ))
                );
                assert_eq!(refused, unwraps, "{command}: {result:?}");
            }
            let command = format!("{spelled} /bin/rm {ws}/targets/target-one.txt");
            for result in [
                requirement(&command, &prompt).await,
                escalated(&command, &prompt).await,
            ] {
                assert_eq!(by_policy(&result), unwraps, "{command}: {result:?}");
            }
        }
    }
    assert!(checked > LAUNCHERS.len() * 4, "{checked}");
}

#[tokio::test]
async fn issue2503_chained_launchers_in_any_case() {
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;
    for chain in [
        "SUDO -n ENV FOO=1 git",
        "/usr/bin/Sudo -n /usr/bin/Env -i Nice -n 5 git",
        "Command Exec TIME -p UNSHARE -r CHROOT / git",
        "BusyBox NoGlob \u{17f}\u{75}do git",
        "Xargs -0 TIMEOUT 5 git",
    ] {
        let result = requirement(&format!("{chain} push"), rules).await;
        assert_eq!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            CASE_INSENSITIVE,
            "{chain}: {result:?}"
        );
        let result = requirement(&format!("{chain} fetch"), rules).await;
        assert_eq!(by_policy(&result), CASE_INSENSITIVE, "{chain}: {result:?}");
    }
}

const ALLOW_RULES: &str = r#"prefix_rule(pattern=["SU"], decision="allow")
prefix_rule(pattern=["Su"], decision="allow")
prefix_rule(pattern=["SG"], decision="allow")
prefix_rule(pattern=["WATCH"], decision="allow")
prefix_rule(pattern=["FIREJAIL"], decision="allow")
prefix_rule(pattern=["git"], decision="allow")"#;

#[tokio::test]
async fn issue2503_approval_only_launchers_in_any_case() {
    // D676/D678 and firejail: these ask whatever the allow rules say, under
    // any spelling the platform runs as the launcher.
    for command in [
        "SU -c 'git status'",
        "Su root -c 'git status'",
        "\u{17f}u -c 'git status'",
        "SG wheel git status",
        "WATCH git status",
        "FIREJAIL git status",
        "\u{fb01}rejail git status",
    ] {
        let result = requirement(command, ALLOW_RULES).await;
        assert_eq!(asks(&result), CASE_INSENSITIVE, "{command}: {result:?}");
    }
    // A login shell only: the allow rule still applies.
    for command in ["SU", "SU -", "Su root"] {
        assert!(
            matches!(
                requirement(command, ALLOW_RULES).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
}

const XARGS_RULES: &str = r#"prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")
prefix_rule(pattern=["git", "fetch"], decision="prompt")"#;

#[tokio::test]
async fn issue2503_xargs_in_any_case_keeps_d686_d687() {
    // D677/D686: prompt or forbidden rules that stdin could complete ask;
    // D687: forms where stdin supplies the command or script ask.
    for command in [
        "XARGS git",
        "/usr/bin/XArgs -0 git",
        "GXARGS git",
        "XARGS sh -c",
        "XARGS env",
        "XARGS command -v",
        "XARGS SH -c",
        "XARGS ENV",
        "XARGS SU",
    ] {
        for result in [
            requirement(command, XARGS_RULES).await,
            escalated(command, XARGS_RULES).await,
        ] {
            assert_eq!(asks(&result), CASE_INSENSITIVE, "{command}: {result:?}");
        }
    }
    // A forbidden rule the fixed argv matches refuses.
    let result = requirement("XARGS -0 git push", XARGS_RULES).await;
    assert_eq!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        CASE_INSENSITIVE,
        "{result:?}"
    );
    // D686: allow rules alone never add the xargs approval.
    let rules = r#"prefix_rule(pattern=["git", "status"], decision="allow")"#;
    for command in ["XARGS git", "XARGS git status", "Xargs ls"] {
        assert!(
            matches!(
                requirement(command, rules).await,
                ExecApprovalRequirement::Skip { .. }
            ),
            "{command}"
        );
    }
}

#[tokio::test]
async fn issue2503_other_names_are_not_launchers() {
    // Folding does not make other words launchers.
    let rules = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
    for command in [
        "SUDOX git push",
        "XARGSX git push",
        "sud\u{f3} git push",
        "\u{ff53}\u{75}do git push",
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
