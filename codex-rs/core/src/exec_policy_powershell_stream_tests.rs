//! #2699 (sync): a PowerShell spelled so Windows may run another file (an
//! NTFS stream, `pwsh.exe:x.exe`, or an 8.3 short-name tail) is never read
//! as PowerShell: Windows runs the stream's content, not the `-Command`
//! script. For a Windows executor on any host it stays one command, which
//! asks wherever a prompt or forbidden rule exists, at the top level and
//! nested alike, never offers an amendment, and is refused with approvals
//! off.
use super::*;

const STREAM_SHELLS: &[&str] = &[
    "pwsh.exe:x.exe",
    r"C:\PS\pwsh.exe:s",
    "pwsh.exe::$DATA",
    "powershell.exe:x.exe",
    r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe:s",
    "C:pwsh.exe:s",
    "/opt/ps/pwsh.exe:x.exe",
    "pwsh.~1",
];

fn stream_forms(script: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    for &shell in STREAM_SHELLS {
        commands.push(argv(&[shell, "-Command", script]));
        commands.push(argv(&[shell, "-NoProfile", "-c", script]));
        // Nested in a bash script, a PowerShell script and cmd.exe.
        commands.push(argv(&[
            "bash",
            "-lc",
            &format!("'{shell}' -Command '{script}'"),
        ]));
        commands.push(argv(&[
            "pwsh.exe",
            "-Command",
            &format!("& '{shell}' -Command '{script}'"),
        ]));
        commands.push(argv(&[
            "cmd.exe",
            "/c",
            &format!("{shell} -Command \"{script}\""),
        ]));
        // Behind a launcher.
        commands.push(argv(&["env", shell, "-Command", script]));
    }
    commands
}

#[tokio::test]
async fn stream_spelled_powershell_asks_for_a_windows_executor() {
    let mut failures = Vec::new();
    // A harmless script the plain spelling reads (and runs unasked) too.
    for script in ["git status", "git push"] {
        for command in stream_forms(script) {
            for rules in [FORBID, PROMPT] {
                let results = windows_requirements(&command, rules).await;
                let last = results.len() - 1;
                for (mode, result) in results.into_iter().enumerate() {
                    if !acceptable(rules, &result, mode == last) {
                        failures.push(format!("{rules} mode {mode} {command:?}: {result:?}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn stream_spelled_powershell_is_not_lowered() {
    for &shell in STREAM_SHELLS {
        let command = argv(&[shell, "-Command", "git status"]);
        assert_eq!(
            commands_for_exec_policy_for_platform(&command, DangerousCommandPlatform::Windows)
                .commands,
            vec![command.clone()],
            "{shell}"
        );
    }
}
