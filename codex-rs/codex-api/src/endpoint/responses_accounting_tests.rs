use super::*;
use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::sse::responses::spawn_response_stream_with_observer;
use codex_client::StreamResponse;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Default)]
struct Sink {
    values: Mutex<Vec<(i64, Result<ResponsesUsagePatch, InvalidResponsesUsage>)>>,
    gate: Option<Semaphore>,
    reject: bool,
}
impl ResponsesUsageObserver for Sink {
    fn observe(
        &self,
        position: i64,
        usage: Result<ResponsesUsagePatch, InvalidResponsesUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            self.values.lock().unwrap().push((position, usage));
            if let Some(gate) = &self.gate {
                gate.acquire().await.unwrap().forget();
            }
            if self.reject {
                Err(ApiError::Stream("fixture rejection".into()))
            } else {
                Ok(())
            }
        })
    }
}
fn stream(events: Vec<Value>, sink: Option<Arc<Sink>>) -> ResponseStream {
    let data: String = events
        .iter()
        .map(|value| format!("data: {value}\n\n"))
        .collect();
    spawn_response_stream_with_observer(
        StreamResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            bytes: futures::stream::iter(vec![Ok(data.into())]).boxed(),
        },
        Duration::from_secs(1),
        None,
        None,
        sink.map(|value| value as Arc<dyn ResponsesUsageObserver>),
    )
}
fn event(kind: &str, usage: Value) -> Value {
    json!({"type":kind,"response":{"id":"fixture","usage":usage}})
}
async fn drain(mut stream: ResponseStream) -> Vec<Result<ResponseEvent, ApiError>> {
    let mut values = Vec::new();
    while let Some(value) = stream.next().await {
        values.push(value);
    }
    values
}
fn completed(events: &[Result<ResponseEvent, ApiError>]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Ok(ResponseEvent::Completed { .. })))
        .count()
}

#[test]
fn responses_accounting_presence_matrix() {
    for (key, detail) in [
        ("input_tokens", None),
        ("output_tokens", None),
        ("total_tokens", None),
        ("cached_tokens", Some("input_tokens_details")),
        ("cache_write_tokens", Some("input_tokens_details")),
        ("reasoning_tokens", Some("output_tokens_details")),
    ] {
        for (value, expected) in [
            (Value::Null, ResponsesTokenPresence::Null),
            (json!(0), ResponsesTokenPresence::Number(0)),
            (json!(17), ResponsesTokenPresence::Number(17)),
            (json!(i64::MAX), ResponsesTokenPresence::Number(i64::MAX)),
        ] {
            let mut usage = json!({});
            if let Some(detail) = detail {
                usage[detail] = json!({key: value});
            } else {
                usage[key] = value;
            }
            let got = decode(&event("response.completed", usage).to_string())
                .unwrap()
                .unwrap();
            let mut want = ResponsesUsagePatch::default();
            match key {
                "input_tokens" => want.input_tokens = expected,
                "output_tokens" => want.output_tokens = expected,
                "total_tokens" => want.total_tokens = expected,
                "cached_tokens" => want.cached_tokens = expected,
                "cache_write_tokens" => want.cache_write_tokens = expected,
                "reasoning_tokens" => want.reasoning_tokens = expected,
                _ => unreachable!(),
            }
            assert_eq!(got, want);
        }
        for bad in [
            json!(-1),
            json!(0.5),
            json!("private-body"),
            json!(true),
            json!(9223372036854775808_u64),
            json!([]),
        ] {
            let usage = if let Some(detail) = detail {
                json!({detail:{key:bad}})
            } else {
                json!({key:bad})
            };
            assert_eq!(
                decode(&event("response.completed", usage).to_string()),
                Err(InvalidResponsesUsage)
            );
        }
    }
    for bad in [json!(false), json!([]), json!("private-body")] {
        assert_eq!(
            decode(&event("response.completed", bad.clone()).to_string()),
            Err(InvalidResponsesUsage)
        );
        assert_eq!(
            decode(&event("response.completed", json!({"input_tokens_details":bad})).to_string()),
            Err(InvalidResponsesUsage)
        );
    }
    assert_eq!(
        format!("{:?}", InvalidResponsesUsage),
        "InvalidResponsesUsage"
    );
}

#[tokio::test]
async fn responses_accounting_completed_before_lossy_conversion() {
    let sink = Arc::new(Sink::default());
    let events = drain(stream(
        vec![event("response.completed", json!({"input_tokens":7}))],
        Some(sink.clone()),
    ))
    .await;
    assert_eq!(
        *sink.values.lock().unwrap(),
        vec![(
            1,
            Ok(ResponsesUsagePatch {
                input_tokens: ResponsesTokenPresence::Number(7),
                ..Default::default()
            })
        )]
    );
    assert_eq!(completed(&events), 0);
    assert!(events.last().unwrap().is_err());
}

