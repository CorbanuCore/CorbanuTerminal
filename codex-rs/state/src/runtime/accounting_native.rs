//! Native owner selection and main-state deletion; excluded from normal builds.
use super::*;
use codex_protocol::ThreadId;
use sqlx::Sqlite;
use sqlx::SqlitePool;
use sqlx::Transaction;

impl Journal<'_> {
    pub(in crate::runtime::accounting) async fn append_native(
        &self,
        owner: ThreadId,
        attempt: &Attempt,
        batch: &[Observation],
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        let mut tx = self.runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        match Self::append_native_on_connection(&mut tx, owner, attempt, batch, as_of_ms).await {
            Ok(()) => tx.commit().await?,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }
        Ok(())
    }

    /// The caller owns BEGIN IMMEDIATE and commits admission and maintenance together.
    pub(in crate::runtime::accounting) async fn append_native_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        attempt: &Attempt,
        batch: &[Observation],
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        require_active(conn).await?;
        ensure!(owner == attempt.thread_id, "native owner mismatch");
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?)")
            .bind(owner.to_string())
            .fetch_one(&mut *conn)
            .await?;
        ensure!(exists, "native owner missing");
        Self::maintain_native_on_connection(conn, as_of_ms).await?;
        Self::append_on_connection(conn, attempt, batch).await
    }
}

async fn require_active(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    ensure!(
        matches!(
            retention_fixture_on_connection(conn).await?,
            RetentionFixture::Active(_)
        ),
        "native accounting requires active retention"
    );
    Ok(())
}

async fn installed(conn: &mut SqliteConnection) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name GLOB 'draft_accounting_*')",
    )
    .fetch_one(conn)
    .await?)
}

/// Fixture installation is explicit and not concurrent with native operations.
/// The absent-schema route retains the existing deferred transaction behavior.
pub(in crate::runtime) async fn begin_delete(
    pool: &SqlitePool,
) -> anyhow::Result<Transaction<'_, Sqlite>> {
    let accounting = installed(&mut *pool.acquire().await?).await?;
    Ok(if accounting {
        pool.begin_with("BEGIN IMMEDIATE").await?
    } else {
        pool.begin().await?
    })
}

pub(in crate::runtime) async fn delete_on_connection(
    conn: &mut SqliteConnection,
    owners: &[ThreadId],
    as_of_ms: i64,
) -> anyhow::Result<()> {
    if installed(conn).await? {
        require_active(conn).await?;
        for owner in owners {
            Journal::delete_native_on_connection(conn, *owner, as_of_ms).await?;
        }
    }
    Ok(())
}
