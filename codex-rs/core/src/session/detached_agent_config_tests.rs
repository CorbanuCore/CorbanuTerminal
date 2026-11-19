use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_security_policy::SecurityLevel;
use pretty_assertions::assert_eq;

use super::super::session::SessionSettingsUpdate;
use super::super::step_settings::StepSettingsUpdate;
use super::super::tests::make_session_and_context;
use super::super::tests::make_session_and_context_with_rx;
use crate::windows_sandbox::WindowsSandboxLevelExt;
use crate::windows_sandbox::raise_windows_sandbox;

/// #2015: an agent a client starts from a thread gets the thread's current
/// settings, including changes made after the thread started.
#[tokio::test]
async fn issue_2015_detached_agent_config_follows_the_live_session() {
    let (session, _turn) = make_session_and_context().await;
    // The clone shares the session's policy tree.
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    session
        .update_settings(SessionSettingsUpdate {
            step_settings: StepSettingsUpdate {
                approval_policy: Some(AskForApproval::UnlessTrusted),
                approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
                ..Default::default()
            },
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
    let controller = control
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
    let (session, _turn) = make_session_and_context().await;
    // The clone shares the session's policy tree.
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    let controller = control
        .trusted_security_controller()
        .expect("trusted controller");
    let home = tempfile::tempdir().expect("home");
    let store = crate::security::recovery::HomeTransitionStore::new(home.path());
    // Exercise the trusted production transition: unlike the low-level test
    // state replacement, it retains the live level when saving a downgrade.
    for level in [SecurityLevel::Aggressive, SecurityLevel::Permissive] {
        let epoch = control
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
            .confirm_security_request(request, /*now_unix_seconds*/ 1_000)
            .expect("confirm");
        let prepared = controller
            .prepare_transition(confirmed, crate::security::transition::ProbeOutcome::Passed)
            .expect("prepare");
        let committed = controller
            .commit_transition(prepared, &store, /*now_unix_seconds*/ 1_000)
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
    let (session, _turn) = make_session_and_context().await;
    // The clone shares the session's policy tree.
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    let controller = control
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

/// The loosest config a fork request could load: Permissive, no Windows
/// sandbox.
async fn loosest_fork_config(session: &super::super::session::Session) -> crate::config::Config {
    let mut config = (*session.get_config().await).clone();
    config.security_level = SecurityLevel::Permissive;
    config.permissions.windows_sandbox_mode = None;
    for feature in [
        codex_features::Feature::WindowsSandbox,
        codex_features::Feature::WindowsSandboxElevated,
    ] {
        config.features.disable(feature).expect("disable feature");
    }
    config
}

/// D711 (a): a thread forked from this one (upstream's replacement for a
/// detached review) is held at this thread's current security level and
/// Windows sandbox, as tightened mid-session, however low the fork asked;
/// a stricter fork stays stricter.
#[tokio::test]
async fn d711_fork_is_held_at_the_parents_current_level_and_windows_sandbox() {
    let (session, _turn) = make_session_and_context().await;
    // The clone shares the session's policy tree.
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    let mut fork = loosest_fork_config(&session).await;
    session.hold_fork_to_current_floor(&mut fork).await;
    assert_eq!(fork.security_level, SecurityLevel::Permissive);
    assert_eq!(
        WindowsSandboxLevel::from_config(&fork),
        WindowsSandboxLevel::Disabled
    );

    // Tightened mid-session: a Windows sandbox set up and a stricter level.
    session
        .update_settings(SessionSettingsUpdate {
            windows_sandbox_level: Some(WindowsSandboxLevel::RestrictedToken),
            ..Default::default()
        })
        .await
        .expect("sandbox setup");
    let controller = control
        .trusted_security_controller()
        .expect("trusted controller");
    let change = controller
        .confirm_level_change(
            SecurityLevel::Aggressive,
            codex_security_policy::RevocationState::new(),
        )
        .expect("confirm");
    controller.apply_confirmed_change(change).expect("apply");

    let mut fork = loosest_fork_config(&session).await;
    session.hold_fork_to_current_floor(&mut fork).await;
    assert_eq!(fork.security_level, SecurityLevel::Aggressive);
    assert_eq!(
        WindowsSandboxLevel::from_config(&fork),
        WindowsSandboxLevel::RestrictedToken
    );

    let mut fork = loosest_fork_config(&session).await;
    fork.set_windows_elevated_sandbox_enabled(/*value*/ true);
    session.hold_fork_to_current_floor(&mut fork).await;
    assert_eq!(
        WindowsSandboxLevel::from_config(&fork),
        WindowsSandboxLevel::Elevated
    );
}

/// D711 (a), #2062: after a downgrade saved for the next start, the parent
/// still runs at its stricter level, and so does a thread forked from it,
/// although a new thread would recover the saved lower level.
#[tokio::test]
async fn d711_fork_keeps_the_parents_level_after_a_saved_downgrade() {
    let (session, _turn) = make_session_and_context().await;
    // The clone shares the session's policy tree.
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    let controller = control
        .trusted_security_controller()
        .expect("trusted controller");
    let home = tempfile::tempdir().expect("home");
    let store = crate::security::recovery::HomeTransitionStore::new(home.path());
    for level in [SecurityLevel::Aggressive, SecurityLevel::Permissive] {
        let epoch = control
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
            .confirm_security_request(request, /*now_unix_seconds*/ 1_000)
            .expect("confirm");
        let prepared = controller
            .prepare_transition(confirmed, crate::security::transition::ProbeOutcome::Passed)
            .expect("prepare");
        controller
            .commit_transition(prepared, &store, /*now_unix_seconds*/ 1_000)
            .expect("commit");
    }
    let mut fork = loosest_fork_config(&session).await;
    session.hold_fork_to_current_floor(&mut fork).await;
    assert_eq!(fork.security_level, SecurityLevel::Aggressive);
    // The fork's session start recovers from its config's level.
    assert_eq!(
        crate::security::recovery::recover(home.path(), fork.security_level).level,
        SecurityLevel::Aggressive
    );
    assert_eq!(
        crate::security::recovery::recover(home.path(), SecurityLevel::Permissive).level,
        SecurityLevel::Permissive
    );
}

/// D721: a fork asking for a stricter Windows sandbox than its parent's keeps
/// it, an equal one is unchanged and a looser one is raised to the parent's.
#[tokio::test]
async fn d721_fork_keeps_a_stricter_windows_sandbox_and_floors_a_looser_one() {
    let (session, _turn) = make_session_and_context().await;
    session
        .update_settings(SessionSettingsUpdate {
            windows_sandbox_level: Some(WindowsSandboxLevel::RestrictedToken),
            ..Default::default()
        })
        .await
        .expect("sandbox setup");
    for (requested, expected) in [
        (WindowsSandboxLevel::Elevated, WindowsSandboxLevel::Elevated),
        (
            WindowsSandboxLevel::RestrictedToken,
            WindowsSandboxLevel::RestrictedToken,
        ),
        (
            WindowsSandboxLevel::Disabled,
            WindowsSandboxLevel::RestrictedToken,
        ),
    ] {
        let mut fork = loosest_fork_config(&session).await;
        raise_windows_sandbox(&mut fork, requested);
        session.hold_fork_to_current_floor(&mut fork).await;
        assert_eq!(
            WindowsSandboxLevel::from_config(&fork),
            expected,
            "requested {requested:?}"
        );
    }
}

/// D721 with D711 (a): when the parent's policy state can't be read, the
/// fork still runs at Aggressive, and a stricter Windows sandbox it asked
/// for stays.
#[tokio::test]
async fn d721_unreadable_parent_policy_forks_aggressive_with_the_stricter_sandbox() {
    let (session, _turn) = make_session_and_context().await;
    // The clone shares the session's policy tree.
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("security policy");
    control.effective_security_policy().poison_for_test();

    let mut fork = loosest_fork_config(&session).await;
    raise_windows_sandbox(&mut fork, WindowsSandboxLevel::Elevated);
    session.hold_fork_to_current_floor(&mut fork).await;
    assert_eq!(fork.security_level, SecurityLevel::Aggressive);
    assert_eq!(
        WindowsSandboxLevel::from_config(&fork),
        WindowsSandboxLevel::Elevated
    );
}
