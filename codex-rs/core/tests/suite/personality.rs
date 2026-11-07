use codex_core::TurnInputRequest;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::config_types::ModeKind;
use codex_protocol::config_types::Settings;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::ThreadSettingsOverrides;
use codex_protocol::user_input::UserInput;
<<<<<<< HEAD
use core_test_support::load_default_config_for_test_in;
use core_test_support::responses::mount_models_once;
||||||| 413492cd6c
use core_test_support::load_default_config_for_test;
use core_test_support::responses::mount_models_once;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse_completed;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::local_requests;
use core_test_support::test_codex::test_codex;
use core_test_support::test_codex::turn_permission_fields;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use test_case::test_case;

const CUSTOM_INSTRUCTIONS: &str = "Custom instructions\n# Personality\nThis must remain\n## Writing Style\nThis must also remain\n# General\nGeneral instructions";

fn read_only_text_turn(
    test: &TestCodex,
    text: &str,
    model: String,
    approval_policy: AskForApproval,
) -> TurnInputRequest {
    let (sandbox_policy, permission_profile) =
        turn_permission_fields(PermissionProfile::read_only(), test.cwd_path());
    TurnInputRequest::user_input(vec![UserInput::Text {
        text: text.into(),
        text_elements: Vec::new(),
    }])
    .with_thread_settings(ThreadSettingsOverrides {
        environments: Some(local_requests(test.config.cwd.clone())),
        approval_policy: Some(approval_policy),
        sandbox_policy: Some(sandbox_policy),
        permission_profile,
        collaboration_mode: Some(CollaborationMode {
            mode: ModeKind::Default,
            settings: Settings {
                model,
                reasoning_effort: test.config.model_reasoning_effort.clone(),
                developer_instructions: None,
            },
        }),
        ..Default::default()
    })
}

