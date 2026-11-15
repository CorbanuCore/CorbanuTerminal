//! Corbanu app tests (crews, panes, Claude panes, spawn orchestration, named accounts, providers,
//! GPU and plan models): moved out of `app/tests.rs` to keep it under the 512,000-byte blob limit.

use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn collab_receiver_notification_caches_result_preview() {
    let mut app = make_test_app().await;
    let receiver_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000125").expect("valid thread id");

    app.upsert_agent_picker_thread(
        receiver_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        ServerNotification::ItemCompleted(codex_app_server_protocol::ItemCompletedNotification {
            thread_id: ThreadId::new().to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: ThreadItem::untimed(ThreadItemPayload::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: codex_app_server_protocol::CollabAgentTool::Wait,
                status: codex_app_server_protocol::CollabAgentToolCallStatus::Completed,
                sender_thread_id: ThreadId::new().to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: None,
                reasoning_effort: None,
                provider_account: None,
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    codex_app_server_protocol::CollabAgentState {
                        status: codex_app_server_protocol::CollabAgentStatus::Completed,
                        message: Some("created animated proof card".to_string()),
                    },
                )]),
            }),
        }),
    )));

    assert_eq!(
        app.agent_navigation
            .get(&receiver_thread_id)
            .and_then(|entry| entry.last_result_message.as_deref()),
        Some("created animated proof card")
    );
}

#[tokio::test]
async fn native_followup_dispatch_transitions_only_the_matching_assignment() {
    let mut app = make_test_app().await;
    write_test_whip(
        &app,
        "native-assignment",
        "# assignment: native-assignment\nComplete the worker task.",
    );
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000611").expect("manager id");
    let first_worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000612").expect("worker id");
    let second_worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000613").expect("worker id");
    let unrelated_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000614").expect("unrelated id");
    for (thread_id, name) in [
        (manager_thread_id, "Manager"),
        (first_worker_thread_id, "First Worker"),
        (second_worker_thread_id, "Second Worker"),
        (unrelated_thread_id, "Unrelated"),
    ] {
        app.upsert_agent_picker_thread(
            thread_id,
            Some(name.to_string()),
            /*agent_role*/ None,
            /*is_closed*/ false,
        );
    }
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let first_worker_node = crate::spawn_orchestration::thread_node_id(first_worker_thread_id);
    let second_worker_node = crate::spawn_orchestration::thread_node_id(second_worker_thread_id);
    app.handle_orchestrate_command(format!(
        "attach {first_worker_node} native-assignment --mode review --holder {manager_node} --for 8h"
    ));
    app.handle_orchestrate_command(format!(
        "attach {second_worker_node} native-assignment --mode review --holder {manager_node} --for 8h"
    ));

    let completed_send = |receiver_thread_id: ThreadId, call_id: &str| {
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: manager_thread_id.to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: ThreadItem::untimed(ThreadItemPayload::SubAgentActivity {
                id: call_id.to_string(),
                kind: codex_app_server_protocol::SubAgentActivityKind::Interacted,
                agent_thread_id: receiver_thread_id.to_string(),
                agent_path: format!("/root/worker-{receiver_thread_id}"),
                agent_nickname: None,
                agent_role: None,
                task_preview: None,
                model: None,
                reasoning_effort: None,
            }),
        })
    };

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(completed_send(
        unrelated_thread_id,
        "send-unrelated",
    ))));
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Drafting,
            ..
        })
    ));
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-2")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Drafting,
            ..
        })
    ));

    // Deliberately dispatch to the other active assignment than the legacy holder-only lookup
    // returns. Native delivery identifies both endpoints and must not collapse multiple
    // assignments managed by the same durable thread onto an arbitrary HashMap entry.
    let holder_only_target = app
        .assignment_dispatch_target_for_holder(&manager_node)
        .expect("one active assignment")
        .1;
    let (matching_thread_id, matching_assignment_id, other_assignment_id) =
        if holder_only_target == first_worker_node {
            (second_worker_thread_id, "assignment-2", "assignment-1")
        } else {
            (first_worker_thread_id, "assignment-1", "assignment-2")
        };

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(completed_send(
        matching_thread_id,
        "send-worker",
    ))));
    let assignment = app
        .orchestrate_whips
        .get(matching_assignment_id)
        .expect("assignment should remain registered");
    assert!(matches!(
        assignment.kind,
        crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            execution_started_utc: Some(_),
            ..
        }
    ));
    assert_eq!(
        assignment.last_dispatch_result.as_deref(),
        Some("delivered")
    );
    let other_assignment = app
        .orchestrate_whips
        .get(other_assignment_id)
        .expect("other assignment should remain registered");
    assert!(matches!(
        other_assignment.kind,
        crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Drafting,
            ..
        }
    ));
    assert_eq!(other_assignment.last_dispatch_result, None);
}

#[tokio::test]
async fn native_direct_parent_assignment_uses_core_result_delivery_without_duplicate_mandate() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "native-parent-assignment",
        "# assignment: native-parent-assignment\nComplete the worker task.",
    );
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000621").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000622").expect("worker id");
    for (thread_id, name) in [(manager_thread_id, "Manager"), (worker_thread_id, "Worker")] {
        app.upsert_agent_picker_thread(
            thread_id,
            Some(name.to_string()),
            /*agent_role*/ None,
            /*is_closed*/ false,
        );
    }
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);
    app.spawn_parent_by_node
        .insert(worker_node.clone(), manager_node.clone());
    app.handle_orchestrate_command(format!(
        "attach {worker_node} native-parent-assignment --mode review --holder {manager_node} --for 8h"
    ));
    while app_event_rx.try_recv().is_ok() {}
    app.note_whip_holder_dispatched(&manager_node, &worker_node);

    app.note_whip_target_idle_with_fire_control(
        &worker_node,
        Some("NATIVE_PARENT_RESULT"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );

    assert!(
        drain_spawn_agent_tasks_for(&mut app_event_rx, manager_thread_id).is_empty(),
        "Core already delivers a direct native child's result to its parent Manager"
    );
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .and_then(|whip| whip.last_target_output.as_deref()),
        Some("NATIVE_PARENT_RESULT")
    );

    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("WHIP_DONE"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Done,
            ..
        })
    ));
}

#[tokio::test]
async fn spawn_status_shows_orc_task_preview_from_troll_activity() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000126").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000127").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000128").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);

    app.agent_navigation
        .record_sub_agent_activity(SubAgentActivityDisplay {
            thread_id: orc_thread_id,
            agent_path: "/root/troll_burzum/orc_snaga".to_string(),
            agent_nickname: None,
            agent_role: None,
            task_preview: Some("build the animated website shell".to_string()),
            is_running_hint: true,
        });

    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    assert!(status_items.iter().any(|item| {
        item.name.contains("Snaga [orc]")
            && item
                .description
                .as_deref()
                .is_some_and(|description| description.contains("build the animated website shell"))
    }));
}

#[tokio::test]
async fn pane_spawn_tree_hides_task_actions() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000228").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000229").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000230").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);

    let pane_items = app.spawn_tree_items(/*show_task_actions*/ false);
    assert!(
        pane_items
            .iter()
            .any(|item| item.name.contains("Burzum [troll]"))
    );
    assert!(
        pane_items
            .iter()
            .any(|item| item.name.contains("Snaga [orc]"))
    );
    assert!(
        pane_items
            .iter()
            .all(|item| !item.name.contains("Send task to")),
        "/panes should only switch panes, not show dispatch actions"
    );

    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    assert!(
        status_items
            .iter()
            .any(|item| item.name.contains("Send task to Burzum [troll]"))
    );
    assert!(
        status_items
            .iter()
            .any(|item| item.name.contains("Send task to Snaga [orc]"))
    );

    let troll_task_item = status_items
        .iter()
        .find(|item| item.name.contains("Send task to Burzum [troll]"))
        .expect("Burzum task action row");
    assert_eq!(troll_task_item.actions.len(), 1);
    (troll_task_item.actions[0])(&app.app_event_tx);
    match rx
        .try_recv()
        .expect("Burzum task action should emit an event")
    {
        AppEvent::OpenSpawnAgentTaskPrompt { thread_id } => assert_eq!(thread_id, troll_thread_id),
        event => panic!("expected Burzum task prompt event, got {event:?}"),
    }

    let orc_task_item = status_items
        .iter()
        .find(|item| item.name.contains("Send task to Snaga [orc]"))
        .expect("Snaga task action row");
    assert_eq!(orc_task_item.actions.len(), 1);
    (orc_task_item.actions[0])(&app.app_event_tx);
    match rx
        .try_recv()
        .expect("Snaga task action should emit an event")
    {
        AppEvent::OpenSpawnAgentTaskPrompt { thread_id } => assert_eq!(thread_id, orc_thread_id),
        event => panic!("expected Snaga task prompt event, got {event:?}"),
    }
}

/// D95 (#1146): only the person's Spawn status prompt sends a task as the
/// person's own input; the route agents' tasks take (assignment fires,
/// model dispatch) still sends them as inter-agent messages.
#[tokio::test]
async fn spawn_status_prompt_alone_sends_person_tasks() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000001146").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000001147").expect("valid thread id");
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, main_thread_id);

    (crate::spawn_orchestration::spawn_agent_task_prompt_submit(
        app.app_event_tx.clone(),
        orc_thread_id,
    ))("typed by the person".to_string());
    match rx.try_recv().expect("the prompt sends an event") {
        AppEvent::SubmitSpawnAgentTaskFromPerson { thread_id, task } => {
            assert_eq!(thread_id, orc_thread_id);
            assert_eq!(task, "typed by the person");
        }
        event => panic!("expected the person's task, got {event:?}"),
    }

    app.send_native_task_event(orc_thread_id, "sent by an agent".to_string());
    match rx.try_recv().expect("an agent's task sends an event") {
        AppEvent::SubmitSpawnAgentTask {
            thread_id, task, ..
        } => {
            assert_eq!(thread_id, orc_thread_id);
            assert_eq!(task, "sent by an agent");
        }
        event => panic!("expected an agent's task, got {event:?}"),
    }
}

/// D106 (#1176): an agent's task for a pane the person opened takes the
/// mailbox route, as for an agent-opened pane; it is never sent as the
/// person's input. The person's own Spawn status task still is (D95).
#[tokio::test]
async fn agent_task_for_a_person_opened_pane_takes_the_mailbox_route() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000001176").expect("valid thread id");
    let pane_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000001177").expect("valid thread id");
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        pane_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    let entry = app
        .agent_navigation
        .get(&pane_thread_id)
        .expect("pane entry")
        .clone();
    assert!(app.is_operator_owned_codex_user_pane(pane_thread_id, &entry));

    app.send_native_task_event(pane_thread_id, "sent by an agent".to_string());
    match rx.try_recv().expect("an agent's task sends an event") {
        AppEvent::SubmitSpawnAgentTask {
            thread_id, task, ..
        } => {
            assert_eq!(thread_id, pane_thread_id);
            assert_eq!(task, "sent by an agent");
        }
        event => panic!("expected an agent's task for the mailbox, got {event:?}"),
    }
    assert!(rx.try_recv().is_err(), "one event only");

    (crate::spawn_orchestration::spawn_agent_task_prompt_submit(
        app.app_event_tx.clone(),
        pane_thread_id,
    ))("typed by the person".to_string());
    match rx.try_recv().expect("the prompt sends an event") {
        AppEvent::SubmitSpawnAgentTaskFromPerson { thread_id, task } => {
            assert_eq!(thread_id, pane_thread_id);
            assert_eq!(task, "typed by the person");
        }
        event => panic!("expected the person's task, got {event:?}"),
    }
}

#[tokio::test]
async fn pane_spawn_tree_disables_restored_unloaded_native_rows() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000231").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000232").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000233").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ true,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ true,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);

    let pane_items = app.spawn_tree_items(/*show_task_actions*/ false);
    let troll_item = pane_items
        .iter()
        .find(|item| item.name.contains("Burzum [troll]"))
        .expect("restored Troll row");
    assert!(troll_item.is_disabled);
    assert!(troll_item.actions.is_empty());
    assert!(
        troll_item
            .disabled_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("no replay transcript or live session"))
    );
    assert!(
        troll_item
            .description
            .as_deref()
            .is_some_and(|description| description.contains("saved-only"))
    );
    let orc_item = pane_items
        .iter()
        .find(|item| item.name.contains("Snaga [orc]"))
        .expect("restored Orc row");
    assert!(orc_item.is_disabled);
    assert!(orc_item.actions.is_empty());

    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    let troll_task_item = status_items
        .iter()
        .find(|item| item.name.contains("Send task to Burzum [troll]"))
        .expect("restored Troll task row");
    assert!(!troll_task_item.is_disabled);
    assert!(!troll_task_item.actions.is_empty());
}

#[tokio::test]
async fn pane_picker_keeps_failed_operator_restore_visible_without_automatic_retry() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000241").expect("valid thread id");
    let failed_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000242").expect("valid thread id");
    let pending_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000243").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        failed_thread_id,
        Some("Unavailable Pane".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ true,
    );
    app.upsert_agent_picker_thread(
        pending_thread_id,
        Some("Pending Pane".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );

    assert_eq!(
        app.restorable_operator_owned_codex_user_pane_ids(),
        vec![pending_thread_id],
        "a failed pane must not be retried automatically whenever /panes opens"
    );

    let items = app.pane_picker_items();
    let failed_item = items
        .iter()
        .find(|item| item.name.contains("Unavailable Pane"))
        .expect("failed pane remains inspectable");
    assert!(failed_item.is_disabled);
    assert!(failed_item.actions.is_empty());
    assert!(!failed_item.dismiss_on_select);
    assert!(
        failed_item
            .disabled_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("no replay transcript or live session"))
    );
}

#[tokio::test]
async fn terminal_operator_pane_lifecycle_removes_layout_membership_and_falls_back_to_main() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000251").expect("valid thread id");
    let operator_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000252").expect("valid thread id");
    let managed_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000253").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(operator_thread_id);
    app.upsert_agent_picker_thread(
        operator_thread_id,
        Some("Archive Me".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        managed_thread_id,
        Some("Keep Managed".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );

    assert!(app.forget_terminal_operator_pane(operator_thread_id));
    assert_eq!(app.active_thread_id, Some(main_thread_id));
    assert!(app.agent_navigation.get(&operator_thread_id).is_none());
    assert!(app.agent_navigation.get(&managed_thread_id).is_some());
    assert!(!app.forget_terminal_operator_pane(managed_thread_id));
    assert!(
        app.pane_picker_items()
            .iter()
            .all(|item| !item.name.contains("Archive Me"))
    );
}

#[tokio::test]
async fn terminal_lifecycle_scope_blocks_external_managed_and_parent_owned_panes() {
    let mut app = make_test_app().await;
    let main_thread_id = ThreadId::new();
    let operator_thread_id = ThreadId::new();
    let crew_thread_id = ThreadId::new();
    let parent_owned_thread_id = ThreadId::new();
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(operator_thread_id);
    app.upsert_agent_picker_thread(
        operator_thread_id,
        Some("Operator".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );

    assert!(
        app.terminal_thread_lifecycle_block_reason("archive", main_thread_id)
            .is_none()
    );
    assert!(
        app.terminal_thread_lifecycle_block_reason("delete", operator_thread_id)
            .is_none()
    );

    app.ensure_custom_spawn_root(crate::claude_panes::CODEX_MAIN_PANE_ID)
        .expect("bind custom root");
    app.record_custom_spawn_member(
        &crate::spawn_orchestration::thread_node_id(crew_thread_id),
        crate::claude_panes::CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Orc,
        "Snaga".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "deepseek-v4-flash".to_string(),
            provider: "deepseek".to_string(),
            reasoning_effort: Some(codex_protocol::openai_models::ReasoningEffort::High),
        },
    )
    .expect("record managed crew member");
    let crew_reason = app
        .terminal_thread_lifecycle_block_reason("delete", crew_thread_id)
        .expect("managed member must be protected");
    assert!(crew_reason.contains("managed /spawn member"));
    assert!(crew_reason.contains("native graph and pane layout"));

    app.agent_navigation
        .mark_parent_owned(parent_owned_thread_id);
    assert!(
        app.terminal_thread_lifecycle_block_reason("archive", parent_owned_thread_id)
            .is_some_and(|reason| reason.contains("parent-controlled task worker"))
    );

    let claude_pane_id = app
        .claude_panes
        .create_pane_without_vault_for_test(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create Claude pane");
    app.claude_panes
        .set_active_user_pane(&claude_pane_id)
        .expect("activate Claude pane");
    let claude_reason = app
        .terminal_thread_lifecycle_block_reason("archive", main_thread_id)
        .expect("external pane must not fall back to Main");
    assert!(claude_reason.contains("cannot target Claude pane"));
    assert!(claude_reason.contains("Select Main"));
}

#[tokio::test]
async fn delete_current_thread_routes_active_claude_pane_to_external_cleanup() {
    let mut app = make_test_app().await;
    app.primary_thread_id = Some(ThreadId::new());
    let pane_id = app
        .claude_panes
        .create_pane_without_vault_for_test(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create Claude pane");
    let artifact_dir = app.config.codex_home.join("panes").join(&pane_id);
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let pane = app
        .claude_panes
        .panes
        .iter_mut()
        .find(|pane| pane.id == pane_id)
        .expect("pane");
    pane.status = crate::claude_panes::ClaudePaneStatus::Running;
    pane.cancel_token = Some(cancel_token.clone());

    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(
        app.chat_widget.config_ref(),
    ))
    .await
    .expect("embedded app server");
    let mut tui = crate::tui::test_support::make_test_tui().expect("test tui");
    let control = Box::pin(app.delete_current_thread(&mut tui, &mut app_server))
        .await
        .expect("delete current thread");

    assert!(matches!(
        control,
        AppRunControl::Exit(ExitReason::UserRequested)
    ));
    assert!(cancel_token.is_cancelled());
    assert!(app.claude_panes.panes().is_empty());
    assert_eq!(app.claude_panes.active_user_pane_id(), CODEX_MAIN_PANE_ID);
    assert!(!artifact_dir.exists());
}

#[tokio::test]
async fn active_claude_pane_routes_before_native_thread_resolution() {
    let (mut app, mut event_rx, _op_rx) = make_test_app_with_channels().await;
    app.active_thread_id = None;
    app.claude_panes
        .create_pane_without_vault_for_test(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create Claude pane");
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(
        app.chat_widget.config_ref(),
    ))
    .await
    .expect("embedded app server");

    app.submit_active_thread_op(&mut app_server, claude_pane_text_op("/panes"))
        .await
        .expect("route external pane command");

    assert!(
        std::iter::from_fn(|| event_rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::OpenPanePicker)),
        "the selected external pane must consume input before native-thread lookup"
    );
}

#[test]
fn whole_crew_removal_deletes_mixed_members_and_preserves_bound_main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let mut app = Box::new(make_test_app().await);
        let mut app_server = start_config_write_test_app_server(&app).await?;
        let main = app_server.start_thread(&app.config).await?;
        let main_thread_id = main.session.thread_id;
        app.primary_thread_id = Some(main_thread_id);
        app.active_thread_id = Some(main_thread_id);
        app.primary_session_configured = Some(main.session.clone());
        app.set_spawn_nazgul_pane_binding(CODEX_MAIN_PANE_ID.to_string());

        let provider = app.config.model_provider_id.clone();
        let model = app.chat_widget.current_model().to_string();
        let effort = app.chat_widget.current_reasoning_effort();
        let spec = codex_protocol::crew::CrewSpec {
            schema_version: codex_protocol::crew::CURRENT_CREW_SCHEMA_VERSION,
            crew_id: "mixed-removal-fixture".to_string(),
            preset_id: None,
            members: vec![
                codex_protocol::crew::CrewMemberSpec {
                    logical_member_id: "nazgul".to_string(),
                    display_name: "Main".to_string(),
                    role_profile: "nazgul".to_string(),
                    parent_member_id: None,
                    runtime_request: codex_protocol::crew::RuntimeRequest::exact(
                        provider.clone(),
                        model.clone(),
                        effort.clone(),
                    ),
                },
                codex_protocol::crew::CrewMemberSpec {
                    logical_member_id: "troll".to_string(),
                    display_name: "Burzum".to_string(),
                    role_profile: "troll".to_string(),
                    parent_member_id: Some("nazgul".to_string()),
                    runtime_request: codex_protocol::crew::RuntimeRequest::exact(
                        provider.clone(),
                        model.clone(),
                        effort.clone(),
                    ),
                },
                codex_protocol::crew::CrewMemberSpec {
                    logical_member_id: "orc".to_string(),
                    display_name: "Snaga".to_string(),
                    role_profile: "orc".to_string(),
                    parent_member_id: Some("troll".to_string()),
                    runtime_request: codex_protocol::crew::RuntimeRequest::exact(
                        provider.clone(),
                        model.clone(),
                        effort.clone(),
                    ),
                },
            ],
            policy: codex_protocol::crew::CrewPolicy {
                delegation_mode: codex_protocol::crew::DelegationMode::Proactive,
                allow_ephemeral_descendants: true,
                provider_allowlist: vec![provider.clone()],
                maximum_spend_usd: None,
            },
        };
        let mut crew = crate::crew_state::CrewInstanceState::begin(spec).expect("valid crew");
        crew.record_member("nazgul", CODEX_MAIN_PANE_ID)
            .expect("map bound Main");

        let spawn_config = app.native_spawn_agent_config()?;
        let troll = app_server
            .spawn_agent_thread_with_class(
                &spawn_config,
                main_thread_id,
                "troll".to_string(),
                Some("Burzum".to_string()),
                codex_protocol::crew::AgentClass::CrewMember {
                    crew_id: crew.spec.crew_id.clone(),
                    logical_member_id: "troll".to_string(),
                    human_addressable: true,
                },
                model,
                Some(provider),
                effort,
                /*base_instructions*/ None,
            )
            .await?;
        let troll_thread_id = troll.session.thread_id;
        let troll_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);
        app.register_spawn_agent_pane(
            troll_thread_id,
            main_thread_id,
            crate::spawn_orchestration::pane_node_id(CODEX_MAIN_PANE_ID),
            Some("Burzum".to_string()),
            "troll",
            troll,
            /*persist_layout*/ true,
        )
        .await;
        crew.record_member("troll", &troll_node_id)
            .expect("map Troll");

        let orc_pane_id = app
            .claude_panes
            .create_pane_with_role(
                crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
                app.config.cwd.to_path_buf(),
                app.config.codex_home.as_ref(),
                Some(crate::spawn_orchestration::SpawnRole::Orc),
                Some("Snaga".to_string()),
            )
            .expect("create managed Claude Orc");
        let orc_node_id = crate::spawn_orchestration::pane_node_id(&orc_pane_id);
        app.spawn_parent_by_node
            .insert(orc_node_id.clone(), troll_node_id.clone());
        crew.record_member("orc", &orc_node_id).expect("map Orc");
        crew.mark_ready().expect("ready crew");
        app.spawn_crew = Some(crew);
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let orc_pane = app
            .claude_panes
            .panes
            .iter_mut()
            .find(|pane| pane.id == orc_pane_id)
            .expect("Orc pane");
        orc_pane.status = crate::claude_panes::ClaudePaneStatus::Running;
        orc_pane.cancel_token = Some(cancel_token.clone());
        app.claude_panes
            .set_active_user_pane(&orc_pane_id)
            .expect("select Orc");
        let orc_artifact_dir = app.config.codex_home.join("panes").join(&orc_pane_id);

        let mut tui = crate::tui::test_support::make_test_tui()?;
        let result = Box::pin(app.remove_spawn_crew(&mut tui, &mut app_server)).await?;

        assert!(result.contains("preserved the bound user-owned root"));
        assert!(cancel_token.is_cancelled());
        assert!(!orc_artifact_dir.exists());
        assert!(app.claude_panes.panes().is_empty());
        assert!(app.spawn_crew.is_none());
        assert!(app.spawn_parent_by_node.is_empty());
        assert_eq!(app.claude_panes.active_user_pane_id(), CODEX_MAIN_PANE_ID);
        assert_eq!(app.active_thread_id, Some(main_thread_id));
        let empty_rows = app.spawn_tree_items(/*show_task_actions*/ false);
        assert_eq!(empty_rows.len(), 1);
        assert_eq!(empty_rows[0].name, "No managed crew");
        assert!(empty_rows[0].is_disabled);
        assert!(
            empty_rows.iter().all(|row| !row.name.contains("Nazgul")),
            "removing a crew must not reclassify the surviving Main pane as its Nazgul"
        );
        app_server
            .thread_read(main_thread_id, /*include_turns*/ false)
            .await
            .expect("bound Main must survive");
        assert!(
            app_server
                .thread_read(troll_thread_id, /*include_turns*/ false)
                .await
                .is_err(),
            "native managed member must be permanently deleted"
        );
        Ok(())
    })
}

#[tokio::test]
async fn empty_spawn_tree_does_not_synthesize_main_as_nazgul() {
    let mut app = make_test_app().await;
    let main_thread_id = ThreadId::new();
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);

    let rows = app.spawn_tree_items(/*show_task_actions*/ false);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "No managed crew");
    assert!(rows[0].is_disabled);
    assert!(rows.iter().all(|row| !row.name.contains("Nazgul")));
    assert!(rows.iter().all(|row| !row.name.contains("Main")));
}

