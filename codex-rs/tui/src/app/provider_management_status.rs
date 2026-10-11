use codex_provider_auth::ProviderManagementPhase;

use super::App;
use super::AppEvent;
use super::AppServerSession;

pub(super) fn spawn_provider_status_job(job: impl FnOnce() + Send + 'static) {
    tokio::task::spawn_blocking(job);
}

/// PF-84-S04: named-account rows for `/providers` while the feature is on.
/// A vault that cannot be read shows no rows rather than blocking the screen.
fn load_named_accounts(
    enabled: bool,
    codex_home: &std::path::Path,
) -> Option<Vec<crate::provider_named_accounts::NamedAccountRow>> {
    enabled.then(|| {
        crate::provider_named_accounts::load_rows(codex_home).unwrap_or_else(|error| {
            tracing::warn!("{error}");
            Vec::new()
        })
    })
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
        &config.model_provider_id,
    );
    catalog.refresh_provider_policy();
}

impl App {
    pub(super) fn reusable_provider_status_host(
        &self,
    ) -> Option<crate::provider_status_host::ProviderStatusHost> {
        self.shared_provider_status_host.clone().or_else(|| {
            self.model_catalog
                .provider_policy()
                .map(|policy| policy.host())
        })
    }

    pub(super) fn open_provider_manager(&mut self, _app_server: &AppServerSession) {
        sync_provider_manager_model_policy(&self.model_catalog, &self.config);
        let generation = self.next_provider_management_generation();
        self.provider_management_host = None;
        let config = self.config.clone();
        let shared_status_host = self.reusable_provider_status_host();
        let tx = self.app_event_tx.clone();
        spawn_provider_status_job(move || {
            let status_host = provider_manager_status_host(&config, shared_status_host);
            let statuses = status_host.resolve().entries().to_vec();
            let accounts = load_named_accounts(
                config
                    .features
                    .enabled(codex_features::Feature::NamedAccounts),
                &config.codex_home,
            );
            tx.send(AppEvent::ProviderManagerStatusesResolved {
                generation,
                status_host,
                statuses,
                accounts,
            });
        });
    }

    pub(super) fn provider_manager_statuses_resolved(
        &mut self,
        generation: u64,
        status_host: crate::provider_status_host::ProviderStatusHost,
        statuses: Vec<codex_provider_auth::ProviderStatusSnapshot>,
        accounts: Option<Vec<crate::provider_named_accounts::NamedAccountRow>>,
        app_server: &AppServerSession,
    ) {
        if generation != self.provider_management_generation {
            return;
        }
        if self
            .provider_management_host
            .as_ref()
            .is_some_and(|host| !matches!(host.phase(), ProviderManagementPhase::Browsing))
        {
            return;
        }
        // The asynchronous discovery may have settled after the picker cached
        // its eligibility snapshot. Publish the same status result to both UIs.
        self.model_catalog.update_provider_statuses(&statuses);
        let selected_index = self.chat_widget.provider_manager_selected_index();
        let selected_provider =
            selected_index.and_then(|index| self.provider_manager_row_provider(index));
        if let Some(host) = self.provider_management_host.as_mut() {
            if let Some(provider_id) = selected_provider {
                host.remember_focused_provider(provider_id);
            }
            let accounts_changed = host.named_accounts() != accounts.as_deref();
            host.set_named_accounts(accounts);
            // A late discovery result still refreshes policy, but must not
            // reopen a manager the user dismissed (for example to open /model).
            if (host.apply_statuses(statuses).applied || accounts_changed)
                && selected_index.is_some()
            {
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
        let mut host = host;
        host.set_named_accounts(accounts);
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
        let status_host = host.status_host().clone();
        let generation = self.next_provider_management_generation();
        let worker_host = status_host.clone();
        let tx = self.app_event_tx.clone();
        let named_accounts_enabled = self
            .config
            .features
            .enabled(codex_features::Feature::NamedAccounts);
        let codex_home = self.config.codex_home.to_path_buf();
        spawn_provider_status_job(move || {
            let statuses = worker_host.resolve().entries().to_vec();
            let accounts = load_named_accounts(named_accounts_enabled, &codex_home);
            tx.send(AppEvent::ProviderManagerStatusesResolved {
                generation,
                status_host,
                statuses,
                accounts,
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
        let accounts = self.provider_manager_accounts_view();
        self.chat_widget.open_provider_manager(
            &catalog,
            &statuses,
            focused_provider.as_ref(),
            accounts.as_ref(),
        );
    }

    /// PF-84-S04: the accounts `/providers` shows; `None` while the feature is off.
    pub(super) fn provider_manager_accounts_view(
        &self,
    ) -> Option<crate::provider_named_accounts::AccountsView> {
        let rows = self.provider_management_host.as_ref()?.named_accounts()?;
        Some(crate::provider_named_accounts::AccountsView::new(
            rows.to_vec(),
            self.chat_widget.config_ref(),
            &self.config,
        ))
    }

    /// The catalog provider of a `/providers` row, account rows included.
    pub(super) fn provider_manager_row_provider(
        &self,
        index: usize,
    ) -> Option<codex_provider_auth::ProviderCatalogId> {
        let host = self.provider_management_host.as_ref()?;
        let accounts = self.provider_manager_accounts_view();
        crate::chatwidget::provider_manager_rows(
            host.status_host().catalog(),
            host.statuses(),
            accounts.as_ref(),
        )
        .get(index)
        .map(|row| row.provider_id().clone())
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
        config.model_provider_id = "manager-added".to_string();
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
