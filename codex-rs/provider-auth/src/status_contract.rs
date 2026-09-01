use std::collections::BTreeMap;

use codex_login::OpenAiAuthMetadata;
use codex_login::ProviderApiKeyStorageMetadata;
use codex_login::ProviderApiKeyStorageSource;
use codex_vault::ClaudeAuthResolution;
use codex_vault::ClaudeAuthSource;

use crate::ProviderCatalogEntry;
use crate::ProviderCatalogId;
use crate::ProviderEligibility;
use crate::ProviderEligibilityError;
use crate::ProviderSetupCapability;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentCredentialMetadata {
    Present,
    Missing,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedApiKeyMetadata {
    Stored { source: ProviderApiKeyStorageSource },
    Missing,
    Suppressed,
    Unavailable,
}

impl From<Result<ProviderApiKeyStorageMetadata, std::io::Error>> for ManagedApiKeyMetadata {
    fn from(value: Result<ProviderApiKeyStorageMetadata, std::io::Error>) -> Self {
        match value {
            Ok(ProviderApiKeyStorageMetadata::Stored { source }) => Self::Stored { source },
            Ok(ProviderApiKeyStorageMetadata::Missing) => Self::Missing,
            Ok(ProviderApiKeyStorageMetadata::Suppressed) => Self::Suppressed,
            Err(_) => Self::Unavailable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiKeyCredentialMetadata {
    pub environment: EnvironmentCredentialMetadata,
    pub managed: ManagedApiKeyMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCredentialMetadata {
    Checking,
    NotConfigured,
    Configured { source: ClaudeCredentialSource },
    Unavailable,
    RecoveryRequired { reason: ProviderRecoveryReason },
}

impl From<ClaudeAuthResolution> for ClaudeCredentialMetadata {
    fn from(resolution: ClaudeAuthResolution) -> Self {
        match resolution {
            ClaudeAuthResolution::SelectionRequired { available } if available.is_empty() => {
                Self::NotConfigured
            }
            ClaudeAuthResolution::SelectionRequired { .. } => Self::RecoveryRequired {
                reason: ProviderRecoveryReason::AmbiguousClaudeSources,
            },
            ClaudeAuthResolution::Selected(metadata) => Self::Configured {
                source: match metadata.source {
                    ClaudeAuthSource::ManagedSubscriptionToken => ClaudeCredentialSource::Managed,
                    ClaudeAuthSource::EnvironmentToken => ClaudeCredentialSource::Environment,
                    ClaudeAuthSource::ClaudeCodeLogin => ClaudeCredentialSource::ClaudeCodeLogin,
                },
            },
            ClaudeAuthResolution::MissingSelected(_) => Self::RecoveryRequired {
                reason: ProviderRecoveryReason::MissingClaudeSelection,
            },
            ClaudeAuthResolution::UnhealthySelected(_) => Self::RecoveryRequired {
                reason: ProviderRecoveryReason::UnhealthyClaudeSelection,
            },
            ClaudeAuthResolution::Conflict { .. } => Self::RecoveryRequired {
                reason: ProviderRecoveryReason::AmbiguousClaudeSources,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCredentialSource {
    Managed,
    Environment,
    ClaudeCodeLogin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorbanuPlanMetadata {
    Checking,
    NotConfigured,
    Configured {
        source: CorbanuCredentialSource,
        availability: ConfiguredAvailability,
    },
    RecoveryRequired,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorbanuCredentialSource {
    Managed,
    Environment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalProviderMetadata {
    Checking,
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAuthMetadata {
    StatusOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderMetadata {
    Checking,
    OpenAi(OpenAiAuthMetadata),
    ApiKey(ApiKeyCredentialMetadata),
    Claude(ClaudeCredentialMetadata),
    CorbanuPlan(CorbanuPlanMetadata),
    Local(LocalProviderMetadata),
    CommandAuth(CommandAuthMetadata),
    StatusOnly,
    Unavailable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderMetadataSnapshot {
    providers: BTreeMap<String, ProviderMetadata>,
}

impl ProviderMetadataSnapshot {
    pub fn insert(&mut self, entry: &ProviderCatalogEntry, metadata: ProviderMetadata) {
        self.providers
            .insert(entry.id.as_str().to_string(), metadata);
    }

    pub(crate) fn get(&self, entry: &ProviderCatalogEntry) -> Option<ProviderMetadata> {
        self.providers.get(entry.id.as_str()).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderEligibilitySnapshot {
    Loaded(ProviderEligibility),
    Unavailable(ProviderEligibilityError),
}

impl From<Result<ProviderEligibility, ProviderEligibilityError>> for ProviderEligibilitySnapshot {
    fn from(value: Result<ProviderEligibility, ProviderEligibilityError>) -> Self {
        match value {
            Ok(eligibility) => Self::Loaded(eligibility),
            Err(error) => Self::Unavailable(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentProviderSelection {
    None,
    RuntimeId(String),
}

impl CurrentProviderSelection {
    pub fn runtime_id(runtime_id: impl Into<String>) -> Self {
        Self::RuntimeId(runtime_id.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStatusCatalog {
    pub(crate) entries: Vec<ProviderStatusSnapshot>,
}

impl ProviderStatusCatalog {
    pub fn entries(&self) -> &[ProviderStatusSnapshot] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&ProviderStatusSnapshot> {
        self.entries.iter().find(|entry| entry.id.as_str() == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderStatusSnapshot {
    pub id: ProviderCatalogId,
    pub methods: Vec<ProviderMethodStatus>,
    pub configuration: ProviderConfigurationState,
    pub eligibility: ProviderEligibilityState,
    pub current: ProviderCurrentState,
    pub availability: ProviderAvailabilityState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderMethodStatus {
    pub capability: ProviderSetupCapability,
    pub state: ProviderMethodState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderMethodState {
    Checking,
    NotConfigured,
    Configured {
        source: ProviderCredentialSource,
        control: CredentialControl,
        availability: ConfiguredAvailability,
    },
    Unavailable {
        reason: ProviderUnavailableReason,
    },
    RecoveryRequired {
        reason: ProviderRecoveryReason,
    },
    StatusOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCredentialSource {
    Environment,
    EncryptedVault,
    LegacyPlaintext,
    OpenAiAccount,
    OpenAiApiKey,
    ExternallyManaged,
    ClaudeManaged,
    ClaudeEnvironment,
    ClaudeCodeLogin,
    CorbanuPlan,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialControl {
    ManagedByCorbanu,
    ExternalEnvironment,
    ExternalProvider,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfiguredAvailability {
    Ready,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRecoveryReason {
    InvalidEnvironmentCredential,
    OpenAiRefreshRequired,
    MissingMetadataAdapter,
    UnsupportedAuthMode,
    AmbiguousClaudeSources,
    MissingClaudeSelection,
    UnhealthyClaudeSelection,
    CorbanuCredentialRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderUnavailableReason {
    CredentialMetadata,
    ProviderService,
    LocalRuntime,
    NotConfigured,
    RecoveryRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderConfigurationState {
    Checking,
    NotConfigured,
    Configured,
    Unavailable,
    RecoveryRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderEligibilityState {
    Active,
    Inactive,
    NotConfigured,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCurrentState {
    Current,
    NotCurrent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAvailabilityState {
    Checking,
    Ready,
    Unavailable { reason: ProviderUnavailableReason },
    StatusOnly,
}