#[tokio::test]
async fn pane_spawn_tree_hides_superseded_saved_native_duplicates() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000241").expect("valid thread id");
    let old_troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000242").expect("valid thread id");
    let old_orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000243").expect("valid thread id");
    let live_troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000244").expect("valid thread id");
    let live_orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000245").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.spawn_nazgul_pane_id = Some(crate::spawn_orchestration::thread_node_id(main_thread_id));
    app.upsert_agent_picker_thread(
        old_troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ true,
    );
    app.upsert_agent_picker_thread(
        old_orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ true,
    );
    app.upsert_agent_picker_thread(
        live_troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        live_orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(old_troll_thread_id),
        crate::spawn_orchestration::thread_node_id(main_thread_id),
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(old_orc_thread_id),
        crate::spawn_orchestration::thread_node_id(old_troll_thread_id),
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(live_troll_thread_id),
        crate::spawn_orchestration::thread_node_id(main_thread_id),
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(live_orc_thread_id),
        crate::spawn_orchestration::thread_node_id(live_troll_thread_id),
    );

    let items = app.spawn_tree_items(/*show_task_actions*/ false);
    let burzum_rows = items
        .iter()
        .filter(|item| item.name.contains("Burzum [troll]"))
        .collect::<Vec<_>>();
    let snaga_rows = items
        .iter()
        .filter(|item| item.name.contains("Snaga [orc]"))
        .collect::<Vec<_>>();

    assert_eq!(burzum_rows.len(), 1);
    assert_eq!(snaga_rows.len(), 1);
    assert!(
        !burzum_rows[0]
            .description
            .as_deref()
            .is_some_and(|description| description.contains(&old_troll_thread_id.to_string()))
    );
    assert!(
        !snaga_rows[0]
            .description
            .as_deref()
            .is_some_and(|description| description.contains(&old_orc_thread_id.to_string()))
    );
}

#[tokio::test]
async fn duplicate_live_native_replacements_are_pruned() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000246").expect("valid thread id");
    let old_snaga_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000247").expect("valid thread id");
    let old_ghash_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000248").expect("valid thread id");
    let new_snaga_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000249").expect("valid thread id");
    let new_ghash_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000250").expect("valid thread id");
    let troll_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);

    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    for (thread_id, nickname) in [
        (old_snaga_thread_id, "Snaga"),
        (old_ghash_thread_id, "Ghash"),
        (new_snaga_thread_id, "Snaga"),
        (new_ghash_thread_id, "Ghash"),
    ] {
        app.upsert_agent_picker_thread(
            thread_id,
            Some(nickname.to_string()),
            Some("orc".to_string()),
            /*is_closed*/ false,
        );
        app.spawn_parent_by_node.insert(
            crate::spawn_orchestration::thread_node_id(thread_id),
            troll_node_id.clone(),
        );
    }

    app.prune_duplicate_live_native_spawn_threads();

    assert!(
        app.agent_navigation.get(&old_snaga_thread_id).is_none(),
        "saved Snaga should be replaced by a live endpoint; navigation={:?}; endpoints={:?}",
        app.agent_navigation.ordered_threads(),
        app.spawn_native_endpoint_by_node
    );
    assert!(
        app.agent_navigation.get(&old_ghash_thread_id).is_none(),
        "saved Ghash should be replaced by a live endpoint; navigation={:?}; endpoints={:?}",
        app.agent_navigation.ordered_threads(),
        app.spawn_native_endpoint_by_node
    );
    assert!(app.agent_navigation.get(&new_snaga_thread_id).is_some());
    assert!(app.agent_navigation.get(&new_ghash_thread_id).is_some());
    let child_thread_ids = app
        .spawn_parent_by_node
        .iter()
        .filter_map(|(child_node_id, parent_node_id)| {
            (parent_node_id == &troll_node_id)
                .then(|| crate::spawn_orchestration::node_id_thread(child_node_id))
                .flatten()
        })
        .collect::<Vec<_>>();
    assert_eq!(child_thread_ids.len(), 2);
    assert!(child_thread_ids.contains(&old_snaga_thread_id));
    assert!(child_thread_ids.contains(&old_ghash_thread_id));
    assert_eq!(
        app.spawn_native_endpoint_by_node
            [&crate::spawn_orchestration::thread_node_id(old_snaga_thread_id)],
        new_snaga_thread_id
    );
    assert_eq!(
        app.spawn_native_endpoint_by_node
            [&crate::spawn_orchestration::thread_node_id(old_ghash_thread_id)],
        new_ghash_thread_id
    );
    assert!(
        !app.spawn_parent_by_node
            .contains_key(&crate::spawn_orchestration::thread_node_id(
                new_snaga_thread_id
            ))
    );
    assert!(
        !app.spawn_parent_by_node
            .contains_key(&crate::spawn_orchestration::thread_node_id(
                new_ghash_thread_id
            ))
    );
}

#[tokio::test]
async fn restore_materializes_saved_native_orcs_without_rollouts() -> Result<()> {
    let mut app = make_test_app().await;
    let codex_home = tempdir()?;
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    std::fs::write(
        app.config.codex_home.join("provider_auth.json"),
        format!(
            r#"{{"api_keys":{{"{ZAI_API_KEY_ENV_VAR}":"test-key","{VERCEL_API_KEY_ENV_VAR}":"test-key"}}}}"#
        ),
    )?;
    app.chat_widget.update_account_state(
        /*status_account_display*/ None, /*plan_type*/ None,
        /*has_chatgpt_account*/ true, /*has_codex_backend_auth*/ true,
    );
    let mut app_server = start_config_write_test_app_server(&app).await?;
    let main = app_server.start_thread(&app.config).await?;
    let main_thread_id = main.session.thread_id;
    let main_rollout_path = main
        .session
        .rollout_path
        .as_ref()
        .expect("Main rollout path");
    assert!(main_rollout_path.is_file(), "Main rollout must be durable");
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.primary_session_configured = Some(main.session.clone());

    let spawn_config = app.native_spawn_agent_config()?;
    let nazgul = app_server
        .spawn_agent_thread(
            &spawn_config,
            main_thread_id,
            "nazgul".to_string(),
            Some("Angmar".to_string()),
            App::STANDARD_NAZGUL_MODEL.to_string(),
            Some(CLAUDE_PLAN_PROVIDER_ID.to_string()),
            /*reasoning_effort*/ None,
            /*base_instructions*/ None,
        )
        .await?;
    let nazgul_thread_id = nazgul.session.thread_id;
    let nazgul_rollout_path = nazgul
        .session
        .rollout_path
        .as_ref()
        .expect("nazgul rollout path");
    assert!(nazgul_rollout_path.exists());
    app.register_spawn_agent_pane(
        nazgul_thread_id,
        main_thread_id,
        crate::spawn_orchestration::pane_node_id(crate::claude_panes::CODEX_MAIN_PANE_ID),
        Some("Angmar".to_string()),
        "nazgul",
        nazgul,
        /*persist_layout*/ true,
    )
    .await;
    app.set_spawn_nazgul_pane_binding(crate::spawn_orchestration::thread_node_id(nazgul_thread_id));

    let troll = app_server
        .spawn_agent_thread(
            &spawn_config,
            nazgul_thread_id,
            "troll".to_string(),
            Some("Burzum".to_string()),
            App::STANDARD_TROLL_MODEL.to_string(),
            Some(OPENAI_PROVIDER_ID.to_string()),
            Some(ReasoningEffortConfig::XHigh),
            /*base_instructions*/ None,
        )
        .await?;
    let troll_thread_id = troll.session.thread_id;
    let troll_rollout_path = troll
        .session
        .rollout_path
        .as_ref()
        .expect("troll rollout path");
    assert!(troll_rollout_path.exists());
    app.register_spawn_agent_pane(
        troll_thread_id,
        nazgul_thread_id,
        crate::spawn_orchestration::thread_node_id(nazgul_thread_id),
        Some("Burzum".to_string()),
        "troll",
        troll,
        /*persist_layout*/ true,
    )
    .await;
    app.thread_event_channels
        .get_mut(&troll_thread_id)
        .expect("registered Troll channel")
        .mark_replay_only();
    assert!(app.thread_event_channels.contains_key(&troll_thread_id));
    assert!(!app.native_agent_thread_has_loaded_session(troll_thread_id));

    let old_snaga_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000251").expect("valid thread id");
    let old_ghash_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000252").expect("valid thread id");
    let troll_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);
    let old_snaga_node_id = crate::spawn_orchestration::thread_node_id(old_snaga_thread_id);
    let old_ghash_node_id = crate::spawn_orchestration::thread_node_id(old_ghash_thread_id);
    app.spawn_parent_by_node
        .insert(old_snaga_node_id.clone(), troll_node_id.clone());
    app.spawn_parent_by_node
        .insert(old_ghash_node_id.clone(), troll_node_id.clone());
    app.upsert_agent_picker_thread(
        old_snaga_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ true,
    );
    app.upsert_agent_picker_thread(
        old_ghash_thread_id,
        Some("Ghash".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ true,
    );
    // Durable restoration is CrewSpec-backed. Register the saved logical members so the
    // restoration boundary does not mistake these deliberate no-rollout fixtures for unrelated
    // legacy task agents when Main itself is already durable.
    app.ensure_custom_spawn_root(&crate::spawn_orchestration::thread_node_id(
        nazgul_thread_id,
    ))
    .expect("record durable Nazgul root");
    app.record_custom_spawn_member(
        &troll_node_id,
        &crate::spawn_orchestration::thread_node_id(nazgul_thread_id),
        crate::spawn_orchestration::SpawnRole::Troll,
        "Burzum".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: App::STANDARD_TROLL_MODEL.to_string(),
            provider: OPENAI_PROVIDER_ID.to_string(),
            reasoning_effort: Some(ReasoningEffortConfig::XHigh),
        },
    )
    .expect("record durable Troll");
    for (node_id, nickname) in [(&old_snaga_node_id, "Snaga"), (&old_ghash_node_id, "Ghash")] {
        app.record_custom_spawn_member(
            node_id,
            &troll_node_id,
            crate::spawn_orchestration::SpawnRole::Orc,
            nickname.to_string(),
            crate::dispatch_queue::SavedNativeSpawnRuntime {
                model: App::STANDARD_ORC_MODEL.to_string(),
                provider: OPENAI_PROVIDER_ID.to_string(),
                reasoning_effort: Some(ReasoningEffortConfig::XHigh),
            },
        )
        .expect("record durable Orc");
    }

    app.restore_native_spawn_panes_from_saved_state(&mut app_server)
        .await;

    assert!(
        app.agent_navigation.get(&old_snaga_thread_id).is_none(),
        "saved Snaga should be replaced by a live endpoint; navigation={:?}; endpoints={:?}",
        app.agent_navigation.ordered_threads(),
        app.spawn_native_endpoint_by_node
    );
    assert!(
        app.agent_navigation.get(&old_ghash_thread_id).is_none(),
        "saved Ghash should be replaced by a live endpoint; navigation={:?}; endpoints={:?}",
        app.agent_navigation.ordered_threads(),
        app.spawn_native_endpoint_by_node
    );
    assert!(app.native_agent_thread_has_loaded_session(troll_thread_id));
    assert_eq!(
        app.agent_navigation
            .get(&troll_thread_id)
            .and_then(|entry| entry.model.as_deref()),
        Some(App::STANDARD_TROLL_MODEL)
    );
    // Saved pane ids are durable logical identities. Materialization keeps those parent edges and
    // routes each logical id to the new live thread through spawn_native_endpoint_by_node.
    assert_eq!(
        app.spawn_parent_by_node.get(&old_snaga_node_id),
        Some(&troll_node_id)
    );
    assert_eq!(
        app.spawn_parent_by_node.get(&old_ghash_node_id),
        Some(&troll_node_id)
    );
    let restored_orc_thread_ids = [&old_snaga_node_id, &old_ghash_node_id]
        .into_iter()
        .map(|node_id| {
            *app.spawn_native_endpoint_by_node
                .get(node_id)
                .expect("saved Orc has a live endpoint")
        })
        .collect::<Vec<_>>();
    assert!(!restored_orc_thread_ids.contains(&old_snaga_thread_id));
    assert!(!restored_orc_thread_ids.contains(&old_ghash_thread_id));
    let restored_orcs = restored_orc_thread_ids
        .into_iter()
        .map(|thread_id| {
            (
                thread_id,
                app.agent_navigation
                    .get(&thread_id)
                    .expect("materialized Orc is visible"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(restored_orcs.len(), 2);
    let restored_names = restored_orcs
        .iter()
        .filter_map(|(_, entry)| entry.agent_nickname.as_deref())
        .collect::<Vec<_>>();
    assert!(restored_names.contains(&"Snaga"), "{restored_names:?}");
    assert!(restored_names.contains(&"Ghash"), "{restored_names:?}");
    for (thread_id, entry) in restored_orcs {
        assert!(!entry.is_closed);
        assert!(app.native_agent_thread_has_loaded_session(thread_id));
    }
    let items = app.spawn_tree_items(/*show_task_actions*/ false);
    assert!(
        items
            .iter()
            .filter(|item| item.name.contains("[orc]"))
            .all(|item| !item.is_disabled)
    );
    assert!(items.iter().all(|item| {
        !item
            .description
            .as_deref()
            .is_some_and(|description| description.contains("saved-only"))
    }));

    app_server.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn app_server_spawn_treats_hierarchy_role_as_metadata_under_general_worker() -> Result<()> {
    let app = make_test_app().await;
    let mut app_server = start_config_write_test_app_server(&app).await?;
    let main = app_server.start_thread(&app.config).await?;
    let spawn_config = app.native_spawn_agent_config()?;
    let worker = app_server
        .spawn_agent_thread(
            &spawn_config,
            main.session.thread_id,
            "worker".to_string(),
            Some("GeneralWorker".to_string()),
            App::STANDARD_TROLL_MODEL.to_string(),
            Some(OPENAI_PROVIDER_ID.to_string()),
            Some(ReasoningEffortConfig::High),
            /*base_instructions*/ None,
        )
        .await?;

    let orc = app_server
        .spawn_agent_thread(
            &spawn_config,
            worker.session.thread_id,
            "orc".to_string(),
            Some("ForgedOrc".to_string()),
            App::STANDARD_ORC_MODEL.to_string(),
            Some(OPENAI_PROVIDER_ID.to_string()),
            Some(ReasoningEffortConfig::High),
            /*base_instructions*/ None,
        )
        .await?;
    assert_eq!(orc.session.model, App::STANDARD_ORC_MODEL);
    assert_eq!(orc.session.model_provider_id, OPENAI_PROVIDER_ID);
    Ok(())
}

#[tokio::test]
async fn nazgul_can_be_bound_to_a_codex_agent_pane() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000301").expect("valid thread id");
    // A second Codex agent pane (no Troll/Orc role) that should be bindable as the Nazgul root.
    let codex_pane_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000302").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        codex_pane_thread_id,
        Some("Mordor".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.agent_navigation
        .set_running(codex_pane_thread_id, /*is_running*/ true);

    // The Nazgul picker should offer the Codex agent pane as a bindable target.
    let picker_items = app.nazgul_codex_pane_picker_items();
    assert!(
        picker_items.iter().any(|item| item.name.contains("Mordor")),
        "Codex agent pane should appear in the Nazgul binding picker"
    );

    // Bind the Codex agent pane as the Nazgul root.
    let node_id = crate::spawn_orchestration::thread_node_id(codex_pane_thread_id);
    app.spawn_native_runtime_by_node.insert(
        node_id.clone(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: app.chat_widget.current_model().to_string(),
            provider: app.config.model_provider_id.clone(),
            reasoning_effort: app.config.model_reasoning_effort.clone(),
        },
    );
    app.bind_spawn_nazgul_pane(node_id.clone());
    assert_eq!(app.spawn_nazgul_pane_id.as_deref(), Some(node_id.as_str()));

    // The spawn status tree should render the bound Codex pane by name, not as a raw node id.
    let items = app.spawn_tree_items(/*show_task_actions*/ false);
    let nazgul_item = items
        .iter()
        .find(|item| item.name.starts_with("Nazgul:"))
        .expect("status tree has a Nazgul row");
    assert!(
        nazgul_item.name.contains("Mordor"),
        "bound Codex pane should render by name, got: {}",
        nazgul_item.name
    );
    assert!(
        !nazgul_item.name.contains("thread:"),
        "bound Codex pane should not leak the raw thread node id: {}",
        nazgul_item.name
    );
    assert_eq!(
        nazgul_item.description.as_deref(),
        Some(
            "Root role binding; running; 00000000-0000-0000-0000-000000000302; same user pane listed above."
        ),
        "bound Nazgul must project the same live running state as other managed native roles"
    );

    app.spawn_status_by_thread.insert(
        codex_pane_thread_id,
        codex_app_server_protocol::CollabAgentState {
            status: codex_app_server_protocol::CollabAgentStatus::Completed,
            message: Some("finished root work".to_string()),
        },
    );
    let completed_items = app.spawn_tree_items(/*show_task_actions*/ false);
    let completed_nazgul_item = completed_items
        .iter()
        .find(|item| item.name.starts_with("Nazgul:"))
        .expect("completed status tree has a Nazgul row");
    assert_eq!(
        completed_nazgul_item.description.as_deref(),
        Some(
            "Root role binding; completed; 00000000-0000-0000-0000-000000000302; same user pane listed above."
        ),
        "bound Nazgul must project terminal state through the shared managed-role boundary"
    );

    // When the bound Codex pane is the active thread (and the user pane is Codex Main), the
    // Nazgul row should be marked current.
    app.active_thread_id = Some(codex_pane_thread_id);
    let items = app.spawn_tree_items(/*show_task_actions*/ false);
    let nazgul_item = items
        .iter()
        .find(|item| item.name.starts_with("Nazgul:"))
        .expect("status tree has a Nazgul row");
    assert!(
        nazgul_item.is_current,
        "Nazgul row should be current when the bound Codex pane is active"
    );
}

#[tokio::test]
async fn unbound_codex_main_never_receives_nazgul_context() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000301").expect("valid thread id");
    let stale_child_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000302").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.spawn_nazgul_pane_id = None;
    app.spawn_parent_by_thread
        .insert(stale_child_thread_id, main_thread_id);
    app.spawn_status_by_thread.insert(
        stale_child_thread_id,
        codex_app_server_protocol::CollabAgentState {
            status: codex_app_server_protocol::CollabAgentStatus::Completed,
            message: Some("stale child result".to_string()),
        },
    );

    assert!(
        app.spawn_additional_context_for_thread(main_thread_id)
            .is_none(),
        "an unbound primary thread must keep the default prompt"
    );
    assert!(
        app.spawn_context_for_user_pane(CODEX_MAIN_PANE_ID)
            .is_none(),
        "an unbound user pane must not be assigned the Nazgul role"
    );
}

#[tokio::test]
async fn codex_main_bound_nazgul_turn_receives_domain_neutral_hierarchy_context() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000303").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000304").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.bind_spawn_nazgul_pane(CODEX_MAIN_PANE_ID.to_string());
    let troll_runtime = crate::dispatch_queue::SavedNativeSpawnRuntime {
        model: app.chat_widget.current_model().to_string(),
        provider: app.config.model_provider_id.clone(),
        reasoning_effort: app.config.model_reasoning_effort.clone(),
    };
    app.record_custom_spawn_member(
        &crate::spawn_orchestration::thread_node_id(troll_thread_id),
        CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Troll,
        "Burzum".to_string(),
        troll_runtime,
    )
    .expect("register Troll in the bound CrewSpec");
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(troll_thread_id),
        crate::spawn_orchestration::pane_node_id(CODEX_MAIN_PANE_ID),
    );

    let context_map = app
        .spawn_additional_context_for_thread(main_thread_id)
        .expect("bound Codex Main turn should receive Nazgul application context");
    let entry = context_map
        .get("pfterminal_spawn_context")
        .expect("spawn context entry");

    assert_eq!(entry.kind, AdditionalContextKind::Application);
    assert!(entry.value.contains("Nazgul/root pane"));
    assert!(entry.value.contains("Burzum [troll]"));
    assert!(entry.value.contains("send_message"));
    assert!(entry.value.contains("followup_task"));
    assert!(!entry.value.contains("<pfterminal_send_task"));
}

#[tokio::test]
async fn pane_picker_separates_user_panes_from_managed_spawn_crew() {
    let mut app = make_test_app().await;
    let manager_thread = ThreadId::new();
    let worker_thread = ThreadId::new();
    let parent_controlled_thread = ThreadId::new();
    app.upsert_agent_picker_thread(
        manager_thread,
        Some("Codex manager".to_string()),
        Some("default".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        worker_thread,
        Some("Codex worker".to_string()),
        Some("default".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(worker_thread, manager_thread);
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread);
    app.spawn_native_runtime_by_node.insert(
        manager_node.clone(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: app.chat_widget.current_model().to_string(),
            provider: app.config.model_provider_id.clone(),
            reasoning_effort: app.config.model_reasoning_effort.clone(),
        },
    );
    app.bind_spawn_nazgul_pane(manager_node);
    app.upsert_agent_picker_thread(
        parent_controlled_thread,
        Some("Task-only subagent".to_string()),
        Some("default".to_string()),
        /*is_closed*/ false,
    );
    app.agent_navigation
        .mark_parent_owned(parent_controlled_thread);
    let items = app.pane_picker_items();
    let names = items
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"Corbanu Terminal - Codex manager"));
    assert!(names.contains(&"Corbanu Terminal - Codex worker"));
    assert!(
        !names.contains(&"Corbanu Terminal - Task-only subagent"),
        "managed workers stay out of user panes"
    );

    let user_index = names
        .iter()
        .position(|name| *name == "User Panes")
        .expect("user panes section");
    let create_index = names
        .iter()
        .position(|name| *name == "Create User Pane")
        .expect("user pane creation section");
    let crew_index = names
        .iter()
        .position(|name| *name == "Managed Crew (/spawn)")
        .expect("managed crew section");

    assert!(user_index < create_index);
    assert!(create_index < crew_index);
    assert!(
        items
            .iter()
            .find(|item| item.name.starts_with("Nazgul:"))
            .and_then(|item| item.description.as_deref())
            .is_some_and(|description| description.contains("same user pane listed above"))
    );
}

#[tokio::test]
async fn pane_picker_marks_exactly_the_active_native_thread_current() {
    let mut app = make_test_app().await;
    let main_thread = ThreadId::new();
    let user_thread = ThreadId::new();
    app.primary_thread_id = Some(main_thread);
    app.active_thread_id = Some(user_thread);
    app.upsert_agent_picker_thread(
        main_thread,
        Some("Main".to_string()),
        Some("default".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        user_thread,
        Some("Matrix Twin".to_string()),
        Some("default".to_string()),
        /*is_closed*/ false,
    );
    app.primary_session_configured = Some(ThreadSessionState {
        model: "deepseek-v4-flash".to_string(),
        ..test_thread_session(main_thread, test_path_buf("/tmp/main"))
    });
    app.agent_navigation
        .set_model(user_thread, Some("gpt-5.6-sol".to_string()));
    app.chat_widget.set_model("gpt-5.6-sol");

    let items = app.pane_picker_items();
    let main = items
        .iter()
        .find(|item| item.name == "Corbanu Terminal - Main")
        .expect("Main pane");
    let user = items
        .iter()
        .find(|item| item.name == "Corbanu Terminal - Matrix Twin")
        .expect("operator pane");

    assert!(!main.is_current, "inactive Main marked current");
    assert!(user.is_current, "active pane not current");
    assert_eq!(
        items.iter().filter(|item| item.is_current).count(),
        1,
        "expected one current pane"
    );
    assert_snapshot!(
        format!(
            "Main: {}\nMatrix Twin: {}",
            main.description.as_deref().expect("Main description"),
            user.description.as_deref().expect("operator pane description")
        ),
        @r###"
    Main: deepseek-v4-flash; idle
    Matrix Twin: gpt-5.6-sol; idle
    "###
    );
}

/// Drains native task events for `thread_id`, whether the target is a managed agent or an
/// operator-owned user pane (both take agents' tasks from the mailbox, D106). Returns the task
/// texts.
fn drain_spawn_agent_tasks_for(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    thread_id: ThreadId,
) -> Vec<String> {
    let mut matched = Vec::new();
    while let Ok(event) = rx.try_recv() {
        match event {
            AppEvent::SubmitSpawnAgentTask {
                thread_id: t, task, ..
            } if t == thread_id => matched.push(task),
            _ => {}
        }
    }
    matched
}

/// Drains app events looking for a `SubmitSpawnAgentTask` for `thread_id`. Returns its task text.
pub(super) fn drain_spawn_agent_task_for(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    thread_id: ThreadId,
) -> Option<String> {
    drain_spawn_agent_tasks_for(rx, thread_id).pop()
}

fn register_native_dispatch_pair(app: &mut App) -> (ThreadId, ThreadId) {
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000451").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000452").expect("valid thread id");
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);
    (troll_thread_id, orc_thread_id)
}

fn issue_1208_layout(edges: &[(&str, &str)]) -> crate::claude_panes::PaneLayoutState {
    crate::claude_panes::PaneLayoutState {
        spawn_parent_by_node: edges
            .iter()
            .map(|(child, parent)| (child.to_string(), parent.to_string()))
            .collect(),
        ..Default::default()
    }
}

const ISSUE_1208_MAIN: &str = "thread:00000000-0000-0000-0000-000000001208";
const ISSUE_1208_CHILD: &str = "thread:00000000-0000-0000-0000-000000001209";
const ISSUE_1208_GRANDCHILD: &str = "thread:00000000-0000-0000-0000-000000001210";

