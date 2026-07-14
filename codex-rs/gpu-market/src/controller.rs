use crate::CreateInstanceRequest;
use crate::GpuInstance;
use crate::GpuInstanceState;
use crate::GpuOffer;
use crate::GpuProvider;
use crate::OwnedInstanceQuery;
use crate::ProviderError;
use crate::ProviderErrorKind;
use crate::RecipeCatalog;
use crate::SearchOffersRequest;
use codex_state::GpuOperationKind;
use codex_state::GpuRental;
use codex_state::GpuRentalLease;
use codex_state::GpuRentalState;
use codex_state::GpuRentalUpdate;
use codex_state::StateRuntime;
use std::sync::Arc;

#[path = "controller_create.rs"]
mod create;
#[path = "controller_termination.rs"]
mod termination;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileConfig {
    pub controller_id: String,
    pub lease_ttl_ms: i64,
    pub normal_poll_ms: i64,
    pub maximum_retry_ms: i64,
    pub batch_size: usize,
}

impl Default for ReconcileConfig {
    fn default() -> Self {
        Self {
            controller_id: format!("gpu-controller-{}", uuid::Uuid::new_v4()),
            lease_ttl_ms: 30_000,
            normal_poll_ms: 5_000,
            maximum_retry_ms: 5 * 60_000,
            batch_size: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerEvent {
    StateChanged {
        rental_id: String,
        state: GpuRentalState,
    },
    NeedsProvisioning {
        rental_id: String,
        provider_resource_id: String,
    },
    Warning {
        rental_id: String,
        code: String,
        message: String,
    },
}

pub struct GpuRentalController<P> {
    state: Arc<StateRuntime>,
    provider: P,
    recipes: RecipeCatalog,
    installation_id: String,
    config: ReconcileConfig,
}

impl<P> GpuRentalController<P>
where
    P: GpuProvider,
{
    pub fn new(
        state: Arc<StateRuntime>,
        provider: P,
        recipes: RecipeCatalog,
        installation_id: String,
        config: ReconcileConfig,
    ) -> Self {
        Self {
            state,
            provider,
            recipes,
            installation_id,
            config,
        }
    }

    pub async fn reconcile_due(&self, now_ms: i64) -> anyhow::Result<Vec<ControllerEvent>> {
        let provider = self.provider.capabilities().provider;
        let leases = self
            .state
            .claim_due_gpu_rentals_for_provider(
                self.config.controller_id.as_str(),
                provider.as_str(),
                now_ms,
                self.config.lease_ttl_ms,
                self.config.batch_size,
            )
            .await?;
        let mut events = Vec::new();
        for lease in leases {
            events.extend(self.reconcile_lease(lease, now_ms).await?);
        }
        Ok(events)
    }

    async fn reconcile_lease(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let capabilities = self.provider.capabilities();
        if lease.rental.provider != capabilities.provider {
            self.release_later(&lease, now_ms).await?;
            return Ok(Vec::new());
        }
        if !capabilities.supports_inventory || !capabilities.supports_ownership_tags {
            return self
                .record_terminal_failure(
                    lease,
                    "provider-recovery-unsupported",
                    "Provider cannot safely reconcile PFTerminal-owned resources.",
                    now_ms,
                )
                .await;
        }

        if lease.rental.observed_state.may_be_billable()
            && (now_ms >= lease.rental.terminate_at_ms
                || lease.rental.estimated_accrued_microusd >= lease.rental.max_total_microusd)
            && lease.rental.desired_state != GpuRentalState::TerminateRequested
        {
            self.state
                .request_gpu_rental_termination(lease.rental.rental_id.as_str(), now_ms)
                .await?;
            self.state
                .release_gpu_rental_lease(&lease, now_ms, now_ms)
                .await?;
            return Ok(vec![ControllerEvent::Warning {
                rental_id: lease.rental.rental_id,
                code: "spend-limit".to_string(),
                message:
                    "Rental reached its authorized spend or time limit; termination requested."
                        .to_string(),
            }]);
        }

        if lease.rental.desired_state == GpuRentalState::TerminateRequested
            || matches!(
                lease.rental.observed_state,
                GpuRentalState::TerminateRequested
                    | GpuRentalState::Terminating
                    | GpuRentalState::TerminationUnconfirmed
            )
        {
            return self.reconcile_termination(lease, now_ms).await;
        }

        match lease.rental.observed_state {
            GpuRentalState::Quoted
                if lease.rental.desired_state == GpuRentalState::CreatePending =>
            {
                self.reconcile_create(lease, now_ms).await
            }
            GpuRentalState::Allocating | GpuRentalState::Reconciling => {
                self.reconcile_allocating(lease, now_ms).await
            }
            GpuRentalState::Bootstrapping
            | GpuRentalState::Downloading
            | GpuRentalState::Starting
            | GpuRentalState::Probing => self.request_provisioning(lease, now_ms).await,
            GpuRentalState::Ready | GpuRentalState::Degraded => {
                self.reconcile_ready(lease, now_ms).await
            }
            GpuRentalState::Draft
            | GpuRentalState::Quoted
            | GpuRentalState::CreatePending
            | GpuRentalState::TerminateRequested
            | GpuRentalState::Terminating
            | GpuRentalState::TerminatedConfirmed
            | GpuRentalState::TerminationUnconfirmed
            | GpuRentalState::Orphaned
            | GpuRentalState::Failed => {
                self.release_later(&lease, now_ms).await?;
                Ok(Vec::new())
            }
        }
    }

    async fn request_provisioning(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let Some(resource_id) = lease.rental.provider_resource_id.clone() else {
            return self
                .record_terminal_failure(
                    lease,
                    "resource-id-missing",
                    "Provisioning cannot continue without a provider resource id.",
                    now_ms,
                )
                .await;
        };
        self.release_later(&lease, now_ms).await?;
        Ok(vec![ControllerEvent::NeedsProvisioning {
            rental_id: lease.rental.rental_id,
            provider_resource_id: resource_id,
        }])
    }

    async fn reconcile_ready(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let Some(resource_id) = lease.rental.provider_resource_id.clone() else {
            return self
                .record_terminal_failure(
                    lease,
                    "resource-id-missing",
                    "Ready rental is missing its provider resource id.",
                    now_ms,
                )
                .await;
        };
        match self.provider.billing_state(resource_id).await {
            Ok(billing) => {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        estimated_accrued_microusd: Some(billing.estimated_accrued_microusd),
                        provider_reported_cost_microusd: billing.provider_reported_cost_microusd,
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                        clear_last_error: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(Vec::new())
            }
            Err(error) => self.record_retry(lease, error, now_ms).await,
        }
    }

    async fn find_owned(&self, rental: &GpuRental) -> Result<Option<GpuInstance>, ProviderError> {
        let instances = self
            .provider
            .list_owned_instances(OwnedInstanceQuery {
                installation_id: self.installation_id.clone(),
                ownership_tag: Some(rental.ownership_tag.clone()),
            })
            .await?;
        match instances.as_slice() {
            [] => Ok(None),
            [instance] => Ok(Some(instance.clone())),
            _ => Err(ProviderError::new(
                ProviderErrorKind::Permanent,
                "Multiple provider resources have the same PFTerminal ownership tag.",
            )),
        }
    }

    async fn record_retry(
        &self,
        lease: GpuRentalLease,
        error: ProviderError,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let retry_ms = error.retry_after_ms.unwrap_or_else(|| {
            let exponent = u32::try_from(lease.rental.retry_count.clamp(0, 6)).unwrap_or(6);
            1_000_i64.saturating_mul(2_i64.saturating_pow(exponent))
        });
        let next_retry = now_ms.saturating_add(retry_ms.min(self.config.maximum_retry_ms));
        let state = if error.kind == ProviderErrorKind::Ambiguous {
            Some(GpuRentalState::Reconciling)
        } else if lease.rental.observed_state == GpuRentalState::Terminating {
            Some(GpuRentalState::TerminationUnconfirmed)
        } else {
            None
        };
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                observed_state: state,
                last_error_code: Some(format!("{:?}", error.kind).to_ascii_lowercase()),
                last_error_message: Some(error.safe_message),
                diagnostic_ref: error.diagnostic_ref,
                next_retry_at_ms: Some(next_retry),
                increment_retry_count: true,
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        Ok(Vec::new())
    }

    async fn record_terminal_failure(
        &self,
        lease: GpuRentalLease,
        code: &str,
        message: &str,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                desired_state: Some(GpuRentalState::Failed),
                observed_state: Some(GpuRentalState::Failed),
                last_error_code: Some(code.to_string()),
                last_error_message: Some(message.to_string()),
                next_retry_at_ms: Some(i64::MAX),
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        Ok(vec![ControllerEvent::StateChanged {
            rental_id: lease.rental.rental_id,
            state: GpuRentalState::Failed,
        }])
    }

    async fn release_later(&self, lease: &GpuRentalLease, now_ms: i64) -> anyhow::Result<()> {
        self.state
            .release_gpu_rental_lease(
                lease,
                now_ms.saturating_add(self.config.normal_poll_ms),
                now_ms,
            )
            .await?;
        Ok(())
    }

    async fn apply_update(
        &self,
        lease: &GpuRentalLease,
        update: GpuRentalUpdate,
        now_ms: i64,
    ) -> anyhow::Result<()> {
        if !self.state.update_gpu_rental(lease, &update, now_ms).await? {
            return Err(anyhow::anyhow!(
                "GPU rental lease was lost before its state update"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;
