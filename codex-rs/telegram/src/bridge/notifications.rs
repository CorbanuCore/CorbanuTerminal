use std::time::Instant;

use anyhow::Context;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::TurnStatus;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::Requester;
use teloxide::types::ChatAction;
use teloxide::types::ParseMode;
use tracing::warn;

use super::BridgeRuntime;
use crate::render::render_html_chunks;
use crate::session::StreamUpdate;

impl BridgeRuntime {
    pub(super) async fn handle_notification(
        &mut self,
        notification: ServerNotification,
    ) -> anyhow::Result<()> {
        match notification {
            ServerNotification::TurnStarted(notification) => {
                if let Some(chat_id) = self.sessions.chat_for_thread(&notification.thread_id).await
                {
                    let _ = self.bot.send_chat_action(chat_id, ChatAction::Typing).await;
                }
                Ok(())
            }
            ServerNotification::AgentMessageDelta(notification) => {
                if let Some(update) = self
                    .sessions
                    .append_stream_delta(
                        &notification.thread_id,
                        &notification.item_id,
                        &notification.delta,
                        Instant::now(),
                    )
                    .await
                {
                    self.apply_stream_update(update).await?;
                }
                Ok(())
            }
            ServerNotification::ItemCompleted(notification) => {
                self.handle_item_completed(
                    &notification.thread_id,
                    &notification.turn_id,
                    notification.item,
                )
                .await
            }
            ServerNotification::TurnCompleted(notification) => {
                let thread_id = notification.thread_id;
                let status = notification.turn.status;
                if let Some(chat_id) = self.sessions.chat_for_thread(&thread_id).await {
                    match status {
                        TurnStatus::Completed => {}
                        TurnStatus::Interrupted => {
                            self.send_text(chat_id, "Turn interrupted.").await?;
                        }
                        TurnStatus::Failed => {
                            self.send_text(chat_id, "Turn failed.").await?;
                        }
                        TurnStatus::InProgress => {}
                    }
                }
                self.sessions.clear_turn_for_thread(&thread_id).await;
                Ok(())
            }
            ServerNotification::Warning(notification) => {
                if let Some(thread_id) = notification.thread_id
                    && let Some(chat_id) = self.sessions.chat_for_thread(&thread_id).await
                {
                    self.send_text(chat_id, &format!("Warning: {}", notification.message))
                        .await?;
                }
                Ok(())
            }
            ServerNotification::GuardianWarning(notification) => {
                if let Some(chat_id) = self.sessions.chat_for_thread(&notification.thread_id).await
                {
                    self.send_text(chat_id, &format!("Warning: {}", notification.message))
                        .await?;
                }
                Ok(())
            }
            ServerNotification::Error(notification) => {
                if let Some(chat_id) = self.sessions.chat_for_thread(&notification.thread_id).await
                {
                    self.send_text(chat_id, &format!("Error: {}", notification.error))
                        .await?;
                }
                Ok(())
            }
            ServerNotification::ConfigWarning(notification) => {
                let message = notification
                    .details
                    .map(|details| format!("Config warning: {} ({details})", notification.summary))
                    .unwrap_or_else(|| format!("Config warning: {}", notification.summary));
                self.broadcast(&message).await
            }
            ServerNotification::DeprecationNotice(notification) => {
                self.broadcast(&format!("Deprecated: {}", notification.summary))
                    .await
            }
            ServerNotification::ThreadStarted(_)
            | ServerNotification::ThreadStatusChanged(_)
            | ServerNotification::ThreadArchived(_)
            | ServerNotification::ThreadDeleted(_)
            | ServerNotification::ThreadUnarchived(_)
            | ServerNotification::ThreadClosed(_)
            | ServerNotification::SkillsChanged(_)
            | ServerNotification::ThreadNameUpdated(_)
            | ServerNotification::ThreadGoalUpdated(_)
            | ServerNotification::ThreadGoalCleared(_)
            | ServerNotification::ThreadSettingsUpdated(_)
            | ServerNotification::ThreadTokenUsageUpdated(_)
            | ServerNotification::HookStarted(_)
            | ServerNotification::HookCompleted(_)
            | ServerNotification::TurnDiffUpdated(_)
            | ServerNotification::TurnPlanUpdated(_)
            | ServerNotification::ItemStarted(_)
            | ServerNotification::ItemGuardianApprovalReviewStarted(_)
            | ServerNotification::ItemGuardianApprovalReviewCompleted(_)
            | ServerNotification::RawResponseItemCompleted(_)
            | ServerNotification::PlanDelta(_)
            | ServerNotification::CommandExecOutputDelta(_)
            | ServerNotification::ProcessOutputDelta(_)
            | ServerNotification::ProcessExited(_)
            | ServerNotification::CommandExecutionOutputDelta(_)
            | ServerNotification::TerminalInteraction(_)
            | ServerNotification::FileChangeOutputDelta(_)
            | ServerNotification::FileChangePatchUpdated(_)
            | ServerNotification::ServerRequestResolved(_)
            | ServerNotification::McpToolCallProgress(_)
            | ServerNotification::McpServerOauthLoginCompleted(_)
            | ServerNotification::McpServerStatusUpdated(_)
            | ServerNotification::AccountUpdated(_)
            | ServerNotification::AccountRateLimitsUpdated(_)
            | ServerNotification::AppListUpdated(_)
            | ServerNotification::RemoteControlStatusChanged(_)
            | ServerNotification::ExternalAgentConfigImportProgress(_)
            | ServerNotification::ExternalAgentConfigImportCompleted(_)
            | ServerNotification::FsChanged(_)
            | ServerNotification::ReasoningSummaryTextDelta(_)
            | ServerNotification::ReasoningSummaryPartAdded(_)
            | ServerNotification::ReasoningTextDelta(_)
            | ServerNotification::ContextCompacted(_)
            | ServerNotification::ModelRerouted(_)
            | ServerNotification::ModelVerification(_)
            | ServerNotification::TurnModerationMetadata(_)
            | ServerNotification::FuzzyFileSearchSessionUpdated(_)
            | ServerNotification::FuzzyFileSearchSessionCompleted(_)
            | ServerNotification::ThreadRealtimeStarted(_)
            | ServerNotification::ThreadRealtimeItemAdded(_)
            | ServerNotification::ThreadRealtimeTranscriptDelta(_)
            | ServerNotification::ThreadRealtimeTranscriptDone(_)
            | ServerNotification::ThreadRealtimeOutputAudioDelta(_)
            | ServerNotification::ThreadRealtimeSdp(_)
            | ServerNotification::ThreadRealtimeError(_)
            | ServerNotification::ThreadRealtimeClosed(_)
            | ServerNotification::WindowsWorldWritableWarning(_)
            | ServerNotification::WindowsSandboxSetupCompleted(_)
            | ServerNotification::AccountLoginCompleted(_) => Ok(()),
        }
    }

