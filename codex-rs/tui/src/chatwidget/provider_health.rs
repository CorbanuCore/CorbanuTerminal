//! Attribute live typed failures to the credential captured when a request began.
use codex_app_server_protocol::CodexErrorInfo;
use codex_app_server_protocol::McpServerStartupFailureReason;
use codex_app_server_protocol::McpServerStartupState;
use codex_app_server_protocol::ServerNotification;

use super::ChatWidget;

fn is_credential_rejection(info: Option<&CodexErrorInfo>) -> bool {
    matches!(
        info,
        Some(
            CodexErrorInfo::Unauthorized
                | CodexErrorInfo::HttpConnectionFailed {
                    http_status_code: Some(401)
                }
                | CodexErrorInfo::ResponseStreamConnectionFailed {
                    http_status_code: Some(401)
                }
                | CodexErrorInfo::ResponseStreamDisconnected {
                    http_status_code: Some(401)
                }
                | CodexErrorInfo::ResponseTooManyFailedAttempts {
                    http_status_code: Some(401)
                }
        )
    )
}

impl ChatWidget {
    pub(super) fn observe_provider_health(&mut self, notification: &ServerNotification) {
        let Some(policy) = self.model_catalog.provider_policy() else {
            return;
        };
        let host = policy.host();
        let rejected = match notification {
            ServerNotification::TurnStarted(n) => {
                host.begin_credential_attempt(
                    format!("turn:{}:{}", n.thread_id, n.turn.id),
                    &self.config.model_provider_id,
                );
                None
            }
            ServerNotification::Error(n)
                if !n.will_retry && is_credential_rejection(n.error.codex_error_info.as_ref()) =>
            {
                host.reject_credential_attempt(&format!("turn:{}:{}", n.thread_id, n.turn_id))
            }
            ServerNotification::TurnCompleted(n) => {
                let scope = format!("turn:{}:{}", n.thread_id, n.turn.id);
                let rejected = n.turn.error.as_ref().and_then(|error| {
                    is_credential_rejection(error.codex_error_info.as_ref())
                        .then(|| host.reject_credential_attempt(&scope))
                        .flatten()
                });
                host.finish_credential_attempt(&scope);
                rejected
            }
            ServerNotification::McpServerStatusUpdated(n) if n.name == "codex_apps" => {
                let scope = format!("mcp:{}:codex_apps", n.thread_id.as_deref().unwrap_or("app"));
                match n.status {
                    McpServerStartupState::Starting => {
                        if host.resolve_provider("openai").is_some_and(|status| status.methods.iter().any(|method| {
                            matches!(method.capability, codex_provider_auth::ProviderSetupCapability::OpenAiAccount)
                                && matches!(method.state, codex_provider_auth::ProviderMethodState::Configured { .. })
                        })) {
                            host.begin_credential_attempt(scope, "openai");
                        }
                        None
                    }
                    McpServerStartupState::Failed if n.failure_reason == Some(
                        McpServerStartupFailureReason::OpenAiAccountReauthenticationRequired
                    ) => host.reject_credential_attempt(&scope),
                    McpServerStartupState::Ready | McpServerStartupState::Failed
                    | McpServerStartupState::Stopped | McpServerStartupState::Cancelled => {
                        host.finish_credential_attempt(&scope);
                        None
                    }
                }
            }
            _ => None,
        };
        if let Some(provider) = rejected {
            let name = host
                .catalog()
                .get(&provider)
                .map(|entry| entry.display_name.as_str())
                .unwrap_or(&provider);
            let warning = format!(
                "{name} ({provider}) credential was rejected. Open /providers, select {name}, and press r to recover. Other providers are unchanged."
            );
            self.model_catalog.refresh_provider_policy();
            self.on_warning(warning);
        }
    }
}

#[cfg(test)]
#[path = "provider_health_tests.rs"]
mod tests;
