//! Task completion preserves a queued notification's budget alongside real user input.

use super::*;
use pretty_assertions::assert_eq;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn task_finish_preserves_notification_budget_and_queued_user_input() {
    // Corbanu: user input left at a natural turn end starts a follow-up turn, which needs the
    // submission loop.
    let (session, turn, rx) = make_session_and_context_with_submission_loop_and_rx().await;
    session
        .spawn_task(
            Arc::clone(&turn),
            Vec::new(),
            NeverEndingTask {
                kind: TaskKind::Regular,
                listen_to_cancellation_token: false,
            },
        )
        .await;
    let text = "notification diagnostic line\n".repeat(500);
    let notification = |text| ResponseItem::CustomToolCallOutput {
        id: None,
        call_id: "call-a".to_string(),
        name: Some("exec".to_string()),
        output: FunctionCallOutputPayload::from_text(text),
        internal_chat_message_metadata_passthrough: None,
    };
    session
        .inject_if_running(vec![ResponseItemEnvelope {
            item: notification(text.clone()),
            metadata: Some(CodexHarnessMetadata {
                history_truncation_token_limit: Some(120),
                ..Default::default()
            }),
        }])
        .await
        .unwrap();
    let user_input = vec![UserInput::Text {
        text: "keep this queued user input".to_string(),
        text_elements: Vec::new(),
    }];
    assert!(matches!(
        submit_steer_only(&session, user_input.clone(), &turn.sub_id).await,
        TurnInputSubmission::Steered { .. }
    ));
    let mut current = turn.initial_settings.as_ref().clone();
    Arc::make_mut(&mut current.model_info).truncation_policy =
        codex_protocol::openai_models::TruncationPolicyConfig::tokens(/*limit*/ 400);
    turn.next_step_settings.store(Arc::new(current));

    // Stall recording the first batch, then inject while finalization is in progress.
    let state = session.state.lock().await;
    let finish = session.on_task_finished(Arc::clone(&turn), /*task_result*/ Ok(None));
    let late = vec![notification("late notification".to_string())];
    tokio::join!(finish, async {
        while session
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("turn remains active while final recording is blocked")
            .finishing
            .is_none()
        {
            tokio::task::yield_now().await;
        }
        // Admission is closed even if recording has not yet reached the history lock.
        let notice = vec![notification("producer still running".to_string())];
        assert_eq!(session.inject_if_running(notice.clone()).await, Err(notice));
        let inject = session.inject_client_response_items(late.clone(), &turn);
        tokio::pin!(inject);
        assert!(futures::poll!(inject.as_mut()).is_pending());
        drop(state);
        inject.await;
    });

    // The finishing turn keeps the notification's budget and the late notification. Corbanu
    // hands the queued user input to a follow-up turn (upstream records it on this turn), which
    // records it after them.
    let expected = vec![
        notification(codex_utils_output_truncation::truncate_text(
            &text,
            codex_utils_output_truncation::TruncationPolicy::Tokens(120),
        )),
        late[0].clone(),
    ];
    expect_user_message_item_started(&rx, &user_input).await;
    let user_item = strip_response_item_ids(&strip_metadata_from_items(&[
        session.response_item_from_user_input(user_input)
    ]));
    let items = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let items = strip_response_item_ids(&strip_metadata_from_items(&raw_history_items(
                &session.clone_history().await,
            )));
            if items.iter().any(|item| user_item.contains(item)) {
                break items;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the follow-up turn records the queued user input");
    assert_eq!(
        items[..expected.len()].to_vec(),
        strip_response_item_ids(&strip_metadata_from_items(&expected)),
    );
    session.abort_all_tasks(TurnAbortReason::Interrupted).await;
}
