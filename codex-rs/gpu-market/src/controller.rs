use crate::CreateInstanceRequest;
use crate::GpuInstance;
use crate::GpuInstanceState;
use crate::GpuOffer;
use crate::GpuProvider;
use crate::GpuProvisionPhase;
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
use codex_state::GpuRuntimeProviderUpsert;
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
    pub health_poll_ms: i64,
    pub batch_size: usize,
}

impl Default for ReconcileConfig {
    fn default() -> Self {
        Self {
            controller_id: format!("gpu-controller-{}", uuid::Uuid::new_v4()),
            lease_ttl_ms: 30_000,
            normal_poll_ms: 5_000,
            maximum_retry_ms: 5 * 60_000,
            health_poll_ms: 60_000,
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
    credentials: Option<Arc<dyn crate::GpuCredentialResolver>>,
    readiness: Option<Arc<dyn crate::GpuReadinessProbe>>,
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
            credentials: None,
            readiness: None,
        }
    }

    pub fn new_with_credentials(
        state: Arc<StateRuntime>,
        provider: P,
        recipes: RecipeCatalog,
        installation_id: String,
        config: ReconcileConfig,
        credentials: Arc<dyn crate::GpuCredentialResolver>,
    ) -> Self {
        Self {
            state,
            provider,
            recipes,
            installation_id,
            config,
            credentials: Some(credentials),
            readiness: None,
        }
    }

    pub fn new_with_runtime(
        state: Arc<StateRuntime>,
        provider: P,
        recipes: RecipeCatalog,
        installation_id: String,
        config: ReconcileConfig,
        credentials: Arc<dyn crate::GpuCredentialResolver>,
        readiness: Arc<dyn crate::GpuReadinessProbe>,
    ) -> Self {
        Self {
            state,
            provider,
            recipes,
            installation_id,
            config,
            credentials: Some(credentials),
            readiness: Some(readiness),
        }
    }

