use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use sqlx::Connection;

#[path = "accounting_retention_test_support.rs"]
mod support;
use support::*;
#[path = "accounting_retention_atomic_test_support.rs"]
mod atomic_support;
use atomic_support::*;

#[tokio::test]
async fn schema_staging_admission_and_active_null_reopens() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM sqlite_schema WHERE name LIKE 'draft_accounting_%'"
        )
        .fetch_one(runtime.pool.as_ref())
        .await?,
        0
    );
    let store = install(&runtime).await?;
    save(&store, &attempt(1, 0), &[snapshot()]).await?;
    let mut conn = runtime.pool.acquire().await?;
    read_checked(&mut conn, 0, 0, Ok(&RetainedDay::NeedsActivation)).await?;
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = 0")
        .execute(&mut *conn)
        .await?;
    store
        .estimates
        .journal
        .begin_attempt(&attempt(2, 0))
        .await?;
    read_checked(&mut conn, 0, 0, Ok(&RetainedDay::NeedsActivation)).await?;
    let original = dump(&mut conn).await?;
    for mask in 0..7 {
        let mut tx = conn.begin().await?;
        for (bit, table) in [
            "DROP TABLE draft_accounting_compact_snapshots",
            "DROP TABLE draft_accounting_compact_days",
            "DROP TABLE draft_accounting_retention_checkpoint",
        ]
        .iter()
        .enumerate()
        {
            if mask & (1 << bit) == 0 {
                sqlx::query(*table).execute(&mut *tx).await?;
            }
        }
        if mask == 0 {
            assert_eq!(
                retention_fixture_on_connection(&mut tx).await?,
                RetentionFixture::Absent
            );
            Journal::append_on_connection(&mut tx, &attempt(3, 0), &[]).await?;
            assert_error(
                read_retained_on_connection(&mut tx, attempt(1, 0).thread_id, 0, 0)
                    .await
                    .unwrap_err(),
                "absent",
            );
        } else {
            schema_rejected(&mut tx, "partial").await?;
        }
        tx.rollback().await?;
        assert_eq!(dump(&mut conn).await?, original);
    }
    for (sql, marker) in [
        (
            "DELETE FROM draft_accounting_retention_checkpoint",
            "checkpoints",
        ),
        (
            "INSERT INTO draft_accounting_retention_checkpoint VALUES (2,0,0)",
            "checkpoints",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET singleton=2",
            "checkpoint key",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms=-1",
            "negative",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms=0.5",
            "mismatched types",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms='bad'",
            "mismatched types",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET admission_active=2",
            "admission bit",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET admission_active=0.5",
            "mismatched types",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET admission_active='bad'",
            "mismatched types",
        ),
        (
            "ALTER TABLE draft_accounting_retention_checkpoint DROP COLUMN admission_active",
            "schema",
        ),
        (
            "ALTER TABLE draft_accounting_compact_days RENAME COLUMN payload TO wrong",
            "schema",
        ),
        (
            "DROP TABLE draft_accounting_retention_checkpoint;
             CREATE TABLE draft_accounting_retention_checkpoint (singleton INTEGER PRIMARY KEY, completed_as_of_ms INTEGER, admission_active INTEGER);
             INSERT INTO draft_accounting_retention_checkpoint VALUES (1,0,NULL)",
            "schema",
        ),
    ] {
        let mut tx = conn.begin().await?;
        sqlx::query("PRAGMA ignore_check_constraints=ON")
            .execute(&mut *tx)
            .await?;
        sqlx::raw_sql(sql).execute(&mut *tx).await?;
        schema_rejected(&mut tx, marker).await?;
        tx.rollback().await?;
        assert_eq!(dump(&mut conn).await?, original);
    }
    sqlx::query("PRAGMA ignore_check_constraints=OFF")
        .execute(&mut *conn)
        .await?;
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms=NULL, admission_active=1").execute(&mut *conn).await?;
    read_checked(&mut conn, 0, 0, Err("active NULL")).await?;
    let baseline = dump(&mut conn).await?;
    drop(conn);
    runtime.close().await;
    reopened(&home, &baseline, 0, 0, Err("active NULL")).await
}

