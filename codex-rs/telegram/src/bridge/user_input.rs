use std::collections::VecDeque;
use std::path::PathBuf;

use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnSteerParams;
use codex_app_server_protocol::TurnSteerResponse;
use codex_protocol::user_input::UserInput;
use teloxide::payloads::SendChatActionSetters;
use teloxide::prelude::Requester;
use teloxide::types::ChatAction;
use teloxide::types::UserId;
use tokio::sync::oneshot;
use tracing::instrument;
use tracing::warn;

use super::BridgeRuntime;
use super::UserInputAdmission;
use crate::conversation::ConversationKey;

const PER_CONVERSATION_QUEUE_CAPACITY: usize = 16;
const PER_CONVERSATION_QUEUE_BYTES: usize = 256 * 1024;

pub(super) struct QueuedUserInput {
    text: String,
    images: Vec<PathBuf>,
    client_user_message_id: String,
    actor_user_id: Option<UserId>,
    completion_tx: oneshot::Sender<Result<(), String>>,
}

impl QueuedUserInput {
    fn estimated_bytes(&self) -> usize {
        self.text.len()
            + self
                .images
                .iter()
                .map(|path| path.to_string_lossy().len())
                .sum::<usize>()
    }
}

impl BridgeRuntime {
    pub(super) fn cancel_pending_inputs(&mut self, conversation: ConversationKey) -> usize {
        let Some(queue) = self.pending_inputs.remove(&conversation) else {
            return 0;
        };
        let count = queue.len();
        for input in queue {
            let _ = input.completion_tx.send(Ok(()));
        }
        count
    }

