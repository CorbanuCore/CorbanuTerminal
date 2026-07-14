use super::*;

impl<P> GpuRentalController<P>
where
    P: GpuProvider,
{
    pub(super) async fn reconcile_termination(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let resource_id = if let Some(resource_id) = lease.rental.provider_resource_id.clone() {
            Some(resource_id)
        } else {
            match self.find_owned(&lease.rental).await {
                Ok(Some(instance)) => Some(instance.resource_id),
                Ok(None) => None,
                Err(error) => return self.record_retry(lease, error, now_ms).await,
            }
        };
        let Some(resource_id) = resource_id else {
            return self.confirm_terminated(lease, now_ms).await;
        };

        match self.provider.get_instance(resource_id.clone()).await {
            Ok(None) => self.confirm_terminated(lease, now_ms).await,
            Ok(Some(_)) if lease.rental.observed_state == GpuRentalState::Terminating => {
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        provider_resource_id: Some(resource_id),
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(Vec::new())
            }
            Ok(Some(_)) => self.send_termination(lease, resource_id, now_ms).await,
            Err(error) => self.record_termination_retry(lease, error, now_ms).await,
        }
    }

    async fn send_termination(
        &self,
        lease: GpuRentalLease,
        resource_id: String,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let operation_id = format!("gpu-terminate-{}", uuid::Uuid::new_v4());
        let began = self
            .state
            .begin_gpu_rental_operation(
                operation_id.as_str(),
                lease.rental.rental_id.as_str(),
                GpuOperationKind::Terminate,
                lease.rental.state_sequence,
                now_ms,
            )
            .await?;
        if !began {
            return self
                .record_termination_retry(
                    lease,
                    ProviderError::new(
                        ProviderErrorKind::Ambiguous,
                        "A prior termination request is unresolved.",
                    ),
                    now_ms,
                )
                .await;
        }
        match self.provider.terminate_instance(resource_id.clone()).await {
            Ok(()) => {
                self.state
                    .finish_gpu_rental_operation(
                        operation_id.as_str(),
                        "succeeded",
                        None,
                        Some(resource_id.as_str()),
                        None,
                        now_ms,
                    )
                    .await?;
                self.apply_update(
                    &lease,
                    GpuRentalUpdate {
                        observed_state: Some(GpuRentalState::Terminating),
                        provider_resource_id: Some(resource_id),
                        next_retry_at_ms: Some(now_ms.saturating_add(self.config.normal_poll_ms)),
                        clear_last_error: true,
                        ..GpuRentalUpdate::default()
                    },
                    now_ms,
                )
                .await?;
                Ok(Vec::new())
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
                        None,
                        Some(resource_id.as_str()),
                        Some(error.safe_message.as_str()),
                        now_ms,
                    )
                    .await?;
                self.record_termination_retry(lease, error, now_ms).await
            }
        }
    }

    async fn record_termination_retry(
        &self,
        lease: GpuRentalLease,
        error: ProviderError,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
        let retry_ms = error.retry_after_ms.unwrap_or_else(|| {
            let exponent = u32::try_from(lease.rental.retry_count.clamp(0, 6)).unwrap_or(6);
            1_000_i64.saturating_mul(2_i64.saturating_pow(exponent))
        });
        self.apply_update(
            &lease,
            GpuRentalUpdate {
                observed_state: Some(GpuRentalState::TerminationUnconfirmed),
                last_error_code: Some(format!("{:?}", error.kind).to_ascii_lowercase()),
                last_error_message: Some(error.safe_message),
                diagnostic_ref: error.diagnostic_ref,
                next_retry_at_ms: Some(
                    now_ms.saturating_add(retry_ms.min(self.config.maximum_retry_ms)),
                ),
                increment_retry_count: true,
                ..GpuRentalUpdate::default()
            },
            now_ms,
        )
        .await?;
        Ok(Vec::new())
    }

    async fn confirm_terminated(
        &self,
        lease: GpuRentalLease,
        now_ms: i64,
    ) -> anyhow::Result<Vec<ControllerEvent>> {
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
        Ok(vec![ControllerEvent::StateChanged {
            rental_id: lease.rental.rental_id,
            state: GpuRentalState::TerminatedConfirmed,
        }])
    }
}