#[test]
fn model_spawned_sub_agents_alone_are_not_a_legacy_spawn_hierarchy() {
    // #1208: the spawn-agent tool records only thread -> thread edges.
    let native_only = issue_1208_layout(&[
        (ISSUE_1208_CHILD, ISSUE_1208_MAIN),
        (ISSUE_1208_GRANDCHILD, ISSUE_1208_CHILD),
    ]);
    assert_eq!(
        crate::spawn_orchestration::pane_layout_has_legacy_spawn_hierarchy(&native_only),
        false
    );

    // A /spawn Troll without a Nazgul binding hangs off the Main pane node.
    let legacy_pane_root = issue_1208_layout(&[
        (ISSUE_1208_CHILD, "pane:codex-main"),
        (ISSUE_1208_GRANDCHILD, ISSUE_1208_CHILD),
    ]);
    assert_eq!(
        crate::spawn_orchestration::pane_layout_has_legacy_spawn_hierarchy(&legacy_pane_root),
        true
    );

    // A /spawn hierarchy bound to a native Nazgul has only thread edges plus the binding.
    let legacy_bound_nazgul = crate::claude_panes::PaneLayoutState {
        spawn_nazgul_pane_id: Some(ISSUE_1208_MAIN.to_string()),
        ..issue_1208_layout(&[(ISSUE_1208_CHILD, ISSUE_1208_MAIN)])
    };
    assert_eq!(
        crate::spawn_orchestration::pane_layout_has_legacy_spawn_hierarchy(&legacy_bound_nazgul),
        true
    );

    // A /spawn Claude member is a pane node.
    let legacy_claude_member = issue_1208_layout(&[("pane:claude-1", ISSUE_1208_MAIN)]);
    assert_eq!(
        crate::spawn_orchestration::pane_layout_has_legacy_spawn_hierarchy(&legacy_claude_member),
        true
    );
}

#[tokio::test]
async fn resume_with_only_model_spawned_sub_agents_shows_no_legacy_spawn_notice() -> Result<()> {
    let native_only = issue_1208_layout(&[(ISSUE_1208_CHILD, ISSUE_1208_MAIN)]);
    let legacy = issue_1208_layout(&[(ISSUE_1208_CHILD, "pane:codex-main")]);
    for (layout, expect_notice) in [(native_only, false), (legacy, true)] {
        let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
        // Set by hand as App construction does it in app.rs
        // (`restored_spawn_legacy_read_only`, the value given to
        // `spawn_legacy_read_only` when the App is built from the restored
        // layout). If that site stops calling
        // `pane_layout_has_legacy_spawn_hierarchy`, change this line too.
        app.spawn_legacy_read_only =
            crate::spawn_orchestration::pane_layout_has_legacy_spawn_hierarchy(&layout);
        let mut app_server = start_config_write_test_app_server(&app).await?;

        app.restore_native_spawn_panes_from_saved_state(&mut app_server)
            .await;

        let mut rendered = String::new();
        while let Ok(event) = rx.try_recv() {
            if let AppEvent::InsertHistoryCell(cell) = event {
                rendered.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 200)));
            }
        }
        assert_eq!(
            (
                app.spawn_legacy_read_only,
                rendered.contains("Restored a legacy /spawn hierarchy in read-only mode")
            ),
            (expect_notice, expect_notice),
            "layout edges {:?}; rendered: {rendered}",
            layout.spawn_parent_by_node
        );
    }
    Ok(())
}

#[tokio::test]
async fn restored_legacy_spawn_hierarchy_is_inspectable_but_rejects_mutation() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (source, target) = register_native_dispatch_pair(&mut app);
    app.spawn_legacy_read_only = true;

    app.dispatch_spawn_task_blocks(
        &thread_node_id(source),
        vec![crate::spawn_orchestration::SpawnTaskDispatch {
            target: target.to_string(),
            task: "must not enter the new control plane".to_string(),
            seq: Some(1),
        }],
    );

    while let Ok(event) = rx.try_recv() {
        assert!(
            !matches!(event, AppEvent::SubmitSpawnAgentTask { .. }),
            "legacy read-only sessions must never dispatch work"
        );
    }
}

#[tokio::test]
async fn restored_crew_validation_rejects_runtime_drift() {
    let mut app = make_test_app().await;
    let thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000499").expect("thread id");
    let node_id = thread_node_id(thread_id);
    app.upsert_agent_picker_thread(
        thread_id,
        Some("Manager".to_string()),
        Some("manager".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_node.insert(
        node_id.clone(),
        crate::spawn_orchestration::pane_node_id(CODEX_MAIN_PANE_ID),
    );
    app.spawn_native_runtime_by_node.insert(
        node_id.clone(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            provider: "openai".to_string(),
            model: "gpt-5.6-sol".to_string(),
            // `None` in CrewSpec means use the provider/model default. Recovery records the
            // resolved effort and must not mistake that resolution for runtime drift.
            reasoning_effort: Some(ReasoningEffortConfig::High),
        },
    );
    let spec = codex_protocol::crew::CrewSpec {
        schema_version: codex_protocol::crew::CURRENT_CREW_SCHEMA_VERSION,
        crew_id: "validation-fixture".to_string(),
        preset_id: None,
        members: vec![codex_protocol::crew::CrewMemberSpec {
            logical_member_id: "manager".to_string(),
            display_name: "Manager".to_string(),
            role_profile: "manager".to_string(),
            parent_member_id: None,
            runtime_request: codex_protocol::crew::RuntimeRequest::exact(
                "openai",
                "gpt-5.6-sol",
                /*reasoning_effort*/ None,
            ),
        }],
        policy: codex_protocol::crew::CrewPolicy {
            delegation_mode: codex_protocol::crew::DelegationMode::ExplicitOnly,
            allow_ephemeral_descendants: true,
            provider_allowlist: vec!["openai".to_string()],
            maximum_spend_usd: None,
        },
    };
    let mut state = crate::crew_state::CrewInstanceState::begin(spec).expect("valid crew");
    state.record_member("manager", &node_id).expect("mapping");
    state.mark_ready().expect("ready");
    app.spawn_crew = Some(state);

    app.validate_restored_crew_state()
        .expect("exact restored mapping");
    app.spawn_native_runtime_by_node
        .get_mut(&node_id)
        .expect("runtime")
        .model = "different-model".to_string();
    let error = app
        .validate_restored_crew_state()
        .expect_err("runtime drift must stop recovery");
    assert!(error.to_string().contains("runtime changed"));
}

#[tokio::test]
async fn restored_spawn_resume_uses_saved_runtime_instead_of_parent_runtime() {
    let mut app = make_test_app().await;
    app.config.model = Some(codex_model_provider_info::CLAUDE_PLAN_MODEL.to_string());
    app.config.model_reasoning_effort = Some(ReasoningEffortConfig::High);
    let saved_runtime = crate::dispatch_queue::SavedNativeSpawnRuntime {
        provider: CLAUDE_PLAN_PROVIDER_ID.to_string(),
        model: codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL.to_string(),
        reasoning_effort: Some(ReasoningEffortConfig::High),
    };

    let (resume_config, model_override, permission_settings) =
        app.native_spawn_resume_request(Some(&saved_runtime));

    assert_eq!(
        resume_config.model.as_deref(),
        Some(codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL)
    );
    assert_eq!(
        resume_config.model_reasoning_effort,
        Some(ReasoningEffortConfig::High)
    );
    assert_eq!(
        model_override,
        Some(crate::app_server_session::ResumeModelOverride {
            model: Some(codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL.to_string()),
            model_provider: Some(CLAUDE_PLAN_PROVIDER_ID.to_string()),
        })
    );
    assert_eq!(
        permission_settings,
        crate::resume_permissions::ResumePermissions::default()
    );

    app.harness_overrides.sandbox_mode = Some(SandboxMode::WorkspaceWrite);
    app.harness_overrides.approval_policy =
        Some(codex_protocol::protocol::AskForApproval::OnRequest);
    let (_, _, permission_settings) = app.native_spawn_resume_request(Some(&saved_runtime));
    assert!(
        permission_settings.profile && permission_settings.approval_policy,
        "{permission_settings:?}"
    );
}

#[tokio::test]
async fn startup_resume_applies_persisted_runtime_before_bootstrap() {
    let mut app = make_test_app().await;
    let provider = app
        .config
        .model_providers
        .get(CLAUDE_PLAN_PROVIDER_ID)
        .expect("Claude Plan provider")
        .clone();
    app.config.model = Some("gpt-5.6-sol".to_string());
    app.config.model_provider_id = OPENAI_PROVIDER_ID.to_string();
    app.config.model_reasoning_effort = Some(ReasoningEffortConfig::XHigh);

    let model_override = apply_persisted_resume_runtime(
        &mut app.config,
        Some(codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL),
        CLAUDE_PLAN_PROVIDER_ID,
        Some(ReasoningEffortConfig::High),
    )
    .expect("persisted runtime");

    assert_eq!(
        app.config.model.as_deref(),
        Some(codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL)
    );
    assert_eq!(app.config.model_provider_id, CLAUDE_PLAN_PROVIDER_ID);
    assert_eq!(app.config.model_provider, provider);
    assert_eq!(
        app.config.model_reasoning_effort,
        Some(ReasoningEffortConfig::High)
    );
    assert_eq!(
        model_override,
        crate::app_server_session::ResumeModelOverride {
            model: Some(codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL.to_string()),
            model_provider: Some(CLAUDE_PLAN_PROVIDER_ID.to_string()),
        }
    );
}

#[tokio::test]
async fn manually_assembled_multimodel_spawn_crew_is_crewspec_backed() {
    let mut app = make_test_app().await;
    app.ensure_custom_spawn_root(crate::claude_panes::CODEX_MAIN_PANE_ID)
        .expect("bind custom root");
    app.record_custom_spawn_member(
        "thread:troll",
        crate::claude_panes::CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Troll,
        "Burzum".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "claude-opus-5-plan".to_string(),
            provider: "claude-plan".to_string(),
            reasoning_effort: None,
        },
    )
    .expect("add Opus Troll");
    app.record_custom_spawn_member(
        "thread:orc-kimi",
        "thread:troll",
        crate::spawn_orchestration::SpawnRole::Orc,
        "Snaga".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "k3".to_string(),
            provider: "kimi-code".to_string(),
            reasoning_effort: None,
        },
    )
    .expect("add Kimi Orc");

    let crew = app.spawn_crew.as_ref().expect("custom crew");
    assert!(matches!(
        crew.status,
        crate::crew_state::CrewCreationStatus::Ready
    ));
    assert_eq!(crew.spec.preset_id, None);
    assert_eq!(crew.spec.members.len(), 3);
    assert_eq!(
        crew.member_node_by_id
            .values()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["codex-main", "thread:orc-kimi", "thread:troll"]
    );
    crew.spec.validate().expect("custom crew stays valid");
    assert!(!app.spawn_legacy_read_only);
}

#[tokio::test]
async fn restored_crewspec_roles_drive_spawn_and_pane_projection_when_liveness_omits_roles() {
    let mut app = make_test_app().await;
    let main_thread_id = ThreadId::new();
    let nazgul_thread_id = ThreadId::new();
    let troll_thread_id = ThreadId::new();
    let orc_thread_id = ThreadId::new();
    let nazgul_node = crate::spawn_orchestration::thread_node_id(nazgul_thread_id);
    let troll_node = crate::spawn_orchestration::thread_node_id(troll_thread_id);
    let orc_node = crate::spawn_orchestration::thread_node_id(orc_thread_id);
    let runtime = crate::dispatch_queue::SavedNativeSpawnRuntime {
        model: "deepseek-v4-flash".to_string(),
        provider: "deepseek".to_string(),
        reasoning_effort: Some(ReasoningEffortConfig::High),
    };

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.spawn_nazgul_pane_id = Some(nazgul_node.clone());
    app.spawn_native_runtime_by_node
        .insert(nazgul_node.clone(), runtime.clone());
    app.ensure_custom_spawn_root(&nazgul_node)
        .expect("restore CrewSpec Nazgul");
    app.record_custom_spawn_member(
        &troll_node,
        &nazgul_node,
        crate::spawn_orchestration::SpawnRole::Troll,
        "Burzum".to_string(),
        runtime.clone(),
    )
    .expect("restore CrewSpec Troll");
    app.record_custom_spawn_member(
        &orc_node,
        &troll_node,
        crate::spawn_orchestration::SpawnRole::Orc,
        "Snaga".to_string(),
        runtime,
    )
    .expect("restore CrewSpec Orc");
    app.spawn_parent_by_node.insert(
        nazgul_node.clone(),
        crate::spawn_orchestration::pane_node_id(crate::claude_panes::CODEX_MAIN_PANE_ID),
    );
    app.spawn_parent_by_node
        .insert(troll_node.clone(), nazgul_node.clone());
    app.spawn_parent_by_node
        .insert(orc_node.clone(), troll_node.clone());
    for (node, thread_id, nickname) in [
        (&nazgul_node, nazgul_thread_id, "Angmar"),
        (&troll_node, troll_thread_id, "Burzum"),
        (&orc_node, orc_thread_id, "Snaga"),
    ] {
        app.spawn_native_endpoint_by_node
            .insert(node.clone(), thread_id);
        // `thread/list` and other liveness projections may omit role metadata. CrewSpec must
        // remain sufficient to render and operate every persistent member.
        app.upsert_agent_picker_thread(
            thread_id,
            Some(nickname.to_string()),
            /*agent_role*/ None,
            /*is_closed*/ false,
        );
    }

    let rows = app.spawn_tree_items(/*show_task_actions*/ false);

    let nazgul_row = rows
        .iter()
        .find(|row| row.name.contains("Nazgul: Angmar"))
        .expect("restored Nazgul row");
    let nazgul_search = nazgul_row
        .search_value
        .as_deref()
        .expect("Nazgul row must be searchable");
    assert!(nazgul_search.contains("Angmar"));
    assert!(nazgul_search.contains(nazgul_thread_id.to_string().as_str()));
    assert!(rows.iter().any(|row| row.name.contains("Burzum [troll]")));
    assert!(rows.iter().any(|row| row.name.contains("Snaga [orc]")));
    assert!(app.spawn_context_for_thread(troll_thread_id).is_some());
    assert!(app.spawn_context_for_thread(orc_thread_id).is_some());
}

#[tokio::test]
async fn custom_spawn_core_classes_match_persisted_crew_identity() {
    let mut app = make_test_app().await;
    let root_class = app
        .prepare_custom_spawn_root(
            "Angmar".to_string(),
            crate::dispatch_queue::SavedNativeSpawnRuntime {
                model: "deepseek-v4-flash".to_string(),
                provider: "deepseek".to_string(),
                reasoning_effort: Some(ReasoningEffortConfig::High),
            },
        )
        .expect("prepare custom root identity");
    let codex_protocol::crew::AgentClass::CrewMember {
        crew_id,
        logical_member_id,
        human_addressable,
    } = root_class
    else {
        panic!("custom root must be a persistent crew member");
    };
    assert_eq!(logical_member_id, "nazgul");
    assert!(human_addressable);
    assert_eq!(
        app.spawn_crew.as_ref().expect("prepared crew").spec.crew_id,
        crew_id
    );

    app.ensure_custom_spawn_root("thread:angmar")
        .expect("materialize prepared root identity");
    let troll_class = app
        .custom_spawn_member_agent_class(crate::spawn_orchestration::SpawnRole::Troll)
        .expect("allocate Troll identity from the same crew");
    let codex_protocol::crew::AgentClass::CrewMember {
        crew_id: troll_crew_id,
        logical_member_id: troll_member_id,
        human_addressable: troll_human_addressable,
    } = troll_class
    else {
        panic!("custom Troll must be a persistent crew member");
    };
    assert_eq!(troll_crew_id, crew_id);
    assert_eq!(troll_member_id, "troll-1");
    assert!(troll_human_addressable);

    app.record_custom_spawn_member(
        "thread:burzum",
        "thread:angmar",
        crate::spawn_orchestration::SpawnRole::Troll,
        "Burzum".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "claude-opus-5-plan".to_string(),
            provider: "claude-plan".to_string(),
            reasoning_effort: Some(ReasoningEffortConfig::High),
        },
    )
    .expect("persist Troll using the reserved Core identity");
    let crew = app.spawn_crew.as_ref().expect("ready custom crew");
    assert_eq!(
        crew.member_node_by_id
            .get(&troll_member_id)
            .map(String::as_str),
        Some("thread:burzum")
    );
    assert_eq!(crew.spec.crew_id, crew_id);
}

/// #435 (AGG-20, G12/G12b): `/spawn` → Troll before any Nazgul ended with
/// "Custom crew root was not created." and no reason. The Troll now starts the
/// crew rooted at its parent (the main pane), and a spawn that fails drops it.
#[tokio::test]
async fn sec_435_first_troll_starts_the_crew_at_its_parent() {
    use crate::spawn_orchestration::SpawnRole;
    let mut app = make_test_app().await;
    // The milestone failure.
    let err = app
        .custom_spawn_member_agent_class(SpawnRole::Troll)
        .expect_err("no crew yet");
    assert!(err.to_string().contains("spawn a Nazgul first"), "{err}");

    // The parent `/spawn` gives a Troll: the root node, `pane:codex-main`.
    let root = app.spawn_root_node_id();
    assert_eq!(root, format!("pane:{CODEX_MAIN_PANE_ID}"));
    let (class, started) = app
        .custom_spawn_member_agent_class_under(SpawnRole::Troll, &root)
        .expect("a first Troll starts the crew");
    assert!(started);
    let codex_protocol::crew::AgentClass::CrewMember {
        logical_member_id, ..
    } = class
    else {
        panic!("a Troll is a crew member");
    };
    assert_eq!(logical_member_id, "troll-1");
    let crew = app.spawn_crew.as_ref().expect("crew started");
    assert_eq!(
        crew.member_node_by_id.get("nazgul").map(String::as_str),
        Some(root.as_str())
    );

    // The spawn failed: the crew it started is dropped, so a Nazgul can
    // still be created afterwards.
    app.abort_started_custom_spawn_root();
    assert!(app.spawn_crew.is_none());
    app.preflight_new_custom_spawn_root()
        .expect("a Nazgul can still be created");

    // With a crew already present nothing new is started.
    app.ensure_custom_spawn_root(CODEX_MAIN_PANE_ID)
        .expect("bind custom root");
    let (_, started) = app
        .custom_spawn_member_agent_class_under(SpawnRole::Troll, CODEX_MAIN_PANE_ID)
        .expect("second Troll");
    assert!(!started);
}

#[tokio::test]
async fn native_task_agent_role_does_not_make_it_a_persistent_spawn_crew_member() {
    let mut app = make_test_app().await;
    let crew_orc = ThreadId::new();
    let task_agent = ThreadId::new();
    app.ensure_custom_spawn_root(crate::claude_panes::CODEX_MAIN_PANE_ID)
        .expect("bind custom root");
    app.upsert_agent_picker_thread(
        crew_orc,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        task_agent,
        Some("Ephemeral reviewer".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.record_custom_spawn_member(
        &crate::spawn_orchestration::thread_node_id(crew_orc),
        crate::claude_panes::CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Orc,
        "Snaga".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "k3".to_string(),
            provider: "kimi-code".to_string(),
            reasoning_effort: None,
        },
    )
    .expect("record durable crew member");

    assert!(app.is_managed_spawn_crew_thread(crew_orc));
    assert!(!app.is_managed_spawn_crew_thread(task_agent));
    let crew_rows = app.spawn_tree_items(/*show_task_actions*/ false);
    assert!(crew_rows.iter().any(|item| item.name.contains("Snaga")));
    assert!(
        crew_rows
            .iter()
            .all(|item| !item.name.contains("Ephemeral reviewer"))
    );
}

#[tokio::test]
async fn crewspec_restore_prunes_unrelated_native_tree_but_keeps_crew_descendants() {
    let mut app = make_test_app().await;
    let crew_orc = ThreadId::new();
    let crew_descendant = ThreadId::new();
    let unrelated_main = ThreadId::new();
    let unrelated_descendant = ThreadId::new();
    let crew_orc_node = thread_node_id(crew_orc);
    let crew_descendant_node = thread_node_id(crew_descendant);
    let unrelated_main_node = thread_node_id(unrelated_main);
    let unrelated_descendant_node = thread_node_id(unrelated_descendant);

    app.ensure_custom_spawn_root(CODEX_MAIN_PANE_ID)
        .expect("bind custom root");
    app.record_custom_spawn_member(
        &crew_orc_node,
        CODEX_MAIN_PANE_ID,
        crate::spawn_orchestration::SpawnRole::Orc,
        "Snaga".to_string(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "k3".to_string(),
            provider: "kimi-code".to_string(),
            reasoning_effort: None,
        },
    )
    .expect("record crew Orc");

    app.spawn_parent_by_node
        .insert(crew_orc_node.clone(), CODEX_MAIN_PANE_ID.to_string());
    app.spawn_parent_by_node
        .insert(crew_descendant_node.clone(), crew_orc_node.clone());
    app.spawn_parent_by_node.insert(
        unrelated_main_node.clone(),
        crate::spawn_orchestration::pane_node_id(CODEX_MAIN_PANE_ID),
    );
    app.spawn_parent_by_node.insert(
        unrelated_descendant_node.clone(),
        unrelated_main_node.clone(),
    );

    for (node_id, thread_id) in [
        (&crew_orc_node, crew_orc),
        (&crew_descendant_node, crew_descendant),
        (&unrelated_main_node, unrelated_main),
        (&unrelated_descendant_node, unrelated_descendant),
    ] {
        app.spawn_native_endpoint_by_node
            .insert(node_id.clone(), thread_id);
        app.spawn_native_runtime_by_node.insert(
            node_id.clone(),
            crate::dispatch_queue::SavedNativeSpawnRuntime {
                model: "fixture".to_string(),
                provider: "fixture".to_string(),
                reasoning_effort: None,
            },
        );
    }

    app.prune_noncrew_native_spawn_recovery_nodes();

    assert_eq!(
        app.spawn_parent_by_node.get(&crew_orc_node),
        Some(&CODEX_MAIN_PANE_ID.to_string())
    );
    assert_eq!(
        app.spawn_parent_by_node.get(&crew_descendant_node),
        Some(&crew_orc_node)
    );
    assert!(!app.spawn_parent_by_node.contains_key(&unrelated_main_node));
    assert!(
        !app.spawn_parent_by_node
            .contains_key(&unrelated_descendant_node)
    );
    assert!(
        app.spawn_native_endpoint_by_node
            .contains_key(&crew_descendant_node)
    );
    assert!(
        !app.spawn_native_endpoint_by_node
            .contains_key(&unrelated_main_node)
    );
    assert!(
        !app.spawn_native_runtime_by_node
            .contains_key(&unrelated_descendant_node)
    );
}

impl App {
    /// Test adapter for older unit fixtures. Production has no direct-dispatch bypass: this helper
    /// enters through the same stable model-origin path used by completed agent turns.
    fn dispatch_spawn_task_blocks(
        &mut self,
        source_pane_id: &str,
        dispatches: Vec<crate::spawn_orchestration::SpawnTaskDispatch>,
    ) {
        self.dispatch_spawn_task_blocks_from_model_turn(
            source_pane_id,
            source_pane_id,
            &ThreadId::new().to_string(),
            dispatches,
        );
    }
}

#[tokio::test]
async fn native_turn_completion_is_projection_only_and_never_schedules_tui_delivery() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (parent_thread_id, child_thread_id) = register_native_dispatch_pair(&mut app);

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_completed_with_agent_message(
            child_thread_id,
            "turn-native-terminal",
            TurnStatus::Completed,
            "native terminal result",
        ),
    )));

    while let Ok(event) = rx.try_recv() {
        assert!(
            !matches!(
                event,
                AppEvent::SubmitSpawnAgentTask { .. }
                    | AppEvent::SendSpawnAgentMailboxMessage { .. }
            ),
            "native completion is already delivered by core and must not create a second TUI transport"
        );
    }
    let projected_reports = app
        .spawn_parent_reports_by_node
        .get(&thread_node_id(parent_thread_id))
        .expect("native completion should be projected into durable parent context");
    assert!(
        projected_reports
            .iter()
            .any(|report| report.contains("result=native terminal result")),
        "projection must retain the result without scheduling a second delivery"
    );
    assert_eq!(
        app.agent_navigation
            .get(&child_thread_id)
            .and_then(|entry| entry.last_result_message.as_deref()),
        Some("native terminal result"),
        "the TUI may project the native terminal state without transporting it"
    );
}

#[tokio::test]
async fn external_child_report_enters_native_parent_through_one_canonical_mailbox_message() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (parent_thread_id, _child_thread_id) = register_native_dispatch_pair(&mut app);
    let report = "child_report; seq=41; as_of=2026-07-25T18:00:00Z; child=External [orc]; status=done; result=verified";

    app.record_spawn_parent_report(thread_node_id(parent_thread_id), report.to_string());
    app.record_spawn_parent_report(thread_node_id(parent_thread_id), report.to_string());

    let mut mailbox_messages = Vec::new();
    while let Ok(event) = rx.try_recv() {
        match event {
            AppEvent::SendSpawnAgentMailboxMessage { params } => mailbox_messages.push(params),
            AppEvent::SubmitSpawnAgentTask { .. } => {
                panic!("external reports must not use the obsolete synthetic-assignment path")
            }
            _ => {}
        }
    }
    assert_eq!(
        mailbox_messages.len(),
        1,
        "the edge adapter must deduplicate the report before entering the native mailbox"
    );
    let params = &mailbox_messages[0];
    assert_eq!(params.target_thread_id, parent_thread_id.to_string());
    assert_eq!(
        params.kind,
        codex_protocol::protocol::AgentMessageKind::TerminalResult
    );
    assert!(params.trigger_turn);
    assert_eq!(params.content, report);
    assert!(
        params
            .message_id
            .as_deref()
            .is_some_and(|message_id| message_id.starts_with("external-report:"))
    );
    assert_eq!(params.assignment_id, params.message_id);
}

#[tokio::test]
async fn spawn_roster_keeps_context_pressure_out_of_model_context() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let (troll_thread_id, orc_thread_id) = register_native_dispatch_pair(&mut app);

    app.update_spawn_status_for_thread_notification(&token_usage_notification_with_total(
        orc_thread_id,
        "turn-context",
        /*total_tokens*/ 90_000,
        Some(100_000),
    ));

    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive spawn context");
    assert!(!context.contains("context_left="), "got context: {context}");
}

