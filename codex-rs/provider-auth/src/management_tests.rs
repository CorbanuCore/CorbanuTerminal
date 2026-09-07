use std::collections::HashMap;

use codex_model_provider_info::ModelProviderInfo;
use pretty_assertions::assert_eq;

use super::*;
use crate::ExplicitProviderSelection;
use crate::ProviderCatalog;
use crate::ProviderRuntimeId;

#[test]
fn newly_configured_defaults_active_but_recovery_preserves_explicit_inactivity() {
    let inactive = status(
        "inactive",
        ProviderConfigurationState::Configured,
        ProviderEligibilityState::Inactive,
        ProviderCurrentState::NotCurrent,
        ProviderAvailabilityState::Ready,
    );
    let mut session = ProviderManagementSession::new(vec![inactive.clone()]);
    let provider_id = inactive.id;
    assert!(
        session
            .dispatch(ProviderManagementAction::BeginAuthentication {
                provider_id: provider_id.clone(),
            })
            .applied
    );
    let recovered =
        session.dispatch(ProviderManagementAction::AuthenticationConfigured { provider_id });
    assert_eq!(recovered.phase, ProviderManagementPhase::Browsing);
    assert_eq!(recovered.effects, vec![ProviderManagementEffect::Refresh]);

    let missing = status(
        "new",
        ProviderConfigurationState::NotConfigured,
        ProviderEligibilityState::NotConfigured,
        ProviderCurrentState::NotCurrent,
        ProviderAvailabilityState::Unavailable {
            reason: crate::ProviderUnavailableReason::NotConfigured,
        },
    );
    let mut session = ProviderManagementSession::new(vec![missing.clone()]);
    let provider_id = missing.id;
    session.dispatch(ProviderManagementAction::BeginAuthentication {
        provider_id: provider_id.clone(),
    });
    let configured = session.dispatch(ProviderManagementAction::AuthenticationConfigured {
        provider_id: provider_id.clone(),
    });
    assert!(matches!(
        configured.effects.as_slice(),
        [ProviderManagementEffect::PersistEligibility {
            provider_id: effected,
            policy: ProviderActivationPolicy::Active,
            ..
        }] if *effected == provider_id
    ));
}

#[test]
fn current_deactivation_requires_explicit_usable_replacement_and_cancel_is_inert() {
    let current = usable("current", ProviderCurrentState::Current);
    let replacement = usable("replacement", ProviderCurrentState::NotCurrent);
    let mut session = ProviderManagementSession::new(vec![current.clone(), replacement]);
    let transition = session.dispatch(ProviderManagementAction::RequestPolicy {
        provider_id: current.id.clone(),
        policy: ProviderActivationPolicy::Inactive,
    });
    assert_eq!(
        transition.effects,
        vec![ProviderManagementEffect::PresentReplacement {
            target_provider_id: current.id.clone(),
        }]
    );
    assert!(matches!(
        transition.phase,
        ProviderManagementPhase::AwaitingReplacement { .. }
    ));

    let cancelled = session.dispatch(ProviderManagementAction::CancelReplacement {
        target_provider_id: current.id,
    });
    assert_eq!(cancelled.phase, ProviderManagementPhase::Browsing);
    assert_eq!(cancelled.effects, vec![ProviderManagementEffect::Refresh]);
}

#[test]
fn replacement_is_exact_and_stale_completion_cannot_finish_it() {
    let current = usable("current", ProviderCurrentState::Current);
    let replacement = usable("replacement", ProviderCurrentState::NotCurrent);
    let mut session = ProviderManagementSession::new(vec![current.clone(), replacement.clone()]);
    session.dispatch(ProviderManagementAction::RequestPolicy {
        provider_id: current.id.clone(),
        policy: ProviderActivationPolicy::Inactive,
    });
    let selection = ExplicitProviderSelection {
        provider_id: replacement.id,
        runtime_provider_id: runtime_id("replacement"),
        model: "replacement-model".to_string(),
    };
    let chosen = session.dispatch(ProviderManagementAction::ChooseReplacement {
        target_provider_id: current.id.clone(),
        replacement: selection.clone(),
    });
    let [
        ProviderManagementEffect::PersistReplacementThenDeactivate {
            attempt_id,
            target_provider_id,
            replacement: effected,
        },
    ] = chosen.effects.as_slice()
    else {
        panic!("expected replacement persistence effect")
    };
    assert_eq!(target_provider_id, &current.id);
    assert_eq!(effected, &selection);

    let stale = ProviderManagementAttemptId(attempt_id.0.saturating_add(1));
    let ignored = session.dispatch(ProviderManagementAction::PersistenceFinished {
        attempt_id: stale,
        result: ProviderManagementPersistenceResult::Applied,
    });
    assert!(!ignored.applied);
    assert_eq!(ignored.phase, chosen.phase);

    let finished = session.dispatch(ProviderManagementAction::PersistenceFinished {
        attempt_id: *attempt_id,
        result: ProviderManagementPersistenceResult::ReplacementAppliedDeactivationFailed,
    });
    assert!(finished.applied);
    assert_eq!(finished.phase, ProviderManagementPhase::Browsing);
    assert_eq!(
        finished.persistence_result,
        Some(ProviderManagementPersistenceResult::ReplacementAppliedDeactivationFailed)
    );
}

