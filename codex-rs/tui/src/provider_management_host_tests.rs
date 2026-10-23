use codex_model_provider_info::ModelProviderInfo;
use codex_provider_auth::ProviderAvailabilityState;
use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderCurrentState;
use codex_provider_auth::ProviderEligibilityState;
use codex_provider_auth::ProviderStatusSnapshot;
use tempfile::tempdir;

use super::*;

#[tokio::test]
async fn replacement_candidates_are_exact_ready_active_non_current_rows() {
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_providers =
        codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None);
    let status_host = ProviderStatusHost::from_config(
        &config,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );
    let entries = status_host.catalog().entries();
    let ready = entries
        .iter()
        .find(|entry| {
            entry.runtime_provider_ids.first().is_some_and(|runtime| {
                codex_model_provider_info::resolve_model_for_provider(
                    /*model*/ None,
                    runtime.as_str(),
                )
                .is_some()
            })
        })
        .unwrap();
    let rejected = entries.iter().find(|entry| entry.id != ready.id).unwrap();
    let statuses = vec![
        status(ready, ProviderEligibilityState::Active),
        status(rejected, ProviderEligibilityState::Inactive),
    ];

    let candidates =
        replacement_candidates_for(&status_host, &statuses, /*current_model*/ None);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].provider_id, ready.id);
    assert_eq!(
        candidates[0].runtime_provider_id,
        ready.runtime_provider_ids[0]
    );
    assert!(!candidates[0].model.trim().is_empty());
}

#[tokio::test]
async fn current_and_unready_providers_are_never_replacement_candidates() {
    let home = tempdir().unwrap();
    let provider = ModelProviderInfo {
        name: "Rejected".into(),
        env_key: Some("PF54_REJECTED_KEY".into()),
        ..Default::default()
    };
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_providers = std::collections::HashMap::from([("rejected".into(), provider)]);
    let status_host = ProviderStatusHost::from_config(
        &config,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );
    let entry = &status_host.catalog().entries()[0];
    let mut current = status(entry, ProviderEligibilityState::Active);
    current.current = ProviderCurrentState::Current;
    let mut unavailable = status(entry, ProviderEligibilityState::Active);
    unavailable.availability = ProviderAvailabilityState::Unavailable {
        reason: codex_provider_auth::ProviderUnavailableReason::NotConfigured,
    };
    assert!(
        replacement_candidates_for(
            &status_host,
            &[current, unavailable],
            /*current_model*/ None
        )
        .is_empty()
    );
}

/// #665: Z.AI replaces the current provider over the connection that can
/// sign in: here `zai-anthropic` with the default key, because `zai` runs on
/// an account that is not enrolled.
#[tokio::test]
async fn a_replacement_uses_a_connection_with_its_own_credential() {
    let home = tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.model_providers =
        codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None);
    let status_host = |config: &crate::legacy_core::config::Config| {
        ProviderStatusHost::from_config_with_account_registry(
            config,
            crate::provider_status_host::ProviderAccountMetadata::default(),
            |_, _| false,
        )
        .with_environment(|env_key| {
            if env_key == "ZAI_API_KEY" {
                codex_provider_auth::EnvironmentCredentialMetadata::Present
            } else {
                codex_provider_auth::EnvironmentCredentialMetadata::Missing
            }
        })
    };
    let replacement = |status_host: &ProviderStatusHost| {
        let entry = status_host.catalog().get("zai").expect("zai entry").clone();
        replacement_candidates_for(
            status_host,
            &[status(&entry, ProviderEligibilityState::Active)],
            /*current_model*/ None,
        )
        .into_iter()
        .map(|candidate| candidate.runtime_provider_id.as_str().to_string())
        .collect::<Vec<_>>()
    };
    assert_eq!(replacement(&status_host(&config)), ["zai"]);

    config
        .model_providers
        .get_mut("zai")
        .expect("built-in zai provider")
        .account = Some(codex_model_provider_info::NamedProviderAccount {
        provider_id: "zai".into(),
        name: "work".into(),
    });
    assert_eq!(replacement(&status_host(&config)), ["zai-anthropic"]);
}

fn status(
    entry: &codex_provider_auth::ProviderCatalogEntry,
    eligibility: ProviderEligibilityState,
) -> ProviderStatusSnapshot {
    ProviderStatusSnapshot {
        id: entry.id.clone(),
        methods: Vec::new(),
        configuration: ProviderConfigurationState::Configured,
        eligibility,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Ready,
    }
}
