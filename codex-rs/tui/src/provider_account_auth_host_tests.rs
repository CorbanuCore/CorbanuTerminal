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

#[test]
fn claude_recovery_snapshot_presents_the_method_choice() {
    use codex_provider_auth::claude_account_flow::ClaudeAccountFlow;
    use codex_provider_auth::claude_account_flow::ClaudeAccountIntent;
    use codex_provider_auth::claude_account_flow::ClaudeAccountRecoveryReason;
    use codex_provider_auth::claude_account_flow::ClaudeAccountTarget;
    use codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource;

    let catalog = codex_provider_auth::ProviderCatalog::from_runtime_providers(
        &codex_model_provider_info::built_in_model_providers(None),
    );
    let target = ClaudeAccountTarget::from_catalog_entry(
        catalog
            .get(codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID)
            .expect("Claude Account catalog entry"),
    )
    .expect("Claude Account target");
    let snapshot = ClaudeAccountSnapshot::RecoveryRequired {
        flow: ClaudeAccountFlow {
            target,
            intent: ClaudeAccountIntent::UnauthorizedRecovery {
                source: ClaudeUnauthorizedRecoverySource::ManagedToken,
            },
        },
        reason: ClaudeAccountRecoveryReason::MissingSelection,
    };

    assert!(presents_claude_method_choice(&snapshot));
}
