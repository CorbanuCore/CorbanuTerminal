use std::panic::catch_unwind;
use std::time::Duration;

use anyhow::Result;

use super::*;

fn server_or_skip(scenario: &str) -> Result<Option<TmuxServer>> {
    if !TmuxServer::should_run(scenario)? {
        return Ok(None);
    }
    Ok(Some(TmuxServer::start(scenario)?))
}

#[test]
fn servers_are_isolated_and_cleanup_their_private_sessions() -> Result<()> {
    let Some(first) = server_or_skip("servers_are_isolated")? else {
        return Ok(());
    };
    let Some(second) = server_or_skip("servers_are_isolated")? else {
        return Ok(());
    };
    let first_socket_root = first.socket_root();
    let second_socket_root = second.socket_root();
    let first_session = first.new_session(SessionSpec::new(
        "isolation",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("printf 'first\n'; sleep 30"),
    ))?;
    let second_session = second.new_session(SessionSpec::new(
        "isolation",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("printf 'second\n'; sleep 30"),
    ))?;

    assert!(first.has_session(&first_session.name));
    assert!(!first.has_session(&second_session.name));
    assert!(second.has_session(&second_session.name));
    assert!(!second.has_session(&first_session.name));

    drop(first_session);
    drop(second_session);
    drop(first);
    drop(second);
    assert!(!first_socket_root.exists());
    assert!(!second_socket_root.exists());
    Ok(())
}

#[test]
fn literal_text_is_distinct_from_named_enter_key() -> Result<()> {
    let Some(server) = server_or_skip("literal_text_is_distinct")? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "input",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("IFS= read -r line; printf 'LINE:%s\n' \"$line\"; sleep 30"),
    ))?;
    let pane = session.primary_pane();

    pane.send_literal("Enter")?;
    let before_key = pane.capture_viewport()?;
    assert!(!before_key.contains("LINE:Enter"));
    pane.send_key(TmuxKey::Enter)?;
    let after_key = pane.wait_stable_contains("LINE:Enter", Duration::from_secs(/*secs*/ 3))?;
    assert!(after_key.contains("LINE:Enter"));
    Ok(())
}

#[test]
fn paste_input_emits_expected_bracketed_paste_bytes() -> Result<()> {
    let Some(server) = server_or_skip("paste_input_emits_expected_bracketed_paste_bytes")? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "paste-input",
        TerminalSize::new(/*columns*/ 80, /*rows*/ 8),
        command_for_shell(
            "stty raw -echo; printf 'READY\r\n'; dd bs=1 count=21 2>/dev/null | od -An -tx1; sleep 30",
        ),
    ))?;
    let pane = session.primary_pane();

    pane.wait_stable_contains("READY", Duration::from_secs(/*secs*/ 3))?;
    pane.send_paste("image.png")?;
    // The true-TUI large-image test is the end-to-end gate that crossterm emits
    // one paste event; this unit test pins the exact bytes delivered to the PTY.
    let expected = "1b 5b 32 30 30 7e 69 6d 61 67 65 2e 70 6e 67 1b 5b 32 30 31 7e";
    pane.wait_stable_until(
        "one bracketed paste byte sequence",
        Duration::from_secs(/*secs*/ 3),
        |capture| {
            capture
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .contains(expected)
        },
    )?;
    Ok(())
}

#[test]
fn paste_input_rejects_embedded_escape_character() -> Result<()> {
    let Some(server) = server_or_skip("paste_input_rejects_embedded_escape_character")? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "paste-input-escape",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("sleep 30"),
    ))?;

    let error = session
        .primary_pane()
        .send_paste("before\u{1b}[201~after")
        .expect_err("embedded paste terminator must be rejected")
        .to_string();
    assert!(error.contains("must not contain an escape character"));
    Ok(())
}

#[test]
fn viewport_and_scrollback_are_captured_separately() -> Result<()> {
    let Some(server) = server_or_skip("viewport_and_scrollback")? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "capture",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 5),
        command_for_shell(
            "printf 'old-line\n'; i=1; while [ $i -le 12 ]; do printf 'line-%02d\n' $i; i=$((i + 1)); done; sleep 30",
        ),
    ))?;
    let pane = session.primary_pane();

    let viewport = pane.wait_stable_contains("line-12", Duration::from_secs(/*secs*/ 3))?;
    let scrollback = pane.capture_scrollback_tail(/*lines*/ 20)?;
    assert!(!viewport.contains("old-line"));
    assert!(scrollback.contains("old-line"));
    assert!(scrollback.contains("line-12"));
    Ok(())
}

