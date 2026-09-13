//! Normal-library facade and native ownership: no private implementation inclusion.
use anyhow::Context;
use codex_protocol::ThreadId;
use codex_state::accounting::*;
use codex_state::*;
use pretty_assertions::assert_eq;
use sqlx::Connection;
use sqlx::SqliteConnection;
use uuid::Uuid;

#[path = "../src/runtime/accounting_late_import_test_support.rs"]
mod support;
use support::*;

#[tokio::test]
async fn accounting_late_import_public_native_delete_serialized_outcomes() -> anyhow::Result<()> {
    // These are deterministic serial checks, not contention evidence. Await the
    // first public operation completely before invoking the second.
    for import_first in [true, false] {
        let path = home();
        let runtime = open(&path).await?;
        let other = open(&path).await?;
        let now = chrono::Utc::now().timestamp_millis();
        let e = entry(1, now - 100 * DAY)?;
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, now).await?;
        let mut conn = connection(&runtime).await?;
        if import_first {
            assert_eq!(
                store
                    .import_retained(e.attempt.thread_id, std::slice::from_ref(&e), now)
                    .await?,
                vec![RetainedImportOutcome::Imported]
            );
            assert_eq!(
                sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_compact_days")
                    .fetch_one(&mut conn)
                    .await?,
                1
            );
        }
        assert_eq!(other.delete_thread(e.attempt.thread_id).await?, 1);
        if !import_first {
            // Use the actual completed checkpoint as the explicit test time so
            // this check reaches owner rejection, independently of wall-clock tick.
            let as_of: i64 = sqlx::query_scalar(
                "SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint",
            )
            .fetch_one(&mut conn)
            .await?;
            let before = dump(&mut conn).await?;
            let error = store
                .import_retained(e.attempt.thread_id, std::slice::from_ref(&e), as_of)
                .await
                .expect_err("deleted owner must reject import");
            assert!(format!("{error:#}").contains("missing native import owner"));
            assert_eq!(dump(&mut conn).await?, before);
        }
        assert_eq!(other.delete_thread(e.attempt.thread_id).await?, 0);
        assert!(runtime.get_thread(e.attempt.thread_id).await?.is_none());
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_compact_days")
                .fetch_one(&mut conn)
                .await?,
            0
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_price_snapshots")
                .fetch_one(&mut conn)
                .await?,
            0
        );
        conn.close().await?;
        other.close().await;
        runtime.close().await;
        for _ in 0..2 {
            let runtime = open(&path).await?;
            let mut conn = connection(&runtime).await?;
            let fences: Vec<(String, i64)> =
                sqlx::query_as("SELECT * FROM draft_accounting_tombstones ORDER BY attempt_id")
                    .fetch_all(&mut conn)
                    .await?;
            let expected = if import_first {
                vec![(
                    e.attempt.attempt_id.to_string(),
                    i64::from(e.attempt.dispatched_at_ms) + 365 * DAY,
                )]
            } else {
                vec![]
            };
            assert_eq!(fences, expected);
            assert!(runtime.get_thread(e.attempt.thread_id).await?.is_none());
            conn.close().await?;
            runtime.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_public_native_delete_contends_without_winner_assumption()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let other = open(&path).await?;
    let now = chrono::Utc::now().timestamp_millis();
    let e = entry(1, now - 100 * DAY)?;
    native(&runtime, e.attempt.thread_id).await?;
    let store = AccountingStore::open(&runtime, now).await?;
    let mut conn = connection(&runtime).await?;
    let tx = conn.begin_with("BEGIN IMMEDIATE").await?;
    let bundle = [e.clone()];
    let import = store.import_retained(e.attempt.thread_id, &bundle, now);
    let delete = other.delete_thread(e.attempt.thread_id);
    tokio::pin!(import, delete);
    let delay = std::time::Duration::from_millis(30);
    // Both actual public futures are polled while a separate connection owns
    // the writer. Neither can finish until it is released; no FIFO promise.
    assert!(tokio::time::timeout(delay, &mut import).await.is_err());
    assert!(tokio::time::timeout(delay, &mut delete).await.is_err());
    tx.commit().await?;
    let (imported, deleted) = tokio::join!(&mut import, &mut delete);
    assert_eq!(deleted?, 1);
    let expected_fences = match imported {
        Ok(outcomes) => {
            assert_eq!(outcomes, vec![RetainedImportOutcome::Imported]);
            vec![(
                e.attempt.attempt_id.to_string(),
                i64::from(e.attempt.dispatched_at_ms) + 365 * DAY,
            )]
        }
        Err(error) => {
            let text = format!("{error:#}");
            assert!(
                text.contains("backward import") || text.contains("missing native import owner"),
                "{text}"
            );
            vec![]
        }
    };
    assert!(runtime.get_thread(e.attempt.thread_id).await?.is_none());
    for table in ["compact_days", "compact_snapshots", "price_snapshots"] {
        assert_eq!(
            sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
                "SELECT count(*) FROM draft_accounting_{table}"
            )))
            .fetch_one(&mut conn)
            .await?,
            0
        );
    }
    assert_eq!(
        sqlx::query_as::<_, (String, i64)>(
            "SELECT * FROM draft_accounting_tombstones ORDER BY attempt_id"
        )
        .fetch_all(&mut conn)
        .await?,
        expected_fences
    );
    conn.close().await?;
    other.close().await;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_native_family_literal_golden_and_young_reconstruction()
