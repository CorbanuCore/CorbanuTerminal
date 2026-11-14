//! #688: cancellation after enqueue must not make a delivered warning repeat.

use std::future::Future;
use std::task::Context;
use std::time::Duration;

use codex_protocol::ToolName;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use pretty_assertions::assert_eq;

use super::notify_omitted_tools;
use super::omitted_tools_notice;

#[tokio::test]
async fn cancelling_after_delivery_does_not_repeat_the_warning() {
    let (mut session, context) = crate::session::tests::make_session_and_context().await;
    assert!(session.services.live_thread.is_none());
    let (tx, rx) = async_channel::bounded(/*cap*/ 1);
    session.tx_event = tx;
    session
        .tx_event
        .try_send(Event {
            id: "backpressure-control".to_string(),
            msg: EventMsg::Warning(WarningEvent {
                message: "occupied event channel".to_string(),
            }),
        })
        .expect("the initially empty channel has one slot");
    let omitted = [ToolName::namespaced("mcp__bulk", "a")];
    let mut first = Box::pin(tokio::task::unconstrained(notify_omitted_tools(
        &session, &context, &omitted,
    )));
    assert!(futures::poll!(first.as_mut()).is_pending());

    // Hold unrelated session state only after selection reaches the real
    // blocked send. Delivery must not depend on acquiring it to record success.
    let held_state = session.state.lock().await;
    assert_eq!(
        rx.try_recv().expect("the control fills the channel").id,
        "backpressure-control"
    );
    // A correct implementation may already finish here. Do not require it to
    // retain the old post-delivery suspension just to exercise cancellation.
    let mut poll_context = Context::from_waker(futures::task::noop_waker_ref());
    let _completion = first.as_mut().poll(&mut poll_context);
    let EventMsg::Warning(delivered) = rx
        .try_recv()
        .expect("the actual warning is enqueued while session state is held")
        .msg
    else {
        panic!("notification emitted a non-warning event");
    };
    drop(first);
    drop(held_state);

    tokio::time::timeout(
        Duration::from_secs(/*secs*/ 5),
        notify_omitted_tools(&session, &context, &omitted),
    )
    .await
    .expect("a retry must finish without a repeated warning");
    let mut messages = vec![delivered.message];
    while let Ok(event) = rx.try_recv() {
        let EventMsg::Warning(warning) = event.msg else {
            panic!("retry emitted a non-warning event");
        };
        messages.push(warning.message);
    }
    assert_eq!(
        messages,
        vec![omitted_tools_notice(&["mcp__bulk__a".to_string()])]
    );
}
