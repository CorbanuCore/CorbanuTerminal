use codex_model_provider_info::ModelProviderInfo;
use codex_provider_auth::ProviderAvailabilityState;
use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderRuntimeAuthorization;
use codex_provider_auth::ProviderRuntimeAuthorizations;
use tempfile::tempdir;

use super::*;

#[test]
fn every_corbanu_environment_alias_is_ready_and_precedes_stored_account_status() {
    let home = tempdir().unwrap();
    for alias in CORBANU_API_KEY_ENV_VARS {
        for supplied in [
            CorbanuPlanMetadata::NotConfigured,
            CorbanuPlanMetadata::Unavailable,
            CorbanuPlanMetadata::Configured {
                source: CorbanuCredentialSource::Managed,
                availability: ConfiguredAvailability::Ready,
            },
        ] {
            let metadata = corbanu_metadata_with_environment(home.path(), supplied, |name| {
                if name == alias {
                    EnvironmentCredentialMetadata::Present
                } else {
                    EnvironmentCredentialMetadata::Invalid
                }
            });
            assert_eq!(
                metadata,
                CorbanuPlanMetadata::Configured {
                    source: CorbanuCredentialSource::Environment,
                    availability: ConfiguredAvailability::Ready,
                }
            );
        }
    }
}

#[test]
fn blank_or_missing_corbanu_aliases_allow_the_managed_credential_fallback() {
    let home = tempdir().unwrap();
    let managed = CorbanuPlanMetadata::Configured {
        source: CorbanuCredentialSource::Managed,
        availability: ConfiguredAvailability::Ready,
    };
    for environment in [
        EnvironmentCredentialMetadata::Missing,
        EnvironmentCredentialMetadata::Invalid,
    ] {
        assert_eq!(
            corbanu_metadata_with_environment(home.path(), managed, |_| environment),
            managed
        );
        assert_eq!(
            corbanu_metadata_with_environment(
                home.path(),
                CorbanuPlanMetadata::NotConfigured,
                |_| environment
            ),
            CorbanuPlanMetadata::NotConfigured
        );
    }
}

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

#[tokio::test]
async fn saved_managed_custom_key_resolves_configured_ready_without_secret_leakage() {
    let home = tempdir().unwrap();
    let env_key = "PF54_POST_SAVE_MANAGED_TEST_KEY";
    let config = config(
        home.path(),
        "managed",
        ModelProviderInfo {
            name: "Managed".into(),
            env_key: Some(env_key.into()),
            ..Default::default()
        },
    )
    .await;
    let canary = "pf54-post-save-secret-canary";
    codex_login::login_with_provider_api_key(
        home.path(),
        env_key,
        canary,
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    )
    .unwrap();

    let host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    let status = host.resolve_provider("managed").unwrap();
    assert_eq!(status.configuration, ProviderConfigurationState::Configured);
    assert_eq!(
        status.availability,
        codex_provider_auth::ProviderAvailabilityState::Ready
    );
    assert!(!format!("{host:?} {status:?}").contains(canary));
}

#[tokio::test]
async fn local_and_no_auth_custom_providers_are_ready_without_enrollment() {
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_providers = codex_model_provider_info::built_in_model_providers(None);
    config.model_providers.insert(
        "no-auth-custom".into(),
        ModelProviderInfo {
            name: "No Auth Custom".into(),
            ..Default::default()
        },
    );
    let host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());

    let local = host
        .resolve_provider(codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID)
        .expect("Ollama status");
    assert_eq!(local.configuration, ProviderConfigurationState::Configured);
    assert_eq!(local.availability, ProviderAvailabilityState::Ready);

    let no_auth = host
        .resolve_provider("no-auth-custom")
        .expect("custom status");
    assert_eq!(no_auth.availability, ProviderAvailabilityState::StatusOnly);
}

