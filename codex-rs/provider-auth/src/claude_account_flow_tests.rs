use pretty_assertions::assert_eq;

use super::*;
use crate::*;

#[test]
fn target_add_short_circuit_environment_block_and_restart_are_typed() {
    let entry = claude_entry();
    assert_eq!(
        ClaudeAccountTarget::from_catalog_entry(&entry),
        Ok(target())
    );
    let wrong = ProviderCatalogEntry {
        id: ProviderCatalogId("wrong".into()),
        display_name: "Wrong".into(),
        runtime_provider_ids: vec![ProviderRuntimeId("wrong".into())],
        setup_capabilities: ProviderSetupCapabilities::one(ProviderSetupCapability::StatusOnly {
            reason: StatusOnlyReason::NoInteractiveSetup,
        }),
    };
    assert_eq!(
        ClaudeAccountTarget::from_catalog_entry(&wrong),
        Err(ClaudeAccountTargetError::UnsupportedCapability)
    );

    let configured = status(Some(ProviderCredentialSource::ClaudeCodeLogin));
    let mut controller = ProviderAuthController::default();
    assert_eq!(
        controller.dispatch(start(ClaudeAccountIntent::Add, configured.clone())),
        transition(
            ClaudeAccountSnapshot::Configured {
                target: target(),
                status: configured.clone(),
            },
            vec![ProviderAuthEffect::Complete(
                ProviderAuthCompletion::ClaudeAccount(ClaudeAccountCompletion::Configured {
                    target: target(),
                    status: configured,
                })
            )]
        )
    );

    let mut restarted = ProviderAuthController::default();
    assert_eq!(restarted.snapshot(), &ProviderAuthFlowSnapshot::Idle);
    assert!(matches!(
        restarted
            .dispatch(start(
                unauthorized(ClaudeUnauthorizedRecoverySource::Environment),
                status(Some(ProviderCredentialSource::ClaudeEnvironment)),
            ))
            .snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Blocked {
            reason: ClaudeAccountBlockedReason::ExternallyManagedEnvironment,
            ..
        })
    ));
}

