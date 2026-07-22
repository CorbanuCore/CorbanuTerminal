use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use codex_app_server_protocol::AskForApproval;
use codex_app_server_protocol::RequestId;
use serde::Deserialize;
use serde::Serialize;
use teloxide::types::MessageId;
use teloxide::types::UserId;
use tokio::sync::Mutex;
use tracing::warn;

use crate::approvals::PendingApproval;
use crate::conversation::ConversationKey;
use crate::render::StreamingText;

#[derive(Debug, Clone, PartialEq)]
struct PendingApprovalState {
    approval: PendingApproval,
    message_id: Option<MessageId>,
    actor_user_id: Option<UserId>,
    created_at: Instant,
}

const PENDING_APPROVAL_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, Default)]
pub struct ChatSession {
    pub thread_id: Option<String>,
    pub thread_loaded: bool,
    pub model: Option<String>,
    pub model_provider: Option<String>,
    pub approval_policy: Option<AskForApproval>,
    pub turn_id: Option<String>,
    pub active_user_id: Option<UserId>,
    pub streaming_item_id: Option<String>,
    pub streaming_text: StreamingText,
    pub streaming_message_id: Option<MessageId>,
    pub stream_edits_suppressed_until: Option<Instant>,
    pending_approvals: HashMap<RequestId, PendingApprovalState>,
    delivered_item_ids: HashSet<String>,
    last_delivered_item_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PendingApprovalRecord {
    pub conversation: ConversationKey,
    pub approval: PendingApproval,
    pub message_id: Option<MessageId>,
    pub actor_user_id: Option<UserId>,
}

#[derive(Debug, Clone)]
pub struct StreamUpdate {
    pub conversation: ConversationKey,
    pub message_id: Option<MessageId>,
    pub html: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedState {
    chats: HashMap<String, PersistedChat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedChat {
    #[serde(default)]
    thread_id: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    model_provider: Option<String>,
    #[serde(default)]
    approval_policy: Option<AskForApproval>,
    #[serde(default)]
    last_delivered_item_id: Option<String>,
}

#[derive(Debug)]
pub struct SessionStore {
    inner: Mutex<HashMap<ConversationKey, ChatSession>>,
    state_path: PathBuf,
}

impl SessionStore {
    pub async fn load(codex_home: &Path) -> anyhow::Result<Arc<Self>> {
        let state_path = codex_home.join("telegram").join("state.json");
        let mut sessions = HashMap::new();
        match tokio::fs::read_to_string(&state_path).await {
            Ok(contents) => {
                let parsed = serde_json::from_str::<PersistedState>(&contents)
                    .with_context(|| format!("failed to parse {}", state_path.display()))
                    .and_then(|persisted| {
                        let mut sessions = HashMap::new();
                        for (conversation, chat) in persisted.chats {
                            let conversation =
                                conversation.parse::<ConversationKey>().with_context(|| {
                                    format!(
                                        "invalid Telegram conversation key in {}",
                                        state_path.display()
                                    )
                                })?;
                            let mut delivered_item_ids = HashSet::new();
                            if let Some(item_id) = &chat.last_delivered_item_id {
                                delivered_item_ids.insert(item_id.clone());
                            }
                            sessions.insert(
                                conversation,
                                ChatSession {
                                    thread_id: chat.thread_id,
                                    thread_loaded: false,
                                    model: chat.model,
                                    model_provider: chat.model_provider,
                                    approval_policy: chat.approval_policy,
                                    delivered_item_ids,
                                    last_delivered_item_id: chat.last_delivered_item_id,
                                    ..Default::default()
                                },
                            );
                        }
                        Ok(sessions)
                    });
                match parsed {
                    Ok(parsed_sessions) => {
                        sessions = parsed_sessions;
                    }
                    Err(err) => {
                        warn!(
                            path = %state_path.display(),
                            "ignoring corrupt Telegram state file: {err}"
                        );
                        if let Err(rename_err) = rename_corrupt_state(&state_path).await {
                            warn!(
                                path = %state_path.display(),
                                "failed to rename corrupt Telegram state file: {rename_err}"
                            );
                        }
                    }
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to read {}", state_path.display()));
            }
        }
        Ok(Arc::new(Self {
            inner: Mutex::new(sessions),
            state_path,
        }))
    }

    pub async fn thread_id(&self, conversation: impl Into<ConversationKey>) -> Option<String> {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .and_then(|session| session.thread_id.clone())
    }

    pub async fn model(&self, conversation: impl Into<ConversationKey>) -> Option<String> {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .and_then(|session| session.model.clone())
    }

    pub async fn model_provider(&self, conversation: impl Into<ConversationKey>) -> Option<String> {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .and_then(|session| session.model_provider.clone())
    }

    pub async fn approval_policy(
        &self,
        conversation: impl Into<ConversationKey>,
    ) -> Option<AskForApproval> {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .and_then(|session| session.approval_policy)
    }

    pub async fn thread_loaded(&self, conversation: impl Into<ConversationKey>) -> bool {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .is_some_and(|session| session.thread_loaded)
    }

    pub async fn turn_id(&self, conversation: impl Into<ConversationKey>) -> Option<String> {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .and_then(|session| session.turn_id.clone())
    }

    pub async fn pending_approval_count(&self, conversation: impl Into<ConversationKey>) -> usize {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get(&conversation)
            .map_or(0, |session| session.pending_approvals.len())
    }

    pub async fn set_thread(
        &self,
        conversation: impl Into<ConversationKey>,
        thread_id: String,
    ) -> anyhow::Result<()> {
        let conversation = conversation.into();
        {
            let mut sessions = self.inner.lock().await;
            let session = sessions.entry(conversation).or_default();
            session.thread_id = Some(thread_id);
            session.thread_loaded = true;
            session.turn_id = None;
            session.active_user_id = None;
            session.streaming_item_id = None;
            session.streaming_message_id = None;
            session.streaming_text = StreamingText::new();
            session.pending_approvals.clear();
            session.delivered_item_ids.clear();
            session.last_delivered_item_id = None;
        }
        self.persist().await
    }

    pub async fn set_model(
        &self,
        conversation: impl Into<ConversationKey>,
        model: String,
        model_provider: String,
    ) -> anyhow::Result<()> {
        let conversation = conversation.into();
        {
            let mut sessions = self.inner.lock().await;
            let session = sessions.entry(conversation).or_default();
            session.model = Some(model);
            session.model_provider = Some(model_provider);
        }
        self.persist().await
    }

    pub async fn set_approval_policy(
        &self,
        conversation: impl Into<ConversationKey>,
        approval_policy: AskForApproval,
    ) -> anyhow::Result<()> {
        let conversation = conversation.into();
        {
            let mut sessions = self.inner.lock().await;
            sessions.entry(conversation).or_default().approval_policy = Some(approval_policy);
        }
        self.persist().await
    }

    pub async fn set_turn(
        &self,
        conversation: impl Into<ConversationKey>,
        turn_id: String,
        actor_user_id: Option<UserId>,
    ) {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        let session = sessions.entry(conversation).or_default();
        session.turn_id = Some(turn_id);
        session.active_user_id = actor_user_id;
    }

    pub async fn mark_thread_loaded(&self, conversation: impl Into<ConversationKey>) {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        sessions.entry(conversation).or_default().thread_loaded = true;
    }

    pub async fn clear_turn_for_thread(&self, thread_id: &str) {
        let mut sessions = self.inner.lock().await;
        for session in sessions.values_mut() {
            if session.thread_id.as_deref() == Some(thread_id) {
                session.turn_id = None;
                session.active_user_id = None;
                session.streaming_item_id = None;
                session.streaming_message_id = None;
                session.streaming_text = StreamingText::new();
                session.stream_edits_suppressed_until = None;
                session.pending_approvals.clear();
            }
        }
    }

    pub async fn conversation_for_thread(&self, thread_id: &str) -> Option<ConversationKey> {
        self.inner
            .lock()
            .await
            .iter()
            .find_map(|(conversation, session)| {
                (session.thread_id.as_deref() == Some(thread_id)).then_some(*conversation)
            })
    }

    pub async fn thread_ids(&self) -> Vec<String> {
        self.inner
            .lock()
            .await
            .values()
            .filter_map(|session| session.thread_id.clone())
            .collect()
    }

    pub async fn conversations_with_threads(&self) -> Vec<ConversationKey> {
        self.inner
            .lock()
            .await
            .iter()
            .filter_map(|(conversation, session)| session.thread_id.as_ref().map(|_| *conversation))
            .collect()
    }

    pub async fn insert_pending_approval(
        &self,
        conversation: impl Into<ConversationKey>,
        approval: PendingApproval,
    ) {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        let session = sessions.entry(conversation).or_default();
        let actor_user_id = session.active_user_id;
        session.pending_approvals.insert(
            approval.request_id.clone(),
            PendingApprovalState {
                approval,
                message_id: None,
                actor_user_id,
                created_at: Instant::now(),
            },
        );
    }

    pub async fn set_pending_approval_message(
        &self,
        conversation: impl Into<ConversationKey>,
        request_id: &RequestId,
        message_id: MessageId,
    ) {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        if let Some(state) = sessions
            .get_mut(&conversation)
            .and_then(|session| session.pending_approvals.get_mut(request_id))
        {
            state.message_id = Some(message_id);
        }
    }

    pub async fn remove_pending_approval(
        &self,
        conversation: impl Into<ConversationKey>,
        request_id: &RequestId,
    ) -> Option<PendingApproval> {
        let conversation = conversation.into();
        self.inner
            .lock()
            .await
            .get_mut(&conversation)
            .and_then(|session| session.pending_approvals.remove(request_id))
            .map(|state| state.approval)
    }

    pub async fn pending_approval(
        &self,
        conversation: impl Into<ConversationKey>,
        request_id: &RequestId,
    ) -> Option<PendingApprovalRecord> {
        let conversation = conversation.into();
        let sessions = self.inner.lock().await;
        let state = sessions
            .get(&conversation)?
            .pending_approvals
            .get(request_id)?;
        if approval_expired(state.created_at, Instant::now()) {
            return None;
        }
        Some(PendingApprovalRecord {
            conversation,
            approval: state.approval.clone(),
            message_id: state.message_id,
            actor_user_id: state.actor_user_id,
        })
    }

    pub async fn take_pending_approval(
        &self,
        conversation: impl Into<ConversationKey>,
        request_id: &RequestId,
    ) -> Option<PendingApprovalRecord> {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        let state = sessions
            .get_mut(&conversation)?
            .pending_approvals
            .remove(request_id)?;
        if approval_expired(state.created_at, Instant::now()) {
            return None;
        }
        Some(PendingApprovalRecord {
            conversation,
            approval: state.approval,
            message_id: state.message_id,
            actor_user_id: state.actor_user_id,
        })
    }

    pub async fn mark_item_delivered(&self, thread_id: &str, item_id: &str) {
        let updated = {
            let mut sessions = self.inner.lock().await;
            if let Some(session) = sessions
                .values_mut()
                .find(|session| session.thread_id.as_deref() == Some(thread_id))
            {
                session.delivered_item_ids.insert(item_id.to_string());
                session.last_delivered_item_id = Some(item_id.to_string());
                true
            } else {
                false
            }
        };
        if updated && let Err(err) = self.persist().await {
            warn!("failed to persist Telegram delivered item marker: {err}");
        }
    }

    pub async fn item_delivered(&self, thread_id: &str, item_id: &str) -> bool {
        self.inner
            .lock()
            .await
            .values()
            .find(|session| session.thread_id.as_deref() == Some(thread_id))
            .is_some_and(|session| session.delivered_item_ids.contains(item_id))
    }

    pub async fn last_delivered_item_id(&self, thread_id: &str) -> Option<String> {
        self.inner
            .lock()
            .await
            .values()
            .find(|session| session.thread_id.as_deref() == Some(thread_id))
            .and_then(|session| session.last_delivered_item_id.clone())
    }

    pub async fn append_stream_delta(
        &self,
        thread_id: &str,
        item_id: &str,
        delta: &str,
        now: Instant,
    ) -> Option<StreamUpdate> {
        let mut sessions = self.inner.lock().await;
        let (conversation, session) = sessions
            .iter_mut()
            .find(|(_, session)| session.thread_id.as_deref() == Some(thread_id))?;
        if session.streaming_item_id.as_deref() != Some(item_id) {
            session.streaming_item_id = Some(item_id.to_string());
            session.streaming_text = StreamingText::new();
            session.streaming_message_id = None;
        }
        if !session.streaming_text.push_delta(delta, now) {
            return None;
        }
        Some(StreamUpdate {
            conversation: *conversation,
            message_id: session.streaming_message_id,
            html: session.streaming_text.preview_html(),
        })
    }

    pub async fn set_stream_message(
        &self,
        conversation: impl Into<ConversationKey>,
        message_id: MessageId,
    ) {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        sessions
            .entry(conversation)
            .or_default()
            .streaming_message_id = Some(message_id);
    }

    pub async fn stream_edits_suppressed(
        &self,
        conversation: impl Into<ConversationKey>,
        now: Instant,
    ) -> bool {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        let Some(session) = sessions.get_mut(&conversation) else {
            return false;
        };
        match session.stream_edits_suppressed_until {
            Some(until) if now < until => true,
            Some(_) => {
                session.stream_edits_suppressed_until = None;
                false
            }
            None => false,
        }
    }

    pub async fn suppress_stream_edits_until(
        &self,
        conversation: impl Into<ConversationKey>,
        until: Instant,
    ) {
        let conversation = conversation.into();
        let mut sessions = self.inner.lock().await;
        sessions
            .entry(conversation)
            .or_default()
            .stream_edits_suppressed_until = Some(until);
    }

    pub async fn take_stream_for_item(
        &self,
        thread_id: &str,
        item_id: &str,
    ) -> Option<(ConversationKey, Option<MessageId>)> {
        let mut sessions = self.inner.lock().await;
        let (conversation, session) = sessions.iter_mut().find(|(_, session)| {
            session.thread_id.as_deref() == Some(thread_id)
                && session.streaming_item_id.as_deref() == Some(item_id)
        })?;
        let message_id = session.streaming_message_id.take();
        session.streaming_item_id = None;
        session.streaming_text = StreamingText::new();
        Some((*conversation, message_id))
    }

    async fn persist(&self) -> anyhow::Result<()> {
        let persisted = {
            let sessions = self.inner.lock().await;
            PersistedState {
                chats: sessions
                    .iter()
                    .filter(|(_, session)| session.should_persist())
                    .map(|(conversation, session)| {
                        (
                            conversation.storage_key(),
                            PersistedChat {
                                thread_id: session.thread_id.clone(),
                                model: session.model.clone(),
                                model_provider: session.model_provider.clone(),
                                approval_policy: session.approval_policy,
                                last_delivered_item_id: session.last_delivered_item_id.clone(),
                            },
                        )
                    })
                    .collect(),
            }
        };
        crate::persistence::write_atomically(
            &self.state_path,
            serde_json::to_string_pretty(&persisted)?,
        )
        .await
    }
}

pub(crate) fn approval_expired(created_at: Instant, now: Instant) -> bool {
    now.checked_duration_since(created_at)
        .is_some_and(|age| age > PENDING_APPROVAL_TTL)
}

async fn rename_corrupt_state(state_path: &Path) -> anyhow::Result<()> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let aside_path = state_path.with_extension(format!("json.corrupt.{suffix}"));
    tokio::fs::rename(state_path, aside_path).await?;
    Ok(())
}

impl ChatSession {
    fn should_persist(&self) -> bool {
        self.thread_id.is_some()
            || self.model.is_some()
            || self.model_provider.is_some()
            || self.approval_policy.is_some()
            || self.last_delivered_item_id.is_some()
    }
}