#[tokio::test]
async fn eligibility_survives_restart_without_touching_credentials() {
    let home = tempdir().unwrap();
    let provider = ModelProviderInfo {
        name: "Restart Provider".into(),
        env_key: Some("PF54_RESTART_KEY".into()),
        ..Default::default()
    };
    let config = config(home.path(), "restart", provider).await;
    let host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    host.persist_policy("restart", ProviderActivationPolicy::Inactive)
        .unwrap();

    let restarted = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    let entry = restarted.catalog().get("restart").unwrap();
    let loaded = ProviderEligibilityStore::new(home.path()).load().unwrap();
    assert_eq!(loaded.policy_for(entry), ProviderActivationPolicy::Inactive);
    assert_eq!(
        std::fs::read_dir(home.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec!["provider-eligibility.json"]
    );
}

#[tokio::test]
async fn current_selection_changes_status_without_persisting_policy() {
    let home = tempdir().unwrap();
    let provider = ModelProviderInfo {
        name: "Selectable".into(),
        env_key: Some("PF54_SELECTABLE_KEY".into()),
        ..Default::default()
    };
    let config = config(home.path(), "selectable", provider).await;
    let host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    host.set_current_runtime("another");
    assert_eq!(
        host.resolve_provider("selectable").unwrap().current,
        codex_provider_auth::ProviderCurrentState::NotCurrent
    );
    assert!(!home.path().join("provider-eligibility.json").exists());
}

#[tokio::test]
async fn authorized_command_runtime_converges_to_configured_ready() {
    let home = tempdir().unwrap();
    let config = config(home.path(), "command", command_provider(home.path())).await;
    let mut host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    let mut authorizations = ProviderRuntimeAuthorizations::default();
    authorizations.set("command", ProviderRuntimeAuthorization::Authorized);
    host.set_runtime_authorizations(authorizations);

    let status = host.resolve_provider("command").unwrap();
    assert_eq!(status.configuration, ProviderConfigurationState::Configured);
    assert_eq!(status.availability, ProviderAvailabilityState::Ready);
}

#[tokio::test]
async fn unchecked_command_is_configured_but_a_rejected_authorization_is_status_only() {
    let home = tempdir().unwrap();
    let config = config(home.path(), "command", command_provider(home.path())).await;
    for (id, state, expected) in [
        (
            "command",
            ProviderRuntimeAuthorization::Rejected,
            ProviderAvailabilityState::StatusOnly,
        ),
        (
            "other",
            ProviderRuntimeAuthorization::Authorized,
            ProviderAvailabilityState::Ready,
        ),
        (
            "command",
            ProviderRuntimeAuthorization::NotChecked,
            ProviderAvailabilityState::Ready,
        ),
    ] {
        let mut host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
        let mut authorizations = ProviderRuntimeAuthorizations::default();
        authorizations.set(id, state);
        host.set_runtime_authorizations(authorizations);
        assert_eq!(
            host.resolve_provider("command").unwrap().availability,
            expected
        );
    }
}

#[tokio::test]
async fn command_authorization_rejection_is_exact_and_debug_output_is_secret_free() {
    let home = tempdir().unwrap();
    let mut config = config(home.path(), "one", command_provider(home.path())).await;
    config
        .model_providers
        .insert("two".into(), command_provider(home.path()));
    let mut host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    let mut authorizations = ProviderRuntimeAuthorizations::default();
    authorizations.set("one", ProviderRuntimeAuthorization::Rejected);
    host.set_runtime_authorizations(authorizations);

    let statuses = host.resolve();
    assert_eq!(
        statuses.get("one").unwrap().availability,
        ProviderAvailabilityState::StatusOnly
    );
    assert_eq!(
        statuses.get("two").unwrap().availability,
        ProviderAvailabilityState::Ready
    );
    assert!(!format!("{host:?} {statuses:?}").contains("secret-canary"));
}

#[test]
fn environment_precedence_skips_managed_reads_until_environment_is_missing() {
    for environment in [
        EnvironmentCredentialMetadata::Present,
        EnvironmentCredentialMetadata::Invalid,
    ] {
        let metadata = prioritized_api_key_metadata(environment, || {
            panic!("authoritative environment metadata must bypass managed storage")
        });
        assert_eq!(metadata.environment, environment);
        assert_eq!(metadata.managed, ManagedApiKeyMetadata::Missing);
    }

    let metadata = prioritized_api_key_metadata(EnvironmentCredentialMetadata::Missing, || {
        ManagedApiKeyMetadata::Stored {
            source: codex_login::ProviderApiKeyStorageSource::EncryptedVault,
        }
    });
    assert_eq!(
        metadata.managed,
        ManagedApiKeyMetadata::Stored {
            source: codex_login::ProviderApiKeyStorageSource::EncryptedVault,
        }
    );
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

fn command_provider(path: &std::path::Path) -> ModelProviderInfo {
    ModelProviderInfo {
        name: "Command".into(),
        auth: Some(
            serde_json::from_value(serde_json::json!({
                "command": path.join("secret-canary-command"),
                "cwd": path,
            }))
            .unwrap(),
        ),
        ..Default::default()
    }
}
