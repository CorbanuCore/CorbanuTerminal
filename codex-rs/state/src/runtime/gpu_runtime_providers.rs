use super::*;
use crate::GpuProvisionStep;
use crate::GpuRuntimeProvider;
use crate::GpuRuntimeProviderUpsert;

impl StateRuntime {
    pub async fn begin_gpu_provision_step(
        &self,
        rental_id: &str,
        step_id: &str,
        command_digest: &str,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        validate_identifier("step_id", step_id)?;
        validate_identifier("command_digest", command_digest)?;
        let result = sqlx::query(
            r#"
INSERT INTO gpu_provision_steps (
    rental_id, step_id, command_digest, status, attempt_count, started_at_ms, updated_at_ms
) VALUES (?, ?, ?, 'running', 1, ?, ?)
ON CONFLICT(rental_id, step_id) DO UPDATE SET
    status = 'running',
    attempt_count = gpu_provision_steps.attempt_count + 1,
    sanitized_error = NULL,
    started_at_ms = excluded.started_at_ms,
    completed_at_ms = NULL,
    updated_at_ms = excluded.updated_at_ms
WHERE gpu_provision_steps.command_digest = excluded.command_digest
  AND gpu_provision_steps.status IN ('pending', 'failed')
            "#,
        )
        .bind(rental_id)
        .bind(step_id)
        .bind(command_digest)
        .bind(now_ms)
        .bind(now_ms)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn finish_gpu_provision_step(
        &self,
        rental_id: &str,
        step_id: &str,
        succeeded: bool,
        postcondition_json: Option<&str>,
        sanitized_error: Option<&str>,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        if let Some(value) = postcondition_json {
            let parsed: serde_json::Value = serde_json::from_str(value)?;
            if !parsed.is_object() {
                return Err(anyhow::anyhow!(
                    "provision postcondition must be a JSON object"
                ));
            }
        }
        let status = if succeeded { "succeeded" } else { "failed" };
        let result = sqlx::query(
            r#"
UPDATE gpu_provision_steps
SET status = ?, postcondition_json = ?, sanitized_error = ?,
    completed_at_ms = ?, updated_at_ms = ?
WHERE rental_id = ? AND step_id = ? AND status = 'running'
            "#,
        )
        .bind(status)
        .bind(postcondition_json)
        .bind(sanitized_error)
        .bind(now_ms)
        .bind(now_ms)
        .bind(rental_id)
        .bind(step_id)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn list_gpu_provision_steps(
        &self,
        rental_id: &str,
    ) -> anyhow::Result<Vec<GpuProvisionStep>> {
        Ok(sqlx::query_as::<_, GpuProvisionStep>(
            "SELECT * FROM gpu_provision_steps WHERE rental_id = ? ORDER BY step_id",
        )
        .bind(rental_id)
        .fetch_all(self.pool.as_ref())
        .await?)
    }

    pub async fn upsert_gpu_runtime_provider(
        &self,
        provider: &GpuRuntimeProviderUpsert,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        validate_runtime_provider(provider)?;
        let result = sqlx::query(
            r#"
INSERT INTO gpu_runtime_providers (
    rental_id, provider_id, base_url, model_id, wire_api, health,
    display_hourly_microusd, catalog_sequence, updated_at_ms
)
SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?
WHERE EXISTS (
    SELECT 1 FROM gpu_rentals
    WHERE rental_id = ? AND observed_state IN ('ready', 'degraded')
)
ON CONFLICT(rental_id) DO UPDATE SET
    provider_id = excluded.provider_id, base_url = excluded.base_url,
    model_id = excluded.model_id, wire_api = excluded.wire_api,
    health = excluded.health,
    display_hourly_microusd = excluded.display_hourly_microusd,
    catalog_sequence = excluded.catalog_sequence,
    updated_at_ms = excluded.updated_at_ms
WHERE excluded.catalog_sequence >= gpu_runtime_providers.catalog_sequence
            "#,
        )
        .bind(provider.rental_id.as_str())
        .bind(provider.provider_id.as_str())
        .bind(provider.base_url.as_str())
        .bind(provider.model_id.as_str())
        .bind(provider.wire_api.as_str())
        .bind(provider.health.as_str())
        .bind(provider.display_hourly_microusd)
        .bind(provider.catalog_sequence)
        .bind(now_ms)
        .bind(provider.rental_id.as_str())
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn list_gpu_runtime_providers(&self) -> anyhow::Result<Vec<GpuRuntimeProvider>> {
        Ok(sqlx::query_as::<_, GpuRuntimeProvider>(
            r#"
SELECT p.* FROM gpu_runtime_providers p
JOIN gpu_rentals r ON r.rental_id = p.rental_id
WHERE r.observed_state IN ('ready', 'degraded')
ORDER BY p.updated_at_ms DESC
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?)
    }

    pub async fn remove_gpu_runtime_provider(&self, rental_id: &str) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
DELETE FROM gpu_runtime_providers
WHERE rental_id = ?
  AND EXISTS (
      SELECT 1 FROM gpu_rentals
      WHERE rental_id = ? AND observed_state = 'terminated_confirmed'
  )
            "#,
        )
        .bind(rental_id)
        .bind(rental_id)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }
}

fn validate_identifier(name: &str, value: &str) -> anyhow::Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
    {
        return Err(anyhow::anyhow!("invalid {name}"));
    }
    Ok(())
}

fn validate_runtime_provider(provider: &GpuRuntimeProviderUpsert) -> anyhow::Result<()> {
    validate_identifier("provider_id", provider.provider_id.as_str())?;
    if provider.rental_id.is_empty() || provider.model_id.is_empty() {
        return Err(anyhow::anyhow!(
            "runtime provider identity must not be empty"
        ));
    }
    if provider.wire_api != "chat" || !matches!(provider.health.as_str(), "ready" | "degraded") {
        return Err(anyhow::anyhow!(
            "invalid runtime provider protocol or health"
        ));
    }
    let url = url::Url::parse(provider.base_url.as_str())?;
    if url.scheme() != "https"
        && !url
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "::1" | "localhost"))
    {
        return Err(anyhow::anyhow!(
            "runtime provider endpoint must use HTTPS or loopback"
        ));
    }
    if provider.display_hourly_microusd <= 0 || provider.catalog_sequence <= 0 {
        return Err(anyhow::anyhow!(
            "invalid runtime provider price or sequence"
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "gpu_runtime_providers_tests.rs"]
mod tests;
