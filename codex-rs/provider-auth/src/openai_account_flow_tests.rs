use pretty_assertions::assert_eq;

use super::*;
use crate::*;

#[test]
fn settled_openai_failures_offer_working_retry_and_cancel() {
    for recovery in [false, true] {
        for retry in [false, true] {
            let (mut controller, attempt_id, _) = started(
                OpenAiAccountMethod::DeviceCode,
                OpenAiAccountLoginContext::ProviderEnrollment,
                status(AccountState::NotConfigured),
            );
            controller.dispatch(
                OpenAiAccountAction::StartFinished {
                    attempt_id,
                    result: if recovery {
                        OpenAiAccountStartResult::TransportLost
                    } else {
                        OpenAiAccountStartResult::Rejected
                    },
                }
                .into(),
            );
            if recovery {
                controller.dispatch(
                    OpenAiAccountAction::StatusResolved {
                        attempt_id,
                        status: status(AccountState::NotConfigured),
                    }
                    .into(),
                );
            }
            let result = controller.dispatch(
                if retry {
                    OpenAiAccountAction::Retry
                } else {
                    OpenAiAccountAction::Cancel
                }
                .into(),
            );
            if retry {
                assert!(
                    matches!(result.snapshot, ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Starting { attempt_id: next, .. }) if next != attempt_id)
                );
            } else {
                assert!(matches!(
                    result.effects.as_slice(),
                    [ProviderAuthEffect::Complete(
                        ProviderAuthCompletion::OpenAiAccount(
                            OpenAiAccountCompletion::Cancelled { .. }
                        )
                    )]
                ));
            }
        }
    }
}

#[test]
fn account_target_and_start_policy_freeze_account_specific_capability() {
    let entry = openai_entry();
    assert_eq!(
        OpenAiAccountTarget::from_catalog_entry(&entry),
        Ok(target())
    );
    let api_key_only_entry = ProviderCatalogEntry {
        id: ProviderCatalogId("custom".into()),
        display_name: "Custom".into(),
        runtime_provider_ids: vec![ProviderRuntimeId("custom".into())],
        setup_capabilities: ProviderSetupCapabilities::one(ProviderSetupCapability::ApiKey {
            storage: ApiKeyStorage::EnvironmentVariable {
                env_key: "CUSTOM_KEY".into(),
            },
        }),
    };
    assert_eq!(
        OpenAiAccountTarget::from_catalog_entry(&api_key_only_entry),
        Err(OpenAiAccountTargetError::UnsupportedCapability)
    );

    let mut controller = ProviderAuthController::default();
    let transition = controller.dispatch(start_action(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::ProviderEnrollment,
        status(AccountState::NotConfigured),
    ));
    assert_eq!(
        transition.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Blocked {
            flow: flow(
                OpenAiAccountMethod::Browser,
                OpenAiAccountLoginContext::ProviderEnrollment
            ),
            reason: OpenAiAccountBlockedReason::BrowserUnavailableForProviderEnrollment,
        })
    );
    assert!(transition.effects.is_empty());
}

#[test]
fn account_method_alternative_controls_short_circuit_not_api_key() {
    let configured = status(AccountState::Account);
    let mut controller = ProviderAuthController::default();
    assert_eq!(
        controller.dispatch(start_action(
            OpenAiAccountMethod::Browser,
            OpenAiAccountLoginContext::PrimaryAuth,
            configured.clone(),
        )),
        transition(
            ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Configured {
                target: target(),
                status: configured.clone(),
            }),
            vec![ProviderAuthEffect::Complete(
                ProviderAuthCompletion::OpenAiAccount(OpenAiAccountCompletion::Configured {
                    target: target(),
                    status: configured,
                })
            )],
        )
    );

    let (api_key_controller, _, api_key_start) = started(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::ApiKeyOnly),
    );
    assert!(matches!(
        api_key_start.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Starting { .. })
    ));
    drop(api_key_controller);

    let mut external = ProviderAuthController::default();
    assert!(matches!(
        external
            .dispatch(start_action(
                OpenAiAccountMethod::DeviceCode,
                OpenAiAccountLoginContext::PrimaryAuth,
                status(AccountState::External),
            ))
            .snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Blocked {
            reason: OpenAiAccountBlockedReason::ExternallyManaged,
            ..
        })
    ));
}

