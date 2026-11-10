//! D512 (#1997): a role declared by a project gets the project-config
//! denylist; user, managed and built-in roles are unchanged. Since the upstream
//! sync (D526 (a), provisional) a role applies only the keys in
//! `APPLIED_ROLE_KEYS`; the others are reported at startup.

use super::*;
use crate::config::ConfigBuilder;
use crate::config::ConfigOverrides;
use codex_agent_roles::AGENTS_SETTINGS_KEYS;
use codex_agent_roles::unapplied_role_keys_warning;
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
model = "relay-model"
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

/// The project-denylist warnings (D512).
fn role_warnings(config: &Config) -> Vec<&str> {
    config
        .startup_warnings
        .iter()
        .map(String::as_str)
        .filter(|warning| warning.contains("project-local config keys in agent role file"))
        .collect()
}

/// The warnings for keys a spawned agent does not apply (D526 (a)).
fn unapplied_warnings(config: &Config) -> Vec<&str> {
    config
        .startup_warnings
        .iter()
        .map(String::as_str)
        .filter(|warning| warning.contains("which an agent spawned with this role does not apply"))
        .collect()
}

/// The keys of `DENYLISTED_ROLE` a user role sets but a spawned agent does not apply.
const UNAPPLIED_USER_KEYS: [&str; 6] = [
    "debug",
    "features.respect_system_proxy",
    "log_dir",
    "notify",
    "openai_base_url",
    "sqlite_home",
];

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
async fn user_role_file_still_chooses_its_provider_for_its_child() {
    let fixture = Fixture::new();
    // A role may only select a provider the parent has configured.
    let user_config = fixture.codex_home.path().join(CONFIG_TOML_FILE);
    let mut contents = std::fs::read_to_string(&user_config).expect("user config");
    contents.push_str(
        "\n[model_providers.relay]\nname = \"configured relay\"\nbase_url = \"https://configured.example/v1\"\n",
    );
    std::fs::write(&user_config, contents).expect("user config");
    let role_file = Fixture::write(&fixture.user_agents(), "relay.toml", DENYLISTED_ROLE);
    let mut config = fixture.config().await;
    let parent = config.clone();

    apply_role_to_config(&mut config, Some("relay"))
        .await
        .expect("user role should apply");

    // D526 (a): a user role may choose a configured provider and overlay its table.
    assert_eq!(config.model_provider_id, "relay");
    assert_eq!(config.model.as_deref(), Some("relay-model"));
    assert_eq!(config.model_provider.name, "relay");
    assert_eq!(
        config.model_provider.base_url.as_deref(),
        Some("https://relay.example/v1")
    );
    assert_eq!(config.model_providers["relay"], config.model_provider);
    // Keys outside `APPLIED_ROLE_KEYS` keep the parent's values and are reported.
    assert_eq!(config.notify, parent.notify);
    assert_eq!(config.log_dir, parent.log_dir);
    assert_eq!(config.sqlite, parent.sqlite);
    assert_eq!(config.config_lock_export_dir, parent.config_lock_export_dir);
    assert_eq!(config.permissions, parent.permissions);
    assert!(role_warnings(&config).is_empty());
    assert_eq!(
        unapplied_warnings(&config),
        vec![
            unapplied_role_keys_warning(&role_file, &UNAPPLIED_USER_KEYS.map(ToOwned::to_owned),)
                .as_str()
        ]
    );
}

