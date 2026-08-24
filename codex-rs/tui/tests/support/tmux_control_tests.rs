use std::io::Cursor;
use std::sync::Arc;

use pretty_assertions::assert_eq;

use super::*;
use crate::support::tmux::CommandSpec;
use crate::support::tmux::SessionSpec;
use crate::support::tmux::TerminalSize;
use crate::support::tmux::TmuxServer;
use crate::support::tmux_artifacts::ArtifactRecorder;
use crate::support::tmux_artifacts::ControlTranscript;
use crate::support::tmux_artifacts::FailureCapture;
use crate::support::tmux_control_protocol::ControlEvent;
use crate::support::tmux_control_protocol::ControlItem;
use crate::support::tmux_control_protocol::ParsedItem;

const WAIT: Duration = Duration::from_secs(3);

#[test]
fn queue_limits_bound_event_count_and_retained_bytes() {
    let count_queue = ControlQueue::default();
    for _ in 0..MAX_CONTROL_EVENTS {
        count_queue
            .push(unknown_item(/*retained_bytes*/ 1))
            .expect("event within count limit should queue");
    }
    assert_eq!(
        count_queue
            .push(unknown_item(/*retained_bytes*/ 1))
            .expect_err("event beyond count limit should fail")
            .to_string(),
        format!(
            "tmux control backlog exceeded {MAX_CONTROL_EVENTS} events or \
             {MAX_CONTROL_BACKLOG_BYTES} bytes"
        )
    );

    let byte_queue = ControlQueue::default();
    byte_queue
        .push(unknown_item(MAX_CONTROL_BACKLOG_BYTES))
        .expect("event within byte limit should queue");
    assert!(
        byte_queue
            .push(unknown_item(/*retained_bytes*/ 1))
            .expect_err("event beyond byte limit should fail")
            .to_string()
            .contains("tmux control backlog exceeded")
    );
}

#[test]
fn line_reader_accepts_the_limit_and_rejects_the_next_byte() {
    let mut at_limit = vec![b'x'; MAX_CONTROL_LINE_BYTES];
    at_limit.push(b'\n');
    assert_eq!(
        read_control_line(&mut Cursor::new(at_limit)).expect("line at limit should read"),
        Some(vec![b'x'; MAX_CONTROL_LINE_BYTES])
    );

    let oversized = vec![b'x'; MAX_CONTROL_LINE_BYTES + 1];
    assert_eq!(
        read_control_line(&mut Cursor::new(oversized))
            .expect_err("oversized line should fail")
            .to_string(),
        format!("tmux control line exceeded {MAX_CONTROL_LINE_BYTES} bytes")
    );
}

#[test]
fn unfinished_command_at_eof_fails_the_reader_queue() {
    let queue = Arc::new(ControlQueue::default());
    let trace = Arc::new(ControlTranscript::default());
    read_control_stream(Cursor::new(b"%begin 1 1 0\n"), Arc::clone(&queue), trace);

    assert_eq!(
        queue
            .wait_remove("unfinished command", WAIT, |_| true)
            .expect_err("reader should preserve EOF framing error")
            .to_string(),
        "tmux control stream ended inside a command block"
    );
}

#[test]
fn parser_error_and_transcript_join_failure_bundle() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let recorder = ArtifactRecorder::new(root.path().to_path_buf(), "control_artifacts", 1);
    let transcript = Arc::new(ControlTranscript::default());
    transcript.record_line(b"%begin 1 1 0");
    transcript.record_error("invalid tmux octal escape".into());
    recorder.register_control_transcript(transcript);
    let directory = recorder.emit(FailureCapture {
        reason: "parser failed".into(),
        viewport: String::new(),
        scrollback: String::new(),
        pane_metadata: String::new(),
    })?;

    assert_eq!(
        std::fs::read_to_string(directory.join("control-transcript.txt"))?,
        "%begin 1 1 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(directory.join("control-parser-error.txt"))?,
        "invalid tmux octal escape"
    );
    Ok(())
}

