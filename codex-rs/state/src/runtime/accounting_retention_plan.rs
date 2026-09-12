//! Validated whole-store input only; retention reduction is a separate allocation.
use super::compact_values::CompactValues;
use super::*;
use std::collections::BTreeSet;

const DETAIL_MS: i64 = 90 * DAY_MS;
type DayKey = (String, i64);

#[derive(Debug, PartialEq, Eq)]
struct CompactInput {
    values: CompactValues,
    snapshots: BTreeSet<String>,
    expires_at_ms: i64,
}

#[derive(Debug, PartialEq, Eq)]
enum ContributionFreshness {
    Missing,
    Stale,
    Current,
}

#[derive(Debug, PartialEq, Eq)]
struct RawInput {
    quote: ObservationQuote,
    day: DayKey,
    detail_expires_at_ms: i64,
    replay_expires_at_ms: i64,
    day_expires_at_ms: i64,
    contribution: ContributionFreshness,
}

/// Ephemeral caller-snapshot input, not a retention plan or apply capability.
#[derive(Debug, PartialEq, Eq)]
struct ValidatedRetentionInput {
    previous_as_of_ms: Option<i64>,
    as_of_ms: i64,
    compact_days: BTreeMap<DayKey, CompactInput>,
    raw_attempts: BTreeMap<Uuid, RawInput>,
    tombstones: BTreeMap<Uuid, i64>,
}

fn day_key(thread: String, day: i64, as_of_ms: i64) -> anyhow::Result<(DayKey, i64)> {
    ensure!(
        ThreadId::from_string(&thread)
            .context("invalid thread key")?
            .to_string()
            == thread,
        "noncanonical thread key"
    );
    ensure!(day >= 0, "negative day");
    let start = day.checked_mul(DAY_MS).context("day start overflow")?;
    let expiry = start
        .checked_add(REPLAY_MS)
        .context("day expiry overflow")?;
    ensure!(start <= as_of_ms, "future day");
    Ok(((thread, day), expiry))
}

fn attempt_key(text: &str) -> anyhow::Result<Uuid> {
    let id = Uuid::parse_str(text).context("invalid attempt UUID")?;
    ensure!(id.to_string() == text, "noncanonical attempt key");
    Ok(id)
}

