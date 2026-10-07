//! PF-23-S02: which running processes started inside the protected-path
//! rules. A process keeps the sandbox it started with, so one started
//! before the rules applied (before untrusted content under Moderate, or
//! with the rules lifted by an approval or a grant) can still read
//! credentials. Typing into it after untrusted content is a protected
//! action of its own (`write_stdin`).
//!
//! Keyed by the call that started the process. Unknown means unconfined.

use codex_protocol::ThreadId;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;

/// Starts remembered per session; the oldest are forgotten first (and are
/// then treated as unconfined: fail closed).
const MAX_PER_SESSION: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    /// Started with the protected-path rules.
    Confined,
    /// Unconfined, and the human allowed typing into it (Moderate), under
    /// this policy epoch.
    Allowed { epoch: u64 },
}

static STARTS: LazyLock<Mutex<HashMap<ThreadId, VecDeque<(String, Mark)>>>> =
    LazyLock::new(Default::default);

fn mark(thread: ThreadId, call_id: &str, mark: Mark) {
    let mut starts = STARTS.lock().unwrap_or_else(PoisonError::into_inner);
    let marks = starts.entry(thread).or_default();
    marks.retain(|(id, _)| id != call_id);
    if marks.len() >= MAX_PER_SESSION {
        marks.pop_front();
    }
    marks.push_back((call_id.to_string(), mark));
}

fn get(thread: ThreadId, call_id: &str) -> Option<Mark> {
    STARTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&thread)?
        .iter()
        .rev()
        .find(|(id, _)| id == call_id)
        .map(|(_, mark)| *mark)
}

/// The command started by `call_id` runs with the protected-path rules.
pub(crate) fn note_confined(thread: ThreadId, call_id: &str) {
    mark(thread, call_id, Mark::Confined);
}

/// The command started by `call_id` runs without them.
pub(crate) fn note_unconfined(thread: ThreadId, call_id: &str) {
    let mut starts = STARTS.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(marks) = starts.get_mut(&thread) {
        marks.retain(|(id, _)| id != call_id);
    }
}

pub(crate) fn is_confined(thread: ThreadId, call_id: &str) -> bool {
    get(thread, call_id) == Some(Mark::Confined)
}

/// The human allowed typing into the unconfined process `call_id` started.
pub(crate) fn note_allowed(thread: ThreadId, call_id: &str, epoch: u64) {
    mark(thread, call_id, Mark::Allowed { epoch });
}

pub(crate) fn is_allowed(thread: ThreadId, call_id: &str, epoch: u64) -> bool {
    get(thread, call_id) == Some(Mark::Allowed { epoch })
}
