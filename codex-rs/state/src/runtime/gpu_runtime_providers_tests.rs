use super::*;
use crate::GpuLimitEnforcement;
use crate::GpuRentalCreateParams;
use crate::GpuRentalState;
use crate::GpuRentalUpdate;

const NOW_MS: i64 = 1_800_000_000_000;

async fn runtime() -> Arc<StateRuntime> {
    let path = std::env::temp_dir().join(format!("gpu-overlay-test-{}", uuid::Uuid::new_v4()));
    StateRuntime::init_for_testing(path, "test-provider".to_string())
        .await
        .expect("initialize state")
}

fn params(rental_id: &str) -> GpuRentalCreateParams {
    GpuRentalCreateParams {
        rental_id: rental_id.to_string(),
        installation_id: "installation-1".to_string(),
        client_operation_id: format!("operation-{rental_id}"),
        provider: "fake".to_string(),
        recipe_id: "recipe-1".to_string(),
        recipe_revision: "revision-1".to_string(),
        offer_snapshot_json: "{\"offer_id\":\"offer-1\"}".to_string(),
        quote_expires_at_ms: Some(NOW_MS + 60_000),
        max_hourly_microusd: 1_000_000,
        max_total_microusd: 4_000_000,
        terminate_at_ms: NOW_MS + 4 * 60 * 60 * 1000,
        enforcement_class: GpuLimitEnforcement::LocalControllerDependent,
        ownership_tag: format!("pft-{rental_id}"),
    }
}

async fn set_state(runtime: &StateRuntime, rental_id: &str, state: GpuRentalState, now_ms: i64) {
    let lease = runtime
        .claim_due_gpu_rentals(
            "controller",
            now_ms,
            /*lease_ttl_ms*/ 10_000,
            /*limit*/ 1,
        )
        .await
        .expect("claim")
        .remove(0);
    assert_eq!(lease.rental.rental_id, rental_id);
    assert!(
        runtime
            .update_gpu_rental(
                &lease,
                &GpuRentalUpdate {
                    desired_state: Some(state),
                    observed_state: Some(state),
                    next_retry_at_ms: Some(now_ms),
                    ..GpuRentalUpdate::default()
                },
                now_ms,
            )
            .await
            .expect("update")
    );
}

#[tokio::test]
async fn provision_steps_are_idempotent_and_digest_bound() {
    let runtime = runtime().await;
    runtime
        .create_gpu_rental(&params("rental-step"), NOW_MS)
        .await
        .expect("create rental");

    assert!(
        runtime
            .begin_gpu_provision_step("rental-step", "01-hardware", "sha256:abc", NOW_MS)
            .await
            .expect("begin")
    );
    assert!(
        runtime
            .finish_gpu_provision_step(
                "rental-step",
                "01-hardware",
                /*succeeded*/ true,
                Some("{\"gpu_count\":2}"),
                /*sanitized_error*/ None,
                NOW_MS + 1,
            )
            .await
            .expect("finish")
    );
    assert!(
        !runtime
            .begin_gpu_provision_step("rental-step", "01-hardware", "sha256:changed", NOW_MS + 2,)
            .await
            .expect("replay")
    );
    let steps = runtime
        .list_gpu_provision_steps("rental-step")
        .await
        .expect("steps");
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].status, "succeeded");
    assert_eq!(steps[0].attempt_count, 1);
}

#[tokio::test]
async fn running_provision_step_can_be_resumed_after_controller_death() {
    let state = runtime().await;
    state
        .create_gpu_rental(&params("rental-resume"), NOW_MS)
        .await
        .expect("create rental");
    assert!(
        state
            .begin_gpu_provision_step("rental-resume", "01-hardware", "sha256:abc", NOW_MS)
            .await
            .expect("begin")
    );
    assert!(
        state
            .begin_gpu_provision_step("rental-resume", "01-hardware", "sha256:abc", NOW_MS + 1,)
            .await
            .expect("resume")
    );
    let steps = state
        .list_gpu_provision_steps("rental-resume")
        .await
        .expect("steps");
    assert_eq!(steps[0].status, "running");
    assert_eq!(steps[0].attempt_count, 2);
}