#[test]
fn browser_and_device_challenges_are_ephemeral_and_enclosing_debug_is_redacted() {
    for (method, challenge) in [
        (
            OpenAiAccountMethod::Browser,
            OpenAiAccountChallenge::browser("https://canary.example/secret"),
        ),
        (
            OpenAiAccountMethod::DeviceCode,
            OpenAiAccountChallenge::device_code("https://canary.example/device", "CANARY-CODE"),
        ),
    ] {
        let (mut controller, attempt_id, _) = started(
            method,
            OpenAiAccountLoginContext::PrimaryAuth,
            status(AccountState::NotConfigured),
        );
        let action = OpenAiAccountAction::StartFinished {
            attempt_id,
            result: OpenAiAccountStartResult::Started {
                login_id: login("login-1"),
                challenge,
            },
        };
        assert!(!format!("{action:?}").contains("canary"));
        let transition = controller.dispatch(action.into());
        assert!(matches!(
            transition.snapshot,
            ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::AwaitingUser { .. })
        ));
        let debug = format!("{transition:?}");
        assert!(!debug.contains("canary"));
        assert!(!debug.contains("CANARY-CODE"));
    }
}

#[test]
fn cancel_before_start_dispatches_late_cancel_without_presenting_challenge() {
    let (mut controller, attempt_id, _) = started(
        OpenAiAccountMethod::DeviceCode,
        OpenAiAccountLoginContext::ProviderEnrollment,
        status(AccountState::NotConfigured),
    );
    controller.dispatch(OpenAiAccountAction::Cancel.into());
    assert_eq!(
        controller.dispatch(
            OpenAiAccountAction::StartFinished {
                attempt_id,
                result: OpenAiAccountStartResult::Started {
                    login_id: login("late-login"),
                    challenge: OpenAiAccountChallenge::device_code("https://example.test", "CODE",),
                },
            }
            .into(),
        ),
        transition(
            ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Cancelling {
                flow: flow(
                    OpenAiAccountMethod::DeviceCode,
                    OpenAiAccountLoginContext::ProviderEnrollment,
                ),
                attempt_id,
                login_id: login("late-login"),
                purpose: OpenAiAccountCancelPurpose::UserRequested,
            }),
            vec![ProviderAuthEffect::OpenAiAccount(
                OpenAiAccountEffect::CancelLogin {
                    attempt_id,
                    login_id: login("late-login"),
                }
            )],
        )
    );
}

#[test]
fn cancel_before_start_cancels_even_when_late_challenge_variant_is_unexpected() {
    let (mut controller, attempt_id, _) = started(
        OpenAiAccountMethod::DeviceCode,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::NotConfigured),
    );
    controller.dispatch(OpenAiAccountAction::Cancel.into());
    let transition = controller.dispatch(
        OpenAiAccountAction::StartFinished {
            attempt_id,
            result: OpenAiAccountStartResult::Started {
                login_id: login("late-login"),
                challenge: OpenAiAccountChallenge::browser("https://unexpected.example"),
            },
        }
        .into(),
    );
    assert!(matches!(
        transition.effects.as_slice(),
        [ProviderAuthEffect::OpenAiAccount(
            OpenAiAccountEffect::CancelLogin { login_id, .. }
        )] if login_id == &login("late-login")
    ));
    assert!(matches!(
        transition.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Cancelling { .. })
    ));
}

#[test]
fn cancel_not_found_stays_unknown_and_accepts_correlated_late_completion() {
    let (mut controller, attempt_id) = awaiting_user();
    controller.dispatch(OpenAiAccountAction::Cancel.into());
    assert!(matches!(
        controller
            .dispatch(
                OpenAiAccountAction::CancelFinished {
                    attempt_id,
                    result: OpenAiCancelResult::NotFound,
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::OutcomeUnknown {
            reason: OpenAiAccountOutcomeUnknownReason::CancelNotFound,
            ..
        })
    ));
    assert_eq!(
        controller
            .dispatch(OpenAiAccountAction::Retry.into())
            .disposition,
        ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
    );
    let late = controller.dispatch(
        OpenAiAccountAction::LoginCompleted {
            attempt_id,
            login_id: login("login-1"),
            outcome: OpenAiAccountLoginOutcome::Succeeded,
        }
        .into(),
    );
    assert!(matches!(
        late.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Reconciling { .. })
    ));
    assert!(matches!(
        late.effects.as_slice(),
        [ProviderAuthEffect::OpenAiAccount(
            OpenAiAccountEffect::RefreshStatus { .. }
        )]
    ));
}

