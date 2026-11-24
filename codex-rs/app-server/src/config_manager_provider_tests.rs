//! Checks retained providers against managed policy and the current provider selection.

use super::*;
use anyhow::Result;
use codex_config::CloudConfigBundleLoadError;
use codex_config::CloudConfigBundleLoadErrorCode;
use codex_config::ThreadConfigContext;
use codex_config::ThreadConfigLoadError;
use codex_config::ThreadConfigLoadErrorCode;
use codex_config::ThreadConfigLoaderFuture;
use codex_config::ThreadConfigSource;
use codex_config::test_support::CloudConfigBundleFixture;
use pretty_assertions::assert_eq;
use tempfile::tempdir;
use test_case::test_case;

#[test_case(AMAZON_BEDROCK_PROVIDER_ID, "aws.region = 'us-west-2'", "us-west-2", "us-east-1", None; "bedrock")]
#[test_case(AMAZON_BEDROCK_RUNTIME_PROVIDER_ID, "aws.region = 'us-west-2'", "us-west-2", "us-east-1", None; "bedrock_runtime")]
#[test_case(GCP_PROVIDER_ID, "project_id = 'example-project'\nlocation = 'us-central1'", "example-project", "other-project", None; "gcp_project")]
#[test_case(GCP_PROVIDER_ID, "project_id = 'example-project'\nlocation = 'us-central1'", "us-central1", "us-east4", None; "gcp_location")]
#[test_case(GCP_PROVIDER_ID, "project_id = 'example-project'\nauth.command = 'print-token'", "print-token", "other-token", None; "gcp_auth")]
#[test_case("openai", "model_catalog_url = 'https://chatgpt.com/backend-api/codex/models'", "https://chatgpt.com/backend-api/codex/models", "https://chatgpt.com/backend-api/codex/other-models", None; "openai_catalog")]
#[test_case("openai", "model_catalog_url = 'https://chatgpt.com/backend-api/codex/models'", "https://chatgpt.com/backend-api/codex/models", "https://chatgpt.com/backend-api/codex/other-models", Some("https://us.api.openai.com/v1"); "openai_regional_catalog")]
#[tokio::test]
async fn provider_requirements_resolve_builtin_overrides(
    provider_id: &str,
    provider_config: &str,
    original: &str,
    replacement: &str,
    openai_base_url: Option<&str>,
) -> Result<()> {
    let home = tempdir()?;
    let workspace = tempdir()?;
    if let Some(base_url) = openai_base_url {
        std::fs::write(
            home.path().join("config.toml"),
            format!("openai_base_url = '{base_url}'"),
        )?;
    }
    let requirements = format!(
        "model_provider = '{provider_id}'\n[model_providers.{provider_id}]\n{provider_config}"
    );
    let manager = ConfigManager::new_for_tests(
        home.path().to_path_buf(),
        Vec::new(),
        LoaderOverrides::without_managed_config_for_tests(),
        CloudConfigBundleFixture::loader_with_enterprise_requirement(&requirements),
    );
    let current = manager
        .load_latest_config(Some(workspace.path().to_path_buf()))
        .await?;
    if provider_id == "openai" {
        assert_eq!(current.model_provider.base_url.as_deref(), openai_base_url);
        // Retained threads keep their inference route even if user defaults later change.
        std::fs::write(
            home.path().join("config.toml"),
            "openai_base_url = 'https://eu.api.openai.com/v1'",
        )?;
    }
    manager.check_thread_model_provider(&current).await?;

    let changed = ConfigManager::new_for_tests(
        home.path().to_path_buf(),
        Vec::new(),
        LoaderOverrides::without_managed_config_for_tests(),
        CloudConfigBundleFixture::loader_with_enterprise_requirement(
            requirements.replace(original, replacement),
        ),
    );
    assert_eq!(
        changed
            .check_thread_model_provider(&current)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied,
    );
    Ok(())
}

struct UnavailableThreadConfig;

