use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use codex_provider_auth::ProviderAuthAttemptId;
use codex_provider_auth::claude_account_flow::ClaudeAccountAction;
use codex_provider_auth::claude_account_flow::ClaudeAccountEffect;
use codex_provider_auth::claude_account_flow::ClaudeCodeChallenge;
use codex_provider_auth::claude_account_flow::ClaudeCodeLoginOutcome;
use codex_provider_auth::claude_account_flow::ClaudeCodeProcessId;
use codex_provider_auth::claude_account_flow::ClaudeExistingLoginResult;
use codex_provider_auth::claude_account_flow::ClaudeManagedTokenResult;
use tokio::sync::mpsc;

use super::claude_code_login;
use super::claude_code_login::ClaudeCodeLoginBackendError;
use super::claude_code_login::ClaudeCodeLoginBackendEvent;
use super::claude_code_login::ClaudeCodeLoginInput;
use crate::app_event::ClaudeSubscriptionTokenSecret;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClaudeAuthAdapterError {
    UnsupportedEffect,
    MissingProcess,
    ActionReceiverClosed,
}

type ProcessKey = (ProviderAuthAttemptId, ClaudeCodeProcessId);
type ProcessRegistry = Arc<Mutex<BTreeMap<ProcessKey, ProcessRegistration>>>;

struct ProcessRegistration {
    sender: Option<mpsc::UnboundedSender<ClaudeCodeLoginInput>>,
    start_returned: bool,
    finished: bool,
}

#[derive(Clone)]
pub(crate) struct ClaudeAuthAdapter {
    codex_home: PathBuf,
    processes: ProcessRegistry,
}

