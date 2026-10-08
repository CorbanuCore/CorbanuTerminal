use codex_protocol::ThreadId;
#[cfg(test)]
use codex_protocol::config_types::EnvironmentVariablePattern;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::models::ActivePermissionProfile;
use codex_protocol::shell_environment;
use std::collections::HashMap;

pub use codex_protocol::shell_environment::CODEX_THREAD_ID_ENV_VAR;

/// JSON-encoded optional user profile for Task Node helpers launched by this turn.
/// This is routing context, not a credential or an OS security boundary.
pub const CORBANU_TASKNODE_PROFILE_ENV_VAR: &str = "CORBANU_TASKNODE_PROFILE";

pub(crate) fn inject_tasknode_profile_env(
    env: &mut HashMap<String, String>,
    config: &crate::config::Config,
) {
    let profile = config
        .config_layer_stack
        .get_active_user_layer()
        .and_then(|layer| match layer.metadata().name {
            codex_config::ConfigLayerSource::User { profile, .. } => profile,
            _ => None,
        });
    env.retain(|key, _| {
        !key.eq_ignore_ascii_case(CORBANU_TASKNODE_PROFILE_ENV_VAR)
            && !key.eq_ignore_ascii_case("CODEX_HOME")
    });
    env.insert(
        CORBANU_TASKNODE_PROFILE_ENV_VAR.to_string(),
        // Same JSON as serde_json::to_string (`null` or a string) without a fallible path.
        serde_json::Value::from(profile).to_string(),
    );
    env.insert(
        "CODEX_HOME".to_string(),
        config.codex_home.to_string_lossy().into_owned(),
    );
}

/// Well-known provider credential names that no built-in provider uses as its
/// `env_key`; the rest come from the built-in provider table.
const EXTRA_PROVIDER_AUTH_ENV_VARS: &[&str] = &[
    codex_login::OPENAI_API_KEY_ENV_VAR,
    codex_login::CODEX_API_KEY_ENV_VAR,
    "AZURE_OPENAI_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
];

fn built_in_provider_auth_env_vars<'a>() -> impl Iterator<Item = &'a str> {
    codex_model_provider_info::built_in_provider_api_key_env_vars()
        .iter()
        .map(String::as_str)
        .chain(EXTRA_PROVIDER_AUTH_ENV_VARS.iter().copied())
}

/// Informational name of the active permission profile. Child processes can
/// overwrite this value, so it must not be treated as proof of enforcement.
pub const CODEX_PERMISSION_PROFILE_ENV_VAR: &str = "CODEX_PERMISSION_PROFILE";

/// Construct an environment map based on the rules in the specified policy. The
/// resulting map can be passed directly to `Command::envs()` after calling
/// `env_clear()` to ensure no unintended variables are leaked to the spawned
/// process.
///
/// The derivation follows the algorithm documented in the struct-level comment
/// for [`ShellEnvironmentPolicy`].
///
/// `CODEX_THREAD_ID` is injected when a thread id is provided, even when
/// `include_only` is set.
pub fn create_env(
    policy: &ShellEnvironmentPolicy,
    thread_id: Option<ThreadId>,
) -> HashMap<String, String> {
    let thread_id = thread_id.map(|thread_id| thread_id.to_string());
    shell_environment::create_env(policy, thread_id.as_deref())
}

/// Builds the environment for a command the model runs. Provider credential
/// variables are removed unless the policy explicitly passes them (see
/// [`blocked_provider_auth_env_vars`]).
pub fn create_shell_tool_env<'a, I>(
    policy: &ShellEnvironmentPolicy,
    thread_id: Option<ThreadId>,
    provider_env_keys: I,
) -> HashMap<String, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut env = create_env(policy, thread_id);
    let blocked = blocked_provider_auth_env_vars(policy, provider_env_keys);
    env.retain(|key, _| !blocked.iter().any(|name| key.eq_ignore_ascii_case(name)));
    env
}

