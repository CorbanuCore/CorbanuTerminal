use crate::ProviderAuthAction;
use crate::ProviderAuthAttemptId;
use crate::ProviderAuthCompletion;
use crate::ProviderAuthController;
use crate::ProviderAuthDisposition;
use crate::ProviderAuthEffect;
use crate::ProviderAuthFlowSnapshot;
use crate::ProviderAuthRejectionReason;
use crate::ProviderConfigurationState;
use crate::ProviderRecoveryReason;
use crate::auth_flow::Reduction;
use crate::claude_account_flow::*;

impl ProviderAuthController {
    pub(crate) fn claude_account(&mut self, action: ClaudeAccountAction) -> Reduction {
        match action {
            ClaudeAccountAction::Start(start) => self.start_claude(start),
            ClaudeAccountAction::ChooseMethod(method) => self.choose_claude_method(method),
            ClaudeAccountAction::SetManagedToken(secret) => self.set_claude_token(secret),
            ClaudeAccountAction::Submit => self.submit_claude(),
            ClaudeAccountAction::SubmitAuthorizationCode(secret) => self.submit_claude_code(secret),
            ClaudeAccountAction::Cancel | ClaudeAccountAction::KeepCurrent => self.cancel_claude(),
            ClaudeAccountAction::Retry => self.retry_claude(),
            ClaudeAccountAction::ManagedTokenFinished { attempt_id, result } => {
                self.claude_managed_finished(attempt_id, result)
            }
            ClaudeAccountAction::ManagedTimeoutElapsed { attempt_id } => {
                self.claude_managed_timeout(attempt_id)
            }
            ClaudeAccountAction::ExistingLoginChecked { attempt_id, result } => {
                self.claude_existing_checked(attempt_id, result)
            }
            ClaudeAccountAction::ClaudeCodeReady {
                attempt_id,
                process_id,
                challenge,
            } => self.claude_code_ready(attempt_id, process_id, challenge),
            ClaudeAccountAction::ClaudeCodeFinished {
                attempt_id,
                process_id,
                outcome,
            } => self.claude_code_finished(attempt_id, process_id, outcome),
            ClaudeAccountAction::BackendTransportLost {
                attempt_id,
                process_id,
            } => self.claude_transport_lost(attempt_id, process_id),
            ClaudeAccountAction::StatusResolved { attempt_id, status } => {
                self.claude_status_resolved(attempt_id, status)
            }
        }
    }

