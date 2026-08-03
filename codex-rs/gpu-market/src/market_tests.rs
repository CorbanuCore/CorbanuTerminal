use super::*;
use crate::BillingState;
use crate::CreateInstanceRequest;
use crate::GpuInstance;
use crate::GpuRecipe;
use crate::HardwareRequirements;
use crate::OwnedInstanceQuery;
use crate::ProviderCapabilities;
use crate::ProviderResult;
use codex_state::GpuRentalState;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

const NOW_MS: i64 = 1_800_000_000_000;

#[derive(Clone)]
struct QuoteProvider {
    name: &'static str,
    price: i64,
    secure_transport: bool,
    atomic_create_handle: bool,
    search_calls: Arc<AtomicUsize>,
    transient_omissions: Arc<AtomicUsize>,
}

impl QuoteProvider {
    fn new(name: &'static str, price: i64) -> Self {
        Self {
            name,
            price,
            secure_transport: true,
            atomic_create_handle: false,
            search_calls: Arc::new(AtomicUsize::new(0)),
            transient_omissions: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn without_secure_transport(mut self) -> Self {
        self.secure_transport = false;
        self
    }

    fn with_transient_omissions(self, count: usize) -> Self {
        self.transient_omissions.store(count, Ordering::SeqCst);
        self
    }

    fn with_atomic_create_handle(mut self) -> Self {
        self.atomic_create_handle = true;
        self
    }
}

impl GpuProvider for QuoteProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider: self.name.to_string(),
            supports_ownership_tags: true,
            supports_inventory: true,
            supports_secure_endpoint_transport: self.secure_transport,
            supports_native_ttl: false,
            supports_native_spend_cap: false,
            security_classes: vec!["secure".to_string()],
        }
    }

    fn create_revalidates_exact_offer_atomically(&self) -> bool {
        self.atomic_create_handle
    }

    async fn search_offers(&self, request: SearchOffersRequest) -> ProviderResult<Vec<GpuOffer>> {
        self.search_calls.fetch_add(1, Ordering::SeqCst);
        if self
            .transient_omissions
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
        {
            return Ok(Vec::new());
        }
        Ok(vec![offer(self.name, self.price, &request)])
    }

    async fn create_instance(
        &self,
        _request: CreateInstanceRequest,
    ) -> ProviderResult<GpuInstance> {
        panic!("confirmation must not perform the billable create")
    }

    async fn get_instance(&self, _resource_id: String) -> ProviderResult<Option<GpuInstance>> {
        Ok(None)
    }

    async fn list_owned_instances(
        &self,
        _query: OwnedInstanceQuery,
    ) -> ProviderResult<Vec<GpuInstance>> {
        Ok(Vec::new())
    }

    async fn terminate_instance(&self, _resource_id: String) -> ProviderResult<()> {
        Ok(())
    }

    async fn billing_state(&self, resource_id: String) -> ProviderResult<BillingState> {
        Ok(BillingState {
            resource_id,
            estimated_accrued_microusd: 0,
            provider_reported_cost_microusd: None,
            still_billable: false,
        })
    }
}

