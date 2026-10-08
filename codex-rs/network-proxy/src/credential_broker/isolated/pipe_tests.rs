//! PF-27-S06 measured probes for the broker's named pipes.
//!
//! Child processes are this test binary re-executed and filtered to the
//! entry test below: as a same-user process, under the restricted token the
//! unelevated Windows sandbox gives agent commands, or as a handle scanner.

use super::PipeListener;
use super::client_pid;
use super::connect_control;
use super::connect_data;
use super::pipe_dacl_sddl;
use super::pipe_names;
use super::valid_pipe_name;
use codex_windows_sandbox::ConsoleMode;
use codex_windows_sandbox::StderrMode;
use codex_windows_sandbox::StdinMode;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::io::Read as _;
use std::io::Write as _;
use std::os::windows::io::AsRawHandle;
use std::process::Command;
use std::process::Stdio;
use tokio::io::AsyncWriteExt as _;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::GetHandleInformation;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HANDLE_FLAG_INHERIT;
use windows_sys::Win32::Storage::FileSystem::FILE_TYPE_PIPE;
use windows_sys::Win32::Storage::FileSystem::FileNameInfo;
use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandleEx;
use windows_sys::Win32::Storage::FileSystem::GetFileType;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const ROLE_ENV: &str = "CODEX_PF27S06_PIPE_ROLE";
const NAME_ENV: &str = "CODEX_PF27S06_PIPE_NAME";
const CHILD_TEST: &str = "credential_broker::isolated::pipe::tests::pf_27_s06_pipe_child_entry";
const REPORT_PREFIX: &str = "pf27s06-pipe:";
const GREETING: &[u8] = b"broker-hello\n";

#[test]
fn pf_27_s06_pipe_child_entry() {
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("connect") => child_connect(),
        Ok("scan") => child_scan(),
        _ => {}
    }
}

#[test]
fn pf_27_s06_pipe_names_and_dacl() {
    let (control, data) = pipe_names();
    assert!(valid_pipe_name(&control, /*control*/ true));
    assert!(valid_pipe_name(&data, /*control*/ false));
    assert!(!valid_pipe_name(&control, /*control*/ false));
    assert!(!valid_pipe_name(
        r"\\.\pipe\corbanu-cbk-zz-c",
        /*control*/ true
    ));
    assert!(!valid_pipe_name(r"\\.\pipe\other-c", /*control*/ true));
    assert_ne!(pipe_names().0, control);
    assert_eq!(
        pipe_dacl_sddl("S-1-5-21-1-2-3-1001"),
        "D:P(A;;GA;;;S-1-5-21-1-2-3-1001)"
    );
}

/// A second process creating the same name as the first instance fails:
/// the broker's pipe cannot be squatted or joined.
#[tokio::test]
async fn pf_27_s06_pipe_name_cannot_be_claimed_twice() {
    let (control, _) = pipe_names();
    let _listener = PipeListener::bind(&control).expect("bind");
    assert!(PipeListener::bind(&control).is_err());
}

/// The broker serves only the expected client process: another process of
/// the same user opens the pipe (the DACL grants the user) but is dropped
/// before anything is sent; the expected process is served.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_27_s06_broker_pipe_serves_only_the_expected_client() {
    let (control, _) = pipe_names();
    let _server = serve_greeting(&control, std::process::id());
    let report = tokio::task::spawn_blocking({
        let control = control.clone();
        move || run_same_user_child("connect", &control)
    })
    .await
    .expect("child");
    assert_eq!(report, "open=granted,served=no");

    let mut file =
        tokio::task::spawn_blocking(move || connect_control(&control, std::process::id()))
            .await
            .expect("connect")
            .expect("expected client is accepted");
    let mut greeting = vec![0_u8; GREETING.len()];
    file.read_exact(&mut greeting).expect("served");
    assert_eq!(greeting, GREETING);
}

/// A command under the unelevated sandbox's restricted token cannot open
/// the broker's pipe at all.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_27_s06_restricted_token_cannot_open_broker_pipe() {
    let (control, _) = pipe_names();
    let _server = serve_greeting(&control, std::process::id());
    let report = tokio::task::spawn_blocking(move || run_restricted_child("connect", &control))
        .await
        .expect("child");
    assert_eq!(report, "open=denied");
}

