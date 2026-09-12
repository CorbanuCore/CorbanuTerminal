use anyhow::Result;
use codex_core::config::AccountingMode;
use codex_features::Feature;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use codex_state::StateRuntime;
use codex_state::accounting::Attempt;
use codex_state::accounting::Observation;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::TestCodexBuilder;
use core_test_support::test_codex::test_codex;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::json;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

pub fn builder(endpoint: String, mode: AccountingMode) -> TestCodexBuilder {
    test_codex().with_config(move |config| {
        config.model = Some("claude-opus-5".into());
        config.model_provider_id = "anthropic".into();
        config.model_provider = ModelProviderInfo {
            base_url: Some(endpoint),
            env_key: None,
            experimental_bearer_token: Some("synthetic-local-accounting-key".into()),
            request_max_retries: Some(1),
            stream_max_retries: Some(1),
            stream_idle_timeout_ms: Some(2000),
            ..ModelProviderInfo::create_anthropic_provider()
        };
        config.accounting = mode;
        config.model_catalog = Some(codex_models_manager::bundled_models_response().unwrap());
        config.features.enable(Feature::Sqlite).unwrap();
    })
}

pub fn enabled(endpoint: &str) -> AccountingMode {
    AccountingMode::DirectAnthropic {
        scope: Uuid::new_v4(),
        approved_endpoint: endpoint.into(),
    }
}

pub fn sse(events: &[Value]) -> String {
    events
        .iter()
        .map(|value| {
            format!(
                "event: {}\ndata: {value}\n\n",
                value["type"].as_str().unwrap()
            )
        })
        .collect()
}

pub fn start(usage: Value) -> Value {
    json!({"type":"message_start","message":{"id":"reused-provider-id","model":"claude-opus-5","usage":usage}})
}

pub fn ending(usage: Value) -> Vec<Value> {
    vec![
        json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"fixture complete"}}),
        json!({"type":"content_block_stop","index":0}),
        json!({"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":usage}),
        json!({"type":"message_stop"}),
    ]
}

pub fn success(input: Value, output: Value) -> wiremock::ResponseTemplate {
    let mut events = vec![start(input)];
    events.extend(ending(output));
    wiremock::ResponseTemplate::new(200).set_body_raw(sse(&events), "text/event-stream")
}

pub async fn connection(db: &StateRuntime) -> Result<SqliteConnection> {
    Ok(SqliteConnection::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db.sqlite().state_db_path())
            .foreign_keys(true),
    )
    .await?)
}

pub async fn payloads<T: DeserializeOwned>(db: &StateRuntime, table: &str) -> Result<Vec<T>> {
    assert!(
        [
            "draft_accounting_attempts",
            "draft_accounting_observations",
            "draft_accounting_price_snapshots"
        ]
        .contains(&table)
    );
    let query = match table {
        "draft_accounting_attempts" => {
            "SELECT payload FROM draft_accounting_attempts ORDER BY rowid"
        }
        "draft_accounting_observations" => {
            "SELECT payload FROM draft_accounting_observations ORDER BY rowid"
        }
        "draft_accounting_price_snapshots" => {
            "SELECT payload FROM draft_accounting_price_snapshots ORDER BY rowid"
        }
        _ => unreachable!("whitelisted fixture table"),
    };
    let values: Vec<String> = sqlx::query_scalar(query)
        .fetch_all(&mut connection(db).await?)
        .await?;
    values
        .into_iter()
        .map(|value| serde_json::from_str(&value).map_err(Into::into))
        .collect()
}

pub async fn attempts(db: &StateRuntime) -> Result<Vec<Attempt>> {
    payloads(db, "draft_accounting_attempts").await
}

pub async fn observations(db: &StateRuntime) -> Result<Vec<Observation>> {
    payloads(db, "draft_accounting_observations").await
}

pub async fn submit(test: &TestCodex) -> Result<()> {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "accounting native fixture".into(),
                text_elements: vec![],
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    Ok(())
}

pub async fn terminal(test: &TestCodex) -> Result<Vec<EventMsg>> {
    tokio::time::timeout(Duration::from_secs(20), async {
        let mut events = Vec::new();
        loop {
            let event = test.codex.next_event().await?.msg;
            let finished = matches!(event, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_));
            events.push(event);
            if finished {
                return Ok(events);
            }
        }
    })
    .await?
}

pub async fn stop(test: &TestCodex) {
    test.thread_manager
        .shutdown_all_threads_bounded(Duration::from_secs(3))
        .await;
}

pub async fn wait_observations(db: &StateRuntime, count: usize) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if observations(db).await?.len() >= count {
                return anyhow::Ok(());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await?
}

/// An actual local Messages connection; the test controls each byte segment.
pub struct GateRequest {
    pub body: Value,
    pub head: String,
    pub chunks: mpsc::Sender<String>,
}

pub struct GateServer {
    pub endpoint: String,
    pub incoming: mpsc::Receiver<GateRequest>,
    task: tokio::task::JoinHandle<()>,
}

impl GateServer {
    pub async fn start() -> Result<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}/v1", listener.local_addr()?);
        let (send, incoming) = mpsc::channel(8);
        let task = tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let send = send.clone();
                tokio::spawn(async move {
                    let mut data = Vec::new();
                    let header_end = loop {
                        let mut buffer = [0; 4096];
                        let size = stream.read(&mut buffer).await.unwrap();
                        if size == 0 {
                            return;
                        }
                        data.extend_from_slice(&buffer[..size]);
                        if let Some(end) = data.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                            break end + 4;
                        }
                    };
                    let head = String::from_utf8(data[..header_end].to_vec()).unwrap();
                    let length: usize = head
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap_or(0);
                    while data.len() < header_end + length {
                        let mut buffer = [0; 4096];
                        let size = stream.read(&mut buffer).await.unwrap();
                        if size == 0 {
                            return;
                        }
                        data.extend_from_slice(&buffer[..size]);
                    }
                    if !head.starts_with("POST /v1/messages ") {
                        let _ = stream
                            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                            .await;
                        return;
                    }
                    let body =
                        serde_json::from_slice(&data[header_end..header_end + length]).unwrap();
                    let unavailable = head
                        .lines()
                        .any(|line| line.eq_ignore_ascii_case("x-accounting-fixture-status: 503"));
                    let (chunks, mut receive) = mpsc::channel::<String>(8);
                    if send.send(GateRequest { body, head, chunks }).await.is_err() {
                        return;
                    }
                    if unavailable {
                        let _ = stream
                            .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                            .await;
                        return;
                    }
                    if stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n").await.is_err() { return; }
                    while let Some(chunk) = receive.recv().await {
                        if stream.write_all(chunk.as_bytes()).await.is_err() {
                            break;
                        }
                        let _ = stream.flush().await;
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

    pub async fn next(&mut self) -> Result<GateRequest> {
        Ok(
            tokio::time::timeout(Duration::from_secs(15), self.incoming.recv())
                .await?
                .expect("Messages request"),
        )
    }
}

impl Drop for GateServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
