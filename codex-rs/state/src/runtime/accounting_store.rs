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

/// One UTC day's immutable, recorded-only explanation. No collection coverage claim.
#[derive(Debug, PartialEq, Eq)]
pub struct Inspection {
    pub owner: ThreadId,
    pub utc_day: i64,
    pub read_at_ms: i64,
    pub coverage: RetentionCoverage,
    /// Resolved root plus descendants; unresolved ancestry is excluded.
    pub totals: DayTotals,
    pub own_totals: DayTotals,
    pub descendant_totals: DayTotals,
    pub unknown_parent_totals: DayTotals,
    pub unknown_parent_requests: std::collections::BTreeMap<uuid::Uuid, Vec<ObservationQuote>>,
    pub requests: std::collections::BTreeMap<uuid::Uuid, Vec<ObservationQuote>>,
}

/// Unavailability never carries a partial amount.
#[derive(Debug, PartialEq, Eq)]
pub enum InspectionDay {
    Absent,
    MissingThread,
    /// The requested view is beyond, or has no completed, maintenance checkpoint.
    CheckpointLag,
    NeedsRefresh,
    TooLarge,
    DetailUnavailable {
        coverage: RetentionCoverage,
        read_at_ms: i64,
        compact: bool,
    },
    Ready(Inspection),
}

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
    /// Inspect an existing ledger in one read transaction, without installation,
    /// maintenance, repricing or repair. All times are UTC milliseconds/days.
    pub async fn inspect_day(
        runtime: &StateRuntime,
        owner: ThreadId,
        utc_day: i64,
        read_at_ms: i64,
    ) -> anyhow::Result<InspectionDay> {
        let start = utc_day.checked_mul(86_400_000).context("day overflow")?;
        ensure!(utc_day >= 0 && read_at_ms >= start, "invalid or future day");
        start.checked_add(86_400_000).context("day end overflow")?;
        let mut tx = runtime.pool.begin().await?;
        if !ledger_exists(&mut tx).await? {
            let objects: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_schema WHERE name GLOB 'draft_accounting_*'",
            )
            .fetch_one(&mut *tx)
            .await?;
            ensure!(objects == 0, "unversioned accounting schema");
            return Ok(InspectionDay::Absent);
        }
        validate_on_connection(&mut tx).await?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?)")
            .bind(owner.to_string())
            .fetch_one(&mut *tx)
            .await?;
        if !exists {
            return Ok(InspectionDay::MissingThread);
        }
        let result = inspect_tree(&mut tx, owner, utc_day, read_at_ms).await?;
        tx.commit().await?;
        Ok(result)
    }

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

// Reuse the single-owner validator inside the caller's deferred snapshot. Never
// open a store or refresh contributions while assembling the tree explanation.
async fn inspect_tree(
    conn: &mut SqliteConnection,
    owner: ThreadId,
    day: i64,
    read_at_ms: i64,
) -> anyhow::Result<InspectionDay> {
    use codex_protocol::protocol::SessionSource;
    use std::collections::BTreeMap;
    use std::collections::HashSet;
    let first = Journal::inspect_on_connection(conn, owner, day, read_at_ms).await?;
    let InspectionDay::Ready(mut view) = first else {
        return Ok(first);
    };
    let candidates: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT json_extract(payload, '$.thread_id') FROM draft_accounting_attempts
         WHERE json_extract(payload, '$.dispatched_at_ms') / 86400000 = ?
         UNION SELECT thread_id FROM draft_accounting_compact_days WHERE utc_day = ?",
    )
    .bind(day)
    .bind(day)
    .fetch_all(&mut *conn)
    .await?;
    // Cache only ancestry visited by selected-day owners. A malformed source,
    // absent ancestor, conflict or cycle is an unknown population, never another root.
    let mut ancestry: BTreeMap<String, (Option<String>, bool)> = BTreeMap::new();
    let mut graph_bytes = 0usize;
    let mut packet_bytes = 8192usize;
    let mut attempts = view.totals.attempts;
    for quotes in view.requests.values() {
        for quote in quotes {
            packet_bytes += serde_json::to_vec(quote)?.len() + 2048;
        }
    }
    for candidate in candidates {
        if candidate == owner.to_string() {
            continue;
        }
        let mut cursor = candidate.clone();
        let mut seen = HashSet::new();
        let mut reached = false;
        let relation = loop {
            if !seen.insert(cursor.clone()) {
                break None;
            }
            reached |= cursor == owner.to_string();
            if !ancestry.contains_key(&cursor) {
                let row: Option<(String, Option<String>)> = sqlx::query_as(
                    "SELECT t.source, e.parent_thread_id FROM threads t
                     LEFT JOIN thread_spawn_edges e ON e.child_thread_id = t.id WHERE t.id = ?",
                )
                .bind(&cursor)
                .fetch_optional(&mut *conn)
                .await?;
                let (parent, terminal) = if let Some((source, edge)) = row {
                    graph_bytes +=
                        source.len() + edge.as_ref().map_or(0, String::len) + cursor.len();
                    let parsed = serde_json::from_str::<SessionSource>(&source)
                        .or_else(|_| serde_json::from_value(serde_json::Value::String(source)))
                        .ok();
                    let source_parent = parsed
                        .as_ref()
                        .and_then(SessionSource::parent_thread_id)
                        .map(|id| id.to_string());
                    if source_parent.is_some() && edge != source_parent {
                        (None, false)
                    } else {
                        let terminal = matches!(
                            parsed,
                            Some(
                                SessionSource::Cli
                                    | SessionSource::VSCode
                                    | SessionSource::Exec
                                    | SessionSource::Mcp
                            )
                        );
                        (edge, terminal)
                    }
                } else {
                    (None, false)
                };
                ancestry.insert(cursor.clone(), (parent, terminal));
                if ancestry.len() > 10_000 || packet_bytes + graph_bytes > 4 * 1024 * 1024 {
                    return Ok(InspectionDay::TooLarge);
                }
            }
            let (parent, terminal) = &ancestry[&cursor];
            match parent {
                Some(parent) => cursor = parent.clone(),
                None => {
                    break terminal.then_some(reached);
                }
            }
        };
        if relation == Some(false) {
            continue;
        }
        let other = Journal::inspect_on_connection(
            conn,
            ThreadId::from_string(&candidate)?,
            day,
            read_at_ms,
        )
        .await?;
        let InspectionDay::Ready(other) = other else {
            return Ok(other);
        };
        attempts = attempts
            .checked_add(other.totals.attempts)
            .context("attempt count overflow")?;
        if attempts > 512 {
            return Ok(InspectionDay::TooLarge);
        }
        let target = if relation == Some(true) {
            &mut view.requests
        } else {
            &mut view.unknown_parent_requests
        };
        for (request, quotes) in other.requests {
            for quote in &quotes {
                packet_bytes += serde_json::to_vec(quote)?.len() + 2048;
            }
            if packet_bytes + graph_bytes > 4 * 1024 * 1024 {
                return Ok(InspectionDay::TooLarge);
            }
            ensure!(
                target.insert(request, quotes).is_none(),
                "request owner mismatch"
            );
        }
    }
    view.totals = DayTotals::from_quotes(view.requests.values().flatten())?;
    view.descendant_totals = DayTotals::from_quotes(
        view.requests
            .values()
            .flatten()
            .filter(|q| q.attempt.thread_id != owner),
    )?;
    view.unknown_parent_totals =
        DayTotals::from_quotes(view.unknown_parent_requests.values().flatten())?;
    Ok(InspectionDay::Ready(view))
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
