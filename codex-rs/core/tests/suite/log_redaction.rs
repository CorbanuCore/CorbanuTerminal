//! Secrets never reach the TUI's log sinks: the trace-level log file, the
//! `/feedback` buffer and the logs database (#179, #196, #380).
#![cfg(not(target_os = "windows"))]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;

use anyhow::Context;
use codex_login::CodexAuth;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ExecCommandEndEvent;
use codex_protocol::protocol::Op;
use core_test_support::PathBufExt;
use core_test_support::responses::WebSocketConnectionConfig;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_once;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::sse_response;
use core_test_support::responses::start_mock_server;
use core_test_support::responses::start_websocket_server_with_headers;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;

/// A user's `RUST_LOG` that asks for everything, including the HTTP and
/// websocket libraries by name (#380).
const RUST_LOG_TRACE: &str = "trace,tungstenite=trace,tokio_tungstenite=trace,hyper=trace,hyper_util=trace,h2=trace,reqwest=trace,rustls=trace";

/// The TUI's log sinks, guarded and redacted as the TUI installs them: a
/// file filtered by [`RUST_LOG_TRACE`] with span events, the feedback buffer
/// (uploaded by /feedback) and the logs database. `log` records (tungstenite,
/// reqwest) are bridged as the binaries' `try_init` does.
///
/// The sinks are installed with `set_default`, so they see events from this
/// thread only. Use a current-thread runtime so every spawned task logs
/// through them; events from `spawn_blocking` or separate runtimes are not
/// captured.
struct TraceSinks {
    _logs: TempDir,
    log_path: PathBuf,
    state_home: TempDir,
    state_db: Arc<codex_state::StateRuntime>,
    log_db: codex_state::log_db::LogDbLayer,
    feedback: codex_feedback::CodexFeedback,
}

impl TraceSinks {
    async fn install() -> anyhow::Result<(Self, tracing::subscriber::DefaultGuard)> {
        let logs = TempDir::new()?;
        let log_path = logs.path().join("codex-tui.log");
        let log_file = std::fs::File::create(&log_path)?;
        let state_home = TempDir::new()?;
        let state_db = codex_state::StateRuntime::init(
            codex_state::SqliteConfig::new_for_testing(state_home.path().to_path_buf().abs()),
            "test-provider".to_string(),
        )
        .await?;
        let log_db = codex_state::log_db::start(Arc::clone(&state_db));
        let feedback = codex_feedback::CodexFeedback::new();
        static BRIDGE_LOG_RECORDS: Once = Once::new();
        BRIDGE_LOG_RECORDS.call_once(|| {
            // Another test's subscriber may have installed the bridge already.
            let _ = tracing_log::LogTracer::init();
        });
        let subscriber = codex_log_guard::guard(
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(codex_log_guard::RedactingMakeWriter::new(Mutex::new(
                            log_file,
                        )))
                        .with_target(true)
                        .with_ansi(false)
                        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                        .with_filter(EnvFilter::new(RUST_LOG_TRACE)),
                )
                .with(feedback.logger_layer())
                .with(
                    log_db
                        .clone()
                        .with_filter(codex_state::log_db::default_filter()),
                ),
        );
        let guard = tracing::subscriber::set_default(subscriber);
        Ok((
            Self {
                _logs: logs,
                log_path,
                state_home,
                state_db,
                log_db,
                feedback,
            },
            guard,
        ))
    }

    /// Flushes the sinks, checks that the log file and the logs database
    /// recorded every `recorded` marker, and that no sink holds any byte
    /// sequence of `secrets`. Returns the log file text.
    async fn assert_clean(
        self,
        guard: tracing::subscriber::DefaultGuard,
        recorded: &[&str],
        secrets: &[&str],
    ) -> anyhow::Result<String> {
        self.log_db.flush().await;
        drop(guard);
        assert!(
            secrets.iter().all(|secret| !secret.is_empty()),
            "empty secret"
        );
        let contains = |bytes: &[u8], needle: &str| {
            bytes
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
        };

        let log_text = std::fs::read_to_string(&self.log_path)?;
        let feedback_bytes = self
            .feedback
            .snapshot(/*session_id*/ None)
            .log_bytes()
            .to_vec();
        assert!(!feedback_bytes.is_empty(), "feedback buffer is empty");
        let rows = self
            .state_db
            .query_logs(&codex_state::LogQuery::default())
            .await?;
        for marker in recorded {
            assert!(
                log_text.contains(marker),
                "trace log should record {marker}"
            );
            assert!(
                rows.iter().any(|row| row
                    .message
                    .as_deref()
                    .is_some_and(|message| message.contains(marker))),
                "log DB should record {marker}"
            );
        }
        drop(self.state_db);
        let mut db_files = Vec::new();
        for entry in walkdir::WalkDir::new(self.state_home.path()) {
            let entry = entry?;
            if entry.file_type().is_file() {
                db_files.push((entry.path().to_path_buf(), std::fs::read(entry.path())?));
            }
        }
        for secret in secrets {
            assert!(
                !log_text.contains(secret),
                "secret leaked into the trace log file"
            );
            assert!(
                !contains(&feedback_bytes, secret),
                "secret leaked into the feedback log buffer"
            );
            for (path, bytes) in &db_files {
                assert!(
                    !contains(bytes, secret),
                    "secret leaked into {}",
                    path.display()
                );
            }
        }
        Ok(log_text)
    }
}

