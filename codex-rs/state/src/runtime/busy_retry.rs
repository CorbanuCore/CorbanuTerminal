//! Bounded retry for short state-DB writes that met another process's write lock.
//!
//! The state DB is shared by the TUI, `corbanu exec` workers and other Corbanu
//! processes. SQLite waits up to its 5 s busy timeout for the write lock and
//! then fails with SQLITE_BUSY ("database is locked"), which used to reach the
//! user as a failed turn. A write wrapped here is one transaction that rolls
//! back whole on that error, so it is simply made again, under one deadline.
use std::future::Future;
use std::time::Duration;
use std::time::Instant;
use tracing::info;
use tracing::warn;

/// Total time a write keeps retrying, counting SQLite's own busy waits.
const BUSY_RETRY_DEADLINE: Duration = Duration::from_secs(60);
const FIRST_PAUSE: Duration = Duration::from_millis(50);
const MAX_PAUSE: Duration = Duration::from_secs(1);
const SQLITE_BUSY: i32 = 5;
const SQLITE_READONLY: i32 = 8;
const SQLITE_IOERR: i32 = 10;
const SQLITE_FULL: i32 = 13;
const SQLITE_CANTOPEN: i32 = 14;

#[cfg(test)]
tokio::task_local! {
    /// Writes `retry_busy_within` attempted in this task, for tests that
    /// count them (`count_attempts`).
    static ATTEMPTS: std::cell::Cell<u32>;
}

/// Run `future`, returning its output and how many writes `retry_busy_within`
/// attempted in it.
#[cfg(test)]
pub(crate) async fn count_attempts<F: Future>(future: F) -> (F::Output, u32) {
    ATTEMPTS
        .scope(std::cell::Cell::new(0), async {
            let output = future.await;
            (output, ATTEMPTS.with(std::cell::Cell::get))
        })
        .await
}

/// Whether `error` is SQLITE_BUSY, in any of its extended codes: another
/// connection held the lock past the busy timeout. Nothing else is retried; a
/// pool timeout means this process's own connections are all in use, which
/// waiting longer on the lock does not help. SQLITE_LOCKED (6) is a conflict
/// inside one connection, which waiting does not clear either.
pub(crate) fn is_busy(error: &anyhow::Error) -> bool {
    has_primary_code(error, SQLITE_BUSY)
}

/// Whether `error` is SQLITE_READONLY, in any of its extended codes: the
/// database file (or its directory) cannot be written.
pub(crate) fn is_read_only(error: &anyhow::Error) -> bool {
    has_primary_code(error, SQLITE_READONLY)
}

/// Whether `error` is the storage failing for now - full (SQLITE_FULL), an
/// I/O error (SQLITE_IOERR) or a file that cannot be opened (SQLITE_CANTOPEN) -
/// rather than the data being refused.
pub(crate) fn is_storage_failure(error: &anyhow::Error) -> bool {
    [SQLITE_IOERR, SQLITE_FULL, SQLITE_CANTOPEN]
        .into_iter()
        .any(|primary| has_primary_code(error, primary))
}

fn has_primary_code(error: &anyhow::Error, primary: i32) -> bool {
    error
        .chain()
        .any(|cause| match cause.downcast_ref::<sqlx::Error>() {
            // The primary result code is the low byte.
            Some(sqlx::Error::Database(database)) => database
                .code()
                .and_then(|code| code.parse::<i32>().ok())
                .is_some_and(|code| code & 0xff == primary),
            _ => false,
        })
}

/// Run `write` until it succeeds, fails with anything but SQLITE_BUSY, or
/// `BUSY_RETRY_DEADLINE` has passed. `write` must be one transaction (or one
/// statement) that is safe to repeat after it failed. `operation` names it in
/// the log; it carries no data.
pub(crate) async fn retry_busy<T, F, Fut>(operation: &'static str, write: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    retry_busy_within(operation, BUSY_RETRY_DEADLINE, write).await
}

pub(crate) async fn retry_busy_within<T, F, Fut>(
    operation: &'static str,
    deadline: Duration,
    mut write: F,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let started = Instant::now();
    let mut pause = FIRST_PAUSE;
    let mut retries = 0u32;
    loop {
        #[cfg(test)]
        let _ = ATTEMPTS.try_with(|attempts| attempts.set(attempts.get() + 1));
        match write().await {
            Ok(value) => {
                if retries > 0 {
                    info!(
                        target: "codex_state::busy_retry",
                        operation,
                        retries,
                        waited_ms = started.elapsed().as_millis() as u64,
                        "state DB write succeeded after contention"
                    );
                }
                return Ok(value);
            }
            Err(error) if is_busy(&error) && started.elapsed() + pause < deadline => {
                retries += 1;
                warn!(
                    target: "codex_state::busy_retry",
                    operation,
                    retries,
                    waited_ms = started.elapsed().as_millis() as u64,
                    error = %format!("{error:#}"),
                    "state DB busy; retrying"
                );
                tokio::time::sleep(pause).await;
                pause = (pause * 2).min(MAX_PAUSE);
            }
            Err(error) => {
                if retries > 0 {
                    warn!(
                        target: "codex_state::busy_retry",
                        operation,
                        retries,
                        waited_ms = started.elapsed().as_millis() as u64,
                        error = %format!("{error:#}"),
                        "state DB write failed after retrying"
                    );
                }
                return Err(error);
            }
        }
    }
}

/// How long the state DB counts as busy after a write met another
/// connection's lock, unless a write gets through first.
pub(crate) const CONTENDED_FOR: Duration = Duration::from_secs(10);

/// Whether this process recently found the shared state DB's write lock held
/// by someone else. Writes that may be put off (thread metadata, accounting)
/// skip their attempt while it is set and are applied later instead, so a
/// model request does not wait the lock out again and again.
#[derive(Debug, Default)]
pub(crate) struct Contention {
    until: std::sync::Mutex<Option<Instant>>,
}

impl Contention {
    pub(crate) fn note(&self) {
        let until = Instant::now() + CONTENDED_FOR;
        *self
            .until
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(until);
    }

    pub(crate) fn clear(&self) {
        *self
            .until
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    pub(crate) fn active(&self) -> bool {
        self.until
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some_and(|until| Instant::now() < until)
    }
}

#[cfg(test)]
#[path = "busy_retry_tests.rs"]
mod tests;
