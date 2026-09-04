use std::cell::Cell;
use std::cell::RefCell;
use std::process::Command;
use std::process::Output;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;

const CLEANUP_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) const CLEANUP_WATCHDOG: &str = r#"
trap '' HUP INT TERM
while IFS= read -r line; do :; done
panes=$(tmux -L "$1" list-panes -a -F '#{pane_pid}' 2>/dev/null)
for pid in $panes; do
    case "$pid" in ''|*[!0-9]*) continue;; esac
    group=$(ps -o pgid= -p "$pid" | tr -d ' ')
    if [ "$group" = "$pid" ]; then kill -s TERM -- "-$pid" 2>/dev/null; fi
done
sleep 0.2
for pid in $panes; do
    case "$pid" in ''|*[!0-9]*) continue;; esac
    group=$(ps -o pgid= -p "$pid" | tr -d ' ')
    if [ "$group" = "$pid" ]; then kill -s KILL -- "-$pid" 2>/dev/null; fi
done
exec tmux -L "$1" kill-server
"#;

#[derive(Debug, Default)]
pub(super) struct TmuxProcesses {
    server_pid: Cell<Option<u32>>,
    pane_pids: RefCell<Vec<u32>>,
}

impl TmuxProcesses {
    pub(super) fn record(&self, pane_pid: u32, server_pid: u32) {
        self.server_pid.set(Some(server_pid));
        self.pane_pids.borrow_mut().push(pane_pid);
    }

    pub(super) fn wait_for_cleanup(&mut self) {
        for pid in self.pane_pids.get_mut().drain(..) {
            terminate_pane_group(pid);
        }
        if let Some(pid) = self.server_pid.get() {
            wait_for_exit(pid, CLEANUP_TIMEOUT);
        }
    }
}

pub(super) fn terminate_pane_group(pid: u32) {
    // Only signal a pane group while its recorded leader still owns that group.
    let group = Command::new("ps")
        .args(["-o", "pgid=", "-p", &pid.to_string()])
        .output();
    if group
        .ok()
        .is_some_and(|output| String::from_utf8_lossy(&output.stdout).trim() == pid.to_string())
    {
        let _ = Command::new("kill")
            .args(["-s", "TERM", "--", &format!("-{pid}")])
            .output();
        wait_for_exit(pid, CLEANUP_TIMEOUT);
        if is_running(pid) {
            let _ = Command::new("kill")
                .args(["-s", "KILL", "--", &format!("-{pid}")])
                .output();
            wait_for_exit(pid, CLEANUP_TIMEOUT);
        }
    }
}

pub(super) fn parse_report(output: &Output, description: &str) -> Result<(String, u32, u32)> {
    let output = String::from_utf8_lossy(&output.stdout);
    let mut fields = output.trim().split('\t');
    let pane_id = fields.next().unwrap_or_default().to_string();
    anyhow::ensure!(
        !pane_id.is_empty(),
        "tmux did not report a {description} id"
    );
    let pane_pid = fields
        .next()
        .with_context(|| format!("tmux did not report a {description} pid"))?
        .parse::<u32>()
        .with_context(|| format!("parse tmux {description} pid"))?;
    let server_pid = fields
        .next()
        .context("tmux did not report a server pid")?
        .parse::<u32>()
        .context("parse tmux server pid")?;
    Ok((pane_id, pane_pid, server_pid))
}

pub(super) fn is_running(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .is_ok_and(|output| output.status.success())
}

pub(super) fn wait_for_exit(pid: u32, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline && is_running(pid) {
        sleep(Duration::from_millis(10));
    }
}
