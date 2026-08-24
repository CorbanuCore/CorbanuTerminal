use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use tempfile::TempDir;

static NEXT_SERVER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
const STABLE_CAPTURE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TerminalSize {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
}

impl TerminalSize {
    pub(crate) fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TmuxKey {
    Enter,
}

impl TmuxKey {
    fn name(self) -> &'static str {
        match self {
            Self::Enter => "Enter",
        }
    }
}

#[derive(Debug)]
pub(crate) struct CommandSpec {
    program: OsString,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
}

impl CommandSpec {
    pub(crate) fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
        }
    }

    pub(crate) fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub(crate) fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    fn append_to(self, command: &mut Command) {
        if !self.env.is_empty() {
            command.arg("env");
            for (key, value) in self.env {
                let mut assignment = key;
                assignment.push("=");
                assignment.push(value);
                command.arg(assignment);
            }
        }
        command.arg(self.program).args(self.args);
    }
}

#[derive(Debug)]
pub(crate) struct SessionSpec {
    name_prefix: String,
    size: TerminalSize,
    command: CommandSpec,
    current_dir: Option<PathBuf>,
}

impl SessionSpec {
    pub(crate) fn new(
        name_prefix: impl Into<String>,
        size: TerminalSize,
        command: CommandSpec,
    ) -> Self {
        Self {
            name_prefix: name_prefix.into(),
            size,
            command,
            current_dir: None,
        }
    }

    pub(crate) fn current_dir(mut self, current_dir: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(current_dir.into());
        self
    }
}

#[derive(Debug)]
pub(crate) struct TmuxServer {
    socket_name: String,
    socket_dir: TempDir,
}

impl TmuxServer {
    pub(crate) fn is_available() -> bool {
        Command::new("tmux")
            .arg("-V")
            .output()
            .is_ok_and(|output| output.status.success())
    }

    pub(crate) fn start() -> Result<Self> {
        anyhow::ensure!(Self::is_available(), "tmux is unavailable on PATH");
        let id = NEXT_SERVER_ID.fetch_add(1, Ordering::Relaxed);
        let socket_dir = tempfile::Builder::new()
            .prefix("cdx-tmux-")
            .tempdir_in("/tmp")
            .context("create private tmux socket directory")?;
        Ok(Self {
            socket_name: format!("codex-tui-test-{}-{id}", std::process::id()),
            socket_dir,
        })
    }

    pub(crate) fn new_session(&self, spec: SessionSpec) -> Result<TmuxSession<'_>> {
        let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        let session_name = format!("{}-{session_id}", spec.name_prefix);
        let mut command = self.command();
        command
            .arg("new-session")
            .arg("-d")
            .arg("-P")
            .arg("-F")
            .arg("#{pane_id}")
            .arg("-x")
            .arg(spec.size.columns.to_string())
            .arg("-y")
            .arg(spec.size.rows.to_string())
            .arg("-s")
            .arg(&session_name);
        if let Some(current_dir) = spec.current_dir {
            command.arg("-c").arg(current_dir);
        }
        command.arg("--");
        spec.command.append_to(&mut command);

        let output = checked_output(&mut command)?;
        let pane_id = stdout_text(&output).trim().to_string();
        anyhow::ensure!(!pane_id.is_empty(), "tmux did not report a pane id");
        Ok(TmuxSession {
            server: self,
            name: session_name,
            primary_pane: TmuxPane {
                server: self,
                id: pane_id,
            },
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new("tmux");
        command.env("TMUX_TMPDIR", self.socket_dir.path());
        command.arg("-L").arg(&self.socket_name);
        command
    }

    fn socket_root(&self) -> PathBuf {
        self.socket_dir.path().to_path_buf()
    }

    fn has_session(&self, session_name: &str) -> bool {
        self.command()
            .arg("has-session")
            .arg("-t")
            .arg(session_name)
            .output()
            .is_ok_and(|output| output.status.success())
    }
}

impl Drop for TmuxServer {
    fn drop(&mut self) {
        let _ = self.command().arg("kill-server").output();
    }
}

#[derive(Debug)]
pub(crate) struct TmuxSession<'a> {
    server: &'a TmuxServer,
    name: String,
    primary_pane: TmuxPane<'a>,
}

