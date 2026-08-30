use std::sync::Arc;

use codex_arg0::Arg0DispatchPaths;
use codex_config::CloudConfigBundleLoader;
use codex_config::LoaderOverrides;
use codex_core::config::ConfigOverrides;
use codex_model_provider_info::CLAUDE_FABLE_5_PLAN_MODEL;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use tempfile::TempDir;

use super::ConfigManager;

#[tokio::test]
async fn runtime_executable_reaches_provider_auth_configuration() -> std::io::Result<()> {
    let codex_home = TempDir::new()?;
    let codex_self_exe = codex_home.path().join(if cfg!(windows) {
        "corbanu.exe"
    } else {
        "corbanu"
    });
    let manager = ConfigManager::new(
        codex_home.path().to_path_buf(),
        Vec::new(),
        LoaderOverrides::without_managed_config_for_tests(),
        /*strict_config*/ false,
        CloudConfigBundleLoader::default(),
        Arg0DispatchPaths {
            codex_self_exe: Some(codex_self_exe.clone()),
            ..Default::default()
        },
        Arc::new(codex_config::NoopThreadConfigLoader),
    );

    let config = manager
        .load_with_overrides(
            /*request_overrides*/ None,
            ConfigOverrides {
                model: Some(CLAUDE_FABLE_5_PLAN_MODEL.to_string()),
                model_provider: Some(CLAUDE_PLAN_PROVIDER_ID.to_string()),
                ..Default::default()
            },
        )
        .await?;

    assert_eq!(
        config
            .model_provider
            .auth
            .as_ref()
            .map(|auth| auth.command.as_str()),
        codex_self_exe.to_str()
    );
    Ok(())
}
