//! Focused #2563 carries: roots-only authority, MXC refusal and follow-up image rules.

use super::*;
use codex_protocol::turn_input::NotSubmittedReason;
use codex_protocol::turn_input::TurnStartOptions;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
enum RootUpdate {
    Runtime,
    PrimaryEnvironment,
}

#[test_case::test_case(RootUpdate::Runtime; "runtime_roots")]
#[test_case::test_case(RootUpdate::PrimaryEnvironment; "primary_environment_roots")]
#[tokio::test]
async fn roots_only_changes_follow_effective_authority(update: RootUpdate) {
    let (session, turn, _events) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("writable fixture");
        },
    )
    .await;
    session
        .spawn_task(
            Arc::clone(&turn),
            Vec::new(),
            NeverEndingTask {
                kind: TaskKind::Regular,
                listen_to_cancellation_token: true,
            },
        )
        .await;
    let root = tempfile::tempdir().expect("new workspace root");
    let root = AbsolutePathBuf::from_absolute_path(root.path()).expect("absolute root");
    let before = session.thread_settings_snapshot().await;
    let before_environments = session.services.turn_environments.selections();
    let mut settings = SessionSettingsUpdate::default();
    match update {
        RootUpdate::Runtime => settings.runtime_workspace_roots = Some(vec![root.clone()]),
        RootUpdate::PrimaryEnvironment => {
            let mut environments = session.services.turn_environments.selections();
            let primary = environments.first_mut().expect("primary environment");
            primary.workspace_roots = vec![PathUri::from_abs_path(&root)];
            if let EnvironmentConfigState::Ready(config) = &mut primary.config {
                config.workspace_roots = primary.workspace_roots.clone();
            }
            settings.environments = Some(TurnEnvironmentSelections::new(
                session.get_config().await.cwd.clone(),
                environments,
            ));
        }
    }
    session
        .update_settings(settings)
        .await
        .expect("roots-only update");
    let input = vec![UserInput::Text {
        text: "work under the selected roots".to_string(),
        text_elements: Vec::new(),
    }];
    let submission = submit_steer_only(&session, input.clone(), &turn.sub_id).await;
    match update {
        RootUpdate::Runtime => {
            // #2064 requires comparison of the primary environment's materialized
            // permissions, as upstream's explicit selections also do. The first version
            // incorrectly treated runtime fallback metadata as a selected-root change.
            // Prove that only that metadata changed before expecting admission.
            let mut expected = before;
            expected.runtime_workspace_roots = Some(vec![root]);
            assert_eq!(session.thread_settings_snapshot().await, expected);
            assert_eq!(
                session.services.turn_environments.selections(),
                before_environments
            );
            assert!(
                matches!(submission, TurnInputSubmission::Steered { turn_id, .. }
                if turn_id == turn.sub_id)
            );
            let pending = session
                .input_queue
                .get_pending_input_for_running_task(&session, &turn.sub_id)
                .await
                .0;
            assert!(
                matches!(pending.as_slice(), [TurnInput::UserInput { content, .. }]
                if content == &input)
            );
        }
        RootUpdate::PrimaryEnvironment => {
            assert_ne!(
                session.thread_settings_snapshot().await.permission_profile,
                before.permission_profile
            );
            assert_eq!(
                submission,
                TurnInputSubmission::NotSubmitted {
                    reason: NotSubmittedReason::AuthorizationChanged,
                }
            );
        }
    }
    let mail = InterAgentCommunication::new(
        AgentPath::root(),
        AgentPath::root(),
        Vec::new(),
        "work under the new roots".to_string(),
        /*trigger_turn*/ false,
    );
    session
        .input_queue
        .enqueue_mailbox_communication_for_session(
            &session,
            mail.clone().into(),
            TurnStartOptions::default(),
        )
        .await;
    let expected_mail = vec![TurnInput::InterAgentCommunication(mail.into())];
    let (during_turn, after_boundary) = match update {
        RootUpdate::Runtime => (expected_mail, Vec::new()),
        RootUpdate::PrimaryEnvironment => (Vec::new(), expected_mail),
    };
    assert_eq!(
        session
            .input_queue
            .get_pending_input_for_running_task(&session, &turn.sub_id)
            .await
            .0,
        during_turn
    );
    session.abort_all_tasks(TurnAbortReason::Replaced).await;
    assert_eq!(
        session.input_queue.drain_mailbox_input_items().await.0,
        after_boundary
    );
    assert_eq!(
        session.input_queue.drain_mailbox_input_items().await.0,
        Vec::new()
    );
}

