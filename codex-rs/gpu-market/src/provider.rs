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
            || (self.interruptible && !request.allow_interruptible)
        {
            return Err(ProviderError::new(
                ProviderErrorKind::OfferUnavailable,
                "the selected offer no longer satisfies the recipe",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateInstanceRequest {
    pub offer: GpuOffer,
    pub client_operation_id: String,
    pub ownership_tag: String,
    pub image: String,
    pub disk_gib: u64,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub provider: String,
    pub supports_ownership_tags: bool,
    pub supports_inventory: bool,
    pub supports_native_ttl: bool,
    pub supports_native_spend_cap: bool,
    pub security_classes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderErrorKind {
    Unauthorized,
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
