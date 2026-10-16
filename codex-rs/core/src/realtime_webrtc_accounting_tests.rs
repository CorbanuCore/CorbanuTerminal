//! Native HTTP/sideband/SQLite regressions for #889 item 2. No media or live provider.
use super::super as realtime;
use super::provider_bound;
use super::realtime_ledger;
use super::realtime_ledger_exists;
use super::realtime_ledger_session_with;
use super::realtime_ledger_settled;
use super::realtime_websocket_params;
use async_channel::Receiver;
use async_channel::Sender;
use codex_protocol::protocol::ConversationStartTransport;
use codex_protocol::protocol::RealtimeConversationVersion;
use futures::SinkExt;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

enum SidebandEnd {
    Close,
    Drop,
}

struct Sideband {
    endpoint: String,
    accepted: Receiver<()>,
    connected: Receiver<()>,
    end: Sender<SidebandEnd>,
    server: JoinHandle<()>,
}

#[derive(Clone, Copy)]
enum Handshake {
    Complete,
    Pending,
}

impl Drop for Sideband {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn sideband() -> anyhow::Result<Sideband> {
    sideband_handshake(Handshake::Complete).await
}

async fn sideband_handshake(handshake: Handshake) -> anyhow::Result<Sideband> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}/v1", listener.local_addr()?);
    let (connected_tx, connected) = async_channel::bounded(4);
    let (accepted_tx, accepted) = async_channel::bounded(4);
    let (end, end_rx) = async_channel::bounded(4);
    let server = tokio::spawn(async move {
        let mut connections = tokio::task::JoinSet::new();
        while let Ok((socket, _)) = listener.accept().await {
            let connected = connected_tx.clone();
            let accepted = accepted_tx.clone();
            let end = end_rx.clone();
            connections.spawn(async move {
                let _ = accepted.send(()).await;
                if matches!(handshake, Handshake::Pending) {
                    std::future::pending::<()>().await;
                }
                let Ok(mut ws) = tokio_tungstenite::accept_async(socket).await else {
                    return;
                };
                // The real client initializes its legacy sideband with session.update.
                if !matches!(ws.next().await, Some(Ok(_))) {
                    return;
                }
                let _ = connected.send(()).await;
                loop {
                    tokio::select! {
                        next = ws.next() => if !matches!(next, Some(Ok(_))) { return; },
                        action = end.recv() => {
                            if matches!(action, Ok(SidebandEnd::Close)) {
                                let _ = ws.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
                            }
                            return;
                        }
                    }
                }
            });
        }
    });
    Ok(Sideband {
        endpoint,
        accepted,
        connected,
        end,
        server,
    })
}

async fn call_server(status: u16) -> wiremock::MockServer {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/realtime/calls"))
        .respond_with(
            wiremock::ResponseTemplate::new(status)
                .insert_header("Location", "/v1/live/rtc_fixture")
                .set_body_string("v=answer\r\n"),
        )
        .mount(&server)
        .await;
    server
}

fn params() -> codex_protocol::protocol::ConversationStartParams {
    let mut params = realtime_websocket_params();
    params.transport = Some(ConversationStartTransport::Webrtc {
        sdp: "v=offer\r\n".into(),
    });
    params.version = Some(RealtimeConversationVersion::V1);
    params.output_modality = codex_protocol::protocol::RealtimeOutputModality::Audio;
    params
}

async fn session(
    home: &std::path::Path,
    endpoint: &str,
    sideband: &Sideband,
    accounting: crate::config::AccountingMode,
) -> anyhow::Result<(
    Arc<crate::session::session::Session>,
    Arc<codex_state::StateRuntime>,
)> {
    realtime_ledger_session_with(home, endpoint, accounting, |config| {
        config.experimental_realtime_ws_base_url = Some(sideband.endpoint.clone());
        config.experimental_realtime_webrtc_call_base_url = Some(endpoint.to_string());
    })
    .await
}

async fn connected(sideband: &Sideband) -> anyhow::Result<()> {
    tokio::time::timeout(Duration::from_secs(5), sideband.connected.recv()).await??;
    Ok(())
}

