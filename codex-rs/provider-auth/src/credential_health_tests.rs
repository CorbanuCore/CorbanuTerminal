use super::*;
use crate::*;
use pretty_assertions::assert_eq;

fn configured(
    id: &str,
    source: ProviderCredentialSource,
    control: CredentialControl,
) -> ProviderStatusSnapshot {
    let catalog = ProviderCatalog::from_runtime_providers(
        &codex_model_provider_info::built_in_model_providers(None),
    );
    let entry = catalog.get(id).unwrap();
    ProviderStatusSnapshot {
        id: entry.id.clone(),
        methods: vec![ProviderMethodStatus {
            capability: entry.setup_capabilities.iter().next().unwrap().clone(),
            state: ProviderMethodState::Configured {
                source,
                control,
                availability: ConfiguredAvailability::Ready,
            },
        }],
        configuration: ProviderConfigurationState::Configured,
        eligibility: ProviderEligibilityState::Inactive,
        current: ProviderCurrentState::Current,
        availability: ProviderAvailabilityState::Ready,
    }
}

#[test]
fn rejection_identity_is_captured_and_consumed_once() {
    let mut health = ProviderCredentialHealth::default();
    health.begin(
        "a".into(),
        &configured(
            "openai",
            ProviderCredentialSource::OpenAiAccount,
            CredentialControl::ManagedByCorbanu,
        ),
    );
    health.begin(
        "b".into(),
        &configured(
            "claude-plan",
            ProviderCredentialSource::ClaudeManaged,
            CredentialControl::ManagedByCorbanu,
        ),
    );
    assert_eq!(health.reject_provider("a"), Some("openai".into()));
    assert_eq!(health.reject_provider("a"), None);
    health.credential_changed("claude-plan");
    assert_eq!(health.reject_provider("b"), None);
    assert_eq!(health.reject_provider("missing"), None);
}

#[test]
fn rejected_credential_preserves_selection_activation_and_unrelated_provider() {
    for (source, control) in [
        (
            ProviderCredentialSource::OpenAiAccount,
            CredentialControl::ManagedByCorbanu,
        ),
        (
            ProviderCredentialSource::EncryptedVault,
            CredentialControl::ManagedByCorbanu,
        ),
        (
            ProviderCredentialSource::Environment,
            CredentialControl::ExternalEnvironment,
        ),
        (
            ProviderCredentialSource::ExternallyManaged,
            CredentialControl::ExternalProvider,
        ),
    ] {
        let original = configured("openai", source, control);
        let unrelated = configured(
            "claude-plan",
            ProviderCredentialSource::ClaudeManaged,
            CredentialControl::ManagedByCorbanu,
        );
        let mut statuses = ProviderStatusCatalog {
            entries: vec![original.clone(), unrelated.clone()],
        };
        let mut health = ProviderCredentialHealth::default();
        health.begin("request".into(), &original);
        assert!(health.reject("request"));
        health.apply(&mut statuses);
        let mut expected = original.clone();
        expected.methods[0].state = ProviderMethodState::RecoveryRequired {
            reason: ProviderRecoveryReason::CredentialRejected { source, control },
        };
        expected.configuration = ProviderConfigurationState::RecoveryRequired;
        expected.availability = ProviderAvailabilityState::Unavailable {
            reason: ProviderUnavailableReason::RecoveryRequired,
        };
        assert_eq!(statuses.entries(), &[expected, unrelated.clone()]);
        health.credential_changed("openai");
        let mut after = ProviderStatusCatalog {
            entries: vec![original.clone(), unrelated.clone()],
        };
        health.apply(&mut after);
        assert_eq!(after.entries(), &[original, unrelated]);
    }
}

#[test]
fn changed_credential_rejects_stale_failure_and_cancel_does_not_clear_failure() {
    let original = configured(
        "openai",
        ProviderCredentialSource::OpenAiAccount,
        CredentialControl::ManagedByCorbanu,
    );
    let mut health = ProviderCredentialHealth::default();
    health.begin("old".into(), &original);
    health.credential_changed("openai");
    assert!(!health.reject("old"));
    health.begin("current".into(), &original);
    assert!(health.reject("current"));
    health.finish("current");
    let mut statuses = ProviderStatusCatalog {
        entries: vec![original],
    };
    health.apply(&mut statuses);
    assert_eq!(
        statuses.entries()[0].configuration,
        ProviderConfigurationState::RecoveryRequired
    );
    health.begin("repeated".into(), &statuses.entries()[0]);
    assert!(!health.reject("repeated"));
    let mut rediscovered = ProviderStatusCatalog {
        entries: vec![configured(
            "openai",
            ProviderCredentialSource::OpenAiAccount,
            CredentialControl::ManagedByCorbanu,
        )],
    };
    health.apply(&mut rediscovered);
    assert_eq!(rediscovered.entries(), statuses.entries());
}

#[test]
fn old_source_failure_does_not_poison_replacement_source() {
    let original = configured(
        "openai",
        ProviderCredentialSource::OpenAiAccount,
        CredentialControl::ManagedByCorbanu,
    );
    let replacement = configured(
        "openai",
        ProviderCredentialSource::OpenAiApiKey,
        CredentialControl::ManagedByCorbanu,
    );
    let mut health = ProviderCredentialHealth::default();
    health.begin("old".into(), &original);
    health.reject("old");
    let mut statuses = ProviderStatusCatalog {
        entries: vec![replacement.clone()],
    };
    health.apply(&mut statuses);
    assert_eq!(statuses.entries(), &[replacement]);
}

#[test]
fn no_auth_local_provider_is_not_given_a_login_failure() {
    let original = configured(
        "openai",
        ProviderCredentialSource::Local,
        CredentialControl::None,
    );
    let mut health = ProviderCredentialHealth::default();
    health.begin("local".into(), &original);
    health.reject("local");
    let mut statuses = ProviderStatusCatalog {
        entries: vec![original.clone()],
    };
    health.apply(&mut statuses);
    assert_eq!(statuses.entries(), &[original]);
}
