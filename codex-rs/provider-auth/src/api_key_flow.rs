use std::fmt;

use zeroize::Zeroizing;

use crate::ApiKeyCredentialMetadata;
use crate::ApiKeyStorage;
use crate::ProviderCatalogEntry;
use crate::ProviderCatalogId;
use crate::ProviderRuntimeId;
use crate::ProviderSetupCapability;

/// A zeroizing, non-serializable API key carried only to the persistence effect.
#[derive(PartialEq, Eq)]
pub struct ApiKeySecret(Zeroizing<String>);

impl ApiKeySecret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for ApiKeySecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiKeySecret(<redacted>)")
    }
}

/// Why a catalog entry cannot start the generic API-key flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyTargetError {
    UnsupportedCapability,
    MissingRuntimeProvider,
}

/// Generic persistence identity derived from a catalog API-key capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyAuthTarget {
    pub provider_id: ProviderCatalogId,
    pub runtime_provider_id: ProviderRuntimeId,
    pub env_key: String,
}

impl ApiKeyAuthTarget {
    pub fn from_catalog_entry(entry: &ProviderCatalogEntry) -> Result<Self, ApiKeyTargetError> {
        let env_key = entry
            .setup_capabilities
            .iter()
            .find_map(|capability| match capability {
                ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::EnvironmentVariable { env_key },
                } => Some(env_key.clone()),
                ProviderSetupCapability::OpenAiAccount
                | ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::OpenAiAuth,
                }
                | ProviderSetupCapability::ClaudeAccount
                | ProviderSetupCapability::CorbanuPlan
                | ProviderSetupCapability::Local { .. }
                | ProviderSetupCapability::CommandAuth { .. }
                | ProviderSetupCapability::StatusOnly { .. } => None,
            })
            .ok_or(ApiKeyTargetError::UnsupportedCapability)?;
        let runtime_provider_id = entry
            .runtime_provider_ids
            .first()
            .cloned()
            .ok_or(ApiKeyTargetError::MissingRuntimeProvider)?;
        Ok(Self {
            provider_id: entry.id.clone(),
            runtime_provider_id,
            env_key,
        })
    }
}

/// Add and replacement are explicit user intents even though both persist through one adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyFlowIntent {
    Add,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyFlowContext {
    pub target: ApiKeyAuthTarget,
    pub intent: ApiKeyFlowIntent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyFlowStart {
    pub target: ApiKeyAuthTarget,
    pub intent: ApiKeyFlowIntent,
    pub metadata: ApiKeyCredentialMetadata,
}

#[cfg(test)]
#[path = "api_key_flow_tests.rs"]
mod tests;
