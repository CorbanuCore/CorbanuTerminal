use super::*;
use crate::ProviderRequestKey;
use crate::ProviderRequestPreflight;
use crate::StateRuntime;
use crate::runtime::test_support::unique_temp_dir;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

/// Another process's connection to the same state DB.
async fn other_process(runtime: &StateRuntime, busy_timeout: Duration) -> sqlx::SqlitePool {
    crate::sqlite::open_pool_for_testing(
        sqlx::sqlite::SqlitePoolOptions::new().max_connections(1),
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(runtime.sqlite().state_db_path())
            .busy_timeout(busy_timeout),
    )
    .await
    .expect("open another connection")
}

fn key() -> ProviderRequestKey {
    ProviderRequestKey {
        provider_id: "claude-plan".to_string(),
        model: "claude-opus-5-5-plan".to_string(),
        key_fingerprint: "fixture".to_string(),
    }
}

fn preflight() -> ProviderRequestPreflight {
    ProviderRequestPreflight {
        input_tokens: 1,
        cached_input_tokens: 0,
        request_bytes: 1,
        thread_id: None,
        turn_id: None,
    }
}

/// Another process holds the write lock past SQLite's 5 s busy timeout. The
/// turn's throttle check used to fail the turn with "failed to check provider
/// request throttle state: ... database is locked"; it now waits the lock out.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_write_outlasts_a_lock_held_past_the_busy_timeout() -> anyhow::Result<()> {
    let home = unique_temp_dir();
    let runtime = StateRuntime::init_for_testing(home.clone(), "synthetic".into()).await?;
    let other = other_process(&runtime, Duration::from_secs(5)).await;
    let held = other.begin_with("BEGIN IMMEDIATE").await?;
    let release = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(6_500)).await;
        held.rollback().await
    });
    let started = Instant::now();
    let throttle = runtime
        .check_provider_request_cooldown(&key(), &preflight(), /*now_ms*/ 1)
        .await;
    let waited = started.elapsed();
    release.await??;
    assert_eq!(throttle?, None);
    assert!(waited >= Duration::from_secs(6), "{waited:?}");
    runtime.close().await;
    let _ = std::fs::remove_dir_all(home);
    Ok(())
}

#[tokio::test]
async fn only_busy_is_retried_and_only_until_the_deadline() -> anyhow::Result<()> {
    let home = unique_temp_dir();
    let runtime = StateRuntime::init_for_testing(home.clone(), "synthetic".into()).await?;
    let holder = other_process(&runtime, Duration::ZERO).await;
    let impatient = other_process(&runtime, Duration::ZERO).await;
    let held = holder.begin_with("BEGIN IMMEDIATE").await?;

    // Busy for as long as the lock is held: retried, then given up.
    let calls = AtomicU32::new(0);
    let error = retry_busy_within("fixture", Duration::from_millis(300), || async {
        calls.fetch_add(1, Ordering::Relaxed);
        impatient.begin_with("BEGIN IMMEDIATE").await?;
        anyhow::Ok(())
    })
    .await
    .expect_err("the lock stays held");
    assert!(is_busy(&error), "{error:#}");
    assert!(calls.load(Ordering::Relaxed) > 1);

    // Any other error is returned at once.
    let calls = AtomicU32::new(0);
    let error = retry_busy_within("fixture", Duration::from_secs(60), || async {
        calls.fetch_add(1, Ordering::Relaxed);
        sqlx::query("SELECT no_such_column FROM threads")
            .execute(&impatient)
            .await?;
        anyhow::Ok(())
    })
    .await
    .expect_err("no such column");
    assert!(!is_busy(&error), "{error:#}");
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(!is_busy(&anyhow::anyhow!("database is locked")));

    // Released between attempts: the write goes through.
    held.rollback().await?;
    retry_busy_within("fixture", Duration::from_secs(1), || async {
        let tx = impatient.begin_with("BEGIN IMMEDIATE").await?;
        tx.commit().await?;
        anyhow::Ok(())
    })
    .await?;
    runtime.close().await;
    let _ = std::fs::remove_dir_all(home);
    Ok(())
}
