//! D729 (A), #2661: on Windows a PowerShell script is read in full only when every
//! command is an allowlisted cmdlet (none writes files) or a native program found on the
//! command's `PATH`. Anything else asks whenever a prompt or forbidden rule exists, and is
//! refused with approvals off.
use super::*;

const GIT_PUSH_FORBIDDEN: &str = r#"prefix_rule(pattern=["git", "push"], decision="forbidden")"#;

fn home() -> String {
    std::env::var("USERPROFILE").expect("USERPROFILE")
}

fn documents() -> String {
    format!(r"{}\Documents", home())
}

/// The command's environment: a `PATH` holding only a temporary folder with
/// the programs these scripts run, so they read alike wherever the tests run
/// (Bazel's `PATH` holds none of them).
fn command_env() -> std::collections::HashMap<String, String> {
    static BIN: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let bin = BIN.get_or_init(|| {
        let bin = tempfile::tempdir().expect("bin");
        for program in ["git.exe", "pnpm.cmd"] {
            std::fs::write(bin.path().join(program), "").expect("program");
        }
        bin
    });
    std::collections::HashMap::from([("Path".to_string(), bin.path().display().to_string())])
}

async fn decide_in(
    shell: &str,
    script: &str,
    cwd: &Path,
    approval_policy: AskForApproval,
    permission_profile: PermissionProfile,
) -> ExecApprovalRequirement {
    let env = command_env();
    let command = vec![
        shell.to_string(),
        "-NoProfile".to_string(),
        "-Command".to_string(),
        script.to_string(),
    ];
    ExecPolicyManager::new(policy_from_src(Some(GIT_PUSH_FORBIDDEN)))
        .create_exec_approval_requirement_for_command(ExecApprovalRequest {
            command: &command,
            approval_policy,
            permission_profile,
            windows_sandbox_level: WindowsSandboxLevel::RestrictedToken,
            sandbox_permissions: SandboxPermissions::UseDefault,
            prefix_rule: Some(command.clone()),
            cwd: Some(AbsolutePathBuf::from_absolute_path(cwd).expect("absolute cwd")),
            env: Some(&env),
        })
        .await
}

/// The shells whose trusted parser is present: Windows PowerShell, and PowerShell 7
/// where installed.
fn shells() -> Vec<&'static str> {
    let mut shells = vec!["powershell.exe"];
    if codex_shell_command::powershell::read_powershell_command_script(&[
        "pwsh.exe".to_string(),
        "-Command".to_string(),
        "git status".to_string(),
    ]) != Some(codex_shell_command::powershell::PowershellScriptRead::Unreadable)
    {
        shells.push("pwsh.exe");
    }
    shells
}

