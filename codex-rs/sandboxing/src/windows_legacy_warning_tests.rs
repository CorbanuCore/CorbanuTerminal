use super::*;
use pretty_assertions::assert_eq;

fn warning_for(
    host: HostPlatform,
    level: WindowsSandboxLevel,
    permission_profile: &PermissionProfile,
) -> Option<String> {
    legacy_windows_sandbox_warning_for_host(host, level, permission_profile)
}

#[test]
fn legacy_backend_on_windows_warns_for_sandboxed_profiles() {
    for profile in [
        PermissionProfile::read_only(),
        PermissionProfile::workspace_write(),
    ] {
        assert_eq!(
            warning_for(
                HostPlatform::Windows,
                WindowsSandboxLevel::RestrictedToken,
                &profile
            ),
            Some(LEGACY_WINDOWS_SANDBOX_WARNING.to_string())
        );
    }
}

#[test]
fn elevated_backend_on_windows_does_not_warn() {
    assert_eq!(
        warning_for(
            HostPlatform::Windows,
            WindowsSandboxLevel::Elevated,
            &PermissionProfile::workspace_write()
        ),
        None
    );
}

#[test]
fn disabled_windows_sandbox_does_not_warn() {
    assert_eq!(
        warning_for(
            HostPlatform::Windows,
            WindowsSandboxLevel::Disabled,
            &PermissionProfile::workspace_write()
        ),
        None
    );
}

#[test]
fn non_windows_host_does_not_warn() {
    assert_eq!(
        warning_for(
            HostPlatform::Other,
            WindowsSandboxLevel::RestrictedToken,
            &PermissionProfile::workspace_write()
        ),
        None
    );
}

#[test]
fn unsandboxed_profiles_do_not_warn() {
    assert_eq!(
        warning_for(
            HostPlatform::Windows,
            WindowsSandboxLevel::RestrictedToken,
            &PermissionProfile::Disabled
        ),
        None
    );
}