    fn start_claude(&mut self, start: ClaudeAccountFlowStart) -> Reduction {
        if super::auth_flow::commit_in_progress(&self.snapshot) {
            return rejected(ProviderAuthRejectionReason::CommitInProgress);
        }
        self.clear_api_key_input();
        self.clear_claude_input();
        let flow = ClaudeAccountFlow {
            target: start.target,
            intent: start.intent,
        };
        if start.status.id != flow.target.provider_id {
            return self.block_claude(flow, ClaudeAccountBlockedReason::StatusIdentityMismatch);
        }
        if flow.intent == ClaudeAccountIntent::Add
            && start.status.configuration == ProviderConfigurationState::Configured
        {
            return self.complete_claude(flow.target, start.status);
        }
        if matches!(
            flow.intent,
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::Environment
            }
        ) {
            return self.block_claude(
                flow,
                ClaudeAccountBlockedReason::ExternallyManagedEnvironment,
            );
        }
        let recommended = match flow.intent {
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::ManagedToken,
            } => Some(ClaudeAccountMethod::ManagedToken),
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::ClaudeCodeLogin,
            } => Some(ClaudeAccountMethod::ClaudeCodeLogin),
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::Unknown,
            } => {
                self.set_claude(ClaudeAccountSnapshot::RecoveryRequired {
                    flow,
                    reason: ClaudeAccountRecoveryReason::MissingSelection,
                });
                return applied();
            }
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::Environment,
            } => unreachable!(),
            ClaudeAccountIntent::Add | ClaudeAccountIntent::Replace => {
                configured_claude_method(&start.status)
            }
        };
        self.set_claude(match recovery_reason(&start.status) {
            Some(reason) => ClaudeAccountSnapshot::RecoveryRequired { flow, reason },
            None => ClaudeAccountSnapshot::ChoosingMethod { flow, recommended },
        });
        applied()
    }

    fn choose_claude_method(&mut self, method: ClaudeAccountMethod) -> Reduction {
        let flow =
            match &self.snapshot {
                ProviderAuthFlowSnapshot::ClaudeAccount(
                    ClaudeAccountSnapshot::ChoosingMethod { flow, .. },
                )
                | ProviderAuthFlowSnapshot::ClaudeAccount(
                    ClaudeAccountSnapshot::RecoveryRequired { flow, .. },
                ) => flow.clone(),
                _ => return rejected_for_claude(&self.snapshot),
            };
        let allowed = match flow.intent {
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::ManagedToken,
            } => Some(ClaudeAccountMethod::ManagedToken),
            ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::ClaudeCodeLogin,
            } => Some(ClaudeAccountMethod::ClaudeCodeLogin),
            ClaudeAccountIntent::UnauthorizedRecovery { .. } => None,
            ClaudeAccountIntent::Add | ClaudeAccountIntent::Replace => Some(method),
        };
        if allowed != Some(method) {
            return rejected(ProviderAuthRejectionReason::InvalidState);
        }
        self.clear_claude_input();
        self.set_claude(match method {
            ClaudeAccountMethod::ManagedToken => ClaudeAccountSnapshot::EnteringManagedToken {
                flow,
                has_input: false,
            },
            ClaudeAccountMethod::ClaudeCodeLogin => {
                ClaudeAccountSnapshot::ReadyClaudeCodeLogin { flow }
            }
        });
        applied()
    }

    fn set_claude_token(&mut self, secret: ClaudeManagedTokenSecret) -> Reduction {
        let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::EnteringManagedToken {
            has_input,
            ..
        }) = &mut self.snapshot
        else {
            return rejected_for_claude(&self.snapshot);
        };
        *has_input = !secret.expose_secret().trim().is_empty();
        self.claude_input = Some(secret);
        applied()
    }

    fn submit_claude(&mut self) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::ClaudeAccount(state) => state.clone(),
            _ => return rejected_for_claude(&self.snapshot),
        };
        match state {
            ClaudeAccountSnapshot::EnteringManagedToken { flow, .. } => {
                self.submit_managed_token(flow)
            }
            ClaudeAccountSnapshot::ReadyClaudeCodeLogin { flow } => self.submit_claude_login(flow),
            _ => rejected_for_claude(&self.snapshot),
        }
    }

    fn submit_managed_token(&mut self, flow: ClaudeAccountFlow) -> Reduction {
        let Some(secret) = self
            .claude_input
            .take()
            .filter(|secret| !secret.expose_secret().trim().is_empty())
        else {
            return self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ManagedToken),
                ClaudeAccountFailureReason::EmptyCredential,
            );
        };
        let Some(attempt_id) = self.allocate_attempt() else {
            return self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ManagedToken),
                ClaudeAccountFailureReason::AttemptIdExhausted,
            );
        };
        self.set_claude(ClaudeAccountSnapshot::SettlingManagedToken {
            flow: flow.clone(),
            attempt_id,
        });
        effects(vec![
            ClaudeAccountEffect::EnrollManagedToken {
                attempt_id,
                target: flow.target,
                secret,
            },
            ClaudeAccountEffect::ScheduleManagedTimeout {
                attempt_id,
                timeout: CLAUDE_MANAGED_AUTH_TIMEOUT,
            },
        ])
    }

    fn submit_claude_login(&mut self, flow: ClaudeAccountFlow) -> Reduction {
        let Some(attempt_id) = self.allocate_attempt() else {
            return self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ClaudeCodeLogin),
                ClaudeAccountFailureReason::AttemptIdExhausted,
            );
        };
        if flow.intent == ClaudeAccountIntent::Add {
            self.set_claude(ClaudeAccountSnapshot::CheckingExistingLogin {
                flow: flow.clone(),
                attempt_id,
            });
            return effect(ClaudeAccountEffect::CheckExistingClaudeCodeLogin {
                attempt_id,
                target: flow.target,
            });
        }
        self.start_claude_process(flow, attempt_id)
    }

    pub(crate) fn start_claude_process(
        &mut self,
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
    ) -> Reduction {
        let Some(process_id) = self.allocate_claude_process() else {
            return self.fail_claude(
                flow,
                Some(ClaudeAccountMethod::ClaudeCodeLogin),
                ClaudeAccountFailureReason::AttemptIdExhausted,
            );
        };
        let identity_policy = if matches!(
            flow.intent,
            ClaudeAccountIntent::UnauthorizedRecovery { .. }
        ) {
            ClaudeCodeIdentityPolicy::PreserveSelected
        } else {
            ClaudeCodeIdentityPolicy::AllowExplicitChange
        };
        self.set_claude(ClaudeAccountSnapshot::StartingClaudeCodeLogin {
            flow: flow.clone(),
            attempt_id,
            process_id,
        });
        effect(ClaudeAccountEffect::StartClaudeCodeLogin {
            attempt_id,
            process_id,
            target: flow.target,
            identity_policy,
        })
    }

    fn submit_claude_code(&mut self, secret: ClaudeAuthorizationCodeSecret) -> Reduction {
        let ProviderAuthFlowSnapshot::ClaudeAccount(
            ClaudeAccountSnapshot::AwaitingAuthorizationCode {
                flow,
                attempt_id,
                process_id,
            },
        ) = &self.snapshot
        else {
            return rejected_for_claude(&self.snapshot);
        };
        if secret.expose_secret().trim().is_empty() {
            return rejected(ProviderAuthRejectionReason::InvalidState);
        }
        let flow = flow.clone();
        let attempt_id = *attempt_id;
        let process_id = *process_id;
        self.set_claude(ClaudeAccountSnapshot::Authenticating {
            flow,
            attempt_id,
            process_id,
        });
        effect(ClaudeAccountEffect::SendAuthorizationCode {
            attempt_id,
            process_id,
            secret,
        })
    }

    fn cancel_claude(&mut self) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::ClaudeAccount(state) => state.clone(),
            _ => return rejected_for_claude(&self.snapshot),
        };
        match state {
            ClaudeAccountSnapshot::ChoosingMethod { flow, .. }
            | ClaudeAccountSnapshot::EnteringManagedToken { flow, .. }
            | ClaudeAccountSnapshot::ReadyClaudeCodeLogin { flow }
            | ClaudeAccountSnapshot::RecoveryRequired { flow, .. }
            | ClaudeAccountSnapshot::Blocked { flow, .. }
            | ClaudeAccountSnapshot::Failed { flow, .. } => self.cancelled_claude(flow.target),
            ClaudeAccountSnapshot::StartingClaudeCodeLogin {
                flow,
                attempt_id,
                process_id,
            }
            | ClaudeAccountSnapshot::AwaitingAuthorizationCode {
                flow,
                attempt_id,
                process_id,
            } => {
                self.set_claude(ClaudeAccountSnapshot::Cancelling {
                    flow,
                    attempt_id,
                    process_id,
                });
                effect(ClaudeAccountEffect::CancelClaudeCodeLogin {
                    attempt_id,
                    process_id,
                })
            }
            _ => rejected_for_claude(&self.snapshot),
        }
    }

    fn retry_claude(&mut self) -> Reduction {
        let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::Failed {
            flow,
            method,
            ..
        }) = &self.snapshot
        else {
            return rejected_for_claude(&self.snapshot);
        };
        let flow = flow.clone();
        let method = *method;
        self.clear_claude_input();
        self.set_claude(match method {
            Some(ClaudeAccountMethod::ManagedToken) => {
                ClaudeAccountSnapshot::EnteringManagedToken {
                    flow,
                    has_input: false,
                }
            }
            Some(ClaudeAccountMethod::ClaudeCodeLogin) => {
                ClaudeAccountSnapshot::ReadyClaudeCodeLogin { flow }
            }
            None => ClaudeAccountSnapshot::ChoosingMethod {
                flow,
                recommended: None,
            },
        });
        applied()
    }

    pub(crate) fn block_claude(
        &mut self,
        flow: ClaudeAccountFlow,
        reason: ClaudeAccountBlockedReason,
    ) -> Reduction {
        self.set_claude(ClaudeAccountSnapshot::Blocked { flow, reason });
        applied()
    }

    pub(crate) fn fail_claude(
        &mut self,
        flow: ClaudeAccountFlow,
        method: Option<ClaudeAccountMethod>,
        reason: ClaudeAccountFailureReason,
    ) -> Reduction {
        self.set_claude(ClaudeAccountSnapshot::Failed {
            flow,
            method,
            reason,
        });
        applied()
    }

    pub(crate) fn complete_claude(
        &mut self,
        target: ClaudeAccountTarget,
        status: crate::ProviderStatusSnapshot,
    ) -> Reduction {
        self.set_claude(ClaudeAccountSnapshot::Configured {
            target: target.clone(),
            status: status.clone(),
        });
        complete(ClaudeAccountCompletion::Configured { target, status })
    }

    pub(crate) fn cancelled_claude(&mut self, target: ClaudeAccountTarget) -> Reduction {
        self.clear_claude_input();
        self.set_claude(ClaudeAccountSnapshot::Cancelled {
            target: target.clone(),
        });
        complete(ClaudeAccountCompletion::Cancelled { target })
    }

    pub(crate) fn set_claude(&mut self, state: ClaudeAccountSnapshot) {
        self.snapshot = ProviderAuthFlowSnapshot::ClaudeAccount(state);
    }

    fn allocate_claude_process(&mut self) -> Option<ClaudeCodeProcessId> {
        let next = self.next_claude_process_id.checked_add(1)?;
        let process_id = ClaudeCodeProcessId(self.next_claude_process_id);
        self.next_claude_process_id = next;
        Some(process_id)
    }
}