/// Credential variable names of every provider `config` knows: the built-in
/// table, configured providers and the selected one.
pub(crate) fn provider_env_keys(config: &crate::config::Config) -> Vec<&str> {
    config
        .model_providers
        .values()
        .chain(std::iter::once(&config.model_provider))
        .flat_map(|provider| provider.api_key_env_vars())
        .collect()
}

/// A built-in provider credential variable that Core keeps for itself.
pub(crate) fn is_provider_auth_env_var(name: &str) -> bool {
    built_in_provider_auth_env_vars().any(|known| name.eq_ignore_ascii_case(known))
}

/// Provider credential variables to keep out of a model-run command: the
/// built-in names plus `provider_env_keys`, minus any the policy explicitly
/// passes. A policy passes a name by setting it in `set` or by listing it
/// exactly (without wildcards) in `include_only`.
pub fn blocked_provider_auth_env_vars<'a, I>(
    policy: &ShellEnvironmentPolicy,
    provider_env_keys: I,
) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let explicitly_passed = |name: &str| {
        policy
            .r#set
            .keys()
            .any(|key| key.eq_ignore_ascii_case(name))
            || policy.include_only.iter().any(|pattern| {
                let pattern = pattern.to_string();
                !pattern.contains(['*', '?']) && pattern.eq_ignore_ascii_case(name)
            })
    };
    let mut blocked: Vec<String> = Vec::new();
    for name in built_in_provider_auth_env_vars().chain(provider_env_keys) {
        if !explicitly_passed(name) && !blocked.iter().any(|b| b.eq_ignore_ascii_case(name)) {
            blocked.push(name.to_string());
        }
    }
    blocked
}

/// Removes every provider credential variable, ignoring policy opt-ins.
pub fn remove_provider_auth_env_vars<'a, I>(env: &mut HashMap<String, String>, provider_env_keys: I)
where
    I: IntoIterator<Item = &'a str>,
{
    let provider_env_keys = provider_env_keys.into_iter().collect::<Vec<_>>();
    env.retain(|key, _| {
        !is_provider_auth_env_var(key)
            && !provider_env_keys
                .iter()
                .any(|blocked| key.eq_ignore_ascii_case(blocked))
    });
}

/// Injects the selected named permission profile into a shell tool's environment.
///
/// This is applied after the shell environment policy so the runtime-selected
/// profile wins over inherited or configured values.
pub(crate) fn inject_permission_profile_env(
    env: &mut HashMap<String, String>,
    active_permission_profile: Option<&ActivePermissionProfile>,
) {
    if cfg!(windows) {
        env.retain(|key, _| !key.eq_ignore_ascii_case(CODEX_PERMISSION_PROFILE_ENV_VAR));
    } else {
        env.remove(CODEX_PERMISSION_PROFILE_ENV_VAR);
    }
    if let Some(active_permission_profile) = active_permission_profile {
        env.insert(
            CODEX_PERMISSION_PROFILE_ENV_VAR.to_string(),
            active_permission_profile.id.clone(),
        );
    }
}

#[cfg(all(test, target_os = "windows"))]
fn create_env_from_vars<I>(
    vars: I,
    policy: &ShellEnvironmentPolicy,
    thread_id: Option<ThreadId>,
) -> HashMap<String, String>
where
    I: IntoIterator<Item = (String, String)>,
{
    let thread_id = thread_id.map(|thread_id| thread_id.to_string());
    shell_environment::create_env_from_vars(vars, policy, thread_id.as_deref())
}

#[cfg(test)]
fn populate_env<I>(
    vars: I,
    policy: &ShellEnvironmentPolicy,
    thread_id: Option<ThreadId>,
) -> HashMap<String, String>
where
    I: IntoIterator<Item = (String, String)>,
{
    let thread_id = thread_id.map(|thread_id| thread_id.to_string());
    shell_environment::populate_env(vars, policy, thread_id.as_deref())
}

#[cfg(test)]
#[path = "exec_env_tests.rs"]
mod tests;