/// Regression for #179: a `!` command must never write environment values to
/// the trace-level log file or the logs database.
#[tokio::test]
async fn user_shell_cmd_env_values_never_reach_trace_logs() -> anyhow::Result<()> {
    const SENTINEL_NAME: &str = "CORBANU_SENTINEL_API_KEY";
    const SENTINEL_VALUE: &str = "fake-sentinel-179-do-not-log-9f3c2a7b";

    let (sinks, guard) = TraceSinks::install().await?;
    let cwd = TempDir::new()?;
    let cwd_path = cwd.path().to_path_buf();
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config.cwd = cwd_path.abs();
        config
            .permissions
            .shell_environment_policy
            .r#set
            .insert(SENTINEL_NAME.to_string(), SENTINEL_VALUE.to_string());
    });
    let codex = builder.build(&server).await?.codex;
    codex
        .submit(Op::RunUserShellCommand {
            command: format!("printenv {SENTINEL_NAME} | wc -c"),
        })
        .await?;
    let EventMsg::ExecCommandEnd(end) =
        wait_for_event(&codex, |ev| matches!(ev, EventMsg::ExecCommandEnd(_))).await
    else {
        unreachable!()
    };
    let ExecCommandEndEvent {
        stdout, exit_code, ..
    } = &end;
    // The child saw the sentinel (value plus newline).
    assert_eq!(*exit_code, 0, "{:?}", end.stderr);
    assert_eq!(stdout.trim(), (SENTINEL_VALUE.len() + 1).to_string());

    let log_text = sinks
        .assert_clean(guard, &["spawn_child_async"], &[SENTINEL_VALUE])
        .await?;
    let env_names_needle = format!("\"{SENTINEL_NAME}\"");
    assert!(
        log_text
            .lines()
            .any(|line| line.contains("spawn_child_async")
                && line.contains("env_names=")
                && line.contains(&env_names_needle)),
        "trace log should record the spawn with env names only"
    );
    Ok(())
}

