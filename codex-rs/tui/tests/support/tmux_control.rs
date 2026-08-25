use std::collections::VecDeque;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;

use super::tmux::TmuxSession;
use super::tmux_artifacts::ControlTranscript;
use super::tmux_command::render_command;
use super::tmux_control_protocol::CommandResult;
use super::tmux_control_protocol::CommandStatus;
pub(crate) use super::tmux_control_protocol::ControlEvent;
use super::tmux_control_protocol::ControlItem;
use super::tmux_control_protocol::ControlParser;
use super::tmux_control_protocol::MAX_CONTROL_BACKLOG_BYTES;
use super::tmux_control_protocol::MAX_CONTROL_EVENTS;
use super::tmux_control_protocol::MAX_CONTROL_LINE_BYTES;
use super::tmux_control_protocol::ParsedItem;

#[derive(Debug)]
pub(crate) struct TmuxControlClient<'a> {
    session: &'a TmuxSession<'a>,
    child: Child,
    stdin: Option<ChildStdin>,
    queue: Arc<ControlQueue>,
    reader: Option<JoinHandle<()>>,
}

impl<'a> TmuxSession<'a> {
    pub(crate) fn attach_control(&'a self, timeout: Duration) -> Result<TmuxControlClient<'a>> {
        let trace = Arc::new(ControlTranscript::default());
        self.server
            .artifacts
            .register_control_transcript(Arc::clone(&trace));
        let mut command = self.server.command();
        command
            .arg("-C")
            .arg("attach-session")
            .arg("-t")
            .arg(&self.name)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        self.server
            .artifacts
            .record_command(render_command(&command));
        let mut child = command.spawn().context("start tmux control client")?;
        let Some(stdin) = child.stdin.take() else {
            terminate_child(&mut child);
            anyhow::bail!("open tmux control stdin");
        };
        let Some(stdout) = child.stdout.take() else {
            terminate_child(&mut child);
            anyhow::bail!("open tmux control stdout");
        };
        let queue = Arc::new(ControlQueue::default());
        let reader_queue = Arc::clone(&queue);
        let reader = std::thread::Builder::new()
            .name("tmux-control-reader".into())
            .spawn(move || read_control_stream(BufReader::new(stdout), reader_queue, trace))
            .inspect_err(|_| {
                terminate_child(&mut child);
            })
            .context("start tmux control reader")?;
        let client = TmuxControlClient {
            session: self,
            child,
            stdin: Some(stdin),
            queue,
            reader: Some(reader),
        };
        let handshake = client.with_failure_artifact(
            client.wait_command_result("control attach handshake", timeout),
        )?;
        if handshake.status != CommandStatus::Success {
            return client.with_failure_artifact(Err(anyhow::anyhow!(
                "tmux control attach failed: {}",
                command_output(&handshake)
            )));
        }
        Ok(client)
    }
}

impl TmuxControlClient<'_> {
    pub(crate) fn command(&mut self, command: &str, timeout: Duration) -> Result<CommandResult> {
        let result = self.run_command(command, timeout);
        self.with_failure_artifact(result)
    }

    pub(crate) fn wait_event(
        &self,
        description: &str,
        timeout: Duration,
        predicate: impl Fn(&ControlEvent) -> bool,
    ) -> Result<ControlEvent> {
        let result = self
            .queue
            .wait_remove(
                description,
                timeout,
                |item| matches!(item, ControlItem::Event(event) if predicate(event)),
            )
            .and_then(|item| match item {
                ControlItem::Event(event) => Ok(event),
                ControlItem::Command(_) => anyhow::bail!("matched command as control event"),
            });
        self.with_failure_artifact(result)
    }

    fn run_command(&mut self, command: &str, timeout: Duration) -> Result<CommandResult> {
        anyhow::ensure!(
            !command.contains(['\n', '\r']),
            "tmux control command must be one line"
        );
        self.session
            .server
            .artifacts
            .record_command(format!("tmux-control {command}"));
        let stdin = self
            .stdin
            .as_mut()
            .context("tmux control stdin is closed")?;
        stdin
            .write_all(command.as_bytes())
            .context("write tmux control command")?;
        stdin
            .write_all(b"\n")
            .context("terminate tmux control command")?;
        stdin.flush().context("flush tmux control command")?;
        self.wait_command_result(command, timeout)
    }

    fn wait_command_result(&self, description: &str, timeout: Duration) -> Result<CommandResult> {
        self.queue
            .wait_remove(description, timeout, |item| {
                matches!(item, ControlItem::Command(_))
            })
            .and_then(|item| match item {
                ControlItem::Command(result) => Ok(result),
                ControlItem::Event(_) => anyhow::bail!("matched event as command result"),
            })
    }

