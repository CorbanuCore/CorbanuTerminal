use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[path = "accounting_retention_test_support.rs"]
mod support;
use support::*;

const TINY: &str = "0.000000000000000000000001";
const TWO: &str = "0.000000000000000000000002";
const FIVE: &str = "0.000000000000000000000005";

fn empty(as_of: i64) -> RetentionPlan {
    RetentionPlan {
        previous_as_of_ms: None,
        computed_as_of_ms: as_of,
        compact_days: BTreeMap::new(),
        expired_days: BTreeSet::new(),
        raw_removals: BTreeMap::new(),
        raw_days: BTreeMap::new(),
    }
}

fn key(day: i64) -> DayKey {
    (Uuid::from_u128(7).to_string(), day)
}

// Literal seven-population expectations; no quotation or merge implementation used.
fn partial(input: i64, attempts: i64, usd: &str) -> CompactValues {
    CompactValues::decode(
        &json!({"version":1,"known":[0,input,0,0,0,0,0],
        "unknown":[attempts,0,attempts,attempts,attempts,attempts,attempts],
        "known_usd":usd,"unknown_estimates":attempts,"attempts":attempts})
        .to_string(),
    )
    .unwrap()
}

fn day(values: CompactValues, refs: &[String]) -> CompactDay {
    CompactDay {
        values,
        snapshots: refs.iter().cloned().collect(),
    }
}

#[tokio::test]
async fn whole_store_mixed_days_shared_snapshots_and_two_reopens() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let price = snapshot();
    let a = attempt(1, DAY_MS);
    let mut b = attempt(2, DAY_MS);
    b.thread_id = ThreadId::from_string(&Uuid::from_u128(8).to_string())?;
    let fresh = attempt(3, 91 * DAY_MS);
    for a in [&a, &b, &fresh] {
        save(&store, a, &[price.clone()]).await?;
    }
    // Old stored quote is 1e-24; latest source is 3e-24 under the original binding.
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 2, 3)])
        .await?;
    let mut tx = runtime.pool.begin().await?;
    compact(&mut tx, 1, &partial(2, 1, TWO).encode()?).await?;
    compact(&mut tx, 0, &partial(1, 1, "0").encode()?).await?;
    let mut prior = price.clone();
    prior.id = Uuid::from_u128(999);
    sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?)")
        .bind(prior.id.to_string())
        .bind(serde_json::to_string(&prior)?)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO draft_accounting_compact_snapshots SELECT thread_id, utc_day, ? FROM draft_accounting_compact_days")
        .bind(price.id.to_string()).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO draft_accounting_compact_snapshots SELECT thread_id, utc_day, ? FROM draft_accounting_compact_days")
        .bind(prior.id.to_string()).execute(&mut *tx).await?;
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = 0")
        .execute(&mut *tx)
        .await?;
    let mut expected = empty(91 * DAY_MS);
    expected.previous_as_of_ms = Some(0);
    let refs = [price.id.to_string(), prior.id.to_string()];
    expected.compact_days = BTreeMap::from([
        (key(0), day(partial(1, 1, "0"), &refs)),
        (key(1), day(partial(5, 2, FIVE), &refs)),
        (
            (b.thread_id.to_string(), 1),
            day(partial(1, 1, TINY), &[price.id.to_string()]),
        ),
    ]);
    expected.raw_removals =
        BTreeMap::from([(a.attempt_id, 366 * DAY_MS), (b.attempt_id, 366 * DAY_MS)]);
    expected.raw_days.insert(
        key(91),
        Current::Ready(partial(1, 1, TINY).to_day_totals()?),
    );
    unchanged(&mut tx, expected.computed_as_of_ms, Ok(&expected)).await?;
    unchanged(&mut tx, expected.computed_as_of_ms, Ok(&expected)).await?;
    let baseline = dump(&mut tx).await?;
    tx.commit().await?;
    runtime.close().await;
    reopens(&home, baseline, &expected).await
}

