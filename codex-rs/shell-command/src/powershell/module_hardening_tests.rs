//! D731 (#2661): with the agent script prefix, a module planted in a module
//! folder cannot replace a cmdlet the script calls, and the cmdlets the
//! allowlist names still work.
use super::AGENT_SCRIPT_PREFIX;
use super::try_find_powershell_executable_blocking;
use super::try_find_pwsh_executable_blocking;

/// Runs `script` in `shell` with a module folder holding a module that
/// replaces `Select-String` first on `PSModulePath`; returns whether the
/// planted function ran and what the script printed.
fn run_with_planted_module(shell: &std::path::Path, script: &str) -> (bool, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let modules = dir.path().join("Modules");
    let module = modules.join("Z9");
    std::fs::create_dir_all(&module).expect("module folder");
    let marker = dir.path().join("planted.txt");
    std::fs::write(
        module.join("Z9.psm1"),
        format!(
            "function Select-String {{ Set-Content -LiteralPath '{}' -Value ran }}\nExport-ModuleMember -Function Select-String\n",
            marker.display()
        ),
    )
    .expect("module");
    std::fs::write(dir.path().join("input.txt"), "needle\n").expect("input");
    let module_path = std::env::join_paths(
        std::iter::once(modules.into_os_string()).chain(
            std::env::var_os("PSModulePath")
                .iter()
                .flat_map(std::env::split_paths)
                .map(Into::into),
        ),
    )
    .expect("PSModulePath");
    let output = std::process::Command::new(shell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            // Process scope only, so the planted module could load at all.
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .current_dir(dir.path())
        .env("PSModulePath", module_path)
        .output()
        .expect("run PowerShell");
    (
        marker.exists(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn planted_modules_do_not_replace_cmdlets_under_the_agent_prefix() {
    let shells: Vec<_> = [
        try_find_powershell_executable_blocking(),
        try_find_pwsh_executable_blocking(),
    ]
    .into_iter()
    .flatten()
    .collect();
    let script = "Select-String -Path input.txt -Pattern needle | ForEach-Object Line";
    for shell in shells {
        // Without the prefix the planted module replaces the cmdlet.
        let (planted, _) = run_with_planted_module(shell.as_path(), script);
        assert!(planted, "{shell:?}: the plant must work without the prefix");
        let (planted, output) =
            run_with_planted_module(shell.as_path(), &format!("{AGENT_SCRIPT_PREFIX}{script}"));
        assert!(!planted, "{shell:?}: planted module ran");
        assert_eq!(output.trim(), "needle", "{shell:?}");
    }
}

#[test]
fn allowlisted_cmdlets_work_under_the_agent_prefix() {
    let Some(shell) = try_find_powershell_executable_blocking() else {
        return;
    };
    let script = format!(
        "{AGENT_SCRIPT_PREFIX}Get-ChildItem -File | Select-Object -ExpandProperty Name | Sort-Object | Out-String | Set-Content out.txt; Get-FileHash -Path out.txt | Out-Null; Get-Acl . | Out-Null; Get-CimInstance Win32_OperatingSystem | Out-Null; Compress-Archive -Path out.txt -DestinationPath out.zip; Expand-Archive -Path out.zip -DestinationPath x; (Get-Content x\\out.txt -Raw).Trim()"
    );
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.txt"), "a").expect("a");
    let output = std::process::Command::new(shell.as_path())
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .current_dir(dir.path())
        .output()
        .expect("run PowerShell");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "a.txt",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
