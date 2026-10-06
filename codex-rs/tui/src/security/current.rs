//! This session's value for each Aggressive mapping row, shown next to the
//! Aggressive value on the `/security` review (code-blind case B-06).
//!
//! These lines describe protection, so each one states only what the loaded
//! configuration enforces. When there is doubt, the line names the weaker
//! outcome; for example, a sandbox without denied reads lets approved and
//! allow-listed commands run outside it, so "off" network or an unreadable
//! vault store is never claimed for that case.
//!
//! Values come from the chat widget's config: approvals, sandbox, reviewer and
//! workspace roots follow `/permissions` and the session; web search, the
//! environment policy, shell settings and roles are this process's launch
//! settings.

use std::path::Path;

use codex_features::Feature;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::permissions::FileSystemSandboxKind;
use codex_protocol::shell_environment::create_env_from_vars;

use super::aggressive::ROWS;
use super::aggressive::is_secret_name;
use crate::legacy_core::config::Config;

/// One value per row of [`ROWS`], in the same order.
pub(crate) type CurrentValues = [String; ROWS.len()];

/// Secret-like name fragments, each with a probe variable that contains it.
const SECRET_PROBES: [(&str, &str); 7] = [
    ("KEY", "API_KEY"),
    ("SECRET", "AWS_SECRET"),
    ("TOKEN", "GITHUB_TOKEN"),
    ("VAULT", "CORBANU_VAULT_X"),
    ("PASSWORD", "DB_PASSWORD"),
    ("PASSPHRASE", "GPG_PASSPHRASE"),
    ("CREDENTIAL", "SVC_CREDENTIAL"),
];

/// Role config keys that cannot change what a spawned child is allowed to do.
const SAFE_ROLE_KEYS: [&str; 9] = [
    "description",
    "developer_instructions",
    "model",
    "model_provider",
    "model_reasoning_effort",
    "model_reasoning_summary",
    "model_verbosity",
    "nickname_candidates",
    "personality",
];

pub(crate) fn current_values(config: &Config) -> CurrentValues {
    let cwd = config.cwd.as_path();
    let file_system = config.permissions.file_system_sandbox_policy();
    let restricted = matches!(file_system.kind, FileSystemSandboxKind::Restricted);
    // Without denied reads, escalated, allow-listed and retried commands run
    // outside the sandbox (`unsandboxed_execution_allowed` in core).
    let escapes = restricted && !file_system.has_denied_read_restrictions();

    let mut sandbox = match file_system.kind {
        FileSystemSandboxKind::Unrestricted => {
            "no sandbox: commands can write anywhere".to_string()
        }
        FileSystemSandboxKind::ExternalSandbox => {
            "an external sandbox decides; Corbanu Terminal applies none".to_string()
        }
        FileSystemSandboxKind::Restricted if file_system.has_full_disk_write_access() => {
            "commands can write anywhere".to_string()
        }
        FileSystemSandboxKind::Restricted => {
            let roots = file_system
                .get_writable_roots_with_cwd(cwd)
                .into_iter()
                .map(|root| {
                    if root.root.as_path() == cwd {
                        "the current folder".to_string()
                    } else {
                        root.root.display().to_string()
                    }
                })
                .collect::<Vec<_>>();
            if roots.is_empty() {
                "commands cannot write files".to_string()
            } else {
                format!("commands can write to {}", roots.join(", "))
            }
        }
    };
    if let Some(profile) = config.permissions.active_permission_profile()
        && !profile.id.starts_with(':')
    {
        sandbox = format!("profile {}: {sandbox}", profile.id);
    }
    if escapes {
        sandbox.push_str("; approved or allow-listed commands can run outside the sandbox");
    }
    let request_tools = config.features.enabled(Feature::RequestPermissionsTool)
        || config.features.enabled(Feature::ExecPermissionApprovals);
    sandbox.push_str(if request_tools {
        "; permission-request tools are on"
    } else {
        "; permission-request tools are off"
    });

    let reviewer = match config.approvals_reviewer {
        ApprovalsReviewer::User => "you",
        ApprovalsReviewer::AutoReview => "auto-review",
    };
    let approvals = format!(
        "{} (reviewer: {reviewer})",
        config.permissions.approval_policy.value()
    );

    let network = if !restricted || config.permissions.network_sandbox_policy().is_enabled() {
        "on for agent commands"
    } else if escapes {
        "off inside the sandbox, on for commands that run outside it"
    } else {
        "off for agent commands"
    };
    let network = format!("{network}; web search {}", config.web_search_mode.value());

    let secrets = config.codex_home.join("secrets");
    let store_readable = escapes
        || !restricted
        || [
            secrets.as_path(),
            config.codex_home.join("auth.json").as_path(),
        ]
        .iter()
        .any(|path| file_system.can_read_path_with_cwd(path, cwd))
        || file_system
            .get_readable_roots_with_cwd(cwd)
            .iter()
            .any(|root| root.as_path().starts_with(secrets.as_path()));
    let kept = kept_secret_kinds(
        &config.permissions.shell_environment_policy,
        std::env::vars(),
    );
    let vault = format!(
        "the vault store and sign-in file are {} to agent commands; {}; {}",
        if store_readable {
            "readable"
        } else {
            "unreadable"
        },
        if kept.is_empty() {
            "secret-like environment variables are removed".to_string()
        } else {
            format!(
                "secret-like environment variables are passed through ({})",
                kept.join(", ")
            )
        },
        if config.features.enabled(Feature::ShellSnapshot)
            || config.permissions.allow_login_shell
            || config.permissions.shell_environment_policy.use_profile
        {
            "login profiles or shell snapshots are used"
        } else {
            "login profiles and shell snapshots are not used"
        }
    );

    let changing_roles = config
        .agent_roles
        .iter()
        .filter(|(_, role)| role.config_file.as_deref().is_some_and(role_changes_values))
        .map(|(name, _)| format!("`{name}`"))
        .collect::<Vec<_>>();
    let children = if changing_roles.is_empty() {
        "spawned agents get this session's values".to_string()
    } else {
        format!(
            "spawned agents get this session's values, except custom roles that change them: {}",
            changing_roles.join(", ")
        )
    };

    [sandbox, approvals, network, vault, children]
}

/// Secret-like name fragments whose variables reach agent commands: the fixed
/// probes plus every secret-like name in `env` (this process's environment).
/// Only names are kept.
fn kept_secret_kinds(
    policy: &ShellEnvironmentPolicy,
    env: impl IntoIterator<Item = (String, String)>,
) -> Vec<&'static str> {
    let probes = SECRET_PROBES.map(|(_, name)| (name.to_string(), "x".to_string()));
    let names = create_env_from_vars(probes, policy, /*thread_id*/ None)
        .into_iter()
        .chain(create_env_from_vars(env, policy, /*thread_id*/ None))
        .filter(|(name, value)| !value.is_empty() && is_secret_name(name))
        .map(|(name, _)| name.to_ascii_uppercase())
        .collect::<Vec<_>>();
    SECRET_PROBES
        .iter()
        .map(|(kind, _)| *kind)
        .filter(|kind| names.iter().any(|name| name.contains(kind)))
        .collect()
}

/// A role changes its children's values unless its config file sets only
/// [`SAFE_ROLE_KEYS`]; an unreadable file counts as changing them.
fn role_changes_values(file: &Path) -> bool {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|contents| toml::from_str::<toml::Table>(&contents).ok())
        .is_none_or(|table| {
            table
                .keys()
                .any(|key| !SAFE_ROLE_KEYS.contains(&key.as_str()))
        })
}

#[cfg(test)]
#[path = "current_tests.rs"]
mod tests;
