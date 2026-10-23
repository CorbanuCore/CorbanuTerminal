//! #613: kill switch follow-ups from the #602 review (owner decisions of
//! 2026-10-11 and 2026-10-12, #507: nothing restarts after switch-off).

use super::*;
use crate::legacy_core::security_revocation::HumanRevocation;
use pretty_assertions::assert_eq;

/// The dispatch reports the sender (`source`) received, in order.
fn reports_to(app: &App, source: &str) -> Vec<String> {
    app.spawn_parent_reports_by_node
        .get(source)
        .into_iter()
        .flatten()
        .cloned()
        .collect()
}

/// A dispatch ack for `task` sent by `source` to the Claude pane `pane_id`.
fn register_ack(app: &mut App, source: &str, pane_id: &str, task: &str, seq: u64) {
    let target = crate::spawn_orchestration::pane_node_id(pane_id);
    app.register_spawn_dispatch_acks_for_task(
        &target,
        task,
        vec![crate::dispatch_queue::SpawnDispatchAck {
            seq,
            source_node_id: source.to_string(),
            target_node_id: target.clone(),
            target_title: pane_id.to_string(),
            origin_id: None,
            attempt: 0,
        }],
    );
}

/// #613 (test gap): a child's report to a native (Codex) parent while the
/// switch is on reaches the parent's mailbox without asking for a turn.
#[tokio::test]
async fn issue_613_report_to_a_codex_parent_while_on_asks_for_no_turn() {
    let (mut app, mut app_event_rx, _troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let parent_thread =
        ThreadId::from_string("00000000-0000-0000-0000-000000000613").expect("thread id");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&orc_pane_id),
        crate::spawn_orchestration::thread_node_id(parent_thread),
    );
    let mail = |rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>| {
        std::iter::from_fn(|| rx.try_recv().ok())
            .filter_map(|event| match event {
                AppEvent::SendSpawnAgentMailboxMessage { params } => Some(params.trigger_turn),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    while app_event_rx.try_recv().is_ok() {}

    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));
    assert_eq!(mail(&mut app_event_rx), vec![false]);

    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("more text"));
    assert_eq!(mail(&mut app_event_rx), vec![true], "control: switch off");
}

/// #613 (test gap): the same task sent again while it waits is the same
/// task: neither sender is told it was superseded, and both are told it was
/// delivered with the pane's next turn.
#[tokio::test]
async fn issue_613_the_same_task_sent_again_while_it_waits_is_delivered_once() {
    let (mut app, _app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let source = crate::spawn_orchestration::pane_node_id(&troll_pane_id);
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    register_ack(
        &mut app,
        &source,
        &orc_pane_id,
        "pf613 task",
        /*seq*/ 6131,
    );
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf613 task".to_string());
    register_ack(
        &mut app,
        &source,
        &orc_pane_id,
        "pf613 task",
        /*seq*/ 6132,
    );
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf613 task".to_string());
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(orc_pane_id.clone(), "pf613 task".to_string())]
    );
    assert!(
        reports_to(&app, &source).is_empty(),
        "{:?}",
        reports_to(&app, &source)
    );

    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.on_kill_switch_turned_off();
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf613 next".to_string());
    assert!(app.kill_switch_held_claude_tasks.is_empty());
    let prompt = &app.claude_panes.prepared_prompts[0].1;
    assert_eq!(prompt.matches("pf613 task").count(), 1, "{prompt}");
    let reports = reports_to(&app, &source);
    assert!(
        reports.iter().all(|report| !report.contains("superseded")),
        "{reports:?}"
    );
    for seq in [6131, 6132] {
        assert!(
            reports
                .iter()
                .any(|report| report.contains(&format!("#{seq};"))
                    && report.contains("status=delivered")),
            "{reports:?}"
        );
    }
}

