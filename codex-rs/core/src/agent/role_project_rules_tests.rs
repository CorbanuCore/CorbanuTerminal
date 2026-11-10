//! D525 (#2052): settings in a role file a project declares follow the same
//! rules as the same settings in that project's `.codex/config.toml`: hook
//! trust, project MCP servers, feature defaults, `[security]`,
//! `agents.provider_allowlist` (which project config also may no longer
//! widen) and skill settings. User roles are unchanged.

use super::*;
use crate::config::ConfigBuilder;
use codex_config::CONFIG_TOML_FILE;
use codex_core_skills::config_rules::SkillConfigRuleSelector;
use codex_core_skills::config_rules::skill_config_rules_from_stack;
use codex_features::Feature;
use codex_hooks::HookListEntry;
use codex_hooks::HooksConfig;
use codex_protocol::protocol::HookSource;
use codex_protocol::protocol::HookTrustStatus;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;
use std::path::PathBuf;
use tempfile::TempDir;

const HOOK_COMMAND: &str = "echo role-hook";

struct Fixture {
    codex_home: TempDir,
    repo: TempDir,
    user_config: String,
}

impl Fixture {
    fn new() -> Self {
        Self::with_user_config("")
    }

    fn with_user_config(extra: &str) -> Self {
        let codex_home = TempDir::new().expect("codex home");
        let repo = TempDir::new().expect("repo");
        std::fs::create_dir_all(repo.path().join(".git")).expect("git dir");
        std::fs::write(repo.path().join(".git/HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        let workspace_key = repo.path().to_string_lossy().replace('\\', "\\\\");
        let user_config =
            format!("{extra}\n[projects.\"{workspace_key}\"]\ntrust_level = \"trusted\"\n");
        let fixture = Self {
            codex_home,
            repo,
            user_config,
        };
        fixture.write_user_config("");
        fixture
    }

    fn write_user_config(&self, extra: &str) {
        std::fs::write(
            self.codex_home.path().join(CONFIG_TOML_FILE),
            format!("{extra}\n{}", self.user_config),
        )
        .expect("user config");
    }

    fn write_project_config(&self, contents: &str) {
        write_file(&self.repo.path().join(".codex"), CONFIG_TOML_FILE, contents);
    }

    /// A role named `relay`, declared by the project.
    fn write_project_role(&self, body: &str) -> PathBuf {
        let dir = self.repo.path().join(".codex").join("agents");
        write_file(&dir, "relay.toml", &role_file("relay", body))
    }

    /// A role named `helper`, declared by the person's own config.
    fn write_user_role(&self, body: &str) -> PathBuf {
        let dir = self.codex_home.path().join("agents");
        write_file(&dir, "helper.toml", &role_file("helper", body))
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

    async fn child(&self, role: &str) -> Config {
        let mut config = self.config().await;
        apply_role_to_config(&mut config, Some(role))
            .await
            .expect("role should apply");
        config
    }
}

fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    std::fs::create_dir_all(dir).expect("dir");
    let path = dir.join(name);
    std::fs::write(&path, contents).expect("file");
    path
}

fn role_file(name: &str, body: &str) -> String {
    format!(
        "name = \"{name}\"\ndescription = \"{name} role\"\ndeveloper_instructions = \"Help\"\n{body}"
    )
}

/// A hook, optionally with a `[hooks.state]` entry marking it trusted.
fn hook_toml(trust: Option<&HookListEntry>) -> String {
    let state = trust.map(trust_state).unwrap_or_default();
    format!(
        "{state}[[hooks.PreToolUse]]\nmatcher = \"Bash\"\n\n\
         [[hooks.PreToolUse.hooks]]\ntype = \"command\"\ncommand = \"{HOOK_COMMAND}\"\n"
    )
}

fn trust_state(hook: &HookListEntry) -> String {
    format!(
        "[hooks.state.{:?}]\ntrusted_hash = {:?}\n\n",
        hook.key, hook.current_hash
    )
}

fn listed_hook(config: &Config) -> HookListEntry {
    codex_hooks::list_hooks(HooksConfig {
        feature_enabled: true,
        config_layer_stack: Some(config.config_layer_stack.clone()),
        ..Default::default()
    })
    .hooks
    .into_iter()
    .find(|hook| hook.command.as_deref() == Some(HOOK_COMMAND))
    .expect("the hook is listed")
}

#[tokio::test]
async fn project_config_hook_cannot_mark_itself_trusted() {
    let fixture = Fixture::new();
    fixture.write_project_config(&hook_toml(None));
    let hook = listed_hook(&fixture.config().await);
    fixture.write_project_config(&hook_toml(Some(&hook)));

    let hook = listed_hook(&fixture.config().await);

    assert_eq!(
        (hook.source, hook.trust_status),
        (HookSource::Project, HookTrustStatus::Untrusted)
    );
}

#[tokio::test]
async fn project_role_hook_needs_the_same_trust_review_as_project_config_hooks() {
    let fixture = Fixture::new();
    let role_path = fixture.write_project_role(&hook_toml(None));
    let hook = listed_hook(&fixture.child("relay").await);
    fixture.write_project_role(&hook_toml(Some(&hook)));

    let hook = listed_hook(&fixture.child("relay").await);

    assert_eq!(
        (hook.source, hook.trust_status),
        (HookSource::Project, HookTrustStatus::Untrusted)
    );
    // The trust key names the role file, so trusting it in one checkout does
    // not trust a same-looking hook in another.
    assert_eq!(hook.source_path.as_path(), role_path.as_path());

    // Trust the person records in their own config is honoured.
    fixture.write_user_config(&trust_state(&hook));
    let hook = listed_hook(&fixture.child("relay").await);
    assert_eq!(hook.trust_status, HookTrustStatus::Trusted);
}

#[tokio::test]
async fn user_role_hook_keeps_its_own_trust_state() {
    let fixture = Fixture::new();
    fixture.write_user_role(&hook_toml(None));
    let hook = listed_hook(&fixture.child("helper").await);
    fixture.write_user_role(&hook_toml(Some(&hook)));

    let hook = listed_hook(&fixture.child("helper").await);

    assert_eq!(
        (hook.source, hook.trust_status),
        (HookSource::SessionFlags, HookTrustStatus::Trusted)
    );
}

const AGGRESSIVE: &str = "[security]\nversion = 1\nlevel = \"aggressive\"\n";
const BROKER_OFF: &str = "[features]\nbroker_model_auth = false\n";

#[tokio::test]
async fn project_role_cannot_turn_off_a_security_level_feature_default() {
    let fixture = Fixture::with_user_config(AGGRESSIVE);
    fixture.write_project_role(BROKER_OFF);
    fixture.write_user_role(BROKER_OFF);

    let project_child = fixture.child("relay").await;
    let user_child = fixture.child("helper").await;

    assert_eq!(
        project_child.features.enabled(Feature::BrokerModelAuth),
        crate::model_broker_auth::LEVEL_DEFAULT_SUPPORTED
    );
    assert!(!user_child.features.enabled(Feature::BrokerModelAuth));
}

const USER_DOCS_SERVER: &str = "[mcp_servers.docs]\ncommand = \"docs-server\"\n";
const ROLE_DOCS_SERVER: &str = "[mcp_servers.docs]\ncommand = \"relay-server\"\n";

#[tokio::test]
async fn project_role_mcp_server_counts_as_a_project_server() {
    let fixture = Fixture::with_user_config(USER_DOCS_SERVER);
    fixture.write_project_role(ROLE_DOCS_SERVER);
    fixture.write_user_role(ROLE_DOCS_SERVER);

    let project_child = fixture.child("relay").await;
    let user_child = fixture.child("helper").await;

    // D52: no Aggressive "always allow" record for a server a project changes.
    assert!(crate::mcp_tool_call::project_layer_mentions_mcp_server(
        &project_child,
        "docs"
    ));
    assert!(!crate::mcp_tool_call::project_layer_mentions_mcp_server(
        &user_child,
        "docs"
    ));
    // An approval for the role's server is not saved to the person's config.
    assert!(!matches!(
        crate::mcp_tool_call::custom_mcp_tool_approval_config_builder(&project_child, "docs"),
        Ok(Some(_))
    ));
    assert!(matches!(
        crate::mcp_tool_call::custom_mcp_tool_approval_config_builder(&user_child, "docs"),
        Ok(Some(_))
    ));
}

const PERMISSIVE: &str = "[security]\nversion = 1\nlevel = \"permissive\"\n";

#[tokio::test]
async fn project_role_security_table_cannot_lower_the_level() {
    let fixture = Fixture::with_user_config(AGGRESSIVE);
    fixture.write_project_config(PERMISSIVE);
    fixture.write_project_role(PERMISSIVE);

    assert_eq!(
        fixture.config().await.security_level,
        SecurityLevel::Aggressive
    );
    assert_eq!(
        fixture.child("relay").await.security_level,
        SecurityLevel::Aggressive
    );
}

#[tokio::test]
async fn project_role_with_an_incomplete_security_table_is_not_used() {
    // Project config ignores this table with a warning (#845); a role file
    // with it does not load at all, which is stricter.
    let fixture = Fixture::new();
    fixture.write_project_role("[security]\nlevel = \"permissive\"\n");
    let mut config = fixture.config().await;

    assert!(
        apply_role_to_config(&mut config, Some("relay"))
            .await
            .is_err()
    );
}

fn allowlist(providers: &[&str]) -> String {
    format!("[agents]\nprovider_allowlist = {providers:?}\n")
}

fn allowed(config: &Config) -> Option<Vec<String>> {
    config.agent_provider_allowlist.clone()
}

#[tokio::test]
async fn project_config_cannot_widen_the_provider_allowlist() {
    let fixture = Fixture::with_user_config(&allowlist(&["openai"]));
    fixture.write_project_config(&allowlist(&["openai", "relay"]));

    let config = fixture.config().await;

    assert_eq!(allowed(&config), Some(vec!["openai".to_string()]));
}

#[tokio::test]
async fn project_role_cannot_widen_the_provider_allowlist() {
    let fixture = Fixture::with_user_config(&allowlist(&["openai"]));
    fixture.write_project_role(&allowlist(&["openai", "relay"]));
    fixture.write_user_role(&allowlist(&["openai", "relay"]));

    let project_child = fixture.child("relay").await;
    let user_child = fixture.child("helper").await;

    assert_eq!(allowed(&project_child), Some(vec!["openai".to_string()]));
    assert_eq!(
        allowed(&user_child),
        Some(vec!["openai".to_string(), "relay".to_string()])
    );
}

#[tokio::test]
async fn project_config_and_roles_can_narrow_the_provider_allowlist() {
    let fixture = Fixture::with_user_config(&allowlist(&["openai", "anthropic"]));
    fixture.write_project_config(&allowlist(&["openai", "relay"]));
    fixture.write_project_role(&allowlist(&["anthropic"]));

    let config = fixture.config().await;
    assert_eq!(allowed(&config), Some(vec!["openai".to_string()]));

    // Unrestricted until a project narrows it.
    let fixture = Fixture::new();
    fixture.write_project_role(&allowlist(&["openai"]));
    let project_child = fixture.child("relay").await;
    assert_eq!(allowed(&project_child), Some(vec!["openai".to_string()]));
}

const ENABLE_DEPLOY_SKILL: &str = "[[skills.config]]\nname = \"deploy\"\nenabled = true\n";

#[tokio::test]
async fn project_role_cannot_change_skill_settings() {
    let fixture = Fixture::new();
    fixture.write_project_role(ENABLE_DEPLOY_SKILL);
    fixture.write_user_role(ENABLE_DEPLOY_SKILL);

    let names = |config: &Config| {
        skill_config_rules_from_stack(&config.config_layer_stack)
            .entries
            .into_iter()
            .filter_map(|rule| match rule.selector {
                SkillConfigRuleSelector::Name(name) => Some(name),
                SkillConfigRuleSelector::Path(_) => None,
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(names(&fixture.child("relay").await), Vec::<String>::new());
    assert_eq!(
        names(&fixture.child("helper").await),
        vec!["deploy".to_string()]
    );
}
