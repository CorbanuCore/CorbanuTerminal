//! PF-27-S08 probes: what a process under the broker token can still do.
//!
//! Each test re-executes this test binary (filtered to the entry test below)
//! with `spawn_protected`: once under the broker token, and once, as the
//! positive control, with a copy of this process's token (the PF-27-S07
//! broker). The child reports what it could do; the parent compares.
// The reports are the evidence of the measured runs (`--nocapture`).
#![allow(clippy::print_stderr, clippy::print_stdout)]

use crate::windows_protected_spawn::Confinement;
use crate::windows_protected_spawn::spawn_protected_with;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read as _;
use std::path::Path;
use std::path::PathBuf;

const ROLE_ENV: &str = "CODEX_PF27S08_ROLE";
const DIRS_ENV: &str = "CODEX_PF27S08_DIRS";
const CREDENTIAL_ENV: &str = "CODEX_PF27S08_CREDENTIAL";
const CHILD_TEST: &str = "windows_broker_token::tests::pf_27_s08_child_entry";
const REPORT_PREFIX: &str = "pf27s08-report:";
const ERROR_ACCESS_DENIED: i32 = 5;

type Report = BTreeMap<String, String>;

#[test]
fn pf_27_s08_child_entry() {
    let report = match std::env::var(ROLE_ENV).as_deref() {
        Ok("token") => token_report(),
        Ok("files") => file_report(),
        Ok("credential") => credential_report(),
        Ok("network") => network_report(),
        _ => return,
    };
    let line: Vec<String> = report.iter().map(|(k, v)| format!("{k}={v}")).collect();
    // Own line: libtest prints the test name without a newline first.
    println!("\n{REPORT_PREFIX}{}", line.join(","));
}

/// Temporary diagnostics: which token variants can start this test binary
/// and `cmd.exe`, with exit codes.
#[test]
fn pf_27_s08_diag_token_variants() {
    use crate::windows_broker_token::BROKER_TOKEN;
    use crate::windows_broker_token::BrokerTokenOptions;
    use crate::windows_broker_token::create_broker_token_with;
    for low_integrity in [true, false] {
        for logon_sid in [false, true] {
            for everyone in [false, true] {
                let options = BrokerTokenOptions {
                    logon_sid,
                    everyone,
                    low_integrity,
                    ..BROKER_TOKEN
                };
                let exe = std::env::current_exe().expect("exe");
                let system_root = std::env::var("SystemRoot").expect("SystemRoot");
                for argv in [
                    vec![
                        format!(r"{system_root}\System32\cmd.exe"),
                        "/d".to_string(),
                        "/c".to_string(),
                        "echo pf27s08-cmd-ok".to_string(),
                    ],
                    vec![
                        exe.to_string_lossy().into_owned(),
                        CHILD_TEST.to_string(),
                        "--exact".to_string(),
                        "--nocapture".to_string(),
                    ],
                ] {
                    let token = create_broker_token_with(options).expect("token");
                    let (code, output) = run_with_token(token, &argv, &[(ROLE_ENV, "token")]);
                    eprintln!(
                        "pf27s08-diag low={low_integrity} logon={logon_sid} everyone={everyone} {}: exit=0x{code:x} out={}",
                        argv[0].rsplit('\\').next().unwrap_or_default(),
                        output
                            .replace('\n', " | ")
                            .chars()
                            .take(300)
                            .collect::<String>()
                    );
                }
            }
        }
    }
}

