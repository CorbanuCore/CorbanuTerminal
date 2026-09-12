//! PF-60-S02 draft journal. The parent module registers this only under cfg(test).
use super::StateRuntime;
use anyhow::ensure;
use sqlx::SqliteConnection;
use uuid::Uuid;
#[path = "accounting_pricing.rs"]
mod pricing;

#[path = "accounting_types.rs"]
mod types;
use types::Attempt;
use types::Observation;
use types::replay;

#[cfg(test)]
#[path = "accounting_tests.rs"]
mod tests;

struct Journal<'a> {
    runtime: &'a StateRuntime,
}

impl<'a> Journal<'a> {
    /// Explicit fixture setup only; never called by StateRuntime initialization.
    async fn create_for_tests(runtime: &'a StateRuntime) -> anyhow::Result<Self> {
        sqlx::raw_sql(
            "CREATE TABLE draft_accounting_attempts (
                attempt_id TEXT PRIMARY KEY, request_id TEXT NOT NULL, payload TEXT NOT NULL);
             CREATE INDEX draft_accounting_request ON draft_accounting_attempts(request_id);
             CREATE TABLE draft_accounting_observations (
                attempt_id TEXT NOT NULL REFERENCES draft_accounting_attempts(attempt_id),
                revision INTEGER NOT NULL CHECK(revision > 0),
                source TEXT NOT NULL, sequence INTEGER NOT NULL CHECK(sequence >= 0),
                payload TEXT NOT NULL, PRIMARY KEY(attempt_id, revision), UNIQUE(source, sequence));",
        )
        .execute(runtime.pool.as_ref())
        .await?;
        Ok(Self { runtime })
    }

    async fn begin_attempt(&self, attempt: &Attempt) -> anyhow::Result<()> {
        self.append_observation(attempt, &[]).await
    }

    /// All identity, patch and source-position writes either commit together or roll back.
    async fn append_observation(
        &self,
        attempt: &Attempt,
        batch: &[Observation],
    ) -> anyhow::Result<()> {
        attempt.validate()?;
        ensure!(batch.len() <= 256, "observation batch too large");
        let mut tx = self.runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        let result = async {
            let installed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'draft_accounting_tombstones')")
                .fetch_one(&mut *tx).await?;
            if installed {
                let deleted: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM draft_accounting_tombstones WHERE attempt_id = ?)")
                    .bind(attempt.attempt_id.to_string()).fetch_one(&mut *tx).await?;
                ensure!(!deleted, "deleted attempt");
            }
            let existing = sqlx::query_scalar::<_, String>(
                "SELECT payload FROM draft_accounting_attempts WHERE request_id = ?",
            )
            .bind(attempt.request_id.to_string())
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(existing) = existing {
                ensure!(attempt.same_owner(&serde_json::from_str(&existing)?), "request owner conflict");
            }
            if let Some(predecessor) = attempt.retry_of {
                let parent = read_attempt(&mut tx, predecessor).await?;
                ensure!(parent.as_ref().is_some_and(|p| attempt.same_owner(p)), "invalid retry predecessor");
            }
            sqlx::query("INSERT INTO draft_accounting_attempts VALUES (?, ?, ?) ON CONFLICT(attempt_id) DO NOTHING")
                .bind(attempt.attempt_id.to_string())
                .bind(attempt.request_id.to_string())
                .bind(serde_json::to_string(attempt)?)
                .execute(&mut *tx).await?;
            ensure!(read_attempt(&mut tx, attempt.attempt_id).await?.as_ref() == Some(attempt), "immutable identity conflict");
            for observation in batch {
                ensure!(i64::from(observation.revision) > 0, "revision must be positive");
                sqlx::query("INSERT INTO draft_accounting_observations VALUES (?, ?, ?, ?, ?) ON CONFLICT(attempt_id, revision) DO NOTHING")
                    .bind(attempt.attempt_id.to_string())
                    .bind(i64::from(observation.revision))
                    .bind(observation.source.to_string())
                    .bind(i64::from(observation.sequence))
                    .bind(serde_json::to_string(observation)?)
                    .execute(&mut *tx).await?;
                let stored: String = sqlx::query_scalar("SELECT payload FROM draft_accounting_observations WHERE attempt_id = ? AND revision = ?")
                    .bind(attempt.attempt_id.to_string())
                    .bind(i64::from(observation.revision))
                    .fetch_one(&mut *tx).await?;
                ensure!(serde_json::from_str::<Observation>(&stored)? == *observation, "observation conflict");
            }
            replay(attempt.dialect, &read_patches(&mut tx, attempt.attempt_id).await?)?;
            anyhow::Ok(())
        }.await;
        match result {
            Ok(()) => tx.commit().await?,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }
        Ok(())
    }

    async fn read_observations(
        &self,
        id: Uuid,
    ) -> anyhow::Result<Option<(Attempt, Vec<Observation>)>> {
        let mut tx = self.runtime.pool.begin().await?;
        let result = match read_attempt(&mut tx, id).await? {
            Some(attempt) => Some((attempt, read_patches(&mut tx, id).await?)),
            None => None,
        };
        tx.commit().await?;
        Ok(result)
    }
}

async fn read_attempt(conn: &mut SqliteConnection, id: Uuid) -> anyhow::Result<Option<Attempt>> {
    let payload: Option<String> =
        sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts WHERE attempt_id = ?")
            .bind(id.to_string())
            .fetch_optional(conn)
            .await?;
    Ok(payload.map(|p| serde_json::from_str(&p)).transpose()?)
}

async fn read_patches(conn: &mut SqliteConnection, id: Uuid) -> anyhow::Result<Vec<Observation>> {
    let payloads: Vec<String> = sqlx::query_scalar(
        "SELECT payload FROM draft_accounting_observations WHERE attempt_id = ? ORDER BY revision",
    )
    .bind(id.to_string())
    .fetch_all(conn)
    .await?;
    Ok(payloads
        .iter()
        .map(|p| serde_json::from_str(p))
        .collect::<Result<_, _>>()?)
}
