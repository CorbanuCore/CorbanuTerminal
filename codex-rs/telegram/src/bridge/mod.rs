use std::sync::Arc;

use anyhow::Context;
use codex_app_server_client::InProcessAppServerClient;
use codex_app_server_client::InProcessServerEvent;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerRequest;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadSource;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadUnsubscribeParams;
use codex_app_server_protocol::ThreadUnsubscribeResponse;
use codex_app_server_protocol::TurnInterruptParams;
use codex_app_server_protocol::TurnInterruptResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_core::config::Config;
use codex_protocol::user_input::UserInput;
use serde::de::DeserializeOwned;
use teloxide::Bot;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ChatAction;
use teloxide::types::ChatId;
use teloxide::types::Message;
use teloxide::types::MessageId;
use teloxide::types::ParseMode;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::instrument;
use tracing::warn;

use crate::approvals::ApprovalCallback;
use crate::approvals::PendingApproval;
use crate::approvals::PendingApprovalKind;
use crate::approvals::rejection_error;
use crate::render::render_html_chunks;
use crate::session::SessionStore;

mod notifications;

const BRIDGE_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug)]
struct RequestIdSequencer {
    next: i64,
}

impl RequestIdSequencer {
    fn new() -> Self {
        Self { next: 1 }
    }

    fn next(&mut self) -> RequestId {
        let id = self.next;
        self.next += 1;
        RequestId::Integer(id)
    }
}

enum BridgeCommand {
    UserText {
        chat_id: ChatId,
        text: String,
    },
    NewThread {
        chat_id: ChatId,
    },
    Cancel {
        chat_id: ChatId,
    },
    Status {
        chat_id: ChatId,
        response_tx: oneshot::Sender<String>,
    },
    Approval {
        callback: ApprovalCallback,
        response_tx: oneshot::Sender<String>,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct BridgeHandle {
    command_tx: mpsc::Sender<BridgeCommand>,
}

impl BridgeHandle {
    pub fn spawn(
        bot: Bot,
        client: InProcessAppServerClient,
        config: Arc<Config>,
        sessions: Arc<SessionStore>,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::channel(BRIDGE_CHANNEL_CAPACITY);
        let runtime = BridgeRuntime {
            bot,
            client,
            config,
            sessions,
            request_ids: RequestIdSequencer::new(),
        };
        tokio::spawn(async move {
            runtime.run(command_rx).await;
        });
        Self { command_tx }
    }

    #[instrument(skip(self, text))]
    pub async fn send_user_text(&self, chat_id: ChatId, text: String) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::UserText { chat_id, text })
            .await
            .context("telegram bridge task stopped")
    }

    #[instrument(skip(self))]
    pub async fn new_thread(&self, chat_id: ChatId) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::NewThread { chat_id })
            .await
            .context("telegram bridge task stopped")
    }

    #[instrument(skip(self))]
    pub async fn cancel(&self, chat_id: ChatId) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Cancel { chat_id })
            .await
            .context("telegram bridge task stopped")
    }

    pub async fn status_text(&self, chat_id: ChatId) -> anyhow::Result<String> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(BridgeCommand::Status {
                chat_id,
                response_tx,
            })
            .await
            .context("telegram bridge task stopped")?;
        response_rx.await.context("telegram bridge status dropped")
    }

    pub async fn handle_approval_callback(
        &self,
        callback: ApprovalCallback,
    ) -> anyhow::Result<String> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(BridgeCommand::Approval {
                callback,
                response_tx,
            })
            .await
            .context("telegram bridge task stopped")?;
        response_rx
            .await
            .context("telegram bridge approval dropped")
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Shutdown)
            .await
            .context("telegram bridge task stopped")
    }
}

struct BridgeRuntime {
    bot: Bot,
    client: InProcessAppServerClient,
    config: Arc<Config>,
    sessions: Arc<SessionStore>,
    request_ids: RequestIdSequencer,
}

impl BridgeRuntime {
    async fn run(mut self, mut command_rx: mpsc::Receiver<BridgeCommand>) {
        loop {
            tokio::select! {
                command = command_rx.recv() => {
                    match command {
                        Some(BridgeCommand::Shutdown) | None => break,
                        Some(command) => {
                            if let Err(err) = self.handle_command(command).await {
                                warn!("telegram bridge command failed: {err}");
                            }
                        }
                    }
                }
                event = self.client.next_event() => {
                    let Some(event) = event else {
                        break;
                    };
                    if let Err(err) = self.handle_event(event).await {
                        warn!("telegram bridge event handling failed: {err}");
                    }
                }
            }
        }

        let thread_ids = self.sessions.thread_ids().await;
        for thread_id in thread_ids {
            if let Err(err) = self.unsubscribe_thread(&thread_id).await {
                warn!("thread/unsubscribe failed during Telegram shutdown: {err}");
            }
        }
        if let Err(err) = self.client.shutdown().await {
            warn!("in-process app-server shutdown failed: {err}");
        }
    }

