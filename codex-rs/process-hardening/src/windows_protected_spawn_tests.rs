// The race counts are the evidence of the measured runs (`--nocapture`).
#![allow(clippy::print_stderr)]

use super::command_line;
use super::environment_block;
use pretty_assertions::assert_eq;
use std::ffi::OsString;
use std::path::Path;

fn text(wide: &[u16]) -> String {
    String::from_utf16_lossy(wide)
}

#[test]
fn pf_27_s07_command_line_quotes_like_the_msvc_runtime() {
    let args: Vec<OsString> = [
        "plain",
        "",
        "two words",
        r#"say "hi""#,
        r"trailing\",
        r"a\\b c",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    assert_eq!(
        text(&command_line(Path::new(r"C:\Program Files\corbanu.exe"), &args).expect("line")),
        concat!(
            r#""C:\Program Files\corbanu.exe" plain "" "two words" "say \"hi\"" trailing\ "#,
            r#""a\\b c""#,
            "\0"
        )
    );
}

#[test]
fn pf_27_s07_environment_block_is_sorted_and_terminated() {
    let env = [("b", "2"), ("A", "1"), ("c", "")]
        .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    assert_eq!(
        text(&environment_block(&env).expect("block")),
        "A=1\0b=2\0c=\0\0"
    );
    assert_eq!(text(&environment_block(&[]).expect("empty")), "\0\0");
    let bad = [(OsString::from("A=B"), OsString::from("1"))];
    assert!(environment_block(&bad).is_err());
    // cmd.exe's per-drive variables, and a later duplicate in another case.
    let env = [("=C:", r"C:\x"), ("Path", "1"), ("PATH", "2")]
        .map(|(name, value)| (OsString::from(name), OsString::from(value)));
    assert_eq!(
        text(&environment_block(&env).expect("block")),
        "=C:=C:\\x\0PATH=2\0\0"
    );
}

#[test]
fn pf_27_s07_command_line_refuses_nul() {
    assert!(command_line(Path::new("a.exe"), &[OsString::from("x\0y")]).is_err());
}

const ROLE_ENV: &str = "CODEX_SEC_WIN_320_ROLE";
const RACE_TEST: &str =
    "windows_protected_spawn::tests::sec_win_320_protected_stdout_never_reaches_other_children";
const SPAWNS: usize = 300;
const RACED: &str = "sec-win-320: raced";
/// Handle values are multiples of 4; a test process stays far below this.
const MAX_HANDLE_VALUE: usize = 0x1_0000;

fn system32() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot")).join("System32")
}

/// #320: starts protected processes on one thread while another starts
/// ordinary children (`std::process::Command`, which always inherits
/// handles) and checks each child's handle table for pipes. This test's own
/// process opens no other pipes, so any pipe a child holds beyond those it
/// inherits before the race (this process's own inheritable stdio) came from
/// a protected start.
#[test]
fn sec_win_320_protected_stdout_never_reaches_other_children() {
    if std::env::var_os(ROLE_ENV).is_some() {
        race();
        return;
    }
    // Alone in a fresh process, so no other test's pipes are in the table.
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([RACE_TEST, "--exact", "--nocapture", "--test-threads=1"])
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
    use std::os::windows::io::AsRawHandle as _;
    use std::os::windows::process::CommandExt as _;
    use std::process::Stdio;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;

    let protected_program = system32().join("whoami.exe");
    let child_program = system32().join("cmd.exe");
    let start_child = || {
        std::process::Command::new(&child_program)
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
    eprintln!("sec-win-320: pipes every child inherits: {baseline:x?}");
    let env: Vec<(OsString, OsString)> = std::env::vars_os().collect();
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
                    eprintln!("sec-win-320: child {} holds pipes {pipes:x?}", child.id());
                }
                children += 1;
                let _ = child.kill();
                let _ = child.wait();
            }
            (children, leaks)
        });
        for _ in 0..SPAWNS {
            let (mut child, stdout) =
                crate::spawn_protected(&protected_program, &[], &env).expect("protected spawn");
            drop(stdout);
            let _ = child.kill();
            let _ = child.wait();
        }
        done.store(true, Ordering::SeqCst);
        watcher.join().expect("watcher")
    });
    eprintln!(
        "sec-win-320: {SPAWNS} protected starts, {children} children started meanwhile, {leaks} held a protected stdout pipe"
    );
    assert!(children > 0, "no child started during the protected starts");
    assert_eq!(
        leaks, 0,
        "a child inherited a protected child's stdout pipe"
    );
    eprintln!("{RACED}");
}

/// The pipe handles in `process`'s table (a suspended child of this process).
fn pipe_handles(process: windows_sys::Win32::Foundation::HANDLE) -> Vec<usize> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::DUPLICATE_SAME_ACCESS;
    use windows_sys::Win32::Foundation::DuplicateHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Storage::FileSystem::FILE_TYPE_PIPE;
    use windows_sys::Win32::Storage::FileSystem::GetFileType;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

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