    // Admission and completion use separate acknowledgements so same-chat
    // Telegram updates are not serialized behind a queued turn.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn handle_user_input(
        &mut self,
        conversation: ConversationKey,
        text: String,
        images: Vec<PathBuf>,
        client_user_message_id: String,
        actor_user_id: Option<UserId>,
        admission_tx: oneshot::Sender<Result<UserInputAdmission, String>>,
        completion_tx: oneshot::Sender<Result<(), String>>,
    ) -> anyhow::Result<()> {
        match self
            .start_turn(
                conversation,
                text.clone(),
                images.clone(),
                client_user_message_id.clone(),
                actor_user_id,
            )
            .await
        {
            Ok(()) => {
                let _ = admission_tx.send(Ok(UserInputAdmission::Applied));
                Ok(())
            }
            Err(err) if self.sessions.turn_id(conversation).await.is_some() => {
                let queue = self.pending_inputs.entry(conversation).or_default();
                let queued_bytes = queue.iter().map(QueuedUserInput::estimated_bytes).sum();
                let incoming_bytes = text.len()
                    + images
                        .iter()
                        .map(|path| path.to_string_lossy().len())
                        .sum::<usize>();
                if let Some(message) =
                    queue_capacity_error(queue.len(), queued_bytes, incoming_bytes)
                {
                    let _ = admission_tx.send(Err(message.clone()));
                    return Err(anyhow::anyhow!(message));
                }
                queue.push_back(QueuedUserInput {
                    text,
                    images,
                    client_user_message_id,
                    actor_user_id,
                    completion_tx,
                });
                let pending_count = queue.len();
                let _ = admission_tx.send(Ok(UserInputAdmission::Queued));
                if let Err(notice_err) = self
                    .send_text(
                        conversation,
                        &format!("Queued after the running turn ({pending_count} pending)."),
                    )
                    .await
                {
                    warn!(
                        %conversation,
                        "Telegram input remains queued after its status notice failed: {notice_err}"
                    );
                }
                warn!(%conversation, "turn/steer unavailable; queued Telegram input: {err}");
                Ok(())
            }
            Err(err) => {
                let _ = admission_tx.send(Err(format!("{err:#}")));
                Err(err)
            }
        }
    }

    pub(super) async fn pump_pending_inputs(&mut self) {
        let conversations = self.pending_inputs.keys().copied().collect::<Vec<_>>();
        for conversation in conversations {
            if self.sessions.turn_id(conversation).await.is_some() {
                continue;
            }
            let Some(input) = self
                .pending_inputs
                .get_mut(&conversation)
                .and_then(VecDeque::pop_front)
            else {
                continue;
            };
            let result = self
                .start_turn(
                    conversation,
                    input.text.clone(),
                    input.images.clone(),
                    input.client_user_message_id.clone(),
                    input.actor_user_id,
                )
                .await;
            if let Err(err) = result {
                warn!(%conversation, "failed to start queued Telegram input: {err}");
                self.last_errors.insert(conversation, format!("{err:#}"));
                self.pending_inputs
                    .entry(conversation)
                    .or_default()
                    .push_front(input);
                continue;
            }
            let _ = input.completion_tx.send(Ok(()));
            self.last_errors.remove(&conversation);
            if self
                .pending_inputs
                .get(&conversation)
                .is_some_and(VecDeque::is_empty)
            {
                self.pending_inputs.remove(&conversation);
            }
        }
    }

    #[instrument(skip(self, text, images))]
    async fn start_turn(
        &mut self,
        conversation: ConversationKey,
        text: String,
        images: Vec<PathBuf>,
        client_user_message_id: String,
        actor_user_id: Option<UserId>,
    ) -> anyhow::Result<()> {
        let thread_id = self.ensure_thread(conversation).await?;
        if self
            .client_message_already_applied(&thread_id, &client_user_message_id)
            .await?
        {
            return Ok(());
        }
        if let Some(turn_id) = self.sessions.turn_id(conversation).await {
            let request_id = self.request_ids.next();
            let _: TurnSteerResponse = self
                .request_typed(
                    ClientRequest::TurnSteer {
                        request_id,
                        params: TurnSteerParams {
                            thread_id,
                            input: turn_input(text, images),
                            client_user_message_id: Some(client_user_message_id),
                            responsesapi_client_metadata: None,
                            additional_context: None,
                            expected_turn_id: turn_id,
                        },
                    },
                    "turn/steer",
                )
                .await?;
            self.send_text(conversation, "Added to the running turn.")
                .await?;
            return Ok(());
        }

        let approval_policy = self.active_approval_policy(conversation).await;
        let request_id = self.request_ids.next();
        let response: TurnStartResponse = self
            .request_typed(
                ClientRequest::TurnStart {
                    request_id,
                    params: TurnStartParams {
                        thread_id,
                        client_user_message_id: Some(client_user_message_id),
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
        self.sessions
            .set_turn(conversation, response.turn.id, actor_user_id)
            .await;
        let mut action = self
            .bot
            .send_chat_action(conversation.chat_id, ChatAction::Typing);
        if let Some(thread_id) = conversation.thread_id {
            action = action.message_thread_id(thread_id);
        }
        let _ = action.await;
        Ok(())
    }

    async fn client_message_already_applied(
        &mut self,
        thread_id: &str,
        client_user_message_id: &str,
    ) -> anyhow::Result<bool> {
        let request_id = self.request_ids.next();
        let response: ThreadReadResponse = self
            .request_typed(
                ClientRequest::ThreadRead {
                    request_id,
                    params: ThreadReadParams {
                        thread_id: thread_id.to_string(),
                        include_turns: true,
                    },
                },
                "thread/read for Telegram inbox reconciliation",
            )
            .await?;
        Ok(turn_items_contain_client_message(
            response
                .thread
                .turns
                .iter()
                .map(|turn| turn.items.as_slice()),
            client_user_message_id,
        ))
    }
}

fn turn_items_contain_client_message<'a>(
    turn_items: impl IntoIterator<Item = &'a [ThreadItem]>,
    client_user_message_id: &str,
) -> bool {
    turn_items.into_iter().any(|items| {
        items.iter().any(|item| {
            matches!(
                item,
                ThreadItem::UserMessage {
                    client_id: Some(client_id),
                    ..
                } if client_id == client_user_message_id
            )
        })
    })
}

fn queue_capacity_error(
    item_count: usize,
    queued_bytes: usize,
    incoming_bytes: usize,
) -> Option<String> {
    if item_count >= PER_CONVERSATION_QUEUE_CAPACITY {
        return Some(format!(
            "Telegram input queue is full ({PER_CONVERSATION_QUEUE_CAPACITY} messages); retry after the active turn completes"
        ));
    }
    if queued_bytes.saturating_add(incoming_bytes) > PER_CONVERSATION_QUEUE_BYTES {
        return Some(format!(
            "Telegram input queue reached its {} KiB limit; retry after the active turn completes",
            PER_CONVERSATION_QUEUE_BYTES / 1024
        ));
    }
    None
}

fn turn_input(text: String, images: Vec<PathBuf>) -> Vec<codex_app_server_protocol::UserInput> {
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

#[cfg(test)]
#[path = "user_input_tests.rs"]
mod tests;
