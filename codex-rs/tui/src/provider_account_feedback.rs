//! Maps typed failures to safe messages; raw backend errors never reach this UI.
use crate::provider_account_auth_host::ProviderAccountCancelKind;
use codex_provider_auth::OpenAiAccountBlockedReason as OB;
use codex_provider_auth::OpenAiAccountFailureReason as OF;
use codex_provider_auth::OpenAiAccountRecoveryReason as OR;
use codex_provider_auth::OpenAiAccountSnapshot as OS;
use codex_provider_auth::ProviderAuthFlowSnapshot;
use codex_provider_auth::claude_account_flow::ClaudeAccountBlockedReason as CB;
use codex_provider_auth::claude_account_flow::ClaudeAccountFailureReason as CF;
use codex_provider_auth::claude_account_flow::ClaudeAccountRecoveryReason as CR;
use codex_provider_auth::claude_account_flow::ClaudeAccountSnapshot as CS;

pub(crate) struct AccountFailure {
    pub(crate) message: &'static str,
    pub(crate) kind: ProviderAccountCancelKind,
    pub(crate) retry: bool,
}

pub(crate) fn account_failure(snapshot: &ProviderAuthFlowSnapshot) -> Option<AccountFailure> {
    let (message, kind, retry) = match snapshot {
        ProviderAuthFlowSnapshot::OpenAiAccount(OS::Failed { reason, .. }) => (
            match reason {
                OF::StartRejected => {
                    "OpenAI could not start login. Check your connection and retry."
                }
                OF::ProtocolMismatch => {
                    "The login server returned an incompatible response. Update Corbanu and retry."
                }
                OF::LoginRejected => {
                    "OpenAI did not accept the login. Retry and finish signing in in your browser."
                }
                OF::StatusNotConfigured => {
                    "Login finished, but no usable OpenAI credential was found. Retry sign-in."
                }
                OF::AttemptIdExhausted => {
                    "This login session cannot start another attempt. Restart Corbanu."
                }
            },
            ProviderAccountCancelKind::OpenAi,
            *reason != OF::AttemptIdExhausted,
        ),
        ProviderAuthFlowSnapshot::OpenAiAccount(OS::Blocked { reason, .. }) => (
            match reason {
                OB::ExternallyManaged => {
                    "OpenAI authentication is externally managed. Update the credential at its source, then reopen Providers."
                }
                OB::BrowserUnavailableForProviderEnrollment => {
                    "Browser login is unavailable for provider enrollment. Return to Providers and use device-code login."
                }
                OB::StatusIdentityMismatch => {
                    "The returned account does not match the selected provider. Return to Providers and select the intended account."
                }
            },
            ProviderAccountCancelKind::OpenAi,
            false,
        ),
        ProviderAuthFlowSnapshot::OpenAiAccount(OS::RecoveryRequired { reason, .. }) => (
            match reason {
                OR::StartOutcomeUnknown => {
                    "The connection was lost while starting OpenAI login. Its outcome could not be confirmed. Retry to start a new attempt."
                }
                OR::LoginOutcomeUnknown => {
                    "The OpenAI login result could not be confirmed. Check your account and retry."
                }
                OR::CancelOutcomeUnknown | OR::ProtocolMismatchCancellationUnknown => {
                    "OpenAI login cancellation could not be confirmed. Check your account before retrying."
                }
            },
            ProviderAccountCancelKind::OpenAi,
            true,
        ),
        ProviderAuthFlowSnapshot::ClaudeAccount(CS::Failed { reason, .. }) => (
            match reason {
                CF::EmptyCredential => {
                    "The subscription token was empty. Retry and paste the token from claude setup-token."
                }
                CF::InvalidManagedToken => {
                    "The subscription token was not accepted. Run claude setup-token in a private terminal, then retry with its token."
                }
                CF::StorageUnavailable => {
                    "The subscription token could not be stored securely. Check that your credential store is available, then retry."
                }
                CF::LoginUnavailable => {
                    "Claude Code login is unavailable. Install or sign in to Claude Code, then retry, or choose a subscription token."
                }
                CF::LoginRejected => {
                    "Claude did not accept the login. Retry and complete browser authorization."
                }
                CF::LoginTimedOut => {
                    "Claude login timed out. Retry to start a fresh browser login."
                }
                CF::IdentityConflict => {
                    "The Claude account conflicts with the selected identity. Return to Providers and explicitly choose the intended account."
                }
                CF::StatusNotConfigured => {
                    "Login finished, but no usable Claude credential was found. Retry or choose another authentication method."
                }
                CF::AttemptIdExhausted => {
                    "This login session cannot start another attempt. Restart Corbanu."
                }
            },
            ProviderAccountCancelKind::Claude,
            *reason != CF::AttemptIdExhausted,
        ),
        ProviderAuthFlowSnapshot::ClaudeAccount(CS::Blocked { reason, .. }) => (
            match reason {
                CB::ExternallyManagedEnvironment => {
                    "Claude authentication is supplied by the environment. Update or unset it outside Corbanu before selecting a managed credential."
                }
                CB::StatusIdentityMismatch => {
                    "The returned account does not match the selected Claude provider. Return to Providers and select the intended account."
                }
            },
            ProviderAccountCancelKind::Claude,
            false,
        ),
        _ => return None,
    };
    Some(AccountFailure {
        message,
        kind,
        retry,
    })
}

pub(crate) fn claude_recovery_message(reason: CR) -> &'static str {
    match reason {
        CR::AmbiguousSources => {
            "More than one Claude credential was found. Explicitly choose the source to use."
        }
        CR::MissingSelection => {
            "The selected Claude credential is missing. Choose a source to restore access."
        }
        CR::UnhealthySelection => {
            "The selected Claude credential is unavailable. Choose a source to restore access."
        }
        CR::ManagedOutcomeUnknown => {
            "Token storage could not be confirmed. Check the credential store before choosing a source."
        }
        CR::LoginOutcomeUnknown => {
            "The Claude login result could not be confirmed. Check your account before choosing a source."
        }
        CR::CancelOutcomeUnknown => {
            "Claude login cancellation could not be confirmed. Check your account before choosing a source."
        }
    }
}
