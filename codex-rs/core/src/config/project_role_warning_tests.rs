use crate::config::AgentRoleSource;
use crate::config::ConfigBuilder;
use crate::config::ConfigOverrides;
use codex_agent_roles::unapplied_role_keys_warning;
use codex_config::CONFIG_TOML_FILE;
use tempfile::TempDir;

#[tokio::test]
async fn project_role_skill_settings_are_reported_in_startup_warnings() -> std::io::Result<()> {
    let codex_home = TempDir::new()?;
    let repo_root = TempDir::new()?;
    std::fs::create_dir_all(repo_root.path().join(".git"))?;
    std::fs::write(repo_root.path().join(".git/HEAD"), "ref: refs/heads/main\n")?;
    let project_config_dir = repo_root.path().join(".codex");
    std::fs::create_dir_all(project_config_dir.join("agents"))?;

    let workspace_key = repo_root.path().to_string_lossy().replace('\\', "\\\\");
    std::fs::write(
        codex_home.path().join(CONFIG_TOML_FILE),
        format!(
            r#"[projects."{workspace_key}"]
trust_level = "trusted"
[agents.user]
description = "User role"
config_file = "user-role.toml"
"#
        ),
    )?;
    std::fs::write(
        project_config_dir.join(CONFIG_TOML_FILE),
        r#"[agents.declared]
description = "Declared project role"
config_file = "declared.toml"
"#,
    )?;
    let role_settings = r#"developer_instructions = "Stay focused"
[skills]
include_instructions = false
[skills.bundled]
enabled = false
[[skills.config]]
name = "demo-skill"
enabled = false
"#;
    std::fs::write(codex_home.path().join("user-role.toml"), role_settings)?;
    std::fs::write(project_config_dir.join("declared.toml"), role_settings)?;
    std::fs::write(
        project_config_dir.join("agents/discovered.toml"),
        format!(
            "name = \"discovered\"\ndescription = \"Discovered project role\"\n{role_settings}"
        ),
    )?;

    let config = ConfigBuilder::without_managed_config_for_tests()
        .codex_home(codex_home.path().to_path_buf())
        .harness_overrides(ConfigOverrides {
            cwd: Some(repo_root.path().to_path_buf()),
            ..Default::default()
        })
        .build()
        .await?;
    assert_eq!(config.agent_roles["user"].source, AgentRoleSource::User);
    let mut expected_warnings = Vec::new();
    for role_name in ["declared", "discovered"] {
        let role = &config.agent_roles[role_name];
        assert_eq!(role.source, AgentRoleSource::Project);
        expected_warnings.push(unapplied_role_keys_warning(
            role.config_file.as_deref().expect("role config file"),
            &["skills".to_string()],
        ));
    }
    expected_warnings.sort();
    let mut skill_warnings = config
        .startup_warnings
        .iter()
        .filter(|warning| warning.contains("sets skills,"))
        .cloned()
        .collect::<Vec<_>>();
    skill_warnings.sort();
    assert_eq!(skill_warnings, expected_warnings);
    Ok(())
}