#[tokio::test]
async fn strict_admission_boundaries_conflicts_and_no_write_failures() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let mut conn = runtime.pool.acquire().await?;
    active(&mut conn, 0).await?;
    store
        .estimates
        .journal
        .begin_attempt(&attempt(1, 0))
        .await?;
    active(&mut conn, 400 * DAY_MS).await?;
    for (dispatch, marker) in [
        (400 * DAY_MS + 1, "future dispatch"),
        (310 * DAY_MS, "unsupported compact-only"),
        (300 * DAY_MS, "unsupported compact-only"),
        (36 * DAY_MS, "unsupported compact-only"),
        (35 * DAY_MS, "expired replay"),
        (0, "expired replay"),
    ] {
        rejected(&store, &attempt(2, dispatch), &[], marker).await?;
    }
    let a = attempt(2, 310 * DAY_MS + 1);
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 1, 1)])
        .await?;
    rejected(&store, &a, &[row(&a, 1, 2)], "observation conflict").await?;
    let mut changed = a.clone();
    changed.dispatched_at_ms = (310 * DAY_MS + 2).try_into()?;
    rejected(&store, &changed, &[], "immutable identity").await?;
    changed = attempt(3, 400 * DAY_MS);
    changed.retry_of = Some(Uuid::from_u128(99));
    sqlx::query("INSERT INTO draft_accounting_tombstones VALUES (?, ?)")
        .bind(Uuid::from_u128(99).to_string())
        .bind(401 * DAY_MS)
        .execute(&mut *conn)
        .await?;
    rejected(&store, &changed, &[], "retry predecessor").await?;
    rejected(&store, &attempt(99, 400 * DAY_MS), &[], "deleted attempt").await?;
    active(&mut conn, i64::MAX).await?;
    rejected(&store, &attempt(4, i64::MAX), &[], "overflow").await?;
    drop(conn);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn retained_totals_freshness_coverage_corruption_and_reopens() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let a = attempt(1, DAY_MS);
    save(&store, &a, &[snapshot()]).await?;
    let mut conn = runtime.pool.acquire().await?;
    compact(&mut conn, 0, &values([0; 7], [1; 7], "0", 1, 1).encode()?).await?;
    sqlx::query("UPDATE draft_accounting_compact_days SET thread_id = ?")
        .bind(Uuid::from_u128(8).to_string())
        .execute(&mut *conn)
        .await?;
    compact(&mut conn, 1, &values([0; 7], [0; 7], "0", 0, 1).encode()?).await?;
    sqlx::query("INSERT INTO draft_accounting_compact_snapshots SELECT thread_id, utc_day, snapshot_id FROM draft_accounting_compact_days CROSS JOIN draft_accounting_price_snapshots").execute(&mut *conn).await?;
    active(&mut conn, DAY_MS).await?;
    let expected = available(
        DAY_MS,
        Some(1),
        Current::Ready(
            values(
                [0, 1, 0, 0, 0, 0, 0],
                [1, 0, 1, 1, 1, 1, 1],
                "0.000000000000000000000001",
                1,
                2,
            )
            .to_day_totals()?,
        ),
    );
    read_checked(&mut conn, 1, DAY_MS, Ok(&expected)).await?;
    assert_eq!(
        store.read_retained_day(a.thread_id, 1, DAY_MS).await?,
        expected
    );
    read_checked(
        &mut conn,
        0,
        DAY_MS,
        Ok(&available(
            DAY_MS,
            Some(1),
            Current::Ready(DayTotals::default()),
        )),
    )
    .await?;
    assert_eq!(DayTotals::default().full_usd(), None);
    let before = dump(&mut conn).await?;
    assert_error(
        store.read_day(a.thread_id, 1).await.unwrap_err(),
        "read_retained_day",
    );
    assert_eq!(dump(&mut conn).await?, before);
    for (day, time, marker) in [
        (-1, DAY_MS, "negative day"),
        (1, -1, "negative as-of"),
        (0, 0, "backward"),
        (2, DAY_MS, "future day"),
        (i64::MAX, i64::MAX, "overflow"),
        (i64::MAX / DAY_MS, i64::MAX, "overflow"),
    ] {
        read_checked(&mut conn, day, time, Err(marker)).await?;
    }
    read_checked(
        &mut conn,
        1,
        DAY_MS + 1,
        Ok(&RetainedDay::NeedsMaintenance {
            completed_as_of_ms: DAY_MS,
        }),
    )
    .await?;
    for (sql, marker) in [
        (
            "UPDATE draft_accounting_attempts SET request_id='bad'",
            "identity mismatch",
        ),
        (
            "UPDATE draft_accounting_compact_days SET payload='{}'",
            "missing field",
        ),
        (
            "UPDATE draft_accounting_compact_snapshots SET snapshot_id='bad'",
            "orphan",
        ),
        (
            "UPDATE draft_accounting_estimates SET payload='{}'",
            "corrupt estimate",
        ),
        (
            "UPDATE draft_accounting_contributions SET thread_id='bad'",
            "attribution mismatch",
        ),
    ] {
        sqlx::query("PRAGMA foreign_keys=OFF")
            .execute(&mut *conn)
            .await?;
        let mut tx = conn.begin().await?;
        sqlx::query(sql).execute(&mut *tx).await?;
        read_checked(&mut tx, 1, DAY_MS, Err(marker)).await?;
        tx.rollback().await?;
        sqlx::query("PRAGMA foreign_keys=ON")
            .execute(&mut *conn)
            .await?;
    }
    let mut tx = conn.begin().await?;
    sqlx::query("DELETE FROM draft_accounting_contributions")
        .execute(&mut *tx)
        .await?;
    read_checked(
        &mut tx,
        1,
        DAY_MS,
        Ok(&available(DAY_MS, Some(1), Current::NeedsRefresh)),
    )
    .await?;
    tx.rollback().await?;
    let baseline = dump(&mut conn).await?;
    drop(conn);
    runtime.close().await;
    reopened(&home, &baseline, 1, DAY_MS, Ok(&expected)).await
}

