//! #616: what Corbanu does when its home becomes unreadable.
//!
//! Long-running `corbanu exec` workers saw their home turn unreadable
//! ("Operation not permitted") for a few minutes at a time. This module
//! records diagnostics for such failures on the internal disk and lets exec
//! wait for access to return instead of ending the turn.

mod diagnostics;

use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use std::collections::HashMap;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

/// Headline for every report of an unreadable home.
pub const HOME_UNREADABLE_HEADLINE: &str =
    "Corbanu could not read its home (Operation not permitted)";

const FIRST_WAIT: Duration = Duration::from_secs(1);
const MAX_WAIT: Duration = Duration::from_secs(60);
const LOG_MAX_BYTES: u64 = 1024 * 1024;
const RECORD_INTERVAL: Duration = Duration::from_secs(60);

static WAIT_WINDOW: OnceLock<Duration> = OnceLock::new();

/// How long a request waits for an unreadable home to become readable again
/// before failing as usual. Zero (the default) never waits; `corbanu exec`
/// sets it once at startup.
pub fn set_home_wait_window(window: Duration) {
    let _ = WAIT_WINDOW.set(window);
}

fn home_wait_window() -> Duration {
    WAIT_WINDOW.get().copied().unwrap_or(Duration::ZERO)
}

/// Bounded exponential backoff for one request whose failures come from an
/// unreadable home: 1 s, doubling to at most 60 s, until `window` has passed
/// since the first such failure.
#[derive(Debug, Default)]
pub(crate) struct HomeWait {
    started: Option<Instant>,
    attempts: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct HomeWaitStep {
    pub(crate) attempt: u32,
    pub(crate) delay: Duration,
    pub(crate) remaining: Duration,
}

impl HomeWait {
    /// The next wait after a failure, or `None` when the failure is not a
    /// home failure or the window is spent (the caller then fails as usual).
    pub(crate) fn next_step(
        &mut self,
        home_unreadable: bool,
        now: Instant,
        window: Duration,
    ) -> Option<HomeWaitStep> {
        if !home_unreadable {
            return None;
        }
        let started = *self.started.get_or_insert(now);
        let remaining = window.checked_sub(now.saturating_duration_since(started))?;
        if remaining.is_zero() {
            return None;
        }
        self.attempts += 1;
        let doubling = FIRST_WAIT.saturating_mul(1u32 << (self.attempts - 1).min(16));
        Some(HomeWaitStep {
            attempt: self.attempts,
            delay: doubling.min(MAX_WAIT).min(remaining),
            remaining,
        })
    }
}

/// Waits before a retry when `err` came from an unreadable home and the wait
/// window allows it, telling the client about each attempt. Gives `err` back
/// when the caller should handle it as usual.
pub(crate) async fn wait_if_home_unreadable(
    wait: &mut HomeWait,
    sess: &Session,
    turn_context: &TurnContext,
    err: CodexErr,
) -> Result<(), CodexErr> {
    let now = tokio::time::Instant::now().into_std();
    let Some(step) = wait.next_step(err.is_home_unreadable(), now, home_wait_window()) else {
        return Err(err);
    };
    if let Some(mut failures) = home_failures() {
        failures.failed(wait.started.unwrap_or(now));
    }
    let message = format!(
        "{HOME_UNREADABLE_HEADLINE}; waiting {} for access to return (attempt {}, gives up in {})",
        human_duration(step.delay),
        step.attempt,
        human_duration(step.remaining),
    );
    tracing::warn!("{message}: {err}");
    sess.notify_stream_error(turn_context, message, err).await;
    tokio::time::sleep(step.delay).await;
    Ok(())
}

/// What this process knows about home failures: when the current run of
/// them started (a turn waiting on the home, or a rollout append failing with
/// EPERM), and which threads' rollout folders hold unwritten items. The
/// writer keeps those items until its next flush.
#[derive(Debug, Default)]
struct HomeFailures {
    since: Option<Instant>,
    unsaved: HashMap<ThreadId, PathBuf>,
}

static HOME_FAILURES: LazyLock<Mutex<HomeFailures>> = LazyLock::new(Mutex::default);

impl HomeFailures {
    fn failed(&mut self, now: Instant) {
        self.since.get_or_insert(now);
    }

