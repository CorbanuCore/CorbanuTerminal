//! #295 (PF-27-S06 gate, defect 1): with Core's process hardened, starting
//! the elevated sandbox's runner as its user failed with
//! `CreateProcessWithLogonW failed: 5` from a normal (medium-integrity)
//! session, so every agent command failed. An elevated caller is not affected.
//!
//! Needs a local user to start the process as: set `CODEX_PF27S06_LOGON_USER`
//! and `CODEX_PF27S06_LOGON_PASSWORD_FILE` (a file holding its password).
//! Without them the test is skipped, unless `CODEX_PF27S06_REQUIRE_LOGON=1`.
//! With `CODEX_PF27S06_EXPECT_MEDIUM=1` it also fails when not run at medium
//! integrity, so CI can prove it covered the non-elevated path. With
//! `CODEX_PF27S06_DROP_TO_MEDIUM=1` an elevated run reruns the test with the
//! token UAC gives an administrator's normal session (Administrators
//! deny-only, no privileges, medium integrity), as CI's runners are elevated.
#![cfg(windows)]

use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
use std::ptr;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Foundation::PSID;
use windows_sys::Win32::Foundation::SetHandleInformation;
use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;
use windows_sys::Win32::Security::CreateRestrictedToken;
use windows_sys::Win32::Security::DISABLE_MAX_PRIVILEGE;
use windows_sys::Win32::Security::GetLengthSid;
use windows_sys::Win32::Security::GetSidSubAuthority;
use windows_sys::Win32::Security::GetSidSubAuthorityCount;
use windows_sys::Win32::Security::GetTokenInformation;
use windows_sys::Win32::Security::SID_AND_ATTRIBUTES;
use windows_sys::Win32::Security::SetTokenInformation;
use windows_sys::Win32::Security::TOKEN_ADJUST_DEFAULT;
use windows_sys::Win32::Security::TOKEN_ASSIGN_PRIMARY;
use windows_sys::Win32::Security::TOKEN_DUPLICATE;
use windows_sys::Win32::Security::TOKEN_MANDATORY_LABEL;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Security::TokenIntegrityLevel;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::System::Threading::CreateProcessAsUserW;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTF_USESTDHANDLES;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// `SECURITY_MANDATORY_MEDIUM_RID`.
const MEDIUM_INTEGRITY: u32 = 0x2000;
/// `SE_GROUP_INTEGRITY`.
const SE_GROUP_INTEGRITY: u32 = 0x20;
const EXIT_CODE: u32 = 7;
const TEST_NAME: &str = "pf_27_s06_d1_hardened_process_starts_a_process_as_another_user";
const CHILD_ENV: &str = "CODEX_PF27S06_D1_CHILD";

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
    if integrity > MEDIUM_INTEGRITY
        && std::env::var_os("CODEX_PF27S06_DROP_TO_MEDIUM").is_some()
        && std::env::var_os(CHILD_ENV).is_none()
    {
        rerun_at_medium_integrity();
        return;
    }
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

/// Reruns this test with an administrator's normal-session token (see the
/// module docs) and requires it to pass.
fn rerun_at_medium_integrity() {
    let exe = std::env::current_exe().expect("test binary");
    let log_path = std::env::temp_dir().join(format!("pf27s06-d1-{}.log", std::process::id()));
    let log = std::fs::File::create(&log_path).expect("child log");
    // SAFETY: the test runs alone in its binary.
    unsafe { std::env::set_var(CHILD_ENV, "1") };
    let to_wide = |text: &str| text.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let mut command_line = to_wide(&format!(
        "\"{}\" {TEST_NAME} --exact --nocapture --test-threads=1",
        exe.display()
    ));
    // SAFETY: plain Win32 calls on handles and buffers owned here; every
    // handle and SID is released below.
    let exit_code = unsafe {
        let mut token = 0;
        assert_ne!(
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY | TOKEN_ADJUST_DEFAULT,
                &mut token,
            ),
            0
        );
        let mut admins: PSID = ptr::null_mut();
        assert_ne!(
            ConvertStringSidToSidW(to_wide("S-1-5-32-544").as_ptr(), &mut admins),
            0
        );
        let deny_only = SID_AND_ATTRIBUTES {
            Sid: admins,
            Attributes: 0,
        };
        let mut restricted = 0;
        assert_ne!(
            CreateRestrictedToken(
                token,
                DISABLE_MAX_PRIVILEGE,
                1,
                &deny_only,
                0,
                ptr::null(),
                0,
                ptr::null(),
                &mut restricted,
            ),
            0,
            "CreateRestrictedToken"
        );
        let mut medium: PSID = ptr::null_mut();
        assert_ne!(
            ConvertStringSidToSidW(to_wide("S-1-16-8192").as_ptr(), &mut medium),
            0
        );
        let label = TOKEN_MANDATORY_LABEL {
            Label: SID_AND_ATTRIBUTES {
                Sid: medium,
                Attributes: SE_GROUP_INTEGRITY,
            },
        };
        assert_ne!(
            SetTokenInformation(
                restricted,
                TokenIntegrityLevel,
                ptr::from_ref(&label).cast(),
                std::mem::size_of::<TOKEN_MANDATORY_LABEL>() as u32 + GetLengthSid(medium),
            ),
            0,
            "lower the integrity level"
        );
        let log_handle = log.as_raw_handle() as isize;
        assert_ne!(
            SetHandleInformation(log_handle, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT),
            0
        );
        let mut startup: STARTUPINFOW = std::mem::zeroed();
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        startup.dwFlags = STARTF_USESTDHANDLES;
        startup.hStdOutput = log_handle;
        startup.hStdError = log_handle;
        let mut info: PROCESS_INFORMATION = std::mem::zeroed();
        assert_ne!(
            CreateProcessAsUserW(
                restricted,
                ptr::null(),
                command_line.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                1,
                CREATE_NO_WINDOW,
                ptr::null(),
                ptr::null(),
                &startup,
                &mut info,
            ),
            0,
            "start the medium-integrity rerun: {}",
            std::io::Error::last_os_error()
        );
        WaitForSingleObject(info.hProcess, 300_000);
        let mut code = 1;
        GetExitCodeProcess(info.hProcess, &mut code);
        CloseHandle(info.hThread);
        CloseHandle(info.hProcess);
        CloseHandle(restricted);
        CloseHandle(token);
        LocalFree(admins as _);
        LocalFree(medium as _);
        code
    };
    drop(log);
    let output = std::fs::read_to_string(&log_path).unwrap_or_default();
    let _ = std::fs::remove_file(&log_path);
    eprintln!("pf27s06 d1: medium-integrity rerun:\n{output}");
    assert_eq!(exit_code, 0, "medium-integrity rerun failed");
    assert!(
        output.contains("pf27s06 d1: integrity level 0x2000"),
        "the rerun did not run at medium integrity"
    );
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
