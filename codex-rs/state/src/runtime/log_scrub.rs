//! One-time scrub of credentials that older builds wrote to the logs
//! database (#179, #196): redact them in existing rows, then rewrite the file
//! so deleted rows and freed pages cannot hold them either.
//!
//! Progress lives in the logs database's `PRAGMA user_version` (unused
//! otherwise, kept by `VACUUM`, ignored by older builds):
//! 0 = not done, 1 = rows scrubbed and file rewritten, 2 = WAL truncated too.
//! A new database starts at 2.
//!
//! `VACUUM` holds the write lock while it rewrites the file, so on a large
//! database other writers can wait out their busy timeout once, and closing
//! the pool waits for it to finish.

use std::collections::HashSet;
use std::sync::Mutex;

use super::*;

const ROWS_SCRUBBED_AND_VACUUMED: i64 = 1;
const DONE: i64 = 2;
// `sqlx::query` takes only literal SQL; keep these in step with the above.
const MARK_ROWS_SCRUBBED_AND_VACUUMED: &str = "PRAGMA user_version = 1";
const MARK_DONE: &str = "PRAGMA user_version = 2";
const BATCH_ROWS: i64 = 1_000;
const BATCH_BYTES: i64 = 8 * 1024 * 1024;

/// Logs databases this process has started scrubbing.
static STARTED: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

/// Mark a new database done, or scrub an existing one in the background so
/// startup is not delayed. A process that exits first leaves the database
/// consistent; the next start continues.
pub(super) async fn start(pool: &Arc<SqlitePool>, path: &Path, is_new: bool) {
    if is_new {
        if let Err(err) = sqlx::query(MARK_DONE).execute(pool.as_ref()).await {
            warn!("failed to mark a new logs db as scrubbed: {err}");
        }
        return;
    }
    let first_in_process = STARTED
        .lock()
        .map(|mut started| {
            started
                .get_or_insert_with(HashSet::new)
                .insert(path.to_path_buf())
        })
        .unwrap_or(false);
    if !first_in_process {
        return;
    }
    let pool = Arc::clone(pool);
    tokio::spawn(async move {
        if let Err(err) = scrub_once(&pool).await {
            warn!("failed to scrub old secrets from the logs db: {err}");
        }
    });
}

pub(super) async fn scrub_once(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut stage = sqlx::query_scalar::<_, i64>("PRAGMA user_version")
        .fetch_one(pool)
        .await?;
    if stage < ROWS_SCRUBBED_AND_VACUUMED {
        scrub_rows(pool).await?;
        sqlx::query("VACUUM").execute(pool).await?;
        sqlx::query(MARK_ROWS_SCRUBBED_AND_VACUUMED)
            .execute(pool)
            .await?;
        stage = ROWS_SCRUBBED_AND_VACUUMED;
    }
    if stage < DONE {
        // Copies the rewritten pages over the old ones and empties the WAL,
        // which can still hold old frames. A short timeout keeps a busy
        // result from stalling other writers; it is retried on a later start.
        let mut connection = pool.acquire().await?;
        connection.close_on_drop();
        sqlx::query("PRAGMA busy_timeout = 100")
            .execute(&mut *connection)
            .await?;
        let (busy, _, _) =
            sqlx::query_as::<_, (i64, i64, i64)>("PRAGMA wal_checkpoint(TRUNCATE)")
                .fetch_one(&mut *connection)
                .await?;
        if busy != 0 {
            tracing::debug!("logs db checkpoint busy; finishing the scrub on a later start");
            return Ok(());
        }
        sqlx::query(MARK_DONE).execute(&mut *connection).await?;
    }
    Ok(())
}

async fn scrub_rows(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut after_id = 0_i64;
    loop {
        // The next rows up to the byte budget (at least one row).
        let sizes = sqlx::query_as::<_, (i64, i64)>(
            "SELECT id, estimated_bytes FROM logs WHERE id > ? ORDER BY id LIMIT ?",
        )
        .bind(after_id)
        .bind(BATCH_ROWS)
        .fetch_all(pool)
        .await?;
        let Some(&(first_id, _)) = sizes.first() else {
            return Ok(());
        };
        let mut last_id = first_id;
        let mut batch_bytes = 0_i64;
        for (id, size) in sizes {
            batch_bytes += size.max(0);
            if id != first_id && batch_bytes > BATCH_BYTES {
                break;
            }
            last_id = id;
        }
        // BLOB: a body that is not valid UTF-8 must not stop the pass.
        let rows = sqlx::query_as::<_, (i64, Option<Vec<u8>>)>(
            "SELECT id, CAST(feedback_log_body AS BLOB) FROM logs WHERE id > ? AND id <= ? ORDER BY id",
        )
        .bind(after_id)
        .bind(last_id)
        .fetch_all(pool)
        .await?;
        after_id = last_id;
        let updates = rows
            .into_iter()
            .filter_map(|(id, body)| {
                let body = body?;
                let scrubbed = crate::log_scrub::scrub_bytes(&body)?;
                let delta = scrubbed.len() as i64 - body.len() as i64;
                Some((id, scrubbed, delta))
            })
            .collect::<Vec<_>>();
        if updates.is_empty() {
            continue;
        }
        let mut tx = pool.begin().await?;
        for (id, body, delta) in updates {
            sqlx::query(
                "UPDATE logs SET feedback_log_body = ?, estimated_bytes = estimated_bytes + ? WHERE id = ?",
            )
            .bind(body)
            .bind(delta)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
    }
}

#[cfg(test)]
#[path = "log_scrub_tests.rs"]
mod tests;
