//! PF-27-S06 containment probes, measured on real Windows processes.
//!
//! Every test re-executes this test binary twice, filtered to the entry test
//! below: once as the *target* (stands in for Core or the credential broker;
//! it carries a synthetic canary in its environment block and starts a thread
//! after hardening) and once as the *probe* (stands in for an agent command).
//! The probe runs either under the restricted token the unelevated Windows
//! sandbox gives agent commands, or as a plain same-user process, and reports
//! what it could open and read. Unhardened targets are the positive controls.
//!
//! PF-27-S07 targets also hold two brand-new threads open in their creation
//! window (created suspended, so their TLS callback has not run): one through
//! this image's `CreateThread` import, as every Rust thread is created, and
//! one without it, as Windows starts its own threads.

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
const DEFAULT_DACL_ENV: &str = "CODEX_PF27S07_DEFAULT_DACL";
const NEW_THREADS_ENV: &str = "CODEX_PF27S07_NEW_THREADS";
const NEW_THREAD_IDS_ENV: &str = "CODEX_PF27S07_NEW_THREAD_IDS";
const PARK_ENV: &str = "CODEX_PF27S07_PARK";
const THREAD_IDS_ENV: &str = "CODEX_PF27S08_THREAD_IDS";
const NEW_THREADS_MARKER: &str = "new-threads=";
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

/// PF-27-S08: further rights the broker-token probe also tries (reported
/// with an `x_` prefix when `EXTRA_RIGHTS_ENV` is set).
const EXTRA_PROCESS_RIGHTS: &[(&str, u32)] = &[
    ("x_terminate", 0x0001),
    ("x_create_process", 0x0080),
    ("x_set_quota", 0x0100),
    ("x_query_information", 0x0400),
    ("x_query_limited_information", 0x1000),
];
const EXTRA_THREAD_RIGHTS: &[(&str, u32)] = &[
    ("x_thread_terminate", 0x0001),
    ("x_thread_set_information", 0x0020),
    ("x_thread_impersonate", 0x0100),
    ("x_thread_direct_impersonation", 0x0200),
    ("x_thread_query_information", 0x0040),
];
const EXTRA_RIGHTS_ENV: &str = "CODEX_PF27S08_EXTRA_RIGHTS";

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
    let report = probe_with_restricted_token(&target);
    assert_eq!(report["vm_read"], "granted", "{report:?}");
    assert_eq!(report["environment"], "canary_found", "{report:?}");
}

/// Positive control: a plain same-user process gets every right on an
/// unhardened target, threads included.
#[test]
fn pf_27_s06_same_user_probe_controls_an_unhardened_process() {
    let target = Target::spawn(/*harden*/ false);
    let report = probe_as_same_user(&target);
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
    assert_all_denied(&probe_with_restricted_token(&target));
}

/// The same holds for an unsandboxed process of the same user (for example
/// an MCP server or hook, which run outside the sandbox) that has no enabled
/// privileges. (An administrator with `SeDebugPrivilege` enabled, as on the
/// CI runner, bypasses any DACL by design; the probe disables it first.)
#[test]
fn pf_27_s06_same_user_probe_cannot_read_a_hardened_process() {
    let target = Target::spawn(/*harden*/ true);
    assert_all_denied(&probe_as_same_user(&target));
}

/// PF-27-S07: a thread Core or the broker creates through `CreateThread`
/// (every Rust thread) cannot be opened even in its creation window. A
/// thread created another way still can by a same-user process, which is
/// the positive control that the probe reaches that window.
#[test]
fn pf_27_s07_imported_thread_creation_is_protected_from_the_start() {
    let target = Target::spawn_with(TargetOptions {
        harden: true,
        new_threads: true,
        ..TargetOptions::default()
    });
    let same_user = probe_as_same_user(&target);
    assert_new_threads(&same_user, "imported", "denied");
    assert_eq!(
        same_user["new_direct_thread_get_context"], "granted",
        "{same_user:?}"
    );
    assert_eq!(
        same_user["new_direct_thread_set_context"], "granted",
        "{same_user:?}"
    );
    // The restricted token restricts writes only: reading the unprotected
    // thread's context is its positive control.
    let restricted = probe_with_restricted_token(&target);
    assert_new_threads(&restricted, "imported", "denied");
    assert_eq!(
        restricted["new_direct_thread_get_context"], "granted",
        "{restricted:?}"
    );
}

