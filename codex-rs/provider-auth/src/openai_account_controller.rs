use crate::OpenAiAccountAction;
use crate::OpenAiAccountBlockedReason;
use crate::OpenAiAccountCancelPurpose;
use crate::OpenAiAccountCompletion;
use crate::OpenAiAccountEffect;
use crate::OpenAiAccountFailureReason;
use crate::OpenAiAccountFlow;
use crate::OpenAiAccountFlowStart;
use crate::OpenAiAccountLoginContext;
use crate::OpenAiAccountLoginId;
use crate::OpenAiAccountLoginOutcome;
use crate::OpenAiAccountMethod;
use crate::OpenAiAccountOutcomeUnknownReason;
use crate::OpenAiAccountRecoveryReason;
use crate::OpenAiAccountSnapshot;
use crate::OpenAiAccountStartResult;
use crate::OpenAiAccountTarget;
use crate::ProviderAuthAttemptId;
use crate::ProviderAuthCompletion;
use crate::ProviderAuthController;
use crate::ProviderAuthDisposition;
use crate::ProviderAuthEffect;
use crate::ProviderAuthFlowSnapshot;
use crate::ProviderAuthRejectionReason;
use crate::ProviderStatusSnapshot;
use crate::auth_flow::Reduction;
use crate::openai_account_flow::AccountMethodState;
use crate::openai_account_flow::account_method_state;
use crate::openai_account_flow::stale;

impl ProviderAuthController {
    pub(crate) fn openai_account(&mut self, action: OpenAiAccountAction) -> Reduction {
        match action {
            OpenAiAccountAction::Start(start) => self.start_openai_account(start),
            OpenAiAccountAction::Cancel => self.cancel_openai_account(),
            OpenAiAccountAction::Retry => self.retry_openai_account(),
            OpenAiAccountAction::StartFinished { attempt_id, result } => {
                self.openai_start_finished(attempt_id, result)
            }
            OpenAiAccountAction::CancelFinished { attempt_id, result } => {
                self.openai_cancel_finished(attempt_id, result)
            }
            OpenAiAccountAction::LoginCompleted {
                attempt_id,
                login_id,
                outcome,
            } => self.openai_login_completed(attempt_id, login_id, outcome),
            OpenAiAccountAction::TransportLost { attempt_id } => {
                self.openai_transport_lost(attempt_id)
            }
            OpenAiAccountAction::StatusResolved { attempt_id, status } => {
                self.openai_status_resolved(attempt_id, status)
            }
        }
    }

    fn start_openai_account(&mut self, start: OpenAiAccountFlowStart) -> Reduction {
        if super::auth_flow::commit_in_progress(&self.snapshot) {
            return rejected(ProviderAuthRejectionReason::CommitInProgress);
        }
        self.clear_api_key_input();
        let flow = OpenAiAccountFlow {
            target: start.target,
            method: start.method,
            context: start.context,
        };
        if start.status.id != flow.target.provider_id {
            return self.block_openai(flow, OpenAiAccountBlockedReason::StatusIdentityMismatch);
        }
        match account_method_state(&start.status) {
            AccountMethodState::ManagedAccount => {
                return self.complete_openai(flow.target, start.status);
            }
            AccountMethodState::ExternallyManaged => {
                return self.block_openai(flow, OpenAiAccountBlockedReason::ExternallyManaged);
            }
            AccountMethodState::NotConfigured => {}
        }
        if flow.method == OpenAiAccountMethod::Browser
            && flow.context == OpenAiAccountLoginContext::ProviderEnrollment
        {
            return self.block_openai(
                flow,
                OpenAiAccountBlockedReason::BrowserUnavailableForProviderEnrollment,
            );
        }
        let Some(attempt_id) = self.allocate_attempt() else {
            return self.fail_openai(flow, OpenAiAccountFailureReason::AttemptIdExhausted);
        };
        self.set_openai(OpenAiAccountSnapshot::Starting {
            flow: flow.clone(),
            attempt_id,
        });
        openai_effect(OpenAiAccountEffect::StartLogin {
            attempt_id,
            target: flow.target,
            method: flow.method,
            context: flow.context,
        })
    }