fn run_with_token(
    token: std::os::windows::io::OwnedHandle,
    argv: &[String],
    extra_env: &[(&str, &str)],
) -> (u32, String) {
    use std::collections::HashMap;
    use std::os::windows::io::IntoRawHandle as _;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Threading::GetExitCodeProcess;
    use windows_sys::Win32::System::Threading::TerminateProcess;
    use windows_sys::Win32::System::Threading::WaitForSingleObject;
    let mut env: HashMap<String, String> = std::env::vars()
        .filter(|(name, _)| !name.to_ascii_uppercase().starts_with("CODEX_PF27S08_"))
        .collect();
    for (name, value) in extra_env {
        env.insert((*name).to_string(), (*value).to_string());
    }
    let token = token.into_raw_handle() as HANDLE;
    let spawned = match codex_windows_sandbox::spawn_process_with_pipes(
        token,
        argv,
        &std::env::temp_dir(),
        &env,
        codex_windows_sandbox::StdinMode::Closed,
        codex_windows_sandbox::StderrMode::MergeStdout,
        codex_windows_sandbox::ConsoleMode::NoWindow,
        /*use_private_desktop*/ false,
        /*logs_base_dir*/ None,
    ) {
        Ok(spawned) => spawned,
        Err(err) => {
            // SAFETY: owned above.
            unsafe { CloseHandle(token) };
            return (u32::MAX, format!("spawn failed: {err}"));
        }
    };
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    let reader = codex_windows_sandbox::read_handle_loop(spawned.stdout_read, move |chunk| {
        let _ = sender.send(chunk.to_vec());
    });
    let mut code = 0_u32;
    // SAFETY: the handles stay valid until closed below.
    unsafe {
        const WAIT_TIMEOUT: u32 = 0x102;
        if WaitForSingleObject(spawned.process.hProcess, 180_000) == WAIT_TIMEOUT {
            TerminateProcess(spawned.process.hProcess, 1);
        }
        GetExitCodeProcess(spawned.process.hProcess, &mut code);
        CloseHandle(spawned.process.hThread);
        CloseHandle(spawned.process.hProcess);
        CloseHandle(token);
    }
    let _ = reader.join();
    let output: Vec<u8> = receiver.try_iter().flatten().collect();
    (code, String::from_utf8_lossy(&output).into_owned())
}

/// The broker token is low integrity, write-restricted to a SID the token
/// does not otherwise hold, without privileges, and the broker reports it
/// (`token+dacl+job`). Control: without it the check fails and the broker
/// reports `dacl+job`, which Core refuses.
#[test]
fn pf_27_s08_broker_runs_under_its_token_and_reports_it() {
    assert!(
        super::current_token_is_broker_token().is_err(),
        "this test process is not confined"
    );
    let confined = run_child("token", &[], Confinement::BrokerToken);
    eprintln!("pf27s08: broker token: {confined:?}");
    assert_eq!(confined["token"], "ok", "{confined:?}");
    assert_eq!(confined["containment"], "token+dacl+job", "{confined:?}");
    assert_eq!(confined["capability_sids"], "1", "{confined:?}");
    let control = run_child("token", &[], Confinement::SameToken);
    assert_ne!(control["token"], "ok", "{control:?}");
    assert_eq!(control["containment"], "dacl+job", "{control:?}");
    assert_eq!(control["capability_sids"], "0", "{control:?}");
}

/// The broker cannot create, modify, rename or delete a file the user can
/// write, in the user's temporary directory or in the low-integrity
/// `LocalLow` folder (writable at low integrity, so only the write
/// restriction stops it). Control: the PF-27-S07 broker can.
#[test]
fn pf_27_s08_broker_cannot_write_the_users_files() {
    let root = std::env::temp_dir().join(format!("pf27s08-files-{}", std::process::id()));
    let local_low = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .map(|home| home.join("AppData").join("LocalLow"))
        .filter(|dir| dir.is_dir())
        .map(|dir| dir.join(format!("pf27s08-files-{}", std::process::id())));
    let mut bases = vec![("temp", root.clone())];
    bases.extend(local_low.clone().map(|dir| ("locallow", dir)));
    for (role, confinement, expected) in [
        ("confined", Confinement::BrokerToken, "denied"),
        ("control", Confinement::SameToken, "ok"),
    ] {
        let dirs: Vec<PathBuf> = bases
            .iter()
            .map(|(label, base)| {
                let dir = base.join(role);
                prepare_files(&dir);
                std::path::PathBuf::from(format!("{label}={}", dir.display()))
            })
            .collect();
        let joined = std::env::join_paths(&dirs).expect("dirs");
        let report = run_child("files", &[(DIRS_ENV, joined)], confinement);
        eprintln!("pf27s08: files ({role}): {report:?}");
        assert_eq!(
            report.len(),
            bases.len() * FILE_OPERATIONS.len(),
            "{report:?}"
        );
        for (key, value) in &report {
            assert_eq!(value, expected, "{role} {key}: {report:?}");
        }
    }
    // The confined child left everything as it was.
    for (_, base) in &bases {
        let dir = base.join("confined");
        assert_eq!(read(&dir.join("existing.txt")), "original");
        assert!(dir.join("rename.txt").is_file());
        assert!(dir.join("delete.txt").is_file());
        assert!(!dir.join("new.txt").exists());
        assert!(!dir.join("subdir").exists());
    }
    for (_, base) in &bases {
        // The control made this file read-only.
        let existing = base.join("control").join("existing.txt");
        if let Ok(meta) = std::fs::metadata(&existing) {
            let mut permissions = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&existing, permissions);
        }
        let _ = std::fs::remove_dir_all(base);
    }
}

