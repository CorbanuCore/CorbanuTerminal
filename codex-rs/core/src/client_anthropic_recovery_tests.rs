use super::*;
use pretty_assertions::assert_eq;
use tokio::sync::mpsc;

fn rejection() -> ApiError {
    ApiError::Stream(
        "invalid_request_error: thinking blocks in the latest assistant message cannot be modified"
            .to_owned(),
    )
}

fn stream(events: Vec<Result<ResponseEvent, ApiError>>) -> ResponseStream {
    let (sender, rx_event) = mpsc::channel(events.len().max(1));
    for event in events {
        sender.try_send(event).expect("fixture capacity");
    }
    ResponseStream {
        rx_event,
        upstream_request_id: Some("synthetic-request".to_owned()),
    }
}

#[tokio::test]
async fn metadata_before_rejection_is_still_recoverable() {
    let mut stream = stream(vec![
        Ok(ResponseEvent::Created),
        Ok(ResponseEvent::ServerModel("synthetic-model".to_owned())),
        Ok(ResponseEvent::ServerReasoningIncluded(true)),
        Err(rejection()),
    ]);
    assert!(prefetch(&mut stream).await.is_err());
    assert_eq!(
        stream.upstream_request_id.as_deref(),
        Some("synthetic-request")
    );
}

#[tokio::test]
async fn visible_output_prevents_replay_and_preserves_the_later_error() {
    let mut stream = stream(vec![
        Ok(ResponseEvent::Created),
        Ok(ResponseEvent::OutputTextDelta {
            item_id: None,
            delta: "x".to_owned(),
        }),
        Err(rejection()),
    ]);
    let prefix = prefetch(&mut stream).await.expect("output is not retried");
    assert_eq!(prefix.len(), 2);
    assert!(matches!(&prefix[1], Ok(ResponseEvent::OutputTextDelta { delta, .. }) if delta == "x"));
    assert!(matches!(stream.next().await, Some(Err(_))));
}

#[tokio::test]
async fn tool_input_and_reasoning_stop_prefetch_even_before_display() {
    for output in [
        ResponseEvent::ToolCallInputDelta {
            item_id: "item".to_owned(),
            call_id: None,
            delta: "{}".to_owned(),
        },
        ResponseEvent::ReasoningContentDelta {
            delta: "thinking".to_owned(),
            content_index: 0,
        },
    ] {
        let mut stream = stream(vec![Ok(output), Err(rejection())]);
        assert_eq!(
            prefetch(&mut stream).await.expect("output boundary").len(),
            1
        );
        assert!(matches!(stream.next().await, Some(Err(_))));
    }
}

#[tokio::test]
async fn unrelated_error_is_forwarded_without_retry() {
    let mut stream = stream(vec![Err(ApiError::Stream(
        "failed to parse event containing thinking blocks in the latest assistant message cannot be modified".to_owned(),
    ))]);
    let prefix = prefetch(&mut stream)
        .await
        .expect("not a provider rejection");
    assert!(matches!(&prefix[..], [Err(ApiError::Stream(_))]));
}

#[tokio::test]
async fn metadata_buffer_is_bounded_and_never_discards_overflow() {
    let mut events: Vec<_> = (0..33).map(|_| Ok(ResponseEvent::Created)).collect();
    events.push(Err(rejection()));
    let mut stream = stream(events);
    assert_eq!(
        prefetch(&mut stream).await.expect("bounded prefix").len(),
        32
    );
    assert!(matches!(
        stream.next().await,
        Some(Ok(ResponseEvent::Created))
    ));
    assert!(matches!(stream.next().await, Some(Err(_))));

    let mut stream = self::stream(vec![
        Ok(ResponseEvent::ServerModel("x".repeat(1025))),
        Err(rejection()),
    ]);
    assert_eq!(
        prefetch(&mut stream).await.expect("large metadata").len(),
        1
    );
    assert!(matches!(stream.next().await, Some(Err(_))));
}

#[tokio::test]
async fn empty_and_completed_streams_do_not_trigger_recovery() {
    let mut empty = stream(vec![]);
    assert!(prefetch(&mut empty).await.expect("empty stream").is_empty());
    let mut completed = stream(vec![Ok(ResponseEvent::Completed {
        response_id: "complete".to_owned(),
        token_usage: None,
        end_turn: Some(true),
        finish_reason: None,
    })]);
    assert!(matches!(
        &prefetch(&mut completed).await.expect("completed stream")[..],
        [Ok(ResponseEvent::Completed { .. })]
    ));
}