    fn unwritten(&mut self, thread: ThreadId, dir: PathBuf, now: Instant) {
        self.failed(now);
        self.unsaved.insert(thread, dir);
    }

    /// An append wrote to the home: this thread is saved, and once none is
    /// left unsaved the run of failures is over. (A successful flush or
    /// persist does not get here; the exit probe then finds the folder
    /// readable and returns at once.)
    fn written(&mut self, thread: ThreadId) {
        self.unsaved.remove(&thread);
        if self.unsaved.is_empty() {
            self.since = None;
        }
    }

    /// The thread's persistence shut down or was discarded (#872): its items
    /// are written or lost, as nothing writes them later, so the exit wait
    /// has nothing to save for it. A stale entry would keep `since` and
    /// shorten the exit wait of a later, unrelated outage. A thread that
    /// ends shows nothing about the home, so the run ends only when this
    /// thread was the last one unsaved (a waiting turn keeps its start).
    fn ended(&mut self, thread: ThreadId) {
        if self.unsaved.remove(&thread).is_some() && self.unsaved.is_empty() {
            self.since = None;
        }
    }
}

fn home_failures() -> Option<std::sync::MutexGuard<'static, HomeFailures>> {
    HOME_FAILURES.lock().ok()
}

/// A rollout append of `thread` that had items to write succeeded.
pub(crate) fn note_rollout_written(thread: ThreadId) {
    if let Some(mut failures) = home_failures() {
        failures.written(thread);
    }
}

/// The live thread of `thread` shut down, whether or not its last write
/// succeeded, or was discarded after a failed session start.
pub(crate) fn note_thread_ended(thread: ThreadId) {
    if let Some(mut failures) = home_failures() {
        failures.ended(thread);
    }
}

/// Whether `thread` has rollout items the home did not take, and when this
/// run of home failures started.
#[cfg(test)]
pub(crate) fn unsaved_state(thread: ThreadId) -> (bool, Option<Instant>) {
    home_failures().map_or((false, None), |failures| {
        (failures.unsaved.contains_key(&thread), failures.since)
    })
}

/// A rollout append of `thread` failed with EPERM in `dir`.
pub(crate) fn note_rollout_unwritten(thread: ThreadId, dir: PathBuf) {
    if let Some(mut failures) = home_failures() {
        failures.unwritten(thread, dir, Instant::now());
    }
}

/// When rollout items are still unwritten, waits until their folders list
/// again, within what is left of the window since the first home failure of
/// this run (a turn that already waited leaves little or nothing). `corbanu
/// exec` calls it before the session is saved at exit, so the items are
/// written instead of lost.
pub async fn wait_for_unsaved_rollout() {
    let (dirs, since) = match home_failures() {
        Some(failures) if !failures.unsaved.is_empty() => (
            failures.unsaved.values().cloned().collect::<Vec<_>>(),
            failures.since,
        ),
        _ => return,
    };
    wait_until_readable(
        &dirs,
        |dir| {
            let dir = dir.to_path_buf();
            async move { tokio::fs::read_dir(dir).await.map(drop) }
        },
        |operation, path, error| {
            record_home_access_failure(operation, path, error);
        },
        HomeWait {
            started: since,
            attempts: 0,
        },
        home_wait_window(),
    )
    .await;
}

async fn wait_until_readable<F>(
    dirs: &[PathBuf],
    read_dir: impl Fn(&Path) -> F,
    mut record: impl FnMut(&str, &Path, &str),
    mut wait: HomeWait,
    window: Duration,
) where
    F: std::future::Future<Output = std::io::Result<()>>,
{
    const EPERM: i32 = 1;
    loop {
        let mut failure = None;
        for dir in dirs {
            if let Err(error) = read_dir(dir).await {
                failure = Some((dir, error));
                break;
            }
        }
        let Some((dir, error)) = failure else {
            return;
        };
        let not_permitted = error.raw_os_error() == Some(EPERM);
        let now = tokio::time::Instant::now().into_std();
        let Some(step) = wait.next_step(not_permitted, now, window) else {
            return;
        };
        if step.attempt == 1 {
            record("read_dir (opendir)", dir, &error.to_string());
        }
        // Exec prints errors only; each attempt must be visible.
        tracing::error!(
            "{HOME_UNREADABLE_HEADLINE}: {}; waiting {} before saving the session (attempt {}, \
             gives up in {})",
            dir.display(),
            human_duration(step.delay),
            step.attempt,
            human_duration(step.remaining),
        );
        tokio::time::sleep(step.delay).await;
    }
}

