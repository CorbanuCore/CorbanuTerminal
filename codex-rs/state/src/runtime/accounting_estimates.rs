//! Immutable estimates over retained journal evidence.
use super::super::Journal;
#[cfg(test)]
use super::super::StateRuntime;
use super::super::read_attempt;
use super::super::read_patches;
use super::*;
use sqlx::SqliteConnection;
use std::collections::HashMap;

impl Serialize for Decimal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = Self::canonical(self.coefficient, self.scale);
        let divisor = 10_u128.pow(value.scale);
        let text = if value.scale == 0 {
            value.coefficient.to_string()
        } else {
            let whole = value.coefficient / divisor;
            let fraction = value.coefficient % divisor;
            let width = value.scale as usize;
            format!("{whole}.{fraction:0width$}")
        };
        serializer.serialize_str(&text)
    }
}

struct EstimateStore<'a> {
    #[cfg_attr(not(test), allow(dead_code))]
    journal: Journal<'a>,
}

impl<'a> EstimateStore<'a> {
    #[cfg(test)]
    async fn create_for_tests(runtime: &'a StateRuntime) -> anyhow::Result<Self> {
        let journal = Journal::create_for_tests(runtime).await?;
        sqlx::raw_sql(
            "CREATE TABLE draft_accounting_price_snapshots (
                snapshot_id TEXT PRIMARY KEY NOT NULL, payload TEXT NOT NULL);
             CREATE TABLE draft_accounting_price_bindings (
                attempt_id TEXT PRIMARY KEY NOT NULL
                    REFERENCES draft_accounting_attempts(attempt_id),
                snapshot_id TEXT REFERENCES draft_accounting_price_snapshots(snapshot_id));
             CREATE TABLE draft_accounting_estimates (
                attempt_id TEXT NOT NULL REFERENCES draft_accounting_price_bindings(attempt_id),
                evidence TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(attempt_id, evidence));",
        )
        .execute(runtime.pool.as_ref())
        .await?;
        Ok(Self { journal })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    async fn persist_current(
        &self,
        id: Uuid,
        candidates: &[Snapshot],
    ) -> anyhow::Result<ObservationQuote> {
        let mut tx = self
            .journal
            .runtime
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await?;
        let result = Journal::persist_price_on_connection(&mut tx, id, candidates).await;
        match result {
            Ok(quote) => {
                tx.commit().await?;
                Ok(quote)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    /// Requires a caller-owned transaction spanning all reads and any later transfer.
    async fn latest_quote_on_connection(
        conn: &mut SqliteConnection,
        id: Uuid,
    ) -> anyhow::Result<ObservationQuote> {
        // One authority read, one binding and one snapshot read serve every
        // version: they cannot change inside the caller's transaction, and
        // re-reading them per version made each quote cost several times the
        // validation it performs. Every version is still verified in full.
        let (attempt, observations) = authority(conn, id).await?;
        let binding = binding(conn, id).await?;
        let versions: Vec<(String, String)> = sqlx::query_as(
            "SELECT evidence, payload FROM draft_accounting_estimates WHERE attempt_id = ? ORDER BY evidence",
        )
        .bind(id.to_string())
        .fetch_all(&mut *conn)
        .await?;
        let current = serde_json::to_string(&observations)?;
        let mut snapshot = None;
        let mut recorded = None;
        // A stale or absent latest estimate must not conceal corrupt older evidence.
        for (evidence, payload) in versions {
            let bound = binding.as_ref().context("missing binding")?;
            let quote = verify_recorded(
                conn,
                &attempt,
                &observations,
                bound,
                &mut snapshot,
                &evidence,
                &payload,
            )
            .await?;
            if evidence == current {
                recorded = Some(quote);
            }
        }
        match (recorded, binding) {
            (Some(quote), _) => Ok(quote),
            (None, Some(binding)) => {
                let snapshot = bound_snapshot(conn, &binding, &mut snapshot).await?;
                quote_under_snapshot(PRICING_RULES, &attempt, &observations, snapshot)
            }
            (None, None) => quote_observations(&attempt, &observations, &[]),
        }
    }

    /// The quotes of `thread`'s attempts dispatched in `[start, end)`, for the
    /// post-write aggregate bound.
    ///
    /// Each is exactly what `latest_quote_on_connection` returns for an intact
    /// ledger: the recorded estimate's rules for the current evidence, otherwise
    /// today's rules, under the bound snapshot. It reads the day in four
    /// statements instead of re-verifying every attempt's stored history, which
    /// held the write lock for O(attempts that day) - 1.4 s per write on a busy
    /// thread. Explicit maintenance, the hourly validation and reads still verify
    /// that history in full.
    async fn day_quotes_on_connection(
        conn: &mut SqliteConnection,
        thread: &str,
        start: i64,
        end: i64,
    ) -> anyhow::Result<Vec<ObservationQuote>> {
        let attempts: Vec<(String, String)> = sqlx::query_as(
            "SELECT attempt_id, payload FROM draft_accounting_attempts
                WHERE json_extract(payload, '$.thread_id') = ?
                  AND json_extract(payload, '$.dispatched_at_ms') >= ?
                  AND json_extract(payload, '$.dispatched_at_ms') < ?
                ORDER BY attempt_id",
        )
        .bind(thread)
        .bind(start)
        .bind(end)
        .fetch_all(&mut *conn)
        .await?;
        // One scan finds the day; the rest are primary-key lookups.
        let ids = serde_json::to_string(
            &attempts
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>(),
        )?;
        let observations: Vec<(String, String)> = sqlx::query_as(
            "SELECT attempt_id, payload FROM draft_accounting_observations
                WHERE attempt_id IN (SELECT value FROM json_each(?))
                ORDER BY attempt_id, revision",
        )
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await?;
        let bindings: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT attempt_id, snapshot_id FROM draft_accounting_price_bindings
                WHERE attempt_id IN (SELECT value FROM json_each(?))",
        )
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await?;
        // The rules field alone, with its JSON type; see `recorded_rules`.
        let estimates: Vec<(String, String, Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT attempt_id, evidence, json_type(payload, '$.pricing_rules'),
                    CASE json_type(payload, '$.pricing_rules')
                        WHEN 'integer' THEN json_extract(payload, '$.pricing_rules') END
                FROM draft_accounting_estimates
                WHERE attempt_id IN (SELECT value FROM json_each(?))",
        )
        .bind(&ids)
        .fetch_all(&mut *conn)
        .await?;
        let mut patches: HashMap<String, Vec<Observation>> = HashMap::new();
        for (id, payload) in observations {
            patches
                .entry(id)
                .or_default()
                .push(serde_json::from_str(&payload)?);
        }
        let bindings: HashMap<String, Option<String>> = bindings.into_iter().collect();
        // As `recorded_rules`: absent means version 1, otherwise an integer above 1.
        let mut recorded: HashMap<String, HashMap<String, u16>> = HashMap::new();
        for (id, evidence, kind, value) in estimates {
            let rules = match (kind.as_deref(), value) {
                (None, _) => 1,
                (Some("integer"), Some(rules)) if rules > 1 => u16::try_from(rules)?,
                _ => anyhow::bail!("noncanonical pricing rules"),
            };
            recorded.entry(id).or_default().insert(evidence, rules);
        }
        let mut snapshots: HashMap<String, Snapshot> = HashMap::new();
        let mut quotes = Vec::with_capacity(attempts.len());
        for (id, payload) in attempts {
            let attempt: Attempt = serde_json::from_str(&payload)?;
            attempt.validate()?;
            ensure!(
                attempt.attempt_id.to_string() == id,
                "attempt identity mismatch"
            );
            let observations = patches.remove(&id).unwrap_or_default();
            let versions = recorded.remove(&id).unwrap_or_default();
            let quote = match bindings.get(&id) {
                None => {
                    ensure!(versions.is_empty(), "missing binding");
                    quote_observations(&attempt, &observations, &[])?
                }
                Some(binding) => {
                    let rules = versions
                        .get(&serde_json::to_string(&observations)?)
                        .copied()
                        .unwrap_or(PRICING_RULES);
                    let snapshot = match binding {
                        Some(snapshot_id) => {
                            if !snapshots.contains_key(snapshot_id) {
                                let snapshot = read_snapshot(conn, snapshot_id)
                                    .await?
                                    .context("missing snapshot")?;
                                snapshots.insert(snapshot_id.clone(), snapshot);
                            }
                            snapshots.get(snapshot_id)
                        }
                        None => None,
                    };
                    quote_under_snapshot(rules, &attempt, &observations, snapshot)?
                }
            };
            quotes.push(quote);
        }
        Ok(quotes)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    async fn read_estimate(
        &self,
        id: Uuid,
        evidence: &str,
    ) -> anyhow::Result<Option<ObservationQuote>> {
        let mut tx = self.journal.runtime.pool.begin().await?;
        let result = Self::read_on_connection(&mut tx, id, evidence).await?;
        tx.commit().await?;
        Ok(result)
    }

    async fn read_on_connection(
        conn: &mut SqliteConnection,
        id: Uuid,
        evidence: &str,
    ) -> anyhow::Result<Option<ObservationQuote>> {
        let payload: Option<String> = sqlx::query_scalar(
            "SELECT payload FROM draft_accounting_estimates WHERE attempt_id = ? AND evidence = ?",
        )
        .bind(id.to_string())
        .bind(evidence)
        .fetch_optional(&mut *conn)
        .await?;
        let result = if let Some(payload) = payload {
            let (attempt, retained) = authority(conn, id).await?;
            let observations: Vec<Observation> = serde_json::from_str(evidence)?;
            ensure!(
                serde_json::to_string(&observations)? == evidence,
                "noncanonical evidence"
            );
            for observation in &observations {
                let index = retained
                    .binary_search_by_key(&i64::from(observation.revision), |r| {
                        i64::from(r.revision)
                    })
                    .map_err(|_| anyhow::anyhow!("missing retained observation"))?;
                ensure!(
                    retained[index] == *observation,
                    "changed retained observation"
                );
            }
            let binding = binding(conn, id).await?.context("missing binding")?;
            let rules = recorded_rules(&payload)?;
            let quote = bound_quote(conn, rules, &attempt, &observations, binding).await?;
            // Do not deserialize quote decimals through the stricter rate parser.
            ensure!(
                serde_json::to_string(&quote)? == payload,
                "corrupt estimate payload"
            );
            Some(quote)
        } else {
            None
        };
        Ok(result)
    }
}

async fn binding(conn: &mut SqliteConnection, id: Uuid) -> anyhow::Result<Option<Option<String>>> {
    Ok(sqlx::query_scalar(
        "SELECT snapshot_id FROM draft_accounting_price_bindings WHERE attempt_id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(conn)
    .await?)
}

async fn read_snapshot(conn: &mut SqliteConnection, id: &str) -> anyhow::Result<Option<Snapshot>> {
    let payload: Option<String> = sqlx::query_scalar(
        "SELECT payload FROM draft_accounting_price_snapshots WHERE snapshot_id = ?",
    )
    .bind(id)
    .fetch_optional(conn)
    .await?;
    payload
        .map(|payload| {
            let snapshot: Snapshot = serde_json::from_str(&payload)?;
            snapshot.validate()?;
            ensure!(snapshot.id.to_string() == id, "snapshot identity mismatch");
            ensure!(
                serde_json::to_string(&snapshot)? == payload,
                "noncanonical snapshot"
            );
            Ok(snapshot)
        })
        .transpose()
}

/// The pricing rules a recorded estimate payload was computed under. Version 1
/// predates the field and is the only version that omits it.
fn recorded_rules(payload: &str) -> anyhow::Result<u16> {
    let value: serde_json::Value = serde_json::from_str(payload)?;
    match value.get("pricing_rules") {
        None => Ok(1),
        Some(rules) => {
            let rules = rules.as_u64().context("noncanonical pricing rules")?;
            ensure!(rules > 1, "noncanonical pricing rules");
            Ok(u16::try_from(rules)?)
        }
    }
}

/// The quote for exactly this evidence: the recorded estimate when there is
/// one, verified under the rules it was recorded with, otherwise a fresh quote
/// under the current rules. A recorded estimate is never re-priced by a build
/// whose rules changed since it was written.
async fn recorded_or_current(
    conn: &mut SqliteConnection,
    attempt: &Attempt,
    observations: &[Observation],
    binding: Option<String>,
) -> anyhow::Result<ObservationQuote> {
    let evidence = serde_json::to_string(observations)?;
    if let Some(recorded) =
        EstimateStore::read_on_connection(conn, attempt.attempt_id, &evidence).await?
    {
        return Ok(recorded);
    }
    bound_quote(conn, PRICING_RULES, attempt, observations, binding).await
}

async fn bound_quote(
    conn: &mut SqliteConnection,
    rules: u16,
    attempt: &Attempt,
    observations: &[Observation],
    binding: Option<String>,
) -> anyhow::Result<ObservationQuote> {
    let snapshot = match binding {
        Some(id) => Some(
            read_snapshot(conn, &id)
                .await?
                .context("missing snapshot")?,
        ),
        None => None,
    };
    quote_under_snapshot(rules, attempt, observations, snapshot.as_ref())
}

/// `read_on_connection`'s checks for one stored version, given the attempt's
/// authority and binding already read in the same transaction.
async fn verify_recorded(
    conn: &mut SqliteConnection,
    attempt: &Attempt,
    retained: &[Observation],
    binding: &Option<String>,
    snapshot: &mut Option<Option<Snapshot>>,
    evidence: &str,
    payload: &str,
) -> anyhow::Result<ObservationQuote> {
    let observations: Vec<Observation> = serde_json::from_str(evidence)?;
    ensure!(
        serde_json::to_string(&observations)? == evidence,
        "noncanonical evidence"
    );
    for observation in &observations {
        let index = retained
            .binary_search_by_key(&i64::from(observation.revision), |r| i64::from(r.revision))
            .map_err(|_| anyhow::anyhow!("missing retained observation"))?;
        ensure!(
            retained[index] == *observation,
            "changed retained observation"
        );
    }
    let rules = recorded_rules(payload)?;
    let snapshot = bound_snapshot(conn, binding, snapshot).await?;
    let quote = quote_under_snapshot(rules, attempt, &observations, snapshot)?;
    // Do not deserialize quote decimals through the stricter rate parser.
    ensure!(
        serde_json::to_string(&quote)? == payload,
        "corrupt estimate payload"
    );
    Ok(quote)
}

/// The snapshot a binding names, read and validated once per caller.
async fn bound_snapshot<'s>(
    conn: &mut SqliteConnection,
    binding: &Option<String>,
    cache: &'s mut Option<Option<Snapshot>>,
) -> anyhow::Result<Option<&'s Snapshot>> {
    if cache.is_none() {
        *cache = Some(match binding {
            Some(id) => Some(read_snapshot(conn, id).await?.context("missing snapshot")?),
            None => None,
        });
    }
    Ok(cache.as_ref().and_then(Option::as_ref))
}

fn quote_under_snapshot(
    rules: u16,
    attempt: &Attempt,
    observations: &[Observation],
    snapshot: Option<&Snapshot>,
) -> anyhow::Result<ObservationQuote> {
    let candidates: &[Snapshot] = match snapshot {
        Some(snapshot) => std::slice::from_ref(snapshot),
        None => &[],
    };
    let quote = quote_observations_under(rules, attempt, observations, candidates)?;
    ensure!(
        quote.snapshot.as_ref() == snapshot,
        "ineligible bound snapshot"
    );
    Ok(quote)
}

async fn authority(
    conn: &mut SqliteConnection,
    id: Uuid,
) -> anyhow::Result<(Attempt, Vec<Observation>)> {
    let attempt = read_attempt(conn, id).await?.context("missing attempt")?;
    attempt.validate()?;
    let request: String =
        sqlx::query_scalar("SELECT request_id FROM draft_accounting_attempts WHERE attempt_id = ?")
            .bind(id.to_string())
            .fetch_one(&mut *conn)
            .await?;
    ensure!(
        attempt.attempt_id == id && attempt.request_id.to_string() == request,
        "attempt identity mismatch"
    );
    let observations = read_patches(conn, id).await?;
    let positions: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT revision, source, sequence FROM draft_accounting_observations WHERE attempt_id = ? ORDER BY revision",
    )
    .bind(id.to_string()).fetch_all(conn).await?;
    let expected: Vec<_> = observations
        .iter()
        .map(|r| {
            (
                i64::from(r.revision),
                r.source.to_string(),
                i64::from(r.sequence),
            )
        })
        .collect();
    ensure!(positions == expected, "journal position mismatch");
    replay(attempt.dialect, &observations)?;
    Ok((attempt, observations))
}

#[cfg(test)]
#[path = "accounting_estimates_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "accounting_latest_quote_tests.rs"]
mod latest_tests;

#[path = "accounting_lifecycle.rs"]
mod lifecycle;
pub use lifecycle::Current;
pub use lifecycle::DayTotals;
pub use lifecycle::Metric;
pub use lifecycle::RetainedDay;
pub use lifecycle::RetentionCoverage;

impl Journal<'_> {
    pub(in crate::runtime::accounting) async fn inspect_quote_on_connection(
        conn: &mut SqliteConnection,
        id: Uuid,
    ) -> anyhow::Result<ObservationQuote> {
        ensure!(
            binding(conn, id).await?.is_some(),
            "missing original binding"
        );
        EstimateStore::latest_quote_on_connection(conn, id).await
    }

