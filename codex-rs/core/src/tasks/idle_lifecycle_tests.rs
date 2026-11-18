use super::SessionTask;
use super::SessionTaskResult;
use crate::config::Config;
use crate::session::TurnInput;
use crate::session::session::Session;
use crate::session::tests::make_session_and_context;
use crate::session::turn_context::TurnContext;
use crate::state::TaskKind;
use codex_extension_api::ThreadIdleCause;
use codex_protocol::user_input::UserInput;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

struct PendingTask;

impl SessionTask for PendingTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.pending_idle_lifecycle"
    }

    async fn run(
        self: Arc<Self>,
        _session: Arc<Session>,
        _turn_context: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        _cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        std::future::pending().await
    }
}

struct IdleRecorder {
    causes: Arc<std::sync::Mutex<Vec<ThreadIdleCause>>>,
}

impl codex_extension_api::ThreadLifecycleContributor<Config> for IdleRecorder {
    fn on_thread_idle<'a>(
        &'a self,
        input: codex_extension_api::ThreadIdleInput<'a>,
    ) -> codex_extension_api::ExtensionFuture<'a, ()> {
        Box::pin(async move {
            self.causes.lock().expect("idle causes").push(input.cause);
        })
    }
}

#[test_case::test_case(None; "without_follow_up")]
#[test_case::test_case(Some("pending user work"); "follow_up_submission_closed")]
#[tokio::test]
async fn task_finish_emits_idle_once_after_completion(pending_text: Option<&str>) {
    // This fixture has no submission loop. A follow-up therefore cannot be admitted,
    // and must retain the idle notification even when its submission fails.
    let (mut session, turn_context) = make_session_and_context().await;
    let causes = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut builder = codex_extension_api::ExtensionRegistryBuilder::<Config>::new();
    builder.thread_lifecycle_contributor(Arc::new(IdleRecorder {
        causes: Arc::clone(&causes),
    }));
    session.services.extensions = Arc::new(builder.build());
    let session = Arc::new(session);
    let turn_context = Arc::new(turn_context);
    session
        .spawn_task(Arc::clone(&turn_context), Vec::new(), PendingTask)
        .await;

    if let Some(text) = pending_text {
        let turn_state = {
            let active = session.active_turn.lock().await;
            Arc::clone(&active.as_ref().expect("pending task").turn_state)
        };
        session
            .input_queue
            .extend_pending_input_for_turn_state(
                turn_state.as_ref(),
                vec![TurnInput::UserInput {
                    content: vec![UserInput::Text {
                        text: text.to_string(),
                        text_elements: Vec::new(),
                    }],
                    client_id: None,
                    metadata: Default::default(),
                }],
            )
            .await;
    }

    // Await the whole completion path, not its first idle callback: the old test
    // could assert before the duplicate callback ran on another executor thread.
    session
        .on_task_finished(Arc::clone(&turn_context), /*task_result*/ Ok(None))
        .await;

    assert_eq!(
        *causes.lock().expect("idle causes"),
        vec![ThreadIdleCause::Completed]
    );
    assert!(session.active_turn.lock().await.is_none());
}
