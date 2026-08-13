use super::*;
use crate::GpuOperationKind;
use crate::GpuRental;
use crate::GpuRentalCreateParams;
use crate::GpuRentalLease;
use crate::GpuRentalState;
use crate::GpuRentalUpdate;
use crate::model::GpuRentalRow;

const GPU_RENTAL_COLUMNS: &str = r#"
    rental_id,
    installation_id,
    client_operation_id,
    provider,
    recipe_id,
    recipe_revision,
    offer_snapshot_json,
    quote_expires_at_ms,
    max_hourly_microusd,
    max_total_microusd,
    terminate_at_ms,
    enforcement_class,
    desired_state,
    observed_state,
    provider_resource_id,
    ownership_tag,
    state_sequence,
    controller_lease_owner,
    controller_lease_until_ms,
    provision_step,
    endpoint_base_url,
    endpoint_provider_id,
    last_error_code,
    last_error_message,
    diagnostic_ref,
    last_reconciled_at_ms,
    next_retry_at_ms,
    retry_count,
    estimated_accrued_microusd,
    provider_reported_cost_microusd,
    created_at_ms,
    updated_at_ms,
    terminated_confirmed_at_ms
"#;

impl StateRuntime {
    pub async fn create_gpu_rental(
        &self,
        params: &GpuRentalCreateParams,
        now_ms: i64,
    ) -> anyhow::Result<GpuRental> {
        validate_gpu_rental_create(params, now_ms)?;
        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query(
            r#"
INSERT INTO gpu_rentals (
    rental_id,
    installation_id,
    client_operation_id,
    provider,
    recipe_id,
    recipe_revision,
    offer_snapshot_json,
    quote_expires_at_ms,
    max_hourly_microusd,
    max_total_microusd,
    terminate_at_ms,
    enforcement_class,
    desired_state,
    observed_state,
    ownership_tag,
    next_retry_at_ms,
    created_at_ms,
    updated_at_ms
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(client_operation_id) DO NOTHING
            "#,
        )
        .bind(params.rental_id.as_str())
        .bind(params.installation_id.as_str())
        .bind(params.client_operation_id.as_str())
        .bind(params.provider.as_str())
        .bind(params.recipe_id.as_str())
        .bind(params.recipe_revision.as_str())
        .bind(params.offer_snapshot_json.as_str())
        .bind(params.quote_expires_at_ms)
        .bind(params.max_hourly_microusd)
        .bind(params.max_total_microusd)
        .bind(params.terminate_at_ms)
        .bind(params.enforcement_class.as_str())
        .bind(GpuRentalState::Quoted.as_str())
        .bind(GpuRentalState::Quoted.as_str())
        .bind(params.ownership_tag.as_str())
        .bind(now_ms)
        .bind(now_ms)
        .bind(now_ms)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        tx.commit().await?;

        let rental = self
            .get_gpu_rental_by_client_operation(params.client_operation_id.as_str())
            .await?
            .ok_or_else(|| anyhow::anyhow!("GPU rental insert did not produce a row"))?;
        if inserted == 0 && !gpu_rental_matches_create(&rental, params) {
            return Err(anyhow::anyhow!(
                "client operation id is already bound to different GPU rental terms"
            ));
        }
        Ok(rental)
    }

    pub async fn get_gpu_rental(&self, rental_id: &str) -> anyhow::Result<Option<GpuRental>> {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT ");
        builder.push(GPU_RENTAL_COLUMNS);
        builder.push(" FROM gpu_rentals WHERE rental_id = ");
        builder.push_bind(rental_id);
        builder
            .build_query_as::<GpuRentalRow>()
            .fetch_optional(self.pool.as_ref())
            .await?
            .map(GpuRental::try_from)
            .transpose()
    }

    pub async fn get_gpu_rental_by_client_operation(
        &self,
        client_operation_id: &str,
    ) -> anyhow::Result<Option<GpuRental>> {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT ");
        builder.push(GPU_RENTAL_COLUMNS);
        builder.push(" FROM gpu_rentals WHERE client_operation_id = ");
        builder.push_bind(client_operation_id);
        builder
            .build_query_as::<GpuRentalRow>()
            .fetch_optional(self.pool.as_ref())
            .await?
            .map(GpuRental::try_from)
            .transpose()
    }

    pub async fn list_gpu_rentals(&self, limit: usize) -> anyhow::Result<Vec<GpuRental>> {
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT ");
        builder.push(GPU_RENTAL_COLUMNS);
        builder.push(" FROM gpu_rentals ORDER BY created_at_ms DESC LIMIT ");
        builder.push_bind(i64::try_from(limit.max(1))?);
        let rows = builder
            .build_query_as::<GpuRentalRow>()
            .fetch_all(self.pool.as_ref())
            .await?;
        rows.into_iter().map(GpuRental::try_from).collect()
    }

    pub async fn request_gpu_rental_creation(
        &self,
        rental_id: &str,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE gpu_rentals
SET desired_state = ?,
    state_sequence = state_sequence + 1,
    next_retry_at_ms = ?,
    updated_at_ms = ?
WHERE rental_id = ?
  AND desired_state = ?
  AND observed_state = ?
  AND terminate_at_ms > ?
  AND (quote_expires_at_ms IS NULL OR quote_expires_at_ms >= ?)
            "#,
        )
        .bind(GpuRentalState::CreatePending.as_str())
        .bind(now_ms)
        .bind(now_ms)
        .bind(rental_id)
        .bind(GpuRentalState::Quoted.as_str())
        .bind(GpuRentalState::Quoted.as_str())
        .bind(now_ms)
        .bind(now_ms)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn request_gpu_rental_termination(
        &self,
        rental_id: &str,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE gpu_rentals
SET desired_state = ?,
    state_sequence = state_sequence + CASE WHEN desired_state = ? THEN 0 ELSE 1 END,
    next_retry_at_ms = ?,
    updated_at_ms = ?
WHERE rental_id = ? AND observed_state != ?
            "#,
        )
        .bind(GpuRentalState::TerminateRequested.as_str())
        .bind(GpuRentalState::TerminateRequested.as_str())
        .bind(now_ms)
        .bind(now_ms)
        .bind(rental_id)
        .bind(GpuRentalState::TerminatedConfirmed.as_str())
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn claim_due_gpu_rentals(
        &self,
        owner: &str,
        now_ms: i64,
        lease_ttl_ms: i64,
        limit: usize,
    ) -> anyhow::Result<Vec<GpuRentalLease>> {
        self.claim_due_gpu_rentals_inner(owner, /*provider*/ None, now_ms, lease_ttl_ms, limit)
            .await
    }

    pub async fn claim_due_gpu_rentals_for_provider(
        &self,
        owner: &str,
        provider: &str,
        now_ms: i64,
        lease_ttl_ms: i64,
        limit: usize,
    ) -> anyhow::Result<Vec<GpuRentalLease>> {
        if provider.is_empty() {
            return Err(anyhow::anyhow!("GPU provider must not be empty"));
        }
        self.claim_due_gpu_rentals_inner(owner, Some(provider), now_ms, lease_ttl_ms, limit)
            .await
    }

    async fn claim_due_gpu_rentals_inner(
        &self,
        owner: &str,
        provider: Option<&str>,
        now_ms: i64,
        lease_ttl_ms: i64,
        limit: usize,
    ) -> anyhow::Result<Vec<GpuRentalLease>> {
        let lease_until_ms = now_ms.saturating_add(lease_ttl_ms.max(1));
        // One write statement avoids SQLite's deferred-transaction read→write upgrade race when
        // two PFTerminal processes try to claim the same due rental concurrently.
        let mut builder =
            QueryBuilder::<Sqlite>::new("UPDATE gpu_rentals SET controller_lease_owner = ");
        builder.push_bind(owner);
        builder.push(", controller_lease_until_ms = ");
        builder.push_bind(lease_until_ms);
        builder.push(", updated_at_ms = ");
        builder.push_bind(now_ms);
        builder.push(
            " WHERE rental_id IN (SELECT rental_id FROM gpu_rentals WHERE next_retry_at_ms <= ",
        );
        builder.push_bind(now_ms);
        builder.push(" AND observed_state != ");
        builder.push_bind(GpuRentalState::TerminatedConfirmed.as_str());
        if let Some(provider) = provider {
            builder.push(" AND provider = ");
            builder.push_bind(provider);
        }
        builder.push(" AND (controller_lease_until_ms <= ");
        builder.push_bind(now_ms);
        builder.push(" OR controller_lease_owner = ");
        builder.push_bind(owner);
        builder.push(") ORDER BY next_retry_at_ms, created_at_ms LIMIT ");
        builder.push_bind(i64::try_from(limit.max(1))?);
        builder.push(") AND (controller_lease_until_ms <= ");
        builder.push_bind(now_ms);
        builder.push(" OR controller_lease_owner = ");
        builder.push_bind(owner);
        builder.push(") RETURNING ");
        builder.push(GPU_RENTAL_COLUMNS);
        let rows = builder
            .build_query_as::<GpuRentalRow>()
            .fetch_all(self.pool.as_ref())
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(GpuRentalLease {
                    rental: row.try_into()?,
                    owner: owner.to_string(),
                    lease_until_ms,
                })
            })
            .collect()
    }

    pub async fn update_gpu_rental(
        &self,
        lease: &GpuRentalLease,
        update: &GpuRentalUpdate,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let desired_state = update.desired_state.map(GpuRentalState::as_str);
        let observed_state = update.observed_state.map(GpuRentalState::as_str);
        let terminated = observed_state == Some(GpuRentalState::TerminatedConfirmed.as_str());
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
UPDATE gpu_rentals
SET state_sequence = state_sequence + CASE
        WHEN (? IS NOT NULL AND desired_state != ?)
          OR (? IS NOT NULL AND observed_state != ?)
        THEN 1 ELSE 0 END,
    desired_state = COALESCE(?, desired_state),
    observed_state = COALESCE(?, observed_state),
    provider_resource_id = COALESCE(?, provider_resource_id),
    provision_step = COALESCE(?, provision_step),
    endpoint_base_url = COALESCE(?, endpoint_base_url),
    endpoint_provider_id = COALESCE(?, endpoint_provider_id),
    last_error_code = CASE WHEN ? THEN NULL ELSE COALESCE(?, last_error_code) END,
    last_error_message = CASE WHEN ? THEN NULL ELSE COALESCE(?, last_error_message) END,
    diagnostic_ref = CASE WHEN ? THEN NULL ELSE COALESCE(?, diagnostic_ref) END,
    last_reconciled_at_ms = ?,
    next_retry_at_ms = COALESCE(?, next_retry_at_ms),
    retry_count = retry_count + ?,
    estimated_accrued_microusd = COALESCE(?, estimated_accrued_microusd),
    provider_reported_cost_microusd = COALESCE(?, provider_reported_cost_microusd),
    controller_lease_owner = NULL,
    controller_lease_until_ms = 0,
    updated_at_ms = ?,
    terminated_confirmed_at_ms = CASE
        WHEN ? THEN COALESCE(terminated_confirmed_at_ms, ?)
        ELSE terminated_confirmed_at_ms END
WHERE rental_id = ? AND controller_lease_owner = ?
            "#,
        )
        .bind(desired_state)
        .bind(desired_state)
        .bind(observed_state)
        .bind(observed_state)
        .bind(desired_state)
        .bind(observed_state)
        .bind(update.provider_resource_id.as_deref())
        .bind(update.provision_step.as_deref())
        .bind(update.endpoint_base_url.as_deref())
        .bind(update.endpoint_provider_id.as_deref())
        .bind(update.clear_last_error)
        .bind(update.last_error_code.as_deref())
        .bind(update.clear_last_error)
        .bind(update.last_error_message.as_deref())
        .bind(update.clear_last_error)
        .bind(update.diagnostic_ref.as_deref())
        .bind(now_ms)
        .bind(update.next_retry_at_ms)
        .bind(i64::from(update.increment_retry_count))
        .bind(update.estimated_accrued_microusd)
        .bind(update.provider_reported_cost_microusd)
        .bind(now_ms)
        .bind(terminated)
        .bind(now_ms)
        .bind(lease.rental.rental_id.as_str())
        .bind(lease.owner.as_str())
        .execute(&mut *transaction)
        .await?;
        let updated = result.rows_affected() == 1;
        if updated && terminated {
            sqlx::query("DELETE FROM gpu_runtime_providers WHERE rental_id = ?")
                .bind(lease.rental.rental_id.as_str())
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(updated)
    }

    pub async fn release_gpu_rental_lease(
        &self,
        lease: &GpuRentalLease,
        next_retry_at_ms: i64,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE gpu_rentals
SET controller_lease_owner = NULL,
    controller_lease_until_ms = 0,
    next_retry_at_ms = ?,
    updated_at_ms = ?
WHERE rental_id = ? AND controller_lease_owner = ?
            "#,
        )
        .bind(next_retry_at_ms)
        .bind(now_ms)
        .bind(lease.rental.rental_id.as_str())
        .bind(lease.owner.as_str())
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn begin_gpu_rental_operation(
        &self,
        operation_id: &str,
        rental_id: &str,
        kind: GpuOperationKind,
        operation_sequence: i64,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
INSERT INTO gpu_rental_operations (
    operation_id,
    rental_id,
    operation_kind,
    operation_sequence,
    status,
    started_at_ms
) VALUES (?, ?, ?, ?, 'started', ?)
ON CONFLICT(rental_id, operation_kind, operation_sequence) DO NOTHING
            "#,
        )
        .bind(operation_id)
        .bind(rental_id)
        .bind(kind.as_str())
        .bind(operation_sequence)
        .bind(now_ms)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn finish_gpu_rental_operation(
        &self,
        operation_id: &str,
        status: &str,
        provider_request_id: Option<&str>,
        provider_resource_id: Option<&str>,
        sanitized_error: Option<&str>,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        if !matches!(status, "succeeded" | "failed" | "ambiguous") {
            return Err(anyhow::anyhow!("invalid GPU operation status: {status}"));
        }
        let result = sqlx::query(
            r#"
UPDATE gpu_rental_operations
SET status = ?,
    provider_request_id = ?,
    provider_resource_id = ?,
    sanitized_error = ?,
    completed_at_ms = ?
WHERE operation_id = ? AND status = 'started'
            "#,
        )
        .bind(status)
        .bind(provider_request_id)
        .bind(provider_resource_id)
        .bind(sanitized_error)
        .bind(now_ms)
        .bind(operation_id)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn record_gpu_notification_once(
        &self,
        rental_id: &str,
        state_sequence: i64,
        notification_kind: &str,
        now_ms: i64,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
INSERT INTO gpu_rental_notifications (
    rental_id, state_sequence, notification_kind, delivered_at_ms
) VALUES (?, ?, ?, ?)
ON CONFLICT(rental_id, state_sequence, notification_kind) DO NOTHING
            "#,
        )
        .bind(rental_id)
        .bind(state_sequence)
        .bind(notification_kind)
        .bind(now_ms)
        .execute(self.pool.as_ref())
        .await?;
        Ok(result.rows_affected() == 1)
    }
}

