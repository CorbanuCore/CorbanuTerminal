use std::process::Stdio;

use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tokio::process::Command;
use tokio::time::Duration;
use tokio::time::sleep;
use tokio::time::timeout;

use super::spawn_with_reaper;

#[tokio::test]
async fn detached_wait_task_reaps_the_background_child() {
    let temp_dir = TempDir::new().expect("temporary marker directory");
    let pid_marker = temp_dir.path().join("child-pid");
    #[cfg(unix)]
    let mut command = Command::new("/bin/sh");
    #[cfg(unix)]
    command
        .args(["-c", "printf '%s\\n' \"$$\" > \"$PID_MARKER\"; sleep 0.1"])
        .env("PID_MARKER", &pid_marker)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-Command",
                "[IO.File]::WriteAllText($env:PID_MARKER, 'exited')",
            ])
            .env("PID_MARKER", &pid_marker)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    };

    let wait_task = spawn_with_reaper(&mut command).expect("spawn background child");
    drop(wait_task);

    let marker = timeout(Duration::from_secs(5), async {
        loop {
            match std::fs::read_to_string(&pid_marker) {
                Ok(marker) if marker_is_complete(&marker) => break marker,
                Ok(_) => sleep(Duration::from_millis(10)).await,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    sleep(Duration::from_millis(10)).await;
                }
                Err(error) => panic!("read child marker: {error}"),
            }
        }
    })
    .await
    .expect("detached background child completed");

    #[cfg(unix)]
    {
        let child_pid = marker
            .trim_end()
            .parse::<libc::pid_t>()
            .expect("numeric child pid");
        timeout(Duration::from_secs(5), async {
            loop {
                let exists = unsafe { libc::kill(child_pid, 0) };
                if exists == -1
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("detached waiter removed child from the process table");
        let mut wait_status = 0;
        let waited = unsafe { libc::waitpid(child_pid, &mut wait_status, libc::WNOHANG) };
        assert_eq!(waited, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
    }

    #[cfg(windows)]
    assert_eq!(marker, "exited");
}

#[cfg(unix)]
fn marker_is_complete(marker: &str) -> bool {
    marker.ends_with('\n') && marker.trim_end().parse::<libc::pid_t>().is_ok()
}

#[cfg(windows)]
fn marker_is_complete(marker: &str) -> bool {
    marker == "exited"
}
