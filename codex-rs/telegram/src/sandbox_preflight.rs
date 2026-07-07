use codex_protocol::protocol::SandboxPolicy;

const SANDBOX_WARNING_DOCS: &str = "docs/config.md#telegram";

pub(crate) fn warn_if_sandbox_may_fail(sandbox_policy: &SandboxPolicy) {
    #[cfg(target_os = "linux")]
    {
        if let Some(issue) = preflight_issue_for_policy(sandbox_policy, collect_runtime_signals()) {
            tracing::warn!("{}", warning_for_issue(issue));
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = sandbox_policy;
    }
}

#[cfg(any(test, target_os = "linux"))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct SandboxPreflightSignals {
    bwrap_on_path: bool,
    max_user_namespaces: Option<String>,
    apparmor_restrict_unprivileged_userns: Option<String>,
    unprivileged_userns_clone: Option<String>,
}

#[cfg(any(test, target_os = "linux"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SandboxPreflightIssue {
    BwrapMissing,
    MaxUserNamespacesDisabled,
    AppArmorRestrictsUnprivilegedUserns,
    UnprivilegedUsernsCloneDisabled,
}

#[cfg(any(test, target_os = "linux"))]
fn preflight_issue_for_policy(
    sandbox_policy: &SandboxPolicy,
    signals: SandboxPreflightSignals,
) -> Option<SandboxPreflightIssue> {
    if matches!(sandbox_policy, SandboxPolicy::DangerFullAccess) {
        return None;
    }

    if !signals.bwrap_on_path {
        return Some(SandboxPreflightIssue::BwrapMissing);
    }
    if proc_value_is_zero(signals.max_user_namespaces.as_deref()) {
        return Some(SandboxPreflightIssue::MaxUserNamespacesDisabled);
    }
    if proc_value_is_one(signals.apparmor_restrict_unprivileged_userns.as_deref()) {
        return Some(SandboxPreflightIssue::AppArmorRestrictsUnprivilegedUserns);
    }
    if proc_value_is_zero(signals.unprivileged_userns_clone.as_deref()) {
        return Some(SandboxPreflightIssue::UnprivilegedUsernsCloneDisabled);
    }
    None
}

#[cfg(any(test, target_os = "linux"))]
fn proc_value_is_zero(value: Option<&str>) -> bool {
    value
        .and_then(|value| value.trim().parse::<u64>().ok())
        .is_some_and(|value| value == 0)
}

#[cfg(any(test, target_os = "linux"))]
fn proc_value_is_one(value: Option<&str>) -> bool {
    value
        .and_then(|value| value.trim().parse::<u64>().ok())
        .is_some_and(|value| value == 1)
}

#[cfg(target_os = "linux")]
fn collect_runtime_signals() -> SandboxPreflightSignals {
    SandboxPreflightSignals {
        bwrap_on_path: find_executable_on_path("bwrap"),
        max_user_namespaces: read_proc_value("/proc/sys/user/max_user_namespaces"),
        apparmor_restrict_unprivileged_userns: read_proc_value(
            "/proc/sys/kernel/apparmor_restrict_unprivileged_userns",
        ),
        unprivileged_userns_clone: read_proc_value("/proc/sys/kernel/unprivileged_userns_clone"),
    }
}

#[cfg(target_os = "linux")]
fn read_proc_value(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[cfg(target_os = "linux")]
fn find_executable_on_path(program: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&path).any(|dir| is_executable_file(dir.join(program).as_path()))
}

#[cfg(target_os = "linux")]
fn is_executable_file(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(any(test, target_os = "linux"))]
fn warning_for_issue(issue: SandboxPreflightIssue) -> String {
    format!(
        "Sandbox appears unable to launch on this host ({}). Every command will require manual \
         Telegram approval. On a trusted single-user host set sandbox_mode = \
         \"danger-full-access\" in config.toml; otherwise enable unprivileged user namespaces \
         and ensure bwrap is installed on PATH. See {SANDBOX_WARNING_DOCS}.",
        issue.reason()
    )
}

#[cfg(any(test, target_os = "linux"))]
impl SandboxPreflightIssue {
    fn reason(self) -> &'static str {
        match self {
            SandboxPreflightIssue::BwrapMissing => "bwrap was not found on PATH",
            SandboxPreflightIssue::MaxUserNamespacesDisabled => {
                "/proc/sys/user/max_user_namespaces reads as 0"
            }
            SandboxPreflightIssue::AppArmorRestrictsUnprivilegedUserns => {
                "/proc/sys/kernel/apparmor_restrict_unprivileged_userns reads as 1"
            }
            SandboxPreflightIssue::UnprivilegedUsernsCloneDisabled => {
                "/proc/sys/kernel/unprivileged_userns_clone reads as 0"
            }
        }
    }
}

#[cfg(test)]
#[path = "sandbox_preflight_tests.rs"]
mod tests;
