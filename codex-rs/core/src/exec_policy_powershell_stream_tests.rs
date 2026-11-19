//! #2699: a PowerShell spelled so Windows may run another file (an NTFS
//! stream, `pwsh.exe:x.exe`, or an 8.3 short-name tail) is never read as
//! PowerShell: Windows runs the stream's content, not the `-Command` script.
//! It stays one command, which asks wherever a prompt or forbidden rule
//! exists, at the top level and nested alike, and never offers an amendment.
use super::*;
use pretty_assertions::assert_eq;

const ALLOW_STATUS_FORBID_PUSH: &str = r#"prefix_rule(pattern=["git", "status"], decision="allow")
prefix_rule(pattern=["git", "push"], decision="forbidden")"#;

const STREAM_SHELLS: &[&str] = &[
    "pwsh.exe:x.exe",
    r"C:\PS\pwsh.exe:s",
    "pwsh.exe::$DATA",
    "powershell.exe:x.exe",
    r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe:s",
    "C:pwsh.exe:s",
];

fn stream_forms() -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    for &shell in STREAM_SHELLS {
        commands.push(words(&[shell, "-Command", "git status"]));
        commands.push(words(&[shell, "-NoProfile", "-c", "git", "status"]));
        commands.push(words(&[shell, "-EncodedCommand", &encode("git status")]));
        // Nested in a bash script, a PowerShell script and cmd.exe.
        commands.push(words(&[
            "bash",
            "-lc",
            &format!("'{shell}' -Command 'git status'"),
        ]));
        commands.push(words(&[
            "pwsh.exe",
            "-Command",
            &format!("& '{shell}' -Command 'git status'"),
        ]));
        commands.push(words(&[
            "cmd.exe",
            "/c",
            &format!("{shell} -Command \"git status\""),
        ]));
        // Behind a launcher.
        commands.push(words(&["env", shell, "-Command", "git status"]));
    }
    commands
}

#[test]
fn stream_spelled_powershell_is_not_read() {
    for &shell in STREAM_SHELLS {
        let command = words(&[shell, "-Command", "git status"]);
        assert_eq!(read_powershell_command_script(&command), None, "{shell}");
        assert_eq!(
            commands_for_exec_policy(&command).commands,
            vec![command.clone()],
            "{shell}"
        );
    }
}

#[tokio::test]
async fn stream_spelled_powershell_asks_without_amendment() {
    // The plain spelling reads its script: the allow rule applies.
    if trusted_pwsh_reads() {
        let plain = requirement(
            words(&["pwsh.exe", "-Command", "git status"]),
            Some(ALLOW_STATUS_FORBID_PUSH),
        )
        .await;
        assert!(
            matches!(plain, ExecApprovalRequirement::Skip { .. }),
            "{plain:?}"
        );
    }
    for command in stream_forms() {
        let result = requirement(command.clone(), Some(ALLOW_STATUS_FORBID_PUSH)).await;
        assert_eq!(result, ASKED, "{command:?}");
    }
}

#[tokio::test]
async fn stream_spelled_powershell_script_is_not_refused_as_its_script() {
    // Windows runs the stream, not `git push`: an unknown program, which asks.
    for &shell in STREAM_SHELLS {
        let command = words(&[shell, "-Command", "git push"]);
        let result = requirement(command.clone(), Some(FORBID_GIT_PUSH)).await;
        assert_eq!(result, ASKED, "{command:?}");
    }
}
