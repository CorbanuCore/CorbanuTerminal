use super::*;
use crate::settings_confirmation::PendingSettingsConfirmation;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn settings_confirmation_correlates_failure_once_and_rejects_stale_generation() {
    let mut state = ThreadState::default();
    let mut receiver = pending(&mut state, 1, "request-op");
    state.finish_settings_confirmation("unrelated-op", Ok(()));
    assert_eq!(
        receiver.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );
    let error = crate::error_code::invalid_request("Core rejected settings");
    state.finish_settings_confirmation("request-op", Err(error.clone()));
    state.finish_settings_confirmation("request-op", Ok(()));
    assert_eq!(receiver.await.unwrap(), Err(error));
    let stale = pending(&mut state, 1, "stale-op");
    state.listener_generation += 1;
    state.finish_settings_confirmation("stale-op", Ok(()));
    assert!(stale.await.is_err());
    assert!(state.pending_settings.is_empty());
}

fn pending(
    state: &mut ThreadState,
    connection: u64,
    id: &str,
) -> oneshot::Receiver<Result<(), codex_app_server_protocol::JSONRPCErrorError>> {
    let (completion, receiver) = oneshot::channel();
    state.pending_settings.insert(
        id.into(),
        PendingSettingsConfirmation {
            request_id: ConnectionRequestId {
                connection_id: ConnectionId(connection),
                request_id: RequestId::Integer(1),
            },
            generation: state.listener_generation,
            completion,
        },
    );
    receiver
}

#[tokio::test]
async fn settings_confirmation_disconnect_only_clears_its_connection() {
    let manager = ThreadStateManager::new();
    let state = manager.thread_state(ThreadId::new()).await;
    let (first, mut second) = {
        let mut state = state.lock().await;
        (
            pending(&mut state, 1, "first"),
            pending(&mut state, 2, "second"),
        )
    };
    manager.remove_connection(ConnectionId(1)).await;
    assert!(first.await.is_err());
    assert_eq!(second.try_recv(), Err(oneshot::error::TryRecvError::Empty));
    manager.clear_all_listeners().await;
    assert!(second.await.is_err());
    assert!(state.lock().await.pending_settings.is_empty());
}

#[tokio::test]
async fn settings_confirmation_listener_teardown_is_terminal() {
    let manager = ThreadStateManager::new();
    let thread_id = ThreadId::new();
    let state = manager.thread_state(thread_id).await;
    let receiver = pending(&mut *state.lock().await, 1, "operation");
    manager.remove_thread_state(thread_id).await;
    assert!(receiver.await.is_err());
    assert!(state.lock().await.pending_settings.is_empty());
}

#[tokio::test]
async fn settings_confirmation_terminal_reply_cleans_up_and_never_replies_twice() {
    use crate::outgoing_message::OutgoingEnvelope;
    use crate::outgoing_message::OutgoingMessage;
    use crate::outgoing_message::OutgoingMessageSender;
    for terminal in ["applied", "failed", "dropped", "timeout"] {
        let state = Arc::new(Mutex::new(ThreadState::default()));
        let completion = pending(&mut *state.lock().await, 42, "operation");
        let request_id = state.lock().await.pending_settings["operation"]
            .request_id
            .clone();
        let (tx, mut rx) = mpsc::channel(4);
        let outgoing = Arc::new(OutgoingMessageSender::new(
            tx,
            codex_analytics::AnalyticsEventsClient::disabled(),
        ));
        match terminal {
            "applied" => state
                .lock()
                .await
                .finish_settings_confirmation("operation", Ok(())),
            "failed" => state.lock().await.finish_settings_confirmation(
                "operation",
                Err(crate::error_code::invalid_request("rejected")),
            ),
            "dropped" => {
                state.lock().await.pending_settings.clear();
            }
            _ => {}
        }
        crate::settings_confirmation::complete_settings_submission(
            state.clone(),
            "operation".into(),
            request_id.clone(),
            completion,
            outgoing.clone(),
            tokio::time::Instant::now(),
        )
        .await;
        assert!(state.lock().await.pending_settings.is_empty());
        let OutgoingEnvelope::ToConnection {
            connection_id,
            message,
            ..
        } = rx.recv().await.unwrap()
        else {
            panic!("expected targeted terminal reply");
        };
        assert_eq!(connection_id, request_id.connection_id);
        match message {
            OutgoingMessage::Response(response) => {
                assert_ne!(terminal, "failed");
                assert_eq!(response.id, request_id.request_id);
                assert_eq!(
                    serde_json::to_value(response.result).unwrap(),
                    serde_json::json!({
                        "outcome": if terminal == "applied" { "applied" } else { "uncertain" }
                    })
                );
            }
            OutgoingMessage::Error(error) => {
                assert_eq!(terminal, "failed");
                assert_eq!(error.id, request_id.request_id);
                assert_eq!(error.error.message, "rejected");
            }
            _ => panic!("expected response or error"),
        }
        state
            .lock()
            .await
            .finish_settings_confirmation("operation", Ok(()));
        assert!(rx.try_recv().is_err());
    }
}