impl<'a> TmuxSession<'a> {
    pub(crate) fn primary_pane(&self) -> &TmuxPane<'a> {
        &self.primary_pane
    }

    pub(crate) fn split_vertical(
        &self,
        target: &TmuxPane<'a>,
        rows: u16,
        command_spec: CommandSpec,
    ) -> Result<TmuxPane<'a>> {
        let mut command = self.server.command();
        command
            .arg("split-window")
            .arg("-d")
            .arg("-P")
            .arg("-F")
            .arg("#{pane_id}")
            .arg("-v")
            .arg("-l")
            .arg(rows.to_string())
            .arg("-t")
            .arg(&target.id)
            .arg("--");
        command_spec.append_to(&mut command);

        let output = checked_output(&mut command)?;
        let pane_id = stdout_text(&output).trim().to_string();
        anyhow::ensure!(!pane_id.is_empty(), "tmux did not report a split pane id");
        Ok(TmuxPane {
            server: self.server,
            id: pane_id,
        })
    }
}

impl Drop for TmuxSession<'_> {
    fn drop(&mut self) {
        if self.server.has_session(&self.name) {
            let _ = self
                .server
                .command()
                .arg("kill-session")
                .arg("-t")
                .arg(&self.name)
                .output();
        }
    }
}

#[derive(Debug)]
pub(crate) struct TmuxPane<'a> {
    server: &'a TmuxServer,
    id: String,
}

impl TmuxPane<'_> {
    pub(crate) fn send_literal(&self, text: &str) -> Result<()> {
        let mut command = self.server.command();
        checked_output(
            command
                .arg("send-keys")
                .arg("-t")
                .arg(&self.id)
                .arg("-l")
                .arg("--")
                .arg(text),
        )?;
        Ok(())
    }

    pub(crate) fn send_key(&self, key: TmuxKey) -> Result<()> {
        let mut command = self.server.command();
        checked_output(
            command
                .arg("send-keys")
                .arg("-t")
                .arg(&self.id)
                .arg(key.name()),
        )?;
        Ok(())
    }

    pub(crate) fn capture_viewport(&self) -> Result<String> {
        let mut command = self.server.command();
        let output = checked_output(
            command
                .arg("capture-pane")
                .arg("-p")
                .arg("-t")
                .arg(&self.id),
        )?;
        Ok(stdout_text(&output))
    }

    pub(crate) fn capture_scrollback_tail(&self, lines: usize) -> Result<String> {
        let mut command = self.server.command();
        let output = checked_output(
            command
                .arg("capture-pane")
                .arg("-p")
                .arg("-S")
                .arg(format!("-{lines}"))
                .arg("-t")
                .arg(&self.id),
        )?;
        Ok(stdout_text(&output))
    }

    pub(crate) fn wait_stable_contains(&self, needle: &str, timeout: Duration) -> Result<String> {
        self.wait_stable_until(
            &format!("viewport containing {needle:?}"),
            timeout,
            |capture| capture.contains(needle),
        )
    }

    pub(crate) fn wait_stable_until(
        &self,
        description: &str,
        timeout: Duration,
        condition: impl Fn(&str) -> bool,
    ) -> Result<String> {
        let deadline = Instant::now() + timeout;
        let mut previous_matching_capture = None;
        let mut last_capture = String::new();

        while Instant::now() < deadline {
            last_capture = self.capture_viewport()?;
            if condition(&last_capture) {
                if previous_matching_capture.as_deref() == Some(last_capture.as_str()) {
                    return Ok(last_capture);
                }
                previous_matching_capture = Some(last_capture.clone());
            } else {
                previous_matching_capture = None;
            }
            sleep(STABLE_CAPTURE_INTERVAL);
        }

        anyhow::bail!("timed out waiting for stable {description}; last viewport:\n{last_capture}");
    }

    pub(crate) fn close(self) -> Result<()> {
        let mut command = self.server.command();
        checked_output(command.arg("kill-pane").arg("-t").arg(&self.id))?;
        Ok(())
    }
}

fn checked_output(command: &mut Command) -> Result<Output> {
    let rendered = format!("{command:?}");
    let output = command
        .output()
        .with_context(|| format!("failed to start tmux command: {rendered}"))?;
    anyhow::ensure!(
        output.status.success(),
        "tmux command failed: {rendered}\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout_text(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output)
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn command_for_shell(script: &str) -> CommandSpec {
    CommandSpec::new(OsStr::new("sh")).arg("-c").arg(script)
}

#[cfg(test)]
#[path = "tmux_tests.rs"]
mod tests;
