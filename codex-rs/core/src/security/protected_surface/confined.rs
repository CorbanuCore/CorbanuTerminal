//! PF-23-S02: whether a running process can read credentials. A process
//! keeps the sandbox it started with, so one started without the
//! protected-path rules (under a lower level, with the rules lifted by an
//! approval or a grant, or in a sandbox that cannot take them) still can.
//! Typing into it after untrusted content is a protected action of its own
//! (`write_stdin`).
//!
//! Recorded when the process is spawned, from the sandbox that really ran,
//! keyed by the host's process id plus a start number (ids are reused once
//! a process ends). Unknown means unconfined.

use crate::security::aggressive;
use crate::security::tainted_action::PolicyBinding;
use crate::security::tainted_action::PostTaintState;
use crate::security::tainted_action::ProtectedActionKind;
use codex_protocol::ThreadId;
use codex_security_policy::SecurityLevel;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

/// Starts remembered per session; past it the oldest is forgotten (and is
/// then unconfined: fail closed).
const MAX_PER_SESSION: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Start {
    /// Unique per spawn in this host.
    number: u64,
    confined: bool,
    /// The human allowed typing into it (Moderate) under this policy epoch.
    allowed_epoch: Option<u64>,
}

static NEXT_START: AtomicU64 = AtomicU64::new(1);
/// Starts by session, then by process id.
type Starts = HashMap<ThreadId, HashMap<i32, Start>>;

static STARTS: LazyLock<Mutex<Starts>> = LazyLock::new(Default::default);

fn with_starts<T>(f: impl FnOnce(&mut Starts) -> T) -> T {
    f(&mut STARTS.lock().unwrap_or_else(PoisonError::into_inner))
}

/// Process `process_id` was just spawned; `confined` says whether its
/// sandbox denies reading credentials.
pub(crate) fn note_start(thread: ThreadId, process_id: i32, confined: bool) {
    let number = NEXT_START.fetch_add(1, Ordering::Relaxed);
    with_starts(|starts| {
        let processes = starts.entry(thread).or_default();
        if processes.len() >= MAX_PER_SESSION
            && !processes.contains_key(&process_id)
            && let Some(oldest) = processes
                .iter()
                .min_by_key(|(_, start)| start.number)
                .map(|(id, _)| *id)
        {
            processes.remove(&oldest);
        }
        processes.insert(
            process_id,
            Start {
                number,
                confined,
                allowed_epoch: None,
            },
        );
    });
}

fn get(thread: ThreadId, process_id: i32) -> Option<Start> {
    with_starts(|starts| starts.get(&thread)?.get(&process_id).copied())
}

/// The grant operation for typing into this start of `process_id`, for the
/// grant TUI (PF-25-S01).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn grant_operation(thread: ThreadId, process_id: i32) -> Option<String> {
    get(thread, process_id).map(|start| aggressive::process_operation(process_id, start.number))
}

fn policy_epoch(state: &PostTaintState) -> u64 {
    match state.policy {
        PolicyBinding::Bound { epoch, .. } => epoch,
        PolicyBinding::Unbound | PolicyBinding::Unavailable => u64::MAX,
    }
}

/// What typing into `process_id` needs, beyond the typed text itself.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Typing {
    /// Confined, before untrusted content, or already allowed.
    Clear,
    /// Moderate: ask the human once for this process.
    AskOnce,
    /// Aggressive without a grant.
    Refused(String),
}

pub(crate) fn typing(thread: ThreadId, process_id: i32, state: &PostTaintState) -> Typing {
    if state.taint_generation == 0 {
        return Typing::Clear;
    }
    let start = get(thread, process_id);
    if start.is_some_and(|start| start.confined) {
        return Typing::Clear;
    }
    if state.level == SecurityLevel::Aggressive {
        let granted = start.and_then(|start| {
            aggressive::admit(
                thread,
                state,
                aggressive::Surface::UnconfinedProcess,
                &aggressive::process_operation(process_id, start.number),
                aggressive::now_unix_seconds(),
            )
        });
        return match granted {
            Some(_) => Typing::Clear,
            None => Typing::Refused(format!(
                "Not run: this is {} under security level Aggressive, which needs a grant from \
                 the human. Start a new process instead; it gets the protected rules.",
                ProtectedActionKind::UnconfinedProcess.describe()
            )),
        };
    }
    if start.is_some_and(|start| start.allowed_epoch == Some(policy_epoch(state))) {
        Typing::Clear
    } else {
        Typing::AskOnce
    }
}

/// The human allowed typing into the unconfined `process_id` (Moderate).
pub(crate) fn note_allowed(thread: ThreadId, process_id: i32, state: &PostTaintState) {
    let epoch = policy_epoch(state);
    with_starts(|starts| {
        if let Some(start) = starts
            .get_mut(&thread)
            .and_then(|processes| processes.get_mut(&process_id))
        {
            start.allowed_epoch = Some(epoch);
        }
    });
}

#[cfg(test)]
#[path = "confined_tests.rs"]
mod tests;
