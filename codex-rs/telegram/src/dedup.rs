//! Crash-safe Telegram update inbox and bounded replay protection.
//!
//! An update is persisted as pending before a handler may mutate app-server
//! state. It becomes completed only after the handler succeeds. Pending raw
//! updates are replayed on process start, while completed ids form a bounded
//! fast deduplication window. This closes both crash gaps: accepting an update
//! and losing it before dispatch, or replaying it after app-server acceptance.

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use teloxide::types::Update;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tracing::info;
use tracing::warn;

pub const DEDUP_WINDOW: usize = 1024;
pub const INBOX_CAPACITY: usize = 256;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedUpdates {
    #[serde(default, alias = "update_ids")]
    completed_update_ids: Vec<u64>,
    #[serde(default)]
    pending: Vec<Update>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeginUpdate {
    Accepted,
    Duplicate,
    InboxFull,
    PersistenceFailed,
}

#[derive(Debug)]
pub struct UpdateDeduplicator {
    inner: Mutex<State>,
    persist_lock: Semaphore,
    state_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct State {
    completed: HashSet<u64>,
    completed_order: VecDeque<u64>,
    pending: BTreeMap<u64, Update>,
    in_flight: HashSet<u64>,
}

impl State {
    fn mark_completed(&mut self, id: u64) {
        self.pending.remove(&id);
        self.in_flight.remove(&id);
        if self.completed.insert(id) {
            self.completed_order.push_back(id);
        }
        while self.completed_order.len() > DEDUP_WINDOW {
            if let Some(oldest) = self.completed_order.pop_front() {
                self.completed.remove(&oldest);
            }
        }
    }
}

impl UpdateDeduplicator {
    /// Load one bot's inbox. Bot identity is part of the path so rotating or
    /// running multiple bot tokens cannot cross-deduplicate their updates.
    pub async fn load(codex_home: &Path, bot_id: u64) -> Self {
        let state_path = codex_home
            .join("telegram")
            .join(format!("updates-{bot_id}.json"));
        let mut state = State::default();
        match tokio::fs::read_to_string(&state_path).await {
            Ok(contents) => match serde_json::from_str::<PersistedUpdates>(&contents) {
                Ok(persisted) => {
                    for id in persisted.completed_update_ids {
                        state.mark_completed(id);
                    }
                    for update in persisted.pending {
                        state.pending.insert(u64::from(update.id.0), update);
                    }
                }
                Err(err) => {
                    warn!(path = %state_path.display(), "ignoring corrupt Telegram update inbox: {err}");
                    let aside = state_path.with_extension("json.corrupt");
                    if let Err(rename_err) = tokio::fs::rename(&state_path, &aside).await {
                        warn!(path = %state_path.display(), "failed to rename corrupt Telegram update inbox: {rename_err}");
                    }
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                warn!(path = %state_path.display(), "failed to read Telegram update inbox: {err}")
            }
        }
        Self {
            inner: Mutex::new(state),
            persist_lock: Semaphore::new(1),
            state_path,
        }
    }

    /// Persist and claim an update before dispatch. A persisted pending replay
    /// is claimable after restart, but duplicates within this process are not.
    pub async fn begin_update(&self, update: &Update) -> BeginUpdate {
        let id = u64::from(update.id.0);
        let Ok(_persist_permit) = self.persist_lock.acquire().await else {
            warn!(update_id = id, "Telegram inbox persistence gate was closed");
            return BeginUpdate::PersistenceFailed;
        };
        let (before, snapshot) = {
            let mut state = self.inner.lock().await;
            if state.completed.contains(&id) || state.in_flight.contains(&id) {
                info!(update_id = id, "dropping duplicate Telegram update");
                return BeginUpdate::Duplicate;
            }
            if !state.pending.contains_key(&id) && state.pending.len() >= INBOX_CAPACITY {
                warn!(update_id = id, "Telegram durable inbox is full");
                return BeginUpdate::InboxFull;
            }
            let before = state.clone();
            state.pending.entry(id).or_insert_with(|| update.clone());
            state.in_flight.insert(id);
            (before, persisted_snapshot(&state))
        };
        if let Err(err) = self.persist_snapshot(snapshot).await {
            *self.inner.lock().await = before;
            warn!(
                update_id = id,
                "failed to persist Telegram update before dispatch: {err}"
            );
            return BeginUpdate::PersistenceFailed;
        }
        BeginUpdate::Accepted
    }

    /// Mark an update applied only after its handler completed successfully.
    pub async fn complete_update(&self, update_id: u64) -> anyhow::Result<()> {
        let _persist_permit = self
            .persist_lock
            .acquire()
            .await
            .map_err(|_| anyhow::anyhow!("Telegram inbox persistence gate was closed"))?;
        let (before, snapshot) = {
            let mut state = self.inner.lock().await;
            let before = state.clone();
            state.mark_completed(update_id);
            (before, persisted_snapshot(&state))
        };
        if let Err(err) = self.persist_snapshot(snapshot).await {
            *self.inner.lock().await = before;
            return Err(err);
        }
        Ok(())
    }

    /// Release a failed handler's in-process claim. The raw update remains
    /// pending on disk and will be replayed by the next connector start.
    pub async fn release_update(&self, update_id: u64) {
        let Ok(_persist_permit) = self.persist_lock.acquire().await else {
            return;
        };
        self.inner.lock().await.in_flight.remove(&update_id);
    }

    pub async fn pending_updates(&self) -> Vec<Update> {
        self.inner.lock().await.pending.values().cloned().collect()
    }

    async fn persist_snapshot(&self, snapshot: PersistedUpdates) -> anyhow::Result<()> {
        crate::persistence::write_atomically(&self.state_path, serde_json::to_string(&snapshot)?)
            .await
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            inner: Mutex::new(State::default()),
            persist_lock: Semaphore::new(1),
            state_path: std::env::temp_dir().join(format!(
                "codex-telegram-test-{}-{id}.json",
                std::process::id()
            )),
        }
    }

    #[cfg(test)]
    pub async fn len_for_test(&self) -> usize {
        self.inner.lock().await.completed.len()
    }

    #[cfg(test)]
    pub async fn pending_ids_for_test(&self) -> Vec<u64> {
        self.inner.lock().await.pending.keys().copied().collect()
    }
}

fn persisted_snapshot(state: &State) -> PersistedUpdates {
    PersistedUpdates {
        completed_update_ids: state.completed_order.iter().copied().collect(),
        pending: state.pending.values().cloned().collect(),
    }
}
