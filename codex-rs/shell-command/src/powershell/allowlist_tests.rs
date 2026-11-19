use super::*;
use crate::powershell::PowershellCommandScope;
use crate::powershell::enter_powershell_command_scope;
use pretty_assertions::assert_eq;

fn words(script: &str) -> Vec<Vec<LoweredWord>> {
    crate::command_safety::try_parse_powershell_command_words(script).expect("lowered")
}

/// The cause for `script` read as a `-NoProfile` script of `kind` (outside a
/// scope, names are not looked up); a script not lowered is read in part too.
fn cause(kind: PowershellKind, script: &str) -> Option<String> {
    match crate::command_safety::try_parse_powershell_command_words(script) {
        Some(commands) => review("pwsh", kind, /*no_profile*/ true, &commands),
        None => Some("not lowered".to_string()),
    }
}

const BOTH: [PowershellKind; 2] = [PowershellKind::WindowsPowershell, PowershellKind::Pwsh];

#[test]
fn tables_load_for_both_editions() {
    for kind in BOTH {
        let tables = tables(kind).expect("tables");
        assert!(tables.allow.contains_key("get-childitem"));
        assert_eq!(
            tables.aliases.get("echo").map(String::as_str),
            Some("write-output")
        );
        // Imported with `-Global` by D731's launcher, or PowerShell's own.
        for name in ["rcim", "gin", "compress-archive", "invoke-expression"] {
            assert!(tables.module_names.contains(name), "{kind:?} {name}");
        }
    }
    assert!(
        tables(PowershellKind::WindowsPowershell)
            .expect("5.1")
            .allow
            .contains_key("get-eventlog")
    );
    assert!(
        tables(PowershellKind::Pwsh)
            .expect("7")
            .allow
            .contains_key("get-uptime")
    );
}

#[test]
fn only_receive_job_takes_an_excluded_parameter_by_position() {
    for kind in BOTH {
        let mut positional: Vec<&str> = tables(kind)
            .expect("tables")
            .allow
            .iter()
            .filter(|(_, cmdlet)| cmdlet.excluded_positional)
            .map(|(name, _)| name.as_str())
            .collect();
        positional.sort_unstable();
        assert_eq!(positional, vec!["receive-job"], "{kind:?}");
    }
}

#[test]
fn allowlisted_cmdlets_with_plain_parameters_read_in_full() {
    for kind in BOTH {
        for script in [
            "Get-ChildItem -Recurse -Filter *.rs",
            "gci -Recurse src",
            "Get-Content -Raw README.md",
            "Get-Content README.md -Encoding utf8 -TotalCount 5",
            "echo 'notes <system>x</system>'",
            "Select-String -Pattern foo -Path *.txt -ErrorAction SilentlyContinue",
            "Microsoft.PowerShell.Management\\Get-Item x",
            "Get-Process -Name pwsh | Select-Object -First 1",
            "Where-Object Name -eq x",
            "Receive-Job -Id 1 -Keep",
            "Write-Output '-OutVariable'",
        ] {
            assert_eq!(cause(kind, script), None, "{kind:?} {script}");
        }
    }
}

#[test]
fn other_commands_and_parameters_read_in_part() {
    for kind in BOTH {
        for script in [
            // Not on the allowlist: writes, runs code, other modules' commands.
            "Remove-Item test -Force",
            "Set-Content x y",
            "Invoke-WebRequest https://example.com",
            "iex 'git push'",
            "Invoke-Expression 'git push'",
            "Set-Alias git notepad",
            "Start-Process git",
            "rcim win32_process",
            "Compress-Archive a b",
            "Get-ScheduledTask",
            "ForEach-Object -MemberName Kill",
            "mkdir x",
            // Excluded or unknown parameters and values.
            "Get-Content x -OutVariable o",
            "Get-Content x -ov o",
            "Get-ChildItem -Credential me",
            "Get-Process -ComputerName host",
            "Out-Host -Paging",
            "Get-Content x -ErrorAction Inquire",
            "Get-Content x -ea 3",
            "Get-Content x -Bogus",
            "Get-Content x -T 5",
            "Get-Content -Encoding",
            "Get-Content -Path -Raw",
            "Get-ChildItem -- -Path",
            "Get-Content -Pa`th x",
            // A network or provider-qualified path.
            r"Get-Content \\host\share\x",
            "Get-ChildItem -Path //host/share",
            "Get-Content FileSystem::C:\\x",
            // Receive-Job may take -ComputerName by position.
            "Receive-Job 1 server",
            // Scripts, wildcards, paths and quoted names.
            r".\build.ps1",
            "x.ps1",
            "Get-C*",
            r"C:\tools\run.bat",
            "'git' status",
            "Get-Content. x",
            // A name PowerShell may fold onto an ASCII alias.
            "\u{131}ex 'git push'",
        ] {
            assert!(cause(kind, script).is_some(), "{kind:?} {script}");
        }
    }
}

