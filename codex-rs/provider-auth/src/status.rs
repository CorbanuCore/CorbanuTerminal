use codex_login::OpenAiAuthMetadata;
use codex_login::ProviderApiKeyStorageSource;
use codex_model_provider_info::canonical_provider_id;

use crate::ApiKeyStorage;
use crate::ProviderActivationPolicy;
use crate::ProviderCatalog;
use crate::ProviderCatalogEntry;
use crate::ProviderSetupCapability;
use crate::status_contract::*;

pub struct ProviderStatusResolver;

impl ProviderStatusResolver {
    pub fn resolve(
        catalog: &ProviderCatalog,
        metadata: &ProviderMetadataSnapshot,
        eligibility: &ProviderEligibilitySnapshot,
        current: &CurrentProviderSelection,
    ) -> ProviderStatusCatalog {
        ProviderStatusCatalog {
            entries: catalog
                .entries()
                .iter()
                .map(|entry| resolve_entry(entry, metadata.get(entry), eligibility, current))
                .collect(),
        }
    }
}

fn resolve_entry(
    entry: &ProviderCatalogEntry,
    metadata: Option<ProviderMetadata>,
    eligibility: &ProviderEligibilitySnapshot,
    current: &CurrentProviderSelection,
) -> ProviderStatusSnapshot {
    let methods = entry
        .setup_capabilities
        .iter()
        .map(|capability| ProviderMethodStatus {
            capability: capability.clone(),
            state: resolve_method(capability, metadata),
        })
        .collect::<Vec<_>>();
    let configuration = configuration_state(&methods);
    let eligibility = match (configuration, eligibility) {
        (_, ProviderEligibilitySnapshot::Unavailable(_)) => ProviderEligibilityState::Unavailable,
        (ProviderConfigurationState::Configured, ProviderEligibilitySnapshot::Loaded(policy)) => {
            match policy.policy_for(entry) {
                ProviderActivationPolicy::Active => ProviderEligibilityState::Active,
                ProviderActivationPolicy::Inactive => ProviderEligibilityState::Inactive,
            }
        }
        (_, ProviderEligibilitySnapshot::Loaded(_)) => ProviderEligibilityState::NotConfigured,
    };
    ProviderStatusSnapshot {
        id: entry.id.clone(),
        availability: availability_state(&methods, configuration),
        configuration,
        eligibility,
        current: current_state(entry, current),
        methods,
    }
}

fn resolve_method(
    capability: &ProviderSetupCapability,
    metadata: Option<ProviderMetadata>,
) -> ProviderMethodState {
    let Some(metadata) = metadata else {
        return missing_adapter();
    };
    match metadata {
        ProviderMetadata::Checking => ProviderMethodState::Checking,
        ProviderMetadata::Unavailable => unavailable(ProviderUnavailableReason::CredentialMetadata),
        ProviderMetadata::OpenAi(auth) => resolve_openai(capability, auth),
        ProviderMetadata::ApiKey(api_key) => resolve_api_key(capability, api_key),
        ProviderMetadata::Claude(claude) => resolve_claude(capability, claude),
        ProviderMetadata::CorbanuPlan(plan) => resolve_corbanu(capability, plan),
        ProviderMetadata::Local(local) => resolve_local(capability, local),
        ProviderMetadata::CommandAuth(CommandAuthMetadata::StatusOnly)
            if matches!(capability, ProviderSetupCapability::CommandAuth { .. }) =>
        {
            configured(
                ProviderCredentialSource::ExternallyManaged,
                CredentialControl::ExternalProvider,
                ConfiguredAvailability::Ready,
            )
        }
        ProviderMetadata::StatusOnly
            if matches!(capability, ProviderSetupCapability::StatusOnly { .. }) =>
        {
            ProviderMethodState::StatusOnly
        }
        ProviderMetadata::CommandAuth(_) | ProviderMetadata::StatusOnly => missing_adapter(),
    }
}

