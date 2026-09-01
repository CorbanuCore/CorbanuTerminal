use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use codex_login::OpenAiAuthMetadata;
use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_provider_auth::ApiKeyAuthTarget;
use codex_provider_auth::ApiKeyCredentialMetadata;
use codex_provider_auth::ApiKeyStorage;
use codex_provider_auth::ClaudeCredentialMetadata;
use codex_provider_auth::CommandAuthMetadata;
use codex_provider_auth::ConfiguredAvailability;
use codex_provider_auth::CorbanuCredentialSource;
use codex_provider_auth::CorbanuPlanMetadata;
use codex_provider_auth::CurrentProviderSelection;
use codex_provider_auth::EnvironmentCredentialMetadata;
use codex_provider_auth::LocalProviderMetadata;
use codex_provider_auth::ManagedApiKeyMetadata;
use codex_provider_auth::ProviderActivationPolicy;
use codex_provider_auth::ProviderCatalog;
use codex_provider_auth::ProviderCatalogEntry;
use codex_provider_auth::ProviderEligibilitySnapshot;
use codex_provider_auth::ProviderEligibilityStore;
use codex_provider_auth::ProviderMetadata;
use codex_provider_auth::ProviderMetadataSnapshot;
use codex_provider_auth::ProviderSetupCapability;
use codex_provider_auth::ProviderStatusCatalog;
use codex_provider_auth::ProviderStatusResolver;
use codex_provider_auth::ProviderStatusSnapshot;

use crate::legacy_core::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderAccountMetadata {
    pub(crate) openai: OpenAiAuthMetadata,
    pub(crate) claude: ClaudeCredentialMetadata,
    pub(crate) corbanu: CorbanuPlanMetadata,
}

impl Default for ProviderAccountMetadata {
    fn default() -> Self {
        Self {
            openai: OpenAiAuthMetadata::Missing,
            claude: ClaudeCredentialMetadata::NotConfigured,
            corbanu: CorbanuPlanMetadata::NotConfigured,
        }
    }
}

impl ProviderAccountMetadata {
    pub(crate) async fn discover(config: &Config) -> Self {
        let codex_home = config.codex_home.to_path_buf();
        let auth_manager = codex_login::AuthManager::shared_from_config(
            config, /*enable_codex_api_key_env*/ true,
        );
        let claude_status = crate::chatwidget::claude_code_login::current_status_with_timeout(
            codex_home.as_path(),
            std::time::Duration::from_secs(10),
        );
        let (auth_manager, claude_status) = tokio::join!(auth_manager, claude_status);
        let openai = auth_manager.openai_auth_metadata();
        let claude = match claude_status {
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::ManagedToken {
                stored: true,
            } => ClaudeCredentialMetadata::Configured {
                source: codex_provider_auth::ClaudeCredentialSource::Managed,
            },
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::EnvironmentToken {
                available: true,
            } => ClaudeCredentialMetadata::Configured {
                source: codex_provider_auth::ClaudeCredentialSource::Environment,
            },
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::SignedIn { .. } => {
                ClaudeCredentialMetadata::Configured {
                    source: codex_provider_auth::ClaudeCredentialSource::ClaudeCodeLogin,
                }
            }
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::ManagedToken {
                stored: false,
            }
            | crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::EnvironmentToken {
                available: false,
            }
            | crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::InvalidSelection
            | crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::NeedsReauthorization => {
                ClaudeCredentialMetadata::RecoveryRequired {
                    reason: codex_provider_auth::ProviderRecoveryReason::UnhealthyClaudeSelection,
                }
            }
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::SelectionRequired {
                existing_source_detected: true,
            } => ClaudeCredentialMetadata::RecoveryRequired {
                reason: codex_provider_auth::ProviderRecoveryReason::AmbiguousClaudeSources,
            },
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::SelectionRequired {
                existing_source_detected: false,
            }
            | crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::SignedOut => {
                ClaudeCredentialMetadata::NotConfigured
            }
            crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::Checking
            | crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::Unavailable
            | crate::chatwidget::claude_code_login::ClaudeCodePlanStatus::Error => {
                ClaudeCredentialMetadata::Unavailable
            }
        };
        Self {
            openai,
            claude,
            corbanu: CorbanuPlanMetadata::NotConfigured,
        }
    }
}

/// Metadata-only adapter shared by onboarding and the later provider manager.
#[derive(Clone)]
pub(crate) struct ProviderStatusHost {
    catalog: ProviderCatalog,
    codex_home: PathBuf,
    current: CurrentProviderSelection,
    account: Arc<RwLock<ProviderAccountMetadata>>,
}

