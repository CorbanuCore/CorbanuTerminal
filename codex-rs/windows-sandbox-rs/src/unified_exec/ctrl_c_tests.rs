//! #1842: processed Ctrl-C, application handlers and raw input keep their
//! Windows meanings even when the host inherited the ignore-Ctrl-C flag.

use super::*;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use windows_sys::Win32::System::Console::CTRL_C_EVENT;
use windows_sys::Win32::System::Console::ENABLE_ECHO_INPUT;
use windows_sys::Win32::System::Console::ENABLE_LINE_INPUT;
use windows_sys::Win32::System::Console::ENABLE_PROCESSED_INPUT;
use windows_sys::Win32::System::Console::GetConsoleMode;
use windows_sys::Win32::System::Console::GetStdHandle;
use windows_sys::Win32::System::Console::STD_INPUT_HANDLE;
use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
use windows_sys::Win32::System::Console::SetConsoleMode;

const PROBE: &str = "CODEX_TEST_1842_COMMAND";
const ENTRY: &str = "unified_exec::tests::ctrl_c::ctrl_c_command_probe";
const SKIP_ARGS: [&str; 4] = [
    "space and trailing slash\\",
    "embedded \"quote\"",
    "",
    "日本語",
];
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Only the disposable subprocess changes its process-wide console state.
pub(super) fn ignored_ctrl_c_host(test: &str) -> bool {
    const HOST: &str = "CODEX_TEST_1842_IGNORE_CTRL_C_HOST";
    if std::env::var_os(HOST).is_some() {
        assert!(unsafe { SetConsoleCtrlHandler(None, 1) } != 0);
        return true;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(HOST, "1")
        .output()
        .expect("run isolated Ctrl-C host");
    assert!(
        output.status.success(),
        "host failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed"));
    false
}

unsafe extern "system" fn handle_ctrl_c(event: u32) -> i32 {
    if event == CTRL_C_EVENT {
        INTERRUPTED.store(true, Ordering::SeqCst);
        1
    } else {
        0
    }
}

#[test]
fn ctrl_c_command_probe() {
    let Ok(mode) = std::env::var(PROBE) else {
        return;
    };
    let expected: Vec<&str> = SKIP_ARGS.iter().flat_map(|arg| ["--skip", *arg]).collect();
    let args: Vec<String> = std::env::args().collect();
    pretty_assertions::assert_eq!(&args[args.len() - expected.len()..], expected);
    // This is a fresh sandboxed child, never the ordinary test host.
    let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if mode == "raw" {
        let mut flags = 0;
        assert!(unsafe { GetConsoleMode(input, &mut flags) } != 0);
        assert!(
            unsafe {
                SetConsoleMode(
                    input,
                    flags & !(ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT),
                )
            } != 0
        );
    } else {
        assert!(unsafe { SetConsoleCtrlHandler(Some(handle_ctrl_c), 1) } != 0);
    }
    println!("1842-ready");
    std::io::stdout().flush().expect("flush readiness");
    if mode == "raw" {
        let mut byte = [0u8];
        let mut read = 0;
        assert!(
            unsafe {
                windows_sys::Win32::Storage::FileSystem::ReadFile(
                    input,
                    byte.as_mut_ptr(),
                    1,
                    &mut read,
                    std::ptr::null_mut(),
                )
            } != 0
        );
        pretty_assertions::assert_eq!((byte, read), ([3], 1));
        std::process::exit(42);
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while !INTERRUPTED.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        INTERRUPTED.load(Ordering::SeqCst),
        "handler received no Ctrl-C"
    );
    std::process::exit(43);
}

fn run_input_case(mode: &str, code: i32) {
    let _guard = legacy_process_test_guard();
    let root = user_owned_test_root();
    let home = sandbox_home("1842-input");
    let mut env = with_session_temp(home.path(), std::env::vars().collect());
    env.insert(PROBE.to_string(), mode.to_string());
    let mut argv = vec![
        std::env::current_exe()
            .expect("test binary")
            .display()
            .to_string(),
        "--exact".to_string(),
        ENTRY.to_string(),
        "--nocapture".to_string(),
    ];
    argv.extend(
        SKIP_ARGS
            .iter()
            .flat_map(|arg| ["--skip".to_string(), (*arg).to_string()]),
    );
    current_thread_runtime().block_on(async {
        let spawned = spawn_windows_sandbox_session_legacy(
            &PermissionProfile::workspace_write(),
            workspace_roots_for(root.path()).as_slice(),
            home.path(),
            argv,
            root.path(),
            env,
            /*timeout_ms*/ None,
            &[],
            &[],
            /*tty*/ true,
            /*stdin_open*/ true,
            /*use_private_desktop*/ true,
        )
        .await
        .expect("spawn input probe");
        let codex_utils_pty::SpawnedProcess {
            session,
            mut stdout_rx,
            exit_rx,
            ..
        } = spawned;
        let result = timeout(Duration::from_secs(20), async {
            let mut output = Vec::new();
            while let Some(chunk) = stdout_rx.recv().await {
                output.extend(chunk);
                if String::from_utf8_lossy(&output).contains("1842-ready") {
                    if mode == "terminate" {
                        session.request_terminate();
                    } else {
                        session
                            .writer_sender()
                            .send(vec![3])
                            .await
                            .expect("type Ctrl-C");
                    }
                    return exit_rx.await.expect("command exit");
                }
            }
            panic!(
                "probe exited before readiness: {}",
                String::from_utf8_lossy(&output)
            );
        })
        .await;
        session.terminate();
        pretty_assertions::assert_eq!(result.expect("bounded input probe"), code);
    });
}

#[test]
fn sec_win_1842_raw_ctrl_c_is_a_byte() {
    if ignored_ctrl_c_host("unified_exec::tests::ctrl_c::sec_win_1842_raw_ctrl_c_is_a_byte") {
        run_input_case("raw", 42);
    }
}

#[test]
fn sec_win_1842_terminate_ends_the_launcher_and_command() {
    if ignored_ctrl_c_host(
        "unified_exec::tests::ctrl_c::sec_win_1842_terminate_ends_the_launcher_and_command",
    ) {
        run_input_case("terminate", 1);
    }
}

#[test]
fn sec_win_1842_command_handles_ctrl_c() {
    if ignored_ctrl_c_host("unified_exec::tests::ctrl_c::sec_win_1842_command_handles_ctrl_c") {
        run_input_case("handler", 43);
    }
}