impl ThreadConfigLoader for UnavailableThreadConfig {
    fn load(
        &self,
        _context: ThreadConfigContext,
    ) -> ThreadConfigLoaderFuture<'_, Vec<ThreadConfigSource>> {
        Box::pin(async {
            Err(ThreadConfigLoadError::new(
                ThreadConfigLoadErrorCode::RequestFailed,
                /*status_code*/ None,
                "thread config service unavailable",
            ))
        })
    }
}

#[tokio::test]
async fn provider_requirements_do_not_reload_thread_config() -> Result<()> {
    let home = tempdir()?;
    let mut manager = ConfigManager::without_managed_config_for_tests(home.path().to_path_buf());
    manager.thread_config_loader = Arc::new(codex_config::StaticThreadConfigLoader::new(vec![
        ThreadConfigSource::Session(codex_config::SessionThreadConfig {
            model_provider: Some("retained".into()),
            model_providers: toml::from_str("[retained]\nname = 'Retained'")?,
            ..Default::default()
        }),
    ]));
    let current = manager.load_latest_config(/*fallback_cwd*/ None).await?;
    manager.thread_config_loader = Arc::new(UnavailableThreadConfig);
    assert!(
        manager
            .load_latest_config(/*fallback_cwd*/ None)
            .await
            .is_err()
    );
    manager.check_thread_model_provider(&current).await?;
    manager.cloud_config_bundle = Arc::new(RwLock::new(
        CloudConfigBundleFixture::loader_with_enterprise_requirement("model_provider = 'retained'"),
    ));
    let retained = manager
        .load_retained_session_config(&current.config_layer_stack, &current.cwd)
        .await?;
    assert_eq!(retained.model_provider, current.model_provider);
    let factory = current.http_client_factory();
    assert_eq!(retained.http_client_factory(), factory);

    manager.cloud_config_bundle = Arc::new(RwLock::new(
        CloudConfigBundleFixture::loader_with_enterprise_requirement("model_provider = 'other'"),
    ));
    assert_eq!(
        manager
            .check_thread_model_provider(&current)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied,
    );
    Ok(())
}

#[tokio::test]
async fn provider_requirements_ignore_system_defaults_but_reject_requirement_changes() -> Result<()>
{
    let home = tempdir()?;
    let system_config_path = home.path().join("system-config.toml");
    let requirements_path = home.path().join("requirements.toml");
    let managed_config_path = home.path().join("managed-config.toml");
    let mut overrides = LoaderOverrides::without_managed_config_for_tests();
    overrides.system_config_path = Some(system_config_path.clone());
    overrides.system_requirements_path = Some(requirements_path.clone());
    overrides.managed_config_path = Some(managed_config_path.clone());
    let manager = ConfigManager::new_for_tests(
        home.path().to_path_buf(),
        Vec::new(),
        overrides,
        CloudConfigBundleLoader::default(),
    );
    let current = manager.load_latest_config(/*fallback_cwd*/ None).await?;

    std::fs::write(&system_config_path, "invalid toml !!!")?;
    manager.check_thread_model_provider(&current).await?;

    std::fs::write(&requirements_path, "model_provider = 'other'")?;
    assert_eq!(
        manager
            .check_thread_model_provider(&current)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied,
    );

    std::fs::write(&requirements_path, "invalid toml !!!")?;
    assert_eq!(
        manager
            .check_thread_model_provider(&current)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData,
    );

    std::fs::write(
        &requirements_path,
        "[model_providers.gateway]\nbase_url = 'https://example.test'",
    )?;
    assert_eq!(
        manager
            .check_thread_model_provider(&current)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData,
    );
    std::fs::remove_file(requirements_path)?;

    std::fs::write(managed_config_path, "invalid toml !!!")?;
    assert_eq!(
        manager
            .check_thread_model_provider(&current)
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData,
    );
    Ok(())
}

