//! Corbanu: where an agent role was declared, and which of a role file's keys
//! are applied when an agent is spawned with it.

use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerSource;
use codex_config::loader::sanitize_project_role_config;
use std::path::Path;
use toml::Value as TomlValue;

/// The kind of config that declared an agent role's config file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentRoleSource {
    /// Shipped with Corbanu.
    BuiltIn,
    /// The person's own config (user config, `~/.codex/agents/`) or launch
    /// flags.
    User,
    /// System, MDM, packaged or enterprise-managed config.
    Managed,
    /// A trusted project's `.codex` folder: repository contents, so the
    /// project-config denylist applies (D512, #1997). The default, so a role
    /// whose origin was not recorded gets the denylist.
    #[default]
    Project,
}

impl AgentRoleSource {
    /// A project-declared agent role's layer (launch flags in name only)
    /// declares project roles (D525, #2052).
    pub fn from_layer_entry(layer: &ConfigLayerEntry) -> Self {
        if layer.project_role_file().is_some() {
            return Self::Project;
        }
        Self::from_layer(&layer.name)
    }

    pub fn from_layer(source: &ConfigLayerSource) -> Self {
        match source {
            ConfigLayerSource::Project { .. } => Self::Project,
            ConfigLayerSource::User { .. } | ConfigLayerSource::SessionFlags => Self::User,
            ConfigLayerSource::PackagedDefaults { .. }
            | ConfigLayerSource::Mdm { .. }
            | ConfigLayerSource::System { .. }
            | ConfigLayerSource::EnterpriseManaged { .. }
            | ConfigLayerSource::LegacyManagedConfigTomlFromFile { .. }
            | ConfigLayerSource::LegacyManagedConfigTomlFromMdm => Self::Managed,
        }
    }
}

/// The `[agents]` keys that are settings rather than role declarations: the
/// named fields of `AgentsToml`, including the `max_threads` alias.
pub const AGENTS_SETTINGS_KEYS: &[&str] = &[
    "enabled",
    "max_concurrent_threads_per_session",
    "max_threads",
    "max_depth",
    "default_subagent_model",
    "default_subagent_reasoning_effort",
    "job_max_runtime_seconds",
    "interrupt_message",
    "provider_allowlist",
];

/// The top-level role file keys a spawned agent applies (D526 (a), provisional):
/// upstream's role fields plus base instructions, the provider choice and
/// `allow_login_shell = false`. Everything else, permission settings included,
/// leaves the parent's value in place.
pub const APPLIED_ROLE_KEYS: &[&str] = &[
    "developer_instructions",
    "instructions",
    "base_instructions",
    "model",
    "model_provider",
    "model_providers",
    "model_reasoning_effort",
    "model_reasoning_summary",
    "model_verbosity",
    "personality",
    "service_tier",
    "features",
    "skills",
    "allow_login_shell",
];

