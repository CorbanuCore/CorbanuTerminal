use codex_login::ProviderApiKeyStorageSource;
use pretty_assertions::assert_eq;

use super::*;
use crate::*;

#[test]
fn transition_contract_freezes_cancel_commit_timeout_and_enclosing_redaction() {
    let target = target();
    let flow = flow(target.clone(), ApiKeyFlowIntent::Replace);
    let mut cancelled = entering(target.clone(), ApiKeyFlowIntent::Replace);
    assert_eq!(
        cancelled.dispatch(ProviderAuthAction::Cancel),
        transition(
            ProviderAuthFlowSnapshot::Cancelled {
                target: target.clone()
            },
            vec![ProviderAuthEffect::Complete(
                ProviderAuthCompletion::Cancelled {
                    target: target.clone(),
                }
            )],
            ProviderAuthDisposition::Applied,
        )
    );

    let mut submitted = entering(target.clone(), ApiKeyFlowIntent::Replace);
    submitted.dispatch(ProviderAuthAction::SetApiKey(ApiKeySecret::new(
        "canary-secret",
    )));
    let submitted_transition = submitted.dispatch(ProviderAuthAction::Submit);
    assert_eq!(
        submitted_transition,
        transition(
            ProviderAuthFlowSnapshot::Settling {
                flow,
                attempt_id: attempt(1)
            },
            vec![
                ProviderAuthEffect::PersistApiKey {
                    attempt_id: attempt(1),
                    target,
                    secret: ApiKeySecret::new("canary-secret"),
                },
                ProviderAuthEffect::ScheduleTimeout {
                    attempt_id: attempt(1),
                    timeout: Duration::from_secs(120),
                },
            ],
            ProviderAuthDisposition::Applied,
        )
    );
    assert_eq!(
        submitted.dispatch(ProviderAuthAction::Cancel).disposition,
        ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
    );
    let action = ProviderAuthAction::SetApiKey(ApiKeySecret::new("canary-secret"));
    assert!(!format!("{action:?}{submitted_transition:?}").contains("canary-secret"));
}

#[test]
fn environment_precedence_blocks_add_and_replace_for_all_managed_states() {
    let cases = [
        (
            EnvironmentCredentialMetadata::Present,
            ManagedApiKeyMetadata::Missing,
            ProviderAuthBlockedReason::EnvironmentCredentialPresent,
        ),
        (
            EnvironmentCredentialMetadata::Present,
            ManagedApiKeyMetadata::Stored {
                source: ProviderApiKeyStorageSource::EncryptedVault,
            },
            ProviderAuthBlockedReason::EnvironmentCredentialPresent,
        ),
        (
            EnvironmentCredentialMetadata::Invalid,
            ManagedApiKeyMetadata::Suppressed,
            ProviderAuthBlockedReason::InvalidEnvironmentCredential,
        ),
        (
            EnvironmentCredentialMetadata::Invalid,
            ManagedApiKeyMetadata::Stored {
                source: ProviderApiKeyStorageSource::LegacyPlaintext,
            },
            ProviderAuthBlockedReason::InvalidEnvironmentCredential,
        ),
    ];
    for (environment, managed, reason) in cases {
        for intent in [ApiKeyFlowIntent::Add, ApiKeyFlowIntent::Replace] {
            let target = target();
            let mut controller = ProviderAuthController::default();
            let result = controller.dispatch(start(
                target.clone(),
                intent,
                ApiKeyCredentialMetadata {
                    environment,
                    managed,
                },
            ));
            assert_eq!(
                result.snapshot,
                ProviderAuthFlowSnapshot::Blocked {
                    flow: flow(target, intent),
                    reason
                }
            );
        }
    }
}