#[test_case(None, None; "default personality")]
#[test_case(Some("none"), None; "none without feature config")]
#[test_case(Some("none"), Some(false); "none with removed feature disabled")]
#[test_case(Some("none"), Some(true); "none with removed feature enabled")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
<<<<<<< HEAD
async fn personality_does_not_mutate_base_instructions_without_template() {
    let codex_home = TempDir::new().expect("create temp dir");
    let cwd = TempDir::new().expect("create temp dir");
    let mut config =
        load_default_config_for_test_in(&codex_home, Default::default(), cwd.path()).await;
    config
        .features
        .enable(Feature::Personality)
        .expect("test config should allow feature update");
    config.personality = Some(Personality::Friendly);

    let model_info = codex_core::test_support::construct_model_info_offline("gpt-5.4", &config);
    assert_eq!(
        model_info.get_model_instructions(config.personality),
        model_info.base_instructions
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn base_instructions_override_disables_personality_template() {
    let codex_home = TempDir::new().expect("create temp dir");
    let cwd = TempDir::new().expect("create temp dir");
    let mut config =
        load_default_config_for_test_in(&codex_home, Default::default(), cwd.path()).await;
    config
        .features
        .enable(Feature::Personality)
        .expect("test config should allow feature update");
    config.personality = Some(Personality::Friendly);
    config.base_instructions = Some("override instructions".to_string());

    let model_info = codex_core::test_support::construct_model_info_offline("gpt-5.4", &config);

    assert_eq!(model_info.base_instructions, "override instructions");
    assert_eq!(
        model_info.get_model_instructions(config.personality),
        "override instructions"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_none_does_not_add_update_message() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex().with_model("gpt-5.4").with_config(|config| {
        config
            .features
            .enable(Feature::Personality)
            .expect("test config should allow feature update");
    });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let developer_texts = request.message_input_texts("developer");
    assert!(
        !developer_texts
            .iter()
            .any(|text| text.contains("<personality_spec>")),
        "did not expect a personality update message when personality is None"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_some_sets_instructions_template() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex().with_model("gpt-5.4").with_config(|config| {
        config
            .features
            .enable(Feature::Personality)
            .expect("test config should allow feature update");
        config.personality = Some(Personality::Friendly);
    });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let instructions_text = request.instructions_text();

    assert!(
        instructions_text.contains(LOCAL_FRIENDLY_TEMPLATE),
        "expected personality update to include the local friendly template, got: {instructions_text:?}"
    );

    let developer_texts = request.message_input_texts("developer");
    for text in developer_texts {
        assert!(
            !text.contains("<personality_spec>"),
            "expected no personality update message in developer input"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_none_sends_no_personality() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex().with_model("gpt-5.4").with_config(|config| {
        config
            .features
            .enable(Feature::Personality)
            .expect("test config should allow feature update");
        config.personality = Some(Personality::None);
    });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let instructions_text = request.instructions_text();
    assert!(
        !instructions_text.contains(LOCAL_FRIENDLY_TEMPLATE),
        "expected no friendly personality template, got: {instructions_text:?}"
    );
    assert!(
        !instructions_text.contains(LOCAL_PRAGMATIC_TEMPLATE),
        "expected no pragmatic personality template, got: {instructions_text:?}"
    );
    assert!(
        !instructions_text.contains("{{ personality }}"),
        "expected personality placeholder to be removed, got: {instructions_text:?}"
    );

    let developer_texts = request.message_input_texts("developer");
    assert!(
        !developer_texts
            .iter()
            .any(|text| text.contains("<personality_spec>")),
        "did not expect a personality update message when personality is None"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_none_strips_baked_personality_section() -> anyhow::Result<()> {
||||||| 413492cd6c
async fn personality_does_not_mutate_base_instructions_without_template() {
    let codex_home = TempDir::new().expect("create temp dir");
    let mut config = load_default_config_for_test(&codex_home).await;
    config
        .features
        .enable(Feature::Personality)
        .expect("test config should allow feature update");
    config.personality = Some(Personality::Friendly);

    let model_info = codex_core::test_support::construct_model_info_offline("gpt-5.4", &config);
    assert_eq!(
        model_info.get_model_instructions(config.personality),
        model_info.base_instructions
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn base_instructions_override_disables_personality_template() {
    let codex_home = TempDir::new().expect("create temp dir");
    let mut config = load_default_config_for_test(&codex_home).await;
    config
        .features
        .enable(Feature::Personality)
        .expect("test config should allow feature update");
    config.personality = Some(Personality::Friendly);
    config.base_instructions = Some("override instructions".to_string());

    let model_info = codex_core::test_support::construct_model_info_offline("gpt-5.4", &config);

    assert_eq!(model_info.base_instructions, "override instructions");
    assert_eq!(
        model_info.get_model_instructions(config.personality),
        "override instructions"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_none_does_not_add_update_message() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex().with_model("gpt-5.4").with_config(|config| {
        config
            .features
            .enable(Feature::Personality)
            .expect("test config should allow feature update");
    });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let developer_texts = request.message_input_texts("developer");
    assert!(
        !developer_texts
            .iter()
            .any(|text| text.contains("<personality_spec>")),
        "did not expect a personality update message when personality is None"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_some_sets_instructions_template() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex().with_model("gpt-5.4").with_config(|config| {
        config
            .features
            .enable(Feature::Personality)
            .expect("test config should allow feature update");
        config.personality = Some(Personality::Friendly);
    });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let instructions_text = request.instructions_text();

    assert!(
        instructions_text.contains(LOCAL_FRIENDLY_TEMPLATE),
        "expected personality update to include the local friendly template, got: {instructions_text:?}"
    );

    let developer_texts = request.message_input_texts("developer");
    for text in developer_texts {
        assert!(
            !text.contains("<personality_spec>"),
            "expected no personality update message in developer input"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_none_sends_no_personality() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex().with_model("gpt-5.4").with_config(|config| {
        config
            .features
            .enable(Feature::Personality)
            .expect("test config should allow feature update");
        config.personality = Some(Personality::None);
    });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let instructions_text = request.instructions_text();
    assert!(
        !instructions_text.contains(LOCAL_FRIENDLY_TEMPLATE),
        "expected no friendly personality template, got: {instructions_text:?}"
    );
    assert!(
        !instructions_text.contains(LOCAL_PRAGMATIC_TEMPLATE),
        "expected no pragmatic personality template, got: {instructions_text:?}"
    );
    assert!(
        !instructions_text.contains("{{ personality }}"),
        "expected personality placeholder to be removed, got: {instructions_text:?}"
    );

    let developer_texts = request.message_input_texts("developer");
    assert!(
        !developer_texts
            .iter()
            .any(|text| text.contains("<personality_spec>")),
        "did not expect a personality update message when personality is None"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_none_strips_baked_personality_section() -> anyhow::Result<()> {
=======
async fn config_personality_controls_baked_personality_section(
    personality: Option<&'static str>,
    legacy_feature_setting: Option<bool>,
) -> anyhow::Result<()> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex()
        .with_model_info_override("gpt-5.5", |model_info| {
            if let Some(model_messages) = model_info.model_messages.as_mut() {
                model_messages.instructions_template = Some("Base instructions\n# Personality\nBaked personality\n## Writing Style\nNested writing style\n# General\nGeneral instructions".to_string());
            }
        })
        .with_pre_build_hook(move |home| {
            let mut config = String::new();
            if let Some(value) = personality {
                config.push_str(&format!("personality = \"{value}\"\n"));
            }
            if let Some(value) = legacy_feature_setting {
                config.push_str(&format!("[features]\npersonality = {value}\n"));
            }
            std::fs::write(home.join("config.toml"), config).expect("write personality config");
        });
    let test = builder.build_with_auto_env(&server).await?;

    test.codex
        .start_or_steer_turn(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let expected = if personality == Some("none") {
        "Base instructions\n# General\nGeneral instructions"
    } else {
        "Base instructions\n# Personality\nBaked personality\n## Writing Style\nNested writing style\n# General\nGeneral instructions"
    };
    assert_eq!(resp_mock.single_request().instructions_text(), expected);

    Ok(())
}

#[test_case(CUSTOM_INSTRUCTIONS, true; "custom instructions")]
#[test_case("", false; "bridge barebones")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_personality_none_preserves_explicit_base_instructions(
    custom_instructions: &'static str,
    legacy_feature_setting: bool,
) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;
    let mut builder = test_codex()
        .with_model_info_override("gpt-5.5", |model_info| {
            if let Some(model_messages) = model_info.model_messages.as_mut() {
                model_messages.instructions_template = Some("Base instructions\n# Personality\nBaked personality\n## Writing Style\nNested writing style\n# General\nGeneral instructions".to_string());
            }
        })
        .with_pre_build_hook(move |home| {
            let config = format!(
                "personality = \"none\"\n[features]\npersonality = {legacy_feature_setting}\n"
            );
            std::fs::write(home.join("config.toml"), config).expect("write personality config");
        })
        .with_config(move |config| {
            config.base_instructions = Some(custom_instructions.to_string());
        });
    let test = builder.build_with_auto_env(&server).await?;

    test.codex
        .start_or_steer_turn(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
<<<<<<< HEAD
    let instructions_text = request.instructions_text();
    assert!(
        instructions_text.contains(LOCAL_PRAGMATIC_TEMPLATE),
        "expected default friendly template, got: {instructions_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_some_adds_update_message() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let mut builder = test_codex()
        .with_model("exp-codex-personality")
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
        });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Friendly),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected personality update request");

    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains("<personality_spec>"))
        .expect("expected personality update message in developer input");

    assert!(
        personality_text.contains("The user has requested a new communication style."),
        "expected personality update preamble, got {personality_text:?}"
    );
    assert!(
        personality_text.contains(LOCAL_FRIENDLY_TEMPLATE),
        "expected personality update to include the local pragmatic template, got: {personality_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_same_value_does_not_add_update_message() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let mut builder = test_codex()
        .with_model("exp-codex-personality")
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
            config.personality = Some(Personality::Pragmatic);
        });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Pragmatic),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected second request after personality override");

    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains("<personality_spec>"));
    assert!(
        personality_text.is_none(),
        "expected no personality preamble for unchanged personality, got {personality_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn instructions_uses_base_if_feature_disabled() -> anyhow::Result<()> {
    let codex_home = TempDir::new().expect("create temp dir");
    let cwd = TempDir::new().expect("create temp dir");
    let mut config =
        load_default_config_for_test_in(&codex_home, Default::default(), cwd.path()).await;
    config
        .features
        .disable(Feature::Personality)
        .expect("test config should allow feature update");
    config.personality = Some(Personality::Friendly);

    let model_info = codex_core::test_support::construct_model_info_offline("gpt-5.4", &config);
    assert_eq!(
        model_info.get_model_instructions(config.personality),
        model_info.base_instructions
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_skips_if_feature_disabled() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let mut builder = test_codex()
        .with_model("exp-codex-personality")
        .with_config(|config| {
            config
                .features
                .disable(Feature::Personality)
                .expect("test config should allow feature update");
        });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Pragmatic),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected personality update request");

    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains("<personality_spec>"));
    assert!(
        personality_text.is_none(),
        "expected no personality preamble, got {personality_text:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_model_friendly_personality_instructions_with_feature() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = MockServer::builder()
        .body_print_limit(BodyPrintLimit::Limited(80_000))
        .start()
        .await;

    let remote_slug = "codex-remote-default-personality";
    let default_personality_message = "Default from remote template";
    let friendly_personality_message = "Friendly variant";
    let remote_model = ModelInfo {
        slug: remote_slug.to_string(),
        orchestration: None,
        chat_completions: Default::default(),
        display_name: "Remote default personality test".to_string(),
        description: Some("Remote model with default personality template".to_string()),
        default_reasoning_level: Some(ReasoningEffort::Medium),
        supported_reasoning_levels: vec![ReasoningEffortPreset {
            effort: ReasoningEffort::Medium,
            description: ReasoningEffort::Medium.to_string(),
        }],
        shell_type: ConfigShellToolType::UnifiedExec,
        visibility: ModelVisibility::List,
        supported_in_api: true,
        priority: 1,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        upgrade: None,
        base_instructions: "base instructions".to_string(),
        model_messages: Some(ModelMessages {
            instructions_template: Some("Base instructions\n{{ personality }}\n".to_string()),
            instructions_variables: Some(ModelInstructionsVariables {
                personality_default: Some(default_personality_message.to_string()),
                personality_friendly: Some(friendly_personality_message.to_string()),
                personality_pragmatic: Some("Pragmatic variant".to_string()),
            }),
            approvals: None,
            auto_review: None,
            permissions: None,
            token_budget: None,
        }),
        include_skills_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        availability_nux: None,
        apply_patch_tool_type: None,
        web_search_tool_type: Default::default(),
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_parallel_tool_calls: false,
        supports_image_detail_original: false,
        context_window: Some(128_000),
        max_context_window: None,
        max_output_tokens: None,
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: false,
        supports_search_tool: false,
        use_responses_lite: false,
        auto_review_model_override: None,
        tool_mode: None,
        multi_agent_version: None,
    };

    let _models_mock = mount_models_once(
        &server,
        ModelsResponse {
            models: vec![remote_model],
        },
    )
    .await;

    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;

    let mut builder = test_codex()
        .with_auth(codex_login::CodexAuth::create_dummy_chatgpt_auth_for_testing())
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
            config.model = Some(remote_slug.to_string());
            config.personality = Some(Personality::Friendly);
        });
    let test = builder.build(&server).await?;

    wait_for_model_available(&test.thread_manager.get_models_manager(), remote_slug).await;

    test.codex
        .submit(read_only_text_turn_with_personality(
            &test,
            "hello",
            remote_slug.to_string(),
            AskForApproval::Never,
            Some(Personality::Friendly),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let instructions_text = request.instructions_text();

    assert!(
        instructions_text.contains(friendly_personality_message),
        "expected instructions to include the remote friendly personality template, got: {instructions_text:?}"
    );
    assert!(
        !instructions_text.contains(default_personality_message),
        "expected instructions to skip the remote default personality template, got: {instructions_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_remote_model_template_includes_update_message() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = MockServer::builder()
        .body_print_limit(BodyPrintLimit::Limited(80_000))
        .start()
        .await;

    let remote_slug = "codex-remote-personality";
    let remote_friendly_message = "Friendly from remote template";
    let remote_pragmatic_message = "Pragmatic from remote template";
    let remote_model = ModelInfo {
        slug: remote_slug.to_string(),
        orchestration: None,
        chat_completions: Default::default(),
        display_name: "Remote personality test".to_string(),
        description: Some("Remote model with personality template".to_string()),
        default_reasoning_level: Some(ReasoningEffort::Medium),
        supported_reasoning_levels: vec![ReasoningEffortPreset {
            effort: ReasoningEffort::Medium,
            description: ReasoningEffort::Medium.to_string(),
        }],
        shell_type: ConfigShellToolType::UnifiedExec,
        visibility: ModelVisibility::List,
        supported_in_api: true,
        priority: 1,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        upgrade: None,
        base_instructions: "base instructions".to_string(),
        model_messages: Some(ModelMessages {
            instructions_template: Some("Base instructions\n{{ personality }}\n".to_string()),
            instructions_variables: Some(ModelInstructionsVariables {
                personality_default: None,
                personality_friendly: Some(remote_friendly_message.to_string()),
                personality_pragmatic: Some(remote_pragmatic_message.to_string()),
            }),
            approvals: None,
            auto_review: None,
            permissions: None,
            token_budget: None,
        }),
        include_skills_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        availability_nux: None,
        apply_patch_tool_type: None,
        web_search_tool_type: Default::default(),
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_parallel_tool_calls: false,
        supports_image_detail_original: false,
        context_window: Some(128_000),
        max_context_window: None,
        max_output_tokens: None,
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: false,
        supports_search_tool: false,
        use_responses_lite: false,
        auto_review_model_override: None,
        tool_mode: None,
        multi_agent_version: None,
    };

    let _models_mock = mount_models_once(
        &server,
        ModelsResponse {
            models: vec![remote_model],
        },
    )
    .await;

    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;

    let mut builder = test_codex()
        .with_auth(codex_login::CodexAuth::create_dummy_chatgpt_auth_for_testing())
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
            config.model = Some("gpt-5.4".to_string());
        });
    let test = builder.build(&server).await?;

    wait_for_model_available(&test.thread_manager.get_models_manager(), remote_slug).await;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            remote_slug.to_string(),
            AskForApproval::Never,
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Friendly),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            remote_slug.to_string(),
            AskForApproval::Never,
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected personality update request");
    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains(remote_friendly_message))
        .expect("expected personality update message in developer input");

    assert!(
        personality_text.contains("The user has requested a new communication style."),
        "expected personality update preamble, got {personality_text:?}"
    );
    assert!(
        personality_text.contains(remote_friendly_message),
        "expected personality update to include remote template, got: {personality_text:?}"
    );

    Ok(())
}

async fn wait_for_model_available(manager: &SharedModelsManager, slug: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let models = manager
            .list_models(
                RefreshStrategy::OnlineIfUncached,
                codex_core::test_support::default_http_client_factory(),
            )
            .await;
        if models.iter().any(|model| model.model == slug) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for the remote model {slug} to appear");
        }
        sleep(Duration::from_millis(25)).await;
||||||| 413492cd6c
    let instructions_text = request.instructions_text();
    assert!(
        instructions_text.contains(LOCAL_PRAGMATIC_TEMPLATE),
        "expected default friendly template, got: {instructions_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_some_adds_update_message() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let mut builder = test_codex()
        .with_model("exp-codex-personality")
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
        });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Friendly),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected personality update request");

    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains("<personality_spec>"))
        .expect("expected personality update message in developer input");

    assert!(
        personality_text.contains("The user has requested a new communication style."),
        "expected personality update preamble, got {personality_text:?}"
    );
    assert!(
        personality_text.contains(LOCAL_FRIENDLY_TEMPLATE),
        "expected personality update to include the local pragmatic template, got: {personality_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_same_value_does_not_add_update_message() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let mut builder = test_codex()
        .with_model("exp-codex-personality")
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
            config.personality = Some(Personality::Pragmatic);
        });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Pragmatic),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected second request after personality override");

    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains("<personality_spec>"));
    assert!(
        personality_text.is_none(),
        "expected no personality preamble for unchanged personality, got {personality_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn instructions_uses_base_if_feature_disabled() -> anyhow::Result<()> {
    let codex_home = TempDir::new().expect("create temp dir");
    let mut config = load_default_config_for_test(&codex_home).await;
    config
        .features
        .disable(Feature::Personality)
        .expect("test config should allow feature update");
    config.personality = Some(Personality::Friendly);

    let model_info = codex_core::test_support::construct_model_info_offline("gpt-5.4", &config);
    assert_eq!(
        model_info.get_model_instructions(config.personality),
        model_info.base_instructions
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_skips_if_feature_disabled() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;
    let mut builder = test_codex()
        .with_model("exp-codex-personality")
        .with_config(|config| {
            config
                .features
                .disable(Feature::Personality)
                .expect("test config should allow feature update");
        });
    let test = builder.build(&server).await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Pragmatic),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            test.session_configured.model.clone(),
            test.config.permissions.approval_policy.value(),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected personality update request");

    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains("<personality_spec>"));
    assert!(
        personality_text.is_none(),
        "expected no personality preamble, got {personality_text:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn remote_model_friendly_personality_instructions_with_feature() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = MockServer::builder()
        .body_print_limit(BodyPrintLimit::Limited(80_000))
        .start()
        .await;

    let remote_slug = "codex-remote-default-personality";
    let default_personality_message = "Default from remote template";
    let friendly_personality_message = "Friendly variant";
    let remote_model = ModelInfo {
        slug: remote_slug.to_string(),
        display_name: "Remote default personality test".to_string(),
        description: Some("Remote model with default personality template".to_string()),
        default_reasoning_level: Some(ReasoningEffort::Medium),
        supported_reasoning_levels: vec![ReasoningEffortPreset {
            effort: ReasoningEffort::Medium,
            description: ReasoningEffort::Medium.to_string(),
        }],
        shell_type: ConfigShellToolType::UnifiedExec,
        visibility: ModelVisibility::List,
        supported_in_api: true,
        priority: 1,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        upgrade: None,
        base_instructions: "base instructions".to_string(),
        model_messages: Some(ModelMessages {
            instructions_template: Some("Base instructions\n{{ personality }}\n".to_string()),
            instructions_variables: Some(ModelInstructionsVariables {
                personality_default: Some(default_personality_message.to_string()),
                personality_friendly: Some(friendly_personality_message.to_string()),
                personality_pragmatic: Some("Pragmatic variant".to_string()),
            }),
            approvals: None,
            auto_review: None,
            permissions: None,
            token_budget: None,
        }),
        include_skills_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        availability_nux: None,
        apply_patch_tool_type: None,
        web_search_tool_type: Default::default(),
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_parallel_tool_calls: false,
        supports_image_detail_original: false,
        context_window: Some(128_000),
        max_context_window: None,
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: false,
        supports_search_tool: false,
        use_responses_lite: false,
        auto_review_model_override: None,
        tool_mode: None,
        multi_agent_version: None,
    };

    let _models_mock = mount_models_once(
        &server,
        ModelsResponse {
            models: vec![remote_model],
        },
    )
    .await;

    let resp_mock = mount_sse_once(&server, sse_completed("resp-1")).await;

    let mut builder = test_codex()
        .with_auth(codex_login::CodexAuth::create_dummy_chatgpt_auth_for_testing())
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
            config.model = Some(remote_slug.to_string());
            config.personality = Some(Personality::Friendly);
        });
    let test = builder.build(&server).await?;

    wait_for_model_available(&test.thread_manager.get_models_manager(), remote_slug).await;

    test.codex
        .submit(read_only_text_turn_with_personality(
            &test,
            "hello",
            remote_slug.to_string(),
            AskForApproval::Never,
            Some(Personality::Friendly),
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let request = resp_mock.single_request();
    let instructions_text = request.instructions_text();

    assert!(
        instructions_text.contains(friendly_personality_message),
        "expected instructions to include the remote friendly personality template, got: {instructions_text:?}"
    );
    assert!(
        !instructions_text.contains(default_personality_message),
        "expected instructions to skip the remote default personality template, got: {instructions_text:?}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_turn_personality_remote_model_template_includes_update_message() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = MockServer::builder()
        .body_print_limit(BodyPrintLimit::Limited(80_000))
        .start()
        .await;

    let remote_slug = "codex-remote-personality";
    let remote_friendly_message = "Friendly from remote template";
    let remote_pragmatic_message = "Pragmatic from remote template";
    let remote_model = ModelInfo {
        slug: remote_slug.to_string(),
        display_name: "Remote personality test".to_string(),
        description: Some("Remote model with personality template".to_string()),
        default_reasoning_level: Some(ReasoningEffort::Medium),
        supported_reasoning_levels: vec![ReasoningEffortPreset {
            effort: ReasoningEffort::Medium,
            description: ReasoningEffort::Medium.to_string(),
        }],
        shell_type: ConfigShellToolType::UnifiedExec,
        visibility: ModelVisibility::List,
        supported_in_api: true,
        priority: 1,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        upgrade: None,
        base_instructions: "base instructions".to_string(),
        model_messages: Some(ModelMessages {
            instructions_template: Some("Base instructions\n{{ personality }}\n".to_string()),
            instructions_variables: Some(ModelInstructionsVariables {
                personality_default: None,
                personality_friendly: Some(remote_friendly_message.to_string()),
                personality_pragmatic: Some(remote_pragmatic_message.to_string()),
            }),
            approvals: None,
            auto_review: None,
            permissions: None,
            token_budget: None,
        }),
        include_skills_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        availability_nux: None,
        apply_patch_tool_type: None,
        web_search_tool_type: Default::default(),
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_parallel_tool_calls: false,
        supports_image_detail_original: false,
        context_window: Some(128_000),
        max_context_window: None,
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: false,
        supports_search_tool: false,
        use_responses_lite: false,
        auto_review_model_override: None,
        tool_mode: None,
        multi_agent_version: None,
    };

    let _models_mock = mount_models_once(
        &server,
        ModelsResponse {
            models: vec![remote_model],
        },
    )
    .await;

    let resp_mock = mount_sse_sequence(
        &server,
        vec![sse_completed("resp-1"), sse_completed("resp-2")],
    )
    .await;

    let mut builder = test_codex()
        .with_auth(codex_login::CodexAuth::create_dummy_chatgpt_auth_for_testing())
        .with_config(|config| {
            config
                .features
                .enable(Feature::Personality)
                .expect("test config should allow feature update");
            config.model = Some("gpt-5.4".to_string());
        });
    let test = builder.build(&server).await?;

    wait_for_model_available(&test.thread_manager.get_models_manager(), remote_slug).await;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            remote_slug.to_string(),
            AskForApproval::Never,
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    core_test_support::submit_thread_settings(
        &test.codex,
        codex_protocol::protocol::ThreadSettingsOverrides {
            personality: Some(Personality::Friendly),
            ..Default::default()
        },
    )
    .await?;

    test.codex
        .submit(read_only_text_turn(
            &test,
            "hello",
            remote_slug.to_string(),
            AskForApproval::Never,
        ))
        .await?;

    wait_for_event(&test.codex, |ev| matches!(ev, EventMsg::TurnComplete(_))).await;

    let requests = resp_mock.requests();
    assert_eq!(requests.len(), 2, "expected two requests");
    let request = requests
        .last()
        .expect("expected personality update request");
    let developer_texts = request.message_input_texts("developer");
    let personality_text = developer_texts
        .iter()
        .find(|text| text.contains(remote_friendly_message))
        .expect("expected personality update message in developer input");

    assert!(
        personality_text.contains("The user has requested a new communication style."),
        "expected personality update preamble, got {personality_text:?}"
    );
    assert!(
        personality_text.contains(remote_friendly_message),
        "expected personality update to include remote template, got: {personality_text:?}"
    );

    Ok(())
}

async fn wait_for_model_available(manager: &SharedModelsManager, slug: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let models = manager
            .list_models(
                RefreshStrategy::OnlineIfUncached,
                codex_core::test_support::default_http_client_factory(),
            )
            .await;
        if models.iter().any(|model| model.model == slug) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("timed out waiting for the remote model {slug} to appear");
        }
        sleep(Duration::from_millis(25)).await;
=======
    let body = request.body_json();
    assert!(body.get("instructions").is_none());
    if !custom_instructions.is_empty() {
        assert_eq!(request.instructions_text(), custom_instructions);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
    assert!(
        request
            .message_input_texts("developer")
            .iter()
            .all(|text| !text.is_empty())
    );
    assert!(!request.body_contains_text("Base instructions"));

    Ok(())
}