/// #613 (test gap): the sender of a task for a Claude pane that does not
/// exist is told, while the switch is on.
#[tokio::test]
async fn issue_613_the_sender_of_a_task_for_a_missing_pane_is_told() {
    let (mut app, _app_event_rx, troll_pane_id, _orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let source = crate::spawn_orchestration::pane_node_id(&troll_pane_id);
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    register_ack(
        &mut app,
        &source,
        "claude-missing",
        "pf613 task",
        /*seq*/ 6133,
    );
    app.submit_claude_pane_task("claude-missing".to_string(), "pf613 task".to_string());
    assert!(app.kill_switch_held_claude_tasks.is_empty());
    let reports = reports_to(&app, &source);
    assert!(
        reports.iter().any(|report| report.contains("#6133;")
            && report.contains("status=failed")
            && report.contains("does not exist")),
        "{reports:?}"
    );
}

/// #613: a whip a person resumes while the switch is on is not resumed (they
/// are told to resume it after turning the switch off), so it cannot fire
/// after the switch is turned off unless someone resumes it then; one
/// attached while on is paused at once.
#[tokio::test]
async fn issue_613_a_whip_resumed_while_on_stays_paused() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "keep-going", "# whip: keep-going\nContinue the work.");
    let pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create pane");
    app.handle_orchestrate_command(format!(
        "attach {pane_id} keep-going --mode auto --holder none --max 3"
    ));
    while app_event_rx.try_recv().is_ok() {}
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.sweep_orchestrate_whips();

    app.handle_orchestrate_command("resume whip-1".to_string());
    assert_eq!(
        app.orchestrate_whips.get("whip-1").map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused),
        "not resumed while on"
    );
    app.handle_orchestrate_command(format!(
        "attach {pane_id} keep-going --mode auto --holder none --max 3"
    ));
    assert_eq!(
        app.orchestrate_whips.get("whip-2").map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused),
        "attached while on: paused at once"
    );
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.sweep_orchestrate_whips();
    assert!(
        drain_claude_pane_task_events(&mut app_event_rx).is_empty(),
        "switch-off fires nothing"
    );
}

/// #613: a task another agent sent while the switch was on, handled only
/// after it was turned off, waits for the pane's next turn instead of
/// starting one (the event queue's millisecond window).
#[tokio::test]
async fn issue_613_a_task_sent_while_on_and_handled_after_off_waits() {
    let (mut app, _app_event_rx, _troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    // What an agent path stamps on the event while the switch is on.
    let sent_while_on = app.kill_switch_on_now();
    assert!(sent_while_on);
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.submit_claude_pane_task_queued(
        orc_pane_id.clone(),
        "pf613 queued".to_string(),
        sent_while_on,
    );
    assert!(
        app.claude_panes.prepared_prompts.is_empty(),
        "no turn started"
    );
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(orc_pane_id.clone(), "pf613 queued".to_string())]
    );
    // Control: a task sent after the switch was off starts its turn (and
    // takes the waiting one with it).
    app.submit_claude_pane_task(orc_pane_id, "pf613 after".to_string());
    assert_eq!(app.claude_panes.prepared_prompts.len(), 1);
    assert!(app.kill_switch_held_claude_tasks.is_empty());
}

/// #613: through the event handler, a task sent while the switch was on for
/// a Claude pane that is still busy stopping waits instead of being refused
/// as busy, also when handled after switch-off.
#[tokio::test]
async fn issue_613_the_handler_holds_a_task_sent_while_on_for_a_busy_pane() -> Result<()> {
    let (mut app, _app_event_rx, _troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let config = app.chat_widget.config_ref().clone();
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(&config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    let codex_home = app.config.codex_home.to_path_buf();
    let _busy = app
        .claude_panes
        .prepare_turn(&orc_pane_id, "busy".to_string(), &codex_home)
        .expect("a running turn");
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    let sent_while_kill_switch_on = app.kill_switch_on_now();
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    let event = AppEvent::SubmitSpawnClaudePaneTask {
        pane_id: orc_pane_id.clone(),
        task: "pf613 busy".to_string(),
        sent_while_kill_switch_on,
    };
    Box::pin(app.handle_event(&mut tui, &mut app_server, event)).await?;
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(orc_pane_id.clone(), "pf613 busy".to_string())]
    );
    assert_eq!(
        app.claude_panes.prepared_prompts.len(),
        1,
        "only the busy turn"
    );
    Ok(())
}
