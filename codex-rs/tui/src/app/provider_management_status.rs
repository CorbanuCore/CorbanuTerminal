use codex_provider_auth::ProviderManagementPhase;

use super::App;
use super::AppEvent;
use super::AppServerSession;

pub(super) fn spawn_provider_status_job(job: impl FnOnce() + Send + 'static) {
    tokio::task::spawn_blocking(job);
}

pub(super) fn provider_manager_status_host(
    config: &crate::legacy_core::config::Config,
    shared: Option<crate::provider_status_host::ProviderStatusHost>,
) -> crate::provider_status_host::ProviderStatusHost {
    shared.unwrap_or_else(|| {
        crate::provider_status_host::ProviderStatusHost::from_config(
            config,
            crate::provider_status_host::ProviderAccountMetadata {
                claude: codex_provider_auth::ClaudeCredentialMetadata::Checking,
                ..Default::default()
            },
        )
    })
}

fn sync_provider_manager_model_policy(
    catalog: &crate::model_catalog::ModelCatalog,
    config: &crate::legacy_core::config::Config,
) {
    catalog.sync_runtime_models(
        config.model_providers.keys().map(String::as_str),
        config.model.as_deref(),
    );
    catalog.refresh_provider_policy();
}

impl App {
    pub(super) fn open_provider_manager(&mut self, _app_server: &AppServerSession) {
        sync_provider_manager_model_policy(&self.model_catalog, &self.config);
        let generation = self.next_provider_management_generation();
        self.provider_management_host = None;
        let config = self.config.clone();
        let shared_status_host = self.shared_provider_status_host.clone();
        let tx = self.app_event_tx.clone();
        spawn_provider_status_job(move || {
            let status_host = provider_manager_status_host(&config, shared_status_host);
            let statuses = status_host.resolve().entries().to_vec();
            tx.send(AppEvent::ProviderManagerStatusesResolved {
                generation,
                status_host,
                statuses,
            });
        });
    }

    pub(super) fn provider_manager_statuses_resolved(
        &mut self,
        generation: u64,
        status_host: crate::provider_status_host::ProviderStatusHost,
        statuses: Vec<codex_provider_auth::ProviderStatusSnapshot>,
        app_server: &AppServerSession,
    ) {
        if generation != self.provider_management_generation {
            return;
        }
        let selected_index = self.chat_widget.provider_manager_selected_index();
        if let Some(host) = self.provider_management_host.as_mut() {
            if !matches!(host.phase(), ProviderManagementPhase::Browsing) {
                return;
            }
            if let Some(provider_id) = selected_index
                .and_then(|index| host.statuses().get(index))
                .map(|status| status.id.clone())
            {
                host.remember_focused_provider(provider_id);
            }
            if host.apply_statuses(statuses).applied {
                self.render_provider_manager();
            }
            return;
        }
        let host = crate::provider_management_host::ProviderManagementHost::new_with_statuses(
            &self.config,
            app_server.request_handle(),
            self.app_event_tx.clone(),
            status_host,
            statuses,
        );
        self.provider_management_host = Some(host);
        self.render_provider_manager();
        let config = self.config.clone();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let metadata =
                crate::provider_status_host::ProviderAccountMetadata::discover(&config).await;
            tx.send(AppEvent::ProviderManagerMetadataResolved {
                generation,
                metadata,
            });
        });
    }

    fn next_provider_management_generation(&mut self) -> u64 {
        self.provider_management_generation = self.provider_management_generation.wrapping_add(1);
        if self.provider_management_generation == 0 {
            self.provider_management_generation = 1;
        }
        self.provider_management_generation
    }

    pub(super) fn schedule_provider_manager_refresh(&mut self) {
        let Some(host) = self.provider_management_host.as_ref() else {
            return;
        };
        if !matches!(host.phase(), ProviderManagementPhase::Browsing) {
            return;
        }
        let generation = self.next_provider_management_generation();
        let status_host = self
            .provider_management_host
            .as_ref()
            .expect("provider management host disappeared")
            .status_host()
            .clone();
        let worker_host = status_host.clone();
        let tx = self.app_event_tx.clone();
        spawn_provider_status_job(move || {
            let statuses = worker_host.resolve().entries().to_vec();
            tx.send(AppEvent::ProviderManagerStatusesResolved {
                generation,
                status_host,
                statuses,
            });
        });
    }

    pub(super) fn render_provider_manager(&mut self) {
        let Some(host) = self.provider_management_host.as_ref() else {
            return;
        };
        let catalog = host.status_host().catalog().clone();
        let statuses = host.statuses().to_vec();
        let focused_provider = host.focused_provider().cloned();
        self.chat_widget
            .open_provider_manager(&catalog, &statuses, focused_provider.as_ref());
    }

    pub(super) fn provider_manager_metadata_resolved(
        &mut self,
        generation: u64,
        metadata: crate::provider_status_host::ProviderAccountMetadata,
    ) {
        if generation != self.provider_management_generation {
            return;
        }
        let can_refresh = self
            .provider_management_host
            .as_mut()
            .is_some_and(|host| host.update_account_metadata(metadata));
        if can_refresh {
            self.schedule_provider_manager_refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn manager_open_syncs_configured_runtime_models_before_policy_refresh() {
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .build()
            .await
            .unwrap();
        config.model = Some("shared-model".to_string());
        config.model_providers.insert(
            "manager-added".to_string(),
            codex_model_provider_info::ModelProviderInfo {
                name: "Manager Added".to_string(),
                ..Default::default()
            },
        );
        let catalog = crate::model_catalog::ModelCatalog::new(Vec::new());

        sync_provider_manager_model_policy(&catalog, &config);
        sync_provider_manager_model_policy(&catalog, &config);

        let matches = catalog
            .try_list_models()
            .unwrap()
            .into_iter()
            .filter(|preset| preset.provider_id.as_deref() == Some("manager-added"))
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].model, "shared-model");
    }
}
