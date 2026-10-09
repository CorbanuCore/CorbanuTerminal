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

#[cfg(test)]
#[path = "busy_retry_tests.rs"]
mod tests;
