//! PF-23-S01: text typed into each running process since untrusted content
//! arrived, so a protected command split across `write_stdin` calls
//! (`corban` then `u vault list\n`) is judged whole. Only text that was
//! actually sent is kept; a human approval clears it.

use codex_protocol::ThreadId;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::Mutex;

/// Typed text kept per process; more is judged as unreadable.
pub(crate) const MAX_TYPED_BYTES: usize = 16 * 1024;
/// Processes tracked at once; the oldest is forgotten first.
const MAX_PROCESSES: usize = 256;

type Key = (ThreadId, i32);

#[derive(Default)]
struct Store {
    text: HashMap<Key, String>,
    order: VecDeque<Key>,
}

static STORE: LazyLock<Mutex<Store>> = LazyLock::new(Mutex::default);

/// The text a process will have read once `chars` is sent.
pub(crate) struct TypedWindow {
    key: Key,
    pub(crate) text: String,
}

impl TypedWindow {
    pub(crate) fn open(thread: ThreadId, process: i32, chars: &str) -> Self {
        let key = (thread, process);
        let previous = STORE
            .lock()
            .ok()
            .and_then(|store| store.text.get(&key).cloned())
            .unwrap_or_default();
        Self {
            key,
            text: previous + chars,
        }
    }

    pub(crate) fn overflows(&self) -> bool {
        self.text.len() > MAX_TYPED_BYTES
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
                }
            }
        }
    }

    /// A human approved the whole text: start again.
    pub(crate) fn clear(self) {
        if let Ok(mut store) = STORE.lock() {
            store.text.remove(&self.key);
            store.order.retain(|key| *key != self.key);
        }
    }
}
