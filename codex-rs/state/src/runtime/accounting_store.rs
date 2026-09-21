//! Explicit, versioned accounting storage. No collection or user-facing activation switch.
use super::Journal;
use super::StateRuntime;
use anyhow::Context;
use anyhow::ensure;
use codex_protocol::ThreadId;
use sqlx::SqliteConnection;

pub use super::pricing::Basis;
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
    /// Unresolved owners whose day could not be inspected; no amount or coverage inferred.
    pub unknown_parent_unavailable_threads: usize,
    pub unknown_parent_requests: std::collections::BTreeMap<uuid::Uuid, Vec<ObservationQuote>>,
    pub requests: std::collections::BTreeMap<uuid::Uuid, Vec<ObservationQuote>>,
}

/// UTC half-open requested interval and calendar-aligned grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectionRange {
    pub start_ms: i64,
    pub end_ms: i64,
    pub grouping: InspectionGrouping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionGrouping {
    Hour,
    Day,
    Week,
    Month,
}

impl InspectionRange {
    pub fn validate(self) -> anyhow::Result<()> {
        ensure!(
            self.start_ms >= 0,
            "invalid range: start precedes Unix epoch"
        );
        ensure!(
            self.end_ms != self.start_ms,
            "empty range: start equals end"
        );
        ensure!(
            self.end_ms > self.start_ms,
            "reversed range: end precedes start"
        );
        ensure!(
            chrono::DateTime::from_timestamp_millis(self.end_ms).is_some(),
            "invalid range: timestamp overflow"
        );
        Ok(())
    }

