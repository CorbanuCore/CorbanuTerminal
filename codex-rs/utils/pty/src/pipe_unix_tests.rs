use std::collections::HashMap;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::Duration;

use pretty_assertions::assert_eq;

use super::PipeStdinMode;
use super::spawn_process_with_stdin_mode;
use crate::ProcessSignal;

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
            assert_eq!(libc::sigprocmask(/*how*/ 0, std::ptr::null(), &mut mask), 0);
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
        output.status.success(),
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
                "/bin/sh",
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
        }
    }
    Ok(())
}
