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
use codex_provider_auth::ProviderEligibilityError;
use codex_provider_auth::ProviderEligibilitySnapshot;
use codex_provider_auth::ProviderEligibilityStore;
use codex_provider_auth::ProviderMetadata;
use codex_provider_auth::ProviderMetadataSnapshot;
use codex_provider_auth::ProviderSetupCapability;
use codex_provider_auth::ProviderStatusCatalog;
use codex_provider_auth::ProviderStatusResolver;
use codex_provider_auth::ProviderStatusSnapshot;

use crate::legacy_core::config::Config;

type ManagedSnapshot = std::io::Result<codex_login::ProviderApiKeyStorageMetadataSnapshot>;

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
#[derive(Clone, Debug)]
pub(crate) struct ProviderStatusHost {
    catalog: ProviderCatalog,
    codex_home: PathBuf,
    current: Arc<RwLock<CurrentProviderSelection>>,
    account: Arc<RwLock<ProviderAccountMetadata>>,
}

impl ProviderStatusHost {
    pub(crate) fn from_config(config: &Config, account: ProviderAccountMetadata) -> Self {
        Self {
            catalog: ProviderCatalog::from_runtime_providers(&config.model_providers),
            codex_home: config.codex_home.to_path_buf(),
            current: Arc::new(RwLock::new(CurrentProviderSelection::runtime_id(
                config.model_provider_id.clone(),
            ))),
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
            ApiKeyStorage::EnvironmentVariable { env_key } => {
                self.environment_api_key_metadata(env_key, /*snapshot*/ None)
            }
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
        let managed_keys = self.catalog.entries().iter().filter_map(|entry| {
            match entry.setup_capabilities.iter().next() {
                Some(ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::EnvironmentVariable { env_key },
                }) if environment_metadata(env_key) == EnvironmentCredentialMetadata::Missing => {
                    Some(env_key.as_str())
                }
                _ => None,
            }
        });
        let managed = codex_login::provider_api_key_metadata_snapshot_from_auth_storage(
            &self.codex_home,
            managed_keys,
        );
        let mut metadata = ProviderMetadataSnapshot::default();
        for entry in self.catalog.entries() {
            metadata.insert(entry, self.metadata_for(entry, account, Some(&managed)));
        }
        ProviderStatusResolver::resolve(
            &self.catalog,
            &metadata,
            &ProviderEligibilitySnapshot::from(
                ProviderEligibilityStore::new(&self.codex_home).load(),
            ),
            &self
                .current
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    pub(crate) fn resolve_provider(&self, provider_id: &str) -> Option<ProviderStatusSnapshot> {
        let entry = self.catalog.get(provider_id)?;
        let account = *self
            .account
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut metadata = ProviderMetadataSnapshot::default();
        metadata.insert(entry, self.metadata_for(entry, account, /*managed*/ None));
        ProviderStatusResolver::resolve(
            &self.catalog,
            &metadata,
            &ProviderEligibilitySnapshot::from(
                ProviderEligibilityStore::new(&self.codex_home).load(),
            ),
            &self
                .current
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
        .get(provider_id)
        .cloned()
    }
    pub(crate) fn resolve_target(&self, target: &ApiKeyAuthTarget) -> ProviderStatusSnapshot {
        self.resolve_provider(target.provider_id.as_str())
            .unwrap_or_else(|| panic!("catalog target disappeared: {}", target.provider_id))
    }

    pub(crate) fn persist_policy(
        &self,
        provider_id: &str,
        policy: ProviderActivationPolicy,
    ) -> Result<(), ProviderEligibilityError> {
        let entry = self
            .catalog
            .get(provider_id)
            .ok_or(ProviderEligibilityError::WriteUnavailable)?;
        let store = ProviderEligibilityStore::new(&self.codex_home);
        let mut eligibility = store.load()?;
        eligibility.set_policy(entry, policy);
        store.save(&eligibility)
    }

    pub(crate) fn activate(&self, provider_id: &str) -> bool {
        self.persist_policy(provider_id, ProviderActivationPolicy::Active)
            .is_ok()
    }
    pub(crate) fn set_current_runtime(&self, runtime_provider_id: impl Into<String>) {
        *self
            .current
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            CurrentProviderSelection::runtime_id(runtime_provider_id);
    }

    fn metadata_for(
        &self,
        entry: &ProviderCatalogEntry,
        account: ProviderAccountMetadata,
        managed: Option<&ManagedSnapshot>,
    ) -> ProviderMetadata {
        let first = entry.setup_capabilities.iter().next();
        match first {
            Some(ProviderSetupCapability::OpenAiAccount)
            | Some(ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::OpenAiAuth,
            }) => ProviderMetadata::OpenAi(account.openai),
            Some(ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::EnvironmentVariable { env_key },
            }) => ProviderMetadata::ApiKey(self.environment_api_key_metadata(env_key, managed)),
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

    fn environment_api_key_metadata(
        &self,
        env_key: &str,
        snapshot: Option<&ManagedSnapshot>,
    ) -> ApiKeyCredentialMetadata {
        let environment = environment_metadata(env_key);
        prioritized_api_key_metadata(environment, || match snapshot {
            Some(Ok(snapshot)) => snapshot
                .get(env_key)
                .map(|metadata| ManagedApiKeyMetadata::from(Ok(metadata)))
                .unwrap_or(ManagedApiKeyMetadata::Unavailable),
            Some(Err(_)) => ManagedApiKeyMetadata::Unavailable,
            None => ManagedApiKeyMetadata::from(
                codex_login::provider_api_key_metadata_from_auth_storage(&self.codex_home, env_key),
            ),
        })
    }
}

fn prioritized_api_key_metadata(
    environment: EnvironmentCredentialMetadata,
    read_managed: impl FnOnce() -> ManagedApiKeyMetadata,
) -> ApiKeyCredentialMetadata {
    let managed = if environment == EnvironmentCredentialMetadata::Missing {
        read_managed()
    } else {
        ManagedApiKeyMetadata::Missing
    };
    ApiKeyCredentialMetadata {
        environment,
        managed,
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
#[path = "provider_status_host_tests.rs"]
mod tests;