fn unread_scripts() -> Vec<String> {
    let documents = documents();
    let modules = format!(r"{documents}\WindowsPowerShell\Modules");
    vec![
        // #2661 measurement probes that ran their payload on main.
        "Get-Variable ExecutionContext -ValueOnly | ForEach-Object InvokeCommand | ForEach-Object InvokeScript 'git push'".into(),
        "Get-Variable ExecutionContext -ValueOnly | % InvokeCommand | % InvokeScript 'git push'".into(),
        "Get-Item variable:ExecutionContext | % Value | % InvokeCommand | % InvokeScript 'git push'".into(),
        "New-Object -ComObject WScript.Shell | % Run 'git push'".into(),
        r#"Write-Output '{"Path":"alias:f","Value":"git"}' | ConvertFrom-Json | Set-Item; f push"#.into(),
        "'alias:iex' | Rename-Item -NewName zz; zz 'git push'".into(),
        "Set-Item -Path [a]lias:f -Value git; f push".into(),
        "New-Item -Path [f]unction:f -Value 'git push'; f".into(),
        "Start-Process git push".into(),
        "start git push".into(),
        "Set-Content x.bat 'git push'; Invoke-Item x.bat".into(),
        "Trace-Command -Name ParameterBinding -Command git -ArgumentList push".into(),
        "Invoke-WmiMethod -Class Win32_Process -Name Create -ArgumentList 'git push'".into(),
        format!(r"New-Item -ItemType Directory -Force -Path {modules}\Z9 | Out-Null; Set-Content -Path {modules}\Z9\Z9.psm1 -Value 'function Select-String {{ git push }}'; Select-String -Path README.md -Pattern x"),
        format!(r"Set-Content -Path {modules}\Z8\Z8.psm1 -Value 'function Get-ChildItem {{ git push }}'"),
        r"Set-Item env:PSModulePath C:\Temp\m; Select-String -Path README.md -Pattern x".into(),
        r"Set-ItemProperty -Path HKCU:\Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell -Name ExecutionPolicy -Value Bypass".into(),
        "Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy Bypass -Force".into(),
        format!(r"Set-Content -Path {documents}\WindowsPowerShell\Microsoft.PowerShell_profile.ps1 -Value 'git push'"),
        r"Import-Csv C:\Temp\a.csv | Set-Item; f push".into(),
        "cmd /c \"git push\"".into(),
        "Get-Command Get-FileHash".into(),
        "Get-Variable ExecutionContext -ValueOnly | Select-Object -ExpandProperty InvokeCommand | ForEach-Object InvokeScript 'git push'".into(),
        r#"Write-Output '{"Path":"C:\\Temp\\f.ps1","Destination":"function:f"}' | ConvertFrom-Json | Copy-Item; f"#.into(),
        "Set-Item env:GIT_CONFIG_PARAMETERS \"'core.fsmonitor=git push'\"; git status".into(),
        "pnpm.cmd run 'a\" & git push & \"b'".into(),
        "Remove-Item alias:sc; sc start zz".into(),
        // #2661 adversarial review, round 2.
        format!(r"New-Item -Path {documents} -Name 'WindowsPowerShell\Modules\Z9\Z9.psm1' -Value 'function Select-String {{ git push }}' -Force"),
        format!(
            "Set-Content {} 'function Select-String {{ git push }}'",
            format!(r"{modules}\Z9\Z9.psm1").replace('\\', "/")
        ),
        r"Set-Content -Path 'Windows[P]owerShell\x.ps1' -Value 'git push'".into(),
        format!(r"Set-Location {documents}; Set-Content WindowsPowerShell\x.ps1 'git push'"),
        format!(r"Rename-Item -Path x.txt -NewName ..\{}", "y.psm1"),
        r"New-Item -ItemType j -Path link -Value C:\Windows".into(),
        r"New-Item -ItemType sym -Path link -Value C:\Windows".into(),
        r"New-Item -ItemType h -Path link -Value C:\Windows\win.ini".into(),
        r"New-Item -ItemType Junction -Path link -Value C:\Windows".into(),
        "Z9\\Select-String -Path README.md -Pattern x".into(),
        format!(r"'git push' | Set-Content -Encoding ascii {documents}\WindowsPowerShell\profile.ps1"),
        format!(r"Set-Content -NoNewline {documents}\WindowsPowerShell\profile.ps1 x"),
        "Set-Content -Path out.txt -Stream s -Value x".into(),
        // Variables, prompts, unknown and ambiguous parameters.
        "Get-ChildItem -OutVariable PSModuleAutoLoadingPreference".into(),
        "Get-ChildItem -ov x".into(),
        "Get-ChildItem -ErrorAction Inquire".into(),
        "Get-ChildItem -ea 3".into(),
        "Get-ChildItem -Debug".into(),
        "Remove-Item -Confirm out.txt".into(),
        "Get-ChildItem -Bogus".into(),
        // Not on the allowlist, or not a known program.
        "Get-ScheduledTask".into(),
        "Get-Foo".into(),
        r".\build.ps1".into(),
        "D:".into(),
        // Native argv PowerShell may not pass as written.
        "git commit -m ''".into(),
        "git log '--format=\"%H\"'".into(),
        "git -c:core.pager=x status".into(),
        "git log 'a b\\'".into(),
        // Paths from pipeline input.
        "Get-ChildItem | Remove-Item".into(),
        "Get-ChildItem | Copy-Item -Destination out".into(),
        // An archive's entries can name paths outside its destination.
        "Expand-Archive -Path a.zip -DestinationPath out".into(),
        // #2709 review r1: names PowerShell would load from a module (exported
        // aliases, `Get-<name>`), variable-setting web parameters, trees copied
        // over `.git`, removing a folder above it, files cmd.exe runs from the
        // current folder, and a stream bound from pipeline input.
        "inmo EvilMod -Scope CurrentUser -Force; Get-CimInstance Win32_OperatingSystem".into(),
        "content README.md".into(),
        "Invoke-RestMethod -Uri https://example.com -StatusCodeVariable ErrorActionPreference".into(),
        "Invoke-WebRequest -UseBasicParsing -Uri https://example.com -SessionVariable s".into(),
        r"New-Item -ItemType Directory -Force -Path s\.git\hooks | Out-Null; Copy-Item -Path s\.git -Destination . -Recurse -Force".into(),
        r"Move-Item -Path s\.git -Destination .".into(),
        "Remove-Item -Recurse -Force .".into(),
        "Set-Content -Path node.cmd -Value '@git push'; pnpm --version".into(),
        r"Copy-Item C:\Windows\System32\cmd.exe x.exe; .\x.exe /c git push".into(),
        r#"ConvertFrom-Json '{"Stream":"x"}' | Set-Content -Path a.txt"#.into(),
        // D729 (A) after #2709 review r3: no cmdlet that writes, copies, moves,
        // renames, removes or downloads files is on the allowlist.
        "Set-Content -Path out.txt -Value x".into(),
        "Add-Content -Path out.txt -Value x".into(),
        "Out-File -FilePath log.txt -InputObject x".into(),
        "New-Item -ItemType Directory -Force -Path build".into(),
        "mkdir build".into(),
        "Copy-Item -Path a.txt -Destination b.txt".into(),
        "Move-Item -Path a.txt -Destination b.txt".into(),
        "Rename-Item -Path a.txt -NewName b.txt".into(),
        "Remove-Item -Path a.txt".into(),
        "Write-Output x | Tee-Object -FilePath log.txt".into(),
        "Get-Process | Export-Csv -Path p.csv".into(),
        "Compress-Archive -Path a.txt -DestinationPath a.zip".into(),
        "Invoke-WebRequest -UseBasicParsing -Uri https://example.com/x -OutFile x.html".into(),
        // #2709 review r2: a source named `.` (`s\.git\.` is `s\.git`), a copy
        // of a folder's contents, a download into a folder the server names.
        r"Copy-Item -Path s\.git\. -Destination . -Recurse -Force".into(),
        r"Move-Item -Path s\.git\. -Destination .".into(),
        r"Copy-Item -Path s\.. -Destination x -Recurse".into(),
        "Copy-Item -Path s -Destination . -Recurse -Force -Container:0".into(),
        "Invoke-WebRequest -UseBasicParsing -Uri https://example.com/x -OutFile .".into(),
        "Invoke-RestMethod -Uri https://example.com/x -OutFile out".into(),
    ]
}