#[test]
fn responses_accounting_separate_usage_containers() {
    let usage = json!({"input_tokens":4,"input_tokens_details":null});
    let expected = Some(ResponsesUsagePatch {
        input_tokens: ResponsesTokenPresence::Number(4),
        cached_tokens: ResponsesTokenPresence::Null,
        cache_write_tokens: ResponsesTokenPresence::Null,
        ..Default::default()
    });
    for value in [
        json!({"type":"response.usage","usage":usage}),
        event("response.usage", usage.clone()),
        json!({"type":"response.usage","usage":null,"response":{"usage":usage}}),
        json!({"type":"response.usage","usage":usage,"response":{"usage":usage}}),
    ] {
        assert_eq!(decode(&value.to_string()), Ok(expected.clone()));
    }
    assert_eq!(
        decode(
            &json!({"type":"response.usage","usage":usage,"response":{"usage":{"input_tokens":5}}})
                .to_string()
        ),
        Err(InvalidResponsesUsage)
    );
    for value in [
        json!({"type":"response.usage"}),
        event("response.completed", Value::Null),
    ] {
        assert_eq!(decode(&value.to_string()), Ok(None));
    }
}

#[tokio::test]
async fn responses_accounting_terminal_usage_retained() {
    for kind in ["response.failed", "response.incomplete"] {
        let sink = Arc::new(Sink::default());
        let mut value = event(kind, json!({"output_tokens":9}));
        value["response"]["error"] = json!({"code":"invalid_request_error","message":"fixture"});
        value["response"]["incomplete_details"] = json!({"reason":"max_output_tokens"});
        let events = drain(stream(vec![value], Some(sink.clone()))).await;
        assert_eq!(
            *sink.values.lock().unwrap(),
            vec![(
                1,
                Ok(ResponsesUsagePatch {
                    output_tokens: ResponsesTokenPresence::Number(9),
                    ..Default::default()
                })
            )]
        );
        assert_eq!(completed(&events), 0);
        assert!(events.iter().any(Result::is_err));
    }
}

#[tokio::test]
async fn responses_accounting_positions_and_replacement() {
    let sink = Arc::new(Sink::default());
    let events = vec![
        json!({"type":"response.created","response":{"id":"fixture"}}),
        event("response.usage", json!({"input_tokens":3})),
        json!({"type":"response.output_text.delta","delta":"fixture"}),
        event("response.usage", json!({"input_tokens":null})),
        event("response.usage", json!({"output_tokens":2})),
        event("response.usage", json!({"input_tokens":7})),
        event("response.usage", json!({"input_tokens":7})),
        event("response.completed", Value::Null),
    ];
    let output = drain(stream(events, Some(sink.clone()))).await;
    assert_eq!(completed(&output), 1);
    let values = sink.values.lock().unwrap();
    assert_eq!(
        values.iter().map(|v| v.0).collect::<Vec<_>>(),
        vec![2, 4, 5, 6, 7]
    );
    assert_eq!(values[3].1, values[4].1);
    assert_eq!(
        values[1].1.as_ref().unwrap().input_tokens,
        ResponsesTokenPresence::Null
    );
    assert_eq!(
        values[2].1.as_ref().unwrap().input_tokens,
        ResponsesTokenPresence::Missing
    );
}

#[tokio::test]
async fn responses_accounting_awaits_observation_before_completion() {
    for reject in [false, true] {
        let sink = Arc::new(Sink {
            gate: Some(Semaphore::new(0)),
            reject,
            ..Default::default()
        });
        let mut stream = stream(
            vec![event(
                "response.completed",
                json!({"input_tokens":1,"output_tokens":2,"total_tokens":3}),
            )],
            Some(sink.clone()),
        );
        assert!(matches!(
            stream.next().await,
            Some(Ok(ResponseEvent::RateLimits(_)))
        ));
        assert!(
            tokio::time::timeout(Duration::from_millis(30), stream.next())
                .await
                .is_err()
        );
        assert_eq!(sink.values.lock().unwrap().len(), 1);
        sink.gate.as_ref().unwrap().add_permits(1);
        let output = drain(stream).await;
        assert_eq!(completed(&output), usize::from(!reject));
        assert_eq!(
            output.iter().filter(|v| v.is_err()).count(),
            usize::from(reject)
        );
    }
}

#[tokio::test]
async fn responses_accounting_invalid_usage_latches_and_stops() {
    let sink = Arc::new(Sink::default());
    let output = drain(stream(
        vec![
            event("response.usage", json!({"input_tokens":-1})),
            event("response.completed", Value::Null),
        ],
        Some(sink.clone()),
    ))
    .await;
    assert_eq!(
        *sink.values.lock().unwrap(),
        vec![(1, Err(InvalidResponsesUsage))]
    );
    assert_eq!(completed(&output), 0);
    assert_eq!(output.len(), 2);
    assert!(matches!(output[0], Ok(ResponseEvent::RateLimits(_))));
    assert!(output[1].is_err());
}

#[tokio::test]
async fn responses_accounting_none_preserves_legacy_stream() {
    let payload = vec![
        json!({"type":"response.created","response":{"id":"fixture"},"safety_buffering":false}),
        event("response.usage", json!({"input_tokens":-1})),
        event(
            "response.completed",
            json!({"input_tokens":1,"output_tokens":2,"total_tokens":3}),
        ),
    ];
    let output = drain(stream(payload.clone(), None)).await;
    assert_eq!(completed(&output), 1);
    let data: String = payload
        .iter()
        .map(|value| format!("data: {value}\n\n"))
        .collect();
    let legacy = crate::sse::spawn_response_stream(
        StreamResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            bytes: futures::stream::iter(vec![Ok(data.into())]).boxed(),
        },
        Duration::from_secs(1),
        None,
        None,
    );
    assert_eq!(format!("{output:?}"), format!("{:?}", drain(legacy).await));
}
