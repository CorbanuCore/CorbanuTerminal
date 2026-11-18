//! #2563: retries must be admitted again after the live security policy changes.
use super::pf_30_realtime_start;
use crate::client::ModelClient;
use crate::realtime_conversation as realtime;
use crate::security::EffectivePolicyInitialization;
use crate::security::EffectivePolicyView;
use crate::security::PersistedHumanSecurityState;
use crate::security::TrustedSecurityController;
use crate::session::session::Session;
use codex_api::RealtimeEvent;
use codex_api::RealtimeEventParser;
use codex_api::RealtimeWebsocketClient;
use codex_protocol::protocol::ConversationTextParams;
use codex_protocol::protocol::ConversationTextRole;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use futures::StreamExt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use test_case::test_case;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

const LIMIT: Duration = Duration::from_secs(5);

fn live_policy_client(session: &Session) -> (ModelClient, TrustedSecurityController) {
    let view = EffectivePolicyView::default();
    let controller = TrustedSecurityController::initialize(
        &view,
        PersistedHumanSecurityState::new(
            SecuritySettings::new(SecurityLevel::Permissive),
            PolicyPrincipal::new(PrincipalKind::Human, "fixture-human").unwrap(),
            RevocationState::new(),
        )
        .unwrap(),
        session.thread_id,
        codex_protocol::SessionId::from(session.thread_id),
        EffectivePolicyInitialization::Root,
    )
    .unwrap();
    let client = session
        .services
        .model_client()
        .as_ref()
        .clone()
        .with_ingress_policy(SecurityLevel::Permissive, view);
    (client, controller)
}

fn handoff_state(
    output_tx: async_channel::Sender<realtime::RealtimeOutbound>,
    event_parser: RealtimeEventParser,
) -> realtime::RealtimeHandoffState {
    realtime::RealtimeHandoffState {
        output_tx,
        last_output: Arc::new(tokio::sync::Mutex::new(None)),
        stream: Arc::new(tokio::sync::Mutex::new(Default::default())),
        client_managed_handoffs: false,
        codex_responses_as_items: false,
        codex_response_item_prefix: None,
        codex_response_handoff_mode: Default::default(),
        backend_reasoning_status: false,
        codex_response_handoff_channel_prefixes: Arc::new(Default::default()),
        session_kind: realtime::RealtimeSessionKind::V1,
        event_parser,
    }
}

#[derive(Clone, Copy, Debug)]
enum PendingKind {
    Text,
    Handoff,
}

impl PendingKind {
    fn message(self, text: &str) -> Box<realtime::RealtimePendingOutbound> {
        Box::new(match self {
            Self::Text => realtime::RealtimePendingOutbound::Text(ConversationTextParams {
                text: text.into(),
                role: ConversationTextRole::User,
            }),
            Self::Handoff => realtime::RealtimePendingOutbound::Handoff(
                realtime::RealtimeOutbound::StandaloneHandoff {
                    text: text.into(),
                    phase: Some(codex_protocol::models::MessagePhase::FinalAnswer),
                },
            ),
        })
    }
}

#[test_case(PendingKind::Text; "text")]
#[test_case(PendingKind::Handoff; "handoff")]
#[tokio::test]
async fn pending_outbound_resend_rechecks_live_policy(kind: PendingKind) -> anyhow::Result<()> {
    let (session, _) = crate::session::tests::make_session_and_context().await;
    let (model_client, controller) = live_policy_client(&session);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let start = pf_30_realtime_start(
        model_client.clone(),
        format!("http://{}", listener.local_addr()?),
        /*sdp*/ None,
    );
    let client = RealtimeWebsocketClient::new(start.api_provider, start.http_client_factory);
    let (connection, mut websocket) = timeout(LIMIT, async {
        tokio::try_join!(
            async {
                Ok::<_, anyhow::Error>(
                    client
                        .connect(start.session_config, Default::default(), Default::default())
                        .await?,
                )
            },
            async {
                let (socket, _) = listener.accept().await?;
                Ok::<_, anyhow::Error>(tokio_tungstenite::accept_async(socket).await?)
            }
        )
    })
    .await??;
    let initial = timeout(LIMIT, websocket.next()).await?.unwrap()?;
    assert!(
        initial
            .to_text()?
            .contains("unadmitted-initial-instructions")
    );
    let (_text_tx, text_rx) = async_channel::bounded(1);
    let (handoff_tx, handoff_output_rx) = async_channel::bounded(1);
    let (_audio_tx, audio_rx) = async_channel::bounded(1);
    let (events_tx, events_rx) = async_channel::bounded(4);
    let state = handoff_state(handoff_tx, RealtimeEventParser::V1);
    let input = |stop_token| realtime::RealtimeInputTask {
        model_client: model_client.clone(),
        writer: connection.writer(),
        events: connection.events(),
        text_rx: text_rx.clone(),
        handoff_output_rx: handoff_output_rx.clone(),
        audio_rx: audio_rx.clone(),
        events_tx: events_tx.clone(),
        handoff_state: state.clone(),
        session_kind: realtime::RealtimeSessionKind::V1,
        event_parser: RealtimeEventParser::V1,
        stop_token,
        usage_tx: None,
    };

    // First prove this exact pending variant is sendable on the open transport.
    let stop = CancellationToken::new();
    let allowed = realtime::run_realtime_input_task(
        input(stop.clone()),
        Some(kind.message("permissive-pending-control")),
    );
    tokio::pin!(allowed);
    timeout(LIMIT, async {
        tokio::select! {
            _ = &mut allowed => panic!("allowed pending send stopped before its frame"),
            frame = websocket.next() => {
                assert!(frame.unwrap()?.to_text()?.contains("permissive-pending-control"));
            }
        }
        Ok::<_, anyhow::Error>(())
    })
    .await??;
    stop.cancel();
    assert!(matches!(
        timeout(LIMIT, allowed).await?,
        realtime::RealtimeInputTaskExit::Terminal
    ));
    assert!(events_rx.is_empty());

    controller.apply_confirmed_change(
        controller.confirm_level_change(SecurityLevel::Moderate, RevocationState::new())?,
    )?;
    let exit = timeout(
        LIMIT,
        realtime::run_realtime_input_task(
            input(CancellationToken::new()),
            Some(kind.message("protected-pending-canary")),
        ),
    )
    .await?;
    assert!(matches!(exit, realtime::RealtimeInputTaskExit::Terminal));
    let (event, transcript) = timeout(LIMIT, events_rx.recv()).await??;
    assert!(matches!(event, RealtimeEvent::Error(error) if error.contains("source admission")));
    assert!(transcript.is_none());
    // The sending future has returned; a buffered frame would still be readable here.
    assert!(
        timeout(Duration::from_millis(100), websocket.next())
            .await
            .is_err(),
        "a denied pending resend must write no frame"
    );
    Ok(())
}

