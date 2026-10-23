//! Native owner selection and main-state deletion for explicitly installed accounting.
use super::*;
use codex_protocol::ThreadId;
use sqlx::SqlitePool;

impl Journal<'_> {
    pub(super) async fn store_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        attempt: &Attempt,
        batch: &[Observation],
        original_prices: Option<&[super::pricing::Snapshot]>,
        as_of_ms: i64,
        validated_at_ms: Option<i64>,
    ) -> anyhow::Result<super::pricing::ObservationQuote> {
        super::store::validate_on_connection(conn).await?;
        if original_prices.is_none() {
            let bound: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM draft_accounting_price_bindings WHERE attempt_id = ?)",
            )
            .bind(attempt.attempt_id.to_string())
            .fetch_one(&mut *conn)
            .await?;
            if !bound {
                return Err(NotAdmitted.into());
            }
        }
        Self::append_validated_on_connection(
            conn,
            owner,
            attempt,
            batch,
            as_of_ms,
            validated_at_ms,
        )
        .await?;
        let quote = Self::persist_price_on_connection(
            conn,
            attempt.attempt_id,
            original_prices.unwrap_or(&[]),
        )
        .await?;
        ensure!(
            matches!(
                Self::refresh_contribution_on_connection(conn, attempt.attempt_id).await?,
                super::pricing::Current::Ready(())
            ),
            "atomic contribution missing"
        );
        // Validate the resulting raw + compact day before committing any new evidence.
        // Individual quotes can fit while their aggregate overflows exact storage.
        Self::check_day_on_connection(
            conn,
            owner,
            i64::from(attempt.dispatched_at_ms) / 86_400_000,
            as_of_ms,
        )
        .await?;
        Ok(quote)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::runtime::accounting) async fn append_native(
        &self,
        owner: ThreadId,
        attempt: &Attempt,
        batch: &[Observation],
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        let mut tx = crate::runtime::db_lock::begin_immediate(&self.runtime.pool).await?;
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
        Self::append_validated_on_connection(
            conn, owner, attempt, batch, as_of_ms, /*validated_at_ms*/ None,
        )
        .await
    }

    /// As `append_native_on_connection`, after this hour's validation ran on a
    /// read snapshot at `validated_at_ms`, if it did.
    async fn append_validated_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        attempt: &Attempt,
        batch: &[Observation],
        as_of_ms: i64,
        validated_at_ms: Option<i64>,
    ) -> anyhow::Result<()> {
        require_active(conn).await?;
        ensure!(owner == attempt.thread_id, "native owner mismatch");
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?)")
            .bind(owner.to_string())
            .fetch_one(&mut *conn)
            .await?;
        if !exists {
            return Err(OwnerMissing.into());
        }
        Self::maintain_for_write_on_connection(conn, as_of_ms, validated_at_ms).await?;
        Self::append_on_connection(conn, attempt, batch).await
    }
}

/// The conversation a record belongs to has no row in `threads` (yet): its
/// metadata write may have been put off while the state DB was busy.
#[derive(Debug)]
pub(crate) struct OwnerMissing;

impl std::fmt::Display for OwnerMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("native owner missing")
    }
}

impl std::error::Error for OwnerMissing {}

/// Usage evidence for an attempt whose admission is not in the ledger (yet):
/// it may still be queued in another process's ledger writer.
#[derive(Debug)]
pub(crate) struct NotAdmitted;

impl std::fmt::Display for NotAdmitted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("attempt not admitted")
    }
}

impl std::error::Error for NotAdmitted {}

/// A retry whose predecessor attempt is not in the ledger (yet): it may still
/// be queued in another process's ledger writer. Its message is the one this
/// check always had.
#[derive(Debug)]
pub(crate) struct PredecessorMissing;

impl std::fmt::Display for PredecessorMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid retry predecessor")
    }
}

impl std::error::Error for PredecessorMissing {}

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
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name GLOB 'draft_accounting_*' OR name = '_accounting_migrations')",
    )
    .fetch_one(conn)
    .await?)
}

/// Acquire the writer lock before schema inspection, even with accounting absent.
/// No concurrent install or ordinary write can invalidate the deletion snapshot.
pub(in crate::runtime) async fn begin_delete(
    pool: &SqlitePool,
) -> anyhow::Result<crate::runtime::db_lock::WriteTx> {
    Ok(crate::runtime::db_lock::begin_immediate(pool).await?)
}

/// `validated_at_ms`: when this process validated the ledger this hour, from
/// `AccountingStore::prepare_delete`; maintenance then stays bounded.
pub(in crate::runtime) async fn delete_on_connection(
    conn: &mut SqliteConnection,
    owners: &[ThreadId],
    as_of_ms: i64,
    validated_at_ms: Option<i64>,
) -> anyhow::Result<()> {
    if installed(conn).await? {
        if super::store::ledger_exists(conn).await? {
            super::store::validate_on_connection(conn).await?;
        } else {
            // Only existing unit fixtures may use an unversioned schema.
            #[cfg(not(test))]
            anyhow::bail!("unversioned accounting schema");
            #[cfg(test)]
            require_active(conn).await?;
        }
        // One maintenance for the whole deletion, not one per owner: each is
        // a sweep under the write lock (#324).
        Journal::delete_native_owners_on_connection(conn, owners, as_of_ms, validated_at_ms)
            .await?;
    }
    Ok(())
}
