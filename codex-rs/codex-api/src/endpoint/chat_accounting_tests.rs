use super::accounting::*;
use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::Semaphore;

#[derive(Default)]
struct Sink {
    values: Mutex<Vec<(i64, Result<ChatUsagePatch, InvalidChatUsage>)>>,
    committed: Mutex<Vec<i64>>,
    gate: Option<Semaphore>,
    reject: bool,
}
impl ChatUsageObserver for Sink {
    fn observe(
        &self,
        position: i64,
        usage: Result<ChatUsagePatch, InvalidChatUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            self.values.lock().unwrap().push((position, usage));
            if let Some(gate) = &self.gate {
                gate.acquire().await.unwrap().forget();
            }
            if self.reject {
                return Err(ApiError::Stream("fixture rejection".into()));
            }
            self.committed.lock().unwrap().push(position);
            Ok(())
        })
    }
}
fn data(value: Value) -> String {
    format!("data: {value}\n\n")
}
fn usage(value: Value) -> String {
    data(json!({"usage":value}))
}
fn finish(reason: &str) -> String {
    data(
        json!({"id":"fixture","choices":[{"index":0,"delta":{"content":"fixture"},"finish_reason":reason}]}),
    )
}
fn bytes(data: String) -> ByteStream {
    stream::iter(vec![Ok(data.into())]).boxed()
}
type Receiver = mpsc::Receiver<Result<ResponseEvent, ApiError>>;
fn launch(stream: ByteStream, sink: Option<Arc<Sink>>) -> (Receiver, tokio::task::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel(100);
    let task = tokio::spawn(process_chat_sse_observed(
        stream,
        tx,
        Duration::from_millis(100),
        Duration::from_millis(50),
        None,
        Some("fixture".into()),
        None,
        sink.map(|s| s as Arc<dyn ChatUsageObserver>),
    ));
    (rx, task)
}
async fn drain(mut rx: Receiver) -> Vec<Result<ResponseEvent, ApiError>> {
    let mut output = Vec::new();
    while let Some(value) = timeout(Duration::from_secs(2), rx.recv()).await.unwrap() {
        output.push(value);
    }
    output
}
fn completed(events: &[Result<ResponseEvent, ApiError>]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Ok(ResponseEvent::Completed { .. })))
        .count()
}
async fn observed(sink: &Sink) {
    timeout(Duration::from_secs(2), async {
        while sink.values.lock().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[test]
fn chat_accounting_presence_matrix() {
    for (key, detail) in [
        ("prompt_tokens", None),
        ("completion_tokens", None),
        ("total_tokens", None),
        ("cached_tokens", Some("prompt_tokens_details")),
        ("reasoning_tokens", Some("completion_tokens_details")),
    ] {
        for (value, expected) in [
            (Value::Null, ChatTokenPresence::Null),
            (json!(0), ChatTokenPresence::Number(0)),
            (json!(17), ChatTokenPresence::Number(17)),
            (json!(i64::MAX), ChatTokenPresence::Number(i64::MAX)),
        ] {
            let mut value = json!({key:value});
            if let Some(detail) = detail {
                value = json!({detail:value});
            }
            let got = accounting::decode(&json!({"usage":value}).to_string())
                .unwrap()
                .unwrap();
            let mut want = ChatUsagePatch::default();
            match key {
                "prompt_tokens" => want.input_tokens = expected,
                "completion_tokens" => want.output_tokens = expected,
                "total_tokens" => want.total_tokens = expected,
                "cached_tokens" => want.cached_tokens = expected,
                "reasoning_tokens" => want.reasoning_tokens = expected,
                _ => unreachable!(),
            }
            assert_eq!(got, want);
        }
        for bad in [
            json!(-1),
            json!(0.5),
            json!("private"),
            json!(true),
            json!(9223372036854775808_u64),
            json!([]),
        ] {
            let value = detail.map_or_else(|| json!({key:bad}), |d| json!({d:{key:bad}}));
            assert_eq!(
                accounting::decode(&json!({"usage":value}).to_string()),
                Err(InvalidChatUsage)
            );
        }
    }
    for key in ["prompt_tokens_details", "completion_tokens_details"] {
        for bad in [json!(false), json!([]), json!("private"), json!(3)] {
            assert_eq!(
                accounting::decode(&json!({"usage":{key:bad}}).to_string()),
                Err(InvalidChatUsage)
            );
        }
        let patch = accounting::decode(&json!({"usage":{key:null}}).to_string())
            .unwrap()
            .unwrap();
        assert_eq!(
            if key == "prompt_tokens_details" {
                patch.cached_tokens
            } else {
                patch.reasoning_tokens
            },
            ChatTokenPresence::Null
        );
    }
}

#[test]
fn chat_accounting_top_level_containers_only() {
    for value in [
        json!({"usage":{"prompt_tokens":7}}),
        json!({"choices":[],"usage":{"prompt_tokens":7}}),
        json!({"error":{"message":"fixture"},"usage":{"prompt_tokens":7}}),
    ] {
        assert_eq!(
            accounting::decode(&value.to_string()),
            Ok(Some(ChatUsagePatch {
                input_tokens: ChatTokenPresence::Number(7),
                ..Default::default()
            }))
        );
    }
    for value in [
        json!({}),
        json!({"usage":null}),
        json!({"error":{"usage":{"prompt_tokens":7}}}),
        json!({"response":{"usage":{"prompt_tokens":7}}}),
        json!({"billing":{"cost":99}}),
    ] {
        assert_eq!(accounting::decode(&value.to_string()), Ok(None));
    }
    for value in [
        json!({}),
        json!({"cache_write_tokens":5, "cost":7}),
        json!({"prompt_tokens_details":{"cache_write_tokens":8}}),
    ] {
        assert_eq!(
            accounting::decode(&json!({"usage":value}).to_string()),
            Ok(Some(ChatUsagePatch::default()))
        );
    }
    for value in [json!([]), json!(true), json!(5), json!("secret")] {
        assert_eq!(
            accounting::decode(&json!({"usage":value}).to_string()),
            Err(InvalidChatUsage)
        );
    }
}

#[tokio::test]
async fn chat_accounting_raw_before_lossy_chunk_conversion() {
    let sink = Arc::new(Sink::default());
    let (rx, task) = launch(
        bytes(
            usage(json!({"prompt_tokens":7,"completion_tokens":null}))
                + &finish("stop")
                + "data: [DONE]\n\n",
        ),
        Some(sink.clone()),
    );
    let output = drain(rx).await;
    task.await.unwrap();
    assert_eq!(
        *sink.values.lock().unwrap(),
        vec![(
            1,
            Ok(ChatUsagePatch {
                input_tokens: ChatTokenPresence::Number(7),
                output_tokens: ChatTokenPresence::Null,
                ..Default::default()
            })
        )]
    );
    assert_eq!(completed(&output), 1);
}

#[tokio::test]
async fn chat_accounting_error_envelope_usage_precedes_error() {
    for sibling in [true, false] {
        let sink = Arc::new(Sink {
            gate: Some(Semaphore::new(0)),
            ..Default::default()
        });
        let error = json!({"error":{"message":"fixture terminal"}});
        let input = if sibling {
            data(json!({"error":error["error"],"usage":{"prompt_tokens":7}}))
        } else {
            usage(json!({"prompt_tokens":7})) + &data(error)
        };
        let (mut rx, task) = launch(bytes(input), Some(sink.clone()));
        observed(&sink).await;
        assert!(timeout(Duration::from_millis(20), rx.recv()).await.is_err());
        assert!(sink.committed.lock().unwrap().is_empty());
        sink.gate.as_ref().unwrap().add_permits(1);
        let output = drain(rx).await;
        task.await.unwrap();
        assert_eq!(*sink.committed.lock().unwrap(), vec![1]);
        assert_eq!(completed(&output), 0);
        assert!(format!("{output:?}").contains("fixture terminal"));
    }
}

#[tokio::test]
async fn chat_accounting_done_and_finish_reason_are_not_usage() {
    for reason in [
        "stop",
        "length",
        "tool_calls",
        "content_filter",
        "error",
        "unknown",
    ] {
        let sink = Arc::new(Sink::default());
        let (rx, task) = launch(
            bytes(finish(reason) + "data: [DONE]\n\n"),
            Some(sink.clone()),
        );
        let output = drain(rx).await;
        task.await.unwrap();
        assert!(sink.values.lock().unwrap().is_empty());
        let (legacy, task) = launch(bytes(finish(reason) + "data: [DONE]\n\n"), None);
        assert_eq!(format!("{output:?}"), format!("{:?}", drain(legacy).await));
        task.await.unwrap();
    }
}

#[tokio::test]
async fn chat_accounting_positions_and_cumulative_patches() {
    let sink = Arc::new(Sink::default());
    let input = usage(json!({"prompt_tokens":7}))
        + ": keepalive\n\n"
        + &data(json!({"choices":[]}))
        + &usage(json!({"prompt_tokens":7}))
        + &data(json!({"choices":[]}))
        + &usage(json!({"prompt_tokens":null}))
        + &finish("stop")
        + "data: [DONE]\n\n";
    let (rx, task) = launch(bytes(input), Some(sink.clone()));
    assert_eq!(completed(&drain(rx).await), 1);
    task.await.unwrap();
    let values = sink.values.lock().unwrap();
    assert_eq!(
        values.iter().map(|v| v.0).collect::<Vec<_>>(),
        vec![1, 3, 5]
    );
    assert_eq!(values[0].1, values[1].1);
    assert_eq!(
        values[2].1.as_ref().unwrap().input_tokens,
        ChatTokenPresence::Null
    );
    assert_eq!(
        values[2].1.as_ref().unwrap().output_tokens,
        ChatTokenPresence::Missing
    );
}

#[tokio::test]
async fn chat_accounting_observation_barrier_and_rejection() {
    for reject in [false, true] {
        let sink = Arc::new(Sink {
            gate: Some(Semaphore::new(0)),
            reject,
            ..Default::default()
        });
        let (mut rx, task) = launch(
            bytes(usage(json!({"prompt_tokens":1})) + &finish("stop") + "data: [DONE]\n\n"),
            Some(sink.clone()),
        );
        observed(&sink).await;
        assert!(timeout(Duration::from_millis(20), rx.recv()).await.is_err());
        sink.gate.as_ref().unwrap().add_permits(1);
        let output = drain(rx).await;
        task.await.unwrap();
        assert_eq!(completed(&output), usize::from(!reject));
        assert_eq!(
            output.iter().filter(|v| v.is_err()).count(),
            usize::from(reject)
        );
    }
}

#[tokio::test]
async fn chat_accounting_invalid_evidence_stops_before_done() {
    for raw in [
        "private-invalid-json".to_string(),
        json!({"usage":{"prompt_tokens":-1}}).to_string(),
    ] {
        let sink = Arc::new(Sink::default());
        let (rx, task) = launch(
            bytes(format!("data: {raw}\n\ndata: [DONE]\n\n")),
            Some(sink.clone()),
        );
        let output = drain(rx).await;
        task.await.unwrap();
        assert_eq!(
            *sink.values.lock().unwrap(),
            vec![(1, Err(InvalidChatUsage))]
        );
        assert_eq!(completed(&output), 0);
        assert_eq!(
            format!("{output:?}"),
            "[Err(Stream(\"Chat accounting evidence rejected\"))]"
        );
    }
}

#[tokio::test]
async fn chat_accounting_consumer_cancel_and_interruption() {
    for held in [false, true] {
        let sink = Arc::new(Sink {
            gate: Some(Semaphore::new(0)),
            ..Default::default()
        });
        let stream = if held {
            bytes(usage(json!({"prompt_tokens":7})))
        } else {
            stream::pending().boxed()
        };
        let (rx, task) = launch(stream, Some(sink.clone()));
        if held {
            observed(&sink).await;
        }
        drop(rx);
        timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap();
        assert!(sink.committed.lock().unwrap().is_empty());
    }
    for suffix in [0, 1, 2] {
        let sink = Arc::new(Sink::default());
        let first = bytes(usage(json!({"prompt_tokens":7})));
        let stream = match suffix {
            0 => first,
            1 => first.chain(stream::pending()).boxed(),
            _ => first
                .chain(stream::once(async { Ok(": keepalive\n\n".into()) }))
                .chain(stream::pending())
                .boxed(),
        };
        let (rx, task) = launch(stream, Some(sink.clone()));
        let output = drain(rx).await;
        task.await.unwrap();
        assert_eq!(*sink.committed.lock().unwrap(), vec![1]);
        assert_eq!(completed(&output), 0);
        assert!(output.last().unwrap().is_err());
    }
}

#[tokio::test]
async fn chat_accounting_none_preserves_legacy() {
    for input in [
        usage(json!({"prompt_tokens":7,"completion_tokens":3,"total_tokens":10}))
            + &finish("stop")
            + "data: [DONE]\n\n",
        "data: malformed\n\n: keepalive\n\ndata: [DONE]\n\n".into(),
        data(json!({"error":{"message":"fixture"}})),
    ] {
        let (rx, task) = launch(bytes(input.clone()), None);
        let observed = drain(rx).await;
        task.await.unwrap();
        let (tx, rx) = mpsc::channel(100);
        process_chat_sse(
            bytes(input),
            tx,
            Duration::from_millis(100),
            Duration::from_millis(50),
            None,
            Some("fixture".into()),
            None,
        )
        .await;
        assert_eq!(format!("{observed:?}"), format!("{:?}", drain(rx).await));
    }
}
