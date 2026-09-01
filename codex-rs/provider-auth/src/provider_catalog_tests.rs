use super::*;
use codex_model_provider_info::ANTHROPIC_API_KEY_ENV_VAR;
use codex_model_provider_info::CORBANU_PLAN_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::CORBANU_TERMINAL_PLAN_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_model_provider_info::ZAI_API_KEY_ENV_VAR;
use codex_model_provider_info::built_in_model_providers;
use pretty_assertions::assert_eq;

#[test]
fn built_ins_expose_multi_method_openai_and_branded_corbanu_identity() {
    let catalog = ProviderCatalog::from_runtime_providers(&built_in_model_providers(
        /*openai_base_url*/ None,
    ));

    assert_eq!(
        (
            catalog.get(OPENAI_PROVIDER_ID),
            catalog.get(CORBANU_PLAN_PROVIDER_ID),
        ),
        (
            Some(&ProviderCatalogEntry {
                id: ProviderCatalogId(OPENAI_PROVIDER_ID.to_string()),
                display_name: "OpenAI".to_string(),
                runtime_provider_ids: vec![ProviderRuntimeId(OPENAI_PROVIDER_ID.to_string())],
                setup_capabilities: ProviderSetupCapabilities {
                    primary: ProviderSetupCapability::OpenAiAccount,
                    alternatives: vec![ProviderSetupCapability::ApiKey {
                        storage: ApiKeyStorage::OpenAiAuth,
                    }],
                },
            }),
            Some(&ProviderCatalogEntry {
                id: ProviderCatalogId(CORBANU_PLAN_PROVIDER_ID.to_string()),
                display_name: "Corbanu Plan".to_string(),
                runtime_provider_ids: vec![
                    ProviderRuntimeId(PFTERMINAL_PLAN_PROVIDER_ID.to_string()),
                    ProviderRuntimeId(PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID.to_string()),
                ],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::CorbanuPlan,
                ),
            }),
        )
    );
    assert_eq!(catalog.get(PFTERMINAL_PLAN_PROVIDER_ID), None);
}

#[test]
fn built_ins_freeze_claude_local_and_aws_status_shapes() {
    let catalog = ProviderCatalog::from_runtime_providers(&built_in_model_providers(
        /*openai_base_url*/ None,
    ));

    assert_eq!(
        [
            CLAUDE_PLAN_PROVIDER_ID,
            OLLAMA_OSS_PROVIDER_ID,
            LMSTUDIO_OSS_PROVIDER_ID,
            AMAZON_BEDROCK_PROVIDER_ID,
        ]
        .map(|provider_id| catalog.get(provider_id).cloned()),
        [
            Some(ProviderCatalogEntry {
                id: ProviderCatalogId(CLAUDE_PLAN_PROVIDER_ID.to_string()),
                display_name: "Claude Account".to_string(),
                runtime_provider_ids: vec![ProviderRuntimeId(CLAUDE_PLAN_PROVIDER_ID.to_string(),)],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::ClaudeAccount,
                ),
            }),
            Some(ProviderCatalogEntry {
                id: ProviderCatalogId(OLLAMA_OSS_PROVIDER_ID.to_string()),
                display_name: "Ollama".to_string(),
                runtime_provider_ids: vec![ProviderRuntimeId(OLLAMA_OSS_PROVIDER_ID.to_string(),)],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::Local {
                        provider: LocalProvider::Ollama,
                    },
                ),
            }),
            Some(ProviderCatalogEntry {
                id: ProviderCatalogId(LMSTUDIO_OSS_PROVIDER_ID.to_string()),
                display_name: "LM Studio".to_string(),
                runtime_provider_ids: vec![
                    ProviderRuntimeId(LMSTUDIO_OSS_PROVIDER_ID.to_string(),)
                ],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::Local {
                        provider: LocalProvider::LmStudio,
                    },
                ),
            }),
            Some(ProviderCatalogEntry {
                id: ProviderCatalogId(AMAZON_BEDROCK_PROVIDER_ID.to_string()),
                display_name: "Amazon Bedrock".to_string(),
                runtime_provider_ids: vec![ProviderRuntimeId(
                    AMAZON_BEDROCK_PROVIDER_ID.to_string(),
                )],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::StatusOnly {
                        reason: StatusOnlyReason::AwsCredentialChain,
                    },
                ),
            }),
        ]
    );
}

