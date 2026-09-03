use std::collections::HashMap;

use codex_login::OpenAiAuthMetadata;
use codex_login::ProviderApiKeyStorageSource;
use codex_model_provider_info::AMAZON_BEDROCK_PROVIDER_ID;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_model_provider_info::built_in_model_providers;
use codex_vault::ClaudeAuthHealth;
use codex_vault::ClaudeAuthResolution;
use codex_vault::ClaudeAuthSource;
use codex_vault::ClaudeAuthSourceMetadata;
use codex_vault::ClaudeAuthStoreKind;
use pretty_assertions::assert_eq;

use super::*;
use crate::CommandAuthSetup;
use crate::ProviderCatalog;
use crate::ProviderEligibility;
use crate::ProviderEligibilityError;
use crate::ProviderEligibilityId;
use crate::ProviderSetupCapabilities;
use crate::StatusOnlyReason;

#[test]
fn openai_preserves_capability_order_and_account_configuration() {
    let catalog = built_in_catalog();
    let entry = catalog.get(OPENAI_PROVIDER_ID).expect("OpenAI entry");
    let mut metadata = ProviderMetadataSnapshot::default();
    metadata.insert(entry, ProviderMetadata::OpenAi(OpenAiAuthMetadata::Account));

    let statuses = ProviderStatusResolver::resolve(
        &catalog,
        &metadata,
        &loaded_default_eligibility(),
        &CurrentProviderSelection::runtime_id(OPENAI_PROVIDER_ID),
    );

    assert_eq!(
        statuses.get(OPENAI_PROVIDER_ID),
        Some(&ProviderStatusSnapshot {
            id: entry.id.clone(),
            methods: vec![
                ProviderMethodStatus {
                    capability: ProviderSetupCapability::OpenAiAccount,
                    state: configured(
                        ProviderCredentialSource::OpenAiAccount,
                        CredentialControl::ManagedByCorbanu,
                        ConfiguredAvailability::Ready,
                    ),
                },
                ProviderMethodStatus {
                    capability: ProviderSetupCapability::ApiKey {
                        storage: ApiKeyStorage::OpenAiAuth,
                    },
                    state: ProviderMethodState::NotConfigured,
                },
            ],
            configuration: ProviderConfigurationState::Configured,
            eligibility: ProviderEligibilityState::Active,
            current: ProviderCurrentState::Current,
            availability: ProviderAvailabilityState::Ready,
        })
    );
}

#[test]
fn api_key_precedence_and_inactive_policy_keep_environment_removal_external() {
    let catalog = custom_api_catalog();
    let entry = &catalog.entries()[0];
    let mut metadata = ProviderMetadataSnapshot::default();
    metadata.insert(
        entry,
        ProviderMetadata::ApiKey(ApiKeyCredentialMetadata {
            environment: EnvironmentCredentialMetadata::Present,
            managed: ManagedApiKeyMetadata::Stored {
                source: ProviderApiKeyStorageSource::EncryptedVault,
            },
        }),
    );
    let mut eligibility = ProviderEligibility::default();
    eligibility.set_policy(entry, ProviderActivationPolicy::Inactive);

    let statuses = ProviderStatusResolver::resolve(
        &catalog,
        &metadata,
        &ProviderEligibilitySnapshot::Loaded(eligibility),
        &CurrentProviderSelection::runtime_id("custom"),
    );

    assert_eq!(
        statuses.get("custom"),
        Some(&ProviderStatusSnapshot {
            id: entry.id.clone(),
            methods: vec![ProviderMethodStatus {
                capability: ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::EnvironmentVariable {
                        env_key: "CUSTOM_API_KEY".to_string(),
                    },
                },
                state: configured(
                    ProviderCredentialSource::Environment,
                    CredentialControl::ExternalEnvironment,
                    ConfiguredAvailability::Ready,
                ),
            }],
            configuration: ProviderConfigurationState::Configured,
            eligibility: ProviderEligibilityState::Inactive,
            current: ProviderCurrentState::Current,
            availability: ProviderAvailabilityState::Ready,
        })
    );
    assert_eq!(
        ProviderEligibilityId::for_entry(entry).as_str(),
        "credential-env:CUSTOM_API_KEY"
    );

    metadata.insert(
        entry,
        ProviderMetadata::ApiKey(ApiKeyCredentialMetadata {
            environment: EnvironmentCredentialMetadata::Missing,
            managed: ManagedApiKeyMetadata::Stored {
                source: ProviderApiKeyStorageSource::EncryptedVault,
            },
        }),
    );
    let restarted = ProviderStatusResolver::resolve(
        &catalog,
        &metadata,
        &loaded_default_eligibility(),
        &CurrentProviderSelection::None,
    );
    assert_eq!(
        restarted.get("custom").expect("custom status").methods[0].state,
        configured(
            ProviderCredentialSource::EncryptedVault,
            CredentialControl::ManagedByCorbanu,
            ConfiguredAvailability::Ready,
        )
    );
}