#[tokio::test]
async fn provider_requirement_load_errors_reject_input() -> Result<()> {
    let home = tempdir()?;
    let previous = ConfigManager::without_managed_config_for_tests(home.path().to_path_buf())
        .load_latest_config(/*fallback_cwd*/ None)
        .await?;
    for loader in [
        CloudConfigBundleFixture::loader_with_enterprise_requirement("invalid toml !!!"),
        CloudConfigBundleLoader::new(async {
            Err(CloudConfigBundleLoadError::new(
                CloudConfigBundleLoadErrorCode::RequestFailed,
                /*status_code*/ None,
                "policy unavailable",
            ))
        }),
    ] {
        let manager = ConfigManager::new_for_tests(
            home.path().to_path_buf(),
            Vec::new(),
            LoaderOverrides::without_managed_config_for_tests(),
            loader,
        );
        assert!(
            manager
                .check_thread_model_provider(&previous)
                .await
                .is_err()
        );
    }
    Ok(())
}

/// A managed startup selection must disappear when its policy is removed.
#[tokio::test]
async fn catalog_reload_uses_latest_model_provider() -> Result<()> {
    let home = tempdir()?;
    let manager = ConfigManager::new_for_tests(
        home.path().to_path_buf(),
        Vec::new(),
        LoaderOverrides::without_managed_config_for_tests(),
        CloudConfigBundleFixture::loader_with_enterprise_requirement(
            "model_provider = 'gateway'\n[model_providers.gateway]\nname = 'Gateway'\nbase_url = 'https://example.test'",
        ),
    );
    let startup = manager.load_non_project_config().await?;
    assert_eq!(startup.model_provider_id, "gateway");
    manager.clear_chatgpt_config_loader();
    let refreshed = manager
        .load_startup_config(/*fallback_cwd*/ None)
        .await?
        .config;
    assert_eq!(refreshed.model_provider_id, "openai");
    Ok(())
}

/// Invocation overrides and selected profiles still define the current provider.
#[test_case("model_provider", "gateway"; "explicit_provider")]
#[test_case("profile", "gateway-profile"; "selected_profile")]
#[tokio::test]
async fn input_provider_check_uses_effective_selection(key: &str, value: &str) -> Result<()> {
    let home = tempdir()?;
    std::fs::write(
        home.path().join("config.toml"),
        r#"
model_provider = "openai"
[model_providers.gateway]
name = "Gateway"
"#,
    )?;
    let mut loader_overrides = LoaderOverrides::without_managed_config_for_tests();
    let cli_overrides = if key == "profile" {
        std::fs::write(
            home.path().join(format!("{value}.config.toml")),
            "model_provider = 'gateway'\n[model_providers.gateway]\nname = 'Gateway'\n",
        )?;
        loader_overrides.user_config_path = Some(AbsolutePathBuf::from_absolute_path(
            home.path().join(format!("{value}.config.toml")),
        )?);
        loader_overrides.user_config_profile = Some(value.parse()?);
        Vec::new()
    } else {
        vec![(key.into(), TomlValue::String(value.into()))]
    };
    let manager = ConfigManager::new_for_tests(
        home.path().to_path_buf(),
        cli_overrides,
        loader_overrides,
        CloudConfigBundleLoader::default(),
    );
    let current = manager
        .load_latest_config(Some(home.path().to_path_buf()))
        .await?;
    assert_eq!(current.model_provider_id, "gateway");
    manager
        .check_thread_model_provider_for_input(&current)
        .await?;
    // D738 (#555/PF55): a thread keeps its configured, active provider after
    // the saved default moves elsewhere.
    let default_manager =
        ConfigManager::without_managed_config_for_tests(home.path().to_path_buf());
    default_manager
        .check_thread_model_provider_for_input(&current)
        .await?;
    Ok(())
}

const RETAINED_PROVIDER_CONFIG: &str = r#"
model_provider = "openai"
[model_providers.gateway]
name = "Gateway"
"#;

/// Loads a thread config on `gateway` while the home's saved default is OpenAI.
async fn retained_gateway_thread(home: &Path) -> Result<(ConfigManager, Config)> {
    std::fs::write(home.join("config.toml"), RETAINED_PROVIDER_CONFIG)?;
    let thread_manager = ConfigManager::new_for_tests(
        home.to_path_buf(),
        vec![("model_provider".into(), TomlValue::String("gateway".into()))],
        LoaderOverrides::without_managed_config_for_tests(),
        CloudConfigBundleLoader::default(),
    );
    let current = thread_manager
        .load_latest_config(Some(home.to_path_buf()))
        .await?;
    assert_eq!(current.model_provider_id, "gateway");
    Ok((
        ConfigManager::without_managed_config_for_tests(home.to_path_buf()),
        current,
    ))
}

