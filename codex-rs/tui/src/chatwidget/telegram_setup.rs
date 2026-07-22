use std::path::PathBuf;

use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use ratatui::style::Stylize;
use ratatui::text::Line;

use super::*;
use crate::app_event::TelegramBotSecret;
use crate::bottom_pane::vault_secret_entry::VaultSecretEntryView;

mod api;
mod service;

pub(crate) use api::TelegramBotIdentity;
pub(crate) use api::TelegramChatCandidate;
pub(crate) use api::TelegramDiscovery;
pub(crate) use service::TelegramConnectionDefaults;
pub(crate) use service::TelegramStatus;
pub(crate) use service::connect_chat;
pub(crate) use service::disconnect;
pub(crate) use service::ensure_connector;
pub(crate) use service::start_connector;
pub(crate) use service::stop_connector;

const TELEGRAM_VIEW_ID: &str = "telegram-settings";
pub(crate) const TELEGRAM_DISCOVERY_VIEW_ID: &str = "telegram-discovery";

impl ChatWidget {
    pub(crate) fn open_telegram_menu(&mut self) {
        self.show_selection_view(telegram_menu_params(None));
        let codex_home = self.config.codex_home.clone().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = load_status(codex_home).await;
            tx.send(AppEvent::TelegramStatusReady { result });
        });
    }

    pub(crate) fn refresh_telegram_menu(&mut self, result: Result<TelegramStatus, String>) {
        let selected = self
            .bottom_pane
            .selected_index_for_active_view(TELEGRAM_VIEW_ID);
        let mut params = telegram_menu_params(Some(result));
        params.initial_selected_idx = selected;
        self.bottom_pane
            .replace_selection_view_if_present(TELEGRAM_VIEW_ID, params);
    }

    pub(crate) fn open_telegram_token_entry(&mut self) {
        let tx = self.app_event_tx.clone();
        let view = VaultSecretEntryView::new_fixed_secret(
            service::TOKEN_LABEL.to_string(),
            "Connect Telegram bot".to_string(),
            "BotFather token — masked".to_string(),
            "Paste the token from @BotFather. It is validated before encrypted storage."
                .to_string(),
            Box::new(move |_label, token| {
                tx.send(AppEvent::ValidateTelegramToken {
                    token: TelegramBotSecret::new(token),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn begin_telegram_discovery(
        &mut self,
        identity: Option<TelegramBotIdentity>,
    ) -> u64 {
        self.telegram_discovery_generation = self.telegram_discovery_generation.wrapping_add(1);
        let generation = self.telegram_discovery_generation;
        let params = telegram_discovery_params(identity, Vec::new());
        if self.bottom_pane.active_view_id() == Some(TELEGRAM_DISCOVERY_VIEW_ID) {
            self.bottom_pane
                .replace_selection_view_if_present(TELEGRAM_DISCOVERY_VIEW_ID, params);
        } else {
            self.show_selection_view(params);
        }
        generation
    }

    pub(crate) fn apply_telegram_discovery(
        &mut self,
        generation: u64,
        discovery: TelegramDiscovery,
    ) -> bool {
        if generation != self.telegram_discovery_generation
            || self.bottom_pane.active_view_id() != Some(TELEGRAM_DISCOVERY_VIEW_ID)
        {
            return false;
        }
        let should_continue = discovery.candidates.is_empty();
        self.bottom_pane.replace_selection_view_if_present(
            TELEGRAM_DISCOVERY_VIEW_ID,
            telegram_discovery_params(Some(discovery.identity), discovery.candidates),
        );
        should_continue
    }

    pub(crate) fn telegram_discovery_failed(&mut self, generation: u64, error: String) {
        if generation == self.telegram_discovery_generation
            && self.bottom_pane.active_view_id() == Some(TELEGRAM_DISCOVERY_VIEW_ID)
        {
            self.bottom_pane.replace_selection_view_if_present(
                TELEGRAM_DISCOVERY_VIEW_ID,
                telegram_discovery_error_params(error),
            );
        }
    }

    pub(crate) fn confirm_telegram_chat(&mut self, candidate: TelegramChatCandidate) {
        self.telegram_discovery_generation = self.telegram_discovery_generation.wrapping_add(1);
        let model = self.config.model.clone();
        let cwd = self.config.cwd.to_path_buf();
        let approval_policy =
            approval_policy_name(self.config.permissions.approval_policy.value()).to_string();
        let sandbox_mode = sandbox_mode_name(&self.config.legacy_sandbox_policy()).to_string();
        let selected = candidate.clone();
        self.show_selection_view(SelectionViewParams {
            title: Some("Authorize Telegram chat?".to_string()),
            subtitle: Some(format!(
                "{} ({}) will be able to send agent turns. Model: {} · Workspace: {} · Approvals: {} · Sandbox: {}.",
                candidate.display_name,
                candidate.chat_id,
                model.as_deref().unwrap_or("server default"),
                cwd.display(),
                approval_policy,
                sandbox_mode
            )),
            items: vec![SelectionItem {
                name: "Authorize and start connector".to_string(),
                description: Some(
                    "Persist this exact chat and sender, then start the background connector."
                        .to_string(),
                ),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::ConnectTelegramChat {
                        candidate: selected.clone(),
                        defaults: TelegramConnectionDefaults {
                            model: model.clone(),
                            cwd: cwd.clone(),
                            approval_policy: approval_policy.clone(),
                            sandbox_mode: sandbox_mode.clone(),
                        },
                    });
                })],
                dismiss_on_select: true,
                ..Default::default()
            }],
            ..Default::default()
        });
    }

    pub(crate) fn confirm_telegram_disconnect(&mut self) {
        self.telegram_discovery_generation = self.telegram_discovery_generation.wrapping_add(1);
        self.show_selection_view(SelectionViewParams {
            title: Some("Disconnect Telegram?".to_string()),
            subtitle: Some(
                "Stops the connector, removes Telegram authorization, and deletes the bot token from the vault."
                    .to_string(),
            ),
            items: vec![SelectionItem {
                name: "Disconnect and forget bot".to_string(),
                description: Some("The bot itself remains in Telegram and can be reconnected later.".to_string()),
                actions: vec![Box::new(|tx| tx.send(AppEvent::DisconnectTelegram))],
                dismiss_on_select: true,
                ..Default::default()
            }],
            ..Default::default()
        });
    }
}

fn telegram_discovery_params(
    identity: Option<TelegramBotIdentity>,
    candidates: Vec<TelegramChatCandidate>,
) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Connect Telegram".bold()));
    let bot = identity
        .as_ref()
        .map(|identity| format!("@{}", identity.username))
        .unwrap_or_else(|| "verified bot".to_string());
    header.push(Line::from(format!(
        "Bot: {bot} · Send /start in Telegram; this screen checks automatically."
    )));
    header.push(Line::from(
        "Only the chat you explicitly select will be authorized.".dim(),
    ));

    if candidates.is_empty() {
        header.push(Line::from(
            "Waiting for /start… The bot stays silent until you authorize the chat here.".dim(),
        ));
    }

    let mut items = Vec::new();
    for candidate in candidates {
        let selected = candidate.clone();
        items.push(SelectionItem {
            name: candidate.display_name,
            description: Some(format!(
                "{} · chat {} · sender {}",
                candidate.chat_kind, candidate.chat_id, candidate.actor_user_id
            )),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::ConfirmTelegramChat {
                    candidate: selected.clone(),
                });
            })],
            dismiss_on_select: true,
            ..Default::default()
        });
    }
    items.push(SelectionItem {
        name: "Return to Telegram settings".to_string(),
        description: Some("Stop waiting and return to connector settings.".to_string()),
        dismiss_on_select: true,
        ..Default::default()
    });

    SelectionViewParams {
        view_id: Some(TELEGRAM_DISCOVERY_VIEW_ID),
        header: Box::new(header),
        items,
        ..Default::default()
    }
}