#[test]
fn unauthorized_recovery_enforces_exact_selected_source_without_fallback() {
    for (source, allowed, rejected_method) in [
        (
            ClaudeUnauthorizedRecoverySource::ManagedToken,
            ClaudeAccountMethod::ManagedToken,
            ClaudeAccountMethod::ClaudeCodeLogin,
        ),
        (
            ClaudeUnauthorizedRecoverySource::ClaudeCodeLogin,
            ClaudeAccountMethod::ClaudeCodeLogin,
            ClaudeAccountMethod::ManagedToken,
        ),
    ] {
        let mut controller = ProviderAuthController::default();
        let started = controller.dispatch(start(unauthorized(source), status(None)));
        assert!(matches!(
            started.snapshot,
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::ChoosingMethod {
                recommended: Some(method),
                ..
            }) if method == allowed
        ));
        assert_eq!(
            controller
                .dispatch(ClaudeAccountAction::ChooseMethod(rejected_method).into())
                .disposition,
            ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::InvalidState)
        );
        assert_eq!(
            controller
                .dispatch(ClaudeAccountAction::ChooseMethod(allowed).into())
                .disposition,
            ProviderAuthDisposition::Applied
        );
    }

    for (source, expected) in [
        (
            ClaudeUnauthorizedRecoverySource::Environment,
            EitherRecovery::Blocked,
        ),
        (
            ClaudeUnauthorizedRecoverySource::Unknown,
            EitherRecovery::MissingSelection,
        ),
    ] {
        let mut controller = ProviderAuthController::default();
        let transition = controller.dispatch(start(unauthorized(source), status(None)));
        assert_eq!(
            match transition.snapshot {
                ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Blocked {
                    reason: ClaudeAccountBlockedReason::ExternallyManagedEnvironment,
                    ..
                }) => EitherRecovery::Blocked,
                ProviderAuthFlowSnapshot::ClaudeAccount(
                    ClaudeAccountSnapshot::RecoveryRequired {
                        reason: ClaudeAccountRecoveryReason::MissingSelection,
                        ..
                    },
                ) => EitherRecovery::MissingSelection,
                other => panic!("unexpected recovery state: {other:?}"),
            },
            expected
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EitherRecovery {
    Blocked,
    MissingSelection,
}

#[test]
fn managed_submit_is_redacted_non_cancellable_and_reconciles_late_settlement() {
    let (mut controller, flow) = choosing(ClaudeAccountIntent::Replace);
    controller
        .dispatch(ClaudeAccountAction::ChooseMethod(ClaudeAccountMethod::ManagedToken).into());
    let secret_canary = "sk-ant-oat01-MANAGED-CANARY";
    let set = controller.dispatch(
        ClaudeAccountAction::SetManagedToken(ClaudeManagedTokenSecret::new(secret_canary)).into(),
    );
    assert_eq!(
        set.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::EnteringManagedToken {
            flow,
            has_input: true,
        })
    );
    assert!(!format!("{set:?}").contains(secret_canary));

    let submitted = controller.dispatch(ClaudeAccountAction::Submit.into());
    let (attempt_id, effect_debug) = match submitted.effects.as_slice() {
        [
            ProviderAuthEffect::ClaudeAccount(ClaudeAccountEffect::EnrollManagedToken {
                attempt_id,
                ..
            }),
            ProviderAuthEffect::ClaudeAccount(ClaudeAccountEffect::ScheduleManagedTimeout {
                attempt_id: timeout_attempt,
                timeout,
            }),
        ] => {
            assert_eq!(attempt_id, timeout_attempt);
            assert_eq!(*timeout, CLAUDE_MANAGED_AUTH_TIMEOUT);
            (*attempt_id, format!("{:?}", submitted.effects))
        }
        other => panic!("unexpected managed effects: {other:?}"),
    };
    assert!(!effect_debug.contains(secret_canary));
    assert_eq!(
        controller
            .dispatch(ClaudeAccountAction::Cancel.into())
            .disposition,
        ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
    );

    let timed_out =
        controller.dispatch(ClaudeAccountAction::ManagedTimeoutElapsed { attempt_id }.into());
    assert!(matches!(
        timed_out.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
            method: ClaudeAccountMethod::ManagedToken,
            ..
        })
    ));
    let old_status = controller.dispatch(
        ClaudeAccountAction::StatusResolved {
            attempt_id,
            status: status(Some(ProviderCredentialSource::ClaudeManaged)),
        }
        .into(),
    );
    assert!(matches!(
        old_status.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown { .. })
    ));

    let settled = controller.dispatch(
        ClaudeAccountAction::ManagedTokenFinished {
            attempt_id,
            result: ClaudeManagedTokenResult::Stored,
        }
        .into(),
    );
    assert!(matches!(
        settled.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Reconciling { .. })
    ));
    assert!(matches!(
        controller
            .dispatch(
                ClaudeAccountAction::StatusResolved {
                    attempt_id,
                    status: status(Some(ProviderCredentialSource::ClaudeManaged)),
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Configured { .. })
    ));
}

#[test]
fn correlated_managed_success_with_nonmatching_status_fails_terminally() {
    let (mut controller, _) = choosing(ClaudeAccountIntent::Replace);
    controller
        .dispatch(ClaudeAccountAction::ChooseMethod(ClaudeAccountMethod::ManagedToken).into());
    controller.dispatch(
        ClaudeAccountAction::SetManagedToken(ClaudeManagedTokenSecret::new(
            "sk-ant-oat01-correlated",
        ))
        .into(),
    );
    let submitted = controller.dispatch(ClaudeAccountAction::Submit.into());
    let attempt_id = managed_attempt(&submitted);
    controller.dispatch(
        ClaudeAccountAction::ManagedTokenFinished {
            attempt_id,
            result: ClaudeManagedTokenResult::Stored,
        }
        .into(),
    );
    controller.dispatch(
        ClaudeAccountAction::BackendTransportLost {
            attempt_id,
            process_id: None,
        }
        .into(),
    );

    let resolved = controller.dispatch(
        ClaudeAccountAction::StatusResolved {
            attempt_id,
            status: status(Some(ProviderCredentialSource::ClaudeCodeLogin)),
        }
        .into(),
    );
    assert!(matches!(
        resolved.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Failed {
            reason: ClaudeAccountFailureReason::StatusNotConfigured,
            ..
        })
    ));
}

