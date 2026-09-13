//! Explicit, versioned accounting storage. No collection or user-facing activation switch.
use super::Journal;
use super::StateRuntime;
use anyhow::Context;
use anyhow::ensure;
use codex_protocol::ThreadId;
use sqlx::SqliteConnection;

pub use super::pricing::BucketQuote;
pub use super::pricing::Currency;
pub use super::pricing::Current;
pub use super::pricing::DayTotals;
pub use super::pricing::Decimal;
pub use super::pricing::DisplayAmount;
pub use super::pricing::Metric;
pub use super::pricing::ObservationQuote;
pub use super::pricing::Rates;
pub use super::pricing::RetainedDay;
pub use super::pricing::RetentionCoverage;
pub use super::pricing::Snapshot;
pub use super::pricing::SourceKind;
pub use super::pricing::Unit;
pub use super::types::Attempt;
pub use super::types::Count;
pub use super::types::Dialect;
pub use super::types::Observation;
pub use super::types::Patch;
pub use super::types::Presence;
pub use super::types::Usage;

/// Original dispatch-time evidence, never a catalog to select a replacement from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OriginalPriceEvidence {
    Bound(Snapshot),
    Unpriced,
}

/// A complete immutable original-evidence snapshot, not an incremental event feed.
/// Completeness is the caller's contract; interrupted attempts may contain unknowns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedImport {
    pub attempt: Attempt,
    pub observations: Vec<Observation>,
    pub original_price: OriginalPriceEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetainedImportOutcome {
    Imported,
    /// Only UUID/expiry survives; this does not certify payload equality.
    SuppressedByReplayRecord,
    /// Conservative daily expiry discards totals, but retains the replay fence.
    ExpiredAggregateDay,
}

/// Opt-in handle over the native state database. Dropping it stops no background
/// task: there is no collector. Native deletion still cleans installed accounting.
pub struct AccountingStore<'a> {
    runtime: &'a StateRuntime,
}

impl<'a> AccountingStore<'a> {
    /// Atomically import 1..=64 complete attempts aged 90..365 days without raw
    /// staging. Each has at most 256 observations, with at most 4096 in total.
    /// Duplicate identical entries receive the same outcome but contribute once.
    /// An all-suppressed bundle does not advance maintenance or read coverage.
    /// Cancellation may lose a committed acknowledgement: retry the original IDs.
    pub async fn import_retained(
        &self,
        owner: ThreadId,
        bundle: &[RetainedImport],
        as_of_ms: i64,
    ) -> anyhow::Result<Vec<RetainedImportOutcome>> {
        let mut tx = self.runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        match Journal::import_retained_on_connection(&mut tx, owner, bundle, as_of_ms).await {
            Ok(outcomes) => {
                tx.commit().await?;
                Ok(outcomes)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    /// Install only when wholly absent, otherwise validate without schema repair.
    /// Activation is a real complete retention sweep, never a fabricated checkpoint.
    pub async fn open(runtime: &'a StateRuntime, as_of_ms: i64) -> anyhow::Result<Self> {
        ensure!(as_of_ms >= 0, "negative accounting time");
        let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        match install_on_connection(&mut tx).await {
            Ok(()) => tx.commit().await?,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }
        let store = Self { runtime };
        store.maintain(as_of_ms).await?;
        Ok(store)
    }

    /// Persist native intent and original price (including explicit missing price)
    /// before the caller sends. This does not prove actual dispatch or approval.
    pub async fn admit(
        &self,
        owner: ThreadId,
        attempt: &Attempt,
        original_prices: &[Snapshot],
        as_of_ms: i64,
    ) -> anyhow::Result<ObservationQuote> {
        self.write(owner, attempt, &[], Some(original_prices), as_of_ms)
            .await
    }

    /// Append cumulative evidence only to an already durably admitted attempt.
    /// Original NULL/rate bindings cannot be replaced by later catalogs.
    pub async fn observe(
        &self,
        owner: ThreadId,
        attempt: &Attempt,
        observations: &[Observation],
        as_of_ms: i64,
    ) -> anyhow::Result<ObservationQuote> {
        self.write(owner, attempt, observations, None, as_of_ms)
            .await
    }

    async fn write(
        &self,
        owner: ThreadId,
        attempt: &Attempt,
        observations: &[Observation],
        original_prices: Option<&[Snapshot]>,
        as_of_ms: i64,
    ) -> anyhow::Result<ObservationQuote> {
        let mut tx = self.runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        let result = Journal::store_on_connection(
            &mut tx,
            owner,
            attempt,
            observations,
            original_prices,
            as_of_ms,
        )
        .await;
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

    pub async fn maintain(&self, as_of_ms: i64) -> anyhow::Result<()> {
        let mut tx = self.runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        let result = async {
            validate_on_connection(&mut tx).await?;
            Journal::maintain_native_on_connection(&mut tx, as_of_ms).await
        }
        .await;
        match result {
            Ok(()) => tx.commit().await?,
            Err(error) => {
                tx.rollback().await?;
                return Err(error);
            }
        }
        Ok(())
    }

    /// A single read snapshot, with explicit stale/expired/unknown states.
    /// Reading never activates or advances maintenance.
    pub async fn read_day(
        &self,
        owner: ThreadId,
        utc_day: i64,
        as_of_ms: i64,
    ) -> anyhow::Result<RetainedDay> {
        let mut tx = self.runtime.pool.begin().await?;
        validate_on_connection(&mut tx).await?;
        let value = Journal::retained_day_on_connection(&mut tx, owner, utc_day, as_of_ms).await?;
        tx.commit().await?;
        Ok(value)
    }
}

pub(super) async fn ledger_exists(conn: &mut SqliteConnection) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name = '_accounting_migrations')",
    )
    .fetch_one(conn)
    .await?)
}