fn resolve_openai(
    capability: &ProviderSetupCapability,
    metadata: OpenAiAuthMetadata,
) -> ProviderMethodState {
    let is_account = matches!(capability, ProviderSetupCapability::OpenAiAccount);
    let is_api_key = matches!(
        capability,
        ProviderSetupCapability::ApiKey {
            storage: ApiKeyStorage::OpenAiAuth
        }
    );
    if !is_account && !is_api_key {
        return missing_adapter();
    }
    match metadata {
        OpenAiAuthMetadata::Missing => ProviderMethodState::NotConfigured,
        OpenAiAuthMetadata::Account if is_account => configured(
            ProviderCredentialSource::OpenAiAccount,
            CredentialControl::ManagedByCorbanu,
            ConfiguredAvailability::Ready,
        ),
        OpenAiAuthMetadata::ApiKey if is_api_key => configured(
            ProviderCredentialSource::OpenAiApiKey,
            CredentialControl::ManagedByCorbanu,
            ConfiguredAvailability::Ready,
        ),
        OpenAiAuthMetadata::ExternallyManaged if is_account => configured(
            ProviderCredentialSource::ExternallyManaged,
            CredentialControl::ExternalProvider,
            ConfiguredAvailability::Ready,
        ),
        OpenAiAuthMetadata::Account
        | OpenAiAuthMetadata::ApiKey
        | OpenAiAuthMetadata::ExternallyManaged => ProviderMethodState::NotConfigured,
        OpenAiAuthMetadata::RecoveryRequired => {
            recovery(ProviderRecoveryReason::OpenAiRefreshRequired)
        }
        OpenAiAuthMetadata::Unsupported => recovery(ProviderRecoveryReason::UnsupportedAuthMode),
    }
}

fn resolve_api_key(
    capability: &ProviderSetupCapability,
    metadata: ApiKeyCredentialMetadata,
) -> ProviderMethodState {
    if !matches!(
        capability,
        ProviderSetupCapability::ApiKey {
            storage: ApiKeyStorage::EnvironmentVariable { .. }
        }
    ) {
        return missing_adapter();
    }
    match metadata.environment {
        EnvironmentCredentialMetadata::Present => configured(
            ProviderCredentialSource::Environment,
            CredentialControl::ExternalEnvironment,
            ConfiguredAvailability::Ready,
        ),
        EnvironmentCredentialMetadata::Missing => match metadata.managed {
            ManagedApiKeyMetadata::Stored { source } => configured(
                match source {
                    ProviderApiKeyStorageSource::EncryptedVault => {
                        ProviderCredentialSource::EncryptedVault
                    }
                    ProviderApiKeyStorageSource::LegacyPlaintext => {
                        ProviderCredentialSource::LegacyPlaintext
                    }
                },
                CredentialControl::ManagedByCorbanu,
                ConfiguredAvailability::Ready,
            ),
            ManagedApiKeyMetadata::Missing | ManagedApiKeyMetadata::Suppressed => {
                ProviderMethodState::NotConfigured
            }
            ManagedApiKeyMetadata::Unavailable => {
                unavailable(ProviderUnavailableReason::CredentialMetadata)
            }
        },
        EnvironmentCredentialMetadata::Invalid => {
            recovery(ProviderRecoveryReason::InvalidEnvironmentCredential)
        }
    }
}

fn resolve_claude(
    capability: &ProviderSetupCapability,
    metadata: ClaudeCredentialMetadata,
) -> ProviderMethodState {
    if !matches!(capability, ProviderSetupCapability::ClaudeAccount) {
        return missing_adapter();
    }
    match metadata {
        ClaudeCredentialMetadata::Checking => ProviderMethodState::Checking,
        ClaudeCredentialMetadata::NotConfigured => ProviderMethodState::NotConfigured,
        ClaudeCredentialMetadata::Unavailable => {
            unavailable(ProviderUnavailableReason::CredentialMetadata)
        }
        ClaudeCredentialMetadata::RecoveryRequired { reason } => recovery(reason),
        ClaudeCredentialMetadata::Configured { source } => configured(
            match source {
                ClaudeCredentialSource::Managed => ProviderCredentialSource::ClaudeManaged,
                ClaudeCredentialSource::Environment => ProviderCredentialSource::ClaudeEnvironment,
                ClaudeCredentialSource::ClaudeCodeLogin => {
                    ProviderCredentialSource::ClaudeCodeLogin
                }
            },
            match source {
                ClaudeCredentialSource::Managed => CredentialControl::ManagedByCorbanu,
                ClaudeCredentialSource::Environment => CredentialControl::ExternalEnvironment,
                ClaudeCredentialSource::ClaudeCodeLogin => CredentialControl::ExternalProvider,
            },
            ConfiguredAvailability::Ready,
        ),
    }
}

fn resolve_corbanu(
    capability: &ProviderSetupCapability,
    metadata: CorbanuPlanMetadata,
) -> ProviderMethodState {
    if !matches!(capability, ProviderSetupCapability::CorbanuPlan) {
        return missing_adapter();
    }
    match metadata {
        CorbanuPlanMetadata::Checking => ProviderMethodState::Checking,
        CorbanuPlanMetadata::NotConfigured => ProviderMethodState::NotConfigured,
        CorbanuPlanMetadata::Unavailable => {
            unavailable(ProviderUnavailableReason::CredentialMetadata)
        }
        CorbanuPlanMetadata::RecoveryRequired => {
            recovery(ProviderRecoveryReason::CorbanuCredentialRejected)
        }
        CorbanuPlanMetadata::Configured {
            source,
            availability,
        } => configured(
            ProviderCredentialSource::CorbanuPlan,
            match source {
                CorbanuCredentialSource::Managed => CredentialControl::ManagedByCorbanu,
                CorbanuCredentialSource::Environment => CredentialControl::ExternalEnvironment,
            },
            availability,
        ),
    }
}

