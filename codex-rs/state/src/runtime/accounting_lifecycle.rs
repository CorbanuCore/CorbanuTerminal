//! Contribution lifecycle shared by fixtures and atomic normal-library operations.
use super::*;
use crate::runtime::accounting::RetentionFixture;
use crate::runtime::accounting::retention_fixture_on_connection;
use codex_protocol::ThreadId;
use std::collections::BTreeMap;

const DAY_MS: i64 = 86_400_000;
const REPLAY_MS: i64 = 365 * DAY_MS;

#[derive(Debug, PartialEq, Eq)]
pub enum Current<T> {
    Ready(T),
    NeedsEstimate,
    NeedsRefresh,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metric {
    pub known: i64,
    pub unknown: i64,
}

impl Metric {
    fn full(&self) -> Option<i64> {
        (self.unknown == 0).then_some(self.known)
    }

    fn add(&mut self, value: Option<i64>) -> anyhow::Result<()> {
        let slot = if value.is_some() {
            &mut self.known
        } else {
            &mut self.unknown
        };
        *slot = slot
            .checked_add(value.unwrap_or(1))
            .context("metric overflow")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DayTotals {
    // Input, noncached, read, write, output, reasoning, total: never price subsets twice.
    pub measured: [Metric; 7],
    pub known_usd: Decimal,
    pub unknown_estimates: i64,
    pub attempts: i64,
}

impl DayTotals {
    /// Sum each supplied attempt once using the ledger's exact arithmetic.
    pub fn from_quotes<'a>(
        quotes: impl IntoIterator<Item = &'a ObservationQuote>,
    ) -> anyhow::Result<Self> {
        let mut totals = Self::default();
        for quote in quotes {
            totals.add(quote)?;
        }
        Ok(totals)
    }

    fn full_usd(&self) -> Option<Decimal> {
        (self.attempts > 0 && self.unknown_estimates == 0).then_some(self.known_usd)
    }

    fn add(&mut self, quote: &ObservationQuote) -> anyhow::Result<()> {
        let u = &quote.usage;
        for (metric, value) in self.measured.iter_mut().zip([
            u.input,
            u.noncached,
            u.read,
            u.write,
            u.output,
            u.reasoning,
            u.total,
        ]) {
            metric.add(value)?;
        }
        self.known_usd = self.known_usd.add(quote.known_subtotal)?;
        self.unknown_estimates = self
            .unknown_estimates
            .checked_add(i64::from(quote.all_buckets_priced.is_none()))
            .context("estimate count overflow")?;
        self.attempts = self
            .attempts
            .checked_add(1)
            .context("attempt count overflow")?;
        Ok(())
    }
}

struct Lifecycle<'a> {
    estimates: EstimateStore<'a>,
}

impl<'a> Lifecycle<'a> {
    #[cfg(test)]
    async fn create_for_tests(runtime: &'a StateRuntime) -> anyhow::Result<Self> {
        let estimates = EstimateStore::create_for_tests(runtime).await?;
        sqlx::raw_sql(
            "CREATE TABLE draft_accounting_contributions (
                attempt_id TEXT PRIMARY KEY NOT NULL, thread_id TEXT NOT NULL,
                utc_day INTEGER NOT NULL CHECK(utc_day >= 0), evidence TEXT NOT NULL,
                FOREIGN KEY(attempt_id, evidence) REFERENCES draft_accounting_estimates(attempt_id, evidence));
             CREATE INDEX draft_accounting_contribution_day ON draft_accounting_contributions(thread_id, utc_day);
             CREATE TABLE draft_accounting_tombstones (
                attempt_id TEXT PRIMARY KEY NOT NULL,
                expires_at_ms INTEGER NOT NULL CHECK(expires_at_ms >= 0));",
        ).execute(runtime.pool.as_ref()).await?;
        Ok(Self { estimates })
    }