// Caller owns BEGIN IMMEDIATE: schema and independent ledger commit together.
async fn install_on_connection(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    if !ledger_exists(conn).await? {
        let objects: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM sqlite_schema WHERE name GLOB 'draft_accounting_*'",
        )
        .fetch_one(&mut *conn)
        .await?;
        ensure!(objects == 0, "unversioned accounting schema");
        crate::migrations::accounting_migrator()
            .run_direct(/*target*/ None, conn, /*skip*/ false)
            .await?;
    }
    validate_on_connection(conn).await
}

/// Read-only version and physical-schema validation; no migration adoption/repair.
pub(super) async fn validate_on_connection(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    ensure!(ledger_exists(conn).await?, "accounting ledger missing");
    let migrator = crate::migrations::accounting_migrator();
    let rows: Vec<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT version, success, checksum FROM _accounting_migrations ORDER BY version",
    )
    .fetch_all(&mut *conn)
    .await?;
    let expected: Vec<_> = migrator
        .iter()
        .map(|m| (m.version, true, m.checksum.to_vec()))
        .collect();
    ensure!(
        rows == expected,
        "unsupported, failed or checksum-mismatched accounting migration"
    );
    // Compare every owned table/index definition, not just a ledger claim.
    // SQLite preserves CREATE SQL; whitespace is not schema identity.
    for migration in migrator.iter() {
        for statement in migration
            .sql
            .as_str()
            .split(';')
            .map(str::trim)
            .filter(|sql| sql.starts_with("CREATE "))
        {
            let name = statement
                .split_whitespace()
                .nth(2)
                .context("embedded accounting CREATE statement missing object name")?;
            let actual: Option<String> =
                sqlx::query_scalar("SELECT sql FROM sqlite_schema WHERE name = ?")
                    .bind(name)
                    .fetch_optional(&mut *conn)
                    .await?;
            ensure!(
                actual
                    .as_deref()
                    .map(|sql| sql.split_whitespace().collect::<Vec<_>>())
                    == Some(statement.split_whitespace().collect()),
                "partial or changed accounting schema: {name}"
            );
        }
    }
    super::retention_fixture_on_connection(conn).await?;
    Ok(())
}

#[cfg(test)]
#[path = "accounting_store_tests.rs"]
mod tests;
