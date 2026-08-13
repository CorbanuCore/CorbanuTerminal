use super::test_support::unique_temp_dir;
use super::*;
use crate::GpuLimitEnforcement;
use crate::GpuOperationKind;
use crate::GpuRentalCreateParams;
use crate::GpuRentalState;
use crate::GpuRentalUpdate;
use pretty_assertions::assert_eq;

const NOW_MS: i64 = 1_800_000_000_000;

fn create_params(client_operation_id: &str) -> GpuRentalCreateParams {
    GpuRentalCreateParams {
        rental_id: format!("rental-{client_operation_id}"),
        installation_id: "install-1".to_string(),
        client_operation_id: client_operation_id.to_string(),
        provider: "fake".to_string(),
        recipe_id: "deepseek-flash-2xh200".to_string(),
        recipe_revision: "sha256:test".to_string(),
        offer_snapshot_json: r#"{"offer_id":"offer-1","hourly_microusd":2500000}"#.to_string(),
        quote_expires_at_ms: Some(NOW_MS + 60_000),
        max_hourly_microusd: 3_000_000,
        max_total_microusd: 12_000_000,
        terminate_at_ms: NOW_MS + 4 * 60 * 60 * 1000,
        enforcement_class: GpuLimitEnforcement::LocalControllerDependent,
        ownership_tag: format!("pft-install-1-{client_operation_id}"),
    }
}

async fn runtime() -> std::sync::Arc<StateRuntime> {
    StateRuntime::init_for_testing(unique_temp_dir(), "test-provider".to_string())
        .await
        .expect("initialize state runtime")
}

#[tokio::test]
async fn create_is_idempotent_but_rejects_changed_terms() {
    let runtime = runtime().await;
    let params = create_params("op-1");
    let first = runtime
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create rental");
    let replay = runtime
        .create_gpu_rental(&params, NOW_MS + 1)
        .await
        .expect("replay rental creation");
    assert_eq!(replay, first);

    let mut changed = params;
    changed.max_total_microusd += 1;
    let error = runtime
        .create_gpu_rental(&changed, NOW_MS + 2)
        .await
        .expect_err("changed terms must not reuse an authorization identity");
    assert_eq!(
        error.to_string(),
        "client operation id is already bound to different GPU rental terms"
    );
}

#[tokio::test]
async fn expired_quote_never_enters_create_pending() {
    let runtime = runtime().await;
    let params = create_params("op-expiry");
    runtime
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create quoted rental");

    let requested = runtime
        .request_gpu_rental_creation(params.rental_id.as_str(), NOW_MS + 60_001)
        .await
        .expect("request creation");
    assert!(!requested);
    assert_eq!(
        runtime
            .get_gpu_rental(params.rental_id.as_str())
            .await
            .expect("load rental")
            .expect("rental exists")
            .desired_state,
        GpuRentalState::Quoted
    );
}

#[tokio::test]
async fn leases_serialize_two_runtime_instances() {
    let codex_home = unique_temp_dir();
    let first = StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
        .await
        .expect("initialize first runtime");
    let second = StateRuntime::init_for_testing(codex_home, "test-provider".to_string())
        .await
        .expect("initialize second runtime");
    let params = create_params("op-lease");
    first
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create rental");
    first
        .request_gpu_rental_creation(params.rental_id.as_str(), NOW_MS)
        .await
        .expect("request creation");

    let (first_claim, second_claim) = tokio::join!(
        first.claim_due_gpu_rentals(
            "controller-a",
            NOW_MS,
            /*lease_ttl_ms*/ 30_000,
            /*limit*/ 1
        ),
        second.claim_due_gpu_rentals(
            "controller-b",
            NOW_MS,
            /*lease_ttl_ms*/ 30_000,
            /*limit*/ 1
        ),
    );
    let total_claims =
        first_claim.expect("first claim").len() + second_claim.expect("second claim").len();
    assert_eq!(total_claims, 1);
}