/// A workspace folder inside a temporary folder that is not itself writable.
fn workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let parent = tempfile::tempdir().expect("tempdir");
    let workspace = parent.path().join("ws");
    std::fs::create_dir(&workspace).expect("workspace");
    (parent, workspace)
}

#[tokio::test]
async fn unread_powershell_forms_ask_and_refuse_with_approvals_off() {
    let (_parent, workspace) = workspace();
    for shell in shells() {
        let mut scripts = unread_scripts();
        if shell == "powershell.exe" {
            // Windows PowerShell parses HTML with Internet Explorer's engine.
            scripts.push("Invoke-WebRequest http://127.0.0.1:9/".into());
        }
        for script in scripts {
            for permission_profile in [
                PermissionProfile::Disabled,
                PermissionProfile::workspace_write(),
            ] {
                let asked = decide_in(
                    shell,
                    &script,
                    &workspace,
                    AskForApproval::OnRequest,
                    permission_profile.clone(),
                )
                .await;
                assert!(
                    matches!(
                        asked,
                        ExecApprovalRequirement::NeedsApproval {
                            proposed_execpolicy_amendment: None,
                            ..
                        } | ExecApprovalRequirement::Forbidden { .. }
                    ),
                    "{shell} {script} {permission_profile:?}: {asked:?}"
                );
                let refused = decide_in(
                    shell,
                    &script,
                    &workspace,
                    AskForApproval::Never,
                    permission_profile,
                )
                .await;
                assert!(
                    matches!(refused, ExecApprovalRequirement::Forbidden { .. }),
                    "{shell} {script}: {refused:?}"
                );
            }
        }
    }
}

#[tokio::test]
async fn allowlisted_forms_stay_precise() {
    let (_parent, workspace) = workspace();
    for shell in shells() {
        for script in [
            "Get-Content README.md",
            "gc README.md -Raw",
            "Get-Content -Encoding utf8 -Path README.md -TotalCount 5",
            "Get-ChildItem -Recurse -File | Select-Object -First 5",
            "git status; git log -1 --oneline",
            "Select-String -Path README.md -Pattern x",
            "Microsoft.PowerShell.Management\\Get-ChildItem -Force",
            "Get-ChildItem -ErrorAction SilentlyContinue",
            "Invoke-RestMethod -Uri https://example.com/x.json",
            "Invoke-WebRequest -UseBasicParsing -Uri https://example.com/x",
            r"Set-Location src; Get-ChildItem",
            // File-system dynamic parameters, also for a path that does not exist.
            r"Get-ChildItem 'C:\does not\exist' -Directory -ea 0 | Select-Object -ExpandProperty Name",
            r"Get-Content -LiteralPath 'C:\does not\exist.txt' -Raw",
        ] {
            for permission_profile in [
                PermissionProfile::Disabled,
                PermissionProfile::workspace_write(),
            ] {
                let result = decide_in(
                    shell,
                    script,
                    &workspace,
                    AskForApproval::OnRequest,
                    permission_profile.clone(),
                )
                .await;
                assert!(
                    matches!(result, ExecApprovalRequirement::Skip { .. }),
                    "{shell} {script} {permission_profile:?}: {result:?}"
                );
            }
        }
        // A forbidden command is still refused, not only asked about.
        for script in ["git push", "Get-Content README.md; git push origin main"] {
            let result = decide_in(
                shell,
                script,
                &workspace,
                AskForApproval::OnRequest,
                PermissionProfile::Disabled,
            )
            .await;
            assert!(
                matches!(&result, ExecApprovalRequirement::Forbidden { reason } if reason.contains("git push")),
                "{shell} {script}: {result:?}"
            );
        }
    }
}