#[test]
fn invalid_environment_key_shadows_all_managed_storage_states() {
    let catalog = custom_api_catalog();
    let entry = &catalog.entries()[0];
    let expected = ProviderStatusSnapshot {
        id: entry.id.clone(),
        methods: vec![ProviderMethodStatus {
            capability: ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::EnvironmentVariable {
                    env_key: "CUSTOM_API_KEY".to_string(),
                },
            },
            state: ProviderMethodState::RecoveryRequired {
                reason: ProviderRecoveryReason::InvalidEnvironmentCredential,
            },
        }],
        configuration: ProviderConfigurationState::RecoveryRequired,
        eligibility: ProviderEligibilityState::NotConfigured,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Unavailable {
            reason: ProviderUnavailableReason::RecoveryRequired,
        },
    };

    for managed in [
        ManagedApiKeyMetadata::Stored {
            source: ProviderApiKeyStorageSource::EncryptedVault,
        },
        ManagedApiKeyMetadata::Missing,
        ManagedApiKeyMetadata::Suppressed,
    ] {
        let mut metadata = ProviderMetadataSnapshot::default();
        metadata.insert(
            entry,
            ProviderMetadata::ApiKey(ApiKeyCredentialMetadata {
                environment: EnvironmentCredentialMetadata::Invalid,
                managed,
            }),
        );

        let statuses = ProviderStatusResolver::resolve(
            &catalog,
            &metadata,
            &loaded_default_eligibility(),
            &CurrentProviderSelection::None,
        );

        assert_eq!(statuses.get("custom"), Some(&expected));
    }
}

#[test]
fn claude_resolution_and_corbanu_outage_are_typed_without_losing_configuration() {
    let catalog = built_in_catalog();
    let claude = catalog.get(CLAUDE_PLAN_PROVIDER_ID).expect("Claude entry");
    let corbanu = catalog
        .get(CORBANU_PLAN_PROVIDER_ID)
        .expect("Corbanu entry");
    let source = ClaudeAuthSourceMetadata {
        source: ClaudeAuthSource::EnvironmentToken,
        source_id: "environment-token".to_string(),
        store: ClaudeAuthStoreKind::Environment,
        health: ClaudeAuthHealth::Healthy,
        account_hint: None,
    };
    assert_eq!(
        ClaudeCredentialMetadata::from(ClaudeAuthResolution::SelectionRequired {
            available: vec![source.clone()],
        }),
        ClaudeCredentialMetadata::RecoveryRequired {
            reason: ProviderRecoveryReason::AmbiguousClaudeSources,
        }
    );

    let mut metadata = ProviderMetadataSnapshot::default();
    metadata.insert(
        claude,
        ProviderMetadata::Claude(ClaudeCredentialMetadata::from(
            ClaudeAuthResolution::Selected(source),
        )),
    );
    metadata.insert(
        corbanu,
        ProviderMetadata::CorbanuPlan(CorbanuPlanMetadata::Configured {
            source: CorbanuCredentialSource::Managed,
            availability: ConfiguredAvailability::Unavailable,
        }),
    );
    let statuses = ProviderStatusResolver::resolve(
        &catalog,
        &metadata,
        &loaded_default_eligibility(),
        &CurrentProviderSelection::None,
    );

    assert_eq!(
        (
            statuses
                .get(CLAUDE_PLAN_PROVIDER_ID)
                .expect("Claude status")
                .methods[0]
                .state,
            statuses
                .get(CORBANU_PLAN_PROVIDER_ID)
                .expect("Corbanu status")
                .configuration,
            statuses
                .get(CORBANU_PLAN_PROVIDER_ID)
                .expect("Corbanu status")
                .availability,
        ),
        (
            configured(
                ProviderCredentialSource::ClaudeEnvironment,
                CredentialControl::ExternalEnvironment,
                ConfiguredAvailability::Ready,
            ),
            ProviderConfigurationState::Configured,
            ProviderAvailabilityState::Unavailable {
                reason: ProviderUnavailableReason::ProviderService,
            },
        )
    );
}

