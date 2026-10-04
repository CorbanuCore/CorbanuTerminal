use super::*;
use crate::InvalidResponsesUsage;
use crate::ResponsesTokenPresence;
use crate::ResponsesUsageObserver;
use crate::ResponsesUsagePatch;
use futures::FutureExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::sync::Semaphore;

#[derive(Default)]
struct Sink {
    records: StdMutex<Vec<(i64, Result<ResponsesUsagePatch, InvalidResponsesUsage>)>>,
    gate: Option<Semaphore>,
    reject: AtomicBool,
}
impl ResponsesUsageObserver for Sink {
    fn observe(
        &self,
        position: i64,
        usage: Result<ResponsesUsagePatch, InvalidResponsesUsage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async move {
            if let Some(gate) = &self.gate {
                gate.acquire().await.unwrap().forget();
            }
            if self.reject.load(Ordering::SeqCst) {
                return Err(ApiError::Stream("fixture write rejected".into()));
            }
            self.records.lock().unwrap().push((position, usage));
            Ok(())
        })
    }
}
struct Admit {
    sink: Arc<Sink>,
    gate: Semaphore,
    calls: AtomicUsize,
    reject: AtomicBool,
}
impl Admit {
    fn new(permits: usize) -> Arc<Self> {
        Arc::new(Self {
            sink: Arc::new(Sink::default()),
            gate: Semaphore::new(permits),
            calls: AtomicUsize::new(0),
            reject: AtomicBool::new(false),
        })
    }
}
impl accounting::ResponsesWebsocketAdmission for Admit {
    fn admit(
        &self,
        model: String,
        tier: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Arc<dyn ResponsesUsageObserver>, ApiError>> + Send + '_>>
    {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(
                (model.as_str(), tier.as_deref()),
                ("gpt-5.6-sol", Some("default"))
            );
            self.gate.acquire().await.unwrap().forget();
            if self.reject.load(Ordering::SeqCst) {
                return Err(ApiError::Stream("fixture admission rejected".into()));
            }
            Ok(self.sink.clone() as Arc<dyn ResponsesUsageObserver>)
        })
    }
}
fn request() -> ResponsesWsRequest<'static> {
    ResponsesWsRequest::ResponseCreate(crate::ResponseCreateWsRequest {
        model: "gpt-5.6-sol",
        instructions: "",
        previous_response_id: None,
        input: &[],
        tools: None,
        tool_choice: "auto",
        parallel_tool_calls: true,
        reasoning: None,
        store: false,
        stream: true,
        stream_options: None,
        include: &[],
        service_tier: Some("default"),
        prompt_cache_key: None,
        text: None,
        generate: None,
        client_metadata: None,
        thinking_budget: None,
        emit_usage: None,
        enable_thinking: None,
        reasoning_effort: None,
    })
}
type Server = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;
async fn pair() -> anyhow::Result<(ResponsesWebsocketConnection, Server)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = Url::parse(&format!("ws://{}/v1/responses", listener.local_addr()?))?;
    let accept = async {
        let (socket, _) = listener.accept().await?;
        Ok::<_, anyhow::Error>(
            tokio_tungstenite::accept_async_with_config(socket, Some(websocket_config())).await?,
        )
    };
    let factory =
        HttpClientFactory::new(codex_http_client::OutboundProxyPolicy::RespectSystemProxy);
    let connect = connect_websocket(url, HeaderMap::new(), &factory, None);
    let (connected, server) = tokio::join!(connect, accept);
    let (stream, _, _, _, _) = connected?;
    Ok((
        ResponsesWebsocketConnection::new(stream, Duration::from_secs(5), false, None, None, None),
        server?,
    ))
}
async fn start(conn: &ResponsesWebsocketConnection, admit: Option<Arc<Admit>>) -> ResponseStream {
    conn.stream_request_with_accounting(
        request(),
        false,
        None,
        admit.map(|v| v as Arc<dyn accounting::ResponsesWebsocketAdmission>),
    )
    .await
    .unwrap()
}
async fn send(server: &mut Server, event: Value) {
    server
        .send(Message::Text(event.to_string().into()))
        .await
        .unwrap();
}
async fn completed(stream: &mut ResponseStream) -> Result<(), ApiError> {
    while let Some(event) = stream.next().await {
        if matches!(event?, ResponseEvent::Completed { .. }) {
            return Ok(());
        }
    }
    Err(ApiError::Stream("fixture ended".into()))
}
async fn no_frame(server: &mut Server) {
    assert!(
        tokio::time::timeout(Duration::from_millis(40), server.next())
            .await
            .is_err()
    );
}
fn usage(n: Value) -> Value {
    json!({"type":"response.usage","usage":{"input_tokens":n}})
}
fn done() -> Value {
    json!({"type":"response.completed","response":{"id":"same"}})
}