/// A process under the broker token still resolves names and connects out
/// (the broker's upstreams; TLS over it is the network-proxy suite and the
/// real-Windows gate). Skipped without network access (positive control).
#[test]
fn pf_27_s08_broker_still_reaches_the_network() {
    let control = run_child("network", &[], Confinement::SameToken);
    if control["connect"] != "ok" {
        eprintln!("pf27s08: no network for the control ({control:?}); skipped");
        return;
    }
    let confined = run_child("network", &[], Confinement::BrokerToken);
    eprintln!("pf27s08: network under the broker token: {confined:?}");
    assert_eq!(confined["resolve"], "ok", "{confined:?}");
    assert_eq!(confined["connect"], "ok", "{confined:?}");
}

/// The broker token cannot read Credential Manager. Control: the PF-27-S07
/// broker reads the same synthetic credential. Skipped where this session
/// has no Credential Manager (a service logon without a profile).
#[test]
fn pf_27_s08_broker_cannot_read_credential_manager() {
    let target = format!("codex-pf27s08-probe-{}", std::process::id());
    if let Err(err) = credential::write(&target, b"pf27s08-synthetic-credential") {
        eprintln!("pf27s08: no Credential Manager in this session ({err}); skipped");
        return;
    }
    let env = [(CREDENTIAL_ENV, OsString::from(&target))];
    let control = run_child("credential", &env, Confinement::SameToken);
    let confined = run_child("credential", &env, Confinement::BrokerToken);
    let _ = credential::delete(&target);
    eprintln!("pf27s08: credential control={control:?} confined={confined:?}");
    assert_eq!(control["read"], "ok", "{control:?}");
    assert_ne!(confined["read"], "ok", "{confined:?}");
}

/// The broker cannot ask WMI (`Win32_Process.Create`), Task Scheduler
/// (`Schedule.Service`) or an out-of-process COM server (`MMC20.Application`)
/// to run something. PowerShell runs the same script under the broker token
/// (with a default DACL it can work with) and, as the positive control, under
/// this process's own token; every mechanism the control can use must be
/// denied, and the control must reach at least WMI and Task Scheduler.
#[test]
fn pf_27_s08_broker_cannot_start_work_through_wmi_tasks_or_com() {
    let control = run_powershell_probe(/*confined*/ false);
    eprintln!("pf27s08: WMI/tasks/COM control: {control:?}");
    let confined = run_powershell_probe(/*confined*/ true);
    eprintln!("pf27s08: WMI/tasks/COM under the broker token: {confined:?}");
    assert_eq!(
        control.get("wmi").map(String::as_str),
        Some("ok"),
        "{control:?}"
    );
    assert_eq!(
        control.get("task").map(String::as_str),
        Some("ok"),
        "{control:?}"
    );
    for (name, outcome) in &control {
        if outcome == "ok" {
            assert_eq!(
                confined.get(name).map(String::as_str),
                Some("denied"),
                "{name}: {confined:?}"
            );
        }
    }
}

