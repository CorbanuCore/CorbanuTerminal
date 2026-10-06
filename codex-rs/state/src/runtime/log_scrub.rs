//! One-time scrub of credentials that older builds wrote to the logs
//! database (#179, #196): redact them in existing rows, then rewrite the file
//! so deleted rows and freed pages cannot hold them either.
//!
//! Progress lives in the logs database's `PRAGMA user_version` (unused
//! otherwise, kept by `VACUUM`, ignored by older builds):
//! 0 = not done, 1 = rows scrubbed and file rewritten, 2 = WAL truncated too.

use super::*;

const ROWS_SCRUBBED_AND_VACUUMED: i64 = 1;
const DONE: i64 = 2;
// `sqlx::query` takes only literal SQL; keep these in step with the above.
const MARK_ROWS_SCRUBBED_AND_VACUUMED: &str = "PRAGMA user_version = 1";
const MARK_DONE: &str = "PRAGMA user_version = 2";
const BATCH_ROWS: i64 = 1_000;
const BATCH_BYTES: usize = 8 * 1024 * 1024;

impl StateRuntime {
    /// Start the scrub without delaying startup. A process that exits first
    /// leaves the database consistent; the next start continues.
    pub(super) fn spawn_log_scrub(self: &Arc<Self>) {
        let runtime = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(err) = runtime.scrub_logged_secrets_once().await {
                warn!("failed to scrub old secrets from the logs db: {err}");
            }
        });
    }

    pub(crate) async fn scrub_logged_secrets_once(&self) -> anyhow::Result<()> {
        let pool = self.logs_pool.as_ref();
        let mut stage = sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(pool)
            .await?;
        if stage < ROWS_SCRUBBED_AND_VACUUMED {
            self.scrub_log_rows().await?;
            // Rewrites every page, dropping deleted rows and freed space.
            sqlx::query("VACUUM").execute(pool).await?;
            sqlx::query(MARK_ROWS_SCRUBBED_AND_VACUUMED)
                .execute(pool)
                .await?;
            stage = ROWS_SCRUBBED_AND_VACUUMED;
        }
        if stage < DONE {
            // Copies the rewritten pages over the old ones and empties the WAL,
            // which can still hold old frames. Busy while another connection
            // reads; retried on the next start.
            let (busy, _, _) =
                sqlx::query_as::<_, (i64, i64, i64)>("PRAGMA wal_checkpoint(TRUNCATE)")
                    .fetch_one(pool)
                    .await?;
            if busy != 0 {
                tracing::debug!("logs db checkpoint busy; finishing the scrub on a later start");
                return Ok(());
            }
            sqlx::query(MARK_DONE).execute(pool).await?;
        }
        Ok(())
    }

    async fn scrub_log_rows(&self) -> anyhow::Result<()> {
        let pool = self.logs_pool.as_ref();
        let mut after_id = 0_i64;
        loop {
            // Pick the next rows up to the byte budget (at least one row).
            let sizes = sqlx::query_as::<_, (i64, i64)>(
                "SELECT id, COALESCE(length(CAST(feedback_log_body AS BLOB)), 0) FROM logs WHERE id > ? ORDER BY id LIMIT ?",
            )
            .bind(after_id)
            .bind(BATCH_ROWS)
            .fetch_all(pool)
            .await?;
            let Some(&(first_id, _)) = sizes.first() else {
                return Ok(());
            };
            let mut last_id = first_id;
            let mut batch_bytes = 0_usize;
            for (id, size) in sizes {
                batch_bytes += usize::try_from(size).unwrap_or(0);
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
}

#[cfg(test)]
#[path = "log_scrub_tests.rs"]
mod tests;
