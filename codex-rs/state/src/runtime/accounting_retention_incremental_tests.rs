use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[path = "accounting_retention_test_support.rs"]
mod support;
use support::*;
#[path = "accounting_retention_atomic_test_support.rs"]
mod atomic_support;
use atomic_support::*;

fn owned(id: u128, dispatch: i64, owner: u128) -> Attempt {
    let mut a = attempt(id, dispatch);
    a.thread_id = ThreadId::from_string(&Uuid::from_u128(owner).to_string()).unwrap();
    a
}

/// Two threads over several days: shared and absent prices, a stale
/// contribution, and compact days that precede the raw detail.
async fn ledger(runtime: &StateRuntime) -> anyhow::Result<Lifecycle<'_>> {
    let store = install(runtime).await?;
    for (a, prices) in [
        (attempt(/*id*/ 1, /*dispatch*/ 0), vec![snapshot()]),
        (attempt(/*id*/ 2, /*dispatch*/ 1), vec![snapshot()]),
        (owned(/*id*/ 3, DAY_MS, /*owner*/ 8), vec![snapshot()]),
        (attempt(/*id*/ 4, 2 * DAY_MS + 5), vec![]),
        (owned(/*id*/ 5, DAY_MS + 7, /*owner*/ 8), vec![snapshot()]),
        (attempt(/*id*/ 6, 3 * DAY_MS), vec![snapshot()]),
        (attempt(/*id*/ 7, 3 * DAY_MS + 1), vec![snapshot()]),
    ] {
        save(&store, &a, &prices).await?;
    }
    // Newer evidence than the recorded contribution.
    let stale = attempt(/*id*/ 6, 3 * DAY_MS);
    store
        .estimates
        .journal
        .append_observation(&stale, &[row(&stale, /*revision*/ 2, /*input*/ 5)])
        .await?;
    let mut conn = runtime.pool.acquire().await?;
    let partial = values(
        [0, 1, 0, 0, 0, 0, 0],
        [1, 0, 1, 1, 1, 1, 1],
        "0.5",
        /*missing*/ 1,
        /*count*/ 1,
    );
    compact(&mut conn, /*day*/ 0, &partial.encode()?).await?;
    compact(&mut conn, /*day*/ 2, &partial.encode()?).await?;
    drop(conn);
    store.maintain_retention(3 * DAY_MS + 1).await?;
    Ok(store)
}

async fn batch(runtime: &StateRuntime, time: i64, commit: bool) -> anyhow::Result<bool> {
    let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
    let more = expire_batch_on_connection(&mut tx, time, /*limit*/ 1).await?;
    if commit {
        tx.commit().await?;
    } else {
        // A crash before commit: SQLite keeps none of the batch.
        drop(tx);
    }
    Ok(more)
}

/// Retiring expiries a record at a time, across a crash before a commit and a
/// reopen, leaves exactly the rows the full sweep leaves at every boundary.
#[tokio::test]
async fn batches_leave_exactly_what_the_full_sweep_leaves() -> anyhow::Result<()> {
    let (full_home, home) = (support::home(), support::home());
    let full = StateRuntime::init_for_testing(full_home.to_path_buf(), "synthetic".into()).await?;
    let full_store = ledger(&full).await?;
    let mut runtime =
        StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    ledger(&runtime).await?;
    assert_eq!(rows(&runtime).await?, rows(&full).await?);
    let mut restarts = 0;
    for time in [
        DETAIL_MS - 1,
        DETAIL_MS,
        DETAIL_MS + 1,
        DETAIL_MS + DAY_MS + 7,
        DETAIL_MS + 3 * DAY_MS + 1,
        REPLAY_MS,
        REPLAY_MS + DAY_MS,
        REPLAY_MS + 2 * DAY_MS + 5,
        REPLAY_MS + 3 * DAY_MS + 1,
        REPLAY_MS + 4 * DAY_MS,
    ] {
        full_store.maintain_retention(time).await?;
        let before = rows(&runtime).await?;
        batch(&runtime, time, /*commit*/ false).await?;
        assert_eq!(
            rows(&runtime).await?,
            before,
            "an uncommitted batch left nothing"
        );
        while batch(&runtime, time, /*commit*/ true).await? {
            restarts += 1;
            // A restart between batches carries on where the last one stopped.
            runtime.close().await;
            runtime =
                StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        }
        let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        maintain_for_write_on_connection(&mut tx, time, Some(time)).await?;
        tx.commit().await?;
        assert_eq!(rows(&runtime).await?, rows(&full).await?, "at {time}");
    }
    assert!(restarts >= 5, "{restarts}");
    runtime.close().await;
    full.close().await;
    Ok(())
}