#[test]
fn add_timeout_can_be_proved_by_status_but_replace_cannot() {
    for intent in [ClaudeAccountIntent::Add, ClaudeAccountIntent::Replace] {
        let (mut controller, _) = choosing(intent);
        controller
            .dispatch(ClaudeAccountAction::ChooseMethod(ClaudeAccountMethod::ManagedToken).into());
        controller.dispatch(
            ClaudeAccountAction::SetManagedToken(ClaudeManagedTokenSecret::new(
                "sk-ant-oat01-adjacent",
            ))
            .into(),
        );
        let submitted = controller.dispatch(ClaudeAccountAction::Submit.into());
        let attempt_id = managed_attempt(&submitted);
        controller.dispatch(ClaudeAccountAction::ManagedTimeoutElapsed { attempt_id }.into());
        let resolved = controller.dispatch(
            ClaudeAccountAction::StatusResolved {
                attempt_id,
                status: status(Some(ProviderCredentialSource::ClaudeManaged)),
            }
            .into(),
        );
        assert_eq!(
            matches!(
                resolved.snapshot,
                ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Configured { .. })
            ),
            intent == ClaudeAccountIntent::Add
        );
    }
}

#[test]
fn code_login_freezes_identity_policy_challenge_code_cancel_and_late_events() {
    let (mut controller, flow, attempt_id, process_id) = started_code(
        unauthorized(ClaudeUnauthorizedRecoverySource::ClaudeCodeLogin),
        status(Some(ProviderCredentialSource::ClaudeCodeLogin)),
    );
    let start_effect = controller.snapshot().clone();
    assert_eq!(
        start_effect,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::StartingClaudeCodeLogin {
            flow,
            attempt_id,
            process_id,
        })
    );

    let url_canary = "https://claude.example/CANARY-URL";
    let ready = controller.dispatch(
        ClaudeAccountAction::ClaudeCodeReady {
            attempt_id,
            process_id,
            challenge: ClaudeCodeChallenge::new(url_canary),
        }
        .into(),
    );
    assert!(matches!(
        ready.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(
            ClaudeAccountSnapshot::AwaitingAuthorizationCode { .. }
        )
    ));
    assert!(!format!("{ready:?}").contains(url_canary));

    let code_canary = "CLAUDE-CODE-CANARY";
    let submitted = controller.dispatch(
        ClaudeAccountAction::SubmitAuthorizationCode(ClaudeAuthorizationCodeSecret::new(
            code_canary,
        ))
        .into(),
    );
    assert!(!format!("{submitted:?}").contains(code_canary));
    let cancel_after_commit = controller.dispatch(ClaudeAccountAction::Cancel.into());
    assert_eq!(
        cancel_after_commit.disposition,
        ProviderAuthDisposition::Rejected(ProviderAuthRejectionReason::CommitInProgress)
    );
    assert!(cancel_after_commit.effects.is_empty());
    let settled = controller.dispatch(
        ClaudeAccountAction::ClaudeCodeFinished {
            attempt_id,
            process_id,
            outcome: ClaudeCodeLoginOutcome::Succeeded,
        }
        .into(),
    );
    assert!(matches!(
        settled.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Reconciling { .. })
    ));
    assert!(matches!(
        controller
            .dispatch(
                ClaudeAccountAction::StatusResolved {
                    attempt_id,
                    status: status(Some(ProviderCredentialSource::ClaudeCodeLogin)),
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Configured { .. })
    ));
}

