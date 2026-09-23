#![allow(clippy::unwrap_used)]
#[path = "accounting_anthropic_support.rs"]
#[allow(dead_code)]
mod existing;
use codex_core::config::AccountingMode;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_state::accounting::*;
use core_test_support::test_codex::{TestCodexBuilder, test_codex};
pub use existing::{connection, payloads, stop, submit, terminal, wait_observations};
pub async fn attempts(db: &codex_state::StateRuntime) -> anyhow::Result<Vec<Attempt>> {
    let rows = existing::attempts(db).await?;
    eprintln!("CHAT_ATTEMPTS {}", serde_json::to_string(&rows)?);
    Ok(rows)
}
pub async fn observations(db: &codex_state::StateRuntime) -> anyhow::Result<Vec<Observation>> {
    let rows = existing::observations(db).await?;
    eprintln!("CHAT_OBSERVATIONS {}", serde_json::to_string(&rows)?);
    Ok(rows)
}
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use uuid::Uuid;

pub fn enabled(endpoint: &str) -> AccountingMode {
    AccountingMode::DirectOpenAiChat {
        scope: Uuid::new_v4(),
        approved_endpoint: endpoint.into(),
    }
}
pub fn builder(endpoint: String, mode: AccountingMode) -> TestCodexBuilder {
    test_codex()
        .with_auth(CodexAuth::from_api_key("synthetic-chat-accounting"))
        .with_config(move |config| {
            config.model = Some("gpt-5.6-sol".into());
            config.model_provider_id = "openai".into();
            config.model_provider = ModelProviderInfo {
                request_max_retries: Some(0),
                stream_max_retries: Some(0),
                stream_idle_timeout_ms: Some(2000),
                wire_api: codex_model_provider_info::WireApi::Chat,
                supports_websockets: false,
                ..ModelProviderInfo::create_openai_provider(Some(endpoint))
            };
            config.accounting = mode;
            config.model_catalog = Some(
                codex_models_manager::bundled_models_response().expect("synthetic Chat fixture"),
            );
            config
                .features
                .enable(Feature::Sqlite)
                .expect("synthetic Chat fixture");
        })
}
pub fn usage() -> Value {
    json!({"prompt_tokens":100,"prompt_tokens_details":{"cached_tokens":20},
        "completion_tokens":40,"completion_tokens_details":{"reasoning_tokens":10},"total_tokens":140})
}
pub fn data(value: Value) -> String {
    format!("data: {value}\n\n")
}
pub fn event(usage: Value) -> String {
    data(json!({"id":"reused-provider-id","choices":[],"usage":usage}))
}
pub fn ending(reason: &str) -> String {
    data(json!({"id":"reused-provider-id","choices":[{"index":0,
        "delta":{"role":"assistant","content":"fixture complete"},"finish_reason":reason}]}))
        + "data: [DONE]\n\n"
}
pub fn success(usage: Value) -> String {
    event(usage) + &ending("stop")
}
pub async fn mount(server: &wiremock::MockServer, payload: String) {
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/chat/completions"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_raw(payload, "text/event-stream"),
        )
        .mount(server)
        .await;
}
pub async fn posts(server: &wiremock::MockServer, expected: usize) {
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), expected);
    let count = requests.len();
    for request in requests {
        assert_eq!(request.method.as_str(), "POST");
        assert_eq!(request.url.path(), "/v1/chat/completions");
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        eprintln!(
            "CHAT_POSTS {count} {} {} model={} include_usage={}",
            request.method,
            request.url.path(),
            body["model"].as_str().unwrap(),
            body["stream_options"]["include_usage"],
        );
        assert_eq!(body["stream_options"]["include_usage"], true);
        assert_eq!(body["model"], "gpt-5.6-sol");
        assert!(request.headers.get("x-pfterminal-request-id").is_none());
    }
}
pub fn golden(cached: bool, zero: bool, count: i64) -> anyhow::Result<DayTotals> {
    let values = if zero {
        [0; 7]
    } else {
        [100, 0, if cached { 20 } else { 0 }, 0, 40, 10, 140]
    };
    Ok(DayTotals {
        measured: std::array::from_fn(|i| Metric {
            known: values[i] * count,
            unknown: if i == 1 || i == 3 || (i == 2 && !cached) {
                count
            } else {
                0
            },
        }),
        known_usd: match (zero, cached, count) {
            (true, _, _) => "0",
            (false, true, 1) => "0.00121",
            (false, true, 2) => "0.00242",
            (false, false, 1) => "0.0012",
            _ => unreachable!(),
        }
        .to_string()
        .try_into()?,
        unknown_estimates: count,
        attempts: count,
        ..Default::default()
    })
}
pub async fn absent(db: &codex_state::StateRuntime) -> anyhow::Result<()> {
    let mut conn = connection(db).await?;
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_master WHERE name LIKE 'draft_accounting_%'",
    )
    .fetch_one(&mut conn)
    .await?;
    assert_eq!(count, 0);
    Ok(())
}
pub async fn totals(
    db: &codex_state::StateRuntime,
    attempt: &Attempt,
) -> anyhow::Result<DayTotals> {
    let now = chrono::Utc::now().timestamp_millis();
    let store = AccountingStore::open(db, now).await?;
    let day = store
        .read_day(
            attempt.thread_id,
            i64::from(attempt.dispatched_at_ms) / 86_400_000,
            now,
        )
        .await?;
    let RetainedDay::Available {
        totals: Current::Ready(totals),
        ..
    } = day
    else {
        anyhow::bail!("unavailable fixture day")
    };
    eprintln!("CHAT_TOTALS {totals:?}");
    Ok(totals)
}