#[tokio::test]
async fn low_context_telemetry_does_not_manufacture_a_lifecycle_warning() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (troll_thread_id, orc_thread_id) = register_native_dispatch_pair(&mut app);
    app.spawn_operator_input_seen = true;
    let parent_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);

    app.update_spawn_status_for_thread_notification(&token_usage_notification_with_total(
        orc_thread_id,
        "turn-low-1",
        /*total_tokens*/ 90_000,
        Some(100_000),
    ));
    let reports = app.spawn_parent_reports_by_node.get(&parent_node_id);
    assert!(reports.is_none_or(|reports| {
        reports
            .iter()
            .all(|report| !report.contains("low context") && !report.contains("journal/handoff"))
    }));
    assert!(drain_spawn_agent_tasks_for(&mut rx, troll_thread_id).is_empty());

    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive spawn context");
    assert!(!context.contains("context_left="));
}

#[tokio::test]
async fn spawn_roster_lines_carry_dispatch_and_report_seq() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (troll_thread_id, orc_thread_id) = register_native_dispatch_pair(&mut app);
    let source_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);

    app.dispatch_spawn_task_blocks(
        &source_node_id,
        vec![crate::spawn_orchestration::SpawnTaskDispatch {
            target: orc_thread_id.to_string(),
            task: "sequence this work".to_string(),
            seq: Some(120),
        }],
    );
    assert!(matches!(
        rx.try_recv(),
        Ok(AppEvent::SubmitSpawnAgentTask { thread_id, .. }) if thread_id == orc_thread_id
    ));

    app.update_spawn_status_for_thread_notification(&turn_completed_with_agent_message(
        orc_thread_id,
        "turn-report-seq",
        TurnStatus::Completed,
        "sequence proof done",
    ));

    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive spawn context");
    assert!(context.contains("last_dispatch_seq=120"), "got: {context}");
    assert!(context.contains("last_report_seq=121"), "got: {context}");
    assert!(context.contains("as_of="), "got: {context}");

    let reports = app
        .spawn_parent_reports_by_node
        .get(&source_node_id)
        .expect("parent reports");
    assert!(reports.iter().any(|report| {
        report.contains("child_report; seq=121; as_of=") && report.contains("sequence proof done")
    }));
}

#[tokio::test]
async fn oversized_native_self_dispatch_is_rejected_without_delivery() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (source, _target) = register_native_dispatch_pair(&mut app);
    let source_node = thread_node_id(source);
    app.dispatch_spawn_task_blocks(
        &source_node,
        vec![crate::spawn_orchestration::SpawnTaskDispatch {
            target: source.to_string(),
            task: "x".repeat(crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES + 10_000),
            seq: Some(503),
        }],
    );

    while let Ok(event) = rx.try_recv() {
        assert!(
            !matches!(
                event,
                AppEvent::SubmitSpawnAgentTask { .. } | AppEvent::SubmitSpawnClaudePaneTask { .. }
            ),
            "a rejected self-dispatch must not schedule delivery"
        );
    }
}

#[tokio::test]
async fn replacement_transaction_migrates_runtime_waiting_and_relationships() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let (parent, old_thread) = register_native_dispatch_pair(&mut app);
    let new_thread =
        ThreadId::from_string("00000000-0000-0000-0000-000000000453").expect("thread id");
    app.upsert_agent_picker_thread(
        new_thread,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    let old_node = thread_node_id(old_thread);
    app.spawn_parent_by_node
        .insert(old_node.clone(), thread_node_id(parent));
    app.spawn_native_runtime_by_node.insert(
        old_node.clone(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: "saved-custom-model".to_string(),
            provider: "saved-provider".to_string(),
            reasoning_effort: Some(codex_protocol::openai_models::ReasoningEffort::High),
        },
    );
    app.spawn_waiting_for_agents_by_thread
        .insert(old_thread, ("turn-old".to_string(), "wait-old".to_string()));
    app.replace_saved_native_spawn_thread(old_thread, new_thread);

    assert_eq!(
        app.spawn_native_runtime_by_node[&old_node].model,
        "saved-custom-model"
    );
    assert!(
        app.spawn_waiting_for_agents_by_thread
            .contains_key(&new_thread)
    );
    assert_eq!(app.spawn_parent_by_node[&old_node], thread_node_id(parent));
    assert_eq!(app.spawn_native_endpoint_by_node[&old_node], new_thread);
    assert_eq!(app.logical_native_node_for_thread(new_thread), old_node);
}

#[tokio::test]
async fn turn_start_failure_is_buffered_in_only_the_affected_pane() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let affected = ThreadId::from_string("00000000-0000-0000-0000-000000000461")
        .expect("valid affected thread id");
    let unaffected = ThreadId::from_string("00000000-0000-0000-0000-000000000462")
        .expect("valid unaffected thread id");
    app.ensure_thread_channel(affected);
    app.ensure_thread_channel(unaffected);

    app.surface_turn_start_failure(
        affected,
        "turn/start failed in TUI: injected transport fault".to_string(),
        /*will_retry*/ false,
    )
    .await;

    {
        let affected_store = app
            .thread_event_channels
            .get(&affected)
            .expect("affected channel")
            .store
            .lock()
            .await;
        assert!(affected_store.buffer.iter().any(|event| matches!(
            event,
            ThreadBufferedEvent::Notification(notification)
                if matches!(
                    notification.as_ref(),
                    ServerNotification::Error(notification)
                        if !notification.will_retry
                            && notification.error.message.contains("pane remains available")
                            && notification.error.additional_details.as_deref()
                                == Some("turn/start failed in TUI: injected transport fault")
                )
        )));
    }

    let unaffected_store = app
        .thread_event_channels
        .get(&unaffected)
        .expect("unaffected channel")
        .store
        .lock()
        .await;
    assert!(unaffected_store.buffer.is_empty());
}

#[tokio::test]
async fn recovered_turn_start_failure_remains_visible_in_active_pane_history() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    let thread_id = ThreadId::from_string("00000000-0000-0000-0000-000000000463")
        .expect("valid active thread id");
    app.active_thread_id = Some(thread_id);

    app.surface_turn_start_failure(
        thread_id,
        "injected turn/start fault for qualification".to_string(),
        /*will_retry*/ true,
    )
    .await;

    let mut rendered = String::new();
    while let Ok(event) = app_event_rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            rendered.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 100)));
        }
    }
    assert!(rendered.contains("recovered after bounded retry"));
    assert!(rendered.contains("injected turn/start fault for qualification"));
}

#[tokio::test]
async fn failed_dispatch_records_durable_ack_without_fake_native_transport() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let (troll_thread_id, _orc_thread_id) = register_native_dispatch_pair(&mut app);
    app.spawn_operator_input_seen = true;
    let source_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);

    app.dispatch_spawn_task_blocks(
        &source_node_id,
        vec![crate::spawn_orchestration::SpawnTaskDispatch {
            target: "Missing Orc".to_string(),
            task: "handle the missing target".to_string(),
            seq: Some(78),
        }],
    );

    let sender_reports = app
        .spawn_parent_reports_by_node
        .get(&source_node_id)
        .expect("sender reports");
    assert!(sender_reports.iter().any(|report| {
        report.contains("dispatch_ack; #78") && report.contains("status=failed")
    }));
    let failure_ack_turns = drain_spawn_agent_tasks_for(&mut rx, troll_thread_id);
    assert!(
        failure_ack_turns.is_empty(),
        "native agents receive tool results from core; TUI must not synthesize a second turn"
    );
    assert!(
        !app.claude_pane_transcript_cells
            .contains_key(&source_node_id),
        "native sender errors must not be written as fake Claude-pane transcript cells"
    );
}

#[tokio::test]
async fn troll_spawn_task_submission_keeps_mailbox_payload_free_of_synthetic_context() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000132").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000133").expect("valid thread id");
    let snaga_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000134").expect("valid thread id");
    let ghash_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000135").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        snaga_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        ghash_thread_id,
        Some("Ghash".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);
    app.spawn_parent_by_thread
        .insert(snaga_thread_id, troll_thread_id);
    app.spawn_parent_by_thread
        .insert(ghash_thread_id, troll_thread_id);

    app.agent_navigation
        .record_sub_agent_activity(SubAgentActivityDisplay {
            thread_id: snaga_thread_id,
            agent_path: "/root/troll_burzum/orc_snaga".to_string(),
            agent_nickname: None,
            agent_role: None,
            task_preview: None,
            is_running_hint: false,
        });
    app.agent_navigation
        .record_sub_agent_activity(SubAgentActivityDisplay {
            thread_id: ghash_thread_id,
            agent_path: "/root/troll_burzum/orc_ghash".to_string(),
            agent_nickname: None,
            agent_role: None,
            task_preview: None,
            is_running_hint: false,
        });

    let task =
        app.spawn_agent_task_for_submission(troll_thread_id, "Build the site and review it.");

    assert_eq!(task, "Build the site and review it.");
    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive live Core application context");
    assert!(context.contains("Snaga [orc]"));
    assert!(context.contains("/root/troll_burzum/orc_snaga"));
    assert!(context.contains("Ghash [orc]"));
    assert!(context.contains("followup_task"));
    assert!(context.contains("send_message"));
    assert!(!context.contains("<pfterminal_send_task"));
}

#[tokio::test]
async fn troll_live_context_reports_when_no_orcs_exist() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000136").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000137").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);

    let task =
        app.spawn_agent_task_for_submission(troll_thread_id, "Build the site and review it.");

    assert_eq!(task, "Build the site and review it.");
    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive live Core application context");
    assert!(context.contains("Orcs assigned to you:"));
    assert!(context.contains("none assigned yet"));
}

#[tokio::test]
async fn orc_spawn_task_submission_is_the_literal_user_assignment() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000138").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000139").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000140").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);

    let task = app.spawn_agent_task_for_submission(
        orc_thread_id,
        "Run the acceptance command and report its literal output.",
    );

    assert_eq!(
        task,
        "Run the acceptance command and report its literal output."
    );
}

#[tokio::test]
async fn spawn_status_preserves_orc_activity_when_name_arrives_later() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000129").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000130").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000131").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);

    app.agent_navigation
        .record_sub_agent_activity(SubAgentActivityDisplay {
            thread_id: orc_thread_id,
            agent_path: "/root/troll_burzum/orc_snaga".to_string(),
            agent_nickname: None,
            agent_role: None,
            task_preview: Some("build the animated website shell".to_string()),
            is_running_hint: true,
        });
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );

    let entry = app
        .agent_navigation
        .get(&orc_thread_id)
        .expect("orc entry should be merged");
    assert_eq!(entry.agent_nickname.as_deref(), Some("Snaga"));
    assert_eq!(entry.agent_role.as_deref(), Some("orc"));
    assert_eq!(
        entry.agent_path.as_deref(),
        Some("/root/troll_burzum/orc_snaga")
    );
    assert_eq!(
        entry.last_task_message.as_deref(),
        Some("build the animated website shell")
    );

    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    assert!(status_items.iter().any(|item| {
        item.name.contains("Snaga [orc]")
            && item.description.as_deref().is_some_and(|description| {
                description.contains("current task: build the animated website shell")
            })
    }));
}

#[tokio::test]
async fn native_spawn_turn_completion_updates_status_and_result_preview() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000161").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000162").expect("valid thread id");

    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);
    app.agent_navigation
        .set_last_task_message(orc_thread_id, Some("build components".to_string()));

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_started_notification(orc_thread_id, "turn-1"),
    )));
    assert_eq!(
        app.spawn_status_by_thread
            .get(&orc_thread_id)
            .map(|state| &state.status),
        Some(&codex_app_server_protocol::CollabAgentStatus::Running)
    );

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_completed_with_agent_message(
            orc_thread_id,
            "turn-1",
            TurnStatus::Completed,
            "Created the missing components and npm run build passed cleanly.",
        ),
    )));

    let status = app
        .spawn_status_by_thread
        .get(&orc_thread_id)
        .expect("spawn status should be cached");
    assert_eq!(
        status.status,
        codex_app_server_protocol::CollabAgentStatus::Completed
    );
    assert_eq!(
        status.message.as_deref(),
        Some("Created the missing components and npm run build passed cleanly.")
    );
    let entry = app
        .agent_navigation
        .get(&orc_thread_id)
        .expect("orc should stay in picker");
    assert!(!entry.is_running);
    assert_eq!(
        entry.last_result_message.as_deref(),
        Some("Created the missing components and npm run build passed cleanly.")
    );
    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    assert!(status_items.iter().any(|item| {
        item.name.contains("Snaga [orc]")
            && item.description.as_deref().is_some_and(|description| {
                description.contains("completed")
                    && description.contains("latest result: Created the missing components")
                    && !description.starts_with("running")
            })
    }));
}

#[tokio::test]
async fn stale_receiver_running_status_does_not_hide_completed_orc_report() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000163").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000164").expect("valid thread id");

    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(orc_thread_id),
        crate::spawn_orchestration::thread_node_id(troll_thread_id),
    );

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_started_notification(orc_thread_id, "turn-1"),
    )));
    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_completed_with_agent_message(
            orc_thread_id,
            "turn-1",
            TurnStatus::Completed,
            "Wrote /tmp/pfterminal-orc-proof.txt and verified it.",
        ),
    )));

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        ServerNotification::ItemCompleted(codex_app_server_protocol::ItemCompletedNotification {
            thread_id: troll_thread_id.to_string(),
            turn_id: "turn-wait".to_string(),
            completed_at_ms: 0,
            item: ThreadItem::untimed(ThreadItemPayload::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: codex_app_server_protocol::CollabAgentTool::Wait,
                status: codex_app_server_protocol::CollabAgentToolCallStatus::Completed,
                sender_thread_id: troll_thread_id.to_string(),
                receiver_thread_ids: vec![orc_thread_id.to_string()],
                prompt: None,
                model: None,
                reasoning_effort: None,
                provider_account: None,
                agents_states: HashMap::from([(
                    orc_thread_id.to_string(),
                    codex_app_server_protocol::CollabAgentState {
                        status: codex_app_server_protocol::CollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            }),
        }),
    )));

    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive live lifecycle context");
    assert!(
        context.contains("Snaga [orc]; status=completed"),
        "Troll roster must show the completed Orc, got: {context}"
    );
    assert!(
        !context.contains("Snaga [orc]; status=running"),
        "stale receiver status must not regress the completed Orc to running, got: {context}"
    );
    assert!(!context.contains("has_new_report="));
    assert!(!context.contains("Recent child reports delivered to this pane:"));
    assert!(!context.contains("result=Wrote /tmp/pfterminal-orc-proof.txt and verified it."));
}

#[tokio::test]
async fn native_orc_completion_is_projected_but_not_reinjected_into_parent_context() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000231").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000232").expect("valid thread id");

    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);
    app.agent_navigation
        .set_last_task_message(orc_thread_id, Some("audit latency hot paths".to_string()));

    app.enqueue_thread_notification(
        orc_thread_id,
        turn_completed_with_agent_message(
            orc_thread_id,
            "turn-1",
            TurnStatus::Completed,
            "Found two latency issues and no blockers.",
        ),
    )
    .await
    .expect("completion should enqueue");

    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive live lifecycle context");
    assert!(context.contains("Snaga [orc]; status=completed"));
    assert!(!context.contains("has_new_report="));
    assert!(!context.contains("Recent child reports delivered to this pane:"));
    assert!(!context.contains("result=Found two latency issues and no blockers."));
    let reports = app
        .spawn_parent_reports_by_node
        .get(&crate::spawn_orchestration::thread_node_id(troll_thread_id))
        .expect("TUI should retain a display projection of the result");
    assert!(
        reports
            .iter()
            .any(|report| report.contains("result=Found two latency issues and no blockers."))
    );
}

#[tokio::test]
async fn text_only_acknowledge_spawn_turn_reports_done() {
    let mut app = make_test_app().await;
    let (troll_thread_id, orc_thread_id) = register_native_dispatch_pair(&mut app);

    app.update_spawn_status_for_thread_notification(&turn_completed_with_agent_message(
        orc_thread_id,
        "turn-text-only-ack",
        TurnStatus::Completed,
        "Acknowledged. Standing by.",
    ));

    assert_eq!(
        app.spawn_status_by_thread
            .get(&orc_thread_id)
            .map(|state| &state.status),
        Some(&codex_app_server_protocol::CollabAgentStatus::Completed)
    );
    let parent_node_id = crate::spawn_orchestration::thread_node_id(troll_thread_id);
    let reports = app
        .spawn_parent_reports_by_node
        .get(&parent_node_id)
        .expect("parent report");
    assert!(reports.iter().any(|report| {
        report.contains("child_report;")
            && report.contains("status=completed")
            && report.contains("result=Acknowledged. Standing by.")
    }));
}

#[tokio::test]
async fn native_orc_completion_is_reported_to_claude_troll_context() {
    let mut app = make_test_app().await;
    let troll_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create Claude Troll pane");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000233").expect("valid thread id");

    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Ghash".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(orc_thread_id),
        crate::spawn_orchestration::pane_node_id(&troll_pane_id),
    );

    app.enqueue_thread_notification(
        orc_thread_id,
        turn_completed_with_agent_message(
            orc_thread_id,
            "turn-1",
            TurnStatus::Completed,
            "Delivered the code-quality audit with three concrete findings.",
        ),
    )
    .await
    .expect("completion should enqueue");

    let context = app
        .spawn_context_for_user_pane(&troll_pane_id)
        .expect("Troll pane should receive spawn context");
    assert!(context.contains("Recent child reports delivered to this pane:"));
    assert!(context.contains("Ghash [orc]; status=completed; has_new_report=true"));
    assert!(
        context.contains("result=Delivered the code-quality audit with three concrete findings.")
    );
}

#[tokio::test]
async fn direct_six_orc_turn_reports_are_visible_to_claude_troll_context() {
    let mut app = make_test_app().await;
    let troll_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create Claude Troll pane");
    let native_orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000235").expect("valid thread id");
    app.upsert_agent_picker_thread(
        native_orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::thread_node_id(native_orc_thread_id),
        crate::spawn_orchestration::pane_node_id(&troll_pane_id),
    );

    let claude_orc_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Ghash".to_string()),
        )
        .expect("create Claude Orc pane");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&claude_orc_pane_id),
        crate::spawn_orchestration::pane_node_id(&troll_pane_id),
    );

    let turn_reports = [
        ("native", "turn-1", "ORC_A_TURN_1_DONE: alpha finding"),
        ("claude", "turn-2", "ORC_B_TURN_2_DONE: beta finding"),
        ("native", "turn-3", "ORC_A_TURN_3_DONE: gamma finding"),
        ("claude", "turn-4", "ORC_B_TURN_4_DONE: delta finding"),
        ("native", "turn-5", "ORC_A_TURN_5_DONE: epsilon finding"),
        ("claude", "turn-6", "ORC_B_TURN_6_DONE: zeta finding"),
    ];

    for (index, (harness, turn_id, report)) in turn_reports.iter().enumerate() {
        if *harness == "native" {
            app.enqueue_thread_notification(
                native_orc_thread_id,
                turn_completed_with_agent_message(
                    native_orc_thread_id,
                    turn_id,
                    TurnStatus::Completed,
                    report,
                ),
            )
            .await
            .expect("native Orc completion should enqueue");
        } else {
            app.on_claude_pane_turn_finished(
                claude_orc_pane_id.clone(),
                Ok(crate::claude_panes::ClaudePaneTurnOutput {
                    text: (*report).to_string(),
                    status: crate::claude_panes::ClaudePaneTurnStatus::Success,
                    session_id: Some("claude-session".to_string()),
                    usage_summary: None,
                    turn_usage_summary: None,
                    usage_status: crate::claude_panes::ClaudePaneUsageStatus::Missing,
                    artifact_path: app
                        .config
                        .cwd
                        .join(format!("{turn_id}.jsonl"))
                        .to_path_buf(),
                    audit_path: app
                        .config
                        .cwd
                        .join(format!("{turn_id}.audit.json"))
                        .to_path_buf(),
                    duration_ms: 1,
                    terminal_reason: None,
                    error_summary: None,
                    tool_names: Vec::new(),
                    tool_events: Vec::new(),
                    reasoning_events: Vec::new(),
                    command_mode: crate::claude_panes::ClaudeCommandMode::NewSession,
                    direct_accounting: None,
                }),
            );
        }

        let context = app
            .spawn_context_for_user_pane(&troll_pane_id)
            .expect("Troll pane should receive spawn context");
        assert!(context.contains("Recent child reports delivered to this pane:"));
        for (_, _, expected_report) in &turn_reports[..=index] {
            assert!(context.contains(expected_report));
        }
    }

    let context = app
        .spawn_context_for_user_pane(&troll_pane_id)
        .expect("Troll pane should receive spawn context");
    assert!(context.contains("Snaga [orc]; status=completed; has_new_report=true"));
    assert!(context.contains("Claude Code Ghash [orc] - Opus 5.5 Claude Plan; status=success"));
}

#[tokio::test]
async fn enqueued_native_spawn_turn_completion_updates_status_before_replay() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000165").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000166").expect("valid thread id");

    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);
    app.agent_navigation
        .set_last_task_message(orc_thread_id, Some("write proof file".to_string()));

    app.enqueue_thread_notification(
        orc_thread_id,
        turn_completed_with_agent_message(
            orc_thread_id,
            "turn-1",
            TurnStatus::Completed,
            "Wrote /tmp/pfterminal-spawn-status-proof.txt and verified it.",
        ),
    )
    .await
    .expect("completion should enqueue");

    let status = app
        .spawn_status_by_thread
        .get(&orc_thread_id)
        .expect("spawn status should be cached before replay");
    assert_eq!(
        status.status,
        codex_app_server_protocol::CollabAgentStatus::Completed
    );
    assert_eq!(
        status.message.as_deref(),
        Some("Wrote /tmp/pfterminal-spawn-status-proof.txt and verified it.")
    );

    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    assert!(status_items.iter().any(|item| {
        item.name.contains("Snaga [orc]")
            && item.description.as_deref().is_some_and(|description| {
                description.contains("completed")
                    && description
                        .contains("latest result: Wrote /tmp/pfterminal-spawn-status-proof")
                    && !description.starts_with("idle")
            })
    }));
}

#[tokio::test]
async fn native_spawn_turn_interrupt_updates_status_without_closing_app() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000163").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000164").expect("valid thread id");

    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Ghash".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);

    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_started_notification(orc_thread_id, "turn-1"),
    )));
    app.handle_thread_event_now(ThreadBufferedEvent::Notification(Box::new(
        turn_completed_notification(orc_thread_id, "turn-1", TurnStatus::Interrupted),
    )));

    let status = app
        .spawn_status_by_thread
        .get(&orc_thread_id)
        .expect("spawn status should be cached");
    assert_eq!(
        status.status,
        codex_app_server_protocol::CollabAgentStatus::Interrupted
    );
    let entry = app
        .agent_navigation
        .get(&orc_thread_id)
        .expect("orc should stay in picker");
    assert!(!entry.is_running);

    let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
    assert!(status_items.iter().any(|item| {
        item.name.contains("Ghash [orc]")
            && item.description.as_deref().is_some_and(|description| {
                description.starts_with("interrupted") && !description.starts_with("running")
            })
    }));
}

#[tokio::test]
async fn bound_claude_nazgul_context_explains_empty_spawn_hierarchy() {
    let mut app = make_test_app().await;
    app.spawn_nazgul_pane_id = Some("claude-test-pane".to_string());

    let context = app
        .spawn_context_for_user_pane("claude-test-pane")
        .expect("bound Claude pane should receive spawn context");

    assert!(context.contains("You are the Corbanu Terminal Nazgul/root pane"));
    assert!(context.contains("Troll and Orc are Corbanu Terminal orchestration roles"));
    assert!(context.contains("Trolls: none spawned yet."));
    assert!(context.contains("Orcs: none spawned yet."));
    assert!(context.contains("suggest using /spawn"));
    assert!(app.spawn_context_for_user_pane("other-pane").is_none());
}

#[tokio::test]
async fn bound_claude_nazgul_context_lists_named_trolls_and_orcs() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000132").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000133").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000134").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.spawn_nazgul_pane_id = Some("claude-nazgul".to_string());
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, main_thread_id);
    app.spawn_parent_by_thread
        .insert(orc_thread_id, troll_thread_id);
    app.agent_navigation.set_last_task_message(
        orc_thread_id,
        Some("review the large diff before reporting".to_string()),
    );
    app.agent_navigation.set_last_result_message(
        orc_thread_id,
        Some("found a missing hierarchy context bridge".to_string()),
    );

    let context = app
        .spawn_context_for_user_pane("claude-nazgul")
        .expect("bound Claude pane should receive spawn context");

    assert!(context.contains("Burzum [troll]; status=idle"));
    assert!(context.contains("Snaga [orc]; status=idle"));
    assert!(context.contains("current_task=review the large diff before reporting"));
    assert!(context.contains("latest_result=found a missing hierarchy context bridge"));
}

#[tokio::test]
async fn bound_claude_nazgul_context_auto_nests_orphans_under_single_claude_troll() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000135").expect("valid thread id");
    let orc_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000136").expect("valid thread id");

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.spawn_nazgul_pane_id = Some("claude-nazgul".to_string());
    let troll_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create Claude Troll pane");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&troll_pane_id),
        crate::spawn_orchestration::pane_node_id("claude-nazgul"),
    );
    app.upsert_agent_picker_thread(
        orc_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(orc_thread_id, main_thread_id);

    let context = app
        .spawn_context_for_user_pane("claude-nazgul")
        .expect("bound Claude pane should receive spawn context");

    assert!(context.contains("Claude Code Burzum [troll] - Opus 5.5 Claude Plan; role=Troll"));
    assert!(context.contains("  - Snaga [orc]; status=idle"));
    assert!(!context.contains("Unassigned Orcs"));
}

