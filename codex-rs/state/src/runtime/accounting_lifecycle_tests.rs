use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn attempt(id: u128, thread: ThreadId) -> Attempt {
    serde_json::from_value(json!({"attempt_id":Uuid::from_u128(id),
        "request_id":Uuid::from_u128(id + 100), "thread_id":thread,
        "turn":"fixture", "retry_of":null, "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "dialect":"NativeAnthropic", "dispatched_at_ms":100}))
    .unwrap()
}

fn snapshot(rate: &str) -> Snapshot {
    serde_json::from_value(json!({"id":Uuid::from_u128(1000), "provider":"synthetic",
        "model":"fixture", "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":rate,"read":"0.3","write":null,"output":null},
        "source_reference":Uuid::from_u128(1001), "source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null}))
    .unwrap()
}

fn row(a: &Attempt, revision: i64, patch: serde_json::Value) -> Observation {
    serde_json::from_value(json!({"revision":revision,"source":a.attempt_id,
        "sequence":revision,"patch":patch}))
    .unwrap()
}

fn home() -> impl std::ops::Deref<Target = std::path::PathBuf> {
    scopeguard::guard(crate::runtime::test_support::unique_temp_dir(), |p| {
        let _ = std::fs::remove_dir_all(p);
    })
}

async fn dump(runtime: &StateRuntime) -> anyhow::Result<Vec<Vec<String>>> {
    let mut result = Vec::new();
    for sql in [
        "SELECT json_array(attempt_id, request_id, payload) FROM draft_accounting_attempts ORDER BY 1",
        "SELECT json_array(attempt_id, revision, source, sequence, payload) FROM draft_accounting_observations ORDER BY 1",
        "SELECT json_array(snapshot_id, payload) FROM draft_accounting_price_snapshots ORDER BY 1",
        "SELECT json_array(attempt_id, snapshot_id) FROM draft_accounting_price_bindings ORDER BY 1",
        "SELECT json_array(attempt_id, evidence, payload) FROM draft_accounting_estimates ORDER BY 1",
        "SELECT json_array(attempt_id, thread_id, utc_day, evidence) FROM draft_accounting_contributions ORDER BY 1",
        "SELECT json_array(attempt_id, expires_at_ms) FROM draft_accounting_tombstones ORDER BY 1",
    ] {
        result.push(
            sqlx::query_scalar(sql)
                .fetch_all(runtime.pool.as_ref())
                .await?,
        );
    }
    Ok(result)
}

fn attached(runtime: &StateRuntime) -> Lifecycle<'_> {
    Lifecycle {
        estimates: EstimateStore {
            journal: Journal { runtime },
        },
    }
}

async fn sizes(runtime: &StateRuntime) -> anyhow::Result<Vec<usize>> {
    Ok(dump(runtime).await?.iter().map(Vec::len).collect())
}

async fn save(
    store: &Lifecycle<'_>,
    a: &Attempt,
    patch: serde_json::Value,
    prices: &[Snapshot],
) -> anyhow::Result<()> {
    store
        .estimates
        .journal
        .append_observation(a, &[row(a, 3, patch)])
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

#[tokio::test]
async fn exact_day_objects_and_two_disk_reopens() -> anyhow::Result<()> {
    let thread = ThreadId::new();
    let a = attempt(1, thread);
    let b = attempt(2, thread);
    let other = attempt(3, ThreadId::new());
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = Lifecycle::create_for_tests(&runtime).await?;
    save(&store, &a, json!({"input":50,"read":10}), &[snapshot("3")]).await?;
    save(
        &store,
        &b,
        json!({"input":1,"read":0,"write":0,"output":0}),
        &[],
    )
    .await?;
    save(
        &store,
        &other,
        json!({"input":0,"read":0,"write":0,"output":0,"reasoning":0}),
        &[],
    )
    .await?;
    let expected = Current::Ready(DayTotals {
        measured: [(1, 1), (51, 0), (10, 0), (0, 1), (0, 1), (0, 2), (1, 1)]
            .map(|(known, unknown)| Metric { known, unknown }),
        known_usd: Decimal::canonical(153, 6),
        unknown_estimates: 2,
        attempts: 2,
    });
    let zero = Current::Ready(DayTotals {
        measured: std::array::from_fn(|_| Metric::default()),
        attempts: 1,
        ..DayTotals::default()
    });
    assert_eq!(store.read_day(thread, 0).await?, expected);
    assert_eq!(store.read_day(other.thread_id, 0).await?, zero);
    let before = dump(&runtime).await?;
    assert_eq!(
        store.refresh_current(a.attempt_id).await?,
        Current::Ready(())
    );
    assert_eq!(dump(&runtime).await?, before);
    runtime.close().await;
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = attached(&runtime);
        assert_eq!(store.read_day(thread, 0).await?, expected);
        assert_eq!(store.read_day(other.thread_id, 0).await?, zero);
        assert_eq!(dump(&runtime).await?, before);
        runtime.close().await;
        assert!(runtime.pool.is_closed());
    }
    Ok(())
}

#[tokio::test]
async fn sum_before_rounding_and_checked_overflow() -> anyhow::Result<()> {
    for (rate, exact) in [
        ("0.4", Decimal::canonical(8, 7)),
        ("0.000000000000000001", Decimal::canonical(2, 24)),
    ] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = Lifecycle::create_for_tests(&runtime).await?;
        let thread = ThreadId::new();
        for id in [1, 2] {
            save(
                &store,
                &attempt(id, thread),
                json!({"input":1,"read":0,"write":0,"output":0}),
                &[snapshot(rate)],
            )
            .await?;
        }
        let Current::Ready(totals) = store.read_day(thread, 0).await? else {
            panic!("held")
        };
        assert_eq!(
            (
                totals.known_usd,
                totals.full_usd(),
                totals.measured[0].full()
            ),
            (exact, Some(exact), Some(2))
        );
        assert_eq!(
            totals.known_usd.display().text,
            if rate == "0.4" {
                "0.000001"
            } else {
                "0.000000"
            }
        );
        runtime.close().await;
    }
    assert_eq!(DayTotals::default().full_usd(), None);
    assert_eq!(
        Metric {
            known: 3,
            unknown: 1
        }
        .full(),
        None
    );
    for mut metric in [
        Metric {
            known: i64::MAX,
            unknown: 0,
        },
        Metric {
            known: 0,
            unknown: i64::MAX,
        },
    ] {
        let value = (metric.known != 0).then_some(1);
        assert!(metric.add(value).is_err());
    }
    let a = attempt(1, ThreadId::new());
    let quote = quote_observations(&a, &[row(&a, 1, json!({"input":1}))], &[snapshot("3")])?;
    for mut total in [
        DayTotals {
            attempts: i64::MAX,
            ..DayTotals::default()
        },
        DayTotals {
            unknown_estimates: i64::MAX,
            ..DayTotals::default()
        },
        DayTotals {
            known_usd: Decimal::canonical(u128::MAX, 6),
            ..DayTotals::default()
        },
    ] {
        assert!(total.add(&quote).is_err());
    }
    Ok(())
}

#[tokio::test]
async fn revisions_missing_estimates_and_corrupt_attribution() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = Lifecycle::create_for_tests(&runtime).await?;
    let a = attempt(1, ThreadId::new());
    store.estimates.journal.begin_attempt(&a).await?;
    assert_eq!(
        store.refresh_current(a.attempt_id).await?,
        Current::NeedsEstimate
    );
    assert_eq!(store.read_day(a.thread_id, 0).await?, Current::NeedsRefresh);
    save(&store, &a, json!({"input":50,"read":10}), &[snapshot("3")]).await?;
    let first = store.estimates.persist_current(a.attempt_id, &[]).await?;
    for revision in [4, 1] {
        store
            .estimates
            .journal
            .append_observation(&a, &[row(&a, revision, json!({"input":60}))])
            .await?;
        assert_eq!(
            store.refresh_current(a.attempt_id).await?,
            Current::NeedsEstimate
        );
        assert_eq!(store.read_day(a.thread_id, 0).await?, Current::NeedsRefresh);
        store.estimates.persist_current(a.attempt_id, &[]).await?;
        assert_eq!(store.read_day(a.thread_id, 0).await?, Current::NeedsRefresh);
        store.refresh_current(a.attempt_id).await?;
        let Current::Ready(totals) = store.read_day(a.thread_id, 0).await? else {
            panic!("held")
        };
        assert_eq!(
            (totals.attempts, totals.known_usd),
            (1, Decimal::canonical(183, 6))
        );
    }
    assert_eq!(
        store
            .estimates
            .read_estimate(a.attempt_id, &serde_json::to_string(&first.observations)?)
            .await?,
        Some(first)
    );
    assert_eq!(sizes(&runtime).await?, vec![1, 3, 1, 1, 3, 1, 0]);
    for sql in [
        "UPDATE draft_accounting_contributions SET utc_day = 1",
        "UPDATE draft_accounting_contributions SET thread_id = 'forged'",
    ] {
        let mut tx = runtime.pool.begin().await?;
        sqlx::query(sql).execute(&mut *tx).await?;
        tx.commit().await?;
        let before = dump(&runtime).await?;
        assert!(store.read_day(a.thread_id, 0).await.is_err());
        assert!(store.refresh_current(a.attempt_id).await.is_err());
        assert_eq!(dump(&runtime).await?, before);
        sqlx::query("UPDATE draft_accounting_contributions SET utc_day = 0, thread_id = ?")
            .bind(a.thread_id.to_string())
            .execute(runtime.pool.as_ref())
            .await?;
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn deletion_removes_all_versions_and_intentions_with_opaque_reopen_guard()
-> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = Lifecycle::create_for_tests(&runtime).await?;
    let a = attempt(1, ThreadId::new());
    let b = attempt(2, a.thread_id);
    let other = attempt(3, ThreadId::new());
    for owner in [&a, &other] {
        save(&store, owner, json!({"input":50}), &[snapshot("3")]).await?;
    }
    store.estimates.journal.begin_attempt(&b).await?;
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 4, json!({"input":60}))])
        .await?;
    store.estimates.persist_current(a.attempt_id, &[]).await?;
    let before = dump(&runtime).await?;
    store.delete_recorded_thread(a.thread_id, 100).await?;
    let deleted = dump(&runtime).await?;
    for index in 0..6 {
        let expected: Vec<_> = before[index]
            .iter()
            .filter(|r| index == 2 || r.contains(&other.attempt_id.to_string()))
            .cloned()
            .collect();
        assert_eq!(&deleted[index], &expected);
    }
    assert_eq!(
        deleted[6],
        vec![
            json!([a.attempt_id, 100 + REPLAY_MS]).to_string(),
            json!([b.attempt_id, 100 + REPLAY_MS]).to_string()
        ]
    );
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM pragma_table_info('draft_accounting_tombstones') ORDER BY cid",
    )
    .fetch_all(runtime.pool.as_ref())
    .await?;
    assert_eq!(columns, vec!["attempt_id", "expires_at_ms"]);
    store.delete_recorded_thread(a.thread_id, i64::MAX).await?;
    assert_eq!(dump(&runtime).await?, deleted);
    runtime.close().await;
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = attached(&runtime);
        assert!(store.estimates.journal.begin_attempt(&a).await.is_err());
        assert!(store.refresh_current(a.attempt_id).await.is_err());
        assert!(
            store
                .estimates
                .persist_current(a.attempt_id, &[])
                .await
                .is_err()
        );
        assert_eq!(
            store.read_day(a.thread_id, 0).await?,
            Current::Ready(DayTotals::default())
        );
        assert_eq!(dump(&runtime).await?, deleted);
        runtime.close().await;
    }
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = attached(&runtime);
    store.delete_recorded_thread(other.thread_id, 100).await?;
    assert_eq!(sizes(&runtime).await?, vec![0, 0, 0, 0, 0, 0, 3]);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn deletion_abort_invalid_times_and_corrupt_source_roll_back_whole_rows() -> anyhow::Result<()>
{
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = Lifecycle::create_for_tests(&runtime).await?;
    let a = attempt(1, ThreadId::new());
    save(&store, &a, json!({"input":50}), &[snapshot("3")]).await?;
    for (change, restore, as_of) in [
        ("SELECT 1", "SELECT 1", -1),
        ("SELECT 1", "SELECT 1", 99),
        (
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.dispatched_at_ms', 9223372036854775807)",
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.dispatched_at_ms', 100)",
            i64::MAX,
        ),
        (
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.dispatched_at_ms', -1)",
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.dispatched_at_ms', 100)",
            100,
        ),
        (
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.thread_id', 'broken')",
            "UPDATE draft_accounting_attempts SET payload = ?",
            100,
        ),
        (
            "UPDATE draft_accounting_observations SET sequence = 99",
            "UPDATE draft_accounting_observations SET sequence = 3",
            100,
        ),
        (
            "CREATE TRIGGER abort_delete BEFORE DELETE ON draft_accounting_attempts BEGIN SELECT RAISE(ABORT, 'injected'); END",
            "DROP TRIGGER abort_delete",
            100,
        ),
    ] {
        sqlx::query(change).execute(runtime.pool.as_ref()).await?;
        let before = dump(&runtime).await?;
        assert!(
            store
                .delete_recorded_thread(a.thread_id, as_of)
                .await
                .is_err(),
            "{change}"
        );
        assert_eq!(dump(&runtime).await?, before);
        sqlx::query(restore)
            .bind(serde_json::to_string(&a)?)
            .execute(runtime.pool.as_ref())
            .await?;
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn deterministic_replay_refresh_delete_orders_and_reader_snapshot() -> anyhow::Result<()> {
    for delete_first in [false, true] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = Lifecycle::create_for_tests(&runtime).await?;
        let a = attempt(1, ThreadId::new());
        save(&store, &a, json!({"input":50}), &[snapshot("3")]).await?;
        let mut reader = runtime.pool.begin().await?;
        let old: String = sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts")
            .fetch_one(&mut *reader)
            .await?;
        let barrier = tokio::sync::Barrier::new(2);
        let (deleted, replayed) = tokio::join!(
            async {
                if !delete_first {
                    barrier.wait().await;
                }
                let result = store.delete_recorded_thread(a.thread_id, 100).await;
                if delete_first {
                    barrier.wait().await;
                }
                result
            },
            async {
                if delete_first {
                    barrier.wait().await;
                }
                let replay = store
                    .estimates
                    .journal
                    .append_observation(&a, &[row(&a, 4, json!({"input":60}))])
                    .await;
                let estimate = store.estimates.persist_current(a.attempt_id, &[]).await;
                let refresh = store.refresh_current(a.attempt_id).await;
                if !delete_first {
                    barrier.wait().await;
                }
                (replay.is_ok(), estimate.is_ok(), refresh.is_ok())
            }
        );
        deleted?;
        assert_eq!(replayed, (!delete_first, !delete_first, !delete_first));
        let retained: String = sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts")
            .fetch_one(&mut *reader)
            .await?;
        assert_eq!(retained, old);
        reader.commit().await?;
        assert_eq!(sizes(&runtime).await?, vec![0, 0, 0, 0, 0, 0, 1]);
        runtime.close().await;
    }
    Ok(())
}
