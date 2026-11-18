use super::ModelAction;
use crate::MultitoolCli;
use crate::Subcommand;
use clap::Parser;
use pretty_assertions::assert_eq;

#[test]
fn model_clear_cooldown_preserves_provider_model_and_profile_selection() {
    let cli = MultitoolCli::try_parse_from([
        "corbanu",
        "--profile",
        "work",
        "-m",
        "claude-opus-5-5-plan",
        "-c",
        "model_provider=\"claude-plan\"",
        "model",
        "clear-cooldown",
    ])
    .expect("parse cooldown recovery command");
    let command = cli.subcommand.as_ref().expect("model command");
    assert_eq!(
        crate::profile_v2_for_subcommand(&cli.interactive, command)
            .expect("model command supports profiles")
            .map(ToString::to_string),
        Some("work".to_string())
    );
    assert_eq!(
        cli.interactive.model.as_deref(),
        Some("claude-opus-5-5-plan")
    );
    assert_eq!(
        cli.config_overrides.parse_overrides().expect("overrides"),
        vec![(
            "model_provider".to_string(),
            toml::Value::String("claude-plan".to_string())
        )]
    );
    assert!(matches!(
        command,
        Subcommand::Model(super::ModelCommand {
            action: ModelAction::ClearCooldown
        })
    ));
    assert!(matches!(
        crate::nested_security::nested_launch_kind(Some(command)),
        Some(("model", codex_tui::NestedKind::Credentials))
    ));
}

#[test]
fn model_clear_cooldown_rejects_remote_local_state_confusion() {
    for remote_args in [
        ["--remote", "ws://localhost:4500"],
        ["--remote-auth-token-env", "TEST_REMOTE_TOKEN"],
    ] {
        let cli = MultitoolCli::try_parse_from([
            "corbanu",
            remote_args[0],
            remote_args[1],
            "model",
            "clear-cooldown",
        ])
        .expect("remote options parse before dispatch");
        assert!(
            crate::reject_remote_mode_for_subcommand(
                cli.remote.remote.as_deref(),
                cli.remote.remote_auth_token_env.as_deref(),
                "model",
            )
            .is_err()
        );
    }
}

#[test]
fn model_clear_cooldown_requires_an_exact_action() {
    for args in [
        vec!["corbanu", "model"],
        vec!["corbanu", "model", "clear-cooldown", "all"],
        vec!["corbanu", "model", "clear-all"],
    ] {
        assert!(MultitoolCli::try_parse_from(args).is_err());
    }
}
