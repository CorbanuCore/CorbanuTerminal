//! #2000: a Windows sandbox set up during a session applies to it, and a
//! running session's Windows sandbox never gets weaker.

use super::*;
use crate::session::tests::make_session_configuration_for_tests;
use codex_protocol::config_types::WindowsSandboxLevel;
use pretty_assertions::assert_eq;

fn with_level(level: WindowsSandboxLevel) -> SessionSettingsUpdate {
    SessionSettingsUpdate {
        windows_sandbox_level: Some(level),
        ..Default::default()
    }
}

#[tokio::test]
async fn issue_2000_windows_sandbox_only_gets_stronger_mid_session() {
    let mut configuration = make_session_configuration_for_tests().await;
    configuration.windows_sandbox_level = WindowsSandboxLevel::Disabled;

    let restricted = configuration
        .apply(&with_level(WindowsSandboxLevel::RestrictedToken))
        .expect("the non-admin sandbox applies to an unsandboxed session");
    assert_eq!(
        restricted.windows_sandbox_level,
        WindowsSandboxLevel::RestrictedToken
    );
    let elevated = restricted
        .apply(&with_level(WindowsSandboxLevel::Elevated))
        .expect("the default sandbox applies once set up");
    assert_eq!(
        elevated.windows_sandbox_level,
        WindowsSandboxLevel::Elevated
    );
    let unchanged = elevated
        .apply(&with_level(WindowsSandboxLevel::Elevated))
        .expect("the same sandbox is accepted");
    assert_eq!(
        unchanged.windows_sandbox_level,
        WindowsSandboxLevel::Elevated
    );

    for weaker in [
        WindowsSandboxLevel::RestrictedToken,
        WindowsSandboxLevel::Disabled,
    ] {
        let Err(err) = elevated.apply(&with_level(weaker)) else {
            panic!("{weaker:?} lowered a running session's Windows sandbox");
        };
        assert!(err.to_string().contains("next session"), "{err}");
    }
    let Err(err) = restricted.apply(&with_level(WindowsSandboxLevel::Disabled)) else {
        panic!("the non-admin sandbox was turned off mid-session");
    };
    assert!(err.to_string().contains("windows_sandbox_level"), "{err}");
}

/// #2000: what is started from the session's config afterwards (child
/// agents, helper sessions) gets the raised Windows sandbox too.
#[tokio::test]
async fn issue_2000_a_raised_windows_sandbox_reaches_the_per_turn_config() {
    use crate::windows_sandbox::WindowsSandboxLevelExt;
    let mut configuration = make_session_configuration_for_tests().await;
    configuration.windows_sandbox_level = WindowsSandboxLevel::Disabled;
    let per_turn = |configuration: &SessionConfiguration| {
        WindowsSandboxLevel::from_config(&Session::build_per_turn_config(
            configuration,
            configuration.cwd().clone(),
        ))
    };
    assert_eq!(per_turn(&configuration), WindowsSandboxLevel::Disabled);
    for level in [
        WindowsSandboxLevel::RestrictedToken,
        WindowsSandboxLevel::Elevated,
    ] {
        configuration = configuration
            .apply(&with_level(level))
            .expect("a stronger sandbox applies");
        assert_eq!(per_turn(&configuration), level);
    }
}