    async fn refresh_current(&self, id: Uuid) -> anyhow::Result<Current<()>> {
        let mut tx = self
            .estimates
            .journal
            .runtime
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await?;
        let result = Journal::refresh_contribution_on_connection(&mut tx, id).await;
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

    async fn read_day(&self, thread: ThreadId, day: i64) -> anyhow::Result<Current<DayTotals>> {
        ensure!(day >= 0, "negative day");
        let mut tx = self.estimates.journal.runtime.pool.begin().await?;
        ensure!(
            matches!(
                retention_fixture_on_connection(&mut tx).await?,
                RetentionFixture::Absent
            ),
            "installed retention requires read_retained_day"
        );
        let attempts = owned_attempts(&mut tx, thread).await?;
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT attempt_id, evidence FROM draft_accounting_contributions WHERE thread_id = ? AND utc_day = ?",
        ).bind(thread.to_string()).bind(day).fetch_all(&mut *tx).await?;
        let mut references: BTreeMap<_, _> = rows.into_iter().collect();
        let mut totals = DayTotals::default();
        let mut stale = false;
        for (attempt, observations) in attempts {
            if i64::from(attempt.dispatched_at_ms) / DAY_MS != day {
                continue;
            }
            let reference: Option<(String, i64, String)> = sqlx::query_as(
                "SELECT thread_id, utc_day, evidence FROM draft_accounting_contributions WHERE attempt_id = ?",
            ).bind(attempt.attempt_id.to_string()).fetch_optional(&mut *tx).await?;
            if let Some((owner, bucket, evidence)) = reference {
                let quote =
                    validate_reference(&mut tx, &attempt, &owner, bucket, &evidence).await?;
                references.remove(&attempt.attempt_id.to_string());
                stale |= evidence != serde_json::to_string(&observations)?;
                totals.add(&quote)?;
            } else {
                stale = true;
            }
        }
        ensure!(references.is_empty(), "foreign contribution attribution");
        tx.commit().await?;
        Ok(if stale {
            Current::NeedsRefresh
        } else {
            Current::Ready(totals)
        })
    }

    async fn delete_recorded_thread(&self, thread: ThreadId, as_of_ms: i64) -> anyhow::Result<()> {
        ensure!(as_of_ms >= 0, "negative deletion time");
        let mut tx = self
            .estimates
            .journal
            .runtime
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await?;
        let result = async {
            if !matches!(retention_fixture_on_connection(&mut tx).await?, RetentionFixture::Absent) {
                return retention_plan::reduction::atomic::delete_on_connection(&mut tx, thread, as_of_ms).await;
            }
            let attempts = owned_attempts(&mut tx, thread).await?;
            let mut snapshots = HashSet::new();
            for (attempt, _) in attempts {
                let dispatch = i64::from(attempt.dispatched_at_ms);
                ensure!(dispatch <= as_of_ms, "future dispatch");
                let expiry = dispatch.checked_add(REPLAY_MS).context("expiry overflow")?;
                let id = attempt.attempt_id.to_string();
                if let Some(Some(snapshot)) = binding(&mut tx, attempt.attempt_id).await? {
                    snapshots.insert(snapshot);
                }
                sqlx::query("INSERT INTO draft_accounting_tombstones VALUES (?, ?) ON CONFLICT(attempt_id) DO NOTHING")
                    .bind(&id).bind(expiry).execute(&mut *tx).await?;
                for sql in [
                    "DELETE FROM draft_accounting_contributions WHERE attempt_id = ?",
                    "DELETE FROM draft_accounting_estimates WHERE attempt_id = ?",
                    "DELETE FROM draft_accounting_price_bindings WHERE attempt_id = ?",
                    "DELETE FROM draft_accounting_observations WHERE attempt_id = ?",
                    "DELETE FROM draft_accounting_attempts WHERE attempt_id = ?",
                ] {
                    sqlx::query(sql)
                        .bind(&id).execute(&mut *tx).await?;
                }
            }
            for snapshot in snapshots {
                sqlx::query("DELETE FROM draft_accounting_price_snapshots WHERE snapshot_id = ? AND NOT EXISTS (SELECT 1 FROM draft_accounting_price_bindings WHERE snapshot_id = ?)")
                    .bind(&snapshot).bind(&snapshot).execute(&mut *tx).await?;
            }
            anyhow::Ok(())
        }.await;
        match result {
            Ok(()) => tx.commit().await,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }?;
        Ok(())
    }
}

