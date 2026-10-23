//! Issue #505: confirming Aggressive in `/security` requires a person's
//! approval for every command that is not known read-only, from each
//! thread's next turn, before the restart that loads the rest of Aggressive.
//!
//! At confirmation every session of this process on the Corbanu home gets
//! the approval policy `untrusted` with you as the reviewer in its stored
//! settings, as a thread settings update would. Steering, child-agent
//! messages, MCP approvals and `/status` all read that one value. Sessions
//! that start later in this process on the home start with it. Until the
//! restart it holds: a later settings update (a client replaying its old
//! values) cannot lower it.
//!
//! A managed requirement that rules out either value keeps the session's own
//! settings; nothing overrides it, and the person is told in that thread.
//!
//! Only this process is covered here. #919: any other Corbanu process
//! running on the home (another Corbanu Terminal, a running `corbanu exec`
//! or IDE session) takes it from its watch of the saved state when its
//! sessions rise to Aggressive (`kill_switch_watch`); one started later
//! starts under Aggressive. Internal sessions (reviews, compaction, memory)
//! keep their own.

use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::Weak;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

use codex_protocol::ThreadId;

use crate::session::session::Session;

/// How long a confirmation waits for the sessions to take the change before
/// it reports; a session that has not answered by then still takes it.
const WAIT: Duration = Duration::from_secs(5);

/// What happened to one session's settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ApprovalsOutcome {
    /// An internal session (review, compaction, memory): unchanged.
    NotCovered,
    /// Its stored settings require approval for every command.
    Held,
    /// A managed requirement rules it out; the session keeps its own.
    Kept(String),
}

/// What a confirmation did to the sessions of this process (#505).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApprovalsAtConfirmation {
    /// Threads whose stored settings now require approval for every command.
    pub held: usize,
    /// Why a managed requirement kept a thread on its own settings, per
    /// thread.
    pub kept: Vec<String>,
    /// Threads that had not answered when the confirmation reported; they
    /// take it when they do.
    pub pending: usize,
}

struct Registered {
    home: PathBuf,
    thread: ThreadId,
    session: Weak<Session>,
    runtime: tokio::runtime::Handle,
}

#[derive(Default)]
struct State {
    /// Homes on which a person confirmed Aggressive in this process.
    homes: Vec<PathBuf>,
    sessions: Vec<Registered>,
    /// Threads whose stored settings hold approval for every command.
    held: HashSet<ThreadId>,
}

static STATE: LazyLock<Mutex<State>> = LazyLock::new(Default::default);

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The home as the policy trees key it (#619: one home, many spellings).
fn canonical(home: &Path) -> PathBuf {
    std::fs::canonicalize(home).unwrap_or_else(|_| home.to_path_buf())
}

/// Whether a person confirmed Aggressive for `codex_home` in this process,
/// so a session starting on it starts with approval for every command.
pub(crate) fn confirmed_here(codex_home: &Path) -> bool {
    let home = canonical(codex_home);
    state().homes.contains(&home)
}

/// Whether `thread`'s stored settings hold approval for every command
/// because a person confirmed Aggressive in this process.
pub fn approvals_held(thread: ThreadId) -> bool {
    state().held.contains(&thread)
}

/// Record a session's outcome (it starts held, or took the change).
pub(crate) fn note_outcome(thread: ThreadId, outcome: &ApprovalsOutcome) {
    let mut state = state();
    if *outcome == ApprovalsOutcome::Held {
        state.held.insert(thread);
    } else {
        state.held.remove(&thread);
    }
}

/// Register a session of this process on `codex_home`. A confirmation made
/// while it was starting reaches it here; `started_confirmed`: the home was
/// already confirmed when its settings were built, so it has its outcome.
pub(crate) fn register(codex_home: &Path, session: &Arc<Session>, started_confirmed: bool) {
    let home = canonical(codex_home);
    let runtime = tokio::runtime::Handle::current();
    let confirmed = {
        let mut state = state();
        state
            .sessions
            .retain(|registered| registered.session.strong_count() > 0);
        state.sessions.push(Registered {
            home: home.clone(),
            thread: session.thread_id,
            session: Arc::downgrade(session),
            runtime: runtime.clone(),
        });
        state.homes.contains(&home)
    };
    if confirmed && !started_confirmed {
        let session = Arc::clone(session);
        runtime.spawn(async move {
            let outcome = session.require_approval_for_every_command().await;
            note_outcome(session.thread_id, &outcome);
        });
    }
}

/// A person confirmed Aggressive for `codex_home` in this process: every
/// session of this process on it requires approval for every command from
/// its next turn, and sessions that start later on it start that way. Waits
/// up to [`WAIT`] for the sessions to answer. Blocking: call it off the
/// async runtime (the TUI commits on its own thread; tests use
/// `spawn_blocking`). On a runtime worker a session's task can wait behind the
/// blocked worker; it is then reported pending and still takes it.
pub(crate) fn confirm(codex_home: &Path) -> ApprovalsAtConfirmation {
    let home = canonical(codex_home);
    let sessions: Vec<(ThreadId, Arc<Session>, tokio::runtime::Handle)> = {
        let mut state = state();
        if !state.homes.contains(&home) {
            state.homes.push(home.clone());
        }
        state
            .sessions
            .retain(|registered| registered.session.strong_count() > 0);
        state
            .sessions
            .iter()
            .filter(|registered| registered.home == home)
            .filter_map(|registered| {
                registered
                    .session
                    .upgrade()
                    .map(|session| (registered.thread, session, registered.runtime.clone()))
            })
            .collect()
    };
    let (sender, receiver) = mpsc::channel();
    for (thread, session, runtime) in &sessions {
        let thread = *thread;
        let session = Arc::clone(session);
        let sender = sender.clone();
        runtime.spawn(async move {
            let outcome = session.require_approval_for_every_command().await;
            note_outcome(thread, &outcome);
            let _ = sender.send((thread, outcome));
        });
    }
    drop(sender);
    let mut report = ApprovalsAtConfirmation::default();
    let deadline = Instant::now() + WAIT;
    let mut answered = 0;
    while answered < sessions.len() {
        let Ok((thread, outcome)) =
            receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))
        else {
            break;
        };
        answered += 1;
        match outcome {
            ApprovalsOutcome::NotCovered => {}
            ApprovalsOutcome::Held => report.held += 1,
            ApprovalsOutcome::Kept(reason) => report.kept.push(format!("{thread}: {reason}")),
        }
    }
    report.pending = sessions.len() - answered;
    report
}

/// Shown in a thread whose managed configuration rules it out.
pub(crate) fn kept_warning(reason: &str) -> String {
    format!(
        "Aggressive: approval for every command was not applied to this thread ({reason}). Its approval settings stay as they are; Aggressive cannot be fully active here until that changes."
    )
}

#[cfg(all(test, unix))]
#[path = "confirmed_approvals_tests.rs"]
mod tests;