    async fn handle_command(&mut self, command: BridgeCommand) -> anyhow::Result<()> {
        match command {
            BridgeCommand::UserText { chat_id, text } => self.start_turn(chat_id, text).await,
            BridgeCommand::NewThread { chat_id } => {
                let thread_id = self.start_new_thread(chat_id).await?;
                self.send_text(chat_id, &format!("Started new thread {thread_id}."))
                    .await
            }
            BridgeCommand::Cancel { chat_id } => self.cancel_turn(chat_id).await,
            BridgeCommand::Status {
                chat_id,
                response_tx,
            } => {
                let _ = response_tx.send(self.sessions.status_text(chat_id).await);
                Ok(())
            }
            BridgeCommand::Approval {
                callback,
                response_tx,
            } => {
                let result = self.resolve_approval(callback).await;
                let text = match result {
                    Ok(text) => text,
                    Err(err) => format!("Approval failed: {err}"),
                };
                let _ = response_tx.send(text);
                Ok(())
            }
            BridgeCommand::Shutdown => Ok(()),
        }
    }

    async fn handle_event(&mut self, event: InProcessServerEvent) -> anyhow::Result<()> {
        match event {
            InProcessServerEvent::ServerRequest(request) => {
                self.handle_server_request(request).await
            }
            InProcessServerEvent::ServerNotification(notification) => {
                self.handle_notification(notification).await
            }
            InProcessServerEvent::Lagged { skipped } => self.handle_lagged(skipped as u64).await,
        }
    }

    #[instrument(skip(self, text))]
    async fn start_turn(&mut self, chat_id: ChatId, text: String) -> anyhow::Result<()> {
        if self.sessions.turn_id(chat_id).await.is_some() {
            self.send_text(
                chat_id,
                "A turn is already running. Use /cancel before starting another.",
            )
            .await?;
            return Ok(());
        }

        let thread_id = self.ensure_thread(chat_id).await?;
        let request_id = self.request_ids.next();
        let response: TurnStartResponse = self
            .request_typed(
                ClientRequest::TurnStart {
                    request_id,
                    params: TurnStartParams {
                        thread_id: thread_id.clone(),
                        client_user_message_id: None,
                        input: vec![
                            UserInput::Text {
                                text,
                                text_elements: Vec::new(),
                            }
                            .into(),
                        ],
                        responsesapi_client_metadata: None,
                        additional_context: None,
                        environments: None,
                        cwd: Some(self.config.cwd.to_path_buf()),
                        runtime_workspace_roots: None,
                        approval_policy: Some(
                            self.config.permissions.approval_policy.value().into(),
                        ),
                        approvals_reviewer: None,
                        sandbox_policy: None,
                        permissions: None,
                        model: None,
                        service_tier: None,
                        effort: self.config.model_reasoning_effort.clone(),
                        summary: None,
                        personality: None,
                        output_schema: None,
                        collaboration_mode: None,
                        multi_agent_mode: None,
                    },
                },
                "turn/start",
            )
            .await?;
        self.sessions.set_turn(chat_id, response.turn.id).await;
        let _ = self.bot.send_chat_action(chat_id, ChatAction::Typing).await;
        Ok(())
    }

    async fn cancel_turn(&mut self, chat_id: ChatId) -> anyhow::Result<()> {
        let Some(thread_id) = self.sessions.thread_id(chat_id).await else {
            self.send_text(chat_id, "No active thread to cancel.")
                .await?;
            return Ok(());
        };
        let Some(turn_id) = self.sessions.turn_id(chat_id).await else {
            self.send_text(chat_id, "No active turn to cancel.").await?;
            return Ok(());
        };
        let request_id = self.request_ids.next();
        let _: TurnInterruptResponse = self
            .request_typed(
                ClientRequest::TurnInterrupt {
                    request_id,
                    params: TurnInterruptParams { thread_id, turn_id },
                },
                "turn/interrupt",
            )
            .await?;
        self.send_text(chat_id, "Cancel requested.").await
    }

    async fn ensure_thread(&mut self, chat_id: ChatId) -> anyhow::Result<String> {
        if let Some(thread_id) = self.sessions.thread_id(chat_id).await {
            if !self.sessions.thread_loaded(chat_id).await {
                self.resume_thread(chat_id, thread_id.clone()).await?;
            }
            return Ok(thread_id);
        }
        self.start_new_thread(chat_id).await
    }

    async fn start_new_thread(&mut self, chat_id: ChatId) -> anyhow::Result<String> {
        let request_id = self.request_ids.next();
        let response: ThreadStartResponse = self
            .request_typed(
                ClientRequest::ThreadStart {
                    request_id,
                    params: self.thread_start_params(),
                },
                "thread/start",
            )
            .await?;
        let thread_id = response.thread.id;
        self.sessions.set_thread(chat_id, thread_id.clone()).await?;
        Ok(thread_id)
    }