#[test]
fn success_requires_account_status_and_rejects_stale_dual_correlation() {
    let (mut controller, attempt_id) = awaiting_user();
    assert_eq!(
        controller
            .dispatch(
                OpenAiAccountAction::LoginCompleted {
                    attempt_id,
                    login_id: login("wrong"),
                    outcome: OpenAiAccountLoginOutcome::Succeeded,
                }
                .into(),
            )
            .disposition,
        ProviderAuthDisposition::IgnoredStale
    );
    controller.dispatch(
        OpenAiAccountAction::LoginCompleted {
            attempt_id,
            login_id: login("login-1"),
            outcome: OpenAiAccountLoginOutcome::Succeeded,
        }
        .into(),
    );
    let api_key = controller.dispatch(
        OpenAiAccountAction::StatusResolved {
            attempt_id,
            status: status(AccountState::ApiKeyOnly),
        }
        .into(),
    );
    assert!(api_key.effects.is_empty());
    assert!(matches!(
        api_key.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Failed {
            reason: OpenAiAccountFailureReason::StatusNotConfigured,
            ..
        })
    ));

    let (mut controller, attempt_id) = awaiting_user();
    controller.dispatch(
        OpenAiAccountAction::LoginCompleted {
            attempt_id,
            login_id: login("login-1"),
            outcome: OpenAiAccountLoginOutcome::Succeeded,
        }
        .into(),
    );
    let configured = status(AccountState::Account);
    assert!(matches!(
        controller
            .dispatch(
                OpenAiAccountAction::StatusResolved {
                    attempt_id,
                    status: configured,
                }
                .into(),
            )
            .effects
            .as_slice(),
        [ProviderAuthEffect::Complete(
            ProviderAuthCompletion::OpenAiAccount(OpenAiAccountCompletion::Configured { .. })
        )]
    ));
}

#[test]
fn cancel_not_found_plus_nonaccount_status_completes_cancelled() {
    let (mut controller, attempt_id) = awaiting_user();
    controller.dispatch(OpenAiAccountAction::Cancel.into());
    controller.dispatch(
        OpenAiAccountAction::CancelFinished {
            attempt_id,
            result: OpenAiCancelResult::NotFound,
        }
        .into(),
    );
    let transition = controller.dispatch(
        OpenAiAccountAction::StatusResolved {
            attempt_id,
            status: status(AccountState::NotConfigured),
        }
        .into(),
    );
    assert!(matches!(
        transition.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Cancelled { .. })
    ));
    assert!(matches!(
        transition.effects.as_slice(),
        [ProviderAuthEffect::Complete(
            ProviderAuthCompletion::OpenAiAccount(OpenAiAccountCompletion::Cancelled { .. })
        )]
    ));
}

#[test]
fn cancel_transport_loss_preserves_cancel_intent_for_late_failure() {
    let (mut controller, attempt_id) = awaiting_user();
    controller.dispatch(OpenAiAccountAction::Cancel.into());
    assert!(matches!(
        controller
            .dispatch(OpenAiAccountAction::TransportLost { attempt_id }.into())
            .snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::OutcomeUnknown {
            reason: OpenAiAccountOutcomeUnknownReason::CancelTransportLost,
            ..
        })
    ));
    assert!(matches!(
        controller
            .dispatch(
                OpenAiAccountAction::LoginCompleted {
                    attempt_id,
                    login_id: login("login-1"),
                    outcome: OpenAiAccountLoginOutcome::Failed,
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Cancelled { .. })
    ));
}

