//! Lookup indexes on the ledger's attempts that its migrations do not create.
//!
//! Every ledger write reads the owner's day of attempts (`check_day`) and asks
//! whether anything is due to expire. Both select attempts by fields inside
//! their JSON payload, so without these indexes each write scanned and parsed
//! every attempt in the ledger while holding the state DB's write lock: on a
//! 61,000-attempt ledger that held the lock for 0.3-2 s per write, and every
//! other session waited. With them, the same statements are index lookups.
//!
//! They are created here, idempotently, rather than by a ledger migration: a
//! migration raises the ledger's format, and a build that knows only the older
//! format would then refuse the ledger (no collection, and no conversation
//! deletion). An index changes no record and no result, and builds that do not
//! know these indexes neither use nor mind them. A ledger migration that
//! rebuilds `draft_accounting_attempts` drops them with it; they are created
//! again on the next open. Their names are outside the ledger's
//! `draft_accounting_*` namespace, so a future ledger migration cannot collide
//! with them. An expression index evaluates `json_extract` on every inserted
//! payload; payloads are always serialized JSON.
use crate::StateRuntime;
use std::time::Instant;

const LOG_TARGET: &str = "codex_core::accounting";

/// Name and definition. The expressions match the ones the ledger's queries
/// use exactly; SQLite only uses an expression index for an identical one.
const INDEXES: [(&str, &str); 2] = [
    (
        "corbanu_lookup_attempt_owner_dispatch",
        "CREATE INDEX IF NOT EXISTS corbanu_lookup_attempt_owner_dispatch \
         ON draft_accounting_attempts(json_extract(payload, '$.thread_id'), \
         json_extract(payload, '$.dispatched_at_ms'))",
    ),
    (
        "corbanu_lookup_attempt_dispatch",
        "CREATE INDEX IF NOT EXISTS corbanu_lookup_attempt_dispatch \
         ON draft_accounting_attempts(json_extract(payload, '$.dispatched_at_ms'))",
    ),
];

/// Create any missing index; returns whether they all exist now. Failure
/// only costs speed, so it is logged and tried again later.
pub(super) async fn ensure(runtime: &StateRuntime) -> bool {
    match create_missing(runtime).await {
        Ok(()) => true,
        Err(error) => {
            tracing::warn!(
                target: LOG_TARGET,
                error = %format_args!("{error:#}"),
                "accounting: could not create the ledger's lookup indexes yet; ledger writes stay slow until then"
            );
            false
        }
    }
}

async fn create_missing(runtime: &StateRuntime) -> anyhow::Result<()> {
    let mut missing = Vec::new();
    for (name, sql) in INDEXES {
        let present: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name = ?)")
                .bind(name)
                .fetch_one(runtime.pool.as_ref())
                .await?;
        if !present {
            missing.push(sql);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    let started = Instant::now();
    // Read every payload once without the lock, so that building the indexes
    // under it reads from the page cache instead of the disk.
    let _: i64 = sqlx::query_scalar(
        "SELECT count(json_extract(payload, '$.thread_id')) FROM draft_accounting_attempts",
    )
    .fetch_one(runtime.pool.as_ref())
    .await?;
    let mut tx = crate::runtime::db_lock::begin_immediate(&runtime.pool).await?;
    for sql in missing {
        if let Err(error) = sqlx::query(sql).execute(&mut *tx).await {
            tx.rollback().await?;
            return Err(error.into());
        }
    }
    tx.commit().await?;
    tracing::info!(
        target: LOG_TARGET,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "accounting: created the ledger's lookup indexes"
    );
    Ok(())
}

#[cfg(test)]
pub(super) const NAMES: [&str; 2] = [INDEXES[0].0, INDEXES[1].0];