pub struct Held {
    pub body: Value,
    pub headers: String,
    pub chunks: mpsc::Sender<String>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GateRoutes {
    ChatOnly,
    ChatAndCompact,
}
pub struct Gate {
    pub endpoint: String,
    incoming: mpsc::Receiver<Held>,
    task: tokio::task::JoinHandle<()>,
}
impl Gate {
    pub async fn start(routes: GateRoutes) -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}/v1", listener.local_addr()?);
        let (tx, incoming) = mpsc::channel(8);
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            while let Ok((mut socket, _)) = listener.accept().await {
                let tx = tx.clone();
                connections.spawn(async move {
                    let mut bytes = Vec::new();
                    let end = loop {
                        let mut buf = [0; 4096];
                        let n = socket
                            .read(&mut buf)
                            .await
                            .expect("synthetic Chat fixture");
                        if n == 0 {
                            return;
                        }
                        bytes.extend_from_slice(&buf[..n]);
                        if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
                            break end + 4;
                        }
                    };
                    let headers = String::from_utf8(bytes[..end].to_vec())
                        .expect("synthetic Chat fixture");
                    let compact = headers.starts_with("POST /v1/responses/compact ");
                    assert!(
                        headers.starts_with("POST /v1/chat/completions ")
                            || (compact && routes == GateRoutes::ChatAndCompact)
                    );
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().expect("synthetic Chat fixture"))
                        })
                        .expect("synthetic Chat fixture");
                    while bytes.len() < end + length {
                        let mut buf = [0; 4096];
                        let n = socket
                            .read(&mut buf)
                            .await
                            .expect("synthetic Chat fixture");
                        if n == 0 {
                            return;
                        }
                        bytes.extend_from_slice(&buf[..n]);
                    }
                    eprintln!("CHAT_HELD_POST compact={compact}");
                    let body = serde_json::from_slice(&bytes[end..end + length])
                        .expect("synthetic Chat fixture");
                    let (chunks, mut rx) = mpsc::channel::<String>(8);
                    if tx.send(Held { body, headers, chunks }).await.is_err() {
                        return;
                    }
                    if compact {
                        let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 13\r\nConnection: close\r\n\r\n{\"output\":[]}").await;
                        return;
                    }
                    if socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n").await.is_err() { return; }
                    while let Some(chunk) = rx.recv().await {
                        if socket.write_all(chunk.as_bytes()).await.is_err() {
                            return;
                        }
                        let _ = socket.flush().await;
                    }
                });
            }
        });
        Ok(Self {
            endpoint,
            incoming,
            task,
        })
    }
    pub async fn next(&mut self) -> anyhow::Result<Held> {
        Ok(
            tokio::time::timeout(Duration::from_secs(10), self.incoming.recv())
                .await?
                .expect("synthetic Chat fixture"),
        )
    }
    pub fn no_pending(&mut self) {
        assert!(self.incoming.try_recv().is_err());
    }
}
impl Drop for Gate {
    fn drop(&mut self) {
        self.task.abort();
    }
}
