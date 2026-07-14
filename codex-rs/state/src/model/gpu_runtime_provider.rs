#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct GpuProvisionStep {
    pub rental_id: String,
    pub step_id: String,
    pub command_digest: String,
    pub status: String,
    pub attempt_count: i64,
    pub postcondition_json: Option<String>,
    pub sanitized_error: Option<String>,
    pub started_at_ms: Option<i64>,
    pub completed_at_ms: Option<i64>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct GpuRuntimeProvider {
    pub rental_id: String,
    pub provider_id: String,
    pub base_url: String,
    pub model_id: String,
    pub wire_api: String,
    pub health: String,
    pub display_hourly_microusd: i64,
    pub catalog_sequence: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRuntimeProviderUpsert {
    pub rental_id: String,
    pub provider_id: String,
    pub base_url: String,
    pub model_id: String,
    pub wire_api: String,
    pub health: String,
    pub display_hourly_microusd: i64,
    pub catalog_sequence: i64,
}