/// PF-27-S07: with the protected default DACL (the broker), every new
/// thread is protected from creation, however it was started.
#[test]
fn pf_27_s07_default_dacl_protects_every_new_thread() {
    let target = Target::spawn_with(TargetOptions {
        harden: true,
        default_dacl: true,
        new_threads: true,
        ..TargetOptions::default()
    });
    for report in [
        probe_as_same_user(&target),
        probe_with_restricted_token(&target),
    ] {
        assert_new_threads(&report, "imported", "denied");
        assert_new_threads(&report, "direct", "denied");
    }
}

/// PF-27-S07: a process started with `spawn_protected` (the broker) is
/// never openable: no hardening call of its own is needed for its process,
/// first thread, or any thread it starts.
#[test]
fn pf_27_s07_protected_spawn_is_never_openable() {
    let target = Target::spawn_with(TargetOptions {
        new_threads: true,
        protected_spawn: true,
        ..TargetOptions::default()
    });
    for report in [
        probe_as_same_user(&target),
        probe_with_restricted_token(&target),
    ] {
        assert_new_threads(&report, "imported", "denied");
        assert_new_threads(&report, "direct", "denied");
        let original: Report = report
            .iter()
            .filter(|(key, _)| !key.starts_with("new_"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_all_denied(&original);
    }
}

/// PF-27-S07: the broker's own containment still succeeds in a process
/// started protected (it cannot rewrite its own DACL there, and need not).
#[test]
fn pf_27_s07_protected_spawn_then_broker_hardening_succeeds() {
    let target = Target::spawn_with(TargetOptions {
        harden: true,
        default_dacl: true,
        new_threads: true,
        protected_spawn: true,
    });
    for report in [
        probe_as_same_user(&target),
        probe_with_restricted_token(&target),
    ] {
        assert_new_threads(&report, "imported", "denied");
        assert_new_threads(&report, "direct", "denied");
        let original: Report = report
            .iter()
            .filter(|(key, _)| !key.starts_with("new_"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        assert_all_denied(&original);
    }
}

/// What `spawn_protected` needs to start `command`: exactly the
/// environment `command` would pass, and `PARK_ENV` (its stdin is closed).
fn protected_spawn_parts(
    command: &Command,
) -> (
    std::path::PathBuf,
    Vec<std::ffi::OsString>,
    Vec<(std::ffi::OsString, std::ffi::OsString)>,
) {
    let mut env: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os()
        .filter(|(name, _)| {
            !command
                .get_envs()
                .any(|(set, _)| set.eq_ignore_ascii_case(name))
        })
        .collect();
    env.extend(
        command
            .get_envs()
            .filter_map(|(name, value)| Some((name.to_owned(), value?.to_owned()))),
    );
    env.push((PARK_ENV.into(), "1".into()));
    let args = command.get_args().map(ToOwned::to_owned).collect();
    (command.get_program().into(), args, env)
}

/// PF-27-S07: between its creation and its first instruction (suspended), a
/// process from `spawn_protected` and its first thread are already denied.
/// Positive control: the same process started suspended by `std`.
#[test]
fn pf_27_s07_protected_spawn_is_unopenable_while_suspended() {
    let mut command = child_command("target");
    command.env(CANARY_ENV, CANARY);
    let (program, args, env) = protected_spawn_parts(&command);
    let (child, _stdout, _thread) = crate::windows_protected_spawn::spawn_protected_suspended(
        &program,
        &args,
        &env,
        crate::windows_protected_spawn::Confinement::Broker(
            crate::windows_broker_token::BROKER_TOKEN,
        ),
    )
    .expect("protected spawn");
    let target = Target {
        child: TargetChild::Protected(child),
        new_threads: None,
    };
    assert_all_denied(&probe_as_same_user(&target));
    assert_all_denied(&probe_with_restricted_token(&target));

    const CREATE_SUSPENDED: u32 = 0x4;
    std::os::windows::process::CommandExt::creation_flags(&mut command, CREATE_SUSPENDED);
    let control = Target {
        child: TargetChild::Std(command.spawn().expect("std spawn")),
        new_threads: None,
    };
    // The restricted token's own positive control is
    // `pf_27_s06_restricted_token_probe_reads_an_unhardened_process`.
    let report = probe_as_same_user(&control);
    assert_eq!(report["vm_read"], "granted", "{report:?}");
    assert_eq!(
        report["thread_get_context"].split('@').next(),
        Some("granted"),
        "{report:?}"
    );
}

/// PF-27-S08: under the broker token the broker cannot open an ordinary
/// process of the user (or its threads) for any right that reads, injects
/// into, starts a child of, re-ACLs or impersonates it. Measured exceptions
/// (low integrity leaves the execute-class rights): query-limited
/// information and process terminate. Positive control: the same probe
/// started protected with a copy of this process's token (the PF-27-S07
/// broker) gets every right.
#[test]
fn pf_27_s08_broker_token_cannot_open_the_users_processes() {
    use crate::windows_protected_spawn::Confinement;
    const STILL_GRANTED: &[&str] = &["x_query_limited_information", "x_terminate"];
    let target = Target::spawn(/*harden*/ false);
    let control = probe_started_protected(&target, Confinement::SameToken);
    assert_eq!(control["vm_read"], "granted", "{control:?}");
    assert_eq!(control["environment"], "canary_found", "{control:?}");
    let all_rights = PROCESS_RIGHTS
        .iter()
        .chain(THREAD_RIGHTS)
        .chain(EXTRA_PROCESS_RIGHTS)
        .chain(EXTRA_THREAD_RIGHTS);
    for (name, _) in all_rights.clone() {
        assert_eq!(
            control[*name].split('@').next(),
            Some("granted"),
            "{name}: {control:?}"
        );
    }
    let confined = probe_started_protected(
        &target,
        Confinement::Broker(crate::windows_broker_token::BROKER_TOKEN),
    );
    eprintln!("pf27s08: probe under the broker token: {confined:?}");
    assert_eq!(confined["environment"], "unreadable", "{confined:?}");
    assert_eq!(confined["vm_read"], "denied", "{confined:?}");
    for (name, _) in all_rights {
        let expected = if STILL_GRANTED.contains(name) {
            "granted"
        } else {
            "denied"
        };
        assert_eq!(
            confined[*name].split('@').next(),
            Some(expected),
            "{name}: {confined:?}"
        );
    }
}

/// The probe, started with `spawn_protected` under `confinement`.
fn probe_started_protected(
    target: &Target,
    confinement: crate::windows_protected_spawn::Confinement,
) -> Report {
    let mut command = child_command("probe");
    command.env(TARGET_PID_ENV, target.pid().to_string());
    // A low-integrity process cannot list another process's threads.
    let threads: Vec<String> = thread_ids(target.pid())
        .iter()
        .map(u32::to_string)
        .collect();
    command.env(THREAD_IDS_ENV, threads.join(";"));
    command.env(EXTRA_RIGHTS_ENV, "1");
    let (program, args, env) = protected_spawn_parts(&command);
    let (mut child, stdout) =
        crate::windows_protected_spawn::spawn_protected_with(&program, &args, &env, confinement)
            .expect("protected probe");
    let output = read_until(&line_channel(stdout), REPORT_PREFIX);
    let _ = child.kill();
    let _ = child.wait();
    decode(&output)
}

fn assert_new_threads(report: &Report, thread: &str, expected: &str) {
    for (name, _) in THREAD_RIGHTS {
        let key = format!("new_{thread}_{name}");
        assert_eq!(
            report.get(&key).map(String::as_str),
            Some(expected),
            "{key}: {report:?}"
        );
    }
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

/// How a target is started and hardened.
#[derive(Clone, Copy, Default)]
struct TargetOptions {
    /// `restrict_current_process_access` (Core and the broker).
    harden: bool,
    /// `protect_new_objects_by_default` (the broker).
    default_dacl: bool,
    /// Hold two brand-new threads in their creation window.
    new_threads: bool,
    /// Started with `spawn_protected` instead of `std`.
    protected_spawn: bool,
}

enum TargetChild {
    Std(Child),
    Protected(crate::ProtectedChild),
}

/// The target process; killed on drop.
struct Target {
    child: TargetChild,
    /// `imported=<tid>;direct=<tid>` for PF-27-S07 targets.
    new_threads: Option<String>,
}

impl Target {
    fn spawn(harden: bool) -> Self {
        Self::spawn_with(TargetOptions {
            harden,
            ..TargetOptions::default()
        })
    }

    fn spawn_with(options: TargetOptions) -> Self {
        let mut command = child_command("target");
        command.env(CANARY_ENV, CANARY);
        let flags = [
            (options.harden, HARDEN_ENV),
            (options.default_dacl, DEFAULT_DACL_ENV),
            (options.new_threads, NEW_THREADS_ENV),
        ];
        for (_, name) in flags.iter().filter(|(on, _)| *on) {
            command.env(name, "1");
        }
        let (child, lines) = if options.protected_spawn {
            let (program, args, env) = protected_spawn_parts(&command);
            let (child, stdout) =
                crate::spawn_protected(&program, &args, &env).expect("protected spawn");
            (TargetChild::Protected(child), line_channel(stdout))
        } else {
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit());
            let mut child = command.spawn().expect("spawn target");
            let lines = line_channel(child.stdout.take().expect("target stdout"));
            (TargetChild::Std(child), lines)
        };
        // Killed on drop if it never becomes ready.
        let mut target = Self {
            child,
            new_threads: None,
        };
        let output = read_until(&lines, READY);
        if !output.contains(READY) {
            panic!("target did not become ready; it printed: {output}");
        }
        target.new_threads = output
            .lines()
            .find_map(|line| line.split_once(NEW_THREADS_MARKER))
            .map(|(_, ids)| ids.trim().to_string());
        target
    }

    fn pid(&self) -> u32 {
        match &self.child {
            TargetChild::Std(child) => child.id(),
            TargetChild::Protected(child) => child.id(),
        }
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        match &mut self.child {
            TargetChild::Std(child) => {
                let _ = child.kill();
                let _ = child.wait();
            }
            TargetChild::Protected(child) => {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn child_command(role: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test binary"));
    command
        .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, role)
        .env_remove(CANARY_ENV)
        .env_remove(HARDEN_ENV)
        .env_remove(DEFAULT_DACL_ENV)
        .env_remove(NEW_THREADS_ENV)
        .env_remove(NEW_THREAD_IDS_ENV)
        .env_remove(PARK_ENV)
        .env_remove(THREAD_IDS_ENV)
        .env_remove(EXTRA_RIGHTS_ENV);
    command
}

fn run_target() {
    if std::env::var_os(DEFAULT_DACL_ENV).is_some()
        && let Err(err) = crate::protect_new_objects_by_default()
    {
        // The broker token cannot change its own default DACL; Core set it.
        assert!(
            crate::windows_broker_token::default_dacl_is_protected().unwrap_or(false),
            "protect new objects: {err}"
        );
    }
    if std::env::var_os(HARDEN_ENV).is_some() {
        if let Err(err) = restrict_current_process_access() {
            // Visible even when stderr is closed (protected spawn).
            println!("\nrestrict_current_process_access failed: {err}");
            panic!("restrict process access: {err}");
        }
        assert!(
            crate::thread_creation_protected(),
            "CreateThread imports were not redirected"
        );
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
    let new_threads = if std::env::var_os(NEW_THREADS_ENV).is_some() {
        assert_threads_keep_rights_to_themselves();
        format!(" {NEW_THREADS_MARKER}{}", hold_new_threads())
    } else {
        String::new()
    };
    let mut stdout = std::io::stdout();
    // Own line: libtest prints the test name without a newline first.
    writeln!(stdout, "\n{READY}{new_threads}").expect("write ready");
    stdout.flush().expect("flush ready");
    // Stay alive until the test kills us or closes stdin.
    let mut sink = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut sink);
    while std::env::var_os(PARK_ENV).is_some() {
        std::thread::park();
    }
    std::process::exit(0);
}

unsafe extern "system" fn never_runs(_parameter: *mut c_void) -> u32 {
    0
}

/// Creates two suspended threads, which stay in their creation window (the
/// TLS callback runs only when a thread first runs): one through this
/// image's `CreateThread` import, one with `CreateRemoteThread` on itself.
/// Returns `imported=<tid>;direct=<tid>`.
fn hold_new_threads() -> String {
    use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;
    use windows_sys::Win32::System::Threading::CreateRemoteThread;
    use windows_sys::Win32::System::Threading::CreateThread;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut imported = 0_u32;
    let mut direct = 0_u32;
    // SAFETY: the threads never run; their handles are closed at once
    // (suspended threads outlive their handles).
    unsafe {
        let handle = CreateThread(
            std::ptr::null(),
            0,
            Some(never_runs),
            std::ptr::null(),
            CREATE_SUSPENDED,
            &mut imported,
        );
        assert_ne!(handle, 0, "CreateThread");
        CloseHandle(handle);
        let handle = CreateRemoteThread(
            GetCurrentProcess(),
            std::ptr::null(),
            0,
            Some(never_runs),
            std::ptr::null(),
            CREATE_SUSPENDED,
            &mut direct,
        );
        assert_ne!(handle, 0, "CreateRemoteThread");
        CloseHandle(handle);
    }
    format!("imported={imported};direct={direct}")
}

/// A thread created with the protected DACL keeps the rights Rust and its
/// runtimes use on themselves (priority, thread names), and its TLS
/// callback does not count it as a failure.
fn assert_threads_keep_rights_to_themselves() {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::System::Threading::GetCurrentThread;
    use windows_sys::Win32::System::Threading::GetThreadDescription;
    use windows_sys::Win32::System::Threading::SetThreadDescription;
    use windows_sys::Win32::System::Threading::SetThreadPriority;
    use windows_sys::Win32::System::Threading::THREAD_PRIORITY_NORMAL;
    std::thread::spawn(|| {
        let name: Vec<u16> = "pf27s07-named".encode_utf16().chain([0]).collect();
        // SAFETY: plain calls on this thread's own pseudo-handle; the
        // description is freed below.
        unsafe {
            assert_ne!(
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_NORMAL),
                0,
                "SetThreadPriority on itself"
            );
            assert!(
                SetThreadDescription(GetCurrentThread(), name.as_ptr()) >= 0,
                "SetThreadDescription on itself"
            );
            let mut read: *mut u16 = std::ptr::null_mut();
            assert!(
                GetThreadDescription(GetCurrentThread(), &mut read) >= 0,
                "GetThreadDescription on itself"
            );
            let len = (0..).take_while(|&i| *read.add(i) != 0).count();
            let read_name = String::from_utf16_lossy(std::slice::from_raw_parts(read, len));
            LocalFree(read as _);
            assert_eq!(read_name, "pf27s07-named");
        }
    })
    .join()
    .expect("a new thread lost rights to itself");
    assert_eq!(
        super::thread_protection_failures(),
        0,
        "the TLS callback failed on a thread created protected"
    );
}

fn probe_as_same_user(target: &Target) -> Report {
    let mut command = child_command("probe");
    if let Some(ids) = &target.new_threads {
        command.env(NEW_THREAD_IDS_ENV, ids);
    }
    let mut child = command
        .env(TARGET_PID_ENV, target.pid().to_string())
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
fn probe_with_restricted_token(target: &Target) -> Report {
    let exe = std::env::current_exe().expect("test binary");
    let argv = vec![
        exe.to_string_lossy().into_owned(),
        CHILD_TEST.to_string(),
        "--exact".to_string(),
        "--nocapture".to_string(),
        "--test-threads=1".to_string(),
    ];
    let mut env: HashMap<String, String> = std::env::vars()
        .filter(|(key, _)| {
            ![
                CANARY_ENV,
                HARDEN_ENV,
                DEFAULT_DACL_ENV,
                NEW_THREADS_ENV,
                PARK_ENV,
            ]
            .contains(&key.as_str())
        })
        .collect();
    env.insert(ROLE_ENV.to_string(), "probe".to_string());
    env.insert(TARGET_PID_ENV.to_string(), target.pid().to_string());
    if let Some(ids) = &target.new_threads {
        env.insert(NEW_THREAD_IDS_ENV.to_string(), ids.clone());
    }
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
    // The broker token has no privileges to disable (and may not adjust
    // its own token).
    if crate::current_token_is_broker_token().is_err() {
        disable_all_privileges();
    }
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
    let mut threads = thread_ids(pid);
    if threads.is_empty() {
        threads = std::env::var(THREAD_IDS_ENV)
            .unwrap_or_default()
            .split(';')
            .filter_map(|tid| tid.parse().ok())
            .collect();
    }
    for (name, access) in THREAD_RIGHTS {
        report.insert((*name).to_string(), open_threads(&threads, *access));
    }
    if std::env::var_os(EXTRA_RIGHTS_ENV).is_some() {
        for (name, access) in EXTRA_PROCESS_RIGHTS {
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
        for (name, access) in EXTRA_THREAD_RIGHTS {
            report.insert((*name).to_string(), open_threads(&threads, *access));
        }
    }
    // PF-27-S07: the target's brand-new threads, one by one.
    let new_threads = std::env::var(NEW_THREAD_IDS_ENV).unwrap_or_default();
    for (thread, tid) in new_threads
        .split(';')
        .filter_map(|pair| pair.split_once('='))
    {
        let tid: u32 = tid.parse().expect("thread id");
        for (name, access) in THREAD_RIGHTS {
            let outcome = open_threads(&[tid], *access);
            let outcome = outcome.split('@').next().unwrap_or_default().to_string();
            report.insert(format!("new_{thread}_{name}"), outcome);
        }
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
