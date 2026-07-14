use super::*;
use crate::BillingState;
use crate::CreateInstanceRequest;
use crate::GpuCredential;
use crate::GpuCredentialError;
use crate::GpuCredentialKind;
use crate::GpuCredentialResolver;
use crate::GpuInstanceState;
use crate::GpuOffer;
use crate::GpuProvider;
use crate::GpuRecipe;
use crate::HardwareRequirements;
use crate::OwnedInstanceQuery;
use crate::ProviderCapabilities;
use crate::ProviderError;
use crate::ProviderErrorKind;
use crate::ProviderResult;
use crate::SearchOffersRequest;
use crate::SecretValue;
use codex_state::GpuLimitEnforcement;
use codex_state::GpuRentalCreateParams;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const NOW_MS: i64 = 1_800_000_000_000;

#[derive(Debug)]
struct FakeCredentials;

impl GpuCredentialResolver for FakeCredentials {
    fn resolve(&self, kind: &GpuCredentialKind) -> Result<GpuCredential, GpuCredentialError> {
        match kind {
            GpuCredentialKind::RentalEndpointToken { .. } => Ok(GpuCredential {
                label: kind.canonical_label()?,
                secret: SecretValue::new("fake-rental-endpoint-token".to_string())?,
            }),
            GpuCredentialKind::HuggingFaceToken => Err(GpuCredentialError::Missing),
            GpuCredentialKind::ProviderApiKey { .. } => Err(GpuCredentialError::Missing),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CreateBehavior {
    Success,
    AmbiguousAfterCreate,
    AmbiguousWithoutCreate,
}

#[derive(Debug)]
struct FakeState {
    create_behavior: CreateBehavior,
    ambiguous_termination: bool,
    create_calls: usize,
    terminate_calls: usize,
    instances: HashMap<String, GpuInstance>,
}

#[derive(Debug, Clone)]
struct FakeProvider {
    state: Arc<Mutex<FakeState>>,
}

impl FakeProvider {
    fn new(create_behavior: CreateBehavior) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeState {
                create_behavior,
                ambiguous_termination: false,
                create_calls: 0,
                terminate_calls: 0,
                instances: HashMap::new(),
            })),
        }
    }

    async fn make_termination_ambiguous(&self) {
        self.state.lock().await.ambiguous_termination = true;
    }

    async fn create_calls(&self) -> usize {
        self.state.lock().await.create_calls
    }

    async fn terminate_calls(&self) -> usize {
        self.state.lock().await.terminate_calls
    }

    async fn set_instance_state(&self, resource_id: &str, instance_state: GpuInstanceState) {
        self.state
            .lock()
            .await
            .instances
            .get_mut(resource_id)
            .expect("fake instance exists")
            .state = instance_state;
    }
}

