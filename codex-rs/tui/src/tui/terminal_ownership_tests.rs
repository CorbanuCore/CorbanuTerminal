use std::fs::File;
use std::io::Read;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::process::Command;
use std::process::Stdio;

use pretty_assertions::assert_eq;

use super::TerminalOwnership;
use super::TerminalRestoreGuard;
use super::panic_diagnostics;
use super::set_modes_with_ownership;

const CHILD_MODE_ENV: &str = "CORBANU_TUI_TERMINAL_OWNERSHIP_CHILD_MODE";
const DIAGNOSTIC_DIR_ENV: &str = "CORBANU_TUI_PANIC_DIAGNOSTIC_DIR";
const CONTAINED_PANIC_PAYLOAD: &str = "contained-sensitive-payload-must-not-persist";
const FATAL_PANIC_PAYLOAD: &str = "fatal-sensitive-payload-must-not-persist";

#[test]
#[serial_test::serial]
fn contained_and_fatal_panics_respect_terminal_ownership() {
    if let Some(mode) = std::env::var_os(CHILD_MODE_ENV) {
        run_child(&mode.to_string_lossy());
        return;
    }

    let diagnostic_dir = tempfile::tempdir().expect("panic diagnostic directory");
    let contained = spawn_pty_child("contained", diagnostic_dir.path());
    assert!(contained.status.success(), "contained panic child failed");

    let fatal = spawn_pty_child("fatal", diagnostic_dir.path());
    assert!(
        !fatal.status.success(),
        "fatal panic child unexpectedly passed"
    );
    assert!(
        String::from_utf8_lossy(&fatal.output).contains(FATAL_PANIC_PAYLOAD),
        "fatal panic report was not emitted after terminal restoration"
    );

    let diagnostic = std::fs::read_to_string(diagnostic_dir.path().join("tui-panics.log"))
        .expect("persistent panic diagnostic");
    assert!(diagnostic.lines().any(|line| line.contains(
        "phase=classified disposition=contained_or_background terminal_owned=true \
         mode_disposition=preserved"
    )));
    assert!(diagnostic.lines().any(|line| line.contains(
        "phase=classified disposition=fatal_foreground terminal_owned=true \
         mode_disposition=restoring"
    )));
    assert!(!diagnostic.contains(CONTAINED_PANIC_PAYLOAD));
    assert!(!diagnostic.contains(FATAL_PANIC_PAYLOAD));
}

fn run_child(mode: &str) {
    let diagnostic_dir = std::env::var_os(DIAGNOSTIC_DIR_ENV)
        .map(std::path::PathBuf::from)
        .expect("diagnostic directory passed to child");
    panic_diagnostics::configure(diagnostic_dir).expect("configure panic diagnostics");
    super::set_panic_hook();

    let mut restore_guard = TerminalRestoreGuard::new();
    set_modes_with_ownership(TerminalOwnership::Acquire(&mut restore_guard))
        .expect("acquire terminal modes");
    assert_raw_no_echo();
    let _stderr_guard = super::terminal_stderr::TerminalStderrGuard::install()
        .expect("install terminal stderr guard");

    match mode {
        "contained" => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("contained-panic-worker")
                .enable_all()
                .build()
                .expect("contained panic runtime");
            let panic = runtime.block_on(async {
                tokio::spawn(async { panic!("{CONTAINED_PANIC_PAYLOAD}") }).await
            });
            assert!(
                panic
                    .expect_err("spawned panic must be contained")
                    .is_panic()
            );
            assert_raw_no_echo();
            panic_diagnostics::record_tui_survived();
            restore_guard
                .restore()
                .expect("restore after contained panic");
            assert_canonical_echo();
        }
        "fatal" => panic!("{FATAL_PANIC_PAYLOAD}"),
        other => panic!("unknown child mode {other}"),
    }
}

struct PtyChildOutput {
    status: std::process::ExitStatus,
    output: Vec<u8>,
}

fn spawn_pty_child(mode: &str, diagnostic_dir: &std::path::Path) -> PtyChildOutput {
    let (master, slave) = open_pty();
    assert_canonical_echo_fd(&slave);
    let mut command = Command::new(std::env::current_exe().expect("current test executable"));
    command
        .args([
            "--exact",
            "tui::terminal_ownership_tests::contained_and_fatal_panics_respect_terminal_ownership",
            "--nocapture",
        ])
        .env(CHILD_MODE_ENV, mode)
        .env(DIAGNOSTIC_DIR_ENV, diagnostic_dir)
        .env("RUST_BACKTRACE", "full")
        .stdin(Stdio::from(slave.try_clone().expect("clone slave stdin")))
        .stdout(Stdio::from(slave.try_clone().expect("clone slave stdout")))
        .stderr(Stdio::from(slave.try_clone().expect("clone slave stderr")));
    let mut child = command.spawn().expect("spawn terminal ownership child");
    // Command retains its configured Stdio handles so it can be spawned again.
    // Release those parent-side slave descriptors before waiting for master EOF.
    drop(command);
    let output_reader = std::thread::spawn(move || {
        let mut master = master;
        let mut output = Vec::new();
        let mut buffer = [0_u8; 8192];
        loop {
            match master.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => output.extend_from_slice(&buffer[..read]),
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                // Linux PTY masters return EIO rather than EOF after the last
                // slave descriptor closes.
                Err(err) if err.raw_os_error() == Some(libc::EIO) => break,
                Err(err) => panic!("drain terminal ownership child output: {err}"),
            }
        }
        output
    });
    let status = child.wait().expect("wait for terminal ownership child");
    assert_canonical_echo_fd(&slave);
    drop(slave);
    let output = output_reader.join().expect("join pty output reader");
    PtyChildOutput { status, output }
}

fn open_pty() -> (File, File) {
    let mut master_fd = -1;
    let mut slave_fd = -1;
    let result = unsafe {
        libc::openpty(
            &mut master_fd,
            &mut slave_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(
        result,
        0,
        "openpty failed: {}",
        std::io::Error::last_os_error()
    );
    let master = File::from(unsafe { OwnedFd::from_raw_fd(master_fd) });
    let slave = File::from(unsafe { OwnedFd::from_raw_fd(slave_fd) });
    (master, slave)
}

fn assert_raw_no_echo() {
    let termios = termios_for_fd(libc::STDIN_FILENO);
    assert_eq!(termios.c_lflag & (libc::ICANON | libc::ECHO), 0);
}

fn assert_canonical_echo() {
    let termios = termios_for_fd(libc::STDIN_FILENO);
    assert_ne!(termios.c_lflag & libc::ICANON, 0);
    assert_ne!(termios.c_lflag & libc::ECHO, 0);
}

fn assert_canonical_echo_fd(file: &File) {
    use std::os::fd::AsRawFd;

    let termios = termios_for_fd(file.as_raw_fd());
    assert_ne!(termios.c_lflag & libc::ICANON, 0);
    assert_ne!(termios.c_lflag & libc::ECHO, 0);
}

fn termios_for_fd(fd: libc::c_int) -> libc::termios {
    let mut termios = MaybeUninit::uninit();
    let result = unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) };
    assert_eq!(
        result,
        0,
        "tcgetattr failed: {}",
        std::io::Error::last_os_error()
    );
    unsafe { termios.assume_init() }
}
