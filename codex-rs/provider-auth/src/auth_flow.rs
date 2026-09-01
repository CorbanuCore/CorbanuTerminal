use std::time::Duration;

use crate::ApiKeyAuthTarget;
use crate::ApiKeyFlowContext;
use crate::ApiKeyFlowIntent;
use crate::ApiKeyFlowStart;
use crate::ApiKeySecret;
use crate::EnvironmentCredentialMetadata;
use crate::OpenAiAccountAction;
use crate::OpenAiAccountCompletion;
use crate::OpenAiAccountEffect;
use crate::OpenAiAccountSnapshot;
use crate::ProviderConfigurationState;
use crate::ProviderStatusSnapshot;
use crate::claude_account_flow::ClaudeAccountAction;
use crate::claude_account_flow::ClaudeAccountCompletion;
use crate::claude_account_flow::ClaudeAccountEffect;
use crate::claude_account_flow::ClaudeAccountSnapshot;
use crate::claude_account_flow::ClaudeManagedTokenSecret;

pub const PROVIDER_AUTH_FLOW_PROTOCOL_VERSION: u16 = 1;
pub const API_KEY_AUTH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderAuthAttemptId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAuthBlockedReason {
    EnvironmentCredentialPresent,
    InvalidEnvironmentCredential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAuthFailureReason {
    EmptyCredential,
    PersistenceRejected,
    StorageUnavailable,
    StatusNotConfigured,
    AttemptIdExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAuthRejectionReason {
    InvalidState,
    CommitInProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderAuthFlowSnapshot {
    Idle,
    OpenAiAccount(OpenAiAccountSnapshot),
    ClaudeAccount(ClaudeAccountSnapshot),
    Entering {
        flow: ApiKeyFlowContext,
        has_input: bool,
    },
    Blocked {
        flow: ApiKeyFlowContext,
        reason: ProviderAuthBlockedReason,
    },
    Settling {
        flow: ApiKeyFlowContext,
        attempt_id: ProviderAuthAttemptId,
    },
    Reconciling {
        flow: ApiKeyFlowContext,
        attempt_id: ProviderAuthAttemptId,
    },
    OutcomeUnknown {
        flow: ApiKeyFlowContext,
        attempt_id: ProviderAuthAttemptId,
    },
    Failed {
        flow: ApiKeyFlowContext,
        reason: ProviderAuthFailureReason,
    },
    Configured {
        target: ApiKeyAuthTarget,
        status: ProviderStatusSnapshot,
    },
    Cancelled {
        target: ApiKeyAuthTarget,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApiKeyPersistenceResult {
    Stored,
    Rejected,
    StorageUnavailable,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProviderAuthAction {
    OpenAiAccount(OpenAiAccountAction),
    ClaudeAccount(ClaudeAccountAction),
    StartApiKey(ApiKeyFlowStart),
    SetApiKey(ApiKeySecret),
    Submit,
    Cancel,
    Retry,
    PersistenceFinished {
        attempt_id: ProviderAuthAttemptId,
        result: ApiKeyPersistenceResult,
    },
    TimeoutElapsed {
        attempt_id: ProviderAuthAttemptId,
    },
    StatusResolved {
        attempt_id: ProviderAuthAttemptId,
        status: ProviderStatusSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderAuthCompletion {
    OpenAiAccount(OpenAiAccountCompletion),
    ClaudeAccount(ClaudeAccountCompletion),
    Configured {
        target: ApiKeyAuthTarget,
        status: ProviderStatusSnapshot,
    },
    Cancelled {
        target: ApiKeyAuthTarget,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProviderAuthEffect {
    OpenAiAccount(OpenAiAccountEffect),
    ClaudeAccount(ClaudeAccountEffect),
    PersistApiKey {
        attempt_id: ProviderAuthAttemptId,
        target: ApiKeyAuthTarget,
        secret: ApiKeySecret,
    },
    ScheduleTimeout {
        attempt_id: ProviderAuthAttemptId,
        timeout: Duration,
    },
    RefreshProviderStatus {
        attempt_id: ProviderAuthAttemptId,
        target: ApiKeyAuthTarget,
    },
    Complete(ProviderAuthCompletion),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAuthDisposition {
    Applied,
    IgnoredStale,
    Rejected(ProviderAuthRejectionReason),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProviderAuthTransition {
    pub snapshot: ProviderAuthFlowSnapshot,
    pub effects: Vec<ProviderAuthEffect>,
    pub disposition: ProviderAuthDisposition,
}

pub(crate) type Reduction = (Vec<ProviderAuthEffect>, ProviderAuthDisposition);

pub struct ProviderAuthController {
    pub(crate) snapshot: ProviderAuthFlowSnapshot,
    input: Option<ApiKeySecret>,
    pub(crate) claude_input: Option<ClaudeManagedTokenSecret>,
    pub(crate) next_attempt_id: u64,
    pub(crate) next_claude_process_id: u64,
}

impl Default for ProviderAuthController {
    fn default() -> Self {
        Self {
            snapshot: ProviderAuthFlowSnapshot::Idle,
            input: None,
            claude_input: None,
            next_attempt_id: 1,
            next_claude_process_id: 1,
        }
    }
}

impl ProviderAuthController {
    pub fn snapshot(&self) -> &ProviderAuthFlowSnapshot {
        &self.snapshot
    }

    pub fn dispatch(&mut self, action: ProviderAuthAction) -> ProviderAuthTransition {
        let (effects, disposition) = match action {
            ProviderAuthAction::OpenAiAccount(action) => self.openai_account(action),
            ProviderAuthAction::ClaudeAccount(action) => self.claude_account(action),
            ProviderAuthAction::StartApiKey(start) => self.start(start),
            ProviderAuthAction::SetApiKey(secret) => self.set_input(secret),
            ProviderAuthAction::Submit => self.submit(),
            ProviderAuthAction::Cancel => self.cancel(),
            ProviderAuthAction::Retry => self.retry(),
            ProviderAuthAction::PersistenceFinished { attempt_id, result } => {
                self.persistence_finished(attempt_id, result)
            }
            ProviderAuthAction::TimeoutElapsed { attempt_id } => self.timeout(attempt_id),
            ProviderAuthAction::StatusResolved { attempt_id, status } => {
                self.status_resolved(attempt_id, status)
            }
        };
        ProviderAuthTransition {
            snapshot: self.snapshot.clone(),
            effects,
            disposition,
        }
    }

    fn start(&mut self, start: ApiKeyFlowStart) -> Reduction {
        if commit_in_progress(&self.snapshot) {
            return rejected(ProviderAuthRejectionReason::CommitInProgress);
        }
        let ApiKeyFlowStart {
            target,
            intent,
            metadata,
        } = start;
        let flow = ApiKeyFlowContext { target, intent };
        self.input = None;
        self.snapshot = match metadata.environment {
            EnvironmentCredentialMetadata::Missing => ProviderAuthFlowSnapshot::Entering {
                flow,
                has_input: false,
            },
            environment => ProviderAuthFlowSnapshot::Blocked {
                flow,
                reason: match environment {
                    EnvironmentCredentialMetadata::Present => {
                        ProviderAuthBlockedReason::EnvironmentCredentialPresent
                    }
                    EnvironmentCredentialMetadata::Invalid => {
                        ProviderAuthBlockedReason::InvalidEnvironmentCredential
                    }
                    EnvironmentCredentialMetadata::Missing => unreachable!(),
                },
            },
        };
        applied()
    }

    fn set_input(&mut self, secret: ApiKeySecret) -> Reduction {
        let ProviderAuthFlowSnapshot::Entering { has_input, .. } = &mut self.snapshot else {
            return rejected(ProviderAuthRejectionReason::InvalidState);
        };
        *has_input = !secret.expose_secret().trim().is_empty();
        self.input = Some(secret);
        applied()
    }

    fn submit(&mut self) -> Reduction {
        let ProviderAuthFlowSnapshot::Entering { flow, .. } = &self.snapshot else {
            return rejected_for(&self.snapshot);
        };
        let flow = flow.clone();
        let Some(secret) = self
            .input
            .take()
            .filter(|secret| !secret.expose_secret().trim().is_empty())
        else {
            self.fail(flow, ProviderAuthFailureReason::EmptyCredential);
            return applied();
        };
        let Some(attempt_id) = self.allocate_attempt() else {
            self.fail(flow, ProviderAuthFailureReason::AttemptIdExhausted);
            return applied();
        };
        self.snapshot = ProviderAuthFlowSnapshot::Settling {
            flow: flow.clone(),
            attempt_id,
        };
        (
            vec![
                ProviderAuthEffect::PersistApiKey {
                    attempt_id,
                    target: flow.target,
                    secret,
                },
                ProviderAuthEffect::ScheduleTimeout {
                    attempt_id,
                    timeout: API_KEY_AUTH_TIMEOUT,
                },
            ],
            ProviderAuthDisposition::Applied,
        )
    }

    fn cancel(&mut self) -> Reduction {
        let target = match &self.snapshot {
            ProviderAuthFlowSnapshot::Entering { flow, .. }
            | ProviderAuthFlowSnapshot::Blocked { flow, .. } => flow.target.clone(),
            snapshot => return rejected_for(snapshot),
        };
        self.input = None;
        self.snapshot = ProviderAuthFlowSnapshot::Cancelled {
            target: target.clone(),
        };
        (
            vec![ProviderAuthEffect::Complete(
                ProviderAuthCompletion::Cancelled { target },
            )],
            ProviderAuthDisposition::Applied,
        )
    }

    fn retry(&mut self) -> Reduction {
        let ProviderAuthFlowSnapshot::Failed { flow, .. } = &self.snapshot else {
            return rejected_for(&self.snapshot);
        };
        self.snapshot = ProviderAuthFlowSnapshot::Entering {
            flow: flow.clone(),
            has_input: false,
        };
        self.input = None;
        applied()
    }

    fn persistence_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        result: ApiKeyPersistenceResult,
    ) -> Reduction {
        let (ProviderAuthFlowSnapshot::Settling {
            flow,
            attempt_id: current,
        }
        | ProviderAuthFlowSnapshot::OutcomeUnknown {
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
        let reason = match result {
            ApiKeyPersistenceResult::Stored => {
                self.snapshot = ProviderAuthFlowSnapshot::Reconciling {
                    flow: flow.clone(),
                    attempt_id,
                };
                return (
                    vec![ProviderAuthEffect::RefreshProviderStatus {
                        attempt_id,
                        target: flow.target,
                    }],
                    ProviderAuthDisposition::Applied,
                );
            }
            ApiKeyPersistenceResult::Rejected => ProviderAuthFailureReason::PersistenceRejected,
            ApiKeyPersistenceResult::StorageUnavailable => {
                ProviderAuthFailureReason::StorageUnavailable
            }
        };
        self.fail(flow, reason);
        applied()
    }

    fn timeout(&mut self, attempt_id: ProviderAuthAttemptId) -> Reduction {
        let ProviderAuthFlowSnapshot::Settling {
            flow,
            attempt_id: current,
        } = &self.snapshot
        else {
            return stale();
        };
        if *current != attempt_id {
            return stale();
        }
        let flow = flow.clone();
        self.snapshot = ProviderAuthFlowSnapshot::OutcomeUnknown {
            flow: flow.clone(),
            attempt_id,
        };
        (
            vec![ProviderAuthEffect::RefreshProviderStatus {
                attempt_id,
                target: flow.target,
            }],
            ProviderAuthDisposition::Applied,
        )
    }

    fn status_resolved(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        status: ProviderStatusSnapshot,
    ) -> Reduction {
        let Some((flow, current)) = active(&self.snapshot) else {
            return stale();
        };
        if current != attempt_id || status.id != flow.target.provider_id {
            return stale();
        }
        match status.configuration {
            ProviderConfigurationState::Configured => {
                let replacement_is_unsettled = matches!(
                    self.snapshot,
                    ProviderAuthFlowSnapshot::OutcomeUnknown { .. }
                ) && flow.intent == ApiKeyFlowIntent::Replace;
                if replacement_is_unsettled {
                    return applied();
                }
                let target = flow.target;
                self.snapshot = ProviderAuthFlowSnapshot::Configured {
                    target: target.clone(),
                    status: status.clone(),
                };
                (
                    vec![ProviderAuthEffect::Complete(
                        ProviderAuthCompletion::Configured { target, status },
                    )],
                    ProviderAuthDisposition::Applied,
                )
            }
            ProviderConfigurationState::NotConfigured => {
                if matches!(self.snapshot, ProviderAuthFlowSnapshot::Reconciling { .. }) {
                    self.fail(flow, ProviderAuthFailureReason::StatusNotConfigured);
                }
                applied()
            }
            ProviderConfigurationState::Checking
            | ProviderConfigurationState::Unavailable
            | ProviderConfigurationState::RecoveryRequired => applied(),
        }
    }

    fn fail(&mut self, flow: ApiKeyFlowContext, reason: ProviderAuthFailureReason) {
        self.snapshot = ProviderAuthFlowSnapshot::Failed { flow, reason };
    }

    pub(crate) fn clear_api_key_input(&mut self) {
        self.input = None;
    }

    pub(crate) fn clear_claude_input(&mut self) {
        self.claude_input = None;
    }

    pub(crate) fn allocate_attempt(&mut self) -> Option<ProviderAuthAttemptId> {
        let next_attempt_id = self.next_attempt_id.checked_add(1)?;
        let attempt_id = ProviderAuthAttemptId(self.next_attempt_id);
        self.next_attempt_id = next_attempt_id;
        Some(attempt_id)
    }
}

fn active(
    snapshot: &ProviderAuthFlowSnapshot,
) -> Option<(ApiKeyFlowContext, ProviderAuthAttemptId)> {
    match snapshot {
        ProviderAuthFlowSnapshot::Settling { flow, attempt_id }
        | ProviderAuthFlowSnapshot::Reconciling { flow, attempt_id }
        | ProviderAuthFlowSnapshot::OutcomeUnknown { flow, attempt_id } => {
            Some((flow.clone(), *attempt_id))
        }
        _ => None,
    }
}

fn rejected_for(snapshot: &ProviderAuthFlowSnapshot) -> Reduction {
    rejected(if commit_in_progress(snapshot) {
        ProviderAuthRejectionReason::CommitInProgress
    } else {
        ProviderAuthRejectionReason::InvalidState
    })
}

pub(crate) fn commit_in_progress(snapshot: &ProviderAuthFlowSnapshot) -> bool {
    active(snapshot).is_some()
        || matches!(snapshot, ProviderAuthFlowSnapshot::OpenAiAccount(state) if state.is_in_flight())
        || matches!(snapshot, ProviderAuthFlowSnapshot::ClaudeAccount(state) if state.is_in_flight())
}

fn applied() -> Reduction {
    (Vec::new(), ProviderAuthDisposition::Applied)
}

fn rejected(reason: ProviderAuthRejectionReason) -> Reduction {
    (Vec::new(), ProviderAuthDisposition::Rejected(reason))
}

fn stale() -> Reduction {
    (Vec::new(), ProviderAuthDisposition::IgnoredStale)
}

#[cfg(test)]
#[path = "auth_flow_tests.rs"]
mod tests;
