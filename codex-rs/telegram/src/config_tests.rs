use codex_protocol::protocol::AskForApproval;
use pretty_assertions::assert_eq;

use codex_telegram::config::DEFAULT_TOKEN_ENV;
use codex_telegram::config::TelegramConfig;
use codex_telegram::config::TelegramMode;

#[test]
fn telegram_config_defaults_when_table_absent() {
    let config = TelegramConfig::from_toml_str("model = \"x\"").expect("config parses");

    assert_eq!(config.enabled, false);
    assert_eq!(config.bot_token_env, DEFAULT_TOKEN_ENV);
    assert_eq!(config.approval_policy, Some(AskForApproval::OnRequest));
}

#[test]
fn telegram_config_parses_local_table() {
    let config = TelegramConfig::from_toml_str(
        r#"
        [telegram]
        enabled = true
        bot_token_env = "BOT_ENV"
        allowed_chat_ids = [1, -1002]
        mode = "polling"
        default_model = "glm-5.2"
        approval_policy = "on-request"
        webhook_url = ""
        max_consecutive_polling_failures = 3
        "#,
    )
    .expect("config parses");

    assert_eq!(config.enabled, true);
    assert_eq!(config.bot_token_env, "BOT_ENV");
    assert_eq!(config.allowed_chat_ids, vec![1, -1002]);
    assert_eq!(config.mode, TelegramMode::Polling);
    assert_eq!(config.default_model, Some("glm-5.2".to_string()));
    assert_eq!(config.approval_policy, Some(AskForApproval::OnRequest));
    assert_eq!(config.webhook_url, Some(String::new()));
    assert_eq!(config.max_consecutive_polling_failures, 3);
}

#[test]
fn token_resolution_prefers_env_over_vault() {
    let config = TelegramConfig {
        bot_token_env: "BOT_ENV".to_string(),
        ..Default::default()
    };

    let token = config
        .resolve_token_with(
            |name| (name == "BOT_ENV").then(|| "env-token".to_string()),
            || Some("vault-token".to_string()),
        )
        .expect("token resolves");

    assert_eq!(token, "env-token");
}

#[test]
fn token_resolution_falls_back_to_vault() {
    let token = TelegramConfig::default()
        .resolve_token_with(|_| None, || Some("vault-token".to_string()))
        .expect("token resolves");

    assert_eq!(token, "vault-token");
}

#[test]
fn token_resolution_errors_without_env_or_vault() {
    let err = TelegramConfig::default()
        .resolve_token_with(|_| None, || None)
        .expect_err("token missing");

    assert!(err.to_string().contains(DEFAULT_TOKEN_ENV));
}
