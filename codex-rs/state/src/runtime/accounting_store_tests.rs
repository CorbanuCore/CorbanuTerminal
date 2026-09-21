use super::*;
use crate::runtime::test_support::test_thread_metadata;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

fn inspection(value: InspectionDay) -> Inspection {
    let InspectionDay::Ready(value) = value else {
        panic!("{value:?}")
    };
    value
}

async fn inspected(runtime: &StateRuntime, day: i64, time: i64) -> anyhow::Result<InspectionDay> {
    AccountingStore::inspect_day(runtime, attempt(1).thread_id, day, time).await
}

#[tokio::test]
async fn accounting_inspect_noncanonical_own_id_is_an_error() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    AccountingStore::open(&runtime, 0)
        .await?
        .admit(a.thread_id, &a, &[], 0)
        .await?;
    assert_eq!(
        inspection(inspected(&runtime, 0, 0).await?).totals.attempts,
        1
    );
    for spelling in [
        a.thread_id.to_string().replace('-', ""),
        format!("urn:uuid:{}", a.thread_id),
        format!("{{{}}}", a.thread_id),
    ] {
        assert_eq!(ThreadId::from_string(&spelling)?, a.thread_id);
        sqlx::query(
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.thread_id', ?)",
        )
        .bind(spelling)
        .execute(runtime.pool.as_ref())
        .await?;
        let before = rows(&runtime).await?;
        marker(
            inspected(&runtime, 0, 0).await.unwrap_err(),
            "invalid accounting ownership",
        );
        assert_eq!(rows(&runtime).await?, before);
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_work_is_not_refunded_for_unrelated_roots() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    // All six other owners are provably unrelated roots.
    sqlx::query("DELETE FROM thread_spawn_edges")
        .execute(runtime.pool.as_ref())
        .await?;
    let mut conn = runtime.pool.acquire().await?;
    let mut work = InspectionWork::new(&mut conn).await?;
    let before = work.visits;
    let view = inspect_tree_window(&mut conn, attempt(1).thread_id, 0, 0, None, &mut work).await?;
    assert_eq!(inspection(view).totals.attempts, 1);
    // Includes the owner's quote/history work, all seven candidates and six hops.
    assert!(before - work.visits >= 13);
    // Reuse the budget just as range buckets do: the second window must refuse.
    work.visits = 12;
    assert_eq!(
        inspect_tree_window(&mut conn, attempt(1).thread_id, 0, 0, None, &mut work).await?,
        InspectionDay::TooLarge
    );
    // Exhaustion on the final UNKNOWN candidate must not become unavailable+Ready.
    sqlx::query("UPDATE threads SET source = 'unknown' WHERE id = ?")
        .bind(Uuid::from_u128(13).to_string())
        .execute(&mut *conn)
        .await?;
    let mut work = InspectionWork::new(&mut conn).await?;
    work.rows = 60;
    assert_eq!(
        inspect_tree_window(&mut conn, attempt(1).thread_id, 0, 0, None, &mut work).await?,
        InspectionDay::TooLarge
    );
    drop(conn);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_provisional_unrelated_chain_is_bounded() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let mut tx = runtime.pool.begin().await?;
    sqlx::query("DELETE FROM thread_spawn_edges")
        .execute(&mut *tx)
        .await?;
    // Candidate 8 traverses >10,000 nodes before terminating at an unrelated root.
    sqlx::raw_sql(
        "WITH RECURSIVE n(i) AS (VALUES(10000) UNION ALL SELECT i+1 FROM n WHERE i<20001)
         INSERT INTO threads (id,rollout_path,created_at,updated_at,source,model_provider,cwd,title,sandbox_policy,approval_mode)
         SELECT printf('00000000-0000-0000-0000-%012x',i),'fixture',0,0,'cli','fixture','fixture','','','' FROM n;
         WITH RECURSIVE n(i) AS (VALUES(10000) UNION ALL SELECT i+1 FROM n WHERE i<20000)
         INSERT INTO thread_spawn_edges SELECT printf('00000000-0000-0000-0000-%012x',i+1),
         printf('00000000-0000-0000-0000-%012x',i),'closed' FROM n;
         INSERT INTO thread_spawn_edges VALUES ('00000000-0000-0000-0000-000000002710',
         '00000000-0000-0000-0000-000000000008','closed');"
    ).execute(&mut *tx).await?;
    tx.commit().await?;
    assert_eq!(
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            inspected(&runtime, 0, 0)
        )
        .await??,
        InspectionDay::TooLarge
    );
    // An unrelated root's large parsed-and-dropped source is not retained memory.
    sqlx::query("DELETE FROM thread_spawn_edges")
        .execute(runtime.pool.as_ref())
        .await?;
    sqlx::query("UPDATE threads SET source = ? WHERE id = ?")
        .bind(format!("{}\"cli\"", " ".repeat(4 * 1024 * 1024)))
        .bind(Uuid::from_u128(8).to_string())
        .execute(runtime.pool.as_ref())
        .await?;
    let view = inspection(inspected(&runtime, 0, 0).await?);
    assert_eq!(
        (view.totals.attempts, view.unknown_parent_totals.attempts),
        (1, 0)
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_coverage_divergence_with_overdue_other_day() -> anyhow::Result<()> {
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[], 0).await?;
    // Simulate an advanced checkpoint with retention still overdue on day zero.
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = ?")
        .bind(400 * DAY)
        .execute(runtime.pool.as_ref())
        .await?;
    let before = rows(&runtime).await?;
    assert!(matches!(
        store.read_day(a.thread_id, 400, 400 * DAY).await?,
        RetainedDay::NeedsMaintenance { .. }
    ));
    let view = inspection(inspected(&runtime, 400, 400 * DAY).await?);
    assert_eq!(
        (
            view.totals.attempts,
            view.coverage.aggregate_day_floor,
            view.coverage.oldest_recorded_day
        ),
        (0, 36, Some(0))
    );
    assert!(matches!(
        inspected(&runtime, 0, 400 * DAY).await?,
        InspectionDay::CheckpointLag
    ));
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_absent_schema_is_read_only() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let before = rows(&runtime).await?;
    assert_eq!(inspected(&runtime, 0, 0).await?, InspectionDay::Absent);
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_schema_rejection_matrix() -> anyhow::Result<()> {
    for mutation in [
        "DROP TABLE draft_accounting_price_bindings",
        "DROP INDEX draft_accounting_request",
        "UPDATE _accounting_migrations SET success = 0",
        "UPDATE _accounting_migrations SET checksum = X'00'",
        "UPDATE _accounting_migrations SET version = 2",
        "DELETE FROM _accounting_migrations",
        "DROP TABLE _accounting_migrations",
    ] {
        let path = home();
        let runtime = open(&path).await?;
        seed(&runtime).await?;
        sqlx::raw_sql(mutation)
            .execute(runtime.pool.as_ref())
            .await?;
        let before = rows(&runtime).await?;
        assert!(inspected(&runtime, 0, 1).await.is_err(), "{mutation}");
        assert_eq!(rows(&runtime).await?, before);
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_raw_day_reconciles_contributions() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    let mut retry = attempt(2);
    retry.request_id = a.request_id;
    retry.retry_of = Some(a.attempt_id);
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
    store.observe(a.thread_id, &a, &[row(1)], 0).await?;
    let first = store.observe(a.thread_id, &a, &[row(2)], 0).await?;
    let second = store.admit(a.thread_id, &retry, &[], 0).await?;
    let view = inspection(inspected(&runtime, 0, 0).await?);
    assert_eq!(
        view.requests,
        std::collections::BTreeMap::from([(a.request_id, vec![first, second])])
    );
    assert_eq!(
        view.totals,
        DayTotals {
            measured: std::array::from_fn(|index| Metric {
                known: if index == 1 { 2 } else { 0 },
                unknown: if index == 1 { 1 } else { 2 },
            }),
            known_usd: "0.000002".to_owned().try_into()?,
            unknown_estimates: 2,
            attempts: 2,
            ..Default::default()
        }
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_original_null_and_price_are_immutable() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    for (a, prices) in [(attempt(1), vec![snapshot()]), (attempt(2), vec![])] {
        store.admit(a.thread_id, &a, &prices, 0).await?;
        let before = inspected(&runtime, 0, 0).await?;
        let mut later = snapshot();
        later.id = Uuid::from_u128(9999);
        later.rates.noncached = Some("99".to_owned().try_into()?);
        store.admit(a.thread_id, &a, &[later], 0).await?;
        assert_eq!(inspected(&runtime, 0, 0).await?, before);
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_missing_binding_is_not_unpriced() -> anyhow::Result<()> {
    for mutation in [
        "DELETE FROM draft_accounting_contributions; DELETE FROM draft_accounting_estimates; DELETE FROM draft_accounting_price_bindings",
        "UPDATE draft_accounting_contributions SET thread_id = '00000000-0000-0000-0000-000000000008'",
        "UPDATE draft_accounting_attempts SET request_id = '00000000-0000-0000-0000-000000000008'",
    ] {
        let path = home();
        let runtime = open(&path).await?;
        seed(&runtime).await?;
        let a = attempt(1);
        AccountingStore::open(&runtime, 0)
            .await?
            .admit(a.thread_id, &a, &[], 0)
            .await?;
        assert!(
            inspection(inspected(&runtime, 0, 0).await?).requests[&a.request_id][0]
                .snapshot
                .is_none()
        );
        sqlx::raw_sql(mutation)
            .execute(runtime.pool.as_ref())
            .await?;
        let before = rows(&runtime).await?;
        assert!(inspected(&runtime, 0, 0).await.is_err(), "{mutation}");
        assert_eq!(rows(&runtime).await?, before);
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_checkpoint_and_stale_estimate() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[], 0).await?;
    let before = rows(&runtime).await?;
    let lagged = inspection(inspected(&runtime, 0, 12).await?);
    assert_eq!(
        (lagged.read_at_ms, lagged.coverage.completed_as_of_ms),
        (12, 0)
    );
    assert_eq!(rows(&runtime).await?, before);
    assert_eq!(
        inspected(&runtime, 1, 86_400_000).await?,
        InspectionDay::CheckpointLag
    );
    assert_eq!(rows(&runtime).await?, before);
    store.observe(a.thread_id, &a, &[row(1)], 0).await?;
    sqlx::query("UPDATE draft_accounting_contributions SET evidence = '[]'")
        .execute(runtime.pool.as_ref())
        .await?;
    let stale = rows(&runtime).await?;
    assert_eq!(
        inspected(&runtime, 0, 1).await?,
        InspectionDay::NeedsRefresh
    );
    assert_eq!(rows(&runtime).await?, stale);
    sqlx::raw_sql(
        "DELETE FROM draft_accounting_contributions; DELETE FROM draft_accounting_estimates",
    )
    .execute(runtime.pool.as_ref())
    .await?;
    assert_eq!(
        inspected(&runtime, 0, 1).await?,
        InspectionDay::NeedsRefresh
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_never_maintained_is_checkpoint_lag() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    sqlx::query(
        "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = NULL, admission_active = 0",
    )
    .execute(runtime.pool.as_ref())
    .await?;
    let before = rows(&runtime).await?;
    assert_eq!(
        inspected(&runtime, 0, 0).await?,
        InspectionDay::CheckpointLag
    );
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_retention_and_compact_matrix() -> anyhow::Result<()> {
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[], 0).await?;
    assert!(matches!(
        inspected(&runtime, 0, 90 * DAY - 1).await?,
        InspectionDay::Ready(_)
    ));
    let before = rows(&runtime).await?;
    for now in [90 * DAY, 90 * DAY + 1, 365 * DAY] {
        assert!(matches!(
            inspected(&runtime, 0, now).await?,
            InspectionDay::DetailUnavailable { compact: false, .. }
        ));
    }
    assert_eq!(rows(&runtime).await?, before);
    let mut late = attempt(2);
    late.dispatched_at_ms = 1.try_into()?;
    store.admit(a.thread_id, &late, &[], 1).await?;
    store.maintain(90 * DAY).await?;
    let unavailable = inspected(&runtime, 0, 90 * DAY).await?;
    assert!(matches!(
        unavailable,
        InspectionDay::DetailUnavailable {
            compact: true,
            coverage: RetentionCoverage {
                oldest_recorded_day: Some(0),
                ..
            },
            ..
        }
    ));
    store.maintain(90 * DAY + 1).await?;
    assert!(matches!(
        inspected(&runtime, 0, 90 * DAY + 1).await?,
        InspectionDay::DetailUnavailable { compact: true, .. }
    ));
    store.maintain(365 * DAY).await?;
    assert!(matches!(
        inspected(&runtime, 0, 365 * DAY).await?,
        InspectionDay::DetailUnavailable {
            coverage: RetentionCoverage {
                aggregate_day_floor: 1,
                oldest_recorded_day: None,
                ..
            },
            ..
        }
    ));
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_half_open_date_bounds() -> anyhow::Result<()> {
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    for (id, time) in [(1, 0), (2, DAY - 1), (3, DAY)] {
        let mut a = attempt(id);
        a.dispatched_at_ms = time.try_into()?;
        store.admit(a.thread_id, &a, &[], time).await?;
    }
    // Later observation admission does not change request-day ownership.
    store
        .observe(attempt(1).thread_id, &attempt(1), &[row(1)], DAY)
        .await?;
    assert_eq!(
        inspection(inspected(&runtime, 0, DAY).await?)
            .totals
            .attempts,
        2
    );
    assert_eq!(
        inspection(inspected(&runtime, 1, DAY).await?)
            .totals
            .attempts,
        1
    );
    for (day, now) in [
        (-1, DAY),
        (2, DAY),
        (0, -1),
        (0, DAY - 1),
        (i64::MAX, i64::MAX),
    ] {
        assert!(inspected(&runtime, day, now).await.is_err());
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_corruption_and_unknowns() -> anyhow::Result<()> {
    for mutation in [
        "UPDATE draft_accounting_observations SET source = '00000000-0000-0000-0000-000000000009'",
        "UPDATE draft_accounting_observations SET sequence = 9",
        "UPDATE draft_accounting_estimates SET payload = '{}' WHERE evidence = '[]'",
        "UPDATE draft_accounting_attempts SET payload = '{}'",
    ] {
        let path = home();
        let runtime = open(&path).await?;
        seed(&runtime).await?;
        let a = attempt(1);
        let store = AccountingStore::open(&runtime, 0).await?;
        store.admit(a.thread_id, &a, &[], 0).await?;
        let mut zero = row(1);
        zero.patch = serde_json::from_value(json!({"input":0,"read":null}))?;
        store.observe(a.thread_id, &a, &[zero.clone()], 0).await?;
        let view = inspection(inspected(&runtime, 0, 0).await?);
        assert_eq!(view.requests[&a.request_id][0].observations, vec![zero]);
        assert_eq!(view.requests[&a.request_id][0].usage.noncached, Some(0));
        assert_eq!(view.requests[&a.request_id][0].usage.read, None);
        sqlx::raw_sql(mutation)
            .execute(runtime.pool.as_ref())
            .await?;
        let before = rows(&runtime).await?;
        assert!(inspected(&runtime, 0, 0).await.is_err(), "{mutation}");
        assert_eq!(rows(&runtime).await?, before);
        runtime.close().await;
    }
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    for id in [1, 2] {
        let a = attempt(id);
        store.admit(a.thread_id, &a, &[], 0).await?;
        let mut observation = row(1);
        observation.source = a.attempt_id;
        observation.patch =
            serde_json::from_value(json!({"input": if id == 1 { i64::MAX } else { 1 }}))?;
        sqlx::query("INSERT INTO draft_accounting_observations VALUES (?, 1, ?, 1, ?)")
            .bind(a.attempt_id.to_string())
            .bind(observation.source.to_string())
            .bind(serde_json::to_string(&observation)?)
            .execute(runtime.pool.as_ref())
            .await?;
    }
    marker(inspected(&runtime, 0, 0).await.unwrap_err(), "overflow");
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_single_snapshot_concurrent_writer() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
    let before = inspected(&runtime, 0, 0).await?;
    // Establish the same read snapshot as inspect_day before releasing the writer.
    let mut tx = runtime.pool.begin().await?;
    validate_on_connection(&mut tx).await?;
    let (release, barrier) = tokio::sync::oneshot::channel();
    let writer_runtime = runtime.clone();
    let writer = tokio::spawn(async move {
        barrier.await?;
        AccountingStore::open(&writer_runtime, 0)
            .await?
            .observe(a.thread_id, &a, &[row(1)], 0)
            .await?;
        anyhow::Ok(())
    });
    release.send(()).unwrap();
    writer.await??;
    assert_eq!(
        Journal::inspect_on_connection(&mut tx, attempt(1).thread_id, 0, 0).await?,
        before
    );
    tx.commit().await?;
    let after = inspection(inspected(&runtime, 0, 0).await?);
    assert_eq!(
        after.totals.measured[1],
        Metric {
            known: 1,
            unknown: 0
        }
    );
    assert_eq!(
        after.requests[&attempt(1).request_id][0].usage.noncached,
        Some(1)
    );
    runtime.close().await;
    Ok(())
}

// Root, closed child, grandchild, orphan, unrelated root, and cycle.
async fn tree_fixture(runtime: &StateRuntime) -> anyhow::Result<()> {
    seed(runtime).await?;
    let store = AccountingStore::open(runtime, 0).await?;
    for id in 1..=7 {
        let mut a = attempt(id);
        a.thread_id = ThreadId::from_string(&Uuid::from_u128(id + 6).to_string())?;
        runtime
            .upsert_thread(&test_thread_metadata(
                runtime.sqlite().home(),
                a.thread_id,
                runtime.sqlite().home().to_path_buf(),
            ))
            .await?;
        store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
        store.observe(a.thread_id, &a, &[row(id as i64)], 0).await?;
    }
    for (parent, child) in [(7, 8), (8, 9), (999, 10), (12, 13), (13, 12)] {
        runtime
            .upsert_thread_spawn_edge(
                ThreadId::from_string(&Uuid::from_u128(parent).to_string())?,
                ThreadId::from_string(&Uuid::from_u128(child).to_string())?,
                crate::DirectionalThreadSpawnEdgeStatus::Closed,
            )
            .await?;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_tree_reconciles_and_quarantines_unknown_parents() -> anyhow::Result<()>
{
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let before = rows(&runtime).await?;
    let view = inspection(inspected(&runtime, 0, 0).await?);
    // Independent known-price calculation: one USD per million noncached tokens.
    assert_eq!(view.totals.known_usd, "0.000006".to_owned().try_into()?);
    assert_eq!(view.own_totals.known_usd, "0.000001".to_owned().try_into()?);
    assert_eq!(
        view.descendant_totals.known_usd,
        "0.000005".to_owned().try_into()?
    );
    assert_eq!(
        view.unknown_parent_totals.known_usd,
        "0.000017".to_owned().try_into()?
    );
    assert_eq!(
        (
            view.totals.attempts,
            view.own_totals.attempts,
            view.descendant_totals.attempts,
            view.unknown_parent_totals.attempts
        ),
        (3, 1, 2, 3)
    );
    let ids = |requests: &std::collections::BTreeMap<Uuid, Vec<ObservationQuote>>| {
        requests
            .values()
            .flatten()
            .map(|q| q.attempt.attempt_id)
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&view.requests), [1, 2, 3].map(Uuid::from_u128));
    assert_eq!(
        ids(&view.unknown_parent_requests),
        [4, 6, 7].map(Uuid::from_u128)
    );
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_conflicting_and_missing_edges_are_unknown() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let source =
        json!({"subagent": {"thread_spawn": {"parent_thread_id": Uuid::from_u128(7), "depth": 1}}})
            .to_string();
    // Conflicting source versus edge, then source without an edge: neither is repaired.
    for id in [9, 11] {
        sqlx::query("UPDATE threads SET source = ? WHERE id = ?")
            .bind(&source)
            .bind(Uuid::from_u128(id).to_string())
            .execute(runtime.pool.as_ref())
            .await?;
    }
    let before = rows(&runtime).await?;
    let view = inspection(inspected(&runtime, 0, 0).await?);
    assert_eq!(
        (view.totals.attempts, view.unknown_parent_totals.attempts),
        (2, 5)
    );
    assert_eq!(view.totals.known_usd, "0.000003".to_owned().try_into()?);
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_malformed_source_with_edge_is_unknown() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    // A surviving edge cannot make an unparseable source authoritative.
    sqlx::query("UPDATE threads SET source = 'malformed' WHERE id = ?")
        .bind(Uuid::from_u128(8).to_string())
        .execute(runtime.pool.as_ref())
        .await?;
    let view = inspection(inspected(&runtime, 0, 0).await?);
    assert_eq!(
        (view.totals.attempts, view.unknown_parent_totals.attempts),
        (1, 5)
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_descendant_stale_and_compact_refuse_tree_total() -> anyhow::Result<()> {
    for mutation in [
        "DELETE FROM draft_accounting_contributions WHERE attempt_id = '00000000-0000-0000-0000-000000000002'",
        "DELETE FROM draft_accounting_contributions WHERE attempt_id = '00000000-0000-0000-0000-000000000002'; DELETE FROM draft_accounting_estimates WHERE attempt_id = '00000000-0000-0000-0000-000000000002'",
    ] {
        let path = home();
        let runtime = open(&path).await?;
        tree_fixture(&runtime).await?;
        sqlx::raw_sql(mutation)
            .execute(runtime.pool.as_ref())
            .await?;
        let before = rows(&runtime).await?;
        assert_eq!(
            inspected(&runtime, 0, 0).await?,
            InspectionDay::NeedsRefresh
        );
        assert_eq!(rows(&runtime).await?, before);
        runtime.close().await;
    }
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let store = AccountingStore::open(&runtime, 90 * 86_400_000).await?;
    let before = rows(&runtime).await?;
    assert!(matches!(
        AccountingStore::inspect_day(&runtime, attempt(1).thread_id, 0, 90 * 86_400_000).await?,
        InspectionDay::DetailUnavailable { compact: true, .. }
    ));
    assert_eq!(rows(&runtime).await?, before);
    drop(store);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_unknown_unavailable_preserves_root() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let mut expected = inspection(inspected(&runtime, 0, 0).await?);
    expected.unknown_parent_requests.clear();
    expected.unknown_parent_totals = DayTotals::default();
    expected.unknown_parent_unavailable_threads = 3;
    // Orphan and both cycle members have unavailable contributions.
    for id in [4, 6, 7] {
        sqlx::query("DELETE FROM draft_accounting_contributions WHERE attempt_id = ?")
            .bind(Uuid::from_u128(id).to_string())
            .execute(runtime.pool.as_ref())
            .await?;
    }
    let before = rows(&runtime).await?;
    assert_eq!(
        inspected(&runtime, 0, 0).await?,
        InspectionDay::Ready(expected)
    );
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_unknown_compact_preserves_root_coverage() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let mut expected = inspection(inspected(&runtime, 0, 0).await?);
    expected
        .unknown_parent_requests
        .remove(&attempt(4).request_id);
    expected.unknown_parent_totals =
        DayTotals::from_quotes(expected.unknown_parent_requests.values().flatten())?;
    expected.unknown_parent_unavailable_threads = 1;
    // Synthetic compact evidence isolates attribution loss from the global age cutoff.
    let compact = json!({"version": 1, "known": vec![0; 7], "unknown": vec![1; 7],
        "known_usd": "0", "unknown_estimates": 1, "attempts": 1})
    .to_string();
    for owner in [10, 8] {
        sqlx::query("INSERT INTO draft_accounting_compact_days VALUES (?, 0, ?)")
            .bind(Uuid::from_u128(owner).to_string())
            .bind(&compact)
            .execute(runtime.pool.as_ref())
            .await?;
        let before = rows(&runtime).await?;
        let result = inspected(&runtime, 0, 0).await?;
        if owner == 10 {
            assert_eq!(inspection(result), expected);
        } else {
            assert!(matches!(
                result,
                InspectionDay::DetailUnavailable { compact: true, .. }
            ));
        }
        assert_eq!(rows(&runtime).await?, before);
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_candidate_cap_precedes_unavailable_candidates() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let mut tx = runtime.pool.begin().await?;
    let mut expected = inspection(inspected(&runtime, 0, 0).await?);
    expected.unknown_parent_unavailable_threads += 512;
    // Distinct unknown owners missing contributions remain unavailable unknowns;
    // their count must not suppress the known root and descendant totals.
    for id in 20..532 {
        let mut a = attempt(id);
        a.thread_id = ThreadId::from_string(&Uuid::from_u128(id + 1000).to_string())?;
        sqlx::query("INSERT INTO draft_accounting_attempts VALUES (?, ?, ?)")
            .bind(a.attempt_id.to_string())
            .bind(a.request_id.to_string())
            .bind(serde_json::to_string(&a)?)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, NULL)")
            .bind(a.attempt_id.to_string())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    assert_eq!(
        inspected(&runtime, 0, 0).await?,
        InspectionDay::Ready(expected)
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_tree_lineage_and_cost_share_one_snapshot() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let before = inspected(&runtime, 0, 0).await?;
    let mut read = runtime.pool.begin().await?;
    validate_on_connection(&mut read).await?;
    sqlx::query("UPDATE thread_spawn_edges SET parent_thread_id = ? WHERE child_thread_id = ?")
        .bind(Uuid::from_u128(11).to_string())
        .bind(Uuid::from_u128(8).to_string())
        .execute(runtime.pool.as_ref())
        .await?;
    assert_eq!(
        inspect_tree(&mut read, attempt(1).thread_id, 0, 0).await?,
        before
    );
    read.commit().await?;
    assert_eq!(
        inspection(inspected(&runtime, 0, 0).await?).totals.attempts,
        1
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_tree_combined_attempt_cap_is_not_per_run() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    tree_fixture(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    // Batch synthetic rows to avoid hundreds of unrelated maintenance sweeps.
    let template = store
        .admit(attempt(1).thread_id, &attempt(19), &[], 0)
        .await?;
    let mut tx = runtime.pool.begin().await?;
    for id in 20..530 {
        let mut q = template.clone();
        q.attempt = attempt(id);
        q.attempt.thread_id = ThreadId::from_string(&Uuid::from_u128(8).to_string())?;
        let a = &q.attempt;
        sqlx::query("INSERT INTO draft_accounting_attempts VALUES (?, ?, ?)")
            .bind(a.attempt_id.to_string())
            .bind(a.request_id.to_string())
            .bind(serde_json::to_string(a)?)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, NULL)")
            .bind(a.attempt_id.to_string())
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, '[]', ?)")
            .bind(a.attempt_id.to_string())
            .bind(serde_json::to_string(&q)?)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_contributions VALUES (?, ?, 0, '[]')")
            .bind(a.attempt_id.to_string())
            .bind(a.thread_id.to_string())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    assert_eq!(inspected(&runtime, 0, 0).await?, InspectionDay::TooLarge);
    runtime.close().await;
    Ok(())
}

fn range_buckets(value: InspectionDay) -> Vec<InspectionBucket> {
    let InspectionDay::Range { buckets, .. } = value else {
        panic!("{value:?}")
    };
    buckets
}

async fn range_read(
    runtime: &StateRuntime,
    start: i64,
    end: i64,
    grouping: InspectionGrouping,
    now: i64,
) -> anyhow::Result<InspectionDay> {
    AccountingStore::inspect_range(
        runtime,
        attempt(1).thread_id,
        InspectionRange {
            start_ms: start,
            end_ms: end,
            grouping,
        },
        now,
    )
    .await
}

#[tokio::test]
async fn accounting_inspect_range_invalid_empty_reversed_and_unavailable() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let before = rows(&runtime).await?;
    for (start, end, reason) in [
        (2, 1, "reversed"),
        (1, 1, "empty"),
        (-1, 1, "invalid"),
        (0, i64::MAX, "overflow"),
    ] {
        assert!(
            range_read(&runtime, start, end, InspectionGrouping::Day, 10)
                .await
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
    assert_eq!(
        range_read(&runtime, 0, 1, InspectionGrouping::Hour, 10).await?,
        InspectionDay::Absent
    );
    assert_eq!(rows(&runtime).await?, before);
    seed(&runtime).await?;
    AccountingStore::open(&runtime, 86_400_000).await?;
    let bucket = range_buckets(
        range_read(&runtime, 0, 3_600_000, InspectionGrouping::Hour, 86_400_000).await?,
    )
    .remove(0);
    assert!(!bucket.partial);
    assert_eq!(
        inspection(bucket.days.into_iter().next().unwrap()).totals,
        DayTotals::default()
    );
    runtime.close().await;
    Ok(())
}

#[test]
fn accounting_inspect_range_iso_week_calendar_month_boundaries() -> anyhow::Result<()> {
    let ms = |s: &str| {
        chrono::DateTime::parse_from_rfc3339(s)
            .unwrap()
            .timestamp_millis()
    };
    for (grouping, at, start, end) in [
        (
            InspectionGrouping::Week,
            "2021-01-01T12:00:00Z",
            "2020-12-28T00:00:00Z",
            "2021-01-04T00:00:00Z",
        ),
        (
            InspectionGrouping::Week,
            "2026-04-01T00:00:00Z",
            "2026-03-30T00:00:00Z",
            "2026-04-06T00:00:00Z",
        ),
        (
            InspectionGrouping::Month,
            "2024-02-29T23:00:00Z",
            "2024-02-01T00:00:00Z",
            "2024-03-01T00:00:00Z",
        ),
        (
            InspectionGrouping::Month,
            "2025-12-31T12:00:00Z",
            "2025-12-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
        ),
        (
            InspectionGrouping::Hour,
            "2026-03-08T09:30:00Z",
            "2026-03-08T09:00:00Z",
            "2026-03-08T10:00:00Z",
        ),
    ] {
        assert_eq!(
            InspectionRange {
                start_ms: ms(at),
                end_ms: ms(end),
                grouping
            }
            .bucket(ms(at))?,
            (ms(start), ms(end))
        );
    }
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_range_hours_half_open_and_cutoff_suffix() -> anyhow::Result<()> {
    const HOUR: i64 = 3_600_000;
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    for (id, time) in [(1, 0), (2, HOUR), (3, 2 * HOUR - 1), (4, 2 * HOUR)] {
        let mut a = attempt(id);
        a.dispatched_at_ms = time.try_into()?;
        store.admit(a.thread_id, &a, &[snapshot()], time).await?;
        store
            .observe(a.thread_id, &a, &[row(id as i64)], time)
            .await?;
    }
    store.maintain(90 * DAY).await?;
    let before = rows(&runtime).await?;
    let buckets = range_buckets(
        range_read(&runtime, HOUR, 2 * HOUR, InspectionGrouping::Hour, 90 * DAY).await?,
    );
    assert_eq!(
        (buckets[0].partial, buckets[0].effective),
        (false, Some((HOUR, 2 * HOUR)))
    );
    let view = inspection(buckets.into_iter().next().unwrap().days.remove(0));
    assert_eq!(
        view.requests
            .values()
            .flatten()
            .map(|q| q.attempt.attempt_id)
            .collect::<Vec<_>>(),
        vec![Uuid::from_u128(2), Uuid::from_u128(3)]
    );
    assert_eq!(view.totals.known_usd, "0.000005".to_owned().try_into()?);
    let old =
        range_buckets(range_read(&runtime, 0, HOUR, InspectionGrouping::Hour, 90 * DAY).await?);
    assert!(matches!(
        old[0].days[0],
        InspectionDay::DetailUnavailable { compact: true, .. }
    ));
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_range_partial_edges_and_checkpoint_coverage() -> anyhow::Result<()> {
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.maintain(3 * DAY).await?;
    let buckets = range_buckets(
        range_read(&runtime, 1, 3 * DAY - 1, InspectionGrouping::Day, 3 * DAY).await?,
    );
    assert_eq!(
        buckets
            .iter()
            .map(|b| (b.start_ms, b.end_ms, b.partial, b.effective))
            .collect::<Vec<_>>(),
        vec![
            (0, DAY, true, Some((1, DAY))),
            (DAY, 2 * DAY, false, Some((DAY, 2 * DAY))),
            (2 * DAY, 3 * DAY, true, Some((2 * DAY, 3 * DAY - 1)))
        ]
    );
    let buckets = range_buckets(
        range_read(
            &runtime,
            3 * DAY,
            4 * DAY,
            InspectionGrouping::Day,
            3 * DAY + 10,
        )
        .await?,
    );
    assert_eq!(
        (buckets[0].partial, buckets[0].effective),
        (true, Some((3 * DAY, 3 * DAY + 1)))
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_range_mixed_compaction_exact_no_double_count() -> anyhow::Result<()> {
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    for (id, time) in [(1, 0), (2, DAY)] {
        let mut a = attempt(id);
        a.dispatched_at_ms = time.try_into()?;
        store.admit(a.thread_id, &a, &[snapshot()], time).await?;
        store
            .observe(a.thread_id, &a, &[row(id as i64)], time)
            .await?;
    }
    store.maintain(90 * DAY).await?;
    let before = rows(&runtime).await?;
    let value = range_read(&runtime, 0, 2 * DAY, InspectionGrouping::Day, 90 * DAY).await?;
    assert!(
        matches!(&value, InspectionDay::Range { oldest_aggregate_day: Some(0), read_at_ms, .. } if *read_at_ms == 90 * DAY)
    );
    let mut buckets = range_buckets(value);
    assert!(matches!(
        buckets[0].days[0],
        InspectionDay::DetailUnavailable { compact: true, .. }
    ));
    let raw = inspection(buckets[1].days.remove(0));
    assert_eq!(
        (
            raw.totals.attempts,
            raw.totals.unknown_estimates,
            raw.totals.known_usd
        ),
        (1, 1, "0.000002".to_owned().try_into()?)
    );
    // The store retains the compact sum, but the inspector must never misattribute it.
    let RetainedDay::Available {
        totals: Current::Ready(compact),
        ..
    } = store.read_day(attempt(1).thread_id, 0, 90 * DAY).await?
    else {
        panic!()
    };
    assert_eq!(compact.known_usd, "0.000001".to_owned().try_into()?);
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_range_week_day_slices_and_expired_prefix() -> anyhow::Result<()> {
    const DAY: i64 = 86_400_000;
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    let mut expected = Vec::new();
    for (id, time) in [(1, 4 * DAY + 1), (2, 5 * DAY + 1)] {
        let mut a = attempt(id);
        a.dispatched_at_ms = time.try_into()?;
        store.admit(a.thread_id, &a, &[snapshot()], time).await?;
        let q = store
            .observe(a.thread_id, &a, &[row(id as i64)], time)
            .await?;
        expected.push(std::collections::BTreeMap::from([(a.request_id, vec![q])]));
    }
    // Monday 1970-01-05 through the following Monday, exclusively.
    store.maintain(11 * DAY).await?;
    for now in [11 * DAY, 94 * DAY + DAY / 2] {
        store.maintain(now).await?;
        let before = rows(&runtime).await?;
        let buckets = range_buckets(
            range_read(&runtime, 4 * DAY, 11 * DAY, InspectionGrouping::Week, now).await?,
        );
        assert_eq!(buckets.len(), 1);
        let bucket = &buckets[0];
        assert_eq!(
            (
                bucket.start_ms,
                bucket.end_ms,
                bucket.effective,
                bucket.partial
            ),
            (4 * DAY, 11 * DAY, Some((4 * DAY, 11 * DAY)), false)
        );
        assert_eq!(bucket.days.len(), 7);
        for (index, day) in bucket.days.iter().enumerate() {
            if now > 11 * DAY && index == 0 {
                assert!(matches!(
                    day,
                    InspectionDay::DetailUnavailable { compact: true, coverage, .. }
                        if coverage.detail_expired_through_ms == Some(4 * DAY + DAY / 2)
                ));
                continue;
            }
            let InspectionDay::Ready(view) = day else {
                panic!("{day:?}")
            };
            assert_eq!(view.utc_day, 4 + index as i64);
            let requests = expected.get(index).cloned().unwrap_or_default();
            assert_eq!(
                view.totals,
                DayTotals::from_quotes(requests.values().flatten())?
            );
            assert_eq!(view.requests, requests);
        }
        assert_eq!(rows(&runtime).await?, before);
    }
    // Aggregate expiry clips effective coverage even though the bucket spans days.
    let now = 370 * DAY;
    store.maintain(now).await?;
    let buckets = range_buckets(
        range_read(&runtime, 4 * DAY, 11 * DAY, InspectionGrouping::Week, now).await?,
    );
    assert_eq!(
        (
            buckets[0].days.len(),
            buckets[0].effective,
            buckets[0].partial
        ),
        (7, Some((6 * DAY, 11 * DAY)), true)
    );
    assert!(
        buckets[0]
            .days
            .iter()
            .all(|d| matches!(d, InspectionDay::DetailUnavailable { .. }))
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_range_tree_unknown_and_single_snapshot() -> anyhow::Result<()> {
    const HOUR: i64 = 3_600_000;
    let path = home();
    let runtime = Arc::new(open(&path).await?);
    tree_fixture(&runtime).await?;
    AccountingStore::open(&runtime, 2 * HOUR).await?;
    let requested = InspectionRange {
        start_ms: 0,
        end_ms: 2 * HOUR,
        grouping: InspectionGrouping::Hour,
    };
    let mut tx = runtime.pool.begin().await?;
    // Pin exactly the same snapshot that the public entrypoint owns.
    validate_on_connection(&mut tx).await?;
    let writer = runtime.clone();
    tokio::spawn(async move { AccountingStore::open(&writer, 3 * HOUR).await.map(|_| ()) })
        .await??;
    let mut buckets =
        range_buckets(inspect_buckets(&mut tx, attempt(1).thread_id, requested, 3 * HOUR).await?);
    let view = inspection(buckets[0].days.remove(0));
    assert_eq!(view.coverage.completed_as_of_ms, 2 * HOUR);
    assert_eq!(
        (
            view.totals.attempts,
            view.own_totals.attempts,
            view.descendant_totals.attempts,
            view.unknown_parent_totals.attempts
        ),
        (3, 1, 2, 3)
    );
    assert_eq!(view.totals.known_usd, "0.000006".to_owned().try_into()?);
    assert_eq!(
        view.unknown_parent_totals.known_usd,
        "0.000017".to_owned().try_into()?
    );
    assert_eq!(
        inspection(buckets[1].days.remove(0)).totals,
        DayTotals::default()
    );
    tx.commit().await?;
    runtime.close().await;
    Ok(())
}

fn home() -> impl std::ops::Deref<Target = std::path::PathBuf> {
    scopeguard::guard(crate::runtime::test_support::unique_temp_dir(), |path| {
        let _ = std::fs::remove_dir_all(path);
    })
}

fn attempt(id: u128) -> Attempt {
    serde_json::from_value(json!({
        "attempt_id":Uuid::from_u128(id), "request_id":Uuid::from_u128(id + 100),
        "thread_id":Uuid::from_u128(7), "turn":"fixture", "retry_of":null,
        "provider":"synthetic", "model":"fixture", "scope":Uuid::nil(),
        "dialect":"NativeAnthropic", "dispatched_at_ms":0
    }))
    .unwrap()
}

fn snapshot() -> Snapshot {
    serde_json::from_value(json!({
        "id":Uuid::from_u128(1000), "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"1","read":null,"write":null,"output":null},
        "source_reference":Uuid::from_u128(1001), "source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null
    }))
    .unwrap()
}

fn row(revision: i64) -> Observation {
    serde_json::from_value(json!({
        "revision":revision,"source":Uuid::from_u128(1),
        "sequence":revision,"patch":{"input":revision}
    }))
    .unwrap()
}

async fn open(path: &Path) -> anyhow::Result<Arc<StateRuntime>> {
    StateRuntime::init_for_testing(path.to_path_buf(), "synthetic".into()).await
}

async fn seed(runtime: &StateRuntime) -> anyhow::Result<()> {
    let path = runtime.sqlite().home();
    let owner = attempt(1).thread_id;
    runtime
        .upsert_thread(&test_thread_metadata(path, owner, path.to_path_buf()))
        .await?;
    sqlx::query("INSERT INTO thread_dynamic_tools (thread_id, position, name, description, input_schema) VALUES (?, 0, 'fixture', 'synthetic', '{}')")
        .bind(owner.to_string()).execute(runtime.pool.as_ref()).await?;
    runtime
        .upsert_thread_spawn_edge(
            owner,
            ThreadId::new(),
            crate::DirectionalThreadSpawnEdgeStatus::Closed,
        )
        .await?;
    AccountingStore::open(runtime, 0).await?;
    Ok(())
}

// Every column of all ten accounting and three native tables plus both ledgers.
// The caller holds one snapshot; absence is explicit for installation failures.
async fn whole(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Vec<String>>> {
    let mut result = Vec::new();
    for table in [
        "draft_accounting_attempts",
        "draft_accounting_observations",
        "draft_accounting_price_snapshots",
        "draft_accounting_price_bindings",
        "draft_accounting_estimates",
        "draft_accounting_contributions",
        "draft_accounting_tombstones",
        "draft_accounting_compact_days",
        "draft_accounting_compact_snapshots",
        "draft_accounting_retention_checkpoint",
        "threads",
        "thread_spawn_edges",
        "thread_dynamic_tools",
        "_accounting_migrations",
        "_sqlx_migrations",
    ] {
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                .bind(table)
                .fetch_all(&mut *conn)
                .await?;
        if columns.is_empty() {
            result.push(vec!["<absent>".into()]);
            continue;
        }
        let columns = columns
            .iter()
            .map(|name| {
                format!(
                    "CASE WHEN typeof(\"{name}\") = 'blob' THEN hex(\"{name}\") ELSE \"{name}\" END"
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        result.push(
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT json_array({columns}) FROM {table} ORDER BY 1"
            )))
            .fetch_all(&mut *conn)
            .await?,
        );
    }
    Ok(result)
}

async fn rows(runtime: &StateRuntime) -> anyhow::Result<Vec<Vec<String>>> {
    let mut tx = runtime.pool.begin().await?;
    let value = whole(&mut tx).await?;
    tx.commit().await?;
    Ok(value)
}

async fn reopens(path: &Path, expected: &[Vec<String>]) -> anyhow::Result<()> {
    for _ in 0..2 {
        let runtime = open(path).await?;
        assert_eq!(rows(&runtime).await?, expected);
        runtime.close().await;
        assert!(runtime.pool.is_closed());
    }
    Ok(())
}

fn marker(error: anyhow::Error, expected: &str) {
    assert!(
        format!("{error:#}").contains(expected),
        "expected {expected}: {error:#}"
    );
}

#[tokio::test]
async fn store_sql_faults_roll_back_all_tables_then_two_reopens_and_retry() -> anyhow::Result<()> {
    for (event, predicate, observing) in [
        (
            "INSERT ON draft_accounting_attempts",
            "NEW.attempt_id IS NOT NULL AND (SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint) = 1",
            false,
        ),
        (
            "INSERT ON draft_accounting_price_snapshots",
            "EXISTS(SELECT 1 FROM draft_accounting_attempts)",
            false,
        ),
        (
            "INSERT ON draft_accounting_price_bindings",
            "EXISTS(SELECT 1 FROM draft_accounting_price_snapshots)",
            false,
        ),
        (
            "INSERT ON draft_accounting_estimates",
            "EXISTS(SELECT 1 FROM draft_accounting_price_bindings)",
            false,
        ),
        (
            "INSERT ON draft_accounting_contributions",
            "EXISTS(SELECT 1 FROM draft_accounting_estimates)",
            false,
        ),
        (
            "INSERT ON draft_accounting_observations",
            "(SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint) = 1",
            true,
        ),
        (
            "UPDATE ON draft_accounting_contributions",
            "(SELECT count(*) FROM draft_accounting_estimates) = 2",
            true,
        ),
    ] {
        let path = home();
        let runtime = open(&path).await?;
        seed(&runtime).await?;
        let a = attempt(1);
        let store = AccountingStore::open(&runtime, 0).await?;
        if observing {
            store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
        }
        let before = rows(&runtime).await?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "CREATE TRIGGER store_fault BEFORE {event} WHEN {predicate} BEGIN SELECT RAISE(ABORT, 'store-exact-fault'); END"
        ))).execute(runtime.pool.as_ref()).await?;
        let error = if observing {
            store
                .observe(a.thread_id, &a, &[row(1)], 1)
                .await
                .unwrap_err()
        } else {
            store
                .admit(a.thread_id, &a, &[snapshot()], 1)
                .await
                .unwrap_err()
        };
        marker(error, "store-exact-fault");
        assert_eq!(rows(&runtime).await?, before);
        runtime.close().await;
        reopens(&path, &before).await?;
        let runtime = open(&path).await?;
        sqlx::raw_sql("DROP TRIGGER store_fault")
            .execute(runtime.pool.as_ref())
            .await?;
        let store = AccountingStore::open(&runtime, 0).await?;
        let quote = if observing {
            store.observe(a.thread_id, &a, &[row(1)], 1).await?
        } else {
            store.admit(a.thread_id, &a, &[snapshot()], 1).await?
        };
        assert_eq!(quote.snapshot, Some(snapshot()));
        assert_eq!(quote.usage.noncached, observing.then_some(1));
        let success = rows(&runtime).await?;
        assert_eq!(success[0].len(), 1);
        assert_eq!(success[3].len(), 1);
        assert_eq!(success[5].len(), 1);
        assert_eq!(&success[10..], &before[10..]);
        runtime.close().await;
        reopens(&path, &success).await?;
    }
    Ok(())
}

#[tokio::test]
async fn store_commit_failure_is_not_admission_and_rolls_back_complete_state() -> anyhow::Result<()>
{
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let before = rows(&runtime).await?;
    // Post-write accounting validation rejects accounting orphans before COMMIT.
    // A native FK fault still exercises COMMIT failure after the complete store body.
    sqlx::raw_sql(
        "CREATE TRIGGER store_commit AFTER INSERT ON draft_accounting_contributions BEGIN
         INSERT INTO thread_dynamic_tools (thread_id, position, name, description, input_schema)
         VALUES ('orphan', 0, 'fixture', 'synthetic', '{}'); END",
    )
    .execute(runtime.pool.as_ref())
    .await?;
    let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
    sqlx::raw_sql("PRAGMA defer_foreign_keys = ON")
        .execute(&mut *tx)
        .await?;
    let a = attempt(1);
    Journal::store_on_connection(&mut tx, a.thread_id, &a, &[], Some(&[snapshot()]), 1).await?;
    let error = tx.commit().await.unwrap_err();
    assert_eq!(
        error
            .as_database_error()
            .and_then(sqlx::error::DatabaseError::code)
            .as_deref(),
        Some("787")
    );
    assert!(error.to_string().contains("FOREIGN KEY"));
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    reopens(&path, &before).await?;
    let runtime = open(&path).await?;
    sqlx::raw_sql("DROP TRIGGER store_commit")
        .execute(runtime.pool.as_ref())
        .await?;
    AccountingStore::open(&runtime, 0)
        .await?
        .admit(a.thread_id, &a, &[snapshot()], 1)
        .await?;
    let success = rows(&runtime).await?;
    runtime.close().await;
    reopens(&path, &success).await
}

#[tokio::test]
async fn store_rejections_preserve_clock_native_rows_and_original_binding() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let a = attempt(1);
    let store = AccountingStore::open(&runtime, 0).await?;
    marker(
        store
            .observe(a.thread_id, &a, &[row(1)], 1)
            .await
            .unwrap_err(),
        "not admitted",
    );
    store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
    let before = rows(&runtime).await?;
    marker(
        store.admit(ThreadId::new(), &a, &[], 1).await.unwrap_err(),
        "owner mismatch",
    );
    let mut missing = attempt(2);
    missing.thread_id = ThreadId::new();
    marker(
        store
            .admit(missing.thread_id, &missing, &[], 1)
            .await
            .unwrap_err(),
        "owner missing",
    );
    let mut price = snapshot();
    price.model = "conflict".into();
    marker(
        store.admit(a.thread_id, &a, &[price], 1).await.unwrap_err(),
        "snapshot conflict",
    );
    marker(
        store
            .admit(a.thread_id, &a, &[], 90 * 86_400_000)
            .await
            .unwrap_err(),
        "compact-only late import",
    );
    marker(
        store.admit(a.thread_id, &a, &[], -1).await.unwrap_err(),
        "negative",
    );
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    reopens(&path, &before).await
}

async fn peer(runtime: &StateRuntime) -> anyhow::Result<StateRuntime> {
    let mut peer = runtime.clone();
    peer.pool = Arc::new(
        crate::sqlite::open_pool_for_testing(
            sqlx::sqlite::SqlitePoolOptions::new().max_connections(1),
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(runtime.sqlite().state_db_path())
                .foreign_keys(true)
                .busy_timeout(std::time::Duration::ZERO),
        )
        .await?,
    );
    Ok(peer)
}

#[tokio::test]
async fn public_deletion_samples_time_after_cleanup_and_writer_acquisition() -> anyhow::Result<()> {
    // Pause A at each await boundary where sampling the production time is too early.
    for pause_at_writer in [false, true] {
        let path = home();
        let runtime = open(&path).await?;
        seed(&runtime).await?;
        let mut a = attempt(1);
        let mut b = attempt(2);
        b.thread_id = ThreadId::new();
        let unrelated = ThreadId::new();
        for owner in [b.thread_id, unrelated] {
            runtime
                .upsert_thread(&test_thread_metadata(&path, owner, path.to_path_buf()))
                .await?;
        }
        let initial_time = chrono::Utc::now().timestamp_millis();
        a.dispatched_at_ms = Count::try_from(initial_time)?;
        b.dispatched_at_ms = Count::try_from(initial_time)?;
        let store = AccountingStore::open(&runtime, initial_time).await?;
        for attempt in [&a, &b] {
            store
                .admit(attempt.thread_id, attempt, &[], initial_time)
                .await?;
        }
        let before = rows(&runtime).await?;
        let mut delayed = runtime.as_ref().clone();
        let (held_tx, held_rx) = tokio::sync::oneshot::channel();
        let held_tx = Arc::new(std::sync::Mutex::new(Some(held_tx)));
        let release = Arc::new(tokio::sync::Notify::new());
        let gate = Arc::clone(&release);
        let options = if pause_at_writer {
            runtime.pool.connect_options()
        } else {
            runtime.logs_pool.connect_options()
        };
        // Pool checkout is a controlled async barrier, not a production clock hook.
        let blocked_pool = Arc::new(
            crate::sqlite::open_pool_for_testing(
                sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .before_acquire(move |_, _| {
                        let held = held_tx.lock().unwrap().take();
                        let gate = Arc::clone(&gate);
                        Box::pin(async move {
                            if let Some(held) = held {
                                held.send(()).unwrap();
                                gate.notified().await;
                            }
                            Ok(true)
                        })
                    }),
                options.as_ref().clone(),
            )
            .await?,
        );
        if pause_at_writer {
            delayed.pool = Arc::clone(&blocked_pool);
        } else {
            delayed.logs_pool = Arc::clone(&blocked_pool);
        }
        let a_owner = a.thread_id;
        let deletion = tokio::spawn(async move { delayed.delete_threads_strict(&[a_owner]).await });
        tokio::time::timeout(std::time::Duration::from_secs(5), held_rx).await??;
        let paused_at = chrono::Utc::now().timestamp_millis();
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while chrono::Utc::now().timestamp_millis() <= paused_at {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        })
        .await?;
        let other = peer(&runtime).await?;
        assert_eq!(other.delete_thread(b.thread_id).await?, 1);
        let after_b = rows(&runtime).await?;
        let checkpoint: i64 = sqlx::query_scalar(
            "SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint",
        )
        .fetch_one(runtime.pool.as_ref())
        .await?;
        assert!(checkpoint > paused_at);
        assert!(runtime.get_thread(a.thread_id).await?.is_some());
        assert!(runtime.get_thread(b.thread_id).await?.is_none());
        // Explicit clocks remain strict: never max/coerce them to the checkpoint.
        marker(
            other
                .delete_threads_at(&[unrelated], checkpoint - 1)
                .await
                .unwrap_err(),
            "negative or backward checkpoint",
        );
        assert_eq!(rows(&runtime).await?, after_b);
        release.notify_one();
        assert_eq!(
            tokio::time::timeout(std::time::Duration::from_secs(5), deletion).await???,
            1
        );
        let final_time: i64 = sqlx::query_scalar(
            "SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint",
        )
        .fetch_one(runtime.pool.as_ref())
        .await?;
        assert!(final_time >= checkpoint);
        let after = rows(&runtime).await?;
        let mut expected = before;
        for index in [0, 1, 2, 3, 4, 5, 7, 8, 11, 12] {
            expected[index].clear();
        }
        expected[6] = [&a, &b]
            .into_iter()
            .map(|attempt| {
                json!([
                    attempt.attempt_id.to_string(),
                    initial_time + 365 * 86_400_000_i64
                ])
                .to_string()
            })
            .collect();
        expected[6].sort();
        expected[9] = vec![json!([1, final_time, 1]).to_string()];
        expected[10].retain(|row| row.contains(&unrelated.to_string()));
        assert_eq!(after, expected);
        blocked_pool.close().await;
        other.pool.close().await;
        runtime.close().await;
        reopens(&path, &after).await?;
    }
    Ok(())
}

#[tokio::test]
async fn absent_store_delete_locks_before_inspection_and_ordinary_writer() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let owner = attempt(1).thread_id;
    let unrelated = ThreadId::new();
    for id in [owner, unrelated] {
        runtime
            .upsert_thread(&test_thread_metadata(&path, id, path.to_path_buf()))
            .await?;
    }
    let other = peer(&runtime).await?;
    let mut deletion = super::super::native::begin_delete(&runtime.pool).await?;
    let before = whole(&mut deletion).await?;
    assert_eq!(before[0], vec!["<absent>".to_owned()]);
    let write = "UPDATE threads SET title = 'concurrent ordinary writer' WHERE id = ?";
    // A real second connection must not commit between inspection and DELETE.
    // A deferred read transaction permits this commit and then cannot upgrade.
    let error = sqlx::query(write)
        .bind(unrelated.to_string())
        .execute(other.pool.as_ref())
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("5")
    );
    let error = AccountingStore::open(&other, 0).await.err().unwrap();
    assert_eq!(
        error
            .downcast_ref::<sqlx::Error>()
            .unwrap()
            .as_database_error()
            .unwrap()
            .code()
            .as_deref(),
        Some("5")
    );
    assert_eq!(whole(&mut deletion).await?, before);
    assert_eq!(
        StateRuntime::delete_threads_on_connection(&mut deletion, &[owner], 0).await?,
        1
    );
    deletion.commit().await?;
    assert_eq!(
        sqlx::query(write)
            .bind(unrelated.to_string())
            .execute(other.pool.as_ref())
            .await?
            .rows_affected(),
        1
    );
    // Public deletion remains usable after the ordinary writer commits, without installation.
    assert_eq!(runtime.delete_thread(unrelated).await?, 1);
    let after = rows(&runtime).await?;
    assert!(after[10].is_empty());
    for index in (0..10).chain([13]) {
        assert_eq!(after[index], vec!["<absent>".to_owned()]);
    }
    other.pool.close().await;
    runtime.close().await;
    reopens(&path, &after).await
}

#[tokio::test]
async fn resultant_day_overflow_rejects_observation_without_poisoning_native_state()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    seed(&runtime).await?;
    let unrelated = ThreadId::new();
    runtime
        .upsert_thread(&test_thread_metadata(&path, unrelated, path.to_path_buf()))
        .await?;
    let a = attempt(1);
    let b = attempt(2);
    let store = AccountingStore::open(&runtime, 0).await?;
    for attempt in [&a, &b] {
        store.admit(a.thread_id, attempt, &[], 0).await?;
    }
    let mut maximum = row(1);
    maximum.patch.input = Presence::Number(Count::try_from(i64::MAX)?);
    store.observe(a.thread_id, &a, &[maximum], 1).await?;
    let total = store.read_day(a.thread_id, 0, 1).await?;
    let RetainedDay::Available {
        totals: Current::Ready(ref totals),
        ..
    } = total
    else {
        panic!("fresh exact totals expected");
    };
    assert_eq!(
        totals.measured[1],
        Metric {
            known: i64::MAX,
            unknown: 1
        }
    );
    assert_eq!(totals.unknown_estimates, 2);
    let before = rows(&runtime).await?;
    let mut one = row(1);
    one.source = Uuid::from_u128(2);
    marker(
        store.observe(b.thread_id, &b, &[one], 2).await.unwrap_err(),
        "metric overflow",
    );
    assert_eq!(rows(&runtime).await?, before);
    assert_eq!(store.read_day(a.thread_id, 0, 1).await?, total);
    runtime.close().await;
    reopens(&path, &before).await?;
    let runtime = open(&path).await?;
    let store = AccountingStore::open(&runtime, 1).await?;
    assert_eq!(store.read_day(a.thread_id, 0, 1).await?, total);
    assert_eq!(runtime.delete_threads_at(&[unrelated], 1).await?, 1);
    assert_eq!(store.read_day(a.thread_id, 0, 1).await?, total);
    let after = rows(&runtime).await?;
    let mut expected = before;
    expected[10].retain(|row| !row.contains(&unrelated.to_string()));
    assert_eq!(after, expected);
    runtime.close().await;
    reopens(&path, &after).await
}

#[tokio::test]
async fn price_bound_admission_and_native_delete_contend_in_both_orders_with_snapshots()
-> anyhow::Result<()> {
    for delete_first in [false, true] {
        let path = home();
        let runtime = open(&path).await?;
        seed(&runtime).await?;
        let other = peer(&runtime).await?;
        let a = attempt(1);
        AccountingStore::open(&runtime, 0)
            .await?
            .admit(a.thread_id, &a, &[snapshot()], 0)
            .await?;
        let a = attempt(2);
        let before = rows(&runtime).await?;
        let mut read = runtime.pool.begin().await?;
        assert_eq!(whole(&mut read).await?, before);
        let (held_tx, held_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        let writer = Arc::clone(&runtime);
        let task = tokio::spawn(async move {
            let mut tx = writer.pool.begin_with("BEGIN IMMEDIATE").await?;
            let a = attempt(2);
            if delete_first {
                assert_eq!(
                    StateRuntime::delete_threads_on_connection(&mut tx, &[a.thread_id], 1).await?,
                    1
                );
            } else {
                Journal::store_on_connection(&mut tx, a.thread_id, &a, &[], Some(&[snapshot()]), 1)
                    .await?;
            }
            held_tx.send(()).unwrap();
            release_rx.await?;
            tx.commit().await?;
            anyhow::Ok(())
        });
        held_rx.await?;
        let error = if delete_first {
            AccountingStore { runtime: &other }
                .admit(a.thread_id, &a, &[snapshot()], 1)
                .await
                .unwrap_err()
        } else {
            other
                .delete_threads_at(&[a.thread_id], 1)
                .await
                .unwrap_err()
        };
        let sql = error
            .downcast_ref::<sqlx::Error>()
            .expect("actual SQLite contention");
        assert!(matches!(
            sql.as_database_error()
                .and_then(sqlx::error::DatabaseError::code)
                .as_deref(),
            Some("5" | "6" | "517")
        ));
        assert_eq!(whole(&mut read).await?, before);
        release_tx.send(()).unwrap();
        task.await??;
        assert_eq!(whole(&mut read).await?, before);
        if !delete_first {
            assert_eq!(other.delete_threads_at(&[a.thread_id], 1).await?, 1);
        }
        assert_eq!(whole(&mut read).await?, before);
        read.commit().await?;
        let after = rows(&runtime).await?;
        let mut expected = before.clone();
        for index in [0, 1, 2, 3, 4, 5, 7, 8, 10, 11, 12] {
            expected[index].clear();
        }
        expected[6] =
            vec![json!([attempt(1).attempt_id.to_string(), 365 * 86_400_000_i64]).to_string()];
        if !delete_first {
            expected[6].push(json!([a.attempt_id.to_string(), 365 * 86_400_000_i64]).to_string());
        }
        expected[9] = vec!["[1,1,1]".into()];
        assert_eq!(after, expected);
        for retry in [a.clone(), attempt(3)] {
            marker(
                AccountingStore { runtime: &other }
                    .admit(retry.thread_id, &retry, &[snapshot()], 1)
                    .await
                    .unwrap_err(),
                "owner missing",
            );
        }
        assert_eq!(rows(&runtime).await?, after);
        other.pool.close().await;
        runtime.close().await;
        reopens(&path, &after).await?;
    }
    Ok(())
}

#[test]
fn process_interruption_recovers_install_and_activation() -> anyhow::Result<()> {
    const CHILD_HOME: &str = "ACCOUNTING_STORE_INTERRUPT_HOME";
    const CHILD_PHASE: &str = "ACCOUNTING_STORE_INTERRUPT_PHASE";
    let executor = tokio::runtime::Runtime::new()?;
    if let Some(path) = std::env::var_os(CHILD_HOME) {
        let phase = std::env::var(CHILD_PHASE)?;
        return executor.block_on(async {
            let runtime = open(Path::new(&path)).await?;
            let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
            install_on_connection(&mut tx).await?;
            if phase == "activation" {
                tx.commit().await?;
                tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
                Journal::maintain_native_on_connection(&mut tx, 1).await?;
            }
            use std::io::Write;
            println!("ACCOUNTING_TRANSACTION_HELD");
            std::io::stdout().flush()?;
            let mut release = String::new();
            std::io::stdin().read_line(&mut release)?;
            anyhow::bail!("parent must kill the process before transaction completion")
        });
    }
    for phase in ["installation", "activation"] {
        let path = home();
        executor.block_on(async {
            let runtime = open(&path).await?;
            runtime.close().await;
            anyhow::Ok(())
        })?;
        let child = std::process::Command::new(std::env::current_exe()?)
            .args(["--exact", "runtime::accounting::store::tests::process_interruption_recovers_install_and_activation", "--nocapture"])
            .env(CHILD_HOME, path.as_os_str()).env(CHILD_PHASE, phase)
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn()?;
        let mut child = scopeguard::guard(child, |mut child| {
            let _ = child.kill();
            let _ = child.wait();
        });
        let stdout = child.stdout.take().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            use std::io::BufRead;
            for line in std::io::BufReader::new(stdout).lines() {
                if line.unwrap().contains("ACCOUNTING_TRANSACTION_HELD") {
                    let _ = send.send(());
                    break;
                }
            }
        });
        receive.recv_timeout(std::time::Duration::from_secs(20))?;
        reader.join().unwrap();
        child.kill()?;
        assert!(!child.wait()?.success());
        executor.block_on(async {
            for _ in 0..2 {
                let runtime = open(&path).await?;
                let mut conn = runtime.pool.acquire().await?;
                if phase == "installation" {
                    assert!(!ledger_exists(&mut conn).await?);
                    assert_eq!(sqlx::query_scalar::<_, i64>(
                        "SELECT count(*) FROM sqlite_schema WHERE name GLOB 'draft_accounting_*'",
                    ).fetch_one(&mut *conn).await?, 0);
                } else {
                    validate_on_connection(&mut conn).await?;
                    assert_eq!(sqlx::query_as::<_, (Option<i64>, i64)>(
                        "SELECT completed_as_of_ms, admission_active FROM draft_accounting_retention_checkpoint",
                    ).fetch_one(&mut *conn).await?, (None, 0));
                }
                drop(conn);
                runtime.close().await;
            }
            let runtime = open(&path).await?;
            let store = AccountingStore::open(&runtime, 1).await?;
            let a = attempt(1);
            let path = runtime.sqlite().home();
            runtime.upsert_thread(&test_thread_metadata(path, a.thread_id, path.to_path_buf())).await?;
            store.admit(a.thread_id, &a, &[snapshot()], 1).await?;
            let success = rows(&runtime).await?;
            runtime.close().await;
            reopens(path, &success).await
        })?;
    }
    Ok(())
}

#[tokio::test]
async fn inactive_activation_and_installed_native_delete_failures_are_retryable()
-> anyhow::Result<()> {
    for deleting in [false, true] {
        let path = home();
        let runtime = open(&path).await?;
        let a = attempt(1);
        if deleting {
            seed(&runtime).await?;
            AccountingStore::open(&runtime, 0)
                .await?
                .admit(a.thread_id, &a, &[snapshot()], 0)
                .await?;
        } else {
            let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
            install_on_connection(&mut tx).await?;
            tx.commit().await?;
        }
        let before = rows(&runtime).await?;
        let trigger = if deleting {
            "CREATE TRIGGER store_boundary BEFORE DELETE ON threads
             WHEN NOT EXISTS(SELECT 1 FROM draft_accounting_attempts)
             AND (SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint) = 1
             BEGIN SELECT RAISE(ABORT, 'after-accounting-delete'); END"
        } else {
            "CREATE TRIGGER store_boundary BEFORE UPDATE ON draft_accounting_retention_checkpoint
             BEGIN SELECT RAISE(ABORT, 'activation-checkpoint'); END"
        };
        sqlx::raw_sql(trigger)
            .execute(runtime.pool.as_ref())
            .await?;
        if deleting {
            marker(
                runtime
                    .delete_threads_at(&[a.thread_id], 1)
                    .await
                    .unwrap_err(),
                "after-accounting-delete",
            );
        } else {
            marker(
                AccountingStore::open(&runtime, 1)
                    .await
                    .err()
                    .expect("activation fails"),
                "activation-checkpoint",
            );
        }
        assert_eq!(rows(&runtime).await?, before);
        runtime.close().await;
        reopens(&path, &before).await?;
        let runtime = open(&path).await?;
        sqlx::raw_sql("DROP TRIGGER store_boundary")
            .execute(runtime.pool.as_ref())
            .await?;
        if deleting {
            assert_eq!(runtime.delete_threads_at(&[a.thread_id], 1).await?, 1);
        } else {
            AccountingStore::open(&runtime, 1).await?;
        }
        let success = rows(&runtime).await?;
        assert_eq!(success[9], vec!["[1,1,1]".to_owned()]);
        if deleting {
            for index in [0, 1, 2, 3, 4, 5, 7, 8, 10, 11, 12] {
                assert!(success[index].is_empty());
            }
            assert_eq!(
                success[6],
                vec![json!([a.attempt_id.to_string(), 365 * 86_400_000_i64]).to_string()]
            );
        }
        runtime.close().await;
        reopens(&path, &success).await?;
    }
    Ok(())
}
