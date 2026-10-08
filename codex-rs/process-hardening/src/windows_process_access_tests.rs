//! PF-27-S06 containment probes, measured on real Windows processes.
//!
//! Every test re-executes this test binary twice, filtered to the entry test
//! below: once as the *target* (stands in for Core or the credential broker;
//! it carries a synthetic canary in its environment block) and once as the
//! *probe* (stands in for an agent command). The probe runs either under the
//! restricted token the unelevated Windows sandbox gives agent commands, or
//! as a plain same-user process, and reports what it could open and read.

use super::restrict_current_process_access;
use codex_windows_sandbox::ConsoleMode;
use codex_windows_sandbox::StderrMode;
use codex_windows_sandbox::StdinMode;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::ffi::c_void;
use std::io::BufRead as _;
use std::io::Read as _;
use std::io::Write as _;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::PROCESS_BASIC_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_DUP_HANDLE;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_VM_READ;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const ROLE_ENV: &str = "CODEX_PF27S06_ROLE";
const TARGET_PID_ENV: &str = "CODEX_PF27S06_TARGET_PID";
const HARDEN_ENV: &str = "CODEX_PF27S06_HARDEN";
const CANARY_ENV: &str = "CODEX_PF27S06_CANARY";
/// Synthetic value; only its presence in the target's memory is checked.
const CANARY: &str = "pf27s06-synthetic-env-canary-7c41d9";
const CHILD_TEST: &str = "windows_process_access::tests::pf_27_s06_child_entry";
const REPORT_PREFIX: &str = "pf27s06-probe:";
const WRITE_DAC: u32 = 0x0004_0000;
const ERROR_ACCESS_DENIED: u32 = 5;

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
    assert_eq!(report.vm_read, Access::Granted, "{report:?}");
    assert_eq!(report.environment, EnvRead::CanaryFound, "{report:?}");
}

/// A command under the sandbox's restricted token cannot open a hardened
/// process to read its memory or environment, duplicate its handles, or
/// rewrite its DACL.
#[test]
fn pf_27_s06_restricted_token_probe_cannot_read_a_hardened_process() {
    let target = Target::spawn(/*harden*/ true);
    let report = probe_with_restricted_token(target.pid());
    assert_eq!(report, ProbeReport::all_denied());
}