#[test_case::test_case(SecurityLevel::Moderate; "moderate")]
#[test_case::test_case(SecurityLevel::Aggressive; "aggressive")]
#[tokio::test]
async fn preferred_mxc_with_elevated_level_keeps_file_refusal(level: SecurityLevel) {
    let (_session, mut turn) = make_session_and_context().await;
    let config = Arc::make_mut(&mut turn.config);
    config.security_level = level;
    config.prefer_mxc = true;
    config.permissions.windows_sandbox_mode = Some(WindowsSandboxModeToml::Elevated);
    let backend = config.effective_local_windows_sandbox_type();
    turn.windows_sandbox_level = WindowsSandboxLevel::Elevated;
    // A remote Windows path exercises backend selection on every build host without
    // running a Windows executable or connecting to an executor.
    let environment =
        codex_exec_server::Environment::create_for_tests(Some("ws://127.0.0.1:1".to_string()))
            .expect("remote environment");
    let mut environment = TurnEnvironment::for_tests(
        "windows-fixture".to_string(),
        Arc::new(environment),
        PathUri::parse("file:///C:/work").expect("Windows cwd"),
        Vec::new(),
        /*shell*/ None,
    );
    environment.config_mut().windows_sandbox_type = backend;
    environment.config_mut().windows_sandbox_level = WindowsSandboxLevel::Elevated;
    let context =
        turn.file_system_sandbox_context(/*additional_permissions*/ None, &environment);
    assert_eq!(
        context.windows_sandbox_selection,
        codex_file_system::WindowsSandboxSelection::Mxc
    );
    assert_eq!(
        context.unsandboxed_refusal,
        Some(if cfg!(windows) {
            crate::security::windows_unsandboxed::file_refusal(level)
        } else {
            crate::security::windows_unsandboxed::remote_file_refusal(level, "windows-fixture")
        })
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn leftover_user_image_uses_follow_up_models_image_rules() {
    let home = tempfile::tempdir().expect("home");
    let cwd = kept_test_cwd();
    let (session, mut turn, events) = make_session_and_context_with_auth_config_home_and_rx(
        CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        home.path(),
        &cwd,
        |config| {
            config
                .features
                .enable(Feature::UnifiedImageBudget)
                .expect("image budget");
            // The helper's initial turn reads config.model_catalog, but subsequent turns
            // resolve through its provider manager's bundled catalog. Use the same real
            // non-Lite card in both, rather than mutating only the initial-turn fixture.
            let catalog = codex_models_manager::bundled_models_response().expect("catalog");
            let model = catalog
                .models
                .iter()
                .find(|model| model.slug == "gpt-5.5")
                .expect("bundled original-detail model");
            assert!(!model.use_responses_lite);
            assert!(model.supports_image_detail_original);
            config.model = Some(model.slug.clone());
            config.model_catalog = Some(catalog);
        },
        /*start_submission_loop*/ true,
    )
    .await;
    update_turn_settings_for_test(
        Arc::get_mut(&mut turn).expect("unshared turn"),
        |settings| {
            Arc::make_mut(&mut settings.model_info).supports_image_detail_original = false;
        },
    );
    let next = session.new_default_turn().await;
    assert!(!next.model_info().use_responses_lite);
    assert!(next.model_info().supports_image_detail_original);
    assert!(!turn.model_info().supports_image_detail_original);
    session
        .spawn_task(
            Arc::clone(&turn),
            Vec::new(),
            NeverEndingTask {
                kind: TaskKind::Regular,
                listen_to_cancellation_token: true,
            },
        )
        .await;
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgba8(/*w*/ 8, /*h*/ 8)
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("PNG");
    let image = ImageReference::Inline {
        image_url: codex_utils_image::data_url_from_bytes("image/png", &png.into_inner()),
    };
    let input = vec![UserInput::Image {
        image: image.clone(),
        detail: Some(ImageDetail::High),
    }];
    assert!(matches!(
        submit_steer_only(&session, input.clone(), &turn.sub_id).await,
        TurnInputSubmission::Steered { .. }
    ));
    session
        .on_task_finished(Arc::clone(&turn), /*task_result*/ Ok(None))
        .await;
    expect_user_message_item_started(&events, &input).await;
    let expected = ContentItem::InputImage {
        image,
        detail: Some(ImageDetail::Original),
    };
    let history = session.clone_history().await;
    assert!(
        raw_history_items(&history).iter().any(|item| {
            matches!(item, ResponseItem::Message { role, content, .. }
            if role == "user" && content.contains(&expected))
        }),
        "leftover input must use the new turn's image rules, not the old turn's metadata"
    );
    session.abort_all_tasks(TurnAbortReason::Interrupted).await;
}

#[tokio::test]
async fn active_profile_update_keeps_security_proxy_features() -> std::io::Result<()> {
    let codex_home = tempfile::tempdir().expect("create codex home");
    let cwd = tempfile::tempdir().expect("create cwd");
    let web_profile = |port: u16| PermissionProfileToml {
        description: None,
        extends: None,
        workspace_roots: None,
        filesystem: Some(FilesystemPermissionsToml {
            glob_scan_max_depth: None,
            entries: std::collections::BTreeMap::from([(
                ":minimal".to_string(),
                FilesystemPermissionToml::Access(FileSystemAccessMode::Read),
            )]),
        }),
        network: Some(NetworkToml {
            enabled: Some(true),
            proxy_url: Some(format!("http://127.0.0.1:{port}")),
            enable_socks5: Some(false),
            ..Default::default()
        }),
    };
    let base_config = ConfigToml {
        features: Some(
            toml::from_str(
                "network_proxy = true\nurl_destination_policy = true\nsecret_output_gate = true",
            )
            .expect("valid features"),
        ),
        default_permissions: Some("web-a".to_string()),
        permissions: Some(PermissionsToml {
            entries: std::collections::BTreeMap::from([
                ("web-a".to_string(), web_profile(43128)),
                ("web-b".to_string(), web_profile(43129)),
            ]),
        }),
        ..Default::default()
    };
    std::fs::write(
        codex_home.path().join(codex_config::CONFIG_TOML_FILE),
        toml::to_string(&base_config).expect("serialize config"),
    )?;
    let build = |profile: &str| {
        ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .harness_overrides(ConfigOverrides {
                cwd: Some(cwd.path().to_path_buf()),
                default_permissions: Some(profile.to_string()),
                ..Default::default()
            })
            .build()
    };
    let initial_config = Arc::new(build("web-a").await?);
    let initial_network = initial_config
        .permissions
        .network
        .as_ref()
        .expect("initial profile should have a proxy");
    assert!(initial_network.url_destination_policy_enabled());
    assert!(initial_network.credential_response_gate_enabled());
    let selected_config = build("web-b").await?;

    let mut session_configuration = make_session_configuration_for_tests().await;
    session_configuration.permission_profile_state = initial_config
        .permissions
        .permission_profile_state()
        .clone();
    session_configuration.original_config_do_not_use = Arc::clone(&initial_config);

    let updated = session_configuration
        .apply(
            &SessionSettingsUpdate {
                permission_profile: Some(selected_config.permissions.permission_profile().clone()),
                active_permission_profile: selected_config.permissions.active_permission_profile(),
                ..Default::default()
            },
            &[],
        )
        .expect("active profile update should apply");

    let network = updated
        .original_config_do_not_use
        .permissions
        .network
        .as_ref()
        .expect("selected profile proxy should become the session proxy config");
    assert_eq!(network.proxy_host_and_port(), "127.0.0.1:43129");
    assert!(
        network.url_destination_policy_enabled(),
        "url_destination_policy must survive a profile change"
    );
    assert!(
        network.credential_response_gate_enabled(),
        "secret_output_gate must survive a profile change"
    );
    Ok(())
}