#[test]
fn code_transport_loss_preserves_cancel_and_correlated_success_origins() {
    let (mut cancelling, _, attempt_id, process_id) = started_code(
        ClaudeAccountIntent::Replace,
        status(Some(ProviderCredentialSource::ClaudeCodeLogin)),
    );
    cancelling.dispatch(ClaudeAccountAction::Cancel.into());
    cancelling.dispatch(
        ClaudeAccountAction::BackendTransportLost {
            attempt_id,
            process_id: Some(process_id),
        }
        .into(),
    );
    assert!(matches!(
        cancelling
            .dispatch(
                ClaudeAccountAction::ClaudeCodeFinished {
                    attempt_id,
                    process_id,
                    outcome: ClaudeCodeLoginOutcome::Cancelled,
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Cancelled { .. })
    ));

    let (mut succeeded, _, attempt_id, process_id) = started_code(
        ClaudeAccountIntent::Replace,
        status(Some(ProviderCredentialSource::ClaudeManaged)),
    );
    succeeded.dispatch(
        ClaudeAccountAction::ClaudeCodeFinished {
            attempt_id,
            process_id,
            outcome: ClaudeCodeLoginOutcome::Succeeded,
        }
        .into(),
    );
    succeeded.dispatch(
        ClaudeAccountAction::BackendTransportLost {
            attempt_id,
            process_id: Some(process_id),
        }
        .into(),
    );
    assert!(matches!(
        succeeded
            .dispatch(
                ClaudeAccountAction::StatusResolved {
                    attempt_id,
                    status: status(Some(ProviderCredentialSource::ClaudeCodeLogin)),
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Configured { .. })
    ));
}

#[test]
fn unproven_code_transport_and_stale_ready_have_typed_terminal_cleanup() {
    let (mut controller, _, attempt_id, process_id) = started_code(
        ClaudeAccountIntent::Replace,
        status(Some(ProviderCredentialSource::ClaudeManaged)),
    );
    let stale_process = ClaudeCodeProcessId(process_id.0 + 1);
    let stale = controller.dispatch(
        ClaudeAccountAction::ClaudeCodeReady {
            attempt_id,
            process_id: stale_process,
            challenge: ClaudeCodeChallenge::new("https://stale.example/canary"),
        }
        .into(),
    );
    assert_eq!(stale.disposition, ProviderAuthDisposition::IgnoredStale);
    assert_eq!(
        stale.effects,
        vec![ProviderAuthEffect::ClaudeAccount(
            ClaudeAccountEffect::CancelClaudeCodeLogin {
                attempt_id,
                process_id: stale_process,
            }
        )]
    );

    controller.dispatch(
        ClaudeAccountAction::BackendTransportLost {
            attempt_id,
            process_id: Some(process_id),
        }
        .into(),
    );
    assert!(matches!(
        controller
            .dispatch(
                ClaudeAccountAction::StatusResolved {
                    attempt_id,
                    status: status(Some(ProviderCredentialSource::ClaudeManaged)),
                }
                .into(),
            )
            .snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::RecoveryRequired {
            reason: ClaudeAccountRecoveryReason::LoginOutcomeUnknown,
            ..
        })
    ));
}

#[test]
fn correlated_success_requires_exact_method_status_and_rejects_stale_ids() {
    let (mut controller, _, attempt_id, process_id) = started_code(
        ClaudeAccountIntent::Replace,
        status(Some(ProviderCredentialSource::ClaudeManaged)),
    );
    assert_eq!(
        controller
            .dispatch(
                ClaudeAccountAction::ClaudeCodeFinished {
                    attempt_id,
                    process_id: ClaudeCodeProcessId(process_id.0 + 1),
                    outcome: ClaudeCodeLoginOutcome::Succeeded,
                }
                .into(),
            )
            .disposition,
        ProviderAuthDisposition::IgnoredStale
    );
    controller.dispatch(
        ClaudeAccountAction::ClaudeCodeFinished {
            attempt_id,
            process_id,
            outcome: ClaudeCodeLoginOutcome::Succeeded,
        }
        .into(),
    );
    let wrong = controller.dispatch(
        ClaudeAccountAction::StatusResolved {
            attempt_id,
            status: status(Some(ProviderCredentialSource::ClaudeManaged)),
        }
        .into(),
    );
    assert!(matches!(
        wrong.snapshot,
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Failed {
            reason: ClaudeAccountFailureReason::StatusNotConfigured,
            ..
        })
    ));
}

fn choosing(intent: ClaudeAccountIntent) -> (ProviderAuthController, ClaudeAccountFlow) {
    let mut controller = ProviderAuthController::default();
    let transition = controller.dispatch(start(intent, status(None)));
    let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::ChoosingMethod {
        flow, ..
    }) = transition.snapshot
    else {
        panic!("unexpected start: {:?}", transition.snapshot);
    };
    (controller, flow)
}

