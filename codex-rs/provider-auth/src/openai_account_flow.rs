use std::fmt;

use zeroize::Zeroizing;

use crate::ProviderAuthAttemptId;
use crate::ProviderAuthController;
use crate::ProviderAuthFlowSnapshot;
use crate::ProviderCatalogEntry;
use crate::ProviderCatalogId;
use crate::ProviderCredentialSource;
use crate::ProviderMethodState;
use crate::ProviderRuntimeId;
use crate::ProviderSetupCapability;
use crate::ProviderStatusSnapshot;
use crate::auth_flow::Reduction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountMethod {
    Browser,
    DeviceCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountLoginContext {
    PrimaryAuth,
    ProviderEnrollment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountTargetError {
    UnsupportedCapability,
    MissingRuntimeProvider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiAccountTarget {
    pub provider_id: ProviderCatalogId,
    pub runtime_provider_id: ProviderRuntimeId,
}

impl OpenAiAccountTarget {
    pub fn from_catalog_entry(
        entry: &ProviderCatalogEntry,
    ) -> Result<Self, OpenAiAccountTargetError> {
        if !entry
            .setup_capabilities
            .iter()
            .any(|capability| matches!(capability, ProviderSetupCapability::OpenAiAccount))
        {
            return Err(OpenAiAccountTargetError::UnsupportedCapability);
        }
        let runtime_provider_id = entry
            .runtime_provider_ids
            .first()
            .cloned()
            .ok_or(OpenAiAccountTargetError::MissingRuntimeProvider)?;
        Ok(Self {
            provider_id: entry.id.clone(),
            runtime_provider_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiAccountFlow {
    pub target: OpenAiAccountTarget,
    pub method: OpenAiAccountMethod,
    pub context: OpenAiAccountLoginContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiAccountFlowStart {
    pub target: OpenAiAccountTarget,
    pub method: OpenAiAccountMethod,
    pub context: OpenAiAccountLoginContext,
    pub status: ProviderStatusSnapshot,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpenAiAccountLoginId(String);

impl OpenAiAccountLoginId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for OpenAiAccountLoginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("OpenAiAccountLoginId")
            .field(&self.0)
            .finish()
    }
}

#[derive(PartialEq, Eq)]
pub enum OpenAiAccountChallenge {
    Browser {
        auth_url: Zeroizing<String>,
    },
    DeviceCode {
        verification_url: Zeroizing<String>,
        user_code: Zeroizing<String>,
    },
}

impl OpenAiAccountChallenge {
    pub fn browser(auth_url: impl Into<String>) -> Self {
        Self::Browser {
            auth_url: Zeroizing::new(auth_url.into()),
        }
    }

    pub fn device_code(verification_url: impl Into<String>, user_code: impl Into<String>) -> Self {
        Self::DeviceCode {
            verification_url: Zeroizing::new(verification_url.into()),
            user_code: Zeroizing::new(user_code.into()),
        }
    }

    pub fn browser_auth_url(&self) -> Option<&str> {
        match self {
            Self::Browser { auth_url } => Some(auth_url),
            Self::DeviceCode { .. } => None,
        }
    }

    pub fn device_code_values(&self) -> Option<(&str, &str)> {
        match self {
            Self::Browser { .. } => None,
            Self::DeviceCode {
                verification_url,
                user_code,
            } => Some((verification_url, user_code)),
        }
    }

    pub(crate) fn method(&self) -> OpenAiAccountMethod {
        match self {
            Self::Browser { .. } => OpenAiAccountMethod::Browser,
            Self::DeviceCode { .. } => OpenAiAccountMethod::DeviceCode,
        }
    }
}

impl fmt::Debug for OpenAiAccountChallenge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Browser { .. } => {
                formatter.write_str("OpenAiAccountChallenge::Browser(<redacted>)")
            }
            Self::DeviceCode { .. } => {
                formatter.write_str("OpenAiAccountChallenge::DeviceCode(<redacted>)")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountBlockedReason {
    ExternallyManaged,
    BrowserUnavailableForProviderEnrollment,
    StatusIdentityMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountFailureReason {
    StartRejected,
    ProtocolMismatch,
    LoginRejected,
    StatusNotConfigured,
    AttemptIdExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountCancelPurpose {
    UserRequested,
    ProtocolMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountOutcomeUnknownReason {
    StartTransportLost,
    CancelNotFound,
    CancelTransportLost,
    LoginTransportLost,
    ProtocolMismatchCancelTransportLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountRecoveryReason {
    StartOutcomeUnknown,
    LoginOutcomeUnknown,
    CancelOutcomeUnknown,
    ProtocolMismatchCancellationUnknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpenAiAccountStartResult {
    Started {
        login_id: OpenAiAccountLoginId,
        challenge: OpenAiAccountChallenge,
    },
    Rejected,
    TransportLost,
    ProtocolMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiCancelResult {
    Canceled,
    NotFound,
    TransportLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountLoginOutcome {
    Succeeded,
    Failed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpenAiAccountAction {
    Start(OpenAiAccountFlowStart),
    Cancel,
    Retry,
    StartFinished {
        attempt_id: ProviderAuthAttemptId,
        result: OpenAiAccountStartResult,
    },
    CancelFinished {
        attempt_id: ProviderAuthAttemptId,
        result: OpenAiCancelResult,
    },
    LoginCompleted {
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
        outcome: OpenAiAccountLoginOutcome,
    },
    TransportLost {
        attempt_id: ProviderAuthAttemptId,
    },
    StatusResolved {
        attempt_id: ProviderAuthAttemptId,
        status: ProviderStatusSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenAiAccountSnapshot {
    Starting {
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
    },
    CancelPendingStart {
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
    },
    AwaitingUser {
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
    },
    Cancelling {
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
        purpose: OpenAiAccountCancelPurpose,
    },
    Reconciling {
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
    },
    OutcomeUnknown {
        flow: OpenAiAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        login_id: Option<OpenAiAccountLoginId>,
        reason: OpenAiAccountOutcomeUnknownReason,
    },
    RecoveryRequired {
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountRecoveryReason,
    },
    Blocked {
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountBlockedReason,
    },
    Failed {
        flow: OpenAiAccountFlow,
        reason: OpenAiAccountFailureReason,
    },
    Configured {
        target: OpenAiAccountTarget,
        status: ProviderStatusSnapshot,
    },
    Cancelled {
        target: OpenAiAccountTarget,
    },
}

impl OpenAiAccountSnapshot {
    pub(crate) fn is_in_flight(&self) -> bool {
        matches!(
            self,
            Self::Starting { .. }
                | Self::CancelPendingStart { .. }
                | Self::AwaitingUser { .. }
                | Self::Cancelling { .. }
                | Self::Reconciling { .. }
                | Self::OutcomeUnknown { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenAiAccountCompletion {
    Configured {
        target: OpenAiAccountTarget,
        status: ProviderStatusSnapshot,
    },
    Cancelled {
        target: OpenAiAccountTarget,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpenAiAccountEffect {
    StartLogin {
        attempt_id: ProviderAuthAttemptId,
        target: OpenAiAccountTarget,
        method: OpenAiAccountMethod,
        context: OpenAiAccountLoginContext,
    },
    PresentChallenge {
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
        challenge: OpenAiAccountChallenge,
    },
    CancelLogin {
        attempt_id: ProviderAuthAttemptId,
        login_id: OpenAiAccountLoginId,
    },
    RefreshStatus {
        attempt_id: ProviderAuthAttemptId,
        target: OpenAiAccountTarget,
    },
}

impl ProviderAuthController {
    pub(crate) fn openai_cancel_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        result: OpenAiCancelResult,
    ) -> Reduction {
        let ProviderAuthFlowSnapshot::OpenAiAccount(OpenAiAccountSnapshot::Cancelling {
            flow,
            attempt_id: current,
            login_id,
            purpose,
        }) = &self.snapshot
        else {
            return stale();
        };
        if *current != attempt_id {
            return stale();
        }
        let flow = flow.clone();
        let login_id = login_id.clone();
        let purpose = *purpose;
        match result {
            OpenAiCancelResult::Canceled => self.finish_cancel(flow, purpose),
            OpenAiCancelResult::NotFound
                if purpose == OpenAiAccountCancelPurpose::ProtocolMismatch =>
            {
                self.fail_openai(flow, OpenAiAccountFailureReason::ProtocolMismatch)
            }
            OpenAiCancelResult::NotFound => self.unknown_openai(
                flow,
                attempt_id,
                Some(login_id),
                OpenAiAccountOutcomeUnknownReason::CancelNotFound,
            ),
            OpenAiCancelResult::TransportLost => {
                let reason = match purpose {
                    OpenAiAccountCancelPurpose::UserRequested => {
                        OpenAiAccountOutcomeUnknownReason::CancelTransportLost
                    }
                    OpenAiAccountCancelPurpose::ProtocolMismatch => {
                        OpenAiAccountOutcomeUnknownReason::ProtocolMismatchCancelTransportLost
                    }
                };
                self.unknown_openai(flow, attempt_id, Some(login_id), reason)
            }
        }
    }

    pub(crate) fn openai_status_resolved(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        status: ProviderStatusSnapshot,
    ) -> Reduction {
        let state = match &self.snapshot {
            ProviderAuthFlowSnapshot::OpenAiAccount(state) => state.clone(),
            _ => return stale(),
        };
        let (flow, current, origin) = match state {
            OpenAiAccountSnapshot::Reconciling {
                flow, attempt_id, ..
            } => (flow, attempt_id, None),
            OpenAiAccountSnapshot::OutcomeUnknown {
                flow,
                attempt_id,
                reason,
                ..
            } => (flow, attempt_id, Some(reason)),
            _ => return stale(),
        };
        if current != attempt_id || status.id != flow.target.provider_id {
            return stale();
        }
        if account_method_state(&status) == AccountMethodState::ManagedAccount {
            return self.complete_openai(flow.target, status);
        }
        match origin {
            None => self.fail_openai(flow, OpenAiAccountFailureReason::StatusNotConfigured),
            Some(OpenAiAccountOutcomeUnknownReason::CancelNotFound) => {
                self.cancelled_openai(flow.target)
            }
            Some(reason) => self.recovery_openai(flow, recovery_reason(reason)),
        }
    }
}

fn recovery_reason(reason: OpenAiAccountOutcomeUnknownReason) -> OpenAiAccountRecoveryReason {
    match reason {
        OpenAiAccountOutcomeUnknownReason::StartTransportLost => {
            OpenAiAccountRecoveryReason::StartOutcomeUnknown
        }
        OpenAiAccountOutcomeUnknownReason::LoginTransportLost => {
            OpenAiAccountRecoveryReason::LoginOutcomeUnknown
        }
        OpenAiAccountOutcomeUnknownReason::CancelNotFound
        | OpenAiAccountOutcomeUnknownReason::CancelTransportLost => {
            OpenAiAccountRecoveryReason::CancelOutcomeUnknown
        }
        OpenAiAccountOutcomeUnknownReason::ProtocolMismatchCancelTransportLost => {
            OpenAiAccountRecoveryReason::ProtocolMismatchCancellationUnknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccountMethodState {
    ManagedAccount,
    ExternallyManaged,
    NotConfigured,
}

pub(crate) fn account_method_state(status: &ProviderStatusSnapshot) -> AccountMethodState {
    status
        .methods
        .iter()
        .find_map(|method| {
            if !matches!(method.capability, ProviderSetupCapability::OpenAiAccount) {
                return None;
            }
            Some(match method.state {
                ProviderMethodState::Configured {
                    source: ProviderCredentialSource::OpenAiAccount,
                    ..
                } => AccountMethodState::ManagedAccount,
                ProviderMethodState::Configured {
                    source: ProviderCredentialSource::ExternallyManaged,
                    ..
                } => AccountMethodState::ExternallyManaged,
                _ => AccountMethodState::NotConfigured,
            })
        })
        .unwrap_or(AccountMethodState::NotConfigured)
}

pub(crate) fn stale() -> Reduction {
    (Vec::new(), crate::ProviderAuthDisposition::IgnoredStale)
}

#[cfg(test)]
#[path = "openai_account_flow_tests.rs"]
mod tests;
