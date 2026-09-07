use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderManagementAction;

use super::App;

impl App {
    pub(super) fn provider_manager_authentication_cancelled(
        &mut self,
        provider_id: codex_provider_auth::ProviderCatalogId,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let transition =
            host.dispatch(ProviderManagementAction::AuthenticationCancelled { provider_id });
        self.apply_provider_management_transition(transition);
    }

    pub(super) fn provider_manager_api_key_finished(
        &mut self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target: codex_provider_auth::ApiKeyAuthTarget,
        status: Option<codex_provider_auth::ProviderStatusSnapshot>,
    ) {
        let matches = self
            .provider_management_host
            .as_ref()
            .and_then(super::super::provider_management_host::ProviderManagementHost::authenticating_attempt)
            .is_some_and(|(expected, provider)| {
                expected == attempt_id && provider == target.provider_id
            });
        if !matches {
            return;
        }
        let configured = status.is_some_and(|status| {
            status.id == target.provider_id
                && status.configuration == ProviderConfigurationState::Configured
        });
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let action = if configured {
            ProviderManagementAction::AuthenticationConfigured {
                provider_id: target.provider_id,
            }
        } else {
            self.chat_widget
                .add_error_message("Provider credential could not be stored.".to_string());
            ProviderManagementAction::AuthenticationCancelled {
                provider_id: target.provider_id,
            }
        };
        let transition = host.dispatch(action);
        self.apply_provider_management_transition(transition);
    }

    pub(super) fn handle_provider_manager_auth_action(
        &mut self,
        action: codex_provider_auth::ProviderAuthAction,
    ) {
        let presentations = {
            let Some(host) = self.provider_management_host.as_mut() else {
                return;
            };
            host.account_auth_host().dispatch(action)
        };
        for presentation in presentations {
            self.present_provider_manager_auth(presentation);
        }
    }

    fn present_provider_manager_auth(
        &mut self,
        presentation: crate::provider_account_auth_host::ProviderAccountPresentation,
    ) {
        use crate::provider_account_auth_host::ProviderAccountPresentation as Presentation;
        match presentation {
            Presentation::Pending(kind) => self.chat_widget.open_shared_account_pending(kind),
            Presentation::OpenAiChallenge { challenge } => {
                self.chat_widget.open_shared_openai_challenge(challenge);
            }
            Presentation::ClaudeMethodChoice { recovery } => {
                self.chat_widget.open_shared_claude_method_choice(recovery);
            }
            Presentation::ClaudeManagedTokenEntry => {
                self.chat_widget.open_shared_claude_managed_token_entry();
            }
            Presentation::ClaudeChallenge { challenge } => {
                self.chat_widget.open_shared_claude_challenge(challenge);
            }
            Presentation::Completion(completion) => {
                self.apply_provider_manager_account_completion(completion);
            }
            Presentation::Failed(failure) => {
                self.chat_widget.open_shared_account_failure(failure);
            }
        }
    }

    fn apply_provider_manager_account_completion(
        &mut self,
        completion: codex_provider_auth::ProviderAuthCompletion,
    ) {
        use codex_provider_auth::ProviderAuthCompletion as Completion;
        let (provider_id, configured) = match completion {
            Completion::OpenAiAccount(
                codex_provider_auth::OpenAiAccountCompletion::Configured { target, status },
            ) => (
                target.provider_id,
                status.configuration == ProviderConfigurationState::Configured,
            ),
            Completion::ClaudeAccount(
                codex_provider_auth::claude_account_flow::ClaudeAccountCompletion::Configured {
                    target,
                    status,
                },
            ) => (
                target.provider_id,
                status.configuration == ProviderConfigurationState::Configured,
            ),
            Completion::OpenAiAccount(
                codex_provider_auth::OpenAiAccountCompletion::Cancelled { target },
            ) => (target.provider_id, false),
            Completion::ClaudeAccount(
                codex_provider_auth::claude_account_flow::ClaudeAccountCompletion::Cancelled {
                    target,
                },
            ) => (target.provider_id, false),
            _ => return,
        };
        let expected = self
            .provider_management_host
            .as_ref()
            .and_then(|host| host.authenticating_provider().cloned());
        if expected.as_ref() != Some(&provider_id) {
            return;
        }
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let action = if configured {
            ProviderManagementAction::AuthenticationConfigured { provider_id }
        } else {
            ProviderManagementAction::AuthenticationCancelled { provider_id }
        };
        let transition = host.dispatch(action);
        self.apply_provider_management_transition(transition);
    }
}
