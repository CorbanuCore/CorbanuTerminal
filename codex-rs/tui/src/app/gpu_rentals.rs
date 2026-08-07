//! GPU-rental actions kept outside the central TUI event dispatcher.

use super::*;

impl App {
    pub(super) async fn open_gpu_menu(&mut self) {
        let Some(state_db) = self.state_db.as_ref() else {
            self.chat_widget
                .add_error_message("GPU rental state is unavailable in this session.".to_string());
            return;
        };
        match state_db.list_gpu_rentals(/*limit*/ 100).await {
            Ok(rentals) => self.chat_widget.open_gpu_menu(rentals),
            Err(error) => self
                .chat_widget
                .add_error_message(format!("Unable to read GPU rentals: {error}")),
        }
    }

    pub(super) async fn open_gpu_rental(&mut self, rental_id: String) {
        let Some(state_db) = self.state_db.as_ref() else {
            self.chat_widget
                .add_error_message("GPU rental state is unavailable in this session.".to_string());
            return;
        };
        match state_db.get_gpu_rental(rental_id.as_str()).await {
            Ok(Some(rental)) => self.chat_widget.open_gpu_rental(rental),
            Ok(None) => self
                .chat_widget
                .add_error_message(format!("GPU rental {rental_id} was not found.")),
            Err(error) => self
                .chat_widget
                .add_error_message(format!("Unable to read GPU rental: {error}")),
        }
    }

    pub(super) async fn disable_gpu_serving(&mut self, rental_id: String) {
        let Some(state_db) = self.state_db.as_ref() else {
            self.chat_widget
                .add_error_message("GPU rental state is unavailable in this session.".to_string());
            return;
        };
        let now_ms = chrono::Utc::now().timestamp_millis();
        match state_db
            .set_gpu_runtime_provider_health(rental_id.as_str(), "degraded", now_ms)
            .await
        {
            Ok(true) => self.chat_widget.add_info_message(
                format!("Stopped serving GPU rental {rental_id}. Provider billing may continue."),
                /*hint*/ None,
            ),
            Ok(false) => self.chat_widget.add_error_message(format!(
                "GPU rental {rental_id} has no active runtime provider."
            )),
            Err(error) => self
                .chat_widget
                .add_error_message(format!("Unable to stop GPU serving: {error}")),
        }
    }

    pub(super) async fn terminate_gpu_rental(&mut self, rental_id: String) {
        let Some(state_db) = self.state_db.as_ref() else {
            self.chat_widget
                .add_error_message("GPU rental state is unavailable in this session.".to_string());
            return;
        };
        let now_ms = chrono::Utc::now().timestamp_millis();
        let _ = state_db
            .set_gpu_runtime_provider_health(rental_id.as_str(), "degraded", now_ms)
            .await;
        match state_db
            .request_gpu_rental_termination(rental_id.as_str(), now_ms)
            .await
        {
            Ok(true) => {
                self.chat_widget.add_info_message(
                    format!(
                        "Termination requested for GPU rental {rental_id}; billing remains unresolved until the provider confirms absence."
                    ),
                    /*hint*/ None,
                );
                self.start_gpu_controller();
            }
            Ok(false) => self.chat_widget.add_error_message(format!(
                "GPU rental {rental_id} is already terminal or cannot be terminated."
            )),
            Err(error) => self
                .chat_widget
                .add_error_message(format!("Unable to terminate GPU rental: {error}")),
        }
    }

    pub(super) fn search_gpu_offers(
        &mut self,
        recipe_id: String,
        maximum_hourly_microusd: i64,
        maximum_total_microusd: i64,
        ttl_minutes: i64,
    ) {
        let Some(state_db) = self.state_db.clone() else {
            self.chat_widget
                .add_error_message("GPU rental state is unavailable in this session.".to_string());
            return;
        };
        let tx = self.app_event_tx.clone();
        let codex_home = self.config.codex_home.clone();
        let now_ms = chrono::Utc::now().timestamp_millis();
        let authorization = codex_gpu_market::RentalAuthorization {
            client_operation_id: Uuid::new_v4().to_string(),
            maximum_hourly_microusd,
            maximum_total_microusd,
            terminate_at_ms: now_ms.saturating_add(ttl_minutes.saturating_mul(60_000)),
            acknowledged_local_enforcement: true,
        };
        self.chat_widget
            .add_info_message(format!("Searching verified capacity for {recipe_id}…"), /*hint*/ None);
        tokio::spawn(async move {
            let result = async {
                let installation_id = codex_core_api::resolve_installation_id(&codex_home)
                    .await
                    .map_err(|error| error.to_string())?;
                let credentials = Arc::new(codex_gpu_market::VaultGpuCredentialResolver::new(
                    Arc::new(codex_vault::Vault::new(codex_home.to_path_buf())),
                ));
                let service = codex_gpu_market::GpuMarketService::new(
                    state_db,
                    codex_gpu_market::RecipeCatalog::default(),
                    installation_id,
                );
                service
                    .search(
                        recipe_id.as_str(),
                        maximum_hourly_microusd,
                        &codex_gpu_market::VastProvider::new(credentials.clone()),
                        &codex_gpu_market::RunpodProvider::new(credentials),
                    )
                    .await
                    .map_err(|error| error.safe_message)
            }
            .await;
            tx.send(AppEvent::GpuOffersLoaded {
                recipe_id,
                authorization,
                offers: result,
            });
        });
    }

