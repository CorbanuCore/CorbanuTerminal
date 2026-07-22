use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use teloxide::types::Update;

use crate::dedup::BeginUpdate;
use crate::dedup::DEDUP_WINDOW;
use crate::dedup::UpdateDeduplicator;

#[tokio::test]
async fn completion_deduplicates_replay() {
    let inbox = UpdateDeduplicator::new_for_test();
    let update = message_update(42);
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);
    inbox.complete_update(42).await.unwrap();
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Duplicate);
}

#[tokio::test]
async fn pending_update_is_claimed_once_per_process() {
    let inbox = UpdateDeduplicator::new_for_test();
    let update = message_update(43);
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Duplicate);
    inbox.release_update(43).await;
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);
}

#[tokio::test]
async fn completed_window_is_bounded_and_evicts_oldest() {
    let codex_home = unique_temp_dir("codex-telegram-window");
    let inbox = UpdateDeduplicator::load(&codex_home, 1).await;
    for id in 0..=DEDUP_WINDOW as u64 {
        let update = message_update(id);
        assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);
        inbox.complete_update(id).await.unwrap();
    }
    assert_eq!(inbox.len_for_test().await, DEDUP_WINDOW);
    assert_eq!(
        inbox.begin_update(&message_update(0)).await,
        BeginUpdate::Accepted
    );
    assert_eq!(
        inbox
            .begin_update(&message_update(DEDUP_WINDOW as u64))
            .await,
        BeginUpdate::Duplicate
    );
    let _ = std::fs::remove_dir_all(codex_home);
}

#[tokio::test]
async fn pending_survives_restart_and_completion_then_deduplicates() {
    let codex_home = unique_temp_dir("codex-telegram-inbox");
    let update = message_update(9001);
    {
        let inbox = UpdateDeduplicator::load(&codex_home, 77).await;
        assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);
    }

    let inbox = UpdateDeduplicator::load(&codex_home, 77).await;
    assert_eq!(inbox.pending_updates().await, vec![update.clone()]);
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);
    inbox.complete_update(9001).await.unwrap();

    let restarted = UpdateDeduplicator::load(&codex_home, 77).await;
    assert!(restarted.pending_updates().await.is_empty());
    assert_eq!(
        restarted.begin_update(&update).await,
        BeginUpdate::Duplicate
    );
    let _ = std::fs::remove_dir_all(codex_home);
}

#[tokio::test]
async fn failed_completion_persistence_rolls_back_in_memory_state() {
    let codex_home = unique_temp_dir("codex-telegram-inbox-write-failure");
    let inbox_dir = codex_home.join("telegram");
    let moved_inbox_dir = codex_home.join("telegram-before-failure");
    let update = message_update(9002);
    let inbox = UpdateDeduplicator::load(&codex_home, 77).await;
    assert_eq!(inbox.begin_update(&update).await, BeginUpdate::Accepted);

    // Make the existing parent path a regular file. Atomic persistence must
    // fail without committing the in-memory completed marker.
    std::fs::rename(&inbox_dir, &moved_inbox_dir).unwrap();
    std::fs::write(&inbox_dir, b"not a directory").unwrap();
    assert!(inbox.complete_update(9002).await.is_err());
    assert_eq!(inbox.pending_ids_for_test().await, vec![9002]);
    assert_eq!(inbox.len_for_test().await, 0);

    std::fs::remove_file(&inbox_dir).unwrap();
    std::fs::rename(&moved_inbox_dir, &inbox_dir).unwrap();
    inbox.complete_update(9002).await.unwrap();
    assert!(inbox.pending_ids_for_test().await.is_empty());
    assert_eq!(inbox.len_for_test().await, 1);
    let _ = std::fs::remove_dir_all(codex_home);
}

#[tokio::test]
async fn bot_id_partitions_durable_inboxes() {
    let codex_home = unique_temp_dir("codex-telegram-bot-partition");
    let update = message_update(7);
    let first = UpdateDeduplicator::load(&codex_home, 1).await;
    assert_eq!(first.begin_update(&update).await, BeginUpdate::Accepted);
    first.complete_update(7).await.unwrap();

    let second = UpdateDeduplicator::load(&codex_home, 2).await;
    assert_eq!(second.begin_update(&update).await, BeginUpdate::Accepted);
    let _ = std::fs::remove_dir_all(codex_home);
}

#[tokio::test]
async fn corrupt_state_file_starts_empty_and_is_set_aside() {
    let codex_home = unique_temp_dir("codex-telegram-dedup-corrupt");
    let telegram_dir = codex_home.join("telegram");
    std::fs::create_dir_all(&telegram_dir).unwrap();
    let state_path = telegram_dir.join("updates-7.json");
    std::fs::write(&state_path, "{ not json").unwrap();

    let inbox = UpdateDeduplicator::load(&codex_home, 7).await;
    assert_eq!(
        inbox.begin_update(&message_update(7)).await,
        BeginUpdate::Accepted
    );
    assert!(telegram_dir.join("updates-7.json.corrupt").exists());
    let _ = std::fs::remove_dir_all(codex_home);
}

fn message_update(id: u64) -> Update {
    serde_json::from_value(serde_json::json!({
        "update_id": id,
        "message": {
            "message_id": id,
            "date": 1,
            "chat": {"id": 99, "type": "private"},
            "from": {"id": 99, "is_bot": false, "first_name": "Tester"},
            "text": "hello"
        }
    }))
    .unwrap()
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
