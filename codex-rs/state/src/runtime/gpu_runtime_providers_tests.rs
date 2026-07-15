use super::*;
use crate::GpuLimitEnforcement;
use crate::GpuRentalCreateParams;
use crate::GpuRentalState;
use crate::GpuRentalUpdate;

const NOW_MS: i64 = 1_800_000_000_000;

async fn runtime() -> Arc<StateRuntime> {
    let path = std::env::temp_dir().join(format!("gpu-overlay-test-{}", uuid::Uuid::new_v4()));
    StateRuntime::init(path, "test-provider".to_string())
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
        .claim_due_gpu_rentals("controller", now_ms, 10_000, 1)
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
                true,
                Some("{\"gpu_count\":2}"),
                None,
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
    assert_eq!(providers[0].catalog_sequence, 3);
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
