use codex_login::OpenAiAuthMetadata;
use codex_provider_auth::CurrentSelectionDecision;
use codex_provider_auth::ProviderRuntimeAuthorization;
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
    if let Some(openai) = openai {
        account.openai = openai;
    }
    let mut host = ProviderStatusHost::from_config(config, account);
    let mut authorizations = ProviderRuntimeAuthorizations::default();
    for (runtime_id, provider) in &config.model_providers {
        let Some(auth) = provider.auth.as_ref() else {
            continue;
        };
        let state = if codex_login::validate_provider_auth_command(auth)
            .await
            .is_ok()
        {
            ProviderRuntimeAuthorization::Authorized
        } else {
            ProviderRuntimeAuthorization::Rejected
        };
        authorizations.set(runtime_id.clone(), state);
    }
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
}
