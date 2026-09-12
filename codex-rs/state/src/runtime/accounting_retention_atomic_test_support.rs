//! Synthetic state and exercised connection/fault support for C1 and its C2 successor.
use super::*;
use pretty_assertions::assert_eq;

pub(super) fn attach(runtime: &StateRuntime) -> Lifecycle<'_> {
    Lifecycle {
        estimates: EstimateStore {
            journal: Journal { runtime },
        },
    }
}

pub(super) async fn peer(runtime: &StateRuntime) -> anyhow::Result<StateRuntime> {
    let file: String =
        sqlx::query_scalar("SELECT file FROM pragma_database_list WHERE name = 'main'")
            .fetch_one(runtime.pool.as_ref())
            .await?;
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(file)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::ZERO);
    let mut other = runtime.clone();
    other.pool = std::sync::Arc::new(
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?,
    );
    Ok(other)
}

// Literal synthetic active state, never evidence of a completed retention sweep.
pub(super) async fn active(conn: &mut SqliteConnection, time: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = ?, admission_active = 1")
        .bind(time).execute(conn).await?;
    Ok(())
}

pub(super) fn values(
    known: [i64; 7],
    unknown: [i64; 7],
    usd: &str,
    missing: i64,
    count: i64,
) -> CompactValues {
    CompactValues::decode(
        &json!({"version":1,"known":known,"unknown":unknown,
        "known_usd":usd,"unknown_estimates":missing,"attempts":count})
        .to_string(),
    )
    .unwrap()
}

pub(super) fn available(time: i64, oldest: Option<i64>, totals: Current<DayTotals>) -> RetainedDay {
    RetainedDay::Available {
        coverage: RetentionCoverage {
            completed_as_of_ms: time,
            detail_expired_through_ms: None,
            aggregate_day_floor: 0,
            oldest_recorded_day: oldest,
        },
        totals,
    }
}

pub(super) async fn read_checked(
    conn: &mut SqliteConnection,
    day: i64,
    time: i64,
    expected: Result<&RetainedDay, &str>,
) -> anyhow::Result<()> {
    let before = dump(conn).await?;
    let changes: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *conn)
        .await?;
    let result = read_retained_on_connection(conn, attempt(1, 0).thread_id, day, time).await;
    assert_eq!(dump(conn).await?, before);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(conn)
            .await?,
        changes
    );
    match expected {
        Ok(value) => assert_eq!(&result?, value),
        Err(marker) => assert_error(result.unwrap_err(), marker),
    }
    Ok(())
}

pub(super) fn assert_error(error: anyhow::Error, marker: &str) {
    assert!(
        format!("{error:#}").contains(marker),
        "expected {marker}: {error:#}"
    );
}

pub(super) async fn schema_rejected(
    conn: &mut SqliteConnection,
    marker: &str,
) -> anyhow::Result<()> {
    let changes: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *conn)
        .await?;
    assert_error(
        Journal::append_on_connection(conn, &attempt(3, 0), &[])
            .await
            .unwrap_err(),
        marker,
    );
    assert_error(
        read_retained_on_connection(conn, attempt(1, 0).thread_id, 0, 0)
            .await
            .unwrap_err(),
        marker,
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(conn)
            .await?,
        changes
    );
    Ok(())
}

pub(super) async fn rejected(
    store: &Lifecycle<'_>,
    a: &Attempt,
    rows: &[Observation],
    marker: &str,
) -> anyhow::Result<()> {
    let mut conn = store.estimates.journal.runtime.pool.acquire().await?;
    let before = dump(&mut conn).await?;
    assert_error(
        store
            .estimates
            .journal
            .append_observation(a, rows)
            .await
            .unwrap_err(),
        marker,
    );
    assert_eq!(dump(&mut conn).await?, before);
    Ok(())
}

pub(super) async fn fault(
    conn: &mut SqliteConnection,
    site: &str,
    event: &str,
    predicate: &str,
) -> anyhow::Result<()> {
    // All fragments are explicit synthetic fault fixtures; no application input.
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "CREATE TRIGGER retention_fault_{site} BEFORE {event}
        WHEN {predicate} BEGIN SELECT RAISE(ABORT, 'retention_fault_{site}'); END;"
    )))
    .execute(conn)
    .await?;
    Ok(())
}

pub(super) async fn reopened(
    path: &std::path::Path,
    baseline: &[Vec<String>],
    day: i64,
    time: i64,
    expected: Result<&RetainedDay, &str>,
) -> anyhow::Result<()> {
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(path.to_path_buf(), "synthetic".into()).await?;
        let store = attach(&runtime);
        let mut tx = runtime.pool.begin().await?;
        assert_eq!(dump(&mut tx).await?, baseline);
        read_checked(&mut tx, day, time, expected).await?;
        tx.commit().await?;
        if let Err(marker) = expected {
            rejected(&store, &attempt(99, time), &[], marker).await?;
            assert_error(
                store
                    .delete_recorded_thread(attempt(1, 0).thread_id, time)
                    .await
                    .unwrap_err(),
                marker,
            );
        }
        runtime.close().await;
        assert!(runtime.pool.is_closed());
    }
    Ok(())
}
