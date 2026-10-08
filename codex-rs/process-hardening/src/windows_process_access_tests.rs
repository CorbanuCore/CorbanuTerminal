//! PF-27-S06 containment probes, measured on real Windows processes.
//!
//! Every test re-executes this test binary twice, filtered to the entry test
//! below: once as the *target* (stands in for Core or the credential broker;
//! it carries a synthetic canary in its environment block and starts a thread
//! after hardening) and once as the *probe* (stands in for an agent command).
//! The probe runs either under the restricted token the unelevated Windows
//! sandbox gives agent commands, or as a plain same-user process, and reports
//! what it could open and read. Unhardened targets are the positive controls.

use super::restrict_current_process_access;
use codex_windows_sandbox::ConsoleMode;
use codex_windows_sandbox::StderrMode;
use codex_windows_sandbox::StdinMode;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::c_void;
use std::io::BufRead as _;
use std::io::Read as _;
use std::io::Write as _;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
use windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPTHREAD;
use windows_sys::Win32::System::Diagnostics::ToolHelp::THREADENTRY32;
use windows_sys::Win32::System::Diagnostics::ToolHelp::Thread32First;
use windows_sys::Win32::System::Diagnostics::ToolHelp::Thread32Next;
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::OpenThread;
use windows_sys::Win32::System::Threading::PROCESS_BASIC_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_CREATE_THREAD;
use windows_sys::Win32::System::Threading::PROCESS_DUP_HANDLE;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_SET_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_SUSPEND_RESUME;
use windows_sys::Win32::System::Threading::PROCESS_VM_OPERATION;
use windows_sys::Win32::System::Threading::PROCESS_VM_READ;
use windows_sys::Win32::System::Threading::PROCESS_VM_WRITE;
use windows_sys::Win32::System::Threading::THREAD_GET_CONTEXT;
use windows_sys::Win32::System::Threading::THREAD_SET_CONTEXT;
use windows_sys::Win32::System::Threading::THREAD_SUSPEND_RESUME;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const ROLE_ENV: &str = "CODEX_PF27S06_ROLE";
const TARGET_PID_ENV: &str = "CODEX_PF27S06_TARGET_PID";
const HARDEN_ENV: &str = "CODEX_PF27S06_HARDEN";
const CANARY_ENV: &str = "CODEX_PF27S06_CANARY";
/// Synthetic value; only its presence in the target's memory is checked.
const CANARY: &str = "pf27s06-synthetic-env-canary-7c41d9";
const CHILD_TEST: &str = "windows_process_access::tests::pf_27_s06_child_entry";
const REPORT_PREFIX: &str = "pf27s06-probe:";
const READY: &str = "pf27s06-target:ready";
const WRITE_DAC: u32 = 0x0004_0000;
const WRITE_OWNER: u32 = 0x0008_0000;
const ERROR_ACCESS_DENIED: u32 = 5;
const WAIT_TIMEOUT: u32 = 0x102;
const CHILD_TIMEOUT: Duration = Duration::from_secs(60);

/// Rights that would let a probe read, inject into or re-ACL the target.
const PROCESS_RIGHTS: &[(&str, u32)] = &[
    ("dup_handle", PROCESS_DUP_HANDLE),
    ("write_dac", WRITE_DAC),
    ("write_owner", WRITE_OWNER),
    ("vm_write", PROCESS_VM_WRITE),
    ("vm_operation", PROCESS_VM_OPERATION),
    ("create_thread", PROCESS_CREATE_THREAD),
    ("suspend_resume", PROCESS_SUSPEND_RESUME),
    ("set_information", PROCESS_SET_INFORMATION),
];
const THREAD_RIGHTS: &[(&str, u32)] = &[
    ("thread_get_context", THREAD_GET_CONTEXT),
    ("thread_set_context", THREAD_SET_CONTEXT),
    ("thread_suspend", THREAD_SUSPEND_RESUME),
];

type Report = BTreeMap<String, String>;

#[test]
fn pf_27_s06_child_entry() {
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("target") => run_target(),
        Ok("probe") => run_probe(),
        _ => {}
    }
}

