use codex_protocol::models::PermissionProfile;
use std::path::Path;

/// Basename used when the Codex executable self-invokes as the Linux sandbox
/// helper.
pub const CODEX_LINUX_SANDBOX_ARG0: &str = "codex-linux-sandbox";

pub fn allow_network_for_proxy(enforce_managed_network: bool) -> bool {
    // When managed network requirements are active, request proxy-only
    // networking from the Linux sandbox helper. Without managed requirements,
    // preserve existing behavior.
    enforce_managed_network
}

/// Converts the permission profile into the CLI invocation for
/// `codex-linux-sandbox`.
///
/// The helper performs the actual sandboxing (bubblewrap by default + seccomp)
/// after parsing these arguments. The profile JSON flag is emitted before
/// helper feature flags so the argv order matches the helper's CLI shape. See
/// `docs/linux_sandbox.md` for the Linux semantics.
#[allow(clippy::too_many_arguments)]
pub fn create_linux_sandbox_command_args_for_permission_profile(
    command: Vec<String>,
    command_cwd: &Path,
    permission_profile: &PermissionProfile,
    sandbox_policy_cwd: &Path,
    use_legacy_landlock: bool,
    allow_network_for_proxy: bool,
) -> Vec<String> {
    let permission_profile_json = serde_json::to_string(permission_profile)
        .unwrap_or_else(|err| panic!("failed to serialize permission profile: {err}"));
    let sandbox_policy_cwd = sandbox_policy_cwd
        .to_str()
        .unwrap_or_else(|| panic!("cwd must be valid UTF-8"))
        .to_string();
    let command_cwd = command_cwd
        .to_str()
        .unwrap_or_else(|| panic!("command cwd must be valid UTF-8"))
        .to_string();

    let mut linux_cmd: Vec<String> = vec![
        "--sandbox-policy-cwd".to_string(),
        sandbox_policy_cwd,
        "--command-cwd".to_string(),
        command_cwd,
        "--permission-profile".to_string(),
        permission_profile_json,
    ];
    // Proxy-only networking requires bubblewrap's isolated network namespace.
    if use_legacy_landlock && !allow_network_for_proxy {
        linux_cmd.push("--use-legacy-landlock".to_string());
    }
    if allow_network_for_proxy {
        linux_cmd.push("--allow-network-for-proxy".to_string());
    }
    linux_cmd.push("--".to_string());
    linux_cmd.extend(command);
    linux_cmd
}

/// The helper flag carrying the writable roots' pinned real locations
/// (#536): the helper binds each root there, and refuses to run when the
/// root no longer leads there.
pub const PINNED_WRITABLE_ROOTS_FLAG: &str = "--pinned-writable-roots";

/// Adds the pinned writable roots (#536) to helper arguments built by
/// [`create_linux_sandbox_command_args_for_permission_profile`], before the
/// command separator; also when there are none, so the helper checks the
/// roots it computes itself.
pub fn insert_pinned_writable_roots_arg(
    args: &mut Vec<String>,
    pinned: &[codex_protocol::permissions::PinnedWritableRoot],
) {
    let json = serde_json::to_string(pinned)
        .unwrap_or_else(|err| panic!("failed to serialize pinned writable roots: {err}"));
    let separator = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    args.splice(
        separator..separator,
        [PINNED_WRITABLE_ROOTS_FLAG.to_string(), json],
    );
}

/// The helper flag naming the folder where each run records the
/// placeholders it makes (#348), so one a run left behind (killed, then the
/// temporary folder lost on a reboot) is removed by a later run.
pub const PLACEHOLDER_RECORD_DIR_FLAG: &str = "--placeholder-record-dir";

/// That folder's name in the Corbanu home.
pub const PLACEHOLDER_RECORD_DIR_NAME: &str = "sandbox-placeholders";

/// Adds the placeholder record folder in `codex_home` (#348) to helper
/// arguments built by
/// [`create_linux_sandbox_command_args_for_permission_profile`], before the
/// command separator. Nothing when the home is not an existing folder (the
/// helper never makes the home) or its path is not UTF-8.
pub fn insert_placeholder_record_dir_arg(args: &mut Vec<String>, codex_home: &Path) {
    if !codex_home.is_absolute() || !codex_home.is_dir() {
        return;
    }
    let Some(dir) = codex_home
        .join(PLACEHOLDER_RECORD_DIR_NAME)
        .to_str()
        .map(str::to_string)
    else {
        return;
    };
    let separator = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    args.splice(
        separator..separator,
        [PLACEHOLDER_RECORD_DIR_FLAG.to_string(), dir],
    );
}

/// Converts the sandbox cwd and execution options into the CLI invocation for
/// `codex-linux-sandbox`.
#[cfg_attr(not(test), allow(dead_code))]
fn create_linux_sandbox_command_args(
    command: Vec<String>,
    command_cwd: &Path,
    sandbox_policy_cwd: &Path,
    use_legacy_landlock: bool,
    allow_network_for_proxy: bool,
) -> Vec<String> {
    let command_cwd = command_cwd
        .to_str()
        .unwrap_or_else(|| panic!("command cwd must be valid UTF-8"))
        .to_string();
    let sandbox_policy_cwd = sandbox_policy_cwd
        .to_str()
        .unwrap_or_else(|| panic!("cwd must be valid UTF-8"))
        .to_string();

    let mut linux_cmd: Vec<String> = vec![
        "--sandbox-policy-cwd".to_string(),
        sandbox_policy_cwd,
        "--command-cwd".to_string(),
        command_cwd,
    ];
    // Proxy-only networking requires bubblewrap's isolated network namespace.
    if use_legacy_landlock && !allow_network_for_proxy {
        linux_cmd.push("--use-legacy-landlock".to_string());
    }
    if allow_network_for_proxy {
        linux_cmd.push("--allow-network-for-proxy".to_string());
    }

    // Separator so that command arguments starting with `-` are not parsed as
    // options of the helper itself.
    linux_cmd.push("--".to_string());

    // Append the original tool command.
    linux_cmd.extend(command);

    linux_cmd
}

#[cfg(test)]
#[path = "landlock_tests.rs"]
mod tests;