fn telegram_discovery_error_params(error: String) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Connect Telegram".bold()));
    header.push(Line::from(format!("Chat discovery stopped: {error}")).red());
    SelectionViewParams {
        view_id: Some(TELEGRAM_DISCOVERY_VIEW_ID),
        header: Box::new(header),
        items: vec![
            SelectionItem {
                name: "Retry chat discovery".to_string(),
                description: Some("Resume waiting for a message to this bot.".to_string()),
                actions: vec![Box::new(|tx| tx.send(AppEvent::DiscoverTelegramChats))],
                dismiss_on_select: false,
                ..Default::default()
            },
            SelectionItem {
                name: "Return to Telegram settings".to_string(),
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn telegram_menu_params(result: Option<Result<TelegramStatus, String>>) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Telegram".bold()));
    let mut items = Vec::new();
    match result {
        None => header.push(Line::from("Checking connector status…".dim())),
        Some(Err(error)) => {
            header.push(Line::from(format!("Unavailable: {error}").red()));
            items.push(refresh_item());
        }
        Some(Ok(status)) if !status.configured => {
            if status.token_stored {
                let bot = status
                    .bot_username
                    .as_deref()
                    .map(|username| format!("@{username}"))
                    .unwrap_or_else(|| "Verified bot".to_string());
                header.push(Line::from(format!(
                    "{bot} · waiting for chat authorization"
                )));
                items.push(SelectionItem {
                    name: "Finish connection".to_string(),
                    description: Some(
                        "Send /start to the bot, select that chat, and start the connector."
                            .to_string(),
                    ),
                    actions: vec![Box::new(|tx| tx.send(AppEvent::DiscoverTelegramChats))],
                    dismiss_on_select: false,
                    ..Default::default()
                });
                items.push(SelectionItem {
                    name: "Replace bot token".to_string(),
                    description: Some("Validate a different token from @BotFather.".to_string()),
                    actions: vec![Box::new(|tx| tx.send(AppEvent::OpenTelegramTokenEntry))],
                    dismiss_on_select: true,
                    ..Default::default()
                });
                items.push(SelectionItem {
                    name: "Forget bot".to_string(),
                    description: Some("Delete this token without authorizing a chat.".to_string()),
                    actions: vec![Box::new(|tx| tx.send(AppEvent::ConfirmTelegramDisconnect))],
                    dismiss_on_select: true,
                    ..Default::default()
                });
            } else {
                header.push(Line::from("Not connected".dim()));
                items.push(SelectionItem {
                    name: "Connect Telegram bot".to_string(),
                    description: Some(
                        "Paste a BotFather token, verify it, then authorize a chat by messaging the bot."
                            .to_string(),
                    ),
                    actions: vec![Box::new(|tx| tx.send(AppEvent::OpenTelegramTokenEntry))],
                    dismiss_on_select: true,
                    ..Default::default()
                });
            }
            items.push(refresh_item());
        }
        Some(Ok(status)) => {
            let runtime = if status.running {
                format!("running · PID {}", status.pid.unwrap_or_default())
            } else {
                "stopped".to_string()
            };
            let bot = status
                .bot_username
                .as_deref()
                .map(|username| format!("@{username}"))
                .unwrap_or_else(|| "bot identity unavailable".to_string());
            header.push(Line::from(format!("{bot} · {runtime}")));
            if !status.token_stored {
                header.push(Line::from(
                    "Bot token is missing from the vault; replace or disconnect this bot.".red(),
                ));
            }
            header.push(Line::from(format!(
                "Chats: {} · Model: {}",
                join_chat_ids(&status.allowed_chat_ids),
                status.default_model.as_deref().unwrap_or("server default"),
            )));
            header.push(Line::from(format!(
                "Workspace: {}",
                status
                    .default_cwd
                    .as_deref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "current directory".to_string()),
            )));
            header.push(Line::from(format!(
                "Approvals: {} · Sandbox: {}",
                status.approval_policy.as_deref().unwrap_or("on-request"),
                status
                    .sandbox_mode
                    .as_deref()
                    .unwrap_or("configured default")
            )));
            items.push(SelectionItem {
                name: if status.running {
                    "Restart connector".to_string()
                } else {
                    "Start connector".to_string()
                },
                description: Some("Run the configured bot in the background.".to_string()),
                actions: vec![Box::new(|tx| tx.send(AppEvent::StartTelegramConnector))],
                dismiss_on_select: true,
                ..Default::default()
            });
            if status.running {
                items.push(SelectionItem {
                    name: "Stop connector".to_string(),
                    description: Some("Keep its token and authorized chat for later.".to_string()),
                    actions: vec![Box::new(|tx| tx.send(AppEvent::StopTelegramConnector))],
                    dismiss_on_select: true,
                    ..Default::default()
                });
            }
            items.push(SelectionItem {
                name: "Replace bot".to_string(),
                description: Some(
                    "Validate a different BotFather token and choose its chat.".to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::ReplaceTelegramBot))],
                dismiss_on_select: true,
                ..Default::default()
            });
            items.push(SelectionItem {
                name: "Disconnect Telegram".to_string(),
                description: Some(
                    "Stop it and delete its local token and authorization.".to_string(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::ConfirmTelegramDisconnect))],
                dismiss_on_select: true,
                ..Default::default()
            });
            items.push(refresh_item());
        }
    }
    SelectionViewParams {
        view_id: Some(TELEGRAM_VIEW_ID),
        header: Box::new(header),
        items,
        ..Default::default()
    }
}