#[tokio::test]
async fn provider_controller_only_claims_its_own_rentals() {
    let runtime = runtime().await;
    let fake = create_params("op-fake-provider");
    let mut other = create_params("op-other-provider");
    other.provider = "other".to_string();
    runtime
        .create_gpu_rental(&fake, NOW_MS)
        .await
        .expect("create fake rental");
    runtime
        .create_gpu_rental(&other, NOW_MS)
        .await
        .expect("create other rental");
    runtime
        .request_gpu_rental_creation(fake.rental_id.as_str(), NOW_MS)
        .await
        .expect("request fake creation");
    runtime
        .request_gpu_rental_creation(other.rental_id.as_str(), NOW_MS)
        .await
        .expect("request other creation");

    let fake_claims = runtime
        .claim_due_gpu_rentals_for_provider(
            "fake-controller",
            "fake",
            NOW_MS,
            /*lease_ttl_ms*/ 30_000,
            /*limit*/ 10,
        )
        .await
        .expect("claim fake rentals");
    let other_claims = runtime
        .claim_due_gpu_rentals_for_provider(
            "other-controller",
            "other",
            NOW_MS,
            /*lease_ttl_ms*/ 30_000,
            /*limit*/ 10,
        )
        .await
        .expect("claim other rentals");
    assert_eq!(fake_claims.len(), 1);
    assert_eq!(fake_claims[0].rental.rental_id, fake.rental_id);
    assert_eq!(other_claims.len(), 1);
    assert_eq!(other_claims[0].rental.rental_id, other.rental_id);
}

#[tokio::test]
async fn state_update_is_owned_monotonic_and_releases_lease() {
    let runtime = runtime().await;
    let params = create_params("op-update");
    runtime
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create rental");
    runtime
        .request_gpu_rental_creation(params.rental_id.as_str(), NOW_MS)
        .await
        .expect("request creation");
    let lease = runtime
        .claim_due_gpu_rentals(
            "controller",
            NOW_MS,
            /*lease_ttl_ms*/ 30_000,
            /*limit*/ 1,
        )
        .await
        .expect("claim rental")
        .pop()
        .expect("rental lease");
    let updated = runtime
        .update_gpu_rental(
            &lease,
            &GpuRentalUpdate {
                observed_state: Some(GpuRentalState::Allocating),
                provider_resource_id: Some("provider-123".to_string()),
                next_retry_at_ms: Some(NOW_MS + 2_000),
                clear_last_error: true,
                ..GpuRentalUpdate::default()
            },
            NOW_MS + 1,
        )
        .await
        .expect("update rental");
    assert!(updated);

    let rental = runtime
        .get_gpu_rental(params.rental_id.as_str())
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.observed_state, GpuRentalState::Allocating);
    assert_eq!(rental.provider_resource_id.as_deref(), Some("provider-123"));
    assert_eq!(rental.state_sequence, 3);
    assert_eq!(rental.controller_lease_owner, None);

    assert!(
        !runtime
            .update_gpu_rental(&lease, &GpuRentalUpdate::default(), NOW_MS + 2)
            .await
            .expect("stale update")
    );
}

#[tokio::test]
async fn operation_and_notification_replays_are_deduplicated() {
    let runtime = runtime().await;
    let params = create_params("op-dedup");
    let rental = runtime
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create rental");

    assert!(
        runtime
            .begin_gpu_rental_operation(
                "remote-op-1",
                rental.rental_id.as_str(),
                GpuOperationKind::Create,
                rental.state_sequence,
                NOW_MS,
            )
            .await
            .expect("begin operation")
    );
    assert!(
        !runtime
            .begin_gpu_rental_operation(
                "remote-op-2",
                rental.rental_id.as_str(),
                GpuOperationKind::Create,
                rental.state_sequence,
                NOW_MS + 1,
            )
            .await
            .expect("replay operation")
    );

    assert!(
        runtime
            .record_gpu_notification_once(
                rental.rental_id.as_str(),
                rental.state_sequence,
                "ready",
                NOW_MS,
            )
            .await
            .expect("record notification")
    );
    assert!(
        !runtime
            .record_gpu_notification_once(
                rental.rental_id.as_str(),
                rental.state_sequence,
                "ready",
                NOW_MS + 1,
            )
            .await
            .expect("replay notification")
    );
}
