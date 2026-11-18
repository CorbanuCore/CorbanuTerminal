use super::*;
use codex_protocol::protocol::ErrorEvent;
use pretty_assertions::assert_eq;

/// D659, accepted November 16 in managementHandoff.html#decision-D659, supersedes
/// the old failed-compaction re-admission request: save pending input and mail after
/// a lasting error, then stop retrying. Held user input follows that same contract.
#[tokio::test]
async fn failed_compaction_records_interrupted_held_user_input_once_without_replay() {
    const HELD_TEXT: &str = "held user work survives failed compaction";
    let (mut sess, original, events) = make_session_and_context_with_rx().await;
    // Observe every automatic submission directly. Keeping the receiver open makes
    // a mistaken replay observable without a scheduler delay or a network request.
    let (submissions_tx, submissions_rx) = async_channel::bounded(16);
    Arc::get_mut(&mut sess).expect("unique test session").tx_sub = submissions_tx;
    sess.spawn_task(
        Arc::clone(&original),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: true,
        },
    )
    .await;
    let held = TurnInput::UserInput {
        content: vec![UserInput::Text {
            text: HELD_TEXT.to_string(),
            text_elements: Vec::new(),
        }],
        client_id: Some("held-client".to_string()),
        metadata: Default::default(),
    };
    let schema = Some(serde_json::json!({"type": "object", "properties": {}}));
    sess.defer_held_input(held.clone(), schema.clone()).await;
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
    let (interrupted, interrupted_schema) =
        sess.input_queue.next_interrupted_deferred_input().await;
    assert_eq!((&interrupted, &interrupted_schema), (&held, &schema));

    let compact = sess.new_default_turn().await;
    sess.spawn_task(
        Arc::clone(&compact),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Compact,
            listen_to_cancellation_token: false,
        },
    )
    .await;
    super::super::handlers::readmit_interrupted_input(&sess, interrupted, interrupted_schema).await;
    assert_eq!(
        sess.input_queue
            .get_pending_input_for_running_task(&sess, &compact.sub_id)
            .await
            .0,
        Vec::new(),
        "held user input must not enter the running compaction"
    );
    assert!(sess.clone_history().await.raw_items().all(|item| {
        !matches!(item, ResponseItem::Message { content, .. } if content.iter().any(|part| {
            matches!(part, ContentItem::InputText { text } if text == HELD_TEXT)
        }))
    }));
    while events.try_recv().is_ok() {}
    assert!(submissions_rx.is_empty());

    let error = ErrorEvent {
        message: "compaction quota exhausted".to_string(),
        codex_error_info: Some(CodexErrorInfo::UsageLimitExceeded),
        misalignment: None,
    };
    sess.send_event(&compact, EventMsg::Error(error.clone()))
        .await;
    sess.on_task_finished(Arc::clone(&compact), /*task_result*/ Ok(None))
        .await;

    let recorded = sess.clone_history().await;
    let saved_user_input = recorded
        .raw_items()
        .filter(|item| {
            matches!(item, ResponseItem::Message { role, content, .. }
            if role == "user" && content.iter().any(|part| {
                matches!(part, ContentItem::InputText { text } if text == HELD_TEXT)
            }))
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        strip_response_item_ids(&strip_metadata_from_items(&saved_user_input)),
        strip_metadata_from_items(&[user_message(HELD_TEXT)])
    );
    assert!(sess.active_turn.lock().await.is_none());
    assert!(!sess.input_queue.has_pending_input(&sess.active_turn).await);
    assert!(
        matches!(
            submissions_rx.try_recv(),
            Err(async_channel::TryRecvError::Empty)
        ),
        "terminal compaction must not enqueue an automatic replay"
    );
    let terminal_events = std::iter::from_fn(|| events.try_recv().ok())
        .map(|event| event.msg)
        .collect::<Vec<_>>();
    assert!(
        terminal_events
            .iter()
            .all(|event| !matches!(event, EventMsg::TurnStarted(_)))
    );
    let completions = terminal_events
        .iter()
        .filter_map(|event| match event {
            EventMsg::TurnComplete(completed) => Some((&completed.turn_id, &completed.error)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(completions, vec![(&compact.sub_id, &Some(error))]);
}