async fn validate_reference(
    conn: &mut SqliteConnection,
    attempt: &Attempt,
    thread: &str,
    day: i64,
    evidence: &str,
) -> anyhow::Result<ObservationQuote> {
    ensure!(
        thread == attempt.thread_id.to_string()
            && day == i64::from(attempt.dispatched_at_ms) / DAY_MS,
        "contribution attribution mismatch"
    );
    EstimateStore::read_on_connection(conn, attempt.attempt_id, evidence)
        .await?
        .context("missing referenced estimate")
}

async fn owned_attempts(
    conn: &mut SqliteConnection,
    thread: ThreadId,
) -> anyhow::Result<Vec<(Attempt, Vec<Observation>)>> {
    // Ownership lives in typed JSON: parse every row before filtering, never hide malformed owners.
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT attempt_id FROM draft_accounting_attempts ORDER BY attempt_id")
            .fetch_all(&mut *conn)
            .await?;
    let mut owned = Vec::new();
    for id in ids {
        let parsed = Uuid::parse_str(&id)?;
        ensure!(parsed.to_string() == id, "noncanonical attempt key");
        let record = authority(conn, parsed).await?;
        if record.0.thread_id == thread {
            owned.push(record);
        }
    }
    Ok(owned)
}

#[cfg(test)]
#[path = "accounting_lifecycle_tests.rs"]
mod tests;

#[path = "accounting_compact_values.rs"]
mod compact_values;

#[path = "accounting_retention_plan.rs"]
mod retention_plan;
pub use retention_plan::reduction::atomic::RetainedDay;
pub use retention_plan::reduction::atomic::RetentionCoverage;