fn run_child(role: &str, env: &[(&str, OsString)], confinement: Confinement) -> Report {
    let program = std::env::current_exe().expect("test binary");
    let args = [CHILD_TEST, "--exact", "--nocapture", "--test-threads=1"].map(OsString::from);
    let mut vars: Vec<(OsString, OsString)> = std::env::vars_os()
        .filter(|(name, _)| {
            !name
                .to_str()
                .is_some_and(|name| name.to_ascii_uppercase().starts_with("CODEX_PF27S08_"))
        })
        .collect();
    vars.push((ROLE_ENV.into(), role.into()));
    vars.extend(
        env.iter()
            .map(|(name, value)| ((*name).into(), value.clone())),
    );
    let (mut child, mut stdout) =
        spawn_protected_with(&program, &args, &vars, confinement).expect("protected child");
    let mut output = String::new();
    let _ = stdout.read_to_string(&mut output);
    let status = child.wait();
    if !output.contains(REPORT_PREFIX) {
        panic!("{role} child ({confinement:?}) printed no report; exit {status:?}:\n{output}");
    }
    decode(&output)
}

fn decode(output: &str) -> Report {
    let line = output
        .lines()
        .find_map(|line| line.find(REPORT_PREFIX).map(|start| &line[start..]))
        .and_then(|line| line.strip_prefix(REPORT_PREFIX))
        .unwrap_or_else(|| panic!("child printed no report:\n{output}"));
    line.trim()
        .split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn token_report() -> Report {
    let mut report = Report::new();
    let token = match super::current_token_is_broker_token() {
        Ok(()) => "ok".to_string(),
        Err(err) => format!("no:{}", err.to_string().replace([',', '='], " ")),
    };
    report.insert("token".to_string(), token);
    let capabilities = super::current_capability_sid_strings()
        .map(|sids| sids.len().to_string())
        .unwrap_or_else(|_| "error".to_string());
    report.insert("capability_sids".to_string(), capabilities);
    let dir = std::env::temp_dir();
    let containment = crate::contain_credential_broker(&dir).mechanism;
    report.insert("containment".to_string(), containment);
    report
}

const FILE_OPERATIONS: &[&str] = &[
    "create",
    "overwrite",
    "append",
    "attributes",
    "rename",
    "delete",
    "mkdir",
];

fn prepare_files(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).expect("probe dir");
    std::fs::write(dir.join("existing.txt"), "original").expect("existing");
    std::fs::write(dir.join("rename.txt"), "x").expect("rename");
    std::fs::write(dir.join("delete.txt"), "x").expect("delete");
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

fn outcome(result: std::io::Result<()>) -> String {
    match result {
        Ok(()) => "ok".to_string(),
        Err(err) if err.raw_os_error() == Some(ERROR_ACCESS_DENIED) => "denied".to_string(),
        Err(err) => format!("error:{}", err.raw_os_error().unwrap_or(-1)),
    }
}

fn file_report() -> Report {
    use std::io::Write as _;
    let mut report = Report::new();
    let dirs = std::env::var_os(DIRS_ENV).expect("dirs");
    for entry in std::env::split_paths(&dirs) {
        let entry = entry.to_string_lossy().into_owned();
        let (label, dir) = entry.split_once('=').expect("label=dir");
        let dir = Path::new(dir);
        let existing = dir.join("existing.txt");
        let results = [
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join("new.txt"))
                .map(drop),
            std::fs::OpenOptions::new()
                .write(true)
                .open(&existing)
                .and_then(|mut file| file.write_all(b"changed")),
            std::fs::OpenOptions::new()
                .append(true)
                .open(&existing)
                .and_then(|mut file| file.write_all(b"appended")),
            std::fs::metadata(&existing).and_then(|meta| {
                let mut permissions = meta.permissions();
                permissions.set_readonly(true);
                std::fs::set_permissions(&existing, permissions)
            }),
            std::fs::rename(dir.join("rename.txt"), dir.join("renamed.txt")),
            std::fs::remove_file(dir.join("delete.txt")),
            std::fs::create_dir(dir.join("subdir")),
        ];
        for (name, result) in FILE_OPERATIONS.iter().zip(results) {
            report.insert(format!("{label}_{name}"), outcome(result));
        }
    }
    report
}

