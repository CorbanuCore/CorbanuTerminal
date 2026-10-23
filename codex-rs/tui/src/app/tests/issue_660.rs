//! #660: the kill switch's event-queue window for Codex panes (#613
//! follow-up; #507: nothing restarts after switch-off, Claude panes behave
//! like Codex panes).

use super::*;
use crate::legacy_core::security_revocation::HumanRevocation;
use pretty_assertions::assert_eq;

/// The `trigger_turn` of each native report mailbox message sent.
fn report_turns(rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>) -> Vec<bool> {
    std::iter::from_fn(|| rx.try_recv().ok())
        .filter_map(|event| match event {
            AppEvent::SendSpawnAgentMailboxMessage { params } => Some(params.trigger_turn),
            _ => None,
        })
        .collect()
}

/// A dispatch ack for `task` from the native agent `sender` to `target`.
fn register_native_ack(app: &mut App, sender: ThreadId, target: &str, task: &str, seq: u64) {
    app.register_spawn_dispatch_acks_for_task(
        target,
        task,
        vec![crate::dispatch_queue::SpawnDispatchAck {
            seq,
            source_node_id: crate::spawn_orchestration::thread_node_id(sender),
            target_node_id: target.to_string(),
            target_title: "target".to_string(),
            origin_id: None,
            attempt: 0,
        }],
    );
}

/// #660 (same window, Claude side of #613): a native agent's task for a
/// Claude pane that does not exist, sent while the switch was on and handled
/// after it was turned off, is reported failed without starting a turn in
/// the sender.
#[tokio::test]
async fn issue_660_held_claude_task_failure_starts_no_sender_turn() {
    let (mut app, mut app_event_rx, _troll_pane_id, _orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let sender = ThreadId::from_string("00000000-0000-0000-0000-000000006603").expect("id");
    let target = crate::spawn_orchestration::pane_node_id("claude-missing");
    register_native_ack(&mut app, sender, &target, "pf660 claude", /*seq*/ 6603);
    while app_event_rx.try_recv().is_ok() {}

    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    let sent_while_kill_switch_on = app.kill_switch_on_now();
    assert!(sent_while_kill_switch_on);
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.submit_claude_pane_task_queued(
        "claude-missing".to_string(),
        "pf660 claude".to_string(),
        sent_while_kill_switch_on,
    );
    assert_eq!(report_turns(&mut app_event_rx), vec![false]);
}

/// #660 (round 2): ordinary failures of tasks sent while the switch was on,
/// handled after it was turned off, tell the native sender without starting
/// a turn in it: a task for a managed pane of a crew that is read-only, and an
/// oversize Claude-pane task. (A task for a `/panes` pane goes to its mailbox
/// like any agent's task, D106.)
#[tokio::test]
async fn issue_660_failures_of_tasks_sent_while_on_start_no_sender_turn() -> Result<()> {
    let (mut app, mut app_event_rx, mut op_rx) = make_test_app_with_channels().await;
    let config = app.chat_widget.config_ref().clone();
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(&config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    let managed = ThreadId::from_string("00000000-0000-0000-0000-000000006605")?;
    let sender = ThreadId::from_string("00000000-0000-0000-0000-000000006606")?;
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    let sent_while_kill_switch_on = app.kill_switch_on_now();
    assert!(sent_while_kill_switch_on);
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    let oversize = "x".repeat(crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES + 1);

    for sent in [sent_while_kill_switch_on, false] {
        let managed_target = app.logical_native_node_for_thread(managed);
        register_native_ack(
            &mut app,
            sender,
            &managed_target,
            "pf660 managed",
            /*seq*/ 6605,
        );
        let claude_target = crate::spawn_orchestration::pane_node_id("claude-660");
        register_native_ack(
            &mut app,
            sender,
            &claude_target,
            &oversize,
            /*seq*/ 6606,
        );
        while app_event_rx.try_recv().is_ok() {}

        app.spawn_legacy_read_only = true;
        Box::pin(app.handle_event(
            &mut tui,
            &mut app_server,
            AppEvent::SubmitSpawnAgentTask {
                thread_id: managed,
                task: "pf660 managed".to_string(),
                sent_while_kill_switch_on: sent,
            },
        ))
        .await?;
        app.spawn_legacy_read_only = false;
        Box::pin(app.handle_event(
            &mut tui,
            &mut app_server,
            AppEvent::SubmitSpawnClaudePaneTask {
                pane_id: "claude-660".to_string(),
                task: oversize.clone(),
                sent_while_kill_switch_on: sent,
            },
        ))
        .await?;
        assert!(op_rx.try_recv().is_err(), "no user turn was submitted");
        // Control (unstamped): the same failures report with a turn.
        assert_eq!(
            report_turns(&mut app_event_rx),
            vec![!sent; 2],
            "stamped: {sent}"
        );
    }
    Ok(())
}

/// #660 follow-up: a held task that reaches a pane while it runs a turn goes
/// with that turn, so the pane is not listed as having a waiting task; an
/// idle pane is, until its next turn starts or the session is reset.
#[tokio::test]
async fn issue_660_running_pane_takes_its_held_task_and_is_not_listed() -> Result<()> {
    let mut app = make_test_app().await;
    let pane = ThreadId::from_string("00000000-0000-0000-0000-000000006607")?;
    app.upsert_agent_picker_thread(
        pane,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.agent_navigation.set_running(pane, /*is_running*/ true);
    app.note_kill_switch_held_native_task(pane);
    assert_eq!(app.kill_switch_held_native_tasks, Vec::<ThreadId>::new());

    app.agent_navigation.set_running(pane, /*is_running*/ false);
    app.note_kill_switch_held_native_task(pane);
    app.note_kill_switch_held_native_task(pane);
    assert_eq!(app.kill_switch_held_native_tasks, vec![pane]);
    app.clear_kill_switch_held_native_task(pane);
    assert_eq!(app.kill_switch_held_native_tasks, Vec::<ThreadId>::new());
    Ok(())
}