    fn with_failure_artifact<T>(&self, result: Result<T>) -> Result<T> {
        result.map_err(|error| {
            let reason = format!("{error:#}");
            let artifact = self
                .session
                .server
                .emit_failure(&reason, Some(self.session.primary_pane().id.as_str()));
            anyhow::anyhow!("{reason}; tmux control artifacts: {}", artifact.display())
        })
    }
}

impl Drop for TmuxControlClient<'_> {
    fn drop(&mut self) {
        self.stdin.take();
        terminate_child(&mut self.child);
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[derive(Debug, Default)]
struct ControlQueue {
    state: Mutex<QueueState>,
    ready: Condvar,
}

#[derive(Debug, Default)]
struct QueueState {
    items: VecDeque<ParsedItem>,
    retained_bytes: usize,
    failure: Option<String>,
    closed: bool,
}

impl ControlQueue {
    fn push(&self, item: ParsedItem) -> Result<()> {
        let mut state = self.lock();
        let next_bytes = state
            .retained_bytes
            .checked_add(item.retained_bytes)
            .context("tmux control backlog size overflow")?;
        if state.items.len() >= MAX_CONTROL_EVENTS || next_bytes > MAX_CONTROL_BACKLOG_BYTES {
            let error = format!(
                "tmux control backlog exceeded {MAX_CONTROL_EVENTS} events or \
                 {MAX_CONTROL_BACKLOG_BYTES} bytes"
            );
            state.failure = Some(error.clone());
            state.closed = true;
            self.ready.notify_all();
            anyhow::bail!(error);
        }
        state.retained_bytes = next_bytes;
        state.items.push_back(item);
        self.ready.notify_all();
        Ok(())
    }

    fn fail(&self, error: String) {
        let mut state = self.lock();
        state.failure = Some(error);
        state.closed = true;
        self.ready.notify_all();
    }

    fn close(&self) {
        let mut state = self.lock();
        state.closed = true;
        self.ready.notify_all();
    }

    fn wait_remove(
        &self,
        description: &str,
        timeout: Duration,
        predicate: impl Fn(&ControlItem) -> bool,
    ) -> Result<ControlItem> {
        let deadline = Instant::now() + timeout;
        let mut state = self.lock();
        loop {
            if let Some(position) = state.items.iter().position(|item| predicate(&item.value)) {
                let item = state
                    .items
                    .remove(position)
                    .context("tmux control queue item disappeared")?;
                state.retained_bytes = state.retained_bytes.saturating_sub(item.retained_bytes);
                return Ok(item.value);
            }
            if let Some(error) = state.failure.as_deref() {
                anyhow::bail!("{error}");
            }
            anyhow::ensure!(
                !state.closed,
                "tmux control stream closed while waiting for {description}"
            );
            let now = Instant::now();
            anyhow::ensure!(now < deadline, "timed out waiting for {description}");
            let wait = deadline.saturating_duration_since(now);
            let (next_state, _) = self
                .ready
                .wait_timeout(state, wait)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state = next_state;
        }
    }

    fn lock(&self) -> MutexGuard<'_, QueueState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn read_control_stream(
    mut reader: impl BufRead,
    queue: Arc<ControlQueue>,
    trace: Arc<ControlTranscript>,
) {
    let mut parser = ControlParser::default();
    loop {
        match read_control_line(&mut reader) {
            Ok(Some(line)) => {
                trace.record_line(&line);
                match parser.parse_line(&line) {
                    Ok(Some(item)) => {
                        if let Err(error) = queue.push(item) {
                            let error = error.to_string();
                            trace.record_error(error.clone());
                            queue.fail(error);
                            return;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        trace.record_error(format!("{error:#}"));
                        queue.fail(format!("{error:#}"));
                        return;
                    }
                }
            }
            Ok(None) => {
                if let Err(error) = parser.finish() {
                    trace.record_error(format!("{error:#}"));
                    queue.fail(format!("{error:#}"));
                } else {
                    queue.close();
                }
                return;
            }
            Err(error) => {
                trace.record_error(format!("{error:#}"));
                queue.fail(format!("{error:#}"));
                return;
            }
        }
    }
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_control_line(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().context("read tmux control stream")?;
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(line))
            };
        }
        let consumed = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        let content = &available[..consumed];
        let content = content.strip_suffix(b"\n").unwrap_or(content);
        let complete = available.get(consumed.saturating_sub(1)) == Some(&b'\n');
        anyhow::ensure!(
            line.len() + content.len() <= MAX_CONTROL_LINE_BYTES,
            "tmux control line exceeded {MAX_CONTROL_LINE_BYTES} bytes"
        );
        line.extend_from_slice(content);
        reader.consume(consumed);
        if complete {
            return Ok(Some(line));
        }
    }
}

fn command_output(result: &CommandResult) -> String {
    result
        .output
        .iter()
        .map(|line| String::from_utf8_lossy(line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "tmux_control_tests.rs"]
mod tests;
