use anyhow::Context;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerRequest;
use std::future::Future;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::MessageId;
use teloxide::types::ParseMode;
use teloxide::types::UserId;
use tracing::warn;

use super::BridgeRuntime;
use crate::approvals::ApprovalCallback;
use crate::approvals::PendingApproval;
use crate::approvals::PendingApprovalKind;
use crate::conversation::ConversationKey;
use crate::session::PendingApprovalRecord;
use crate::session::SessionStore;

impl BridgeRuntime {
    pub(super) async fn handle_server_request(
        &mut self,
        request: ServerRequest,
    ) -> anyhow::Result<()> {
        match request {
            ServerRequest::CommandExecutionRequestApproval { request_id, params } => {
                self.queue_approval(
                    request_id,
                    PendingApprovalKind::Command(sanitize_command_params(params)),
                )
                .await
            }
            ServerRequest::FileChangeRequestApproval { request_id, params } => {
                self.queue_approval(request_id, PendingApprovalKind::FileChange(params))
                    .await
            }
            ServerRequest::PermissionsRequestApproval { request_id, params } => {
                self.queue_approval(request_id, PendingApprovalKind::Permissions(params))
                    .await
            }
            ServerRequest::ToolRequestUserInput { request_id, params } => {
                self.reject_request(
                    request_id,
                    format!(
                        "tool user input is not supported from Telegram for thread `{}`",
                        params.thread_id
                    ),
                )
                .await
            }
            ServerRequest::McpServerElicitationRequest { request_id, .. } => {
                self.reject_request(
                    request_id,
                    "MCP elicitation is not supported from Telegram".to_string(),
                )
                .await
            }
            ServerRequest::DynamicToolCall { request_id, params } => {
                self.reject_request(
                    request_id,
                    format!(
                        "dynamic tool calls are not supported from Telegram for thread `{}`",
                        params.thread_id
                    ),
                )
                .await
            }
            ServerRequest::ChatgptAuthTokensRefresh { request_id, .. } => {
                self.reject_request(
                    request_id,
                    "ChatGPT auth token refresh is not supported from Telegram".to_string(),
                )
                .await
            }
            ServerRequest::AttestationGenerate { request_id, .. } => {
                self.reject_request(
                    request_id,
                    "attestation generation is not supported from Telegram".to_string(),
                )
                .await
            }
            ServerRequest::CurrentTimeRead { request_id, .. } => {
                self.reject_request(
                    request_id,
                    "external current time is not supported from Telegram".to_string(),
                )
                .await
            }
            ServerRequest::ApplyPatchApproval { request_id, params } => {
                self.reject_request(
                    request_id,
                    format!(
                        "legacy apply_patch approval is not supported for thread `{}`",
                        params.conversation_id
                    ),
                )
                .await
            }
            ServerRequest::ExecCommandApproval { request_id, params } => {
                self.reject_request(
                    request_id,
                    format!(
                        "legacy exec approval is not supported for thread `{}`",
                        params.conversation_id
                    ),
                )
                .await
            }
        }
    }

    async fn queue_approval(
        &mut self,
        request_id: RequestId,
        kind: PendingApprovalKind,
    ) -> anyhow::Result<()> {
        let approval = PendingApproval {
            request_id: request_id.clone(),
            kind,
        };
        let Some(conversation) = self
            .sessions
            .conversation_for_thread(approval.thread_id())
            .await
        else {
            self.reject_request(
                approval.request_id.clone(),
                format!("no Telegram chat owns thread `{}`", approval.thread_id()),
            )
            .await?;
            return Ok(());
        };
        let message = approval.message();
        let keyboard = approval.keyboard();
        self.sessions
            .insert_pending_approval(conversation, approval)
            .await;
        let bot = self.bot.clone();
        let result = crate::outbound::call_with_policy(
            crate::outbound::CallSafety::Mutating,
            crate::outbound::DEFAULT_API_TIMEOUT,
            "telegram approval prompt",
            move || {
                let bot = bot.clone();
                let message = message.clone();
                let keyboard = keyboard.clone();
                async move {
                    let mut request = bot
                        .send_message(conversation.chat_id, message)
                        .parse_mode(ParseMode::Html)
                        .reply_markup(keyboard);
                    if let Some(thread_id) = conversation.thread_id {
                        request = request.message_thread_id(thread_id);
                    }
                    request.await
                }
            },
        )
        .await;
        match result {
            Ok(message) => {
                self.sessions
                    .set_pending_approval_message(conversation, &request_id, message.id)
                    .await;
            }
            Err(err) => {
                self.sessions
                    .remove_pending_approval(conversation, &request_id)
                    .await;
                if let Err(reject_err) = self
                    .reject_request(
                        request_id,
                        "failed to send Telegram approval prompt".to_string(),
                    )
                    .await
                {
                    warn!(
                        "failed to reject app-server approval after Telegram send failure: {reject_err}"
                    );
                }
                return Err(err).context("send Telegram approval prompt");
            }
        }
        Ok(())
    }

