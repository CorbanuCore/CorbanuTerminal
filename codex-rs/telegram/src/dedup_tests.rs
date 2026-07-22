use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::dedup::DEDUP_WINDOW;
use crate::dedup::UpdateDeduplicator;

#[tokio::test]
async fn first_seen_returns_true_and_repeat_returns_false() {
    let dedup = UpdateDeduplicator::new_for_test();
    assert!(dedup.check_and_record(42).await, "first sighting must pass");
    assert!(
        !dedup.check_and_record(42).await,
        "a replayed update id must be dropped"
    );
    assert!(dedup.check_and_record(43).await, "distinct ids pass");
}

#[tokio::test]
async fn window_is_bounded_and_evicts_oldest() {
    let dedup = UpdateDeduplicator::new_for_test();
    for id in 0..DEDUP_WINDOW as u64 {
        assert!(dedup.check_and_record(id).await);
    }
    assert_eq!(dedup.len_for_test().await, DEDUP_WINDOW);
    // One more insert evicts id 0; the window never grows past the bound.
    assert!(dedup.check_and_record(DEDUP_WINDOW as u64).await);
    assert_eq!(dedup.len_for_test().await, DEDUP_WINDOW);
    // The evicted id can be seen again (it fell out of the replay horizon)…
    assert!(dedup.check_and_record(0).await);
    // …but a still-windowed id remains a duplicate.
    assert!(!dedup.check_and_record(DEDUP_WINDOW as u64).await);
}

#[tokio::test]
async fn state_survives_restart_via_persistence() {
    let codex_home = unique_temp_dir("codex-telegram-dedup");
    {
        let dedup = UpdateDeduplicator::load(&codex_home).await;
        assert!(dedup.check_and_record(9001).await);
    }
    // A fresh load (simulating a process restart before Telegram's offset was
    // acknowledged) must still reject the replayed update.
    let dedup = UpdateDeduplicator::load(&codex_home).await;
    assert!(
        !dedup.check_and_record(9001).await,
        "restarted process must still drop a pre-crash update id"
    );
    assert!(dedup.check_and_record(9002).await);
    let _ = std::fs::remove_dir_all(&codex_home);
}

#[tokio::test]
async fn corrupt_state_file_starts_empty_and_is_set_aside() {
    let codex_home = unique_temp_dir("codex-telegram-dedup-corrupt");
    let telegram_dir = codex_home.join("telegram");
    std::fs::create_dir_all(&telegram_dir).expect("create telegram state dir");
    let state_path = telegram_dir.join("updates.json");
    std::fs::write(&state_path, "{ not json").expect("write corrupt state");

    let dedup = UpdateDeduplicator::load(&codex_home).await;
    assert!(
        dedup.check_and_record(7).await,
        "corrupt state must not block startup"
    );
    assert!(
        telegram_dir.join("updates.json.corrupt").exists(),
        "corrupt file must be renamed aside for inspection"
    );
    let _ = std::fs::remove_dir_all(&codex_home);
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
