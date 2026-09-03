use codex_protocol::openai_models::ModelPreset;
use codex_provider_auth::CurrentSelectionDecision;
use codex_provider_auth::ProviderRuntimeAuthorizations;
use codex_provider_auth::ProviderRuntimeSelectionPolicy;
use codex_provider_auth::ProviderStatusCatalog;
use codex_provider_auth::ProviderUseContext;
use codex_provider_auth::ProviderUseDecision;

use crate::provider_status_host::ProviderStatusHost;

#[derive(Clone, Debug)]
pub(crate) struct ProviderModelPolicy {
    host: ProviderStatusHost,
    statuses: ProviderStatusCatalog,
    authorizations: ProviderRuntimeAuthorizations,
}

impl ProviderModelPolicy {
    pub(crate) fn new(
        host: ProviderStatusHost,
        authorizations: ProviderRuntimeAuthorizations,
    ) -> Self {
        let statuses = host.resolve();
        Self {
            host,
            statuses,
            authorizations,
        }
    }

    pub(crate) fn host(&self) -> ProviderStatusHost {
        self.host.clone()
    }

    pub(crate) fn refresh(&mut self) {
        self.statuses = self.host.resolve();
    }

    pub(crate) fn set_current_runtime(&mut self, runtime_provider_id: &str) {
        self.host
            .set_current_runtime(runtime_provider_id.to_string());
        self.refresh();
    }

    pub(crate) fn assess(
        &self,
        runtime_provider_id: &str,
        model: &str,
        context: ProviderUseContext,
    ) -> ProviderUseDecision {
        ProviderRuntimeSelectionPolicy::assess(
            self.host.catalog(),
            &self.statuses,
            &self.authorizations,
            runtime_provider_id,
            model,
            context,
        )
    }

    pub(crate) fn current(
        &self,
        runtime_provider_id: &str,
        model: &str,
    ) -> CurrentSelectionDecision {
        ProviderRuntimeSelectionPolicy::current(
            self.host.catalog(),
            &self.statuses,
            &self.authorizations,
            runtime_provider_id,
            model,
        )
    }

    pub(crate) fn preset_is_selectable(&self, preset: &ModelPreset) -> bool {
        let Some(provider) = preset
            .provider_id
            .as_deref()
            .or_else(|| codex_model_provider_info::canonical_catalog_provider(&preset.model))
        else {
            return false;
        };
        matches!(
            self.assess(provider, &preset.model, ProviderUseContext::ModelPicker),
            ProviderUseDecision::Ready(_)
        )
    }

    pub(crate) fn provider_is_selectable(&self, provider: &str, model: &str) -> bool {
        matches!(
            self.assess(provider, model, ProviderUseContext::ModelPicker),
            ProviderUseDecision::Ready(_)
        )
    }

    pub(crate) fn has_ready_configured_provider(&self) -> bool {
        self.statuses.entries().iter().any(|status| {
            status.configuration == codex_provider_auth::ProviderConfigurationState::Configured
                && status.eligibility == codex_provider_auth::ProviderEligibilityState::Active
                && status.availability == codex_provider_auth::ProviderAvailabilityState::Ready
                && self
                    .host
                    .catalog()
                    .get(status.id.as_str())
                    .is_some_and(|entry| {
                        entry.setup_capabilities.iter().any(|capability| {
                            matches!(
                                capability,
                                codex_provider_auth::ProviderSetupCapability::ApiKey { .. }
                                    | codex_provider_auth::ProviderSetupCapability::OpenAiAccount
                                    | codex_provider_auth::ProviderSetupCapability::ClaudeAccount
                                    | codex_provider_auth::ProviderSetupCapability::CorbanuPlan
                            )
                        })
                    })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_model_provider_info::ModelProviderInfo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn custom_provider_selection_tracks_shared_active_policy() {
        let home = tempdir().unwrap();
        let env_key = "PF55_CUSTOM_POLICY_KEY";
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        config.model_provider_id = "custom".into();
        config.model = Some("custom-model".into());
        config.model_provider = ModelProviderInfo {
            name: "Custom".into(),
            env_key: Some(env_key.into()),
            ..Default::default()
        };
        config
            .model_providers
            .insert("custom".into(), config.model_provider.clone());
        codex_login::login_with_provider_api_key(
            home.path(),
            env_key,
            "pf55-secret-canary",
            config.cli_auth_credentials_store_mode,
            config.auth_keyring_backend_kind(),
        )
        .unwrap();
        let host = ProviderStatusHost::from_config(
            &config,
            crate::provider_status_host::ProviderAccountMetadata::default(),
        );
        let mut policy =
            ProviderModelPolicy::new(host.clone(), ProviderRuntimeAuthorizations::default());

        assert!(policy.provider_is_selectable("custom", "custom-model"));
        host.persist_policy(
            "custom",
            codex_provider_auth::ProviderActivationPolicy::Inactive,
        )
        .unwrap();
        policy.refresh();
        assert!(!policy.provider_is_selectable("custom", "custom-model"));
        assert!(matches!(
            policy.current("custom", "custom-model"),
            CurrentSelectionDecision::RequireExplicitRecovery {
                reason: codex_provider_auth::ProviderUseBlocker::Inactive,
                ..
            }
        ));
        assert!(!format!("{policy:?}").contains("pf55-secret-canary"));
    }
}
