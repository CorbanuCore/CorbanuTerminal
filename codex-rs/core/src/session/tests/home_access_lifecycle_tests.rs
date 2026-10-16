use super::*;
use crate::home_access::note_rollout_unwritten;
use crate::home_access::tests::has_unsaved_rollout;

#[tokio::test]
async fn stopped_session_drops_an_unsaved_rollout_mark() {
    let (mut session, _) = make_session_and_context().await;
    let path = attach_thread_persistence(&mut session).await;
    // Establish the same marker as a prior EPERM append. No OS outage is
    // manufactured here: this test exercises actual Session teardown.
    note_rollout_unwritten(
        session.thread_id(),
        path.parent().expect("rollout directory").into(),
    );
    assert!(has_unsaved_rollout(session.thread_id()));
    let session = Arc::new(session);

    assert!(handlers::shutdown(&session, "stop".into()).await);

    assert!(!has_unsaved_rollout(session.thread_id()));
}

#[tokio::test]
async fn closed_submission_channel_drops_an_unsaved_rollout_mark() {
    let (mut session, turn_context) = make_session_and_context().await;
    let path = attach_thread_persistence(&mut session).await;
    note_rollout_unwritten(
        session.thread_id(),
        path.parent().expect("rollout directory").into(),
    );
    assert!(has_unsaved_rollout(session.thread_id()));
    let session = Arc::new(session);
    let (sender, receiver) = async_channel::bounded(1);
    drop(sender);

    submission_loop(
        Arc::clone(&session),
        Arc::clone(&turn_context.config),
        receiver,
    )
    .await;

    assert!(!has_unsaved_rollout(session.thread_id()));
}

#[tokio::test]
async fn filtered_events_and_flush_preserve_unsaved_until_a_written_append() {
    let (mut session, _) = make_session_and_context().await;
    let path = attach_thread_persistence(&mut session).await;
    let directory = path.parent().expect("rollout directory").to_path_buf();
    let other = ThreadId::new();
    note_rollout_unwritten(session.thread_id(), directory.clone());
    note_rollout_unwritten(other, directory);
    let stream_error = RolloutItem::EventMsg(EventMsg::StreamError(StreamErrorEvent {
        message: "synthetic transient stream error".into(),
        codex_error_info: None,
        additional_details: None,
    }));

    session.persist_rollout_items(&[stream_error]).await;
    session.flush_rollout().await.expect("flush succeeds");
    session
        .try_ensure_rollout_materialized()
        .await
        .expect("persist succeeds");
    assert!(has_unsaved_rollout(session.thread_id()));
    assert!(has_unsaved_rollout(other));

    let saved = user_message("a real persisted synthetic append");
    session
        .persist_rollout_items(&[RolloutItem::ResponseItem(saved.clone())])
        .await;
    session.flush_rollout().await.expect("saved append flushes");
    assert!(!has_unsaved_rollout(session.thread_id()));
    assert!(has_unsaved_rollout(other));
    let InitialHistory::Resumed(history) = RolloutRecorder::get_rollout_history(&path)
        .await
        .expect("read actual durable rollout")
    else {
        panic!("expected persisted history")
    };
    assert!(
        history
            .history
            .iter()
            .any(|item| matches!(item, RolloutItem::ResponseItem(value) if value == &saved))
    );
    assert!(
        !history
            .history
            .iter()
            .any(|item| matches!(item, RolloutItem::EventMsg(EventMsg::StreamError(_))))
    );
    crate::home_access::note_rollout_written(other);
}
