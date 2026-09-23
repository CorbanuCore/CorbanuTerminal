// Each accounting suite module carries its own copy of the shared fixtures.
#[allow(clippy::duplicate_mod)]
#[path = "accounting_anthropic_support.rs"]
#[allow(dead_code)]
mod existing;
use codex_core::config::AccountingMode;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_state::accounting::*;
use core_test_support::responses;
use core_test_support::test_codex::{TestCodexBuilder, test_codex};
pub use existing::{
    attempts, connection, observations, payloads, stop, submit, terminal, wait_observations,
};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use uuid::Uuid;

pub fn enabled(endpoint: &str) -> AccountingMode {
    AccountingMode::DirectOpenAiResponsesHttp {
        scope: Uuid::new_v4(),
        approved_endpoint: endpoint.into(),
    }
}
pub fn builder(endpoint: String, mode: AccountingMode) -> TestCodexBuilder {
    test_codex()
        .with_auth(CodexAuth::from_api_key("synthetic-responses-accounting"))
        .with_config(move |config| {
            config.model = Some("gpt-5.6-sol".into());
            config.model_provider_id = "openai".into();
            config.model_provider = ModelProviderInfo {
                request_max_retries: Some(1),
                stream_max_retries: Some(1),
                stream_idle_timeout_ms: Some(2000),
                ..ModelProviderInfo::create_openai_provider(Some(endpoint))
            };
            config.accounting = mode;
            config.model_catalog = Some(
                codex_models_manager::bundled_models_response()
                    .expect("synthetic Responses fixture"),
            );
            config
                .features
                .enable(Feature::Sqlite)
                .expect("synthetic Responses fixture");
        })
}
pub fn usage(write: Option<i64>) -> Value {
    let mut value = json!({"input_tokens":100,"input_tokens_details":{"cached_tokens":20},
        "output_tokens":40,"output_tokens_details":{"reasoning_tokens":10},"total_tokens":140});
    if let Some(write) = write {
        value["input_tokens_details"]["cache_write_tokens"] = json!(write);
    }
    value
}
pub fn event(kind: &str, usage: Value) -> Value {
    json!({"type":kind,"response":{"id":"reused-provider-id","usage":usage}})
}
pub fn success(usage: Value) -> String {
    responses::sse(vec![
        responses::ev_response_created("reused-provider-id"),
        responses::ev_assistant_message("message", "fixture complete"),
        event("response.completed", usage),
    ])
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
    Ok(totals)
}

pub struct Held {
    pub body: Value,
    pub chunks: mpsc::Sender<String>,
}
pub struct Gate {
    pub endpoint: String,
    incoming: mpsc::Receiver<Held>,
    task: tokio::task::JoinHandle<()>,
}
impl Gate {
    pub async fn start() -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}/v1", listener.local_addr()?);
        let (tx, incoming) = mpsc::channel(8);
        let task = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut bytes = Vec::new();
                    let end = loop {
                        let mut buf = [0; 4096];
                        let n = socket
                            .read(&mut buf)
                            .await
                            .expect("synthetic Responses fixture");
                        if n == 0 {
                            return;
                        }
                        bytes.extend_from_slice(&buf[..n]);
                        if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
                            break end + 4;
                        }
                    };
                    let headers = String::from_utf8(bytes[..end].to_vec())
                        .expect("synthetic Responses fixture");
                    assert!(headers.starts_with("POST /v1/responses "));
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().expect("synthetic Responses fixture"))
                        })
                        .expect("synthetic Responses fixture");
                    while bytes.len() < end + length {
                        let mut buf = [0; 4096];
                        let n = socket
                            .read(&mut buf)
                            .await
                            .expect("synthetic Responses fixture");
                        if n == 0 {
                            return;
                        }
                        bytes.extend_from_slice(&buf[..n]);
                    }
                    let body = serde_json::from_slice(&bytes[end..end + length])
                        .expect("synthetic Responses fixture");
                    let (chunks, mut rx) = mpsc::channel::<String>(8);
                    if tx.send(Held { body, chunks }).await.is_err() {
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
                .expect("synthetic Responses fixture"),
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
