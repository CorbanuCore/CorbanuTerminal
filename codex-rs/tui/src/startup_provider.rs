use codex_login::OpenAiAuthMetadata;
use codex_provider_auth::CurrentSelectionDecision;
use codex_provider_auth::ProviderRuntimeAuthorizations;
use codex_provider_auth::ProviderUseDecision;

use crate::chatwidget::provider_model_policy::ProviderModelPolicy;
use crate::legacy_core::config::Config;
use crate::provider_status_host::ProviderAccountMetadata;
use crate::provider_status_host::ProviderStatusHost;

pub(crate) struct StartupProviderResolution {
    pub(crate) policy: ProviderModelPolicy,
    pub(crate) current: CurrentSelectionDecision,
    pub(crate) has_usable_provider: bool,
    context: StartupProviderContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartupProviderContext {
    codex_home: std::path::PathBuf,
    cwd: std::path::PathBuf,
    model: Option<String>,
    model_provider_id: String,
}

impl StartupProviderContext {
    fn from_config(config: &Config) -> Self {
        Self {
            codex_home: config.codex_home.to_path_buf(),
            cwd: config.cwd.to_path_buf(),
            model: config.model.clone(),
            model_provider_id: config.model_provider_id.clone(),
        }
    }
}

pub(crate) async fn resolve(
    config: &Config,
    openai: Option<OpenAiAuthMetadata>,
) -> StartupProviderResolution {
    let mut account = ProviderAccountMetadata::discover(config).await;
    if config.model_provider_id == codex_model_provider_info::OPENAI_PROVIDER_ID
        && let Some(openai) = openai
    {
        account.openai = openai;
    }
    let mut host = ProviderStatusHost::from_config(config, account);
    // Provider auth commands are executable credential resolvers. Running every configured
    // command during startup creates unrelated side effects and can add a full command timeout
    // before onboarding. The model-provider and native-spawn boundaries validate the selected
    // provider lazily when it is actually used.
    let authorizations = ProviderRuntimeAuthorizations::default();
    host.set_runtime_authorizations(authorizations.clone());
    let policy = ProviderModelPolicy::new(host, authorizations);
    let model = config.model.clone().unwrap_or_default();
    let current = policy.current(&config.model_provider_id, &model);
    let has_usable_provider = config.model_providers.iter().any(|(runtime_id, _)| {
        matches!(
            policy.assess(
                runtime_id,
                &codex_model_provider_info::resolve_model_for_provider(
                    config.model.clone(),
                    runtime_id,
                )
                .unwrap_or_default(),
                codex_provider_auth::ProviderUseContext::AutomaticDefault,
            ),
            ProviderUseDecision::Ready(_)
        )
    });
    StartupProviderResolution {
        policy,
        current,
        has_usable_provider,
        context: StartupProviderContext::from_config(config),
    }
}

pub(crate) async fn resolve_for_app(
    config: &Config,
    cached: Option<StartupProviderResolution>,
) -> StartupProviderResolution {
    match cached {
        Some(cached) if cached.context == StartupProviderContext::from_config(config) => cached,
        _ => resolve(config, /*openai*/ None).await,
    }
}

pub(crate) fn openai_metadata(login_status: crate::LoginStatus) -> OpenAiAuthMetadata {
    match login_status {
        crate::LoginStatus::AuthMode(crate::AuthMode::Chatgpt) => OpenAiAuthMetadata::Account,
        crate::LoginStatus::AuthMode(crate::AuthMode::ApiKey) => OpenAiAuthMetadata::ApiKey,
        crate::LoginStatus::AuthMode(_) => OpenAiAuthMetadata::ExternallyManaged,
        crate::LoginStatus::NotAuthenticated => OpenAiAuthMetadata::Missing,
    }
}

pub(crate) fn should_show_provider_onboarding(
    show_trust_screen: bool,
    has_usable_provider: bool,
    forced_login_method: Option<crate::ForcedLoginMethod>,
    login_status: crate::LoginStatus,
) -> bool {
    show_trust_screen
        || !has_usable_provider
        || matches!(forced_login_method, Some(crate::ForcedLoginMethod::Chatgpt))
            && login_status == crate::LoginStatus::NotAuthenticated
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_model_provider_info::ModelProviderInfo;

    #[test]
    fn usable_provider_skips_onboarding_but_trust_and_forced_login_do_not() {
        assert!(!should_show_provider_onboarding(
            false,
            true,
            None,
            crate::LoginStatus::NotAuthenticated,
        ));
        assert!(should_show_provider_onboarding(
            true,
            true,
            None,
            crate::LoginStatus::NotAuthenticated,
        ));
        assert!(should_show_provider_onboarding(
            false,
            true,
            Some(crate::ForcedLoginMethod::Chatgpt),
            crate::LoginStatus::NotAuthenticated,
        ));
    }

    #[tokio::test]
    async fn app_reuses_unchanged_startup_resolution_with_exact_current() {
        let home = tempfile::tempdir().unwrap();
        let config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        let cached = resolve(&config, None).await;
        let expected = cached.current.clone();

        let reused = resolve_for_app(&config, Some(cached)).await;

        assert_eq!(reused.current, expected);
    }

    #[tokio::test]
    async fn app_refreshes_resolution_after_provider_context_changes() {
        let home = tempfile::tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        let cached = resolve(&config, None).await;
        config.model_provider_id = "provider-added-during-onboarding".to_string();

        let refreshed = resolve_for_app(&config, Some(cached)).await;

        assert!(matches!(
            refreshed.current,
            CurrentSelectionDecision::RequireExplicitRecovery {
                requested_runtime_provider_id,
                ..
            } if requested_runtime_provider_id == "provider-added-during-onboarding"
        ));
    }

    #[tokio::test]
    async fn login_status_metadata_is_scoped_to_the_exact_openai_runtime() {
        let home = tempfile::tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        config.cli_auth_credentials_store_mode = codex_login::AuthCredentialsStoreMode::File;
        let custom = ModelProviderInfo {
            name: "Custom Current".into(),
            env_key: Some("PF56_CUSTOM_CURRENT_KEY".into()),
            ..Default::default()
        };
        config.model_provider_id = "custom-current".into();
        config.model_provider = custom.clone();
        config
            .model_providers
            .insert("custom-current".into(), custom);

        let custom_current = resolve(&config, Some(OpenAiAuthMetadata::ApiKey)).await;
        let custom_openai = custom_current
            .policy
            .host()
            .resolve_provider(codex_model_provider_info::OPENAI_PROVIDER_ID)
            .unwrap();
        assert_eq!(
            custom_openai.configuration,
            codex_provider_auth::ProviderConfigurationState::NotConfigured
        );

        config.model_provider_id = codex_model_provider_info::OPENAI_PROVIDER_ID.into();
        config.model_provider = config
            .model_providers
            .get(codex_model_provider_info::OPENAI_PROVIDER_ID)
            .unwrap()
            .clone();
        let openai_current = resolve(&config, Some(OpenAiAuthMetadata::ApiKey)).await;
        assert_eq!(
            openai_current
                .policy
                .host()
                .resolve_provider(codex_model_provider_info::OPENAI_PROVIDER_ID)
                .unwrap()
                .configuration,
            codex_provider_auth::ProviderConfigurationState::Configured
        );
    }

    #[tokio::test]
    async fn command_authorization_is_lazy_but_current_and_selectable() {
        let home = tempfile::tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        let provider = ModelProviderInfo {
            name: "Command Provider".into(),
            auth: Some(
                serde_json::from_value(serde_json::json!({
                    "command": "sh",
                    "args": ["-c", "touch command-ran; printf pf56-command-token-canary"],
                    "cwd": home.path(),
                }))
                .unwrap(),
            ),
            ..Default::default()
        };
        config.model_provider_id = "command-provider".into();
        config.model_provider = provider.clone();
        config.model = Some("command-model".into());
        config
            .model_providers
            .insert("command-provider".into(), provider);

        let resolution = resolve(&config, None).await;

        assert!(resolution.has_usable_provider);
        assert!(
            resolution
                .policy
                .provider_is_selectable("command-provider", "command-model")
        );
        assert!(matches!(
            resolution.current,
            CurrentSelectionDecision::Preserve(ref selection)
                if selection.runtime_provider_id.as_str() == "command-provider"
                    && selection.model == "command-model"
        ));
        assert!(
            !home.path().join("command-ran").exists(),
            "startup must not execute a provider auth command"
        );
        assert!(!format!("{:?}", resolution.policy).contains("pf56-command-token-canary"));
    }

    #[tokio::test]
    async fn local_only_runtime_skips_provider_onboarding() {
        let home = tempfile::tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        config.model_provider_id = codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID.into();
        config.model_provider = codex_model_provider_info::create_oss_provider_with_base_url(
            "http://localhost:11434/v1",
            codex_model_provider_info::WireApi::Responses,
        );
        config.model = Some("qwen3".into());
        config.model_providers = std::collections::HashMap::from([(
            config.model_provider_id.clone(),
            config.model_provider.clone(),
        )]);

        let resolution = resolve(&config, None).await;

        assert!(resolution.has_usable_provider);
        assert!(matches!(
            resolution.current,
            CurrentSelectionDecision::Preserve(ref selection)
                if selection.runtime_provider_id.as_str()
                    == codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID
        ));
    }

    #[tokio::test]
    async fn credential_free_custom_runtime_skips_provider_onboarding() {
        let home = tempfile::tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        config.model_provider_id = "no-auth-custom".into();
        config.model_provider = ModelProviderInfo {
            name: "No Auth Custom".into(),
            ..Default::default()
        };
        config.model = Some("custom-model".into());
        config.model_providers = std::collections::HashMap::from([(
            config.model_provider_id.clone(),
            config.model_provider.clone(),
        )]);

        let resolution = resolve(&config, None).await;

        assert!(resolution.has_usable_provider);
        assert!(matches!(
            resolution.current,
            CurrentSelectionDecision::Preserve(ref selection)
                if selection.runtime_provider_id.as_str() == "no-auth-custom"
        ));
    }
}
