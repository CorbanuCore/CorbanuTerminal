use std::panic::catch_unwind;
use std::time::Duration;

use anyhow::Result;

use super::*;

fn server_or_skip() -> Result<Option<TmuxServer>> {
    if !TmuxServer::is_available() {
        eprintln!("skipping tmux harness test because tmux is unavailable");
        return Ok(None);
    }
    Ok(Some(TmuxServer::start()?))
}

#[test]
fn servers_are_isolated_and_cleanup_their_private_sessions() -> Result<()> {
    let Some(first) = server_or_skip()? else {
        return Ok(());
    };
    let Some(second) = server_or_skip()? else {
        return Ok(());
    };
    let first_socket_root = first.socket_root();
    let second_socket_root = second.socket_root();
    let first_session = first.new_session(SessionSpec::new(
        "isolation",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("printf 'first\\n'; sleep 30"),
    ))?;
    let second_session = second.new_session(SessionSpec::new(
        "isolation",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("printf 'second\\n'; sleep 30"),
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
    let Some(server) = server_or_skip()? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "input",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("IFS= read -r line; printf 'LINE:%s\\n' \"$line\"; sleep 30"),
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
fn viewport_and_scrollback_are_captured_separately() -> Result<()> {
    let Some(server) = server_or_skip()? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "capture",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 5),
        command_for_shell(
            "printf 'old-line\\n'; i=1; while [ $i -le 12 ]; do printf 'line-%02d\\n' $i; i=$((i + 1)); done; sleep 30",
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
fn command_failures_include_diagnostics() -> Result<()> {
    let Some(server) = server_or_skip()? else {
        return Ok(());
    };
    let mut command = server.command();
    let error = checked_output(command.arg("display-message").arg("-t").arg("missing-pane"))
        .expect_err("missing pane should fail")
        .to_string();

    assert!(error.contains("tmux command failed"));
    assert!(error.contains("display-message"));
    assert!(error.contains("status:"));
    assert!(error.contains("stdout:"));
    assert!(error.contains("stderr:"));
    Ok(())
}

#[test]
fn wait_failures_include_the_last_live_viewport() -> Result<()> {
    let Some(server) = server_or_skip()? else {
        return Ok(());
    };
    let session = server.new_session(SessionSpec::new(
        "timeout",
        TerminalSize::new(/*columns*/ 40, /*rows*/ 8),
        command_for_shell("printf 'visible-sentinel\\n'; sleep 30"),
    ))?;
    let error = session
        .primary_pane()
        .wait_stable_contains("missing-sentinel", Duration::from_millis(/*millis*/ 250))
        .expect_err("missing sentinel should time out")
        .to_string();

    assert!(error.contains("missing-sentinel"));
    assert!(error.contains("last viewport:"));
    assert!(error.contains("visible-sentinel"));
    Ok(())
}

#[test]
fn panic_unwind_cleans_up_the_private_server() -> Result<()> {
    let Some(server) = server_or_skip()? else {
        return Ok(());
    };
    let socket_root = server.socket_root();
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
    Ok(())
}
