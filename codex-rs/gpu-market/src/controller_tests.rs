use super::*;
use crate::BillingState;
use crate::CreateInstanceRequest;
use crate::EndpointReadinessReport;
use crate::GpuCredential;
use crate::GpuCredentialError;
use crate::GpuCredentialKind;
use crate::GpuCredentialResolver;
use crate::GpuInstanceState;
use crate::GpuOffer;
use crate::GpuProvider;
use crate::GpuReadinessProbe;
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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
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

#[derive(Debug)]
struct TransientEndpointCredentials {
    fail_on_call: usize,
    calls: AtomicUsize,
}

impl TransientEndpointCredentials {
    fn unavailable_on_call(fail_on_call: usize) -> Self {
        Self {
            fail_on_call,
            calls: AtomicUsize::new(0),
        }
    }
}

impl GpuCredentialResolver for TransientEndpointCredentials {
    fn resolve(&self, kind: &GpuCredentialKind) -> Result<GpuCredential, GpuCredentialError> {
        FakeCredentials.resolve(kind)
    }

    fn ensure_rental_endpoint_token(
        &self,
        rental_id: &str,
    ) -> Result<GpuCredential, GpuCredentialError> {
        if self.calls.fetch_add(1, Ordering::SeqCst) + 1 == self.fail_on_call {
            return Err(GpuCredentialError::StoreUnavailable);
        }
        self.resolve(&GpuCredentialKind::RentalEndpointToken {
            rental_id: rental_id.to_string(),
        })
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
    ambiguous_get: bool,
    secure_endpoint_error: Option<ProviderError>,
    secure_endpoint_override: Option<String>,
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
                ambiguous_get: false,
                secure_endpoint_error: None,
                secure_endpoint_override: None,
                create_calls: 0,
                terminate_calls: 0,
                instances: HashMap::new(),
            })),
        }
    }

    async fn make_termination_ambiguous(&self) {
        self.state.lock().await.ambiguous_termination = true;
    }

    async fn make_get_ambiguous(&self) {
        self.state.lock().await.ambiguous_get = true;
    }

    async fn set_secure_endpoint_error(&self, error: Option<ProviderError>) {
        self.state.lock().await.secure_endpoint_error = error;
    }

    async fn set_secure_endpoint_override(&self, endpoint: Option<String>) {
        self.state.lock().await.secure_endpoint_override = endpoint;
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

    async fn remove_instance(&self, resource_id: &str) {
        self.state.lock().await.instances.remove(resource_id);
    }
}

impl GpuProvider for FakeProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider: "fake".to_string(),
            supports_ownership_tags: true,
            supports_inventory: true,
            supports_secure_endpoint_transport: true,
            supports_native_ttl: false,
            supports_native_spend_cap: false,
            security_classes: vec!["secure".to_string()],
        }
    }

    async fn secure_endpoint_base_url(
        &self,
        instance: &GpuInstance,
        _inference_port: u16,
    ) -> ProviderResult<String> {
        let state = self.state.lock().await;
        if let Some(error) = state.secure_endpoint_error.clone() {
            return Err(error);
        }
        if let Some(endpoint) = state.secure_endpoint_override.clone() {
            return Ok(endpoint);
        }
        Ok(format!(
            "https://{}.example.invalid/v1",
            instance.resource_id
        ))
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
            host_ram_mib: Some(request.offer.host_ram_mib),
            disk_gib: Some(request.disk_gib),
            high_bandwidth_interconnect: Some(request.offer.high_bandwidth_interconnect),
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
        let state = self.state.lock().await;
        if state.ambiguous_get {
            return Err(ProviderError::new(
                ProviderErrorKind::Ambiguous,
                "instance views are eventually consistent",
            ));
        }
        Ok(state.instances.get(&resource_id).cloned())
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
        if state.ambiguous_get {
            return Err(ProviderError::new(
                ProviderErrorKind::Ambiguous,
                "billing lookup is eventually consistent",
            ));
        }
        Ok(BillingState {
            resource_id: resource_id.clone(),
            estimated_accrued_microusd: 500_000,
            provider_reported_cost_microusd: None,
            still_billable: state.instances.contains_key(&resource_id),
        })
    }
}

#[derive(Debug)]
struct FakeReadiness;

impl GpuReadinessProbe for FakeReadiness {
    fn probe<'a>(
        &'a self,
        _base_url: &'a str,
        _model_id: &'a str,
        _token: &'a SecretValue,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<EndpointReadinessReport>> + Send + 'a>,
    > {
        Box::pin(async {
            Ok(EndpointReadinessReport {
                rejects_missing_token: true,
                rejects_wrong_token: true,
                model_identity_ok: true,
                chat_ok: true,
                streaming_ok: true,
                cancellation_ok: true,
                tool_call_ok: true,
            })
        })
    }
}

