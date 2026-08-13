use super::*;

impl<P> GpuRentalController<P>
where
    P: GpuProvider,
{
    pub(super) async fn reconcile_create(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        match self.find_owned(&lease.rental).await {
            Ok(Some(instance)) => return self.adopt_instance(lease, instance, now_ms).await,
            Ok(None) => {}
            Err(error) => return self.record_retry(lease, error, now_ms).await,
        }

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
        if !recipe.manifest_verified {
            return self
                .record_terminal_failure(
                    lease,
                    "recipe-unverified",
                    "Pinned GPU recipe manifest has not been verified.",
                    now_ms,
                )
                .await;
        }
        let offer: GpuOffer = serde_json::from_str(lease.rental.offer_snapshot_json.as_str())?;
        if let Err(error) = offer.validate_for(
            &SearchOffersRequest {
                hardware: recipe.hardware.clone(),
                allow_interruptible: false,
                require_verified_or_secure: true,
                maximum_hourly_microusd: lease.rental.max_hourly_microusd,
            },
            now_ms,
        ) {
            let message = error.safe_message.clone();
            return self
                .record_terminal_failure(lease, "offer-invalid", &message, now_ms)
                .await;
        }

        let Some(credentials) = self.credentials.as_ref() else {
            return self
                .record_terminal_failure(
                    lease,
                    "endpoint-token-unavailable",
                    "The scoped endpoint credential resolver is unavailable.",
                    now_ms,
                )
                .await;
        };
        let endpoint_token =
            match credentials.ensure_rental_endpoint_token(lease.rental.rental_id.as_str()) {
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
        let huggingface_token = if recipe.requires_huggingface_token {
            match credentials.resolve(&crate::GpuCredentialKind::HuggingFaceToken) {
                Ok(credential) => Some(credential.secret),
                Err(_) => {
                    return self
                        .record_terminal_failure(
                            lease,
                            "huggingface-token-unavailable",
                            "The pinned recipe requires Hugging Face access, but its scoped token is unavailable.",
                            now_ms,
                        )
                        .await;
                }
            }
        } else {
            None
        };
        let operation_id = format!("gpu-create-{}", uuid::Uuid::new_v4());
        let began = self
            .state
            .begin_gpu_rental_operation(
                operation_id.as_str(),
                lease.rental.rental_id.as_str(),
                GpuOperationKind::Create,
                lease.rental.state_sequence,
                now_ms,
            )
            .await?;
        if !began {
            return self
                .record_retry(
                    lease,
                    ProviderError::new(
                        ProviderErrorKind::Ambiguous,
                        "A prior create is unresolved; reconciling provider inventory.",
                    ),
                    now_ms,
                )
                .await;
        }

        let request = CreateInstanceRequest {
            offer,
            client_operation_id: lease.rental.client_operation_id.clone(),
            ownership_tag: lease.rental.ownership_tag.clone(),
            image: recipe.image.clone(),
            disk_gib: recipe.hardware.minimum_disk_gib,
            launch_command: recipe.launch_command.clone(),
            inference_port: recipe.inference_port,
            endpoint_token,
            huggingface_token,
        };
        match self.provider.create_instance(request).await {
            Ok(instance) => {
                self.state
                    .finish_gpu_rental_operation(
                        operation_id.as_str(),
                        "succeeded",
                        /*provider_request_id*/ None,
                        Some(instance.resource_id.as_str()),
                        /*sanitized_error*/ None,
                        now_ms,
                    )
                    .await?;
                self.adopt_instance(lease, instance, now_ms).await
            }
            Err(error) => {
                let status = if error.kind == ProviderErrorKind::Ambiguous {
                    "ambiguous"
                } else {
                    "failed"
                };
                self.state
                    .finish_gpu_rental_operation(
                        operation_id.as_str(),
                        status,
                        /*provider_request_id*/ None,
                        /*provider_resource_id*/ None,
                        Some(error.safe_message.as_str()),
                        now_ms,
                    )
                    .await?;
                if error.retryable() {
                    self.record_retry(lease, error, now_ms).await
                } else {
                    let code = match error.kind {
                        ProviderErrorKind::OfferUnavailable => "offer-unavailable",
                        ProviderErrorKind::PriceDrift => "price-drift",
                        ProviderErrorKind::Unauthorized => "provider-unauthorized",
                        ProviderErrorKind::InsufficientFunds => "insufficient-funds",
                        _ => "create-rejected",
                    };
                    let message = error.safe_message.clone();
                    self.record_terminal_failure(lease, code, &message, now_ms)
                        .await
                }
            }
        }
    }

    pub(super) async fn reconcile_allocating(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let instance = if let Some(resource_id) = &lease.rental.provider_resource_id {
            self.provider.get_instance(resource_id.clone()).await
        } else {
            self.find_owned(&lease.rental).await
        };
        match instance {
            Ok(Some(instance)) if instance.state == GpuInstanceState::Running => {
                let resource_id = instance.resource_id;
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        observed_state: Some(GpuRentalState::Bootstrapping),
                        provider_resource_id: Some(resource_id.clone()),
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                        clear_last_error: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(vec![ControllerEvent::NeedsProvisioning {
                    rental_id: lease.rental.rental_id,
                    provider_resource_id: resource_id,
                }])
            }
            Ok(Some(instance)) if instance.state == GpuInstanceState::Failed => {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        desired_state: Some(GpuRentalState::TerminateRequested),
                        observed_state: Some(GpuRentalState::TerminateRequested),
                        provider_resource_id: Some(instance.resource_id),
                        last_error_code: Some("provider-instance-failed".to_string()),
                        last_error_message: Some(
                            "Provider reported that the instance failed.".to_string(),
                        ),
                        next_retry_at_ms: Some(now_ms),
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(Vec::new())
            }
            Ok(Some(instance)) => {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        provider_resource_id: Some(instance.resource_id),
                        observed_state: Some(GpuRentalState::Allocating),
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                        clear_last_error: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(Vec::new())
            }
            Ok(None) if lease.rental.provider_resource_id.is_none() => {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        observed_state: Some(GpuRentalState::Orphaned),
                        last_error_code: Some("ambiguous-create-unresolved".to_string()),
                        last_error_message: Some(
                            "Create outcome remains ambiguous and no owned resource is visible; human resolution is required."
                                .to_string(),
                        ),
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.maximum_retry_ms)),
                        increment_retry_count: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(vec![ControllerEvent::Warning {
                    rental_id: lease.rental.rental_id,
                    code: "ambiguous-create-unresolved".to_string(),
                    message: "GPU create outcome needs human resolution; no duplicate was created."
                        .to_string(),
                }])
            }
            Ok(None) => {
                self.record_terminal_failure(
                    lease,
                    "instance-disappeared",
                    "Provider instance disappeared before becoming ready.",
                    now_ms,
                )
                .await
            }
            Err(error) => self.record_retry(lease, error, now_ms).await,
        }
    }

    async fn adopt_instance(
        &self,
        lease: GpuRentalLease,
        instance: GpuInstance,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let state = if instance.state == GpuInstanceState::Running {
            GpuRentalState::Bootstrapping
        } else {
            GpuRentalState::Allocating
        };
        let resource_id = instance.resource_id;
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                observed_state: Some(state),
                provider_resource_id: Some(resource_id.clone()),
                next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                clear_last_error: true,
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        let mut events = vec![ControllerEvent::StateChanged {
            rental_id: lease.rental.rental_id.clone(),
            state,
        }];
        if state == GpuRentalState::Bootstrapping {
            events.push(ControllerEvent::NeedsProvisioning {
                rental_id: lease.rental.rental_id,
                provider_resource_id: resource_id,
            });
        }
        Ok(events)
    }
}