/// Caller must hold one transaction spanning all reads (and any future application).
/// This function never owns a transaction, persists a quote, or changes a row.
async fn read_validated_input_on_connection(
    conn: &mut SqliteConnection,
    as_of_ms: i64,
) -> anyhow::Result<ValidatedRetentionInput> {
    ensure!(as_of_ms >= 0, "negative as-of");
    // SQLx's integer decoding also rejects REAL/TEXT checkpoint values.
    let checkpoints: Vec<(i64, Option<i64>)> = sqlx::query_as(
        "SELECT singleton, completed_as_of_ms FROM draft_accounting_retention_checkpoint",
    )
    .fetch_all(&mut *conn)
    .await?;
    ensure!(checkpoints.len() == 1, "missing or multiple checkpoints");
    let (key, previous) = &checkpoints[0];
    ensure!(*key == 1, "invalid checkpoint key");
    ensure!(
        previous.is_none_or(|time| time >= 0 && time <= as_of_ms),
        "negative or backward checkpoint"
    );
    // Check all fixture references, including rows whose owners would otherwise be skipped.
    let orphans: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pragma_foreign_key_check WHERE \"table\" IN (
            'draft_accounting_observations', 'draft_accounting_price_bindings',
            'draft_accounting_estimates', 'draft_accounting_contributions',
            'draft_accounting_compact_snapshots')",
    )
    .fetch_one(&mut *conn)
    .await?;
    ensure!(orphans == 0, "orphan fixture reference");
    let mut input = ValidatedRetentionInput {
        previous_as_of_ms: *previous,
        as_of_ms,
        compact_days: BTreeMap::new(),
        raw_attempts: BTreeMap::new(),
        tombstones: BTreeMap::new(),
    };
    let snapshots: Vec<String> = sqlx::query_scalar(
        "SELECT snapshot_id FROM draft_accounting_price_snapshots ORDER BY snapshot_id",
    )
    .fetch_all(&mut *conn)
    .await?;
    for id in snapshots {
        let snapshot = read_snapshot(conn, &id)
            .await?
            .context("missing snapshot")?;
        ensure!(
            i64::from(snapshot.observed_at_ms) <= as_of_ms
                && i64::from(snapshot.approved_at_ms) <= as_of_ms,
            "future snapshot evidence"
        );
    }
    let days: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT thread_id, utc_day, payload FROM draft_accounting_compact_days ORDER BY thread_id, utc_day",
    ).fetch_all(&mut *conn).await?;
    for (thread, day, payload) in days {
        let (key, expiry) = day_key(thread, day, as_of_ms)?;
        let values = CompactValues::decode(&payload)?;
        // Keep every destination, including expired days; this reader never filters.
        input.compact_days.insert(
            key,
            CompactInput {
                values,
                snapshots: BTreeSet::new(),
                expires_at_ms: expiry,
            },
        );
    }
    let references: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT thread_id, utc_day, snapshot_id FROM draft_accounting_compact_snapshots ORDER BY thread_id, utc_day, snapshot_id",
    ).fetch_all(&mut *conn).await?;
    for (thread, day, snapshot) in references {
        let (key, _) = day_key(thread, day, as_of_ms)?;
        input
            .compact_days
            .get_mut(&key)
            .context("orphan compact reference")?
            .snapshots
            .insert(snapshot);
    }
    let tombstones: Vec<(String, i64)> = sqlx::query_as(
        "SELECT attempt_id, expires_at_ms FROM draft_accounting_tombstones ORDER BY attempt_id",
    )
    .fetch_all(&mut *conn)
    .await?;
    for (id, expiry) in tombstones {
        input.tombstones.insert(attempt_key(&id)?, expiry);
        let dispatch = expiry
            .checked_sub(REPLAY_MS)
            .context("replay expiry underflow")?;
        ensure!(
            dispatch >= 0 && dispatch <= as_of_ms,
            "invalid replay expiry"
        );
    }
    let contributions: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT attempt_id, thread_id, utc_day, evidence FROM draft_accounting_contributions ORDER BY attempt_id",
    ).fetch_all(&mut *conn).await?;
    let mut contributions: BTreeMap<_, _> = contributions
        .into_iter()
        .map(|(id, thread, day, evidence)| (id, (thread, day, evidence)))
        .collect();
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT attempt_id FROM draft_accounting_attempts ORDER BY attempt_id")
            .fetch_all(&mut *conn)
            .await?;
    for id in ids {
        let uuid = attempt_key(&id)?;
        ensure!(
            !input.tombstones.contains_key(&uuid),
            "raw/tombstone collision"
        );
        let quote = EstimateStore::latest_quote_on_connection(conn, uuid).await?;
        let attempt = &quote.attempt;
        let dispatch = i64::from(attempt.dispatched_at_ms);
        ensure!(dispatch <= as_of_ms, "future dispatch");
        let detail_expiry = dispatch
            .checked_add(DETAIL_MS)
            .context("detail expiry overflow")?;
        let replay_expiry = dispatch
            .checked_add(REPLAY_MS)
            .context("replay expiry overflow")?;
        let (key, day_expiry) =
            day_key(attempt.thread_id.to_string(), dispatch / DAY_MS, as_of_ms)?;
        let contribution = contributions.remove(&id);
        let contribution = if let Some((thread, day, evidence)) = &contribution {
            validate_reference(conn, attempt, thread, *day, evidence).await?;
            if *evidence == serde_json::to_string(&quote.observations)? {
                ContributionFreshness::Current
            } else {
                ContributionFreshness::Stale
            }
        } else {
            ContributionFreshness::Missing
        };
        input.raw_attempts.insert(
            uuid,
            RawInput {
                quote,
                day: key,
                detail_expires_at_ms: detail_expiry,
                replay_expires_at_ms: replay_expiry,
                day_expires_at_ms: day_expiry,
                contribution,
            },
        );
    }
    ensure!(contributions.is_empty(), "orphan contribution");
    Ok(input)
}

#[path = "accounting_retention_reduction.rs"]
mod reduction;

#[cfg(test)]
#[path = "accounting_retention_plan_tests.rs"]
mod tests;