fn resolve_local(
    capability: &ProviderSetupCapability,
    metadata: LocalProviderMetadata,
) -> ProviderMethodState {
    if !matches!(capability, ProviderSetupCapability::Local { .. }) {
        return missing_adapter();
    }
    match metadata {
        LocalProviderMetadata::Checking => ProviderMethodState::Checking,
        LocalProviderMetadata::Available => configured(
            ProviderCredentialSource::Local,
            CredentialControl::None,
            ConfiguredAvailability::Ready,
        ),
        LocalProviderMetadata::Unavailable => unavailable(ProviderUnavailableReason::LocalRuntime),
    }
}

fn configured(
    source: ProviderCredentialSource,
    control: CredentialControl,
    availability: ConfiguredAvailability,
) -> ProviderMethodState {
    ProviderMethodState::Configured {
        source,
        control,
        availability,
    }
}

fn unavailable(reason: ProviderUnavailableReason) -> ProviderMethodState {
    ProviderMethodState::Unavailable { reason }
}

fn recovery(reason: ProviderRecoveryReason) -> ProviderMethodState {
    ProviderMethodState::RecoveryRequired { reason }
}

fn missing_adapter() -> ProviderMethodState {
    recovery(ProviderRecoveryReason::MissingMetadataAdapter)
}

fn configuration_state(methods: &[ProviderMethodStatus]) -> ProviderConfigurationState {
    if methods
        .iter()
        .any(|method| matches!(method.state, ProviderMethodState::Configured { .. }))
    {
        ProviderConfigurationState::Configured
    } else if methods
        .iter()
        .any(|method| matches!(method.state, ProviderMethodState::RecoveryRequired { .. }))
    {
        ProviderConfigurationState::RecoveryRequired
    } else if methods
        .iter()
        .any(|method| matches!(method.state, ProviderMethodState::Unavailable { .. }))
    {
        ProviderConfigurationState::Unavailable
    } else if methods
        .iter()
        .any(|method| matches!(method.state, ProviderMethodState::Checking))
    {
        ProviderConfigurationState::Checking
    } else {
        ProviderConfigurationState::NotConfigured
    }
}

fn availability_state(
    methods: &[ProviderMethodStatus],
    configuration: ProviderConfigurationState,
) -> ProviderAvailabilityState {
    if methods.iter().any(|method| {
        matches!(
            method.state,
            ProviderMethodState::Configured {
                availability: ConfiguredAvailability::Ready,
                ..
            }
        )
    }) {
        return ProviderAvailabilityState::Ready;
    }
    if methods
        .iter()
        .any(|method| matches!(method.state, ProviderMethodState::StatusOnly))
        && methods.iter().all(|method| {
            matches!(
                method.state,
                ProviderMethodState::StatusOnly | ProviderMethodState::NotConfigured
            )
        })
    {
        return ProviderAvailabilityState::StatusOnly;
    }
    match configuration {
        ProviderConfigurationState::Checking => ProviderAvailabilityState::Checking,
        ProviderConfigurationState::RecoveryRequired => ProviderAvailabilityState::Unavailable {
            reason: ProviderUnavailableReason::RecoveryRequired,
        },
        ProviderConfigurationState::Configured | ProviderConfigurationState::Unavailable => {
            ProviderAvailabilityState::Unavailable {
                reason: ProviderUnavailableReason::ProviderService,
            }
        }
        ProviderConfigurationState::NotConfigured => ProviderAvailabilityState::Unavailable {
            reason: ProviderUnavailableReason::NotConfigured,
        },
    }
}

fn current_state(
    entry: &ProviderCatalogEntry,
    current: &CurrentProviderSelection,
) -> ProviderCurrentState {
    let CurrentProviderSelection::RuntimeId(current) = current else {
        return ProviderCurrentState::NotCurrent;
    };
    let canonical_current = canonical_provider_id(current);
    if entry.runtime_provider_ids.iter().any(|runtime_id| {
        runtime_id.as_str() == current
            || canonical_provider_id(runtime_id.as_str()) == canonical_current
    }) {
        ProviderCurrentState::Current
    } else {
        ProviderCurrentState::NotCurrent
    }
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