impl ProviderStatusHost {
    pub(crate) fn from_config(config: &Config, account: ProviderAccountMetadata) -> Self {
        Self {
            catalog: ProviderCatalog::from_runtime_providers(&config.model_providers),
            codex_home: config.codex_home.to_path_buf(),
            current: CurrentProviderSelection::runtime_id(config.model_provider_id.clone()),
            account: Arc::new(RwLock::new(account)),
        }
    }

    pub(crate) fn catalog(&self) -> &ProviderCatalog {
        &self.catalog
    }

    pub(crate) fn update_account_metadata(&self, account: ProviderAccountMetadata) {
        *self
            .account
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = account;
    }

    pub(crate) fn mark_openai_api_key(&self) {
        self.account
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .openai = OpenAiAuthMetadata::ApiKey;
    }

    pub(crate) fn mark_openai_account(&self) {
        self.account
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .openai = OpenAiAuthMetadata::Account;
    }

    pub(crate) fn mark_claude_configured(
        &self,
        source: codex_provider_auth::ClaudeCredentialSource,
    ) {
        self.account
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .claude = ClaudeCredentialMetadata::Configured { source };
    }

    pub(crate) fn api_key_metadata(&self, target: &ApiKeyAuthTarget) -> ApiKeyCredentialMetadata {
        match &target.storage {
            ApiKeyStorage::EnvironmentVariable { env_key } => ApiKeyCredentialMetadata {
                environment: environment_metadata(env_key),
                managed: ManagedApiKeyMetadata::from(
                    codex_login::provider_api_key_metadata_from_auth_storage(
                        &self.codex_home,
                        env_key,
                    ),
                ),
            },
            ApiKeyStorage::OpenAiAuth => {
                let openai = self
                    .account
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .openai;
                ApiKeyCredentialMetadata {
                    environment: EnvironmentCredentialMetadata::Missing,
                    managed: if openai == OpenAiAuthMetadata::ApiKey {
                        ManagedApiKeyMetadata::Stored {
                            source: codex_login::ProviderApiKeyStorageSource::EncryptedVault,
                        }
                    } else {
                        ManagedApiKeyMetadata::Missing
                    },
                }
            }
        }
    }

    pub(crate) fn resolve(&self) -> ProviderStatusCatalog {
        let account = *self
            .account
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut metadata = ProviderMetadataSnapshot::default();
        for entry in self.catalog.entries() {
            metadata.insert(entry, self.metadata_for(entry, account));
        }
        ProviderStatusResolver::resolve(
            &self.catalog,
            &metadata,
            &ProviderEligibilitySnapshot::from(
                ProviderEligibilityStore::new(&self.codex_home).load(),
            ),
            &self.current,
        )
    }

    pub(crate) fn resolve_provider(&self, provider_id: &str) -> Option<ProviderStatusSnapshot> {
        let entry = self.catalog.get(provider_id)?;
        let account = *self
            .account
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut metadata = ProviderMetadataSnapshot::default();
        metadata.insert(entry, self.metadata_for(entry, account));
        ProviderStatusResolver::resolve(
            &self.catalog,
            &metadata,
            &ProviderEligibilitySnapshot::from(
                ProviderEligibilityStore::new(&self.codex_home).load(),
            ),
            &self.current,
        )
        .get(provider_id)
        .cloned()
    }

    pub(crate) fn resolve_target(&self, target: &ApiKeyAuthTarget) -> ProviderStatusSnapshot {
        self.resolve_provider(target.provider_id.as_str())
            .unwrap_or_else(|| panic!("catalog target disappeared: {}", target.provider_id))
    }

    pub(crate) fn activate(&self, provider_id: &str) -> bool {
        let Some(entry) = self.catalog.get(provider_id) else {
            return false;
        };
        let store = ProviderEligibilityStore::new(&self.codex_home);
        let Ok(mut eligibility) = store.load() else {
            return false;
        };
        eligibility.set_policy(entry, ProviderActivationPolicy::Active);
        store.save(&eligibility).is_ok()
    }