impl GpuProvider for FakeProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider: "fake".to_string(),
            supports_ownership_tags: true,
            supports_inventory: true,
            supports_native_ttl: false,
            supports_native_spend_cap: false,
            security_classes: vec!["secure".to_string()],
        }
    }

    async fn search_offers(&self, _request: SearchOffersRequest) -> ProviderResult<Vec<GpuOffer>> {
        Ok(vec![offer()])
    }

    async fn create_instance(&self, request: CreateInstanceRequest) -> ProviderResult<GpuInstance> {
        let mut state = self.state.lock().await;
        state.create_calls += 1;
        let instance = GpuInstance {
            provider: "fake".to_string(),
            resource_id: format!("resource-{}", state.create_calls),
            ownership_tag: request.ownership_tag,
            state: GpuInstanceState::Allocating,
            gpu_model: request.offer.gpu_model,
            gpu_count: request.offer.gpu_count,
            hourly_microusd: request.offer.hourly_microusd,
            created_at_ms: Some(NOW_MS),
            public_ip: None,
            ssh_port: None,
        };
        match state.create_behavior {
            CreateBehavior::Success => {
                state
                    .instances
                    .insert(instance.resource_id.clone(), instance.clone());
                Ok(instance)
            }
            CreateBehavior::AmbiguousAfterCreate => {
                state
                    .instances
                    .insert(instance.resource_id.clone(), instance);
                Err(ProviderError::new(
                    ProviderErrorKind::Ambiguous,
                    "create response was lost",
                ))
            }
            CreateBehavior::AmbiguousWithoutCreate => Err(ProviderError::new(
                ProviderErrorKind::Ambiguous,
                "create response was lost",
            )),
        }
    }

    async fn get_instance(&self, resource_id: String) -> ProviderResult<Option<GpuInstance>> {
        Ok(self.state.lock().await.instances.get(&resource_id).cloned())
    }

    async fn list_owned_instances(
        &self,
        query: OwnedInstanceQuery,
    ) -> ProviderResult<Vec<GpuInstance>> {
        Ok(self
            .state
            .lock()
            .await
            .instances
            .values()
            .filter(|instance| {
                query
                    .ownership_tag
                    .as_ref()
                    .is_none_or(|tag| instance.ownership_tag == *tag)
            })
            .cloned()
            .collect())
    }

    async fn terminate_instance(&self, resource_id: String) -> ProviderResult<()> {
        let mut state = self.state.lock().await;
        state.terminate_calls += 1;
        state.instances.remove(&resource_id);
        if state.ambiguous_termination {
            Err(ProviderError::new(
                ProviderErrorKind::Ambiguous,
                "termination response was lost",
            ))
        } else {
            Ok(())
        }
    }

    async fn billing_state(&self, resource_id: String) -> ProviderResult<BillingState> {
        let state = self.state.lock().await;
        Ok(BillingState {
            resource_id: resource_id.clone(),
            estimated_accrued_microusd: 500_000,
            provider_reported_cost_microusd: None,
            still_billable: state.instances.contains_key(&resource_id),
        })
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

fn offer() -> GpuOffer {
    GpuOffer {
        provider: "fake".to_string(),
        offer_id: "offer-1".to_string(),
        gpu_model: "NVIDIA H200".to_string(),
        gpu_count: 2,
        vram_mib_per_gpu: 141_000,
        host_ram_mib: 256_000,
        disk_gib: 800,
        high_bandwidth_interconnect: true,
        cuda_versions: Vec::new(),
        region: "test".to_string(),
        security_class: "secure".to_string(),
        reliability_millionths: Some(999_000),
        interruptible: false,
        hourly_microusd: 2_500_000,
        storage_microusd_per_gib_month: Some(10_000),
        quoted_at_ms: NOW_MS,
        expires_at_ms: Some(NOW_MS + 60_000),
        raw_snapshot: serde_json::json!({"source": "fake"}),
    }
}

fn recipe_catalog() -> RecipeCatalog {
    RecipeCatalog::new(vec![GpuRecipe {
        id: "deepseek-flash-2xh200".to_string(),
        revision: "test-revision".to_string(),
        model_id: "deepseek-ai/DeepSeek-V4-Flash".to_string(),
        model_revision: "test-model-revision".to_string(),
        image: "vllm:test-digest".to_string(),
        runtime: "vllm".to_string(),
        hardware: hardware(),
        tensor_parallel_size: 2,
        maximum_context_tokens: 384_000,
        maximum_concurrent_requests: 2,
        expected_download_bytes: 180_000_000_000,
        launch_arguments: vec!["--tensor-parallel-size=2".to_string()],
        chat_encoding: "deepseek-v4-encoding".to_string(),
        manifest_verified: true,
    }])
    .expect("valid test recipe")
}

fn rental_params(operation_id: &str) -> GpuRentalCreateParams {
    GpuRentalCreateParams {
        rental_id: format!("rental-{operation_id}"),
        installation_id: "installation-1".to_string(),
        client_operation_id: operation_id.to_string(),
        provider: "fake".to_string(),
        recipe_id: "deepseek-flash-2xh200".to_string(),
        recipe_revision: "test-revision".to_string(),
        offer_snapshot_json: serde_json::to_string(&offer()).expect("serialize offer"),
        quote_expires_at_ms: Some(NOW_MS + 60_000),
        max_hourly_microusd: 3_000_000,
        max_total_microusd: 12_000_000,
        terminate_at_ms: NOW_MS + 4 * 60 * 60 * 1000,
        enforcement_class: GpuLimitEnforcement::LocalControllerDependent,
        ownership_tag: format!("pft-installation-1-{operation_id}"),
    }
}

async fn state_runtime() -> Arc<StateRuntime> {
    let path = std::env::temp_dir().join(format!("gpu-controller-test-{}", uuid::Uuid::new_v4()));
    StateRuntime::init(path, "test-provider".to_string())
        .await
        .expect("initialize state")
}

fn controller(
    state: Arc<StateRuntime>,
    provider: FakeProvider,
) -> GpuRentalController<FakeProvider> {
    GpuRentalController::new_with_credentials(
        state,
        provider,
        recipe_catalog(),
        "installation-1".to_string(),
        ReconcileConfig {
            controller_id: format!("controller-{}", uuid::Uuid::new_v4()),
            lease_ttl_ms: 10_000,
            normal_poll_ms: 1,
            maximum_retry_ms: 10_000,
            batch_size: 4,
        },
        Arc::new(FakeCredentials),
    )
}

async fn create_authorized_rental(state: &StateRuntime, operation_id: &str) {
    let params = rental_params(operation_id);
    state
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create rental");
    assert!(
        state
            .request_gpu_rental_creation(params.rental_id.as_str(), NOW_MS)
            .await
            .expect("authorize creation")
    );
}

#[tokio::test]
async fn ordinary_create_records_one_provider_resource() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "ordinary").await;
    let events = controller(state.clone(), provider.clone())
        .reconcile_due(NOW_MS)
        .await
        .expect("reconcile create");

    assert_eq!(provider.create_calls().await, 1);
    assert_eq!(events.len(), 1);
    let rental = state
        .get_gpu_rental("rental-ordinary")
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.observed_state, GpuRentalState::Allocating);
    assert_eq!(rental.provider_resource_id.as_deref(), Some("resource-1"));
}