/// "45 s" or, from a minute up, whole minutes rounded up ("2 min").
fn human_duration(duration: Duration) -> String {
    let secs = duration.as_millis().div_ceil(1000);
    if secs < 60 {
        format!("{secs} s")
    } else {
        format!("{} min", secs.div_ceil(60))
    }
}

/// Whether an error message reports EPERM ("Operation not permitted").
pub(crate) fn is_operation_not_permitted(message: &str) -> bool {
    message.contains("Operation not permitted")
}

/// Whether this process's own sandbox denies reading `path` (macOS), a
/// failure that waiting cannot fix.
pub fn sandbox_denies_read(path: &Path) -> bool {
    diagnostics::sandbox_denies_read(path) == Some(true)
}

/// The diagnostics log on the internal disk (never under the Corbanu home).
/// Unit tests never write it: their failures are simulated, and `HOME` is
/// the operator's.
pub(crate) fn diagnostics_log_path() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    let user_home = dirs::home_dir()?;
    Some(if cfg!(target_os = "macos") {
        user_home.join("Library/Logs/Corbanu/home-access.log")
    } else {
        user_home.join(".local/state/corbanu/home-access.log")
    })
}

/// Appends a diagnostic record of `operation` failing on `path` to the
/// diagnostics log, at most once a minute per operation in this process.
/// Returns the log written to.
pub fn record_home_access_failure(operation: &str, path: &Path, error: &str) -> Option<PathBuf> {
    static LAST: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    let mut last = LAST.get_or_init(Mutex::default).lock().ok()?;
    let now = Instant::now();
    if last
        .get(operation)
        .is_some_and(|at| now.saturating_duration_since(*at) < RECORD_INTERVAL)
    {
        return None;
    }
    last.insert(operation.to_string(), now);
    drop(last);
    let log = diagnostics_log_path()?;
    match write_record(&log, operation, path, error) {
        Ok(()) => Some(log),
        Err(write_error) => {
            tracing::warn!("could not write {}: {write_error}", log.display());
            None
        }
    }
}

/// Appends one record to `log`, keeping one previous file when it grows past
/// 1 MiB.
pub(crate) fn write_record(
    log: &Path,
    operation: &str,
    path: &Path,
    error: &str,
) -> std::io::Result<()> {
    if let Some(parent) = log.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::metadata(log).is_ok_and(|metadata| metadata.len() > LOG_MAX_BYTES) {
        std::fs::rename(log, log.with_extension("log.1"))?;
    }
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let record = diagnostics::record(operation, path, error, &now);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)?
        .write_all(record.as_bytes())
}

/// What a user sees when the home cannot be read: the headline, the possible
/// causes (none is asserted) and where the details went.
pub fn unreadable_home_message(home: &Path, error: &str, log: Option<&Path>) -> String {
    let ssh = if std::env::var_os("SSH_CONNECTION").is_some() {
        " (over SSH: System Settings > General > Sharing > Remote Login > \"Allow full disk \
         access for remote users\")"
    } else {
        ""
    };
    let details = log.map_or_else(
        || "No diagnostics could be written.".to_string(),
        |log| format!("Details: {}", log.display()),
    );
    format!(
        "{HOME_UNREADABLE_HEADLINE}: {} ({error}).\nPossible causes:\n\
         - macOS privacy: the app that started Corbanu lacks Full Disk Access or Files and \
         Folders > Removable Volumes{ssh}\n\
         - the drive holding it was ejected, is remounting or is being checked\n\
         - a sandbox or security tool denied the read\n\
         - its permissions, ACL or flags changed\n\
         {details}",
        home.display()
    )
}

#[cfg(test)]
#[path = "home_access_tests.rs"]
mod tests;
