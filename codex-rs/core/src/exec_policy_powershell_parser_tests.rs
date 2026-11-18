//! #2546 (D553): exec policy reads a PowerShell script with a trusted
//! PowerShell found on the system, never by starting the command's own
//! `argv[0]`, and asks where no trusted PowerShell can read it.

use super::*;
use codex_shell_command::powershell::hide_trusted_powershell_on_this_thread;
use pretty_assertions::assert_eq;
use std::process::Command;

/// Written next to a fake PowerShell when it is started.
const FAKE_STARTED: &str = "fake-powershell-started";

/// A copy of this test binary named `pwsh.exe` or `powershell.exe` is a fake
/// PowerShell: started, it records that in its directory and exits.
#[ctor::ctor(unsafe)]
fn record_start_as_fake_powershell() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let is_fake = exe
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| {
            stem.eq_ignore_ascii_case("pwsh") || stem.eq_ignore_ascii_case("powershell")
        });
    if !is_fake {
        return;
    }
    if let Some(dir) = exe.parent() {
        let _ = std::fs::write(dir.join(FAKE_STARTED), b"started");
    }
    std::process::exit(0);
}

/// A directory holding fake `pwsh.exe` and `powershell.exe`, checked to
/// leave the marker when started.
fn fake_powershell_dir() -> TempDir {
    let exe = std::env::current_exe().expect("current exe");
    // Next to the test binary, so a hard link works; else anywhere.
    let dir = exe
        .parent()
        .and_then(|parent| tempfile::tempdir_in(parent).ok())
        .unwrap_or_else(|| tempdir().expect("tempdir"));
    for name in ["pwsh.exe", "powershell.exe"] {
        let fake = dir.path().join(name);
        if fs::hard_link(&exe, &fake).is_err() {
            fs::copy(&exe, &fake).expect("copy fake PowerShell");
        }
    }
    // Positive control: started, a fake leaves the marker.
    let marker = dir.path().join(FAKE_STARTED);
    let status = Command::new(dir.path().join("pwsh.exe"))
        .args(["-NoLogo", "-NoProfile"])
        .status()
        .expect("start fake PowerShell");
    assert!(status.success(), "{status:?}");
    assert!(marker.exists(), "the fake PowerShell left no marker");
    fs::remove_file(&marker).expect("remove marker");
    dir
}

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(ToString::to_string).collect()
}

async fn requirement(
    command: Vec<String>,
    rules: Option<&str>,
    approval_policy: AskForApproval,
) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: rules.map(str::to_string),
        command,
        approval_policy,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

const FORBID_GIT_PUSH: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const PROMPT_GIT_PUSH: &str = r#"prefix_rule(pattern=["git", "push"], decision="prompt")"#;

#[tokio::test]
async fn policy_never_starts_the_commands_own_powershell() {
    let dir = fake_powershell_dir();
    let fake = |name: &str| dir.path().join(name).to_string_lossy().into_owned();
    let marker = dir.path().join(FAKE_STARTED);

    // The trusted parser still reads the script, so the rule applies.
    for shell in [fake("powershell.exe"), fake("PowerShell.EXE")] {
        let result = requirement(
            words(&[&shell, "-NoProfile", "-Command", "git push"]),
            Some(FORBID_GIT_PUSH),
            AskForApproval::OnRequest,
        )
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{shell}: {result:?}"
        );
        assert_eq!(
            commands_for_exec_policy(&words(&[&shell, "-Command", "echo hi"])),
            ExecPolicyCommands {
                commands: vec![words(&["echo", "hi"])],
                used_complex_parsing: false,
                command_origin: ExecPolicyCommandOrigin::PowerShell,
            },
            "{shell}"
        );
    }
    // `pwsh` is read by a trusted `pwsh`; without one it asks (below).
    for shell in [fake("pwsh.exe"), fake("PWSH.EXE")] {
        let result = requirement(
            words(&[&shell, "-Command", "git push"]),
            Some(FORBID_GIT_PUSH),
            AskForApproval::Never,
        )
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{shell}: {result:?}"
        );
    }
    // With no rules: the unmatched-command fallback, and the safe-command
    // check, which parses the script too.
    for shell in [fake("powershell.exe"), fake("pwsh.exe")] {
        let _ = requirement(
            words(&[&shell, "-Command", "Get-Location"]),
            None,
            AskForApproval::UnlessTrusted,
        )
        .await;
        let _ = codex_shell_command::is_safe_command::is_known_safe_command(&words(&[
            &shell,
            "-Command",
            "Get-Location",
        ]));
    }
    // #2531: nested in another PowerShell script.
    let nested = fake("pwsh.exe");
    if !nested.contains([' ', '\'', '"', '`', '$']) {
        let result = requirement(
            words(&[
                "powershell.exe",
                "-Command",
                &format!("{nested} -Command 'git push'"),
            ]),
            Some(FORBID_GIT_PUSH),
            AskForApproval::Never,
        )
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{result:?}"
        );
    }

    assert!(
        !marker.exists(),
        "exec policy started the command's own PowerShell"
    );
}

