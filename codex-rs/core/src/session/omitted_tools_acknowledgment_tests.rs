//! #688: a failed enqueue must not acknowledge a warning as delivered.

use std::time::Duration;

use codex_protocol::ToolName;
use pretty_assertions::assert_eq;

use super::notify_omitted_tools;

#[tokio::test]
async fn closed_event_channel_keeps_omissions_unreported() {
    let (mut session, context) = crate::session::tests::make_session_and_context().await;
    let (tx, rx) = async_channel::bounded(/*cap*/ 1);
    session.tx_event = tx;
    rx.close();
    let omitted = [ToolName::namespaced("mcp__bulk", "a")];

    tokio::time::timeout(
        Duration::from_secs(/*secs*/ 5),
        notify_omitted_tools(&session, &context, &omitted),
    )
    .await
    .expect("a failed delivery must return without waiting");

    assert_eq!(
        session
            .state
            .lock()
            .await
            .unreported_omitted_tools(&omitted),
        omitted.to_vec()
    );
    assert!(matches!(
        rx.try_recv(),
        Err(async_channel::TryRecvError::Closed)
    ));
}
