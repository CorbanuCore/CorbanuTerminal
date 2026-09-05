use super::*;
use pretty_assertions::assert_eq;

async fn owner(level: SecurityLevel) -> Arc<Session> {
    let (mut session, _) = crate::session::tests::make_session_and_context().await;
    session.services.agent_control = session
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(level, session.thread_id, false)
        .unwrap();
    Arc::new(session)
}

async fn client(owner: &Arc<Session>) -> Result<StageOneMemoryClient, StageOneMemoryError> {
    StageOneMemoryClient::new(
        Arc::downgrade(owner),
        futures::future::pending().boxed().shared(),
        owner.thread_id,
        &owner.provider().await,
    )
    .await
}

#[tokio::test]
async fn pf_30_s04_live_protected_floor_denies_without_a_request() {
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let owner = owner(level).await;
        assert!(matches!(
            client(&owner).await,
            Err(StageOneMemoryError::Denied(
                StageOneMemoryDenial::ProtectedInputUnavailable
            ))
        ));
    }
}

#[tokio::test]
async fn pf_30_s04_binding_is_owner_specific_and_missing_policy_denies() {
    let owner = owner(SecurityLevel::Permissive).await;
    let wrong = StageOneMemoryClient::new(
        Arc::downgrade(&owner),
        futures::future::pending().boxed().shared(),
        ThreadId::new(),
        &owner.provider().await,
    )
    .await;
    assert!(matches!(
        wrong,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::OwnerMismatch
        ))
    ));
    let (session, _) = crate::session::tests::make_session_and_context().await;
    let session = Arc::new(session);
    assert!(matches!(
        client(&session).await,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::PolicyUnavailable
        ))
    ));
}

#[tokio::test]
async fn pf_30_s04_live_strengthening_is_sticky_through_later_downgrade() {
    let owner = owner(SecurityLevel::Permissive).await;
    let client = client(&owner).await.unwrap();
    assert!(client.check_completion().await.is_ok());
    let controller = owner
        .services
        .agent_control
        .trusted_security_controller()
        .unwrap();
    for level in [SecurityLevel::Moderate, SecurityLevel::Permissive] {
        let change = controller
            .confirm_level_change(level, codex_security_policy::RevocationState::new())
            .unwrap();
        controller.apply_confirmed_change(change).unwrap();
        assert!(matches!(
            client.check_completion().await,
            Err(StageOneMemoryError::Denied(
                StageOneMemoryDenial::ProtectedInputUnavailable
            ))
        ));
    }
}

#[tokio::test]
async fn pf_30_s04_termination_and_dropped_owner_fail_closed() {
    let owner = owner(SecurityLevel::Permissive).await;
    let completed = StageOneMemoryClient::new(
        Arc::downgrade(&owner),
        futures::future::ready(()).boxed().shared(),
        owner.thread_id,
        &owner.provider().await,
    )
    .await;
    assert!(matches!(
        completed,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::OwnerTerminated
        ))
    ));
    let client = client(&owner).await.unwrap();
    drop(owner);
    assert!(matches!(
        client.check_completion().await,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::OwnerTerminated
        ))
    ));
}

#[tokio::test]
async fn pf_30_s04_provider_change_invalidates_bound_client() {
    let owner = owner(SecurityLevel::Permissive).await;
    let client = client(&owner).await.unwrap();
    owner
        .update_settings(crate::session::SessionSettingsUpdate {
            model_provider: Some(codex_model_provider_info::ZAI_PROVIDER_ID.into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(matches!(
        client.check_completion().await,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::ProviderChanged
        ))
    ));
}

#[tokio::test]
async fn pf_30_s04_inherited_policy_is_not_weakened_by_permissive_worker_configuration() {
    let parent = owner(SecurityLevel::Moderate).await;
    let (mut child, _) = crate::session::tests::make_session_and_context().await;
    child.services.agent_control = parent
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(SecurityLevel::Permissive, child.thread_id, false)
        .unwrap();
    assert!(matches!(
        client(&Arc::new(child)).await,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::ProtectedInputUnavailable
        ))
    ));
}

#[tokio::test]
async fn pf_30_s04_kill_switch_denies_an_existing_worker_binding() {
    use codex_security_policy::PolicyPrincipal;
    use codex_security_policy::PrincipalKind;
    use codex_security_policy::RevocationEvent;
    use codex_security_policy::RevocationReason;
    use codex_security_policy::RevocationState;
    use codex_security_policy::RevocationTarget;
    let owner = owner(SecurityLevel::Permissive).await;
    let client = client(&owner).await.unwrap();
    let mut revocations = RevocationState::new();
    revocations
        .apply(
            &RevocationEvent::new(
                PolicyPrincipal::new(PrincipalKind::Human, "fixture-human").unwrap(),
                RevocationTarget::KillSwitch { active: true },
                RevocationReason::KillSwitch,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let controller = owner
        .services
        .agent_control
        .trusted_security_controller()
        .unwrap();
    controller
        .apply_confirmed_change(
            controller
                .confirm_level_change(SecurityLevel::Permissive, revocations)
                .unwrap(),
        )
        .unwrap();
    assert!(matches!(
        client.check_completion().await,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::KillSwitchActive
        ))
    ));
}

