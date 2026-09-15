pub use super::accounting_responses::support::absent;
pub use super::accounting_responses::support::connection;
pub use super::accounting_responses::support::event;
pub use super::accounting_responses::support::payloads;
pub use super::accounting_responses::support::stop;
pub use super::accounting_responses::support::submit;
pub use super::accounting_responses::support::terminal;
pub use super::accounting_responses::support::totals;
pub use super::accounting_responses::support::usage;
pub use super::accounting_responses::support::wait_observations;
use codex_core::config::AccountingMode;
use core_test_support::test_codex::TestCodexBuilder;
use futures::SinkExt;
use futures::StreamExt;
use serde_json::Value;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn attempts(
    db: &codex_state::StateRuntime,
) -> anyhow::Result<Vec<codex_state::accounting::Attempt>> {
    let rows = super::accounting_responses::support::attempts(db).await?;
    eprintln!(
        "WS separate-connection attempts: {}",
        serde_json::to_string(&rows)?
    );
    Ok(rows)
}
pub async fn observations(
    db: &codex_state::StateRuntime,
) -> anyhow::Result<Vec<codex_state::accounting::Observation>> {
    let rows = super::accounting_responses::support::observations(db).await?;
    eprintln!(
        "WS separate-connection observations: {}",
        serde_json::to_string(&rows)?
    );
    Ok(rows)
}
pub fn enabled(endpoint: &str) -> AccountingMode {
    AccountingMode::DirectOpenAiResponses {
        scope: Uuid::new_v4(),
        approved_endpoint: endpoint.into(),
    }
}
pub fn builder(endpoint: String, mode: AccountingMode) -> TestCodexBuilder {
    super::accounting_responses::support::builder(endpoint, mode).with_config(|config| {
        config.model_provider.supports_websockets = true;
        config.model_provider.stream_max_retries = Some(0);
    })
}
pub fn success(write: Option<i64>) -> Vec<Value> {
    vec![
        core_test_support::responses::ev_response_created("same"),
        core_test_support::responses::ev_assistant_message("m", "fixture"),
        event("response.completed", usage(write)),
    ]
}
pub fn counts(gate: &Gate, expected: (usize, usize, usize, usize)) {
    let actual = *gate.counts.lock().unwrap();
    eprintln!("WS fixture (handshakes, warmups, sampling frames, HTTP POSTs): {actual:?}");
    assert_eq!(actual, expected);
}
pub struct Held {
    pub body: Value,
    pub http: bool,
    pub chunks: mpsc::Sender<Vec<Value>>,
}
impl Held {
    pub async fn send(&self, events: Vec<Value>) -> anyhow::Result<()> {
        self.chunks.send(events).await?;
        Ok(())
    }
    pub async fn complete(&self) -> anyhow::Result<()> {
        self.send(success(Some(0))).await
    }
}
pub struct Gate {
    pub endpoint: String,
    pub counts: Arc<Mutex<(usize, usize, usize, usize)>>,
    pub hold_warmup: Arc<std::sync::atomic::AtomicBool>,
    incoming: mpsc::Receiver<Held>,
    task: tokio::task::JoinHandle<()>,
}
impl Gate {
    pub async fn start() -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}/v1", listener.local_addr()?);
        let counts = Arc::new(Mutex::new((0, 0, 0, 0)));
        let recorded = counts.clone();
        let hold_warmup = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let warmup_control = hold_warmup.clone();
        let (tx, incoming) = mpsc::channel(32);
        let task = tokio::spawn(async move {
            let mut children = tokio::task::JoinSet::new();
            while let Ok((mut socket, _)) = listener.accept().await {
                let tx = tx.clone();
                let counts = recorded.clone();
                let warmup_control = warmup_control.clone();
                children.spawn(async move {
                    let mut prefix = [0; 4];
                    if socket.peek(&mut prefix).await.ok().is_none() { return; }
                    if prefix.starts_with(b"GET") {
                        counts.lock().unwrap().0 += 1;
                        let mut config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default();
                        config.extensions.permessage_deflate = Some(Default::default());
                        let Ok(mut ws) = tokio_tungstenite::accept_async_with_config(socket, Some(config)).await else { return; };
                        while let Some(Ok(message)) = ws.next().await {
                            let tokio_tungstenite::tungstenite::Message::Text(text) = message else { continue; };
                            let body: Value = serde_json::from_str(&text).unwrap();
                            if body["generate"] == false {
                                counts.lock().unwrap().1 += 1;
                                if warmup_control.load(std::sync::atomic::Ordering::SeqCst) {
                                    let (chunks, mut rx) = mpsc::channel(8);
                                    if tx.send(Held { body, http: false, chunks }).await.is_err() { return; }
                                    let Some(events) = rx.recv().await else { return; };
                                    for event in events {
                                        if ws.send(tokio_tungstenite::tungstenite::Message::Text(event.to_string().into())).await.is_err() { return; }
                                    }
                                    continue;
                                }
                                let events = vec![core_test_support::responses::ev_response_created("warm"),
                                    event("response.completed", json!({"input_tokens":999,"output_tokens":999,"total_tokens":1998}))];
                                for event in events {
                                    if ws.send(tokio_tungstenite::tungstenite::Message::Text(event.to_string().into())).await.is_err() { return; }
                                }
                                continue;
                            }
                            counts.lock().unwrap().2 += 1;
                            let (chunks, mut rx) = mpsc::channel(8);
                            if tx.send(Held { body, http: false, chunks }).await.is_err() { return; }
                            let mut complete = false;
                            while let Some(events) = rx.recv().await {
                                for event in events {
                                    complete |= event["type"] == "response.completed";
                                    if ws.send(tokio_tungstenite::tungstenite::Message::Text(event.to_string().into())).await.is_err() { return; }
                                }
                                if complete { break; }
                            }
                            if !complete { return; }
                        }
                    } else {
                        let mut bytes = Vec::new();
                        let end = loop {
                            let mut buffer = [0; 4096];
                            let Ok(n) = socket.read(&mut buffer).await else { return; };
                            if n == 0 { return; }
                            bytes.extend_from_slice(&buffer[..n]);
                            if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") { break end + 4; }
                        };
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let compact = headers.starts_with("POST /v1/responses/compact ");
                        assert!(compact || headers.starts_with("POST /v1/responses "));
                        let length: usize = headers.lines().find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length").then(|| value.trim().parse().unwrap())
                        }).unwrap();
                        while bytes.len() < end + length {
                            let mut buffer = [0; 4096];
                            let Ok(n) = socket.read(&mut buffer).await else { return; };
                            if n == 0 { return; }
                            bytes.extend_from_slice(&buffer[..n]);
                        }
                        counts.lock().unwrap().3 += 1;
                        if compact {
                            let body = r#"{"output":[{"type":"compaction","encrypted_content":"fixture"}]}"#;
                            let reply = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                            let _ = socket.write_all(reply.as_bytes()).await;
                            return;
                        }
                        let body = serde_json::from_slice(&bytes[end..end + length]).unwrap();
                        let (chunks, mut rx) = mpsc::channel(8);
                        if tx.send(Held { body, http: true, chunks }).await.is_err() { return; }
                        let Some(events) = rx.recv().await else { return; };
                        if events.first().is_some_and(|event| event["fixture_status"] == 503) {
                            let _ = socket.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
                            return;
                        }
                        if socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n").await.is_err() { return; }
                        let _ = socket.write_all(core_test_support::responses::sse(events).as_bytes()).await;
                    }
                });
            }
        });
        Ok(Self {
            endpoint,
            counts,
            hold_warmup,
            incoming,
            task,
        })
    }
    pub async fn next(&mut self) -> anyhow::Result<Held> {
        Ok(
            tokio::time::timeout(Duration::from_secs(15), self.incoming.recv())
                .await?
                .expect("fixture dispatch"),
        )
    }
    pub async fn no_pending(&mut self) {
        assert!(
            tokio::time::timeout(Duration::from_millis(150), self.incoming.recv())
                .await
                .is_err()
        );
    }
}
impl Drop for Gate {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub struct StaticAuth(pub codex_login::CodexAuth);
impl codex_login::ExternalAuth for StaticAuth {
    fn resolve(&self) -> codex_login::ExternalAuthFuture<'_, codex_login::CodexAuth> {
        Box::pin(async { Ok(self.0.clone()) })
    }
    fn refresh(
        &self,
        _: codex_login::ExternalAuthRefreshContext,
    ) -> codex_login::ExternalAuthFuture<'_, codex_login::CodexAuth> {
        self.resolve()
    }
}
pub fn chain(records: &[codex_state::accounting::Attempt]) {
    assert!(!records.is_empty());
    assert_eq!(records[0].retry_of, None);
    for pair in records.windows(2) {
        assert_eq!(pair[1].request_id, pair[0].request_id);
        assert_eq!(pair[1].retry_of, Some(pair[0].attempt_id));
        assert_ne!(pair[1].attempt_id, pair[0].attempt_id);
    }
}