#[tokio::test]
async fn offer_price_drift_is_persisted_without_provider_create() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let mut params = rental_params("price-drift");
    params.max_hourly_microusd = 2_000_000;
    state
        .create_gpu_rental(&params, NOW_MS)
        .await
        .expect("create rental");
    state
        .request_gpu_rental_creation(params.rental_id.as_str(), NOW_MS)
        .await
        .expect("authorize creation");

    controller(state.clone(), provider.clone())
        .reconcile_due(NOW_MS)
        .await
        .expect("reject drifted offer");

    assert_eq!(provider.create_calls().await, 0);
    let rental = state
        .get_gpu_rental(params.rental_id.as_str())
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.observed_state, GpuRentalState::Failed);
    assert_eq!(rental.last_error_code.as_deref(), Some("offer-invalid"));
}

#[tokio::test]
async fn ambiguous_create_adopts_inventory_without_duplicate() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::AmbiguousAfterCreate);
    create_authorized_rental(&state, "ambiguous-adopt").await;
    let controller = controller(state.clone(), provider.clone());
    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("first reconcile");
    controller
        .reconcile_due(NOW_MS + 2_000)
        .await
        .expect("inventory reconcile");

    assert_eq!(provider.create_calls().await, 1);
    let rental = state
        .get_gpu_rental("rental-ambiguous-adopt")
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.observed_state, GpuRentalState::Allocating);
    assert_eq!(rental.provider_resource_id.as_deref(), Some("resource-1"));
}

