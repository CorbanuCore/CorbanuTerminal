use super::*;
use pretty_assertions::assert_eq;

const LONG_RETRY_MS: i64 = 308_374_000;

fn key() -> ProviderRequestKey {
    ProviderRequestKey {
        provider_id: "claude-plan".into(),
        model: "claude-opus-5-5-plan".into(),
        key_fingerprint: "config:auth-command".into(),
    }
}

fn preflight() -> ProviderRequestPreflight {
    ProviderRequestPreflight {
        input_tokens: 10,
        cached_input_tokens: 0,
        request_bytes: 100,
        thread_id: None,
        turn_id: None,
    }
}

fn refusal(retry_after_ms: Option<i64>) -> ProviderRequestResult {
    ProviderRequestResult::Failed {
        status: Some(429),
        request_id: Some("fixture-request".into()),
        retry_after_ms,
    }
}

async fn runtime() -> Arc<StateRuntime> {
    StateRuntime::init_for_testing(test_support::unique_temp_dir(), "claude-plan".into())
        .await
        .expect("state runtime")
}

async fn acquire(runtime: &StateRuntime, now_ms: i64) -> ProviderRequestLease {
    let decision = runtime
        .try_acquire_provider_request_lease_once(
            &key(),
            &preflight(),
            "probe",
            /*lease_ttl_ms*/ 30_000,
            now_ms,
        )
        .await
        .unwrap();
    let ProviderRequestLeaseDecision::Acquired(lease) = decision else {
        panic!("expected probe, got {decision:?}");
    };
    lease
}

#[tokio::test]
async fn multi_day_retry_is_capped_and_successful_probe_clears() {
    let runtime = runtime().await;
    let start = 1_000;
    runtime
        .record_provider_request_unleased_result_once(&key(), &refusal(Some(LONG_RETRY_MS)), start)
        .await
        .unwrap();
    let stored_deadline: i64 =
        sqlx::query_scalar("SELECT cooldown_until_ms FROM provider_request_state")
            .fetch_one(runtime.pool.as_ref())
            .await
            .unwrap();
    assert_eq!(stored_deadline, start + MAX_PROVIDER_COOLDOWN_MS);
    // Another in-flight refusal without a hint must retain the longer hint.
    runtime
        .record_provider_request_unleased_result_once(
            &key(),
            &refusal(/*retry_after_ms*/ None),
            start,
        )
        .await
        .unwrap();
    let block = runtime
        .check_provider_request_cooldown_once(&key(), &preflight(), start)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (block.remaining_ms, block.provider_reset_remaining_ms),
        (MAX_PROVIDER_COOLDOWN_MS, LONG_RETRY_MS)
    );
    let expiry = start + MAX_PROVIDER_COOLDOWN_MS;
    assert!(
        runtime
            .check_provider_request_cooldown_once(&key(), &preflight(), expiry - 1)
            .await
            .unwrap()
            .is_some()
    );
    let lease = acquire(&runtime, expiry).await;
    runtime
        .record_provider_request_result_once(
            &lease,
            ProviderRequestResult::Success {
                input_tokens: Some(12),
                cached_input_tokens: Some(2),
            },
            expiry + 1,
        )
        .await
        .unwrap();
    let state: (i64, i64, i64, i64) = sqlx::query_as("SELECT cooldown_until_ms, provider_reset_at_ms, consecutive_429_count, last_status FROM provider_request_state")
        .fetch_one(runtime.pool.as_ref()).await.unwrap();
    assert_eq!(state, (0, 0, 0, 200));
}

#[tokio::test]
async fn refused_probe_rearms_and_consecutive_backoff_stays_bounded() {
    let runtime = runtime().await;
    let start = 1_000;
    runtime
        .record_provider_request_unleased_result_once(&key(), &refusal(Some(LONG_RETRY_MS)), start)
        .await
        .unwrap();
    let expiry = start + MAX_PROVIDER_COOLDOWN_MS;
    let lease = acquire(&runtime, expiry).await;
    runtime
        .record_provider_request_result_once(&lease, refusal(Some(i64::MAX)), expiry)
        .await
        .unwrap();
    let block = runtime
        .check_provider_request_cooldown_once(&key(), &preflight(), expiry)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block.remaining_ms, MAX_PROVIDER_COOLDOWN_MS);
    let count: i64 = sqlx::query_scalar("SELECT consecutive_429_count FROM provider_request_state")
        .fetch_one(runtime.pool.as_ref())
        .await
        .unwrap();
    assert_eq!(count, 2);
    for count in [1, 2, 3, 4, 100, i64::MAX] {
        let update = UnleasedRow {
            consecutive_429_count: count,
            cooldown_until_ms: 0,
            provider_reset_at_ms: 0,
        }
        .apply(&refusal(/*retry_after_ms*/ None), expiry)
        .unwrap();
        assert!(update.cooldown_until_ms - expiry <= MAX_PROVIDER_COOLDOWN_MS);
    }
}