impl ClaudeAuthAdapter {
    pub(crate) fn new(codex_home: PathBuf) -> Self {
        Self {
            codex_home,
            processes: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub(crate) fn execute(
        &self,
        effect: ClaudeAccountEffect,
        action_tx: mpsc::UnboundedSender<ClaudeAccountAction>,
    ) -> Result<(), ClaudeAuthAdapterError> {
        match effect {
            ClaudeAccountEffect::EnrollManagedToken {
                attempt_id, secret, ..
            } => {
                let codex_home = self.codex_home.clone();
                tokio::spawn(async move {
                    let token = ClaudeSubscriptionTokenSecret::new(secret.into_inner().to_string());
                    let result = claude_code_login::enroll_managed_subscription_token_typed(
                        codex_home, token,
                    )
                    .await;
                    let result = match result {
                        Ok(()) => ClaudeManagedTokenResult::Stored,
                        Err(claude_code_login::ClaudeManagedEnrollmentError::Invalid(_)) => {
                            ClaudeManagedTokenResult::Invalid
                        }
                        Err(
                            claude_code_login::ClaudeManagedEnrollmentError::StorageUnavailable(_),
                        ) => ClaudeManagedTokenResult::StorageUnavailable,
                    };
                    let _ = action_tx
                        .send(ClaudeAccountAction::ManagedTokenFinished { attempt_id, result });
                });
                Ok(())
            }
            ClaudeAccountEffect::ScheduleManagedTimeout {
                attempt_id,
                timeout,
            } => {
                tokio::spawn(async move {
                    tokio::time::sleep(timeout).await;
                    let _ =
                        action_tx.send(ClaudeAccountAction::ManagedTimeoutElapsed { attempt_id });
                });
                Ok(())
            }
            ClaudeAccountEffect::CheckExistingClaudeCodeLogin { attempt_id, .. } => {
                let codex_home = self.codex_home.clone();
                tokio::spawn(async move {
                    let result = claude_code_login::select_existing_claude_code_login(
                        std::path::Path::new("claude"),
                        /*health_executable*/ None,
                        &codex_home,
                        std::time::Duration::from_secs(10),
                    )
                    .await;
                    let result = match result {
                        Ok(true) => ClaudeExistingLoginResult::Selected,
                        Ok(false) => ClaudeExistingLoginResult::LoginRequired,
                        Err(_) => ClaudeExistingLoginResult::Unavailable,
                    };
                    let _ = action_tx
                        .send(ClaudeAccountAction::ExistingLoginChecked { attempt_id, result });
                });
                Ok(())
            }
            ClaudeAccountEffect::StartClaudeCodeLogin {
                attempt_id,
                process_id,
                identity_policy,
                ..
            } => {
                let key = (attempt_id, process_id);
                begin_process(&self.processes, key);
                let processes = Arc::clone(&self.processes);
                let callback_actions = action_tx;
                let callback = Arc::new(move |event| match event {
                    ClaudeCodeLoginBackendEvent::Ready {
                        verification_url,
                        input_tx,
                    } => {
                        if !register_ready(&processes, key, input_tx.clone()) {
                            let _ = input_tx.send(ClaudeCodeLoginInput::Cancel);
                            return;
                        }
                        let action = ClaudeAccountAction::ClaudeCodeReady {
                            attempt_id,
                            process_id,
                            challenge: ClaudeCodeChallenge::new(verification_url),
                        };
                        if callback_actions.send(action).is_err() {
                            let _ = input_tx.send(ClaudeCodeLoginInput::Cancel);
                        }
                    }
                    ClaudeCodeLoginBackendEvent::Finished { result } => {
                        finish_process(&processes, key);
                        let outcome = collapse_backend_result(result);
                        let _ = callback_actions.send(ClaudeAccountAction::ClaudeCodeFinished {
                            attempt_id,
                            process_id,
                            outcome,
                        });
                    }
                });
                let input_tx = claude_code_login::start_with_callback(
                    self.codex_home.clone(),
                    identity_policy,
                    callback,
                );
                register_start_returned(&self.processes, key, input_tx);
                Ok(())
            }
            ClaudeAccountEffect::SendAuthorizationCode {
                attempt_id,
                process_id,
                secret,
            } => self.send_process_input(
                attempt_id,
                process_id,
                ClaudeCodeLoginInput::AuthorizationCode(secret.into_inner()),
            ),
            ClaudeAccountEffect::CancelClaudeCodeLogin {
                attempt_id,
                process_id,
            } => self.send_process_input(attempt_id, process_id, ClaudeCodeLoginInput::Cancel),
            ClaudeAccountEffect::PresentChallenge { .. }
            | ClaudeAccountEffect::RefreshStatus { .. } => {
                Err(ClaudeAuthAdapterError::UnsupportedEffect)
            }
        }
    }

    pub(crate) async fn recovery_source(
        &self,
    ) -> codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource {
        claude_code_login::selected_claude_recovery_source(self.codex_home.clone()).await
    }

    fn send_process_input(
        &self,
        attempt_id: ProviderAuthAttemptId,
        process_id: ClaudeCodeProcessId,
        input: ClaudeCodeLoginInput,
    ) -> Result<(), ClaudeAuthAdapterError> {
        let processes = match self.processes.lock() {
            Ok(processes) => processes,
            Err(poisoned) => poisoned.into_inner(),
        };
        let sender = processes
            .get(&(attempt_id, process_id))
            .and_then(|registration| registration.sender.clone())
            .ok_or(ClaudeAuthAdapterError::MissingProcess)?;
        sender
            .send(input)
            .map_err(|_| ClaudeAuthAdapterError::ActionReceiverClosed)
    }
}

fn begin_process(processes: &ProcessRegistry, key: ProcessKey) {
    processes
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(
            key,
            ProcessRegistration {
                sender: None,
                start_returned: false,
                finished: false,
            },
        );
}

fn register_ready(
    processes: &ProcessRegistry,
    key: ProcessKey,
    sender: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
) -> bool {
    let mut processes = processes
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(registration) = processes.get_mut(&key) else {
        return false;
    };
    if registration.finished {
        return false;
    }
    registration.sender = Some(sender);
    true
}

fn register_start_returned(
    processes: &ProcessRegistry,
    key: ProcessKey,
    sender: mpsc::UnboundedSender<ClaudeCodeLoginInput>,
) {
    let mut processes = processes
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(registration) = processes.get_mut(&key) else {
        return;
    };
    registration.start_returned = true;
    if registration.finished {
        processes.remove(&key);
    } else if registration.sender.is_none() {
        registration.sender = Some(sender);
    }
}

fn finish_process(processes: &ProcessRegistry, key: ProcessKey) {
    let mut processes = processes
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(registration) = processes.get_mut(&key) else {
        return;
    };
    registration.finished = true;
    registration.sender = None;
    if registration.start_returned {
        processes.remove(&key);
    }
}

fn collapse_backend_result(
    result: claude_code_login::ClaudeCodeLoginBackendResult,
) -> ClaudeCodeLoginOutcome {
    match result {
        Some(Ok(_)) => ClaudeCodeLoginOutcome::Succeeded,
        None => ClaudeCodeLoginOutcome::Cancelled,
        Some(Err(ClaudeCodeLoginBackendError::IdentityConflict)) => {
            ClaudeCodeLoginOutcome::IdentityConflict
        }
        Some(Err(ClaudeCodeLoginBackendError::TimedOut)) => ClaudeCodeLoginOutcome::TimedOut,
        Some(Err(ClaudeCodeLoginBackendError::Other(_))) => ClaudeCodeLoginOutcome::Rejected,
    }
}

#[cfg(test)]
#[path = "claude_auth_adapter_tests.rs"]
mod tests;