#[derive(Debug)]
struct SwitchableReadiness {
    ready: AtomicBool,
}

impl SwitchableReadiness {
    fn new(ready: bool) -> Self {
        Self {
            ready: AtomicBool::new(ready),
        }
    }

    fn set(&self, ready: bool) {
        self.ready.store(ready, Ordering::SeqCst);
    }
}

impl GpuReadinessProbe for SwitchableReadiness {
    fn probe<'a>(
        &'a self,
        _base_url: &'a str,
        _model_id: &'a str,
        _token: &'a SecretValue,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<EndpointReadinessReport>> + Send + 'a>,
    > {
        let ready = self.ready.load(Ordering::SeqCst);
        Box::pin(async move {
            Ok(EndpointReadinessReport {
                rejects_missing_token: ready,
                rejects_wrong_token: ready,
                model_identity_ok: ready,
                chat_ok: ready,
                streaming_ok: ready,
                cancellation_ok: ready,
                tool_call_ok: ready,
            })
        })
    }
}

#[derive(Debug)]
struct SaturatedReadiness;

impl GpuReadinessProbe for SaturatedReadiness {
    fn probe<'a>(
        &'a self,
        _base_url: &'a str,
        _model_id: &'a str,
        _token: &'a SecretValue,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<EndpointReadinessReport>> + Send + 'a>,
    > {
        Box::pin(async {
            Ok(EndpointReadinessReport {
                rejects_missing_token: true,
                rejects_wrong_token: true,
                model_identity_ok: true,
                chat_ok: false,
                streaming_ok: false,
                cancellation_ok: false,
                tool_call_ok: false,
            })
        })
    }

    fn probe_health<'a>(
        &'a self,
        _base_url: &'a str,
        _model_id: &'a str,
        _token: &'a SecretValue,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + Send + 'a>>
    {
        Box::pin(async { Ok(true) })
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
        runtime_topology_verification: false,
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
        served_model_id: "deepseek-v4-flash".to_string(),
        wire_api: "responses".to_string(),
        model_revision: "1111111111111111111111111111111111111111".to_string(),
        image:
            "vllm/runtime@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
        runtime: "vllm".to_string(),
        serving_runtime_version: "1.0.0".to_string(),
        license_id: "model-license".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "550.0".to_string(),
        gpu_architectures: vec!["sm90".to_string()],
        weight_format: "fp8".to_string(),
        hardware: hardware(),
        tensor_parallel_size: 2,
        maximum_context_tokens: 384_000,
        maximum_concurrent_requests: 2,
        expected_download_bytes: 180_000_000_000,
        model_weight_bytes: 180_000_000_000,
        kv_cache_reserve_bytes: 40_000_000_000,
        workspace_reserve_bytes: 20_000_000_000,
        launch_command: vec![
            "server".to_string(),
            "deepseek-ai/DeepSeek-V4-Flash".to_string(),
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
        chat_encoding: "deepseek-v4-encoding".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: crate::RecipeStability::Qualified,
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
    controller_with_readiness(state, provider, Arc::new(FakeReadiness))
}

fn controller_with_readiness(
    state: Arc<StateRuntime>,
    provider: FakeProvider,
    readiness: Arc<dyn GpuReadinessProbe>,
) -> GpuRentalController<FakeProvider> {
    controller_with_dependencies(state, provider, Arc::new(FakeCredentials), readiness)
}

fn controller_with_dependencies(
    state: Arc<StateRuntime>,
    provider: FakeProvider,
    credentials: Arc<dyn GpuCredentialResolver>,
    readiness: Arc<dyn GpuReadinessProbe>,
) -> GpuRentalController<FakeProvider> {
    GpuRentalController::new_with_runtime(
        state,
        provider,
        recipe_catalog(),
        "installation-1".to_string(),
        ReconcileConfig {
            controller_id: format!("controller-{}", uuid::Uuid::new_v4()),
            lease_ttl_ms: 10_000,
            normal_poll_ms: 1,
            maximum_retry_ms: 10_000,
            health_poll_ms: 60_000,
            batch_size: 4,
        },
        credentials,
        readiness,
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
async fn transient_endpoint_store_failure_retries_before_provider_create() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "credential-retry").await;
    let controller = controller_with_dependencies(
        state.clone(),
        provider.clone(),
        Arc::new(TransientEndpointCredentials::unavailable_on_call(1)),
        Arc::new(FakeReadiness),
    );

    controller
        .reconcile_due(NOW_MS)
        .await
        .expect("record transient credential failure");

    assert_eq!(provider.create_calls().await, 0);
    let retry_at_ms = state
        .get_gpu_rental("rental-credential-retry")
        .await
        .expect("load rental")
        .expect("rental exists")
        .next_retry_at_ms;
    controller
        .reconcile_due(retry_at_ms)
        .await
        .expect("retry provider create");

    assert_eq!(provider.create_calls().await, 1);
    assert_eq!(
        state
            .get_gpu_rental("rental-credential-retry")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::Allocating
    );
}

#[tokio::test]
async fn transient_endpoint_store_failure_during_readiness_preserves_live_instance() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let controller = controller_with_dependencies(
        state.clone(),
        provider.clone(),
        Arc::new(TransientEndpointCredentials::unavailable_on_call(2)),
        Arc::new(FakeReadiness),
    );
    create_authorized_rental(&state, "readiness-credential-retry").await;

    controller.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Running)
        .await;
    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("observe running");
    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("verify bootstrap");
    controller
        .reconcile_due(NOW_MS + 3)
        .await
        .expect("record transient readiness credential failure");