#[test]
fn stored_result_reconciles_to_metadata_only_completion() {
    let target = target();
    let mut controller = submitted(target.clone());
    assert_eq!(
        controller.dispatch(ProviderAuthAction::PersistenceFinished {
            attempt_id: attempt(1),
            result: ApiKeyPersistenceResult::Stored,
        }),
        transition(
            ProviderAuthFlowSnapshot::Reconciling {
                flow: flow(target.clone(), ApiKeyFlowIntent::Add),
                attempt_id: attempt(1),
            },
            vec![ProviderAuthEffect::RefreshProviderStatus {
                attempt_id: attempt(1),
                target: target.clone(),
            }],
            ProviderAuthDisposition::Applied,
        )
    );
    let status = status(&target, ProviderConfigurationState::Configured);
    assert_eq!(
        controller.dispatch(ProviderAuthAction::StatusResolved {
            attempt_id: attempt(1),
            status: status.clone(),
        }),
        transition(
            ProviderAuthFlowSnapshot::Configured {
                target: target.clone(),
                status: status.clone()
            },
            vec![ProviderAuthEffect::Complete(
                ProviderAuthCompletion::Configured { target, status }
            )],
            ProviderAuthDisposition::Applied,
        )
    );
}

#[test]
fn timeout_nonconfigured_metadata_stays_unknown_until_late_stored_settlement() {
    let target = target();
    let unknown = ProviderAuthFlowSnapshot::OutcomeUnknown {
        flow: flow(target.clone(), ApiKeyFlowIntent::Add),
        attempt_id: attempt(1),
    };
    let mut controller = submitted(target.clone());
    assert_eq!(
        controller.dispatch(ProviderAuthAction::TimeoutElapsed {
            attempt_id: attempt(1)
        }),
        transition(
            unknown.clone(),
            vec![ProviderAuthEffect::RefreshProviderStatus {
                attempt_id: attempt(1),
                target: target.clone(),
            }],
            ProviderAuthDisposition::Applied,
        )
    );
    for action in [ProviderAuthAction::Retry, ProviderAuthAction::Cancel] {
        assert_eq!(
            controller.dispatch(action).disposition,
            ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
        );
    }
    assert_eq!(
        controller
            .dispatch(ProviderAuthAction::StatusResolved {
                attempt_id: attempt(2),
                status: status(&target, ProviderConfigurationState::Configured),
            })
            .disposition,
        ProviderAuthDisposition::IgnoredStale
    );
    for configuration in [
        ProviderConfigurationState::NotConfigured,
        ProviderConfigurationState::Checking,
        ProviderConfigurationState::Unavailable,
        ProviderConfigurationState::RecoveryRequired,
    ] {
        assert_eq!(
            controller.dispatch(ProviderAuthAction::StatusResolved {
                attempt_id: attempt(1),
                status: status(&target, configuration),
            }),
            transition(unknown.clone(), vec![], ProviderAuthDisposition::Applied)
        );
    }
    assert_eq!(
        controller.dispatch(ProviderAuthAction::PersistenceFinished {
            attempt_id: attempt(1),
            result: ApiKeyPersistenceResult::Stored,
        }),
        stored_reconciliation(target, ApiKeyFlowIntent::Add)
    );
}

#[test]
fn timeout_then_stored_settlement_reconciles_without_duplicate_persistence() {
    let target = target();
    let mut controller = submitted(target.clone());
    controller.dispatch(ProviderAuthAction::TimeoutElapsed {
        attempt_id: attempt(1),
    });
    assert_eq!(
        controller.dispatch(ProviderAuthAction::PersistenceFinished {
            attempt_id: attempt(1),
            result: ApiKeyPersistenceResult::Stored,
        }),
        stored_reconciliation(target, ApiKeyFlowIntent::Add)
    );
}

#[test]
fn configured_after_timeout_completes_add_but_replace_waits_for_fresh_status() {
    let target = target();
    let configured = status(&target, ProviderConfigurationState::Configured);
    let mut add = submitted(target.clone());
    add.dispatch(ProviderAuthAction::TimeoutElapsed {
        attempt_id: attempt(1),
    });
    assert_eq!(
        add.dispatch(ProviderAuthAction::StatusResolved {
            attempt_id: attempt(1),
            status: configured.clone(),
        }),
        configured_transition(target.clone(), configured.clone())
    );

    let replace_flow = flow(target.clone(), ApiKeyFlowIntent::Replace);
    let mut replace = submitted_with_intent(target.clone(), ApiKeyFlowIntent::Replace);
    replace.dispatch(ProviderAuthAction::TimeoutElapsed {
        attempt_id: attempt(1),
    });
    assert_eq!(
        replace.dispatch(ProviderAuthAction::StatusResolved {
            attempt_id: attempt(1),
            status: configured.clone(),
        }),
        transition(
            ProviderAuthFlowSnapshot::OutcomeUnknown {
                flow: replace_flow,
                attempt_id: attempt(1),
            },
            vec![],
            ProviderAuthDisposition::Applied,
        )
    );
    assert_eq!(
        replace.dispatch(ProviderAuthAction::PersistenceFinished {
            attempt_id: attempt(1),
            result: ApiKeyPersistenceResult::Stored,
        }),
        stored_reconciliation(target.clone(), ApiKeyFlowIntent::Replace)
    );
    assert_eq!(
        replace.dispatch(ProviderAuthAction::StatusResolved {
            attempt_id: attempt(1),
            status: configured.clone(),
        }),
        configured_transition(target, configured)
    );
}