#[tokio::test]
async fn unverified_recipe_fails_before_provider_search() {
    let state = state().await;
    let service = GpuMarketService::new(
        state,
        RecipeCatalog::default(),
        "installation-1".to_string(),
    );
    let first = QuoteProvider::new("first", 2_000_000);
    let second = QuoteProvider::new("second", 1_000_000);
    let error = service
        .search("qwen-32b-1xh200", 3_000_000, &first, &second)
        .await
        .expect_err("unverified recipe must fail closed");
    assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
    assert_eq!(first.search_calls.load(Ordering::SeqCst), 0);
    assert_eq!(second.search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn compatible_offers_are_ranked_only_after_hard_filters() {
    let service = service().await;
    let first = QuoteProvider::new("first", 2_000_000);
    let second = QuoteProvider::new("second", 1_000_000);
    let offers = service
        .search("test-recipe", 3_000_000, &first, &second)
        .await
        .expect("search offers");
    assert_eq!(offers.len(), 2);
    assert_eq!(offers[0].provider, "second");
    assert_eq!(offers[1].provider, "first");
}

#[tokio::test]
async fn insecure_provider_is_excluded_before_inventory_or_spend() {
    let service = service().await;
    let insecure = QuoteProvider::new("insecure", 500_000).without_secure_transport();
    let secure = QuoteProvider::new("secure", 1_000_000);
    let offers = service
        .search("test-recipe", 2_000_000, &insecure, &secure)
        .await
        .expect("secure provider remains available");

    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].provider, "secure");
    assert_eq!(insecure.search_calls.load(Ordering::SeqCst), 0);
    assert_eq!(secure.search_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn two_rate_limited_providers_do_not_masquerade_as_empty_inventory() {
    let first = ProviderError {
        retry_after_ms: Some(5_000),
        ..ProviderError::new(ProviderErrorKind::RateLimited, "first rate limit")
    };
    let second = ProviderError {
        retry_after_ms: Some(2_000),
        ..ProviderError::new(ProviderErrorKind::RateLimited, "second rate limit")
    };
    let error = merge_search_results(Err(first), Err(second)).expect_err("must preserve outage");
    assert_eq!(error.kind, ProviderErrorKind::RateLimited);
    assert_eq!(error.retry_after_ms, Some(2_000));
}

#[test]
fn one_unconfigured_provider_does_not_block_configured_inventory() {
    let offers = vec![offer(
        "configured",
        1_000_000,
        &SearchOffersRequest {
            hardware: hardware(),
            allow_interruptible: false,
            require_verified_or_secure: true,
            maximum_hourly_microusd: 2_000_000,
        },
    )];
    let merged = merge_search_results(
        Err(ProviderError::new(
            ProviderErrorKind::NotConfigured,
            "missing",
        )),
        Ok(offers),
    )
    .expect("configured provider remains usable");
    assert_eq!(merged.len(), 1);
}

#[test]
fn two_unconfigured_providers_return_actionable_error() {
    let error = merge_search_results(
        Err(ProviderError::new(
            ProviderErrorKind::NotConfigured,
            "missing first",
        )),
        Err(ProviderError::new(
            ProviderErrorKind::NotConfigured,
            "missing second",
        )),
    )
    .expect_err("no configured provider must not look like empty capacity");
    assert_eq!(error.kind, ProviderErrorKind::NotConfigured);
    assert!(error.safe_message.contains("/gpu"));
}

#[test]
fn unfunded_provider_is_actionable_when_no_other_provider_has_capacity() {
    let error = merge_search_results(
        Err(ProviderError::new(
            ProviderErrorKind::InsufficientFunds,
            "Fund the configured provider.",
        )),
        Err(ProviderError::new(
            ProviderErrorKind::NotConfigured,
            "missing second",
        )),
    )
    .expect_err("funding failure must not masquerade as empty inventory");

    assert_eq!(error.kind, ProviderErrorKind::InsufficientFunds);
    assert!(error.safe_message.contains("Fund"));
}

#[test]
fn unfunded_provider_does_not_hide_another_providers_offers() {
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let offers = merge_search_results(
        Err(ProviderError::new(
            ProviderErrorKind::InsufficientFunds,
            "Fund the first provider.",
        )),
        Ok(vec![offer("second", 2_000_000, &request)]),
    )
    .expect("a funded provider remains usable");

    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0].provider, "second");
}

