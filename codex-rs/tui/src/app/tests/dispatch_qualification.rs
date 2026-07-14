use super::*;

use pretty_assertions::assert_eq;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::dispatch_queue::DispatchState;
use crate::dispatch_queue::MAX_GLOBAL_DISPATCH_BYTES;
use crate::dispatch_queue::MAX_GLOBAL_DISPATCH_ITEMS;
use crate::dispatch_queue::MAX_TARGET_DISPATCH_BYTES;
use crate::dispatch_queue::MAX_TARGET_DISPATCH_ITEMS;
use crate::dispatch_queue::queue_payload_bytes;
use crate::spawn_orchestration::PendingDispatchEnqueueResult;
use crate::spawn_orchestration::PendingSpawnDispatch;
use crate::spawn_orchestration::SpawnDispatchPumpDelivery;
use crate::spawn_orchestration::SpawnTaskDispatch;

fn qualification_thread(suffix: u16) -> ThreadId {
    ThreadId::from_string(&format!("00000000-0000-0000-0000-{suffix:012}"))
        .expect("qualification thread id")
}

fn assert_real_queue_invariants(app: &App, seed: u64, step: usize) {
    let mut dispatch_ids = std::collections::HashSet::new();
    let mut global_items = 0usize;
    let mut global_bytes = 0usize;

    for (target, queue) in &app.spawn_pending_dispatches {
        assert!(
            queue.len() <= MAX_TARGET_DISPATCH_ITEMS,
            "seed={seed} step={step}: target item bound exceeded for {target}"
        );
        let target_bytes = queue_payload_bytes(queue);
        assert!(
            target_bytes <= MAX_TARGET_DISPATCH_BYTES,
            "seed={seed} step={step}: target byte bound exceeded for {target}"
        );
        global_items += queue.len();
        global_bytes += target_bytes;

        let mut submitting = 0usize;
        for (index, dispatch) in queue.iter().enumerate() {
            assert_eq!(
                dispatch.target_pane_id, *target,
                "seed={seed} step={step}: target identity drift"
            );
            assert!(
                dispatch_ids.insert(dispatch.dispatch_id.clone()),
                "seed={seed} step={step}: duplicate queued dispatch id {}",
                dispatch.dispatch_id
            );
            match &dispatch.state {
                DispatchState::Queued => {}
                DispatchState::Submitting { delivery_id, .. } => {
                    submitting += 1;
                    assert_eq!(
                        index, 0,
                        "seed={seed} step={step}: only the FIFO head may submit"
                    );
                    assert!(
                        !app.spawn_accepted_delivery_ids.contains(delivery_id),
                        "seed={seed} step={step}: accepted delivery remained queued"
                    );
                }
                other => panic!(
                    "seed={seed} step={step}: terminal state remained in live queue: {other:?}"
                ),
            }
        }
        assert!(
            submitting <= 1,
            "seed={seed} step={step}: multiple submissions for {target}"
        );
        if app.spawn_dispatch_inflight_targets.contains(target) {
            assert!(
                queue.front().is_some_and(|dispatch| matches!(
                    dispatch.state,
                    DispatchState::Submitting { .. }
                )),
                "seed={seed} step={step}: inflight target lacks Submitting head"
            );
        }
    }
    assert!(
        global_items <= MAX_GLOBAL_DISPATCH_ITEMS,
        "seed={seed} step={step}: global item bound exceeded"
    );
    assert!(
        global_bytes <= MAX_GLOBAL_DISPATCH_BYTES,
        "seed={seed} step={step}: global byte bound exceeded"
    );
    assert!(
        app.spawn_dispatch_inflight_targets
            .iter()
            .all(|target| app.spawn_pending_dispatches.contains_key(target)),
        "seed={seed} step={step}: inflight target has no durable record"
    );

    let mut receipt_states = std::collections::HashSet::new();
    for report in app.spawn_parent_reports_by_node.values().flatten() {
        let Some((_, receipt)) = report.split_once("dispatch_ack; #") else {
            continue;
        };
        let Some((seq, after_seq)) = receipt.split_once(';') else {
            continue;
        };
        let Some((_, status)) = after_seq.split_once("status=") else {
            continue;
        };
        let status = status.split([';', '\n']).next().unwrap_or(status);
        assert!(
            receipt_states.insert((seq.to_string(), status.to_string())),
            "seed={seed} step={step}: duplicate receipt state for #{seq} status={status}"
        );
    }
}

