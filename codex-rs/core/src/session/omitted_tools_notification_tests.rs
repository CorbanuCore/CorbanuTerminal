//! #688: concurrent startup/turn notices report each omitted tool once per
//! session, while cancellation before delivery leaves it reportable.
//! These source-aware cases exercise the actual Session event path.

use std::future::Future;
use std::time::Duration;

use async_channel::Receiver;
use codex_protocol::ToolName;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InternalSessionSource;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::protocol::WarningEvent;
use pretty_assertions::assert_eq;

use super::notify_omitted_tools;
use super::omitted_tools_notice;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;

async fn notice_session() -> (Session, TurnContext, Receiver<Event>) {
    let (mut session, context) = crate::session::tests::make_session_and_context().await;
    // The existing fixture has no rollout writer or realtime session. The
    // full event channel is therefore the notification's blocking boundary.
    assert!(session.services.live_thread.is_none());
    let (tx, rx) = async_channel::bounded(/*cap*/ 1);
    session.tx_event = tx;
    (session, context, rx)
}

fn tool(name: &str) -> ToolName {
    ToolName::namespaced("mcp__bulk", name)
}

fn fill_event_channel(session: &Session) {
    session
        .tx_event
        .try_send(Event {
            id: "backpressure-control".to_string(),
            msg: EventMsg::Warning(WarningEvent {
                message: "occupied event channel".to_string(),
            }),
        })
        .expect("the initially empty event channel has one slot");
}

fn release_event_channel(rx: &Receiver<Event>) {
    let marker = rx
        .try_recv()
        .expect("the control occupies the event channel");
    assert_eq!(marker.id, "backpressure-control");
    assert_eq!(warning_message(marker), "occupied event channel");
}

fn warning_message(event: Event) -> String {
    let EventMsg::Warning(warning) = event.msg else {
        panic!("notification emitted a non-warning event");
    };
    warning.message
}

async fn finish_and_collect(
    finished: impl Future<Output = ()>,
    rx: &Receiver<Event>,
) -> Vec<String> {
    tokio::time::timeout(Duration::from_secs(/*secs*/ 5), async {
        let finish = async {
            finished.await;
            rx.close();
        };
        let collect = async {
            let mut messages = Vec::new();
            while let Ok(event) = rx.recv().await {
                messages.push(warning_message(event));
            }
            messages
        };
        let ((), messages) = tokio::join!(finish, collect);
        messages
    })
    .await
    .expect("notifications and event draining must finish")
}

async fn concurrent_notices(first_tools: &[ToolName], second_tools: &[ToolName]) -> Vec<String> {
    let (session, context, rx) = notice_session().await;
    fill_event_channel(&session);
    // Remove cooperative-budget yielding from this manual poll, so the
    // first pending notification is blocked by the actual full event queue.
    let mut first = Box::pin(tokio::task::unconstrained(notify_omitted_tools(
        &session,
        &context,
        first_tools,
    )));
    assert!(futures::poll!(first.as_mut()).is_pending());
    let mut second = Box::pin(tokio::task::unconstrained(notify_omitted_tools(
        &session,
        &context,
        second_tools,
    )));
    let second_pending = futures::poll!(second.as_mut()).is_pending();
    release_event_channel(&rx);
    finish_and_collect(
        async {
            if second_pending {
                tokio::join!(first, second);
            } else {
                first.await;
            }
        },
        &rx,
    )
    .await
}

#[tokio::test]
async fn concurrent_same_tools_emit_one_session_warning() {
    let messages = concurrent_notices(&[tool("a")], &[tool("a")]).await;
    assert_eq!(
        messages,
        vec!["Corbanu sends at most 128 tools per request to providers without tool namespaces, so these are not offered to the model (1): `mcp__bulk__a`. Turn off MCP servers or tools you don't need, or use a provider with tool namespaces (such as OpenAI), to offer them.".to_string()]
    );
}

#[tokio::test]
async fn overlapping_concurrent_notices_never_repeat_a_tool() {
    let mut messages = concurrent_notices(&[tool("a"), tool("b")], &[tool("b"), tool("c")]).await;
    messages.sort();
    // Either sender may win, but each tool must be attributed exactly once.
    let mut first_wins = vec![
        omitted_tools_notice(&["mcp__bulk__a".to_string(), "mcp__bulk__b".to_string()]),
        omitted_tools_notice(&["mcp__bulk__c".to_string()]),
    ];
    first_wins.sort();
    let mut second_wins = vec![
        omitted_tools_notice(&["mcp__bulk__a".to_string()]),
        omitted_tools_notice(&["mcp__bulk__b".to_string(), "mcp__bulk__c".to_string()]),
    ];
    second_wins.sort();
    assert!(
        messages == first_wins || messages == second_wins,
        "{messages:?}"
    );
}

