use codex_provider_auth::ProviderAuthAction;
use codex_provider_auth::claude_account_flow::ClaudeAccountAction;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn pending_cancel_actions_target_the_exact_account_reducer() {
    assert!(matches!(
        ProviderAccountCancelKind::OpenAi.action(),
        ProviderAuthAction::OpenAiAccount(OpenAiAccountAction::Cancel)
    ));
    assert!(matches!(
        ProviderAccountCancelKind::Claude.action(),
        ProviderAuthAction::ClaudeAccount(ClaudeAccountAction::Cancel)
    ));
}

#[test]
fn pending_cancel_actions_are_secret_free_and_distinct() {
    let openai = format!("{:?}", ProviderAccountCancelKind::OpenAi.action());
    let claude = format!("{:?}", ProviderAccountCancelKind::Claude.action());

    assert_eq!(openai, "OpenAiAccount(Cancel)");
    assert_eq!(claude, "ClaudeAccount(Cancel)");
    assert!(!format!("{openai}{claude}").contains("token"));
}
