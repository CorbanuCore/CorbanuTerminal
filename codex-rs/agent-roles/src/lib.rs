mod agent_role_config;
mod discovery;
mod loader;
mod role_source;

pub use agent_role_config::AgentRoleConfig;
pub use agent_role_config::ResolvedAgentRoleFile;
pub use agent_role_config::parse_agent_role_file_contents;
pub use loader::load_agent_roles;
pub use role_source::AGENTS_SETTINGS_KEYS;
pub use role_source::APPLIED_ROLE_KEYS;
pub use role_source::AgentRoleSource;
pub use role_source::project_role_ignored_keys_warning;
pub use role_source::sanitize_role_config_layer;
pub use role_source::unapplied_role_keys;
pub use role_source::unapplied_role_keys_warning;