#[test]
fn timeout_then_correlated_definite_failure_unlocks_retry_but_stale_id_does_not() {
    for (result, reason) in [
        (
            ApiKeyPersistenceResult::Rejected,
            ProviderAuthFailureReason::PersistenceRejected,
        ),
        (
            ApiKeyPersistenceResult::StorageUnavailable,
            ProviderAuthFailureReason::StorageUnavailable,
        ),
    ] {
        let target = target();
        let flow = flow(target.clone(), ApiKeyFlowIntent::Add);
        let mut controller = submitted(target);
        controller.dispatch(ProviderAuthAction::TimeoutElapsed {
            attempt_id: attempt(1),
        });
        assert_eq!(
            controller.dispatch(ProviderAuthAction::PersistenceFinished {
                attempt_id: attempt(2),
                result: ApiKeyPersistenceResult::Stored,
            }),
            transition(
                ProviderAuthFlowSnapshot::OutcomeUnknown {
                    flow: flow.clone(),
                    attempt_id: attempt(1),
                },
                vec![],
                ProviderAuthDisposition::IgnoredStale,
            )
        );
        assert_eq!(
            controller.dispatch(ProviderAuthAction::PersistenceFinished {
                attempt_id: attempt(1),
                result,
            }),
            transition(
                ProviderAuthFlowSnapshot::Failed { flow, reason },
                vec![],
                ProviderAuthDisposition::Applied,
            )
        );
        assert_eq!(
            controller.dispatch(ProviderAuthAction::Retry).disposition,
            ProviderAuthDisposition::Applied
        );
    }
}

#[test]
fn reconciling_transient_status_remains_nonretryable() {
    let target = target();
    let mut controller = submitted(target.clone());
    controller.dispatch(ProviderAuthAction::PersistenceFinished {
        attempt_id: attempt(1),
        result: ApiKeyPersistenceResult::Stored,
    });
    for configuration in [
        ProviderConfigurationState::Checking,
        ProviderConfigurationState::Unavailable,
        ProviderConfigurationState::RecoveryRequired,
    ] {
        assert_eq!(
            controller.dispatch(ProviderAuthAction::StatusResolved {
                attempt_id: attempt(1),
                status: status(&target, configuration),
            }),
            waiting_reconciliation(target.clone())
        );
        assert_eq!(
            controller.dispatch(ProviderAuthAction::Retry).disposition,
            ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
        );
    }
}

#[test]
fn definite_failure_allows_retry_while_old_attempt_results_are_stale() {
    let target = target();
    let mut controller = submitted(target.clone());
    controller.dispatch(ProviderAuthAction::PersistenceFinished {
        attempt_id: attempt(1),
        result: ApiKeyPersistenceResult::Rejected,
    });
    controller.dispatch(ProviderAuthAction::Retry);
    controller.dispatch(ProviderAuthAction::SetApiKey(ApiKeySecret::new(
        "replacement",
    )));
    controller.dispatch(ProviderAuthAction::Submit);
    assert_eq!(
        controller.dispatch(ProviderAuthAction::PersistenceFinished {
            attempt_id: attempt(1),
            result: ApiKeyPersistenceResult::Stored,
        }),
        transition(
            ProviderAuthFlowSnapshot::Settling {
                flow: flow(target, ApiKeyFlowIntent::Add),
                attempt_id: attempt(2),
            },
            vec![],
            ProviderAuthDisposition::IgnoredStale,
        )
    );
}

