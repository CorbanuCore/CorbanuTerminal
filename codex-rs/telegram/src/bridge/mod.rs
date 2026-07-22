use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use codex_app_server_client::InProcessAppServerClient;
use codex_app_server_client::InProcessServerEvent;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::RequestId;
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
use teloxide::ApiError;
use teloxide::Bot;
use teloxide::RequestError;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ChatAction;
use teloxide::types::ChatId;
use teloxide::types::Message;
use teloxide::types::MessageId;
use teloxide::types::ParseMode;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::instrument;
use tracing::warn;

use crate::approvals::ApprovalCallback;
use crate::render::render_html_chunks;
use crate::session::SessionStore;

mod commands;
mod notifications;
mod server_requests;

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
        /// Local paths of already-downloaded inbound images; each becomes a
        /// `UserInput::LocalImage` on the turn.
        images: Vec<std::path::PathBuf>,
    },
    NewThread {
        chat_id: ChatId,
    },
    Cancel {
        chat_id: ChatId,
    },
    Model {
        chat_id: ChatId,
        args: String,
    },
    Approvals {
        chat_id: ChatId,
        args: String,
    },
    Compact {
        chat_id: ChatId,
    },
    Diff {
        chat_id: ChatId,
    },
    Skills {
        chat_id: ChatId,
    },
    Status {
        chat_id: ChatId,
        response_tx: oneshot::Sender<String>,
    },
    Approval {
        chat_id: ChatId,
        callback: ApprovalCallback,
        response_tx: oneshot::Sender<String>,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct BridgeHandle {
    command_tx: mpsc::Sender<BridgeCommand>,
    task: Arc<Mutex<Option<JoinHandle<()>>>>,
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
        let task = tokio::spawn(async move {
            runtime.run(command_rx).await;
        });
        Self {
            command_tx,
            task: Arc::new(Mutex::new(Some(task))),
        }
    }

    #[instrument(skip(self, text))]
    pub async fn send_user_text(&self, chat_id: ChatId, text: String) -> anyhow::Result<()> {
        self.send_user_input(chat_id, text, Vec::new()).await
    }

    #[instrument(skip(self, text, images))]
    pub async fn send_user_input(
        &self,
        chat_id: ChatId,
        text: String,
        images: Vec<std::path::PathBuf>,
    ) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::UserText {
                chat_id,
                text,
                images,
            })
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

    #[instrument(skip(self, args))]
    pub async fn model(&self, chat_id: ChatId, args: String) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Model { chat_id, args })
            .await
            .context("telegram bridge task stopped")
    }

    #[instrument(skip(self, args))]
    pub async fn approvals(&self, chat_id: ChatId, args: String) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Approvals { chat_id, args })
            .await
            .context("telegram bridge task stopped")
    }

    #[instrument(skip(self))]
    pub async fn compact(&self, chat_id: ChatId) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Compact { chat_id })
            .await
            .context("telegram bridge task stopped")
    }

    #[instrument(skip(self))]
    pub async fn diff(&self, chat_id: ChatId) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Diff { chat_id })
            .await
            .context("telegram bridge task stopped")
    }

    #[instrument(skip(self))]
    pub async fn skills(&self, chat_id: ChatId) -> anyhow::Result<()> {
        self.command_tx
            .send(BridgeCommand::Skills { chat_id })
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
        chat_id: ChatId,
        callback: ApprovalCallback,
    ) -> anyhow::Result<String> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .send(BridgeCommand::Approval {
                chat_id,
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
        if let Err(err) = self.command_tx.send(BridgeCommand::Shutdown).await {
            warn!("telegram bridge task already stopped before shutdown request: {err}");
        }
        let task = {
            let mut task = self.task.lock().await;
            task.take()
        };
        let Some(task) = task else {
            return Ok(());
        };
        match timeout(Duration::from_secs(10), task).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => warn!("telegram bridge task failed during shutdown: {err}"),
            Err(_) => warn!("timed out waiting for Telegram bridge shutdown"),
        }
        Ok(())
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
                            let error_chat_id = command.error_chat_id();
                            if let Err(err) = self.handle_command(command).await {
                                warn!("telegram bridge command failed: {err}");
                                if let Some(chat_id) = error_chat_id
                                    && let Err(send_err) = self
                                        .send_text(chat_id, &format!("Error: {err:#}"))
                                        .await
                                {
                                    warn!("failed to report Telegram bridge error to chat: {send_err}");
                                }
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
            BridgeCommand::UserText {
                chat_id,
                text,
                images,
            } => self.start_turn(chat_id, text, images).await,
            BridgeCommand::NewThread { chat_id } => {
                let thread_id = self.start_new_thread(chat_id).await?;
                self.send_text(chat_id, &format!("Started new thread {thread_id}."))
                    .await
            }
            BridgeCommand::Cancel { chat_id } => self.cancel_turn(chat_id).await,
            BridgeCommand::Model { chat_id, args } => self.handle_model(chat_id, args).await,
            BridgeCommand::Approvals { chat_id, args } => {
                self.handle_approvals(chat_id, args).await
            }
            BridgeCommand::Compact { chat_id } => self.compact_thread(chat_id).await,
            BridgeCommand::Diff { chat_id } => self.send_diff(chat_id).await,
            BridgeCommand::Skills { chat_id } => self.list_skills(chat_id).await,
            BridgeCommand::Status {
                chat_id,
                response_tx,
            } => {
                let _ = response_tx.send(self.sessions.status_text(chat_id).await);
                Ok(())
            }
            BridgeCommand::Approval {
                chat_id,
                callback,
                response_tx,
            } => {
                let result = self.resolve_approval(chat_id, callback).await;
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
    async fn start_turn(
        &mut self,
        chat_id: ChatId,
        text: String,
        images: Vec<std::path::PathBuf>,
    ) -> anyhow::Result<()> {
        if self.sessions.turn_id(chat_id).await.is_some() {
            self.send_text(
                chat_id,
                "A turn is already running. Use /cancel before starting another.",
            )
            .await?;
            return Ok(());
        }

        let thread_id = self.ensure_thread(chat_id).await?;
        let approval_policy = self.active_approval_policy(chat_id).await;
        let request_id = self.request_ids.next();
        let response: TurnStartResponse = self
            .request_typed(
                ClientRequest::TurnStart {
                    request_id,
                    params: TurnStartParams {
                        thread_id: thread_id.clone(),
                        client_user_message_id: None,
                        input: turn_input(text, images),
                        responsesapi_client_metadata: None,
                        additional_context: None,
                        environments: None,
                        cwd: Some(self.config.cwd.to_path_buf()),
                        runtime_workspace_roots: None,
                        approval_policy: Some(approval_policy),
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
        let params = self.thread_start_params(chat_id).await;
        let response: ThreadStartResponse = self
            .request_typed(
                ClientRequest::ThreadStart { request_id, params },
                "thread/start",
            )
            .await?;
        let thread_id = response.thread.id;
        self.sessions.set_thread(chat_id, thread_id.clone()).await?;
        Ok(thread_id)
    }

    async fn resume_thread(&mut self, chat_id: ChatId, thread_id: String) -> anyhow::Result<()> {
        let request_id = self.request_ids.next();
        let (model, model_provider) = self.active_model_settings(chat_id).await;
        let approval_policy = self.active_approval_policy(chat_id).await;
        let _: ThreadResumeResponse = self
            .request_typed(
                ClientRequest::ThreadResume {
                    request_id,
                    params: ThreadResumeParams {
                        thread_id,
                        model,
                        model_provider: Some(model_provider),
                        cwd: Some(self.config.cwd.to_string_lossy().to_string()),
                        runtime_workspace_roots: Some(self.config.workspace_roots.clone()),
                        approval_policy: Some(approval_policy),
                        ..ThreadResumeParams::default()
                    },
                },
                "thread/resume",
            )
            .await?;
        self.sessions.mark_thread_loaded(chat_id).await;
        Ok(())
    }

    async fn thread_start_params(&self, chat_id: ChatId) -> ThreadStartParams {
        let (model, model_provider) = self.active_model_settings(chat_id).await;
        let approval_policy = self.active_approval_policy(chat_id).await;
        ThreadStartParams {
            model,
            model_provider: Some(model_provider),
            cwd: Some(self.config.cwd.to_string_lossy().to_string()),
            runtime_workspace_roots: Some(self.config.workspace_roots.clone()),
            approval_policy: Some(approval_policy),
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

    pub(super) async fn send_text(&self, chat_id: ChatId, text: &str) -> anyhow::Result<()> {
        for chunk in render_html_chunks(text) {
            self.send_html(chat_id, &chunk.html).await?;
        }
        Ok(())
    }

    pub(super) async fn send_html(&self, chat_id: ChatId, html: &str) -> anyhow::Result<Message> {
        // Mutating: a timeout bounds the call, but it is never auto-retried —
        // a retried send could post the message twice. Duplicate protection
        // for sends lives at the update level (`crate::dedup`), not here.
        let bot = self.bot.clone();
        let html = html.to_string();
        crate::outbound::call_with_policy(
            crate::outbound::CallSafety::Mutating,
            crate::outbound::DEFAULT_API_TIMEOUT,
            "telegram sendMessage",
            move || {
                let bot = bot.clone();
                let html = html.clone();
                async move {
                    bot.send_message(chat_id, html)
                        .parse_mode(ParseMode::Html)
                        .await
                }
            },
        )
        .await
        .context("send Telegram message")
    }

    pub(super) async fn edit_message(
        &self,
        chat_id: ChatId,
        message_id: MessageId,
        text: &str,
    ) -> anyhow::Result<bool> {
        let chunks = render_html_chunks(text);
        if let Some(first) = chunks.first()
            && let Some(delay) = finish_edit_result(
                crate::outbound::call_with_policy(
                    crate::outbound::CallSafety::Mutating,
                    crate::outbound::DEFAULT_API_TIMEOUT,
                    "telegram editMessageText",
                    || {
                        let bot = self.bot.clone();
                        let html = first.html.clone();
                        async move {
                            bot.edit_message_text(chat_id, message_id, html)
                                .parse_mode(ParseMode::Html)
                                .await
                        }
                    },
                )
                .await,
                "edit Telegram message",
            )?
        {
            self.sessions
                .suppress_stream_edits_until(chat_id, std::time::Instant::now() + delay)
                .await;
            return Ok(false);
        }
        Ok(true)
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

impl BridgeCommand {
    fn error_chat_id(&self) -> Option<ChatId> {
        match self {
            Self::UserText { chat_id, .. }
            | Self::NewThread { chat_id }
            | Self::Cancel { chat_id }
            | Self::Model { chat_id, .. }
            | Self::Approvals { chat_id, .. }
            | Self::Compact { chat_id }
            | Self::Diff { chat_id }
            | Self::Skills { chat_id } => Some(*chat_id),
            Self::Status { .. } | Self::Approval { .. } | Self::Shutdown => None,
        }
    }
}

/// Assemble turn input: images first (so the model reads the screenshot in the
/// context of the caption that follows), then the text if there is any. An
/// image-only message yields image items alone; text-only yields text alone.
fn turn_input(
    text: String,
    images: Vec<std::path::PathBuf>,
) -> Vec<codex_app_server_protocol::UserInput> {
    let mut input: Vec<codex_app_server_protocol::UserInput> = images
        .into_iter()
        .map(|path| UserInput::LocalImage { path, detail: None }.into())
        .collect();
    if !text.is_empty() {
        input.push(
            UserInput::Text {
                text,
                text_elements: Vec::new(),
            }
            .into(),
        );
    }
    input
}

pub(super) fn finish_edit_result(
    result: Result<Message, RequestError>,
    context: &str,
) -> anyhow::Result<Option<Duration>> {
    match result {
        Ok(_) => Ok(None),
        Err(RequestError::Api(ApiError::MessageNotModified)) => Ok(None),
        Err(RequestError::RetryAfter(delay)) => Ok(Some(delay.duration())),
        Err(err) => Err(err).context(context.to_string()),
    }
}

#[cfg(test)]
mod turn_input_tests {
    use super::*;

    #[test]
    fn image_only_message_yields_only_image_items() {
        let input = turn_input(String::new(), vec!["/tmp/a.jpg".into()]);
        assert_eq!(input.len(), 1);
    }

    #[test]
    fn caption_follows_images() {
        let input = turn_input("look at this".into(), vec!["/tmp/a.jpg".into()]);
        assert_eq!(input.len(), 2);
        let json = serde_json::to_value(&input).expect("serialize");
        assert!(json[0].to_string().contains("a.jpg"));
        assert!(json[1].to_string().contains("look at this"));
    }

    #[test]
    fn text_only_unchanged() {
        let input = turn_input("hi".into(), Vec::new());
        assert_eq!(input.len(), 1);
    }
}
