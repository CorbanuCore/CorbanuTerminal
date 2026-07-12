use pretty_assertions::assert_eq;

use super::DispatchState;
use super::MAX_DISPATCH_TASK_BYTES;
use super::MAX_GLOBAL_DISPATCH_BYTES;
use super::MAX_GLOBAL_DISPATCH_ITEMS;
use super::MAX_TARGET_DISPATCH_BYTES;
use super::MAX_TARGET_DISPATCH_ITEMS;
use super::PendingSpawnDispatch;
use super::bounded_delivery_batch;
use super::expand_legacy_batch;
use super::model_dispatch_origin_id;
use super::queue_bound_violation;

#[test]
fn legacy_string_dispatch_deserializes_as_queued() {
    let dispatch: PendingSpawnDispatch =
        serde_json::from_str(r#""legacy task""#).expect("legacy dispatch");

    assert_eq!(dispatch.task, "legacy task");
    assert_eq!(dispatch.state, DispatchState::Queued);
    assert!(dispatch.dispatch_id.is_empty());
}

#[test]
fn legacy_identity_is_stable_and_target_scoped() {
    let mut first = PendingSpawnDispatch::new("task".to_string(), Vec::new());
    first.created_at_ms = 1234;
    let mut same = first.clone();
    let mut other_target = first.clone();

    first.migrate_legacy_identity("thread:a", 0);
    same.migrate_legacy_identity("thread:a", 0);
    other_target.migrate_legacy_identity("thread:b", 0);

    assert_eq!(first.dispatch_id, same.dispatch_id);
    assert_eq!(first.origin, same.origin);
    assert_ne!(first.dispatch_id, other_target.dispatch_id);
}

#[test]
fn legacy_batch_reader_preserves_structural_boundaries() {
    let task_a = "first\nwith markdown";
    let task_b = "second";
    let envelope = format!(
        "Multiple spawn dispatches were queued while you were busy. Execute each task below in order, do not skip any task, and treat every section as assigned work.\n\n## Queued dispatch 1 (bytes={})\n{}\n## Queued dispatch 2 (bytes={})\n{}",
        task_a.len(),
        task_a,
        task_b.len(),
        task_b
    );

    assert_eq!(
        expand_legacy_batch(&envelope),
        Some(vec![task_a.to_string(), task_b.to_string()])
    );
}

#[test]
fn queue_bounds_reject_only_growth_beyond_each_limit() {
    assert_eq!(
        queue_bound_violation(
            MAX_DISPATCH_TASK_BYTES,
            MAX_TARGET_DISPATCH_ITEMS - 1,
            MAX_TARGET_DISPATCH_BYTES - MAX_DISPATCH_TASK_BYTES,
            MAX_GLOBAL_DISPATCH_ITEMS - 1,
            MAX_GLOBAL_DISPATCH_BYTES - MAX_DISPATCH_TASK_BYTES,
        ),
        None
    );
    assert!(queue_bound_violation(MAX_DISPATCH_TASK_BYTES + 1, 0, 0, 0, 0).is_some());
    assert!(queue_bound_violation(1, MAX_TARGET_DISPATCH_ITEMS, 0, 0, 0).is_some());
    assert!(queue_bound_violation(2, 0, MAX_TARGET_DISPATCH_BYTES - 1, 0, 0).is_some());
    assert!(queue_bound_violation(1, 0, 0, MAX_GLOBAL_DISPATCH_ITEMS, 0).is_some());
    assert!(queue_bound_violation(2, 0, 0, 0, MAX_GLOBAL_DISPATCH_BYTES - 1).is_some());
}

#[test]
fn model_origin_uses_turn_and_ordinal_not_task_wording() {
    let first = model_dispatch_origin_id("thread:a", "turn-1", 0);

    assert_eq!(first, model_dispatch_origin_id("thread:a", "turn-1", 0));
    assert_ne!(first, model_dispatch_origin_id("thread:a", "turn-1", 1));
    assert_ne!(first, model_dispatch_origin_id("thread:a", "turn-2", 0));
}

#[test]
fn delivery_batch_preserves_fifo_while_enforcing_item_and_byte_caps() {
    let dispatches = (0..20)
        .map(|index| PendingSpawnDispatch::new(format!("task-{index}"), Vec::new()))
        .collect::<Vec<_>>();
    let item_bounded = bounded_delivery_batch(dispatches);
    assert_eq!(item_bounded.len(), 16);
    assert_eq!(item_bounded[0].task, "task-0");
    assert_eq!(item_bounded[15].task, "task-15");

    let dispatches = (0..8)
        .map(|_| PendingSpawnDispatch::new("x".repeat(32 * 1024), Vec::new()))
        .collect::<Vec<_>>();
    assert_eq!(bounded_delivery_batch(dispatches).len(), 4);
}