    pub(in crate::runtime::accounting) async fn persist_price_on_connection(
        conn: &mut SqliteConnection,
        id: Uuid,
        candidates: &[Snapshot],
    ) -> anyhow::Result<ObservationQuote> {
        let (attempt, observations) = authority(conn, id).await?;
        // Validate every candidate without calculating unused historical prices.
        quote_observations(&attempt, &[], candidates)?;
        for candidate in candidates {
            if let Some(stored) = read_snapshot(conn, &candidate.id.to_string()).await? {
                ensure!(stored == *candidate, "immutable snapshot conflict");
            }
        }
        let binding = binding(conn, id).await?;
        if binding.is_some() {
            let evidence = serde_json::to_string(&observations)?;
            // Already recorded: that estimate stands, verified under its own
            // rules. Recomputing it here under today's rules would turn any
            // pricing change into an "immutable estimate conflict".
            if let Some(recorded) = EstimateStore::read_on_connection(conn, id, &evidence).await? {
                return Ok(recorded);
            }
        }
        let quote = match binding {
            Some(binding) => {
                bound_quote(conn, PRICING_RULES, &attempt, &observations, binding).await?
            }
            None => quote_observations(&attempt, &observations, candidates)?,
        };
        if let Some(snapshot) = &quote.snapshot {
            sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?) ON CONFLICT(snapshot_id) DO NOTHING")
                .bind(snapshot.id.to_string()).bind(serde_json::to_string(snapshot)?)
                .execute(&mut *conn).await?;
        }
        sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, ?) ON CONFLICT(attempt_id) DO NOTHING")
            .bind(id.to_string()).bind(quote.snapshot.as_ref().map(|s| s.id.to_string()))
            .execute(&mut *conn).await?;
        let evidence = serde_json::to_string(&observations)?;
        let payload = serde_json::to_string(&quote)?;
        sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, ?, ?) ON CONFLICT(attempt_id, evidence) DO NOTHING")
            .bind(id.to_string()).bind(&evidence).bind(&payload)
            .execute(&mut *conn).await?;
        let stored: String = sqlx::query_scalar(
            "SELECT payload FROM draft_accounting_estimates WHERE attempt_id = ? AND evidence = ?",
        )
        .bind(id.to_string())
        .bind(evidence)
        .fetch_one(&mut *conn)
        .await?;
        ensure!(stored == payload, "immutable estimate conflict");
        anyhow::Ok(quote)
    }
}