    fn metadata_for(
        &self,
        entry: &ProviderCatalogEntry,
        account: ProviderAccountMetadata,
    ) -> ProviderMetadata {
        let first = entry.setup_capabilities.iter().next();
        match first {
            Some(ProviderSetupCapability::OpenAiAccount)
            | Some(ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::OpenAiAuth,
            }) => ProviderMetadata::OpenAi(account.openai),
            Some(ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::EnvironmentVariable { env_key },
            }) => ProviderMetadata::ApiKey(ApiKeyCredentialMetadata {
                environment: environment_metadata(env_key),
                managed: ManagedApiKeyMetadata::from(
                    codex_login::provider_api_key_metadata_from_auth_storage(
                        &self.codex_home,
                        env_key,
                    ),
                ),
            }),
            Some(ProviderSetupCapability::ClaudeAccount) => {
                ProviderMetadata::Claude(account.claude)
            }
            Some(ProviderSetupCapability::CorbanuPlan) => {
                ProviderMetadata::CorbanuPlan(corbanu_metadata(&self.codex_home, account.corbanu))
            }
            Some(ProviderSetupCapability::Local { .. }) => {
                ProviderMetadata::Local(LocalProviderMetadata::Checking)
            }
            Some(ProviderSetupCapability::CommandAuth { .. }) => {
                ProviderMetadata::CommandAuth(CommandAuthMetadata::StatusOnly)
            }
            Some(ProviderSetupCapability::StatusOnly { .. }) => ProviderMetadata::StatusOnly,
            None => ProviderMetadata::Unavailable,
        }
    }
}

fn corbanu_metadata(
    codex_home: &std::path::Path,
    supplied: CorbanuPlanMetadata,
) -> CorbanuPlanMetadata {
    if supplied != CorbanuPlanMetadata::NotConfigured {
        return supplied;
    }
    if environment_metadata(PFTERMINAL_PLAN_API_KEY_ENV_VAR)
        == EnvironmentCredentialMetadata::Present
    {
        return CorbanuPlanMetadata::Configured {
            source: CorbanuCredentialSource::Environment,
            availability: ConfiguredAvailability::Ready,
        };
    }
    match codex_login::provider_api_key_metadata_from_auth_storage(
        codex_home,
        PFTERMINAL_PLAN_API_KEY_ENV_VAR,
    ) {
        Ok(codex_login::ProviderApiKeyStorageMetadata::Stored { .. }) => {
            CorbanuPlanMetadata::Configured {
                source: CorbanuCredentialSource::Managed,
                availability: ConfiguredAvailability::Ready,
            }
        }
        Ok(codex_login::ProviderApiKeyStorageMetadata::Missing)
        | Ok(codex_login::ProviderApiKeyStorageMetadata::Suppressed) => {
            CorbanuPlanMetadata::NotConfigured
        }
        Err(_) => CorbanuPlanMetadata::Unavailable,
    }
}

fn environment_metadata(env_key: &str) -> EnvironmentCredentialMetadata {
    match std::env::var(env_key) {
        Ok(value) if value.trim().is_empty() => EnvironmentCredentialMetadata::Invalid,
        Ok(_) => EnvironmentCredentialMetadata::Present,
        Err(std::env::VarError::NotPresent) => EnvironmentCredentialMetadata::Missing,
        Err(std::env::VarError::NotUnicode(_)) => EnvironmentCredentialMetadata::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use codex_model_provider_info::ModelProviderInfo;
    use codex_provider_auth::ProviderConfigurationState;
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn correlated_corbanu_metadata_resolves_without_credential_reread() {
        let home = tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        config.model_providers = codex_model_provider_info::built_in_model_providers(None);
        let host = ProviderStatusHost::from_config(
            &config,
            ProviderAccountMetadata {
                corbanu: CorbanuPlanMetadata::Configured {
                    source: CorbanuCredentialSource::Managed,
                    availability: ConfiguredAvailability::Ready,
                },
                ..Default::default()
            },
        );

        let status = host
            .resolve_provider(codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID)
            .unwrap();

        assert_eq!(status.configuration, ProviderConfigurationState::Configured);
    }

    #[tokio::test]
    async fn custom_storage_metadata_is_secret_free_and_missing_file_is_active() {
        let home = tempdir().unwrap();
        let config = config(
            home.path(),
            "custom",
            ModelProviderInfo {
                name: "Custom".into(),
                env_key: Some("PF53_STATUS_HOST_MISSING_KEY".into()),
                ..Default::default()
            },
        )
        .await;
        let host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
        let resolved = host.resolve();
        let status = resolved.get("custom").unwrap();
        assert_eq!(
            status.configuration,
            ProviderConfigurationState::NotConfigured
        );
        assert!(!format!("{resolved:?}").contains("secret"));
    }

    async fn config(path: &std::path::Path, id: &str, provider: ModelProviderInfo) -> Config {
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(path.to_path_buf())
            .build()
            .await
            .unwrap();
        config.model_provider_id = id.into();
        config.model_provider = provider.clone();
        config.model_providers = std::collections::HashMap::from([(id.into(), provider)]);
        config
    }
}