/// Core talks only to the broker it spawned: a pipe served by another
/// process, or a name that is not a broker pipe, is refused.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_27_s06_client_checks_the_server_process() {
    let (control, data) = pipe_names();
    let own_pid = std::process::id();
    let _control_server = serve_greeting(&control, own_pid);
    let _data_server = serve_greeting(&data, own_pid);
    let wrong_pid = own_pid.wrapping_add(4);

    let blocking_control = control.clone();
    let (wrong, right, other_name) = tokio::task::spawn_blocking(move || {
        (
            connect_control(&blocking_control, wrong_pid).is_some(),
            connect_control(&blocking_control, own_pid).is_some(),
            connect_control(r"\\.\pipe\not-a-broker-c", own_pid).is_some(),
        )
    })
    .await
    .expect("connect");
    assert_eq!((wrong, right, other_name), (false, true, false));

    let wrong = connect_data(&data, wrong_pid).await;
    assert_eq!(
        wrong.err().map(|error| error.kind()),
        Some(std::io::ErrorKind::PermissionDenied)
    );
    assert!(connect_data(&data, own_pid).await.is_ok());
    assert!(connect_data(&control, own_pid).await.is_err());
}

/// No broker pipe handle is inheritable, measured in a child spawned the way
/// Rust spawns every process (inheriting all inheritable handles). A
/// deliberately inheritable duplicate is the positive control.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pf_27_s06_pipe_handles_are_not_inheritable() {
    let (control, data) = pipe_names();
    let own_pid = std::process::id();
    let mut control_listener = PipeListener::bind(&control).expect("bind control");
    let mut data_listener = PipeListener::bind(&data).expect("bind data");
    let blocking_control = control.clone();
    let control_client =
        tokio::task::spawn_blocking(move || connect_control(&blocking_control, own_pid));
    let control_server = control_listener
        .accept(own_pid)
        .await
        .expect("accept control");
    let control_client = control_client.await.expect("join").expect("control client");
    let data_client = tokio::spawn({
        let data = data.clone();
        async move { connect_data(&data, own_pid).await }
    });
    let data_server = data_listener.accept(own_pid).await.expect("accept data");
    let data_client = data_client.await.expect("join").expect("data client");

    for handle in [
        control_listener.listening.as_raw_handle(),
        control_server.as_raw_handle(),
        control_client.as_raw_handle(),
        data_listener.listening.as_raw_handle(),
        data_server.as_raw_handle(),
        data_client.as_raw_handle(),
    ] {
        assert!(
            !inheritable(handle as HANDLE),
            "broker pipe handle is inheritable"
        );
    }
    assert_eq!(client_pid(&control_server), Some(own_pid));

    let nonce = control
        .trim_start_matches(super::PIPE_PREFIX)
        .trim_end_matches("-c")
        .to_string();
    let duplicate = inheritable_duplicate(control_client.as_raw_handle() as HANDLE);
    let with_duplicate = run_scan_child(&nonce);
    // SAFETY: created above and closed once.
    unsafe { CloseHandle(duplicate) };
    let without = run_scan_child(&nonce);
    assert_eq!((with_duplicate, without), (1, 0));
}

/// Accepts clients from `expected_pid` and greets each one.
fn serve_greeting(name: &str, expected_pid: u32) -> tokio::task::JoinHandle<()> {
    let mut listener = PipeListener::bind(name).expect("bind");
    tokio::spawn(async move {
        while let Ok(mut connection) = listener.accept(expected_pid).await {
            let _ = connection.write_all(GREETING).await;
            // Keep the connection open for the client to read.
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                drop(connection);
            });
        }
    })
}

fn child_connect() {
    let name = std::env::var(NAME_ENV).expect("pipe name");
    let report = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&name)
    {
        Ok(mut file) => {
            let mut byte = [0_u8; 1];
            let served = matches!(file.read(&mut byte), Ok(1));
            format!("open=granted,served={}", if served { "yes" } else { "no" })
        }
        Err(error) if error.raw_os_error() == Some(5) => "open=denied".to_string(),
        Err(error) => format!("open=failed:{}", error.raw_os_error().unwrap_or(-1)),
    };
    report_and_exit(&report);
}

/// Counts handles in this process that refer to a pipe whose name contains
/// the nonce. Only inherited handles can match: the child never opens it.
fn child_scan() {
    let nonce = std::env::var(NAME_ENV).expect("nonce");
    let mut found = 0;
    for value in (4..=0x0001_0000_isize).step_by(4) {
        let handle = value as HANDLE;
        // SAFETY: probing handle values; invalid ones fail harmlessly.
        if unsafe { GetFileType(handle) } != FILE_TYPE_PIPE {
            continue;
        }
        if pipe_name(handle).is_some_and(|name| name.contains(&nonce)) {
            found += 1;
        }
    }
    report_and_exit(&found.to_string());
}