#[tokio::test]
async fn pending_cleanup_expired_days_stale_and_known_zero() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let a = attempt(1, 0);
    save(&store, &a, &[]).await?;
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 2, 2)])
        .await?;
    let mut conn = runtime.pool.acquire().await?;
    active(&mut conn, 0).await?;
    read_checked(
        &mut conn,
        0,
        0,
        Ok(&available(0, Some(0), Current::NeedsRefresh)),
    )
    .await?;
    active(&mut conn, DETAIL_MS).await?;
    read_checked(
        &mut conn,
        0,
        DETAIL_MS,
        Ok(&RetainedDay::NeedsMaintenance {
            completed_as_of_ms: DETAIL_MS,
        }),
    )
    .await?;
    active(&mut conn, 0).await?;
    sqlx::query("UPDATE draft_accounting_estimates SET payload='{}'")
        .execute(&mut *conn)
        .await?;
    read_checked(&mut conn, 0, 0, Err("corrupt estimate")).await?;
    // Clear only this disposable fixture's rows to construct independent cleanup boundaries.
    for table in [
        "DELETE FROM draft_accounting_contributions",
        "DELETE FROM draft_accounting_estimates",
        "DELETE FROM draft_accounting_price_bindings",
        "DELETE FROM draft_accounting_observations",
        "DELETE FROM draft_accounting_attempts",
    ] {
        sqlx::query(table).execute(&mut *conn).await?;
    }
    compact(&mut conn, 0, &values([0; 7], [0; 7], "0", 0, 1).encode()?).await?;
    active(&mut conn, DETAIL_MS).await?;
    let coverage = RetentionCoverage {
        completed_as_of_ms: DETAIL_MS,
        detail_expired_through_ms: Some(0),
        aggregate_day_floor: 0,
        oldest_recorded_day: Some(0),
    };
    let zero = values([0; 7], [0; 7], "0", 0, 1).to_day_totals()?;
    assert_eq!(zero.full_usd(), Some(Decimal::default()));
    assert!(zero.measured.iter().all(|metric| metric.full() == Some(0)));
    read_checked(
        &mut conn,
        0,
        DETAIL_MS,
        Ok(&RetainedDay::Available {
            coverage,
            totals: Current::Ready(zero),
        }),
    )
    .await?;
    for time in [REPLAY_MS, REPLAY_MS + DAY_MS] {
        active(&mut conn, time).await?;
        read_checked(
            &mut conn,
            0,
            time,
            Ok(&RetainedDay::NeedsMaintenance {
                completed_as_of_ms: time,
            }),
        )
        .await?;
    }
    sqlx::query("DELETE FROM draft_accounting_compact_days")
        .execute(&mut *conn)
        .await?;
    active(&mut conn, REPLAY_MS).await?;
    read_checked(
        &mut conn,
        0,
        REPLAY_MS,
        Ok(&RetainedDay::Expired(RetentionCoverage {
            completed_as_of_ms: REPLAY_MS,
            detail_expired_through_ms: Some(275 * DAY_MS),
            aggregate_day_floor: 1,
            oldest_recorded_day: None,
        })),
    )
    .await?;
    sqlx::query("INSERT INTO draft_accounting_tombstones VALUES (?, ?)")
        .bind(Uuid::from_u128(99).to_string())
        .bind(REPLAY_MS)
        .execute(&mut *conn)
        .await?;
    read_checked(
        &mut conn,
        365,
        REPLAY_MS,
        Ok(&RetainedDay::NeedsMaintenance {
            completed_as_of_ms: REPLAY_MS,
        }),
    )
    .await?;
    drop(conn);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn actual_append_contention_and_observation_fault_rollback() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let other = peer(&runtime).await?;
    let (held_tx, held_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let pool = runtime.pool.clone();
    let holder = tokio::spawn(async move {
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        Journal::append_on_connection(&mut tx, &attempt(1, 0), &[row(&attempt(1, 0), 1, 1)])
            .await?;
        held_tx.send(()).unwrap();
        release_rx.await?;
        tx.commit().await?;
        anyhow::Ok(())
    });
    held_rx.await?;
    let error = attach(&other)
        .estimates
        .journal
        .begin_attempt(&attempt(2, 0))
        .await
        .unwrap_err();
    let sql = error
        .downcast_ref::<sqlx::Error>()
        .unwrap()
        .as_database_error()
        .unwrap();
    assert!(matches!(sql.code().as_deref(), Some("5" | "6")), "{sql}");
    release_tx.send(()).unwrap();
    holder.await??;
    attach(&other)
        .estimates
        .journal
        .begin_attempt(&attempt(2, 0))
        .await?;
    other.pool.close().await;
    let mut conn = runtime.pool.acquire().await?;
    let baseline = dump(&mut conn).await?;
    assert_eq!(
        baseline.iter().map(Vec::len).collect::<Vec<_>>(),
        [2, 1, 0, 0, 0, 0, 0, 0, 0, 1]
    );
    fault(
        &mut conn,
        "observation",
        "INSERT ON draft_accounting_observations",
        "1",
    )
    .await?;
    drop(conn);
    rejected(
        &store,
        &attempt(3, 0),
        &[row(&attempt(3, 0), 1, 1)],
        "retention_fault_observation",
    )
    .await?;
    runtime.close().await;
    reopened(&home, &baseline, 0, 0, Ok(&RetainedDay::NeedsActivation)).await?;
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    sqlx::query("DROP TRIGGER retention_fault_observation")
        .execute(runtime.pool.as_ref())
        .await?;
    let store = attach(&runtime);
    save(&store, &attempt(3, 0), &[]).await?;
    let mut conn = runtime.pool.acquire().await?;
    let before = dump(&mut conn).await?;
    store
        .estimates
        .journal
        .append_observation(&attempt(3, 0), &[row(&attempt(3, 0), 1, 1)])
        .await?;
    assert_eq!(dump(&mut conn).await?, before);
    active(&mut conn, 0).await?;
    read_checked(
        &mut conn,
        0,
        0,
        Ok(&available(0, Some(0), Current::NeedsRefresh)),
    )
    .await?;
    drop(conn);
    runtime.close().await;
    Ok(())
}
