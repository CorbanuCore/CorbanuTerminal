use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[path = "accounting_retention_test_support.rs"]
mod support;
use support::*;
#[path = "accounting_retention_atomic_test_support.rs"]
mod atomic_support;
use atomic_support::*;

const TINY: &str = "0.000000000000000000000001";
const TWO: &str = "0.000000000000000000000002";

fn partial(input: i64, count: i64, usd: &str) -> CompactValues {
    values(
        [0, input, 0, 0, 0, 0, 0],
        [count, 0, count, count, count, count, count],
        usd,
        count,
        count,
    )
}

fn owned(id: u128, dispatch: i64, owner: u128) -> Attempt {
    let mut a = attempt(id, dispatch);
    a.thread_id = ThreadId::from_string(&Uuid::from_u128(owner).to_string()).unwrap();
    a
}

fn response(time: i64, day: i64, oldest: Option<i64>, totals: CompactValues) -> RetainedDay {
    let coverage = RetentionCoverage {
        completed_as_of_ms: time,
        detail_expired_through_ms: (time >= DETAIL_MS).then(|| time - DETAIL_MS),
        aggregate_day_floor: if time < REPLAY_MS {
            0
        } else {
            (time - REPLAY_MS) / DAY_MS + 1
        },
        oldest_recorded_day: oldest,
    };
    if day < coverage.aggregate_day_floor {
        RetainedDay::Expired(coverage)
    } else {
        RetainedDay::Available {
            coverage,
            totals: Current::Ready(totals.to_day_totals().unwrap()),
        }
    }
}

async fn rows(runtime: &StateRuntime) -> anyhow::Result<Vec<Vec<String>>> {
    dump(&mut *runtime.pool.acquire().await?).await
}

#[tokio::test]
async fn exact_boundaries_replay_shared_prices_and_two_reopens() -> anyhow::Result<()> {
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
    for a in [attempt(1, 0), attempt(2, 1), owned(3, DAY_MS, 8)] {
        save(&store, &a, &[snapshot()]).await?;
    }
    let baseline = rows(&runtime).await?;
    let cases = [
        (DETAIL_MS - 1, vec![], vec![], vec![], partial(2, 2, TWO)),
        (
            DETAIL_MS,
            vec![1],
            vec![(7, 0, partial(1, 1, TINY), vec![1000])],
            vec![(1, REPLAY_MS)],
            partial(2, 2, TWO),
        ),
        (
            DETAIL_MS + 1,
            vec![1, 2],
            vec![(7, 0, partial(2, 2, TWO), vec![1000])],
            vec![(1, REPLAY_MS), (2, REPLAY_MS + 1)],
            partial(2, 2, TWO),
        ),
        (
            DETAIL_MS + DAY_MS,
            vec![1, 2, 3],
            vec![
                (7, 0, partial(2, 2, TWO), vec![1000]),
                (8, 1, partial(1, 1, TINY), vec![1000]),
            ],
            vec![(1, REPLAY_MS), (2, REPLAY_MS + 1), (3, REPLAY_MS + DAY_MS)],
            partial(2, 2, TWO),
        ),
        (
            REPLAY_MS,
            vec![1, 2, 3],
            vec![(8, 1, partial(1, 1, TINY), vec![1000])],
            vec![(2, REPLAY_MS + 1), (3, REPLAY_MS + DAY_MS)],
            partial(0, 0, "0"),
        ),
        (
            REPLAY_MS + 1,
            vec![1, 2, 3],
            vec![(8, 1, partial(1, 1, TINY), vec![1000])],
            vec![(3, REPLAY_MS + DAY_MS)],
            partial(0, 0, "0"),
        ),
    ];
    for (time, removed, days, tombstones, total) in cases {
        let expected = retained_rows(&baseline, &removed, &days, &tombstones, &[1000], time)?;
        let expected_read = response(time, 0, (time < REPLAY_MS).then_some(0), total);
        for _ in 0..2 {
            store.maintain_retention(time).await?;
            assert_eq!(rows(&runtime).await?, expected);
            read_checked(
                &mut *runtime.pool.acquire().await?,
                0,
                time,
                Ok(&expected_read),
            )
            .await?;
        }
        if time >= DETAIL_MS {
            rejected(
                &store,
                &attempt(99, 0),
                &[],
                if time >= REPLAY_MS {
                    "expired replay"
                } else {
                    "unsupported compact-only"
                },
            )
            .await?;
        }
    }
    let expected = rows(&runtime).await?;
    let expired = response(REPLAY_MS + 1, 0, None, partial(0, 0, "0"));
    runtime.close().await;
    reopened(&home, &expected, 0, REPLAY_MS + 1, Ok(&expired)).await?;
    for reopen in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = attach(&runtime);
        rejected(&store, &attempt(1, 0), &[], "expired replay").await?;
        rejected(&store, &attempt(2, 1), &[], "expired replay").await?;
        if reopen == 0 {
            assert_eq!(
                store
                    .read_retained_day(owned(3, DAY_MS, 8).thread_id, 1, REPLAY_MS + 1)
                    .await?,
                response(REPLAY_MS + 1, 1, Some(1), partial(1, 1, TINY))
            );
        }
        store
            .delete_recorded_thread(owned(3, DAY_MS, 8).thread_id, REPLAY_MS + 1)
            .await?;
        let deleted = retained_rows(
            &expected,
            &[],
            &[],
            &[(3, REPLAY_MS + DAY_MS)],
            &[],
            REPLAY_MS + 1,
        )?;
        assert_eq!(rows(&runtime).await?, deleted);
        runtime.close().await;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Append,
    Maintain,
    Delete,
}