#[tokio::test]
async fn claude_orc_completion_is_reported_to_parent_troll_context() {
    let mut app = make_test_app().await;
    let troll_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create Claude Troll pane");
    let orc_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Snaga".to_string()),
        )
        .expect("create Claude Orc pane");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&orc_pane_id),
        crate::spawn_orchestration::pane_node_id(&troll_pane_id),
    );

    let artifact_path = app.config.cwd.join("turn-0001.jsonl").to_path_buf();
    let audit_path = app.config.cwd.join("turn-0001.audit.json").to_path_buf();
    app.on_claude_pane_turn_finished(
        orc_pane_id,
        Ok(crate::claude_panes::ClaudePaneTurnOutput {
            text: "Implemented the mock website and npm run build passed.".to_string(),
            status: crate::claude_panes::ClaudePaneTurnStatus::Success,
            session_id: Some("claude-session".to_string()),
            usage_summary: None,
            turn_usage_summary: None,
            usage_status: crate::claude_panes::ClaudePaneUsageStatus::Missing,
            artifact_path,
            audit_path,
            duration_ms: 1,
            terminal_reason: None,
            error_summary: None,
            tool_names: Vec::new(),
            tool_events: Vec::new(),
            reasoning_events: Vec::new(),
            command_mode: crate::claude_panes::ClaudeCommandMode::NewSession,
            direct_accounting: None,
        }),
    );

    let context = app
        .spawn_context_for_user_pane(&troll_pane_id)
        .expect("Troll pane should receive spawn context");
    assert!(context.contains("Recent child reports delivered to this pane:"));
    assert!(context.contains("Claude Code Snaga [orc] - Opus 5.5 Claude Plan"));
    assert!(context.contains("status=idle"));
    assert!(context.contains("result=Implemented the mock website and npm run build passed."));
}

pub(super) async fn make_child_report_auto_claude_pane_app() -> (
    Box<App>,
    tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    String,
    String,
) {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    app.spawn_operator_input_seen = true;
    let troll_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create Claude Troll pane");
    let orc_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Snaga".to_string()),
        )
        .expect("create Claude Orc pane");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&orc_pane_id),
        crate::spawn_orchestration::pane_node_id(&troll_pane_id),
    );
    while app_event_rx.try_recv().is_ok() {}
    (app, app_event_rx, troll_pane_id, orc_pane_id)
}

pub(super) fn drain_claude_pane_task_events(
    app_event_rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
) -> Vec<(String, String)> {
    let mut submitted_tasks = Vec::new();
    while let Ok(event) = app_event_rx.try_recv() {
        if let AppEvent::SubmitSpawnClaudePaneTask { pane_id, task, .. } = event {
            submitted_tasks.push((pane_id, task));
        }
    }
    submitted_tasks
}

pub(super) fn write_test_whip(app: &App, name: &str, body: &str) {
    let dir = app.config.codex_home.join("whips");
    std::fs::create_dir_all(&dir).expect("create whips dir");
    std::fs::write(dir.join(format!("{name}.md")), body).expect("write whip");
}

#[tokio::test]
async fn orchestrate_auto_whip_fires_once_for_idle_sweep() {
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
    app.sweep_orchestrate_whips();
    app.sweep_orchestrate_whips();

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(submitted_tasks.len(), 1);
    assert_eq!(submitted_tasks[0].0, pane_id);
    assert!(submitted_tasks[0].1.contains("Whip #whip-1 fire 1/3"));
    assert!(submitted_tasks[0].1.contains("Continue the work."));
}

#[tokio::test]
async fn native_legacy_whip_target_receives_orchestration_lifecycle() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "keep-going", "# whip: keep-going\nContinue the work.");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000467").expect("worker id");
    app.upsert_agent_picker_thread(
        worker_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);

    app.handle_orchestrate_command(format!(
        "attach {worker_node} keep-going --mode auto --holder none --max 2"
    ));

    assert!(
        app.is_spawn_orchestration_thread(worker_thread_id),
        "an armed native legacy-whip target must receive completion lifecycle events"
    );
    app.orchestrate_whips
        .get_mut("whip-1")
        .expect("legacy whip")
        .state = crate::orchestrate::WhipState::Exhausted;
    assert!(
        app.is_spawn_orchestration_thread(worker_thread_id),
        "an exhausted whip target must remain classified until its terminal turn arrives"
    );
}

#[tokio::test]
async fn orchestrate_stop_marker_pauses_before_fire() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "stop-aware",
        "# whip: stop-aware\nContinue until done.",
    );
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
    let target_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);

    app.handle_orchestrate_command(format!(
        "attach {pane_id} stop-aware --mode auto --holder none --max 3"
    ));
    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("done\nWHIP_DONE"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );

    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());
    assert_eq!(
        app.orchestrate_whips.get("whip-1").map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused)
    );
}

#[tokio::test]
async fn orchestrate_empty_output_loop_pauses_whip() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "loop-aware",
        "# whip: loop-aware\nContinue unless spinning.",
    );
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
    let target_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);

    app.handle_orchestrate_command(format!(
        "attach {pane_id} loop-aware --mode auto --holder none --max 3 --cooldown 1s"
    ));
    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some(""),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );
    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("   "),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(submitted_tasks.len(), 1);
    assert_eq!(
        app.orchestrate_whips.get("whip-1").map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused)
    );
}

#[tokio::test]
async fn orchestrate_failed_turn_loop_pauses_and_success_resets_streak() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "failure-aware",
        "# whip: failure-aware\nContinue the work.",
    );
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
    let target_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);

    app.handle_orchestrate_command(format!(
        "attach {pane_id} failure-aware --mode auto --holder none --max 5 --cooldown 1s"
    ));
    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("provider error"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ false,
    );
    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("recovered"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );
    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("provider error"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ false,
    );
    assert_eq!(
        app.orchestrate_whips
            .get("whip-1")
            .map(|whip| whip.consecutive_failed_turns),
        Some(1)
    );

    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("provider error"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ false,
    );
    let whip = app.orchestrate_whips.get("whip-1").expect("whip");
    assert_eq!(whip.state, crate::orchestrate::WhipState::Paused);
    assert!(whip.fires <= 2);
    assert!(drain_claude_pane_task_events(&mut app_event_rx).len() <= 2);
}

#[tokio::test]
async fn assignment_overnight_loop_survives_cycles_backoff_and_manager_markers() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "overnight",
        "# assignment: overnight\nShip the flagship.",
    );
    let manager_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Manager".to_string()),
        )
        .expect("create manager");
    let worker_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Worker".to_string()),
        )
        .expect("create worker");
    let manager_node = crate::spawn_orchestration::pane_node_id(&manager_pane_id);
    let worker_node = crate::spawn_orchestration::pane_node_id(&worker_pane_id);
    let started = chrono::DateTime::parse_from_rfc3339("2026-07-10T20:00:00Z")
        .expect("timestamp")
        .with_timezone(&chrono::Utc);
    app.orchestrate_now_override = Some(started);

    app.handle_orchestrate_command(format!(
        "attach {worker_pane_id} overnight --mode review --holder {manager_pane_id} --for 8h --cooldown 900s"
    ));
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Drafting,
            ..
        })
    ));
    let birth_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(birth_tasks.len(), 1);
    assert!(birth_tasks[0].1.contains("The spec below is locked."));
    assert!(
        birth_tasks[0]
            .1
            .contains("Send the first concrete Worker task now")
    );
    assert!(birth_tasks[0].1.contains(&worker_node));
    assert!(birth_tasks[0].1.contains("wait for the Worker result"));
    assert!(birth_tasks[0].1.contains("not a shell command or tool"));
    assert!(!birth_tasks[0].1.contains("First iterate with the user"));
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("I will emit WHIP_DONE when complete and ASSIGNMENT_BLOCKED: <reason> if needed."),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Drafting,
            ..
        })
    ));
    app.sweep_orchestrate_whips();
    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());

    app.note_whip_holder_dispatched(&manager_node, &worker_node);
    assert!(matches!(
        app.orchestrate_whips.get("assignment-1").map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            execution_started_utc: Some(value),
            ..
        }) if *value == started
    ));
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .and_then(|whip| whip.expires_at),
        Some(started + chrono::Duration::hours(8))
    );
    app.note_assignment_user_turn(&manager_node);
    app.note_whip_target_idle_with_fire_control(
        &worker_node,
        Some("WORKER_AUDIT_RESULT: baseline checkout is invalid"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(
        drain_claude_pane_task_events(&mut app_event_rx).is_empty(),
        "a Worker completion must not preempt recent user activity"
    );
    if let Some(whip) = app.orchestrate_whips.get_mut("assignment-1") {
        whip.last_fire_utc = Some(started);
        if let crate::orchestrate::WhipKind::Assignment {
            last_user_turn_utc, ..
        } = &mut whip.kind
        {
            *last_user_turn_utc = None;
        }
    }
    app.orchestrate_now_override = Some(started + chrono::Duration::milliseconds(500));
    app.sweep_orchestrate_whips();
    assert!(
        drain_claude_pane_task_events(&mut app_event_rx).is_empty(),
        "completion handoffs must be rate limited"
    );
    app.orchestrate_now_override = Some(started + chrono::Duration::seconds(3));
    app.sweep_orchestrate_whips();
    assert_eq!(
        drain_claude_pane_task_events(&mut app_event_rx).len(),
        1,
        "the pending completion must retry after the rate-limit window"
    );
    let assignment = app
        .orchestrate_whips
        .get("assignment-1")
        .expect("assignment");
    let mandate = crate::orchestrate::assignment_mandate_task(
        assignment,
        "Worker",
        Some("Run the audit."),
        /*path*/ None,
        started,
        crate::orchestrate::AssignmentDispatchProtocol::HostAdapter,
    );
    assert!(mandate.contains("Worker's latest completed output:"));
    assert!(mandate.contains("WORKER_AUDIT_RESULT: baseline checkout is invalid"));
    for cycle in 1..=24 {
        app.orchestrate_now_override =
            Some(started + chrono::Duration::seconds(3) + chrono::Duration::seconds(901 * cycle));
        app.sweep_orchestrate_whips();
    }
    assert_eq!(drain_claude_pane_task_events(&mut app_event_rx).len(), 24);
    let assignment = app
        .orchestrate_whips
        .get("assignment-1")
        .expect("assignment");
    assert_eq!(assignment.state, crate::orchestrate::WhipState::Armed);
    assert_eq!(assignment.fires, 25); // one completion retry plus 24 watchdog cycles

    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("temporary provider failure"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ false,
    );
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("temporary provider failure"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ false,
    );
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(crate::orchestrate::assignment_effective_cadence_s),
        Some(3600)
    );
    let backed_off_now = started + chrono::Duration::hours(7);
    app.orchestrate_now_override = Some(backed_off_now);
    if let Some(whip) = app.orchestrate_whips.get_mut("assignment-1") {
        whip.last_fire_utc = Some(backed_off_now - chrono::Duration::seconds(3601));
        if let crate::orchestrate::WhipKind::Assignment {
            last_user_turn_utc, ..
        } = &mut whip.kind
        {
            *last_user_turn_utc = Some(backed_off_now - chrono::Duration::minutes(20));
        }
    }
    app.sweep_orchestrate_whips();
    assert_eq!(drain_claude_pane_task_events(&mut app_event_rx).len(), 1);
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("recovered"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(crate::orchestrate::assignment_effective_cadence_s),
        Some(900)
    );

    app.note_whip_target_idle_with_fire_control(
        &worker_node,
        Some("ASSIGNMENT_BLOCKED: worker said it\nWHIP_DONE"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            ..
        })
    ));
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("I may emit WHIP_DONE later and mention ASSIGNMENT_BLOCKED: in prose."),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            ..
        })
    ));
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("ASSIGNMENT_BLOCKED:\nprogress continues"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            ..
        })
    ));
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("progress\nASSIGNMENT_BLOCKED: waiting for production credentials"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Blocked { .. },
            ..
        })
    ));
    app.note_assignment_user_turn(&manager_node);
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            ..
        })
    ));
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("WHIP_DONE"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Done,
            ..
        })
    ));
}

#[tokio::test]
async fn assignment_manager_empty_completion_retries_current_turn_once_then_pauses() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "empty-manager", "Draft and run the recovery audit.");
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000463").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000464").expect("worker id");
    app.upsert_agent_picker_thread(
        manager_thread_id,
        Some("Manager".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        worker_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);
    app.handle_orchestrate_command(format!(
        "attach {worker_node} empty-manager --mode review --holder {manager_node} --for 1h"
    ));
    while app_event_rx.try_recv().is_ok() {}
    assert!(
        app.is_spawn_orchestration_thread(manager_thread_id),
        "an active native assignment Manager must receive terminal lifecycle events"
    );
    assert!(
        app.is_spawn_orchestration_thread(worker_thread_id),
        "an active native assignment Worker must receive terminal lifecycle events"
    );

    // This is the incident shape: the picker still has an older visible answer, but the current
    // provider turn completes without an AgentMessage item.
    app.agent_navigation.set_last_result_message(
        manager_thread_id,
        Some("Before I dispatch, give me the assignment spec.".to_string()),
    );
    app.update_spawn_status_for_thread_notification(&turn_completed_notification(
        manager_thread_id,
        "manager-empty-1",
        TurnStatus::Completed,
    ));
    assert_eq!(
        app.agent_navigation
            .get(&manager_thread_id)
            .and_then(|entry| entry.last_result_message.as_deref()),
        Some("Turn completed without visible output."),
        "the picker should describe the terminal state without hiding the empty provider result"
    );

    let retry = drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id)
        .expect("first empty completion should retry the Manager");
    assert!(retry.contains("previous turn completed successfully"));
    assert!(retry.contains("latest user message already present"));
    assert!(retry.contains("Do not ask the user to repeat"));
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| (whip.state, whip.empty_output_fires)),
        Some((crate::orchestrate::WhipState::Armed, 1))
    );

    // The app observes the same completion once when received and again during buffered replay.
    // Replaying that terminal notification must not consume the one allowed recovery retry.
    app.update_spawn_status_for_thread_notification(&turn_completed_notification(
        manager_thread_id,
        "manager-empty-1",
        TurnStatus::Completed,
    ));
    assert!(drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id).is_none());
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| (whip.state, whip.empty_output_fires)),
        Some((crate::orchestrate::WhipState::Armed, 1))
    );

    app.update_spawn_status_for_thread_notification(&turn_completed_notification(
        manager_thread_id,
        "manager-empty-2",
        TurnStatus::Completed,
    ));
    assert!(drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id).is_none());
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused)
    );
}

#[tokio::test]
async fn assignment_manager_visible_paraphrase_resets_empty_completion_guard() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "empty-reset", "Draft and run the recovery audit.");
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000465").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000466").expect("worker id");
    app.upsert_agent_picker_thread(
        manager_thread_id,
        Some("Manager".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        worker_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);
    app.handle_orchestrate_command(format!(
        "attach {worker_node} empty-reset --mode review --holder {manager_node} --for 1h"
    ));
    while app_event_rx.try_recv().is_ok() {}

    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        /*last_output*/ None,
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    let _ = drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id)
        .expect("first empty completion should retry");
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("I have enough context; drafting the acceptance criteria now."),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| whip.empty_output_fires),
        Some(0)
    );
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("   "),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );
    assert!(drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id).is_some());
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Armed)
    );
}

#[tokio::test]
async fn codex_pane_description_uses_cached_model_instead_of_unknown() {
    let mut app = make_test_app().await;
    let thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000467").expect("thread id");
    app.upsert_agent_picker_thread(
        thread_id,
        Some("Manager".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.agent_navigation
        .set_model(thread_id, Some("claude-fable-5-plan".to_string()));
    let entry = app.agent_navigation.get(&thread_id).expect("picker entry");

    let description = app.codex_pane_description(thread_id, entry);

    assert!(
        description.starts_with("claude-fable-5-plan; idle"),
        "{description}"
    );
    assert!(!description.contains("model unknown"), "{description}");
}

#[tokio::test]
async fn assignment_bad_target_retries_durable_worker_once_then_pauses() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "durable-retry", "Run the recovery audit.");
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000471").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000472").expect("worker id");
    app.upsert_agent_picker_thread(
        manager_thread_id,
        Some("Manager".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        worker_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);
    app.handle_orchestrate_command(format!(
        "attach {worker_node} durable-retry --mode review --holder {manager_node} --for 1h"
    ));
    while app_event_rx.try_recv().is_ok() {}

    app.dispatch_spawn_task_blocks(
        &manager_node,
        vec![crate::spawn_orchestration::SpawnTaskDispatch {
            target: "Worker nickname that does not resolve".to_string(),
            task: "Audit read-only state.".to_string(),
            seq: Some(1),
        }],
    );

    let retried_task = drain_spawn_agent_task_for(&mut app_event_rx, worker_thread_id)
        .expect("one retry should target the durable Worker ID");
    let assignment = app
        .orchestrate_whips
        .get("assignment-1")
        .expect("assignment");
    assert_eq!(assignment.state, crate::orchestrate::WhipState::Armed);
    assert!(
        assignment
            .last_dispatch_result
            .as_deref()
            .is_some_and(|result| result.contains("retrying durable Worker ID"))
    );

    app.record_spawn_dispatch_failed_for_task(
        &worker_node,
        &retried_task,
        "Worker closed before turn start",
    );

    let assignment = app
        .orchestrate_whips
        .get("assignment-1")
        .expect("assignment");
    assert_eq!(assignment.state, crate::orchestrate::WhipState::Paused);
    assert!(
        assignment
            .last_dispatch_result
            .as_deref()
            .is_some_and(|result| result.contains("retry failed"))
    );
    assert!(drain_spawn_agent_task_for(&mut app_event_rx, worker_thread_id).is_none());
}

#[tokio::test]
async fn assignment_creation_fails_visibly_when_layout_cannot_be_persisted() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "persistence-check",
        "Verify durable assignment state.",
    );
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000481").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000482").expect("worker id");
    app.upsert_agent_picker_thread(
        manager_thread_id,
        Some("Manager".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        worker_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.primary_thread_id = Some(ThreadId::new());

    let panes_path = app.config.codex_home.join("panes");
    if panes_path.exists() {
        std::fs::remove_dir_all(&panes_path).expect("remove test panes directory");
    }
    std::fs::write(&panes_path, "path collision").expect("create persistence failure fixture");

    app.handle_orchestrate_command(format!(
        "attach {} persistence-check --mode review --holder {} --for 1h",
        crate::spawn_orchestration::thread_node_id(worker_thread_id),
        crate::spawn_orchestration::thread_node_id(manager_thread_id),
    ));

    assert!(app.orchestrate_whips.is_empty());
    let mut rendered = String::new();
    while let Ok(event) = app_event_rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            rendered.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 120)));
        }
    }
    assert!(rendered.contains("Pane layout was not saved"), "{rendered}");
    assert!(
        rendered.contains("Assignment was not created because its state could not be saved"),
        "{rendered}"
    );
}

#[tokio::test]
async fn assignment_unreachable_watchdog_pauses_after_four_cadences() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "watchdog", "Keep managing.");
    let manager_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            /*spawn_role*/ None,
            Some("Manager".to_string()),
        )
        .expect("create manager");
    let worker_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            /*spawn_role*/ None,
            Some("Worker".to_string()),
        )
        .expect("create worker");
    let manager_node = crate::spawn_orchestration::pane_node_id(&manager_pane_id);
    let worker_node = crate::spawn_orchestration::pane_node_id(&worker_pane_id);
    let started = chrono::DateTime::parse_from_rfc3339("2026-07-10T20:00:00Z")
        .expect("timestamp")
        .with_timezone(&chrono::Utc);
    app.orchestrate_now_override = Some(started);
    app.handle_orchestrate_command(format!(
        "attach {worker_pane_id} watchdog --mode review --holder {manager_pane_id} --for 8h --cooldown 900s"
    ));
    app.note_whip_holder_dispatched(&manager_node, &worker_node);
    app.orchestrate_whips
        .get_mut("assignment-1")
        .expect("assignment")
        .target = "pane:missing-worker".to_string();

    app.sweep_orchestrate_whips();
    app.orchestrate_now_override = Some(started + chrono::Duration::seconds(3599));
    app.sweep_orchestrate_whips();
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Armed)
    );
    app.orchestrate_now_override = Some(started + chrono::Duration::seconds(3600));
    app.sweep_orchestrate_whips();
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused)
    );
}

#[tokio::test]
async fn assignment_for_one_hour_keeps_the_whole_hour() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "watchdog", "Keep managing.");
    let manager_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            /*spawn_role*/ None,
            Some("Manager".to_string()),
        )
        .expect("create manager");
    let worker_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            /*spawn_role*/ None,
            Some("Worker".to_string()),
        )
        .expect("create worker");
    // No clock override: the real clock moves between reads, as in the product.
    assert_eq!(app.orchestrate_now_override, None);
    app.handle_orchestrate_command(format!(
        "attach {worker_pane_id} watchdog --mode review --holder {manager_pane_id} --for 1h"
    ));

    let execution_duration_s =
        app.orchestrate_whips
            .get("assignment-1")
            .and_then(|whip| match &whip.kind {
                crate::orchestrate::WhipKind::Assignment {
                    execution_duration_s,
                    ..
                } => *execution_duration_s,
                crate::orchestrate::WhipKind::LegacyNudge => None,
            });
    assert_eq!(execution_duration_s, Some(3600));
}

#[tokio::test]
async fn assignment_restart_lifecycle_waits_one_cadence_then_continues() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "restart-loop", "Continue the recovery work.");
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000501").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000502").expect("worker id");
    app.upsert_agent_picker_thread(
        manager_thread_id,
        Some("Manager".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        worker_thread_id,
        Some("Worker".to_string()),
        /*agent_role*/ None,
        /*is_closed*/ false,
    );
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);
    let started = chrono::DateTime::parse_from_rfc3339("2026-07-10T20:00:00Z")
        .expect("timestamp")
        .with_timezone(&chrono::Utc);
    app.orchestrate_now_override = Some(started);
    app.handle_orchestrate_command(format!(
        "attach {worker_node} restart-loop --mode review --holder {manager_node} --for 8h --cooldown 60s"
    ));
    app.note_whip_holder_dispatched(&manager_node, &worker_node);
    while app_event_rx.try_recv().is_ok() {}

    let restarted = started + chrono::Duration::seconds(10);
    app.orchestrate_now_override = Some(restarted);
    let assignment = app
        .orchestrate_whips
        .get_mut("assignment-1")
        .expect("assignment");
    assignment.last_fire_utc = Some(restarted);
    assignment.last_idle_generation_fired = Some(0);
    assignment.last_target_output = Some("saved Worker output".to_string());
    app.audit_restored_assignments();

    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|assignment| assignment.state),
        Some(crate::orchestrate::WhipState::Armed)
    );
    let mut restart_notice = String::new();
    while let Ok(event) = app_event_rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            restart_notice.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 120)));
        }
    }
    assert!(
        restart_notice.contains("restored; the next Manager mandate waits one cadence"),
        "{restart_notice}"
    );

    app.orchestrate_now_override = Some(restarted + chrono::Duration::seconds(59));
    app.sweep_orchestrate_whips();
    assert!(drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id).is_none());
    app.orchestrate_now_override = Some(restarted + chrono::Duration::seconds(61));
    app.sweep_orchestrate_whips();
    assert!(drain_spawn_agent_task_for(&mut app_event_rx, manager_thread_id).is_some());
}

#[tokio::test]
async fn assignment_rejects_codex_main_as_manager() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    app.primary_thread_id = Some(ThreadId::new());
    write_test_whip(&app, "main-manager", "Keep managing.");
    let worker_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            /*spawn_role*/ None,
            Some("Worker".to_string()),
        )
        .expect("create worker");

    app.handle_orchestrate_command(format!(
        "attach {worker_pane_id} main-manager --mode review --holder codex-main --for 1h"
    ));

    assert!(app.orchestrate_whips.is_empty());
}

#[tokio::test]
async fn orchestrate_review_holder_ignored_twice_pauses_whip() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "review-loop", "# whip: review-loop\nReview target.");
    let holder_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create holder pane");
    let target_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create target pane");
    let holder_node_id = crate::spawn_orchestration::pane_node_id(&holder_pane_id);

    app.handle_orchestrate_command(format!(
        "attach {target_pane_id} review-loop --mode review --holder {holder_pane_id} --max 5"
    ));
    app.orchestrate_whips
        .get_mut("assignment-1")
        .expect("legacy review whip")
        .kind = crate::orchestrate::WhipKind::LegacyNudge;
    let _ = drain_claude_pane_task_events(&mut app_event_rx);
    app.handle_orchestrate_command("fire assignment-1".to_string());
    app.note_whip_target_idle_with_fire_control(
        &holder_node_id,
        Some("no dispatch"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );
    app.handle_orchestrate_command("fire assignment-1".to_string());
    app.note_whip_target_idle_with_fire_control(
        &holder_node_id,
        Some("still no dispatch"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(
        submitted_tasks
            .iter()
            .filter(|(pane_id, _)| pane_id == &holder_pane_id)
            .count(),
        2
    );
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused)
    );
}

#[tokio::test]
async fn orchestrate_detach_removes_whip_and_idle_generation() {
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
    app.handle_orchestrate_command("detach whip-1".to_string());
    app.sweep_orchestrate_whips();

    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());
    assert!(app.orchestrate_whips.is_empty());
    assert!(app.orchestrate_idle_generation_by_target.is_empty());
}

