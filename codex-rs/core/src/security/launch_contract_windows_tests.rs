//! PF-27-S06: measured pass of the secretless launch contract on Windows.
//!
//! A real command runs under the elevated Windows sandbox with the profile
//! the contract produces, and must not read the vault, sign-in files, policy
//! store or state databases, nor write `CODEX_HOME`. A probe (this test
//! binary, re-executed) then runs under the same sandbox and must not open a
//! hardened stand-in for Core to read its memory or environment.

use super::LaunchContract;
use super::harden_current_process;
use crate::exec::ExecCapturePolicy;
use crate::exec::ExecParams;
use crate::exec::process_exec_tool_call;
use crate::sandboxing::SandboxPermissions;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_sandboxing::SandboxType;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::io::BufRead as _;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;

const ROLE_ENV: &str = "CODEX_PF27S06_CORE_ROLE";
const TARGET_PID_ENV: &str = "CODEX_PF27S06_CORE_TARGET_PID";
const CHILD_TEST: &str = "security::launch_contract::windows_tests::pf_27_s06_core_child_entry";
const REPORT_PREFIX: &str = "pf27s06-core-probe:";
const ERROR_ACCESS_DENIED: u32 = 5;

#[test]
fn pf_27_s06_core_child_entry() {
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("target") => run_target(/*harden*/ true),
        Ok("plain-target") => run_target(/*harden*/ false),
        Ok("probe") => run_probe(),
        _ => {}
    }
}