#[test]
fn wrong_challenge_is_cancelled_before_typed_protocol_failure() {
    let (mut controller, attempt_id, _) = started(
        OpenAiAccountMethod::DeviceCode,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::NotConfigured),
    );
    let transition = controller.dispatch(
        OpenAiAccountAction::StartFinished {
            attempt_id,
            result: OpenAiAccountStartResult::Started {
                login_id: login("wrong-mode"),
                challenge: OpenAiAccountChallenge::browser("https://unexpected.example"),
            },
        }
        .into(),
    );
    assert!(matches!(
        transition.effects.as_slice(),
        [ProviderAuthEffect::OpenAiAccount(OpenAiAccountEffect::CancelLogin {
            login_id,
            ..
        })] if login_id == &login("wrong-mode")
    ));
    assert!(matches!(
        controller
            .dispatch(
                OpenAiAccountAction::CancelFinished {
                    attempt_id,
                    result: OpenAiCancelResult::Canceled,
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Failed {
            reason: OpenAiAccountFailureReason::ProtocolMismatch,
            ..
        })
    ));
}

#[test]
fn transport_loss_is_outcome_unknown_and_restart_reconciles_from_metadata() {
    let (mut controller, attempt_id, _) = started(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::NotConfigured),
    );
    let unknown = controller.dispatch(
        OpenAiAccountAction::StartFinished {
            attempt_id,
            result: OpenAiAccountStartResult::TransportLost,
        }
        .into(),
    );
    assert!(matches!(
        unknown.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::OutcomeUnknown {
            login_id: None,
            reason: OpenAiAccountOutcomeUnknownReason::StartTransportLost,
            ..
        })
    ));
    for action in [OpenAiAccountAction::Retry, OpenAiAccountAction::Cancel] {
        assert_eq!(
            controller.dispatch(action.into()).disposition,
            ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
        );
    }

    let recovered = controller.dispatch(
        OpenAiAccountAction::StatusResolved {
            attempt_id,
            status: status(AccountState::NotConfigured),
        }
        .into(),
    );
    assert!(matches!(
        recovered.snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::RecoveryRequired {
            reason: OpenAiAccountRecoveryReason::StartOutcomeUnknown,
            ..
        })
    ));
    let replacement = controller.dispatch(start_action(
        OpenAiAccountMethod::DeviceCode,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::NotConfigured),
    ));
    let replacement_attempt = match replacement.snapshot {
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Starting {
            attempt_id,
            ..
        }) => attempt_id,
        other => panic!("unexpected replacement state: {other:?}"),
    };
    assert_ne!(attempt_id, replacement_attempt);
    assert_eq!(
        controller
            .dispatch(
                OpenAiAccountAction::StartFinished {
                    attempt_id,
                    result: OpenAiAccountStartResult::Rejected,
                }
                .into(),
            )
            .disposition,
        ProviderAuthDisposition::IgnoredStale
    );

    let mut restarted = ProviderAuthController::default();
    assert!(matches!(
        restarted
            .dispatch(start_action(
                OpenAiAccountMethod::Browser,
                OpenAiAccountLoginContext::PrimaryAuth,
                status(AccountState::Account),
            ))
            .snapshot,
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Configured { .. })
    ));
}

#[test]
fn failed_attempt_retries_monotonically_and_old_results_are_stale() {
    let (mut controller, old_attempt, _) = started(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::NotConfigured),
    );
    controller.dispatch(
        OpenAiAccountAction::StartFinished {
            attempt_id: old_attempt,
            result: OpenAiAccountStartResult::Rejected,
        }
        .into(),
    );
    let retry = controller.dispatch(OpenAiAccountAction::Retry.into());
    let new_attempt = match retry.snapshot {
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Starting {
            attempt_id,
            ..
        }) => attempt_id,
        other => panic!("unexpected retry state: {other:?}"),
    };
    assert_ne!(old_attempt, new_attempt);
    assert_eq!(
        controller
            .dispatch(
                OpenAiAccountAction::StartFinished {
                    attempt_id: old_attempt,
                    result: OpenAiAccountStartResult::Rejected,
                }
                .into(),
            )
            .disposition,
        ProviderAuthDisposition::IgnoredStale
    );
}

