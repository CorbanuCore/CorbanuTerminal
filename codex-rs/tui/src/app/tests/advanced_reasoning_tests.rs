use super::*;
use crate::app_server_session::ResumeModelSettings;
use crate::app_server_session::ResumePermissionSettings;
use app_test_support::create_fake_rollout;
use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use codex_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn fork_current_session_preserves_conversation_ultra() -> Result<()> {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    let codex_home = tempdir()?;
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    app.config.sqlite = codex_state::SqliteConfig::new_for_testing(codex_home.path().abs());
    let source_thread_id = ThreadId::from_string(
        &create_fake_rollout(
            codex_home.path(),
            "2025-01-05T12-00-00",
            "2025-01-05T12:00:00Z",
            "Saved user message",
            Some(codex_model_provider_info::OPENAI_PROVIDER_ID),
            /*git_info*/ None,
        )
        .expect("create source rollout"),
    )?;
    let mut app_server = crate::start_embedded_app_server_for_picker(&app.config).await?;
    app_server
        .resume_thread(
            app.config.clone(),
            source_thread_id,
            ResumeModelSettings::RestoreFromThread,
            ResumePermissionSettings::RestoreFromThread,
        )
        .await?;
    app.chat_widget.handle_thread_session(ThreadSessionState {
        daybreak_enabled: false,
        windows_sandbox_host: crate::app::WindowsSandboxHost::Local,
        model: "gpt-5.4".to_string(),
<<<<<<< HEAD
        model_provider_id: codex_model_provider_info::OPENAI_PROVIDER_ID.to_string(),
||||||| 413492cd6c
=======
        model_provider_id: app.config.model_provider_id.clone(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        reasoning_effort: Some(ReasoningEffortConfig::Ultra),
        ..test_thread_session(source_thread_id, test_path_buf("/tmp/project"))
    });
<<<<<<< HEAD
    while app_event_rx.try_recv().is_ok() {}
||||||| 413492cd6c
=======
    app.ensure_thread_channel(source_thread_id);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let mut tui = crate::tui::test_support::make_test_tui()?;

    let control = Box::pin(app.handle_event(
        &mut tui,
        &mut app_server,
        AppEvent::ForkCurrentSession { name: None },
    ))
    .await?;

    assert!(matches!(control, AppRunControl::Continue));
<<<<<<< HEAD
    let history = std::iter::from_fn(|| app_event_rx.try_recv().ok())
        .filter_map(|event| match event {
            AppEvent::InsertHistoryCell(cell) => {
                Some(lines_to_single_string(&cell.display_lines(/*width*/ 120)))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_ne!(
        app.chat_widget.thread_id(),
        Some(source_thread_id),
        "fork history: {history:?}"
    );
||||||| 413492cd6c
    assert_ne!(app.chat_widget.thread_id(), Some(source_thread_id));
=======
    assert_ne!(app.chat_widget.thread_id(), Some(source_thread_id));
    assert!(
        app.agents_overview
            .dispatched_requests
            .contains_key(&source_thread_id)
    );
    assert!(!app.thread_event_channels.contains_key(&source_thread_id));
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    assert_eq!(app.chat_widget.current_model(), "gpt-5.4");
    assert_eq!(
        app.chat_widget.current_reasoning_effort(),
        Some(ReasoningEffortConfig::Ultra)
    );
    app_server.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn switching_from_ultra_thread_restores_configured_plan_effort() {
    let mut app = make_test_app().await;
    app.config.plan_mode_reasoning_effort = Some(ReasoningEffortConfig::High);
    app.chat_widget
        .set_feature_enabled(Feature::CollaborationModes, /*enabled*/ true);
    let ultra_session = ThreadSessionState {
        daybreak_enabled: false,
        windows_sandbox_host: crate::app::WindowsSandboxHost::Local,
        model: "gpt-5.4".to_string(),
        reasoning_effort: Some(ReasoningEffortConfig::Ultra),
        ..test_thread_session(ThreadId::new(), test_path_buf("/tmp/ultra"))
    };
    let normal_session = ThreadSessionState {
        daybreak_enabled: false,
        windows_sandbox_host: crate::app::WindowsSandboxHost::Local,
        model: "gpt-5.4".to_string(),
        reasoning_effort: Some(ReasoningEffortConfig::Medium),
        ..test_thread_session(ThreadId::new(), test_path_buf("/tmp/normal"))
    };

    app.replay_thread_snapshot(
        ThreadEventSnapshot {
            delegated_turns: Vec::new(),
            session: Some(ultra_session),
            turns: Vec::new(),
            events: Vec::new(),
            active_reasoning_item: None,
            input_state: None,
        },
        /*resume_restored_queue*/ false,
    );
    app.replay_thread_snapshot(
        ThreadEventSnapshot {
            delegated_turns: Vec::new(),
            session: Some(normal_session),
            turns: Vec::new(),
            events: Vec::new(),
            active_reasoning_item: None,
            input_state: None,
        },
        /*resume_restored_queue*/ false,
    );
    app.chat_widget
        .handle_key_event(KeyEvent::from(KeyCode::BackTab));

    assert_eq!(
        app.chat_widget.active_collaboration_mode_kind(),
        ModeKind::Plan
    );
    assert_eq!(
        app.chat_widget.current_reasoning_effort(),
        Some(ReasoningEffortConfig::High)
    );
}

#[tokio::test]
async fn switching_threads_restores_session_speed_for_the_next_turn() {
    let (mut app, _events, mut ops) = make_test_app_with_channels().await;
    app.config.service_tier = Some("flex".into());
    app.chat_widget.set_service_tier(Some("flex".into()));
    let standard_thread = ThreadId::new();
    let flex_thread = ThreadId::new();
    let session = |thread_id, service_tier: &str| ThreadSessionState {
        model: "gpt-5.5".into(),
        service_tier: Some(service_tier.into()),
        ..test_thread_session(thread_id, test_path_buf("/tmp/project"))
    };
    let snapshot = |session| ThreadEventSnapshot {
        delegated_turns: Vec::new(),
        session: Some(session),
        turns: Vec::new(),
        events: Vec::new(),
        active_reasoning_item: None,
        input_state: None,
    };

    app.replay_thread_snapshot(
        snapshot(session(standard_thread, SERVICE_TIER_DEFAULT_REQUEST_VALUE)),
        /*resume_restored_queue*/ false,
    );
    app.replay_thread_snapshot(
        snapshot(session(flex_thread, "flex")),
        /*resume_restored_queue*/ false,
    );
    assert_eq!(
        app.chat_widget.configured_service_tier().as_deref(),
        Some("flex")
    );
    app.replay_thread_snapshot(
        snapshot(session(standard_thread, SERVICE_TIER_DEFAULT_REQUEST_VALUE)),
        /*resume_restored_queue*/ false,
    );
    app.chat_widget
        .apply_external_edit("Check this task".into());
    app.chat_widget.handle_key_event(KeyCode::Esc.into());
    app.chat_widget.handle_key_event(KeyCode::Enter.into());

    assert_matches!(next_user_turn_op(&mut ops), Op::UserTurn { service_tier, .. }
        if service_tier == Some(Some(SERVICE_TIER_DEFAULT_REQUEST_VALUE.into())));
    assert_eq!(app.config.service_tier.as_deref(), Some("flex"));
}