#[tokio::test]
async fn pf_27_s06_elevated_launch_cannot_read_protected_files_or_core_memory() {
    // Inside the user profile, like the default `~/.codex`: the elevated
    // sandbox's setup grants its user read access to profile folders.
    let profile_dir = std::env::var_os("USERPROFILE").expect("USERPROFILE");
    let codex_home_dir = tempfile::Builder::new()
        .prefix("pf27s06-codex-home-")
        .tempdir_in(profile_dir)
        .expect("codex home");
    let workspace_dir = tempfile::tempdir().expect("workspace");
    let codex_home = absolute(codex_home_dir.path());
    let cwd = absolute(workspace_dir.path());
    let _codex_home = EnvGuard::set("CODEX_HOME", codex_home.as_path());
    stage_windows_sandbox_helpers();

    std::fs::create_dir_all(codex_home.join("secrets")).expect("secrets dir");
    for (name, body) in [
        ("secrets/vault.json", "pf27s06-vault"),
        ("auth.json", "pf27s06-auth"),
        ("config.toml", "model = \"pf27s06\""),
        ("state_5.sqlite", "pf27s06-state"),
        ("notes.txt", "unprotected control"),
    ] {
        std::fs::write(codex_home.join(name), body).expect("fixture file");
    }
    std::fs::write(cwd.join("public.txt"), "public ok\n").expect("public file");

    let contract = LaunchContract::capture(&codex_home, std::iter::empty(), /*hardened*/ true);
    assert_eq!(
        contract.check_sandbox(
            SandboxType::WindowsRestrictedToken,
            /*sandbox_requested*/ true,
            /*exec_server*/ false,
            /*windows_elevated*/ true,
        ),
        Ok(())
    );
    let base = PermissionProfile::workspace_write_with(
        &[],
        NetworkSandboxPolicy::Restricted,
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    )
    .materialize_project_roots_with_workspace_roots(std::slice::from_ref(&cwd));
    let protected = contract
        .protect_permissions(&base, cwd.as_path())
        .expect("protected profile");

    // Unquoted: the elevated runner re-quotes argv, which cmd.exe then sees
    // as part of the path. The profile path has no spaces on the CI runner.
    let home = codex_home.as_path().display().to_string();
    assert!(!home.contains(' '), "CODEX_HOME path has a space: {home}");
    let read = |label: &str, file: &str| {
        format!("(type {home}\\{file} 1>NUL 2>NUL && echo {label}-READ || echo {label}-DENIED)")
    };
    let script = [
        read("VAULT", "secrets\\vault.json"),
        read("AUTH", "auth.json"),
        read("CONFIG", "config.toml"),
        read("SQLITE", "state_5.sqlite"),
        read("NOTES", "notes.txt"),
        format!(
            "(echo x> {home}\\config.toml 2>NUL && echo CONFIG-WRITE-ALLOWED || echo CONFIG-WRITE-DENIED)"
        ),
        format!(
            "(echo x> {home}\\planted.txt 2>NUL && echo HOME-WRITE-ALLOWED || echo HOME-WRITE-DENIED)"
        ),
        "type public.txt".to_string(),
    ]
    .join(" & ");
    // Positive control: without the contract's profile the sandbox user
    // reads these files, so the denials below come from the contract.
    let control = run_sandboxed(
        vec!["cmd.exe".into(), "/D".into(), "/C".into(), script.clone()],
        HashMap::new(),
        &base,
        &cwd,
    )
    .await;
    eprintln!("pf27s06 elevated file probes, script: {script}");
    eprintln!("pf27s06 elevated file probes, base profile: {control}");
    for readable in ["VAULT", "AUTH", "CONFIG", "SQLITE", "NOTES"] {
        assert!(
            control.contains(&format!("{readable}-READ")),
            "{readable}: {control}"
        );
    }
    let files = run_sandboxed(
        vec!["cmd.exe".into(), "/D".into(), "/C".into(), script],
        HashMap::new(),
        &protected,
        &cwd,
    )
    .await;
    eprintln!("pf27s06 elevated file probes, protected profile: {files}");
    assert!(files.contains("NOTES-READ"), "unprotected file: {files}");
    for denied in ["VAULT", "AUTH", "CONFIG", "SQLITE"] {
        assert!(
            files.contains(&format!("{denied}-DENIED")),
            "{denied}: {files}"
        );
        assert!(
            !files.contains(&format!("{denied}-READ")),
            "{denied}: {files}"
        );
    }
    assert!(files.contains("CONFIG-WRITE-DENIED"), "{files}");
    assert!(files.contains("HOME-WRITE-DENIED"), "{files}");
    assert!(
        files.contains("public ok"),
        "allowed reads still work: {files}"
    );
    assert_eq!(
        std::fs::read_to_string(codex_home.join("config.toml")).expect("config"),
        "model = \"pf27s06\""
    );

    // Files that appear while a protected command runs: one in a protected
    // directory (inherited deny), and auth.json replaced by rename, as a
    // token refresh does (recorded: a per-file deny does not carry over).
    let later = {
        let profile = protected.clone();
        let cwd = cwd.clone();
        let script = format!(
            "ping -n 6 127.0.0.1 >NUL & {} & {}",
            read("LATER", "secrets\\later.json"),
            read("REPLACED", "auth.json")
        );
        tokio::spawn(async move {
            run_sandboxed(
                vec!["cmd.exe".into(), "/D".into(), "/C".into(), script],
                HashMap::new(),
                &profile,
                &cwd,
            )
            .await
        })
    };
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    std::fs::write(
        codex_home.join("secrets").join("later.json"),
        "pf27s06-later",
    )
    .expect("later file");
    std::fs::write(codex_home.join("auth.json.tmp"), "pf27s06-refreshed").expect("tmp");
    std::fs::rename(
        codex_home.join("auth.json.tmp"),
        codex_home.join("auth.json"),
    )
    .expect("replace auth.json");
    let later = later.await.expect("later run");
    eprintln!("pf27s06 elevated probes for files created during a run: {later}");
    assert!(later.contains("LATER-DENIED"), "{later}");

    // Positive control: the probe reads an unhardened process of its user.
    let unhardened = Target::spawn(/*harden*/ false);
    assert_eq!(
        report(&run_probe_unsandboxed(unhardened.pid())),
        "vm_read=granted,threads=granted"
    );
    drop(unhardened);

    // Core stand-in: hardened exactly as an armed contract hardens Core.
    let target = Target::spawn(/*harden*/ true);
    let unsandboxed = run_probe_unsandboxed(target.pid());
    let exe = std::env::current_exe().expect("test binary");
    let env = HashMap::from([
        (ROLE_ENV.to_string(), "probe".to_string()),
        (TARGET_PID_ENV.to_string(), target.pid().to_string()),
        (
            "SystemRoot".to_string(),
            std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()),
        ),
    ]);
    let sandboxed = run_sandboxed(
        vec![
            exe.to_string_lossy().into_owned(),
            CHILD_TEST.to_string(),
            "--exact".to_string(),
            "--nocapture".to_string(),
            "--test-threads=1".to_string(),
        ],
        env,
        &protected,
        &cwd,
    )
    .await;
    eprintln!("pf27s06 elevated memory probe (hardened stand-in): {sandboxed}");
    // Recorded: is the separate sandbox user alone already the boundary?
    let plain = Target::spawn(/*harden*/ false);
    let mut env = HashMap::from([
        (ROLE_ENV.to_string(), "probe".to_string()),
        (TARGET_PID_ENV.to_string(), plain.pid().to_string()),
    ]);
    if let Ok(root) = std::env::var("SystemRoot") {
        env.insert("SystemRoot".to_string(), root);
    }
    let plain_report = run_sandboxed(
        vec![
            exe.to_string_lossy().into_owned(),
            CHILD_TEST.to_string(),
            "--exact".to_string(),
            "--nocapture".to_string(),
            "--test-threads=1".to_string(),
        ],
        env,
        &protected,
        &cwd,
    )
    .await;
    eprintln!("pf27s06 elevated memory probe (unhardened stand-in): {plain_report}");
    assert_eq!(report(&unsandboxed), "vm_read=denied,threads=denied");
    assert_eq!(report(&sandboxed), "vm_read=denied,threads=denied");
}