fn awaiting_user() -> (ProviderAuthController, ProviderAuthAttemptId) {
    let (mut controller, attempt_id, _) = started(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
        status(AccountState::NotConfigured),
    );
    controller.dispatch(
        OpenAiAccountAction::StartFinished {
            attempt_id,
            result: OpenAiAccountStartResult::Started {
                login_id: login("login-1"),
                challenge: OpenAiAccountChallenge::browser("https://example.test"),
            },
        }
        .into(),
    );
    (controller, attempt_id)
}

fn started(
    method: OpenAiAccountMethod,
    context: OpenAiAccountLoginContext,
    initial_status: ProviderStatusSnapshot,
) -> (
    ProviderAuthController,
    ProviderAuthAttemptId,
    ProviderAuthTransition,
) {
    let mut controller = ProviderAuthController::default();
    let transition = controller.dispatch(start_action(method, context, initial_status));
    let attempt_id = match &transition.snapshot {
        ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Starting {
            attempt_id,
            ..
        }) => *attempt_id,
        other => panic!("unexpected start state: {other:?}"),
    };
    (controller, attempt_id, transition)
}

fn start_action(
    method: OpenAiAccountMethod,
    context: OpenAiAccountLoginContext,
    status: ProviderStatusSnapshot,
) -> ProviderAuthAction {
    OpenAiAccountAction::Start(OpenAiAccountFlowStart {
        target: target(),
        method,
        context,
        status,
    })
    .into()
}

#[derive(Clone, Copy)]
enum AccountState {
    NotConfigured,
    Account,
    ApiKeyOnly,
    External,
}

fn status(state: AccountState) -> ProviderStatusSnapshot {
    let account = match state {
        AccountState::Account => configured(ProviderCredentialSource::OpenAiAccount),
        AccountState::External => configured(ProviderCredentialSource::ExternallyManaged),
        AccountState::NotConfigured | AccountState::ApiKeyOnly => {
            ProviderMethodState::NotConfigured
        }
    };
    let api_key = match state {
        AccountState::ApiKeyOnly => configured(ProviderCredentialSource::OpenAiApiKey),
        _ => ProviderMethodState::NotConfigured,
    };
    ProviderStatusSnapshot {
        id: target().provider_id,
        methods: vec![
            ProviderMethodStatus {
                capability: ProviderSetupCapability::OpenAiAccount,
                state: account,
            },
            ProviderMethodStatus {
                capability: ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::OpenAiAuth,
                },
                state: api_key,
            },
        ],
        configuration: if matches!(state, AccountState::NotConfigured) {
            ProviderConfigurationState::NotConfigured
        } else {
            ProviderConfigurationState::Configured
        },
        eligibility: ProviderEligibilityState::Active,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Ready,
    }
}

fn configured(source: ProviderCredentialSource) -> ProviderMethodState {
    ProviderMethodState::Configured {
        source,
        control: CredentialControl::ManagedByCorbanu,
        availability: ConfiguredAvailability::Ready,
    }
}

fn openai_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: ProviderCatalogId("openai".into()),
        display_name: "OpenAI".into(),
        runtime_provider_ids: vec![ProviderRuntimeId("openai".into())],
        setup_capabilities: ProviderSetupCapabilities {
            primary: ProviderSetupCapability::OpenAiAccount,
            alternatives: vec![ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::OpenAiAuth,
            }],
        },
    }
}

fn target() -> OpenAiAccountTarget {
    OpenAiAccountTarget {
        provider_id: ProviderCatalogId("openai".into()),
        runtime_provider_id: ProviderRuntimeId("openai".into()),
    }
}

fn flow(method: OpenAiAccountMethod, context: OpenAiAccountLoginContext) -> OpenAiAccountFlow {
    OpenAiAccountFlow {
        target: target(),
        method,
        context,
    }
}

fn login(value: &str) -> OpenAiAccountLoginId {
    OpenAiAccountLoginId::new(value)
}

fn transition(
    snapshot: ProviderAuthFlowSnapshot,
    effects: Vec<ProviderAuthEffect>,
) -> ProviderAuthTransition {
    ProviderAuthTransition {
        snapshot,
        effects,
        disposition: ProviderAuthDisposition::Applied,
    }
}