    fn cancel_openai_account(&mut self) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::OpenAiAccount(state) => state.clone(),
            _ => return rejected(ProviderAuthRejectionReason::InvalidState),
        };
        match state {
            OpenAiAccountSnapshot::Starting { flow, attempt_id } => {
                self.set_openai(OpenAiAccountSnapshot::CancelPendingStart { flow, attempt_id });
                applied()
            }
            OpenAiAccountSnapshot::AwaitingUser {
                flow,
                attempt_id,
                login_id,
            } => {
                self.set_openai(OpenAiAccountSnapshot::Cancelling {
                    flow,
                    attempt_id,
                    login_id: login_id.clone(),
                    purpose: OpenAiAccountCancelPurpose::UserRequested,
                });
                openai_effect(OpenAiAccountEffect::CancelLogin {
                    attempt_id,
                    login_id,
                })
            }
            OpenAiAccountSnapshot::Blocked { flow, .. }
            | OpenAiAccountSnapshot::Failed { flow, .. }
            | OpenAiAccountSnapshot::RecoveryRequired { flow, .. } => {
                self.cancelled_openai(flow.target)
            }
            state if state.is_in_flight() => {
                rejected(ProviderAuthRejectionReason::CommitInProgress)
            }
            _ => rejected(ProviderAuthRejectionReason::InvalidState),
        }
    }

    fn retry_openai_account(&mut self) -> Reduction {
        let ProviderAuthFlowSnapshot::OpenAiAccount(
            OpenAiAccountSnapshot::Failed { flow, .. }
            | OpenAiAccountSnapshot::RecoveryRequired { flow, .. },
        ) = &self.snapshot
        else {
            return rejected_for_openai(&self.snapshot);
        };
        let flow = flow.clone();
        let Some(attempt_id) = self.allocate_attempt() else {
            return self.fail_openai(flow, OpenAiAccountFailureReason::AttemptIdExhausted);
        };
        self.set_openai(OpenAiAccountSnapshot::Starting {
            flow: flow.clone(),
            attempt_id,
        });
        openai_effect(OpenAiAccountEffect::StartLogin {
            attempt_id,
            target: flow.target,
            method: flow.method,
            context: flow.context,
        })
    }

    fn openai_start_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        result: OpenAiAccountStartResult,
    ) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::OpenAiAccount(state) => state.clone(),
            _ => return stale(),
        };
        let (flow, cancel_pending) = match state {
            OpenAiAccountSnapshot::Starting {
                flow,
                attempt_id: current,
            } if current == attempt_id => (flow, false),
            OpenAiAccountSnapshot::CancelPendingStart {
                flow,
                attempt_id: current,
            } if current == attempt_id => (flow, true),
            _ => return stale(),
        };
        match result {
            OpenAiAccountStartResult::Started {
                login_id,
                challenge,
            } => {
                if cancel_pending {
                    self.set_openai(OpenAiAccountSnapshot::Cancelling {
                        flow,
                        attempt_id,
                        login_id: login_id.clone(),
                        purpose: OpenAiAccountCancelPurpose::UserRequested,
                    });
                    openai_effect(OpenAiAccountEffect::CancelLogin {
                        attempt_id,
                        login_id,
                    })
                } else if challenge.method() != flow.method {
                    self.set_openai(OpenAiAccountSnapshot::Cancelling {
                        flow,
                        attempt_id,
                        login_id: login_id.clone(),
                        purpose: OpenAiAccountCancelPurpose::ProtocolMismatch,
                    });
                    openai_effect(OpenAiAccountEffect::CancelLogin {
                        attempt_id,
                        login_id,
                    })
                } else {
                    self.set_openai(OpenAiAccountSnapshot::AwaitingUser {
                        flow,
                        attempt_id,
                        login_id: login_id.clone(),
                    });
                    openai_effect(OpenAiAccountEffect::PresentChallenge {
                        attempt_id,
                        login_id,
                        challenge,
                    })
                }
            }
            OpenAiAccountStartResult::Rejected => self.finish_failed_start(
                flow,
                OpenAiAccountFailureReason::StartRejected,
                cancel_pending,
            ),
            OpenAiAccountStartResult::ProtocolMismatch => self.finish_failed_start(
                flow,
                OpenAiAccountFailureReason::ProtocolMismatch,
                cancel_pending,
            ),
            OpenAiAccountStartResult::TransportLost => self.unknown_openai(
                flow,
                attempt_id,
                None,
                OpenAiAccountOutcomeUnknownReason::StartTransportLost,
            ),
        }
    }

    fn finish_failed_start(
        &mut self,
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountFailureReason,
        cancel_pending: bool,
    ) -> Reduction {
        if cancel_pending {
            self.cancelled_openai(flow.target)
        } else {
            self.fail_openai(flow, reason)
        }
    }

    pub(crate) fn finish_cancel(
        &mut self,
        flow: OpenAiAccountFlow,
        purpose: OpenAiAccountCancelPurpose,
    ) -> Reduction {
        match purpose {
            OpenAiAccountCancelPurpose::UserRequested => self.cancelled_openai(flow.target),
            OpenAiAccountCancelPurpose::ProtocolMismatch => {
                self.fail_openai(flow, OpenAiAccountFailureReason::ProtocolMismatch)
            }
        }
    }

    fn openai_login_completed(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
        outcome: OpenAiAccountLoginOutcome,
    ) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::OpenAiAccount(state) => state.clone(),
            _ => return stale(),
        };
        let (flow, current, expected_login, cancel_purpose) = match state {
            OpenAiAccountSnapshot::AwaitingUser {
                flow,
                attempt_id,
                login_id,
            } => (flow, attempt_id, Some(login_id), None),
            OpenAiAccountSnapshot::Cancelling {
                flow,
                attempt_id,
                login_id,
                purpose,
            } => (flow, attempt_id, Some(login_id), Some(purpose)),
            OpenAiAccountSnapshot::OutcomeUnknown {
                flow,
                attempt_id,
                login_id,
                reason,
            } => (
                flow,
                attempt_id,
                login_id,
                match reason {
                    OpenAiAccountOutcomeUnknownReason::CancelNotFound
                    | OpenAiAccountOutcomeUnknownReason::CancelTransportLost => {
                        Some(OpenAiAccountCancelPurpose::UserRequested)
                    }
                    OpenAiAccountOutcomeUnknownReason::ProtocolMismatchCancelTransportLost => {
                        Some(OpenAiAccountCancelPurpose::ProtocolMismatch)
                    }
                    _ => None,
                },
            ),
            _ => return stale(),
        };
        if current != attempt_id || expected_login.as_ref() != Some(&login_id) {
            return stale();
        }
        match outcome {
            OpenAiAccountLoginOutcome::Succeeded => {
                self.reconcile_openai(flow, attempt_id, login_id)
            }
            OpenAiAccountLoginOutcome::Failed => match cancel_purpose {
                Some(purpose) => self.finish_cancel(flow, purpose),
                None => self.fail_openai(flow, OpenAiAccountFailureReason::LoginRejected),
            },
        }
    }

    fn openai_transport_lost(&mut self, attempt_id: ProviderAuthAttemptId) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::OpenAiAccount(state) => state.clone(),
            _ => return stale(),
        };
        let (flow, current, login_id, reason) = match state {
            OpenAiAccountSnapshot::AwaitingUser {
                flow,
                attempt_id,
                login_id,
            } => (
                flow,
                attempt_id,
                login_id,
                OpenAiAccountOutcomeUnknownReason::LoginTransportLost,
            ),
            OpenAiAccountSnapshot::Cancelling {
                flow,
                attempt_id,
                login_id,
                purpose,
            } => (
                flow,
                attempt_id,
                login_id,
                match purpose {
                    OpenAiAccountCancelPurpose::UserRequested => {
                        OpenAiAccountOutcomeUnknownReason::CancelTransportLost
                    }
                    OpenAiAccountCancelPurpose::ProtocolMismatch => {
                        OpenAiAccountOutcomeUnknownReason::ProtocolMismatchCancelTransportLost
                    }
                },
            ),
            _ => return stale(),
        };
        if current != attempt_id {
            return stale();
        }
        self.unknown_openai(flow, attempt_id, Some(login_id), reason)
    }

    fn reconcile_openai(
        &mut self,
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
    ) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::Reconciling {
            flow: flow.clone(),
            attempt_id,
            login_id,
        });
        openai_effect(OpenAiAccountEffect::RefreshStatus {
            attempt_id,
            target: flow.target,
        })
    }

    pub(crate) fn unknown_openai(
        &mut self,
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        login_id: Option<OpenAiAccountLoginId>,
        reason: OpenAiAccountOutcomeUnknownReason,
    ) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::OutcomeUnknown {
            flow: flow.clone(),
            attempt_id,
            login_id,
            reason,
        });
        openai_effect(OpenAiAccountEffect::RefreshStatus {
            attempt_id,
            target: flow.target,
        })
    }

    fn block_openai(
        &mut self,
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountBlockedReason,
    ) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::Blocked { flow, reason });
        applied()
    }

    pub(crate) fn fail_openai(
        &mut self,
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountFailureReason,
    ) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::Failed { flow, reason });
        applied()
    }

    pub(crate) fn recovery_openai(
        &mut self,
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountRecoveryReason,
    ) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::RecoveryRequired { flow, reason });
        applied()
    }

    pub(crate) fn complete_openai(
        &mut self,
        target: OpenAiAccountTarget,
        status: ProviderStatusSnapshot,
    ) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::Configured {
            target: target.clone(),
            status: status.clone(),
        });
        complete(OpenAiAccountCompletion::Configured { target, status })
    }

    pub(crate) fn cancelled_openai(&mut self, target: OpenAiAccountTarget) -> Reduction {
        self.set_openai(OpenAiAccountSnapshot::Cancelled {
            target: target.clone(),
        });
        complete(OpenAiAccountCompletion::Cancelled { target })
    }

    fn set_openai(&mut self, state: OpenAiAccountSnapshot) {
        self.snapshot = ProviderAuthFlowSnapshot::OpenAiAccount(state);
    }
}