#[test]
fn tmux03_control_mode_tracks_two_pane_lifecycle() -> anyhow::Result<()> {
    if !TmuxServer::should_run("tmux03 control lifecycle")? {
        return Ok(());
    }
    let server = TmuxServer::start("tmux03_control_mode_tracks")?;
    let session = server.new_session(SessionSpec::new(
        "control-lifecycle",
        TerminalSize::new(/*columns*/ 80, /*rows*/ 20),
        CommandSpec::new("cat"),
    ))?;
    let mut control = session.attach_control(WAIT)?;
    let primary = command_text(&control.command("list-panes -F \"#{pane_id}\"", WAIT)?);
    let primary = primary.trim().to_string();
    assert!(primary.starts_with('%'));

    let split = command_text(&control.command(
        &format!("split-window -d -P -F \"#{{pane_id}}\" -h -l 30 -t {primary} -- cat"),
        WAIT,
    )?);
    let split = split.trim().to_string();
    assert!(split.starts_with('%'));
    assert_ne!(primary, split);
    let split_layout = control.wait_event("layout after split", WAIT, |event| {
        matches!(event, ControlEvent::LayoutChange { .. })
    })?;
    assert!(matches!(split_layout, ControlEvent::LayoutChange { .. }));

    send_marker(&mut control, &primary, "primary-marker")?;
    assert_output(&control, &primary, b"primary-marker")?;
    send_marker(&mut control, &split, "secondary-marker")?;
    assert_output(&control, &split, b"secondary-marker")?;

    assert_success(control.command(&format!("kill-pane -t {split}"), WAIT)?)?;
    let restored_layout = control.wait_event("layout after pane removal", WAIT, |event| {
        matches!(event, ControlEvent::LayoutChange { .. })
    })?;
    assert!(matches!(restored_layout, ControlEvent::LayoutChange { .. }));
    let remaining = command_text(&control.command("list-panes -F \"#{pane_id}\"", WAIT)?);
    assert_eq!(remaining.trim(), primary);
    Ok(())
}

#[test]
fn control_wait_timeout_writes_live_transcript() -> anyhow::Result<()> {
    if !TmuxServer::should_run("control timeout transcript")? {
        return Ok(());
    }
    let root = tempfile::tempdir()?;
    let server = TmuxServer::start_with_artifact_root(
        "control_wait_timeout_writes",
        root.path().to_path_buf(),
    )?;
    let artifact_dir = server.artifact_dir();
    let session = server.new_session(SessionSpec::new(
        "control-timeout",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        CommandSpec::new("cat"),
    ))?;
    let control = session.attach_control(WAIT)?;
    let error = control
        .wait_event(
            "impossible control event",
            Duration::from_millis(/*millis*/ 50),
            |_| false,
        )
        .expect_err("impossible event should time out")
        .to_string();

    assert!(error.contains("timed out waiting for impossible control event"));
    assert!(
        std::fs::read_to_string(artifact_dir.join("control-transcript.txt"))?.contains("%begin")
    );
    assert!(artifact_dir.join("manifest.json").is_file());
    Ok(())
}

fn unknown_item(retained_bytes: usize) -> ParsedItem {
    ParsedItem {
        retained_bytes,
        value: ControlItem::Event(ControlEvent::Unknown {
            name: "%fixture".into(),
            arguments: Vec::new(),
        }),
    }
}

fn send_marker(
    control: &mut TmuxControlClient<'_>,
    pane_id: &str,
    marker: &str,
) -> anyhow::Result<()> {
    assert_success(control.command(&format!("send-keys -l -t {pane_id} {marker}"), WAIT)?)?;
    assert_success(control.command(&format!("send-keys -t {pane_id} Enter"), WAIT)?)?;
    Ok(())
}

fn assert_output(
    control: &TmuxControlClient<'_>,
    pane_id: &str,
    marker: &[u8],
) -> anyhow::Result<()> {
    let event = control.wait_event("pane-specific marker", WAIT, |event| {
        matches!(
            event,
            ControlEvent::Output {
                pane_id: actual,
                data,
            } if actual == pane_id && data.windows(marker.len()).any(|window| window == marker)
        )
    })?;
    assert!(matches!(event, ControlEvent::Output { .. }));
    Ok(())
}

fn assert_success(result: CommandResult) -> anyhow::Result<()> {
    anyhow::ensure!(
        result.status == CommandStatus::Success,
        "tmux control command failed: {}",
        command_output(&result)
    );
    Ok(())
}

fn command_text(result: &CommandResult) -> String {
    result
        .output
        .iter()
        .map(|line| String::from_utf8_lossy(line))
        .collect::<Vec<_>>()
        .join("\n")
}