async fn start(owner: &Arc<crate::session::session::Session>, sub_id: &str) -> anyhow::Result<()> {
    let prepared = realtime::prepare_realtime_start(owner, params()).await?;
    realtime::handle_start_inner(owner, sub_id, prepared).await?;
    Ok(())
}

async fn stopped(owner: &crate::session::session::Session) -> anyhow::Result<()> {
    tokio::time::timeout(Duration::from_secs(5), async {
        while owner.conversation.running_state().await.is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    Ok(())
}

fn assert_end(attempts: &[Value], observations: &[Value], ended_after: i64) {
    assert_eq!(
        attempts.len(),
        1,
        "one HTTP creation attempt: {attempts:#?}"
    );
    assert_eq!(attempts[0]["turn"], "realtime:webrtc-fixture");
    assert_eq!(attempts[0]["model"], "gpt-realtime-1.5");
    assert_eq!(
        observations.len(),
        1,
        "one sideband end observation: {observations:#?}"
    );
    let end = observations[0]["patch"]["session_ended_at_ms"]
        .as_i64()
        .expect("sideband end");
    assert!(end >= ended_after);
    assert_eq!(
        observations[0]["patch"],
        serde_json::json!({"session_ended_at_ms":end}),
        "the end must not invent token counts, zero usage, price or refusal"
    );
}

#[tokio::test]
async fn pf_889_webrtc_sideband_normal_end_observes_existing_attempt() -> anyhow::Result<()> {
    let http = call_server(/*status*/ 200).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband().await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(home.path(), &endpoint, &sideband, provider_bound(&endpoint)).await?;
    start(&owner, "webrtc-fixture").await?;
    connected(&sideband).await?;
    let ended_after = chrono::Utc::now().timestamp_millis();
    sideband.end.send(SidebandEnd::Close).await?;
    stopped(&owner).await?;
    let (attempts, observations) = realtime_ledger_settled(&db, /*count*/ 1).await?;
    assert_end(&attempts, &observations, ended_after);
    assert_eq!(
        http.received_requests().await.expect("captured HTTP").len(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn pf_889_webrtc_collection_off_keeps_session_unrecorded() -> anyhow::Result<()> {
    let http = call_server(/*status*/ 200).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband().await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(
        home.path(),
        &endpoint,
        &sideband,
        crate::config::AccountingMode::Disabled,
    )
    .await?;
    start(&owner, "webrtc-fixture").await?;
    connected(&sideband).await?;
    realtime::handle_close(&owner, "webrtc-fixture".into()).await;
    assert!(!realtime_ledger_exists(&db).await?);
    Ok(())
}

#[tokio::test]
async fn pf_889_webrtc_requested_close_observes_end_before_return() -> anyhow::Result<()> {
    let http = call_server(/*status*/ 200).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband().await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(home.path(), &endpoint, &sideband, provider_bound(&endpoint)).await?;
    start(&owner, "webrtc-fixture").await?;
    connected(&sideband).await?;
    let ended_after = chrono::Utc::now().timestamp_millis();
    realtime::handle_close(&owner, "webrtc-fixture".into()).await;
    let (attempts, observations) = realtime_ledger_settled(&db, /*count*/ 1).await?;
    assert_end(&attempts, &observations, ended_after);
    Ok(())
}

#[tokio::test]
async fn pf_889_webrtc_transport_drop_observes_existing_attempt() -> anyhow::Result<()> {
    let http = call_server(/*status*/ 200).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband().await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(home.path(), &endpoint, &sideband, provider_bound(&endpoint)).await?;
    start(&owner, "webrtc-fixture").await?;
    connected(&sideband).await?;
    let ended_after = chrono::Utc::now().timestamp_millis();
    sideband.end.send(SidebandEnd::Drop).await?;
    stopped(&owner).await?;
    let (attempts, observations) = realtime_ledger_settled(&db, /*count*/ 1).await?;
    assert_end(&attempts, &observations, ended_after);
    Ok(())
}

#[tokio::test]
async fn pf_889_webrtc_cancel_pending_sideband_preserves_creation_attempt() -> anyhow::Result<()> {
    let http = call_server(/*status*/ 200).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband_handshake(Handshake::Pending).await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(home.path(), &endpoint, &sideband, provider_bound(&endpoint)).await?;
    start(&owner, "webrtc-fixture").await?;
    tokio::time::timeout(Duration::from_secs(5), sideband.accepted.recv()).await??;
    let ended_after = chrono::Utc::now().timestamp_millis();
    realtime::handle_close(&owner, "webrtc-fixture".into()).await;
    let (attempts, observations) = realtime_ledger_settled(&db, /*count*/ 1).await?;
    assert_end(&attempts, &observations, ended_after);
    assert_eq!(
        http.received_requests().await.expect("captured HTTP").len(),
        1
    );
    Ok(())
}

async fn bound_ends(db: &codex_state::StateRuntime) -> anyhow::Result<Vec<(String, Value)>> {
    let pool = db
        .sqlite()
        .open_read_only_pool(&db.sqlite().state_db_path())
        .await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT attempt_id, payload FROM draft_accounting_observations ORDER BY rowid",
    )
    .fetch_all(&pool)
    .await?;
    pool.close().await;
    rows.into_iter()
        .map(|(id, body)| Ok((id, serde_json::from_str(&body)?)))
        .collect()
}

#[tokio::test]
async fn pf_889_webrtc_replacement_shutdown_and_stale_stop_keep_attempt_identity()
-> anyhow::Result<()> {
    let http = call_server(/*status*/ 200).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband().await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(home.path(), &endpoint, &sideband, provider_bound(&endpoint)).await?;
    start(&owner, "first").await?;
    connected(&sideband).await?;
    let first_stop = owner
        .conversation
        .state
        .lock()
        .await
        .as_ref()
        .expect("first state")
        .stop_token
        .clone();
    start(&owner, "second").await?;
    connected(&sideband).await?;
    let (attempts, observations) = realtime_ledger_settled(&db, /*count*/ 1).await?;
    assert_eq!(attempts.len(), 2);
    assert_eq!(observations.len(), 1, "only first session ended");
    let ends = bound_ends(&db).await?;
    assert_eq!(
        ends[0].0,
        attempts[0]["attempt_id"].as_str().expect("first id")
    );
    let first_end = ends[0].1["patch"]["session_ended_at_ms"]
        .as_i64()
        .expect("first end");
    assert!(
        first_end
            <= attempts[1]["dispatched_at_ms"]
                .as_i64()
                .expect("second start")
    );
    first_stop.cancel();
    owner
        .conversation
        .text_in(codex_protocol::protocol::ConversationTextParams {
            text: "synthetic second-session probe".into(),
            role: codex_protocol::protocol::ConversationTextRole::User,
        })
        .await?;
    assert!(owner.conversation.running_state().await.is_some());
    assert_eq!(
        realtime_ledger(&db).await?.1,
        observations,
        "old cancellation adds no duplicate end"
    );
    owner.conversation.shutdown().await?;
    let (_, observations) = realtime_ledger_settled(&db, /*count*/ 2).await?;
    assert_eq!(observations.len(), 2);
    let ends = bound_ends(&db).await?;
    assert_eq!(
        ends[1].0,
        attempts[1]["attempt_id"].as_str().expect("second id")
    );
    for (_, observation) in ends {
        let end = observation["patch"]["session_ended_at_ms"]
            .as_i64()
            .expect("end");
        assert_eq!(
            observation["patch"],
            serde_json::json!({"session_ended_at_ms":end})
        );
    }
    Ok(())
}

#[tokio::test]
async fn pf_889_webrtc_http_failure_does_not_invent_sideband_end() -> anyhow::Result<()> {
    let http = call_server(/*status*/ 500).await;
    let endpoint = format!("{}/v1", http.uri());
    let sideband = sideband().await?;
    let home = tempfile::tempdir()?;
    let (owner, db) = session(home.path(), &endpoint, &sideband, provider_bound(&endpoint)).await?;
    assert!(start(&owner, "webrtc-fixture").await.is_err());
    assert!(owner.conversation.running_state().await.is_none());
    let (attempts, observations) = realtime_ledger_settled(&db, /*count*/ 0).await?;
    assert_eq!(
        attempts.len(),
        1,
        "failed HTTP creation still retains its existing request"
    );
    assert!(
        observations
            .iter()
            .all(|o| o["patch"]["session_ended_at_ms"].is_null())
    );
    assert!(sideband.accepted.is_empty());
    Ok(())
}
