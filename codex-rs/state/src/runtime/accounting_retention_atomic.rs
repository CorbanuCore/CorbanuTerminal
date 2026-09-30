//! Complete atomic retention and deletion shared by fixtures and installed stores.
use super::*;

#[path = "accounting_late_import.rs"]
mod late_import;

impl Journal<'_> {
    pub(in crate::runtime::accounting) async fn retained_day_on_connection(
        conn: &mut SqliteConnection,
        thread: ThreadId,
        day: i64,
        as_of_ms: i64,
    ) -> anyhow::Result<RetainedDay> {
        read_retained_on_connection(conn, thread, day, as_of_ms).await
    }

    /// Check one day's exact aggregate after a write without re-reading the whole ledger.
    pub(in crate::runtime::accounting) async fn check_day_on_connection(
        conn: &mut SqliteConnection,
        thread: ThreadId,
        day: i64,
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        check_day_on_connection(conn, thread, day, as_of_ms).await
    }

    pub(in crate::runtime::accounting) async fn maintain_native_on_connection(
        conn: &mut SqliteConnection,
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        maintain_on_connection(conn, as_of_ms).await
    }

    /// Maintenance on the per-request write path; see `maintain_for_write_on_connection`.
    pub(in crate::runtime::accounting) async fn maintain_for_write_on_connection(
        conn: &mut SqliteConnection,
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        maintain_for_write_on_connection(conn, as_of_ms).await
    }

    pub(in crate::runtime::accounting) async fn delete_native_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        as_of_ms: i64,
    ) -> anyhow::Result<()> {
        delete_on_connection(conn, owner, as_of_ms).await
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RetentionCoverage {
    pub completed_as_of_ms: i64,
    pub detail_expired_through_ms: Option<i64>,
    pub aggregate_day_floor: i64,
    pub oldest_recorded_day: Option<i64>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Eq)]
pub enum RetainedDay {
    NeedsActivation,
    NeedsMaintenance {
        completed_as_of_ms: i64,
    },
    Expired(RetentionCoverage),
    Available {
        coverage: RetentionCoverage,
        totals: Current<DayTotals>,
    },
}

impl Lifecycle<'_> {
    #[cfg_attr(not(test), allow(dead_code))]
    async fn maintain_retention(&self, as_of_ms: i64) -> anyhow::Result<()> {
        let mut tx = self
            .estimates
            .journal
            .runtime
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await?;
        match maintain_on_connection(&mut tx, as_of_ms).await {
            Ok(()) => tx.commit().await?,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }
        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::runtime::accounting::pricing::storage::lifecycle) async fn read_retained_day(
        &self,
        thread: ThreadId,
        day: i64,
        as_of_ms: i64,
    ) -> anyhow::Result<RetainedDay> {
        let mut tx = self.estimates.journal.runtime.pool.begin().await?;
        let result = read_retained_on_connection(&mut tx, thread, day, as_of_ms).await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }
}

/// Caller holds BEGIN IMMEDIATE across preparation, application and commit.
async fn maintain_on_connection(conn: &mut SqliteConnection, as_of_ms: i64) -> anyhow::Result<()> {
    ensure!(
        !matches!(
            retention_fixture_on_connection(conn).await?,
            RetentionFixture::Absent
        ),
        "retention schema absent"
    );
    let plan = prepare_on_connection(conn, as_of_ms).await?;
    for ((thread, day), compact) in plan.compact_days {
        sqlx::query("INSERT INTO draft_accounting_compact_days VALUES (?, ?, ?) ON CONFLICT(thread_id, utc_day) DO UPDATE SET payload = excluded.payload")
            .bind(&thread).bind(day).bind(compact.values.encode()?).execute(&mut *conn).await?;
        for snapshot in compact.snapshots {
            sqlx::query("INSERT INTO draft_accounting_compact_snapshots VALUES (?, ?, ?) ON CONFLICT(thread_id, utc_day, snapshot_id) DO NOTHING")
                .bind(&thread).bind(day).bind(snapshot).execute(&mut *conn).await?;
        }
    }
    for (id, expiry) in plan.raw_removals {
        remove_raw(conn, id, expiry, as_of_ms).await?;
    }
    for (thread, day) in plan.expired_days {
        for sql in [
            "DELETE FROM draft_accounting_compact_snapshots WHERE thread_id = ? AND utc_day = ?",
            "DELETE FROM draft_accounting_compact_days WHERE thread_id = ? AND utc_day = ?",
        ] {
            sqlx::query(sql)
                .bind(&thread)
                .bind(day)
                .execute(&mut *conn)
                .await?;
        }
    }
    gc_snapshots(conn).await?;
    sqlx::query("DELETE FROM draft_accounting_tombstones WHERE expires_at_ms <= ?")
        .bind(as_of_ms)
        .execute(&mut *conn)
        .await?;
    let updated = sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = ?, admission_active = 1 WHERE singleton = 1")
        .bind(as_of_ms).execute(conn).await?;
    ensure!(updated.rows_affected() == 1, "missing checkpoint update");
    Ok(())
}

