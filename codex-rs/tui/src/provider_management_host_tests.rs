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
    config.model_providers = codex_model_provider_info::built_in_model_providers(None);
    let status_host = ProviderStatusHost::from_config(
        &config,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );
    let entries = status_host.catalog().entries();
    let ready = entries
        .iter()
        .find(|entry| {
            entry.runtime_provider_ids.first().is_some_and(|runtime| {
                codex_model_provider_info::resolve_model_for_provider(None, runtime.as_str())
                    .is_some()
            })
        })
        .unwrap();
    let rejected = entries.iter().find(|entry| entry.id != ready.id).unwrap();
    let statuses = vec![
        status(ready, ProviderEligibilityState::Active),
        status(rejected, ProviderEligibilityState::Inactive),
    ];

    let candidates = replacement_candidates_for(&status_host, &statuses, None);

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
    assert!(replacement_candidates_for(&status_host, &[current, unavailable], None).is_empty());
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