#[tokio::test]
async fn runtime_overlay_requires_ready_and_https_and_is_sequence_monotonic() {
    let runtime = runtime().await;
    runtime
        .create_gpu_rental(&params("rental-overlay"), NOW_MS)
        .await
        .expect("create rental");
    let overlay = GpuRuntimeProviderUpsert {
        rental_id: "rental-overlay".to_string(),
        provider_id: "gpu-rental-overlay".to_string(),
        base_url: "https://rental.example.invalid/v1".to_string(),
        model_id: "pinned/model".to_string(),
        wire_api: "chat".to_string(),
        health: "ready".to_string(),
        display_hourly_microusd: 1_000_000,
        maximum_context_tokens: 65_536,
        catalog_sequence: 2,
    };
    assert!(
        !runtime
            .upsert_gpu_runtime_provider(&overlay, NOW_MS)
            .await
            .expect("pre-ready upsert")
    );

    set_state(&runtime, "rental-overlay", GpuRentalState::Ready, NOW_MS).await;
    assert!(
        runtime
            .upsert_gpu_runtime_provider(&overlay, NOW_MS + 1)
            .await
            .expect("ready upsert")
    );
    let mut stale = overlay.clone();
    stale.catalog_sequence = 1;
    stale.health = "degraded".to_string();
    assert!(
        !runtime
            .upsert_gpu_runtime_provider(&stale, NOW_MS + 2)
            .await
            .expect("stale upsert")
    );
    let providers = runtime
        .list_gpu_runtime_providers()
        .await
        .expect("providers");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].health, "ready");
    assert_eq!(providers[0].maximum_context_tokens, Some(65_536));
    assert!(!providers[0].base_url.contains("token"));

    let mut responses = overlay.clone();
    responses.catalog_sequence = 3;
    responses.wire_api = "responses".to_string();
    assert!(
        runtime
            .upsert_gpu_runtime_provider(&responses, NOW_MS + 2)
            .await
            .expect("responses upsert")
    );
    assert_eq!(
        runtime
            .list_gpu_runtime_providers()
            .await
            .expect("responses provider")[0]
            .wire_api,
        "responses"
    );

    assert!(
        runtime
            .set_gpu_runtime_provider_health("rental-overlay", "degraded", NOW_MS + 3)
            .await
            .expect("disable provider")
    );
    let providers = runtime
        .list_gpu_runtime_providers()
        .await
        .expect("providers after disable");
    assert_eq!(providers[0].health, "degraded");
    assert_eq!(providers[0].catalog_sequence, 4);

    let mut controller_refresh = overlay.clone();
    controller_refresh.base_url = "https://replacement.example.invalid/v1".to_string();
    controller_refresh.catalog_sequence = 2;
    assert!(
        runtime
            .refresh_gpu_runtime_provider(&controller_refresh, NOW_MS + 4)
            .await
            .expect("controller endpoint refresh")
    );
    let refreshed = &runtime
        .list_gpu_runtime_providers()
        .await
        .expect("provider after endpoint refresh")[0];
    assert_eq!(refreshed.base_url, "https://replacement.example.invalid/v1");
    assert_eq!(refreshed.catalog_sequence, 5);

    assert!(
        runtime
            .set_gpu_runtime_provider_health("rental-overlay", "disabled", NOW_MS + 4)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn plaintext_remote_http_endpoint_is_rejected() {
    let runtime = runtime().await;
    runtime
        .create_gpu_rental(&params("rental-http"), NOW_MS)
        .await
        .expect("create rental");
    set_state(&runtime, "rental-http", GpuRentalState::Ready, NOW_MS).await;
    let error = runtime
        .upsert_gpu_runtime_provider(
            &GpuRuntimeProviderUpsert {
                rental_id: "rental-http".to_string(),
                provider_id: "gpu-rental-http".to_string(),
                base_url: "http://203.0.113.1:8000/v1".to_string(),
                model_id: "pinned/model".to_string(),
                wire_api: "chat".to_string(),
                health: "ready".to_string(),
                display_hourly_microusd: 1_000_000,
                maximum_context_tokens: 65_536,
                catalog_sequence: 1,
            },
            NOW_MS,
        )
        .await
        .expect_err("remote plaintext endpoint must fail");
    assert!(error.to_string().contains("HTTPS or loopback"));
}

/// #2723: config loading's read-only reader returns what the runtime lists,
/// and refuses a state database with migrations this build doesn't know.
#[tokio::test]
async fn read_only_overlay_reader_matches_runtime_and_refuses_foreign_databases() {
    let home = std::env::temp_dir().join(format!("gpu-overlay-ro-{}", uuid::Uuid::new_v4()));
    let runtime = StateRuntime::init_for_testing(home.clone(), "test-provider".to_string())
        .await
        .expect("initialize state");
    runtime
        .create_gpu_rental(&params("rental-ro"), NOW_MS)
        .await
        .expect("create rental");
    set_state(&runtime, "rental-ro", GpuRentalState::Ready, NOW_MS).await;
    assert!(
        runtime
            .upsert_gpu_runtime_provider(
                &GpuRuntimeProviderUpsert {
                    rental_id: "rental-ro".to_string(),
                    provider_id: "gpu-rental-ro".to_string(),
                    base_url: "https://rental.example.invalid/v1".to_string(),
                    model_id: "pinned/model".to_string(),
                    wire_api: "chat".to_string(),
                    health: "ready".to_string(),
                    display_hourly_microusd: 1_000_000,
                    maximum_context_tokens: 65_536,
                    catalog_sequence: 1,
                },
                NOW_MS + 1,
            )
            .await
            .expect("ready upsert")
    );
    let sqlite = SqliteConfig::new_for_testing(home.as_path().abs());
    let timeout = std::time::Duration::from_secs(2);

    let read = sqlite
        .read_gpu_runtime_providers(timeout)
        .await
        .expect("read-only overlay");
    assert_eq!(read.len(), 1);
    assert_eq!(
        read,
        runtime.list_gpu_runtime_providers().await.expect("list")
    );

    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(9_999_i64)
    .bind("foreign migration")
    .bind(true)
    .bind(vec![1_u8, 2, 3, 4])
    .bind(1_i64)
    .execute(runtime.pool.as_ref())
    .await
    .expect("insert foreign migration record");
    let err = sqlite
        .read_gpu_runtime_providers(timeout)
        .await
        .expect_err("a database with foreign migrations is refused");
    assert!(err.to_string().contains("foreign migrations"), "{err}");

    runtime.close().await;
}

/// #2728: while another connection holds the state database's write lock,
/// the read-only overlay reader waits at most its own `busy_timeout` in all:
/// not SQLite's default 5 s per step, nor a full timeout for each of its steps
/// (open, migration check, read).
#[tokio::test]
async fn read_only_overlay_reader_waits_only_its_busy_timeout_for_a_locked_database() {
    let home = tempfile::tempdir().expect("temp dir");
    let sqlite = SqliteConfig::new_for_testing(home.path().abs());
    let path = sqlite.state_db_path();
    let pool = sqlite
        .open_read_write_pool(&path)
        .await
        .expect("open state database");
    let mut writer = pool.acquire().await.expect("writer connection");
    sqlx::query("CREATE TABLE synthetic_history (id INTEGER)")
        .execute(&mut *writer)
        .await
        .expect("create table");
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode=DELETE")
        .fetch_one(&mut *writer)
        .await
        .expect("journal mode");
    // Only DELETE journaling makes the write lock block readers.
    assert_eq!(journal_mode, "delete");
    sqlx::query("BEGIN EXCLUSIVE")
        .execute(&mut *writer)
        .await
        .expect("take write lock");

    let busy_timeout = std::time::Duration::from_secs(1);
    // How long one wait of `busy_timeout` really takes here: opening a reader
    // waits once. SQLite's sleeps overshoot on Windows (about 1.8 s for 1 s).
    let started = std::time::Instant::now();
    let reader = sqlite
        .open_read_only_pool(&path, Some(busy_timeout))
        .await
        .expect("open a reader");
    let one_wait = started.elapsed();
    reader.close().await;
    assert!(
        one_wait >= busy_timeout.mul_f32(0.9),
        "opening a reader did not wait for the write lock: {one_wait:?}"
    );

    let started = std::time::Instant::now();
    let result = sqlite.read_gpu_runtime_providers(busy_timeout).await;
    let elapsed = started.elapsed();
    assert!(
        result.is_err(),
        "the locked database cannot be read: {result:?}"
    );
    // The read waited for the lock and gave up after one wait, not one per
    // step (two before the fix: the open and the migration check).
    assert!(
        elapsed >= busy_timeout.mul_f32(0.9),
        "the read did not wait for the write lock: {elapsed:?}, {result:?}"
    );
    assert!(
        elapsed < one_wait.mul_f32(1.5),
        "waited {elapsed:?} for the write lock, more than one wait ({one_wait:?})"
    );

    sqlx::query("ROLLBACK")
        .execute(&mut *writer)
        .await
        .expect("release write lock");
}