#[test]
fn built_in_wire_siblings_share_one_api_key_setup_identity() {
    let catalog = ProviderCatalog::from_runtime_providers(&built_in_model_providers(
        /*openai_base_url*/ None,
    ));

    assert_eq!(
        catalog.get(ZAI_PROVIDER_ID),
        Some(&ProviderCatalogEntry {
            id: ProviderCatalogId(ZAI_PROVIDER_ID.to_string()),
            display_name: "Z.AI".to_string(),
            runtime_provider_ids: vec![
                ProviderRuntimeId(ZAI_PROVIDER_ID.to_string()),
                ProviderRuntimeId(ZAI_ANTHROPIC_PROVIDER_ID.to_string()),
            ],
            setup_capabilities: ProviderSetupCapabilities::one(ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::EnvironmentVariable {
                    env_key: ZAI_API_KEY_ENV_VAR.to_string(),
                },
            },),
        })
    );
}

#[test]
fn custom_env_key_providers_are_adjacent_deduplicated_and_deterministic() {
    let alpha = custom_api_key_provider("Alpha", "SHARED_API_KEY");
    let duplicate = custom_api_key_provider("Duplicate", "SHARED_API_KEY");
    let beta = custom_api_key_provider("Beta", "BETA_API_KEY");

    let first = HashMap::from([
        ("duplicate".to_string(), duplicate.clone()),
        ("beta".to_string(), beta.clone()),
        ("alpha".to_string(), alpha.clone()),
    ]);
    let second = HashMap::from([
        ("alpha".to_string(), alpha),
        ("duplicate".to_string(), duplicate),
        ("beta".to_string(), beta),
    ]);

    let expected = ProviderCatalog {
        entries: vec![
            ProviderCatalogEntry {
                id: ProviderCatalogId("alpha".to_string()),
                display_name: "Alpha".to_string(),
                runtime_provider_ids: vec![
                    ProviderRuntimeId("alpha".to_string()),
                    ProviderRuntimeId("duplicate".to_string()),
                ],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::ApiKey {
                        storage: ApiKeyStorage::EnvironmentVariable {
                            env_key: "SHARED_API_KEY".to_string(),
                        },
                    },
                ),
            },
            ProviderCatalogEntry {
                id: ProviderCatalogId("beta".to_string()),
                display_name: "Beta".to_string(),
                runtime_provider_ids: vec![ProviderRuntimeId("beta".to_string())],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::ApiKey {
                        storage: ApiKeyStorage::EnvironmentVariable {
                            env_key: "BETA_API_KEY".to_string(),
                        },
                    },
                ),
            },
        ],
    };

    assert_eq!(ProviderCatalog::from_runtime_providers(&first), expected);
    assert_eq!(ProviderCatalog::from_runtime_providers(&second), expected);
}