#[tokio::test]
async fn assignment_worker_completion_wakes_manager_without_waiting_for_watchdog() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "completion-handoff", "Keep managing the Worker.");
    let manager_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Manager".to_string()),
        )
        .expect("create manager");
    let worker_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Worker".to_string()),
        )
        .expect("create worker");
    let manager_node = crate::spawn_orchestration::pane_node_id(&manager_pane_id);
    let worker_node = crate::spawn_orchestration::pane_node_id(&worker_pane_id);
    let started = chrono::DateTime::parse_from_rfc3339("2026-07-10T20:00:00Z")
        .expect("timestamp")
        .with_timezone(&chrono::Utc);
    app.orchestrate_now_override = Some(started);
    app.handle_orchestrate_command(format!(
        "attach {worker_pane_id} completion-handoff --mode review --holder {manager_pane_id} --for 8h --cooldown 900s"
    ));
    app.note_whip_holder_dispatched(&manager_node, &worker_node);
    while app_event_rx.try_recv().is_ok() {}
    app.orchestrate_whips
        .get_mut("assignment-1")
        .expect("assignment")
        .last_fire_utc = Some(started);
    app.orchestrate_now_override = Some(started + chrono::Duration::seconds(3));

    app.note_whip_target_idle_with_fire_control(
        &worker_node,
        Some("first completed result"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );

    let immediate = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(immediate.len(), 1);
    assert_eq!(immediate[0].0, manager_pane_id);
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .and_then(|whip| whip.last_idle_generation_fired),
        Some(1)
    );

    app.claude_panes
        .panes
        .iter_mut()
        .find(|pane| pane.id == manager_pane_id)
        .expect("manager pane")
        .status = crate::claude_panes::ClaudePaneStatus::Running;
    app.note_whip_target_idle_with_fire_control(
        &worker_node,
        Some("second completed result"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );
    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());

    app.claude_panes
        .panes
        .iter_mut()
        .find(|pane| pane.id == manager_pane_id)
        .expect("manager pane")
        .status = crate::claude_panes::ClaudePaneStatus::Idle;
    app.orchestrate_now_override = Some(started + chrono::Duration::seconds(6));
    app.note_whip_target_idle_with_fire_control(
        &manager_node,
        Some("manager finished its prior audit"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );

    let pending = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].0, manager_pane_id);
    assert_eq!(
        app.orchestrate_whips
            .get("assignment-1")
            .and_then(|whip| whip.last_idle_generation_fired),
        Some(2)
    );

    app.orchestrate_now_override = Some(started + chrono::Duration::seconds(9));
    app.note_whip_target_idle_with_fire_control(
        &worker_node,
        /*last_output*/ None,
        /*allow_fire*/ true,
        /*turn_succeeded*/ false,
    );

    let interrupted = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(interrupted.len(), 1);
    assert_eq!(interrupted[0].0, manager_pane_id);
    let assignment = app
        .orchestrate_whips
        .get("assignment-1")
        .expect("assignment");
    assert_eq!(assignment.last_idle_generation_fired, Some(3));
    assert_eq!(
        assignment.last_target_output.as_deref(),
        Some("Worker turn ended unsuccessfully without visible output.")
    );
}

#[tokio::test]
async fn orchestrate_restored_whip_waits_for_fresh_idle_edge() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "resume-safe",
        "# whip: resume-safe\nContinue the work.",
    );
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
    let target_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);

    app.handle_orchestrate_command(format!(
        "attach {pane_id} resume-safe --mode auto --holder none --max 3 --cooldown 1s"
    ));
    app.orchestrate_whips
        .get_mut("whip-1")
        .expect("whip")
        .last_idle_generation_fired = Some(0);

    app.sweep_orchestrate_whips();
    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());

    app.note_whip_target_idle_with_fire_control(
        &target_node_id,
        Some("completed"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );
    assert_eq!(drain_claude_pane_task_events(&mut app_event_rx).len(), 1);
}

#[tokio::test]
async fn orchestrate_rejects_pathy_whip_names() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;

    app.handle_orchestrate_command(
        "attach x ../../etc/hosts --mode auto --holder none".to_string(),
    );

    assert!(app.orchestrate_whips.is_empty());
    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());
}

#[tokio::test]
async fn orchestrate_fire_suppression_still_counts_ignored_review() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "review-loop", "# whip: review-loop\nReview target.");
    let holder_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create holder pane");
    let target_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create target pane");
    let holder_node_id = crate::spawn_orchestration::pane_node_id(&holder_pane_id);

    app.handle_orchestrate_command(format!(
        "attach {target_pane_id} review-loop --mode review --holder {holder_pane_id} --max 5"
    ));
    app.orchestrate_whips
        .get_mut("assignment-1")
        .expect("legacy review whip")
        .kind = crate::orchestrate::WhipKind::LegacyNudge;
    let _ = drain_claude_pane_task_events(&mut app_event_rx);
    app.handle_orchestrate_command("fire assignment-1".to_string());
    app.note_whip_target_idle_with_fire_control(
        &holder_node_id,
        Some("no dispatch"),
        /*allow_fire*/ false,
        /*turn_succeeded*/ true,
    );

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(
        submitted_tasks
            .iter()
            .filter(|(pane_id, _)| pane_id == &holder_pane_id)
            .count(),
        1
    );
    let whip = app.orchestrate_whips.get("assignment-1").expect("whip");
    assert_eq!(whip.ignored_review_fires, 1);
    assert_eq!(whip.pending_review_fire, None);
    assert_eq!(whip.state, crate::orchestrate::WhipState::Armed);
}

#[tokio::test]
async fn orchestrate_agent_block_cannot_replace_user_held_whip() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "first", "# whip: first\nFirst mandate.");
    write_test_whip(&app, "second", "# whip: second\nSecond mandate.");
    let holder_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create holder pane");
    let agent_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Gorbag".to_string()),
        )
        .expect("create agent pane");
    let target_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create target pane");

    app.handle_orchestrate_command(format!(
        "attach {target_pane_id} first --mode review --holder {holder_pane_id} --max 5"
    ));
    let agent_node_id = crate::spawn_orchestration::pane_node_id(&agent_pane_id);
    app.dispatch_orchestrate_blocks_from_text(
        &agent_node_id,
        &format!(
            "```pfterminal-orchestrate\naction: attach\ntarget: {target_pane_id}\nwhip: second\nmode: auto\n```"
        ),
    );

    let active_whips: Vec<_> = app
        .orchestrate_whips
        .values()
        .filter(|whip| whip.state != crate::orchestrate::WhipState::Detached)
        .collect();
    assert_eq!(active_whips.len(), 1);
    assert_eq!(active_whips[0].instructions, "first");
    assert_eq!(
        active_whips[0].holder.as_deref(),
        Some(crate::spawn_orchestration::pane_node_id(&holder_pane_id).as_str())
    );
}

#[tokio::test]
async fn orchestrate_agent_block_cannot_replace_user_holderless_whip() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "first", "# whip: first\nFirst mandate.");
    write_test_whip(&app, "second", "# whip: second\nSecond mandate.");
    let agent_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Gorbag".to_string()),
        )
        .expect("create agent pane");
    let target_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create target pane");

    app.handle_orchestrate_command(format!(
        "attach {target_pane_id} first --mode auto --holder none --max 5"
    ));
    let agent_node_id = crate::spawn_orchestration::pane_node_id(&agent_pane_id);
    app.dispatch_orchestrate_blocks_from_text(
        &agent_node_id,
        &format!(
            "```pfterminal-orchestrate\naction: attach\ntarget: {target_pane_id}\nwhip: second\nmode: auto\n```"
        ),
    );

    let active_whips: Vec<_> = app
        .orchestrate_whips
        .values()
        .filter(|whip| whip.state != crate::orchestrate::WhipState::Detached)
        .collect();
    assert_eq!(active_whips.len(), 1);
    assert_eq!(active_whips[0].instructions, "first");
    assert_eq!(active_whips[0].holder, None);
}

#[tokio::test]
async fn orchestrate_agent_unlimited_attach_is_rejected() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "forever", "# whip: forever\nDo not run forever.");
    let agent_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create agent pane");
    let target_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create target pane");
    let agent_node_id = crate::spawn_orchestration::pane_node_id(&agent_pane_id);

    app.dispatch_orchestrate_blocks_from_text(
        &agent_node_id,
        &format!(
            "```pfterminal-orchestrate\naction: attach\ntarget: {target_pane_id}\nwhip: forever\nfor: unlimited\n```"
        ),
    );

    assert!(app.orchestrate_whips.is_empty());
}

#[tokio::test]
async fn orchestrate_agent_can_pause_own_whip() {
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(&app, "review-loop", "# whip: review-loop\nReview target.");
    let agent_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Troll),
            Some("Burzum".to_string()),
        )
        .expect("create agent pane");
    let target_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Krimp".to_string()),
        )
        .expect("create target pane");
    let agent_node_id = crate::spawn_orchestration::pane_node_id(&agent_pane_id);

    app.dispatch_orchestrate_blocks_from_text(
        &agent_node_id,
        &format!(
            "```pfterminal-orchestrate\naction: attach\ntarget: {target_pane_id}\nwhip: review-loop\nmode: review\n```"
        ),
    );
    app.dispatch_orchestrate_blocks_from_text(
        &agent_node_id,
        "```pfterminal-orchestrate\naction: pause\nid: assignment-1\n```",
    );

    let whip = app.orchestrate_whips.get("assignment-1").expect("whip");
    assert_eq!(whip.holder.as_deref(), Some(agent_node_id.as_str()));
    assert_eq!(whip.state, crate::orchestrate::WhipState::Paused);
}

#[tokio::test]
async fn claude_adapter_emits_direct_records_without_prose_batching() {
    let (mut app, mut rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    app.dispatch_spawn_task_blocks(
        &troll_pane_id,
        vec![
            crate::spawn_orchestration::SpawnTaskDispatch {
                target: orc_pane_id.clone(),
                task: "first exact task".to_string(),
                seq: Some(601),
            },
            crate::spawn_orchestration::SpawnTaskDispatch {
                target: orc_pane_id,
                task: "second exact task".to_string(),
                seq: Some(602),
            },
        ],
    );
    let tasks = drain_claude_pane_task_events(&mut rx);
    assert_eq!(tasks.len(), 2);
    assert!(tasks[0].1.contains("first exact task"));
    assert!(!tasks[0].1.contains("second exact task"));
    assert!(tasks[1].1.contains("second exact task"));
    assert!(!tasks[1].1.contains("Multiple spawn dispatches"));
}

/// #507: turning the kill switch on interrupts every running Claude pane
/// turn (its cancel token ends the Claude process), and the pane says why.
#[tokio::test]
async fn issue_507_kill_switch_interrupts_running_claude_panes() {
    let (mut app, _app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let codex_home = app.config.codex_home.to_path_buf();
    let running = app
        .claude_panes
        .prepare_turn(&troll_pane_id, "long task".to_string(), &codex_home)
        .expect("prepare running Troll pane");
    assert!(app.claude_panes.claude_pane_is_running(&troll_pane_id));
    assert!(!app.claude_panes.claude_pane_is_running(&orc_pane_id));

    app.stop_claude_panes_for_kill_switch();

    assert!(running.cancel_token.is_cancelled());
    let said = |pane_id: &str| {
        app.claude_pane_transcript_cells
            .get(pane_id)
            .into_iter()
            .flatten()
            .any(|cell| {
                lines_to_single_string(&cell.display_lines(/*width*/ 400))
                    .contains("security kill switch was turned on")
            })
    };
    assert!(said(&troll_pane_id));
    assert!(!said(&orc_pane_id), "an idle pane has nothing to stop");
}

/// #507 (Travis, 2026-10-12): a kill switch another Corbanu Terminal process
/// on this home turns on stops this app's running Claude panes too and says
/// where it came from; the same change reported again (by `/security` here
/// or the watch) is handled once; turning it off there is announced and
/// starts nothing here.
#[tokio::test]
async fn issue_507_kill_switch_from_another_process_stops_claude_panes_once() {
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let codex_home = app.config.codex_home.to_path_buf();
    let rendered = |rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>| {
        let mut text = String::new();
        while let Ok(event) = rx.try_recv() {
            if let AppEvent::InsertHistoryCell(cell) = event {
                text.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 400)));
            }
        }
        text
    };
    let _ = rendered(&mut app_event_rx);
    let running = app
        .claude_panes
        .prepare_turn(&troll_pane_id, "long task".to_string(), &codex_home)
        .expect("prepare running Troll pane");
    assert!(!app.kill_switch_known_on);

    app.on_kill_switch_changed(/*on*/ true, /*elsewhere*/ true);
    assert!(running.cancel_token.is_cancelled());
    assert!(app.kill_switch_known_on);
    let text = rendered(&mut app_event_rx);
    assert!(
        text.contains("turned on in another Corbanu Terminal on this home"),
        "{text}"
    );

    // Reported again: nothing more is stopped or said.
    let later = app
        .claude_panes
        .prepare_turn(&orc_pane_id, "later".to_string(), &codex_home)
        .expect("prepare a later turn");
    app.on_kill_switch_changed(/*on*/ true, /*elsewhere*/ false);
    app.on_kill_switch_changed(/*on*/ true, /*elsewhere*/ true);
    assert!(!later.cancel_token.is_cancelled());
    assert!(!rendered(&mut app_event_rx).contains("another Corbanu Terminal"));

    // Turned off elsewhere: nothing starts here by itself (Travis,
    // 2026-10-12); a task held for a Claude pane stays held.
    app.kill_switch_held_claude_tasks
        .push((troll_pane_id.clone(), "held task".to_string()));
    app.on_kill_switch_changed(/*on*/ false, /*elsewhere*/ true);
    assert!(!app.kill_switch_known_on);
    let text = rendered(&mut app_event_rx);
    assert!(
        text.contains("turned off in another Corbanu Terminal on this home"),
        "{text}"
    );
    assert!(text.contains("Nothing restarts by itself here"), "{text}");
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(troll_pane_id, "held task".to_string())]
    );
    // It goes with that pane's next turn (#602's delivery, tested there).
}

/// #842 (Travis, 2026-10-15): Moderate confirmed in another Corbanu Terminal
/// on this home (Core already raised this process's sessions) is announced
/// here, saying what applies now and what waits for the restart.
#[tokio::test]
async fn issue_842_moderate_from_another_terminal_is_announced() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    while app_event_rx.try_recv().is_ok() {}
    app.on_level_raised_elsewhere(crate::legacy_core::security_revocation::SecurityLevel::Moderate);
    let mut text = String::new();
    while let Ok(event) = app_event_rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            text.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 400)));
        }
    }
    for expected in [
        "Moderate was confirmed in another Corbanu Terminal on this home",
        "Core's Moderate rules apply here now",
        "shown as Permissive until you restart",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}

/// #919 (Travis, 2026-10-16, D20): Aggressive confirmed in another Corbanu
/// Terminal on this home (Core already raised this process's sessions and
/// required approval for every command) is announced here, saying what
/// applies now and what waits for the restart; the protections claimed
/// follow `source_envelopes`, and the level shown is this session's own.
#[tokio::test]
async fn issue_919_aggressive_from_another_terminal_is_announced() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    while app_event_rx.try_recv().is_ok() {}
    app.on_level_raised_elsewhere(
        crate::legacy_core::security_revocation::SecurityLevel::Aggressive,
    );
    let mut text = String::new();
    while let Ok(event) = app_event_rx.try_recv() {
        if let AppEvent::InsertHistoryCell(cell) = event {
            text.push_str(&lines_to_single_string(&cell.display_lines(/*width*/ 1000)));
        }
    }
    for expected in [
        "Aggressive was confirmed in another Corbanu Terminal on this home",
        "Core's Aggressive protections apply here now: agent commands cannot run `corbanu vault …`",
        "From each pane's next turn you approve every command that is not known read-only",
        "Grants and \"for session\" approvals here ended",
        "The rest of Aggressive starts when you restart Corbanu Terminal: its sandbox",
        "this session is shown as Permissive",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    // A lower level is never announced as a raise.
    app.on_level_raised_elsewhere(
        crate::legacy_core::security_revocation::SecurityLevel::Permissive,
    );
    assert!(app_event_rx.try_recv().is_err());
}

/// #919: the notice claims the protected-path denials only with the
/// untrusted-content labels on, and names the level this session is shown
/// as until the restart.
#[test]
fn issue_919_aggressive_notice_follows_labels_and_the_shown_level() {
    use crate::claude_panes::app_integration::aggressive_confirmed_elsewhere;
    use crate::security::level::ChosenLevel;
    let labelled =
        aggressive_confirmed_elsewhere(/*untrusted_content*/ true, ChosenLevel::Moderate);
    assert!(labelled.contains("protected paths"), "{labelled}");
    assert!(labelled.contains("shown as Moderate"), "{labelled}");
    // Moderate's credential broker already holds the model keys.
    assert!(!labelled.contains("credential broker"), "{labelled}");
    let vault_only =
        aggressive_confirmed_elsewhere(/*untrusted_content*/ false, ChosenLevel::Permissive);
    assert!(!vault_only.contains("protected paths"), "{vault_only}");
    assert!(
        vault_only.contains("model keys held by the credential broker, and the child-agent rules"),
        "{vault_only}"
    );
    assert!(
        vault_only.contains("in every Corbanu pane and child agent of this Corbanu Terminal"),
        "{vault_only}"
    );
}

fn interrupted_claude_turn(app: &App) -> crate::claude_panes::ClaudePaneTurnOutput {
    crate::claude_panes::ClaudePaneTurnOutput {
        text: String::new(),
        status: crate::claude_panes::ClaudePaneTurnStatus::Interrupted,
        session_id: None,
        usage_summary: None,
        turn_usage_summary: None,
        usage_status: crate::claude_panes::ClaudePaneUsageStatus::Missing,
        artifact_path: app.config.cwd.join("turn-0001.jsonl").to_path_buf(),
        audit_path: app.config.cwd.join("turn-0001.audit.json").to_path_buf(),
        duration_ms: 1,
        terminal_reason: Some("interrupted".to_string()),
        error_summary: Some("Claude pane turn interrupted by user.".to_string()),
        tool_names: Vec::new(),
        tool_events: Vec::new(),
        reasoning_events: Vec::new(),
        command_mode: crate::claude_panes::ClaudeCommandMode::NewSession,
        direct_accounting: None,
    }
}

/// #554: a Claude turn the kill switch stopped ends with the line Codex
/// panes show, not "interrupted by user"; a person's own interrupt keeps it.
#[tokio::test]
async fn issue_554_kill_switch_stop_does_not_say_interrupted_by_user() {
    let (mut app, _app_event_rx, troll_pane_id, _orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let codex_home = app.config.codex_home.to_path_buf();
    let transcript = |app: &App, pane_id: &str| {
        app.claude_pane_transcript_cells
            .get(pane_id)
            .into_iter()
            .flatten()
            .map(|cell| lines_to_single_string(&cell.display_lines(/*width*/ 400)))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let running = app
        .claude_panes
        .prepare_turn(&troll_pane_id, "long task".to_string(), &codex_home)
        .expect("prepare running Troll pane");
    app.stop_claude_panes_for_kill_switch();
    drop(running);
    let output = interrupted_claude_turn(&app);
    app.on_claude_pane_turn_finished(troll_pane_id.clone(), Ok(output));
    let said = transcript(&app, &troll_pane_id);
    assert!(
        said.contains("security kill switch was turned on"),
        "{said}"
    );
    assert!(
        said.contains(crate::chatwidget::KILL_SWITCH_TURN_STOPPED),
        "{said}"
    );
    assert!(!said.contains("interrupted by user"), "{said}");

    // The next turn, interrupted by the person, says so as before.
    let running = app
        .claude_panes
        .prepare_turn(&troll_pane_id, "another task".to_string(), &codex_home)
        .expect("prepare the next turn");
    app.claude_panes
        .interrupt_turn(&troll_pane_id)
        .expect("interrupt");
    drop(running);
    let output = interrupted_claude_turn(&app);
    app.on_claude_pane_turn_finished(troll_pane_id.clone(), Ok(output));
    let said = transcript(&app, &troll_pane_id);
    assert!(said.contains("interrupted by user"), "{said}");
}

/// #507: the person confirms `choice` in `/security`.
pub(super) fn issue_507_switch(
    app: &App,
    choice: crate::legacy_core::security_revocation::HumanRevocation,
) {
    use crate::legacy_core::security_level_change::LevelBasis;
    use crate::legacy_core::security_level_change::SecurityLevel;
    use crate::legacy_core::security_revocation::commit_human_revocation;
    let codex_home = app.config.codex_home.to_path_buf();
    let basis = LevelBasis::read(&codex_home, SecurityLevel::Permissive, /*thread*/ None);
    commit_human_revocation(
        &codex_home,
        SecurityLevel::Permissive,
        /*thread*/ None,
        &basis,
        choice,
        /*now_unix_seconds*/ 1_000,
    )
    .expect("commit");
}

fn issue_507_transcript(app: &App, pane_id: &str) -> String {
    app.claude_pane_transcript_cells
        .get(pane_id)
        .into_iter()
        .flatten()
        .map(|cell| lines_to_single_string(&cell.display_lines(/*width*/ 400)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// #507 (owner decision 2026-10-12): tasks other agents send to Claude panes
/// while the kill switch is on wait (one per pane, the newest; the older one
/// is recorded as superseded). Turning the switch off starts nothing, and a
/// pane's turn ending does not send its waiting task either.
#[tokio::test]
async fn issue_507_switch_off_starts_no_claude_pane_turn() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, _app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let codex_home = app.config.codex_home.to_path_buf();
    let held = |app: &App| app.kill_switch_held_claude_tasks.clone();
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);

    for tick in 1..=3 {
        app.submit_claude_pane_task(troll_pane_id.clone(), format!("assignment fire {tick}"));
    }
    app.submit_claude_pane_task(orc_pane_id.clone(), "orc task".to_string());
    let waiting = vec![
        (troll_pane_id.clone(), "assignment fire 3".to_string()),
        (orc_pane_id.clone(), "orc task".to_string()),
    ];
    assert_eq!(held(&app), waiting);
    assert!(app.claude_panes.prepared_prompts.is_empty());

    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.on_kill_switch_turned_off();
    assert!(!app.claude_panes.claude_pane_is_running(&troll_pane_id));
    assert!(!app.claude_panes.claude_pane_is_running(&orc_pane_id));
    assert!(
        app.claude_panes.prepared_prompts.is_empty(),
        "turning the switch off starts no turn"
    );
    assert_eq!(held(&app), waiting, "the tasks keep waiting");

    // A turn that ends in the pane (one its operator started) does not send
    // the waiting task either.
    drop(
        app.claude_panes
            .prepare_turn(&troll_pane_id, "busy".to_string(), &codex_home)
            .expect("a running turn"),
    );
    app.claude_panes
        .interrupt_turn(&troll_pane_id)
        .expect("stop the busy turn");
    app.on_claude_pane_turn_finished(troll_pane_id.clone(), Err("interrupted".to_string()));
    assert!(!app.claude_panes.claude_pane_is_running(&troll_pane_id));
    assert_eq!(
        app.claude_panes.prepared_prompts.len(),
        1,
        "only the busy turn"
    );
    assert_eq!(held(&app), waiting);
}

/// #507: after the switch is off, a Claude pane's waiting task goes with its
/// next turn: the one a person types (the waiting task first), or one new work
/// starts. Either way it is sent once, and the pane says so.
#[tokio::test]
async fn issue_507_waiting_claude_task_goes_with_the_next_turn() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, _app_event_rx, _op_rx) = make_test_app_with_channels().await;
    let codex_home = app.config.codex_home.to_path_buf();
    let typed_pane = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            &codex_home,
            /*spawn_role*/ None,
            /*spawn_nickname*/ None,
        )
        .expect("create user pane");
    let worker_pane = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            &codex_home,
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Snaga".to_string()),
        )
        .expect("create worker pane");
    assert_eq!(
        app.claude_panes.active_claude_pane_id(),
        Some(typed_pane.as_str())
    );
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.submit_claude_pane_task(typed_pane.clone(), "pf507 waiting task A".to_string());
    app.submit_claude_pane_task(worker_pane.clone(), "pf507 waiting task B".to_string());
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.on_kill_switch_turned_off();
    assert!(app.claude_panes.prepared_prompts.is_empty());

    // A person types in the active pane.
    assert!(app.try_submit_active_claude_pane_op(&claude_pane_text_op("pf507 typed message")));
    let prompts = app.claude_panes.prepared_prompts.clone();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].0, typed_pane);
    let prompt = &prompts[0].1;
    let task = prompt
        .find("pf507 waiting task A")
        .expect("waiting task sent");
    let typed = prompt.find("pf507 typed message").expect("typed text sent");
    assert!(task < typed, "the waiting task arrived first: {prompt}");
    assert!(!prompt.contains("pf507 waiting task B"));

    // New work for the other pane: its waiting task goes with it.
    app.submit_claude_pane_task(worker_pane.clone(), "pf507 new task".to_string());
    let prompts = app.claude_panes.prepared_prompts.clone();
    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[1].0, worker_pane);
    assert!(prompts[1].1.contains("pf507 waiting task B"));
    assert!(prompts[1].1.contains("pf507 new task"));
    assert!(
        issue_507_transcript(&app, &worker_pane).contains("Sent with this turn"),
        "{}",
        issue_507_transcript(&app, &worker_pane)
    );
    assert!(app.kill_switch_held_claude_tasks.is_empty(), "sent once");
}

/// #507: a waiting task stays waiting when the next turn cannot start (the
/// pane is busy), and goes with the first turn that does.
#[tokio::test]
async fn issue_507_waiting_claude_task_survives_a_refused_turn() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, _app_event_rx, _troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let codex_home = app.config.codex_home.to_path_buf();
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf507 superseded".to_string());
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf507 waiting".to_string());
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(orc_pane_id.clone(), "pf507 waiting".to_string())],
        "the newest waits; the older one was reported superseded"
    );
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    let busy = app
        .claude_panes
        .prepare_turn(&orc_pane_id, "busy".to_string(), &codex_home)
        .expect("a running turn");
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf507 refused".to_string());
    assert_eq!(
        app.claude_panes.prepared_prompts.len(),
        1,
        "only the busy turn"
    );
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(orc_pane_id.clone(), "pf507 waiting".to_string())]
    );
    drop(busy);
    app.claude_panes
        .interrupt_turn(&orc_pane_id)
        .expect("stop the busy turn");
    app.on_claude_pane_turn_finished(orc_pane_id.clone(), Err("interrupted".to_string()));
    app.submit_claude_pane_task(orc_pane_id, "pf507 next".to_string());
    assert!(app.kill_switch_held_claude_tasks.is_empty());
    let prompt = &app.claude_panes.prepared_prompts[1].1;
    assert!(prompt.contains("pf507 waiting"), "{prompt}");
    assert!(!prompt.contains("pf507 superseded"), "{prompt}");
}

