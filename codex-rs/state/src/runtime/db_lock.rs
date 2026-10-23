//! Timing of SQLite write locks, the shared state DB's above all.
//!
//! The state DB is shared by every Corbanu process on a home, and SQLite has
//! one write lock per database: while one connection holds it, every other
//! writer waits (up to the 5 s busy timeout) and then fails with
//! SQLITE_BUSY. [`begin_immediate`] takes the lock like
//! `pool.begin_with("BEGIN IMMEDIATE")` and logs, under `codex_state::db_lock`,
//! how long the lock took to get and how long it was held, with the caller's
//! source location. A hold longer than [`SLOW_HOLD`] is a warning: it is what
//! makes other sessions wait.
use sqlx::Sqlite;
use sqlx::SqliteConnection;
use sqlx::SqlitePool;
use sqlx::Transaction;
use std::panic::Location;
use std::time::Duration;
use std::time::Instant;

pub(crate) const LOG_TARGET: &str = "codex_state::db_lock";

/// A write transaction held longer than this is logged as a warning.
pub(crate) const SLOW_HOLD: Duration = Duration::from_millis(250);

/// A `BEGIN IMMEDIATE` transaction whose wait and hold are logged on release.
pub(crate) struct WriteTx {
    tx: Transaction<'static, Sqlite>,
    timer: HoldTimer,
}

/// Logs the hold when the transaction is released, including when it is
/// dropped without commit or rollback (sqlx then rolls it back).
struct HoldTimer {
    site: &'static Location<'static>,
    waited: Duration,
    acquired: Instant,
    released: bool,
}

/// Take the state DB's write lock (`BEGIN IMMEDIATE`) on a connection from
/// `pool`, timing the wait and, on release, the hold.
#[track_caller]
pub(crate) fn begin_immediate(
    pool: &SqlitePool,
) -> impl std::future::Future<Output = Result<WriteTx, sqlx::Error>> + '_ {
    let site = Location::caller();
    async move {
        let started = Instant::now();
        let result = pool.begin_with("BEGIN IMMEDIATE").await;
        let waited = started.elapsed();
        match result {
            Ok(tx) => Ok(WriteTx {
                tx,
                timer: HoldTimer {
                    site,
                    waited,
                    acquired: Instant::now(),
                    released: false,
                },
            }),
            Err(error) => {
                tracing::debug!(
                    target: LOG_TARGET,
                    site = %site,
                    wait_ms = waited.as_millis() as u64,
                    "write lock not acquired"
                );
                Err(error)
            }
        }
    }
}

impl WriteTx {
    pub(crate) async fn commit(self) -> Result<(), sqlx::Error> {
        let Self { tx, mut timer } = self;
        let result = tx.commit().await;
        timer.release("commit");
        result
    }

    pub(crate) async fn rollback(self) -> Result<(), sqlx::Error> {
        let Self { tx, mut timer } = self;
        let result = tx.rollback().await;
        timer.release("rollback");
        result
    }
}

impl HoldTimer {
    fn release(&mut self, how: &'static str) {
        self.released = true;
        let held = self.acquired.elapsed();
        let wait_ms = self.waited.as_millis() as u64;
        let held_ms = held.as_millis() as u64;
        if held > SLOW_HOLD {
            tracing::warn!(
                target: LOG_TARGET,
                site = %self.site,
                wait_ms,
                held_ms,
                how,
                "SQLite write lock held long; other connections wait for it (site names the database)"
            );
        } else {
            tracing::debug!(
                target: LOG_TARGET,
                site = %self.site,
                wait_ms,
                held_ms,
                how,
                "SQLite write lock released"
            );
        }
    }
}

impl Drop for HoldTimer {
    fn drop(&mut self) {
        if !self.released {
            self.release("drop");
        }
    }
}

impl std::ops::Deref for WriteTx {
    type Target = SqliteConnection;

    fn deref(&self) -> &SqliteConnection {
        &self.tx
    }
}

impl std::ops::DerefMut for WriteTx {
    fn deref_mut(&mut self) -> &mut SqliteConnection {
        &mut self.tx
    }
}
