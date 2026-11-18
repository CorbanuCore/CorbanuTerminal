//! #2568 (D553), sync port: exec policy reads a PowerShell command line as
//! PowerShell reads it: every argument after `-Command` is script,
//! `-EncodedCommand` is decoded, and parameters are abbreviated as
//! PowerShell allows. A command line it cannot read that way asks wherever
//! a prompt or forbidden rule could match (as an unread wrapper does,
//! #2519); `-File` is checked as its file and arguments, and asks too. The
//! script itself is parsed in process here (#2569), so every script form
//! reads without a trusted PowerShell.

use super::*;
use base64::Engine as _;
use codex_shell_command::powershell::UnreadPowershell;
use codex_shell_command::powershell::unread_powershell_command_line;
use pretty_assertions::assert_eq;

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(ToString::to_string).collect()
}

/// Base64 of UTF-16LE, as `-EncodedCommand` takes it.
fn encode(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

async fn requirement(command: Vec<String>, rules: Option<&str>) -> ExecApprovalRequirement {
    exec_approval_requirement_for_command(ExecApprovalRequirementScenario {
        policy_src: rules.map(str::to_string),
        command,
        approval_policy: AskForApproval::OnRequest,
        permission_profile: PermissionProfile::Disabled,
        sandbox_permissions: SandboxPermissions::UseDefault,
        prefix_rule: None,
    })
    .await
}

const FORBID_GIT_PUSH: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;
const FORBID_RM: &str = r#"prefix_rule(pattern=["rm"], decision="forbidden")"#;
/// Asked, with no amendment that could approve the unread script.
fn asked(result: &ExecApprovalRequirement) -> bool {
    matches!(
        result,
        ExecApprovalRequirement::NeedsApproval {
            proposed_execpolicy_amendment: None,
            ..
        } | ExecApprovalRequirement::NeedsHumanApproval { .. }
    )
}

async fn assert_forbidden(command: Vec<String>) {
    let result = requirement(command.clone(), Some(FORBID_GIT_PUSH)).await;
    assert!(
        matches!(result, ExecApprovalRequirement::Forbidden { .. }),
        "{command:?}: {result:?}"
    );
}

#[tokio::test]
async fn trailing_arguments_are_part_of_the_script() {
    for command in [
        words(&["powershell.exe", "-Command", "git status", "; git push"]),
        words(&["powershell.exe", "-c", "git", "push"]),
        words(&[
            "powershell.exe",
            "-NoProfile",
            "-Command",
            "Write-Output",
            "x;",
            "git",
            "push",
        ]),
        // Nested in a bash script, which passes the words apart (#2531,
        // now on this branch too).
        words(&[
            "bash",
            "-lc",
            "powershell.exe -Command 'git status' '; git push'",
        ]),
    ] {
        assert_forbidden(command).await;
    }
    // Negative control: the same shape with no forbidden command.
    let result = requirement(
        words(&["powershell.exe", "-Command", "git status", "; git log"]),
        Some(FORBID_GIT_PUSH),
    )
    .await;
    assert!(
        matches!(result, ExecApprovalRequirement::Skip { .. }),
        "{result:?}"
    );
}

#[tokio::test]
async fn parameters_and_abbreviations_are_read() {
    let encoded = encode("git push");
    let mut lines = vec![
        vec!["-NonInteractive", "-Command", "git push"],
        vec!["-nop", "-ep", "Bypass", "-w", "Hidden", "-c", "git push"],
        vec![
            "-ExecutionPolicy",
            "RemoteSigned",
            "-OutputFormat",
            "Text",
            "-c",
            "git push",
        ],
        vec!["--command", "git push"],
        vec!["/c", "git push"],
        vec!["-COMM", "git push"],
        vec!["-EncodedCommand", &encoded],
        vec!["-enc", &encoded],
        vec!["-ec", &encoded],
        vec!["-e", &encoded, "-NoProfile"],
    ];
    for line in &lines {
        let mut command = vec!["powershell.exe"];
        command.extend(line);
        assert_forbidden(words(&command)).await;
    }
    // PowerShell 7 too, with its own parameters.
    lines.extend([
        vec!["-wd", "C:\\", "-c", "git push"],
        vec!["-WorkingDirectory", "C:\\", "-NoProfileLoadTime", "-c", "git push"],
        vec!["-cwa", "git push"],
    ]);
    for line in &lines {
        let mut command = vec!["pwsh.exe"];
        command.extend(line);
        assert_forbidden(words(&command)).await;
    }
}

#[tokio::test]
async fn unreadable_command_lines_ask_when_a_rule_could_match() {
    for command in [
        words(&["powershell.exe", "-NoExit", "-Command", "git push"]),
        words(&["powershell.exe", "-Version", "2", "-Command", "git push"]),
        words(&["powershell.exe", "-Foo", "-Command", "git push"]),
        words(&["powershell.exe", "-Command:git push"]),
        words(&["powershell.exe", "git", "push"]),
        words(&["powershell.exe", "-Command", "-"]),
        words(&["powershell.exe", "-EncodedCommand", "!!!"]),
        words(&["powershell.exe", "-wd", "C:\\", "-c", "git push"]),
        words(&["powershell.exe", "\u{a0}-NoProfile", "-c", "git push"]),
        words(&["pwsh.exe", "-cwa", "& $args[0] $args[1]", "git", "push"]),
    ] {
        assert_eq!(
            unread_powershell_command_line(&command),
            Some(UnreadPowershell::Unreadable),
            "{command:?}"
        );
        // Any prompt or forbidden rule could match a command in it.
        for rules in [FORBID_GIT_PUSH, FORBID_RM] {
            let result = requirement(command.clone(), Some(rules)).await;
            assert!(asked(&result), "{command:?} {rules}: {result:?}");
        }
        // With no such rule it changes nothing.
        let result = requirement(command.clone(), None).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{command:?}: {result:?}"
        );
    }
}

#[tokio::test]
async fn file_reads_as_its_file_and_asks() {
    const FORBID_DEPLOY: &str = r#"prefix_rule(pattern=["deploy.ps1"], decision="forbidden")"#;
    for command in [
        words(&["powershell.exe", "-File", "deploy.ps1", "-Force"]),
        words(&["pwsh.exe", "-NoProfile", "-f", "deploy.ps1"]),
        // PowerShell 7 runs its first positional argument as a file.
        words(&["pwsh.exe", "deploy.ps1"]),
    ] {
        // Rules see the file as the program.
        let result = requirement(command.clone(), Some(FORBID_DEPLOY)).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{command:?}: {result:?}"
        );
        // Its content is not read: any prompt or forbidden rule asks.
        let result = requirement(command.clone(), Some(FORBID_GIT_PUSH)).await;
        assert!(asked(&result), "{command:?}: {result:?}");
        let result = requirement(command.clone(), None).await;
        assert!(
            matches!(result, ExecApprovalRequirement::Skip { .. }),
            "{command:?}: {result:?}"
        );
    }
}