/// Each record a batch retires is verified as the full sweep verifies it, and
/// a failing batch keeps nothing.
#[tokio::test]
async fn a_batch_fails_visibly_on_a_corrupt_record_and_keeps_nothing() -> anyhow::Result<()> {
    let home = support::home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    ledger(&runtime).await?;
    let before = rows(&runtime).await?;
    for (sql, marker) in [
        (
            "UPDATE draft_accounting_estimates SET payload = '{}' WHERE attempt_id = ?",
            "corrupt estimate payload",
        ),
        (
            "UPDATE draft_accounting_contributions SET utc_day = 9 WHERE attempt_id = ?",
            "contribution attribution mismatch",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES (?, 1)",
            "raw/tombstone collision",
        ),
    ] {
        let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query(sql)
            .bind(Uuid::from_u128(1).to_string())
            .execute(&mut *tx)
            .await?;
        assert_error(
            expire_batch_on_connection(&mut tx, DETAIL_MS + 1, /*limit*/ 32)
                .await
                .unwrap_err(),
            marker,
        );
        tx.rollback().await?;
        assert_eq!(rows(&runtime).await?, before);
    }
    // Expired compact days and tombstones are checked before they go, as the
    // full sweep checks them.
    for (sql, marker) in [
        (
            "UPDATE draft_accounting_compact_days SET payload = '[]' WHERE utc_day = 2",
            "expected compact object",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES ('not-an-attempt', 0)",
            "invalid attempt UUID",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES ('00000000-0000-0000-0000-000000000063', 1)",
            "invalid replay expiry",
        ),
    ] {
        let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query(sql).execute(&mut *tx).await?;
        assert_error(
            expire_batch_on_connection(&mut tx, REPLAY_MS + 4 * DAY_MS, /*limit*/ 32)
                .await
                .unwrap_err(),
            marker,
        );
        tx.rollback().await?;
        assert_eq!(rows(&runtime).await?, before);
    }
    // Nothing applies before the checkpoint or to an inactive ledger.
    let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
    assert_error(
        expire_batch_on_connection(&mut tx, 3 * DAY_MS, /*limit*/ 32)
            .await
            .unwrap_err(),
        "backward retention expiry",
    );
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET admission_active = 0")
        .execute(&mut *tx)
        .await?;
    assert_error(
        expire_batch_on_connection(&mut tx, DETAIL_MS + 1, /*limit*/ 32)
            .await
            .unwrap_err(),
        "requires active retention",
    );
    tx.rollback().await?;
    assert_eq!(rows(&runtime).await?, before);
    runtime.close().await;
    Ok(())
}

/// The write's own transaction retires one more batch. What is left over aged
/// out only this hour, so the checkpoint still moves and reads report the lag
/// until a later write retires the rest.
#[tokio::test]
async fn the_write_leaves_this_hours_leftovers_as_checkpoint_lag() -> anyhow::Result<()> {
    const HOUR: i64 = 3_600_000;
    let home = support::home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let count = EXPIRY_BATCH + 8;
    for id in 1..=count {
        save(&store, &attempt(id as u128, HOUR + id), &[snapshot()]).await?;
    }
    store.maintain_retention(2 * HOUR).await?;
    // Every attempt is due, each only after this hour began.
    let now = DETAIL_MS + 2 * HOUR - 1;
    let raw = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_attempts")
            .fetch_one(runtime.pool.as_ref())
            .await
    };
    let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
    maintain_for_write_on_connection(&mut tx, now, Some(now)).await?;
    tx.commit().await?;
    let left = raw().await?;
    assert!((8..count).contains(&left), "{left}");
    let mut conn = runtime.pool.acquire().await?;
    assert_eq!(
        read_retained_on_connection(
            &mut conn,
            attempt(/*id*/ 1, /*dispatch*/ 0).thread_id,
            /*day*/ 0,
            now
        )
        .await?,
        RetainedDay::NeedsMaintenance {
            completed_as_of_ms: now
        }
    );
    drop(conn);
    while raw().await? > 0 {
        let mut tx = runtime.pool.begin_with("BEGIN IMMEDIATE").await?;
        maintain_for_write_on_connection(&mut tx, now + 1, Some(now)).await?;
        tx.commit().await?;
    }
    let mut conn = runtime.pool.acquire().await?;
    assert!(matches!(
        read_retained_on_connection(
            &mut conn,
            attempt(/*id*/ 1, /*dispatch*/ 0).thread_id,
            /*day*/ 0,
            now + 1
        )
        .await?,
        RetainedDay::Available { .. }
    ));
    drop(conn);
    runtime.close().await;
    Ok(())
}

async fn rows(runtime: &StateRuntime) -> anyhow::Result<Vec<Vec<String>>> {
    dump(&mut *runtime.pool.acquire().await?).await
}