const FULL_SWEEP_INTERVAL_MS: i64 = 60 * 60 * 1000;

/// Caller holds BEGIN IMMEDIATE. Every request writes here several times, while
/// the full sweep reads, validates and prices every retained attempt under the
/// write lock. On a large ledger that holds the lock for seconds and starves the
/// session's other writers and any other Corbanu process sharing the state DB.
///
/// Run the full sweep on the first write of each UTC hour, or whenever anything
/// aged out before this hour began. Otherwise, the hour's full sweep has already
/// validated the ledger and applied every earlier expiry, so only the checkpoint
/// advances. Detail that ages out mid-hour is retired by the next hour's sweep;
/// inspection reports that span as checkpoint lag in the meantime.
async fn maintain_for_write_on_connection(
    conn: &mut SqliteConnection,
    as_of_ms: i64,
) -> anyhow::Result<()> {
    let hour_start = as_of_ms - as_of_ms.rem_euclid(FULL_SWEEP_INTERVAL_MS);
    if let RetentionFixture::Active(checkpoint) = retention_fixture_on_connection(conn).await?
        && (hour_start..=as_of_ms).contains(&checkpoint)
        && !retention_due_on_connection(conn, hour_start).await?
    {
        let updated = sqlx::query(
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = ? WHERE singleton = 1",
        )
        .bind(as_of_ms)
        .execute(conn)
        .await?;
        ensure!(updated.rows_affected() == 1, "missing checkpoint update");
        return Ok(());
    }
    maintain_on_connection(conn, as_of_ms).await
}

/// Whether a full sweep at `as_of_ms` would expire raw detail, a compact day or a tombstone.
async fn retention_due_on_connection(
    conn: &mut SqliteConnection,
    as_of_ms: i64,
) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM draft_accounting_attempts
                WHERE json_extract(payload, '$.dispatched_at_ms') + ? <= ?)
            OR EXISTS(SELECT 1 FROM draft_accounting_compact_days WHERE utc_day * ? + ? <= ?)
            OR EXISTS(SELECT 1 FROM draft_accounting_tombstones WHERE expires_at_ms <= ?)",
    )
    .bind(DETAIL_MS)
    .bind(as_of_ms)
    .bind(DAY_MS)
    .bind(REPLAY_MS)
    .bind(as_of_ms)
    .bind(as_of_ms)
    .fetch_one(conn)
    .await?)
}

/// Caller holds the write transaction and has just maintained retention at `as_of_ms`.
/// Sums the compact and raw evidence a retained read of this day would, so an
/// aggregate that overflows exact storage still rejects the write before commit.
async fn check_day_on_connection(
    conn: &mut SqliteConnection,
    thread: ThreadId,
    day: i64,
    as_of_ms: i64,
) -> anyhow::Result<()> {
    let ((thread_key, _), _) = day_key(thread.to_string(), day, as_of_ms)?;
    let start = day * DAY_MS;
    let end = start.checked_add(DAY_MS).context("day end overflow")?;
    let mut values = CompactValues::from_day_totals(&DayTotals::default())?;
    let compact: Option<String> = sqlx::query_scalar(
        "SELECT payload FROM draft_accounting_compact_days WHERE thread_id = ? AND utc_day = ?",
    )
    .bind(&thread_key)
    .bind(day)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(payload) = compact {
        values = values.checked_add(&CompactValues::decode(&payload)?)?;
    }
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT attempt_id FROM draft_accounting_attempts
            WHERE json_extract(payload, '$.thread_id') = ?
              AND json_extract(payload, '$.dispatched_at_ms') >= ?
              AND json_extract(payload, '$.dispatched_at_ms') < ?
            ORDER BY attempt_id",
    )
    .bind(&thread_key)
    .bind(start)
    .bind(end)
    .fetch_all(&mut *conn)
    .await?;
    let mut totals = DayTotals::default();
    for id in ids {
        let quote = EstimateStore::latest_quote_on_connection(conn, attempt_key(&id)?).await?;
        ensure!(
            quote.attempt.thread_id == thread
                && i64::from(quote.attempt.dispatched_at_ms) / DAY_MS == day,
            "attempt payload does not match its day"
        );
        totals.add(&quote)?;
    }
    values
        .checked_add(&CompactValues::from_day_totals(&totals)?)?
        .to_day_totals()?;
    Ok(())
}

/// The lifecycle wrapper owns the single transaction, including preceding maintenance.
pub(in crate::runtime::accounting::pricing::storage::lifecycle) async fn delete_on_connection(
    conn: &mut SqliteConnection,
    thread: ThreadId,
    as_of_ms: i64,
) -> anyhow::Result<()> {
    maintain_on_connection(conn, as_of_ms).await?;
    for (attempt, _) in owned_attempts(conn, thread).await? {
        let expiry = i64::from(attempt.dispatched_at_ms)
            .checked_add(REPLAY_MS)
            .context("expiry overflow")?;
        remove_raw(conn, attempt.attempt_id, expiry, as_of_ms).await?;
    }
    for sql in [
        "DELETE FROM draft_accounting_compact_snapshots WHERE thread_id = ?",
        "DELETE FROM draft_accounting_compact_days WHERE thread_id = ?",
    ] {
        sqlx::query(sql)
            .bind(thread.to_string())
            .execute(&mut *conn)
            .await?;
    }
    gc_snapshots(conn).await
}