/// Positive control: without hardening, a command under the sandbox's
/// restricted token reads the target's memory and finds the canary in its
/// environment (the token restricts writes only).
#[test]
fn pf_27_s06_restricted_token_probe_reads_an_unhardened_process() {
    let target = Target::spawn(/*harden*/ false);
    let report = probe_with_restricted_token(target.pid());
    assert_eq!(report["vm_read"], "granted", "{report:?}");
    assert_eq!(report["environment"], "canary_found", "{report:?}");
}

/// Positive control: a plain same-user process gets every right on an
/// unhardened target, threads included.
#[test]
fn pf_27_s06_same_user_probe_controls_an_unhardened_process() {
    let target = Target::spawn(/*harden*/ false);
    let report = probe_as_same_user(target.pid());
    assert_eq!(report.len(), 2 + PROCESS_RIGHTS.len() + THREAD_RIGHTS.len());
    for (key, value) in &report {
        let expected = if key == "environment" {
            "canary_found"
        } else {
            "granted"
        };
        assert_eq!(value.split('@').next(), Some(expected), "{key}: {report:?}");
    }
}

/// A command under the sandbox's restricted token cannot open a hardened
/// process or any of its threads (including one started after hardening).
#[test]
fn pf_27_s06_restricted_token_probe_cannot_read_a_hardened_process() {
    let target = Target::spawn(/*harden*/ true);
    assert_all_denied(&probe_with_restricted_token(target.pid()));
}

/// The same holds for an unsandboxed process of the same user (for example
/// an MCP server or hook, which run outside the sandbox) that has no enabled
/// privileges. (An administrator with `SeDebugPrivilege` enabled, as on the
/// CI runner, bypasses any DACL by design; the probe disables it first.)
#[test]
fn pf_27_s06_same_user_probe_cannot_read_a_hardened_process() {
    let target = Target::spawn(/*harden*/ true);
    assert_all_denied(&probe_as_same_user(target.pid()));
}

fn assert_all_denied(report: &Report) {
    assert_eq!(report.len(), 2 + PROCESS_RIGHTS.len() + THREAD_RIGHTS.len());
    for (key, value) in report {
        let expected = if key == "environment" {
            "unreadable"
        } else {
            "denied"
        };
        assert_eq!(value, expected, "{key}: {report:?}");
    }
}

/// The target process; killed on drop.
struct Target {
    child: Child,
}

impl Target {
    fn spawn(harden: bool) -> Self {
        let mut command = child_command("target");
        command
            .env(CANARY_ENV, CANARY)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if harden {
            command.env(HARDEN_ENV, "1");
        }
        let mut child = command.spawn().expect("spawn target");
        let lines = line_channel(child.stdout.take().expect("target stdout"));
        let output = read_until(&lines, READY);
        if !output.contains(READY) {
            let _ = child.kill();
            panic!("target did not become ready; it printed: {output}");
        }
        Self { child }
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn child_command(role: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test binary"));
    command
        .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, role)
        .env_remove(CANARY_ENV)
        .env_remove(HARDEN_ENV);
    command
}

fn run_target() {
    if std::env::var_os(HARDEN_ENV).is_some() {
        restrict_current_process_access().expect("restrict process access");
    }
    // A thread started after hardening must be protected too.
    let (started, wait) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = started.send(());
        std::thread::park();
    });
    let _ = wait.recv();
    assert_eq!(
        super::thread_protection_failures(),
        0,
        "a new thread stayed unprotected"
    );
    let mut stdout = std::io::stdout();
    // Own line: libtest prints the test name without a newline first.
    writeln!(stdout, "\n{READY}").expect("write ready");
    stdout.flush().expect("flush ready");
    // Stay alive until the test kills us or closes stdin.
    let mut sink = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut sink);
    std::process::exit(0);
}