    let rental = state
        .get_gpu_rental("rental-readiness-credential-retry")
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_eq!(rental.observed_state, GpuRentalState::Probing);
    assert_ne!(rental.desired_state, GpuRentalState::TerminateRequested);
    assert_eq!(provider.create_calls().await, 1);
    assert_eq!(provider.terminate_calls().await, 0);

    controller
        .reconcile_due(rental.next_retry_at_ms)
        .await
        .expect("retry authenticated readiness");
    assert_eq!(
        state
            .get_gpu_rental("rental-readiness-credential-retry")
            .await
            .expect("load rental")
            .expect("rental exists")
            .observed_state,
        GpuRentalState::Ready
    );
}

#[tokio::test]
async fn provider_native_bootstrap_reaches_ready_and_registers_runtime_once() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let controller = controller(state.clone(), provider.clone());
    create_authorized_rental(&state, "ready").await;

    controller.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Running)
        .await;
    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("observe running");
    let bootstrapping = state
        .get_gpu_rental("rental-ready")
        .await
        .expect("load bootstrapping rental")
        .expect("rental");
    assert_eq!(bootstrapping.observed_state, GpuRentalState::Bootstrapping);
    assert_eq!(bootstrapping.estimated_accrued_microusd, 500_000);
    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("verify bootstrap");
    let events = controller
        .reconcile_due(NOW_MS + 3)
        .await
        .expect("authenticated readiness");

    assert_eq!(provider.create_calls().await, 1);
    assert_eq!(
        events,
        vec![ControllerEvent::StateChanged {
            rental_id: "rental-ready".to_string(),
            state: GpuRentalState::Ready,
        }]
    );
    let rental = state
        .get_gpu_rental("rental-ready")
        .await
        .expect("load rental")
        .expect("rental");
    assert_eq!(rental.observed_state, GpuRentalState::Ready);
    assert_eq!(rental.desired_state, GpuRentalState::Ready);
    assert_eq!(
        rental.endpoint_base_url.as_deref(),
        Some("https://resource-1.example.invalid/v1")
    );
    let steps = state
        .list_gpu_provision_steps("rental-ready")
        .await
        .expect("steps");
    assert_eq!(steps.len(), 2);
    assert!(steps.iter().all(|step| step.status == "succeeded"));
    let providers = state
        .list_gpu_runtime_providers()
        .await
        .expect("runtime providers");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].model_id, "deepseek-v4-flash");
    assert_eq!(providers[0].wire_api, "responses");
    assert_eq!(providers[0].health, "ready");
    assert_eq!(providers[0].display_hourly_microusd, 2_500_000);

    // Repeated health publication advances the runtime catalog independently of rental state.
    // Endpoint replacement must still publish the controller's new process-local address.
    for offset in 0..5 {
        state
            .set_gpu_runtime_provider_health("rental-ready", "ready", NOW_MS + 10 + offset)
            .await
            .expect("advance runtime catalog sequence");
    }

    provider
        .set_secure_endpoint_override(Some("https://replacement.example.invalid/v1".to_string()))
        .await;
    controller
        .reconcile_due(NOW_MS + 60_003)
        .await
        .expect("recreate controller-owned endpoint");
    assert_eq!(
        state
            .get_gpu_rental("rental-ready")
            .await
            .unwrap()
            .unwrap()
            .endpoint_base_url
            .as_deref(),
        Some("https://replacement.example.invalid/v1")
    );
    assert_eq!(
        state.list_gpu_runtime_providers().await.unwrap()[0].base_url,
        "https://replacement.example.invalid/v1"
    );
}

