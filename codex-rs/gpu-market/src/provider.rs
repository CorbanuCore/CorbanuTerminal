use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::future::Future;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareRequirements {
    pub gpu_model: String,
    pub gpu_count: u16,
    pub minimum_vram_mib_per_gpu: u64,
    pub minimum_host_ram_mib: u64,
    pub minimum_disk_gib: u64,
    pub requires_high_bandwidth_interconnect: bool,
    pub allowed_cuda_versions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchOffersRequest {
    pub hardware: HardwareRequirements,
    pub allow_interruptible: bool,
    pub require_verified_or_secure: bool,
    pub maximum_hourly_microusd: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuOffer {
    pub provider: String,
    pub offer_id: String,
    pub gpu_model: String,
    pub gpu_count: u16,
    pub vram_mib_per_gpu: u64,
    pub host_ram_mib: u64,
    pub disk_gib: u64,
    /// True when the provider inventory itself attests the requested link.
    pub high_bandwidth_interconnect: bool,
    /// True when the pinned launch command gates server startup on an
    /// allocation-local topology probe. This is disclosed separately because
    /// it is not a provider offer guarantee.
    pub runtime_topology_verification: bool,
    pub cuda_versions: Vec<String>,
    pub region: String,
    pub security_class: String,
    pub reliability_millionths: Option<u32>,
    pub interruptible: bool,
    pub hourly_microusd: i64,
    pub storage_microusd_per_gib_month: Option<i64>,
    pub quoted_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub raw_snapshot: serde_json::Value,
}

impl GpuOffer {
    pub fn validate_for(
        &self,
        request: &SearchOffersRequest,
        now_ms: i64,
    ) -> Result<(), ProviderError> {
        if self.expires_at_ms.is_some_and(|expiry| expiry < now_ms) {
            return Err(ProviderError::new(
                ProviderErrorKind::OfferUnavailable,
                "the selected offer expired",
            ));
        }
        if self.hourly_microusd <= 0 || self.hourly_microusd > request.maximum_hourly_microusd {
            return Err(ProviderError::new(
                ProviderErrorKind::PriceDrift,
                "the selected offer is outside the authorized hourly price",
            ));
        }
        if self.gpu_model != request.hardware.gpu_model
            || self.gpu_count < request.hardware.gpu_count
            || self.vram_mib_per_gpu < request.hardware.minimum_vram_mib_per_gpu
            || self.host_ram_mib < request.hardware.minimum_host_ram_mib
            || self.disk_gib < request.hardware.minimum_disk_gib
            || (request.hardware.requires_high_bandwidth_interconnect
                && !self.high_bandwidth_interconnect
                && !self.runtime_topology_verification)
            || (!request.hardware.allowed_cuda_versions.is_empty()
                && !self.cuda_versions.is_empty()
                && !self.cuda_versions.iter().any(|available| {
                    request
                        .hardware
                        .allowed_cuda_versions
                        .iter()
                        .any(|allowed| allowed == available)
                }))
            || (self.interruptible && !request.allow_interruptible)
            || (request.require_verified_or_secure
                && !matches!(self.security_class.as_str(), "verified" | "secure"))
        {
            return Err(ProviderError::new(
                ProviderErrorKind::OfferUnavailable,
                "the selected offer no longer satisfies the recipe",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CreateInstanceRequest {
    pub offer: GpuOffer,
    pub client_operation_id: String,
    pub ownership_tag: String,
    pub image: String,
    pub disk_gib: u64,
    pub container_entrypoint: Vec<String>,
    pub launch_command: Vec<String>,
    pub inference_port: u16,
    pub endpoint_token: crate::SecretValue,
    pub huggingface_token: Option<crate::SecretValue>,
}

impl fmt::Debug for CreateInstanceRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateInstanceRequest")
            .field("offer", &self.offer)
            .field("client_operation_id", &self.client_operation_id)
            .field("ownership_tag", &self.ownership_tag)
            .field("image", &self.image)
            .field("disk_gib", &self.disk_gib)
            .field("container_entrypoint", &self.container_entrypoint)
            .field("launch_command", &self.launch_command)
            .field("inference_port", &self.inference_port)
            .field("endpoint_token", &"[REDACTED]")
            .field(
                "huggingface_token",
                &self.huggingface_token.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedInstanceQuery {
    pub installation_id: String,
    pub ownership_tag: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuInstanceState {
    Allocating,
    Running,
    Stopped,
    Failed,
    Terminating,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuInstance {
    pub provider: String,
    pub resource_id: String,
    pub ownership_tag: String,
    pub state: GpuInstanceState,
    pub gpu_model: String,
    pub gpu_count: u16,
    pub host_ram_mib: Option<u64>,
    pub disk_gib: Option<u64>,
    /// `None` means the provider response did not prove the allocated topology.
    pub high_bandwidth_interconnect: Option<bool>,
    pub hourly_microusd: i64,
    pub created_at_ms: Option<i64>,
    pub public_ip: Option<String>,
    pub ssh_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingState {
    pub resource_id: String,
    pub estimated_accrued_microusd: i64,
    pub provider_reported_cost_microusd: Option<i64>,
    pub still_billable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuProvisionPhase {
    HardwareCheck,
    RuntimeSetup,
    RuntimeBuild,
    ModelDownload,
    ModelVerification,
    ModelLoading,
    EndpointProbing,
}

impl GpuProvisionPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HardwareCheck => "hardware_check",
            Self::RuntimeSetup => "runtime_setup",
            Self::RuntimeBuild => "runtime_build",
            Self::ModelDownload => "model_download",
            Self::ModelVerification => "model_verification",
            Self::ModelLoading => "model_loading",
            Self::EndpointProbing => "endpoint_probing",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "hardware_check" => Some(Self::HardwareCheck),
            "runtime_setup" => Some(Self::RuntimeSetup),
            "runtime_build" => Some(Self::RuntimeBuild),
            "model_download" => Some(Self::ModelDownload),
            "model_verification" => Some(Self::ModelVerification),
            "model_loading" => Some(Self::ModelLoading),
            "endpoint_probing" => Some(Self::EndpointProbing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub provider: String,
    pub supports_ownership_tags: bool,
    pub supports_inventory: bool,
    pub supports_secure_endpoint_transport: bool,
    pub supports_native_ttl: bool,
    pub supports_native_spend_cap: bool,
    pub security_classes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderErrorKind {
    NotConfigured,
    CapabilityUnavailable,
    Unauthorized,
    InsufficientFunds,
    InvalidRequest,
    OfferUnavailable,
    PriceDrift,
    RateLimited,
    Retryable,
    Ambiguous,
    Permanent,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProviderError {
    pub kind: ProviderErrorKind,
    pub safe_message: String,
    pub retry_after_ms: Option<i64>,
    pub diagnostic_ref: Option<String>,
}

impl ProviderError {
    pub fn new(kind: ProviderErrorKind, safe_message: impl Into<String>) -> Self {
        Self {
            kind,
            safe_message: safe_message.into(),
            retry_after_ms: None,
            diagnostic_ref: None,
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self.kind,
            ProviderErrorKind::RateLimited
                | ProviderErrorKind::Retryable
                | ProviderErrorKind::Ambiguous
        )
    }

    pub fn with_retry_after_ms(mut self, retry_after_ms: i64) -> Self {
        self.retry_after_ms = Some(retry_after_ms.max(0));
        self
    }

    pub fn with_diagnostic_ref(mut self, diagnostic_ref: String) -> Self {
        self.diagnostic_ref = Some(diagnostic_ref);
        self
    }
}

impl fmt::Debug for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderError")
            .field("kind", &self.kind)
            .field("safe_message", &self.safe_message)
            .field("retry_after_ms", &self.retry_after_ms)
            .field("diagnostic_ref", &self.diagnostic_ref)
            .finish()
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.safe_message.as_str())
    }
}

impl std::error::Error for ProviderError {}

pub type ProviderResult<T> = Result<T, ProviderError>;

/// Provider boundary for marketplace discovery and resource lifecycle reconciliation.
///
/// Implementations must return sanitized errors, preserve ambiguous remote outcomes, and make
/// owned-resource inventory broad enough to recover a create whose response was lost.
pub trait GpuProvider: Send + Sync {
    fn capabilities(&self) -> ProviderCapabilities;

    /// True when the provider's create call accepts the selected offer ID as an immutable,
    /// atomic acceptance handle and rejects an unavailable handle without substitution.
    fn create_revalidates_exact_offer_atomically(&self) -> bool {
        false
    }

    fn secure_endpoint_base_url(
        &self,
        _instance: &GpuInstance,
        _inference_port: u16,
    ) -> impl Future<Output = ProviderResult<String>> + Send {
        async {
            Err(ProviderError::new(
                ProviderErrorKind::Permanent,
                "The provider adapter cannot prove a secure inference transport.",
            ))
        }
    }

    fn provision_phase(
        &self,
        _instance: &GpuInstance,
    ) -> impl Future<Output = ProviderResult<Option<GpuProvisionPhase>>> + Send {
        async { Ok(None) }
    }

    fn search_offers(
        &self,
        request: SearchOffersRequest,
    ) -> impl Future<Output = ProviderResult<Vec<GpuOffer>>> + Send;

    fn create_instance(
        &self,
        request: CreateInstanceRequest,
    ) -> impl Future<Output = ProviderResult<GpuInstance>> + Send;

    fn get_instance(
        &self,
        resource_id: String,
    ) -> impl Future<Output = ProviderResult<Option<GpuInstance>>> + Send;

    fn list_owned_instances(
        &self,
        query: OwnedInstanceQuery,
    ) -> impl Future<Output = ProviderResult<Vec<GpuInstance>>> + Send;

    fn terminate_instance(
        &self,
        resource_id: String,
    ) -> impl Future<Output = ProviderResult<()>> + Send;

    fn billing_state(
        &self,
        resource_id: String,
    ) -> impl Future<Output = ProviderResult<BillingState>> + Send;
}