#[tokio::test]
async fn legacy_row_is_clamped_once_without_sliding_or_losing_hint() {
    let runtime = runtime().await;
    let start = 1_000;
    sqlx::query("INSERT INTO provider_request_state (provider_id, model, key_fingerprint, cooldown_until_ms, consecutive_429_count, last_status, updated_at_ms) VALUES (?, ?, ?, ?, 4, 429, ?)")
        .bind(&key().provider_id).bind(&key().model).bind(&key().key_fingerprint)
        .bind(start + LONG_RETRY_MS).bind(start)
        .execute(runtime.pool.as_ref()).await.unwrap();
    let block = runtime
        .check_provider_request_cooldown_once(&key(), &preflight(), start + 10_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (block.until_ms, block.provider_reset_remaining_ms),
        (start + MAX_PROVIDER_COOLDOWN_MS, LONG_RETRY_MS - 10_000)
    );
    // Another process reads it repeatedly: the deadline still counts from the
    // original row, never from a preflight timestamp refreshed by a reader.
    let second =
        StateRuntime::init_for_testing(runtime.sqlite.home().to_path_buf(), "claude-plan".into())
            .await
            .unwrap();
    let expiry = start + MAX_PROVIDER_COOLDOWN_MS;
    let block = second
        .check_provider_request_cooldown_once(&key(), &preflight(), expiry - 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block.remaining_ms, 1);
    acquire(&second, expiry).await;
    let count: i64 = sqlx::query_scalar("SELECT consecutive_429_count FROM provider_request_state")
        .fetch_one(runtime.pool.as_ref())
        .await
        .unwrap();
    assert_eq!(count, 4);
}

#[tokio::test]
async fn adopting_a_recent_cooldown_into_an_old_row_keeps_its_age() {
    let runtime = runtime().await;
    runtime
        .check_provider_request_cooldown_once(&key(), &preflight(), /*now_ms*/ 0)
        .await
        .unwrap();
    let now = 10 * MAX_PROVIDER_COOLDOWN_MS;
    sqlx::query("INSERT INTO provider_request_state (provider_id, model, key_fingerprint, cooldown_until_ms, last_status, updated_at_ms) VALUES (?, ?, 'by-name', ?, 429, ?)")
        .bind(&key().provider_id).bind(&key().model).bind(now + 60_000).bind(now)
        .execute(runtime.pool.as_ref()).await.unwrap();
    runtime
        .adopt_named_account_provider_request_key(
            &key().provider_id,
            "by-name",
            &key().key_fingerprint,
            now,
        )
        .await;
    let block = runtime
        .check_provider_request_cooldown_once(&key(), &preflight(), now)
        .await
        .unwrap()
        .expect("recent adopted cooldown");
    assert_eq!(block.remaining_ms, 60_000);
}

#[tokio::test]
async fn manual_clear_is_scoped_and_preserves_an_active_lease() {
    let runtime = runtime().await;
    let start = 1_000;
    let lease = acquire(&runtime, start).await;
    let keys = [
        key(),
        ProviderRequestKey {
            model: "other-model".into(),
            ..key()
        },
        ProviderRequestKey {
            provider_id: "other-provider".into(),
            ..key()
        },
        ProviderRequestKey {
            key_fingerprint: "other-key".into(),
            ..key()
        },
    ];
    for key in &keys {
        runtime
            .record_provider_request_throttle_outcome(key, refusal(Some(LONG_RETRY_MS)), start)
            .await;
    }
    runtime
        .clear_provider_request_cooldown(&key())
        .await
        .unwrap();
    assert!(
        runtime
            .check_provider_request_throttle(&key(), &preflight(), start + 1)
            .await
            .decision
            .is_none()
    );
    for key in &keys[1..] {
        assert!(
            runtime
                .check_provider_request_throttle(key, &preflight(), start + 1)
                .await
                .decision
                .is_some()
        );
    }
    let state: (i64, i64, i64, Option<String>, i64) = sqlx::query_as("SELECT cooldown_until_ms, provider_reset_at_ms, consecutive_429_count, lease_owner, lease_until_ms FROM provider_request_state WHERE provider_id = ? AND model = ? AND key_fingerprint = ?")
        .bind(&key().provider_id).bind(&key().model).bind(&key().key_fingerprint)
        .fetch_one(runtime.pool.as_ref()).await.unwrap();
    assert_eq!(state, (0, 0, 0, Some(lease.owner), lease.lease_until_ms));
    // Idempotent, including a key with no persisted row.
    runtime
        .clear_provider_request_cooldown(&key())
        .await
        .unwrap();
    runtime
        .clear_provider_request_cooldown(&ProviderRequestKey {
            model: "missing".into(),
            ..key()
        })
        .await
        .unwrap();
}
