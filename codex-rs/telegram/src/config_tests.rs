use clap::Parser;
use codex_protocol::config_types::SandboxMode;
use codex_protocol::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_telegram::Cli;
use codex_telegram::config::DEFAULT_TOKEN_ENV;
use codex_telegram::config::TelegramConfig;
use codex_telegram::config::TelegramMode;

#[test]
fn telegram_cli_accepts_health_mode() {
    let cli = Cli::try_parse_from(["corbanu telegram", "--health"])
        .expect("--health should be a valid Telegram connector mode");

    assert!(cli.health);
    assert!(!cli.strict_config);
}

#[test]
fn telegram_cli_accepts_setup_mode() {
    let cli = Cli::try_parse_from(["telegram", "--setup"])
        .expect("--setup should be a valid Telegram connector mode");
    assert!(cli.setup);
    assert!(!cli.health);
}

#[test]
fn telegram_cli_rejects_setup_with_health() {
    Cli::try_parse_from(["telegram", "--setup", "--health"])
        .expect_err("setup and health are mutually exclusive");
}

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
        allowed_user_ids = [1, 77]
        max_attachment_bytes = 5242880
        media_retention_days = 3
        max_media_store_bytes = 104857600
        mode = "polling"
        default_model = "glm-5.2"
        approval_policy = "on-request"
        sandbox_mode = "workspace-write"
        webhook_url = ""
        max_consecutive_polling_failures = 3
        "#,
    )
    .expect("config parses");

    assert_eq!(config.enabled, true);
    assert_eq!(config.bot_token_env, "BOT_ENV");
    assert_eq!(config.allowed_chat_ids, vec![1, -1002]);
    assert_eq!(config.allowed_user_ids, vec![1, 77]);
    assert_eq!(config.max_attachment_bytes, 5 * 1024 * 1024);
    assert_eq!(config.media_retention_days, 3);
    assert_eq!(config.max_media_store_bytes, 100 * 1024 * 1024);
    assert_eq!(config.mode, TelegramMode::Polling);
    assert_eq!(config.default_model, Some("glm-5.2".to_string()));
    assert_eq!(config.approval_policy, Some(AskForApproval::OnRequest));
    assert_eq!(config.sandbox_mode, Some(SandboxMode::WorkspaceWrite));
    assert_eq!(config.webhook_url, Some(String::new()));
    assert_eq!(config.max_consecutive_polling_failures, 3);
}

#[test]
fn telegram_config_rejects_unknown_fields() {
    let err = TelegramConfig::from_toml_str(
        r#"
        [telegram]
        enabled = true
        alowed_chat_ids = [42]
        "#,
    )
    .expect_err("unknown telegram key is rejected");

    assert!(
        err.to_string()
            .contains("failed to parse [telegram] config")
    );
}

#[test]
fn telegram_config_rejects_unsafe_media_limits() {
    for (config, expected) in [
        (
            "[telegram]\nmax_attachment_bytes = 0",
            "max_attachment_bytes must be greater than zero",
        ),
        (
            "[telegram]\nmedia_retention_days = 0",
            "media_retention_days must be greater than zero",
        ),
        (
            "[telegram]\nmax_attachment_bytes = 20\nmax_media_store_bytes = 10",
            "max_media_store_bytes must be at least max_attachment_bytes",
        ),
    ] {
        let error = TelegramConfig::from_toml_str(config).unwrap_err();
        assert!(error.to_string().contains(expected));
    }
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

#[test]
fn load_from_codex_home_reads_config_file() {
    let codex_home = unique_temp_dir("codex-telegram-config");
    fs::create_dir_all(&codex_home).expect("create codex home");
    fs::write(
        codex_home.join("config.toml"),
        r#"
        [telegram]
        enabled = true
        allowed_chat_ids = [42]
        "#,
    )
    .expect("write config");

    let config = TelegramConfig::load_from_codex_home(&codex_home).expect("load config");

    assert_eq!(
        config,
        TelegramConfig {
            enabled: true,
            allowed_chat_ids: vec![42],
            ..Default::default()
        }
    );

    fs::remove_dir_all(codex_home).expect("remove codex home");
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
