use codex_model_provider_info::ModelProviderInfo;
use codex_provider_auth::ProviderAvailabilityState;
use codex_provider_auth::ProviderCatalog;
use codex_provider_auth::ProviderCatalogId;
use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderCurrentState;
use codex_provider_auth::ProviderEligibilityState;
use codex_provider_auth::ProviderRuntimeId;
use codex_provider_auth::ProviderStatusSnapshot;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn configure_many_preserves_first_fresh_selection_and_returns_to_list() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    configure(&mut session, "alpha", true);
    assert_eq!(
        session.snapshot().first_fresh_runtime,
        Some(runtime_id("alpha"))
    );
    configure(&mut session, "beta", true);
    assert_eq!(
        session.snapshot(),
        &ProviderSetupSnapshot {
            phase: ProviderSetupPhase::ProviderList,
            configured: ids(["alpha", "beta"]),
            usable: ids(["alpha", "beta"]),
            queued_corbanu: false,
            first_fresh_runtime: Some(runtime_id("alpha")),
            preserve_initial_current: false,
        }
    );
}

#[test]
fn existing_usable_current_is_preserved() {
    let mut session =
        ProviderSetupSession::from_statuses(&[status("current", ProviderCurrentState::Current)]);
    let transition = configure(&mut session, "new", true);
    assert!(
        !transition
            .effects
            .iter()
            .any(|effect| matches!(effect, ProviderSetupEffect::PersistInitialSelection(_)))
    );
    assert_eq!(session.snapshot().first_fresh_runtime, None);
}

#[test]
fn late_qualified_claude_status_preserves_existing_current_before_new_setup() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    assert!(session.refresh_from_statuses(&[status(
        codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID,
        ProviderCurrentState::Current,
    )]));

    let transition = configure(&mut session, "new", true);

    assert!(session.snapshot().preserve_initial_current);
    assert_eq!(session.snapshot().first_fresh_runtime, None);
    assert!(
        !transition
            .effects
            .iter()
            .any(|effect| matches!(effect, ProviderSetupEffect::PersistInitialSelection(_)))
    );
}

#[test]
fn late_status_is_stale_during_an_authentication_attempt() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    session.dispatch(ProviderSetupAction::Begin {
        provider_id: catalog_id("alpha"),
    });
    let before = session.snapshot().clone();

    assert!(!session.can_refresh_statuses());
    assert!(!session.refresh_from_statuses(&[status("other", ProviderCurrentState::Current,)]));
    assert_eq!(session.snapshot(), &before);
}

#[test]
fn done_is_gated_and_corbanu_is_deferred_until_done() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    assert!(!session.dispatch(ProviderSetupAction::Done).applied);
    assert!(
        session
            .dispatch(ProviderSetupAction::QueueCorbanu(true))
            .effects
            .is_empty()
    );
    let done = session.dispatch(ProviderSetupAction::Done);
    assert!(matches!(
        done.effects.as_slice(),
        [ProviderSetupEffect::BeginDeferred(
            DeferredProviderSetup::CorbanuPlan {
                has_usable_fallback: false,
                ..
            }
        )]
    ));
}

#[test]
fn deferred_cancel_continues_with_fallback_or_returns_without_one() {
    for (with_fallback, expected_phase) in [
        (true, ProviderSetupPhase::Finished),
        (false, ProviderSetupPhase::ProviderList),
    ] {
        let mut session = ProviderSetupSession::from_statuses(&[]);
        if with_fallback {
            configure(&mut session, "alpha", true);
        }
        session.dispatch(ProviderSetupAction::QueueCorbanu(true));
        session.dispatch(ProviderSetupAction::Done);
        let ProviderSetupPhase::Deferred(DeferredProviderSetup::CorbanuPlan {
            continuation_id,
            ..
        }) = session.snapshot().phase
        else {
            panic!("expected deferred plan");
        };
        assert_eq!(
            session
                .dispatch(ProviderSetupAction::DeferredPlanCancelled { continuation_id })
                .snapshot
                .phase,
            expected_phase
        );
    }
}

#[test]
fn stale_deferred_results_are_ignored() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    session.dispatch(ProviderSetupAction::QueueCorbanu(true));
    session.dispatch(ProviderSetupAction::Done);
    assert!(
        !session
            .dispatch(ProviderSetupAction::DeferredPlanConfigured {
                continuation_id: ProviderSetupContinuationId(99),
            })
            .applied
    );
}

#[test]
fn wrong_provider_completion_is_stale_and_cannot_mutate_the_session() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    session.dispatch(ProviderSetupAction::Begin {
        provider_id: catalog_id("alpha"),
    });
    let before = session.snapshot().clone();
    let transition = session.dispatch(ProviderSetupAction::AuthConfigured {
        provider_id: catalog_id("beta"),
        runtime_provider_id: runtime_id("beta"),
    });
    assert!(!transition.applied);
    assert_eq!(transition.snapshot, before);
    assert!(transition.effects.is_empty());
}

#[test]
fn configured_but_unavailable_provider_does_not_become_the_fresh_default() {
    let mut session = ProviderSetupSession::from_statuses(&[]);
    let unavailable = configure(&mut session, "alpha", false);
    assert_eq!(session.snapshot().first_fresh_runtime, None);
    assert!(
        !unavailable
            .effects
            .iter()
            .any(|effect| matches!(effect, ProviderSetupEffect::PersistInitialSelection(_)))
    );
    let usable = configure(&mut session, "beta", true);
    assert_eq!(
        session.snapshot().first_fresh_runtime,
        Some(runtime_id("beta"))
    );
    assert!(usable.effects.iter().any(|effect| matches!(
        effect,
        ProviderSetupEffect::PersistInitialSelection(runtime) if *runtime == runtime_id("beta")
    )));
}

fn configure(
    session: &mut ProviderSetupSession,
    id: &str,
    usable: bool,
) -> ProviderSetupTransition {
    session.dispatch(ProviderSetupAction::Begin {
        provider_id: catalog_id(id),
    });
    session.dispatch(ProviderSetupAction::AuthConfigured {
        provider_id: catalog_id(id),
        runtime_provider_id: runtime_id(id),
    });
    session.dispatch(ProviderSetupAction::ActivationResolved {
        provider_id: catalog_id(id),
        runtime_provider_id: runtime_id(id),
        usable,
    })
}

fn status(id: &str, current: ProviderCurrentState) -> ProviderStatusSnapshot {
    ProviderStatusSnapshot {
        id: catalog_id(id),
        methods: vec![],
        configuration: ProviderConfigurationState::Configured,
        eligibility: ProviderEligibilityState::Active,
        current,
        availability: ProviderAvailabilityState::Ready,
    }
}

fn ids<const N: usize>(values: [&str; N]) -> std::collections::BTreeSet<ProviderCatalogId> {
    values.into_iter().map(catalog_id).collect()
}

fn entry(id: &str) -> codex_provider_auth::ProviderCatalogEntry {
    let catalog = ProviderCatalog::from_runtime_providers(&std::collections::HashMap::from([(
        id.to_string(),
        ModelProviderInfo {
            name: id.to_string(),
            env_key: Some(format!("{}_KEY", id.to_ascii_uppercase())),
            ..Default::default()
        },
    )]));
    catalog.entries()[0].clone()
}

fn catalog_id(id: &str) -> ProviderCatalogId {
    entry(id).id
}

fn runtime_id(id: &str) -> ProviderRuntimeId {
    entry(id).runtime_provider_ids[0].clone()
}
