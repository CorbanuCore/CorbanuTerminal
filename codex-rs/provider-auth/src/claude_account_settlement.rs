use crate::ProviderAuthAttemptId;
use crate::ProviderAuthController;
use crate::ProviderAuthFlowSnapshot;
use crate::auth_flow::Reduction;
use crate::claude_account_controller::applied;
use crate::claude_account_controller::effect;
use crate::claude_account_controller::stale;
use crate::claude_account_flow::*;

impl ProviderAuthController {
    pub(crate) fn claude_managed_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        result: ClaudeManagedTokenResult,
    ) -> Reduction {
        let (flow, current) = match &self.snapshot {
            ProviderAuthFlowSnapshot::ClaudeAccount(
                ClaudeAccountSnapshot::SettlingManagedToken { flow, attempt_id }
                | ClaudeAccountSnapshot::OutcomeUnknown {
                    flow,
                    method: ClaudeAccountMethod::ManagedToken,
                    attempt_id,
                    ..
                },
            ) => (flow.clone(), *attempt_id),
            _ => return stale(),
        };
        if current != attempt_id {
            return stale();
        }
        match result {
            ClaudeManagedTokenResult::Stored => {
                self.reconcile_claude(flow, ClaudeAccountMethod::ManagedToken, attempt_id, None)
            }
            ClaudeManagedTokenResult::Invalid => self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ManagedToken),
                ClaudeAccountFailureReason::InvalidManagedToken,
            ),
            ClaudeManagedTokenResult::StorageUnavailable => self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ManagedToken),
                ClaudeAccountFailureReason::StorageUnavailable,
            ),
        }
    }

    pub(crate) fn claude_managed_timeout(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
    ) -> Reduction {
        let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::SettlingManagedToken {
            flow,
            attempt_id: current,
        }) = &self.snapshot
        else {
            return stale();
        };
        if *current != attempt_id {
            return stale();
        }
        let flow = flow.clone();
        self.set_claude(ClaudeAccountSnapshot::OutcomeUnknown {
            flow: flow.clone(),
            method: ClaudeAccountMethod::ManagedToken,
            attempt_id,
            process_id: None,
            correlated_success: false,
            cancel_requested: false,
        });
        refresh(flow, attempt_id)
    }

    pub(crate) fn claude_existing_checked(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        result: ClaudeExistingLoginResult,
    ) -> Reduction {
        let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::CheckingExistingLogin {
            flow,
            attempt_id: current,
        }) = &self.snapshot
        else {
            return stale();
        };
        if *current != attempt_id {
            return stale();
        }
        let flow = flow.clone();
        match result {
            ClaudeExistingLoginResult::Selected => {
                self.reconcile_claude(flow, ClaudeAccountMethod::ClaudeCodeLogin, attempt_id, None)
            }
            ClaudeExistingLoginResult::LoginRequired => self.start_claude_process(flow, attempt_id),
            ClaudeExistingLoginResult::Unavailable => self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ClaudeCodeLogin),
                ClaudeAccountFailureReason::LoginUnavailable,
            ),
        }
    }

    pub(crate) fn claude_code_ready(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        challenge: ClaudeCodeChallenge,
    ) -> Reduction {
        match &self.snapshot {
            ProviderAuthFlowSnapshot::ClaudeAccount(
                ClaudeAccountSnapshot::StartingClaudeCodeLogin {
                    flow,
                    attempt_id: current_attempt,
                    process_id: current_process,
                },
            ) if *current_attempt == attempt_id && *current_process == process_id => {
                let flow = flow.clone();
                self.set_claude(ClaudeAccountSnapshot::AwaitingAuthorizationCode {
                    flow,
                    attempt_id,
                    process_id,
                });
                effect(ClaudeAccountEffect::PresentChallenge {
                    attempt_id,
                    process_id,
                    challenge,
                })
            }
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Cancelling {
                attempt_id: current_attempt,
                process_id: current_process,
                ..
            }) if *current_attempt == attempt_id && *current_process == process_id => applied(),
            _ => stale_cleanup(attempt_id, process_id),
        }
    }

    pub(crate) fn claude_code_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        outcome: ClaudeCodeLoginOutcome,
    ) -> Reduction {
        let (flow, cancelling) = match &self.snapshot {
            ProviderAuthFlowSnapshot::ClaudeAccount(
                ClaudeAccountSnapshot::StartingClaudeCodeLogin {
                    flow,
                    attempt_id: current_attempt,
                    process_id: current_process,
                }
                | ClaudeAccountSnapshot::AwaitingAuthorizationCode {
                    flow,
                    attempt_id: current_attempt,
                    process_id: current_process,
                }
                | ClaudeAccountSnapshot::Authenticating {
                    flow,
                    attempt_id: current_attempt,
                    process_id: current_process,
                },
            ) if *current_attempt == attempt_id && *current_process == process_id => {
                (flow.clone(), false)
            }
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Cancelling {
                flow,
                attempt_id: current_attempt,
                process_id: current_process,
            }) if *current_attempt == attempt_id && *current_process == process_id => {
                (flow.clone(), true)
            }
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
                flow,
                method: ClaudeAccountMethod::ClaudeCodeLogin,
                attempt_id: current_attempt,
                process_id: Some(current_process),
                cancel_requested,
                ..
            }) if *current_attempt == attempt_id && *current_process == process_id => {
                (flow.clone(), *cancel_requested)
            }
            _ => return stale(),
        };
        if cancelling && outcome == ClaudeCodeLoginOutcome::Cancelled {
            return self.cancelled_claude(flow.target);
        }
        match outcome {
            ClaudeCodeLoginOutcome::Succeeded => self.reconcile_claude(
                flow,
                ClaudeAccountMethod::ClaudeCodeLogin,
                attempt_id,
                Some(process_id),
            ),
            ClaudeCodeLoginOutcome::IdentityConflict => self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ClaudeCodeLogin),
                ClaudeAccountFailureReason::IdentityConflict,
            ),
            ClaudeCodeLoginOutcome::TimedOut => self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ClaudeCodeLogin),
                ClaudeAccountFailureReason::LoginTimedOut,
            ),
            ClaudeCodeLoginOutcome::Cancelled | ClaudeCodeLoginOutcome::Rejected => self
                .fail_claude(
                    flow,
                    Some(ClaudeAccountMethod::ClaudeCodeLogin),
                    ClaudeAccountFailureReason::LoginRejected,
                ),
        }
    }

    pub(crate) fn claude_transport_lost(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        process_id: Option<ClaudeCodeProcessId>,
    ) -> Reduction {
        let Some((flow, method, current_attempt, current_process, proof, cancel_requested)) =
            active_claude(&self.snapshot)
        else {
            return stale();
        };
        if current_attempt != attempt_id || process_id.is_some_and(|id| Some(id) != current_process)
        {
            return stale();
        }
        self.set_claude(ClaudeAccountSnapshot::OutcomeUnknown {
            flow: flow.clone(),
            method,
            attempt_id,
            process_id: current_process,
            correlated_success: proof,
            cancel_requested,
        });
        refresh(flow, attempt_id)
    }

    pub(crate) fn claude_status_resolved(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        status: crate::ProviderStatusSnapshot,
    ) -> Reduction {
        let Some((flow, method, current, _, _, _)) = active_claude(&self.snapshot) else {
            return stale();
        };
        if current != attempt_id || status.id != flow.target.provider_id {
            return stale();
        }
        let matching = configured_claude_method(&status) == Some(method);
        match &self.snapshot {
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Reconciling {
                ..
            }) if matching => self.complete_claude(flow.target, status),
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Reconciling {
                ..
            }) => self.fail_claude(
                flow,
                Some(method),
                ClaudeAccountFailureReason::StatusNotConfigured,
            ),
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
                correlated_success: true,
                ..
            }) if matching => self.complete_claude(flow.target, status),
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
                correlated_success: true,
                ..
            }) => self.fail_claude(
                flow,
                Some(method),
                ClaudeAccountFailureReason::StatusNotConfigured,
            ),
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
                method: ClaudeAccountMethod::ManagedToken,
                ..
            }) if matching && flow.intent == ClaudeAccountIntent::Add => {
                self.complete_claude(flow.target, status)
            }
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
                method: ClaudeAccountMethod::ManagedToken,
                ..
            }) => applied(),
            ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::OutcomeUnknown {
                cancel_requested,
                ..
            }) => {
                let reason = if *cancel_requested {
                    ClaudeAccountRecoveryReason::CancelOutcomeUnknown
                } else {
                    ClaudeAccountRecoveryReason::LoginOutcomeUnknown
                };
                self.set_claude(ClaudeAccountSnapshot::RecoveryRequired { flow, reason });
                applied()
            }
            _ => applied(),
        }
    }

    fn reconcile_claude(
        &mut self,
        flow: ClaudeAccountFlow,
        method: ClaudeAccountMethod,
        attempt_id: ProviderAuthAttemptId,
        process_id: Option<ClaudeCodeProcessId>,
    ) -> Reduction {
        self.set_claude(ClaudeAccountSnapshot::Reconciling {
            flow: flow.clone(),
            method,
            attempt_id,
            process_id,
        });
        refresh(flow, attempt_id)
    }
}

