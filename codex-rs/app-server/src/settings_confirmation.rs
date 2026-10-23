//! Request-specific completion, independent of settings snapshot deduplication.
use crate::outgoing_message::ConnectionRequestId;
use crate::outgoing_message::OutgoingMessageSender;
use crate::thread_state::ThreadState;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::ThreadSettingsUpdateOutcome;
use codex_app_server_protocol::ThreadSettingsUpdateResponse;
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
