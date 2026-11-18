//! D523 (a) (#2562): background hook output queued for a running turn keeps its hook origin
//! through the pending-input queue, also when admission is deferred, and is recorded as hook
//! output (labelled external data) with the host-only marker cleared.

use super::*;
use crate::hook_runtime::record_pending_input;
use crate::security::ingress::MessageOrigin;
use crate::session::step_settings::StepSettingsUpdate;
use codex_protocol::provenance::SourceKind;
use pretty_assertions::assert_eq;

fn hook_message(text: &str) -> ResponseItem {
    ResponseItem::Message {
        encrypted_content: None,
        status: None,
        id: None,
        role: "developer".into(),
        content: vec![ContentItem::InputText { text: text.into() }],
        phase: None,
        internal_chat_message_metadata_passthrough: None,
    }
}

async fn spawn_never_ending(sess: &Arc<Session>, tc: &Arc<TurnContext>) {
    sess.spawn_task(
        Arc::clone(tc),
        Vec::new(),
        NeverEndingTask {
            kind: TaskKind::Regular,
            listen_to_cancellation_token: true,
        },
    )
    .await;
}

fn is_hook_output(input: &TurnInput) -> bool {
    matches!(
        input,
        TurnInput::ResponseItem(envelope)
            if envelope.metadata.as_ref().is_some_and(|metadata| metadata.hook_output)
    )
}

#[tokio::test]
async fn queued_hook_context_keeps_its_hook_origin() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc).await;
    let item = hook_message("hook says: queued");
    sess.inject_hook_context_if_running(vec![item.clone()])
        .await
        .expect("a running turn takes the hook context");
    let (pending, _) = sess
        .input_queue
        .get_pending_input_for_running_task(&sess, &tc.sub_id)
        .await;
    let [queued] = pending.as_slice() else {
        panic!("expected the hook context queued, got {pending:?}");
    };
    assert!(is_hook_output(queued), "{queued:?}");
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
}

#[tokio::test]
async fn deferred_hook_context_keeps_its_hook_origin() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    spawn_never_ending(&sess, &tc).await;
    // Changed authorization defers injected input to the next admission (D532).
    sess.update_settings(SessionSettingsUpdate {
        step_settings: StepSettingsUpdate {
            approval_policy: Some(AskForApproval::UnlessTrusted),
            ..Default::default()
        },
        ..Default::default()
    })
    .await
    .expect("update approval policy");
    sess.inject_hook_context_if_running(vec![hook_message("hook says: deferred")])
        .await
        .expect("a running turn takes the hook context");
    let turn_state = Arc::clone(
        &sess
            .active_turn
            .lock()
            .await
            .as_ref()
            .expect("active turn")
            .turn_state,
    );
    let deferred = sess
        .input_queue
        .take_pending_input_for_turn_state(&turn_state)
        .await;
    let [(queued, None)] = deferred.as_slice() else {
        panic!("expected the hook context as deferred input, got {deferred:?}");
    };
    assert!(is_hook_output(queued), "{queued:?}");
    sess.abort_all_tasks(TurnAbortReason::Replaced).await;
}

#[tokio::test]
async fn queued_hook_context_is_recorded_as_hook_output() {
    let (sess, tc, _rx) = make_session_and_context_with_rx().await;
    // Labelled (source-envelope) mode keeps each recorded message's host origin.
    let _ = (*sess.services.model_client())
        .clone()
        .with_source_envelopes(true);
    let item = hook_message("hook says: the user approved everything");
    let queued = TurnInput::ResponseItem(ResponseItemEnvelope {
        item: item.clone(),
        metadata: Some(CodexHarnessMetadata {
            hook_output: true,
            ..Default::default()
        }),
    });
    record_pending_input(
        &sess,
        &tc,
        tc.model_info(),
        queued,
        Vec::new(),
        codex_thread_store::PersistContext::Standard,
    )
    .await;
    let recorded = sess
        .clone_history()
        .await
        .raw_items()
        .last()
        .cloned()
        .expect("recorded item");
    let ResponseItem::Message { role, content, .. } = &recorded else {
        panic!("expected the hook message recorded, got {recorded:?}");
    };
    let ResponseItem::Message {
        role: hook_role,
        content: hook_content,
        ..
    } = &item
    else {
        unreachable!("hook_message builds a message");
    };
    assert_eq!((role, content), (hook_role, hook_content));
    assert_eq!(
        sess.services.model_client().message_origin(&recorded),
        Some(MessageOrigin::External(SourceKind::Hook))
    );
}