fn pipe_name(handle: HANDLE) -> Option<String> {
    let mut buffer = [0_u32; 256];
    // SAFETY: `buffer` is valid and aligned for FILE_NAME_INFO.
    let ok = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileNameInfo,
            buffer.as_mut_ptr().cast(),
            std::mem::size_of_val(&buffer) as u32,
        )
    };
    if ok == 0 {
        return None;
    }
    let bytes = (buffer[0] as usize).min((buffer.len() - 1) * 4);
    // SAFETY: FILE_NAME_INFO's name follows the 4-byte length.
    let name =
        unsafe { std::slice::from_raw_parts(buffer.as_ptr().add(1).cast::<u16>(), bytes / 2) };
    Some(String::from_utf16_lossy(name))
}

fn report_and_exit(report: &str) -> ! {
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "{REPORT_PREFIX}{report}");
    let _ = stdout.flush();
    std::process::exit(0);
}

fn inheritable(handle: HANDLE) -> bool {
    let mut flags = 0_u32;
    // SAFETY: a valid handle and out pointer.
    let ok = unsafe { GetHandleInformation(handle, &mut flags) };
    assert_ne!(ok, 0, "GetHandleInformation failed");
    flags & HANDLE_FLAG_INHERIT != 0
}

fn inheritable_duplicate(handle: HANDLE) -> HANDLE {
    let mut duplicate: HANDLE = 0;
    // SAFETY: duplicates a valid handle within this process.
    let ok = unsafe {
        DuplicateHandle(
            GetCurrentProcess(),
            handle,
            GetCurrentProcess(),
            &mut duplicate,
            0,
            /*bInheritHandle*/ 1,
            DUPLICATE_SAME_ACCESS,
        )
    };
    assert_ne!(ok, 0, "DuplicateHandle failed");
    duplicate
}

fn child_args() -> Vec<String> {
    vec![
        CHILD_TEST.to_string(),
        "--exact".to_string(),
        "--nocapture".to_string(),
        "--test-threads=1".to_string(),
    ]
}

fn parse_report(output: &[u8]) -> String {
    let output = String::from_utf8_lossy(output);
    output
        .lines()
        .find_map(|line| line.trim().strip_prefix(REPORT_PREFIX))
        .unwrap_or_else(|| panic!("child printed no report:\n{output}"))
        .to_string()
}

fn run_same_user_child(role: &str, name: &str) -> String {
    let output = Command::new(std::env::current_exe().expect("test binary"))
        .args(child_args())
        .env(ROLE_ENV, role)
        .env(NAME_ENV, name)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .expect("run child");
    parse_report(&output.stdout)
}

fn run_scan_child(nonce: &str) -> usize {
    run_same_user_child("scan", nonce)
        .parse()
        .expect("handle count")
}

/// Runs a child under the restricted token the unelevated Windows sandbox
/// creates for agent commands.
fn run_restricted_child(role: &str, name: &str) -> String {
    let exe = std::env::current_exe().expect("test binary");
    let mut argv = vec![exe.to_string_lossy().into_owned()];
    argv.extend(child_args());
    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.insert(ROLE_ENV.to_string(), role.to_string());
    env.insert(NAME_ENV.to_string(), name.to_string());
    let cwd = std::env::current_dir().expect("cwd");
    let capability = codex_windows_sandbox::LocalSid::from_string(
        "S-1-5-21-2718281828-3141592653-1618033988-1002",
    )
    .expect("capability SID");
    // SAFETY: both tokens are closed below.
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
    .expect("spawn restricted child");
    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();
    // Not joined: a process spawned concurrently by another test can inherit
    // the pipe's write end while it is briefly inheritable, so EOF may come
    // late. The report line is all that is needed.
    let _reader = codex_windows_sandbox::read_handle_loop(spawned.stdout_read, move |chunk| {
        let _ = sender.send(chunk.to_vec());
    });
    // SAFETY: the handles stay valid until closed here.
    unsafe {
        WaitForSingleObject(spawned.process.hProcess, 60_000);
        CloseHandle(spawned.process.hThread);
        CloseHandle(spawned.process.hProcess);
        CloseHandle(token);
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut output = Vec::new();
    loop {
        let text = String::from_utf8_lossy(&output);
        let complete = text
            .find(REPORT_PREFIX)
            .is_some_and(|start| text[start..].contains('\n'));
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if complete || remaining.is_zero() {
            break;
        }
        match receiver.recv_timeout(remaining) {
            Ok(chunk) => output.extend(chunk),
            Err(_) => break,
        }
    }
    parse_report(&output)
}
