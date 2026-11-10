use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;

use super::super::session::SessionSettingsUpdate;
use super::super::tests::make_session_and_context_with_rx;
use super::raise_windows_sandbox;
use crate::windows_sandbox::WindowsSandboxLevelExt;

/// #2015: an agent a client starts from a thread gets the thread's current
/// settings, including changes made after the thread started.
#[tokio::test]
async fn issue_2015_detached_agent_config_follows_the_live_session() {
    let (session, _turn, _rx) = make_session_and_context_with_rx().await;
    session
        .update_settings(SessionSettingsUpdate {
            approval_policy: Some(AskForApproval::UnlessTrusted),
            sandbox_policy: Some(SandboxPolicy::new_read_only_policy()),
            windows_sandbox_level: Some(WindowsSandboxLevel::RestrictedToken),
            ..Default::default()
        })
        .await
        .expect("settings update");
    let controller = session
        .services
        .agent_control
        .trusted_security_controller()
        .expect("trusted controller");
    let change = controller
        .confirm_level_change(
            SecurityLevel::Aggressive,
            codex_security_policy::RevocationState::new(),
        )
        .expect("confirm");
    controller.apply_confirmed_change(change).expect("apply");

    let config = session.detached_agent_config().await;
    let (permission_profile, cwd) = {
        let state = session.state.lock().await;
        (
            state.session_configuration.permission_profile(),
            state.session_configuration.cwd().clone(),
        )
    };

    assert_eq!(
        config.permissions.approval_policy.value(),
        AskForApproval::UnlessTrusted
    );
    assert_eq!(config.permissions.permission_profile(), &permission_profile);
    assert_eq!(
        config.legacy_sandbox_policy(),
        SandboxPolicy::new_read_only_policy()
    );
    assert_eq!(config.cwd, cwd);
    assert_eq!(
        WindowsSandboxLevel::from_config(&config),
        WindowsSandboxLevel::RestrictedToken
    );
    assert_eq!(config.security_level, SecurityLevel::Aggressive);
}

/// #2015: the Windows sandbox is only ever raised, never lowered.
#[tokio::test]
async fn issue_2015_windows_sandbox_is_never_lowered() {
    let (session, _turn, _rx) = make_session_and_context_with_rx().await;
    let mut config = session.detached_agent_config().await;
    config.set_windows_elevated_sandbox_enabled(/*value*/ true);
    for level in [
        WindowsSandboxLevel::Disabled,
        WindowsSandboxLevel::RestrictedToken,
    ] {
        raise_windows_sandbox(&mut config, level);
        assert_eq!(
            WindowsSandboxLevel::from_config(&config),
            WindowsSandboxLevel::Elevated,
            "{level:?}"
        );
    }
    let mut config = session.detached_agent_config().await;
    config.permissions.windows_sandbox_mode = None;
    for feature in [
        codex_features::Feature::WindowsSandbox,
        codex_features::Feature::WindowsSandboxElevated,
    ] {
        config.features.disable(feature).expect("disable feature");
    }
    raise_windows_sandbox(&mut config, WindowsSandboxLevel::Elevated);
    assert_eq!(
        WindowsSandboxLevel::from_config(&config),
        WindowsSandboxLevel::Elevated
    );
}