async fn remove_raw(
    conn: &mut SqliteConnection,
    id: Uuid,
    expiry: i64,
    as_of_ms: i64,
) -> anyhow::Result<()> {
    let id = id.to_string();
    if expiry > as_of_ms {
        sqlx::query("INSERT INTO draft_accounting_tombstones VALUES (?, ?)")
            .bind(&id)
            .bind(expiry)
            .execute(&mut *conn)
            .await?;
    }
    for sql in [
        "DELETE FROM draft_accounting_contributions WHERE attempt_id = ?",
        "DELETE FROM draft_accounting_estimates WHERE attempt_id = ?",
        "DELETE FROM draft_accounting_price_bindings WHERE attempt_id = ?",
        "DELETE FROM draft_accounting_observations WHERE attempt_id = ?",
        "DELETE FROM draft_accounting_attempts WHERE attempt_id = ?",
    ] {
        sqlx::query(sql).bind(&id).execute(&mut *conn).await?;
    }
    Ok(())
}

async fn gc_snapshots(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM draft_accounting_price_snapshots AS price WHERE NOT EXISTS (SELECT 1 FROM draft_accounting_price_bindings WHERE snapshot_id = price.snapshot_id) AND NOT EXISTS (SELECT 1 FROM draft_accounting_compact_snapshots WHERE snapshot_id = price.snapshot_id)")
        .execute(conn).await?;
    Ok(())
}

/// Caller holds one read snapshot; this path never refreshes or advances retention.
pub(in crate::runtime::accounting::pricing::storage::lifecycle) async fn read_retained_on_connection(
    conn: &mut SqliteConnection,
    thread: ThreadId,
    day: i64,
    as_of_ms: i64,
) -> anyhow::Result<RetainedDay> {
    ensure!(as_of_ms >= 0, "negative as-of");
    let (key, _) = day_key(thread.to_string(), day, as_of_ms)?;
    let checkpoint = match retention_fixture_on_connection(conn).await? {
        RetentionFixture::Absent => anyhow::bail!("retention schema absent"),
        RetentionFixture::Staging => return Ok(RetainedDay::NeedsActivation),
        RetentionFixture::Active(time) => time,
    };
    ensure!(as_of_ms >= checkpoint, "backward retained read");
    let maintenance = RetainedDay::NeedsMaintenance {
        completed_as_of_ms: checkpoint,
    };
    if as_of_ms > checkpoint {
        return Ok(maintenance);
    }
    let plan = prepare_on_connection(conn, as_of_ms).await?;
    let expired_tombstones: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM draft_accounting_tombstones WHERE expires_at_ms <= ?)",
    )
    .bind(as_of_ms)
    .fetch_one(&mut *conn)
    .await?;
    if !plan.raw_removals.is_empty() || !plan.expired_days.is_empty() || expired_tombstones {
        return Ok(maintenance);
    }
    let coverage = RetentionCoverage {
        completed_as_of_ms: checkpoint,
        detail_expired_through_ms: (checkpoint >= DETAIL_MS).then(|| checkpoint - DETAIL_MS),
        aggregate_day_floor: if checkpoint < REPLAY_MS {
            0
        } else {
            (checkpoint - REPLAY_MS) / DAY_MS + 1
        },
        oldest_recorded_day: plan
            .compact_days
            .keys()
            .chain(plan.raw_days.keys())
            .filter(|(owner, _)| owner == &key.0)
            .map(|(_, day)| *day)
            .min(),
    };
    if day < coverage.aggregate_day_floor {
        return Ok(RetainedDay::Expired(coverage));
    }
    let mut values = CompactValues::from_day_totals(&DayTotals::default())?;
    if let Some(compact) = plan.compact_days.get(&key) {
        values = values.checked_add(&compact.values)?;
    }
    let totals = match plan.raw_days.get(&key) {
        Some(Current::Ready(raw)) => Current::Ready(
            values
                .checked_add(&CompactValues::from_day_totals(raw)?)?
                .to_day_totals()?,
        ),
        Some(Current::NeedsEstimate | Current::NeedsRefresh) => Current::NeedsRefresh,
        None => Current::Ready(values.to_day_totals()?),
    };
    Ok(RetainedDay::Available { coverage, totals })
}

#[cfg(test)]
#[path = "accounting_retention_consumers_tests.rs"]
mod consumer_tests;

#[cfg(test)]
#[path = "accounting_retention_atomic_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "accounting_native_tests.rs"]
mod native_tests;
