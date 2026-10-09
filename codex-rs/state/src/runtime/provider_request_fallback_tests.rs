use super::*;
use crate::runtime::test_support::unique_temp_dir;
use pretty_assertions::assert_eq;
use std::path::Path;

fn key() -> ProviderRequestKey {
    ProviderRequestKey {
        provider_id: "zai".to_string(),
        model: "glm-5.2".to_string(),
        key_fingerprint: "fixture".to_string(),
    }
}

fn preflight() -> ProviderRequestPreflight {
    ProviderRequestPreflight {
        input_tokens: 37_492,
        cached_input_tokens: 0,
        request_bytes: 160_000,
        thread_id: None,
        turn_id: None,
    }
}

fn rate_limited() -> ProviderRequestResult {
    ProviderRequestResult::Failed {
        status: Some(429),
        request_id: Some("req-1".to_string()),
        retry_after_ms: Some(60_000),
    }
}

/// Another process holding the state DB's write lock until it is dropped.
async fn hold_write_lock(path: &Path) -> anyhow::Result<sqlx::Transaction<'static, sqlx::Sqlite>> {
    let pool = crate::sqlite::open_pool_for_testing(
        sqlx::sqlite::SqlitePoolOptions::new().max_connections(1),
        sqlx::sqlite::SqliteConnectOptions::new().filename(path),
    )
    .await?;
    Ok(pool.begin_with("BEGIN IMMEDIATE").await?)
}

async fn acquire(
    runtime: &StateRuntime,
    owner: &str,
    now_ms: i64,
) -> ProviderRequestThrottled<ProviderRequestLeaseDecision> {
    runtime
        .acquire_provider_request_throttle_lease(&key(), &preflight(), owner, 600_000, now_ms)
        .await
}

fn acquired(
    throttled: ProviderRequestThrottled<ProviderRequestLeaseDecision>,
) -> ProviderRequestLease {
    match throttled.decision {
        ProviderRequestLeaseDecision::Acquired(lease) => lease,
        ProviderRequestLeaseDecision::Blocked(block) => panic!("expected a lease, got {block:?}"),
    }
}

fn blocked_reason(
    throttled: &ProviderRequestThrottled<ProviderRequestLeaseDecision>,
) -> Option<ProviderRequestBlockReason> {
    match &throttled.decision {
        ProviderRequestLeaseDecision::Acquired(_) => None,
        ProviderRequestLeaseDecision::Blocked(block) => Some(block.reason.clone()),
    }
}

/// Another process holds the write lock for longer than the throttle waits.
/// Each step goes on from this process's memory, waits only once while the DB
/// is paused, and the limits this process knows still hold.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_lock_held_past_the_wait_falls_back_to_this_process() -> anyhow::Result<()> {
    let home = unique_temp_dir();
    let runtime = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;
    // A cooldown recorded while the DB answers still applies once it doesn't.
    let first = acquired(acquire(&runtime, "worker-a", 1_000).await);
    assert_eq!(first.in_state_db, true);
    let recorded = runtime
        .record_provider_request_throttle_result(&first, rate_limited(), 2_000)
        .await;
    assert_eq!(recorded, ProviderRequestThrottled::new(true, None));

    let held = hold_write_lock(&runtime.sqlite().state_db_path()).await?;
    let started = Instant::now();
    let check = runtime
        .check_provider_request_throttle(&key(), &preflight(), 3_000)
        .await;
    let first_wait = started.elapsed();
    assert_eq!(check.state_db_fallback, Some(StateDbFallback::Busy));
    let block = check.decision.expect("the cooldown still holds");
    assert_eq!(
        (block.reason, block.last_status, block.last_request_id),
        (
            ProviderRequestBlockReason::Cooldown,
            Some(429),
            Some("req-1".to_string())
        )
    );
    assert!(first_wait >= Duration::from_secs(5), "{first_wait:?}");

    // While paused, later steps don't wait the lock out again.
    let started = Instant::now();
    let after_cooldown = 70_000;
    let worker_b = acquire(&runtime, "worker-b", after_cooldown).await;
    assert_eq!(worker_b.state_db_fallback, Some(StateDbFallback::Busy));
    let worker_b = acquired(worker_b);
    assert_eq!(worker_b.in_state_db, false);
    let worker_c = acquire(&runtime, "worker-c", after_cooldown + 1).await;
    assert_eq!(
        blocked_reason(&worker_c),
        Some(ProviderRequestBlockReason::Lease)
    );
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );

    // The lease lives only here, so ending it needs no DB.
    let recorded = runtime
        .record_provider_request_throttle_result(&worker_b, rate_limited(), after_cooldown + 2)
        .await;
    assert_eq!(recorded, ProviderRequestThrottled::new(true, None));
    let check = runtime
        .check_provider_request_throttle(&key(), &preflight(), after_cooldown + 3)
        .await;
    assert_eq!(
        check.decision.map(|block| block.reason),
        Some(ProviderRequestBlockReason::Cooldown)
    );

    held.rollback().await?;
    // Once the DB answers again, the cooldown it never saw still applies here.
    runtime
        .provider_request_memory
        .paused
        .lock()
        .expect("pause lock")
        .take();
    let worker_d = acquire(&runtime, "worker-d", after_cooldown + 4).await;
    assert_eq!(worker_d.state_db_fallback, None);
    assert_eq!(
        blocked_reason(&worker_d),
        Some(ProviderRequestBlockReason::Cooldown)
    );
    // ...without taking a DB lease it would have to give back.
    let other_process = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;
    let elsewhere = acquire(&other_process, "worker-e", after_cooldown + 5).await;
    assert_eq!(blocked_reason(&elsewhere), None);

    other_process.close().await;
    runtime.close().await;
    let _ = std::fs::remove_dir_all(home);
    Ok(())
}

