//! PF-84-S04: one onboarding row per configured provider, and "Add another
//! account" behind it.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::named_account::ConfiguredChoice;
use super::tests::buffer_text;
use super::tests::widget_forced_chatgpt;
use super::*;
use crate::provider_named_accounts::AddAccountMethod;

const CANARY: &str = "sk-ant-oat01-pf84-s04-onboarding-canary";

fn configure_claude(widget: &mut AuthModeWidget) -> usize {
    widget.forced_login_method = None;
    widget.provider_status_host.update_account_metadata(
        crate::provider_status_host::ProviderAccountMetadata {
            claude: codex_provider_auth::ClaudeCredentialMetadata::Configured {
                source: codex_provider_auth::ClaudeCredentialSource::Managed,
            },
            ..Default::default()
        },
    );
    let statuses = widget.provider_status_host.resolve();
    *widget.provider_setup_session.write().unwrap() =
        ProviderSetupSession::from_statuses(statuses.entries());
    *widget.provider_statuses.write().unwrap() = statuses;
    widget
        .provider_status_host
        .catalog()
        .entries()
        .iter()
        .position(|entry| entry.id.as_str() == CLAUDE_PLAN_PROVIDER_ID)
        .expect("Claude catalog entry")
}

fn render(widget: &AuthModeWidget) -> String {
    let area = Rect::new(0, 0, 90, 60);
    let mut buf = Buffer::empty(area);
    widget.render_ref(area, &mut buf);
    buffer_text(&buf, area)
}

fn press(widget: &mut AuthModeWidget, code: KeyCode) {
    widget.handle_key_event(KeyEvent::new(code, KeyModifiers::NONE));
}

fn type_text(widget: &mut AuthModeWidget, text: &str) {
    for character in text.chars() {
        press(widget, KeyCode::Char(character));
    }
}

#[tokio::test]
async fn configured_claude_plan_has_exactly_one_row() {
    let (mut widget, _home) = widget_forced_chatgpt().await;
    let entry_index = configure_claude(&mut widget);

    let options = widget.displayed_sign_in_options();
    // The bug: both the setup row and the "use existing" row were offered.
    assert!(!options.contains(&SignInOption::AnthropicAccount));
    assert_eq!(
        options
            .iter()
            .filter(|option| **option == SignInOption::ProviderRuntime(entry_index))
            .count(),
        1
    );
    let screen = render(&widget);
    assert!(!screen.contains("Anthropic Claude Account"), "{screen}");
    assert_eq!(
        screen.matches("Provider: Claude Account").count(),
        1,
        "{screen}"
    );
}

#[tokio::test]
async fn configured_row_offers_use_replace_and_add_account() {
    let (mut widget, _home) = widget_forced_chatgpt().await;
    let entry_index = configure_claude(&mut widget);

    // Named accounts off: use or replace only.
    assert_eq!(
        widget.configured_choices(entry_index),
        vec![
            ConfiguredChoice::UseExisting,
            ConfiguredChoice::Replace(SignInOption::AnthropicAccount),
        ]
    );
    widget.named_account_methods = std::collections::BTreeMap::from([(
        CLAUDE_PLAN_PROVIDER_ID.to_string(),
        vec![AddAccountMethod::ClaudeToken],
    )]);
    assert_eq!(
        widget.configured_choices(entry_index),
        vec![
            ConfiguredChoice::UseExisting,
            ConfiguredChoice::AddAccount,
            ConfiguredChoice::Replace(SignInOption::AnthropicAccount),
        ]
    );

    widget.handle_sign_in_option(SignInOption::ProviderRuntime(entry_index));
    let screen = render(&widget);
    assert!(screen.contains("Use configured credentials"), "{screen}");
    assert!(screen.contains("Add another account"), "{screen}");
    assert!(
        screen.contains("Replace with Claude account sign-in"),
        "{screen}"
    );

    // Cancel is inert.
    press(&mut widget, KeyCode::Esc);
    assert!(matches!(
        &*widget.sign_in_state.read().unwrap(),
        SignInState::PickMode
    ));
}

#[tokio::test]
async fn add_another_account_saves_a_masked_named_account() {
    let (mut widget, home) = widget_forced_chatgpt().await;
    let entry_index = configure_claude(&mut widget);
    widget.named_account_methods = std::collections::BTreeMap::from([(
        CLAUDE_PLAN_PROVIDER_ID.to_string(),
        vec![AddAccountMethod::ClaudeToken],
    )]);
    widget.handle_sign_in_option(SignInOption::ProviderRuntime(entry_index));
    press(&mut widget, KeyCode::Down);
    press(&mut widget, KeyCode::Enter);

    // An invalid name is refused with the grammar.
    type_text(&mut widget, "Work");
    press(&mut widget, KeyCode::Enter);
    assert!(render(&widget).contains("invalid account name"));
    press(&mut widget, KeyCode::Backspace);
    press(&mut widget, KeyCode::Backspace);
    press(&mut widget, KeyCode::Backspace);
    press(&mut widget, KeyCode::Backspace);
    type_text(&mut widget, "work");
    press(&mut widget, KeyCode::Enter);

    widget.handle_paste(CANARY.to_string());
    let screen = render(&widget);
    assert!(!screen.contains(CANARY), "the value is masked");
    assert!(screen.contains("account `work`"), "{screen}");
    press(&mut widget, KeyCode::Enter);

    for _ in 0..200 {
        if matches!(
            &*widget.sign_in_state.read().unwrap(),
            SignInState::AccountSaved { .. }
        ) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let screen = render(&widget);
    assert!(
        screen.contains("Saved claude-plan account `work`."),
        "{screen}"
    );
    assert!(!screen.contains(CANARY));
    let stored = codex_vault::Vault::new(home.path().to_path_buf())
        .read_provider_account(
            CLAUDE_PLAN_PROVIDER_ID,
            &codex_vault::ProviderAccountName::parse("work").unwrap(),
            codex_vault::ProviderAccountKind::ClaudeOauthToken,
        )
        .expect("read")
        .map(|value| value.to_string());
    assert_eq!(stored.as_deref(), Some(CANARY));

    press(&mut widget, KeyCode::Enter);
    assert!(matches!(
        &*widget.sign_in_state.read().unwrap(),
        SignInState::PickMode
    ));
}
