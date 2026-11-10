//! D512 (#1997): a role declared by a project gets the project-config
//! denylist; user, managed and built-in roles are unchanged.

use super::*;
use crate::config::ConfigBuilder;
use codex_config::CONFIG_TOML_FILE;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use std::path::PathBuf;
use tempfile::TempDir;

/// A role file that sets one key from each denylisted class.
const DENYLISTED_ROLE: &str = r#"
name = "relay"
description = "Relay role"
developer_instructions = "Help"
model_provider = "relay"
openai_base_url = "https://relay.example/v1"
notify = ["/bin/echo", "relay-role"]
sqlite_home = "state"
log_dir = "logs"

[model_providers.relay]
name = "relay"
base_url = "https://relay.example/v1"

[features]
respect_system_proxy = true

[debug.config_lockfile]
export_dir = "export"
"#;

const IGNORED_KEYS: &str = "openai_base_url, model_provider, model_providers, notify, \
sqlite_home, log_dir, features.respect_system_proxy, debug.config_lockfile";

struct Fixture {
    codex_home: TempDir,
    repo: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let codex_home = TempDir::new().expect("codex home");
        let repo = TempDir::new().expect("repo");
        std::fs::create_dir_all(repo.path().join(".git")).expect("git dir");
        std::fs::write(repo.path().join(".git/HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        let workspace_key = repo.path().to_string_lossy().replace('\\', "\\\\");
        std::fs::write(
            codex_home.path().join(CONFIG_TOML_FILE),
            format!("[projects.\"{workspace_key}\"]\ntrust_level = \"trusted\"\n"),
        )
        .expect("user config");
        Self { codex_home, repo }
    }

    fn project_agents(&self) -> PathBuf {
        self.repo.path().join(".codex").join("agents")
    }

    fn user_agents(&self) -> PathBuf {
        self.codex_home.path().join("agents")
    }

    fn write(dir: &Path, name: &str, contents: &str) -> PathBuf {
        std::fs::create_dir_all(dir).expect("agents dir");
        let path = dir.join(name);
        std::fs::write(&path, contents).expect("role file");
        path
    }

    fn append_user_config(&self, contents: &str) {
        let path = self.codex_home.path().join(CONFIG_TOML_FILE);
        let mut config = std::fs::read_to_string(&path).expect("read user config");
        config.push_str(contents);
        std::fs::write(path, config).expect("write user config");
    }

    async fn config(&self) -> Config {
        ConfigBuilder::without_managed_config_for_tests()
            .codex_home(self.codex_home.path().to_path_buf())
            .harness_overrides(ConfigOverrides {
                cwd: Some(self.repo.path().to_path_buf()),
                ..Default::default()
            })
            .build()
            .await
            .expect("load config")
    }
}

/// The settings the denylist protects, as the child session sees them.
fn protected_settings(config: &Config) -> impl std::fmt::Debug + PartialEq + use<> {
    (
        config.model_provider_id.clone(),
        config.model_providers.contains_key("relay"),
        config.model_providers.get("openai").cloned(),
        config.notify.clone(),
        config.sqlite.clone(),
        config.log_dir.clone(),
        config.config_lock_export_dir.clone(),
    )
}

fn role_warnings(config: &Config) -> Vec<&str> {
    config
        .startup_warnings
        .iter()
        .map(String::as_str)
        .filter(|warning| warning.contains("agent role file"))
        .collect()
}

#[tokio::test]
async fn project_role_file_cannot_set_denylisted_keys_for_its_child() {
    let fixture = Fixture::new();
    Fixture::write(&fixture.project_agents(), "relay.toml", DENYLISTED_ROLE);
    let mut config = fixture.config().await;
    let before = protected_settings(&config);

    apply_role_to_config(&mut config, Some("relay"))
        .await
        .expect("project role should apply");

    assert_eq!(protected_settings(&config), before);
}

#[tokio::test]
async fn project_config_role_entry_cannot_set_denylisted_keys_for_its_child() {
    let fixture = Fixture::new();
    // Outside `.codex/agents/`, named by the project's own config.
    let role_dir = fixture.repo.path().join("roles");
    Fixture::write(&role_dir, "relay.toml", DENYLISTED_ROLE);
    Fixture::write(
        &fixture.repo.path().join(".codex"),
        CONFIG_TOML_FILE,
        "[agents.relay]\nconfig_file = \"../roles/relay.toml\"\n",
    );
    let mut config = fixture.config().await;
    let before = protected_settings(&config);

    apply_role_to_config(&mut config, Some("relay"))
        .await
        .expect("project role should apply");

    assert_eq!(protected_settings(&config), before);
}

#[tokio::test]
async fn user_role_file_still_sets_denylisted_keys_for_its_child() {
    let fixture = Fixture::new();
    let role_file = Fixture::write(&fixture.user_agents(), "relay.toml", DENYLISTED_ROLE);
    let mut config = fixture.config().await;
    let before = protected_settings(&config);

    apply_role_to_config(&mut config, Some("relay"))
        .await
        .expect("user role should apply");

    assert_ne!(protected_settings(&config), before);
    assert_eq!(config.model_provider_id, "relay");
    let role_dir = role_file.parent().expect("role dir");
    assert_eq!(config.log_dir, role_dir.join("logs"));
    assert_eq!(config.config_lock_export_dir.as_ref().map(AbsolutePathBuf::as_path), Some(role_dir.join("export").as_path()));
    assert!(role_warnings(&config).is_empty());
}

#[tokio::test]
async fn roles_record_where_they_were_declared_and_project_roles_warn() {
    let fixture = Fixture::new();
    let project_file = Fixture::write(&fixture.project_agents(), "relay.toml", DENYLISTED_ROLE);
    let user_file = Fixture::write(
        &fixture.user_agents(),
        "user-relay.toml",
        &DENYLISTED_ROLE.replace("name = \"relay\"", "name = \"user-relay\""),
    );
    // The project names the user's role without a file of its own: the
    // user's file still applies, so the role stays a user role.
    Fixture::write(
        &fixture.repo.path().join(".codex"),
        CONFIG_TOML_FILE,
        "[agents.user-relay]\ndescription = \"Renamed by the project\"\n",
    );
    let config = fixture.config().await;

    let source = |name: &str| config.agent_roles.get(name).map(|role| role.source);
    assert_eq!(source("relay"), Some(AgentRoleSource::Project));
    assert_eq!(source("user-relay"), Some(AgentRoleSource::User));
    assert_eq!(
        config
            .agent_roles
            .get("user-relay")
            .and_then(|role| role.config_file.clone()),
        Some(user_file)
    );
    assert!(
        built_in::configs()
            .values()
            .all(|role| role.source == AgentRoleSource::BuiltIn)
    );
    assert_eq!(
        role_warnings(&config),
        vec![
            project_role_ignored_keys_warning(
                &project_file,
                &IGNORED_KEYS
                    .split(", ")
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>(),
            )
            .as_str()
        ]
    );
}

#[tokio::test]
async fn project_role_is_sanitized_when_spawned_not_only_when_discovered() {
    let fixture = Fixture::new();
    let mut config = fixture.config().await;
    // Written after startup: only the spawn-time read sees these keys.
    let role_file = Fixture::write(&fixture.project_agents(), "late.toml", DENYLISTED_ROLE);
    config.agent_roles.insert(
        "late".to_string(),
        AgentRoleConfig {
            description: Some("Late role".to_string()),
            config_file: Some(role_file),
            nickname_candidates: None,
            source: AgentRoleSource::Project,
        },
    );
    let before = protected_settings(&config);

    apply_role_to_config(&mut config, Some("late"))
        .await
        .expect("project role should apply");

    assert_eq!(protected_settings(&config), before);
}

#[test]
fn role_source_follows_the_config_layer_kind() {
    let file = AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("config.toml"))
        .expect("absolute");
    let cases = [
        (
            ConfigLayerSource::Project {
                dot_codex_folder: file.clone(),
            },
            AgentRoleSource::Project,
        ),
        (
            ConfigLayerSource::User {
                file: file.clone(),
                profile: None,
            },
            AgentRoleSource::User,
        ),
        (ConfigLayerSource::SessionFlags, AgentRoleSource::User),
        (
            ConfigLayerSource::System { file: file.clone() },
            AgentRoleSource::Managed,
        ),
        (
            ConfigLayerSource::Mdm {
                domain: "d".to_string(),
                key: "k".to_string(),
            },
            AgentRoleSource::Managed,
        ),
        (
            ConfigLayerSource::EnterpriseManaged {
                id: "i".to_string(),
                name: "n".to_string(),
            },
            AgentRoleSource::Managed,
        ),
        (
            ConfigLayerSource::LegacyManagedConfigTomlFromFile { file },
            AgentRoleSource::Managed,
        ),
        (
            ConfigLayerSource::LegacyManagedConfigTomlFromMdm,
            AgentRoleSource::Managed,
        ),
    ];
    for (layer, expected) in cases {
        assert_eq!(AgentRoleSource::from_layer(&layer), expected, "{layer:?}");
    }
}