#[tokio::test]
async fn exact_90_365_dispatch_zero_and_one_millisecond_day_loss() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    for a in [attempt(1, 0), attempt(2, 1), attempt(3, DAY_MS)] {
        save(&store, &a, &[]).await?;
    }
    let mut tx = runtime.pool.begin().await?;
    compact(&mut tx, 0, &partial(1, 1, "0").encode()?).await?;
    unchanged(&mut tx, 0, Err("future dispatch")).await?;
    // Time, removed raw count, surviving compact populations, raw populations.
    for (now, removed, compact0, compact1, raw0, raw1) in [
        (DETAIL_MS - 1, 0, 1, 0, 2, 1),
        (DETAIL_MS, 1, 2, 0, 1, 1),
        (DETAIL_MS + 1, 2, 3, 0, 0, 1),
        (REPLAY_MS - 1, 3, 3, 1, 0, 0),
        (REPLAY_MS, 3, 0, 1, 0, 0),
        (REPLAY_MS + DAY_MS, 3, 0, 0, 0, 0),
    ] {
        let mut expected = empty(now);
        if compact0 == 0 {
            expected.expired_days.insert(key(0));
        }
        for (bucket, count) in [(0, compact0), (1, compact1)] {
            if count > 0 {
                expected
                    .compact_days
                    .insert(key(bucket), day(partial(count, count, "0"), &[]));
            }
        }
        for (id, expiry) in [(1, REPLAY_MS), (2, REPLAY_MS + 1), (3, REPLAY_MS + DAY_MS)]
            .into_iter()
            .take(removed)
        {
            expected.raw_removals.insert(Uuid::from_u128(id), expiry);
        }
        for (bucket, count) in [(0, raw0), (1, raw1)] {
            if count > 0 {
                expected.raw_days.insert(
                    key(bucket),
                    Current::Ready(partial(count, count, "0").to_day_totals()?),
                );
            }
        }
        unchanged(&mut tx, now, Ok(&expected)).await?;
    }
    tx.rollback().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn missing_stale_null_unbound_and_original_prices_survive_reopen() -> anyhow::Result<()> {
    // Mode, raw freshness, latest noncached input, original bound amount/reference.
    for (mode, ready, input, amount, refs) in [
        ("null", true, 1, "0", vec![]),
        ("unbound", false, 1, "0", vec![]),
        ("intent", false, 0, "0", vec![]),
        (
            "bound_without_estimate",
            false,
            1,
            TINY,
            vec![snapshot().id.to_string()],
        ),
        ("stale", false, 2, TWO, vec![snapshot().id.to_string()]),
        ("current", true, 1, TINY, vec![snapshot().id.to_string()]),
    ] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = install(&runtime).await?;
        let a = attempt(1, 0);
        let price = snapshot();
        if mode == "intent" {
            store.estimates.journal.begin_attempt(&a).await?;
        } else if mode == "unbound" || mode == "bound_without_estimate" {
            store
                .estimates
                .journal
                .append_observation(&a, &[row(&a, 1, 1)])
                .await?;
        } else {
            let prices = if mode == "null" {
                vec![]
            } else {
                vec![price.clone()]
            };
            save(&store, &a, &prices).await?;
        }
        if mode == "stale" {
            store
                .estimates
                .journal
                .append_observation(&a, &[row(&a, 2, 2)])
                .await?;
        }
        let mut tx = runtime.pool.begin().await?;
        // An eligible newer catalog row must never price null or never-bound attempts.
        let mut newer = price.clone();
        newer.id = Uuid::from_u128(1002);
        newer.rates.noncached = Some(Decimal::canonical(9, 0));
        for price in [&price, &newer] {
            sqlx::query(
                "INSERT INTO draft_accounting_price_snapshots VALUES (?, ?) ON CONFLICT DO NOTHING",
            )
            .bind(price.id.to_string())
            .bind(serde_json::to_string(price)?)
            .execute(&mut *tx)
            .await?;
        }
        if mode == "bound_without_estimate" {
            sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, ?)")
                .bind(a.attempt_id.to_string())
                .bind(price.id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        let mut raw = empty(DETAIL_MS - 1);
        let freshness = if ready {
            Current::Ready(partial(input, 1, amount).to_day_totals()?)
        } else {
            Current::NeedsRefresh
        };
        raw.raw_days.insert(key(0), freshness);
        unchanged(&mut tx, raw.computed_as_of_ms, Ok(&raw)).await?;
        let mut values = partial(input, 1, amount);
        if mode == "intent" {
            values = CompactValues::decode(
                r#"{"version":1,"known":[0,0,0,0,0,0,0],"unknown":[1,1,1,1,1,1,1],"known_usd":"0","unknown_estimates":1,"attempts":1}"#,
            )?;
        }
        let mut expected = empty(DETAIL_MS);
        expected.compact_days.insert(key(0), day(values, &refs));
        expected.raw_removals.insert(a.attempt_id, REPLAY_MS);
        unchanged(&mut tx, DETAIL_MS, Ok(&expected)).await?;
        let baseline = dump(&mut tx).await?;
        tx.commit().await?;
        runtime.close().await;
        reopens(&home, baseline, &expected).await?;
    }
    Ok(())
}

