use codex_protocol::protocol::SandboxPolicy;
use pretty_assertions::assert_eq;

use super::SandboxPreflightIssue;
use super::SandboxPreflightSignals;
use super::preflight_issue_for_policy;
use super::warning_for_issue;

#[test]
fn danger_full_access_is_quiet_even_when_host_signals_fail() {
    let issue = preflight_issue_for_policy(
        &SandboxPolicy::DangerFullAccess,
        signals(
            /*bwrap_on_path*/ false,
            Some("0\n"),
            Some("1\n"),
            Some("0\n"),
        ),
    );

    assert_eq!(issue, None);
}

#[test]
fn sandboxed_policy_is_quiet_when_host_signals_pass() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ true,
            Some("1024\n"),
            Some("0\n"),
            Some("1\n"),
        ),
    );

    assert_eq!(issue, None);
}

#[test]
fn missing_optional_unprivileged_userns_clone_file_is_quiet() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ true,
            Some("1024\n"),
            /*apparmor_restrict_unprivileged_userns*/ None,
            /*unprivileged_userns_clone*/ None,
        ),
    );

    assert_eq!(issue, None);
}

#[test]
fn missing_bwrap_warns_for_sandboxed_policy() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ false,
            Some("1024\n"),
            Some("0\n"),
            Some("1\n"),
        ),
    );

    assert_eq!(issue, Some(SandboxPreflightIssue::BwrapMissing));
}

#[test]
fn disabled_max_user_namespaces_warns() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ true,
            Some("0\n"),
            Some("0\n"),
            Some("1\n"),
        ),
    );

    assert_eq!(
        issue,
        Some(SandboxPreflightIssue::MaxUserNamespacesDisabled)
    );
}

#[test]
fn apparmor_restricting_unprivileged_userns_warns() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ true,
            Some("1024\n"),
            Some("1\n"),
            Some("1\n"),
        ),
    );

    assert_eq!(
        issue,
        Some(SandboxPreflightIssue::AppArmorRestrictsUnprivilegedUserns)
    );
}

#[test]
fn disabled_unprivileged_userns_clone_warns() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ true,
            Some("1024\n"),
            Some("0\n"),
            Some("0\n"),
        ),
    );

    assert_eq!(
        issue,
        Some(SandboxPreflightIssue::UnprivilegedUsernsCloneDisabled)
    );
}

#[test]
fn invalid_proc_values_are_not_treated_as_disabled() {
    let issue = preflight_issue_for_policy(
        &read_only_policy(),
        signals(
            /*bwrap_on_path*/ true,
            Some("not-a-number\n"),
            Some("not-a-number\n"),
            Some(""),
        ),
    );

    assert_eq!(issue, None);
}

#[test]
fn warning_includes_actionable_remediation() {
    let warning = warning_for_issue(SandboxPreflightIssue::BwrapMissing);

    assert!(warning.contains("Sandbox appears unable to launch on this host"));
    assert!(warning.contains("sandbox_mode = \"danger-full-access\""));
    assert!(warning.contains("docs/config.md#telegram"));
}

fn read_only_policy() -> SandboxPolicy {
    SandboxPolicy::ReadOnly {
        network_access: false,
    }
}

fn signals(
    bwrap_on_path: bool,
    max_user_namespaces: Option<&str>,
    apparmor_restrict_unprivileged_userns: Option<&str>,
    unprivileged_userns_clone: Option<&str>,
) -> SandboxPreflightSignals {
    SandboxPreflightSignals {
        bwrap_on_path,
        max_user_namespaces: max_user_namespaces.map(str::to_string),
        apparmor_restrict_unprivileged_userns: apparmor_restrict_unprivileged_userns
            .map(str::to_string),
        unprivileged_userns_clone: unprivileged_userns_clone.map(str::to_string),
    }
}
