//! Retained-detail fixture lifecycle; no native ownership or retention service.
use super::*;
use crate::runtime::accounting::RetentionFixture;
use crate::runtime::accounting::retention_fixture_on_connection;
use codex_protocol::ThreadId;
use std::collections::BTreeMap;

const DAY_MS: i64 = 86_400_000;
const REPLAY_MS: i64 = 365 * DAY_MS;

#[derive(Debug, PartialEq, Eq)]
enum Current<T> {
    Ready(T),
    NeedsEstimate,
    NeedsRefresh,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Metric {
    known: i64,
    unknown: i64,
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
struct DayTotals {
    // Input, noncached, read, write, output, reasoning, total: never price subsets twice.
    measured: [Metric; 7],
    known_usd: Decimal,
    unknown_estimates: i64,
    attempts: i64,
}

impl DayTotals {
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
        let result = async {
            let deleted: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM draft_accounting_tombstones WHERE attempt_id = ?)")
                .bind(id.to_string()).fetch_one(&mut *tx).await?;
            ensure!(!deleted, "deleted attempt");
            let (attempt, observations) = authority(&mut tx, id).await?;
            let evidence = serde_json::to_string(&observations)?;
            if EstimateStore::read_on_connection(&mut tx, id, &evidence).await?.is_none() {
                return Ok(Current::NeedsEstimate);
            }
            let old: Option<(String, i64, String)> = sqlx::query_as(
                "SELECT thread_id, utc_day, evidence FROM draft_accounting_contributions WHERE attempt_id = ?",
            ).bind(id.to_string()).fetch_optional(&mut *tx).await?;
            if let Some((thread, day, old_evidence)) = old {
                validate_reference(&mut tx, &attempt, &thread, day, &old_evidence).await?;
                if old_evidence == evidence {
                    return Ok(Current::Ready(()));
                }
            }
            sqlx::query("INSERT INTO draft_accounting_contributions VALUES (?, ?, ?, ?) ON CONFLICT(attempt_id) DO UPDATE SET evidence = excluded.evidence")
                .bind(id.to_string()).bind(attempt.thread_id.to_string())
                .bind(i64::from(attempt.dispatched_at_ms) / DAY_MS).bind(evidence)
                .execute(&mut *tx).await?;
            anyhow::Ok(Current::Ready(()))
        }.await;
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

#[path = "accounting_lifecycle_tests.rs"]
mod tests;

#[path = "accounting_compact_values.rs"]
mod compact_values;

#[path = "accounting_retention_plan.rs"]
mod retention_plan;