-> anyhow::Result<()> {
    let direct_home = home();
    let raw_home = home();
    let mut entries = Vec::new();
    for id in 1..=4 {
        let mut e = entry(id, 0)?;
        e.attempt.thread_id = ThreadId::from_string(&Uuid::from_u128(id + 6).to_string())?;
        if id == 3 {
            e.original_price = OriginalPriceEvidence::Unpriced;
        }
        entries.push(e);
    }
    let mut young = entry(5, 100 * DAY)?;
    young.attempt.thread_id = entries[0].attempt.thread_id;
    for (path, direct) in [(&*direct_home, true), (&*raw_home, false)] {
        let runtime = open(path).await?;
        let mut conn = connection(&runtime).await?;
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM sqlite_schema WHERE name = '_accounting_migrations'"
            )
            .fetch_one(&mut conn)
            .await?,
            0
        );
        for e in &entries {
            native(&runtime, e.attempt.thread_id).await?;
        }
        for child in &entries[1..3] {
            runtime
                .upsert_thread_spawn_edge(
                    entries[0].attempt.thread_id,
                    child.attempt.thread_id,
                    DirectionalThreadSpawnEdgeStatus::Open,
                )
                .await?;
        }
        let store = AccountingStore::open(&runtime, 0).await?;
        if !direct {
            for e in &entries {
                let prices = match &e.original_price {
                    OriginalPriceEvidence::Bound(price) => std::slice::from_ref(price),
                    OriginalPriceEvidence::Unpriced => &[],
                };
                store
                    .admit(e.attempt.thread_id, &e.attempt, prices, 0)
                    .await?;
                store
                    .observe(e.attempt.thread_id, &e.attempt, &e.observations, 0)
                    .await?;
            }
        }
        store
            .admit(
                young.attempt.thread_id,
                &young.attempt,
                &[snapshot()?],
                100 * DAY,
            )
            .await?;
        store
            .observe(
                young.attempt.thread_id,
                &young.attempt,
                &young.observations,
                100 * DAY,
            )
            .await?;
        if direct {
            raw_insert_guards(&mut conn).await?;
            for e in &entries {
                store
                    .import_retained(e.attempt.thread_id, std::slice::from_ref(e), 100 * DAY)
                    .await?;
            }
        }
        for (index, e) in entries.iter().enumerate() {
            let mut expected = expected_day(true, 100 * DAY, 1)?;
            if index == 2
                && let RetainedDay::Available {
                    totals: Current::Ready(ref mut totals),
                    ..
                } = expected
            {
                totals.known_usd = Decimal::default();
            }
            assert_eq!(
                store.read_day(e.attempt.thread_id, 0, 100 * DAY).await?,
                expected
            );
        }
        let snapshots: Vec<(String, String)> =
            sqlx::query_as("SELECT * FROM draft_accounting_price_snapshots ORDER BY snapshot_id")
                .fetch_all(&mut conn)
                .await?;
        assert_eq!(
            snapshots,
            vec![(
                Uuid::from_u128(1000).to_string(),
                serde_json::to_string(&snapshot()?)?
            )]
        );
        let refs: Vec<(String, i64, String)> = sqlx::query_as("SELECT * FROM draft_accounting_compact_snapshots ORDER BY thread_id, utc_day, snapshot_id").fetch_all(&mut conn).await?;
        assert_eq!(
            refs,
            [7, 8, 10]
                .map(|id| (
                    Uuid::from_u128(id).to_string(),
                    0,
                    Uuid::from_u128(1000).to_string()
                ))
                .to_vec()
        );
        let tombstones: Vec<(String, i64)> =
            sqlx::query_as("SELECT * FROM draft_accounting_tombstones ORDER BY attempt_id")
                .fetch_all(&mut conn)
                .await?;
        assert_eq!(
            tombstones,
            (1..=4)
                .map(|id| (Uuid::from_u128(id).to_string(), 365 * DAY))
                .collect::<Vec<_>>()
        );
        let edges: Vec<(String, String, String)> = sqlx::query_as("SELECT parent_thread_id, child_thread_id, status FROM thread_spawn_edges ORDER BY child_thread_id").fetch_all(&mut conn).await?;
        assert_eq!(
            edges,
            [8, 9]
                .map(|id| (
                    Uuid::from_u128(7).to_string(),
                    Uuid::from_u128(id).to_string(),
                    "open".into()
                ))
                .to_vec()
        );
        conn.close().await?;
        runtime.close().await;
    }
    for _ in 0..2 {
        let direct = open(&direct_home).await?;
        let raw = open(&raw_home).await?;
        let mut a = connection(&direct).await?;
        let mut b = connection(&raw).await?;
        let accounting = |rows: Vec<(String, Vec<String>)>| {
            rows.into_iter()
                .filter(|(name, _)| name.starts_with("draft_accounting_"))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            accounting(dump(&mut a).await?),
            accounting(dump(&mut b).await?)
        );
        a.close().await?;
        b.close().await?;
        direct.close().await;
        raw.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_public_delete_off_recreated_owner_and_snapshot_gc()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let now = chrono::Utc::now().timestamp_millis();
    let e = entry(1, now - 100 * DAY)?;
    let mut other = entry(2, now - 100 * DAY)?;
    other.attempt.thread_id = ThreadId::new();
    for owner in [e.attempt.thread_id, other.attempt.thread_id] {
        native(&runtime, owner).await?;
    }
    {
        let store = AccountingStore::open(&runtime, now).await?;
        for entry in [&e, &other] {
            store
                .import_retained(entry.attempt.thread_id, std::slice::from_ref(entry), now)
                .await?;
        }
    }
    runtime.close().await;
    let runtime = open(&path).await?; // Collection OFF; no accounting handle needed to delete.
    assert_eq!(runtime.delete_thread(e.attempt.thread_id).await?, 1);
    assert_eq!(runtime.delete_thread(e.attempt.thread_id).await?, 0);
    let mut conn = connection(&runtime).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_price_snapshots")
            .fetch_one(&mut conn)
            .await?,
        1
    );
    let later = chrono::Utc::now().timestamp_millis();
    let store = AccountingStore::open(&runtime, later).await?;
    let before = dump(&mut conn).await?;
    assert!(
        store
            .import_retained(e.attempt.thread_id, &[e.clone()], later)
            .await
            .is_err()
    );
    assert_eq!(dump(&mut conn).await?, before);
    native(&runtime, e.attempt.thread_id).await?;
    assert_eq!(
        store
            .import_retained(e.attempt.thread_id, &[e.clone()], later)
            .await?,
        vec![RetainedImportOutcome::SuppressedByReplayRecord]
    );
    assert_eq!(
        runtime
            .delete_threads_strict(&[other.attempt.thread_id])
            .await?,
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_price_snapshots")
            .fetch_one(&mut conn)
            .await?,
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_tombstones")
            .fetch_one(&mut conn)
            .await?,
        2
    );
    conn.close().await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let store = AccountingStore::open(&runtime, chrono::Utc::now().timestamp_millis()).await?;
        assert_eq!(
            store
                .import_retained(
                    e.attempt.thread_id,
                    &[e.clone()],
                    chrono::Utc::now().timestamp_millis()
                )
                .await?,
            vec![RetainedImportOutcome::SuppressedByReplayRecord]
        );
        runtime.close().await;
    }
    Ok(())
}