fn entering(target: ApiKeyAuthTarget, intent: ApiKeyFlowIntent) -> ProviderAuthController {
    let mut controller = ProviderAuthController::default();
    controller.dispatch(start(target, intent, missing()));
    controller
}

fn submitted(target: ApiKeyAuthTarget) -> ProviderAuthController {
    submitted_with_intent(target, ApiKeyFlowIntent::Add)
}

fn submitted_with_intent(
    target: ApiKeyAuthTarget,
    intent: ApiKeyFlowIntent,
) -> ProviderAuthController {
    let mut controller = entering(target, intent);
    controller.dispatch(ProviderAuthAction::SetApiKey(ApiKeySecret::new("secret")));
    controller.dispatch(ProviderAuthAction::Submit);
    controller
}

fn start(
    target: ApiKeyAuthTarget,
    intent: ApiKeyFlowIntent,
    metadata: ApiKeyCredentialMetadata,
) -> ProviderAuthAction {
    ProviderAuthAction::StartApiKey(ApiKeyFlowStart {
        target,
        intent,
        metadata,
    })
}

fn transition(
    snapshot: ProviderAuthFlowSnapshot,
    effects: Vec<ProviderAuthEffect>,
    disposition: ProviderAuthDisposition,
) -> ProviderAuthTransition {
    ProviderAuthTransition {
        snapshot,
        effects,
        disposition,
    }
}

fn stored_reconciliation(
    target: ApiKeyAuthTarget,
    intent: ApiKeyFlowIntent,
) -> ProviderAuthTransition {
    transition(
        ProviderAuthFlowSnapshot::Reconciling {
            flow: flow(target.clone(), intent),
            attempt_id: attempt(1),
        },
        vec![ProviderAuthEffect::RefreshProviderStatus {
            attempt_id: attempt(1),
            target,
        }],
        ProviderAuthDisposition::Applied,
    )
}

fn configured_transition(
    target: ApiKeyAuthTarget,
    status: ProviderStatusSnapshot,
) -> ProviderAuthTransition {
    transition(
        ProviderAuthFlowSnapshot::Configured {
            target: target.clone(),
            status: status.clone(),
        },
        vec![ProviderAuthEffect::Complete(
            ProviderAuthCompletion::Configured { target, status },
        )],
        ProviderAuthDisposition::Applied,
    )
}

fn waiting_reconciliation(target: ApiKeyAuthTarget) -> ProviderAuthTransition {
    transition(
        reconciliation_snapshot(target),
        vec![],
        ProviderAuthDisposition::Applied,
    )
}

fn reconciliation_snapshot(target: ApiKeyAuthTarget) -> ProviderAuthFlowSnapshot {
    ProviderAuthFlowSnapshot::Reconciling {
        flow: flow(target, ApiKeyFlowIntent::Add),
        attempt_id: attempt(1),
    }
}

fn attempt(id: u64) -> ProviderAuthAttemptId {
    ProviderAuthAttemptId(id)
}

fn flow(target: ApiKeyAuthTarget, intent: ApiKeyFlowIntent) -> ApiKeyFlowContext {
    ApiKeyFlowContext { target, intent }
}

fn target() -> ApiKeyAuthTarget {
    ApiKeyAuthTarget {
        provider_id: ProviderCatalogId("custom".into()),
        runtime_provider_id: ProviderRuntimeId("custom".into()),
        storage: ApiKeyStorage::EnvironmentVariable {
            env_key: "CUSTOM_KEY".into(),
        },
    }
}

fn missing() -> ApiKeyCredentialMetadata {
    ApiKeyCredentialMetadata {
        environment: EnvironmentCredentialMetadata::Missing,
        managed: ManagedApiKeyMetadata::Missing,
    }
}

fn status(
    target: &ApiKeyAuthTarget,
    configuration: ProviderConfigurationState,
) -> ProviderStatusSnapshot {
    ProviderStatusSnapshot {
        id: target.provider_id.clone(),
        methods: vec![],
        configuration,
        eligibility: ProviderEligibilityState::Active,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Ready,
    }
}