impl Journal<'_> {
    pub(in crate::runtime::accounting) async fn inspect_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        day: i64,
        read_at_ms: i64,
    ) -> anyhow::Result<crate::accounting::InspectionDay> {
        Self::inspect_window_on_connection(conn, owner, day, read_at_ms, None).await
    }

    pub(in crate::runtime::accounting) async fn inspect_window_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        day: i64,
        read_at_ms: i64,
        window: Option<(i64, i64)>,
    ) -> anyhow::Result<crate::accounting::InspectionDay> {
        use crate::accounting::Inspection;
        use crate::accounting::InspectionDay;
        let checkpoint = match retention_fixture_on_connection(conn).await? {
            RetentionFixture::Active(time) => time,
            RetentionFixture::Staging => return Ok(InspectionDay::CheckpointLag),
            RetentionFixture::Absent => anyhow::bail!("missing retention schema"),
        };
        ensure!(read_at_ms >= checkpoint, "backward inspection read");
        // The inherited retained reader validates the whole store. Bound its inputs
        // before it deserializes any payload; no LIMIT-based partial total.
        let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM draft_accounting_attempts")
            .fetch_one(&mut *conn)
            .await?;
        if rows > 10_000 {
            return Ok(InspectionDay::TooLarge);
        }
        let excessive: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM draft_accounting_observations GROUP BY attempt_id HAVING count(*) > 4096)",
        ).fetch_one(&mut *conn).await?;
        if excessive {
            return Ok(InspectionDay::TooLarge);
        }
        // Conservative input budget also bounds historical estimate versions and
        // unrelated retained data visited by the inherited whole-store validator.
        let bytes: i64 = sqlx::query_scalar(
            "SELECT coalesce(sum(n), 0) FROM (
             SELECT length(CAST(payload AS BLOB)) AS n FROM draft_accounting_attempts UNION ALL
             SELECT length(CAST(payload AS BLOB)) FROM draft_accounting_observations UNION ALL
             SELECT length(CAST(payload AS BLOB)) FROM draft_accounting_price_snapshots UNION ALL
             SELECT length(CAST(payload AS BLOB)) + length(CAST(evidence AS BLOB)) FROM draft_accounting_estimates UNION ALL
             SELECT length(CAST(evidence AS BLOB)) FROM draft_accounting_contributions UNION ALL
             SELECT length(CAST(payload AS BLOB)) FROM draft_accounting_compact_days UNION ALL
             SELECT length(thread_id) + length(snapshot_id) + 8 FROM draft_accounting_compact_snapshots UNION ALL
             SELECT length(attempt_id) + 8 FROM draft_accounting_tombstones)",
        ).fetch_one(&mut *conn).await?;
        if bytes > 4 * 1024 * 1024 {
            return Ok(InspectionDay::TooLarge);
        }
        if day > checkpoint / DAY_MS {
            return Ok(InspectionDay::CheckpointLag);
        }
        let selected: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM draft_accounting_attempts WHERE json_extract(payload, '$.thread_id') = ? AND json_extract(payload, '$.dispatched_at_ms') / 86400000 = ?",
        ).bind(owner.to_string()).bind(day).fetch_one(&mut *conn).await?;
        if selected > 512 {
            return Ok(InspectionDay::TooLarge);
        }
        // Validate original bindings and old estimates before freshness hides corruption.
        let ids: Vec<String> =
            sqlx::query_scalar("SELECT attempt_id FROM draft_accounting_attempts")
                .fetch_all(&mut *conn)
                .await?;
        for id in ids {
            Self::inspect_quote_on_connection(conn, Uuid::parse_str(&id)?).await?;
        }
        let missing: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM draft_accounting_contributions c LEFT JOIN draft_accounting_estimates e ON c.attempt_id = e.attempt_id AND c.evidence = e.evidence WHERE e.attempt_id IS NULL)",
        ).fetch_one(&mut *conn).await?;
        if missing {
            return Ok(InspectionDay::NeedsRefresh);
        }
        let retained = Self::retained_day_on_connection(conn, owner, day, checkpoint).await?;
        let (coverage, expected) = match retained {
            RetainedDay::Available {
                coverage,
                totals: Current::Ready(totals),
            } => (coverage, totals),
            RetainedDay::Expired(coverage) => {
                return Ok(InspectionDay::DetailUnavailable {
                    coverage,
                    read_at_ms,
                    compact: false,
                });
            }
            RetainedDay::Available { .. } => return Ok(InspectionDay::NeedsRefresh),
            RetainedDay::NeedsMaintenance { .. } => {
                // Retention can stop before reporting raw-day freshness. Preserve
                // that diagnostic for retained raw contributions in this day only.
                for (attempt, observations) in owned_attempts(conn, owner).await? {
                    let dispatch = i64::from(attempt.dispatched_at_ms);
                    if dispatch / DAY_MS != day || dispatch <= checkpoint - 90 * DAY_MS {
                        continue;
                    }
                    let evidence: Option<String> = sqlx::query_scalar(
                        "SELECT evidence FROM draft_accounting_contributions WHERE attempt_id = ?",
                    )
                    .bind(attempt.attempt_id.to_string())
                    .fetch_optional(&mut *conn)
                    .await?;
                    if evidence.as_deref() != Some(serde_json::to_string(&observations)?.as_str()) {
                        return Ok(InspectionDay::NeedsRefresh);
                    }
                }
                return Ok(InspectionDay::CheckpointLag);
            }
            RetainedDay::NeedsActivation => return Ok(InspectionDay::CheckpointLag),
        };
        let compact: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM draft_accounting_compact_days WHERE thread_id = ? AND utc_day = ?)",
        ).bind(owner.to_string()).bind(day).fetch_one(&mut *conn).await?;
        let start = day.checked_mul(DAY_MS).context("day overflow")?;
        let (lower, upper) = window.unwrap_or((start, start + DAY_MS));
        // A compact prefix cannot supply detail, but retained raw suffixes can.
        if (compact && window.is_none())
            || (read_at_ms >= 90 * DAY_MS && lower <= read_at_ms - 90 * DAY_MS)
        {
            return Ok(InspectionDay::DetailUnavailable {
                coverage,
                read_at_ms,
                compact,
            });
        }
        let records = owned_attempts(conn, owner).await?;
        let selected = records
            .iter()
            .filter(|(a, _)| i64::from(a.dispatched_at_ms) / DAY_MS == day)
            .count();
        if selected > 512 {
            return Ok(InspectionDay::TooLarge);
        }
        let mut requests: BTreeMap<Uuid, Vec<ObservationQuote>> = BTreeMap::new();
        let mut totals = DayTotals::default();
        // Budget the packet header, totals, map keys and per-row containers too.
        let mut projected = 8192usize;
        for (attempt, _) in records {
            let dispatch = i64::from(attempt.dispatched_at_ms);
            if dispatch / DAY_MS != day || dispatch < lower || dispatch >= upper {
                continue;
            }
            // Check logical ownership across days and threads, including retries.
            let siblings: Vec<String> = sqlx::query_scalar(
                "SELECT payload FROM draft_accounting_attempts WHERE request_id = ?",
            )
            .bind(attempt.request_id.to_string())
            .fetch_all(&mut *conn)
            .await?;
            for payload in siblings {
                ensure!(
                    attempt.same_owner(&serde_json::from_str(&payload)?),
                    "request owner mismatch"
                );
            }
            let mut predecessor = attempt.retry_of;
            let mut seen = HashSet::from([attempt.attempt_id]);
            while let Some(id) = predecessor {
                ensure!(seen.insert(id), "retry cycle");
                let parent = read_attempt(conn, id).await?;
                // Compaction can legitimately remove a cross-day predecessor.
                let Some(parent) = parent else { break };
                ensure!(attempt.same_owner(&parent), "retry owner mismatch");
                predecessor = parent.retry_of;
            }
            let quote = Self::inspect_quote_on_connection(conn, attempt.attempt_id).await?;
            projected = projected
                .checked_add(serde_json::to_vec(&quote)?.len() + 2048)
                .context("inspection size overflow")?;
            if projected > 4 * 1024 * 1024 {
                return Ok(InspectionDay::TooLarge);
            }
            totals.add(&quote)?;
            requests.entry(attempt.request_id).or_default().push(quote);
        }
        if window.is_none() || (!compact && lower == start && upper == start + DAY_MS) {
            ensure!(totals == expected, "inspection contribution mismatch");
        }
        Ok(InspectionDay::Ready(Inspection {
            owner,
            utc_day: day,
            read_at_ms,
            coverage,
            own_totals: totals.clone(),
            descendant_totals: DayTotals::default(),
            unknown_parent_totals: DayTotals::default(),
            unknown_parent_unavailable_threads: 0,
            unknown_parent_requests: BTreeMap::new(),
            totals,
            requests,
        }))
    }

    pub(in crate::runtime::accounting) async fn refresh_contribution_on_connection(
        conn: &mut SqliteConnection,
        id: Uuid,
    ) -> anyhow::Result<Current<()>> {
        let deleted: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM draft_accounting_tombstones WHERE attempt_id = ?)",
        )
        .bind(id.to_string())
        .fetch_one(&mut *conn)
        .await?;
        ensure!(!deleted, "deleted attempt");
        let (attempt, observations) = authority(conn, id).await?;
        let evidence = serde_json::to_string(&observations)?;
        if EstimateStore::read_on_connection(conn, id, &evidence)
            .await?
            .is_none()
        {
            return Ok(Current::NeedsEstimate);
        }
        let old: Option<(String, i64, String)> = sqlx::query_as(
            "SELECT thread_id, utc_day, evidence FROM draft_accounting_contributions WHERE attempt_id = ?",
        ).bind(id.to_string()).fetch_optional(&mut *conn).await?;
        if let Some((thread, day, old_evidence)) = old {
            validate_reference(conn, &attempt, &thread, day, &old_evidence).await?;
            if old_evidence == evidence {
                return Ok(Current::Ready(()));
            }
        }
        sqlx::query("INSERT INTO draft_accounting_contributions VALUES (?, ?, ?, ?) ON CONFLICT(attempt_id) DO UPDATE SET evidence = excluded.evidence")
            .bind(id.to_string()).bind(attempt.thread_id.to_string())
            .bind(i64::from(attempt.dispatched_at_ms) / DAY_MS).bind(evidence)
            .execute(&mut *conn).await?;
        anyhow::Ok(Current::Ready(()))
    }
}
