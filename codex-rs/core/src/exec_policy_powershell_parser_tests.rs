//! #2546 (D553): exec policy reads a PowerShell script in process and never
//! starts the command's own `argv[0]` before approval. Upstream's in-process
//! parser already does; this guards against a sync bringing a subprocess
//! parser back.

use super::*;
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

#[tokio::test]
async fn policy_never_starts_the_commands_own_powershell() {
    let dir = fake_powershell_dir();
    let fake = |name: &str| dir.path().join(name).to_string_lossy().into_owned();
    let marker = dir.path().join(FAKE_STARTED);

    for shell in [fake("powershell.exe"), fake("pwsh.exe")] {
        // The script is still read, so the rule applies to its command.
        for approval_policy in [AskForApproval::OnRequest, AskForApproval::Never] {
            let result = requirement(
                words(&[&shell, "-NoProfile", "-Command", "git push"]),
                Some(FORBID_GIT_PUSH),
                approval_policy,
            )
            .await;
            assert!(
                matches!(result, ExecApprovalRequirement::Forbidden { .. }),
                "{shell}: {result:?}"
            );
        }
        // Negative control: a script the rule does not match is not refused.
        let result = requirement(
            words(&[&shell, "-Command", "git status"]),
            Some(FORBID_GIT_PUSH),
            AskForApproval::OnRequest,
        )
        .await;
        assert!(
            !matches!(result, ExecApprovalRequirement::Forbidden { .. }),
            "{shell}: {result:?}"
        );
        // With no rules: the unmatched-command fallback.
        let _ = requirement(
            words(&[&shell, "-Command", "Get-Location"]),
            None,
            AskForApproval::UnlessTrusted,
        )
        .await;
    }

    assert!(
        !marker.exists(),
        "exec policy started the command's own PowerShell"
    );
}
