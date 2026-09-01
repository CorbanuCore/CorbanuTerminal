use std::collections::BTreeSet;
use std::collections::HashMap;

use codex_model_provider_info::ModelProviderInfo;
use pretty_assertions::assert_eq;

use super::*;
use crate::ProviderCatalog;

#[test]
fn shared_custom_api_key_has_stable_eligibility_identity() {
    let providers = HashMap::from([
        (
            "zeta".to_string(),
            ModelProviderInfo {
                name: "Zeta".to_string(),
                env_key: Some("SHARED_API_KEY".to_string()),
                ..ModelProviderInfo::default()
            },
        ),
        (
            "alpha".to_string(),
            ModelProviderInfo {
                name: "Alpha".to_string(),
                env_key: Some("SHARED_API_KEY".to_string()),
                ..ModelProviderInfo::default()
            },
        ),
    ]);
    let catalog = ProviderCatalog::from_runtime_providers(&providers);

    assert_eq!(
        catalog
            .entries()
            .iter()
            .map(ProviderEligibilityId::for_entry)
            .collect::<Vec<_>>(),
        vec![ProviderEligibilityId(
            "credential-env:SHARED_API_KEY".to_string()
        )]
    );
}

#[test]
fn missing_state_migrates_configured_providers_to_active_without_touching_config() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let config_path = codex_home.path().join("config.toml");
    let config = "model = \"existing-model\"\nmodel_provider = \"existing-provider\"\n";
    std::fs::write(&config_path, config).expect("write config fixture");
    let entry = single_entry();

    let eligibility = ProviderEligibilityStore::new(codex_home.path())
        .load()
        .expect("missing state should migrate by interpretation");

    assert_eq!(
        (
            eligibility.policy_for(&entry),
            std::fs::read_to_string(config_path).expect("read unchanged config"),
            codex_home.path().join(PROVIDER_ELIGIBILITY_FILE).exists(),
        ),
        (ProviderActivationPolicy::Active, config.to_string(), false)
    );
}

#[test]
fn round_trip_is_deterministic_retains_unknown_identities_and_survives_restart() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let store = ProviderEligibilityStore::new(codex_home.path());
    let entry = single_entry();
    let mut eligibility = ProviderEligibility {
        inactive_identities: BTreeSet::from([ProviderEligibilityId(
            "provider:future-provider".to_string(),
        )]),
    };
    eligibility.set_policy(&entry, ProviderActivationPolicy::Inactive);

    store.save(&eligibility).expect("save eligibility");
    let first = std::fs::read_to_string(codex_home.path().join(PROVIDER_ELIGIBILITY_FILE))
        .expect("read eligibility");
    let restarted = ProviderEligibilityStore::new(codex_home.path())
        .load()
        .expect("load eligibility after restart");
    store.save(&restarted).expect("resave eligibility");
    let second = std::fs::read_to_string(codex_home.path().join(PROVIDER_ELIGIBILITY_FILE))
        .expect("read resaved eligibility");

    assert_eq!(
        first,
        "{\n  \"version\": 1,\n  \"inactive_identities\": [\n    \"credential-env:CUSTOM_API_KEY\",\n    \"provider:future-provider\"\n  ]\n}\n"
    );
    assert_eq!((restarted, second), (eligibility, first));
    assert_eq!(
        std::fs::read_dir(codex_home.path())
            .expect("read codex home")
            .count(),
        1
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(codex_home.path().join(PROVIDER_ELIGIBILITY_FILE))
                .expect("eligibility metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn malformed_and_future_state_fail_visibly_without_overwrite() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let path = codex_home.path().join(PROVIDER_ELIGIBILITY_FILE);
    let store = ProviderEligibilityStore::new(codex_home.path());

    std::fs::write(&path, "secret-canary-not-json").expect("write malformed state");
    assert_eq!(store.load(), Err(ProviderEligibilityError::Malformed));
    assert_eq!(
        std::fs::read_to_string(&path).expect("malformed state retained"),
        "secret-canary-not-json"
    );

    let future = "{\"version\":99,\"inactive_identities\":[\"provider:future\"]}";
    std::fs::write(&path, future).expect("write future state");
    assert_eq!(
        store.load(),
        Err(ProviderEligibilityError::UnsupportedVersion { found: 99 })
    );
    assert_eq!(
        std::fs::read_to_string(path).expect("future state retained"),
        future
    );
}

fn single_entry() -> crate::ProviderCatalogEntry {
    let catalog = ProviderCatalog::from_runtime_providers(&HashMap::from([(
        "custom".to_string(),
        ModelProviderInfo {
            name: "Custom".to_string(),
            env_key: Some("CUSTOM_API_KEY".to_string()),
            ..ModelProviderInfo::default()
        },
    )]));
    catalog.entries()[0].clone()
}