fn openai_effect(effect: OpenAiAccountEffect) -> Reduction {
    (
        vec![ProviderAuthEffect::OpenAiAccount(effect)],
        ProviderAuthDisposition::Applied,
    )
}

fn complete(completion: OpenAiAccountCompletion) -> Reduction {
    (
        vec![ProviderAuthEffect::Complete(
            ProviderAuthCompletion::OpenAiAccount(completion),
        )],
        ProviderAuthDisposition::Applied,
    )
}

fn applied() -> Reduction {
    (Vec::new(), ProviderAuthDisposition::Applied)
}

fn rejected(reason: ProviderAuthRejectionReason) -> Reduction {
    (Vec::new(), ProviderAuthDisposition::Rejected(reason))
}

fn rejected_for_openai(snapshot: &ProviderAuthFlowSnapshot) -> Reduction {
    rejected(
        if matches!(snapshot, ProviderAuthFlowSnapshot::OpenAiAccount(state) if state.is_in_flight())
        {
            ProviderAuthRejectionReason::CommitInProgress
        } else {
            ProviderAuthRejectionReason::InvalidState
        },
    )
}

impl From<OpenAiAccountAction> for crate::ProviderAuthAction {
    fn from(action: OpenAiAccountAction) -> Self {
        Self::OpenAiAccount(action)
    }
}