#[tokio::test]
async fn unreadable_powershell_script_asks_when_a_rule_could_match() {
    let _hidden = hide_trusted_powershell_on_this_thread();
    let command = words(&["powershell.exe", "-NoProfile", "-Command", "git push"]);
    assert_eq!(
        codex_shell_command::powershell::read_powershell_command_script(&command),
        Some(codex_shell_command::powershell::PowershellScriptRead::Unreadable)
    );
    // Rules cannot see the script's commands, so the literal argv alone
    // would run it unasked under full access.
    for rules in [
        FORBID_GIT_PUSH,
        PROMPT_GIT_PUSH,
        // Any prompt or forbidden rule could match a command in the script.
        r#"prefix_rule(pattern=["rm"], decision="forbidden")"#,
    ] {
        let result = requirement(command.clone(), Some(rules), AskForApproval::OnRequest).await;
        assert_eq!(
            result,
            ExecApprovalRequirement::NeedsApproval {
                reason: None,
                proposed_execpolicy_amendment: None,
            },
            "{rules}"
        );
        let result = requirement(command.clone(), Some(rules), AskForApproval::Never).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{rules}: {result:?}"
        );
    }
    // Respelled and nested in a bash script too.
    for command in [
        words(&["PWSH", "-c", "git push"]),
        words(&["bash", "-lc", "pwsh -Command git"]),
    ] {
        let result = requirement(
            command.clone(),
            Some(FORBID_GIT_PUSH),
            AskForApproval::OnRequest,
        )
        .await;
        assert!(
            matches!(result, ExecApprovalRequirement::NeedsApproval { .. }),
            "{command:?}: {result:?}"
        );
    }
}

#[tokio::test]
async fn unreadable_powershell_script_changes_nothing_without_strict_rules() {
    // The literal argv decides, as for any command rules cannot read, and
    // its amendment stays available: a later prompt or forbidden rule still
    // asks for the unread script, since the strictest match wins.
    let _hidden = hide_trusted_powershell_on_this_thread();
    let command = words(&["powershell.exe", "-Command", "git push"]);
    for rules in [
        None,
        Some(r#"prefix_rule(pattern=["git"], decision="allow")"#),
    ] {
        let unread = requirement(command.clone(), rules, AskForApproval::OnRequest).await;
        assert_eq!(
            unread,
            ExecApprovalRequirement::Skip {
                bypass_sandbox: false,
                proposed_execpolicy_amendment: Some(ExecPolicyAmendment::new(command.clone())),
            },
            "{rules:?}"
        );
    }
    // Once remembered, that amendment cannot outrank a strict rule.
    let rules = format!(
        "prefix_rule(pattern=[\"powershell.exe\", \"-Command\", \"git push\"], decision=\"allow\")\n{FORBID_GIT_PUSH}"
    );
    let result = requirement(command.clone(), Some(&rules), AskForApproval::OnRequest).await;
    assert_eq!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            reason: None,
            proposed_execpolicy_amendment: None,
        }
    );
}

#[tokio::test]
async fn trusted_parser_keeps_reading_scripts() {
    // Negative controls: a readable script decides by its own commands.
    let result = requirement(
        words(&["powershell.exe", "-Command", "git status"]),
        Some(FORBID_GIT_PUSH),
        AskForApproval::OnRequest,
    )
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Skip { .. }),
        "{result:?}"
    );
    let result = requirement(
        words(&["powershell.exe", "-Command", "git push"]),
        Some(FORBID_GIT_PUSH),
        AskForApproval::OnRequest,
    )
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        "{result:?}"
    );
}
