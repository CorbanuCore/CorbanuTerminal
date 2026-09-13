use super::*;
use codex_protocol::protocol::SessionSource;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use sqlx::Connection;
use sqlx::Row;
use std::path::Path;
use std::sync::Arc;

pub(super) const DAY: i64 = 86_400_000;

pub(super) fn home() -> impl std::ops::Deref<Target = std::path::PathBuf> {
    scopeguard::guard(
        std::env::temp_dir().join(format!("accounting-late-{}", Uuid::new_v4())),
        |path| {
            let _ = std::fs::remove_dir_all(path);
        },
    )
}

pub(super) async fn open(path: &Path) -> anyhow::Result<Arc<StateRuntime>> {
    StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(path.to_path_buf())?),
        "synthetic".into(),
    )
    .await
}

pub(super) async fn connection(runtime: &StateRuntime) -> anyhow::Result<SqliteConnection> {
    let pool = runtime
        .sqlite()
        .open_read_write_pool(&runtime.sqlite().state_db_path())
        .await?;
    let conn = pool.acquire().await?.detach();
    pool.close().await;
    Ok(conn)
}

pub(super) async fn native(runtime: &StateRuntime, owner: ThreadId) -> anyhow::Result<()> {
    let path = runtime.sqlite().home();
    let mut builder = ThreadMetadataBuilder::new(
        owner,
        path.join("synthetic.jsonl"),
        chrono::DateTime::from_timestamp(1_700_000_000, 0).context("native fixture timestamp")?,
        SessionSource::Cli,
    );
    builder.cwd = path.to_path_buf();
    runtime.upsert_thread(&builder.build("synthetic")).await
}

pub(super) fn entry(id: u128, time: i64) -> anyhow::Result<RetainedImport> {
    Ok(RetainedImport {
        attempt: Attempt {
            attempt_id: Uuid::from_u128(id),
            request_id: Uuid::from_u128(id + 10000),
            thread_id: ThreadId::from_string(&Uuid::from_u128(7).to_string())?,
            turn: "fixture".into(),
            retry_of: None,
            provider: "synthetic".into(),
            model: "fixture".into(),
            scope: Uuid::nil(),
            dialect: Dialect::NativeAnthropic,
            dispatched_at_ms: time.try_into()?,
        },
        observations: vec![Observation {
            revision: 1.try_into()?,
            source: Uuid::from_u128(id),
            sequence: 1.try_into()?,
            patch: serde_json::from_value(serde_json::json!({"input":1}))?,
        }],
        original_price: OriginalPriceEvidence::Bound(snapshot()?),
    })
}

pub(super) fn snapshot() -> anyhow::Result<Snapshot> {
    serde_json::from_value(serde_json::json!({
        "id":Uuid::from_u128(1000), "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"1","read":null,"write":null,"output":null},
        "source_reference":Uuid::from_u128(1001), "source_kind":"NativeCatalog",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null
    }))
    .context("original synthetic snapshot")
}

// Full main DB contents, including all ten accounting tables, both migration
// ledgers, native owners and spawn edges. SQL quote preserves types and blobs.
pub(super) async fn dump(
    conn: &mut SqliteConnection,
) -> anyhow::Result<Vec<(String, Vec<String>)>> {
    let tables: Vec<String> = sqlx::query_scalar("SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .fetch_all(&mut *conn).await?;
    let mut result = Vec::new();
    for table in tables {
        let columns: Vec<String> =
            sqlx::query("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                .bind(&table)
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .map(|row| row.try_get("name"))
                .collect::<Result<_, _>>()?;
        let expression = columns
            .iter()
            .map(|name| format!("quote(\"{}\")", name.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" || '|' || ");
        let sql = format!(
            "SELECT {expression} AS row FROM \"{}\" ORDER BY row",
            table.replace('"', "\"\"")
        );
        let rows = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_all(&mut *conn)
            .await?;
        result.push((table, rows));
    }
    Ok(result)
}

pub(super) async fn reject_unchanged(
    runtime: &StateRuntime,
    bundle: &[RetainedImport],
    as_of: i64,
) -> anyhow::Result<String> {
    let store = AccountingStore::open(runtime, 0).await?;
    let mut conn = connection(runtime).await?;
    let before = dump(&mut conn).await?;
    let owner = bundle
        .first()
        .context("negative fixture entry")?
        .attempt
        .thread_id;
    let error = store
        .import_retained(owner, bundle, as_of)
        .await
        .err()
        .context("invalid import unexpectedly succeeded")?;
    assert_eq!(dump(&mut conn).await?, before);
    conn.close().await?;
    Ok(format!("{error:#}"))
}

pub(super) fn expected_day(
    owner_has_data: bool,
    as_of: i64,
    attempts: i64,
) -> anyhow::Result<RetainedDay> {
    let measured = std::array::from_fn(|index| {
        if index == 1 {
            Metric {
                known: attempts,
                unknown: 0,
            }
        } else {
            Metric {
                known: 0,
                unknown: attempts,
            }
        }
    });
    Ok(RetainedDay::Available {
        coverage: RetentionCoverage {
            completed_as_of_ms: as_of,
            detail_expired_through_ms: (as_of >= 90 * DAY).then(|| as_of - 90 * DAY),
            aggregate_day_floor: 0,
            oldest_recorded_day: owner_has_data.then_some(0),
        },
        totals: Current::Ready(DayTotals {
            measured,
            known_usd: format!("0.{attempts:06}").try_into()?,
            unknown_estimates: attempts,
            attempts,
        }),
    })
}

pub(super) async fn raw_insert_guards(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    for table in [
        "attempts",
        "observations",
        "price_bindings",
        "estimates",
        "contributions",
    ] {
        let sql = format!(
            "CREATE TRIGGER no_raw_{table} BEFORE INSERT ON draft_accounting_{table} BEGIN SELECT RAISE(ABORT, 'AGED_RAW_STAGING'); END"
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

pub(super) async fn wait_file(path: &Path) -> anyhow::Result<()> {
    tokio::time::timeout(std::time::Duration::from_secs(20), async {
        while !path.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .context("process barrier timeout")
}

pub(super) fn spawn_worker(
    path: &Path,
    name: &str,
    mode: &str,
) -> anyhow::Result<std::process::Child> {
    std::process::Command::new(std::env::current_exe()?)
        .args(["--exact", name, "--nocapture"])
        .env("ACCOUNTING_LATE_CHILD_HOME", path)
        .env("ACCOUNTING_LATE_CHILD_MODE", mode)
        .spawn()
        .context("spawn isolated accounting test process")
}

pub(super) fn child_input() -> Option<(std::path::PathBuf, String)> {
    Some((
        std::env::var_os("ACCOUNTING_LATE_CHILD_HOME")?.into(),
        std::env::var("ACCOUNTING_LATE_CHILD_MODE").ok()?,
    ))
}
