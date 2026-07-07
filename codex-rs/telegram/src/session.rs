use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use codex_app_server_protocol::RequestId;
use serde::Deserialize;
use serde::Serialize;
use teloxide::types::ChatId;
use teloxide::types::MessageId;
use tokio::sync::Mutex;
use tracing::warn;

use crate::approvals::PendingApproval;
use crate::render::StreamingText;

#[derive(Debug, Clone, PartialEq)]
struct PendingApprovalState {
    approval: PendingApproval,
    message_id: Option<MessageId>,
}

#[derive(Debug, Clone, Default)]
pub struct ChatSession {
    pub thread_id: Option<String>,
    pub thread_loaded: bool,
    pub turn_id: Option<String>,
    pub streaming_item_id: Option<String>,
    pub streaming_text: StreamingText,
    pub streaming_message_id: Option<MessageId>,
    pending_approvals: HashMap<RequestId, PendingApprovalState>,
    delivered_item_ids: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PendingApprovalRecord {
    pub chat_id: ChatId,
    pub approval: PendingApproval,
    pub message_id: Option<MessageId>,
}

#[derive(Debug, Clone)]
pub struct StreamUpdate {
    pub chat_id: ChatId,
    pub message_id: Option<MessageId>,
    pub html: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedState {
    chats: HashMap<String, PersistedChat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedChat {
    thread_id: String,
}

#[derive(Debug)]
pub struct SessionStore {
    inner: Mutex<HashMap<ChatId, ChatSession>>,
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
                        for (chat_id, chat) in persisted.chats {
                            let chat_id = ChatId(chat_id.parse().with_context(|| {
                                format!("invalid Telegram chat id in {}", state_path.display())
                            })?);
                            sessions.insert(
                                chat_id,
                                ChatSession {
                                    thread_id: Some(chat.thread_id),
                                    thread_loaded: false,
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

    pub async fn status_text(&self, chat_id: ChatId) -> String {
        let sessions = self.inner.lock().await;
        let Some(session) = sessions.get(&chat_id) else {
            return "No active Telegram session.".to_string();
        };
        match (&session.thread_id, &session.turn_id) {
            (Some(thread_id), Some(turn_id)) => {
                format!("Thread: {thread_id}\nActive turn: {turn_id}")
            }
            (Some(thread_id), None) => format!("Thread: {thread_id}\nNo active turn."),
            (None, _) => "No active thread.".to_string(),
        }
    }

    pub async fn thread_id(&self, chat_id: ChatId) -> Option<String> {
        self.inner
            .lock()
            .await
            .get(&chat_id)
            .and_then(|session| session.thread_id.clone())
    }

    pub async fn thread_loaded(&self, chat_id: ChatId) -> bool {
        self.inner
            .lock()
            .await
            .get(&chat_id)
            .is_some_and(|session| session.thread_loaded)
    }

    pub async fn turn_id(&self, chat_id: ChatId) -> Option<String> {
        self.inner
            .lock()
            .await
            .get(&chat_id)
            .and_then(|session| session.turn_id.clone())
    }

    pub async fn set_thread(&self, chat_id: ChatId, thread_id: String) -> anyhow::Result<()> {
        {
            let mut sessions = self.inner.lock().await;
            let session = sessions.entry(chat_id).or_default();
            session.thread_id = Some(thread_id);
            session.thread_loaded = true;
            session.turn_id = None;
            session.streaming_item_id = None;
            session.streaming_message_id = None;
            session.streaming_text = StreamingText::new();
            session.pending_approvals.clear();
        }
        self.persist().await
    }

    pub async fn set_turn(&self, chat_id: ChatId, turn_id: String) {
        let mut sessions = self.inner.lock().await;
        sessions.entry(chat_id).or_default().turn_id = Some(turn_id);
    }

    pub async fn mark_thread_loaded(&self, chat_id: ChatId) {
        let mut sessions = self.inner.lock().await;
        sessions.entry(chat_id).or_default().thread_loaded = true;
    }

    pub async fn clear_turn_for_thread(&self, thread_id: &str) {
        let mut sessions = self.inner.lock().await;
        for session in sessions.values_mut() {
            if session.thread_id.as_deref() == Some(thread_id) {
                session.turn_id = None;
                session.streaming_item_id = None;
                session.streaming_message_id = None;
                session.streaming_text = StreamingText::new();
                session.pending_approvals.clear();
            }
        }
    }

    pub async fn chat_for_thread(&self, thread_id: &str) -> Option<ChatId> {
        self.inner
            .lock()
            .await
            .iter()
            .find_map(|(chat_id, session)| {
                (session.thread_id.as_deref() == Some(thread_id)).then_some(*chat_id)
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

    pub async fn chat_ids_with_threads(&self) -> Vec<ChatId> {
        self.inner
            .lock()
            .await
            .iter()
            .filter_map(|(chat_id, session)| session.thread_id.as_ref().map(|_| *chat_id))
            .collect()
    }

    pub async fn insert_pending_approval(&self, chat_id: ChatId, approval: PendingApproval) {
        let mut sessions = self.inner.lock().await;
        sessions
            .entry(chat_id)
            .or_default()
            .pending_approvals
            .insert(
                approval.request_id.clone(),
                PendingApprovalState {
                    approval,
                    message_id: None,
                },
            );
    }

    pub async fn set_pending_approval_message(
        &self,
        chat_id: ChatId,
        request_id: &RequestId,
        message_id: MessageId,
    ) {
        let mut sessions = self.inner.lock().await;
        if let Some(state) = sessions
            .get_mut(&chat_id)
            .and_then(|session| session.pending_approvals.get_mut(request_id))
        {
            state.message_id = Some(message_id);
        }
    }

    pub async fn remove_pending_approval(
        &self,
        chat_id: ChatId,
        request_id: &RequestId,
    ) -> Option<PendingApproval> {
        self.inner
            .lock()
            .await
            .get_mut(&chat_id)
            .and_then(|session| session.pending_approvals.remove(request_id))
            .map(|state| state.approval)
    }

    pub async fn take_pending_approval(
        &self,
        chat_id: ChatId,
        request_id: &RequestId,
    ) -> Option<PendingApprovalRecord> {
        let mut sessions = self.inner.lock().await;
        let state = sessions
            .get_mut(&chat_id)?
            .pending_approvals
            .remove(request_id)?;
        Some(PendingApprovalRecord {
            chat_id,
            approval: state.approval,
            message_id: state.message_id,
        })
    }

    pub async fn mark_item_delivered(&self, thread_id: &str, item_id: &str) {
        let mut sessions = self.inner.lock().await;
        if let Some(session) = sessions
            .values_mut()
            .find(|session| session.thread_id.as_deref() == Some(thread_id))
        {
            session.delivered_item_ids.insert(item_id.to_string());
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

    pub async fn append_stream_delta(
        &self,
        thread_id: &str,
        item_id: &str,
        delta: &str,
        now: Instant,
    ) -> Option<StreamUpdate> {
        let mut sessions = self.inner.lock().await;
        let (chat_id, session) = sessions
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
            chat_id: *chat_id,
            message_id: session.streaming_message_id,
            html: session.streaming_text.preview_html(),
        })
    }

    pub async fn set_stream_message(&self, chat_id: ChatId, message_id: MessageId) {
        let mut sessions = self.inner.lock().await;
        sessions.entry(chat_id).or_default().streaming_message_id = Some(message_id);
    }

    pub async fn take_stream_for_item(
        &self,
        thread_id: &str,
        item_id: &str,
    ) -> Option<(ChatId, Option<MessageId>)> {
        let mut sessions = self.inner.lock().await;
        let (chat_id, session) = sessions.iter_mut().find(|(_, session)| {
            session.thread_id.as_deref() == Some(thread_id)
                && session.streaming_item_id.as_deref() == Some(item_id)
        })?;
        let message_id = session.streaming_message_id.take();
        session.streaming_item_id = None;
        session.streaming_text = StreamingText::new();
        Some((*chat_id, message_id))
    }

    async fn persist(&self) -> anyhow::Result<()> {
        let persisted = {
            let sessions = self.inner.lock().await;
            PersistedState {
                chats: sessions
                    .iter()
                    .filter_map(|(chat_id, session)| {
                        session.thread_id.as_ref().map(|thread_id| {
                            (
                                chat_id.0.to_string(),
                                PersistedChat {
                                    thread_id: thread_id.clone(),
                                },
                            )
                        })
                    })
                    .collect(),
            }
        };
        if let Some(parent) = self.state_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let bytes = serde_json::to_vec_pretty(&persisted)?;
        tokio::fs::write(&self.state_path, bytes).await?;
        Ok(())
    }
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
