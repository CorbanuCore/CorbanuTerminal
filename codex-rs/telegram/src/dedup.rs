//! Duplicate Telegram update protection for mutating actions.
//!
//! WHY (2026-07-22): Telegram can deliver the same `update_id` more than once
//! — long-poll reconnects, service restarts before the offset is acknowledged,
//! and Telegram's own at-least-once delivery semantics all replay updates. For
//! a connector whose updates trigger *mutating* agent actions (starting turns,
//! approving commands, changing models), a replayed update can fire the same
//! action twice: two identical turns, a double approval, a duplicated spend of
//! paid model tokens. The Bot API gives no idempotency guarantee, so the
//! connector must deduplicate itself.
//!
//! Design: a bounded in-memory set of recently seen update ids, persisted
//! alongside the session state so a restart cannot replay the in-flight tail
//! of the stream. Bounded so a long-lived poller does not grow memory without
//! limit; the window only needs to cover Telegram's realistic replay horizon
//! (minutes to a restart), not the whole history.

use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::info;
use tracing::warn;

/// How many recent update ids to remember. Telegram replays only unacknowledged
/// updates, and long-poll acknowledges by advancing the offset on the next
/// request, so the realistic duplicate window is small. 1024 covers hours of
/// typical operator traffic with a large safety margin.
pub const DEDUP_WINDOW: usize = 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedSeen {
    #[serde(default)]
    update_ids: Vec<u64>,
}

/// Bounded, persisted tracker of processed Telegram update ids.
///
/// `check_and_record` is the single entry point: it returns `true` the first
/// time an id is seen and `false` for every repeat, recording the id before
/// returning so a crash after the check still cannot re-fire the action.
#[derive(Debug)]
pub struct UpdateDeduplicator {
    inner: Mutex<State>,
    state_path: PathBuf,
}

#[derive(Debug, Default)]
struct State {
    seen: HashSet<u64>,
    order: VecDeque<u64>,
    dirty: bool,
}

impl State {
    fn insert(&mut self, id: u64) -> bool {
        if self.seen.contains(&id) {
            return false;
        }
        self.seen.insert(id);
        self.order.push_back(id);
        while self.order.len() > DEDUP_WINDOW {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        self.dirty = true;
        true
    }
}

impl UpdateDeduplicator {
    /// Load the persisted tail of seen ids from `<codex_home>/telegram/updates.json`.
    /// A missing or corrupt file starts empty (never blocks startup); corrupt
    /// files are renamed aside for inspection, matching the session store.
    pub async fn load(codex_home: &Path) -> Self {
        let state_path = codex_home.join("telegram").join("updates.json");
        let mut state = State::default();
        match tokio::fs::read_to_string(&state_path).await {
            Ok(contents) => match serde_json::from_str::<PersistedSeen>(&contents) {
                Ok(persisted) => {
                    for id in persisted.update_ids {
                        state.insert(id);
                    }
                    // Loaded ids are already on disk; no need to re-persist immediately.
                    state.dirty = false;
                }
                Err(err) => {
                    warn!(
                        path = %state_path.display(),
                        "ignoring corrupt Telegram update-dedup file: {err}"
                    );
                    let aside = state_path.with_extension("json.corrupt");
                    if let Err(rename_err) = tokio::fs::rename(&state_path, &aside).await {
                        warn!(
                            path = %state_path.display(),
                            "failed to rename corrupt Telegram update-dedup file: {rename_err}"
                        );
                    }
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                warn!(
                    path = %state_path.display(),
                    "failed to read Telegram update-dedup file (starting empty): {err}"
                );
            }
        }
        Self {
            inner: Mutex::new(state),
            state_path,
        }
    }

    /// Returns `true` if this update id has never been seen (caller should
    /// process the update), `false` if it is a duplicate (caller must skip).
    /// First-seen ids are persisted best-effort before returning.
    pub async fn check_and_record(&self, update_id: u64) -> bool {
        let first_seen = {
            let mut state = self.inner.lock().await;
            state.insert(update_id)
        };
        if !first_seen {
            info!(update_id, "dropping duplicate Telegram update");
            return false;
        }
        if let Err(err) = self.persist().await {
            // Dedup still holds in memory for this process; a persist failure only
            // widens the replay window after a crash. Log and continue rather
            // than dropping a legitimate update.
            warn!("failed to persist Telegram update-dedup state: {err}");
        }
        true
    }

    async fn persist(&self) -> anyhow::Result<()> {
        let snapshot = {
            let state = self.inner.lock().await;
            if !state.dirty {
                return Ok(());
            }
            let ids: Vec<u64> = state.order.iter().copied().collect();
            ids
        };
        if let Some(parent) = self.state_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let tmp = self.state_path.with_extension("json.tmp");
        let body = serde_json::to_string(&PersistedSeen {
            update_ids: snapshot,
        })?;
        tokio::fs::write(&tmp, body).await?;
        tokio::fs::rename(&tmp, &self.state_path).await?;
        let mut state = self.inner.lock().await;
        state.dirty = false;
        Ok(())
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self {
            inner: Mutex::new(State::default()),
            state_path: PathBuf::from("/nonexistent/updates.json"),
        }
    }

    #[cfg(test)]
    pub async fn len_for_test(&self) -> usize {
        self.inner.lock().await.seen.len()
    }
}