#[test]
fn command_failures_include_diagnostics_and_artifacts() -> Result<()> {
    if !TmuxServer::should_run("command failure artifacts")? {
        return Ok(());
    }
    let artifact_root = tempfile::tempdir()?;
    let server = TmuxServer::start_with_artifact_root(
        "command_failures_include_diagnostics",
        artifact_root.path().to_path_buf(),
    )?;
    let artifact_dir = server.artifact_dir();
    let mut command = server.command();
    let error = server
        .checked_output(
            command.arg("display-message").arg("-t").arg("missing-pane"),
            /*pane_id*/ None,
        )
        .expect_err("missing pane should fail")
        .to_string();

    for expected in [
        "tmux command failed",
        "display-message",
        "status:",
        "stdout:",
        "stderr:",
        "reproduction:",
    ] {
        assert!(error.contains(expected));
    }
    assert!(artifact_dir.join("manifest.json").is_file());
    Ok(())
}

#[test]
fn wait_failures_write_complete_redacted_artifacts() -> Result<()> {
    if !TmuxServer::should_run("timeout artifacts")? {
        return Ok(());
    }
    let artifact_root = tempfile::tempdir()?;
    let attachment_root = tempfile::tempdir()?;
    let config = attachment_root.path().join("config.toml");
    let candidate_log = attachment_root.path().join("candidate.log");
    std::fs::write(&config, "model = \"test\"\n")?;
    std::fs::write(&candidate_log, "candidate fixture log\n")?;
    let server = TmuxServer::start_with_artifact_root(
        "wait_failures_write_complete",
        artifact_root.path().to_path_buf(),
    )?;
    server.register_artifact("config.toml", &config);
    server.register_artifact("candidate.log", &candidate_log);
    let artifact_dir = server.artifact_dir();
    let session = server.new_session(SessionSpec::new(
        "timeout",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        CommandSpec::new("sh")
            .env("OPENAI_API_KEY", "must-not-leak")
            .arg("-c")
            .arg("IFS= read -r line; printf 'visible:%s\\n' \"$line\"; sleep 30"),
    ))?;
    let pane = session.primary_pane();
    pane.send_literal("ok")?;
    pane.send_key(TmuxKey::Enter)?;
    pane.wait_stable_contains("visible:ok", Duration::from_secs(/*secs*/ 3))?;
    let error = pane
        .wait_stable_contains("missing-sentinel", Duration::from_millis(/*millis*/ 250))
        .expect_err("missing sentinel should time out")
        .to_string();

    assert!(error.contains("missing-sentinel"));
    assert!(error.contains("last viewport:"));
    assert!(error.contains("visible:ok"));
    let mut files = std::fs::read_dir(&artifact_dir)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().to_string()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.sort();
    assert_eq!(
        files,
        vec![
            "candidate.log",
            "command-log.txt",
            "config.toml",
            "dimensions.txt",
            "input-events.txt",
            "manifest.json",
            "pane-metadata.txt",
            "reason.txt",
            "reproduce.sh",
            "scrollback.txt",
            "viewport.txt",
        ]
    );
    let command_log = std::fs::read_to_string(artifact_dir.join("command-log.txt"))?;
    assert!(command_log.contains("OPENAI_API_KEY=<redacted>"));
    assert!(!command_log.contains("must-not-leak"));
    assert_eq!(
        std::fs::read_to_string(artifact_dir.join("input-events.txt"))?,
        "literal bytes=2\nkey Enter"
    );
    assert_eq!(
        std::fs::read_to_string(artifact_dir.join("reproduce.sh"))?,
        "CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all \
         wait_failures_write_complete"
    );
    assert!(std::fs::read_to_string(artifact_dir.join("pane-metadata.txt"))?.contains("size=40x8"));
    Ok(())
}

#[test]
fn successful_session_does_not_emit_artifacts() -> Result<()> {
    if !TmuxServer::should_run("success artifact laziness")? {
        return Ok(());
    }
    let artifact_root = tempfile::tempdir()?;
    let server = TmuxServer::start_with_artifact_root(
        "successful_session_does_not_emit",
        artifact_root.path().to_path_buf(),
    )?;
    let artifact_dir = server.artifact_dir();
    let _session = server.new_session(SessionSpec::new(
        "success",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("sleep 30"),
    ))?;
    assert!(!artifact_dir.exists());
    Ok(())
}

#[test]
fn panic_unwind_cleans_up_and_writes_artifacts() -> Result<()> {
    if !TmuxServer::should_run("panic cleanup artifacts")? {
        return Ok(());
    }
    let artifact_root = tempfile::tempdir()?;
    let server = TmuxServer::start_with_artifact_root(
        "panic_unwind_cleans_up",
        artifact_root.path().to_path_buf(),
    )?;
    let socket_root = server.socket_root();
    let artifact_dir = server.artifact_dir();
    let result = catch_unwind(move || {
        let _session = server
            .new_session(SessionSpec::new(
                "panic",
                TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
                command_for_shell("sleep 30"),
            ))
            .expect("session should start");
        panic!("intentional unwind");
    });

    assert!(result.is_err());
    assert!(!socket_root.exists());
    assert!(artifact_dir.join("manifest.json").is_file());
    assert!(
        std::fs::read_to_string(artifact_dir.join("reason.txt"))?
            .contains("panicked while tmux session was active")
    );
    Ok(())
}