fn probe_as_same_user(target_pid: u32) -> Report {
    let mut child = child_command("probe")
        .env(TARGET_PID_ENV, target_pid.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("run probe");
    // Read the report line rather than waiting for EOF, which a process
    // spawned concurrently elsewhere could delay by inheriting the pipe.
    let lines = line_channel(child.stdout.take().expect("probe stdout"));
    let output = read_until(&lines, REPORT_PREFIX);
    let _ = child.kill();
    let _ = child.wait();
    decode(&output)
}

/// Lines from a child's stdout, read on a detached thread.
fn line_channel<R: std::io::Read + Send + 'static>(stdout: R) -> std::sync::mpsc::Receiver<String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}

/// Collects lines until one contains `marker`, or the child timeout passes.
fn read_until(lines: &std::sync::mpsc::Receiver<String>, marker: &str) -> String {
    let deadline = Instant::now() + CHILD_TIMEOUT;
    let mut output = String::new();
    while !output.contains(marker) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match lines.recv_timeout(remaining) {
            Ok(line) => {
                output.push_str(&line);
                output.push('\n');
            }
            Err(_) => break,
        }
    }
    output
}

/// Runs the probe under the same restricted token the unelevated Windows
/// sandbox creates for agent commands.
fn probe_with_restricted_token(target_pid: u32) -> Report {
    let exe = std::env::current_exe().expect("test binary");
    let argv = vec![
        exe.to_string_lossy().into_owned(),
        CHILD_TEST.to_string(),
        "--exact".to_string(),
        "--nocapture".to_string(),
        "--test-threads=1".to_string(),
    ];
    let mut env: HashMap<String, String> = std::env::vars()
        .filter(|(key, _)| key != CANARY_ENV && key != HARDEN_ENV)
        .collect();
    env.insert(ROLE_ENV.to_string(), "probe".to_string());
    env.insert(TARGET_PID_ENV.to_string(), target_pid.to_string());
    let cwd = std::env::current_dir().expect("cwd");
    let capability = codex_windows_sandbox::LocalSid::from_string(
        "S-1-5-21-2718281828-3141592653-1618033988-1001",
    )
    .expect("capability SID");
    // SAFETY: the base token and the restricted token are closed below.
    let token = unsafe {
        let base =
            codex_windows_sandbox::get_current_token_for_restriction().expect("current token");
        let restricted =
            codex_windows_sandbox::create_readonly_token_with_cap_from(base, capability.as_ptr());
        CloseHandle(base);
        restricted.expect("restricted token").0
    };
    let spawned = codex_windows_sandbox::spawn_process_with_pipes(
        token,
        &argv,
        &cwd,
        &env,
        StdinMode::Closed,
        StderrMode::MergeStdout,
        ConsoleMode::NoWindow,
        /*use_private_desktop*/ false,
        /*logs_base_dir*/ None,
    )
    .expect("spawn restricted probe");
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    // Not joined: EOF can come late (see `probe_as_same_user`).
    let _reader = codex_windows_sandbox::read_handle_loop(spawned.stdout_read, move |chunk| {
        let _ = sender.send(chunk.to_vec());
    });
    // SAFETY: the handles stay valid until closed below.
    unsafe {
        if WaitForSingleObject(spawned.process.hProcess, CHILD_TIMEOUT.as_millis() as u32)
            == WAIT_TIMEOUT
        {
            TerminateProcess(spawned.process.hProcess, 1);
        }
        CloseHandle(spawned.process.hThread);
        CloseHandle(spawned.process.hProcess);
        CloseHandle(token);
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = Vec::new();
    loop {
        let text = String::from_utf8_lossy(&output);
        let complete = text
            .find(REPORT_PREFIX)
            .is_some_and(|start| text[start..].contains('\n'));
        let remaining = deadline.saturating_duration_since(Instant::now());
        if complete || remaining.is_zero() {
            break;
        }
        match receiver.recv_timeout(remaining) {
            Ok(chunk) => output.extend(chunk),
            Err(_) => break,
        }
    }
    decode(&String::from_utf8_lossy(&output))
}

fn run_probe() {
    let pid: u32 = std::env::var(TARGET_PID_ENV)
        .expect("target pid")
        .parse()
        .expect("numeric pid");
    disable_all_privileges();
    let mut report = Report::new();
    match open_process(pid, PROCESS_VM_READ | PROCESS_QUERY_INFORMATION) {
        Ok(handle) => {
            let environment = match read_environment_block(handle) {
                Some(block) if contains_utf16(&block, CANARY) => "canary_found",
                Some(_) => "canary_absent",
                None => "unreadable",
            };
            // SAFETY: opened above.
            unsafe { CloseHandle(handle) };
            report.insert("vm_read".to_string(), "granted".to_string());
            report.insert("environment".to_string(), environment.to_string());
        }
        Err(code) => {
            report.insert("vm_read".to_string(), access_from_error(code));
            report.insert("environment".to_string(), "unreadable".to_string());
        }
    }
    for (name, access) in PROCESS_RIGHTS {
        let outcome = match open_process(pid, *access) {
            Ok(handle) => {
                // SAFETY: opened above.
                unsafe { CloseHandle(handle) };
                "granted".to_string()
            }
            Err(code) => access_from_error(code),
        };
        report.insert((*name).to_string(), outcome);
    }
    let threads = thread_ids(pid);
    for (name, access) in THREAD_RIGHTS {
        report.insert((*name).to_string(), open_threads(&threads, *access));
    }
    let encoded: Vec<String> = report.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let mut stdout = std::io::stdout();
    // Own line: libtest prints the test name without a newline first.
    let _ = writeln!(stdout, "\n{REPORT_PREFIX}{}", encoded.join(","));
    let _ = stdout.flush();
    std::process::exit(0);
}

/// `granted` if any thread opens with `access`, `denied` if every thread
/// refuses with access denied, otherwise the first other failure.
fn open_threads(threads: &[u32], access: u32) -> String {
    if threads.is_empty() {
        return "no_threads".to_string();
    }
    let mut outcome = "denied".to_string();
    for tid in threads {
        // SAFETY: plain OpenThread; the handle is closed at once.
        let handle = unsafe { OpenThread(access, 0, *tid) };
        if handle != 0 {
            // SAFETY: opened above.
            unsafe { CloseHandle(handle) };
            return format!("granted@{tid}");
        }
        // SAFETY: reads the thread's last error.
        let code = unsafe { GetLastError() };
        if code != ERROR_ACCESS_DENIED && outcome == "denied" {
            outcome = format!("failed:{code}");
        }
    }
    outcome
}

fn thread_ids(pid: u32) -> Vec<u32> {
    let mut ids = Vec::new();
    // SAFETY: a thread snapshot; closed below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return ids;
    }
    // SAFETY: zeroed POD with its size set.
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    // SAFETY: valid snapshot and entry.
    let mut more = unsafe { Thread32First(snapshot, &mut entry) } != 0;
    while more {
        if entry.th32OwnerProcessID == pid {
            ids.push(entry.th32ThreadID);
        }
        // SAFETY: as above.
        more = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
    }
    // SAFETY: opened above.
    unsafe { CloseHandle(snapshot) };
    ids
}

