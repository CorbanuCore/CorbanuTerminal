use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
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
#[serde(default)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token_env: String,
    pub allowed_chat_ids: Vec<i64>,
    pub mode: TelegramMode,
    pub default_model: Option<String>,
    pub default_cwd: Option<PathBuf>,
    pub approval_policy: Option<AskForApproval>,
    pub webhook_url: Option<String>,
    pub max_consecutive_polling_failures: u32,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token_env: DEFAULT_TOKEN_ENV.to_string(),
            allowed_chat_ids: Vec::new(),
            mode: TelegramMode::Polling,
            default_model: None,
            default_cwd: None,
            approval_policy: Some(AskForApproval::OnRequest),
            webhook_url: None,
            max_consecutive_polling_failures: 8,
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
        table
            .clone()
            .try_into()
            .context("failed to parse [telegram] config")
    }

    pub fn allowlist(&self) -> ChatAllowlist {
        ChatAllowlist::new(self.allowed_chat_ids.clone())
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