/// #196: the provider's bearer token, static header and query values, sent on
/// every model request, a `set-cookie` response header, and the environment
/// of a model-requested shell command never reach the log sinks. The session-configured line and the
/// request URL are still logged, with the values redacted.
#[tokio::test]
async fn provider_credentials_and_tool_env_never_reach_trace_logs() -> anyhow::Result<()> {
    const PROVIDER_KEY: &str = "fake-provider-key-trace-0001-7d41c9e2";
    const HEADER_VALUE: &str = "fake-provider-header-0002-41aa90c3";
    const QUERY_VALUE: &str = "fake-provider-query-0003-0e5d77b1";
    const ENV_NAME: &str = "CORBANU_SENTINEL_TOOL_TOKEN";
    const ENV_VALUE: &str = "fake-tool-env-trace-0004-b83f05aa";
    const COOKIE_VALUE: &str = "fake-response-cookie-0005-c2e94d10";

    let (sinks, guard) = TraceSinks::install().await?;
    let cwd = TempDir::new()?;
    let cwd_path = cwd.path().to_path_buf();
    let server = start_mock_server().await;
    let mut builder = test_codex().with_config(move |config| {
        config.cwd = cwd_path.abs();
        let provider = &mut config.model_provider;
        provider.experimental_bearer_token = Some(PROVIDER_KEY.to_string());
        provider
            .http_headers
            .get_or_insert_default()
            .insert("X-Sentinel".to_string(), HEADER_VALUE.to_string());
        provider
            .query_params
            .get_or_insert_default()
            .insert("key".to_string(), QUERY_VALUE.to_string());
        config
            .permissions
            .shell_environment_policy
            .r#set
            .insert(ENV_NAME.to_string(), ENV_VALUE.to_string());
    });
    let fixture = builder.build(&server).await?;
    let call_id = "trace-secret-shell";
    let args = serde_json::json!({
        "command": format!("printenv {ENV_NAME} | wc -c"),
        "timeout_ms": 5_000,
    });
    let first = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_function_call(call_id, "shell_command", &serde_json::to_string(&args)?),
            ev_completed("resp-1"),
        ]),
    )
    .await;
    // Response headers are logged; credential-like values must not be.
    let second = mount_response_once(
        &server,
        sse_response(sse(vec![
            ev_assistant_message("msg-1", "done"),
            ev_completed("resp-2"),
        ]))
        .insert_header("set-cookie", format!("session={COOKIE_VALUE}; Path=/")),
    )
    .await;
    fixture
        .submit_turn_with_permission_profile("run it", PermissionProfile::Disabled)
        .await?;

    // Every credential was sent, and the command saw the variable.
    let request = first.single_request();
    assert_eq!(
        (
            request.header("authorization"),
            request.header("x-sentinel"),
            request.query_param("key"),
        ),
        (
            Some(format!("Bearer {PROVIDER_KEY}")),
            Some(HEADER_VALUE.to_string()),
            Some(QUERY_VALUE.to_string()),
        )
    );
    let output = second
        .single_request()
        .function_call_output_text(call_id)
        .context("function_call_output present for shell_command call")?;
    assert!(
        output.contains(&(ENV_VALUE.len() + 1).to_string()),
        "{output}"
    );

    let log_text = sinks
        .assert_clean(
            guard,
            &[
                "spawn_child_async",
                "experimental_bearer_token: Some(\"<redacted>\")",
            ],
            &[
                PROVIDER_KEY,
                HEADER_VALUE,
                QUERY_VALUE,
                ENV_VALUE,
                COOKIE_VALUE,
            ],
        )
        .await?;
    assert!(
        log_text.contains("\"X-Sentinel\": \"<redacted>\"")
            && log_text.contains("key=REDACTED")
            && log_text.contains("\"set-cookie\": \"REDACTED\""),
        "the provider headers, the request URL and the response headers should be logged with values redacted"
    );
    Ok(())
}