fn active_claude(
    snapshot: &ProviderAuthFlowSnapshot,
) -> Option<(
    ClaudeAccountFlow,
    ClaudeAccountMethod,
    ProviderAuthAttemptId,
    Option<ClaudeCodeProcessId>,
    bool,
    bool,
)> {
    let ProviderAuthFlowSnapshot::ClaudeAccount(state) = snapshot else {
        return None;
    };
    match state {
        ClaudeAccountSnapshot::CheckingExistingLogin { flow, attempt_id } => Some((
            flow.clone(),
            ClaudeAccountMethod::ClaudeCodeLogin,
            *attempt_id,
            None,
            false,
            false,
        )),
        ClaudeAccountSnapshot::SettlingManagedToken { flow, attempt_id } => Some((
            flow.clone(),
            ClaudeAccountMethod::ManagedToken,
            *attempt_id,
            None,
            false,
            false,
        )),
        ClaudeAccountSnapshot::StartingClaudeCodeLogin {
            flow,
            attempt_id,
            process_id,
        }
        | ClaudeAccountSnapshot::AwaitingAuthorizationCode {
            flow,
            attempt_id,
            process_id,
        }
        | ClaudeAccountSnapshot::Authenticating {
            flow,
            attempt_id,
            process_id,
        } => Some((
            flow.clone(),
            ClaudeAccountMethod::ClaudeCodeLogin,
            *attempt_id,
            Some(*process_id),
            false,
            false,
        )),
        ClaudeAccountSnapshot::Cancelling {
            flow,
            attempt_id,
            process_id,
        } => Some((
            flow.clone(),
            ClaudeAccountMethod::ClaudeCodeLogin,
            *attempt_id,
            Some(*process_id),
            false,
            true,
        )),
        ClaudeAccountSnapshot::Reconciling {
            flow,
            method,
            attempt_id,
            process_id,
        } => Some((flow.clone(), *method, *attempt_id, *process_id, true, false)),
        ClaudeAccountSnapshot::OutcomeUnknown {
            flow,
            method,
            attempt_id,
            process_id,
            correlated_success,
            cancel_requested,
        } => Some((
            flow.clone(),
            *method,
            *attempt_id,
            *process_id,
            *correlated_success,
            *cancel_requested,
        )),
        _ => None,
    }
}

fn refresh(flow: ClaudeAccountFlow, attempt_id: ProviderAuthAttemptId) -> Reduction {
    effect(ClaudeAccountEffect::RefreshStatus {
        attempt_id,
        target: flow.target,
    })
}

fn stale_cleanup(attempt_id: ProviderAuthAttemptId, process_id: ClaudeCodeProcessId) -> Reduction {
    (
        vec![crate::ProviderAuthEffect::ClaudeAccount(
            ClaudeAccountEffect::CancelClaudeCodeLogin {
                attempt_id,
                process_id,
            },
        )],
        crate::ProviderAuthDisposition::IgnoredStale,
    )
}