async fn run_sandboxed(
    command: Vec<String>,
    env: HashMap<String, String>,
    profile: &PermissionProfile,
    cwd: &AbsolutePathBuf,
) -> String {
    let output = process_exec_tool_call(
        ExecParams {
            command,
            cwd: cwd.clone(),
            expiration: 120_000.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env,
            network: None,
            network_environment_id: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: WindowsSandboxLevel::Elevated,
            windows_sandbox_private_desktop: false,
            justification: None,
            arg0: None,
        },
        profile,
        cwd,
        std::slice::from_ref(cwd),
        &None,
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await
    .expect("elevated sandbox run");
    format!("{}{}", output.stdout.text, output.stderr.text)
}

fn report(output: &str) -> String {
    output
        .lines()
        .find_map(|line| line.find(REPORT_PREFIX).map(|start| &line[start..]))
        .and_then(|line| line.strip_prefix(REPORT_PREFIX))
        .map(str::trim)
        .unwrap_or_else(|| panic!("probe printed no report:\n{output}"))
        .to_string()
}

fn run_probe_unsandboxed(pid: u32) -> String {
    let mut child = child_command("probe")
        .env(TARGET_PID_ENV, pid.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("unsandboxed probe");
    let lines = line_channel(child.stdout.take().expect("probe stdout"));
    let output = read_until(&lines, REPORT_PREFIX);
    let _ = child.kill();
    let _ = child.wait();
    output
}

/// Lines from a child's stdout, read on a detached thread.
fn line_channel(stdout: std::process::ChildStdout) -> std::sync::mpsc::Receiver<String> {
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

/// Collects lines until one contains `marker`, or a minute passes.
fn read_until(lines: &std::sync::mpsc::Receiver<String>, marker: &str) -> String {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut output = String::new();
    while !output.contains(marker) {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
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

fn child_command(role: &str) -> std::process::Command {
    let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
    command
        .args([CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, role);
    command
}

/// The Core stand-in; killed on drop.
struct Target {
    child: std::process::Child,
}

impl Target {
    fn spawn(harden: bool) -> Self {
        let mut child = child_command(if harden { "target" } else { "plain-target" })
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("spawn target");
        let lines = line_channel(child.stdout.take().expect("target stdout"));
        let output = read_until(&lines, "pf27s06-core-target:ready");
        if !output.contains("pf27s06-core-target:ready") {
            let _ = child.kill();
            panic!("Core stand-in did not start or could not harden itself: {output}");
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

fn run_target(harden: bool) {
    let mut stdout = std::io::stdout();
    if !harden || harden_current_process() {
        // Own line: libtest prints the test name without a newline first.
        let _ = writeln!(stdout, "\npf27s06-core-target:ready");
    }
    let _ = stdout.flush();
    let mut sink = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut sink);
    std::process::exit(0);
}

fn run_probe() {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPTHREAD;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::THREADENTRY32;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::Thread32First;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::Thread32Next;
    use windows_sys::Win32::System::Threading::OpenProcess;
    use windows_sys::Win32::System::Threading::OpenThread;
    use windows_sys::Win32::System::Threading::PROCESS_QUERY_INFORMATION;
    use windows_sys::Win32::System::Threading::PROCESS_VM_READ;
    use windows_sys::Win32::System::Threading::THREAD_GET_CONTEXT;
    use windows_sys::Win32::System::Threading::THREAD_SET_CONTEXT;
    let pid: u32 = std::env::var(TARGET_PID_ENV)
        .ok()
        .and_then(|pid| pid.parse().ok())
        .unwrap_or_default();
    disable_all_privileges();
    let classify = |handle: isize| {
        if handle == 0 {
            // SAFETY: reads the thread's last error.
            match unsafe { GetLastError() } {
                ERROR_ACCESS_DENIED => "denied".to_string(),
                code => format!("failed:{code}"),
            }
        } else {
            // SAFETY: opened by the caller; closed once.
            unsafe { CloseHandle(handle) };
            "granted".to_string()
        }
    };
    // SAFETY: plain OpenProcess; classified (and closed) at once.
    let vm_read =
        classify(unsafe { OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, 0, pid) });
    // Any of the target's threads opened for reading or setting its context.
    let mut threads = "no_threads".to_string();
    // SAFETY: a thread snapshot; closed below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot != INVALID_HANDLE_VALUE {
        // SAFETY: zeroed POD with its size set.
        let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
        entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
        // SAFETY: valid snapshot and entry.
        let mut more = unsafe { Thread32First(snapshot, &mut entry) } != 0;
        while more {
            if entry.th32OwnerProcessID == pid && threads != "granted" {
                // SAFETY: plain OpenThread; classified (and closed) at once.
                threads = classify(unsafe {
                    OpenThread(
                        THREAD_GET_CONTEXT | THREAD_SET_CONTEXT,
                        0,
                        entry.th32ThreadID,
                    )
                });
            }
            // SAFETY: as above.
            more = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
        }
        // SAFETY: opened above.
        unsafe { CloseHandle(snapshot) };
    }
    let mut stdout = std::io::stdout();
    let _ = writeln!(
        stdout,
        "\n{REPORT_PREFIX}vm_read={vm_read},threads={threads}"
    );
    let _ = stdout.flush();
    std::process::exit(0);
}

/// Disables every privilege in this process's token, as in an ordinary user
/// process: CI runners run elevated with `SeDebugPrivilege` enabled, which
/// opens any process regardless of its DACL.
fn disable_all_privileges() {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::AdjustTokenPrivileges;
    use windows_sys::Win32::Security::TOKEN_ADJUST_PRIVILEGES;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    let mut token = 0;
    // SAFETY: opens this process's token; closed below.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token) } != 0 {
        // SAFETY: DisableAllPrivileges ignores the new-state arguments.
        unsafe {
            AdjustTokenPrivileges(
                token,
                /*DisableAllPrivileges*/ 1,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            CloseHandle(token);
        }
    }
}

fn absolute(path: &Path) -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path(dunce::canonicalize(path).expect("canonical path"))
        .expect("absolute path")
}

/// Copies the elevated sandbox helpers next to this test binary, where the
/// sandbox looks for them (as the Windows sandbox integration tests do).
fn stage_windows_sandbox_helpers() {
    let exe = std::env::current_exe().expect("test binary");
    let resources = exe.parent().expect("test dir").join("codex-resources");
    std::fs::create_dir_all(&resources).expect("resources dir");
    for helper in ["codex-windows-sandbox-setup", "codex-command-runner"] {
        let source = codex_utils_cargo_bin::cargo_bin(helper).expect("sandbox helper built");
        let destination = resources.join(format!("{helper}.exe"));
        if let Err(error) = std::fs::copy(&source, &destination)
            && !destination.exists()
        {
            panic!("stage {helper}: {error}");
        }
    }
}

struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let original = std::env::var_os(key);
        // SAFETY: the pf_27_s06 Windows tests run alone in their process.
        unsafe { std::env::set_var(key, value) };
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: as in `set`.
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}
