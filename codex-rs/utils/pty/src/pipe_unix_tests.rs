//! Pipe and PTY compatibility under inherited-descriptor pressure, and pipe
//! children's signal state when the parent ignores or blocks signals.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::io::Seek;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use pretty_assertions::assert_eq;

use super::PipeStdinMode;
use super::spawn_process_with_stdin_mode;
use crate::ProcessSignal;

#[tokio::test]
async fn descriptor_capture_pressure_falls_back_to_direct_spawn() -> anyhow::Result<()> {
    if std::env::var_os("CODEX_TEST_PIPE_FD_PRESSURE").is_none() {
        let output = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "pipe::unix_tests::descriptor_capture_pressure_falls_back_to_direct_spawn",
                "--nocapture",
            ])
            .env("CODEX_TEST_PIPE_FD_PRESSURE", "1")
            .output()?;
        assert!(output.status.success(), "{output:?}");
        return Ok(());
    }
    let root = tempfile::tempdir()?;
    let path = root.path().join("input");
    std::fs::write(&path, "preserved")?;
    let mut source = std::fs::File::open(path)?;
    let descriptors = (0..17)
        .map(|_| {
            // SAFETY: fcntl duplicates a live source into a newly owned slot.
            let fd = unsafe { libc::fcntl(source.as_raw_fd(), libc::F_DUPFD, 3) };
            anyhow::ensure!(fd >= 3, "{}", io::Error::last_os_error());
            Ok(unsafe { OwnedFd::from_raw_fd(fd) })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let targets = descriptors[..16]
        .iter()
        .map(AsRawFd::as_raw_fd)
        .collect::<Vec<_>>();
    let excluded = descriptors[16].as_raw_fd();
    let script = format!(
        "for fd in \"$@\"; do test -e /dev/fd/$fd || exit 10; done; \
         test ! -e /dev/fd/{excluded} || exit 11; /bin/cat /dev/fd/$1"
    );
    let mut args = vec!["-c".to_owned(), script, "sh".to_owned()];
    args.extend(targets.iter().map(ToString::to_string));
    let mut limit = std::mem::MaybeUninit::<libc::rlimit>::uninit();
    // SAFETY: getrlimit initializes writable storage.
    assert_eq!(
        unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, limit.as_mut_ptr()) },
        0
    );
    let mut limit = unsafe { limit.assume_init() };
    limit.rlim_cur = limit.rlim_cur.min(128);
    // SAFETY: Only this isolated subprocess's soft descriptor limit changes.
    assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limit) }, 0);
    enum Transport {
        Pipe(super::PipeStdinMode),
        Pty,
    }
    for (spare_slots, transport) in [
        (12, Transport::Pipe(super::PipeStdinMode::Piped)),
        (12, Transport::Pipe(super::PipeStdinMode::Null)),
        (12, Transport::Pty),
        (20, Transport::Pipe(super::PipeStdinMode::Piped)),
        (20, Transport::Pipe(super::PipeStdinMode::Null)),
        (20, Transport::Pty),
    ] {
        source.rewind()?;
        let mut occupied = Vec::new();
        loop {
            let fd = unsafe { libc::fcntl(source.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
            if fd == -1 {
                assert_eq!(
                    io::Error::last_os_error().raw_os_error(),
                    Some(libc::EMFILE)
                );
                break;
            }
            occupied.push(unsafe { OwnedFd::from_raw_fd(fd) });
        }
        // Twelve slots reject capture itself; twenty allow all sixteen copies
        // but leave too little room for stdio. Both must preserve the old launch.
        anyhow::ensure!(
            occupied.len() >= spare_slots,
            "not enough descriptors to reserve"
        );
        occupied.truncate(occupied.len() - spare_slots);
        let mut spawned = match transport {
            Transport::Pipe(stdin) => {
                super::spawn_process_with_stdin_mode(
                    std::ffi::OsStr::new("/bin/sh"),
                    &args,
                    Path::new("."),
                    &HashMap::new(),
                    &None,
                    stdin,
                    &targets,
                )
                .await?
            }
            Transport::Pty => {
                crate::spawn_pty_process(
                    "/bin/sh",
                    &args,
                    Path::new("."),
                    &HashMap::new(),
                    &None,
                    crate::TerminalSize::default(),
                    crate::ChildFds::Inherited(&targets),
                )
                .await?
            }
        };
        drop(occupied);
        spawned.session.close_stdin();
        let mut stdout = Vec::new();
        while let Some(bytes) = spawned.stdout_rx.recv().await {
            stdout.extend(bytes);
        }
        let mut stderr = Vec::new();
        while let Some(bytes) = spawned.stderr_rx.recv().await {
            stderr.extend(bytes);
        }
        assert_eq!(
            (spawned.exit_rx.await?, stdout, stderr),
            (0, b"preserved".to_vec(), vec![])
        );
        for fd in &descriptors {
            assert_eq!(unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFD) }, 0);

const HELPER_MODE: &str = "CODEX_PTY_SIGNAL_TEST_HELPER";

#[test]
fn pipe_children_reset_ignored_signals() -> anyhow::Result<()> {
    run_isolated(
        "pipe::unix_tests::pipe_children_reset_ignored_signals",
        "ignored",
    )
}

#[test]
fn pipe_children_clear_blocked_signals() -> anyhow::Result<()> {
    run_isolated(
        "pipe::unix_tests::pipe_children_clear_blocked_signals",
        "blocked",
    )
}

fn run_isolated(test_name: &str, mode: &str) -> anyhow::Result<()> {
    if std::env::var(HELPER_MODE).as_deref() == Ok(mode) {
        // Check the inherited state before Tokio installs any signal handlers.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            assert_eq!(
                libc::sigaction(libc::SIGINT, std::ptr::null(), &mut action),
                0
            );
            let mut mask: libc::sigset_t = std::mem::zeroed();
            assert_eq!(
                libc::sigprocmask(libc::SIG_BLOCK, std::ptr::null(), &mut mask),
                0
            );
            if mode == "ignored" {
                assert_eq!(action.sa_sigaction, libc::SIG_IGN);
            } else {
                assert_eq!(libc::sigismember(&mask, libc::SIGINT), 1);
            }
        }
        return tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(assert_pipe_children_interruptible());
    }

    // Set dispositions/masks only in the re-exec child, never in the shared
    // test runner. The helper becomes the parent of the actual pipe commands.
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args(["--exact", test_name, "--nocapture"])
        .env(HELPER_MODE, mode);
    let ignored = mode == "ignored";
    unsafe {
        command.pre_exec(move || {
            let mut mask: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut mask);
            for signal in [libc::SIGINT, libc::SIGQUIT] {
                let mut action: libc::sigaction = std::mem::zeroed();
                action.sa_sigaction = if ignored {
                    libc::SIG_IGN
                } else {
                    libc::SIG_DFL
                };
                libc::sigemptyset(&mut action.sa_mask);
                if libc::sigaction(signal, &action, std::ptr::null_mut()) == -1 {
                    return Err(io::Error::last_os_error());
                }
                if !ignored {
                    libc::sigaddset(&mut mask, signal);
                }
            }
            if libc::sigprocmask(libc::SIG_SETMASK, &mask, std::ptr::null_mut()) == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let output = command.output()?;
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed;"),
        "{mode} helper failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

async fn assert_pipe_children_interruptible() -> anyhow::Result<()> {
    let env: HashMap<String, String> = std::env::vars().collect();
    let cwd = std::env::current_dir()?;
    for stdin_mode in [PipeStdinMode::Piped, PipeStdinMode::Null] {
        for (script, expected_code) in [
            ("echo ready; exec sleep 30", 128 + libc::SIGINT),
            (
                "trap 'exit 42' INT; echo ready; while :; do sleep 1; done",
                42,
            ),
            // A shell cannot install this trap if SIGQUIT was ignored on entry.
            ("trap 'exit 43' QUIT; kill -s QUIT $$; exit 99", 43),
        ] {
            let mut child = spawn_process_with_stdin_mode(
                OsStr::new("/bin/sh"),
                &["-c".to_string(), script.to_string()],
                &cwd,
                &env,
                &None,
                stdin_mode,
                &[],
            )
            .await?;
            let result = tokio::time::timeout(Duration::from_secs(5), async {
                if expected_code != 43 {
                    let mut output = Vec::new();
                    while !output.ends_with(b"ready\n") {
                        let chunk =
                            child.stdout_rx.recv().await.ok_or_else(|| {
                                anyhow::anyhow!("pipe child exited before readiness")
                            })?;
                        output.extend_from_slice(&chunk);
                    }
                    child.session.signal(ProcessSignal::Interrupt)?;
                }
                let code = (&mut child.exit_rx).await?;
                assert_eq!(code, expected_code, "script: {script}");
                anyhow::Ok(())
            })
            .await;
            // Kill and reap even on a timeout, so fail-before runs leave no child.
            if result.is_err() {
                child.session.request_terminate();
                let _ = tokio::time::timeout(Duration::from_secs(5), &mut child.exit_rx).await;
            }
            result??;
