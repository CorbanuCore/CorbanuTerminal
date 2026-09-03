use super::*;
use crate::ProviderCatalogEntry;
use crate::ProviderCatalogId;
use crate::ProviderCurrentState;
use crate::ProviderMethodStatus;
use crate::ProviderSetupCapabilities;
use crate::ProviderSetupCapability;
use crate::ProviderStatusSnapshot;
use crate::ProviderUnavailableReason;

fn fixture(
    configuration: ProviderConfigurationState,
    eligibility: ProviderEligibilityState,
    availability: ProviderAvailabilityState,
) -> (ProviderCatalog, ProviderStatusCatalog) {
    let id = ProviderCatalogId("custom".into());
    (
        ProviderCatalog {
            entries: vec![ProviderCatalogEntry {
                id: id.clone(),
                display_name: "Custom".into(),
                runtime_provider_ids: vec![ProviderRuntimeId("custom-runtime".into())],
                setup_capabilities: ProviderSetupCapabilities::one(
                    ProviderSetupCapability::StatusOnly {
                        reason: crate::StatusOnlyReason::NoInteractiveSetup,
                    },
                ),
            }],
        },
        ProviderStatusCatalog {
            entries: vec![ProviderStatusSnapshot {
                id,
                methods: Vec::new(),
                configuration,
                eligibility,
                current: ProviderCurrentState::Current,
                availability,
            }],
        },
    )
}

#[test]
fn active_ready_exact_runtime_is_preserved() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::Configured,
        ProviderEligibilityState::Active,
        ProviderAvailabilityState::Ready,
    );

    let decision = ProviderRuntimeSelectionPolicy::current(
        &catalog,
        &statuses,
        &ProviderRuntimeAuthorizations::default(),
        "custom-runtime",
        "same-model",
    );

    let CurrentSelectionDecision::Preserve(selection) = decision else {
        panic!("usable current selection should be preserved");
    };
    assert_eq!(selection.runtime_provider_id.as_str(), "custom-runtime");
    assert_eq!(selection.model, "same-model");
}

#[test]
fn inactive_current_requires_explicit_recovery() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::Configured,
        ProviderEligibilityState::Inactive,
        ProviderAvailabilityState::Ready,
    );

    assert!(matches!(
        ProviderRuntimeSelectionPolicy::current(
            &catalog,
            &statuses,
            &ProviderRuntimeAuthorizations::default(),
            "custom-runtime",
            "same-model",
        ),
        CurrentSelectionDecision::RequireExplicitRecovery {
            reason: ProviderUseBlocker::Inactive,
            ..
        }
    ));
}

#[test]
fn unknown_current_is_not_rewritten_to_a_catalog_default() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::Configured,
        ProviderEligibilityState::Active,
        ProviderAvailabilityState::Ready,
    );

    assert!(matches!(
        ProviderRuntimeSelectionPolicy::current(
            &catalog,
            &statuses,
            &ProviderRuntimeAuthorizations::default(),
            "removed-provider",
            "historical-model",
        ),
        CurrentSelectionDecision::RequireExplicitRecovery {
            requested_runtime_provider_id,
            requested_model,
            reason: ProviderUseBlocker::UnknownProvider,
        } if requested_runtime_provider_id == "removed-provider"
            && requested_model == "historical-model"
    ));
}

#[test]
fn status_only_is_visible_without_eager_runtime_authorization() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::NotConfigured,
        ProviderEligibilityState::NotConfigured,
        ProviderAvailabilityState::StatusOnly,
    );
    let mut authorizations = ProviderRuntimeAuthorizations::default();

    assert!(matches!(
        ProviderRuntimeSelectionPolicy::assess(
            &catalog,
            &statuses,
            &authorizations,
            "custom-runtime",
            "custom-model",
            ProviderUseContext::ModelPicker,
        ),
        ProviderUseDecision::Ready(_)
    ));

    authorizations.set("custom-runtime", ProviderRuntimeAuthorization::Authorized);
    assert!(matches!(
        ProviderRuntimeSelectionPolicy::assess(
            &catalog,
            &statuses,
            &authorizations,
            "custom-runtime",
            "custom-model",
            ProviderUseContext::ModelPicker,
        ),
        ProviderUseDecision::Ready(_)
    ));
}

#[test]
fn explicit_status_only_use_is_preserved_for_the_runtime_adapter() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::NotConfigured,
        ProviderEligibilityState::NotConfigured,
        ProviderAvailabilityState::StatusOnly,
    );

    assert!(matches!(
        ProviderRuntimeSelectionPolicy::assess(
            &catalog,
            &statuses,
            &ProviderRuntimeAuthorizations::default(),
            "custom-runtime",
            "custom-model",
            ProviderUseContext::NativeSpawn,
        ),
        ProviderUseDecision::Ready(_)
    ));
}

#[test]
fn rejected_runtime_authorization_is_typed_and_secret_free() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::NotConfigured,
        ProviderEligibilityState::NotConfigured,
        ProviderAvailabilityState::StatusOnly,
    );
    let mut authorizations = ProviderRuntimeAuthorizations::default();
    authorizations.set("custom-runtime", ProviderRuntimeAuthorization::Rejected);

    let decision = ProviderRuntimeSelectionPolicy::assess(
        &catalog,
        &statuses,
        &authorizations,
        "custom-runtime",
        "custom-model",
        ProviderUseContext::NativeSpawn,
    );
    assert!(matches!(
        decision,
        ProviderUseDecision::Blocked {
            reason: ProviderUseBlocker::RuntimeAuthorizationRejected,
            ..
        }
    ));
    assert!(!format!("{decision:?}").contains("credential"));
}

#[test]
fn status_only_runtime_cannot_bypass_unavailable_eligibility() {
    let (catalog, statuses) = fixture(
        ProviderConfigurationState::NotConfigured,
        ProviderEligibilityState::Unavailable,
        ProviderAvailabilityState::StatusOnly,
    );
    let mut authorizations = ProviderRuntimeAuthorizations::default();
    authorizations.set("custom-runtime", ProviderRuntimeAuthorization::Authorized);

    assert!(matches!(
        ProviderRuntimeSelectionPolicy::assess(
            &catalog,
            &statuses,
            &authorizations,
            "custom-runtime",
            "custom-model",
            ProviderUseContext::NativeSpawn,
        ),
        ProviderUseDecision::Blocked {
            reason: ProviderUseBlocker::Unavailable,
            ..
        }
    ));
}

#[test]
fn command_authorization_preserves_unavailable_eligibility() {
    let (mut catalog, mut statuses) = fixture(
        ProviderConfigurationState::NotConfigured,
        ProviderEligibilityState::Unavailable,
        ProviderAvailabilityState::Unavailable {
            reason: ProviderUnavailableReason::RecoveryRequired,
        },
    );
    let capability = ProviderSetupCapability::CommandAuth {
        setup: crate::CommandAuthSetup::StatusOnly,
    };
    catalog.entries[0].setup_capabilities = ProviderSetupCapabilities::one(capability.clone());
    statuses.entries[0].methods = vec![ProviderMethodStatus {
        capability,
        state: ProviderMethodState::StatusOnly,
    }];
    let mut authorizations = ProviderRuntimeAuthorizations::default();
    authorizations.set("custom-runtime", ProviderRuntimeAuthorization::Authorized);

    authorizations.apply_to_status_catalog(&catalog, &mut statuses);

    assert_eq!(
        statuses.entries[0].eligibility,
        ProviderEligibilityState::Unavailable
    );
    assert!(matches!(
        statuses.entries[0].availability,
        ProviderAvailabilityState::Unavailable { .. }
    ));
}