    pub(super) fn confirm_gpu_rental(
        &mut self,
        recipe_id: String,
        authorization: codex_gpu_market::RentalAuthorization,
        offer: codex_gpu_market::GpuOffer,
    ) {
        let Some(state_db) = self.state_db.clone() else {
            self.chat_widget
                .add_error_message("GPU rental state is unavailable in this session.".to_string());
            return;
        };
        let tx = self.app_event_tx.clone();
        let codex_home = self.config.codex_home.clone();
        tokio::spawn(async move {
            let result = async {
                let installation_id = codex_core_api::resolve_installation_id(&codex_home)
                    .await
                    .map_err(|error| error.to_string())?;
                let credentials = Arc::new(codex_gpu_market::VaultGpuCredentialResolver::new(
                    Arc::new(codex_vault::Vault::new(codex_home.to_path_buf())),
                ));
                let rental_id = format!("gpu-{}", authorization.client_operation_id);
                credentials
                    .ensure_rental_endpoint_token(rental_id.as_str())
                    .map_err(|_| {
                        "Could not create the scoped GPU endpoint credential. No rental was started."
                            .to_string()
                    })?;
                let service = codex_gpu_market::GpuMarketService::new(
                    state_db,
                    codex_gpu_market::RecipeCatalog::default(),
                    installation_id,
                );
                let now_ms = chrono::Utc::now().timestamp_millis();
                match offer.provider.as_str() {
                    "vast" => {
                        service
                            .confirm(
                                recipe_id.as_str(),
                                &offer,
                                &authorization,
                                &codex_gpu_market::VastProvider::new(credentials.clone()),
                                now_ms,
                            )
                            .await
                    }
                    "runpod" => {
                        service
                            .confirm(
                                recipe_id.as_str(),
                                &offer,
                                &authorization,
                                &codex_gpu_market::RunpodProvider::new(credentials.clone()),
                                now_ms,
                            )
                            .await
                    }
                    _ => Err(codex_gpu_market::ProviderError::new(
                        codex_gpu_market::ProviderErrorKind::InvalidRequest,
                        "Unsupported GPU provider.",
                    )),
                }
                .map_err(|error| error.safe_message)
            }
            .await;
            tx.send(AppEvent::GpuRentalConfirmationFinished { result });
        });
    }

    pub(super) fn on_gpu_rental_confirmation_finished(
        &mut self,
        result: Result<codex_state::GpuRental, String>,
    ) {
        match result {
            Ok(rental) if rental.observed_state == codex_state::GpuRentalState::Failed => {
                let reason = rental.last_error_message.as_deref().unwrap_or(
                    "The earlier allocation attempt failed before a resource was created.",
                );
                self.chat_widget.add_error_message(format!(
                    "That confirmation already produced GPU rental {}, which failed: {reason} Choose another current offer from /gpu.",
                    rental.rental_id
                ));
            }
            Ok(rental) => {
                self.chat_widget.add_info_message(
                    format!(
                        "GPU rental {} was authorized. The independent controller is starting; /gpu remains authoritative for billing state.",
                        rental.rental_id
                    ),
                    /*hint*/ None,
                );
                self.start_gpu_controller();
            }
            Err(message) => self.chat_widget.add_error_message(message),
        }
    }

    pub(super) fn save_gpu_provider_credential(
        &mut self,
        provider: String,
        api_key: crate::app_event::ProviderApiKeySecret,
    ) {
        let (label, display_name) = match provider.as_str() {
            "runpod" => (codex_gpu_market::RUNPOD_API_KEY_LABEL, "RunPod"),
            "vast" => (codex_gpu_market::VAST_API_KEY_LABEL, "Vast.ai"),
            _ => {
                self.chat_widget
                    .add_error_message("Unsupported GPU provider credential.".to_string());
                return;
            }
        };
        let vault = codex_vault::Vault::new(self.config.codex_home.clone().to_path_buf());
        let secret = api_key.into_inner();
        let result = if vault.exists(label).unwrap_or(false) {
            vault
                .update(label, Some(secret), /*provider*/ None, /*notes*/ None, /*revocation_notes*/ None)
                .map(|_| ())
        } else {
            vault.add(codex_vault::AddCredential {
                label: label.to_string(),
                credential_type: codex_vault::CredentialType::ApiKey,
                provider: Some(provider),
                notes: Some("PFTerminal GPU rental provider credential".to_string()),
                revocation_notes: Some(
                    "Revoke at the provider and delete from /vault when retired.".to_string(),
                ),
                secret,
            })
        };
        match result {
            Ok(()) => self.chat_widget.add_info_message(
                format!("Stored {display_name} GPU rental credential in the encrypted vault."),
                /*hint*/ None,
            ),
            Err(error) => self
                .chat_widget
                .add_error_message(format!("Unable to store {display_name} credential: {error}")),
        }
    }

    #[cfg(test)]
    pub(super) fn start_gpu_controller(&mut self) {}

    #[cfg(not(test))]
    pub(super) fn start_gpu_controller(&mut self) {
        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(error) => {
                self.chat_widget.add_error_message(format!(
                    "GPU rental state was saved, but the independent controller could not be located: {error}"
                ));
                return;
            }
        };
        if let Err(error) = std::process::Command::new(executable)
            .arg("internal-gpu-controller")
            .env("CODEX_HOME", self.config.codex_home.as_path())
            .env(codex_state::SQLITE_HOME_ENV, self.config.sqlite.home())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            self.chat_widget.add_error_message(format!(
                "GPU rental state was saved, but the independent controller did not start: {error}. Run `pfterminal internal-gpu-controller` before relying on local TTL or spend enforcement."
            ));
        }
    }
}