/// #507: assignments and whips (timers) do not fire while the kill switch
/// is on; they are paused, and turning it off does not resume them. A person
/// resumes them.
#[tokio::test]
async fn issue_507_whips_pause_and_stay_paused_after_switch_off() {
    use crate::legacy_core::security_revocation::HumanRevocation;
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
    app.note_whip_target_idle_with_fire_control(
        &crate::spawn_orchestration::pane_node_id(&pane_id),
        Some("done"),
        /*allow_fire*/ true,
        /*turn_succeeded*/ true,
    );
    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());
    assert_eq!(
        app.orchestrate_whips.get("whip-1").map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused)
    );

    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    app.sweep_orchestrate_whips();
    assert!(
        drain_claude_pane_task_events(&mut app_event_rx).is_empty(),
        "switch-off resumes no whip"
    );

    app.handle_orchestrate_command("resume whip-1".to_string());
    app.sweep_orchestrate_whips();
    let fired = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(fired.len(), 1, "a person resumed it");
    assert!(fired[0].1.contains("Continue the work."));
}

/// #507 review: the switch events themselves (as `/security` sends them):
/// turning it on pauses armed whips at once, with no sweep in between, so the
/// first sweep after it is turned off fires nothing; turning it off starts
/// no turn, task, op or agent message, and the Claude pane's task keeps
/// waiting.
#[tokio::test]
async fn issue_507_switch_events_pause_whips_at_once_and_start_nothing() -> Result<()> {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, mut app_event_rx, mut op_rx) = make_test_app_with_channels().await;
    let config = app.chat_widget.config_ref().clone();
    let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(&config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
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
    app.submit_claude_pane_task(pane_id.clone(), "pf507 waiting".to_string());
    let control =
        Box::pin(app.handle_event(&mut tui, &mut app_server, AppEvent::KillSwitchTurnedOn)).await?;
    assert!(matches!(control, AppRunControl::Continue));
    assert_eq!(
        app.orchestrate_whips.get("whip-1").map(|whip| whip.state),
        Some(crate::orchestrate::WhipState::Paused),
        "paused when the switch went on, not at the next sweep"
    );

    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    Box::pin(app.handle_event(&mut tui, &mut app_server, AppEvent::KillSwitchTurnedOff)).await?;
    app.sweep_orchestrate_whips();
    while let Ok(event) = app_event_rx.try_recv() {
        assert!(
            !matches!(
                event,
                AppEvent::CodexOp(_)
                    | AppEvent::SubmitSpawnAgentTask { .. }
                    | AppEvent::SubmitSpawnClaudePaneTask { .. }
                    | AppEvent::SendSpawnAgentMailboxMessage { .. }
            ),
            "switch-off sent work"
        );
    }
    assert!(op_rx.try_recv().is_err(), "switch-off submitted an op");
    assert!(app.claude_panes.prepared_prompts.is_empty());
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(pane_id, "pf507 waiting".to_string())]
    );
    Ok(())
}

/// #507 review: a Claude turn the kill switch stopped may only finish after
/// the switch is off; its report reaches the parent but starts no turn there.
#[tokio::test]
async fn issue_507_report_of_a_stopped_claude_turn_starts_no_parent_turn() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let codex_home = app.config.codex_home.to_path_buf();
    let running = app
        .claude_panes
        .prepare_turn(&orc_pane_id, "long task".to_string(), &codex_home)
        .expect("a running Orc turn");
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.stop_claude_panes_for_kill_switch();
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    drop(running);
    let output = interrupted_claude_turn(&app);
    app.on_claude_pane_turn_finished(orc_pane_id, Ok(output));

    assert!(
        drain_claude_pane_task_events(&mut app_event_rx).is_empty(),
        "no auto-processing turn for the parent"
    );
    assert!(!app.claude_panes.claude_pane_is_running(&troll_pane_id));
    let troll_node = crate::spawn_orchestration::pane_node_id(&troll_pane_id);
    assert_eq!(
        app.spawn_parent_reports_by_node
            .get(&troll_node)
            .map(std::collections::VecDeque::len),
        Some(1),
        "the parent still has the report"
    );
}

/// #507 review: a removed Claude pane's waiting task is dropped (its sender
/// is told), and the switch-off notice does not name it.
#[tokio::test]
async fn issue_507_removed_pane_drops_its_waiting_task() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, _app_event_rx, _troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.submit_claude_pane_task(orc_pane_id.clone(), "pf507 waiting".to_string());
    app.claude_panes
        .remove_managed_crew_pane(&orc_pane_id, app.config.codex_home.as_ref())
        .expect("remove the pane");
    app.drop_held_claude_task_of_removed_pane(&orc_pane_id);
    assert!(app.kill_switch_held_claude_tasks.is_empty());
}

/// #507 review: while the switch is on, a child report starts no turn in a
/// Claude parent either (it would only wait, and push out the parent's real
/// waiting task); the report is kept for the parent's next turn.
#[tokio::test]
async fn issue_507_child_report_while_on_starts_no_parent_turn() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.submit_claude_pane_task(troll_pane_id.clone(), "pf507 real task".to_string());
    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));
    assert!(drain_claude_pane_task_events(&mut app_event_rx).is_empty());
    assert_eq!(
        app.kill_switch_held_claude_tasks,
        vec![(troll_pane_id.clone(), "pf507 real task".to_string())],
        "the real task still waits"
    );
    let troll_node = crate::spawn_orchestration::pane_node_id(&troll_pane_id);
    assert_eq!(
        app.spawn_parent_reports_by_node
            .get(&troll_node)
            .map(std::collections::VecDeque::len),
        Some(1)
    );
}

/// #507 review: the report of a stopped Claude turn reaches a native parent
/// as mail that does not ask for a turn; an ordinary report still does.
#[tokio::test]
async fn issue_507_stopped_claude_report_to_a_native_parent_asks_for_no_turn() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, mut app_event_rx, _troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let parent_thread =
        ThreadId::from_string("00000000-0000-0000-0000-000000000508").expect("thread id");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&orc_pane_id),
        crate::spawn_orchestration::thread_node_id(parent_thread),
    );
    let codex_home = app.config.codex_home.to_path_buf();
    let mail = |rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>| {
        std::iter::from_fn(|| rx.try_recv().ok())
            .filter_map(|event| match event {
                AppEvent::SendSpawnAgentMailboxMessage { params } => Some(params.trigger_turn),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    let running = app
        .claude_panes
        .prepare_turn(&orc_pane_id, "long task".to_string(), &codex_home)
        .expect("a running Orc turn");
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.stop_claude_panes_for_kill_switch();
    issue_507_switch(&app, HumanRevocation::KillSwitchOff);
    drop(running);
    let output = interrupted_claude_turn(&app);
    app.on_claude_pane_turn_finished(orc_pane_id.clone(), Ok(output));
    assert_eq!(mail(&mut app_event_rx), vec![false]);

    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));
    assert_eq!(
        mail(&mut app_event_rx),
        vec![true],
        "control: a normal report"
    );
}

/// #507 review: a task for a Claude pane that does not exist is refused, not
/// held, while the switch is on.
#[tokio::test]
async fn issue_507_no_waiting_task_for_a_missing_claude_pane() {
    use crate::legacy_core::security_revocation::HumanRevocation;
    let (mut app, _app_event_rx, _troll_pane_id, _orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    issue_507_switch(&app, HumanRevocation::KillSwitchOn);
    app.submit_claude_pane_task("claude-missing".to_string(), "pf507 task".to_string());
    assert!(app.kill_switch_held_claude_tasks.is_empty());
}

#[tokio::test]
async fn child_report_auto_starts_turn_on_idle_claude_pane_parent() {
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;

    assert!(!app.claude_panes.claude_pane_is_running(&troll_pane_id));
    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(submitted_tasks.len(), 1);
    let (pane_id, task) = &submitted_tasks[0];
    assert_eq!(pane_id, &troll_pane_id);
    assert!(task.contains("A child pane has reported back"));
    assert!(task.contains("result text"));
}

#[tokio::test]
async fn child_report_auto_does_not_start_turn_on_running_claude_pane_parent() {
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let _prepared = app
        .claude_panes
        .prepare_turn(
            &troll_pane_id,
            "already running".to_string(),
            app.config.codex_home.as_ref(),
        )
        .expect("prepare running Troll pane");

    assert!(app.claude_panes.claude_pane_is_running(&troll_pane_id));
    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert!(
        submitted_tasks
            .iter()
            .all(|(pane_id, _)| pane_id != &troll_pane_id)
    );
}

#[tokio::test]
async fn child_report_auto_duplicate_child_report_does_not_trigger_duplicate_turns() {
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;

    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));
    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("result text"));

    let submitted_tasks = drain_claude_pane_task_events(&mut app_event_rx);
    assert_eq!(
        submitted_tasks
            .iter()
            .filter(|(pane_id, _)| pane_id == &troll_pane_id)
            .count(),
        1
    );
}

#[tokio::test]
async fn external_pane_auto_dispatch_chain_pauses_and_resumes_on_fresh_input() {
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let troll_node_id = crate::spawn_orchestration::pane_node_id(&troll_pane_id);

    for cycle in 0..codex_protocol::crew::CREW_AUTO_DISPATCH_CHAIN_LIMIT {
        app.record_spawn_child_report_for_claude_pane(
            &orc_pane_id,
            "done",
            Some(&format!("cycle {cycle} result")),
        );
        let auto_tasks = drain_claude_pane_task_events(&mut app_event_rx);
        assert_eq!(
            auto_tasks
                .iter()
                .filter(|(pane_id, _)| pane_id == &troll_pane_id)
                .count(),
            1
        );
        app.note_spawn_turn_started_for_auto_loop(&troll_node_id);
        app.dispatch_spawn_task_blocks(
            &troll_pane_id,
            vec![crate::spawn_orchestration::SpawnTaskDispatch {
                target: orc_pane_id.clone(),
                task: format!("cycle {cycle} follow-up"),
                seq: Some(700 + u64::from(cycle)),
            }],
        );
        let dispatched_tasks = drain_claude_pane_task_events(&mut app_event_rx);
        assert_eq!(
            dispatched_tasks
                .iter()
                .filter(|(pane_id, _)| pane_id == &orc_pane_id)
                .count(),
            1
        );
        app.note_spawn_turn_completed_for_auto_loop(&troll_node_id);
    }

    app.record_spawn_child_report_for_claude_pane(
        &orc_pane_id,
        "done",
        Some("the report after the configured chain limit"),
    );
    assert!(
        drain_claude_pane_task_events(&mut app_event_rx)
            .iter()
            .all(|(pane_id, _)| pane_id != &troll_pane_id),
        "the next report must remain visible without starting another paid parent turn"
    );
    assert_eq!(
        app.spawn_auto_loop_state_by_node[&troll_node_id].chain,
        codex_protocol::crew::CREW_AUTO_DISPATCH_CHAIN_LIMIT
    );

    // A real user task is represented by a non-pending turn start and resumes the same pane.
    app.note_spawn_turn_started_for_auto_loop(&troll_node_id);
    app.note_spawn_turn_completed_for_auto_loop(&troll_node_id);
    app.record_spawn_child_report_for_claude_pane(
        &orc_pane_id,
        "done",
        Some("fresh result after operator input"),
    );
    assert_eq!(
        drain_claude_pane_task_events(&mut app_event_rx)
            .iter()
            .filter(|(pane_id, _)| pane_id == &troll_pane_id)
            .count(),
        1
    );
}

#[tokio::test]
async fn external_pane_auto_acknowledgement_terminates_dispatch_chain() {
    let (mut app, mut app_event_rx, troll_pane_id, orc_pane_id) =
        make_child_report_auto_claude_pane_app().await;
    let troll_node_id = crate::spawn_orchestration::pane_node_id(&troll_pane_id);

    app.record_spawn_child_report_for_claude_pane(&orc_pane_id, "done", Some("first result"));
    drain_claude_pane_task_events(&mut app_event_rx);
    app.note_spawn_turn_started_for_auto_loop(&troll_node_id);
    app.dispatch_spawn_task_blocks(
        &troll_pane_id,
        vec![crate::spawn_orchestration::SpawnTaskDispatch {
            target: orc_pane_id.clone(),
            task: "one follow-up".to_string(),
            seq: Some(801),
        }],
    );
    drain_claude_pane_task_events(&mut app_event_rx);
    app.note_spawn_turn_completed_for_auto_loop(&troll_node_id);
    assert_eq!(app.spawn_auto_loop_state_by_node[&troll_node_id].chain, 1);

    app.record_spawn_child_report_for_claude_pane(
        &orc_pane_id,
        "done",
        Some("acknowledge this result"),
    );
    drain_claude_pane_task_events(&mut app_event_rx);
    app.note_spawn_turn_started_for_auto_loop(&troll_node_id);
    app.note_spawn_turn_completed_for_auto_loop(&troll_node_id);
    assert_eq!(app.spawn_auto_loop_state_by_node[&troll_node_id].chain, 0);
}

#[tokio::test]
async fn native_nazgul_sees_live_troll_and_orc_tree_even_if_spawned_before_them() {
    // The bug: a spawned Nazgul's base_instructions are frozen at spawn time, so if it was created
    // before its Troll/Orcs it would forever answer "none spawned yet". The fix renders the live
    // hierarchy per turn via spawn_context_for_thread. This test proves the live view sees a Troll
    // and two Orcs that exist now, regardless of when the Nazgul was spawned.
    let mut app = make_test_app().await;
    let nazgul_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000601").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000602").expect("valid thread id");
    let orc_a =
        ThreadId::from_string("00000000-0000-0000-0000-000000000603").expect("valid thread id");
    let orc_b =
        ThreadId::from_string("00000000-0000-0000-0000-000000000604").expect("valid thread id");

    app.upsert_agent_picker_thread(
        nazgul_thread_id,
        Some("Euclid".to_string()),
        Some("nazgul".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_a,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_b,
        Some("Ghash".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, nazgul_thread_id);
    app.spawn_parent_by_thread.insert(orc_a, troll_thread_id);
    app.spawn_parent_by_thread.insert(orc_b, troll_thread_id);

    let context = app
        .spawn_context_for_thread(nazgul_thread_id)
        .expect("Nazgul thread should receive live spawn context");
    assert!(context.contains("You are the Corbanu Terminal Nazgul/root pane"));
    assert!(
        context.contains("Euclid [nazgul]"),
        "context uses the Nazgul picker label, got: {context}"
    );
    assert!(
        context.contains("built-in nazgul.toml agent config"),
        "native Nazgul role prompt should come from role config, got: {context}"
    );
    assert!(
        !context.contains("You are not an individual contributor or coder"),
        "native Nazgul context should not duplicate the persistent role prompt, got: {context}"
    );
    assert!(
        context.contains("Burzum [troll]"),
        "Nazgul must see the live Troll, got: {context}"
    );
    assert!(
        context.contains("Snaga [orc]"),
        "Nazgul must see the live Orc Snaga, got: {context}"
    );
    assert!(
        context.contains("Ghash [orc]"),
        "Nazgul must see the live Orc Ghash, got: {context}"
    );
    assert!(
        !context.contains("none spawned yet"),
        "Nazgul must not claim the tree is empty when it is not"
    );
}

#[tokio::test]
async fn native_agent_text_is_never_used_as_dispatch_transport() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let nazgul_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000648").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000649").expect("valid thread id");
    let snaga_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000650").expect("valid thread id");

    app.upsert_agent_picker_thread(
        nazgul_thread_id,
        Some("Angmar".to_string()),
        Some("nazgul".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        snaga_thread_id,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, nazgul_thread_id);
    app.spawn_parent_by_thread
        .insert(snaga_thread_id, troll_thread_id);

    let partial_message = r#"Dispatch started.
<pfterminal_send_task target="Snaga">
Reply with exactly OK."#;

    app.update_spawn_status_for_thread_notification(&item_completed_notification(
        troll_thread_id,
        "turn-partial-item-dispatch",
        "agent-message-partial-item-dispatch",
        partial_message,
    ));
    assert!(
        drain_spawn_agent_task_for(&mut rx, snaga_thread_id).is_none(),
        "partial item blocks must not dispatch"
    );

    app.update_spawn_status_for_thread_notification(&turn_completed_with_agent_message(
        troll_thread_id,
        "turn-partial-item-dispatch",
        TurnStatus::Interrupted,
        partial_message,
    ));
    assert!(
        drain_spawn_agent_task_for(&mut rx, snaga_thread_id).is_none(),
        "interrupted turn catch-all must not dispatch partial blocks"
    );

    let complete_legacy_block = r#"<pfterminal_send_task target="Snaga">
Reply with exactly OK.
</pfterminal_send_task>"#;
    app.update_spawn_status_for_thread_notification(&turn_completed_with_agent_message(
        troll_thread_id,
        "turn-complete-legacy-block",
        TurnStatus::Completed,
        complete_legacy_block,
    ));
    assert!(
        drain_spawn_agent_task_for(&mut rx, snaga_thread_id).is_none(),
        "native agents must dispatch with collaboration tools, never assistant-text tags"
    );
}

#[tokio::test]
async fn interrupted_turn_never_dispatches_even_with_complete_blocks() {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let nazgul_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000651").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000652").expect("valid thread id");

    app.upsert_agent_picker_thread(
        nazgul_thread_id,
        Some("Angmar".to_string()),
        Some("nazgul".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, nazgul_thread_id);

    let message = r#"Dispatching.
<pfterminal_send_task target="Burzum">
Task: um to remove the artifact and stop generating unvalidated busywork.
</pfterminal_send_task>
More text that never finished"#;

    for status in [TurnStatus::Interrupted, TurnStatus::Failed] {
        app.update_spawn_status_for_thread_notification(&turn_completed_with_agent_message(
            nazgul_thread_id,
            "turn-interrupted-dispatch",
            status.clone(),
            message,
        ));
        while let Ok(event) = rx.try_recv() {
            if let AppEvent::SubmitSpawnAgentTask { thread_id, .. } = event
                && thread_id == troll_thread_id
            {
                panic!("{status:?} turn must never dispatch spawn task blocks");
            }
        }
    }
}

#[tokio::test]
async fn manual_troll_spawn_uses_bound_native_nazgul_as_core_parent() {
    let mut app = make_test_app().await;
    let main_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000305").expect("valid main id");
    let nazgul_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000306").expect("valid Nazgul id");
    let nazgul_node_id = crate::spawn_orchestration::thread_node_id(nazgul_thread_id);

    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.upsert_agent_picker_thread(
        nazgul_thread_id,
        Some("Angmar".to_string()),
        Some("nazgul".to_string()),
        /*is_closed*/ false,
    );
    app.set_spawn_nazgul_pane_binding(nazgul_node_id.clone());

    assert_eq!(
        app.backend_parent_thread_for_spawn(
            crate::spawn_orchestration::SpawnRole::Troll,
            Some(&nazgul_node_id),
        ),
        Some(nazgul_thread_id)
    );
    assert_eq!(
        app.logical_parent_node_for_spawn(
            crate::spawn_orchestration::SpawnRole::Troll,
            Some(&nazgul_node_id),
        ),
        nazgul_node_id
    );
}

