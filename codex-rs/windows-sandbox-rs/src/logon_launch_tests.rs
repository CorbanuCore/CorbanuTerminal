//! #307: the logon launcher's pipe ends must never reach a process Core
//! starts on another thread while the launcher is being started.

// The race counts are the evidence of these measured runs (`--nocapture`).
#![allow(clippy::print_stderr)]

use super::spawn_launcher;
use pretty_assertions::assert_eq;
use std::os::windows::io::AsRawHandle as _;
use std::os::windows::process::CommandExt as _;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
use windows_sys::Win32::Foundation::DuplicateHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Storage::FileSystem::FILE_TYPE_PIPE;
use windows_sys::Win32::Storage::FileSystem::GetFileType;
use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const ROLE_ENV: &str = "CODEX_SEC_WIN_307_ROLE";
const TEST_NAME: &str =
    "logon_launch::tests::sec_win_307_launcher_pipes_never_reach_other_children";
const LAUNCHES: usize = 300;
const RACED: &str = "sec-win-307: raced";
/// Handle values are multiples of 4; a test process stays far below this.
const MAX_HANDLE_VALUE: usize = 0x1_0000;

/// Starts launchers on one thread while another starts ordinary children
/// (`std::process::Command`, which always inherits handles) and checks each
/// child's handle table for pipes. This test's own process opens no other
/// pipes, so any pipe a child holds beyond those it inherits before the
/// race (this process's own inheritable stdio) came from a launcher start.
#[test]
fn sec_win_307_launcher_pipes_never_reach_other_children() {
    if std::env::var_os(ROLE_ENV).is_some() {
        race();
        return;
    }
    // Alone in a fresh process, so no other test's pipes are in the table.
    let output = Command::new(std::env::current_exe().expect("test binary"))
        .args([TEST_NAME, "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, "1")
        .output()
        .expect("rerun this test alone");
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprint!("{stderr}");
    assert!(output.status.success(), "{}", output.status);
    // A rename would make the rerun match nothing and pass.
    assert!(stderr.contains(RACED), "the race did not run");
}

fn race() {
    let system32 =
        PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot")).join("System32");
    // Any program does: the test only starts the launcher, never talks to it.
    let launcher = system32.join("whoami.exe");
    let child_program = system32.join("cmd.exe");
    let start_child = || {
        Command::new(&child_program)
            .creation_flags(CREATE_SUSPENDED)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start a child")
    };
    let mut first = start_child();
    let baseline = pipe_handles(first.as_raw_handle() as HANDLE);
    let _ = first.kill();
    let _ = first.wait();
    eprintln!("sec-win-307: pipes every child inherits: {baseline:x?}");
    let done = AtomicBool::new(false);
    let (children, leaks) = std::thread::scope(|scope| {
        let watcher = scope.spawn(|| {
            let (mut children, mut leaks) = (0_usize, 0_usize);
            while !done.load(Ordering::SeqCst) {
                let mut child = start_child();
                let mut pipes = pipe_handles(child.as_raw_handle() as HANDLE);
                pipes.retain(|pipe| !baseline.contains(pipe));
                if !pipes.is_empty() {
                    leaks += 1;
                    eprintln!("sec-win-307: child {} holds pipes {pipes:x?}", child.id());
                }
                children += 1;
                let _ = child.kill();
                let _ = child.wait();
            }
            (children, leaks)
        });
        for _ in 0..LAUNCHES {
            let (process, stdin, stdout) = spawn_launcher(&launcher).expect("start the launcher");
            drop((stdin, stdout));
            // SAFETY: `process.0` is the live launcher process.
            unsafe {
                if WaitForSingleObject(process.0, /*dwmilliseconds*/ 5_000) != 0 {
                    TerminateProcess(process.0, 1);
                }
            }
        }
        done.store(true, Ordering::SeqCst);
        watcher.join().expect("watcher")
    });
    eprintln!(
        "sec-win-307: {LAUNCHES} launcher starts, {children} children started meanwhile, {leaks} held a launcher pipe"
    );
    assert!(children > 0, "no child started during the launches");
    assert_eq!(leaks, 0, "a child inherited a launcher pipe end");
    eprintln!("{RACED}");
}

/// The pipe handles in `process`'s table (a suspended child of this process).
fn pipe_handles(process: HANDLE) -> Vec<usize> {
    let mut pipes = Vec::new();
    for value in (4..MAX_HANDLE_VALUE).step_by(4) {
        let mut copy: HANDLE = 0;
        // SAFETY: `process` is our child, opened with full access; a value
        // that is not a handle there just fails.
        let ok = unsafe {
            DuplicateHandle(
                process,
                value as HANDLE,
                GetCurrentProcess(),
                &mut copy,
                /*dwdesiredaccess*/ 0,
                /*binherithandle*/ 0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if ok == 0 {
            continue;
        }
        // SAFETY: `copy` is a live handle owned here.
        unsafe {
            if GetFileType(copy) == FILE_TYPE_PIPE {
                pipes.push(value);
            }
            CloseHandle(copy);
        }
    }
    pipes
}
