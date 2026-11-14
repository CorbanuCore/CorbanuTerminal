use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;

use super::super::session::SessionSettingsUpdate;
use super::super::tests::make_session_and_context;
use super::super::tests::make_session_and_context_with_rx;
use crate::windows_sandbox::WindowsSandboxLevelExt;
use crate::windows_sandbox::raise_windows_sandbox;

/// #2015: an agent a client starts from a thread gets the thread's current
/// settings, including changes made after the thread started.
#[tokio::test]
async fn issue_2015_detached_agent_config_follows_the_live_session() {
    let (mut session, _turn) = make_session_and_context().await;
    session.services.agent_control = session
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    session
        .update_settings(SessionSettingsUpdate {
            approval_policy: Some(AskForApproval::UnlessTrusted),
            approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
            sandbox_policy: Some(SandboxPolicy::new_read_only_policy()),
            windows_sandbox_level: Some(WindowsSandboxLevel::RestrictedToken),
            ..Default::default()
        })
        .await
        .expect("settings update");
    assert_eq!(
        session.detached_agent_config().await.security_level,
        SecurityLevel::Permissive
    );
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
    assert_eq!(config.approvals_reviewer, ApprovalsReviewer::AutoReview);
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

/// #2062: a saved downgrade must not lower a review of a still-protected parent.
#[tokio::test]
async fn issue_2062_detached_review_keeps_the_parent_level_until_next_start() {
    let (mut session, _turn) = make_session_and_context().await;
    session.services.agent_control = session
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    let controller = session
        .services
        .agent_control
        .trusted_security_controller()
        .expect("trusted controller");
    let home = tempfile::tempdir().expect("home");
    let store = crate::security::recovery::HomeTransitionStore::new(home.path());
    // Exercise the trusted production transition: unlike the low-level test
    // state replacement, it retains the live level when saving a downgrade.
    for level in [SecurityLevel::Aggressive, SecurityLevel::Permissive] {
        let epoch = session
            .services
            .agent_control
            .effective_security_policy()
            .snapshot_for_agent(session.thread_id)
            .expect("parent policy")
            .authority_epoch()
            .expect("epoch");
        let request = codex_protocol::security::SecurityControlRequest::new(
            epoch,
            codex_protocol::security::SecurityControlAction::SetLevel { level },
        )
        .expect("request");
        let confirmed = controller
            .confirm_security_request(request, 1_000)
            .expect("confirm");
        let prepared = controller
            .prepare_transition(confirmed, crate::security::transition::ProbeOutcome::Passed)
            .expect("prepare");
        let committed = controller
            .commit_transition(prepared, &store, 1_000)
            .expect("commit");
        assert_eq!(committed.level, SecurityLevel::Aggressive);
        assert_eq!(committed.next_start_level, level);
        assert_eq!(
            session.detached_agent_config().await.security_level,
            SecurityLevel::Aggressive
        );
    }
    let config = session.detached_agent_config().await;
    // A new review runtime must retain the parent's level even though the
    // trusted transition saved Permissive for an ordinary new session.
    assert_eq!(
        crate::security::recovery::recover(home.path(), config.security_level).level,
        SecurityLevel::Aggressive
    );
    assert_eq!(
        crate::security::recovery::recover(home.path(), SecurityLevel::Permissive).level,
        SecurityLevel::Permissive
    );
}

#[tokio::test]
async fn issue_2062_detached_review_refreshes_the_missing_windows_sandbox_notice() {
    let (mut session, _turn) = make_session_and_context().await;
    session.services.agent_control = session
        .services
        .agent_control
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    let controller = session
        .services
        .agent_control
        .trusted_security_controller()
        .expect("trusted controller");
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let change = controller
            .confirm_level_change(level, codex_security_policy::RevocationState::new())
            .expect("confirm");
        controller.apply_confirmed_change(change).expect("apply");
        let config = session.detached_agent_config().await;
        let notices: Vec<_> = config
            .startup_warnings
            .iter()
            .filter(|warning| warning.contains("needs the default Windows sandbox"))
            .cloned()
            .collect();
        let expected = if cfg!(windows) {
            vec![crate::security::windows_unsandboxed::startup_notice(level)]
        } else {
            Vec::new()
        };
        assert_eq!(notices, expected);
    }
}

#[tokio::test]
async fn issue_2062_detached_review_drops_a_stale_missing_sandbox_notice_after_setup() {
    let (session, _turn) = make_session_and_context().await;
    {
        let mut state = session.state.lock().await;
        let config =
            std::sync::Arc::make_mut(&mut state.session_configuration.original_config_do_not_use);
        config.startup_warnings = vec![
            "unrelated warning".to_string(),
            crate::security::windows_unsandboxed::startup_notice(SecurityLevel::Moderate),
        ];
    }
    session
        .update_settings(SessionSettingsUpdate {
            windows_sandbox_level: Some(WindowsSandboxLevel::Elevated),
            ..Default::default()
        })
        .await
        .expect("sandbox setup");
    assert_eq!(
        session.detached_agent_config().await.startup_warnings,
        vec!["unrelated warning".to_string()]
    );
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
