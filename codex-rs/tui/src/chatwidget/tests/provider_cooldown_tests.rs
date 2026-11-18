use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn model_clear_cooldown_dispatches_only_the_current_provider_and_model() {
    let model = "claude-opus-5-5-plan";
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(Some(model)).await;
    chat.config.model_provider_id = "claude-plan".to_string();
    drain_insert_history(&mut rx);

    chat.dispatch_command_with_args(
        SlashCommand::Model,
        "clear-cooldown".to_string(),
        Vec::new(),
    );

    let event = rx.try_recv().expect("clear cooldown action");
    let AppEvent::ClearProviderCooldown {
        config,
        model: selected_model,
    } = event
    else {
        panic!("expected ClearProviderCooldown, got {event:?}");
    };
    assert_eq!(
        (config.model_provider_id.as_str(), selected_model.as_str()),
        ("claude-plan", model)
    );
    assert!(rx.try_recv().is_err());
    assert!(ops.try_recv().is_err());
}

#[tokio::test]
async fn model_clear_cooldown_keeps_existing_inline_model_settings() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(Some("gpt-5.5")).await;
    drain_insert_history(&mut rx);

    chat.dispatch_command_with_args(
        SlashCommand::Model,
        "--reasoning high --speed standard".to_string(),
        Vec::new(),
    );

    assert_matches!(rx.try_recv(), Ok(AppEvent::SelectSessionModel {
        model,
        effort: Some(ReasoningEffortConfig::High),
        service_tier: Some(service_tier),
    }) if model == "gpt-5.5" && service_tier == SERVICE_TIER_DEFAULT_REQUEST_VALUE);
    assert_matches!(rx.try_recv(), Ok(AppEvent::SettingsSelectionClosed));
    assert!(rx.try_recv().is_err());
    assert!(ops.try_recv().is_err());
}

#[tokio::test]
async fn model_clear_cooldown_rejects_a_broad_clear_with_usage() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(/*model_override*/ None).await;
    drain_insert_history(&mut rx);

    chat.dispatch_command_with_args(
        SlashCommand::Model,
        "clear-cooldown all".to_string(),
        Vec::new(),
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1);
    insta::assert_snapshot!(
        lines_to_single_string(&cells[0]),
        @"■ Usage: /model clear-cooldown"
    );
    assert!(rx.try_recv().is_err());
    assert!(ops.try_recv().is_err());
}