    /// Full aligned bounds; callers must not total a partially covered bucket.
    pub fn bucket(self, time: i64) -> anyhow::Result<(i64, i64)> {
        use chrono::Datelike;
        let date = chrono::DateTime::from_timestamp_millis(time)
            .context("invalid timestamp")?
            .date_naive();
        let (start, end) = match self.grouping {
            InspectionGrouping::Hour => {
                return Ok((
                    time.div_euclid(3_600_000) * 3_600_000,
                    (time.div_euclid(3_600_000) + 1) * 3_600_000,
                ));
            }
            InspectionGrouping::Day => (date, date.succ_opt().context("day overflow")?),
            InspectionGrouping::Week => {
                let start =
                    date - chrono::Duration::days(i64::from(date.weekday().num_days_from_monday()));
                (
                    start,
                    start
                        .checked_add_signed(chrono::Duration::days(7))
                        .context("week overflow")?,
                )
            }
            InspectionGrouping::Month => {
                let start = date.with_day(1).context("month start")?;
                (
                    start,
                    start
                        .checked_add_months(chrono::Months::new(1))
                        .context("month overflow")?,
                )
            }
        };
        Ok((
            start
                .and_hms_opt(0, 0, 0)
                .context("midnight")?
                .and_utc()
                .timestamp_millis(),
            end.and_hms_opt(0, 0, 0)
                .context("midnight")?
                .and_utc()
                .timestamp_millis(),
        ))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InspectionBucket {
    pub start_ms: i64,
    pub end_ms: i64,
    pub effective: Option<(i64, i64)>,
    pub partial: bool,
    pub days: Vec<InspectionDay>,
}

/// Unavailability never carries a partial amount.
#[derive(Debug, PartialEq, Eq)]
pub enum InspectionDay {
    Range {
        requested: InspectionRange,
        oldest_aggregate_day: Option<i64>,
        read_at_ms: i64,
        buckets: Vec<InspectionBucket>,
    },
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
        Self::inspect(runtime, owner, utc_day, read_at_ms, None).await
    }

    pub async fn inspect_range(
        runtime: &StateRuntime,
        owner: ThreadId,
        requested: InspectionRange,
        read_at_ms: i64,
    ) -> anyhow::Result<InspectionDay> {
        requested.validate()?;
        Self::inspect(
            runtime,
            owner,
            requested.start_ms / 86_400_000,
            read_at_ms,
            Some(requested),
        )
        .await
    }

    async fn inspect(
        runtime: &StateRuntime,
        owner: ThreadId,
        utc_day: i64,
        read_at_ms: i64,
        requested: Option<InspectionRange>,
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
        let result = match requested {
            Some(requested) => inspect_buckets(&mut tx, owner, requested, read_at_ms).await?,
            None => inspect_tree(&mut tx, owner, utc_day, read_at_ms).await?,
        };
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

// Work, unlike selected output, is never refunded for an unrelated candidate.
// Reserve table-scan row visits before SQL, and bound all candidate/chain walks.
// One budget covers the entire range, not each bucket independently.
pub(super) struct InspectionWork {
    rows: usize,
    visits: usize,
    scan_rows: usize,
}

impl InspectionWork {
    pub(super) async fn new(conn: &mut SqliteConnection) -> anyhow::Result<Self> {
        let scanned: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM (
             SELECT 1 FROM draft_accounting_attempts
             UNION ALL SELECT 1 FROM draft_accounting_compact_days LIMIT 4000001)",
        )
        .fetch_one(&mut *conn)
        .await?;
        Ok(Self {
            rows: 4_000_000usize.saturating_sub(scanned as usize),
            visits: 20_000,
            scan_rows: scanned as usize,
        })
    }

    pub(super) fn scans(&mut self, count: usize) -> bool {
        let cost = self.scan_rows.saturating_mul(count);
        if cost > self.rows {
            self.rows = 0;
            return false;
        }
        self.rows -= cost;
        true
    }

    pub(super) fn visit(&mut self) -> bool {
        if self.visits == 0 || self.rows == 0 {
            self.rows = 0;
            return false;
        }
        self.visits -= 1;
        self.rows -= 1;
        true
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
    let mut work = InspectionWork::new(conn).await?;
    inspect_tree_window(conn, owner, day, read_at_ms, None, &mut work).await
}

async fn inspect_tree_window(
    conn: &mut SqliteConnection,
    owner: ThreadId,
    day: i64,
    read_at_ms: i64,
    window: Option<(i64, i64)>,
    work: &mut InspectionWork,
) -> anyhow::Result<InspectionDay> {
    use codex_protocol::protocol::SessionSource;
    use std::collections::BTreeMap;
    use std::collections::HashSet;
    let first =
        Journal::inspect_window_on_connection(conn, owner, day, read_at_ms, window, work).await?;
    let InspectionDay::Ready(mut view) = first else {
        return Ok(first);
    };
    if !work.scans(1) {
        return Ok(InspectionDay::TooLarge);
    }
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
        if !work.visit() {
            return Ok(InspectionDay::TooLarge);
        }
        if candidate == owner.to_string() {
            continue;
        }
        let mut cursor = candidate.clone();
        let mut seen = HashSet::new();
        let mut reached = false;
        let mut discovered = BTreeMap::new();
        let relation = loop {
            if !work.visit() {
                return Ok(InspectionDay::TooLarge);
            }
            if !seen.insert(cursor.clone()) {
                break None;
            }
            reached |= cursor == owner.to_string();
            if !ancestry.contains_key(&cursor) && !discovered.contains_key(&cursor) {
                let row: Option<(String, Option<String>)> = sqlx::query_as(
                    "SELECT t.source, e.parent_thread_id FROM threads t
                     LEFT JOIN thread_spawn_edges e ON e.child_thread_id = t.id WHERE t.id = ?",
                )
                .bind(&cursor)
                .fetch_optional(&mut *conn)
                .await?;
                let (parent, terminal) = if let Some((source, edge)) = row {
                    let parsed = serde_json::from_str::<SessionSource>(&source)
                        .or_else(|_| serde_json::from_value(serde_json::Value::String(source)))
                        .ok();
                    let source_parent = parsed
                        .as_ref()
                        .and_then(SessionSource::parent_thread_id)
                        .map(|id| id.to_string());
                    if matches!(parsed.as_ref(), None | Some(SessionSource::Unknown))
                        || (source_parent.is_some() && edge != source_parent)
                    {
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
                discovered.insert(cursor.clone(), (parent, terminal));
                if ancestry.len() + discovered.len() > 10_000 {
                    return Ok(InspectionDay::TooLarge);
                }
            }
            let (parent, terminal) = ancestry
                .get(&cursor)
                .unwrap_or_else(|| &discovered[&cursor]);
            match parent {
                Some(parent) => cursor = parent.clone(),
                None => {
                    break terminal.then_some(reached);
                }
            }
        };
        if relation == Some(false) {
            // Unrelated roots cannot consume the selected ancestry/packet budget.
            continue;
        }
        // Source text is parsed and dropped; only retained ancestry strings count.
        // Provisional memory is bounded above by the node limit before rollback.
        graph_bytes += discovered
            .iter()
            .map(|(id, (parent, _))| id.len() + parent.as_ref().map_or(0, String::len))
            .sum::<usize>();
        if packet_bytes + graph_bytes > 4 * 1024 * 1024 {
            return Ok(InspectionDay::TooLarge);
        }
        ancestry.extend(discovered);
        let other = Journal::inspect_window_on_connection(
            conn,
            ThreadId::from_string(&candidate)?,
            day,
            read_at_ms,
            window,
            work,
        )
        .await?;
        if work.rows == 0 {
            return Ok(InspectionDay::TooLarge);
        }
        let InspectionDay::Ready(other) = other else {
            if relation.is_none() {
                view.unknown_parent_unavailable_threads += 1;
                continue;
            }
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

// All buckets reuse the already-open deferred snapshot, including ancestry.
async fn inspect_buckets(
    conn: &mut SqliteConnection,
    owner: ThreadId,
    requested: InspectionRange,
    read_at_ms: i64,
) -> anyhow::Result<InspectionDay> {
    let mut work = InspectionWork::new(conn).await?;
    if !work.scans(1) {
        return Ok(InspectionDay::TooLarge);
    }
    let oldest_aggregate_day =
        sqlx::query_scalar("SELECT min(utc_day) FROM draft_accounting_compact_days")
            .fetch_one(&mut *conn)
            .await?;
    let mut buckets = Vec::new();
    let mut cursor = requested.start_ms;
    let mut attempts = 0;
    let mut bytes = 0;
    while cursor < requested.end_ms {
        if buckets.len() >= 10_000 {
            return Ok(InspectionDay::TooLarge);
        }
        let (start_ms, end_ms) = requested.bucket(cursor)?;
        let lower = start_ms.max(requested.start_ms);
        let upper = end_ms.min(requested.end_ms);
        let mut effective = Some((lower, upper));
        let mut days = Vec::new();
        let mut day = lower / 86_400_000;
        while day <= (upper - 1) / 86_400_000 {
            let window = (
                lower.max(day * 86_400_000),
                upper.min((day + 1) * 86_400_000),
            );
            let value =
                inspect_tree_window(conn, owner, day, read_at_ms, Some(window), &mut work).await?;
            let coverage = match &value {
                InspectionDay::Ready(view) => {
                    attempts += view.totals.attempts + view.unknown_parent_totals.attempts;
                    for quote in view
                        .requests
                        .values()
                        .chain(view.unknown_parent_requests.values())
                        .flatten()
                    {
                        bytes += serde_json::to_vec(quote)?.len() + 2048;
                    }
                    Some(&view.coverage)
                }
                InspectionDay::DetailUnavailable { coverage, .. } => Some(coverage),
                InspectionDay::TooLarge => return Ok(value),
                _ => None,
            };
            if let (Some((from, to)), Some(coverage)) = (effective, coverage) {
                let from = from.max(coverage.aggregate_day_floor * 86_400_000);
                let to = to
                    .min(coverage.completed_as_of_ms.saturating_add(1))
                    .min(read_at_ms.saturating_add(1));
                effective = (from < to).then_some((from, to));
            } else {
                effective = None;
            }
            // Include empty-bucket/header costs in the unchanged packet ceiling.
            bytes += 8192;
            if attempts > 512 || bytes > 4 * 1024 * 1024 {
                return Ok(InspectionDay::TooLarge);
            }
            days.push(value);
            day += 1;
        }
        buckets.push(InspectionBucket {
            start_ms,
            end_ms,
            effective,
            partial: effective != Some((start_ms, end_ms)),
            days,
        });
        cursor = end_ms;
    }
    Ok(InspectionDay::Range {
        requested,
        oldest_aggregate_day,
        read_at_ms,
        buckets,
    })
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
