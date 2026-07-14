use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuRentalState {
    Draft,
    Quoted,
    CreatePending,
    Allocating,
    Bootstrapping,
    Downloading,
    Starting,
    Probing,
    Ready,
    Degraded,
    TerminateRequested,
    Terminating,
    TerminatedConfirmed,
    TerminationUnconfirmed,
    Reconciling,
    Orphaned,
    Failed,
}

impl GpuRentalState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Quoted => "quoted",
            Self::CreatePending => "create_pending",
            Self::Allocating => "allocating",
            Self::Bootstrapping => "bootstrapping",
            Self::Downloading => "downloading",
            Self::Starting => "starting",
            Self::Probing => "probing",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::TerminateRequested => "terminate_requested",
            Self::Terminating => "terminating",
            Self::TerminatedConfirmed => "terminated_confirmed",
            Self::TerminationUnconfirmed => "termination_unconfirmed",
            Self::Reconciling => "reconciling",
            Self::Orphaned => "orphaned",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "draft" => Ok(Self::Draft),
            "quoted" => Ok(Self::Quoted),
            "create_pending" => Ok(Self::CreatePending),
            "allocating" => Ok(Self::Allocating),
            "bootstrapping" => Ok(Self::Bootstrapping),
            "downloading" => Ok(Self::Downloading),
            "starting" => Ok(Self::Starting),
            "probing" => Ok(Self::Probing),
            "ready" => Ok(Self::Ready),
            "degraded" => Ok(Self::Degraded),
            "terminate_requested" => Ok(Self::TerminateRequested),
            "terminating" => Ok(Self::Terminating),
            "terminated_confirmed" => Ok(Self::TerminatedConfirmed),
            "termination_unconfirmed" => Ok(Self::TerminationUnconfirmed),
            "reconciling" => Ok(Self::Reconciling),
            "orphaned" => Ok(Self::Orphaned),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow::anyhow!("invalid GPU rental state: {value}")),
        }
    }

    pub fn may_be_billable(self) -> bool {
        !matches!(
            self,
            Self::Draft | Self::Quoted | Self::TerminatedConfirmed | Self::Failed
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuLimitEnforcement {
    ProviderGuaranteed,
    LocalControllerDependent,
}

impl GpuLimitEnforcement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderGuaranteed => "provider_guaranteed",
            Self::LocalControllerDependent => "local_controller_dependent",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "provider_guaranteed" => Ok(Self::ProviderGuaranteed),
            "local_controller_dependent" => Ok(Self::LocalControllerDependent),
            _ => Err(anyhow::anyhow!("invalid GPU limit enforcement: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRental {
    pub rental_id: String,
    pub installation_id: String,
    pub client_operation_id: String,
    pub provider: String,
    pub recipe_id: String,
    pub recipe_revision: String,
    pub offer_snapshot_json: String,
    pub quote_expires_at_ms: Option<i64>,
    pub max_hourly_microusd: i64,
    pub max_total_microusd: i64,
    pub terminate_at_ms: i64,
    pub enforcement_class: GpuLimitEnforcement,
    pub desired_state: GpuRentalState,
    pub observed_state: GpuRentalState,
    pub provider_resource_id: Option<String>,
    pub ownership_tag: String,
    pub state_sequence: i64,
    pub controller_lease_owner: Option<String>,
    pub controller_lease_until_ms: i64,
    pub provision_step: Option<String>,
    pub endpoint_base_url: Option<String>,
    pub endpoint_provider_id: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub diagnostic_ref: Option<String>,
    pub last_reconciled_at_ms: Option<i64>,
    pub next_retry_at_ms: i64,
    pub retry_count: i64,
    pub estimated_accrued_microusd: i64,
    pub provider_reported_cost_microusd: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub terminated_confirmed_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRentalCreateParams {
    pub rental_id: String,
    pub installation_id: String,
    pub client_operation_id: String,
    pub provider: String,
    pub recipe_id: String,
    pub recipe_revision: String,
    pub offer_snapshot_json: String,
    pub quote_expires_at_ms: Option<i64>,
    pub max_hourly_microusd: i64,
    pub max_total_microusd: i64,
    pub terminate_at_ms: i64,
    pub enforcement_class: GpuLimitEnforcement,
    pub ownership_tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRentalLease {
    pub rental: GpuRental,
    pub owner: String,
    pub lease_until_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GpuRentalUpdate {
    pub desired_state: Option<GpuRentalState>,
    pub observed_state: Option<GpuRentalState>,
    pub provider_resource_id: Option<String>,
    pub provision_step: Option<String>,
    pub endpoint_base_url: Option<String>,
    pub endpoint_provider_id: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub diagnostic_ref: Option<String>,
    pub clear_last_error: bool,
    pub next_retry_at_ms: Option<i64>,
    pub estimated_accrued_microusd: Option<i64>,
    pub provider_reported_cost_microusd: Option<i64>,
    pub increment_retry_count: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuOperationKind {
    Create,
    Query,
    Provision,
    Probe,
    Terminate,
    Billing,
}

impl GpuOperationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Query => "query",
            Self::Provision => "provision",
            Self::Probe => "probe",
            Self::Terminate => "terminate",
            Self::Billing => "billing",
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct GpuRentalRow {
    pub(crate) rental_id: String,
    pub(crate) installation_id: String,
    pub(crate) client_operation_id: String,
    pub(crate) provider: String,
    pub(crate) recipe_id: String,
    pub(crate) recipe_revision: String,
    pub(crate) offer_snapshot_json: String,
    pub(crate) quote_expires_at_ms: Option<i64>,
    pub(crate) max_hourly_microusd: i64,
    pub(crate) max_total_microusd: i64,
    pub(crate) terminate_at_ms: i64,
    pub(crate) enforcement_class: String,
    pub(crate) desired_state: String,
    pub(crate) observed_state: String,
    pub(crate) provider_resource_id: Option<String>,
    pub(crate) ownership_tag: String,
    pub(crate) state_sequence: i64,
    pub(crate) controller_lease_owner: Option<String>,
    pub(crate) controller_lease_until_ms: i64,
    pub(crate) provision_step: Option<String>,
    pub(crate) endpoint_base_url: Option<String>,
    pub(crate) endpoint_provider_id: Option<String>,
    pub(crate) last_error_code: Option<String>,
    pub(crate) last_error_message: Option<String>,
    pub(crate) diagnostic_ref: Option<String>,
    pub(crate) last_reconciled_at_ms: Option<i64>,
    pub(crate) next_retry_at_ms: i64,
    pub(crate) retry_count: i64,
    pub(crate) estimated_accrued_microusd: i64,
    pub(crate) provider_reported_cost_microusd: Option<i64>,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
    pub(crate) terminated_confirmed_at_ms: Option<i64>,
}

impl TryFrom<GpuRentalRow> for GpuRental {
    type Error = anyhow::Error;

    fn try_from(value: GpuRentalRow) -> Result<Self> {
        Ok(Self {
            rental_id: value.rental_id,
            installation_id: value.installation_id,
            client_operation_id: value.client_operation_id,
            provider: value.provider,
            recipe_id: value.recipe_id,
            recipe_revision: value.recipe_revision,
            offer_snapshot_json: value.offer_snapshot_json,
            quote_expires_at_ms: value.quote_expires_at_ms,
            max_hourly_microusd: value.max_hourly_microusd,
            max_total_microusd: value.max_total_microusd,
            terminate_at_ms: value.terminate_at_ms,
            enforcement_class: GpuLimitEnforcement::parse(value.enforcement_class.as_str())?,
            desired_state: GpuRentalState::parse(value.desired_state.as_str())?,
            observed_state: GpuRentalState::parse(value.observed_state.as_str())?,
            provider_resource_id: value.provider_resource_id,
            ownership_tag: value.ownership_tag,
            state_sequence: value.state_sequence,
            controller_lease_owner: value.controller_lease_owner,
            controller_lease_until_ms: value.controller_lease_until_ms,
            provision_step: value.provision_step,
            endpoint_base_url: value.endpoint_base_url,
            endpoint_provider_id: value.endpoint_provider_id,
            last_error_code: value.last_error_code,
            last_error_message: value.last_error_message,
            diagnostic_ref: value.diagnostic_ref,
            last_reconciled_at_ms: value.last_reconciled_at_ms,
            next_retry_at_ms: value.next_retry_at_ms,
            retry_count: value.retry_count,
            estimated_accrued_microusd: value.estimated_accrued_microusd,
            provider_reported_cost_microusd: value.provider_reported_cost_microusd,
            created_at_ms: value.created_at_ms,
            updated_at_ms: value.updated_at_ms,
            terminated_confirmed_at_ms: value.terminated_confirmed_at_ms,
        })
    }
}