#[tokio::test]
async fn roles_record_where_they_were_declared_and_project_roles_warn() {
    let fixture = Fixture::new();
    let project_file = Fixture::write(&fixture.project_agents(), "relay.toml", DENYLISTED_ROLE);
    // The project's `relay` replaces the user's role of the same name.
    let user_relay_file = Fixture::write(&fixture.user_agents(), "relay.toml", DENYLISTED_ROLE);
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
    assert_eq!(
        config
            .agent_roles
            .get("relay")
            .and_then(|role| role.config_file.clone()),
        Some(project_file.clone())
    );
    assert_eq!(source("user-relay"), Some(AgentRoleSource::User));
    assert_eq!(
        config
            .agent_roles
            .get("user-relay")
            .and_then(|role| role.config_file.clone()),
        Some(user_file.clone())
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
    // The denylist empties `debug` and leaves only applied keys, so the project's
    // file gets no D526 warning; both user files do.
    let unapplied = UNAPPLIED_USER_KEYS.map(ToOwned::to_owned);
    let mut warnings = unapplied_warnings(&config);
    warnings.sort();
    let mut expected = vec![
        unapplied_role_keys_warning(&user_relay_file, &unapplied),
        unapplied_role_keys_warning(&user_file, &unapplied),
    ];
    expected.sort();
    assert_eq!(warnings, expected);
}

#[tokio::test]
async fn project_role_cannot_declare_roles_for_a_grandchild() {
    for nested_role in [
        "[agents.inner]\ndescription = \"Inner role\"\nconfig_file = \"../inner/inner.toml\"\n",
        "[agents]\ninner = { description = \"Inner role\", config_file = \"../inner/inner.toml\" }\n",
        // serde reads a struct from an array too.
        "[agents]\ninner = [\"Inner role\", \"../inner/inner.toml\", [\"Nick\"]]\n",
    ] {
        project_role_cannot_declare_role_for_a_grandchild(nested_role).await;
    }
}

async fn project_role_cannot_declare_role_for_a_grandchild(nested_role: &str) {
    let fixture = Fixture::new();
    let agents = fixture.project_agents();
    Fixture::write(
        &agents,
        "outer.toml",
        &format!(
            "name = \"outer\"\ndescription = \"Outer role\"\n\
             developer_instructions = \"Delegate\"\n\n{nested_role}"
        ),
    );
    // Outside `.codex/agents/`, so only the outer role's entry names it.
    Fixture::write(
        &fixture.repo.path().join(".codex").join("inner"),
        "inner.toml",
        &DENYLISTED_ROLE.replace("name = \"relay\"", "name = \"inner\""),
    );
    let mut config = fixture.config().await;
    assert!(config.agent_roles.contains_key("outer"), "{nested_role}");
    assert!(!config.agent_roles.contains_key("inner"));
    let before = protected_settings(&config);

    apply_role_to_config(&mut config, Some("outer"))
        .await
        .expect("project role should apply");

    assert!(!config.agent_roles.contains_key("inner"), "{nested_role}");
    assert_eq!(
        apply_role_to_config(&mut config, Some("inner")).await,
        Err("unknown agent_type 'inner'".to_string())
    );
    assert_eq!(protected_settings(&config), before);
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
fn project_role_layer_keeps_agent_settings_but_not_role_declarations() {
    let mut layer: TomlValue = toml::from_str(
        r#"
model = "gpt-5.5"
sqlite_home = "/tmp/state"

[agents]
max_depth = 3
max_threads = 2
provider_allowlist = ["openai"]
listed = ["Listed role", "/tmp/listed.toml"]
inline = { description = "Inline role" }

[agents.inner]
description = "Inner role"
"#,
    )
    .expect("toml");
    let user_layer = layer.clone();

    let ignored = sanitize_role_config_layer(AgentRoleSource::Project, &mut layer);

    let mut role_keys = ignored[1..].to_vec();
    role_keys.sort();
    assert_eq!(ignored[0], "sqlite_home");
    assert_eq!(
        role_keys,
        vec!["agents.inline", "agents.inner", "agents.listed"]
    );
    assert_eq!(
        layer,
        toml::from_str::<TomlValue>(
            r#"
model = "gpt-5.5"

[agents]
max_depth = 3
max_threads = 2
provider_allowlist = ["openai"]
"#
        )
        .expect("toml")
    );
    for source in [
        AgentRoleSource::User,
        AgentRoleSource::Managed,
        AgentRoleSource::BuiltIn,
    ] {
        let mut unchanged = user_layer.clone();
        assert!(sanitize_role_config_layer(source, &mut unchanged).is_empty());
        assert_eq!(unchanged, user_layer);
    }
}

/// Naming every field (no `..`) makes a new `[agents]` setting fail to
/// compile here until `AGENTS_SETTINGS_KEYS` lists it.
#[test]
fn agents_settings_keys_match_agents_toml() {
    let settings = codex_config::config_toml::AgentsToml {
        enabled: Some(true),
        max_concurrent_threads_per_session: Some(1),
        max_depth: Some(1),
        default_subagent_model: Some("model".to_string()),
        default_subagent_reasoning_effort: Some(ReasoningEffort::Low),
        job_max_runtime_seconds: Some(1),
        interrupt_message: Some(true),
        provider_allowlist: Some(Vec::new()),
        roles: BTreeMap::new(),
    };
    let mut keys = TomlValue::try_from(&settings)
        .expect("serialize")
        .as_table()
        .expect("table")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.push("max_threads".to_string());
    keys.sort();
    let mut expected = AGENTS_SETTINGS_KEYS
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(keys, expected);
}

#[test]
fn role_source_follows_the_config_layer_kind() {
    let file = AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("config.toml"))
        .expect("absolute");
    let cases = [
        (
            ConfigLayerSource::PackagedDefaults { file: file.clone() },
            AgentRoleSource::Managed,
        ),
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
