//! PF-23-S01: text typed into each running process since untrusted content
//! arrived, so a protected command split across `write_stdin` calls
//! (`corban` then `u vault list\n`) is judged whole. Only text that was
//! actually sent is kept; a human approval clears it. Writes to one process
//! are judged one at a time (see [`lock`]).

use codex_protocol::ThreadId;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

/// Typed text kept per process; more is judged as unreadable.
pub(crate) const MAX_TYPED_BYTES: usize = 16 * 1024;
/// Processes tracked at once across threads (each thread prunes its exited
/// processes, so this is only a backstop). A forgotten process's next write
/// is judged as unreadable.
const MAX_PROCESSES: usize = 4096;

type Key = (ThreadId, i32);

#[derive(Default)]
struct Store {
    text: HashMap<Key, String>,
    order: VecDeque<Key>,
    /// Evicted while text was kept: judged as unreadable once.
    lost: HashSet<Key>,
    locks: HashMap<Key, Arc<tokio::sync::Mutex<()>>>,
}

static STORE: LazyLock<Mutex<Store>> = LazyLock::new(Mutex::default);

/// Hold while judging and sending one write, so parallel writes to the same
/// process cannot each be judged against an empty window.
pub(crate) async fn lock(thread: ThreadId, process: i32) -> tokio::sync::OwnedMutexGuard<()> {
    let lock = STORE.lock().map_or_else(
        |_| Arc::new(tokio::sync::Mutex::new(())),
        |mut store| Arc::clone(store.locks.entry((thread, process)).or_default()),
    );
    lock.lock_owned().await
}

/// The text a process will have read once `chars` is sent.
pub(crate) struct TypedWindow {
    key: Key,
    lost: bool,
    pub(crate) text: String,
}

impl TypedWindow {
    /// `live`: this thread's running process ids; kept text of the others
    /// is dropped.
    pub(crate) fn open(thread: ThreadId, process: i32, chars: &str, live: &[i32]) -> Self {
        let key = (thread, process);
        let Ok(mut store) = STORE.lock() else {
            return Self {
                key,
                lost: true,
                text: chars.to_string(),
            };
        };
        let dead: Vec<Key> = store
            .text
            .keys()
            .chain(store.locks.keys())
            .filter(|(owner, id)| *owner == thread && !live.contains(id))
            .copied()
            .collect();
        for dead in dead {
            store.text.remove(&dead);
            store.lost.remove(&dead);
            if store
                .locks
                .get(&dead)
                .is_some_and(|lock| Arc::strong_count(lock) == 1)
            {
                store.locks.remove(&dead);
            }
        }
        let Store { text, order, .. } = &mut *store;
        order.retain(|key| text.contains_key(key));
        let previous = store.text.get(&key).cloned().unwrap_or_default();
        Self {
            key,
            lost: store.lost.contains(&key),
            text: previous + chars,
        }
    }

    /// Too much to judge, or earlier text was forgotten.
    pub(crate) fn unreadable(&self) -> bool {
        self.lost || self.text.len() > MAX_TYPED_BYTES
    }

    /// The text was sent after untrusted content: keep it.
    pub(crate) fn keep(self) {
        if let Ok(mut store) = STORE.lock() {
            if store.text.insert(self.key, self.text).is_none() {
                store.order.push_back(self.key);
            }
            while store.order.len() > MAX_PROCESSES {
                if let Some(oldest) = store.order.pop_front() {
                    store.text.remove(&oldest);
                    if store.lost.len() < MAX_PROCESSES {
                        store.lost.insert(oldest);
                    }
                }
            }
        }
    }

    /// A human approved the whole text: start again.
    pub(crate) fn clear(self) {
        if let Ok(mut store) = STORE.lock() {
            store.text.remove(&self.key);
            store.lost.remove(&self.key);
            store.order.retain(|key| *key != self.key);
        }
    }
}
