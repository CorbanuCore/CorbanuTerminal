use std::sync::Arc;
use std::sync::Mutex;

use codex_app_server_client::AppServerRequestHandle;
use codex_app_server_client::OpenAiAccountAppServerAdapter;
use codex_app_server_protocol::CancelLoginAccountResponse;
use codex_app_server_protocol::LoginAccountResponse;
use codex_app_server_protocol::RequestId;
use codex_provider_auth::OpenAiAccountAction;
use codex_provider_auth::OpenAiAccountChallenge;
use codex_provider_auth::OpenAiAccountEffect;
use codex_provider_auth::ProviderAuthAction;
use codex_provider_auth::ProviderAuthCompletion;
use codex_provider_auth::ProviderAuthController;
use codex_provider_auth::ProviderAuthEffect;
use codex_provider_auth::ProviderAuthFlowSnapshot;
use codex_provider_auth::claude_account_flow::ClaudeAccountAction;
use codex_provider_auth::claude_account_flow::ClaudeAccountEffect;
use codex_provider_auth::claude_account_flow::ClaudeAccountSnapshot;
use codex_provider_auth::claude_account_flow::ClaudeCodeChallenge;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::chatwidget::claude_auth_adapter::ClaudeAuthAdapter;
use crate::legacy_core::config::Config;
use crate::provider_status_host::ProviderAccountMetadata;
use crate::provider_status_host::ProviderStatusHost;

pub(crate) enum ProviderAccountPresentation {
    Pending(ProviderAccountCancelKind),
    OpenAiChallenge { challenge: OpenAiAccountChallenge },
    ClaudeMethodChoice,
    ClaudeManagedTokenEntry,
    ClaudeChallenge { challenge: ClaudeCodeChallenge },
    Completion(ProviderAuthCompletion),
    Failed,
}

#[derive(Clone, Copy)]
pub(crate) enum ProviderAccountCancelKind {
    OpenAi,
    Claude,
}

impl ProviderAccountCancelKind {
    pub(crate) fn action(self) -> ProviderAuthAction {
        match self {
            Self::OpenAi => OpenAiAccountAction::Cancel.into(),
            Self::Claude => ClaudeAccountAction::Cancel.into(),
        }
    }
}

pub(crate) struct ProviderAccountAuthHost {
    controller: ProviderAuthController,
    request_handle: AppServerRequestHandle,
    openai: Arc<Mutex<OpenAiAccountAppServerAdapter>>,
    claude: ClaudeAuthAdapter,
    claude_action_tx: mpsc::UnboundedSender<ClaudeAccountAction>,
    app_event_tx: AppEventSender,
    status_host: ProviderStatusHost,
    config: Config,
}

