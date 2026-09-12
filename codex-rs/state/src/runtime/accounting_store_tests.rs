use super::*;
use crate::runtime::test_support::test_thread_metadata;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

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
        error.as_database_error().and_then(|e| e.code()).as_deref(),
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
        sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
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
                })
                .connect_with(options.as_ref().clone())
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
            sql.as_database_error().and_then(|e| e.code()).as_deref(),
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
            reopens(&path, &success).await
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