#[tokio::test]
async fn unresolved_ambiguous_create_never_retries_creation() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::AmbiguousWithoutCreate);
    create_authorized_rental(&state, "ambiguous-empty").await;
    let controller = controller(state.clone(), provider.clone());
    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("first reconcile");
    let events = controller
        .reconcile_due(NOW_MS + 2_000)
        .await
        .expect("inventory reconcile");

    assert_eq!(provider.create_calls().await, 1);
    assert_eq!(events.len(), 1);
    assert_eq!(
        state
            .get_gpu_rental("rental-ambiguous-empty")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::Orphaned
    );
}

#[tokio::test]
async fn termination_is_confirmed_only_after_inventory_absence() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "terminate").await;
    let controller = controller(state.clone(), provider.clone());
    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("create instance");
    state
        .request_gpu_rental_termination("rental-terminate", NOW_MS + 1)
        .await
        .expect("request termination");
    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("send termination");
    assert_eq!(provider.terminate_calls().await, 1);
    assert_eq!(
        state
            .get_gpu_rental("rental-terminate")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::Terminating
    );

    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("confirm absence");
    assert_eq!(
        state
            .get_gpu_rental("rental-terminate")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::TerminatedConfirmed
    );
}

#[tokio::test]
async fn ambiguous_termination_is_reconciled_by_inventory_without_duplicate_delete() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "ambiguous-terminate").await;
    let controller = controller(state.clone(), provider.clone());
    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("create instance");
    provider.make_termination_ambiguous().await;
    state
        .request_gpu_rental_termination("rental-ambiguous-terminate", NOW_MS + 1)
        .await
        .expect("request termination");

    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("send ambiguous termination");
    assert_eq!(provider.terminate_calls().await, 1);
    assert_eq!(
        state
            .get_gpu_rental("rental-ambiguous-terminate")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::TerminationUnconfirmed
    );

    controller
        .reconcile_due(NOW_MS + 2_000)
        .await
        .expect("confirm inventory absence");
    assert_eq!(provider.terminate_calls().await, 1);
    assert_eq!(
        state
            .get_gpu_rental("rental-ambiguous-terminate")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::TerminatedConfirmed
    );
}

#[tokio::test]
async fn provider_failure_with_known_resource_enters_cleanup_instead_of_failed() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "provider-failed").await;
    let controller = controller(state.clone(), provider.clone());
    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("create instance");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Failed)
        .await;

    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("observe provider failure");

    let rental = state
        .get_gpu_rental("rental-provider-failed")
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.desired_state, GpuRentalState::TerminateRequested);
    assert_eq!(rental.observed_state, GpuRentalState::TerminateRequested);
    assert!(rental.may_be_billable());
}

#[tokio::test]
async fn ttl_boundary_transitions_atomically_under_the_controller_lease() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "ttl").await;
    let controller = controller(state.clone(), provider);
    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("create instance");
    let terminate_at_ms = state
        .get_gpu_rental("rental-ttl")
        .await
        .expect("load rental")
        .expect("rental exists")
        .terminate_at_ms;

    let events = controller
        .reconcile_due(terminate_at_ms)
        .await
        .expect("enforce ttl");

    assert!(
        matches!(events.as_slice(), [ControllerEvent::Warning { code, .. }] if code == "spend-limit")
    );
    let rental = state
        .get_gpu_rental("rental-ttl")
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.desired_state, GpuRentalState::TerminateRequested);
    assert_eq!(rental.observed_state, GpuRentalState::TerminateRequested);
}

#[tokio::test]
async fn retry_backoff_is_stable_jittered_and_honors_retry_after() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "retry-jitter").await;
    let controller = controller(state.clone(), provider);
    let rental = state
        .get_gpu_rental("rental-retry-jitter")
        .await
        .expect("load rental")
        .expect("rental exists");

    let first = controller.retry_delay_ms(&rental, None);
    let replay = controller.retry_delay_ms(&rental, None);
    assert_eq!(first, replay);
    assert!((750..=1_250).contains(&first));
    assert_eq!(controller.retry_delay_ms(&rental, Some(7_777)), 7_777);
    assert_eq!(controller.retry_delay_ms(&rental, Some(90_000)), 10_000);
}