fn recovery_reason(status: &crate::ProviderStatusSnapshot) -> Option<ClaudeAccountRecoveryReason> {
    status.methods.iter().find_map(|method| match method.state {
        crate::ProviderMethodState::RecoveryRequired { reason } => Some(match reason {
            ProviderRecoveryReason::AmbiguousClaudeSources => {
                ClaudeAccountRecoveryReason::AmbiguousSources
            }
            ProviderRecoveryReason::MissingClaudeSelection => {
                ClaudeAccountRecoveryReason::MissingSelection
            }
            ProviderRecoveryReason::UnhealthyClaudeSelection => {
                ClaudeAccountRecoveryReason::UnhealthySelection
            }
            _ => return None,
        }),
        _ => None,
    })
}

pub(crate) fn effects(effects: Vec<ClaudeAccountEffect>) -> Reduction {
    (
        effects
            .into_iter()
            .map(ProviderAuthEffect::ClaudeAccount)
            .collect(),
        ProviderAuthDisposition::Applied,
    )
}

pub(crate) fn effect(effect: ClaudeAccountEffect) -> Reduction {
    effects(vec![effect])
}

fn complete(completion: ClaudeAccountCompletion) -> Reduction {
    (
        vec![ProviderAuthEffect::Complete(
            ProviderAuthCompletion::ClaudeAccount(completion),
        )],
        ProviderAuthDisposition::Applied,
    )
}

pub(crate) fn applied() -> Reduction {
    (Vec::new(), ProviderAuthDisposition::Applied)
}

pub(crate) fn rejected(reason: ProviderAuthRejectionReason) -> Reduction {
    (Vec::new(), ProviderAuthDisposition::Rejected(reason))
}

pub(crate) fn rejected_for_claude(snapshot: &ProviderAuthFlowSnapshot) -> Reduction {
    rejected(
        if matches!(snapshot, ProviderAuthFlowSnapshot::ClaudeAccount(state) if state.is_in_flight())
        {
            ProviderAuthRejectionReason::CommitInProgress
        } else {
            ProviderAuthRejectionReason::InvalidState
        },
    )
}

pub(crate) fn stale() -> Reduction {
    (Vec::new(), ProviderAuthDisposition::IgnoredStale)
}

impl From<ClaudeAccountAction> for ProviderAuthAction {
    fn from(action: ClaudeAccountAction) -> Self {
        Self::ClaudeAccount(action)
    }
}