#[tokio::test]
async fn cancelling_a_blocked_notice_keeps_its_tools_reportable() {
    let (session, context, rx) = notice_session().await;
    let omitted = [tool("a")];
    fill_event_channel(&session);
    let mut cancelled = Box::pin(tokio::task::unconstrained(notify_omitted_tools(
        &session, &context, &omitted,
    )));
    assert!(futures::poll!(cancelled.as_mut()).is_pending());
    drop(cancelled);
    release_event_channel(&rx);
    let messages =
        finish_and_collect(notify_omitted_tools(&session, &context, &omitted), &rx).await;
    assert_eq!(
        messages,
        vec![omitted_tools_notice(&["mcp__bulk__a".to_string()])]
    );
}

#[tokio::test]
async fn cancelling_one_concurrent_notice_keeps_the_other_and_allows_retry() {
    let (session, context, rx) = notice_session().await;
    let cancelled_tools = [tool("a")];
    let other_tools = [tool("b")];
    fill_event_channel(&session);
    let mut cancelled = Box::pin(tokio::task::unconstrained(notify_omitted_tools(
        &session,
        &context,
        &cancelled_tools,
    )));
    assert!(futures::poll!(cancelled.as_mut()).is_pending());
    let mut other = Box::pin(tokio::task::unconstrained(notify_omitted_tools(
        &session,
        &context,
        &other_tools,
    )));
    let other_pending = futures::poll!(other.as_mut()).is_pending();
    drop(cancelled);
    release_event_channel(&rx);
    let messages = finish_and_collect(
        async {
            if other_pending {
                other.await;
            }
            notify_omitted_tools(&session, &context, &cancelled_tools).await;
        },
        &rx,
    )
    .await;
    assert_eq!(
        messages,
        vec![
            omitted_tools_notice(&["mcp__bulk__b".to_string()]),
            omitted_tools_notice(&["mcp__bulk__a".to_string()]),
        ]
    );
}

#[tokio::test]
async fn delivered_tools_stay_quiet_while_newly_omitted_tools_are_reported() {
    let (session, context, rx) = notice_session().await;
    let messages = finish_and_collect(
        async {
            notify_omitted_tools(&session, &context, &[tool("a")]).await;
            notify_omitted_tools(&session, &context, &[tool("a")]).await;
            notify_omitted_tools(&session, &context, &[tool("a"), tool("b")]).await;
        },
        &rx,
    )
    .await;
    assert_eq!(
        messages,
        vec![
            omitted_tools_notice(&["mcp__bulk__a".to_string()]),
            omitted_tools_notice(&["mcp__bulk__b".to_string()]),
        ]
    );
}

#[tokio::test]
async fn separate_sessions_each_report_the_same_omitted_tool() {
    for _ in 0..2 {
        let (session, context, rx) = notice_session().await;
        let messages =
            finish_and_collect(notify_omitted_tools(&session, &context, &[tool("a")]), &rx).await;
        assert_eq!(
            messages,
            vec![omitted_tools_notice(&["mcp__bulk__a".to_string()])]
        );
    }
}

#[tokio::test]
async fn quiet_sources_and_empty_input_do_not_consume_the_regular_notice() {
    let (session, mut context, rx) = notice_session().await;
    let messages = finish_and_collect(
        async {
            for source in [
                SessionSource::Internal(InternalSessionSource::MemoryConsolidation),
                SessionSource::SubAgent(SubAgentSource::Review),
                SessionSource::SubAgent(SubAgentSource::Compact),
                SessionSource::SubAgent(SubAgentSource::MemoryConsolidation),
                SessionSource::SubAgent(SubAgentSource::Other("internal fixture".to_string())),
            ] {
                context.session_source = source;
                notify_omitted_tools(&session, &context, &[tool("a")]).await;
            }
            context.session_source = SessionSource::Exec;
            notify_omitted_tools(&session, &context, &[]).await;
            notify_omitted_tools(&session, &context, &[tool("a")]).await;
        },
        &rx,
    )
    .await;
    assert_eq!(
        messages,
        vec![omitted_tools_notice(&["mcp__bulk__a".to_string()])]
    );
}