impl Operation {
    async fn on_connection(
        self,
        conn: &mut SqliteConnection,
        a: &Attempt,
        time: i64,
    ) -> anyhow::Result<()> {
        match self {
            Self::Append => Journal::append_on_connection(conn, a, &[row(a, 2, 2)]).await,
            Self::Maintain => maintain_on_connection(conn, time).await,
            Self::Delete => delete_on_connection(conn, a.thread_id, time).await,
        }
    }

    async fn run(self, store: &Lifecycle<'_>, a: &Attempt, time: i64) -> anyhow::Result<()> {
        match self {
            Self::Append => {
                store
                    .estimates
                    .journal
                    .append_observation(a, &[row(a, 2, 2)])
                    .await
            }
            Self::Maintain => store.maintain_retention(time).await,
            Self::Delete => store.delete_recorded_thread(a.thread_id, time).await,
        }
    }
}

#[tokio::test]
async fn fourteen_reachable_sql_faults_rollback_reopen_retry_both_routes() -> anyhow::Result<()> {
    let sites = [
        ("compact_insert", "INSERT ON draft_accounting_compact_days"),
        ("compact_update", "UPDATE ON draft_accounting_compact_days"),
        (
            "reference_insert",
            "INSERT ON draft_accounting_compact_snapshots",
        ),
        ("tombstone_insert", "INSERT ON draft_accounting_tombstones"),
        (
            "contribution_delete",
            "DELETE ON draft_accounting_contributions",
        ),
        ("estimate_delete", "DELETE ON draft_accounting_estimates"),
        (
            "binding_delete",
            "DELETE ON draft_accounting_price_bindings",
        ),
        (
            "observation_delete",
            "DELETE ON draft_accounting_observations",
        ),
        ("attempt_delete", "DELETE ON draft_accounting_attempts"),
        (
            "reference_expiry",
            "DELETE ON draft_accounting_compact_snapshots",
        ),
        ("day_expiry", "DELETE ON draft_accounting_compact_days"),
        ("snapshot_gc", "DELETE ON draft_accounting_price_snapshots"),
        ("tombstone_expiry", "DELETE ON draft_accounting_tombstones"),
        (
            "checkpoint",
            "UPDATE ON draft_accounting_retention_checkpoint",
        ),
    ];
    assert_eq!(sites.len(), 14);
    for operation in [Operation::Maintain, Operation::Delete] {
        for (site, event) in sites {
            let home = home();
            let runtime =
                StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
            let store = install(&runtime).await?;
            for a in [attempt(1, 0), attempt(2, 1), owned(3, DAY_MS, 8)] {
                save(&store, &a, &[snapshot()]).await?;
            }
            let original = rows(&runtime).await?;
            let expiry = matches!(site, "reference_expiry" | "day_expiry" | "tombstone_expiry");
            let (time, removed, days, tombstones) = if expiry {
                store.maintain_retention(DETAIL_MS + DAY_MS).await?;
                (
                    REPLAY_MS,
                    vec![1, 2, 3],
                    vec![(8, 1, partial(1, 1, TINY), vec![1000])],
                    vec![(2, REPLAY_MS + 1), (3, REPLAY_MS + DAY_MS)],
                )
            } else if site == "compact_update" {
                store.maintain_retention(DETAIL_MS).await?;
                (
                    DETAIL_MS + 1,
                    vec![1, 2],
                    vec![(7, 0, partial(2, 2, TWO), vec![1000])],
                    vec![(1, REPLAY_MS), (2, REPLAY_MS + 1)],
                )
            } else {
                (
                    DETAIL_MS,
                    vec![1],
                    vec![(7, 0, partial(1, 1, TINY), vec![1000])],
                    vec![(1, REPLAY_MS)],
                )
            };
            let mut conn = runtime.pool.acquire().await?;
            if site == "snapshot_gc" {
                let mut spare = snapshot();
                spare.id = Uuid::from_u128(999);
                sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?)")
                    .bind(spare.id.to_string())
                    .bind(serde_json::to_string(&spare)?)
                    .execute(&mut *conn)
                    .await?;
            }
            fault(&mut conn, site, event, "1").await?;
            let baseline = dump(&mut conn).await?;
            drop(conn);
            let target = owned(99, time, 99);
            assert_error(
                operation.run(&store, &target, time).await.unwrap_err(),
                &format!("retention_fault_{site}"),
            );
            assert_eq!(rows(&runtime).await?, baseline);
            if site == "checkpoint" {
                assert_eq!(
                    store
                        .read_retained_day(attempt(1, 0).thread_id, 0, time)
                        .await?,
                    RetainedDay::NeedsActivation
                );
                // A failed first sweep leaves only the explicitly staged setup permission.
                let mut setup = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
                Journal::append_on_connection(&mut setup, &attempt(88, 0), &[]).await?;
                setup.rollback().await?;
                assert_eq!(rows(&runtime).await?, baseline);
            }
            runtime.close().await;
            for _ in 0..2 {
                let runtime =
                    StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
                assert_eq!(rows(&runtime).await?, baseline);
                runtime.close().await;
            }
            let runtime =
                StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "DROP TRIGGER retention_fault_{site}"
            )))
            .execute(runtime.pool.as_ref())
            .await?;
            let expected = retained_rows(&original, &removed, &days, &tombstones, &[1000], time)?;
            for _ in 0..2 {
                operation.run(&attach(&runtime), &target, time).await?;
                assert_eq!(rows(&runtime).await?, expected);
            }
            runtime.close().await;
            let expected_read = response(
                time,
                0,
                (!expiry).then_some(0),
                if expiry {
                    partial(0, 0, "0")
                } else {
                    partial(2, 2, TWO)
                },
            );
            reopened(&home, &expected, 0, time, Ok(&expected_read)).await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn target_deletion_faults_follow_maintenance_and_preserve_all_owners() -> anyhow::Result<()> {
    let sites = [
        (
            "target_tombstone",
            "INSERT ON draft_accounting_tombstones",
            "NEW.attempt_id",
        ),
        (
            "target_contribution",
            "DELETE ON draft_accounting_contributions",
            "OLD.attempt_id",
        ),
        (
            "target_estimate",
            "DELETE ON draft_accounting_estimates",
            "OLD.attempt_id",
        ),
        (
            "target_binding",
            "DELETE ON draft_accounting_price_bindings",
            "OLD.attempt_id",
        ),
        (
            "target_observation",
            "DELETE ON draft_accounting_observations",
            "OLD.attempt_id",
        ),
        (
            "target_attempt",
            "DELETE ON draft_accounting_attempts",
            "OLD.attempt_id",
        ),
        (
            "target_reference",
            "DELETE ON draft_accounting_compact_snapshots",
            "OLD.thread_id",
        ),
        (
            "target_day",
            "DELETE ON draft_accounting_compact_days",
            "OLD.thread_id",
        ),
        (
            "target_gc",
            "DELETE ON draft_accounting_price_snapshots",
            "OLD.snapshot_id",
        ),
    ];
    assert_eq!(sites.len(), 9);
    for (site, event, column) in sites {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = install(&runtime).await?;
        let target = attempt(1, DAY_MS);
        save(&store, &target, &[snapshot()]).await?;
        save(&store, &attempt(2, 0), &[snapshot()]).await?;
        let mut other_price = snapshot();
        if site == "target_gc" {
            other_price.id = Uuid::from_u128(1002);
        }
        save(&store, &owned(3, 1, 8), &[other_price.clone()]).await?;
        let original = rows(&runtime).await?;
        // First-activation failure as well as already-active checkpoint advancement.
        if site != "target_tombstone" {
            store.maintain_retention(DETAIL_MS).await?;
        }
        let time = DETAIL_MS + 1;
        let key = match column {
            "OLD.thread_id" => 7,
            "OLD.snapshot_id" => 1000,
            _ => 1,
        };
        let predicate = format!(
            "{column} = '{}' AND (SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint) = {time}",
            Uuid::from_u128(key)
        );
        let mut conn = runtime.pool.acquire().await?;
        fault(&mut conn, site, event, &predicate).await?;
        let baseline = dump(&mut conn).await?;
        drop(conn);
        assert_error(
            store
                .delete_recorded_thread(target.thread_id, time)
                .await
                .unwrap_err(),
            &format!("retention_fault_{site}"),
        );
        assert_eq!(rows(&runtime).await?, baseline);
        runtime.close().await;
        for _ in 0..2 {
            let runtime =
                StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
            assert_eq!(rows(&runtime).await?, baseline);
            runtime.close().await;
        }
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "DROP TRIGGER retention_fault_{site}"
        )))
        .execute(runtime.pool.as_ref())
        .await?;
        let price_id = other_price.id.as_u128();
        let tombstones = [(1, REPLAY_MS + DAY_MS), (2, REPLAY_MS), (3, REPLAY_MS + 1)];
        let expected = retained_rows(
            &original,
            &[1, 2, 3],
            &[(8, 0, partial(1, 1, TINY), vec![price_id])],
            &tombstones,
            &[price_id],
            time,
        )?;
        for _ in 0..2 {
            attach(&runtime)
                .delete_recorded_thread(target.thread_id, time)
                .await?;
            assert_eq!(rows(&runtime).await?, expected);
        }
        runtime.close().await;
        let empty = response(time, 0, None, partial(0, 0, "0"));
        reopened(&home, &expected, 0, time, Ok(&empty)).await?;
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        assert_eq!(
            attach(&runtime)
                .read_retained_day(owned(3, 1, 8).thread_id, 0, time)
                .await?,
            response(time, 0, Some(0), partial(1, 1, TINY))
        );
        attach(&runtime)
            .delete_recorded_thread(owned(3, 1, 8).thread_id, time)
            .await?;
        assert_eq!(
            rows(&runtime).await?,
            retained_rows(&original, &[1, 2, 3], &[], &tombstones, &[], time)?
        );
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn deferred_foreign_key_commit_failure_rolls_back_and_retries() -> anyhow::Result<()> {
    for operation in [Operation::Maintain, Operation::Delete] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = install(&runtime).await?;
        save(&store, &attempt(1, 0), &[snapshot()]).await?;
        let baseline = rows(&runtime).await?;
        sqlx::raw_sql("CREATE TRIGGER retention_commit_fault AFTER UPDATE ON draft_accounting_retention_checkpoint BEGIN INSERT INTO draft_accounting_compact_snapshots VALUES ('00000000-0000-0000-0000-000000000007', 0, '00000000-0000-0000-0000-000000009999'); END;")
            .execute(runtime.pool.as_ref()).await?;
        let other = peer(&runtime).await?;
        sqlx::query("PRAGMA defer_foreign_keys=ON")
            .execute(other.pool.as_ref())
            .await?;
        let target = owned(99, DETAIL_MS, 99);
        let error = operation
            .run(&attach(&other), &target, DETAIL_MS)
            .await
            .unwrap_err();
        let sql = error
            .downcast_ref::<sqlx::Error>()
            .unwrap()
            .as_database_error()
            .unwrap();
        assert_eq!(sql.code().as_deref(), Some("787"));
        assert_error(error, "FOREIGN KEY constraint failed");
        // Reacquisition waits for SQLx's dropped failed-commit transaction rollback.
        assert_eq!(rows(&other).await?, baseline);
        sqlx::query("PRAGMA defer_foreign_keys=OFF")
            .execute(other.pool.as_ref())
            .await?;
        other.pool.close().await;
        runtime.close().await;
        reopened(
            &home,
            &baseline,
            0,
            DETAIL_MS,
            Ok(&RetainedDay::NeedsActivation),
        )
        .await?;
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        sqlx::query("DROP TRIGGER retention_commit_fault")
            .execute(runtime.pool.as_ref())
            .await?;
        let expected = retained_rows(
            &baseline,
            &[1],
            &[(7, 0, partial(1, 1, TINY), vec![1000])],
            &[(1, REPLAY_MS)],
            &[1000],
            DETAIL_MS,
        )?;
        for _ in 0..2 {
            operation.run(&attach(&runtime), &target, DETAIL_MS).await?;
            assert_eq!(rows(&runtime).await?, expected);
        }
        runtime.close().await;
        reopened(
            &home,
            &expected,
            0,
            DETAIL_MS,
            Ok(&response(DETAIL_MS, 0, Some(0), partial(1, 1, TINY))),
        )
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn four_real_writer_orders_and_reader_snapshot_transfer() -> anyhow::Result<()> {
    let orders = [
        (Operation::Append, Operation::Maintain),
        (Operation::Maintain, Operation::Append),
        (Operation::Delete, Operation::Maintain),
        (Operation::Maintain, Operation::Delete),
    ];
    assert_eq!(orders.len(), 4);
    for (first, second) in orders {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = install(&runtime).await?;
        let a = attempt(1, 0);
        save(&store, &a, &[snapshot()]).await?;
        store.maintain_retention(DETAIL_MS - 1).await?;
        let baseline = rows(&runtime).await?;
        let other = peer(&runtime).await?;
        let (held_tx, held_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        let pool = runtime.pool.clone();
        let holder_attempt = a.clone();
        let holder = tokio::spawn(async move {
            let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
            first
                .on_connection(&mut tx, &holder_attempt, DETAIL_MS)
                .await?;
            held_tx.send(()).unwrap();
            release_rx.await?;
            tx.commit().await?;
            anyhow::Ok(())
        });
        held_rx.await?;
        let error = second
            .run(&attach(&other), &a, DETAIL_MS)
            .await
            .unwrap_err();
        let sql = error
            .downcast_ref::<sqlx::Error>()
            .unwrap()
            .as_database_error()
            .unwrap();
        assert!(matches!(sql.code().as_deref(), Some("5" | "6")), "{sql}");
        assert_eq!(rows(&other).await?, baseline);
        release_tx.send(()).unwrap();
        holder.await??;
        if matches!(second, Operation::Append) {
            let before_retry = rows(&other).await?;
            assert_error(
                second
                    .run(&attach(&other), &a, DETAIL_MS)
                    .await
                    .unwrap_err(),
                "unsupported compact-only",
            );
            assert_eq!(rows(&other).await?, before_retry);
        } else {
            second.run(&attach(&other), &a, DETAIL_MS).await?;
        }
        let (days, total, oldest) = match (first, second) {
            (Operation::Append, Operation::Maintain) => (
                vec![(7, 0, partial(2, 1, TWO), vec![1000])],
                partial(2, 1, TWO),
                Some(0),
            ),
            (Operation::Maintain, Operation::Append) => (
                vec![(7, 0, partial(1, 1, TINY), vec![1000])],
                partial(1, 1, TINY),
                Some(0),
            ),
            (Operation::Delete, Operation::Maintain) | (Operation::Maintain, Operation::Delete) => {
                (vec![], partial(0, 0, "0"), None)
            }
            _ => unreachable!("only the four explicit contention orders"),
        };
        let prices = if days.is_empty() { vec![] } else { vec![1000] };
        assert_eq!(
            rows(&runtime).await?,
            retained_rows(
                &baseline,
                &[1],
                &days,
                &[(1, REPLAY_MS)],
                &prices,
                DETAIL_MS
            )?
        );
        assert_eq!(
            store.read_retained_day(a.thread_id, 0, DETAIL_MS).await?,
            response(DETAIL_MS, 0, oldest, total)
        );
        other.pool.close().await;
        runtime.close().await;
    }
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    save(&store, &attempt(1, 0), &[snapshot()]).await?;
    save(&store, &attempt(2, 1), &[snapshot()]).await?;
    store.maintain_retention(DETAIL_MS - 1).await?;
    let mut reader = runtime.pool.begin().await?;
    let old_rows = dump(&mut reader).await?; // SELECT establishes the WAL snapshot.
    let before = response(DETAIL_MS - 1, 0, Some(0), partial(2, 2, TWO));
    read_checked(&mut reader, 0, DETAIL_MS - 1, Ok(&before)).await?;
    store.maintain_retention(DETAIL_MS).await?;
    assert_eq!(dump(&mut reader).await?, old_rows);
    read_checked(&mut reader, 0, DETAIL_MS - 1, Ok(&before)).await?;
    read_checked(
        &mut reader,
        0,
        DETAIL_MS,
        Ok(&RetainedDay::NeedsMaintenance {
            completed_as_of_ms: DETAIL_MS - 1,
        }),
    )
    .await?;
    let after = response(DETAIL_MS, 0, Some(0), partial(2, 2, TWO));
    read_checked(
        &mut *runtime.pool.acquire().await?,
        0,
        DETAIL_MS,
        Ok(&after),
    )
    .await?;
    assert_eq!(
        rows(&runtime).await?,
        retained_rows(
            &old_rows,
            &[1],
            &[(7, 0, partial(1, 1, TINY), vec![1000])],
            &[(1, REPLAY_MS)],
            &[1000],
            DETAIL_MS
        )?
    );
    reader.commit().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn latest_original_price_null_binding_intent_and_known_zero() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let a = attempt(1, 0);
    let mut price_json = serde_json::to_value(snapshot())?;
    price_json["rates"]["noncached"] = json!("3");
    price_json["rates"]["read"] = json!("0.3");
    let price: Snapshot = serde_json::from_value(price_json)?;
    let mut first = row(&a, 1, 50);
    first.patch = serde_json::from_value(json!({"input":50,"read":10}))?;
    store
        .estimates
        .journal
        .append_observation(&a, &[first])
        .await?;
    store
        .estimates
        .persist_current(a.attempt_id, &[price.clone()])
        .await?;
    assert_eq!(
        store.refresh_current(a.attempt_id).await?,
        Current::Ready(())
    );
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 2, 60)])
        .await?;
    let b = attempt(2, 1);
    let mut raw = row(&b, 1, 1);
    raw.patch = serde_json::from_value(json!({"input":1,"read":0,"write":0,"output":0}))?;
    store
        .estimates
        .journal
        .append_observation(&b, &[raw])
        .await?;
    store.estimates.persist_current(b.attempt_id, &[]).await?;
    assert_eq!(
        store.refresh_current(b.attempt_id).await?,
        Current::Ready(())
    );
    save(&store, &owned(3, DAY_MS, 8), &[price]).await?;
    let intent = owned(4, 0, 9);
    store.estimates.journal.begin_attempt(&intent).await?;
    let zero_attempt = owned(5, 0, 10);
    let mut zero_row = row(&zero_attempt, 1, 0);
    zero_row.patch = serde_json::from_value(
        json!({"input":0,"read":0,"write":0,"output":0,"reasoning":0,"total":0}),
    )?;
    store
        .estimates
        .journal
        .append_observation(&zero_attempt, &[zero_row])
        .await?;
    store
        .estimates
        .persist_current(zero_attempt.attempt_id, &[])
        .await?;
    assert_eq!(
        store.refresh_current(zero_attempt.attempt_id).await?,
        Current::Ready(())
    );
    let baseline = rows(&runtime).await?;
    let main_total = values(
        [1, 61, 10, 0, 0, 0, 1],
        [1, 0, 0, 1, 1, 2, 1],
        "0.000183",
        2,
        2,
    );
    let zero = values([0; 7], [0; 7], "0", 0, 1);
    let unknown = values([0; 7], [1; 7], "0", 1, 1);
    let zero_totals = zero.to_day_totals()?;
    assert_eq!(zero_totals.full_usd(), Some(Decimal::default()));
    assert!(
        zero_totals
            .measured
            .iter()
            .all(|metric| metric.full() == Some(0))
    );
    assert_eq!(unknown.to_day_totals()?.full_usd(), None);
    store.maintain_retention(DETAIL_MS).await?;
    let expected = retained_rows(
        &baseline,
        &[1, 4, 5],
        &[
            (
                7,
                0,
                values(
                    [0, 60, 10, 0, 0, 0, 0],
                    [1, 0, 0, 1, 1, 1, 1],
                    "0.000183",
                    1,
                    1,
                ),
                vec![1000],
            ),
            (9, 0, unknown, vec![]),
            (10, 0, zero, vec![]),
        ],
        &[(1, REPLAY_MS), (4, REPLAY_MS), (5, REPLAY_MS)],
        &[1000],
        DETAIL_MS,
    )?;
    assert_eq!(rows(&runtime).await?, expected);
    let read = response(DETAIL_MS, 0, Some(0), main_total);
    read_checked(&mut *runtime.pool.acquire().await?, 0, DETAIL_MS, Ok(&read)).await?;
    runtime.close().await;
    reopened(&home, &expected, 0, DETAIL_MS, Ok(&read)).await?;
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = attach(&runtime);
    // Raw+compact target is deleted only after two successful on-disk reopens.
    store.delete_recorded_thread(a.thread_id, DETAIL_MS).await?;
    let mut deleted = expected.clone();
    for index in [0, 1, 3, 4, 5] {
        deleted[index].retain(|text| !text.contains(&b.attempt_id.to_string()));
    }
    deleted[6].insert(1, json!([b.attempt_id, REPLAY_MS + 1]).to_string());
    deleted[7].remove(0);
    deleted[8].clear();
    assert_eq!(rows(&runtime).await?, deleted); // Other owner's raw binding retains original snapshot.
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn expired_first_sweep_and_invalid_times_never_resurrect_detail() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    for a in [attempt(1, 0), attempt(2, 1), attempt(3, DAY_MS)] {
        save(&store, &a, &[snapshot()]).await?;
    }
    let baseline = rows(&runtime).await?;
    for (time, marker) in [(-1, "negative"), (0, "future dispatch")] {
        for op in [Operation::Maintain, Operation::Delete] {
            assert_error(
                op.run(&store, &attempt(1, 0), time).await.unwrap_err(),
                marker,
            );
            assert_eq!(rows(&runtime).await?, baseline);
        }
    }
    store.maintain_retention(REPLAY_MS).await?;
    let expected = retained_rows(
        &baseline,
        &[1, 2, 3],
        &[(7, 1, partial(1, 1, TINY), vec![1000])],
        &[(2, REPLAY_MS + 1), (3, REPLAY_MS + DAY_MS)],
        &[1000],
        REPLAY_MS,
    )?;
    assert_eq!(rows(&runtime).await?, expected);
    for op in [Operation::Maintain, Operation::Delete] {
        assert_error(
            op.run(&store, &attempt(1, 0), REPLAY_MS - 1)
                .await
                .unwrap_err(),
            "backward",
        );
        assert_eq!(rows(&runtime).await?, expected);
    }
    runtime.close().await;
    reopened(
        &home,
        &expected,
        0,
        REPLAY_MS,
        Ok(&response(REPLAY_MS, 0, Some(1), partial(0, 0, "0"))),
    )
    .await?;
    let late_home = support::home();
    let runtime =
        StateRuntime::init_for_testing(late_home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    for a in [attempt(1, 0), attempt(2, 1)] {
        save(&store, &a, &[snapshot()]).await?;
    }
    let before = rows(&runtime).await?;
    // First activation strictly after day expiry and both exact replay expiries.
    store.maintain_retention(REPLAY_MS + 1).await?;
    let empty = retained_rows(&before, &[1, 2], &[], &[], &[], REPLAY_MS + 1)?;
    assert_eq!(rows(&runtime).await?, empty);
    runtime.close().await;
    reopened(
        &late_home,
        &empty,
        0,
        REPLAY_MS + 1,
        Ok(&response(REPLAY_MS + 1, 0, None, partial(0, 0, "0"))),
    )
    .await?;
    let overflow_home = support::home();
    let runtime =
        StateRuntime::init_for_testing(overflow_home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let a = attempt(1, i64::MAX);
    store.estimates.journal.begin_attempt(&a).await?;
    let baseline = rows(&runtime).await?;
    for op in [Operation::Maintain, Operation::Delete] {
        assert_error(op.run(&store, &a, i64::MAX).await.unwrap_err(), "overflow");
        assert_eq!(rows(&runtime).await?, baseline);
    }
    read_checked(
        &mut *runtime.pool.acquire().await?,
        i64::MAX,
        i64::MAX,
        Err("overflow"),
    )
    .await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn corrupt_inputs_fail_maintenance_read_and_deletion_then_restore_and_retry()
-> anyhow::Result<()> {
    for (table, column, bad, marker) in [
        (
            "draft_accounting_estimates",
            "payload",
            "{}",
            "corrupt estimate",
        ),
        (
            "draft_accounting_contributions",
            "thread_id",
            "bad",
            "attribution mismatch",
        ),
        (
            "draft_accounting_compact_days",
            "payload",
            "{}",
            "missing field",
        ),
        (
            "draft_accounting_compact_snapshots",
            "snapshot_id",
            "bad",
            "orphan",
        ),
        (
            "draft_accounting_price_snapshots",
            "payload",
            "{}",
            "missing field",
        ),
        (
            "draft_accounting_retention_checkpoint",
            "completed_as_of_ms",
            "NULL",
            "active NULL",
        ),
    ] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = install(&runtime).await?;
        save(&store, &attempt(1, 0), &[snapshot()]).await?;
        save(&store, &attempt(2, 1), &[snapshot()]).await?;
        store.maintain_retention(DETAIL_MS).await?;
        // Keep a valid newer quote so corruption specifically exercises an older version.
        let b = attempt(2, 1);
        store
            .estimates
            .journal
            .append_observation(&b, &[row(&b, 2, 1)])
            .await?;
        store.estimates.persist_current(b.attempt_id, &[]).await?;
        assert_eq!(
            store.refresh_current(b.attempt_id).await?,
            Current::Ready(())
        );
        let valid = rows(&runtime).await?;
        assert_eq!(valid[4].len(), 2);
        let mut conn = runtime.pool.acquire().await?;
        let original: Option<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT CAST({column} AS TEXT) FROM {table} ORDER BY rowid LIMIT 1"
        )))
        .fetch_one(&mut *conn)
        .await?;
        sqlx::query("PRAGMA foreign_keys=OFF")
            .execute(&mut *conn)
            .await?;
        let update = format!(
            "UPDATE {table} SET {column} = ? WHERE rowid = (SELECT MIN(rowid) FROM {table})"
        );
        sqlx::query(sqlx::AssertSqlSafe(update.clone()))
            .bind((bad != "NULL").then_some(bad))
            .execute(&mut *conn)
            .await?;
        let broken = dump(&mut conn).await?;
        for op in [Operation::Maintain, Operation::Delete] {
            assert_error(
                op.run(&store, &attempt(1, 0), DETAIL_MS).await.unwrap_err(),
                marker,
            );
            assert_eq!(dump(&mut conn).await?, broken);
        }
        read_checked(&mut conn, 0, DETAIL_MS, Err(marker)).await?;
        sqlx::query(sqlx::AssertSqlSafe(update))
            .bind(original)
            .execute(&mut *conn)
            .await?;
        sqlx::query("PRAGMA foreign_keys=ON")
            .execute(&mut *conn)
            .await?;
        assert_eq!(dump(&mut conn).await?, valid);
        drop(conn);
        store.maintain_retention(DETAIL_MS + 1).await?;
        let expected = retained_rows(
            &valid,
            &[2],
            &[(7, 0, partial(2, 2, TWO), vec![1000])],
            &[(1, REPLAY_MS), (2, REPLAY_MS + 1)],
            &[1000],
            DETAIL_MS + 1,
        )?;
        assert_eq!(rows(&runtime).await?, expected);
        store
            .delete_recorded_thread(attempt(1, 0).thread_id, DETAIL_MS + 1)
            .await?;
        assert_eq!(
            rows(&runtime).await?,
            retained_rows(
                &expected,
                &[],
                &[],
                &[(1, REPLAY_MS), (2, REPLAY_MS + 1)],
                &[],
                DETAIL_MS + 1
            )?
        );
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn checked_compact_overflow_propagates_through_all_coupled_paths() -> anyhow::Result<()> {
    for (bad, marker) in [
        (partial(i64::MAX, 1, "0"), "known overflow"),
        (partial(1, i64::MAX, "0"), "unknown overflow"),
        (
            partial(1, 1, "340282366920938463463374607431768211455"),
            "decimal alignment overflow",
        ),
    ] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = install(&runtime).await?;
        save(&store, &attempt(1, 0), &[snapshot()]).await?;
        save(&store, &attempt(2, 1), &[snapshot()]).await?;
        store.maintain_retention(DETAIL_MS).await?;
        let valid = rows(&runtime).await?;
        let mut conn = runtime.pool.acquire().await?;
        sqlx::query("UPDATE draft_accounting_compact_days SET payload = ?")
            .bind(bad.encode()?)
            .execute(&mut *conn)
            .await?;
        // Deliberately corrupt the checkpoint to expose pending cleanup to the reader.
        sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = ?")
            .bind(DETAIL_MS + 1)
            .execute(&mut *conn)
            .await?;
        let broken = dump(&mut conn).await?;
        read_checked(&mut conn, 0, DETAIL_MS + 1, Err(marker)).await?;
        for op in [Operation::Maintain, Operation::Delete] {
            assert_error(
                op.run(&store, &attempt(1, 0), DETAIL_MS + 1)
                    .await
                    .unwrap_err(),
                marker,
            );
            assert_eq!(dump(&mut conn).await?, broken);
        }
        sqlx::query("UPDATE draft_accounting_compact_days SET payload = ?")
            .bind(partial(1, 1, TINY).encode()?)
            .execute(&mut *conn)
            .await?;
        sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = ?")
            .bind(DETAIL_MS)
            .execute(&mut *conn)
            .await?;
        assert_eq!(dump(&mut conn).await?, valid);
        drop(conn);
        store.maintain_retention(DETAIL_MS + 1).await?;
        let expected = retained_rows(
            &valid,
            &[2],
            &[(7, 0, partial(2, 2, TWO), vec![1000])],
            &[(1, REPLAY_MS), (2, REPLAY_MS + 1)],
            &[1000],
            DETAIL_MS + 1,
        )?;
        assert_eq!(rows(&runtime).await?, expected);
        read_checked(
            &mut *runtime.pool.acquire().await?,
            0,
            DETAIL_MS + 1,
            Ok(&response(DETAIL_MS + 1, 0, Some(0), partial(2, 2, TWO))),
        )
        .await?;
        store
            .delete_recorded_thread(attempt(1, 0).thread_id, DETAIL_MS + 1)
            .await?;
        assert_eq!(
            rows(&runtime).await?,
            retained_rows(
                &expected,
                &[],
                &[],
                &[(1, REPLAY_MS), (2, REPLAY_MS + 1)],
                &[],
                DETAIL_MS + 1
            )?
        );
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn transferred_replay_suppression_survives_two_reopens_without_setup() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let a = attempt(1, 0);
    save(&store, &a, &[snapshot()]).await?;
    save(&store, &attempt(2, 1), &[snapshot()]).await?;
    let original = rows(&runtime).await?;
    let time = DETAIL_MS + 1;
    store.maintain_retention(time).await?;
    let expected = retained_rows(
        &original,
        &[1, 2],
        &[(7, 0, partial(2, 2, TWO), vec![1000])],
        &[(1, REPLAY_MS), (2, REPLAY_MS + 1)],
        &[1000],
        time,
    )?;
    assert_eq!(rows(&runtime).await?, expected);
    runtime.close().await;
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = attach(&runtime);
        assert_eq!(rows(&runtime).await?, expected);
        read_checked(
            &mut *runtime.pool.acquire().await?,
            0,
            time,
            Ok(&response(time, 0, Some(0), partial(2, 2, TWO))),
        )
        .await?;
        rejected(&store, &a, &[row(&a, 2, 60)], "unsupported compact-only").await?;
        rejected(&store, &attempt(1, time), &[], "deleted attempt").await?;
        rejected(&store, &attempt(99, 0), &[], "unsupported compact-only").await?;
        let mut retry = attempt(99, time);
        retry.request_id = a.request_id;
        retry.retry_of = Some(a.attempt_id);
        rejected(&store, &retry, &[], "retry predecessor").await?;
        store.maintain_retention(time).await?;
        assert_eq!(rows(&runtime).await?, expected);
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn target_failure_rolls_back_day365_expiry_and_replay_floor() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let target = attempt(1, 276 * DAY_MS);
    for a in [
        target.clone(),
        owned(2, 0, 9),
        owned(3, 275 * DAY_MS, 8),
        attempt(4, DAY_MS),
    ] {
        save(&store, &a, &[snapshot()]).await?;
    }
    let original = rows(&runtime).await?;
    store.maintain_retention(REPLAY_MS - 1).await?;
    let baseline = retained_rows(
        &original,
        &[2, 4],
        &[
            (7, 1, partial(1, 1, TINY), vec![1000]),
            (9, 0, partial(1, 1, TINY), vec![1000]),
        ],
        &[(2, REPLAY_MS), (4, REPLAY_MS + DAY_MS)],
        &[1000],
        REPLAY_MS - 1,
    )?;
    assert_eq!(rows(&runtime).await?, baseline);
    let mut conn = runtime.pool.acquire().await?;
    let predicate = format!(
        "NEW.attempt_id = '{}' AND (SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint) = {REPLAY_MS}",
        target.attempt_id
    );
    fault(
        &mut conn,
        "target_floor",
        "INSERT ON draft_accounting_tombstones",
        &predicate,
    )
    .await?;
    drop(conn);
    assert_error(
        store
            .delete_recorded_thread(target.thread_id, REPLAY_MS)
            .await
            .unwrap_err(),
        "retention_fault_target_floor",
    );
    assert_eq!(rows(&runtime).await?, baseline);
    // Day zero is still available at the old checkpoint; expiry/floor did not partially commit.
    let before_read = response(REPLAY_MS - 1, 0, Some(1), partial(0, 0, "0"));
    read_checked(
        &mut *runtime.pool.acquire().await?,
        0,
        REPLAY_MS - 1,
        Ok(&before_read),
    )
    .await?;
    runtime.close().await;
    reopened(&home, &baseline, 0, REPLAY_MS - 1, Ok(&before_read)).await?;
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    sqlx::query("DROP TRIGGER retention_fault_target_floor")
        .execute(runtime.pool.as_ref())
        .await?;
    let expected = retained_rows(
        &original,
        &[1, 2, 3, 4],
        &[(8, 275, partial(1, 1, TINY), vec![1000])],
        &[(1, 641 * DAY_MS), (3, 640 * DAY_MS), (4, 366 * DAY_MS)],
        &[1000],
        REPLAY_MS,
    )?;
    for _ in 0..2 {
        attach(&runtime)
            .delete_recorded_thread(target.thread_id, REPLAY_MS)
            .await?;
        assert_eq!(rows(&runtime).await?, expected);
    }
    rejected(&attach(&runtime), &owned(2, 0, 9), &[], "expired replay").await?;
    let after_read = response(REPLAY_MS, 0, None, partial(0, 0, "0"));
    read_checked(
        &mut *runtime.pool.acquire().await?,
        0,
        REPLAY_MS,
        Ok(&after_read),
    )
    .await?;
    runtime.close().await;
    reopened(&home, &expected, 0, REPLAY_MS, Ok(&after_read)).await?;
    Ok(())
}
