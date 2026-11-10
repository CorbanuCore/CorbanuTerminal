use super::*;
use crate::EnvironmentCapabilities;
use crate::ShellInfo;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;

use HostSandboxVerdict::Confined;
use HostSandboxVerdict::Unreported;
use HostSandboxVerdict::WindowsWithoutSandbox;
use WindowsSandboxLevel::Disabled;
use WindowsSandboxLevel::Elevated;
use WindowsSandboxLevel::RestrictedToken;

fn info(shell: (&str, &str), cwd: Option<&str>, host: Option<HostSandboxInfo>) -> EnvironmentInfo {
    EnvironmentInfo {
        shell: ShellInfo {
            name: shell.0.to_string(),
            path: shell.1.to_string(),
        },
        cwd: cwd.map(|cwd| PathUri::parse(cwd).expect("cwd uri")),
        capabilities: EnvironmentCapabilities::default(),
        host_sandbox: host,
    }
}

fn verdicts(info: &EnvironmentInfo) -> [HostSandboxVerdict; 3] {
    [Disabled, RestrictedToken, Elevated].map(|level| info.host_sandbox_verdict(level))
}

const POWERSHELL: (&str, &str) = (
    "powershell",
    r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
);
const ZSH: (&str, &str) = ("zsh", "/bin/zsh");

/// #2050: a Windows executor confines work only with a Windows sandbox for
/// the level the client sends: none, or the elevated one not set up there,
/// is refused; Linux and macOS executors always confine it.
#[test]
fn issue_2050_windows_host_without_its_sandbox_is_not_confined() {
    let windows = |set_up| {
        Some(HostSandboxInfo {
            os: HostOs::Windows,
            windows_sandbox_set_up: set_up,
        })
    };
    assert_eq!(
        verdicts(&info(POWERSHELL, Some("file:///C:/work"), windows(false))),
        [WindowsWithoutSandbox, Confined, WindowsWithoutSandbox]
    );
    assert_eq!(
        verdicts(&info(POWERSHELL, Some("file:///C:/work"), windows(true))),
        [WindowsWithoutSandbox, Confined, Confined]
    );
    for os in [HostOs::Linux, HostOs::Macos] {
        let host = Some(HostSandboxInfo {
            os,
            windows_sandbox_set_up: false,
        });
        assert_eq!(
            verdicts(&info(ZSH, Some("file:///home/me"), host)),
            [Confined; 3]
        );
    }
}

/// #2050: an executor that predates the report is refused unless it clearly
/// runs on Linux or macOS (fail closed for Windows and for unclear ones).
#[test]
fn issue_2050_older_executors_fail_closed_unless_clearly_posix() {
    // Older Windows executors: a Windows working directory or shell.
    for older_windows in [
        info(POWERSHELL, Some("file:///C:/work"), /*host*/ None),
        info(POWERSHELL, /*cwd*/ None, /*host*/ None),
        info(("cmd", "cmd.exe"), /*cwd*/ None, /*host*/ None),
        info(
            ("bash", r"C:\Program Files\Git\bin\bash.exe"),
            /*cwd*/ None,
            /*host*/ None,
        ),
        info(ZSH, Some("file:///C:/work"), /*host*/ None),
        info(ZSH, Some("file://server/share/work"), /*host*/ None),
    ] {
        assert_eq!(
            verdicts(&older_windows),
            [Unreported; 3],
            "{older_windows:?}"
        );
    }
    // Nothing to tell the platform by.
    assert_eq!(
        verdicts(&info(("sh", "sh"), /*cwd*/ None, /*host*/ None)),
        [Unreported; 3]
    );
    // Older Linux or macOS executors keep working.
    for older_posix in [
        info(ZSH, Some("file:///home/me"), /*host*/ None),
        info(ZSH, /*cwd*/ None, /*host*/ None),
        info(("sh", "sh"), Some("file:///srv/work"), /*host*/ None),
    ] {
        assert_eq!(verdicts(&older_posix), [Confined; 3], "{older_posix:?}");
    }
}

/// #2050: the report is optional on the wire both ways: older clients ignore
/// it, and older executors' responses still parse (with no report).
#[test]
fn issue_2050_host_sandbox_report_is_backward_compatible() {
    let reported = info(
        POWERSHELL,
        Some("file:///C:/work"),
        Some(HostSandboxInfo {
            os: HostOs::Windows,
            windows_sandbox_set_up: true,
        }),
    );
    let json = serde_json::to_value(&reported).expect("serialize");
    assert_eq!(
        json["hostSandbox"],
        serde_json::json!({ "os": "windows", "windowsSandboxSetUp": true })
    );
    let parsed: EnvironmentInfo = serde_json::from_value(json).expect("parse");
    assert_eq!(parsed, reported);

    let older: EnvironmentInfo = serde_json::from_value(serde_json::json!({
        "shell": { "name": "zsh", "path": "/bin/zsh" }
    }))
    .expect("older executor info");
    assert_eq!(older.host_sandbox, None);
    let unreported =
        serde_json::to_value(info(ZSH, /*cwd*/ None, /*host*/ None)).expect("serialize");
    assert!(unreported.get("hostSandbox").is_none());
    // An operating system this client does not know: it cannot tell.
    let future: HostSandboxInfo =
        serde_json::from_value(serde_json::json!({ "os": "plan9" })).expect("future os");
    assert_eq!(future.os, HostOs::Other);
    assert_eq!(
        verdicts(&info(ZSH, Some("file:///home/me"), Some(future))),
        [Unreported; 3]
    );
}
