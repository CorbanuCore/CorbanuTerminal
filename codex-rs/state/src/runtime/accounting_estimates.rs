//! Immutable synthetic estimates over retained journal evidence; no production DDL.
use super::super::Journal;
use super::super::StateRuntime;
use super::super::read_attempt;
use super::super::read_patches;
use super::*;
use sqlx::SqliteConnection;

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
    journal: Journal<'a>,
}

impl<'a> EstimateStore<'a> {
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
        let result = async {
            let (attempt, observations) = authority(&mut tx, id).await?;
            // Validate every candidate without calculating unused historical prices.
            quote_observations(&attempt, &[], candidates)?;
            for candidate in candidates {
                if let Some(stored) = read_snapshot(&mut tx, &candidate.id.to_string()).await? {
                    ensure!(stored == *candidate, "immutable snapshot conflict");
                }
            }
            let binding = binding(&mut tx, id).await?;
            let quote = match binding {
                Some(binding) => bound_quote(&mut tx, &attempt, &observations, binding).await?,
                None => quote_observations(&attempt, &observations, candidates)?,
            };
            if let Some(snapshot) = &quote.snapshot {
                sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?) ON CONFLICT(snapshot_id) DO NOTHING")
                    .bind(snapshot.id.to_string()).bind(serde_json::to_string(snapshot)?)
                    .execute(&mut *tx).await?;
            }
            sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, ?) ON CONFLICT(attempt_id) DO NOTHING")
                .bind(id.to_string()).bind(quote.snapshot.as_ref().map(|s| s.id.to_string()))
                .execute(&mut *tx).await?;
            let evidence = serde_json::to_string(&observations)?;
            let payload = serde_json::to_string(&quote)?;
            sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, ?, ?) ON CONFLICT(attempt_id, evidence) DO NOTHING")
                .bind(id.to_string()).bind(&evidence).bind(&payload)
                .execute(&mut *tx).await?;
            let stored: String = sqlx::query_scalar("SELECT payload FROM draft_accounting_estimates WHERE attempt_id = ? AND evidence = ?")
                .bind(id.to_string()).bind(evidence).fetch_one(&mut *tx).await?;
            ensure!(stored == payload, "immutable estimate conflict");
            anyhow::Ok(quote)
        }.await;
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
            let quote = bound_quote(conn, &attempt, &observations, binding).await?;
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

async fn bound_quote(
    conn: &mut SqliteConnection,
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
    let quote = quote_observations(attempt, observations, snapshot.as_slice())?;
    ensure!(quote.snapshot == snapshot, "ineligible bound snapshot");
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

#[path = "accounting_lifecycle.rs"]
mod lifecycle;
