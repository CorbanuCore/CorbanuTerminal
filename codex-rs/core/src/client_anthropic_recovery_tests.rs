use super::*;
use pretty_assertions::assert_eq;
use tokio::sync::mpsc;

async fn prefetch(
    stream: &mut ResponseStream,
) -> Result<Vec<Result<ResponseEvent, ApiError>>, ApiError> {
    super::prefetch(stream, || {}).await
}

#[tokio::test]
async fn cancelled_prefetch_calls_lifecycle_handler_once() {
    let (sender, rx_event) = mpsc::channel(1);
    let mut stream = ResponseStream {
        rx_event,
        upstream_request_id: Some("synthetic-cancelled-request".to_owned()),
    };
    let recorded = std::cell::RefCell::new(Vec::new());
    let request_id = stream.upstream_request_id.take();
    let mut future = Box::pin(super::prefetch(&mut stream, || {
        recorded.borrow_mut().push(request_id);
    }));
    assert!(futures::poll!(&mut future).is_pending());
    drop(future);
    assert_eq!(
        &*recorded.borrow(),
        &[Some("synthetic-cancelled-request".to_owned())]
    );
    drop(sender);
}

#[tokio::test]
async fn completed_prefetch_does_not_report_cancellation() {
    let called = std::cell::Cell::new(false);
    let mut stream = stream(vec![Err(rejection())]);
    assert!(
        super::prefetch(&mut stream, || called.set(true))
            .await
            .is_err()
    );
    assert!(!called.get());
}

#[test]
fn forwarded_history_rejection_is_not_retryable() {
    let Err(error) = terminalize(Err(rejection())) else {
        panic!("expected failure");
    };
    assert!(!codex_api::map_api_error(error).is_retryable());
    let Err(error) = terminalize(Err(ApiError::Stream("ordinary connection loss".to_owned())))
    else {
        panic!("expected failure");
    };
    assert!(codex_api::map_api_error(error).is_retryable());
}

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
        Ok(ResponseEvent::Created { response_id: None }),
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
        Ok(ResponseEvent::Created { response_id: None }),
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
    let mut events: Vec<_> = (0..33).map(|_| Ok(ResponseEvent::Created { response_id: None })).collect();
    events.push(Err(rejection()));
    let mut stream = stream(events);
    assert_eq!(
        prefetch(&mut stream).await.expect("bounded prefix").len(),
        32
    );
    assert!(matches!(
        stream.next().await,
        Some(Ok(ResponseEvent::Created { response_id: None }))
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