#[tokio::test]
async fn responses_websocket_accounting_none_preserves_legacy() -> anyhow::Result<()> {
    let (conn, mut server) = pair().await?;
    let mut stream = start(&conn, None).await;
    let frame: Value = serde_json::from_str(server.next().await.unwrap()?.to_text()?)?;
    assert_eq!(
        (frame["type"].as_str(), frame["model"].as_str()),
        (Some("response.create"), Some("gpt-5.6-sol"))
    );
    send(&mut server, usage(json!(-1))).await;
    send(
        &mut server,
        json!({"type":"response.created","response":{"id":"same","model":"fixture-model"}}),
    )
    .await;
    send(&mut server, done()).await;
    completed(&mut stream).await?;
    assert!(!conn.is_closed().await);
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_pump_admission_barrier() -> anyhow::Result<()> {
    for rejected in [false, true] {
        let (conn, mut server) = pair().await?;
        let admit = Admit::new(0);
        admit.reject.store(rejected, Ordering::SeqCst);
        let mut stream = start(&conn, Some(admit.clone())).await;
        no_frame(&mut server).await;
        assert_eq!(admit.calls.load(Ordering::SeqCst), 1);
        admit.gate.add_permits(1);
        if rejected {
            assert!(completed(&mut stream).await.is_err());
            assert!(conn.is_closed().await);
            assert!(!matches!(server.next().await, Some(Ok(Message::Text(_)))));
        } else {
            assert!(matches!(server.next().await, Some(Ok(Message::Text(_)))));
            send(&mut server, done()).await;
            completed(&mut stream).await?;
        }
    }
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_queued_cancel_and_send_failure() -> anyhow::Result<()> {
    let (conn, mut server) = pair().await?;
    let admit = Admit::new(8);
    let mut first = start(&conn, Some(admit.clone())).await;
    server.next().await.unwrap()?;
    let queued = start(&conn, Some(admit.clone())).await;
    drop(queued);
    send(&mut server, done()).await;
    completed(&mut first).await?;
    no_frame(&mut server).await;
    assert_eq!(admit.calls.load(Ordering::SeqCst), 1);
    let mut later = start(&conn, Some(admit.clone())).await;
    server.next().await.unwrap()?;
    drop(server);
    assert!(completed(&mut later).await.is_err());
    assert_eq!(admit.calls.load(Ordering::SeqCst), 2);
    assert!(admit.sink.records.lock().unwrap().is_empty());
    assert!(conn.is_closed().await);
    for cancel in [false, true] {
        let (conn, mut server) = pair().await?;
        let admit = Admit::new(0);
        let stream = start(&conn, Some(admit.clone())).await;
        no_frame(&mut server).await;
        assert_eq!(admit.calls.load(Ordering::SeqCst), 1);
        if cancel {
            drop(stream);
            admit.gate.add_permits(1);
            assert!(!matches!(server.next().await, Some(Ok(Message::Text(_)))));
        } else {
            drop(server);
            admit.gate.add_permits(1);
            let mut stream = stream;
            assert!(completed(&mut stream).await.is_err());
        }
        assert!(admit.sink.records.lock().unwrap().is_empty());
        assert!(conn.is_closed().await);
    }
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_positions_and_replacement() -> anyhow::Result<()> {
    let sink = Arc::new(Sink::default());
    let mut evidence = accounting::Evidence::new(sink.clone());
    for value in [
        usage(json!(3)),
        json!({"type":"response.created"}),
        usage(json!(7)),
        usage(json!(7)),
        usage(Value::Null),
        json!({"type":"response.usage","usage":{}}),
    ] {
        evidence.text(&value.to_string()).await?;
    }
    let records = sink.records.lock().unwrap();
    assert_eq!(
        records.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        vec![1, 3, 4, 5, 6]
    );
    assert_eq!(records[1].1, records[2].1);
    assert_eq!(
        records[3].1.as_ref().unwrap().input_tokens,
        ResponsesTokenPresence::Null
    );
    assert_eq!(
        records[4].1.as_ref().unwrap().input_tokens,
        ResponsesTokenPresence::Missing
    );
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_usage_and_terminal_containers() -> anyhow::Result<()> {
    for kind in [
        "response.completed",
        "response.failed",
        "response.incomplete",
        "response.usage",
    ] {
        for nested in [true, false] {
            if !nested && kind != "response.usage" {
                continue;
            }
            let (conn, mut server) = pair().await?;
            let admit = Admit::new(1);
            let mut stream = start(&conn, Some(admit.clone())).await;
            server.next().await.unwrap()?;
            let mut event = json!({"type":kind,"response":{"id":"same","error":{"code":"fixture","message":"fixture"},"incomplete_details":{"reason":"max_output_tokens"}}});
            if nested {
                event["response"]["usage"] =
                    json!({"input_tokens":7,"output_tokens":0,"total_tokens":7});
            } else {
                event["usage"] = json!({"input_tokens":7});
            }
            send(&mut server, event).await;
            if kind == "response.usage" {
                send(&mut server, done()).await;
            }
            let result = completed(&mut stream).await;
            assert_eq!(
                result.is_ok(),
                matches!(kind, "response.completed" | "response.usage")
            );
            assert_eq!(admit.sink.records.lock().unwrap().len(), 1);
        }
    }
    let sink = Arc::new(Sink::default());
    let mut evidence = accounting::Evidence::new(sink.clone());
    assert!(evidence.text(&json!({"type":"response.usage","usage":{"input_tokens":1},"response":{"usage":{"input_tokens":2}}}).to_string()).await.is_err());
    assert_eq!(
        sink.records.lock().unwrap()[0].1,
        Err(InvalidResponsesUsage)
    );
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_invalid_evidence_stops() -> anyhow::Result<()> {
    for value in [
        json!(-1),
        json!(1.5),
        json!("1"),
        json!(true),
        json!(9223372036854775808u64),
    ] {
        for field in [
            "input_tokens",
            "output_tokens",
            "total_tokens",
            "cached_tokens",
            "cache_write_tokens",
            "reasoning_tokens",
        ] {
            let (conn, mut server) = pair().await?;
            let admit = Admit::new(1);
            let mut stream = start(&conn, Some(admit.clone())).await;
            server.next().await.unwrap()?;
            let mut patch = json!({});
            match field {
                "cached_tokens" | "cache_write_tokens" => {
                    patch["input_tokens_details"] = json!({field:value})
                }
                "reasoning_tokens" => patch["output_tokens_details"] = json!({field:value}),
                _ => patch[field] = value.clone(),
            }
            send(&mut server, json!({"type":"response.usage","usage":patch})).await;
            assert!(completed(&mut stream).await.is_err());
            assert_eq!(
                admit.sink.records.lock().unwrap()[0].1,
                Err(InvalidResponsesUsage)
            );
            assert!(conn.is_closed().await);
        }
    }
    for text in [
        "{",
        r#"{"type":"response.usage","usage":[]}"#,
        r#"{"type":"response.usage","usage":{"input_tokens_details":[]}}"#,
    ] {
        let sink = Arc::new(Sink::default());
        assert!(accounting::Evidence::new(sink).text(text).await.is_err());
    }
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_observation_barrier() -> anyhow::Result<()> {
    for rejected in [false, true] {
        let (conn, mut server) = pair().await?;
        let admit = Arc::new(Admit {
            sink: Arc::new(Sink {
                gate: Some(Semaphore::new(0)),
                ..Default::default()
            }),
            gate: Semaphore::new(1),
            calls: AtomicUsize::new(0),
            reject: AtomicBool::new(false),
        });
        let mut stream = start(&conn, Some(admit.clone())).await;
        server.next().await.unwrap()?;
        send(&mut server, json!({"type":"response.completed","response":{"id":"same","usage":{"input_tokens":7,"output_tokens":0,"total_tokens":7}}})).await;
        let mut completion = completed(&mut stream).boxed();
        assert!(
            tokio::time::timeout(Duration::from_millis(40), &mut completion)
                .await
                .is_err()
        );
        admit.sink.reject.store(rejected, Ordering::SeqCst);
        admit.sink.gate.as_ref().unwrap().add_permits(1);
        assert_eq!(completion.await.is_err(), rejected);
    }
    Ok(())
}
#[tokio::test]
async fn responses_websocket_accounting_response_local_binding() -> anyhow::Result<()> {
    let old = Arc::new(Sink::default());
    let new = Arc::new(Sink::default());
    let mut a = accounting::Evidence::new(old.clone());
    let mut b = accounting::Evidence::new(new.clone());
    b.text(&usage(json!(7)).to_string()).await?;
    a.text(&usage(json!(3)).to_string()).await?;
    assert_eq!(old.records.lock().unwrap()[0].0, 1);
    assert_eq!(new.records.lock().unwrap()[0].0, 1);
    assert_ne!(*old.records.lock().unwrap(), *new.records.lock().unwrap());
    let first = Admit::new(1);
    let second = Admit::new(1);
    for (admit, tokens) in [(&first, 11), (&second, 23)] {
        let (conn, mut server) = pair().await?;
        let mut stream = start(&conn, Some(admit.clone())).await;
        server.next().await.unwrap()?;
        send(
            &mut server,
            json!({"type":"response.created","response":{"id":"same"}}),
        )
        .await;
        send(&mut server, usage(json!(tokens))).await;
        send(&mut server, done()).await;
        completed(&mut stream).await?;
        assert_eq!(admit.sink.records.lock().unwrap()[0].0, 2);
    }
    assert_ne!(
        *first.sink.records.lock().unwrap(),
        *second.sink.records.lock().unwrap()
    );
    Ok(())
}