#[tokio::test]
async fn active_native_nazgul_turn_receives_live_spawn_context_with_orcs() {
    // Direct user input in the active Nazgul pane uses the normal active-thread submit path, not
    // SubmitSpawnAgentTask. It still must receive live hierarchy context; otherwise the Nazgul sees
    // only spawn-time instructions and says no Orcs exist even while /spawn status lists them.
    let mut app = make_test_app().await;
    let nazgul_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000611").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000612").expect("valid thread id");
    let orc_a =
        ThreadId::from_string("00000000-0000-0000-0000-000000000613").expect("valid thread id");
    let orc_b =
        ThreadId::from_string("00000000-0000-0000-0000-000000000614").expect("valid thread id");

    app.active_thread_id = Some(nazgul_thread_id);
    app.upsert_agent_picker_thread(
        nazgul_thread_id,
        Some("Euclid".to_string()),
        Some("nazgul".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_a,
        Some("Snaga".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.upsert_agent_picker_thread(
        orc_b,
        Some("Ghash".to_string()),
        Some("orc".to_string()),
        /*is_closed*/ false,
    );
    app.spawn_parent_by_thread
        .insert(troll_thread_id, nazgul_thread_id);
    app.spawn_parent_by_thread.insert(orc_a, troll_thread_id);
    app.spawn_parent_by_thread.insert(orc_b, troll_thread_id);

    let context_map = app
        .spawn_additional_context_for_thread(nazgul_thread_id)
        .expect("active Nazgul turn should receive live spawn context");
    let context = &context_map
        .get("pfterminal_spawn_context")
        .expect("spawn context entry")
        .value;

    assert!(context.contains("Euclid [nazgul]"), "got: {context}");
    assert!(context.contains("Burzum [troll]"), "got: {context}");
    assert!(context.contains("Snaga [orc]"), "got: {context}");
    assert!(context.contains("Ghash [orc]"), "got: {context}");
    assert!(!context.contains("none spawned yet"), "got: {context}");
    assert!(context.contains("send_message"), "got: {context}");
    assert!(context.contains("followup_task"), "got: {context}");
    assert!(!context.contains("<pfterminal_send_task"), "got: {context}");
}

#[tokio::test]
async fn claude_orc_completion_uses_core_edge_message_not_native_prompt_reinjection() {
    let mut app = make_test_app().await;
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000234").expect("valid thread id");
    app.upsert_agent_picker_thread(
        troll_thread_id,
        Some("Burzum".to_string()),
        Some("troll".to_string()),
        /*is_closed*/ false,
    );
    let orc_pane_id = app
        .claude_panes
        .create_pane_with_role(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
            Some(crate::spawn_orchestration::SpawnRole::Orc),
            Some("Snaga".to_string()),
        )
        .expect("create Claude Orc pane");
    app.spawn_parent_by_node.insert(
        crate::spawn_orchestration::pane_node_id(&orc_pane_id),
        crate::spawn_orchestration::thread_node_id(troll_thread_id),
    );

    app.on_claude_pane_turn_finished(
        orc_pane_id,
        Ok(crate::claude_panes::ClaudePaneTurnOutput {
            text: "Finished the latency benchmark table and saved the output.".to_string(),
            status: crate::claude_panes::ClaudePaneTurnStatus::Success,
            session_id: Some("claude-session".to_string()),
            usage_summary: None,
            turn_usage_summary: None,
            usage_status: crate::claude_panes::ClaudePaneUsageStatus::Missing,
            artifact_path: app.config.cwd.join("turn-0001.jsonl").to_path_buf(),
            audit_path: app.config.cwd.join("turn-0001.audit.json").to_path_buf(),
            duration_ms: 1,
            terminal_reason: None,
            error_summary: None,
            tool_names: Vec::new(),
            tool_events: Vec::new(),
            reasoning_events: Vec::new(),
            command_mode: crate::claude_panes::ClaudeCommandMode::NewSession,
            direct_accounting: None,
        }),
    );

    let context = app
        .spawn_context_for_thread(troll_thread_id)
        .expect("Troll should receive live lifecycle context");
    assert!(!context.contains("Recent child reports delivered to this pane:"));
    assert!(context.contains("Claude Code Snaga [orc] - Opus 5.5 Claude Plan"));
    assert!(context.contains("status=idle"));
    assert!(!context.contains("result=Finished the latency benchmark table and saved the output."));
}

#[tokio::test]
async fn unbound_main_does_not_persist_nazgul_role_metadata() {
    let mut app = make_test_app().await;
    let codex_home = tempdir().expect("codex home");
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    app.config.sqlite = codex_state::SqliteConfig::new_for_testing(codex_home.path().abs());
    let state_db = codex_state::StateRuntime::init(
        app.config.sqlite.clone(),
        app.config.model_provider_id.clone(),
    )
    .await
    .expect("state db");
    app.state_db = Some(state_db.clone());

    let root_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000237").expect("valid thread id");
    app.primary_thread_id = Some(root_thread_id);
    app.active_thread_id = Some(root_thread_id);
    app.spawn_nazgul_pane_id = None;

    app.persist_bound_nazgul_root_thread_metadata().await;

    assert!(
        state_db
            .get_thread(root_thread_id)
            .await
            .expect("read metadata")
            .is_none(),
        "an unbound primary thread must not be persisted as Nazgul"
    );
}

#[tokio::test]
async fn bound_nazgul_root_persists_role_metadata_to_state_db() {
    let mut app = make_test_app().await;
    let codex_home = tempdir().expect("codex home");
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    app.config.sqlite = codex_state::SqliteConfig::new_for_testing(codex_home.path().abs());
    let state_db = codex_state::StateRuntime::init(
        app.config.sqlite.clone(),
        app.config.model_provider_id.clone(),
    )
    .await
    .expect("state db");
    app.state_db = Some(state_db.clone());

    let root_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000236").expect("valid thread id");
    app.primary_thread_id = Some(root_thread_id);
    app.active_thread_id = Some(root_thread_id);
    let root_node_id = crate::spawn_orchestration::thread_node_id(root_thread_id);
    app.spawn_native_runtime_by_node.insert(
        root_node_id.clone(),
        crate::dispatch_queue::SavedNativeSpawnRuntime {
            model: codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL.to_string(),
            provider: CLAUDE_PLAN_PROVIDER_ID.to_string(),
            reasoning_effort: Some(ReasoningEffortConfig::High),
        },
    );
    app.set_spawn_nazgul_pane_binding(root_node_id);

    app.persist_bound_nazgul_root_thread_metadata().await;

    let metadata = state_db
        .get_thread(root_thread_id)
        .await
        .expect("read metadata")
        .expect("root row should be persisted");
    assert_eq!(metadata.agent_role.as_deref(), Some("nazgul"));
    assert_eq!(metadata.agent_nickname.as_deref(), Some("Main"));
    assert_eq!(
        metadata.model.as_deref(),
        Some(codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL)
    );
    assert_eq!(metadata.model_provider, CLAUDE_PLAN_PROVIDER_ID);
    assert_eq!(metadata.reasoning_effort, Some(ReasoningEffortConfig::High));
}

#[tokio::test]
async fn native_spawn_registration_persists_started_session_model_provider_pair() {
    let mut app = make_test_app().await;
    let codex_home = tempdir().expect("codex home");
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    app.config.sqlite = codex_state::SqliteConfig::new_for_testing(codex_home.path().abs());
    app.config.model_provider_id = "claude-plan".to_string();
    let state_db = codex_state::StateRuntime::init(
        app.config.sqlite.clone(),
        app.config.model_provider_id.clone(),
    )
    .await
    .expect("state db");
    app.state_db = Some(state_db.clone());

    let parent_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000238").expect("valid thread id");
    let troll_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000239").expect("valid thread id");
    app.upsert_agent_picker_thread(
        parent_thread_id,
        Some("Main".to_string()),
        Some("nazgul".to_string()),
        /*is_closed*/ false,
    );

    let started = crate::app_server_session::AppServerStartedThread {
        session: ThreadSessionState {
            model: "gpt-5.5".to_string(),
            model_provider_id: "openai".to_string(),
            reasoning_effort: Some(ReasoningEffortConfig::XHigh),
            ..test_thread_session(troll_thread_id, test_path_buf("/tmp/project"))
        },
        turns: Vec::new(),
        blocks_direct_input: false,
        task_tools_available: false,
    };
    app.register_spawn_agent_pane(
        troll_thread_id,
        parent_thread_id,
        crate::spawn_orchestration::thread_node_id(parent_thread_id),
        Some("Burzum".to_string()),
        "troll",
        started,
        /*persist_layout*/ true,
    )
    .await;

    let metadata = state_db
        .get_thread(troll_thread_id)
        .await
        .expect("read metadata")
        .expect("spawn row should be persisted");
    assert_eq!(metadata.agent_role.as_deref(), Some("troll"));
    assert_eq!(metadata.agent_nickname.as_deref(), Some("Burzum"));
    assert_eq!(metadata.model.as_deref(), Some("gpt-5.5"));
    assert_eq!(metadata.model_provider, "openai");
    assert_eq!(
        metadata.reasoning_effort,
        Some(ReasoningEffortConfig::XHigh)
    );
}

async fn codex_user_pane_remains_interactive_after_liveness_refresh_impl() -> Result<()> {
    let (app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    let mut app = Box::new(app);
    let mut app_server = start_config_write_test_app_server(&app).await?;
    let main = app_server.start_thread(&app.config).await?;
    let main_thread_id = main.session.thread_id;
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.primary_session_configured = Some(main.session.clone());
    app.thread_event_channels.insert(
        main_thread_id,
        ThreadEventChannel::new_with_session(/*capacity*/ 4, main.session, main.turns),
    );

    let pane_config = app.native_spawn_agent_config()?;
    let model = app.chat_widget.current_model().to_string();
    let provider = app.config.model_provider_id.clone();
    let started = app_server
        .start_user_pane_thread(
            &pane_config,
            model,
            Some(provider),
            app.chat_widget.current_reasoning_effort(),
        )
        .await?;
    assert!(
        !started.blocks_direct_input,
        "operator-created panes must be server-authored user threads"
    );
    let pane_rollout_path = started
        .session
        .rollout_path
        .as_ref()
        .expect("user-pane rollout path");
    assert!(
        pane_rollout_path.is_file(),
        "user-pane rollout must be durable"
    );
    let pane_thread_id = started.session.thread_id;
    app.register_codex_user_pane(
        &mut app_server,
        pane_thread_id,
        Some("Interactive".to_string()),
        started,
    )
    .await;

    let persisted_pane = app_server
        .thread_read(pane_thread_id, /*include_turns*/ false)
        .await?;
    assert_eq!(
        persisted_pane.name.as_deref(),
        Some("Interactive"),
        "user-pane name must persist"
    );

    let mut tui = crate::tui::test_support::make_test_tui()?;
    // Selection carries the full replay/reattachment state machine. Pin that nested future on the
    // heap so this test also guards against accidentally embedding it in its caller's stack frame.
    Box::pin(app.select_agent_thread(&mut tui, &mut app_server, pane_thread_id)).await?;
    while app_event_rx.try_recv().is_ok() {}
    assert!(!app.agent_navigation.is_parent_owned(pane_thread_id));
    assert_eq!(
        app.agent_navigation
            .get(&pane_thread_id)
            .and_then(|entry| entry.agent_nickname.as_deref()),
        Some("Interactive"),
        "liveness refresh must not erase the operator-assigned pane name"
    );

    app.chat_widget
        .restore_user_message_to_composer("operator pane input".into());
    app.chat_widget
        .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(
        std::iter::from_fn(|| app_event_rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::CodexOp(Op::UserTurn { .. }))),
        "operator-created pane input must reach the normal user-turn path"
    );
    app_server.shutdown().await?;
    Ok(())
}

#[test]
fn codex_user_pane_remains_interactive_after_liveness_refresh() -> Result<()> {
    // The Rust test harness uses a much smaller stack than the terminal process. Run this
    // full-App integration fixture on an explicitly sized test thread; the production behavior
    // remains exercised with the ordinary Tokio runtime and no environment-variable override.
    std::thread::Builder::new()
        .name("codex-user-pane-liveness".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(codex_user_pane_remains_interactive_after_liveness_refresh_impl())
        })
        .expect("spawn user-pane liveness test thread")
        .join()
        .expect("user-pane liveness test thread panicked")
}

#[tokio::test]
async fn human_addressable_spawn_pane_remains_interactive_after_liveness_refresh() -> Result<()> {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    let mut app_server = start_config_write_test_app_server(&app).await?;
    let main = app_server.start_thread(&app.config).await?;
    let main_thread_id = main.session.thread_id;
    app.primary_thread_id = Some(main_thread_id);
    app.active_thread_id = Some(main_thread_id);
    app.primary_session_configured = Some(main.session.clone());
    app.thread_event_channels.insert(
        main_thread_id,
        ThreadEventChannel::new_with_session(/*capacity*/ 4, main.session, main.turns),
    );

    let spawn_config = app.native_spawn_agent_config()?;
    let model = app.chat_widget.current_model().to_string();
    let provider = app.config.model_provider_id.clone();
    let started = app_server
        .spawn_agent_thread(
            &spawn_config,
            main_thread_id,
            "nazgul".to_string(),
            Some("Angmar".to_string()),
            model,
            Some(provider),
            app.chat_widget.current_reasoning_effort(),
            /*base_instructions*/ None,
        )
        .await?;
    assert!(
        !started.blocks_direct_input,
        "human-addressable /spawn panes must accept operator input"
    );
    let pane_thread_id = started.session.thread_id;
    app.register_spawn_agent_pane(
        pane_thread_id,
        main_thread_id,
        crate::spawn_orchestration::thread_node_id(main_thread_id),
        Some("Angmar".to_string()),
        "nazgul",
        started,
        /*persist_layout*/ true,
    )
    .await;

    let mut tui = crate::tui::test_support::make_test_tui()?;
    app.select_agent_thread(&mut tui, &mut app_server, pane_thread_id)
        .await?;
    while app_event_rx.try_recv().is_ok() {}
    assert!(!app.agent_navigation.is_parent_owned(pane_thread_id));

    app.chat_widget
        .restore_user_message_to_composer("manage the Troll and Orc review crew".into());
    app.chat_widget
        .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(
        std::iter::from_fn(|| app_event_rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::CodexOp(Op::UserTurn { .. }))),
        "operator input in a human-addressable /spawn pane must reach the normal user-turn path"
    );
    app_server.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn native_spawn_config_inherits_active_thread_permissions_not_stale_app_defaults() {
    let mut app = make_test_app().await;

    // Reproduce the real boundary: the app-level config still contains persisted defaults while
    // the active thread reflects CLI/runtime permission overrides.
    app.config
        .permissions
        .set_permission_profile(PermissionProfile::workspace_write())
        .expect("workspace profile should be accepted");
    app.config
        .permissions
        .approval_policy
        .set(AskForApproval::OnRequest.to_core())
        .expect("on-request should be accepted");
    app.chat_widget
        .set_permission_profile_from_session_snapshot(PermissionProfileSnapshot::legacy(
            PermissionProfile::Disabled,
        ))
        .expect("danger-full-access profile should be accepted");
    app.chat_widget.set_approval_policy(AskForApproval::Never);

    let spawn_config = app
        .native_spawn_agent_config()
        .expect("native spawn config");

    assert_eq!(
        spawn_config.permissions.approval_policy.value(),
        AskForApproval::Never.to_core()
    );
    assert_eq!(
        spawn_config.permissions.effective_permission_profile(),
        PermissionProfile::Disabled
    );
    assert_eq!(spawn_config.permissions.active_permission_profile(), None);
}

#[tokio::test]
async fn app_server_spawn_preserves_active_thread_yolo_permissions() -> Result<()> {
    let mut app = make_test_app().await;
    let mut app_server = start_config_write_test_app_server(&app).await?;
    let main = app_server.start_thread(&app.config).await?;

    app.chat_widget
        .set_permission_profile_from_session_snapshot(PermissionProfileSnapshot::legacy(
            PermissionProfile::Disabled,
        ))?;
    app.chat_widget.set_approval_policy(AskForApproval::Never);
    let spawn_config = app.native_spawn_agent_config()?;
    assert_eq!(
        spawn_config.permissions.approval_policy.value(),
        AskForApproval::Never.to_core()
    );
    assert_eq!(
        spawn_config.permissions.effective_permission_profile(),
        PermissionProfile::Disabled
    );
    assert_eq!(spawn_config.permissions.active_permission_profile(), None);
    let started = app_server
        .spawn_agent_thread(
            &spawn_config,
            main.session.thread_id,
            "nazgul".to_string(),
            Some("Angmar".to_string()),
            app.chat_widget.current_model().to_string(),
            Some(app.config.model_provider_id.clone()),
            app.chat_widget.current_reasoning_effort(),
            /*base_instructions*/ None,
        )
        .await?;

    assert_eq!(started.session.approval_policy, AskForApproval::Never);
    assert_eq!(
        started.session.permission_profile,
        PermissionProfile::Disabled
    );
    assert_eq!(started.session.active_permission_profile, None);
    app_server.shutdown().await?;
    Ok(())
}

#[test]
#[ignore = "live integration test: spawns real threads and needs Claude Plan/OpenAI/OpenRouter provider auth present in the environment"]
fn spawn_app_path_creates_default_heterogeneous_crew() -> Result<()> {
    const WORKER_THREADS: usize = 1;
    const TEST_STACK_SIZE_BYTES: usize = 8 * 1024 * 1024;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(WORKER_THREADS)
        .thread_stack_size(TEST_STACK_SIZE_BYTES)
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let mut app = make_test_app().await;
        if let Some(env_key) = app.config.model_provider.env_key.as_deref() {
            std::fs::write(
                app.config.codex_home.join("provider_auth.json"),
                format!(r#"{{"api_keys":{{"{env_key}":"test-key"}}}}"#),
            )?;
        }
        let mut app_server = Box::pin(crate::start_embedded_app_server_for_picker(
            app.chat_widget.config_ref(),
        ))
        .await?;
        let main = app_server
            .start_thread(app.chat_widget.config_ref())
            .await?;
        let main_thread_id = main.session.thread_id;
        app.primary_thread_id = Some(main_thread_id);
        app.active_thread_id = Some(main_thread_id);
        app.primary_session_configured = Some(main.session.clone());

        let native_spawn_config = app.native_spawn_agent_config()?;
        assert!(native_spawn_config.features.enabled(Feature::MultiAgentV2));
        assert!(
            native_spawn_config
                .features
                .enabled(Feature::MultiAgentMode)
        );

        let (nazgul_thread_id, troll_thread_id) =
            app.create_spawn_standard_crew(&mut app_server).await?;
        // The Nazgul is parented to Codex Main and auto-bound as root.
        assert_eq!(
            app.spawn_parent_by_thread.get(&nazgul_thread_id),
            Some(&main_thread_id)
        );
        assert_eq!(
            app.spawn_nazgul_pane_id.as_deref(),
            Some(crate::spawn_orchestration::thread_node_id(nazgul_thread_id).as_str())
        );
        // The Troll is parented to the Nazgul.
        assert_eq!(
            app.spawn_parent_by_thread.get(&troll_thread_id),
            Some(&nazgul_thread_id)
        );

        let orcs = app
            .agent_navigation
            .ordered_threads()
            .into_iter()
            .filter(|(thread_id, entry)| {
                app.spawn_parent_by_thread.get(thread_id) == Some(&troll_thread_id)
                    && entry.agent_role.as_deref() == Some("orc")
            })
            .collect::<Vec<_>>();
        assert_eq!(orcs.len(), 3);
        assert_eq!(orcs[0].1.agent_nickname.as_deref(), Some("Snaga"));
        assert_eq!(orcs[1].1.agent_nickname.as_deref(), Some("Ghash"));
        assert_eq!(orcs[2].1.agent_nickname.as_deref(), Some("Krimp"));
        assert_eq!(
            orcs.iter()
                .map(|(_, entry)| entry.model.as_deref())
                .collect::<Vec<_>>(),
            vec![
                Some(App::STANDARD_ORC_MODEL),
                Some(App::STANDARD_ORC_2_MODEL),
                Some(App::STANDARD_ORC_3_MODEL),
            ]
        );
        let status_items = app.spawn_tree_items(/*show_task_actions*/ true);
        assert!(
            status_items
                .iter()
                .all(|item| !item.name.contains("Demo task")),
            "spawn status must not expose a built-in demo action"
        );

        Ok(())
    })
}

#[tokio::test]
async fn native_spawn_defaults_follow_active_claude_profile() -> Result<()> {
    let mut app = make_test_app().await;
    let pane_id = app
        .claude_panes
        .create_pane_without_vault_for_test(
            crate::claude_panes::ClaudeProviderProfileKind::VercelGlm52Fast,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create Vercel Claude pane");

    app.claude_panes
        .set_active_user_pane(&pane_id)
        .expect("activate Vercel Claude pane");

    let model = app.native_spawn_default_model();
    assert_eq!(model, VERCEL_GLM_5_2_FAST_MODEL);
    assert_eq!(
        ChatWidget::model_provider_for_selection(&model).as_deref(),
        Some(VERCEL_ANTHROPIC_FAST_PROVIDER_ID)
    );
    Ok(())
}

#[tokio::test]
async fn idle_claude_pane_selection_clears_previous_live_status_panel() -> Result<()> {
    let mut app = make_test_app().await;
    let burzum_pane_id = app
        .claude_panes
        .create_pane_without_vault_for_test(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create running Troll pane");
    let nazgul_pane_id = app
        .claude_panes
        .create_pane_without_vault_for_test(
            crate::claude_panes::ClaudeProviderProfileKind::ClaudePlan,
            app.config.cwd.to_path_buf(),
            app.config.codex_home.as_ref(),
        )
        .expect("create idle Nazgul pane");

    app.claude_panes
        .set_active_user_pane(&burzum_pane_id)
        .expect("activate running pane");
    let _prepared = app
        .claude_panes
        .prepare_turn(
            &burzum_pane_id,
            "long running task".to_string(),
            app.config.codex_home.as_ref(),
        )
        .expect("prepare running pane turn");
    app.sync_external_pane_turn_display(&burzum_pane_id);

    assert!(app.chat_widget.is_task_running_for_test());
    assert!(app.chat_widget.status_indicator_visible_for_test());
    assert!(app.claude_panes.claude_pane_is_running(&burzum_pane_id));

    app.claude_panes
        .set_active_user_pane(&nazgul_pane_id)
        .expect("activate idle pane");
    app.sync_external_pane_turn_display(&nazgul_pane_id);

    assert!(!app.chat_widget.is_task_running_for_test());
    assert!(!app.chat_widget.status_indicator_visible_for_test());
    assert!(app.claude_panes.claude_pane_is_running(&burzum_pane_id));
    Ok(())
}

#[tokio::test]
async fn native_spawn_auth_guard_blocks_unauthenticated_openai() {
    let app = make_test_app().await;
    install_runtime_provider_policy(
        &app,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );

    let error = app
        .native_spawn_provider_auth_error(Some(OPENAI_PROVIDER_ID))
        .expect("OpenAI without auth should be rejected");

    assert!(error.contains("inactive or unavailable"));
}

#[tokio::test]
async fn native_spawn_auth_guard_accepts_openai_api_key_auth() {
    let mut app = make_test_app().await;
    app.chat_widget.update_account_state(
        Some(StatusAccountDisplay::ApiKey { env_var: None }),
        /*plan_type*/ None,
        /*has_chatgpt_account*/ false,
        /*has_codex_backend_auth*/ false,
    );
    install_runtime_provider_policy(
        &app,
        crate::provider_status_host::ProviderAccountMetadata {
            openai: codex_login::OpenAiAuthMetadata::ApiKey,
            ..Default::default()
        },
    );

    assert_eq!(
        app.native_spawn_provider_auth_error(Some(OPENAI_PROVIDER_ID)),
        None
    );
}

#[tokio::test]
async fn spawn_provider_allowlist_is_operator_policy_not_model_choice() {
    let mut app = make_test_app().await;
    app.config.agent_provider_allowlist = Some(vec![
        CLAUDE_PLAN_PROVIDER_ID.to_string(),
        OPENAI_PROVIDER_ID.to_string(),
    ]);

    // Authorized providers pass regardless of which path asked for them.
    for provider in [CLAUDE_PLAN_PROVIDER_ID, OPENAI_PROVIDER_ID] {
        app.ensure_native_spawn_provider_authorized(Some(provider))
            .unwrap_or_else(|err| panic!("{provider} should be authorized: {err}"));
    }

    // Every unauthorized provider is refused, not just the one that caused the
    // original incident. This is the class: a model selects a runtime, it never
    // authorizes one.
    for provider in [
        ANTHROPIC_PROVIDER_ID,
        OPENROUTER_PROVIDER_ID,
        KIMI_CODE_PROVIDER_ID,
        VERCEL_PROVIDER_ID,
    ] {
        let error = app
            .ensure_native_spawn_provider_authorized(Some(provider))
            .expect_err("unauthorized provider must be refused")
            .to_string();
        assert!(
            error.contains("agents.provider_allowlist"),
            "{provider} rejection should name the operator setting, got: {error}"
        );
    }
}

#[tokio::test]
async fn spawn_provider_allowlist_unset_stays_unrestricted() {
    let app = make_test_app().await;
    assert!(app.config.agent_provider_allowlist.is_none());

    for provider in [
        ANTHROPIC_PROVIDER_ID,
        OPENROUTER_PROVIDER_ID,
        KIMI_CODE_PROVIDER_ID,
    ] {
        app.ensure_native_spawn_provider_authorized(Some(provider))
            .unwrap_or_else(|err| panic!("{provider} should be unrestricted: {err}"));
    }

    // The declared ceiling for a new custom crew comes from configured providers,
    // never from whichever runtime a model happened to request first.
    let authorized = app.authorized_spawn_providers();
    assert!(
        authorized.len() > 1,
        "expected configured providers, got {authorized:?}"
    );
}

#[tokio::test]
async fn native_spawn_auth_guard_accepts_provider_key_storage() -> Result<()> {
    let app = make_test_app().await;
    install_runtime_provider_policy(
        &app,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );

    let missing = app
        .native_spawn_provider_auth_error(Some(VERCEL_PROVIDER_ID))
        .expect("Vercel without key should be rejected");
    assert!(missing.contains("inactive or unavailable"));

    std::fs::write(
        app.config.codex_home.join("provider_auth.json"),
        format!(r#"{{"api_keys":{{"{VERCEL_API_KEY_ENV_VAR}":"test-key"}}}}"#),
    )?;
    install_runtime_provider_policy(
        &app,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );

    assert!(
        app.native_spawn_provider_auth_error(Some(VERCEL_PROVIDER_ID))
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn native_spawn_auth_guard_rejects_unavailable_external_bearer_command() {
    let mut app = make_test_app().await;
    let mut provider = app
        .config
        .model_providers
        .get(CLAUDE_PLAN_PROVIDER_ID)
        .expect("Claude Plan provider should be configured")
        .clone();
    provider
        .auth
        .as_mut()
        .expect("Claude Plan provider should use external bearer auth")
        .command = app
        .config
        .codex_home
        .join("missing-provider-auth-command")
        .display()
        .to_string();
    app.config
        .model_providers
        .insert(CLAUDE_PLAN_PROVIDER_ID.to_string(), provider);

    let error = app
        .ensure_native_spawn_provider_ready(Some(CLAUDE_PLAN_PROVIDER_ID))
        .await
        .expect_err("unavailable external bearer auth should be rejected before native spawn");
    let message = error.to_string();

    assert!(message.contains("Claude Plan"));
    assert!(message.contains("provider authentication is unavailable"));
    assert!(message.contains("failed to start"));
}

#[tokio::test]
async fn native_spawn_auth_guard_uses_selected_provider_after_onboarding() -> Result<()> {
    let mut app = make_test_app().await;
    let provider = app
        .config
        .model_providers
        .get(OPENROUTER_PROVIDER_ID)
        .expect("OpenRouter provider should be configured")
        .clone();
    app.config.model_provider_id = OPENROUTER_PROVIDER_ID.to_string();
    app.config.model_provider = provider;
    std::fs::write(
        app.config.codex_home.join("provider_auth.json"),
        format!(r#"{{"api_keys":{{"{OPENROUTER_API_KEY_ENV_VAR}":"test-key"}}}}"#),
    )?;
    install_runtime_provider_policy(
        &app,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );

    assert!(
        app.native_spawn_provider_auth_error(/*provider_id*/ None)
            .is_none()
    );
    Ok(())
}

#[test]
fn guarded_model_selection_rejects_inactive_provider() -> Result<()> {
    std::thread::Builder::new()
        .name("guarded-model-selection".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async {
                    let mut app = Box::pin(make_test_app()).await;
                    std::fs::write(
                        app.config.codex_home.join("provider_auth.json"),
                        format!(r#"{{"api_keys":{{"{VERCEL_API_KEY_ENV_VAR}":"test-key"}}}}"#),
                    )?;
                    let host = crate::provider_status_host::ProviderStatusHost::from_config(
                        &app.config,
                        crate::provider_status_host::ProviderAccountMetadata::default(),
                    );
                    host.persist_policy(
                        VERCEL_PROVIDER_ID,
                        codex_provider_auth::ProviderActivationPolicy::Inactive,
                    )?;
                    app.model_catalog.set_provider_policy(
                        crate::chatwidget::provider_model_policy::ProviderModelPolicy::new(
                            host,
                            codex_provider_auth::ProviderRuntimeAuthorizations::default(),
                        ),
                    );
                    let original_provider = app.config.model_provider_id.clone();
                    let original_model = app.chat_widget.current_model().to_string();
                    let mut app_server =
                        Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
                    let mut tui = crate::tui::test_support::make_test_tui()?;

                    let control = Box::pin(app.handle_event(
                        &mut tui,
                        &mut app_server,
                        AppEvent::UpdateModelSelection {
                            model: VERCEL_GLM_5_2_FAST_MODEL.to_string(),
                            provider: Some(VERCEL_PROVIDER_ID.to_string()),
                        },
                    ))
                    .await?;

                    assert!(matches!(control, AppRunControl::Continue));
                    assert_eq!(app.config.model_provider_id, original_provider);
                    assert_eq!(app.chat_widget.current_model(), original_model);
                    app_server.shutdown().await?;
                    Ok(())
                })
        })?
        .join()
        .map_err(|_| color_eyre::eyre::eyre!("guarded model selection thread panicked"))?
}

fn install_runtime_provider_policy(
    app: &App,
    account: crate::provider_status_host::ProviderAccountMetadata,
) {
    let host = crate::provider_status_host::ProviderStatusHost::from_config(&app.config, account);
    app.model_catalog.set_provider_policy(
        crate::chatwidget::provider_model_policy::ProviderModelPolicy::new(
            host,
            codex_provider_auth::ProviderRuntimeAuthorizations::default(),
        ),
    );
}

#[tokio::test]
async fn stored_corbanu_key_can_refresh_the_current_runtime_policy_in_process() {
    let app = make_test_app().await;
    install_runtime_provider_policy(
        &app,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );
    let policy = app
        .model_catalog
        .provider_policy()
        .expect("test app should install a provider policy");
    let host = policy.host();
    codex_login::login_with_provider_api_key(
        &app.config.codex_home,
        codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR,
        "corbanu-refresh-test-key",
        app.config.cli_auth_credentials_store_mode,
        app.config.auth_keyring_backend_kind(),
    )
    .expect("store Corbanu API key");
    assert!(host.activate(codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID));
    app.model_catalog.refresh_provider_policy();

    assert!(app.model_catalog.provider_is_selectable(
        codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID,
        "corbanu/glm-5.3-flash",
    ));

    assert!(
        codex_login::delete_provider_api_key(
            &app.config.codex_home,
            codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR,
        )
        .expect("delete Corbanu API key")
    );
    app.model_catalog.refresh_provider_policy();
    assert!(!app.model_catalog.provider_is_selectable(
        codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID,
        "corbanu/glm-5.3-flash",
    ));
}
