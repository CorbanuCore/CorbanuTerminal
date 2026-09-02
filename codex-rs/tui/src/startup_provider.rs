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
}

pub(crate) async fn resolve(
    config: &Config,
    openai: Option<OpenAiAuthMetadata>,
) -> StartupProviderResolution {
    let mut account = ProviderAccountMetadata::discover(config).await;
    if let Some(openai) = openai {
        account.openai = openai;
    }
    let host = ProviderStatusHost::from_config(config, account);
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
}