fn network_report() -> Report {
    use std::net::ToSocketAddrs as _;
    let mut report = Report::new();
    let addresses: Vec<std::net::SocketAddr> = match ("github.com", 443).to_socket_addrs() {
        Ok(addresses) => addresses.collect(),
        Err(_) => Vec::new(),
    };
    report.insert(
        "resolve".to_string(),
        if addresses.is_empty() { "failed" } else { "ok" }.to_string(),
    );
    let connected = addresses.iter().any(|address| {
        std::net::TcpStream::connect_timeout(address, std::time::Duration::from_secs(10)).is_ok()
    });
    report.insert(
        "connect".to_string(),
        if connected { "ok" } else { "failed" }.to_string(),
    );
    report
}

fn credential_report() -> Report {
    let target = std::env::var(CREDENTIAL_ENV).expect("credential target");
    let read = match credential::read(&target) {
        Ok(true) => "ok".to_string(),
        Ok(false) => "wrong_value".to_string(),
        Err(code) => format!("error:{code}"),
    };
    Report::from([("read".to_string(), read)])
}

/// Credential Manager calls (declared here: the crate's `windows-sys`
/// features do not include them).
mod credential {
    use std::ffi::c_void;

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    const EXPECTED: &[u8] = b"pf27s08-synthetic-credential";

    #[repr(C)]
    struct Credential {
        flags: u32,
        kind: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: [u32; 2],
        blob_size: u32,
        blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn CredWriteW(credential: *const Credential, flags: u32) -> i32;
        fn CredReadW(target: *const u16, kind: u32, flags: u32, out: *mut *mut Credential) -> i32;
        fn CredDeleteW(target: *const u16, kind: u32, flags: u32) -> i32;
        fn CredFree(buffer: *mut c_void);
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain([0]).collect()
    }