fn deactivate_provider(home: &Path, config: &Config, provider: &str) -> Result<()> {
    let catalog =
        codex_provider_auth::ProviderCatalog::from_runtime_providers(&config.model_providers);
    let entry = catalog
        .entries()
        .iter()
        .find(|entry| {
            entry
                .runtime_provider_ids
                .iter()
                .any(|id| id.as_str() == provider)
        })
        .expect("provider catalog entry");
    let store = codex_provider_auth::ProviderEligibilityStore::new(home);
    let mut eligibility = store.load()?;
    eligibility.set_policy(
        entry,
        codex_provider_auth::ProviderActivationPolicy::Inactive,
    );
    store.save(&eligibility)?;
    Ok(())
}

fn provider_error(error: &std::io::Error) -> Option<&ModelProviderError> {
    error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ModelProviderError>())
}

/// D738 keeps upstream's read-only chat when this home deactivated the
/// thread's provider.
#[tokio::test]
async fn input_provider_check_rejects_inactive_retained_provider() -> Result<()> {
    let home = tempdir()?;
    let (manager, current) = retained_gateway_thread(home.path()).await?;
    manager
        .check_thread_model_provider_for_input(&current)
        .await?;
    deactivate_provider(home.path(), &current, "gateway")?;
    let error = manager
        .check_thread_model_provider_for_input(&current)
        .await
        .unwrap_err();
    assert_eq!(
        provider_error(&error),
        Some(&ModelProviderError::ModelProviderMismatch {
            model_provider: "gateway".into(),
            current_model_provider: "openai".into(),
        })
    );
    Ok(())
}

/// Activation state that cannot be read fails closed to a read-only chat.
#[tokio::test]
async fn input_provider_check_rejects_retained_provider_when_activation_is_unreadable() -> Result<()>
{
    let home = tempdir()?;
    let (manager, current) = retained_gateway_thread(home.path()).await?;
    std::fs::write(home.path().join("provider-eligibility.json"), "not json")?;
    let error = manager
        .check_thread_model_provider_for_input(&current)
        .await
        .unwrap_err();
    assert_eq!(
        provider_error(&error),
        Some(&ModelProviderError::ModelProviderMismatch {
            model_provider: "gateway".into(),
            current_model_provider: "openai".into(),
        })
    );
    Ok(())
}

/// A provider this home no longer configures is refused, as upstream does.
#[tokio::test]
async fn input_provider_check_rejects_unconfigured_retained_provider() -> Result<()> {
    let home = tempdir()?;
    let (manager, current) = retained_gateway_thread(home.path()).await?;
    std::fs::write(
        home.path().join("config.toml"),
        "model_provider = \"openai\"\n",
    )?;
    let error = manager
        .check_thread_model_provider_for_input(&current)
        .await
        .unwrap_err();
    assert_eq!(
        provider_error(&error),
        Some(&ModelProviderError::ModelProviderNotConfigured {
            model_provider: "gateway".into(),
        })
    );
    Ok(())
}

/// Resume admits the thread's own configured, active provider, and refuses it
/// once the provider is deactivated or removed.
#[tokio::test]
async fn resume_admits_only_configured_active_retained_provider() -> Result<()> {
    let home = tempdir()?;
    let (manager, current) = retained_gateway_thread(home.path()).await?;
    let resume = || {
        manager.load_for_resume(
            Some("gateway"),
            None,
            ConfigOverrides {
                model_provider: Some("gateway".into()),
                ..Default::default()
            },
            Some(home.path().to_path_buf()),
        )
    };
    assert_eq!(resume().await?.model_provider_id, "gateway");

    deactivate_provider(home.path(), &current, "gateway")?;
    let error = resume().await.unwrap_err();
    assert_eq!(
        provider_error(&error),
        Some(&ModelProviderError::ModelProviderMismatch {
            model_provider: "gateway".into(),
            current_model_provider: "openai".into(),
        })
    );

    std::fs::write(
        home.path().join("config.toml"),
        "model_provider = \"openai\"\n",
    )?;
    let error = resume().await.unwrap_err();
    assert_eq!(
        provider_error(&error),
        Some(&ModelProviderError::ModelProviderNotConfigured {
            model_provider: "gateway".into(),
        })
    );
    Ok(())
}