#[tokio::test]
async fn confirmation_is_idempotent_and_never_calls_provider_create() {
    let service = service().await;
    let provider = QuoteProvider::new("first", 2_000_000);
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let selected = offer("first", 2_000_000, &request);
    let authorization = RentalAuthorization {
        client_operation_id: "operation-1".to_string(),
        maximum_hourly_microusd: 3_000_000,
        maximum_total_microusd: 12_000_000,
        terminate_at_ms: NOW_MS + 3_600_000,
        acknowledged_local_enforcement: true,
    };
    let first = service
        .confirm("test-recipe", &selected, &authorization, &provider, NOW_MS)
        .await
        .expect("confirm rental");
    let replay = service
        .confirm(
            "test-recipe",
            &selected,
            &authorization,
            &provider,
            NOW_MS + 1,
        )
        .await
        .expect("replay confirmation");
    assert_eq!(first.rental_id, replay.rental_id);
    assert_eq!(replay.desired_state, GpuRentalState::CreatePending);
    assert_eq!(replay.observed_state, GpuRentalState::Quoted);
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn recorded_confirmation_replay_survives_expired_authorization() {
    let service = service().await;
    let provider = QuoteProvider::new("first", 2_000_000);
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let authorization = RentalAuthorization {
        client_operation_id: "expired-replay".to_string(),
        maximum_hourly_microusd: 3_000_000,
        maximum_total_microusd: 12_000_000,
        terminate_at_ms: NOW_MS + 1_000,
        acknowledged_local_enforcement: true,
    };
    let created = service
        .confirm(
            "test-recipe",
            &offer("first", 2_000_000, &request),
            &authorization,
            &provider,
            NOW_MS,
        )
        .await
        .expect("initial confirmation");

    let replay = service
        .confirm(
            "test-recipe",
            &offer("first", 2_000_000, &request),
            &authorization,
            &provider,
            NOW_MS + 2_000,
        )
        .await
        .expect("durable replay must return the recorded rental");

    assert_eq!(replay.rental_id, created.rental_id);
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn new_expired_authorization_has_actionable_recovery() {
    let service = service().await;
    let provider = QuoteProvider::new("first", 2_000_000);
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let error = service
        .confirm(
            "test-recipe",
            &offer("first", 2_000_000, &request),
            &RentalAuthorization {
                client_operation_id: "new-expired".to_string(),
                maximum_hourly_microusd: 3_000_000,
                maximum_total_microusd: 12_000_000,
                terminate_at_ms: NOW_MS,
                acknowledged_local_enforcement: true,
            },
            &provider,
            NOW_MS,
        )
        .await
        .expect_err("a genuinely new expired authorization must be rejected");

    assert!(error.safe_message.contains("expired before confirmation"));
    assert!(error.safe_message.contains("/gpu"));
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn confirmation_tolerates_bounded_transient_offer_inventory_omissions() {
    let service = service().await;
    let provider = QuoteProvider::new("first", 2_000_000).with_transient_omissions(2);
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let rental = service
        .confirm(
            "test-recipe",
            &offer("first", 2_000_000, &request),
            &RentalAuthorization {
                client_operation_id: "transient-inventory".to_string(),
                maximum_hourly_microusd: 3_000_000,
                maximum_total_microusd: 12_000_000,
                terminate_at_ms: NOW_MS + 3_600_000,
                acknowledged_local_enforcement: true,
            },
            &provider,
            NOW_MS,
        )
        .await
        .expect("bounded inventory omission should recover");

    assert_eq!(rental.desired_state, GpuRentalState::CreatePending);
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn atomic_offer_handle_skips_rotating_inventory_revalidation() {
    let service = service().await;
    let provider = QuoteProvider::new("first", 2_000_000).with_atomic_create_handle();
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let rental = service
        .confirm(
            "test-recipe",
            &offer("first", 2_000_000, &request),
            &RentalAuthorization {
                client_operation_id: "atomic-offer-handle".to_string(),
                maximum_hourly_microusd: 3_000_000,
                maximum_total_microusd: 12_000_000,
                terminate_at_ms: NOW_MS + 3_600_000,
                acknowledged_local_enforcement: true,
            },
            &provider,
            NOW_MS,
        )
        .await
        .expect("atomic offer handle should persist without broad re-query");

    assert_eq!(rental.desired_state, GpuRentalState::CreatePending);
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn confirmation_rejects_unqualified_transport_before_provider_search() {
    let service = service().await;
    let provider = QuoteProvider::new("insecure", 1_000_000).without_secure_transport();
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 2_000_000,
    };
    let error = service
        .confirm(
            "test-recipe",
            &offer("insecure", 1_000_000, &request),
            &RentalAuthorization {
                client_operation_id: "unqualified-transport".to_string(),
                maximum_hourly_microusd: 2_000_000,
                maximum_total_microusd: 4_000_000,
                terminate_at_ms: NOW_MS + 3_600_000,
                acknowledged_local_enforcement: true,
            },
            &provider,
            NOW_MS,
        )
        .await
        .expect_err("unqualified transport must fail before authorization persists");

    assert_eq!(error.kind, ProviderErrorKind::CapabilityUnavailable);
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn local_enforcement_requires_explicit_acknowledgement() {
    let service = service().await;
    let provider = QuoteProvider::new("first", 2_000_000);
    let request = SearchOffersRequest {
        hardware: hardware(),
        allow_interruptible: false,
        require_verified_or_secure: true,
        maximum_hourly_microusd: 3_000_000,
    };
    let error = service
        .confirm(
            "test-recipe",
            &offer("first", 2_000_000, &request),
            &RentalAuthorization {
                client_operation_id: "operation-no-ack".to_string(),
                maximum_hourly_microusd: 3_000_000,
                maximum_total_microusd: 12_000_000,
                terminate_at_ms: NOW_MS + 3_600_000,
                acknowledged_local_enforcement: false,
            },
            &provider,
            NOW_MS,
        )
        .await
        .expect_err("local enforcement acknowledgement is mandatory");
    assert_eq!(error.kind, ProviderErrorKind::InvalidRequest);
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 0);
}

async fn state() -> Arc<StateRuntime> {
    let path = std::env::temp_dir().join(format!("gpu-market-test-{}", uuid::Uuid::new_v4()));
    let sqlite = codex_state::SqliteConfig::new_for_testing(
        path.try_into().expect("absolute temporary state path"),
    );
    StateRuntime::init(sqlite, "test-provider".to_string())
        .await
        .expect("initialize state")
}

async fn service() -> GpuMarketService {
    GpuMarketService::new(
        state().await,
        RecipeCatalog::new(vec![recipe()]).expect("recipe catalog"),
        "installation-1".to_string(),
    )
}

fn recipe() -> GpuRecipe {
    GpuRecipe {
        id: "test-recipe".to_string(),
        revision: "manifest-v1".to_string(),
        model_id: "test/model".to_string(),
        served_model_id: "test/model".to_string(),
        wire_api: "chat".to_string(),
        model_revision: "1111111111111111111111111111111111111111".to_string(),
        image: "test/image@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        runtime: "test-runtime".to_string(),
        serving_runtime_version: "1.0.0".to_string(),
        license_id: "apache-2.0".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "550.0".to_string(),
        gpu_architectures: vec!["sm90".to_string()],
        weight_format: "fp8".to_string(),
        hardware: hardware(),
        tensor_parallel_size: 2,
        maximum_context_tokens: 32_768,
        maximum_concurrent_requests: 2,
        expected_download_bytes: 1_000_000,
        model_weight_bytes: 100_000_000_000,
        kv_cache_reserve_bytes: 20_000_000_000,
        workspace_reserve_bytes: 10_000_000_000,
        launch_command: vec![
            "server".to_string(),
            "test/model".to_string(),
            "1111111111111111111111111111111111111111".to_string(),
            "nvidia-smi topo -m; printf PFTERMINAL_RUNTIME_GATE=nvlink-ok".to_string(),
            "--api-key".to_string(),
            "$PFT_ENDPOINT_TOKEN".to_string(),
        ],
        environment_allowlist: vec!["PFT_ENDPOINT_TOKEN".to_string()],
        startup_deadline_ms: 120_000,
        download_deadline_ms: 3_600_000,
        probe_deadline_ms: 60_000,
        inference_port: 8000,
        chat_encoding: "test-encoding".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: crate::RecipeStability::Qualified,
        manifest_verified: true,
    }
}

fn hardware() -> HardwareRequirements {
    HardwareRequirements {
        gpu_model: "NVIDIA H200".to_string(),
        gpu_count: 2,
        minimum_vram_mib_per_gpu: 130_000,
        minimum_host_ram_mib: 128_000,
        minimum_disk_gib: 400,
        requires_high_bandwidth_interconnect: true,
        allowed_cuda_versions: Vec::new(),
    }
}

fn offer(provider: &str, price: i64, request: &SearchOffersRequest) -> GpuOffer {
    GpuOffer {
        provider: provider.to_string(),
        offer_id: format!("{provider}-offer"),
        gpu_model: request.hardware.gpu_model.clone(),
        gpu_count: request.hardware.gpu_count,
        vram_mib_per_gpu: request.hardware.minimum_vram_mib_per_gpu,
        host_ram_mib: request.hardware.minimum_host_ram_mib,
        disk_gib: request.hardware.minimum_disk_gib,
        high_bandwidth_interconnect: request.hardware.requires_high_bandwidth_interconnect,
        runtime_topology_verification: false,
        cuda_versions: request.hardware.allowed_cuda_versions.clone(),
        region: "test".to_string(),
        security_class: "secure".to_string(),
        reliability_millionths: Some(999_000),
        interruptible: false,
        hourly_microusd: price,
        storage_microusd_per_gib_month: Some(10_000),
        quoted_at_ms: NOW_MS,
        expires_at_ms: Some(NOW_MS + 60_000),
        raw_snapshot: serde_json::json!({"provider": provider}),
    }
}
