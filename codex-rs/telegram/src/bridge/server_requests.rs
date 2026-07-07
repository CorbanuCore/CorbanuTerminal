use anyhow::Context;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerRequest;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::types::MessageId;
use teloxide::types::ParseMode;
use tracing::warn;

use super::BridgeRuntime;
use crate::approvals::ApprovalCallback;
use crate::approvals::PendingApproval;
use crate::approvals::PendingApprovalKind;

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
        let Some(chat_id) = self.sessions.chat_for_thread(approval.thread_id()).await else {
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
            .insert_pending_approval(chat_id, approval)
            .await;
        let result = self
            .bot
            .send_message(chat_id, message)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await;
        match result {
            Ok(message) => {
                self.sessions
                    .set_pending_approval_message(chat_id, &request_id, message.id)
                    .await;
            }
            Err(err) => {
                self.sessions
                    .remove_pending_approval(chat_id, &request_id)
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
        chat_id: ChatId,
        callback: ApprovalCallback,
    ) -> anyhow::Result<String> {
        let Some(pending) = self
            .sessions
            .pending_approval(chat_id, &callback.request_id)
            .await
        else {
            return Ok("Approval is not pending for this chat.".to_string());
        };
        let value = match pending.approval.resolve_value(callback.decision_index) {
            Ok(value) => value,
            Err(err) => {
                warn!("rejecting unavailable Telegram approval decision: {err}");
                return Ok("Approval decision is not available for this request.".to_string());
            }
        };
        let response = pending.approval.response_text(callback.decision_index)?;

        let Some(record) = self
            .sessions
            .take_pending_approval(chat_id, &callback.request_id)
            .await
        else {
            return Ok("Approval is not pending for this chat.".to_string());
        };

        self.client
            .resolve_server_request(callback.request_id, value)
            .await
            .context("resolve server request")?;
        if let Some(message_id) = record.message_id
            && let Err(err) = self
                .clear_approval_keyboard(record.chat_id, message_id)
                .await
        {
            warn!("failed to clear Telegram approval keyboard: {err}");
        }
        self.send_text(record.chat_id, response).await?;
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
        chat_id: ChatId,
        message_id: MessageId,
    ) -> anyhow::Result<()> {
        self.bot
            .edit_message_reply_markup(chat_id, message_id)
            .await
            .context("clear Telegram approval keyboard")?;
        Ok(())
    }
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
