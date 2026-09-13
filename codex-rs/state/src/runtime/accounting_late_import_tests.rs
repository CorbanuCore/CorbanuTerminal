use super::*;
use crate::SqliteConfig;
use crate::ThreadMetadataBuilder;
use crate::runtime::accounting::store::*;
use pretty_assertions::assert_eq;
use sqlx::Connection;

#[path = "accounting_late_import_test_support.rs"]
mod support;
use support::*;

#[tokio::test]
async fn accounting_late_import_raw_plus_compact_overflow_and_unrelated_delete()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let e = entry(1, 0)?;
    let mut raw = entry(2, DAY - 1)?;
    raw.original_price = OriginalPriceEvidence::Unpriced;
    raw.observations[0].patch.input = Presence::Number(i64::MAX.try_into()?);
    native(&runtime, e.attempt.thread_id).await?;
    let unrelated = ThreadId::new();
    native(&runtime, unrelated).await?;
    let store = AccountingStore::open(&runtime, DAY - 1).await?;
    store
        .admit(raw.attempt.thread_id, &raw.attempt, &[], DAY - 1)
        .await?;
    store
        .observe(
            raw.attempt.thread_id,
            &raw.attempt,
            &raw.observations,
            DAY - 1,
        )
        .await?;
    let mut conn = connection(&runtime).await?;
    let before = dump(&mut conn).await?;
    let error = store
        .import_retained(e.attempt.thread_id, &[e], 90 * DAY)
        .await
        .expect_err("raw plus compact overflow");
    assert!(format!("{error:#}").contains("known overflow"));
    assert_eq!(dump(&mut conn).await?, before);
    conn.close().await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let mut conn = connection(&runtime).await?;
        assert_eq!(dump(&mut conn).await?, before);
        conn.close().await?;
        runtime.close().await;
    }
    let runtime = open(&path).await?;
    assert_eq!(runtime.delete_thread(unrelated).await?, 1);
    assert!(runtime.get_thread(raw.attempt.thread_id).await?.is_some());
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_original_snapshot_collision_and_invalid_prefix()
-> anyhow::Result<()> {
    for case in ["snapshot", "prefix", "erased-parent", "raw-parent"] {
        let path = home();
        let runtime = open(&path).await?;
        let mut e = entry(1, 0)?;
        let mut parent = entry(2, 0)?;
        parent.attempt.request_id = e.attempt.request_id;
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 0).await?;
        let mut conn = connection(&runtime).await?;
        match case {
            "snapshot" => {
                let mut price = snapshot()?;
                price.rates.noncached = Some("9".to_owned().try_into()?);
                sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?)")
                    .bind(price.id.to_string())
                    .bind(serde_json::to_string(&price)?)
                    .execute(&mut conn)
                    .await?;
            }
            "prefix" => {
                e.attempt.dialect = Dialect::Inclusive;
                e.observations[0].patch =
                    serde_json::from_value(serde_json::json!({"input":1,"read":2,"write":0}))?;
                let mut next = e.observations[0].clone();
                next.revision = 2.try_into()?;
                next.sequence = 2.try_into()?;
                next.patch.input = Presence::Number(3.try_into()?);
                e.observations.push(next);
            }
            "erased-parent" => {
                store
                    .import_retained(e.attempt.thread_id, &[parent.clone()], 100 * DAY)
                    .await?;
                e.attempt.retry_of = Some(parent.attempt.attempt_id);
            }
            "raw-parent" => {
                store
                    .admit(e.attempt.thread_id, &parent.attempt, &[], 0)
                    .await?;
                e.attempt.retry_of = Some(parent.attempt.attempt_id);
            }
            _ => unreachable!(),
        }
        let before = dump(&mut conn).await?;
        let result = store
            .import_retained(e.attempt.thread_id, &[e], 100 * DAY)
            .await;
        if case == "raw-parent" {
            assert_eq!(result?, vec![RetainedImportOutcome::Imported]);
        } else {
            assert!(result.is_err(), "{case}");
            assert_eq!(dump(&mut conn).await?, before);
        }
        conn.close().await?;
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_writer_orders_and_read_snapshot() -> anyhow::Result<()> {
    for order in ["same", "distinct", "maintenance-first", "maintenance-last"] {
        let path = home();
        let runtime = open(&path).await?;
        let other = open(&path).await?;
        let e = entry(1, 0)?;
        native(&runtime, e.attempt.thread_id).await?;
        AccountingStore::open(&runtime, 99 * DAY).await?;
        let store = AccountingStore::open(&other, 99 * DAY).await?;
        let mut conn = connection(&runtime).await?;
        let mut reader = connection(&other).await?;
        let before = dump(&mut reader).await?;
        let mut snapshot = reader.begin().await?;
        assert_eq!(dump(&mut snapshot).await?, before);
        let mut tx = conn.begin_with("BEGIN IMMEDIATE").await?;
        if order == "maintenance-first" {
            maintain_on_connection(&mut tx, 100 * DAY).await?;
        } else {
            Journal::import_retained_on_connection(
                &mut tx,
                e.attempt.thread_id,
                &[e.clone()],
                100 * DAY,
            )
            .await?;
        }
        let next = if order == "distinct" {
            entry(2, 0)?
        } else {
            e.clone()
        };
        let pending = async {
            if order == "maintenance-last" {
                store.maintain(100 * DAY).await?;
            } else {
                let expected = if order == "same" {
                    RetainedImportOutcome::SuppressedByReplayRecord
                } else {
                    RetainedImportOutcome::Imported
                };
                assert_eq!(
                    store
                        .import_retained(next.attempt.thread_id, &[next], 100 * DAY)
                        .await?,
                    vec![expected]
                );
            }
            anyhow::Ok(())
        };
        tokio::pin!(pending);
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut pending)
                .await
                .is_err()
        );
        assert_eq!(dump(&mut snapshot).await?, before);
        tx.commit().await?;
        pending.await?;
        assert_eq!(dump(&mut snapshot).await?, before);
        snapshot.commit().await?;
        assert_ne!(dump(&mut reader).await?, before);
        let attempts = if order == "distinct" { 2 } else { 1 };
        assert_eq!(
            store.read_day(e.attempt.thread_id, 0, 100 * DAY).await?,
            expected_day(true, 100 * DAY, attempts)?
        );
        reader.close().await?;
        conn.close().await?;
        other.close().await;
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_uncommitted_process_worker() -> anyhow::Result<()> {
    let Some((path, _)) = child_input() else {
        return Ok(());
    };
    let runtime = open(&path).await?;
    let mut conn = connection(&runtime).await?;
    let mut tx = conn.begin_with("BEGIN IMMEDIATE").await?;
    let e = entry(1, 0)?;
    Journal::import_retained_on_connection(&mut tx, e.attempt.thread_id, &[e], 100 * DAY).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_tombstones")
            .fetch_one(&mut *tx)
            .await?,
        1
    );
    std::fs::write(
        path.join("written"),
        b"real import body complete, transaction uncommitted",
    )?;
    std::future::pending::<()>().await;
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_process_kill_after_uncommitted_writes() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let e = entry(1, 0)?;
    native(&runtime, e.attempt.thread_id).await?;
    AccountingStore::open(&runtime, 99 * DAY).await?;
    let mut conn = connection(&runtime).await?;
    let before = dump(&mut conn).await?;
    conn.close().await?;
    runtime.close().await;
    let name = format!(
        "{}::accounting_late_import_uncommitted_process_worker",
        module_path!()
            .strip_prefix("codex_state::")
            .context("unit module prefix")?
    );
    let mut child = spawn_worker(&path, &name, "uncommitted")?;
    wait_file(&path.join("written")).await?;
    child.kill()?;
    assert!(!child.wait()?.success());
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let mut conn = connection(&runtime).await?;
        assert_eq!(dump(&mut conn).await?, before);
        conn.close().await?;
        runtime.close().await;
    }
    let runtime = open(&path).await?;
    let store = AccountingStore::open(&runtime, 99 * DAY).await?;
    assert_eq!(
        store
            .import_retained(e.attempt.thread_id, &[e], 100 * DAY)
            .await?,
        vec![RetainedImportOutcome::Imported]
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_presence_numeric_and_exact_price_goldens() -> anyhow::Result<()> {
    use serde_json::json;
    // Literal measured populations, not expectations generated by the replay reducer.
    for (dialect, patch, known, unknown, estimates) in [
        (Dialect::NativeAnthropic, json!({}), [0; 7], [1; 7], 1),
        (
            Dialect::NativeAnthropic,
            json!({"input":null}),
            [0; 7],
            [1; 7],
            1,
        ),
        (
            Dialect::NativeAnthropic,
            json!({"input":0,"read":0,"write":0,"output":0,"reasoning":0}),
            [0; 7],
            [0; 7],
            0,
        ),
        (
            Dialect::NativeAnthropic,
            json!({"input":1,"read":2,"write":3,"output":4,"reasoning":1}),
            [6, 1, 2, 3, 4, 1, 10],
            [0; 7],
            1,
        ),
        (
            Dialect::Inclusive,
            json!({"input":6,"read":2,"write":3,"output":4,"reasoning":1,"total":10}),
            [6, 1, 2, 3, 4, 1, 10],
            [0; 7],
            1,
        ),
        (
            Dialect::UnknownCompatible,
            json!({"input":6,"read":2,"write":3,"output":4,"reasoning":1,"total":10}),
            [0, 0, 2, 3, 4, 0, 0],
            [1, 1, 0, 0, 0, 1, 1],
            1,
        ),
    ] {
        let path = home();
        let runtime = open(&path).await?;
        let mut e = entry(1, 0)?;
        e.attempt.dialect = dialect;
        e.observations[0].patch = serde_json::from_value(patch)?;
        e.original_price = OriginalPriceEvidence::Unpriced;
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 100 * DAY).await?;
        store
            .import_retained(e.attempt.thread_id, &[e.clone()], 100 * DAY)
            .await?;
        assert_eq!(
            store.read_day(e.attempt.thread_id, 0, 100 * DAY).await?,
            RetainedDay::Available {
                coverage: RetentionCoverage {
                    completed_as_of_ms: 100 * DAY,
                    detail_expired_through_ms: Some(10 * DAY),
                    aggregate_day_floor: 0,
                    oldest_recorded_day: Some(0)
                },
                totals: Current::Ready(DayTotals {
                    measured: std::array::from_fn(|i| Metric {
                        known: known[i],
                        unknown: unknown[i]
                    }),
                    known_usd: Decimal::default(),
                    unknown_estimates: estimates,
                    attempts: 1,
                }),
            }
        );
        runtime.close().await;
    }
    for (rate, amount, display) in [
        (
            "0.000000000000000001",
            Decimal::canonical(1, 24),
            "0.000000",
        ),
        ("0.5", Decimal::canonical(5, 7), "0.000000"),
        ("1.5", Decimal::canonical(15, 7), "0.000002"),
    ] {
        let path = home();
        let runtime = open(&path).await?;
        let mut e = entry(1, 0)?;
        let mut price = snapshot()?;
        price.rates.noncached = Some(rate.to_owned().try_into()?);
        e.original_price = OriginalPriceEvidence::Bound(price);
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 100 * DAY).await?;
        store
            .import_retained(e.attempt.thread_id, &[e.clone()], 100 * DAY)
            .await?;
        let mut expected = expected_day(true, 100 * DAY, 1)?;
        if let RetainedDay::Available {
            totals: Current::Ready(ref mut totals),
            ..
        } = expected
        {
            totals.known_usd = amount;
        }
        assert_eq!(
            store.read_day(e.attempt.thread_id, 0, 100 * DAY).await?,
            expected
        );
        assert_eq!(
            amount.display(),
            DisplayAmount {
                text: display.into(),
                rounded: true,
                nonzero_sub_micro: amount.coefficient == 1 || rate == "0.5"
            }
        );
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_overflow_rolls_back_entire_database() -> anyhow::Result<()> {
    for case in 0..5 {
        let path = home();
        let runtime = open(&path).await?;
        let mut e = entry(1, 0)?;
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 100 * DAY).await?;
        let mut conn = connection(&runtime).await?;
        let mut totals = DayTotals {
            attempts: 1,
            ..DayTotals::default()
        };
        match case {
            0 => totals.measured[1].known = i64::MAX,
            1 => {
                totals.attempts = i64::MAX;
                totals.measured[0].unknown = i64::MAX;
            }
            2 => {
                totals.attempts = i64::MAX;
                totals.unknown_estimates = i64::MAX;
            }
            3 => {
                totals.known_usd = Decimal::canonical(u128::MAX, 6);
            }
            4 => {
                e.observations[0].patch =
                    serde_json::from_value(serde_json::json!({"input":i64::MAX,"read":1}))?;
            }
            _ => unreachable!(),
        }
        if case != 4 {
            sqlx::query("INSERT INTO draft_accounting_compact_days VALUES (?, 0, ?)")
                .bind(e.attempt.thread_id.to_string())
                .bind(CompactValues::from_day_totals(&totals)?.encode()?)
                .execute(&mut conn)
                .await?;
        }
        let before = dump(&mut conn).await?;
        assert!(
            store
                .import_retained(e.attempt.thread_id, &[e], 100 * DAY)
                .await
                .is_err()
        );
        assert_eq!(dump(&mut conn).await?, before);
        conn.close().await?;
        runtime.close().await;
        for _ in 0..2 {
            let runtime = open(&path).await?;
            let mut conn = connection(&runtime).await?;
            assert_eq!(dump(&mut conn).await?, before);
            conn.close().await?;
            runtime.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_raw_authority_and_input_bounds_are_not_bypassed()
-> anyhow::Result<()> {
    for case in 0..7 {
        let path = home();
        let runtime = open(&path).await?;
        let raw = entry(1, 0)?;
        native(&runtime, raw.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 0).await?;
        store
            .admit(raw.attempt.thread_id, &raw.attempt, &[], 0)
            .await?;
        store
            .observe(raw.attempt.thread_id, &raw.attempt, &raw.observations, 0)
            .await?;
        let mut e = entry(2, 0)?;
        let mut bundle = Vec::new();
        match case {
            0 => e = raw.clone(),
            1 => {
                e.attempt.request_id = raw.attempt.request_id;
                e.attempt.turn = "wrong".into();
            }
            2 => e.observations = raw.observations.clone(),
            3 => e.observations = vec![e.observations[0].clone(); 257],
            4 => bundle = vec![e.clone(); 65],
            5 => {
                e.observations = vec![e.observations[0].clone(); 256];
                bundle = vec![e.clone(); 17];
            }
            6 => e.attempt.thread_id = ThreadId::new(),
            _ => unreachable!(),
        }
        if bundle.is_empty() {
            bundle.push(e);
        }
        let mut conn = connection(&runtime).await?;
        let before = dump(&mut conn).await?;
        assert!(
            store
                .import_retained(raw.attempt.thread_id, &bundle, 100 * DAY)
                .await
                .is_err()
        );
        assert!(
            store
                .import_retained(raw.attempt.thread_id, &[], 100 * DAY)
                .await
                .is_err()
        );
        assert_eq!(dump(&mut conn).await?, before);
        conn.close().await?;
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_sql_faults_full_rollback_reopen_retry() -> anyhow::Result<()> {
    for (event, table, marker, seeded) in [
        ("INSERT", "price_snapshots", "LATE_SNAPSHOT_INSERT", false),
        ("INSERT", "compact_days", "LATE_DAY_INSERT", false),
        ("UPDATE", "compact_days", "LATE_DAY_UPDATE", true),
        (
            "INSERT",
            "compact_snapshots",
            "LATE_REFERENCE_INSERT",
            false,
        ),
        ("INSERT", "tombstones", "LATE_TOMBSTONE_INSERT", false),
        (
            "UPDATE",
            "retention_checkpoint",
            "LATE_CHECKPOINT_UPDATE",
            false,
        ),
    ] {
        let path = home();
        let runtime = open(&path).await?;
        let e = entry(1, 0)?;
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 99 * DAY).await?;
        if seeded {
            store
                .import_retained(e.attempt.thread_id, &[entry(2, 0)?], 99 * DAY)
                .await?;
        }
        let mut conn = connection(&runtime).await?;
        let condition = if seeded {
            "WHEN json_extract(NEW.payload, '$.attempts') > 1"
        } else {
            ""
        };
        let trigger = format!(
            "CREATE TRIGGER late_fault BEFORE {event} ON draft_accounting_{table} {condition} BEGIN SELECT RAISE(ABORT, '{marker}'); END"
        );
        sqlx::raw_sql(sqlx::AssertSqlSafe(trigger))
            .execute(&mut conn)
            .await?;
        let before = dump(&mut conn).await?;
        let error = store
            .import_retained(e.attempt.thread_id, &[e.clone()], 100 * DAY)
            .await
            .expect_err("injected SQL fault");
        assert!(format!("{error:#}").contains(marker), "{error:#}");
        assert_eq!(dump(&mut conn).await?, before);
        conn.close().await?;
        runtime.close().await;
        for _ in 0..2 {
            let runtime = open(&path).await?;
            let mut conn = connection(&runtime).await?;
            assert_eq!(dump(&mut conn).await?, before);
            conn.close().await?;
            runtime.close().await;
        }
        let runtime = open(&path).await?;
        let mut conn = connection(&runtime).await?;
        sqlx::raw_sql("DROP TRIGGER late_fault")
            .execute(&mut conn)
            .await?;
        let store = AccountingStore::open(&runtime, 99 * DAY).await?;
        assert_eq!(
            store
                .import_retained(e.attempt.thread_id, &[e], 100 * DAY)
                .await?,
            vec![RetainedImportOutcome::Imported]
        );
        conn.close().await?;
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_deferred_commit_failure_rolls_back() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let e = entry(1, 0)?;
    native(&runtime, e.attempt.thread_id).await?;
    AccountingStore::open(&runtime, 99 * DAY).await?;
    let mut conn = connection(&runtime).await?;
    sqlx::raw_sql("CREATE TABLE late_deferred (id TEXT REFERENCES threads(id) DEFERRABLE INITIALLY DEFERRED); CREATE TRIGGER late_commit AFTER INSERT ON draft_accounting_tombstones BEGIN INSERT INTO late_deferred VALUES ('LATE_COMMIT_FK'); END").execute(&mut conn).await?;
    let before = dump(&mut conn).await?;
    let store = AccountingStore::open(&runtime, 99 * DAY).await?;
    let error = store
        .import_retained(e.attempt.thread_id, &[e.clone()], 100 * DAY)
        .await
        .expect_err("commit FK failure");
    assert!(format!("{error:#}").contains("FOREIGN KEY"));
    assert_eq!(dump(&mut conn).await?, before);
    conn.close().await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let mut conn = connection(&runtime).await?;
        assert_eq!(dump(&mut conn).await?, before);
        conn.close().await?;
        runtime.close().await;
    }
    let runtime = open(&path).await?;
    let mut conn = connection(&runtime).await?;
    sqlx::raw_sql("DROP TRIGGER late_commit")
        .execute(&mut conn)
        .await?;
    let store = AccountingStore::open(&runtime, 99 * DAY).await?;
    store
        .import_retained(e.attempt.thread_id, &[e], 100 * DAY)
        .await?;
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_boundaries_and_no_raw_staging() -> anyhow::Result<()> {
    for (time, as_of, expected) in [
        (0, 90 * DAY - 1, None),
        (0, 90 * DAY, Some(RetainedImportOutcome::Imported)),
        (0, 100 * DAY, Some(RetainedImportOutcome::Imported)),
        (0, 364 * DAY, Some(RetainedImportOutcome::Imported)),
        (0, 365 * DAY, None),
        (
            1,
            365 * DAY,
            Some(RetainedImportOutcome::ExpiredAggregateDay),
        ),
        (1, 365 * DAY + 1, None),
        (0, -1, None),
        (DAY, 0, None),
        (i64::MAX - 1, i64::MAX, None),
    ] {
        let path = home();
        let runtime = open(&path).await?;
        let e = entry(1, time)?;
        native(&runtime, e.attempt.thread_id).await?;
        let store = AccountingStore::open(&runtime, 0).await?;
        let mut conn = connection(&runtime).await?;
        raw_insert_guards(&mut conn).await?;
        let before = dump(&mut conn).await?;
        let result = store
            .import_retained(e.attempt.thread_id, &[e.clone()], as_of)
            .await;
        match expected {
            None => {
                assert!(result.is_err(), "time={time} as_of={as_of}");
                assert_eq!(dump(&mut conn).await?, before);
            }
            Some(outcome) => {
                assert_eq!(result?, vec![outcome]);
                let tombstones: Vec<(String, i64)> =
                    sqlx::query_as("SELECT * FROM draft_accounting_tombstones")
                        .fetch_all(&mut conn)
                        .await?;
                assert_eq!(
                    tombstones,
                    vec![(e.attempt.attempt_id.to_string(), time + 365 * DAY)]
                );
                if outcome == RetainedImportOutcome::ExpiredAggregateDay {
                    assert_eq!(
                        sqlx::query_scalar::<_, i64>(
                            "SELECT count(*) FROM draft_accounting_price_snapshots"
                        )
                        .fetch_one(&mut conn)
                        .await?,
                        0
                    );
                    assert_eq!(
                        sqlx::query_scalar::<_, i64>(
                            "SELECT count(*) FROM draft_accounting_compact_days"
                        )
                        .fetch_one(&mut conn)
                        .await?,
                        0
                    );
                    store.maintain(as_of + 1).await?;
                    assert!(
                        store
                            .import_retained(e.attempt.thread_id, &[e], as_of + 1)
                            .await
                            .is_err()
                    );
                } else {
                    assert_eq!(
                        store.read_day(e.attempt.thread_id, 0, as_of).await?,
                        expected_day(true, as_of, 1)?
                    );
                }
            }
        }
        conn.close().await?;
        runtime.close().await;
    }
    assert!(day_key(Uuid::from_u128(7).to_string(), i64::MAX, i64::MAX).is_err());
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_replay_price_and_two_reopens() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let e = entry(1, 0)?;
    native(&runtime, e.attempt.thread_id).await?;
    let store = AccountingStore::open(&runtime, 100 * DAY).await?;
    assert_eq!(
        store
            .import_retained(e.attempt.thread_id, &[e.clone(), e.clone()], 100 * DAY)
            .await?,
        vec![RetainedImportOutcome::Imported; 2]
    );
    let mut conn = connection(&runtime).await?;
    let before = dump(&mut conn).await?;
    conn.close().await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let store = AccountingStore::open(&runtime, 100 * DAY).await?;
        let mut altered = e.clone();
        altered.original_price = OriginalPriceEvidence::Unpriced;
        altered.attempt.retry_of = Some(Uuid::from_u128(999));
        altered.observations.clear();
        assert_eq!(
            store
                .import_retained(e.attempt.thread_id, &[altered], 101 * DAY)
                .await?,
            vec![RetainedImportOutcome::SuppressedByReplayRecord]
        );
        assert_eq!(
            store.read_day(e.attempt.thread_id, 0, 100 * DAY).await?,
            expected_day(true, 100 * DAY, 1)?
        );
        let mut conn = connection(&runtime).await?;
        assert_eq!(dump(&mut conn).await?, before);
        assert!(
            store
                .import_retained(e.attempt.thread_id, &[e.clone()], 99 * DAY)
                .await
                .is_err()
        );
        conn.close().await?;
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_invalid_identity_revision_retry_and_prices() -> anyhow::Result<()> {
    for case in 0..15 {
        let path = home();
        let runtime = open(&path).await?;
        let mut e = entry(1, 0)?;
        native(&runtime, e.attempt.thread_id).await?;
        let mut second = entry(2, 0)?;
        match case {
            0 => e.attempt.retry_of = Some(e.attempt.attempt_id),
            1 => e.attempt.retry_of = Some(Uuid::from_u128(999)),
            2 => {
                e.attempt.retry_of = Some(second.attempt.attempt_id);
                second.attempt.retry_of = Some(e.attempt.attempt_id);
                second.attempt.request_id = e.attempt.request_id;
            }
            3 => {
                second.attempt.request_id = e.attempt.request_id;
                second.attempt.turn = "other".into();
            }
            4 => second.attempt.thread_id = ThreadId::new(),
            5 => second.attempt.attempt_id = e.attempt.attempt_id,
            6 => second.observations = e.observations.clone(),
            7 => {
                let mut bad = e.observations[0].clone();
                bad.patch.input = Presence::Null;
                e.observations.push(bad);
            }
            8 => e.observations[0].revision = 0.try_into()?,
            9..=14 => {
                let OriginalPriceEvidence::Bound(ref mut price) = e.original_price else {
                    unreachable!()
                };
                match case {
                    9 => price.model = "other".into(),
                    10 => price.effective_from_ms = 1.try_into()?,
                    11 => price.observed_at_ms = 1.try_into()?,
                    12 => price.effective_end_ms = Some(0.try_into()?),
                    13 => price.approved_at_ms = 1.try_into()?,
                    14 => price.rates.noncached = Some("2".to_owned().try_into()?),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        let error = reject_unchanged(&runtime, &[e, second], 100 * DAY).await?;
        assert!(!error.is_empty(), "case {case}");
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_retry_chain_normalizes_reordered_duplicate_revisions()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let first = entry(1, 0)?;
    let mut retry = entry(2, 0)?;
    retry.attempt.request_id = first.attempt.request_id;
    retry.attempt.retry_of = Some(first.attempt.attempt_id);
    let mut revision = retry.observations[0].clone();
    revision.revision = 2.try_into()?;
    revision.sequence = 2.try_into()?;
    revision.patch.input = Presence::Null;
    retry.observations = vec![revision.clone(), retry.observations[0].clone(), revision];
    native(&runtime, first.attempt.thread_id).await?;
    let store = AccountingStore::open(&runtime, 100 * DAY).await?;
    assert_eq!(
        store
            .import_retained(
                first.attempt.thread_id,
                &[retry.clone(), first.clone()],
                100 * DAY
            )
            .await?,
        vec![RetainedImportOutcome::Imported; 2]
    );
    assert_eq!(
        store
            .read_day(first.attempt.thread_id, 0, 100 * DAY)
            .await?,
        expected_day(true, 100 * DAY, 2)?
    );
    assert_eq!(
        store
            .import_retained(first.attempt.thread_id, &[first, retry], 100 * DAY)
            .await?,
        vec![RetainedImportOutcome::SuppressedByReplayRecord; 2]
    );
    runtime.close().await;
    Ok(())
}