/// Strips the project-config denylist, and any role declarations, from a role
/// file's config layer when a project declared the role (D512, #1997),
/// returning the removed keys. User, managed and built-in roles are unchanged.
pub fn sanitize_role_config_layer(source: AgentRoleSource, config: &mut TomlValue) -> Vec<String> {
    match source {
        AgentRoleSource::Project => {
            let mut ignored_keys = sanitize_project_role_config(config);
            // A project role must not declare roles of its own: they would
            // reach a grandchild unsanitized. Every `agents` key that is not a
            // setting declares a role (as a table, or as an array serde reads
            // like one), so only the settings stay.
            if let Some(agents) = config.get_mut("agents").and_then(TomlValue::as_table_mut) {
                let roles = agents
                    .keys()
                    .filter(|key| !AGENTS_SETTINGS_KEYS.contains(&key.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                for name in roles {
                    agents.remove(&name);
                    ignored_keys.push(format!("agents.{name}"));
                }
            }
            ignored_keys
        }
        AgentRoleSource::BuiltIn | AgentRoleSource::User | AgentRoleSource::Managed => Vec::new(),
    }
}

/// The warning for keys a project's role file sets that project config may
/// not, worded like the project config warning.
pub fn project_role_ignored_keys_warning(role_file: &Path, ignored_keys: &[String]) -> String {
    format!(
        "Ignored unsupported project-local config keys in agent role file {}: {}. \
         A project's agent roles can't set what a project's config can't set; if you \
         want these settings to apply, declare the role in your user-level config.",
        role_file.display(),
        ignored_keys.join(", "),
    )
}

/// The `features` a role may turn off for its child; enabling a feature, or turning off any
/// other, is not applied (upstream's list).
pub const APPLIED_FEATURE_DISABLES: &[&str] = &[
    "shell_tool",
    "apps",
    "plugins",
    "memories",
    "request_permissions_tool",
];

/// The keys of a role file's config layer that a spawned agent does not apply (D526 (a)),
/// sorted: top-level keys outside `APPLIED_ROLE_KEYS`, feature enables and disables outside
/// `APPLIED_FEATURE_DISABLES`, `allow_login_shell = true`, and provider tables other than the
/// selected provider's (the role's `model_provider`, else `parent_provider` when known).
/// `skills` entries are not itemized.
pub fn unapplied_role_keys(config: &TomlValue, parent_provider: Option<&str>) -> Vec<String> {
    let Some(table) = config.as_table() else {
        return Vec::new();
    };
    let selected_provider = table
        .get("model_provider")
        .and_then(TomlValue::as_str)
        .or(parent_provider);
    let mut keys = Vec::new();
    for (key, value) in table {
        match key.as_str() {
            "features" => {
                for (feature, enabled) in value.as_table().into_iter().flatten() {
                    if enabled.as_bool() != Some(false)
                        || !APPLIED_FEATURE_DISABLES.contains(&feature.as_str())
                    {
                        keys.push(format!("features.{feature}"));
                    }
                }
            }
            "allow_login_shell" if value.as_bool() != Some(false) => keys.push(key.clone()),
            "model_providers" => {
                if let Some(selected_provider) = selected_provider {
                    for provider in value.as_table().into_iter().flatten().map(|(id, _)| id) {
                        if provider != selected_provider {
                            keys.push(format!("model_providers.{provider}"));
                        }
                    }
                }
            }
            // A table the denylist emptied sets nothing.
            _ if APPLIED_ROLE_KEYS.contains(&key.as_str())
                || value.as_table().is_some_and(toml::map::Map::is_empty) => {}
            _ => keys.push(key.clone()),
        }
    }
    keys.sort();
    keys
}

/// The warning for role file keys a spawned agent does not apply.
pub fn unapplied_role_keys_warning(role_file: &Path, keys: &[String]) -> String {
    format!(
        "Agent role file {} sets {}, which an agent spawned with this role does not apply: \
         it keeps its parent's settings for them. A role may change the model, a configured \
         provider, instructions and reasoning settings, and may only make permissions stricter \
         (`allow_login_shell = false`, turning some features off); a stricter or \
         incomparable sandbox_mode, approval_policy, approvals_reviewer, sandbox_workspace_write, \
         default_permissions, permissions, profile(s), shell_environment_policy or Windows \
         sandbox setting than the parent's refuses the spawn.",
        role_file.display(),
        keys.join(", "),
    )
}

/// Startup warnings for one role file's config layer: keys the project
/// denylist removes (D512) and keys a spawned agent does not apply (D526).
pub fn push_role_file_warnings(
    source: AgentRoleSource,
    role_file: &Path,
    config: &TomlValue,
    startup_warnings: &mut Vec<String>,
) {
    let mut config = config.clone();
    let ignored_keys = sanitize_role_config_layer(source, &mut config);
    if !ignored_keys.is_empty() {
        startup_warnings.push(project_role_ignored_keys_warning(role_file, &ignored_keys));
    }
    let unapplied_keys = unapplied_role_keys(&config, /*parent_provider*/ None);
    if !unapplied_keys.is_empty() {
        startup_warnings.push(unapplied_role_keys_warning(role_file, &unapplied_keys));
    }
}

#[cfg(test)]
mod project_role_layer_tests {
    use super::AgentRoleSource;
    use codex_config::ConfigLayerEntry;
    use codex_config::ConfigLayerSource;
    use codex_utils_absolute_path::AbsolutePathBuf;

    /// D525 (#2052): roles declared in a project-declared role's layer count
    /// as project roles; the same launch-flags layer without a role file is
    /// the person's.
    #[test]
    fn a_project_role_layer_declares_project_roles() {
        let layer = ConfigLayerEntry::new(
            ConfigLayerSource::SessionFlags,
            toml::Value::Table(Default::default()),
        );
        assert_eq!(AgentRoleSource::from_layer_entry(&layer), AgentRoleSource::User);
        let role_file = AbsolutePathBuf::from_absolute_path(
            std::env::temp_dir().join("project/.codex/agents/reviewer.toml"),
        )
        .expect("absolute role file");
        assert_eq!(
            AgentRoleSource::from_layer_entry(&layer.with_project_role_file(role_file)),
            AgentRoleSource::Project
        );
    }
}