#[tokio::test]
async fn checked_compact_merges_fail_without_writes() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    save(&store, &attempt(1, 0), &[snapshot()]).await?;
    for (values, error) in [
        (partial(i64::MAX, 1, "0"), "known overflow"),
        (partial(1, i64::MAX, "0"), "unknown overflow"),
        (
            partial(1, 1, "340282366920938463463374607431768211455"),
            "decimal alignment overflow",
        ),
    ] {
        let mut tx = runtime.pool.begin().await?;
        compact(&mut tx, 0, &values.encode()?).await?;
        unchanged(&mut tx, DETAIL_MS, Err(error)).await?;
        tx.rollback().await?;
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn raw_day_freshness_and_checked_sums_cover_every_member() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    for id in [1, 2] {
        save(&store, &attempt(id, 0), &[snapshot()]).await?;
    }
    let mut ready = empty(DETAIL_MS - 1);
    ready
        .raw_days
        .insert(key(0), Current::Ready(partial(2, 2, TWO).to_day_totals()?));
    let mut stale = empty(DETAIL_MS - 1);
    stale.raw_days.insert(key(0), Current::NeedsRefresh);
    for id in [1, 2] {
        let mut tx = runtime.pool.begin().await?;
        unchanged(&mut tx, DETAIL_MS - 1, Ok(&ready)).await?;
        sqlx::query("DELETE FROM draft_accounting_contributions WHERE attempt_id = ?")
            .bind(Uuid::from_u128(id).to_string())
            .execute(&mut *tx)
            .await?;
        unchanged(&mut tx, DETAIL_MS - 1, Ok(&stale)).await?;
        tx.rollback().await?;
    }
    // The first or last stale member poisons the entire day, but cannot hide overflow.
    for id in [1, 2] {
        store
            .estimates
            .journal
            .append_observation(&attempt(id, 0), &[row(&attempt(id, 0), 2, 2)])
            .await?;
        let mut tx = runtime.pool.begin().await?;
        unchanged(&mut tx, DETAIL_MS - 1, Ok(&stale)).await?;
        tx.rollback().await?;
        store
            .estimates
            .persist_current(Uuid::from_u128(id), &[])
            .await?;
        assert_eq!(
            store.refresh_current(Uuid::from_u128(id)).await?,
            Current::Ready(())
        );
    }
    store
        .estimates
        .journal
        .append_observation(&attempt(1, 0), &[row(&attempt(1, 0), 3, i64::MAX)])
        .await?;
    let mut tx = runtime.pool.begin().await?;
    unchanged(&mut tx, DETAIL_MS - 1, Err("metric overflow")).await?;
    tx.rollback().await?;
    runtime.close().await;
    Ok(())
}

async fn unchanged(
    conn: &mut SqliteConnection,
    as_of: i64,
    expected: Result<&RetentionPlan, &str>,
) -> anyhow::Result<()> {
    let before = dump(conn).await?;
    let changes: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *conn)
        .await?;
    let result = prepare_on_connection(conn, as_of).await;
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

async fn reopens(
    path: &std::path::Path,
    baseline: Vec<Vec<String>>,
    expected: &RetentionPlan,
) -> anyhow::Result<()> {
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(path.to_path_buf(), "synthetic".into()).await?;
        let mut tx = runtime.pool.begin().await?;
        assert_eq!(dump(&mut tx).await?, baseline);
        unchanged(&mut tx, expected.computed_as_of_ms, Ok(expected)).await?;
        tx.commit().await?;
        runtime.close().await;
        assert!(runtime.pool.is_closed());
    }
    Ok(())
}