/// Resume admits only providers the home's own configuration defines: a
/// provider the resume request itself defines is still not configured here.
#[tokio::test]
async fn resume_does_not_admit_a_provider_defined_only_by_the_request() -> Result<()> {
    let home = tempdir()?;
    std::fs::write(
        home.path().join("config.toml"),
        "model_provider = \"openai\"\n",
    )?;
    let manager = ConfigManager::without_managed_config_for_tests(home.path().to_path_buf());
    let error = manager
        .load_for_resume(
            Some("gateway"),
            Some(HashMap::from([(
                "model_providers".to_string(),
                serde_json::json!({"gateway": {"name": "Gateway"}}),
            )])),
            ConfigOverrides {
                model_provider: Some("gateway".into()),
                ..Default::default()
            },
            Some(home.path().to_path_buf()),
        )
        .await
        .unwrap_err();
    assert_eq!(
        provider_error(&error),
        Some(&ModelProviderError::ModelProviderNotConfigured {
            model_provider: "gateway".into(),
        })
    );
    Ok(())
}

/// #2853: a model request's workspace-routing reload keeps the session's model
/// and provider after the saved default changes to a pair that does not resolve.
#[tokio::test]
async fn workspace_routing_reload_keeps_session_runtime_over_incompatible_saved_default()
-> Result<()> {
    let home = tempdir()?;
    let config_path = home.path().join("config.toml");
    std::fs::write(&config_path, "model_provider = 'openai'\n")?;
    let manager = ConfigManager::without_managed_config_for_tests(home.path().to_path_buf());
    let current = manager.load_latest_config(/*fallback_cwd*/ None).await?;
    std::fs::write(
        &config_path,
        "model = 'gpt-6-astra'\nmodel_provider = 'claude-plan'\n",
    )?;
    let unpinned = manager
        .load_retained_session_config(&current.config_layer_stack, &current.cwd)
        .await
        .expect_err("the saved default alone is an incompatible pair");
    assert!(
        unpinned.to_string().contains("is incompatible"),
        "{unpinned}"
    );

    // The session resolved `gpt-6-astra` on `openai`; the reload keeps both.
    let session = current.workspace_routing_session(Some("gpt-6-astra"));
    let retained = manager
        .load_retained_session_config(&session.config_layer_stack, &session.cwd)
        .await?;
    assert_eq!(
        (
            retained.model.as_deref(),
            retained.model_provider_id.as_str()
        ),
        (Some("gpt-6-astra"), "openai")
    );
    Ok(())
}

/// #2853: with no known model nothing is pinned, so a valid saved default on
/// another provider still reloads; the provider alone would not pair with it.
#[tokio::test]
async fn workspace_routing_reload_without_a_model_pins_nothing() -> Result<()> {
    let home = tempdir()?;
    let config_path = home.path().join("config.toml");
    std::fs::write(&config_path, "model_provider = 'openai'\n")?;
    let manager = ConfigManager::without_managed_config_for_tests(home.path().to_path_buf());
    let current = manager.load_latest_config(/*fallback_cwd*/ None).await?;
    std::fs::write(
        &config_path,
        format!(
            "model = '{}'\nmodel_provider = 'claude-plan'\n",
            codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL
        ),
    )?;
    let session = current.workspace_routing_session(/*model*/ None);
    let retained = manager
        .load_retained_session_config(&session.config_layer_stack, &session.cwd)
        .await?;
    assert_eq!(retained.model_provider_id, "claude-plan");
    Ok(())
}
