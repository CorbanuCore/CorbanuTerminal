//! Synthetic fixture installation and complete storage/no-write evidence only.
use super::*;
use pretty_assertions::assert_eq;

pub(super) fn home() -> impl std::ops::Deref<Target = std::path::PathBuf> {
    scopeguard::guard(crate::runtime::test_support::unique_temp_dir(), |path| {
        let _ = std::fs::remove_dir_all(path);
    })
}

pub(super) fn attempt(id: u128, dispatch: i64) -> Attempt {
    serde_json::from_value(json!({"attempt_id":Uuid::from_u128(id),
        "request_id":Uuid::from_u128(id + 100), "thread_id":Uuid::from_u128(7),
        "turn":"fixture", "retry_of":null, "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "dialect":"NativeAnthropic", "dispatched_at_ms":dispatch}))
    .unwrap()
}

pub(super) fn snapshot() -> Snapshot {
    serde_json::from_value(json!({"id":Uuid::from_u128(1000), "provider":"synthetic",
        "model":"fixture", "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"0.000000000000000001","read":null,"write":null,"output":null},
        "source_reference":Uuid::from_u128(1001), "source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null}))
    .unwrap()
}

pub(super) fn row(a: &Attempt, revision: i64, input: i64) -> Observation {
    serde_json::from_value(json!({"revision":revision,"source":a.attempt_id,
        "sequence":revision,"patch":{"input":input}}))
    .unwrap()
}

pub(super) async fn install(runtime: &StateRuntime) -> anyhow::Result<Lifecycle<'_>> {
    let store = Lifecycle::create_for_tests(runtime).await?;
    let mut tx = runtime.pool.begin().await?;
    sqlx::raw_sql(
        "CREATE TABLE draft_accounting_compact_days (
            thread_id TEXT NOT NULL, utc_day INTEGER NOT NULL, payload TEXT NOT NULL,
            PRIMARY KEY(thread_id, utc_day));
         CREATE TABLE draft_accounting_compact_snapshots (
            thread_id TEXT NOT NULL, utc_day INTEGER NOT NULL, snapshot_id TEXT NOT NULL,
            PRIMARY KEY(thread_id, utc_day, snapshot_id),
            FOREIGN KEY(thread_id, utc_day) REFERENCES draft_accounting_compact_days(thread_id, utc_day),
            FOREIGN KEY(snapshot_id) REFERENCES draft_accounting_price_snapshots(snapshot_id));
         CREATE TABLE draft_accounting_retention_checkpoint (
            singleton INTEGER PRIMARY KEY, completed_as_of_ms INTEGER);
         INSERT INTO draft_accounting_retention_checkpoint VALUES (1, NULL);",
    ).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(store)
}

pub(super) async fn save(
    store: &Lifecycle<'_>,
    a: &Attempt,
    prices: &[Snapshot],
) -> anyhow::Result<()> {
    store
        .estimates
        .journal
        .append_observation(a, &[row(a, 1, 1)])
        .await?;
    store
        .estimates
        .persist_current(a.attempt_id, prices)
        .await?;
    assert_eq!(
        store.refresh_current(a.attempt_id).await?,
        Current::Ready(())
    );
    Ok(())
}

pub(super) async fn compact(
    conn: &mut SqliteConnection,
    day: i64,
    payload: &str,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO draft_accounting_compact_days VALUES (?, ?, ?)")
        .bind(Uuid::from_u128(7).to_string())
        .bind(day)
        .bind(payload)
        .execute(conn)
        .await?;
    Ok(())
}

pub(super) async fn dump(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Vec<String>>> {
    let mut tables = Vec::new();
    for query in [
        "SELECT json_array(attempt_id, request_id, payload) FROM draft_accounting_attempts ORDER BY attempt_id",
        "SELECT json_array(attempt_id, revision, source, sequence, payload) FROM draft_accounting_observations ORDER BY attempt_id, revision",
        "SELECT json_array(snapshot_id, payload) FROM draft_accounting_price_snapshots ORDER BY snapshot_id",
        "SELECT json_array(attempt_id, snapshot_id) FROM draft_accounting_price_bindings ORDER BY attempt_id",
        "SELECT json_array(attempt_id, evidence, payload) FROM draft_accounting_estimates ORDER BY attempt_id, evidence",
        "SELECT json_array(attempt_id, thread_id, utc_day, evidence) FROM draft_accounting_contributions ORDER BY attempt_id",
        "SELECT json_array(attempt_id, expires_at_ms) FROM draft_accounting_tombstones ORDER BY attempt_id",
        "SELECT json_array(thread_id, utc_day, payload) FROM draft_accounting_compact_days ORDER BY thread_id, utc_day",
        "SELECT json_array(thread_id, utc_day, snapshot_id) FROM draft_accounting_compact_snapshots ORDER BY thread_id, utc_day, snapshot_id",
        "SELECT json_array(singleton, completed_as_of_ms) FROM draft_accounting_retention_checkpoint ORDER BY singleton",
    ] {
        tables.push(sqlx::query_scalar(query).fetch_all(&mut *conn).await?);
    }
    Ok(tables)
}

pub(super) async fn unchanged(
    conn: &mut SqliteConnection,
    as_of: i64,
    expected: Result<&ValidatedRetentionInput, &str>,
) -> anyhow::Result<()> {
    let before = dump(conn).await?;
    let changes: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *conn)
        .await?;
    let result = read_validated_input_on_connection(conn, as_of).await;
    assert_eq!(dump(conn).await?, before);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(conn)
            .await?,
        changes
    );
    match expected {
        Ok(plan) => assert_eq!(&result?, plan),
        Err(message) => {
            let error = result.unwrap_err();
            assert!(
                error.to_string().contains(message),
                "expected {message}: {error:#}"
            );
        }
    }
    Ok(())
}

pub(super) async fn reopens(
    path: &std::path::Path,
    baseline: Vec<Vec<String>>,
    expected: &ValidatedRetentionInput,
) -> anyhow::Result<()> {
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(path.to_path_buf(), "synthetic".into()).await?;
        let mut tx = runtime.pool.begin().await?;
        assert_eq!(dump(&mut tx).await?, baseline);
        unchanged(&mut tx, expected.as_of_ms, Ok(expected)).await?;
        tx.commit().await?;
        runtime.close().await;
        assert!(runtime.pool.is_closed());
    }
    Ok(())
}
