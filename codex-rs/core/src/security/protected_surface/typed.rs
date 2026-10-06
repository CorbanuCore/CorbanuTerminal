//! PF-23-S01: text typed into each running process since untrusted content
//! arrived, so a protected command split across `write_stdin` calls
//! (`corban` then `u vault list\n`) is judged whole. Only text that was
//! actually sent is kept; a human approval clears it. Writes to one process,
//! interrupts included, are judged and sent one at a time (see [`lock`]).

use codex_protocol::ThreadId;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

/// Typed text kept per process; more is judged as unreadable.
pub(crate) const MAX_TYPED_BYTES: usize = 16 * 1024;
/// Interrupts remembered per process; more are judged as unreadable.
const MAX_INTERRUPTS: usize = 16;
/// Processes tracked at once across threads (each thread prunes its exited
/// processes, so this is only a backstop). A forgotten process's next write
/// is judged as unreadable.
const MAX_PROCESSES: usize = 4096;

type Key = (ThreadId, i32);

#[derive(Clone, Default)]
struct Kept {
    text: String,
    /// Offsets in `text` where a lone interrupt (Ctrl-C/Z) was sent: the
    /// program may have dropped what came before, so the text from each one
    /// on is judged too.
    starts: Vec<usize>,
}

#[derive(Default)]
struct Store {
    kept: HashMap<Key, Kept>,
    order: VecDeque<Key>,
    /// Evicted while text was kept: judged as unreadable until a human
    /// approves (tiny keys; only grows past the backstop).
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

/// A lone interrupt is being sent to the process after untrusted content
/// (call while holding [`lock`]).
pub(crate) fn note_interrupt(thread: ThreadId, process: i32) {
    if let Ok(mut store) = STORE.lock() {
        let key = (thread, process);
        if !store.kept.contains_key(&key) {
            store.order.push_back(key);
        }
        let kept = store.kept.entry(key).or_default();
        let at = kept.text.len();
        kept.starts.push(at);
    }
}

/// The text a process will have read once `chars` is sent.
pub(crate) struct TypedWindow {
    key: Key,
    lost: bool,
    kept: Kept,
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
                kept: Kept {
                    text: chars.to_string(),
                    starts: Vec::new(),
                },
            };
        };
        let dead: Vec<Key> = store
            .kept
            .keys()
            .chain(store.locks.keys())
            .filter(|(owner, id)| *owner == thread && !live.contains(id))
            .copied()
            .collect();
        for dead in dead {
            store.kept.remove(&dead);
            store.lost.remove(&dead);
        }
        let Store {
            kept, order, locks, ..
        } = &mut *store;
        order.retain(|key| kept.contains_key(key));
        // Idle locks of processes with no kept text (other threads too).
        locks.retain(|key, lock| kept.contains_key(key) || Arc::strong_count(lock) > 1);
        let mut window = store.kept.get(&key).cloned().unwrap_or_default();
        window.text.push_str(chars);
        Self {
            key,
            lost: store.lost.contains(&key),
            kept: window,
        }
    }

    /// The whole text, then the text from each interrupt on.
    pub(crate) fn texts(&self) -> Vec<String> {
        std::iter::once(self.kept.text.clone())
            .chain(
                self.kept
                    .starts
                    .iter()
                    .filter_map(|start| self.kept.text.get(*start..))
                    .map(str::to_string),
            )
            .collect()
    }

    /// The whole text the human is asked about.
    pub(crate) fn text(&self) -> &str {
        &self.kept.text
    }

    /// Too much to judge, or earlier text was forgotten.
    pub(crate) fn unreadable(&self) -> bool {
        self.lost
            || self.kept.text.len() > MAX_TYPED_BYTES
            || self.kept.starts.len() > MAX_INTERRUPTS
    }

    /// The text was sent after untrusted content: keep it.
    pub(crate) fn keep(self) {
        if let Ok(mut store) = STORE.lock() {
            if store.kept.insert(self.key, self.kept).is_none() {
                store.order.push_back(self.key);
            }
            while store.order.len() > MAX_PROCESSES {
                if let Some(oldest) = store.order.pop_front() {
                    store.kept.remove(&oldest);
                    store.lost.insert(oldest);
                }
            }
        }
    }

    /// A human approved the whole text: start again.
    pub(crate) fn clear(self) {
        if let Ok(mut store) = STORE.lock() {
            store.kept.remove(&self.key);
            store.lost.remove(&self.key);
            store.order.retain(|key| *key != self.key);
        }
    }
}