/// The Responses websocket logs its handshake response headers at INFO; a
/// `set-cookie` value must not reach any sink.
#[tokio::test]
async fn websocket_handshake_response_headers_never_reach_logs() -> anyhow::Result<()> {
    const COOKIE_VALUE: &str = "fake-websocket-cookie-0006-9a1b2c3d";

    let (sinks, guard) = TraceSinks::install().await?;
    let server = start_websocket_server_with_headers(vec![WebSocketConnectionConfig {
        requests: vec![
            // Startup prewarm, then the turn.
            vec![ev_response_created("warm-1"), ev_completed("warm-1")],
            vec![
                ev_response_created("resp-1"),
                ev_assistant_message("msg-1", "done"),
                ev_completed("resp-1"),
            ],
        ],
        response_headers: vec![(
            "set-cookie".to_string(),
            format!("session={COOKIE_VALUE}; Path=/"),
        )],
        accept_delay: None,
        close_after_requests: false,
    }])
    .await;
    let mut builder = test_codex();
    let fixture = builder.build_with_websocket_server(&server).await?;
    fixture.submit_turn("hi").await?;
    assert_eq!(server.handshakes().len(), 1);
    server.shutdown().await;

    let log_text = sinks
        .assert_clean(
            guard,
            &["successfully connected to websocket"],
            &[COOKIE_VALUE],
        )
        .await?;
    assert!(
        log_text.contains("\"set-cookie\": \"REDACTED\""),
        "the handshake headers should be logged with values redacted"
    );
    Ok(())
}

/// No line of the trace log is tungstenite's handshake request, whatever
/// redaction did to its header values.
fn assert_no_handshake_request(log_text: &str) {
    assert!(
        !log_text
            .lines()
            .any(|line| line.contains("tungstenite::handshake::client") && line.contains("Request")),
        "tungstenite's handshake request reached the trace log"
    );
}

/// #380: with `RUST_LOG=trace`, tungstenite logs the websocket upgrade
/// request at TRACE, `Authorization: Bearer <key>` included. The API key
/// must reach the server and no log sink.
#[tokio::test]
async fn websocket_api_key_never_reaches_trace_logs() -> anyhow::Result<()> {
    const API_KEY: &str = "fake-ws-api-key-380-0001-6be0c2f1";

    let (sinks, guard) = TraceSinks::install().await?;
    let server = start_websocket_server_with_headers(vec![WebSocketConnectionConfig {
        requests: vec![
            vec![ev_response_created("warm-1"), ev_completed("warm-1")],
            vec![
                ev_response_created("resp-1"),
                ev_assistant_message("msg-1", "done"),
                ev_completed("resp-1"),
            ],
        ],
        response_headers: Vec::new(),
        accept_delay: None,
        close_after_requests: false,
    }])
    .await;
    let mut builder = test_codex().with_auth(CodexAuth::from_api_key(API_KEY));
    let fixture = builder.build_with_websocket_server(&server).await?;
    fixture.submit_turn("hi").await?;
    let handshakes = server.handshakes();
    assert_eq!(
        handshakes
            .iter()
            .map(|handshake| handshake.header("authorization"))
            .collect::<Vec<_>>(),
        vec![Some(format!("Bearer {API_KEY}"))]
    );
    server.shutdown().await;

    let log_text = sinks
        .assert_clean(guard, &["successfully connected to websocket"], &[API_KEY])
        .await?;
    // The bridge delivered tungstenite's records; only TRACE was dropped.
    assert!(
        log_text.contains("Client handshake done."),
        "tungstenite's DEBUG records should reach the trace log"
    );
    assert_no_handshake_request(&log_text);
    Ok(())
}

/// #380: the same API key on the HTTP (SSE) transport, sent by reqwest,
/// hyper and h2 under the same `RUST_LOG`.
#[tokio::test]
async fn http_api_key_never_reaches_trace_logs() -> anyhow::Result<()> {
    const API_KEY: &str = "fake-http-api-key-380-0002-0d93a7e4";

    let (sinks, guard) = TraceSinks::install().await?;
    let server = start_mock_server().await;
    let response = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-1"),
            ev_assistant_message("msg-1", "done"),
            ev_completed("resp-1"),
        ]),
    )
    .await;
    let mut builder = test_codex().with_auth(CodexAuth::from_api_key(API_KEY));
    let fixture = builder.build(&server).await?;
    fixture.submit_turn("hi").await?;
    assert_eq!(
        response.single_request().header("authorization"),
        Some(format!("Bearer {API_KEY}"))
    );

    let log_text = sinks
        .assert_clean(guard, &["Request completed"], &[API_KEY])
        .await?;
    assert!(
        log_text.contains("reqwest::"),
        "reqwest's bridged records should reach the trace log"
    );
    Ok(())
}