#[tokio::test]
async fn retryable_secure_endpoint_discovery_does_not_destroy_a_live_rental() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let controller = controller(state.clone(), provider.clone());
    create_authorized_rental(&state, "endpoint-retry").await;

    controller.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Running)
        .await;
    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("observe running");
    provider
        .set_secure_endpoint_error(Some(ProviderError::new(
            ProviderErrorKind::Retryable,
            "The secure endpoint is still starting.",
        )))
        .await;

    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("verify bootstrap");
    controller
        .reconcile_due(NOW_MS + 3)
        .await
        .expect("retry endpoint discovery");

    let rental = state
        .get_gpu_rental("rental-endpoint-retry")
        .await
        .expect("load rental")
        .expect("rental exists");
    assert_ne!(rental.desired_state, GpuRentalState::TerminateRequested);
    assert_eq!(rental.observed_state, GpuRentalState::Probing);
    assert_eq!(rental.retry_count, 1);
    assert_ne!(
        rental.last_error_code.as_deref(),
        Some("secure-endpoint-unavailable")
    );
    assert_eq!(provider.terminate_calls().await, 0);
}

#[tokio::test]
async fn readiness_loss_disables_runtime_until_the_full_contract_recovers() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let readiness = Arc::new(SwitchableReadiness::new(true));
    let controller = controller_with_readiness(state.clone(), provider.clone(), readiness.clone());
    create_authorized_rental(&state, "health").await;
    controller.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Running)
        .await;
    controller.reconcile_due(NOW_MS + 1).await.expect("running");
    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("bootstrap");
    controller.reconcile_due(NOW_MS + 3).await.expect("ready");

    let runtime = state.list_gpu_runtime_providers().await.unwrap()[0].clone();
    state
        .upsert_gpu_runtime_provider(
            &GpuRuntimeProviderUpsert {
                rental_id: runtime.rental_id,
                provider_id: runtime.provider_id,
                base_url: runtime.base_url,
                model_id: runtime.model_id,
                wire_api: runtime.wire_api,
                health: runtime.health,
                display_hourly_microusd: 3_000_000,
                maximum_context_tokens: runtime.maximum_context_tokens.unwrap_or(65_536),
                catalog_sequence: 99,
            },
            NOW_MS + 3,
        )
        .await
        .expect("seed stale runtime price");

    readiness.set(false);
    let degraded = controller.reconcile_due(NOW_MS + 4).await.expect("degrade");
    assert_eq!(
        degraded,
        vec![ControllerEvent::StateChanged {
            rental_id: "rental-health".to_string(),
            state: GpuRentalState::Degraded,
        }]
    );
    assert_eq!(
        state.list_gpu_runtime_providers().await.unwrap()[0].health,
        "degraded"
    );
    assert_eq!(
        state.list_gpu_runtime_providers().await.unwrap()[0].display_hourly_microusd,
        2_500_000
    );

    readiness.set(true);
    let recovered = controller
        .reconcile_due(NOW_MS + 60_004)
        .await
        .expect("recover");
    assert_eq!(
        recovered,
        vec![ControllerEvent::StateChanged {
            rental_id: "rental-health".to_string(),
            state: GpuRentalState::Ready,
        }]
    );
    assert_eq!(
        state.list_gpu_runtime_providers().await.unwrap()[0].health,
        "ready"
    );
    assert_eq!(
        state.list_gpu_runtime_providers().await.unwrap()[0].display_hourly_microusd,
        2_500_000
    );
}

#[tokio::test]
async fn ready_health_poll_does_not_consume_saturated_generation_capacity() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let initial = controller(state.clone(), provider.clone());
    create_authorized_rental(&state, "saturated-health").await;
    initial.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Running)
        .await;
    initial.reconcile_due(NOW_MS + 1).await.expect("running");
    initial.reconcile_due(NOW_MS + 2).await.expect("bootstrap");
    initial.reconcile_due(NOW_MS + 3).await.expect("ready");

    let saturated =
        controller_with_readiness(state.clone(), provider, Arc::new(SaturatedReadiness));
    saturated
        .reconcile_due(NOW_MS + 60_003)
        .await
        .expect("lightweight health poll");

    assert_eq!(
        state
            .get_gpu_rental("rental-saturated-health")
            .await
            .unwrap()
            .unwrap()
            .observed_state,
        GpuRentalState::Ready
    );
    assert_eq!(
        state.list_gpu_runtime_providers().await.unwrap()[0].health,
        "ready"
    );
}