#[tokio::test]
async fn pf_30_s04_http_attempt_guard_withholds_canary_after_live_change() {
    let owner = owner(SecurityLevel::Permissive).await;
    let client = client(&owner).await.unwrap();
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(wiremock::ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let http = codex_login::default_client::create_client_for_route(
        &codex_http_client::HttpClientFactory::new(
            codex_http_client::OutboundProxyPolicy::ReqwestDefault,
        ),
        &server.uri(),
        codex_http_client::ClientRouteClass::Api,
    )
    .unwrap();
    let transport = StageOneGuardedTransport::new(
        ReqwestTransport::from_http_client(http),
        Some(Arc::clone(&client.binding)),
    );
    let mut request = Request::new(http::Method::POST, server.uri());
    request.body = Some(codex_http_client::RequestBody::Json(
        serde_json::json!({"canary": "synthetic-private-rollout"}),
    ));
    transport.execute(request.clone()).await.unwrap();
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    let controller = owner
        .services
        .agent_control
        .trusted_security_controller()
        .unwrap();
    controller
        .apply_confirmed_change(
            controller
                .confirm_level_change(
                    SecurityLevel::Aggressive,
                    codex_security_policy::RevocationState::new(),
                )
                .unwrap(),
        )
        .unwrap();
    // Both lower HTTP entry points enforce the same binding on every invocation,
    // including retry attempts after auth/backoff has already completed.
    assert!(matches!(
        transport.execute(request.clone()).await,
        Err(TransportError::Build(_))
    ));
    assert!(matches!(
        transport.stream(request).await,
        Err(TransportError::Build(_))
    ));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn pf_30_s04_websocket_connect_race_denies_before_first_frame() {
    let (mut session, context) = crate::session::tests::make_session_and_context().await;
    session.services.agent_control = session
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(SecurityLevel::Permissive, session.thread_id, false)
        .unwrap();
    let owner = Arc::new(session);
    let mut client = client(&owner).await.unwrap();
    let controller = owner
        .services
        .agent_control
        .trusted_security_controller()
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut provider = owner.provider().await;
    provider.base_url = Some(format!("http://{}", listener.local_addr().unwrap()));
    provider.requires_openai_auth = false;
    provider.env_key = None;
    provider.supports_websockets = true;
    // Only this private fixture substitutes a socket endpoint; the live binding
    // and production post-connect frame dispatch remain unchanged.
    client.client = client.client.for_provider(&provider);
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut websocket_config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default();
        websocket_config.extensions.permessage_deflate = Some(Default::default());
        let mut socket = tokio_tungstenite::accept_hdr_async_with_config(
            socket,
            move |_: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
                controller
                    .apply_confirmed_change(
                        controller
                            .confirm_level_change(
                                SecurityLevel::Moderate,
                                codex_security_policy::RevocationState::new(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                Ok(response)
            },
            Some(websocket_config),
        )
        .await
        .unwrap();
        let frame =
            tokio::time::timeout(std::time::Duration::from_millis(250), socket.next()).await;
        assert!(!matches!(
            frame,
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(_))))
        ));
    });
    let prompt = Prompt::default();
    let metadata = CodexResponsesMetadata::new(
        "fixture".into(),
        "fixture".into(),
        owner.thread_id.to_string(),
        "fixture:0".into(),
    );
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.extract(StageOneMemoryRequest {
            prompt: &prompt,
            model_info: &context.model_info,
            session_telemetry: &context.session_telemetry,
            reasoning_effort: None,
            reasoning_summary: ReasoningSummary::default(),
            service_tier: None,
            responses_metadata: &metadata,
        }),
    )
    .await
    .unwrap();
    assert!(matches!(
        result,
        Err(StageOneMemoryError::Denied(
            StageOneMemoryDenial::ProtectedInputUnavailable
        ))
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn pf_30_s04_owner_termination_cancels_pending_http_without_a_retry() {
    let (mut session, context) = crate::session::tests::make_session_and_context().await;
    session.services.agent_control = session.services.agent_control.clone()
        .with_effective_security_policy(SecurityLevel::Permissive, session.thread_id, false).unwrap();
    let owner = Arc::new(session);
    let (terminate, terminated) = tokio::sync::oneshot::channel();
    let mut client = StageOneMemoryClient::new(Arc::downgrade(&owner),
        async move { let _ = terminated.await; }.boxed().shared(), owner.thread_id, &owner.provider().await).await.unwrap();
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(30)))
        .mount(&server).await;
    let mut provider = owner.provider().await;
    provider.base_url = Some(server.uri());
    provider.requires_openai_auth = false;
    provider.env_key = None;
    provider.supports_websockets = false;
    client.client = client.client.for_provider(&provider);
    let task = tokio::spawn(async move {
        let prompt = Prompt::default();
        let metadata = CodexResponsesMetadata::new("fixture".into(), "fixture".into(), "fixture".into(), "fixture:0".into());
        client.extract(StageOneMemoryRequest { prompt: &prompt, model_info: &context.model_info,
            session_telemetry: &context.session_telemetry, reasoning_effort: None,
            reasoning_summary: ReasoningSummary::default(), service_tier: None, responses_metadata: &metadata }).await
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while server.received_requests().await.unwrap().is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }).await.unwrap();
    terminate.send(()).unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(1), task).await.unwrap().unwrap();
    assert!(matches!(result, Err(StageOneMemoryError::Denied(StageOneMemoryDenial::OwnerTerminated))));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