#[tokio::test]
async fn seeded_real_pump_sequences_preserve_dispatch_invariants() {
    const SEEDS: [u64; 4] = [0x5A17_0001, 0x5A17_0029, 0x5A17_10FF, 0x5A17_C0DE];

    for seed in SEEDS {
        let (mut app, mut event_rx, _op_rx) = make_test_app_with_channels().await;
        let sources = [qualification_thread(810), qualification_thread(814)];
        for (index, source) in sources.iter().copied().enumerate() {
            app.upsert_agent_picker_thread(
                source,
                Some(format!("Qualification source {index}")),
                Some("troll".to_string()),
                false,
            );
        }
        let mut targets = [
            qualification_thread(811),
            qualification_thread(812),
            qualification_thread(813),
        ];
        for (index, target) in targets.iter().copied().enumerate() {
            app.upsert_agent_picker_thread(
                target,
                Some(format!("Qualification target {index}")),
                Some("orc".to_string()),
                false,
            );
            app.spawn_parent_by_thread.insert(target, sources[0]);
        }

        let boundary = PendingSpawnDispatch::new(
            "b".repeat(crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES),
            Vec::new(),
        );
        assert!(matches!(
            app.enqueue_pending_dispatch_for_thread(targets[0], boundary),
            PendingDispatchEnqueueResult::Queued
        ));

        let mut rng = StdRng::seed_from_u64(seed);
        let mut next_task = 0u64;
        let mut last_selected_target: Option<String> = None;
        let mut replacement_done = false;

        for step in 0..800usize {
            let action = rng.random_range(0..13);
            let target_index = rng.random_range(0..targets.len());
            let target = targets[target_index];
            match action {
                0..=2 => {
                    next_task += 1;
                    let pending = PendingSpawnDispatch::new(
                        format!("seed={seed} direct-task={next_task}"),
                        Vec::new(),
                    );
                    assert!(matches!(
                        app.enqueue_pending_dispatch_for_thread(target, pending),
                        PendingDispatchEnqueueResult::Queued
                    ));
                }
                3 => {
                    next_task += 1;
                    let source = sources[rng.random_range(0..sources.len())];
                    let turn_id = format!("seed-{seed}-source-turn-{step}");
                    let dispatch = SpawnTaskDispatch {
                        target: target.to_string(),
                        task: format!("seed={seed} model-task={next_task}"),
                        seq: Some(next_task),
                    };
                    let before = app
                        .spawn_pending_dispatches
                        .values()
                        .map(std::collections::VecDeque::len)
                        .sum::<usize>();
                    app.dispatch_spawn_task_blocks_from_model_turn(
                        &crate::spawn_orchestration::thread_node_id(source),
                        &crate::spawn_orchestration::thread_node_id(source),
                        &turn_id,
                        vec![dispatch.clone()],
                    );
                    app.dispatch_spawn_task_blocks_from_model_turn(
                        &crate::spawn_orchestration::thread_node_id(source),
                        &crate::spawn_orchestration::thread_node_id(source),
                        &turn_id,
                        vec![dispatch],
                    );
                    let after = app
                        .spawn_pending_dispatches
                        .values()
                        .map(std::collections::VecDeque::len)
                        .sum::<usize>();
                    assert_eq!(
                        after,
                        before + 1,
                        "seed={seed} step={step}: replayed source turn re-enqueued"
                    );
                }
                4 => {
                    while event_rx.try_recv().is_ok() {}
                    // Draining the test channel stands in for the top-level handler receiving the
                    // event; mirror the handler's first transition before testing duplicate wakes.
                    app.spawn_dispatch_pump_scheduled = false;
                    app.request_spawn_dispatch_pump();
                    app.request_spawn_dispatch_pump();
                    let wakes = std::iter::from_fn(|| event_rx.try_recv().ok())
                        .filter(|event| matches!(event, AppEvent::PumpSpawnDispatches))
                        .count();
                    assert_eq!(wakes, 1, "seed={seed} step={step}: pump wake not coalesced");
                }
                5..=7 => {
                    let eligible = app
                        .spawn_pending_dispatches
                        .iter()
                        .filter(|(node, queue)| {
                            !app.spawn_dispatch_inflight_targets.contains(*node)
                                && queue.front().is_some_and(|dispatch| {
                                    matches!(dispatch.state, DispatchState::Queued)
                                })
                                && app
                                    .resolve_spawn_task_target(node)
                                    .ok()
                                    .is_some_and(|target| match target {
                                        crate::spawn_orchestration::SpawnTaskTarget::Native(id) => {
                                            app.native_spawn_target_is_waiting_for_agents(id)
                                                || !app.native_spawn_target_is_busy(id)
                                        }
                                        _ => false,
                                    })
                        })
                        .map(|(node, _)| node.clone())
                        .collect::<Vec<_>>();
                    let front_by_target = app
                        .spawn_pending_dispatches
                        .iter()
                        .filter_map(|(node, queue)| {
                            queue
                                .front()
                                .map(|dispatch| (node.clone(), dispatch.task.clone()))
                        })
                        .collect::<std::collections::HashMap<_, _>>();
                    if let Some(delivery) = app.select_spawn_dispatch_pump_delivery() {
                        let (selected_target, task, delivery_id) = match delivery {
                            SpawnDispatchPumpDelivery::Native {
                                thread_id,
                                task,
                                delivery_id,
                                ..
                            } => (
                                app.logical_native_node_for_thread(thread_id),
                                task,
                                delivery_id,
                            ),
                            SpawnDispatchPumpDelivery::Claude { .. } => {
                                panic!("seed={seed} step={step}: native fixture selected Claude")
                            }
                        };
                        assert_eq!(
                            front_by_target.get(&selected_target),
                            Some(&task),
                            "seed={seed} step={step}: pump skipped FIFO head"
                        );
                        if eligible.len() > 1 {
                            assert_ne!(
                                last_selected_target.as_deref(),
                                Some(selected_target.as_str()),
                                "seed={seed} step={step}: fair pump repeated a target"
                            );
                        }
                        last_selected_target = Some(selected_target.clone());
                        match action {
                            5 => app.finish_spawn_dispatch_delivery(
                                &selected_target,
                                &delivery_id,
                                &task,
                            ),
                            6 => app
                                .defer_spawn_dispatch_for_capacity(&selected_target, &delivery_id),
                            _ => {
                                app.contain_ambiguous_spawn_dispatch(&selected_target);
                                assert!(
                                    app.submitting_spawn_dispatches()
                                        .iter()
                                        .any(|(node, id, _)| node == &selected_target
                                            && id == &delivery_id)
                                );
                                // A delayed thread/read witness reporting absence permits the same
                                // identified record to return to Queued without manufacturing work.
                                app.defer_spawn_dispatch_for_capacity(
                                    &selected_target,
                                    &delivery_id,
                                );
                            }
                        }
                    }
                }
                8 => {
                    let busy = !app.native_spawn_target_is_busy(target);
                    app.agent_navigation.set_running(target, busy);
                }
                9 => {
                    if app
                        .spawn_waiting_for_agents_by_thread
                        .remove(&target)
                        .is_none()
                    {
                        app.spawn_waiting_for_agents_by_thread.insert(
                            target,
                            (format!("wait-turn-{step}"), format!("wait-item-{step}")),
                        );
                    }
                }
                10 if !replacement_done => {
                    let replacement = qualification_thread(899);
                    app.upsert_agent_picker_thread(
                        replacement,
                        Some("Qualification replacement".to_string()),
                        Some("orc".to_string()),
                        false,
                    );
                    app.replace_saved_native_spawn_thread(target, replacement);
                    targets[target_index] = replacement;
                    replacement_done = true;
                }
                11 => {
                    app.fail_pending_dispatches_for_thread(
                        target,
                        format!("seed={seed} step={step}: injected target removal"),
                    );
                }
                _ => {
                    let oversized = PendingSpawnDispatch::new(
                        "x".repeat(crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES + 1),
                        Vec::new(),
                    );
                    assert!(matches!(
                        app.enqueue_pending_dispatch_for_thread(target, oversized),
                        PendingDispatchEnqueueResult::Rejected { .. }
                    ));
                    app.update_spawn_status_for_thread_notification(
                        &token_usage_notification_with_total(
                            target,
                            &format!("compaction-turn-{step}"),
                            99_000,
                            Some(100_000),
                        ),
                    );
                    assert!(app.agent_navigation.get(&target).is_some());
                }
            }

            assert_real_queue_invariants(&app, seed, step);
        }
    }
}
