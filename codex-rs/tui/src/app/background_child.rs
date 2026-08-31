use std::io;
use std::process::ExitStatus;

use tokio::process::Command;
use tokio::task::JoinHandle;

/// Spawns an independent child while retaining an asynchronous parent-side waiter.
///
/// Dropping the returned task handle detaches the task; the task continues to own
/// and reap the child when it exits. The child is not killed if the TUI runtime
/// shuts down first, preserving independent-controller behavior.
pub(super) fn spawn_with_reaper(
    command: &mut Command,
) -> io::Result<JoinHandle<io::Result<ExitStatus>>> {
    let mut child = command.spawn()?;
    let child_pid = child.id();
    Ok(tokio::spawn(async move {
        let result = child.wait().await;
        match &result {
            Ok(status) => tracing::debug!(?child_pid, ?status, "background child reaped"),
            Err(error) => {
                tracing::warn!(?child_pid, %error, "failed to reap background child")
            }
        }
        result
    }))
}

#[cfg(test)]
#[path = "background_child_tests.rs"]
mod tests;