    pub(super) fn write(target: &str, value: &[u8]) -> std::io::Result<()> {
        let mut name = wide(target);
        let mut user = wide("pf27s08");
        let mut blob = value.to_vec();
        let credential = Credential {
            flags: 0,
            kind: CRED_TYPE_GENERIC,
            target_name: name.as_mut_ptr(),
            comment: std::ptr::null_mut(),
            last_written: [0, 0],
            blob_size: blob.len() as u32,
            blob: blob.as_mut_ptr(),
            persist: CRED_PERSIST_LOCAL_MACHINE,
            attribute_count: 0,
            attributes: std::ptr::null_mut(),
            target_alias: std::ptr::null_mut(),
            user_name: user.as_mut_ptr(),
        };
        // SAFETY: every pointer refers to a live buffer above.
        if unsafe { CredWriteW(&credential, 0) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    /// Ok(true) if the credential reads back with the synthetic value;
    /// Err(Win32 error) if the read fails.
    pub(super) fn read(target: &str) -> Result<bool, i32> {
        let name = wide(target);
        let mut out: *mut Credential = std::ptr::null_mut();
        // SAFETY: `name` is NUL-terminated; `out` is freed below.
        if unsafe { CredReadW(name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut out) } == 0 {
            return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(-1));
        }
        // SAFETY: CredReadW returned a credential with `blob_size` bytes.
        let matches = unsafe {
            std::slice::from_raw_parts((*out).blob, (*out).blob_size as usize) == EXPECTED
        };
        // SAFETY: allocated by CredReadW.
        unsafe { CredFree(out.cast()) };
        Ok(matches)
    }

    pub(super) fn delete(target: &str) -> std::io::Result<()> {
        let name = wide(target);
        // SAFETY: `name` is NUL-terminated.
        if unsafe { CredDeleteW(name.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

/// The PowerShell probe: one `name=outcome` per mechanism, where outcome is
/// `ok`, `denied` (access denied in any of its forms) or `error:<detail>`.
const POWERSHELL_PROBE: &str = r#"
$ErrorActionPreference = 'Stop'
$task = "codex-pf27s08-$PID"
function Probe([string]$Name, [scriptblock]$Body) {
  try { & $Body; return "$Name=ok" }
  catch {
    $e = $_.Exception
    while ($e.InnerException) { $e = $e.InnerException }
    $hr = '{0:x8}' -f $e.HResult
    $denied = ($e -is [System.UnauthorizedAccessException]) -or
      ($hr -in '80070005', '80041003') -or ($e.Message -match 'denied|ReturnValue 2$')
    if ($denied) { return "$Name=denied" }
    return "$Name=error:$hr"
  }
}
$results = @()
$results += Probe 'wmi' {
  $r = ([wmiclass]'root\cimv2:Win32_Process').Create("$env:SystemRoot\System32\cmd.exe /d /c exit 0")
  if ($r.ReturnValue -ne 0) { throw "ReturnValue $($r.ReturnValue)" }
}
$results += Probe 'task' {
  $service = New-Object -ComObject Schedule.Service
  $service.Connect()
  $folder = $service.GetFolder('\')
  $definition = $service.NewTask(0)
  $action = $definition.Actions.Create(0)
  $action.Path = "$env:SystemRoot\System32\cmd.exe"
  $action.Arguments = '/d /c exit 0'
  try {
    $null = $folder.RegisterTaskDefinition($task, $definition, 6, $null, $null, 3)
  } finally {
    try { $folder.DeleteTask($task, 0) } catch {}
  }
}
$results += Probe 'com' {
  $mmc = [Activator]::CreateInstance([Type]::GetTypeFromProgID('MMC20.Application'))
  $mmc.Quit()
}
"`npf27s08-report:" + ($results -join ',')
"#;

fn run_powershell_probe(confined: bool) -> Report {
    use crate::windows_broker_token::BrokerDefaultDacl;
    use crate::windows_broker_token::create_broker_token;
    use std::collections::HashMap;
    use std::os::windows::io::IntoRawHandle as _;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Threading::TerminateProcess;
    use windows_sys::Win32::System::Threading::WaitForSingleObject;

    let system_root = std::env::var("SystemRoot").expect("SystemRoot");
    let powershell = format!(r"{system_root}\System32\WindowsPowerShell\v1.0\powershell.exe");
    let script: Vec<u8> = POWERSHELL_PROBE
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let argv = vec![
        powershell,
        "-NoLogo".to_string(),
        "-NoProfile".to_string(),
        "-NonInteractive".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-EncodedCommand".to_string(),
        base64(&script),
    ];
    let env: HashMap<String, String> = std::env::vars()
        .filter(|(name, _)| !name.to_ascii_uppercase().starts_with("CODEX_PF27S08_"))
        .collect();
    let token: HANDLE = if confined {
        create_broker_token(BrokerDefaultDacl::OwnedByCapability)
            .expect("broker token")
            .into_raw_handle() as HANDLE
    } else {
        // SAFETY: returns an owned handle to this process's token.
        unsafe { codex_windows_sandbox::get_current_token_for_restriction() }.expect("token")
    };
    let cwd = std::env::temp_dir();
    let spawned = codex_windows_sandbox::spawn_process_with_pipes(
        token,
        &argv,
        &cwd,
        &env,
        codex_windows_sandbox::StdinMode::Closed,
        codex_windows_sandbox::StderrMode::MergeStdout,
        codex_windows_sandbox::ConsoleMode::NoWindow,
        /*use_private_desktop*/ false,
        /*logs_base_dir*/ None,
    )
    .expect("spawn PowerShell");
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    let reader = codex_windows_sandbox::read_handle_loop(spawned.stdout_read, move |chunk| {
        let _ = sender.send(chunk.to_vec());
    });
    // SAFETY: the handles stay valid until closed below.
    unsafe {
        const WAIT_TIMEOUT: u32 = 0x102;
        if WaitForSingleObject(spawned.process.hProcess, 180_000) == WAIT_TIMEOUT {
            TerminateProcess(spawned.process.hProcess, 1);
        }
        CloseHandle(spawned.process.hThread);
        CloseHandle(spawned.process.hProcess);
        CloseHandle(token);
    }
    let _ = reader.join();
    let output: Vec<u8> = receiver.try_iter().flatten().collect();
    decode(&String::from_utf8_lossy(&output))
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::new();
    for chunk in bytes.chunks(3) {
        let value = chunk.iter().enumerate().fold(0_u32, |acc, (i, byte)| {
            acc | u32::from(*byte) << (16 - 8 * i)
        });
        for i in 0..4 {
            if i <= chunk.len() {
                text.push(char::from(ALPHABET[(value >> (18 - 6 * i)) as usize & 63]));
            } else {
                text.push('=');
            }
        }
    }
    text
}