impl ProviderAccountAuthHost {
    pub(crate) fn new(
        request_handle: AppServerRequestHandle,
        app_event_tx: AppEventSender,
        status_host: ProviderStatusHost,
        config: Config,
    ) -> Self {
        let (claude_action_tx, mut claude_action_rx) =
            mpsc::unbounded_channel::<ClaudeAccountAction>();
        let forward_tx = app_event_tx.clone();
        tokio::spawn(async move {
            while let Some(action) = claude_action_rx.recv().await {
                forward_tx.send(AppEvent::SharedProviderAuthAction(action.into()));
            }
        });
        Self {
            controller: ProviderAuthController::default(),
            request_handle,
            openai: Arc::new(Mutex::new(OpenAiAccountAppServerAdapter::default())),
            claude: ClaudeAuthAdapter::new(config.codex_home.to_path_buf()),
            claude_action_tx,
            app_event_tx,
            status_host,
            config,
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        action: ProviderAuthAction,
    ) -> Vec<ProviderAccountPresentation> {
        let transition = self.controller.dispatch(action);
        let mut presentations = transition
            .effects
            .into_iter()
            .filter_map(|effect| self.execute(effect))
            .collect::<Vec<_>>();
        match &transition.snapshot {
            ProviderAuthFlowSnapshot::OpenAiAccount(
                codex_provider_auth::OpenAiAccountSnapshot::Starting { .. }
                | codex_provider_auth::OpenAiAccountSnapshot::CancelPendingStart { .. },
            ) => presentations.push(ProviderAccountPresentation::Pending(
                ProviderAccountCancelKind::OpenAi,
            )),
            ProviderAuthFlowSnapshot::ClaudeAccount(
                ClaudeAccountSnapshot::CheckingExistingLogin { .. }
                | ClaudeAccountSnapshot::StartingClaudeCodeLogin { .. },
            ) => presentations.push(ProviderAccountPresentation::Pending(
                ProviderAccountCancelKind::Claude,
            )),
            ProviderAuthFlowSnapshot::ClaudeAccount(snapshot)
                if presents_claude_method_choice(snapshot) =>
            {
                presentations.push(ProviderAccountPresentation::ClaudeMethodChoice)
            }
            ProviderAuthFlowSnapshot::ClaudeAccount(
                ClaudeAccountSnapshot::EnteringManagedToken { .. },
            ) => presentations.push(ProviderAccountPresentation::ClaudeManagedTokenEntry),
            ProviderAuthFlowSnapshot::OpenAiAccount(
                codex_provider_auth::OpenAiAccountSnapshot::Failed { .. }
                | codex_provider_auth::OpenAiAccountSnapshot::RecoveryRequired { .. }
                | codex_provider_auth::OpenAiAccountSnapshot::Blocked { .. },
            )
            | ProviderAuthFlowSnapshot::ClaudeAccount(
                ClaudeAccountSnapshot::Failed { .. } | ClaudeAccountSnapshot::Blocked { .. },
            ) => presentations.push(ProviderAccountPresentation::Failed),
            _ => {}
        }
        presentations
    }

    pub(crate) fn openai_login_completed(&self, login_id: String, success: bool) {
        let notification = codex_app_server_protocol::AccountLoginCompletedNotification {
            login_id: Some(login_id),
            success,
            error: None,
        };
        let action = self
            .openai
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .login_completed(notification);
        if let Some(action) = action {
            self.app_event_tx
                .send(AppEvent::SharedProviderAuthAction(action.into()));
        }
    }

    fn execute(&self, effect: ProviderAuthEffect) -> Option<ProviderAccountPresentation> {
        match effect {
            ProviderAuthEffect::OpenAiAccount(effect) => self.execute_openai(effect),
            ProviderAuthEffect::ClaudeAccount(effect) => self.execute_claude(effect),
            ProviderAuthEffect::Complete(completion) => {
                Some(ProviderAccountPresentation::Completion(completion))
            }
            ProviderAuthEffect::PersistApiKey { .. }
            | ProviderAuthEffect::ScheduleTimeout { .. }
            | ProviderAuthEffect::RefreshProviderStatus { .. } => {
                Some(ProviderAccountPresentation::Failed)
            }
        }
    }

    fn execute_openai(&self, effect: OpenAiAccountEffect) -> Option<ProviderAccountPresentation> {
        match effect {
            OpenAiAccountEffect::PresentChallenge { challenge, .. } => {
                Some(ProviderAccountPresentation::OpenAiChallenge { challenge })
            }
            OpenAiAccountEffect::RefreshStatus { attempt_id, target } => {
                self.refresh_account_status(
                    move |status| OpenAiAccountAction::StatusResolved { attempt_id, status }.into(),
                    target.provider_id.to_string(),
                );
                None
            }
            effect @ OpenAiAccountEffect::StartLogin { attempt_id, .. } => {
                let request = self
                    .openai
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .request_for_effect(RequestId::String(Uuid::new_v4().to_string()), &effect);
                let Ok(request) = request else {
                    self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
                        OpenAiAccountAction::StartFinished {
                            attempt_id,
                            result: codex_provider_auth::OpenAiAccountStartResult::ProtocolMismatch,
                        }
                        .into(),
                    ));
                    return None;
                };
                let request_handle = self.request_handle.clone();
                let adapter = Arc::clone(&self.openai);
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let response = request_handle
                        .request_typed::<LoginAccountResponse>(request)
                        .await;
                    let action = {
                        let mut adapter = adapter
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        match response {
                            Ok(response) => adapter.start_finished(attempt_id, response),
                            Err(error) => adapter.start_failed(attempt_id, &error),
                        }
                    };
                    tx.send(AppEvent::SharedProviderAuthAction(action.into()));
                });
                None
            }
            effect @ OpenAiAccountEffect::CancelLogin { attempt_id, .. } => {
                let request = self
                    .openai
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .request_for_effect(RequestId::String(Uuid::new_v4().to_string()), &effect);
                let Ok(request) = request else {
                    self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
                        OpenAiAccountAction::CancelFinished {
                            attempt_id,
                            result: codex_provider_auth::OpenAiCancelResult::TransportLost,
                        }
                        .into(),
                    ));
                    return None;
                };
                let request_handle = self.request_handle.clone();
                let adapter = Arc::clone(&self.openai);
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let response = request_handle
                        .request_typed::<CancelLoginAccountResponse>(request)
                        .await;
                    let action = {
                        let mut adapter = adapter
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        match response {
                            Ok(response) => adapter.cancel_finished(attempt_id, response),
                            Err(error) => adapter.cancel_failed(attempt_id, &error),
                        }
                    };
                    tx.send(AppEvent::SharedProviderAuthAction(action.into()));
                });
                None
            }
        }
    }

    fn execute_claude(&self, effect: ClaudeAccountEffect) -> Option<ProviderAccountPresentation> {
        match effect {
            ClaudeAccountEffect::PresentChallenge { challenge, .. } => {
                Some(ProviderAccountPresentation::ClaudeChallenge { challenge })
            }
            ClaudeAccountEffect::RefreshStatus { attempt_id, target } => {
                self.refresh_account_status(
                    move |status| ClaudeAccountAction::StatusResolved { attempt_id, status }.into(),
                    target.provider_id.to_string(),
                );
                None
            }
            effect => {
                let correlation = claude_effect_correlation(&effect);
                if self
                    .claude
                    .execute(effect, self.claude_action_tx.clone())
                    .is_err()
                {
                    if let Some((attempt_id, process_id)) = correlation {
                        self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
                            ClaudeAccountAction::BackendTransportLost {
                                attempt_id,
                                process_id,
                            }
                            .into(),
                        ));
                    }
                    None
                } else {
                    None
                }
            }
        }
    }

    fn refresh_account_status(
        &self,
        action: impl FnOnce(codex_provider_auth::ProviderStatusSnapshot) -> ProviderAuthAction
        + Send
        + 'static,
        provider_id: String,
    ) {
        let config = self.config.clone();
        let status_host = self.status_host.clone();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let metadata = ProviderAccountMetadata::discover(&config).await;
            status_host.update_account_metadata(metadata);
            if let Some(status) = status_host.resolve().get(&provider_id).cloned() {
                tx.send(AppEvent::SharedProviderAuthAction(action(status)));
            }
        });
    }
}