fn started_code(
    intent: ClaudeAccountIntent,
    initial_status: ProviderStatusSnapshot,
) -> (
    ProviderAuthController,
    ClaudeAccountFlow,
    ProviderAuthAttemptId,
    ClaudeCodeProcessId,
) {
    let mut controller = ProviderAuthController::default();
    controller.dispatch(start(intent, initial_status));
    controller
        .dispatch(ClaudeAccountAction::ChooseMethod(ClaudeAccountMethod::ClaudeCodeLogin).into());
    let mut submitted = controller.dispatch(ClaudeAccountAction::Submit.into());
    if let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::CheckingExistingLogin {
        attempt_id,
        ..
    }) = submitted.snapshot
    {
        submitted = controller.dispatch(
            ClaudeAccountAction::ExistingLoginChecked {
                attempt_id,
                result: ClaudeExistingLoginResult::LoginRequired,
            }
            .into(),
        );
    }
    let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::StartingClaudeCodeLogin {
        flow,
        attempt_id,
        process_id,
    }) = submitted.snapshot
    else {
        panic!("unexpected code start: {:?}", submitted.snapshot);
    };
    let [
        ProviderAuthEffect::ClaudeAccount(ClaudeAccountEffect::StartClaudeCodeLogin {
            identity_policy,
            ..
        }),
    ] = submitted.effects.as_slice()
    else {
        panic!("unexpected code effect: {:?}", submitted.effects);
    };
    assert_eq!(
        *identity_policy,
        if matches!(intent, ClaudeAccountIntent::UnauthorizedRecovery { .. }) {
            ClaudeCodeIdentityPolicy::PreserveSelected
        } else {
            ClaudeCodeIdentityPolicy::AllowExplicitChange
        }
    );
    (controller, flow, attempt_id, process_id)
}

fn unauthorized(source: ClaudeUnauthorizedRecoverySource) -> ClaudeAccountIntent {
    ClaudeAccountIntent::UnauthorizedRecovery { source }
}

fn managed_attempt(transition: &ProviderAuthTransition) -> ProviderAuthAttemptId {
    match &transition.snapshot {
        ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::SettlingManagedToken {
            attempt_id,
            ..
        }) => *attempt_id,
        other => panic!("unexpected managed state: {other:?}"),
    }
}

fn start(intent: ClaudeAccountIntent, status: ProviderStatusSnapshot) -> ProviderAuthAction {
    ClaudeAccountAction::Start(ClaudeAccountFlowStart {
        target: target(),
        intent,
        status,
    })
    .into()
}

fn status(source: Option<ProviderCredentialSource>) -> ProviderStatusSnapshot {
    ProviderStatusSnapshot {
        id: target().provider_id,
        methods: vec![ProviderMethodStatus {
            capability: ProviderSetupCapability::ClaudeAccount,
            state: source.map_or(ProviderMethodState::NotConfigured, |source| {
                ProviderMethodState::Configured {
                    source,
                    control: match source {
                        ProviderCredentialSource::ClaudeManaged => {
                            CredentialControl::ManagedByCorbanu
                        }
                        ProviderCredentialSource::ClaudeEnvironment => {
                            CredentialControl::ExternalEnvironment
                        }
                        _ => CredentialControl::ExternalProvider,
                    },
                    availability: ConfiguredAvailability::Ready,
                }
            }),
        }],
        configuration: if source.is_some() {
            ProviderConfigurationState::Configured
        } else {
            ProviderConfigurationState::NotConfigured
        },
        eligibility: ProviderEligibilityState::Active,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Ready,
    }
}

fn claude_entry() -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: target().provider_id,
        display_name: "Claude Account".into(),
        runtime_provider_ids: vec![target().runtime_provider_id],
        setup_capabilities: ProviderSetupCapabilities::one(ProviderSetupCapability::ClaudeAccount),
    }
}

fn target() -> ClaudeAccountTarget {
    ClaudeAccountTarget {
        provider_id: ProviderCatalogId("claude-plan".into()),
        runtime_provider_id: ProviderRuntimeId("claude-plan".into()),
    }
}

fn transition(
    snapshot: ClaudeAccountSnapshot,
    effects: Vec<ProviderAuthEffect>,
) -> ProviderAuthTransition {
    ProviderAuthTransition {
        snapshot: ProviderAuthFlowSnapshot::ClaudeAccount(snapshot),
        effects,
        disposition: ProviderAuthDisposition::Applied,
    }
}
