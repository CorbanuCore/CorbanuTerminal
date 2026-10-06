//! Aggressive built only from controls that already exist (PF-24-S03 mapping
//! table). The launch path applies these values at the highest config
//! precedence, then [`verify`] checks the loaded config before Aggressive is
//! reported as active. Nothing here writes `config.toml`, so returning to
//! Permissive restores the user's own settings exactly.
//!
//! The sandbox row uses a named permission profile rather than a legacy
//! `SandboxPolicy`: under `untrusted`, an approved command that the legacy
//! sandbox blocks is retried outside it without asking again. A profile with a
//! denied-read path (the vault store) can never run unsandboxed, so approved
//! commands stay inside the current folder with network off.

use std::path::Path;

use codex_config::types::ShellEnvironmentPolicyToml;
use codex_features::Feature;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::WebSearchMode;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::shell_environment::create_env_from_vars;

use crate::legacy_core::config::Config;
use crate::legacy_core::config::ConfigOverrides;

/// The permission profile Aggressive defines and selects.
pub(crate) const PROFILE_ID: &str = "corbanu-aggressive";

/// Removed from agent command environments in addition to the built-in
/// `*KEY*`, `*SECRET*` and `*TOKEN*` excludes.
const SECRET_ENV_PATTERNS: [&str; 4] = ["*VAULT*", "*PASSWORD*", "*PASSPHRASE*", "*CREDENTIAL*"];

/// One row of the mapping table, as shown before confirmation.
pub(crate) const ROWS: [(&str, &str); 5] = [
    (
        "Sandbox",
        "write only in the current folder: no extra writable folders, no /tmp or $TMPDIR; approved commands stay inside it too",
    ),
    (
        "Approvals",
        "untrusted: you approve every command that is not known read-only (reviewer: you, never auto-review)",
    ),
    ("Network", "off for agent commands; web search off"),
    (
        "Vault",
        "agent commands running `corbanu vault …` are refused; the vault store is unreadable; secret-like environment variables (KEY, SECRET, TOKEN, VAULT, PASSWORD, PASSPHRASE, CREDENTIAL) are removed and shell profiles are not loaded",
    ),
    ("Child agents", "spawned agents get the same values"),
];

pub(crate) const UNCHANGED: &str = "Unchanged: model and provider, MCP servers and apps, wallet scopes, and commands you have already allowed permanently.";

fn string(value: &str) -> toml::Value {
    toml::Value::String(value.to_string())
}

/// `-c`-level overrides; they also reach the embedded app server, so every
/// thread it starts (and every child it spawns) is built from them.
pub(crate) fn base_overrides(codex_home: &Path) -> Vec<(String, toml::Value)> {
    let vault_store = codex_home.join("secrets").display().to_string();
    let profile = toml::toml! {
        extends = ":workspace"
        [filesystem]
        ":tmpdir" = "read"
        ":slash_tmp" = "read"
        [network]
        enabled = false
    };
    let mut profile = toml::Value::Table(profile);
    if let Some(filesystem) = profile
        .get_mut("filesystem")
        .and_then(toml::Value::as_table_mut)
    {
        // Read-only even when the workspace contains it: the stored level,
        // rules and config cannot be rewritten by an agent command.
        filesystem.insert(codex_home.display().to_string(), string("read"));
        filesystem.insert(vault_store, string("deny"));
    }
    vec![
        ("approval_policy".to_string(), string("untrusted")),
        ("approvals_reviewer".to_string(), string("user")),
        (format!("permissions.{PROFILE_ID}"), profile),
        ("default_permissions".to_string(), string(PROFILE_ID)),
        ("web_search".to_string(), string("disabled")),
        // The shell snapshot and login-shell profiles re-export variables the
        // environment policy removed; agent commands must not load either.
        (
            "features.shell_snapshot".to_string(),
            toml::Value::Boolean(false),
        ),
        ("allow_login_shell".to_string(), toml::Value::Boolean(false)),
        (
            "shell_environment_policy.ignore_default_excludes".to_string(),
            toml::Value::Boolean(false),
        ),
    ]
}

/// Environment overrides that extend, never replace, the user's own policy.
pub(crate) fn env_overrides(user_env: &ShellEnvironmentPolicyToml) -> Vec<(String, toml::Value)> {
    let mut overrides = Vec::new();
    // Keep the user's filters; legacy arrays and `filters` cannot be mixed.
    if user_env.filters.is_some() {
        overrides.extend(SECRET_ENV_PATTERNS.iter().map(|pattern| {
            (
                format!("shell_environment_policy.filters.{pattern}"),
                string("exclude"),
            )
        }));
    } else {
        let mut exclude = user_env.exclude.clone().unwrap_or_default();
        for pattern in SECRET_ENV_PATTERNS {
            if !exclude.iter().any(|existing| existing == pattern) {
                exclude.push(pattern.to_string());
            }
        }
        overrides.push((
            "shell_environment_policy.exclude".to_string(),
            toml::Value::Array(exclude.iter().map(|value| string(value)).collect()),
        ));
    }
    // Explicit `set` entries are applied after excludes; blank secret-named ones.
    let mut secret_sets = user_env
        .r#set
        .iter()
        .flatten()
        .map(|(name, _)| name)
        .filter(|name| is_secret_name(name))
        .collect::<Vec<_>>();
    secret_sets.sort();
    overrides.extend(
        secret_sets
            .into_iter()
            .map(|name| (format!("shell_environment_policy.set.{name}"), string(""))),
    );
    overrides
}