/// An interrupt while the result waits for a busy DB keeps the 429 cooldown in
/// this process.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_interrupted_record_keeps_the_cooldown() -> anyhow::Result<()> {
    let home = unique_temp_dir();
    let runtime = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;
    let lease = acquired(acquire(&runtime, "worker-a", 1_000).await);
    let held = hold_write_lock(&runtime.sqlite().state_db_path()).await?;
    let interrupted = tokio::time::timeout(
        Duration::from_millis(100),
        runtime.record_provider_request_throttle_result(&lease, rate_limited(), 2_000),
    )
    .await;
    assert!(interrupted.is_err());
    let block = runtime.provider_request_memory.with_row(&key(), |row| {
        row.cooldown_block(3_000, /*state_db_answered*/ true)
    });
    assert_eq!(
        block.map(|block| (block.reason, block.until_ms)),
        Some((ProviderRequestBlockReason::Cooldown, 62_000))
    );

    held.rollback().await?;
    runtime.close().await;
    let _ = std::fs::remove_dir_all(home);
    Ok(())
}

/// A state DB lease whose release failed (the DB stayed busy) is released by
/// the next step that reaches the DB, so it doesn't hold every process's
/// requests back until it expires.
#[tokio::test]
async fn a_lease_left_in_the_state_db_is_released_once_it_answers() -> anyhow::Result<()> {
    let home = unique_temp_dir();
    let runtime = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;
    let other_process = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;
    let lease = acquired(acquire(&runtime, "worker-a", 1_000).await);
    // What `release_provider_request_throttle_lease` leaves when the DB fails.
    let memory = &runtime.provider_request_memory;
    memory.with_row(&lease.key, |row| row.release(&lease.owner));
    memory.defer_releases([lease]);
    let elsewhere = acquire(&other_process, "worker-b", 2_000).await;
    assert_eq!(
        blocked_reason(&elsewhere),
        Some(ProviderRequestBlockReason::Lease)
    );

    let check = runtime
        .check_provider_request_throttle(&key(), &preflight(), 3_000)
        .await;
    assert_eq!(check, ProviderRequestThrottled::new(None, None));
    let elsewhere = acquire(&other_process, "worker-c", 4_000).await;
    assert_eq!(blocked_reason(&elsewhere), None);

    other_process.close().await;
    runtime.close().await;
    let _ = std::fs::remove_dir_all(home);
    Ok(())
}

/// A read-only state DB fails at once; the throttle goes on from memory and
/// still limits this process.
#[cfg(unix)]
#[tokio::test]
async fn a_read_only_state_db_falls_back_to_this_process() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let home = unique_temp_dir();
    let writable = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;
    let db_path = writable.sqlite().state_db_path();
    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o444))?;
    let runtime = StateRuntime::init_for_testing(home.clone(), "zai".into()).await?;

    let check = runtime
        .check_provider_request_throttle(&key(), &preflight(), 1_000)
        .await;
    assert_eq!(
        check,
        ProviderRequestThrottled::new(None, Some(StateDbFallback::ReadOnly))
    );
    let lease = acquired(acquire(&runtime, "worker-a", 1_001).await);
    let second = acquire(&runtime, "worker-b", 1_002).await;
    assert_eq!(second.state_db_fallback, Some(StateDbFallback::ReadOnly));
    assert_eq!(
        blocked_reason(&second),
        Some(ProviderRequestBlockReason::Lease)
    );
    assert_eq!(
        runtime
            .release_provider_request_throttle_lease(&lease, 1_003)
            .await?,
        1
    );
    acquired(acquire(&runtime, "worker-b", 1_004).await);

    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o644))?;
    runtime.close().await;
    writable.close().await;
    let _ = std::fs::remove_dir_all(home);
    Ok(())
}