#[test]
fn corbanu_runtime_aliases_deduplicate_without_leaking_legacy_catalog_identity() {
    let plan = ModelProviderInfo::create_pfterminal_plan_provider();
    let plan_anthropic = ModelProviderInfo::create_pfterminal_plan_anthropic_provider();
    let providers = HashMap::from([
        (CORBANU_PLAN_PROVIDER_ID.to_string(), plan.clone()),
        (CORBANU_TERMINAL_PLAN_PROVIDER_ID.to_string(), plan.clone()),
        (PFTERMINAL_PLAN_PROVIDER_ID.to_string(), plan),
        (
            CORBANU_PLAN_ANTHROPIC_PROVIDER_ID.to_string(),
            plan_anthropic.clone(),
        ),
        (
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID.to_string(),
            plan_anthropic,
        ),
    ]);

    assert_eq!(
        ProviderCatalog::from_runtime_providers(&providers),
        ProviderCatalog {
            entries: vec![ProviderCatalogEntry {
                id: ProviderCatalogId(CORBANU_PLAN_PROVIDER_ID.to_string()),
                display_name: "Corbanu Plan".to_string(),
                runtime_provider_ids: vec![
                    ProviderRuntimeId(CORBANU_PLAN_PROVIDER_ID.to_string()),
                    ProviderRuntimeId(CORBANU_PLAN_ANTHROPIC_PROVIDER_ID.to_string()),
                    ProviderRuntimeId(CORBANU_TERMINAL_PLAN_PROVIDER_ID.to_string()),
                    ProviderRuntimeId(PFTERMINAL_PLAN_PROVIDER_ID.to_string()),
                    ProviderRuntimeId(PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID.to_string()),
                ],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::CorbanuPlan,
                ),
            }],
        }
    );
}

#[test]
fn unsupported_custom_command_auth_is_status_only_while_adjacent_api_key_is_interactive() {
    let mut command_provider = ModelProviderInfo::create_claude_plan_provider();
    command_provider.name = "Custom Command".to_string();
    let providers = HashMap::from([
        ("custom-command".to_string(), command_provider),
        (
            "custom-key".to_string(),
            custom_api_key_provider("Custom Key", ANTHROPIC_API_KEY_ENV_VAR),
        ),
    ]);

    assert_eq!(
        ProviderCatalog::from_runtime_providers(&providers),
        ProviderCatalog {
            entries: vec![
                ProviderCatalogEntry {
                    id: ProviderCatalogId("custom-command".to_string()),
                    display_name: "Custom Command".to_string(),
                    runtime_provider_ids: vec![ProviderRuntimeId("custom-command".to_string(),)],
                    setup_capabilities: ProviderSetupCapabilities::one(
                        ProviderSetupCapability::CommandAuth {
                            setup: CommandAuthSetup::StatusOnly,
                        },
                    ),
                },
                ProviderCatalogEntry {
                    id: ProviderCatalogId("custom-key".to_string()),
                    display_name: "Custom Key".to_string(),
                    runtime_provider_ids: vec![ProviderRuntimeId("custom-key".to_string())],
                    setup_capabilities: ProviderSetupCapabilities::one(
                        ProviderSetupCapability::ApiKey {
                            storage: ApiKeyStorage::EnvironmentVariable {
                                env_key: ANTHROPIC_API_KEY_ENV_VAR.to_string(),
                            },
                        },
                    ),
                },
            ],
        }
    );
}

#[test]
fn blank_runtime_identity_is_ignored_without_hiding_adjacent_provider() {
    let providers = HashMap::from([
        (
            "   ".to_string(),
            custom_api_key_provider("Invalid", PFTERMINAL_PLAN_API_KEY_ENV_VAR),
        ),
        (
            "valid".to_string(),
            custom_api_key_provider("Valid", "VALID_API_KEY"),
        ),
    ]);

    assert_eq!(
        ProviderCatalog::from_runtime_providers(&providers).into_entries(),
        vec![ProviderCatalogEntry {
            id: ProviderCatalogId("valid".to_string()),
            display_name: "Valid".to_string(),
            runtime_provider_ids: vec![ProviderRuntimeId("valid".to_string())],
            setup_capabilities: ProviderSetupCapabilities::one(ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::EnvironmentVariable {
                    env_key: "VALID_API_KEY".to_string(),
                },
            },),
        }]
    );
}

fn custom_api_key_provider(name: &str, env_key: &str) -> ModelProviderInfo {
    ModelProviderInfo {
        name: name.to_string(),
        env_key: Some(env_key.to_string()),
        ..ModelProviderInfo::default()
    }
}