fn is_secret_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    [
        "KEY",
        "SECRET",
        "TOKEN",
        "VAULT",
        "PASSWORD",
        "PASSPHRASE",
        "CREDENTIAL",
    ]
    .iter()
    .any(|needle| upper.contains(needle))
}

/// Launch flags (`--sandbox`, `--ask-for-approval`, `--add-dir`, `--yolo`)
/// cannot weaken a stored Aggressive level. Returns what was overridden.
pub(crate) fn apply_launch_overrides(overrides: &mut ConfigOverrides) -> Vec<&'static str> {
    let mut replaced = Vec::new();
    if overrides
        .approval_policy
        .is_some_and(|policy| policy != AskForApproval::UnlessTrusted)
    {
        replaced.push("--ask-for-approval");
    }
    if overrides.sandbox_mode.is_some() || overrides.permission_profile.is_some() {
        replaced.push("--sandbox");
    }
    if !overrides.additional_writable_roots.is_empty() {
        replaced.push("--add-dir");
    }
    overrides.approval_policy = Some(AskForApproval::UnlessTrusted);
    overrides.approvals_reviewer = Some(ApprovalsReviewer::User);
    overrides.sandbox_mode = None;
    overrides.permission_profile = None;
    overrides.default_permissions = Some(PROFILE_ID.to_string());
    overrides.additional_writable_roots.clear();
    overrides.workspace_roots = None;
    overrides.tools_web_search_request = None;
    replaced
}

/// Every row must be observed in the loaded config; otherwise the failing
/// rows are returned and Aggressive must not be shown as active.
pub(crate) fn verify(config: &Config, rules_present: bool) -> Vec<String> {
    let mut failures = Vec::new();
    let cwd = config.cwd.as_path();
    let profile = config.permissions.active_permission_profile();
    if profile.as_ref().map(|profile| profile.id.as_str()) != Some(PROFILE_ID) {
        failures.push(format!(
            "Sandbox: expected profile {PROFILE_ID}, got {profile:?}"
        ));
    }
    let file_system = config.permissions.file_system_sandbox_policy();
    if !file_system.can_write_path_with_cwd(cwd, cwd) {
        failures.push("Sandbox: the current folder is not writable".to_string());
    }
    if !file_system.has_denied_read_restrictions() {
        failures.push("Sandbox: approved commands could run outside the sandbox".to_string());
    }
    let state_file = config
        .codex_home
        .join(super::level::STATE_FILE)
        .to_path_buf();
    if file_system.can_write_path_with_cwd(&state_file, cwd) {
        failures.push(format!("Sandbox: {} is writable", state_file.display()));
    }
    for outside in outside_paths(config) {
        if !outside.starts_with(cwd) && file_system.can_write_path_with_cwd(&outside, cwd) {
            failures.push(format!("Sandbox: {} is writable", outside.display()));
        }
    }
    let approval = config.permissions.approval_policy.value();
    if approval != AskForApproval::UnlessTrusted {
        failures.push(format!("Approvals: expected untrusted, got {approval}"));
    }
    if config.approvals_reviewer != ApprovalsReviewer::User {
        failures.push("Approvals: reviewer is not you".to_string());
    }
    if config.permissions.network_sandbox_policy().is_enabled() {
        failures.push("Network: sandbox network is enabled".to_string());
    }
    let web_search = config.web_search_mode.value();
    if web_search != WebSearchMode::Disabled {
        failures.push(format!("Network: web search is {web_search}"));
    }
    if !rules_present {
        failures.push("Vault: the exec-policy rule file is missing".to_string());
    }
    let probe = [
        "CORBANU_VAULT_X",
        "DB_PASSWORD",
        "GITHUB_TOKEN",
        "API_KEY",
        "AWS_SECRET",
    ]
    .map(|name| (name.to_string(), "x".to_string()));
    let leaked = create_env_from_vars(probe, &config.permissions.shell_environment_policy, None)
        .into_iter()
        .filter(|(name, value)| is_secret_name(name) && !value.is_empty())
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    if !leaked.is_empty() {
        failures.push(format!("Vault: environment keeps {}", leaked.join(", ")));
    }
    if config.features.enabled(Feature::ShellSnapshot) || config.permissions.allow_login_shell {
        failures.push("Vault: shell snapshots or login profiles can re-add variables".to_string());
    }
    let explicit = config
        .permissions
        .shell_environment_policy
        .r#set
        .iter()
        .filter(|(name, value)| is_secret_name(name) && !value.is_empty())
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    if !explicit.is_empty() {
        failures.push(format!("Vault: environment sets {}", explicit.join(", ")));
    }
    failures
}

fn outside_paths(config: &Config) -> Vec<std::path::PathBuf> {
    let mut paths = vec![std::path::PathBuf::from("/tmp/corbanu-aggressive-probe")];
    if let Some(parent) = config.cwd.as_path().parent() {
        paths.push(parent.join("corbanu-aggressive-probe"));
    }
    if let Some(tmpdir) = std::env::var_os("TMPDIR") {
        paths.push(Path::new(&tmpdir).join("corbanu-aggressive-probe"));
    }
    paths
}

#[cfg(test)]
#[path = "aggressive_tests.rs"]
mod tests;
