#![cfg(test)]

//! Regression coverage for a key held across the sign-in menus.

use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn held_enter_cannot_cross_the_openai_other_methods_menu() {
    let (mut widget, _home) = tests::widget_openai_sign_in_menu().await;
    widget.auth_config.forced_login_method = None;
    let enter = |kind| KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, kind);

    widget.handle_key_event(KeyCode::Up.into());
    widget.handle_key_event(enter(KeyEventKind::Press));
    widget.handle_key_event(enter(KeyEventKind::Repeat));
    widget.handle_key_event(enter(KeyEventKind::Press));
    widget.handle_key_event(KeyCode::Down.into());
    widget.handle_key_event(enter(KeyEventKind::Repeat));
    assert_eq!(widget.highlighted_mode, SignInOption::ApiKey);
    assert!(matches!(
        *widget.sign_in_state.read().unwrap(),
        SignInState::PickMode
    ));

    widget.handle_key_event(enter(KeyEventKind::Release));
    widget.handle_key_event(enter(KeyEventKind::Press));
    assert!(matches!(
        *widget.sign_in_state.read().unwrap(),
        SignInState::ApiKeyEntry(_)
    ));
}

#[tokio::test]
async fn held_enter_requires_a_fresh_confirmation_to_save_a_prefilled_api_key() {
    let (mut widget, _home) = tests::widget_forced_chatgpt().await;
    widget.auth_config.forced_login_method = None;
    widget.api_key_provider_options = vec![tests::api_key_option(
        "deepseek",
        "DeepSeek",
        "DEEPSEEK_API_KEY",
    )];
    widget.highlighted_mode = SignInOption::ProviderApiKey(0);
    let (actions, mut received_actions) = tokio::sync::mpsc::unbounded_channel();
    widget.provider_auth_action_tx = actions;
    let enter = |kind| KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, kind);

    widget.handle_key_event(enter(KeyEventKind::Press));
    {
        let mut state = widget.sign_in_state.write().unwrap();
        let SignInState::ApiKeyEntry(input) = &mut *state else {
            panic!("expected API key entry");
        };
        input.value = "test-api-key".into();
        input.prepopulated_env_var = Some("DEEPSEEK_API_KEY".into());
    }
    let (request_frame, mut frames) = FrameRequester::test_channel();
    widget.request_frame = request_frame;
    widget.handle_key_event(enter(KeyEventKind::Repeat));
    widget.handle_key_event(enter(KeyEventKind::Press));
    widget.handle_key_event(KeyCode::Down.into());
    widget.handle_key_event(enter(KeyEventKind::Repeat));
    widget.handle_key_event(KeyCode::Left.into());
    widget.handle_key_event(enter(KeyEventKind::Repeat));
    assert_eq!(widget.highlighted_mode, SignInOption::ProviderApiKey(0));
    assert!(matches!(
        *widget.sign_in_state.read().unwrap(),
        SignInState::ApiKeyEntry(_)
    ));
    assert_eq!(
        frames.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    );
    assert!(received_actions.try_recv().is_err());

    widget.handle_key_event(enter(KeyEventKind::Release));
    widget.handle_key_event(enter(KeyEventKind::Press));
    frames.try_recv().expect("fresh Enter should start saving");
    assert!(matches!(
        received_actions.try_recv().unwrap(),
        ProviderAuthAction::StartApiKey(_)
    ));
    assert!(matches!(
        received_actions.try_recv().unwrap(),
        ProviderAuthAction::SetApiKey(_)
    ));
    assert!(matches!(
        received_actions.try_recv().unwrap(),
        ProviderAuthAction::Submit
    ));
    assert!(received_actions.try_recv().is_err());
    assert!(matches!(
        *widget.sign_in_state.read().unwrap(),
        SignInState::ApiKeySaving
    ));
    // Completion now comes from the shared provider-account flow.
    widget.on_api_key_configured("deepseek");
    frames
        .try_recv()
        .expect("API key completion should repaint");
    assert!(matches!(
        *widget.sign_in_state.read().unwrap(),
        SignInState::ApiKeyConfigured { .. }
    ));
}