#[test]
fn non_current_activation_changes_only_eligibility() {
    let inactive = status(
        "inactive",
        ProviderConfigurationState::Configured,
        ProviderEligibilityState::Inactive,
        ProviderCurrentState::NotCurrent,
        ProviderAvailabilityState::Ready,
    );
    let mut session = ProviderManagementSession::new(vec![inactive.clone()]);
    let transition = session.dispatch(ProviderManagementAction::RequestPolicy {
        provider_id: inactive.id.clone(),
        policy: ProviderActivationPolicy::Active,
    });
    assert!(matches!(
        transition.effects.as_slice(),
        [ProviderManagementEffect::PersistEligibility {
            provider_id,
            policy: ProviderActivationPolicy::Active,
            ..
        }] if *provider_id == inactive.id
    ));
}

#[test]
fn replacement_write_failure_does_not_authorize_deactivation() {
    let current = usable("current", ProviderCurrentState::Current);
    let replacement = usable("replacement", ProviderCurrentState::NotCurrent);
    let mut session = ProviderManagementSession::new(vec![current.clone(), replacement.clone()]);
    session.dispatch(ProviderManagementAction::RequestPolicy {
        provider_id: current.id.clone(),
        policy: ProviderActivationPolicy::Inactive,
    });
    let chosen = session.dispatch(ProviderManagementAction::ChooseReplacement {
        target_provider_id: current.id.clone(),
        replacement: ExplicitProviderSelection {
            provider_id: replacement.id.clone(),
            runtime_provider_id: runtime_id("replacement"),
            model: "exact-model".into(),
        },
    });
    let attempt_id = match chosen.phase {
        ProviderManagementPhase::Persisting { attempt_id, .. } => attempt_id,
        _ => panic!("expected persistence"),
    };
    let failed = session.dispatch(ProviderManagementAction::PersistenceFinished {
        attempt_id,
        result: ProviderManagementPersistenceResult::Failed,
    });
    assert_eq!(failed.phase, ProviderManagementPhase::Browsing);
    assert_eq!(failed.effects, vec![ProviderManagementEffect::Refresh]);
    assert_eq!(session.statuses(), &[current, replacement]);
}

#[test]
fn configured_current_provider_cannot_be_directly_deactivated() {
    let current = usable("current", ProviderCurrentState::Current);
    let mut session = ProviderManagementSession::new(vec![current.clone()]);
    let transition = session.dispatch(ProviderManagementAction::RequestPolicy {
        provider_id: current.id,
        policy: ProviderActivationPolicy::Inactive,
    });
    assert!(matches!(
        transition.effects.as_slice(),
        [ProviderManagementEffect::PresentReplacement { .. }]
    ));
    assert!(!transition.effects.iter().any(|effect| matches!(
        effect,
        ProviderManagementEffect::PersistEligibility {
            policy: ProviderActivationPolicy::Inactive,
            ..
        }
    )));
}

fn usable(id: &str, current: ProviderCurrentState) -> ProviderStatusSnapshot {
    status(
        id,
        ProviderConfigurationState::Configured,
        ProviderEligibilityState::Active,
        current,
        ProviderAvailabilityState::Ready,
    )
}

fn status(
    id: &str,
    configuration: ProviderConfigurationState,
    eligibility: ProviderEligibilityState,
    current: ProviderCurrentState,
    availability: ProviderAvailabilityState,
) -> ProviderStatusSnapshot {
    ProviderStatusSnapshot {
        id: entry(id).id,
        methods: Vec::new(),
        configuration,
        eligibility,
        current,
        availability,
    }
}

fn runtime_id(id: &str) -> ProviderRuntimeId {
    entry(id).runtime_provider_ids[0].clone()
}

fn entry(id: &str) -> crate::ProviderCatalogEntry {
    ProviderCatalog::from_runtime_providers(&HashMap::from([(
        id.to_string(),
        ModelProviderInfo {
            name: id.to_string(),
            env_key: Some(format!("{}_API_KEY", id.to_ascii_uppercase())),
            ..Default::default()
        },
    )]))
    .entries()[0]
        .clone()
}
