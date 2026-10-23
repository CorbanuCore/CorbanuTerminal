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
    config.model_providers =
        codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None);
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
    config.model_providers =
        codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None);
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

/// #625: under Aggressive the credential broker takes the provider key out of
/// the environment; the provider stays configured (it serves the session).
#[test]
fn a_key_held_by_the_credential_broker_still_configures_its_provider() {
    let missing = || Err(std::env::VarError::NotPresent);
    assert_eq!(
        environment_metadata_from(missing(), /*held_by_broker*/ true),
        EnvironmentCredentialMetadata::Present
    );
    assert_eq!(
        environment_metadata_from(missing(), /*held_by_broker*/ false),
        EnvironmentCredentialMetadata::Missing
    );
    assert_eq!(
        environment_metadata_from(Ok(" ".to_string()), /*held_by_broker*/ true),
        EnvironmentCredentialMetadata::Invalid
    );
    assert_eq!(
        environment_metadata_from(Ok("key".to_string()), /*held_by_broker*/ false),
        EnvironmentCredentialMetadata::Present
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

#[test]
fn unselected_claude_environment_token_is_configured_not_ambiguous() {
    assert_eq!(
        claude_credential_metadata(ClaudeCodePlanStatus::EnvironmentTokenWithoutSelection),
        ClaudeCredentialMetadata::Configured {
            source: codex_provider_auth::ClaudeCredentialSource::Environment,
        }
    );
    assert_eq!(
        claude_credential_metadata(ClaudeCodePlanStatus::EnvironmentToken { available: false }),
        ClaudeCredentialMetadata::RecoveryRequired {
            reason: codex_provider_auth::ProviderRecoveryReason::UnhealthyClaudeSelection,
        }
    );
}

/// PF-84 (#417): a provider that runs on an enrolled named account is
/// configured even when it has no default key, so onboarding does not demand
/// one; an unenrolled account still counts as not configured.
#[tokio::test]
async fn enrolled_named_account_configures_its_provider_without_a_default_key() {
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_provider_id = "zai".into();
    config.model = Some("glm-5.3-flash".into());
    config
        .model_providers
        .get_mut("zai")
        .expect("built-in zai provider")
        .account = Some(NamedProviderAccount {
        provider_id: "zai".into(),
        name: "main".into(),
    });
    for (enrolled, expected) in [
        (true, ProviderConfigurationState::Configured),
        (false, ProviderConfigurationState::NotConfigured),
    ] {
        let host = ProviderStatusHost::from_config_with_account_registry(
            &config,
            ProviderAccountMetadata::default(),
            |account, kind| {
                enrolled
                    && account.provider_id == "zai"
                    && account.name == "main"
                    && kind == ProviderAccountKind::ApiKey
            },
        )
        .with_environment(|_| EnvironmentCredentialMetadata::Missing);
        assert_eq!(
            host.resolve_provider("zai").unwrap().configuration,
            expected,
            "enrolled = {enrolled}"
        );
        let policy = crate::chatwidget::provider_model_policy::ProviderModelPolicy::new(
            host,
            ProviderRuntimeAuthorizations::default(),
        );
        assert_eq!(
            matches!(
                policy.current("zai", "glm-5.3-flash"),
                codex_provider_auth::CurrentSelectionDecision::Preserve(_)
            ),
            enrolled
        );
    }
}

/// PF-84 (#571): OpenAI on a named sign-in account selected for the session
/// (`--account openai:work` from a Z.AI session) is configured by that
/// account even without a default OpenAI login, so `/model` offers it; an
/// unenrolled account is not configured, and without a selection the
/// default login decides as before.
#[tokio::test]
async fn openai_on_a_named_sign_in_account_is_configured_without_a_default_login() {
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_provider_id = "zai".into();
    config.model = Some("glm-5.3-flash".into());
    let host = |config: &crate::legacy_core::config::Config, enrolled: bool| {
        ProviderStatusHost::from_config_with_account_registry(
            config,
            ProviderAccountMetadata::default(),
            |account, kind| {
                enrolled
                    && account.provider_id == "openai"
                    && account.name == "work"
                    && kind == ProviderAccountKind::ChatgptAuth
            },
        )
    };
    assert_eq!(
        host(&config, /*enrolled*/ true)
            .resolve_provider("openai")
            .unwrap()
            .configuration,
        ProviderConfigurationState::NotConfigured
    );
    config
        .model_providers
        .get_mut("openai")
        .expect("built-in openai provider")
        .account = Some(NamedProviderAccount {
        provider_id: "openai".into(),
        name: "work".into(),
    });
    for (enrolled, expected) in [
        (true, ProviderConfigurationState::Configured),
        (false, ProviderConfigurationState::NotConfigured),
    ] {
        assert_eq!(
            host(&config, enrolled)
                .resolve_provider("openai")
                .unwrap()
                .configuration,
            expected,
            "enrolled = {enrolled}"
        );
    }
    // #609: from another provider's session, the default login's state says
    // nothing about an unenrolled named account, whatever kind it is.
    for default_login in [
        codex_login::OpenAiAuthMetadata::ApiKey,
        codex_login::OpenAiAuthMetadata::Account,
    ] {
        let host = ProviderStatusHost::from_config_with_account_registry(
            &config,
            ProviderAccountMetadata {
                openai: default_login,
                ..Default::default()
            },
            |_, _| false,
        );
        assert_eq!(
            host.resolve_provider("openai").unwrap().configuration,
            ProviderConfigurationState::NotConfigured,
            "{default_login:?}"
        );
    }
    // The session's own account, vetted at startup, stays configured even
    // when this home's vault does not hold it (a remote app server) and a
    // refresh re-reads a missing default login.
    config.model_provider_id = "openai".into();
    config.model_provider = config.model_providers["openai"].clone();
    for default_login in [
        codex_login::OpenAiAuthMetadata::Missing,
        codex_login::OpenAiAuthMetadata::ApiKey,
        codex_login::OpenAiAuthMetadata::Account,
    ] {
        let host = ProviderStatusHost::from_config_with_account_registry(
            &config,
            ProviderAccountMetadata::default(),
            |_, _| false,
        );
        host.update_account_metadata(ProviderAccountMetadata {
            openai: default_login,
            ..Default::default()
        });
        assert_eq!(
            host.resolve_provider("openai").unwrap().configuration,
            ProviderConfigurationState::Configured,
            "{default_login:?}"
        );
    }
}

/// #665: `zai` and `zai-anthropic` share one catalog entry, but PF-84 accounts
/// are per connection. With one on a named account, each connection is judged
/// on its own credential and the entry is set up when any connection has one;
/// without any account nothing changes.
#[tokio::test]
async fn each_connection_on_a_mixed_account_entry_needs_its_own_credential() {
    use std::collections::BTreeSet;

    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.cli_auth_credentials_store_mode = codex_login::AuthCredentialsStoreMode::File;
    config.model_provider_id = "zai".into();
    let host = |config: &Config, enrolled: bool| {
        ProviderStatusHost::from_config_with_account_registry(
            config,
            ProviderAccountMetadata::default(),
            |account, kind| {
                enrolled
                    && account.provider_id == "zai"
                    && account.name == "work"
                    && kind == ProviderAccountKind::ApiKey
            },
        )
        .with_environment(|_| EnvironmentCredentialMetadata::Missing)
    };
    let check = |config: &Config, enrolled: bool, missing: &[&str], entry| {
        let host = host(config, enrolled);
        assert_eq!(
            host.connections_without_credential(),
            missing
                .iter()
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>(),
            "enrolled = {enrolled}"
        );
        assert_eq!(
            host.resolve_provider("zai").unwrap().configuration,
            entry,
            "enrolled = {enrolled}"
        );
    };
    use ProviderConfigurationState::Configured;
    use ProviderConfigurationState::NotConfigured;

    check(&config, /*enrolled*/ false, &[], NotConfigured);
    config
        .model_providers
        .get_mut("zai")
        .expect("built-in zai provider")
        .account = Some(NamedProviderAccount {
        provider_id: "zai".into(),
        name: "work".into(),
    });
    check(
        &config,
        /*enrolled*/ true,
        &["zai-anthropic"],
        Configured,
    );
    check(
        &config,
        /*enrolled*/ false,
        &["zai", "zai-anthropic"],
        NotConfigured,
    );

    codex_login::login_with_provider_api_key(
        home.path(),
        "ZAI_API_KEY",
        "synthetic-default-key",
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    )
    .unwrap();
    check(&config, /*enrolled*/ true, &[], Configured);
    // The default key alone sets the entry up for `zai-anthropic`.
    check(&config, /*enrolled*/ false, &["zai"], Configured);
    // #609: `/providers` names the account only when it set the entry up.
    assert_eq!(
        host(&config, /*enrolled*/ true).status_accounts(),
        std::collections::BTreeMap::from([("zai".to_string(), "work".to_string())])
    );
    assert_eq!(
        host(&config, /*enrolled*/ false).status_accounts(),
        std::collections::BTreeMap::new()
    );
}

/// #665: a key in the environment (or held by the credential broker, which
/// reports it as present) is the default key of a connection without an account.
#[tokio::test]
async fn an_environment_default_key_serves_the_connection_without_an_account() {
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_provider_id = "zai-anthropic".into();
    config
        .model_providers
        .get_mut("zai")
        .expect("built-in zai provider")
        .account = Some(NamedProviderAccount {
        provider_id: "zai".into(),
        name: "work".into(),
    });
    let host = ProviderStatusHost::from_config_with_account_registry(
        &config,
        ProviderAccountMetadata::default(),
        |_, _| false,
    )
    .with_environment(|env_key| {
        if env_key == "ZAI_API_KEY" {
            EnvironmentCredentialMetadata::Present
        } else {
            EnvironmentCredentialMetadata::Missing
        }
    });
    assert_eq!(
        host.connections_without_credential(),
        std::collections::BTreeSet::from(["zai".to_string()])
    );
    assert_eq!(
        host.resolve_provider("zai").unwrap().configuration,
        ProviderConfigurationState::Configured
    );
    let policy = crate::chatwidget::provider_model_policy::ProviderModelPolicy::new(
        host,
        ProviderRuntimeAuthorizations::default(),
    );
    assert!(policy.provider_is_selectable("zai-anthropic", "glm-5.3-flash"));
    assert!(!policy.provider_is_selectable("zai", "glm-5.3-flash"));
    // A session on the connection that cannot sign in is asked to recover.
    assert!(matches!(
        policy.current("zai", "glm-5.3-flash"),
        codex_provider_auth::CurrentSelectionDecision::RequireExplicitRecovery { .. }
    ));
    assert!(matches!(
        policy.current("zai-anthropic", "glm-5.3-flash"),
        codex_provider_auth::CurrentSelectionDecision::Preserve(_)
    ));
}

/// #609 item 4: OpenAI on a named account reports what the account holds,
/// from the account registry's metadata (the sign-in is not read on a
/// refresh): an OpenAI API key as an API key, a ChatGPT sign-in as an
/// account, and either one `forced_login_method` rules out as not
/// configured. An account saved by an earlier build reads as a ChatGPT
/// sign-in, as before, until its next use records what it holds.
#[tokio::test]
async fn a_named_openai_account_reports_what_it_holds() {
    use codex_protocol::config_types::ForcedLoginMethod;
    use codex_provider_auth::ProviderCredentialSource;
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    // A Z.AI session, which implies an API-key-only onboarding default that
    // says nothing about a named account; `/model` offers OpenAI on `work`.
    config.model_provider_id = "zai".into();
    config.forced_login_method = Some(ForcedLoginMethod::Api);
    config
        .model_providers
        .get_mut("openai")
        .expect("built-in openai provider")
        .account = Some(NamedProviderAccount {
        provider_id: "openai".into(),
        name: "work".into(),
    });
    let account = codex_login::NamedChatgptAccount {
        provider_id: "openai".to_string(),
        name: "work".to_string(),
    };
    let status = |config: &Config, forced: Option<ForcedLoginMethod>| {
        let mut config = config.clone();
        config.configured_forced_login_method = forced;
        let status = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default())
            .resolve_provider("openai")
            .unwrap();
        let source = status.methods.iter().find_map(|method| match method.state {
            codex_provider_auth::ProviderMethodState::Configured { source, .. } => Some(source),
            _ => None,
        });
        (status.configuration, source)
    };
    let detail = |config: &Config, forced: Option<ForcedLoginMethod>| {
        let mut config = config.clone();
        config.configured_forced_login_method = forced;
        ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default())
            .status_account_details()
            .get("openai")
            .cloned()
    };
    let configured = |source| (ProviderConfigurationState::Configured, Some(source));
    let ruled_out = (ProviderConfigurationState::NotConfigured, None);

    codex_login::login_named_account_with_api_key(home.path(), &account, "sk-canary-609")
        .expect("api key");
    assert_eq!(
        status(&config, /*forced*/ None),
        configured(ProviderCredentialSource::OpenAiApiKey)
    );
    assert_eq!(
        status(&config, Some(ForcedLoginMethod::Api)),
        configured(ProviderCredentialSource::OpenAiApiKey)
    );
    assert_eq!(status(&config, Some(ForcedLoginMethod::Chatgpt)), ruled_out);
    assert_eq!(
        detail(&config, /*forced*/ None).as_deref(),
        Some("an OpenAI API key")
    );
    assert_eq!(
        detail(&config, Some(ForcedLoginMethod::Chatgpt)).as_deref(),
        Some("an OpenAI API key; this configuration requires a ChatGPT sign-in")
    );

    let jwt = "e30.eyJlbWFpbCI6ImFubkBleGFtcGxlLmNvbSIsImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJjaGF0Z3B0X3BsYW5fdHlwZSI6InBsdXMiLCJjaGF0Z3B0X3VzZXJfaWQiOiJ1LTEifX0.c2ln";
    let sign_in = codex_login::AuthDotJson {
        auth_mode: Some(codex_protocol::auth::AuthMode::Chatgpt),
        openai_api_key: None,
        tokens: Some(codex_login::token_data::TokenData {
            id_token: codex_login::token_data::parse_chatgpt_jwt_claims(jwt).expect("claims"),
            access_token: "access-canary-0609".to_string(),
            refresh_token: "refresh-canary-0609".to_string(),
            account_id: Some("ws-1".to_string()),
        }),
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    };
    codex_login::save_named_account_auth(
        home.path(),
        &account,
        &sign_in,
        /*must_be_new*/ false,
    )
    .expect("sign in");
    assert_eq!(
        status(&config, /*forced*/ None),
        configured(ProviderCredentialSource::OpenAiAccount)
    );
    assert_eq!(status(&config, Some(ForcedLoginMethod::Api)), ruled_out);
    assert_eq!(
        detail(&config, Some(ForcedLoginMethod::Api)).as_deref(),
        Some("a ChatGPT sign-in; this configuration requires an API key")
    );

    // Saved by an earlier build (nothing recorded): an account, as before,
    // until a use reads the sign-in and records what it holds.
    let api_key = serde_json::to_string(&codex_login::AuthDotJson {
        auth_mode: Some(codex_protocol::auth::AuthMode::ApiKey),
        openai_api_key: Some("sk-canary-old".to_string()),
        tokens: None,
        last_refresh: None,
        agent_identity: None,
        personal_access_token: None,
        bedrock_api_key: None,
    })
    .unwrap();
    codex_vault::Vault::new(home.path().to_path_buf())
        .write_provider_account(
            "openai",
            &codex_vault::ProviderAccountName::parse("work").unwrap(),
            ProviderAccountKind::ChatgptAuth,
            &api_key,
        )
        .expect("earlier build");
    assert_eq!(
        status(&config, /*forced*/ None),
        configured(ProviderCredentialSource::OpenAiAccount)
    );
    assert_eq!(detail(&config, /*forced*/ None), None);
    assert!(
        codex_login::named_account_summary(home.path(), &account)
            .expect("use")
            .is_some()
    );
    assert_eq!(
        status(&config, /*forced*/ None),
        configured(ProviderCredentialSource::OpenAiApiKey)
    );
}