    pub(super) async fn resolve_approval(
        &mut self,
        conversation: ConversationKey,
        actor_user_id: UserId,
        callback: ApprovalCallback,
    ) -> anyhow::Result<String> {
        let Some(pending) = self
            .sessions
            .pending_approval(conversation, &callback.request_id)
            .await
        else {
            return Ok("Approval is not pending for this chat.".to_string());
        };
        if pending
            .actor_user_id
            .is_some_and(|expected| expected != actor_user_id)
        {
            return Ok("This approval belongs to the user who started the turn.".to_string());
        }
        let value = match pending.approval.resolve_value(callback.decision_index) {
            Ok(value) => value,
            Err(err) => {
                warn!("rejecting unavailable Telegram approval decision: {err}");
                return Ok("Approval decision is not available for this request.".to_string());
            }
        };
        let response = pending.approval.response_text(callback.decision_index)?;

        // Keep the approval available while the in-process request is in
        // flight. Removing it first turns a recoverable app-server failure
        // into a permanently stuck turn. Bridge commands are serialized, so
        // no second callback can consume this record concurrently.
        let record = resolve_then_take_pending(
            &self.sessions,
            conversation,
            &callback.request_id,
            self.client
                .resolve_server_request(callback.request_id.clone(), value),
        )
        .await?
        .unwrap_or(pending);
        if let Some(message_id) = record.message_id
            && let Err(err) = self
                .clear_approval_keyboard(record.conversation, message_id)
                .await
        {
            warn!("failed to clear Telegram approval keyboard: {err}");
        }
        self.notify_after_effect(
            record.conversation,
            response,
            "approval-decision confirmation",
        )
        .await;
        Ok(response.to_string())
    }

    async fn reject_request(
        &mut self,
        request_id: RequestId,
        reason: String,
    ) -> anyhow::Result<()> {
        self.client
            .reject_server_request(
                request_id,
                JSONRPCErrorError {
                    code: -32000,
                    message: reason,
                    data: None,
                },
            )
            .await
            .context("reject server request")
    }

    async fn clear_approval_keyboard(
        &self,
        conversation: ConversationKey,
        message_id: MessageId,
    ) -> anyhow::Result<()> {
        let bot = self.bot.clone();
        crate::outbound::call_with_policy(
            crate::outbound::CallSafety::Mutating,
            crate::outbound::DEFAULT_API_TIMEOUT,
            "telegram clear approval keyboard",
            move || {
                let bot = bot.clone();
                async move {
                    bot.edit_message_reply_markup(conversation.chat_id, message_id)
                        .await
                }
            },
        )
        .await
        .context("clear Telegram approval keyboard")?;
        Ok(())
    }
}

async fn resolve_then_take_pending<F, E>(
    sessions: &SessionStore,
    conversation: ConversationKey,
    request_id: &RequestId,
    resolution: F,
) -> anyhow::Result<Option<PendingApprovalRecord>>
where
    F: Future<Output = Result<(), E>>,
    E: Into<anyhow::Error>,
{
    resolution
        .await
        .map_err(Into::into)
        .context("resolve server request")?;
    Ok(sessions
        .take_pending_approval(conversation, request_id)
        .await)
}

fn sanitize_command_params(
    mut params: CommandExecutionRequestApprovalParams,
) -> CommandExecutionRequestApprovalParams {
    if let Some(command) = params.command.as_mut() {
        *command = truncate_command_for_prompt(command);
    }
    params
}

fn truncate_command_for_prompt(text: &str) -> String {
    const MAX: usize = 2048;
    if text.chars().count() <= MAX {
        return text.to_string();
    }
    let mut truncated = text.chars().take(MAX).collect::<String>();
    truncated.push_str("...(truncated)");
    truncated
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
    use teloxide::types::ChatId;

    use super::*;

    #[tokio::test]
    async fn failed_resolution_preserves_pending_approval_for_retry() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let codex_home = std::env::temp_dir().join(format!(
            "codex-telegram-approval-resolution-{}-{suffix}",
            std::process::id()
        ));
        std::fs::create_dir_all(&codex_home).unwrap();
        let sessions = SessionStore::load(&codex_home).await.unwrap();
        let conversation = ConversationKey::from(ChatId(10));
        let request_id = RequestId::Integer(9);
        sessions
            .insert_pending_approval(
                conversation,
                PendingApproval {
                    request_id: request_id.clone(),
                    kind: PendingApprovalKind::Command(CommandExecutionRequestApprovalParams {
                        thread_id: "thread".into(),
                        turn_id: "turn".into(),
                        item_id: "item".into(),
                        started_at_ms: 1,
                        approval_id: None,
                        environment_id: None,
                        reason: None,
                        network_approval_context: None,
                        command: Some("true".into()),
                        cwd: None,
                        command_actions: None,
                        additional_permissions: None,
                        proposed_execpolicy_amendment: None,
                        proposed_network_policy_amendments: None,
                        available_decisions: None,
                    }),
                },
            )
            .await;

        let error = resolve_then_take_pending(&sessions, conversation, &request_id, async {
            anyhow::bail!("transient app-server failure")
        })
        .await
        .unwrap_err();
        assert!(error.to_string().contains("resolve server request"));
        assert!(
            sessions
                .pending_approval(conversation, &request_id)
                .await
                .is_some()
        );

        let consumed = resolve_then_take_pending(&sessions, conversation, &request_id, async {
            Ok::<(), anyhow::Error>(())
        })
        .await
        .unwrap();
        assert!(consumed.is_some());
        assert!(
            sessions
                .pending_approval(conversation, &request_id)
                .await
                .is_none()
        );
        std::fs::remove_dir_all(codex_home).unwrap();
    }
}