fn refresh_item() -> SelectionItem {
    SelectionItem {
        name: "Refresh".to_string(),
        description: Some("Reload configuration and connector health.".to_string()),
        actions: vec![Box::new(|tx| tx.send(AppEvent::OpenTelegram))],
        dismiss_on_select: false,
        ..Default::default()
    }
}

pub(crate) async fn validate_and_store_token(
    codex_home: PathBuf,
    token: TelegramBotSecret,
) -> Result<TelegramBotIdentity, String> {
    let token = token.into_inner();
    let identity = api::validate_token(&token).await?;
    tokio::task::spawn_blocking(move || service::store_token(&codex_home, token))
        .await
        .map_err(|error| format!("Telegram vault task failed: {error}"))??;
    Ok(identity)
}

pub(crate) async fn load_status(codex_home: PathBuf) -> Result<TelegramStatus, String> {
    let status_home = codex_home.clone();
    let mut status = tokio::task::spawn_blocking(move || service::read_status(&status_home))
        .await
        .map_err(|error| format!("Telegram status task failed: {error}"))??;
    if status.token_stored {
        let token_home = codex_home;
        let token = tokio::task::spawn_blocking(move || service::reveal_token(token_home))
            .await
            .map_err(|error| format!("Telegram vault task failed: {error}"))??;
        if let Ok(identity) = api::telegram_identity(&token).await {
            status.bot_username = Some(identity.username);
        }
    }
    Ok(status)
}

pub(crate) async fn discover_chats(codex_home: PathBuf) -> Result<TelegramDiscovery, String> {
    let token = tokio::task::spawn_blocking(move || service::reveal_token(codex_home))
        .await
        .map_err(|error| format!("Telegram vault task failed: {error}"))??;
    api::discover(&token).await
}

fn approval_policy_name(policy: AskForApproval) -> &'static str {
    match policy {
        AskForApproval::UnlessTrusted => "untrusted",
        AskForApproval::OnFailure => "on-failure",
        AskForApproval::OnRequest => "on-request",
        AskForApproval::Granular(_) => "on-request",
        AskForApproval::Never => "never",
    }
}

fn sandbox_mode_name(policy: &SandboxPolicy) -> &'static str {
    match policy {
        SandboxPolicy::DangerFullAccess => "danger-full-access",
        SandboxPolicy::ReadOnly { .. } | SandboxPolicy::ExternalSandbox { .. } => "read-only",
        SandboxPolicy::WorkspaceWrite { .. } => "workspace-write",
    }
}

fn join_chat_ids(chat_ids: &[i64]) -> String {
    if chat_ids.is_empty() {
        "none".to_string()
    } else {
        chat_ids
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}
