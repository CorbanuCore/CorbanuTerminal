use super::*;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn resume_defaults_remember_acknowledged_selection_and_standard_tier() -> Result<()> {
    let home = tempfile::tempdir()?;
    std::fs::write(
        home.path().join("config.toml"),
        "model = \"gpt-5.4\"\nservice_tier = \"fast\"\ncli_auth_credentials_store = \"file\"\n",
    )?;
    let config = ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .loader_overrides(LoaderOverrides::without_managed_config_for_tests())
        .build()
        .await?;
    let server = crate::start_embedded_app_server_for_picker(&config).await?;
    let mut session =
        super::super::tests::test_thread_session(ThreadId::new(), home.path().to_path_buf());
    session.model = "claude-fable-5-1-plan".to_string();
    session.model_provider_id = "claude-plan".to_string();
    session.reasoning_effort = Some(ReasoningEffort::High);
    remember_resumed_model(
        &server,
        &config,
        ResumeModelSettings::RestoreFromThread,
        &session,
    )
    .await?;
    let saved: toml::Value =
        toml::from_str(&std::fs::read_to_string(home.path().join("config.toml"))?)?;
    assert_eq!(
        (
            saved["model"].as_str(),
            saved["model_provider"].as_str(),
            saved["model_reasoning_effort"].as_str(),
            saved["service_tier"].as_str()
        ),
        (
            Some("claude-fable-5-1-plan"),
            Some("claude-plan"),
            Some("high"),
            Some("default")
        ),
    );
    let before = std::fs::read(home.path().join("config.toml"))?;
    session.model = "one-off-model".to_string();
    remember_resumed_model(
        &server,
        &config,
        ResumeModelSettings::OverrideFromCurrentConfig,
        &session,
    )
    .await?;
    assert_eq!(std::fs::read(home.path().join("config.toml"))?, before);
    let mut tier_override_config = config.clone();
    tier_override_config.config_layer_stack = codex_config::ConfigLayerStack::new(
        vec![codex_config::ConfigLayerEntry::new(
            codex_config::ConfigLayerSource::SessionFlags,
            toml::from_str("service_tier = 'flex'")?,
        )],
        Default::default(),
        Default::default(),
    )?;
    session.service_tier = Some("flex".to_string());
    remember_resumed_model(
        &server,
        &tier_override_config,
        ResumeModelSettings::RestoreFromThread,
        &session,
    )
    .await?;
    assert_eq!(std::fs::read(home.path().join("config.toml"))?, before);
    session.model_provider_id = "unknown-provider".to_string();
    remember_resumed_model(
        &server,
        &config,
        ResumeModelSettings::RestoreFromThread,
        &session,
    )
    .await?;
    assert_eq!(std::fs::read(home.path().join("config.toml"))?, before);
    server.shutdown().await?;
    Ok(())
}
