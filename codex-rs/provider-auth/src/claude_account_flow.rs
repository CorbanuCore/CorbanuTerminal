use std::fmt;
use std::time::Duration;

use zeroize::Zeroizing;

use crate::ProviderAuthAttemptId;
use crate::ProviderCatalogEntry;
use crate::ProviderCatalogId;
use crate::ProviderCredentialSource;
use crate::ProviderMethodState;
use crate::ProviderRuntimeId;
use crate::ProviderSetupCapability;
use crate::ProviderStatusSnapshot;

pub const CLAUDE_MANAGED_AUTH_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeAccountMethod {
    ManagedToken,
    ClaudeCodeLogin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeAccountIntent {
    Add,
    Replace,
    UnauthorizedRecovery {
        source: ClaudeUnauthorizedRecoverySource,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeUnauthorizedRecoverySource {
    ManagedToken,
    Environment,
    ClaudeCodeLogin,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeAccountTargetError {
    UnsupportedCapability,
    MissingRuntimeProvider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeAccountTarget {
    pub provider_id: ProviderCatalogId,
    pub runtime_provider_id: ProviderRuntimeId,
}

impl ClaudeAccountTarget {
    pub fn from_catalog_entry(
        entry: &ProviderCatalogEntry,
    ) -> Result<Self, ClaudeAccountTargetError> {
        if !entry
            .setup_capabilities
            .iter()
            .any(|capability| matches!(capability, ProviderSetupCapability::ClaudeAccount))
        {
            return Err(ClaudeAccountTargetError::UnsupportedCapability);
        }
        let runtime_provider_id = entry
            .runtime_provider_ids
            .first()
            .cloned()
            .ok_or(ClaudeAccountTargetError::MissingRuntimeProvider)?;
        Ok(Self {
            provider_id: entry.id.clone(),
            runtime_provider_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeAccountFlow {
    pub target: ClaudeAccountTarget,
    pub intent: ClaudeAccountIntent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeAccountFlowStart {
    pub target: ClaudeAccountTarget,
    pub intent: ClaudeAccountIntent,
    pub status: ProviderStatusSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaudeCodeProcessId(pub(crate) u64);

#[derive(PartialEq, Eq)]
pub struct ClaudeManagedTokenSecret(Zeroizing<String>);

impl ClaudeManagedTokenSecret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> Zeroizing<String> {
        self.0
    }
}

impl fmt::Debug for ClaudeManagedTokenSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ClaudeManagedTokenSecret(<redacted>)")
    }
}

#[derive(PartialEq, Eq)]
pub struct ClaudeAuthorizationCodeSecret(Zeroizing<String>);

impl ClaudeAuthorizationCodeSecret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Zeroizing::new(value.into()))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_inner(self) -> Zeroizing<String> {
        self.0
    }
}

impl fmt::Debug for ClaudeAuthorizationCodeSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ClaudeAuthorizationCodeSecret(<redacted>)")
    }
}

#[derive(PartialEq, Eq)]
pub struct ClaudeCodeChallenge(Zeroizing<String>);

impl ClaudeCodeChallenge {
    pub fn new(verification_url: impl Into<String>) -> Self {
        Self(Zeroizing::new(verification_url.into()))
    }

    pub fn verification_url(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for ClaudeCodeChallenge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ClaudeCodeChallenge(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCodeIdentityPolicy {
    AllowExplicitChange,
    PreserveSelected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeAccountBlockedReason {
    ExternallyManagedEnvironment,
    StatusIdentityMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeAccountRecoveryReason {
    AmbiguousSources,
    MissingSelection,
    UnhealthySelection,
    ManagedOutcomeUnknown,
    LoginOutcomeUnknown,
    CancelOutcomeUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeAccountFailureReason {
    EmptyCredential,
    InvalidManagedToken,
    StorageUnavailable,
    LoginUnavailable,
    LoginRejected,
    LoginTimedOut,
    IdentityConflict,
    StatusNotConfigured,
    AttemptIdExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeManagedTokenResult {
    Stored,
    Invalid,
    StorageUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeExistingLoginResult {
    Selected,
    LoginRequired,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCodeLoginOutcome {
    Succeeded,
    Cancelled,
    Rejected,
    TimedOut,
    IdentityConflict,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClaudeAccountAction {
    Start(ClaudeAccountFlowStart),
    ChooseMethod(ClaudeAccountMethod),
    SetManagedToken(ClaudeManagedTokenSecret),
    Submit,
    SubmitAuthorizationCode(ClaudeAuthorizationCodeSecret),
    Cancel,
    KeepCurrent,
    Retry,
    ManagedTokenFinished {
        attempt_id: ProviderAuthAttemptId,
        result: ClaudeManagedTokenResult,
    },
    ManagedTimeoutElapsed {
        attempt_id: ProviderAuthAttemptId,
    },
    ExistingLoginChecked {
        attempt_id: ProviderAuthAttemptId,
        result: ClaudeExistingLoginResult,
    },
    ClaudeCodeReady {
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        challenge: ClaudeCodeChallenge,
    },
    ClaudeCodeFinished {
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        outcome: ClaudeCodeLoginOutcome,
    },
    BackendTransportLost {
        attempt_id: ProviderAuthAttemptId,
        process_id: Option<ClaudeCodeProcessId>,
    },
    StatusResolved {
        attempt_id: ProviderAuthAttemptId,
        status: ProviderStatusSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeAccountSnapshot {
    ChoosingMethod {
        flow: ClaudeAccountFlow,
        recommended: Option<ClaudeAccountMethod>,
    },
    EnteringManagedToken {
        flow: ClaudeAccountFlow,
        has_input: bool,
    },
    ReadyClaudeCodeLogin {
        flow: ClaudeAccountFlow,
    },
    CheckingExistingLogin {
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
    },
    SettlingManagedToken {
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
    },
    StartingClaudeCodeLogin {
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
    },
    AwaitingAuthorizationCode {
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
    },
    Authenticating {
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
    },
    Cancelling {
        flow: ClaudeAccountFlow,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
    },
    Reconciling {
        flow: ClaudeAccountFlow,
        method: ClaudeAccountMethod,
        attempt_id: ProviderAuthAttemptId,
        process_id: Option<ClaudeCodeProcessId>,
    },
    OutcomeUnknown {
        flow: ClaudeAccountFlow,
        method: ClaudeAccountMethod,
        attempt_id: ProviderAuthAttemptId,
        process_id: Option<ClaudeCodeProcessId>,
        correlated_success: bool,
        cancel_requested: bool,
    },
    RecoveryRequired {
        flow: ClaudeAccountFlow,
        reason: ClaudeAccountRecoveryReason,
    },
    Blocked {
        flow: ClaudeAccountFlow,
        reason: ClaudeAccountBlockedReason,
    },
    Failed {
        flow: ClaudeAccountFlow,
        method: Option<ClaudeAccountMethod>,
        reason: ClaudeAccountFailureReason,
    },
    Configured {
        target: ClaudeAccountTarget,
        status: ProviderStatusSnapshot,
    },
    Cancelled {
        target: ClaudeAccountTarget,
    },
}

impl ClaudeAccountSnapshot {
    pub(crate) fn is_in_flight(&self) -> bool {
        matches!(
            self,
            Self::CheckingExistingLogin { .. }
                | Self::SettlingManagedToken { .. }
                | Self::StartingClaudeCodeLogin { .. }
                | Self::AwaitingAuthorizationCode { .. }
                | Self::Authenticating { .. }
                | Self::Cancelling { .. }
                | Self::Reconciling { .. }
                | Self::OutcomeUnknown { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeAccountCompletion {
    Configured {
        target: ClaudeAccountTarget,
        status: ProviderStatusSnapshot,
    },
    Cancelled {
        target: ClaudeAccountTarget,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClaudeAccountEffect {
    EnrollManagedToken {
        attempt_id: ProviderAuthAttemptId,
        target: ClaudeAccountTarget,
        secret: ClaudeManagedTokenSecret,
    },
    ScheduleManagedTimeout {
        attempt_id: ProviderAuthAttemptId,
        timeout: Duration,
    },
    CheckExistingClaudeCodeLogin {
        attempt_id: ProviderAuthAttemptId,
        target: ClaudeAccountTarget,
    },
    StartClaudeCodeLogin {
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        target: ClaudeAccountTarget,
        identity_policy: ClaudeCodeIdentityPolicy,
    },
    PresentChallenge {
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        challenge: ClaudeCodeChallenge,
    },
    SendAuthorizationCode {
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        secret: ClaudeAuthorizationCodeSecret,
    },
    CancelClaudeCodeLogin {
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
    },
    RefreshStatus {
        attempt_id: ProviderAuthAttemptId,
        target: ClaudeAccountTarget,
    },
}

pub(crate) fn configured_claude_method(
    status: &ProviderStatusSnapshot,
) -> Option<ClaudeAccountMethod> {
    status.methods.iter().find_map(|method| {
        if method.capability != ProviderSetupCapability::ClaudeAccount {
            return None;
        }
        match method.state {
            ProviderMethodState::Configured {
                source: ProviderCredentialSource::ClaudeManaged,
                ..
            } => Some(ClaudeAccountMethod::ManagedToken),
            ProviderMethodState::Configured {
                source: ProviderCredentialSource::ClaudeCodeLogin,
                ..
            } => Some(ClaudeAccountMethod::ClaudeCodeLogin),
            _ => None,
        }
    })
}

#[cfg(test)]
#[path = "claude_account_flow_tests.rs"]
mod tests;