fn presents_claude_method_choice(snapshot: &ClaudeAccountSnapshot) -> bool {
    matches!(
        snapshot,
        ClaudeAccountSnapshot::ChoosingMethod { .. }
            | ClaudeAccountSnapshot::RecoveryRequired { .. }
    )
}

fn claude_effect_correlation(
    effect: &ClaudeAccountEffect,
) -> Option<(
    codex_provider_auth::ProviderAuthAttemptId,
    Option<codex_provider_auth::claude_account_flow::ClaudeCodeProcessId>,
)> {
    match effect {
        ClaudeAccountEffect::EnrollManagedToken { attempt_id, .. }
        | ClaudeAccountEffect::ScheduleManagedTimeout { attempt_id, .. }
        | ClaudeAccountEffect::CheckExistingClaudeCodeLogin { attempt_id, .. }
        | ClaudeAccountEffect::RefreshStatus { attempt_id, .. } => Some((*attempt_id, None)),
        ClaudeAccountEffect::StartClaudeCodeLogin {
            attempt_id,
            process_id,
            ..
        }
        | ClaudeAccountEffect::PresentChallenge {
            attempt_id,
            process_id,
            ..
        }
        | ClaudeAccountEffect::SendAuthorizationCode {
            attempt_id,
            process_id,
            ..
        }
        | ClaudeAccountEffect::CancelClaudeCodeLogin {
            attempt_id,
            process_id,
        } => Some((*attempt_id, Some(*process_id))),
    }
}

#[cfg(test)]
#[path = "provider_account_auth_host_tests.rs"]
mod tests;