    async fn resume_thread(&mut self, chat_id: ChatId, thread_id: String) -> anyhow::Result<()> {
        let request_id = self.request_ids.next();
        let _: ThreadResumeResponse = self
            .request_typed(
                ClientRequest::ThreadResume {
                    request_id,
                    params: ThreadResumeParams {
                        thread_id,
                        model: self.config.model.clone(),
                        model_provider: Some(self.config.model_provider_id.clone()),
                        cwd: Some(self.config.cwd.to_string_lossy().to_string()),
                        runtime_workspace_roots: Some(self.config.workspace_roots.clone()),
                        approval_policy: Some(
                            self.config.permissions.approval_policy.value().into(),
                        ),
                        ..ThreadResumeParams::default()
                    },
                },
                "thread/resume",
            )
            .await?;
        self.sessions.mark_thread_loaded(chat_id).await;
        Ok(())
    }

    fn thread_start_params(&self) -> ThreadStartParams {
        ThreadStartParams {
            model: self.config.model.clone(),
            model_provider: Some(self.config.model_provider_id.clone()),
            cwd: Some(self.config.cwd.to_string_lossy().to_string()),
            runtime_workspace_roots: Some(self.config.workspace_roots.clone()),
            approval_policy: Some(self.config.permissions.approval_policy.value().into()),
            approvals_reviewer: None,
            sandbox: None,
            permissions: None,
            config: None,
            ephemeral: Some(self.config.ephemeral),
            session_start_source: None,
            thread_source: Some(ThreadSource::User),
            ..ThreadStartParams::default()
        }
    }

    async fn handle_server_request(&mut self, request: ServerRequest) -> anyhow::Result<()> {
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
        let approval = PendingApproval { request_id, kind };
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
        self.bot
            .send_message(chat_id, message)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await
            .context("send Telegram approval prompt")?;
        Ok(())
    }

    async fn resolve_approval(&mut self, callback: ApprovalCallback) -> anyhow::Result<String> {
        let Some((chat_id, approval)) = self
            .sessions
            .take_pending_approval(&callback.request_id)
            .await
        else {
            return Ok("Approval is no longer pending.".to_string());
        };

        match approval.resolve_value(callback.action)? {
            Some(value) => {
                self.client
                    .resolve_server_request(callback.request_id, value)
                    .await
                    .context("resolve server request")?;
                self.send_text(chat_id, "Approved.").await?;
                Ok("Approved.".to_string())
            }
            None => {
                self.client
                    .reject_server_request(callback.request_id, rejection_error())
                    .await
                    .context("reject server request")?;
                self.send_text(chat_id, "Declined.").await?;
                Ok("Declined.".to_string())
            }
        }
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

    pub(super) async fn send_text(&self, chat_id: ChatId, text: &str) -> anyhow::Result<()> {
        for chunk in render_html_chunks(text) {
            self.send_html(chat_id, &chunk.html).await?;
        }
        Ok(())
    }

    pub(super) async fn send_html(&self, chat_id: ChatId, html: &str) -> anyhow::Result<Message> {
        self.bot
            .send_message(chat_id, html.to_string())
            .parse_mode(ParseMode::Html)
            .await
            .context("send Telegram message")
    }

    pub(super) async fn edit_message(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
        text: &str,
    ) -> anyhow::Result<()> {
        let chunks = render_html_chunks(text);
        if let Some(first) = chunks.first() {
            self.bot
                .edit_message_text(chat_id, message_id, first.html.clone())
                .parse_mode(ParseMode::Html)
                .await
                .context("edit Telegram message")?;
        }
        Ok(())
    }

    async fn unsubscribe_thread(&mut self, thread_id: &str) -> anyhow::Result<()> {
        let request_id = self.request_ids.next();
        let _: ThreadUnsubscribeResponse = self
            .request_typed(
                ClientRequest::ThreadUnsubscribe {
                    request_id,
                    params: ThreadUnsubscribeParams {
                        thread_id: thread_id.to_string(),
                    },
                },
                "thread/unsubscribe",
            )
            .await?;
        Ok(())
    }

    pub(super) async fn request_typed<T>(
        &self,
        request: ClientRequest,
        method: &str,
    ) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        self.client
            .request_typed(request)
            .await
            .with_context(|| format!("failed app-server request `{method}`"))
    }
}

fn sanitize_command_params(
    mut params: CommandExecutionRequestApprovalParams,
) -> CommandExecutionRequestApprovalParams {
    if let Some(command) = params.command.as_mut() {
        *command = truncate_sensitive(command);
    }
    params
}

fn truncate_sensitive(text: &str) -> String {
    const MAX: usize = 2048;
    if text.chars().count() <= MAX {
        return text.to_string();
    }
    let mut truncated = text.chars().take(MAX).collect::<String>();
    truncated.push_str("...(truncated)");
    truncated
}