/// Disables every privilege in this process's token, as in an ordinary user
/// process (CI runners run elevated with `SeDebugPrivilege` enabled, which
/// opens any process regardless of its DACL).
fn disable_all_privileges() {
    use windows_sys::Win32::Security::AdjustTokenPrivileges;
    use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    let mut token: HANDLE = 0;
    // SAFETY: opens this process's token; closed below.
    let opened =
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token) };
    assert_ne!(opened, 0, "OpenProcessToken failed");
    // SAFETY: DisableAllPrivileges ignores the new-state arguments.
    let adjusted = unsafe {
        AdjustTokenPrivileges(
            token,
            /*DisableAllPrivileges*/ 1,
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    assert_ne!(adjusted, 0, "AdjustTokenPrivileges failed");
}

fn access_from_error(code: u32) -> String {
    if code == ERROR_ACCESS_DENIED {
        "denied".to_string()
    } else {
        format!("failed:{code}")
    }
}

fn open_process(pid: u32, access: u32) -> Result<HANDLE, u32> {
    // SAFETY: plain OpenProcess; the handle is closed by the caller.
    let handle = unsafe { OpenProcess(access, 0, pid) };
    if handle == 0 {
        // SAFETY: reads the thread's last error.
        Err(unsafe { GetLastError() })
    } else {
        Ok(handle)
    }
}

fn decode(output: &str) -> Report {
    let line = output
        .lines()
        .find_map(|line| line.find(REPORT_PREFIX).map(|start| &line[start..]))
        .and_then(|line| line.strip_prefix(REPORT_PREFIX))
        .unwrap_or_else(|| panic!("probe printed no report:\n{output}"));
    line.trim()
        .split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryInformationProcess(
        process: HANDLE,
        class: u32,
        information: *mut c_void,
        length: u32,
        return_length: *mut u32,
    ) -> i32;
}

/// Reads the target's environment block through its PEB, as a debugger or
/// `ps`-style tool would (64-bit layout).
#[cfg(target_pointer_width = "64")]
fn read_environment_block(process: HANDLE) -> Option<Vec<u8>> {
    const PEB_PROCESS_PARAMETERS: usize = 0x20;
    const PARAMS_ENVIRONMENT: usize = 0x80;
    const PARAMS_ENVIRONMENT_SIZE: usize = 0x3F0;
    const MAX_ENVIRONMENT_BYTES: usize = 1 << 20;
    // SAFETY: zeroed POD out-structure.
    let mut info: PROCESS_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let mut returned = 0_u32;
    // SAFETY: `info` is valid for the size passed.
    let status = unsafe {
        NtQueryInformationProcess(
            process,
            /*class*/ 0,
            (&mut info as *mut PROCESS_BASIC_INFORMATION).cast(),
            std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            &mut returned,
        )
    };
    if status < 0 || info.PebBaseAddress.is_null() {
        return None;
    }
    let peb = info.PebBaseAddress as usize;
    let params = read_usize(process, peb + PEB_PROCESS_PARAMETERS)?;
    let environment = read_usize(process, params + PARAMS_ENVIRONMENT)?;
    let size =
        read_usize(process, params + PARAMS_ENVIRONMENT_SIZE)?.clamp(1, MAX_ENVIRONMENT_BYTES);
    read_bytes(process, environment, size)
}

#[cfg(not(target_pointer_width = "64"))]
fn read_environment_block(_process: HANDLE) -> Option<Vec<u8>> {
    None
}

fn read_usize(process: HANDLE, address: usize) -> Option<usize> {
    let bytes = read_bytes(process, address, std::mem::size_of::<usize>())?;
    Some(usize::from_ne_bytes(bytes.try_into().ok()?))
}

fn read_bytes(process: HANDLE, address: usize, len: usize) -> Option<Vec<u8>> {
    let mut buffer = vec![0_u8; len];
    let mut read = 0_usize;
    // SAFETY: `buffer` is valid for `len` bytes.
    let ok = unsafe {
        ReadProcessMemory(
            process,
            address as *const c_void,
            buffer.as_mut_ptr().cast(),
            len,
            &mut read,
        )
    };
    (ok != 0 && read > 0).then(|| {
        buffer.truncate(read);
        buffer
    })
}

fn contains_utf16(haystack: &[u8], needle: &str) -> bool {
    let needle: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[test]
fn pf_27_s06_dacls_limit_user_and_owner_rights() {
    assert_eq!(
        super::process_dacl_sddl("S-1-5-21-1-2-3-1001"),
        "D:P(A;;0x101000;;;S-1-5-21-1-2-3-1001)(A;;GA;;;SY)(A;;RC;;;OW)"
    );
    assert_eq!(
        super::thread_dacl_sddl("S-1-5-21-1-2-3-1001"),
        "D:P(A;;0x100800;;;S-1-5-21-1-2-3-1001)(A;;GA;;;SY)(A;;RC;;;OW)"
    );
}

/// The TLS callback that protects new threads is linked into the image.
#[test]
fn pf_27_s06_thread_callback_is_registered() {
    assert!(super::thread_callback_registered());
}

#[test]
fn pf_27_s06_probe_reports_are_found_after_libtest_output() {
    let report =
        decode("running 1 test\ntest x ... pf27s06-probe:vm_read=denied,write_dac=failed:87\n");
    assert_eq!(report["vm_read"], "denied");
    assert_eq!(report["write_dac"], "failed:87");
}