    async fn handle_item_completed(
        &mut self,
        thread_id: &str,
        _turn_id: &str,
        item: ThreadItem,
    ) -> anyhow::Result<()> {
        match item {
            ThreadItem::AgentMessage { id, text, .. } => {
                let stream = self.sessions.take_stream_for_item(thread_id, &id).await;
                if let Some((chat_id, Some(message_id))) = stream {
                    self.edit_message(chat_id, message_id, &text).await?;
                    let chunks = render_html_chunks(&text);
                    for chunk in chunks.into_iter().skip(1) {
                        self.send_html(chat_id, &chunk.html).await?;
                    }
                } else if let Some(chat_id) = self.sessions.chat_for_thread(thread_id).await {
                    self.send_text(chat_id, &text).await?;
                }
                Ok(())
            }
            ThreadItem::CommandExecution {
                command, status, ..
            } => {
                if let Some(chat_id) = self.sessions.chat_for_thread(thread_id).await {
                    self.send_text(chat_id, &format!("Command {status:?}: {command}"))
                        .await?;
                }
                Ok(())
            }
            ThreadItem::FileChange { status, .. } => {
                if let Some(chat_id) = self.sessions.chat_for_thread(thread_id).await {
                    self.send_text(chat_id, &format!("File change {status:?}."))
                        .await?;
                }
                Ok(())
            }
            ThreadItem::McpToolCall {
                server,
                tool,
                status,
                ..
            } => {
                if let Some(chat_id) = self.sessions.chat_for_thread(thread_id).await {
                    self.send_text(chat_id, &format!("MCP {server}/{tool} {status:?}."))
                        .await?;
                }
                Ok(())
            }
            ThreadItem::UserMessage { .. }
            | ThreadItem::HookPrompt { .. }
            | ThreadItem::Plan { .. }
            | ThreadItem::Reasoning { .. }
            | ThreadItem::DynamicToolCall { .. }
            | ThreadItem::CollabAgentToolCall { .. }
            | ThreadItem::SubAgentActivity { .. }
            | ThreadItem::WebSearch { .. }
            | ThreadItem::ImageView { .. }
            | ThreadItem::ImageGeneration { .. }
            | ThreadItem::EnteredReviewMode { .. }
            | ThreadItem::ExitedReviewMode { .. }
            | ThreadItem::ContextCompaction { .. }
            | ThreadItem::Sleep { .. } => Ok(()),
        }
    }

    pub(super) async fn handle_lagged(&mut self, skipped: u64) -> anyhow::Result<()> {
        warn!(skipped, "Telegram bridge lagged behind app-server events");
        for thread_id in self.sessions.thread_ids().await {
            let request_id = self.request_ids.next();
            let _: ThreadReadResponse = self
                .request_typed(
                    ClientRequest::ThreadRead {
                        request_id,
                        params: ThreadReadParams {
                            thread_id: thread_id.clone(),
                            include_turns: true,
                        },
                    },
                    "thread/read",
                )
                .await?;
            if let Some(chat_id) = self.sessions.chat_for_thread(&thread_id).await {
                self.send_text(
                    chat_id,
                    &format!("Recovered after missing {skipped} app-server events."),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn apply_stream_update(&self, update: StreamUpdate) -> anyhow::Result<()> {
        match update.message_id {
            Some(message_id) => {
                self.bot
                    .edit_message_text(update.chat_id, message_id, update.html)
                    .parse_mode(ParseMode::Html)
                    .await
                    .context("edit Telegram streaming message")?;
            }
            None => {
                let message = self.send_html(update.chat_id, &update.html).await?;
                self.sessions
                    .set_stream_message(update.chat_id, message.id)
                    .await;
            }
        }
        Ok(())
    }

    async fn broadcast(&self, text: &str) -> anyhow::Result<()> {
        for chat_id in self.sessions.chat_ids_with_threads().await {
            self.send_text(chat_id, text).await?;
        }
        Ok(())
    }
}
