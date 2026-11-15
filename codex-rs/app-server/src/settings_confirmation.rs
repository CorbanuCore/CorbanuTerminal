//! Request-specific completion, independent of settings snapshot deduplication.
use crate::error_code::invalid_request;
use crate::outgoing_message::ConnectionRequestId;
use crate::outgoing_message::OutgoingMessageSender;
use crate::thread_state::ThreadState;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::ThreadSettingsUpdateOutcome;
use codex_app_server_protocol::ThreadSettingsUpdateResponse;
use codex_protocol::error::Result as CodexResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

pub(crate) struct PendingSettingsConfirmation {
    pub(crate) request_id: ConnectionRequestId,
    pub(crate) generation: u64,
    pub(crate) completion: oneshot::Sender<Result<(), JSONRPCErrorError>>,
}

impl ThreadState {
    pub(crate) fn finish_settings_confirmation(
        &mut self,
        operation_id: &str,
        result: Result<(), JSONRPCErrorError>,
    ) {
        if let Some(pending) = self.pending_settings.remove(operation_id)
            && pending.generation == self.listener_generation
        {
            let _ = pending.completion.send(result);
        }
    }
}

pub(crate) async fn complete_settings_submission(
    state: Arc<Mutex<ThreadState>>,
    operation_id: String,
    request_id: ConnectionRequestId,
    completion: oneshot::Receiver<Result<(), JSONRPCErrorError>>,
    outgoing: Arc<OutgoingMessageSender>,
    deadline: tokio::time::Instant,
) {
    // One owner sends the terminal reply; removal/drop wakes it on lifecycle loss.
    let result = tokio::time::timeout_at(deadline, completion).await;
    state.lock().await.pending_settings.remove(&operation_id);
    match result {
        Ok(Ok(Err(error))) => outgoing.send_error(request_id, error).await,
        result => {
            let outcome = if matches!(result, Ok(Ok(Ok(())))) {
                ThreadSettingsUpdateOutcome::Applied
            } else {
                ThreadSettingsUpdateOutcome::Uncertain
            };
            outgoing
                .send_response(
                    request_id,
                    ThreadSettingsUpdateResponse::Confirmed { outcome },
                )
                .await;
        }
    }
}

/// Delivers Core's accept/reject reply for a confirmed settings update to its
/// pending confirmation. Disconnect or listener teardown has already removed
/// the confirmation in those cases, so this is then a no-op.
pub(crate) async fn forward_core_settings_result(
    state: Arc<Mutex<ThreadState>>,
    operation_id: String,
    core_result: oneshot::Receiver<CodexResult<()>>,
    deadline: tokio::time::Instant,
) {
    let result = match tokio::time::timeout_at(deadline, core_result).await {
        Ok(Ok(Ok(()))) => Ok(()),
        // Core's rejection message, e.g. "invalid thread settings override: ...".
        Ok(Ok(Err(error))) => Err(invalid_request(error.to_string())),
        // Dropped or late: the confirmation's own deadline reports uncertain.
        Ok(Err(_)) | Err(_) => return,
    };
    state
        .lock()
        .await
        .finish_settings_confirmation(&operation_id, result);
}
