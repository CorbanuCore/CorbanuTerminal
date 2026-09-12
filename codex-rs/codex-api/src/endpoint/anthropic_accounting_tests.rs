use super::*;
use crate::common::ResponseEvent;
use crate::endpoint::anthropic_messages::process_accounted_anthropic_sse;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;

#[test]
fn anthropic_accounting_presence_is_lossless_and_numeric_only() {
    for field in [
        "input_tokens",
        "cache_read_input_tokens",
        "cache_creation_input_tokens",
        "output_tokens",
    ] {
        for value in [Value::Null, json!(0), json!(17), json!(i64::MAX)] {
            let mut usage = serde_json::Map::new();
            usage.insert(field.into(), value.clone());
            usage.insert("private_text".into(), json!("not retained"));
            let decoded = decode(&json!({"type":"message_delta","usage":usage}).to_string())
                .unwrap()
                .unwrap();
            let expected_value = if value.is_null() {
                AnthropicTokenPresence::Null
            } else {
                AnthropicTokenPresence::Number(value.as_i64().unwrap())
            };
            let mut expected = AnthropicUsagePatch::default();
            match field {
                "input_tokens" => expected.input_tokens = expected_value,
                "cache_read_input_tokens" => expected.cache_read_input_tokens = expected_value,
                "cache_creation_input_tokens" => {
                    expected.cache_creation_input_tokens = expected_value
                }
                "output_tokens" => expected.output_tokens = expected_value,
                _ => unreachable!(),
            }
            assert_eq!(decoded, expected);
        }
        for value in [
            json!(-1),
            json!(0.0),
            json!(true),
            json!("2"),
            json!(u64::MAX),
        ] {
            assert_eq!(
                decode(
                    &json!({"type":"message_start","message":{"usage":{field:value}}}).to_string()
                ),
                Err(InvalidAnthropicUsage)
            );
        }
    }
    for usage in [json!({}), json!({"usage":null})] {
        let value = json!({"type":"message_start","message":usage});
        assert_eq!(decode(&value.to_string()), Ok(None));
    }
    assert_eq!(
        decode(r#"{"type":"message_delta","usage":{}}"#),
        Ok(Some(AnthropicUsagePatch::default()))
    );
    assert_eq!(
        decode(r#"{"type":"content_block_delta","delta":{"text":"private"}}"#),
        Ok(None)
    );
}

#[derive(Default)]
struct Recording {
    rows: Mutex<Vec<(i64, Result<AnthropicUsagePatch, InvalidAnthropicUsage>)>>,
}

impl AnthropicUsageObserver for Recording {
    fn observe(
        &self,
        position: i64,
        usage: Result<AnthropicUsagePatch, InvalidAnthropicUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().unwrap().push((position, usage));
            Ok(())
        })
    }
}

fn events(values: &[Value]) -> String {
    values
        .iter()
        .map(|value| format!("data: {value}\n\n"))
        .collect()
}

async fn run(
    data: String,
    observer: Option<Arc<dyn AnthropicUsageObserver>>,
) -> Vec<Result<ResponseEvent, ApiError>> {
    let (tx, mut rx) = mpsc::channel(32);
    process_accounted_anthropic_sse(
        futures::stream::iter([Ok(data.into())]).boxed(),
        tx,
        Duration::from_secs(2),
        /*telemetry*/ None,
        /*response_id_hint*/ None,
        observer,
    )
    .await;
    let mut result = Vec::new();
    while let Some(event) = rx.recv().await {
        result.push(event);
    }
    result
}

#[tokio::test]
async fn anthropic_accounting_source_positions_precede_completion_and_survive_provider_error() {
    let observer = Arc::new(Recording::default());
    let result = run(events(&[
        json!({"type":"ping"}),
            json!({"type":"message_start","message":{"id":"reused","usage":{"input_tokens":7,"output_tokens":0}}}),
            json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}),
            json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"fixture"}}),
            json!({"type":"content_block_stop","index":0}),
        json!({"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":3,"cache_read_input_tokens":null}}),
        json!({"type":"message_stop"}),
    ]), Some(observer.clone())).await;
    assert!(
        result
            .iter()
            .any(|event| matches!(event, Ok(ResponseEvent::Completed { .. })))
    );
    assert_eq!(
        *observer.rows.lock().unwrap(),
        vec![
            (
                2,
                Ok(AnthropicUsagePatch {
                    input_tokens: AnthropicTokenPresence::Number(7),
                    output_tokens: AnthropicTokenPresence::Number(0),
                    ..Default::default()
                })
            ),
            (
                6,
                Ok(AnthropicUsagePatch {
                    output_tokens: AnthropicTokenPresence::Number(3),
                    cache_read_input_tokens: AnthropicTokenPresence::Null,
                    ..Default::default()
                })
            ),
        ]
    );
    let observer = Arc::new(Recording::default());
    let result = run(
        events(&[
            json!({"type":"message_start","message":{"usage":{"input_tokens":7}}}),
            json!({"type":"error","error":{"type":"overloaded_error","message":"fixture"}}),
        ]),
        Some(observer.clone()),
    )
    .await;
    assert_eq!(observer.rows.lock().unwrap().len(), 1);
    assert!(result.last().unwrap().is_err());
}

#[tokio::test]
async fn anthropic_accounting_invalid_evidence_notifies_sink_and_never_completes() {
    let observer = Arc::new(Recording::default());
    let result = run(
        events(&[
            json!({"type":"message_start","message":{"usage":{"input_tokens":-1}}}),
            json!({"type":"message_stop"}),
        ]),
        Some(observer.clone()),
    )
    .await;
    assert_eq!(
        *observer.rows.lock().unwrap(),
        vec![(1, Err(InvalidAnthropicUsage))]
    );
    assert_eq!(result.len(), 1);
    assert!(result[0].is_err());
}

struct Gate {
    entered: tokio::sync::Notify,
    release: tokio::sync::Notify,
}

impl AnthropicUsageObserver for Gate {
    fn observe(
        &self,
        _: i64,
        _: Result<AnthropicUsagePatch, InvalidAnthropicUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            self.entered.notify_one();
            self.release.notified().await;
            Err(ApiError::Stream("fixture persistence failure".into()))
        })
    }
}

#[tokio::test]
async fn anthropic_accounting_awaits_persistence_and_stops_before_completion() {
    let gate = Arc::new(Gate {
        entered: Default::default(),
        release: Default::default(),
    });
    let handle = tokio::spawn(run(
        events(&[
            json!({"type":"message_start","message":{"usage":{"input_tokens":0}}}),
            json!({"type":"message_stop"}),
        ]),
        Some(gate.clone()),
    ));
    gate.entered.notified().await;
    assert!(!handle.is_finished());
    gate.release.notify_one();
    let result = handle.await.unwrap();
    assert!(result.last().unwrap().is_err());
    assert!(
        !result
            .iter()
            .any(|event| matches!(event, Ok(ResponseEvent::Completed { .. })))
    );
}