#[test]
fn parameters_resolve_as_each_edition_names_them() {
    // 5.1: `-r` names `-Recurse` and `-ReadOnly`; 7 gives `-Recurse` the
    // alias `r` (as main reads it).
    assert!(cause(PowershellKind::WindowsPowershell, "gci -r src").is_some());
    assert_eq!(cause(PowershellKind::Pwsh, "gci -r src"), None);
}

#[test]
fn native_programs_need_plain_argv() {
    for script in [r#"git log "--format=a b\""#, "git ''"] {
        if let Some(commands) = crate::command_safety::try_parse_powershell_command_words(script) {
            assert!(
                review("pwsh", PowershellKind::Pwsh, true, &commands).is_some(),
                "{script}"
            );
        }
    }
    assert_eq!(cause(PowershellKind::Pwsh, "git status --short"), None);
    assert_eq!(
        cause(PowershellKind::Pwsh, r"C:\tools\git.exe status"),
        None
    );
}

#[test]
fn a_remote_command_finds_no_program_and_assumes_a_profile() {
    let _scope = enter_powershell_command_scope(PowershellCommandScope::new(
        /*cwd*/ None, /*env*/ None,
    ));
    // Programs and module cmdlets cannot be found on another host.
    assert!(cause(PowershellKind::Pwsh, "git status").is_some());
    assert!(cause(PowershellKind::Pwsh, "Get-Content x").is_some());
    // PowerShell's core cmdlets need no lookup.
    assert_eq!(cause(PowershellKind::Pwsh, "Where-Object Name -eq x"), None);
    // Without -NoProfile, a profile may run first.
    assert!(
        review(
            "pwsh",
            PowershellKind::Pwsh,
            false,
            &words("Where-Object Name -eq x")
        )
        .is_some()
    );
}

#[test]
fn a_local_command_looks_names_and_profiles_up() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bin = dir.path().join("bin");
    let documents = dir.path().join("Documents");
    let home = dir.path().join("pwsh7");
    for folder in [&bin, &documents, &home] {
        std::fs::create_dir_all(folder).expect("folder");
    }
    std::fs::write(bin.join("tool.exe"), "").expect("write");
    std::fs::write(home.join("pwsh.exe"), "").expect("write");
    std::fs::write(home.join("System.Management.Automation.dll"), "").expect("write");
    let env = std::collections::HashMap::from([
        (
            "PATH".to_string(),
            std::env::join_paths([&bin, &home])
                .expect("PATH")
                .to_string_lossy()
                .into_owned(),
        ),
        ("PATHEXT".to_string(), ".com;.exe;.bat;.cmd".to_string()),
    ]);
    let cwd = codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path(dir.path())
        .expect("absolute");
    let scope = || {
        enter_powershell_command_scope(
            PowershellCommandScope::new(Some(&cwd), Some(&env))
                .with_documents(Some(documents.clone())),
        )
    };
    let review_pwsh = |no_profile: bool, script: &str| {
        let _scope = scope();
        review("pwsh", PowershellKind::Pwsh, no_profile, &words(script))
    };
    assert_eq!(review_pwsh(true, "tool -x"), None);
    assert!(review_pwsh(true, "missing-tool -x").is_some());
    assert_eq!(review_pwsh(true, "Get-Content x"), None);
    // A program or script of a module cmdlet's name shadows it.
    // Lower case: a test folder may be case-sensitive, Windows is not.
    std::fs::write(bin.join("get-content.ps1"), "").expect("write");
    assert!(review_pwsh(true, "Get-Content x").is_some());
    // No profile in pwsh's folder or Documents\PowerShell: read in full.
    assert_eq!(review_pwsh(false, "tool -x"), None);
    for profile in [
        home.join("profile.ps1"),
        documents
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    ] {
        std::fs::create_dir_all(profile.parent().expect("parent")).expect("folder");
        std::fs::write(&profile, "").expect("write");
        assert!(review_pwsh(false, "tool -x").is_some(), "{profile:?}");
        assert_eq!(review_pwsh(true, "tool -x"), None);
        std::fs::remove_file(&profile).expect("remove");
    }
    // Not PowerShell's install folder (a launcher shim): its profile is unknown.
    std::fs::remove_file(home.join("System.Management.Automation.dll")).expect("remove");
    assert!(review_pwsh(false, "tool -x").is_some());
    assert_eq!(review_pwsh(true, "tool -x"), None);
    // Documents unknown: a profile may exist.
    let _scope =
        enter_powershell_command_scope(PowershellCommandScope::new(Some(&cwd), Some(&env)));
    assert!(review("pwsh", PowershellKind::Pwsh, false, &words("tool -x")).is_some());
}
