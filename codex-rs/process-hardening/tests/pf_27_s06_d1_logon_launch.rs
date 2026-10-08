//! #295 (PF-27-S06 gate, defect 1): with Core's process hardened, starting
//! the elevated sandbox's runner as its user failed with
//! `CreateProcessWithLogonW failed: 5` from a normal (medium-integrity)
//! session, so every agent command failed. An elevated caller is not affected.
//!
//! Needs a local user to start the process as: set `CODEX_PF27S06_LOGON_USER`
//! and `CODEX_PF27S06_LOGON_PASSWORD_FILE` (a file holding its password).
//! Without them the test is skipped, unless `CODEX_PF27S06_REQUIRE_LOGON=1`.
//! With `CODEX_PF27S06_EXPECT_MEDIUM=1` it also fails when not run at medium
//! integrity, so CI can prove it covered the non-elevated path (CI runs it
//! through `.github/scripts/run-at-medium-integrity.ps1`).
#![cfg(windows)]

use std::path::PathBuf;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Security::GetSidSubAuthority;
use windows_sys::Win32::Security::GetSidSubAuthorityCount;
use windows_sys::Win32::Security::GetTokenInformation;
use windows_sys::Win32::Security::TOKEN_MANDATORY_LABEL;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Security::TokenIntegrityLevel;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// `SECURITY_MANDATORY_MEDIUM_RID`.
const MEDIUM_INTEGRITY: u32 = 0x2000;
const EXIT_CODE: u32 = 7;

#[test]
fn pf_27_s06_d1_hardened_process_starts_a_process_as_another_user() {
    let (Some(user), Some(password_file)) = (
        std::env::var_os("CODEX_PF27S06_LOGON_USER"),
        std::env::var_os("CODEX_PF27S06_LOGON_PASSWORD_FILE"),
    ) else {
        assert!(
            std::env::var_os("CODEX_PF27S06_REQUIRE_LOGON").is_none(),
            "CODEX_PF27S06_REQUIRE_LOGON is set but the logon user is not"
        );
        eprintln!("pf27s06 d1: skipped (no CODEX_PF27S06_LOGON_USER)");
        return;
    };
    let user = user.into_string().expect("user name");
    let password = std::fs::read_to_string(password_file).expect("password file");
    let integrity = integrity_level();
    eprintln!("pf27s06 d1: integrity level {integrity:#x}");
    if std::env::var_os("CODEX_PF27S06_EXPECT_MEDIUM").is_some() {
        assert_eq!(integrity, MEDIUM_INTEGRITY, "not a medium-integrity run");
    }

    // Hardened exactly as an armed launch contract hardens Core.
    codex_process_hardening::restrict_current_process_access().expect("harden this process");

    let system_root = PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot"));
    let cmd = system_root.join("System32").join("cmd.exe");
    let launched = codex_windows_sandbox::create_process_with_logon(
        &codex_windows_sandbox::LogonLaunchRequest {
            username: &user,
            password: password.trim(),
            application: &cmd,
            command_line: &format!("cmd.exe /D /C exit {EXIT_CODE}"),
            cwd: &system_root,
        },
        &command_runner(),
    )
    .expect("start a process as the other user from a hardened process");

    // The handle Core gets back must work: wait for the process and read
    // its exit code.
    // SAFETY: `launched.process` is a live process handle owned here.
    let exit_code = unsafe {
        assert_eq!(WaitForSingleObject(launched.process, 30_000), 0, "wait");
        let mut code = 0;
        assert_ne!(GetExitCodeProcess(launched.process, &mut code), 0);
        CloseHandle(launched.process);
        code
    };
    assert_eq!(exit_code, EXIT_CODE);
}

/// The elevated sandbox's command runner from this build.
fn command_runner() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_codex_command_runner") {
        return PathBuf::from(path);
    }
    // target/<profile>/deps/<test>.exe -> target/<profile>/codex-command-runner.exe
    let exe = std::env::current_exe().expect("test binary");
    let runner = exe
        .parent()
        .and_then(|deps| deps.parent())
        .expect("target dir")
        .join("codex-command-runner.exe");
    assert!(
        runner.exists(),
        "build codex-command-runner first: {}",
        runner.display()
    );
    runner
}

fn integrity_level() -> u32 {
    let mut token = 0;
    // SAFETY: opens this process's token for query; closed below.
    assert_ne!(
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) },
        0
    );
    let mut buffer = vec![0u8; 256];
    let mut length = 0;
    // SAFETY: `buffer` is large enough for a mandatory label and its SID.
    let ok = unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
            &mut length,
        )
    };
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    assert_ne!(ok, 0, "GetTokenInformation");
    // SAFETY: the call filled `buffer` with a TOKEN_MANDATORY_LABEL whose SID
    // points into it.
    unsafe {
        let label = &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let count = *GetSidSubAuthorityCount(label.Label.Sid);
        *GetSidSubAuthority(label.Label.Sid, u32::from(count) - 1)
    }
}
