use super::*;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
struct PanickingTask;

impl SessionTask for PanickingTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.panicking"
    }

    async fn run(
        self: Arc<Self>,
        _session: Arc<SessionTaskContext>,
        _ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        _cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        panic!("#616 simulated panic in a turn task");
    }
}

/// #616: a task that panics still ends its turn, with an error, so clients
/// waiting for the turn to end (exec, the TUI) don't wait forever.
#[tokio::test]
async fn a_panicking_task_fails_its_turn_instead_of_leaving_it_running() {
    let (sess, tc, rx) = make_session_and_context_with_rx().await;
    sess.spawn_task(
        Arc::clone(&tc),
        vec![TurnInput::UserInput {
            content: vec![UserInput::Text {
                text: "hello".to_string(),
                text_elements: Vec::new(),
            }],
            client_id: None,
        }],
        PanickingTask,
    )
    .await;

    let error = timeout(Duration::from_secs(2), async {
        loop {
            let event = rx.recv().await.expect("event");
            match event.msg {
                EventMsg::Error(error) => return error,
                EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_) => {
                    panic!("turn ended without an error: {:?}", event.msg)
                }
                _ => {}
            }
        }
    })
    .await
    .expect("the panic should be reported as a turn error");
    assert_eq!(
        error.message,
        format!("Fatal error: {}", crate::tasks::TURN_PANICKED_MESSAGE)
    );
    let _ = recv_terminal_event(&rx, TerminalEventKind::TurnComplete).await;
    // The turn is cleared just after TurnComplete is sent.
    timeout(Duration::from_secs(2), async {
        while sess.active_turn.lock().await.is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the finished turn is cleared");
}
