use std::collections::BTreeMap;

use codex_app_server_protocol::AccountLoginCompletedNotification;
use codex_app_server_protocol::CancelLoginAccountParams;
use codex_app_server_protocol::CancelLoginAccountResponse;
use codex_app_server_protocol::CancelLoginAccountStatus;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::LoginAccountParams;
use codex_app_server_protocol::LoginAccountResponse;
use codex_app_server_protocol::RequestId;
use codex_provider_auth::OpenAiAccountAction;
use codex_provider_auth::OpenAiAccountChallenge;
use codex_provider_auth::OpenAiAccountEffect;
use codex_provider_auth::OpenAiAccountLoginContext;
use codex_provider_auth::OpenAiAccountLoginId;
use codex_provider_auth::OpenAiAccountLoginOutcome;
use codex_provider_auth::OpenAiAccountMethod;
use codex_provider_auth::OpenAiAccountStartResult;
use codex_provider_auth::OpenAiCancelResult;
use codex_provider_auth::ProviderAuthAttemptId;

use crate::TypedRequestError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiAccountAdapterError {
    UnsupportedEffect,
    BrowserUnavailableForProviderEnrollment,
}

#[derive(Default)]
pub struct OpenAiAccountAppServerAdapter {
    pending_starts: BTreeMap<ProviderAuthAttemptId, OpenAiAccountMethod>,
    login_attempts: BTreeMap<OpenAiAccountLoginId, ProviderAuthAttemptId>,
}

impl OpenAiAccountAppServerAdapter {
    pub fn request_for_effect(
        &mut self,
        request_id: RequestId,
        effect: &OpenAiAccountEffect,
    ) -> Result<ClientRequest, OpenAiAccountAdapterError> {
        match effect {
            OpenAiAccountEffect::StartLogin {
                attempt_id,
                method,
                context,
                ..
            } => {
                let params = match (*method, *context) {
                    (OpenAiAccountMethod::Browser, OpenAiAccountLoginContext::PrimaryAuth) => {
                        LoginAccountParams::Chatgpt {
                            codex_streamlined_login: false,
                            use_hosted_login_success_page: false,
                            app_brand: None,
                        }
                    }
                    (OpenAiAccountMethod::DeviceCode, OpenAiAccountLoginContext::PrimaryAuth) => {
                        LoginAccountParams::ChatgptDeviceCode
                    }
                    (
                        OpenAiAccountMethod::DeviceCode,
                        OpenAiAccountLoginContext::ProviderEnrollment,
                    ) => LoginAccountParams::OpenaiProviderDeviceCode,
                    (
                        OpenAiAccountMethod::Browser,
                        OpenAiAccountLoginContext::ProviderEnrollment,
                    ) => {
                        return Err(
                            OpenAiAccountAdapterError::BrowserUnavailableForProviderEnrollment,
                        );
                    }
                };
                self.pending_starts.insert(*attempt_id, *method);
                Ok(ClientRequest::LoginAccount { request_id, params })
            }
            OpenAiAccountEffect::CancelLogin { login_id, .. } => {
                Ok(ClientRequest::CancelLoginAccount {
                    request_id,
                    params: CancelLoginAccountParams {
                        login_id: login_id.as_str().to_string(),
                    },
                })
            }
            OpenAiAccountEffect::PresentChallenge { .. }
            | OpenAiAccountEffect::RefreshStatus { .. } => {
                Err(OpenAiAccountAdapterError::UnsupportedEffect)
            }
        }
    }

    pub fn start_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        response: LoginAccountResponse,
    ) -> OpenAiAccountAction {
        let expected = self.pending_starts.remove(&attempt_id);
        let result = match (expected, response) {
            (Some(_), LoginAccountResponse::Chatgpt { login_id, auth_url }) => {
                let login_id = OpenAiAccountLoginId::new(login_id);
                self.login_attempts.insert(login_id.clone(), attempt_id);
                OpenAiAccountStartResult::Started {
                    login_id,
                    challenge: OpenAiAccountChallenge::browser(auth_url),
                }
            }
            (
                Some(_),
                LoginAccountResponse::ChatgptDeviceCode {
                    login_id,
                    verification_url,
                    user_code,
                },
            ) => {
                let login_id = OpenAiAccountLoginId::new(login_id);
                self.login_attempts.insert(login_id.clone(), attempt_id);
                OpenAiAccountStartResult::Started {
                    login_id,
                    challenge: OpenAiAccountChallenge::device_code(verification_url, user_code),
                }
            }
            _ => OpenAiAccountStartResult::ProtocolMismatch,
        };
        OpenAiAccountAction::StartFinished { attempt_id, result }
    }

    pub fn start_failed(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        error: &TypedRequestError,
    ) -> OpenAiAccountAction {
        self.pending_starts.remove(&attempt_id);
        let result = match error {
            TypedRequestError::Transport { .. } => OpenAiAccountStartResult::TransportLost,
            TypedRequestError::Server { .. } => OpenAiAccountStartResult::Rejected,
            TypedRequestError::Deserialize { .. } => OpenAiAccountStartResult::ProtocolMismatch,
        };
        OpenAiAccountAction::StartFinished { attempt_id, result }
    }

    pub fn cancel_finished(
        &mut self,
        attempt_id: ProviderAuthAttemptId,
        response: CancelLoginAccountResponse,
    ) -> OpenAiAccountAction {
        let result = match response.status {
            CancelLoginAccountStatus::Canceled => {
                self.forget_attempt(attempt_id);
                OpenAiCancelResult::Canceled
            }
            CancelLoginAccountStatus::NotFound => OpenAiCancelResult::NotFound,
        };
        OpenAiAccountAction::CancelFinished { attempt_id, result }
    }

    pub fn cancel_failed(
        &self,
        attempt_id: ProviderAuthAttemptId,
        _error: &TypedRequestError,
    ) -> OpenAiAccountAction {
        OpenAiAccountAction::CancelFinished {
            attempt_id,
            result: OpenAiCancelResult::TransportLost,
        }
    }

    pub fn login_completed(
        &mut self,
        notification: AccountLoginCompletedNotification,
    ) -> Option<OpenAiAccountAction> {
        let login_id = OpenAiAccountLoginId::new(notification.login_id?);
        let attempt_id = self.login_attempts.remove(&login_id)?;
        Some(OpenAiAccountAction::LoginCompleted {
            attempt_id,
            login_id,
            outcome: if notification.success {
                OpenAiAccountLoginOutcome::Succeeded
            } else {
                OpenAiAccountLoginOutcome::Failed
            },
        })
    }

    pub fn transport_lost(attempt_id: ProviderAuthAttemptId) -> OpenAiAccountAction {
        OpenAiAccountAction::TransportLost { attempt_id }
    }

    fn forget_attempt(&mut self, attempt_id: ProviderAuthAttemptId) {
        self.login_attempts
            .retain(|_, correlated_attempt| *correlated_attempt != attempt_id);
    }
}

#[cfg(test)]
#[path = "provider_auth_tests.rs"]
mod tests;