#[tokio::test]
async fn provider_side_absence_closes_billing_and_removes_runtime_overlay() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    let controller = controller(state.clone(), provider.clone());
    create_authorized_rental(&state, "manual-delete").await;
    controller.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Running)
        .await;
    controller.reconcile_due(NOW_MS + 1).await.unwrap();
    controller.reconcile_due(NOW_MS + 2).await.unwrap();
    controller.reconcile_due(NOW_MS + 3).await.unwrap();
    provider.remove_instance("resource-1").await;

    let events = controller
        .reconcile_due(NOW_MS + 4)
        .await
        .expect("reconcile provider-side deletion");
    assert_eq!(
        events,
        vec![ControllerEvent::StateChanged {
            rental_id: "rental-manual-delete".to_string(),
            state: GpuRentalState::TerminatedConfirmed,
        }]
    );
    let rental = state
        .get_gpu_rental("rental-manual-delete")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rental.observed_state, GpuRentalState::TerminatedConfirmed);
    assert!(!rental.may_be_billable());
    assert!(state.list_gpu_runtime_providers().await.unwrap().is_empty());
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
        .upsert_gpu_runtime_provider(
            &GpuRuntimeProviderUpsert {
                rental_id: "rental-terminate".to_string(),
                provider_id: "gpu-rental-terminate".to_string(),
                base_url: "https://rental-terminate.example/v1".to_string(),
                model_id: "deepseek-ai/DeepSeek-V4-Flash".to_string(),
                wire_api: "chat".to_string(),
                health: "ready".to_string(),
                display_hourly_microusd: 2_500_000,
                maximum_context_tokens: 65_536,
                catalog_sequence: 2,
            },
            NOW_MS,
        )
        .await
        .expect("seed runtime overlay");
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
    assert!(
        state
            .list_gpu_runtime_providers()
            .await
            .expect("list runtime providers")
            .is_empty(),
        "provider-confirmed termination must remove the runtime overlay"
    );
    assert!(
        !state
            .remove_gpu_runtime_provider("rental-terminate")
            .await
            .expect("probe physical runtime row"),
        "terminal transition must delete the physical runtime row atomically"
    );
}

#[tokio::test]
async fn termination_does_not_require_a_consistent_get_by_id_view() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "ambiguous-get-terminate").await;
    let controller = controller(state.clone(), provider.clone());
    controller.reconcile_due(NOW_MS).await.expect("create");
    provider.make_get_ambiguous().await;
    state
        .request_gpu_rental_termination("rental-ambiguous-get-terminate", NOW_MS + 1)
        .await
        .expect("request termination");

    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("send termination without get preflight");
    assert_eq!(provider.terminate_calls().await, 1);
    assert_eq!(
        state
            .get_gpu_rental("rental-ambiguous-get-terminate")
            .await
            .unwrap()
            .unwrap()
            .observed_state,
        GpuRentalState::Terminating
    );

    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("confirm through complete owned inventory");
    assert_eq!(
        state
            .get_gpu_rental("rental-ambiguous-get-terminate")
            .await
            .unwrap()
            .unwrap()
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
async fn cleanup_preserves_the_root_provisioning_failure() {
    let state = state_runtime().await;
    let provider = FakeProvider::new(CreateBehavior::Success);
    create_authorized_rental(&state, "preserve-root-cause").await;
    let controller = controller(state.clone(), provider.clone());
    controller.reconcile_due(NOW_MS).await.expect("create");
    provider
        .set_instance_state("resource-1", GpuInstanceState::Failed)
        .await;
    controller
        .reconcile_due(NOW_MS + 1)
        .await
        .expect("observe failure");
    controller
        .reconcile_due(NOW_MS + 2)
        .await
        .expect("send cleanup");
    controller
        .reconcile_due(NOW_MS + 3)
        .await
        .expect("confirm cleanup");

    let rental = state
        .get_gpu_rental("rental-preserve-root-cause")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rental.observed_state, GpuRentalState::TerminatedConfirmed);
    assert_eq!(
        rental.last_error_code.as_deref(),
        Some("provider-instance-failed")
    );
    assert_eq!(
        rental.last_error_message.as_deref(),
        Some("Provider reported that the instance failed.")
    );
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