#[tokio::test]
async fn webrtc_sideband_reconnect_rechecks_live_policy_before_connecting() -> anyhow::Result<()> {
    let (session, _) = crate::session::tests::make_session_and_context().await;
    let (model_client, controller) = live_policy_client(&session);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let start = pf_30_realtime_start(
        model_client.clone(),
        format!("http://{}", listener.local_addr()?),
        /*sdp*/ None,
    );
    let sideband_base_url = start.api_provider.base_url.clone();
    let mut session_config = start.session_config;
    session_config.event_parser = RealtimeEventParser::FramelessBidi;
    let (text_tx, text_rx) = async_channel::bounded(1);
    let (handoff_tx, handoff_output_rx) = async_channel::bounded(1);
    let (_audio_tx, audio_rx) = async_channel::bounded(1);
    let (events_tx, events_rx) = async_channel::bounded(4);
    let (tail_tx, tail_rx) = async_channel::bounded(1);
    let mut task = realtime::sideband::spawn_webrtc_sideband_input_task(
        realtime::RealtimeWebrtcSidebandInputTask {
            model_client,
            client: RealtimeWebsocketClient::new(start.api_provider, start.http_client_factory)
                .with_webrtc_sideband_base_url(sideband_base_url),
            session_config,
            call_id: "rtc_admission_fixture".into(),
            sideband_headers: Default::default(),
            session_initialization: realtime::RealtimeSidebandSessionInitialization::CoreCreated,
            input_channels: realtime::RealtimeInputChannels {
                text_rx,
                handoff_output_rx,
                audio_rx,
            },
            events_tx,
            handoff_state: handoff_state(handoff_tx, RealtimeEventParser::FramelessBidi),
            session_kind: realtime::RealtimeSessionKind::V1,
            event_parser: RealtimeEventParser::FramelessBidi,
            realtime_active: Arc::new(AtomicBool::new(true)),
            transcript_tail_flush: realtime::RealtimeTranscriptTailFlush {
                enabled: true,
                event_parser: RealtimeEventParser::FramelessBidi,
                tx: tail_tx,
            },
            stop_token: CancellationToken::new(),
        },
        /*record*/ None,
    );
    let result = timeout(LIMIT, async {
        let (socket, _) = listener.accept().await?;
        let mut websocket = tokio_tungstenite::accept_async(socket).await?;
        text_tx
            .send(ConversationTextParams {
                text: "permissive-sideband-control".into(),
                role: ConversationTextRole::User,
            })
            .await?;
        let frame = websocket.next().await.unwrap()?;
        assert!(frame.to_text()?.contains("permissive-sideband-control"));
        // No frame is queued after this point: an abrupt drop reaches TransportLost
        // directly, so the next policy check must be the reconnect-loop admission.
        controller.apply_confirmed_change(
            controller.confirm_level_change(SecurityLevel::Moderate, RevocationState::new())?,
        )?;
        drop(websocket);
        let (event, transcript) = events_rx.recv().await?;
        assert!(matches!(event, RealtimeEvent::Error(error) if error.contains("source admission")));
        assert!(transcript.is_none());
        (&mut task).await?;
        Ok::<_, anyhow::Error>(())
    })
    .await;
    task.abort();
    result??;
    // Completion makes this an assertion about the reconnect attempt, not a race
    // against the backoff timer. A queued TCP connection remains visible to accept.
    assert!(
        timeout(Duration::from_millis(100), listener.accept())
            .await
            .is_err(),
        "strengthened policy must prevent a second sideband connection"
    );
    assert!(tail_rx.try_recv().is_err());
    Ok(())
}
