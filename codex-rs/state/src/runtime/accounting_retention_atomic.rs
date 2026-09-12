//! Retained fixture consumers only. Coupled mutation is the separately allocated C2.
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::runtime::accounting::pricing::storage::lifecycle) struct RetentionCoverage {
    completed_as_of_ms: i64,
    detail_expired_through_ms: Option<i64>,
    aggregate_day_floor: i64,
    oldest_recorded_day: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::runtime::accounting::pricing::storage::lifecycle) enum RetainedDay {
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