    pub async fn reconcile_due(&self, now_ms: i64) -> anyhow::Result<Vec<ControllerEvent>> {
        self.state.prune_terminal_gpu_runtime_providers().await?;
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
        mut lease: GpuRentalLease,
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

        // Cleanup reconciliation must remain able to prove provider absence. A provider may make
        // point lookup ambiguous after deletion, so do not let a billing refresh intercept the
        // inventory-backed termination path.
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

        // Refresh cost before enforcing limits or advancing any other billable rental.
        // Provisioning can take many minutes for a large model, so waiting until Ready would both
        // hide spend from the UI and enforce max_total against a stale zero value.
        if lease.rental.may_be_billable()
            && let Some(resource_id) = lease.rental.provider_resource_id.clone()
        {
            match self.provider.billing_state(resource_id).await {
                Ok(billing) => {
                    lease.rental.estimated_accrued_microusd = billing.estimated_accrued_microusd;
                    lease.rental.provider_reported_cost_microusd =
                        billing.provider_reported_cost_microusd;
                }
                Err(error) => return self.record_retry(lease, error, now_ms).await,
            }
        }

        if lease.rental.may_be_billable()
            && (now_ms >= lease.rental.terminate_at_ms
                || lease.rental.estimated_accrued_microusd >= lease.rental.max_total_microusd)
            && lease.rental.desired_state != GpuRentalState::TerminateRequested
        {
            self.apply_update(
                &lease,
                GpuRentalUpdate {
                    desired_state: Some(GpuRentalState::TerminateRequested),
                    observed_state: Some(GpuRentalState::TerminateRequested),
                    next_retry_at_ms: Some(now_ms),
                    ..GpuRentalUpdate::default()
                },
                now_ms,
            )
            .await?;
            return Ok(vec![ControllerEvent::Warning {
                rental_id: lease.rental.rental_id,
                code: "spend-limit".to_string(),
                message:
                    "Rental reached its authorized spend or time limit; termination requested."
                        .to_string(),
            }]);
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
        let Some(recipe) = self.recipes.get(lease.rental.recipe_id.as_str()) else {
            return self
                .record_terminal_failure(
                    lease,
                    "recipe-missing",
                    "Pinned GPU recipe is missing.",
                    now_ms,
                )
                .await;
        };
        let deadline_ms = lease
            .rental
            .created_at_ms
            .saturating_add(i64::try_from(recipe.download_deadline_ms).unwrap_or(i64::MAX))
            .saturating_add(i64::try_from(recipe.startup_deadline_ms).unwrap_or(i64::MAX));
        if now_ms > deadline_ms {
            return self
                .record_terminal_failure(
                    lease,
                    "provision-deadline-exceeded",
                    "The pinned recipe did not become ready before its authorized provisioning deadline.",
                    now_ms,
                )
                .await;
        }
        let instance = match self.provider.get_instance(resource_id.clone()).await {
            Ok(Some(instance)) => instance,
            Ok(None) => {
                return self
                    .record_terminal_failure(
                        lease,
                        "provider-resource-missing",
                        "The provider resource disappeared during provisioning.",
                        now_ms,
                    )
                    .await;
            }
            Err(error) => return self.record_retry(lease, error, now_ms).await,
        };
        if instance.state != GpuInstanceState::Running {
            if matches!(
                instance.state,
                GpuInstanceState::Failed | GpuInstanceState::Stopped
            ) {
                return self
                    .record_terminal_failure(
                        lease,
                        "provider-instance-failed",
                        "The provider resource stopped before readiness completed.",
                        now_ms,
                    )
                    .await;
            }
            self.release_later(&lease, now_ms).await?;
            return Ok(Vec::new());
        }
        if instance.gpu_model != recipe.hardware.gpu_model
            || instance.gpu_count < recipe.hardware.gpu_count
            || instance
                .host_ram_mib
                .is_none_or(|ram| ram < recipe.hardware.minimum_host_ram_mib)
            || instance
                .disk_gib
                .is_none_or(|disk| disk < recipe.hardware.minimum_disk_gib)
            || (recipe.hardware.requires_high_bandwidth_interconnect
                && instance.high_bandwidth_interconnect == Some(false))
        {
            return self
                .record_terminal_failure(
                    lease,
                    "allocated-hardware-mismatch",
                    "The allocated GPU model, count, or topology does not match the pinned recipe.",
                    now_ms,
                )
                .await;
        }

        if lease.rental.provision_step.is_none() {
            let digest = manifest_step_digest(recipe.revision.as_str());
            if !self
                .state
                .begin_gpu_provision_step(
                    lease.rental.rental_id.as_str(),
                    "01-provider-bootstrap",
                    digest.as_str(),
                    now_ms,
                )
                .await?
            {
                return self
                    .record_terminal_failure(
                        lease,
                        "provision-manifest-conflict",
                        "The persisted provision step does not match the pinned recipe revision.",
                        now_ms,
                    )
                    .await;
            }
            self.state
                .finish_gpu_provision_step(
                    lease.rental.rental_id.as_str(),
                    "01-provider-bootstrap",
                    /*succeeded*/ true,
                    Some(
                        serde_json::json!({
                            "resource_id": resource_id,
                            "gpu_model": instance.gpu_model,
                            "gpu_count": instance.gpu_count,
                            "host_ram_mib": instance.host_ram_mib,
                            "disk_gib": instance.disk_gib,
                            "high_bandwidth_interconnect": instance.high_bandwidth_interconnect,
                            "runtime_topology_gate": recipe.hardware.requires_high_bandwidth_interconnect
                                && instance.high_bandwidth_interconnect.is_none(),
                        })
                        .to_string()
                        .as_str(),
                    ),
                    /*sanitized_error*/ None,
                    now_ms,
                )
                .await?;
        }

        let provision_phase = match self.provider.provision_phase(&instance).await {
            Ok(phase) => phase,
            Err(error) => {
                tracing::debug!(
                    rental_id = %lease.rental.rental_id,
                    message = %error.safe_message,
                    "GPU provisioning phase is not observable yet"
                );
                None
            }
        };
        if let Some(phase) = provision_phase {
            let observed_state = provision_phase_state(phase);
            let phase_changed = lease.rental.observed_state != observed_state
                || lease.rental.provision_step.as_deref() != Some(phase.as_str());
            if phase != GpuProvisionPhase::EndpointProbing || phase_changed {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        observed_state: Some(observed_state),
                        provision_step: Some(phase.as_str().to_string()),
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                        clear_last_error: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                return Ok(phase_changed
                    .then_some(ControllerEvent::StateChanged {
                        rental_id: lease.rental.rental_id,
                        state: observed_state,
                    })
                    .into_iter()
                    .collect());
            }
        }

        if lease.rental.observed_state != GpuRentalState::Probing {
            self.apply_update(
                &lease,
                GpuRentalUpdate {
                    observed_state: Some(GpuRentalState::Probing),
                    provision_step: Some("02-readiness".to_string()),
                    next_retry_at_ms: Some(now_ms),
                    clear_last_error: true,
                    ..GpuRentalUpdate::default()
                },
                now_ms,
            )
            .await?;
            return Ok(vec![ControllerEvent::StateChanged {
                rental_id: lease.rental.rental_id,
                state: GpuRentalState::Probing,
            }]);
        }

        let endpoint = match self
            .provider
            .secure_endpoint_base_url(&instance, recipe.inference_port)
            .await
        {
            Ok(endpoint) => endpoint,
            Err(error) => {
                if error.retryable() {
                    return self.record_retry(lease, error, now_ms).await;
                }
                let message = error.safe_message.clone();
                return self
                    .record_terminal_failure(lease, "secure-endpoint-unavailable", &message, now_ms)
                    .await;
            }
        };
        let (Some(credentials), Some(readiness)) = (&self.credentials, &self.readiness) else {
            return self
                .record_terminal_failure(
                    lease,
                    "readiness-runtime-unavailable",
                    "The authenticated readiness runtime is unavailable.",
                    now_ms,
                )
                .await;
        };
        let token = match credentials.ensure_rental_endpoint_token(lease.rental.rental_id.as_str())
        {
            Ok(credential) => credential.secret,
            Err(error) if error.retryable() => {
                return self
                    .record_retry(
                        lease,
                        ProviderError::new(
                            ProviderErrorKind::Retryable,
                            "The per-rental endpoint credential store is temporarily unavailable.",
                        ),
                        now_ms,
                    )
                    .await;
            }
            Err(_) => {
                return self
                    .record_terminal_failure(
                        lease,
                        "endpoint-token-unavailable",
                        "The per-rental endpoint token is unavailable.",
                        now_ms,
                    )
                    .await;
            }
        };
        let probe_result = readiness
            .probe(endpoint.as_str(), recipe.served_model_id(), &token)
            .await;
        let report = match probe_result {
            Ok(report) if report.ready() => report,
            Ok(_) => {
                return self
                    .record_retry(
                        lease,
                        ProviderError::new(
                            ProviderErrorKind::Retryable,
                            "The authenticated endpoint is not ready yet.",
                        ),
                        now_ms,
                    )
                    .await;
            }
            Err(_) => {
                return self
                    .record_retry(
                        lease,
                        ProviderError::new(
                            ProviderErrorKind::Retryable,
                            "The authenticated endpoint readiness probe could not complete.",
                        ),
                        now_ms,
                    )
                    .await;
            }
        };
        let probe_digest = manifest_step_digest(recipe.probe_contract.as_str());
        if !self
            .state
            .begin_gpu_provision_step(
                lease.rental.rental_id.as_str(),
                "02-readiness",
                probe_digest.as_str(),
                now_ms,
            )
            .await?
        {
            return self
                .record_terminal_failure(
                    lease,
                    "readiness-contract-conflict",
                    "The persisted readiness step does not match the pinned probe contract.",
                    now_ms,
                )
                .await;
        }
        self.state
            .finish_gpu_provision_step(
                lease.rental.rental_id.as_str(),
                "02-readiness",
                /*succeeded*/ true,
                Some(&serde_json::to_string(&serde_json::json!({
                    "authenticated": report.rejects_missing_token && report.rejects_wrong_token,
                    "model_identity": report.model_identity_ok,
                    "chat": report.chat_ok,
                    "streaming": report.streaming_ok,
                    "cancellation": report.cancellation_ok,
                    "tool_call": report.tool_call_ok,
                }))?),
                /*sanitized_error*/ None,
                now_ms,
            )
            .await?;
        let endpoint_provider_id = format!("gpu-{}", lease.rental.rental_id);
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                desired_state: Some(GpuRentalState::Ready),
                observed_state: Some(GpuRentalState::Ready),
                provision_step: Some("ready".to_string()),
                endpoint_base_url: Some(endpoint.clone()),
                endpoint_provider_id: Some(endpoint_provider_id.clone()),
                next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                clear_last_error: true,
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        self.state
            .upsert_gpu_runtime_provider(
                &GpuRuntimeProviderUpsert {
                    rental_id: lease.rental.rental_id.clone(),
                    provider_id: endpoint_provider_id,
                    base_url: endpoint,
                    model_id: recipe.served_model_id().to_string(),
                    wire_api: recipe.wire_api.clone(),
                    health: "ready".to_string(),
                    display_hourly_microusd: quoted_hourly_microusd(&lease.rental),
                    maximum_context_tokens: i64::try_from(recipe.maximum_context_tokens)
                        .unwrap_or(i64::MAX),
                    catalog_sequence: lease.rental.state_sequence.saturating_add(1),
                },
                now_ms,
            )
            .await?;
        Ok(vec![ControllerEvent::StateChanged {
            rental_id: lease.rental.rental_id,
            state: GpuRentalState::Ready,
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
        let instance = match self.provider.get_instance(resource_id.clone()).await {
            Ok(None) => {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        desired_state: Some(GpuRentalState::TerminatedConfirmed),
                        observed_state: Some(GpuRentalState::TerminatedConfirmed),
                        next_retry_at_ms: Some(i64::MAX),
                        clear_last_error: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                return Ok(vec![ControllerEvent::StateChanged {
                    rental_id: lease.rental.rental_id,
                    state: GpuRentalState::TerminatedConfirmed,
                }]);
            }
            Ok(Some(instance))
                if matches!(
                    instance.state,
                    GpuInstanceState::Stopped | GpuInstanceState::Failed
                ) =>
            {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        desired_state: Some(GpuRentalState::TerminateRequested),
                        observed_state: Some(GpuRentalState::TerminateRequested),
                        last_error_code: Some("provider-side-stop".to_string()),
                        last_error_message: Some(
                            "The provider resource stopped outside PFTerminal; cleanup was requested."
                                .to_string(),
                        ),
                        next_retry_at_ms: Some(now_ms),
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                return Ok(vec![ControllerEvent::Warning {
                    rental_id: lease.rental.rental_id,
                    code: "provider-side-stop".to_string(),
                    message: "Provider-side stop detected; PFTerminal is confirming cleanup."
                        .to_string(),
                }]);
            }
            Ok(Some(instance)) => instance,
            Err(error) => return self.record_retry(lease, error, now_ms).await,
        };
        let recipe = self.recipes.get(lease.rental.recipe_id.as_str());
        // The endpoint transport is controller-owned. Re-resolve it on every health cycle so a
        // restarted controller recreates its local SSH forward instead of retaining a dead
        // process-local address in the durable provider catalog.
        let endpoint = match recipe {
            Some(recipe) => self
                .provider
                .secure_endpoint_base_url(&instance, recipe.inference_port)
                .await
                .ok(),
            None => None,
        };
        let health = match (
            recipe,
            endpoint.as_deref(),
            self.credentials.as_ref(),
            self.readiness.as_ref(),
        ) {
            (Some(recipe), Some(endpoint), Some(credentials), Some(readiness)) => match credentials
                .resolve(&crate::GpuCredentialKind::RentalEndpointToken {
                    rental_id: lease.rental.rental_id.clone(),
                }) {
                Ok(credential) if lease.rental.observed_state == GpuRentalState::Ready => readiness
                    .probe_health(endpoint, recipe.served_model_id(), &credential.secret)
                    .await
                    .unwrap_or(false),
                Ok(credential) => readiness
                    .probe(endpoint, recipe.served_model_id(), &credential.secret)
                    .await
                    .is_ok_and(|report| report.ready()),
                Err(_) => false,
            },
            _ => false,
        };
        let observed_state = if health {
            GpuRentalState::Ready
        } else {
            GpuRentalState::Degraded
        };
        let state_changed = observed_state != lease.rental.observed_state;
        let endpoint_changed = endpoint
            .as_deref()
            .is_some_and(|endpoint| lease.rental.endpoint_base_url.as_deref() != Some(endpoint));
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                observed_state: state_changed.then_some(observed_state),
                endpoint_base_url: endpoint_changed.then(|| endpoint.clone()).flatten(),
                last_error_code: (!health).then(|| "endpoint-degraded".to_string()),
                last_error_message: (!health).then(|| {
                    "The authenticated endpoint failed its readiness contract.".to_string()
                }),
                next_retry_at_ms: Some(now_ms.saturating_add(self.config.health_poll_ms)),
                clear_last_error: health,
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        if endpoint_changed {
            let (Some(recipe), Some(endpoint), Some(maximum_context_tokens)) = (
                recipe,
                endpoint,
                recipe.and_then(|recipe| i64::try_from(recipe.maximum_context_tokens).ok()),
            ) else {
                return Ok(Vec::new());
            };
            self.state
                .refresh_gpu_runtime_provider(
                    &GpuRuntimeProviderUpsert {
                        rental_id: lease.rental.rental_id.clone(),
                        provider_id: lease
                            .rental
                            .endpoint_provider_id
                            .clone()
                            .unwrap_or_else(|| format!("gpu-{}", lease.rental.rental_id)),
                        base_url: endpoint,
                        model_id: recipe.served_model_id().to_string(),
                        wire_api: recipe.wire_api.clone(),
                        health: if health { "ready" } else { "degraded" }.to_string(),
                        display_hourly_microusd: quoted_hourly_microusd(&lease.rental),
                        maximum_context_tokens,
                        catalog_sequence: lease.rental.state_sequence.saturating_add(1),
                    },
                    now_ms,
                )
                .await?;
        } else if let Some(maximum_context_tokens) =
            recipe.and_then(|recipe| i64::try_from(recipe.maximum_context_tokens).ok())
        {
            self.state
                .set_gpu_runtime_provider_health_and_price(
                    lease.rental.rental_id.as_str(),
                    if health { "ready" } else { "degraded" },
                    quoted_hourly_microusd(&lease.rental),
                    maximum_context_tokens,
                    now_ms,
                )
                .await?;
        } else {
            self.state
                .set_gpu_runtime_provider_health(
                    lease.rental.rental_id.as_str(),
                    "degraded",
                    now_ms,
                )
                .await?;
        }
        if state_changed {
            Ok(vec![ControllerEvent::StateChanged {
                rental_id: lease.rental.rental_id,
                state: observed_state,
            }])
        } else {
            Ok(Vec::new())
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
        let retry_ms = self.retry_delay_ms(&lease.rental, error.retry_after_ms);
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
        let billing_risk = lease.rental.may_be_billable();
        let terminal_state = if billing_risk {
            GpuRentalState::TerminationUnconfirmed
        } else {
            GpuRentalState::Failed
        };
        let desired_state = if billing_risk {
            GpuRentalState::TerminateRequested
        } else {
            GpuRentalState::Failed
        };
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                desired_state: Some(desired_state),
                observed_state: Some(terminal_state),
                last_error_code: Some(code.to_string()),
                last_error_message: Some(message.to_string()),
                next_retry_at_ms: Some(if billing_risk { now_ms } else { i64::MAX }),
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        if billing_risk {
            Ok(vec![ControllerEvent::Warning {
                rental_id: lease.rental.rental_id,
                code: code.to_string(),
                message: format!(
                    "{message} Cleanup is required before billing can be considered stopped."
                ),
            }])
        } else {
            Ok(vec![ControllerEvent::StateChanged {
                rental_id: lease.rental.rental_id,
                state: GpuRentalState::Failed,
            }])
        }
    }

    pub(super) fn retry_delay_ms(&self, rental: &GpuRental, retry_after_ms: Option<i64>) -> i64 {
        if let Some(retry_after_ms) = retry_after_ms {
            return retry_after_ms.clamp(0, self.config.maximum_retry_ms);
        }
        let exponent = u32::try_from(rental.retry_count.clamp(0, 6)).unwrap_or(6);
        let base_ms = 1_000_i64.saturating_mul(2_i64.saturating_pow(exponent));
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in rental
            .rental_id
            .as_bytes()
            .iter()
            .copied()
            .chain(rental.retry_count.to_le_bytes().iter().copied())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // Stable 75%-125% jitter prevents synchronized retries without volatile RNG state that
        // changes after a process restart.
        let jitter_per_mille = 750_i64 + i64::try_from(hash % 501).unwrap_or(0);
        base_ms
            .saturating_mul(jitter_per_mille)
            .saturating_div(1_000)
            .clamp(1, self.config.maximum_retry_ms)
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
        mut update: GpuRentalUpdate,
        now_ms: i64,
    ) -> anyhow::Result<()> {
        // `reconcile_lease` refreshes these fields in its local lease before routing to the
        // state-specific handler. Fold them into whichever atomic transition releases the
        // controller lease so every billable state, including probing, reports current spend.
        if lease.rental.may_be_billable() {
            update
                .estimated_accrued_microusd
                .get_or_insert(lease.rental.estimated_accrued_microusd);
            if update.provider_reported_cost_microusd.is_none() {
                update.provider_reported_cost_microusd =
                    lease.rental.provider_reported_cost_microusd;
            }
        }
        if !self.state.update_gpu_rental(lease, &update, now_ms).await? {
            return Err(anyhow::anyhow!(
                "GPU rental lease was lost before its state update"
            ));
        }
        Ok(())
    }
}

fn manifest_step_digest(value: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn quoted_hourly_microusd(rental: &GpuRental) -> i64 {
    serde_json::from_str::<GpuOffer>(rental.offer_snapshot_json.as_str())
        .ok()
        .map(|offer| offer.hourly_microusd)
        .filter(|price| *price > 0 && *price <= rental.max_hourly_microusd)
        .unwrap_or(rental.max_hourly_microusd)
}

fn provision_phase_state(phase: GpuProvisionPhase) -> GpuRentalState {
    match phase {
        GpuProvisionPhase::HardwareCheck
        | GpuProvisionPhase::RuntimeSetup
        | GpuProvisionPhase::RuntimeBuild => GpuRentalState::Bootstrapping,
        GpuProvisionPhase::ModelDownload | GpuProvisionPhase::ModelVerification => {
            GpuRentalState::Downloading
        }
        GpuProvisionPhase::ModelLoading => GpuRentalState::Starting,
        GpuProvisionPhase::EndpointProbing => GpuRentalState::Probing,
    }
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;