#[test]
fn local_and_command_adapters_are_configured_while_aws_remains_status_only() {
    let mut providers = built_in_model_providers(/*openai_base_url*/ None);
    let mut command = ModelProviderInfo::create_claude_plan_provider();
    command.name = "Custom Command".to_string();
    providers.insert("custom-command".to_string(), command);
    let catalog = ProviderCatalog::from_runtime_providers(&providers);
    let ollama = catalog.get(OLLAMA_OSS_PROVIDER_ID).expect("Ollama entry");
    let command = catalog.get("custom-command").expect("command entry");
    let aws = catalog.get(AMAZON_BEDROCK_PROVIDER_ID).expect("AWS entry");
    let mut metadata = ProviderMetadataSnapshot::default();
    metadata.insert(
        ollama,
        ProviderMetadata::Local(LocalProviderMetadata::Available),
    );
    metadata.insert(
        command,
        ProviderMetadata::CommandAuth(CommandAuthMetadata::StatusOnly),
    );
    metadata.insert(aws, ProviderMetadata::StatusOnly);

    let statuses = ProviderStatusResolver::resolve(
        &catalog,
        &metadata,
        &loaded_default_eligibility(),
        &CurrentProviderSelection::None,
    );

    assert_eq!(
        [
            OLLAMA_OSS_PROVIDER_ID,
            "custom-command",
            AMAZON_BEDROCK_PROVIDER_ID
        ]
        .map(|id| {
            let status = statuses.get(id).expect("provider status");
            (status.configuration, status.availability)
        }),
        [
            (
                ProviderConfigurationState::Configured,
                ProviderAvailabilityState::Ready,
            ),
            (
                ProviderConfigurationState::Configured,
                ProviderAvailabilityState::Ready,
            ),
            (
                ProviderConfigurationState::NotConfigured,
                ProviderAvailabilityState::StatusOnly,
            ),
        ]
    );
    assert_eq!(
        command.setup_capabilities,
        ProviderSetupCapabilities {
            primary: ProviderSetupCapability::CommandAuth {
                setup: CommandAuthSetup::StatusOnly,
            },
            alternatives: Vec::new(),
        }
    );
    assert_eq!(
        aws.setup_capabilities.primary,
        ProviderSetupCapability::StatusOnly {
            reason: StatusOnlyReason::AwsCredentialChain,
        }
    );
}

#[test]
fn partial_metadata_and_upstream_errors_fail_visibly_without_secret_canary() {
    let catalog = custom_api_catalog();
    let statuses = ProviderStatusResolver::resolve(
        &catalog,
        &ProviderMetadataSnapshot::default(),
        &ProviderEligibilitySnapshot::Unavailable(ProviderEligibilityError::Malformed),
        &CurrentProviderSelection::None,
    );
    let storage =
        ManagedApiKeyMetadata::from(Err(std::io::Error::other("secret-canary-upstream-error")));

    assert_eq!(storage, ManagedApiKeyMetadata::Unavailable);
    assert_eq!(
        statuses.get("custom"),
        Some(&ProviderStatusSnapshot {
            id: catalog.entries()[0].id.clone(),
            methods: vec![ProviderMethodStatus {
                capability: ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::EnvironmentVariable {
                        env_key: "CUSTOM_API_KEY".to_string(),
                    },
                },
                state: ProviderMethodState::RecoveryRequired {
                    reason: ProviderRecoveryReason::MissingMetadataAdapter,
                },
            }],
            configuration: ProviderConfigurationState::RecoveryRequired,
            eligibility: ProviderEligibilityState::Unavailable,
            current: ProviderCurrentState::NotCurrent,
            availability: ProviderAvailabilityState::Unavailable {
                reason: ProviderUnavailableReason::RecoveryRequired,
            },
        })
    );
    assert!(!format!("{storage:?} {statuses:?}").contains("secret-canary"));
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

fn built_in_catalog() -> ProviderCatalog {
    ProviderCatalog::from_runtime_providers(&built_in_model_providers(
        /*openai_base_url*/ None,
    ))
}

fn custom_api_catalog() -> ProviderCatalog {
    ProviderCatalog::from_runtime_providers(&HashMap::from([(
        "custom".to_string(),
        ModelProviderInfo {
            name: "Custom".to_string(),
            env_key: Some("CUSTOM_API_KEY".to_string()),
            ..ModelProviderInfo::default()
        },
    )]))
}

fn loaded_default_eligibility() -> ProviderEligibilitySnapshot {
    ProviderEligibilitySnapshot::Loaded(ProviderEligibility::default())
}