/// The same holds for an unsandboxed process of the same user (for example
/// an MCP server or hook, which run outside the sandbox).
#[test]
fn pf_27_s06_same_user_probe_cannot_read_a_hardened_process() {
    let target = Target::spawn(/*harden*/ true);
    let report = probe_as_same_user(target.pid());
    assert_eq!(report, ProbeReport::all_denied());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Access {
    Granted,
    Denied,
    Failed(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvRead {
    CanaryFound,
    CanaryAbsent,
    Unreadable,
}

#[derive(Debug, PartialEq, Eq)]
struct ProbeReport {
    vm_read: Access,
    environment: EnvRead,
    dup_handle: Access,
    write_dac: Access,
}

impl ProbeReport {
    fn all_denied() -> Self {
        Self {
            vm_read: Access::Denied,
            environment: EnvRead::Unreadable,
            dup_handle: Access::Denied,
            write_dac: Access::Denied,
        }
    }

    fn encode(&self) -> String {
        format!(
            "{REPORT_PREFIX}{}|{}|{}|{}",
            encode_access(self.vm_read),
            match self.environment {
                EnvRead::CanaryFound => "found".to_string(),
                EnvRead::CanaryAbsent => "absent".to_string(),
                EnvRead::Unreadable => "unreadable".to_string(),
            },
            encode_access(self.dup_handle),
            encode_access(self.write_dac),
        )
    }

    fn decode(output: &str) -> Self {
        let line = output
            .lines()
            .find_map(|line| line.trim().strip_prefix(REPORT_PREFIX))
            .unwrap_or_else(|| panic!("probe printed no report:\n{output}"));
        let fields: Vec<&str> = line.split('|').collect();
        assert_eq!(fields.len(), 4, "malformed probe report: {line}");
        Self {
            vm_read: decode_access(fields[0]),
            environment: match fields[1] {
                "found" => EnvRead::CanaryFound,
                "absent" => EnvRead::CanaryAbsent,
                _ => EnvRead::Unreadable,
            },
            dup_handle: decode_access(fields[2]),
            write_dac: decode_access(fields[3]),
        }
    }
}

fn encode_access(access: Access) -> String {
    match access {
        Access::Granted => "granted".to_string(),
        Access::Denied => "denied".to_string(),
        Access::Failed(code) => format!("failed:{code}"),
    }
}

fn decode_access(field: &str) -> Access {
    match field {
        "granted" => Access::Granted,
        "denied" => Access::Denied,
        other => Access::Failed(
            other
                .strip_prefix("failed:")
                .and_then(|code| code.parse().ok())
                .unwrap_or(u32::MAX),
        ),
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
            .stderr(Stdio::null());
        if harden {
            command.env(HARDEN_ENV, "1");
        }
        let mut child = command.spawn().expect("spawn target");
        let stdout = child.stdout.take().expect("target stdout");
        let mut lines = std::io::BufReader::new(stdout).lines();
        let ready = lines.any(|line| line.is_ok_and(|line| line.trim() == "pf27s06-target:ready"));
        assert!(ready, "target did not become ready");
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
    let mut stdout = std::io::stdout();
    writeln!(stdout, "pf27s06-target:ready").expect("write ready");
    stdout.flush().expect("flush ready");
    // Stay alive until the test kills us or closes stdin.
    let mut sink = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut sink);
    std::process::exit(0);
}

fn probe_as_same_user(target_pid: u32) -> ProbeReport {
    let output = child_command("probe")
        .env(TARGET_PID_ENV, target_pid.to_string())
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .expect("run probe");
    ProbeReport::decode(&String::from_utf8_lossy(&output.stdout))
}

/// Runs the probe under the same restricted token the unelevated Windows
/// sandbox creates for agent commands.
fn probe_with_restricted_token(target_pid: u32) -> ProbeReport {
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
    // Not joined: a process spawned concurrently by another test can inherit
    // the pipe's write end while it is briefly inheritable, so EOF may come
    // late. The report line is all that is needed.
    let _reader = codex_windows_sandbox::read_handle_loop(spawned.stdout_read, move |chunk| {
        let _ = sender.send(chunk.to_vec());
    });
    // SAFETY: the process handle stays valid until closed below.
    unsafe {
        WaitForSingleObject(spawned.process.hProcess, 60_000);
        CloseHandle(spawned.process.hThread);
        CloseHandle(spawned.process.hProcess);
        CloseHandle(token);
    }
    ProbeReport::decode(&String::from_utf8_lossy(&collect_report(&receiver)))
}

/// Collects output until a full report line arrives (or a deadline passes).
fn collect_report(receiver: &std::sync::mpsc::Receiver<Vec<u8>>) -> Vec<u8> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut output = Vec::new();
    loop {
        let text = String::from_utf8_lossy(&output);
        let has_report = text
            .find(REPORT_PREFIX)
            .is_some_and(|start| text[start..].contains('\n'));
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if has_report || remaining.is_zero() {
            return output;
        }
        match receiver.recv_timeout(remaining) {
            Ok(chunk) => output.extend(chunk),
            Err(_) => return output,
        }
    }
}

fn run_probe() {
    let pid: u32 = std::env::var(TARGET_PID_ENV)
        .expect("target pid")
        .parse()
        .expect("numeric pid");
    let (vm_read, environment) =
        match open_process(pid, PROCESS_VM_READ | PROCESS_QUERY_INFORMATION) {
            Ok(handle) => {
                let environment = match read_environment_block(handle) {
                    Some(block) if contains_utf16(&block, CANARY) => EnvRead::CanaryFound,
                    Some(_) => EnvRead::CanaryAbsent,
                    None => EnvRead::Unreadable,
                };
                // SAFETY: opened above.
                unsafe { CloseHandle(handle) };
                (Access::Granted, environment)
            }
            Err(code) => (access_from_error(code), EnvRead::Unreadable),
        };
    let report = ProbeReport {
        vm_read,
        environment,
        dup_handle: try_open(pid, PROCESS_DUP_HANDLE),
        write_dac: try_open(pid, WRITE_DAC),
    };
    println!("{}", report.encode());
    let _ = std::io::stdout().flush();
    std::process::exit(0);
}

fn try_open(pid: u32, access: u32) -> Access {
    match open_process(pid, access) {
        Ok(handle) => {
            // SAFETY: opened above.
            unsafe { CloseHandle(handle) };
            Access::Granted
        }
        Err(code) => access_from_error(code),
    }
}

fn access_from_error(code: u32) -> Access {
    if code == ERROR_ACCESS_DENIED {
        Access::Denied
    } else {
        Access::Failed(code)
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
/// `ps`-style tool would (x64 layout).
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
            /*ProcessBasicInformation*/ 0,
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
fn pf_27_s06_dacl_limits_user_and_owner_rights() {
    assert_eq!(
        super::process_dacl_sddl("S-1-5-21-1-2-3-1001"),
        "D:P(A;;0x101000;;;S-1-5-21-1-2-3-1001)(A;;GA;;;SY)(A;;RC;;;OW)"
    );
}

#[test]
fn pf_27_s06_probe_reports_round_trip() {
    let report = ProbeReport {
        vm_read: Access::Granted,
        environment: EnvRead::CanaryFound,
        dup_handle: Access::Denied,
        write_dac: Access::Failed(87),
    };
    assert_eq!(
        ProbeReport::decode(&format!("noise\n{}\n", report.encode())),
        report
    );
}
