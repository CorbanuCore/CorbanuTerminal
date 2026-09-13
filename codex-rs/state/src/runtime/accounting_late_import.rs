//! Complete compact-only imports. The facade owns the writer lock and commit.
use super::*;
use crate::runtime::accounting::read_attempt;
use crate::runtime::accounting::store::OriginalPriceEvidence;
use crate::runtime::accounting::store::RetainedImport;
use crate::runtime::accounting::store::RetainedImportOutcome;
use crate::runtime::accounting::store::validate_on_connection;

impl Journal<'_> {
    pub(in crate::runtime::accounting) async fn import_retained_on_connection(
        conn: &mut SqliteConnection,
        owner: ThreadId,
        bundle: &[RetainedImport],
        as_of_ms: i64,
    ) -> anyhow::Result<Vec<RetainedImportOutcome>> {
        ensure!((1..=64).contains(&bundle.len()), "import attempt bound");
        let mut count = 0usize;
        for entry in bundle {
            ensure!(entry.observations.len() <= 256, "import observation bound");
            count = count
                .checked_add(entry.observations.len())
                .context("import size overflow")?;
        }
        ensure!(count <= 4096, "import bundle observation bound");
        validate_on_connection(conn).await?;
        let RetentionFixture::Active(checkpoint) = retention_fixture_on_connection(conn).await?
        else {
            anyhow::bail!("import requires active accounting");
        };
        ensure!(
            as_of_ms >= 0 && as_of_ms >= checkpoint,
            "negative or backward import time"
        );
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?)")
            .bind(owner.to_string())
            .fetch_one(&mut *conn)
            .await?;
        ensure!(exists, "missing native import owner");

        // Inspect identities before maintenance can erase raw evidence or replay records.
        let mut unseen = BTreeMap::new();
        let mut outcomes = BTreeMap::new();
        for entry in bundle {
            let attempt = &entry.attempt;
            attempt.validate()?;
            ensure!(attempt.thread_id == owner, "wrong native import owner");
            let dispatch = i64::from(attempt.dispatched_at_ms);
            let detail = dispatch
                .checked_add(DETAIL_MS)
                .context("detail expiry overflow")?;
            let expiry = dispatch
                .checked_add(REPLAY_MS)
                .context("replay expiry overflow")?;
            ensure!(
                detail <= as_of_ms && as_of_ms < expiry,
                "outside compact import interval"
            );
            day_key(owner.to_string(), dispatch / DAY_MS, as_of_ms)?;
            ensure!(
                read_attempt(conn, attempt.attempt_id).await?.is_none(),
                "raw import identity collision"
            );
            let seen: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM draft_accounting_tombstones WHERE attempt_id = ?)",
            )
            .bind(attempt.attempt_id.to_string())
            .fetch_one(&mut *conn)
            .await?;
            if seen {
                outcomes.insert(
                    attempt.attempt_id,
                    RetainedImportOutcome::SuppressedByReplayRecord,
                );
                continue;
            }
            let mut normalized = entry.clone();
            normalized
                .observations
                .sort_by_key(|o| i64::from(o.revision));
            for pair in normalized.observations.windows(2) {
                ensure!(
                    pair[0].revision != pair[1].revision || pair[0] == pair[1],
                    "conflicting import revision"
                );
            }
            normalized.observations.dedup();
            if let Some(previous) = unseen.insert(attempt.attempt_id, normalized.clone()) {
                ensure!(previous == normalized, "conflicting import identity");
            }
        }
        // Validate even a suppressed retry against store corruption, without writing or
        // requiring an ancestry graph that retention deliberately erased.
        validate_totals(conn, as_of_ms).await?;
        if unseen.is_empty() {
            return Ok(bundle
                .iter()
                .map(|e| outcomes[&e.attempt.attempt_id])
                .collect());
        }
        let mut requests = BTreeMap::new();
        let mut positions = BTreeSet::new();
        let mut additions: BTreeMap<DayKey, DayTotals> = BTreeMap::new();
        let mut prices = BTreeMap::new();
        let mut references: BTreeMap<DayKey, BTreeSet<Uuid>> = BTreeMap::new();
        for entry in unseen.values() {
            let attempt = &entry.attempt;
            if let Some(previous) = requests.insert(attempt.request_id, attempt) {
                ensure!(
                    attempt.same_owner(previous),
                    "import request owner collision"
                );
            }
            let raw: Vec<String> = sqlx::query_scalar(
                "SELECT payload FROM draft_accounting_attempts WHERE request_id = ?",
            )
            .bind(attempt.request_id.to_string())
            .fetch_all(&mut *conn)
            .await?;
            for payload in raw {
                ensure!(
                    attempt.same_owner(&serde_json::from_str(&payload)?),
                    "raw request owner collision"
                );
            }
            let mut visited = BTreeSet::from([attempt.attempt_id]);
            let mut predecessor = attempt.retry_of;
            while let Some(id) = predecessor {
                ensure!(visited.insert(id), "import retry cycle");
                if let Some(parent) = unseen.get(&id) {
                    ensure!(
                        attempt.same_owner(&parent.attempt),
                        "import retry owner mismatch"
                    );
                    predecessor = parent.attempt.retry_of;
                } else {
                    let parent = read_attempt(conn, id)
                        .await?
                        .context("missing or erased retry predecessor")?;
                    ensure!(attempt.same_owner(&parent), "raw retry owner mismatch");
                    break;
                }
            }
            for observation in &entry.observations {
                let position = (observation.source, i64::from(observation.sequence));
                ensure!(
                    positions.insert(position),
                    "import source position collision"
                );
                let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM draft_accounting_observations WHERE source = ? AND sequence = ?)")
                    .bind(position.0.to_string()).bind(position.1).fetch_one(&mut *conn).await?;
                ensure!(!exists, "raw source position collision");
            }
            let candidates = match &entry.original_price {
                OriginalPriceEvidence::Bound(snapshot) => std::slice::from_ref(snapshot),
                OriginalPriceEvidence::Unpriced => &[],
            };
            let quote = quote_observations(attempt, &entry.observations, candidates)?;
            if let OriginalPriceEvidence::Bound(snapshot) = &entry.original_price {
                ensure!(
                    quote.snapshot.as_ref() == Some(snapshot),
                    "ineligible original price evidence"
                );
                if let Some(stored) = read_snapshot(conn, &snapshot.id.to_string()).await? {
                    ensure!(stored == *snapshot, "original snapshot identity collision");
                }
                if let Some(previous) = prices.insert(snapshot.id, snapshot.clone()) {
                    ensure!(previous == *snapshot, "bundle snapshot identity collision");
                }
            }
            let dispatch = i64::from(attempt.dispatched_at_ms);
            let (key, expiry) = day_key(owner.to_string(), dispatch / DAY_MS, as_of_ms)?;
            let outcome = if expiry > as_of_ms {
                additions.entry(key.clone()).or_default().add(&quote)?;
                if let Some(snapshot) = &quote.snapshot {
                    references.entry(key).or_default().insert(snapshot.id);
                }
                RetainedImportOutcome::Imported
            } else {
                RetainedImportOutcome::ExpiredAggregateDay
            };
            outcomes.insert(attempt.attempt_id, outcome);
        }
        maintain_on_connection(conn, as_of_ms).await?;
        // Only surviving days retain price evidence; expired days get anonymous fences.
        for (id, snapshot) in prices {
            if !references.values().any(|ids| ids.contains(&id)) {
                continue;
            }
            sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?) ON CONFLICT(snapshot_id) DO NOTHING")
                .bind(id.to_string()).bind(serde_json::to_string(&snapshot)?).execute(&mut *conn).await?;
        }
        for ((thread, day), totals) in additions {
            let payload: Option<String> = sqlx::query_scalar("SELECT payload FROM draft_accounting_compact_days WHERE thread_id = ? AND utc_day = ?")
                .bind(&thread).bind(day).fetch_optional(&mut *conn).await?;
            let mut values = CompactValues::from_day_totals(&totals)?;
            if let Some(payload) = payload {
                values = values.checked_add(&CompactValues::decode(&payload)?)?;
            }
            sqlx::query("INSERT INTO draft_accounting_compact_days VALUES (?, ?, ?) ON CONFLICT(thread_id, utc_day) DO UPDATE SET payload = excluded.payload")
                .bind(&thread).bind(day).bind(values.encode()?).execute(&mut *conn).await?;
        }
        for ((thread, day), ids) in references {
            for id in ids {
                sqlx::query("INSERT INTO draft_accounting_compact_snapshots VALUES (?, ?, ?) ON CONFLICT(thread_id, utc_day, snapshot_id) DO NOTHING")
                    .bind(&thread).bind(day).bind(id.to_string()).execute(&mut *conn).await?;
            }
        }
        for entry in unseen.values() {
            let expiry = i64::from(entry.attempt.dispatched_at_ms)
                .checked_add(REPLAY_MS)
                .context("replay expiry overflow")?;
            sqlx::query("INSERT INTO draft_accounting_tombstones VALUES (?, ?)")
                .bind(entry.attempt.attempt_id.to_string())
                .bind(expiry)
                .execute(&mut *conn)
                .await?;
        }
        validate_totals(conn, as_of_ms).await?;
        Ok(bundle
            .iter()
            .map(|e| outcomes[&e.attempt.attempt_id])
            .collect())
    }
}

// Validate the whole result, including raw + compact sums even when a raw
// contribution is stale. Freshness must not mask arithmetic corruption.
async fn validate_totals(conn: &mut SqliteConnection, as_of_ms: i64) -> anyhow::Result<()> {
    let input = read_validated_input_on_connection(conn, as_of_ms).await?;
    let mut raw: BTreeMap<DayKey, DayTotals> = BTreeMap::new();
    for attempt in input.raw_attempts.values() {
        raw.entry(attempt.day.clone())
            .or_default()
            .add(&attempt.quote)?;
    }
    for (key, totals) in raw {
        let values = CompactValues::from_day_totals(&totals)?;
        if let Some(compact) = input.compact_days.get(&key) {
            values.checked_add(&compact.values)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "accounting_late_import_tests.rs"]
mod tests;