fn validate_gpu_rental_create(params: &GpuRentalCreateParams, now_ms: i64) -> anyhow::Result<()> {
    for (name, value) in [
        ("rental_id", params.rental_id.as_str()),
        ("installation_id", params.installation_id.as_str()),
        ("client_operation_id", params.client_operation_id.as_str()),
        ("provider", params.provider.as_str()),
        ("recipe_id", params.recipe_id.as_str()),
        ("recipe_revision", params.recipe_revision.as_str()),
        ("ownership_tag", params.ownership_tag.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow::anyhow!("{name} must not be empty"));
        }
    }
    let offer: serde_json::Value = serde_json::from_str(params.offer_snapshot_json.as_str())?;
    if !offer.is_object() {
        return Err(anyhow::anyhow!("offer snapshot must be a JSON object"));
    }
    if params.max_hourly_microusd <= 0 || params.max_total_microusd <= 0 {
        return Err(anyhow::anyhow!("GPU rental money limits must be positive"));
    }
    if params.terminate_at_ms <= now_ms {
        return Err(anyhow::anyhow!(
            "GPU rental termination time must be in the future"
        ));
    }
    if params
        .quote_expires_at_ms
        .is_some_and(|expires| expires < now_ms)
    {
        return Err(anyhow::anyhow!("GPU rental quote has expired"));
    }
    Ok(())
}

fn gpu_rental_matches_create(rental: &GpuRental, params: &GpuRentalCreateParams) -> bool {
    rental.installation_id == params.installation_id
        && rental.provider == params.provider
        && rental.recipe_id == params.recipe_id
        && rental.recipe_revision == params.recipe_revision
        && rental.offer_snapshot_json == params.offer_snapshot_json
        && rental.quote_expires_at_ms == params.quote_expires_at_ms
        && rental.max_hourly_microusd == params.max_hourly_microusd
        && rental.max_total_microusd == params.max_total_microusd
        && rental.terminate_at_ms == params.terminate_at_ms
        && rental.enforcement_class == params.enforcement_class
        && rental.ownership_tag == params.ownership_tag
}

#[cfg(test)]
#[path = "gpu_rentals_tests.rs"]
mod tests;
