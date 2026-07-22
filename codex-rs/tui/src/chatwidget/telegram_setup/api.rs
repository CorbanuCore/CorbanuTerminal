use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;

const TELEGRAM_API_ROOT: &str = "https://api.telegram.org";
const API_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TelegramBotIdentity {
    pub(crate) id: u64,
    pub(crate) username: String,
    pub(crate) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TelegramChatCandidate {
    pub(crate) chat_id: i64,
    pub(crate) actor_user_id: u64,
    pub(crate) display_name: String,
    pub(crate) chat_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TelegramDiscovery {
    pub(crate) identity: TelegramBotIdentity,
    pub(crate) candidates: Vec<TelegramChatCandidate>,
}

#[derive(Debug, Deserialize)]
struct TelegramResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramUser {
    id: u64,
    first_name: String,
    #[serde(default)]
    username: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WebhookInfo {
    #[serde(default)]
    url: String,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    chat: TelegramChat,
    from: Option<TelegramUser>,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
    #[serde(rename = "type")]
    kind: String,
    first_name: Option<String>,
    last_name: Option<String>,
    title: Option<String>,
    username: Option<String>,
}

pub(crate) async fn validate_token(token: &str) -> Result<TelegramBotIdentity, String> {
    let identity = telegram_identity(token).await?;
    let webhook: WebhookInfo = telegram_call(token, "getWebhookInfo", None).await?;
    if !webhook.url.is_empty() {
        return Err(
            "This bot currently uses a Telegram webhook. Remove that webhook before connecting it to PFTerminal polling."
                .to_string(),
        );
    }
    Ok(identity)
}

pub(crate) async fn telegram_identity(token: &str) -> Result<TelegramBotIdentity, String> {
    telegram_identity_at(TELEGRAM_API_ROOT, token).await
}

pub(crate) async fn discover(token: &str) -> Result<TelegramDiscovery, String> {
    discover_at(TELEGRAM_API_ROOT, token).await
}

async fn discover_at(api_root: &str, token: &str) -> Result<TelegramDiscovery, String> {
    let identity = telegram_identity_at(api_root, token).await?;
    let updates: Vec<TelegramUpdate> = telegram_call_at(
        api_root,
        token,
        "getUpdates",
        Some(json!({"timeout": 10, "allowed_updates": ["message"]})),
    )
    .await?;
    let mut candidates = BTreeMap::new();
    for update in &updates {
        let Some(message) = &update.message else {
            continue;
        };
        let Some(actor) = &message.from else {
            continue;
        };
        candidates.insert(
            (message.chat.id, actor.id),
            TelegramChatCandidate {
                chat_id: message.chat.id,
                actor_user_id: actor.id,
                display_name: chat_display_name(&message.chat, actor),
                chat_kind: message.chat.kind.clone(),
            },
        );
    }
    Ok(TelegramDiscovery {
        identity,
        candidates: candidates.into_values().collect(),
    })
}

async fn telegram_call<T: for<'de> Deserialize<'de>>(
    token: &str,
    method: &str,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    telegram_call_at(TELEGRAM_API_ROOT, token, method, body).await
}

async fn telegram_identity_at(api_root: &str, token: &str) -> Result<TelegramBotIdentity, String> {
    let user: TelegramUser = telegram_call_at(api_root, token, "getMe", None).await?;
    let username = user
        .username
        .ok_or_else(|| "Telegram returned a bot without a username.".to_string())?;
    Ok(TelegramBotIdentity {
        id: user.id,
        username,
        display_name: user.first_name,
    })
}

async fn telegram_call_at<T: for<'de> Deserialize<'de>>(
    api_root: &str,
    token: &str,
    method: &str,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    let client = reqwest::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .map_err(|_| "Could not initialize Telegram networking.".to_string())?;
    let url = format!("{}/bot{token}/{method}", api_root.trim_end_matches('/'));
    let request = client.post(url);
    let response = match body {
        Some(body) => request.json(&body),
        None => request,
    }
    .send()
    .await
    .map_err(|_| "Telegram could not be reached. Check the network and retry.".to_string())?;
    let status = response.status();
    let payload = response
        .json::<TelegramResponse<T>>()
        .await
        .map_err(|_| "Telegram returned an unreadable response.".to_string())?;
    if !status.is_success() || !payload.ok {
        let description = payload.description.unwrap_or_default();
        if status.as_u16() == 401 {
            return Err("BotFather rejected this token. Paste the current bot token.".to_string());
        }
        if description.to_ascii_lowercase().contains("conflict") {
            return Err(
                "Another process is already polling this Telegram bot. Stop that connector and retry."
                    .to_string(),
            );
        }
        return Err(format!(
            "Telegram rejected the request (HTTP {}).",
            status.as_u16()
        ));
    }
    payload
        .result
        .ok_or_else(|| "Telegram response did not contain a result.".to_string())
}

fn chat_display_name(chat: &TelegramChat, actor: &TelegramUser) -> String {
    if let Some(title) = chat
        .title
        .as_deref()
        .filter(|title| !title.trim().is_empty())
    {
        return title.to_string();
    }
    let mut name = [chat.first_name.as_deref(), chat.last_name.as_deref()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    if name.trim().is_empty() {
        name = chat
            .username
            .as_deref()
            .or(actor.username.as_deref())
            .map(|username| format!("@{username}"))
            .unwrap_or_else(|| format!("Telegram chat {}", chat.id));
    }
    name
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
