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
                host.begin_account_credential_attempt(
                    format!("turn:{}:{}", n.thread_id, n.turn.id),
                    &self.config.model_provider_id,
                    self.selected_named_account(),
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
        if let Some(rejected) = rejected {
            let provider = rejected.provider;
            let name = host
                .catalog()
                .get(&provider)
                .map(|entry| entry.display_name.as_str())
                .unwrap_or(&provider);
            let warning = match &rejected.account {
                Some(account) => named_account_rejection_warning(
                    name,
                    account,
                    self.config.model_providers.get(&account.provider_id),
                ),
                None => format!(
                    "{name} ({provider}) credential was rejected. Open /providers, select {name}, and press r to recover. Other providers are unchanged."
                ),
            };
            self.model_catalog.refresh_provider_policy();
            self.on_warning(warning);
        }
    }

    /// PF-84: the named account the current provider runs on, if any.
    fn selected_named_account(&self) -> Option<codex_provider_auth::NamedCredentialAccount> {
        let account = self
            .config
            .model_providers
            .get(&self.config.model_provider_id)?
            .account
            .as_ref()?;
        Some(codex_provider_auth::NamedCredentialAccount {
            provider_id: account.provider_id.clone(),
            name: account.name.clone(),
        })
    }
}

/// PF-84 (#416): `/providers` recovery manages the default credential, so a
/// rejected named account is recovered with `corbanu account add` instead.
fn named_account_rejection_warning(
    display_name: &str,
    account: &codex_provider_auth::NamedCredentialAccount,
    provider: Option<&codex_model_provider_info::ModelProviderInfo>,
) -> String {
    let codex_provider_auth::NamedCredentialAccount {
        provider_id: id,
        name,
    } = account;
    let recovery = if provider
        .is_some_and(codex_model_provider_info::ModelProviderInfo::is_claude_plan)
    {
        format!(
            "Replace its token with `corbanu account add {id} {name} --kind claude-token` (reads stdin)"
        )
    } else if provider.is_some_and(|provider| provider.auth.is_some()) {
        format!("Check what the provider's auth command returns for account `{name}`")
    } else {
        format!("Replace its key with `corbanu account add {id} {name}` (reads stdin)")
    };
    format!(
        "{display_name} ({id}) account `{name}` was rejected. {recovery}, or choose another account. The default {display_name} credential is unchanged."
    )
}

#[cfg(test)]
#[path = "provider_health_tests.rs"]
mod tests;
