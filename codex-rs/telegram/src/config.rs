use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use codex_protocol::config_types::SandboxMode;
use codex_protocol::protocol::AskForApproval;
use codex_vault::Vault;
use serde::Deserialize;

use crate::auth::ChatAllowlist;
use crate::error::TelegramError;

pub const DEFAULT_TOKEN_ENV: &str = "PFTERMINAL_TELEGRAM_TOKEN";
pub const VAULT_TOKEN_LABEL: &str = "telegram/bot_token";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum TelegramMode {
    #[default]
    Polling,
    Webhook,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token_env: String,
    pub allowed_chat_ids: Vec<i64>,
    pub allowed_user_ids: Vec<u64>,
    pub mode: TelegramMode,
    pub default_model: Option<String>,
    pub default_cwd: Option<PathBuf>,
    pub approval_policy: Option<AskForApproval>,
    pub sandbox_mode: Option<SandboxMode>,
    /// Connector-scoped identity instructions injected as developer
    /// instructions for Telegram sessions. `<cwd>` expands to the resolved
    /// session working directory. This replaces the deprecated workflow of
    /// seeding a default `AGENTS.md` into the Telegram workspace.
    pub identity_instructions: Option<String>,
    pub webhook_url: Option<String>,
    pub max_consecutive_polling_failures: u32,
    pub max_attachment_bytes: u32,
    pub media_retention_days: u64,
    pub max_media_store_bytes: u64,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token_env: DEFAULT_TOKEN_ENV.to_string(),
            allowed_chat_ids: Vec::new(),
            allowed_user_ids: Vec::new(),
            mode: TelegramMode::Polling,
            default_model: None,
            default_cwd: None,
            approval_policy: Some(AskForApproval::OnRequest),
            sandbox_mode: None,
            identity_instructions: None,
            webhook_url: None,
            max_consecutive_polling_failures: 8,
            max_attachment_bytes: 10 * 1024 * 1024,
            media_retention_days: 7,
            max_media_store_bytes: 256 * 1024 * 1024,
        }
    }
}

impl TelegramConfig {
    pub fn load_from_codex_home(codex_home: &Path) -> anyhow::Result<Self> {
        let path = codex_home.join("config.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Self::from_toml_str(&contents)
    }

    pub fn from_toml_str(contents: &str) -> anyhow::Result<Self> {
        let value =
            toml::from_str::<toml::Value>(contents).context("failed to parse config.toml")?;
        let Some(table) = value.get("telegram") else {
            return Ok(Self::default());
        };
        let config: Self = table
            .clone()
            .try_into()
            .context("failed to parse [telegram] config")?;
        anyhow::ensure!(
            config.max_attachment_bytes > 0,
            "telegram.max_attachment_bytes must be greater than zero"
        );
        anyhow::ensure!(
            config.media_retention_days > 0,
            "telegram.media_retention_days must be greater than zero"
        );
        anyhow::ensure!(
            config.max_media_store_bytes >= u64::from(config.max_attachment_bytes),
            "telegram.max_media_store_bytes must be at least max_attachment_bytes"
        );
        Ok(config)
    }

    pub fn allowlist(&self) -> ChatAllowlist {
        ChatAllowlist::with_users(self.allowed_chat_ids.clone(), self.allowed_user_ids.clone())
    }

    pub fn resolve_token(&self, codex_home: &Path) -> anyhow::Result<String> {
        self.resolve_token_with(
            |name| std::env::var(name).ok(),
            || {
                Vault::new(codex_home.to_path_buf())
                    .reveal(VAULT_TOKEN_LABEL)
                    .ok()
            },
        )
    }

    pub fn resolve_token_with(
        &self,
        env_lookup: impl Fn(&str) -> Option<String>,
        vault_lookup: impl Fn() -> Option<String>,
    ) -> anyhow::Result<String> {
        if let Some(token) =
            env_lookup(&self.bot_token_env).filter(|token| !token.trim().is_empty())
        {
            return Ok(token);
        }
        if let Some(token) = vault_lookup().filter(|token| !token.trim().is_empty()) {
            return Ok(token);
        }
        Err(TelegramError::MissingToken {
            env_var: self.bot_token_env.clone(),
        }
        .into())
    }
}
