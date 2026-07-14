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
    search_calls: Arc<AtomicUsize>,
}

impl QuoteProvider {
    fn new(name: &'static str, price: i64) -> Self {
        Self {
            name,
            price,
            search_calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl GpuProvider for QuoteProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider: self.name.to_string(),
            supports_ownership_tags: true,
            supports_inventory: true,
            supports_native_ttl: false,
            supports_native_spend_cap: false,
            security_classes: vec!["secure".to_string()],
        }
    }

    async fn search_offers(&self, request: SearchOffersRequest) -> ProviderResult<Vec<GpuOffer>> {
        self.search_calls.fetch_add(1, Ordering::SeqCst);
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
        .search("deepseek-flash-2xh200", 3_000_000, &first, &second)
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
    assert_eq!(provider.search_calls.load(Ordering::SeqCst), 2);
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
    StateRuntime::init(path, "test-provider".to_string())
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
        revision: "sha256:test".to_string(),
        model_id: "test/model".to_string(),
        model_revision: "sha256:model".to_string(),
        image: "test/image@sha256:digest".to_string(),
        runtime: "test-runtime".to_string(),
        hardware: hardware(),
        tensor_parallel_size: 2,
        maximum_context_tokens: 32_768,
        maximum_concurrent_requests: 2,
        expected_download_bytes: 1_000_000,
        launch_arguments: vec!["--tensor-parallel-size=2".to_string()],
        chat_encoding: "test-encoding".to_string(),
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
