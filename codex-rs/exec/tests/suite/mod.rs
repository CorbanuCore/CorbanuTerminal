// Aggregates all former standalone integration tests as modules.
mod add_dir;
mod agents_md;
mod aggressive_broker_default;
mod apply_patch;
mod approval_policy;
mod auth_env;
#[path = "completion_backfill_tests.rs"]
mod completion_backfill;
mod ephemeral;
mod home_unreadable_wait;
mod hooks;
mod mcp_required_exit;
mod named_account_login_restrictions;
mod nested_launch;
mod originator;
mod output_schema;
mod plugin_sync_shutdown;
mod prompt_stdin;
mod provider_auth_failure_exit;
mod provider_key_env;
mod resume;
mod sandbox;
mod server_error_exit;
mod trace_log_credentials;
