use super::*;
use serde_json::json;
use sqlx::Connection;

#[path = "accounting_retention_test_support.rs"]
mod support;
use support::*;

fn key(day: i64) -> DayKey {
    (Uuid::from_u128(7).to_string(), day)
}

// Literal populations, never a reduction of the reader's returned quotes.
fn partial(input: i64, attempts: i64, usd: &str) -> CompactValues {
    CompactValues::decode(
        &json!({"version":1,"known":[0,input,0,0,0,0,0],
        "unknown":[attempts,0,attempts,attempts,attempts,attempts,attempts],
        "known_usd":usd,"unknown_estimates":attempts,"attempts":attempts})
        .to_string(),
    )
    .unwrap()
}

fn expected_raw(
    a: &Attempt,
    observations: Vec<Observation>,
    contribution: ContributionFreshness,
) -> RawInput {
    let (selected, noncached, amount, bucket) = match a.attempt_id.as_u128() {
        1 => (
            Some(snapshot()),
            Some(3),
            Decimal::canonical(3, 24),
            BucketQuote::Priced(Decimal::canonical(3, 24)),
        ),
        2 => (None, None, Decimal::default(), BucketQuote::MissingUsage),
        3 => (None, Some(1), Decimal::default(), BucketQuote::MissingRate),
        _ => unreachable!("literal fixture"),
    };
    RawInput {
        quote: ObservationQuote {
            attempt: a.clone(),
            observations,
            usage: Usage {
                noncached,
                ..Default::default()
            },
            snapshot: selected,
            buckets: [
                bucket,
                BucketQuote::MissingUsage,
                BucketQuote::MissingUsage,
                BucketQuote::MissingUsage,
            ],
            known_subtotal: amount,
            all_buckets_priced: None,
            subtotal_display: DisplayAmount {
                text: "0.000000".into(),
                rounded: a.attempt_id.as_u128() == 1,
                nonzero_sub_micro: a.attempt_id.as_u128() == 1,
            },
        },
        day: key(0),
        detail_expires_at_ms: DETAIL_MS,
        replay_expires_at_ms: REPLAY_MS,
        day_expires_at_ms: REPLAY_MS,
        contribution,
    }
}

#[tokio::test]
async fn complete_input_preserves_expired_rows_bindings_freshness_and_two_reopens()
-> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let (a, intent, null) = (attempt(1, 0), attempt(2, 0), attempt(3, 0));
    save(&store, &a, &[snapshot()]).await?;
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 2, 3)])
        .await?;
    store.estimates.journal.begin_attempt(&intent).await?;
    save(&store, &null, &[]).await?;
    let mut tx = runtime.pool.begin().await?;
    for day in [0, 365] {
        compact(&mut tx, day, &partial(1, 1, "0").encode()?).await?;
    }
    sqlx::query("INSERT INTO draft_accounting_compact_snapshots SELECT thread_id, utc_day, snapshot_id FROM draft_accounting_compact_days CROSS JOIN draft_accounting_price_snapshots").execute(&mut *tx).await?;
    sqlx::query("INSERT INTO draft_accounting_tombstones VALUES (?, ?)")
        .bind(Uuid::from_u128(99).to_string())
        .bind(REPLAY_MS)
        .execute(&mut *tx)
        .await?;
    // Eligible alternative cannot reprice the old binding, explicit NULL or unbound intent.
    let mut alternative = snapshot();
    alternative.id = Uuid::from_u128(1002);
    alternative.rates.noncached = Some(Decimal::canonical(9, 0));
    sqlx::query("INSERT INTO draft_accounting_price_snapshots VALUES (?, ?)")
        .bind(alternative.id.to_string())
        .bind(serde_json::to_string(&alternative)?)
        .execute(&mut *tx)
        .await?;
    let mut expected = ValidatedRetentionInput {
        previous_as_of_ms: None,
        as_of_ms: REPLAY_MS,
        compact_days: [(key(0), REPLAY_MS), (key(365), 730 * DAY_MS)]
            .into_iter()
            .map(|(key, expires_at_ms)| {
                (
                    key,
                    CompactInput {
                        values: partial(1, 1, "0"),
                        snapshots: BTreeSet::from([snapshot().id.to_string()]),
                        expires_at_ms,
                    },
                )
            })
            .collect(),
        raw_attempts: [
            (
                &a,
                vec![row(&a, 1, 1), row(&a, 2, 3)],
                ContributionFreshness::Stale,
            ),
            (&intent, vec![], ContributionFreshness::Missing),
            (
                &null,
                vec![row(&null, 1, 1)],
                ContributionFreshness::Current,
            ),
        ]
        .into_iter()
        .map(|(a, rows, freshness)| (a.attempt_id, expected_raw(a, rows, freshness)))
        .collect(),
        tombstones: BTreeMap::from([(Uuid::from_u128(99), REPLAY_MS)]),
    };
    unchanged(&mut tx, REPLAY_MS, Ok(&expected)).await?;
    let baseline = dump(&mut tx).await?;
    tx.commit().await?;
    let mut tx = runtime.pool.begin().await?;
    sqlx::query("UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = 0")
        .execute(&mut *tx)
        .await?;
    expected.previous_as_of_ms = Some(0);
    unchanged(&mut tx, REPLAY_MS, Ok(&expected)).await?;
    tx.rollback().await?;
    expected.previous_as_of_ms = None;
    runtime.close().await;
    reopens(&home, baseline, &expected).await
}

#[tokio::test]
async fn whole_store_corruption_including_expired_and_orphan_rows_never_writes()
-> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    let a = attempt(1, 0);
    save(&store, &a, &[snapshot()]).await?;
    store
        .estimates
        .journal
        .append_observation(&a, &[row(&a, 2, 2)])
        .await?;
    let mut conn = runtime.pool.acquire().await?;
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await?;
    compact(&mut conn, 0, &partial(1, 1, "0").encode()?).await?;
    sqlx::query("INSERT INTO draft_accounting_compact_snapshots SELECT thread_id, utc_day, snapshot_id FROM draft_accounting_compact_days CROSS JOIN draft_accounting_price_snapshots").execute(&mut *conn).await?;
    for (sql, error) in [
        (
            "DELETE FROM draft_accounting_retention_checkpoint",
            "checkpoints",
        ),
        (
            "INSERT INTO draft_accounting_retention_checkpoint VALUES (2, 0, 0)",
            "checkpoints",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET singleton = 2",
            "checkpoint key",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = -1",
            "checkpoint",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = 999999999999",
            "checkpoint",
        ),
        (
            "UPDATE draft_accounting_retention_checkpoint SET completed_as_of_ms = 1.5",
            "mismatched types",
        ),
        (
            "UPDATE draft_accounting_compact_days SET payload = '{}'",
            "missing field",
        ),
        (
            "UPDATE draft_accounting_compact_days SET payload = json_set(payload, '$.known_usd', '01')",
            "noncanonical stored amount",
        ),
        (
            "UPDATE draft_accounting_compact_days SET payload = json_set(payload, '$.unknown[0]', 2)",
            "invalid unknown count",
        ),
        (
            "UPDATE draft_accounting_compact_days SET payload = json_set(payload, '$.known_usd', '340282366920938463463374607431768211456')",
            "stored amount coefficient overflow",
        ),
        (
            "UPDATE draft_accounting_compact_days SET thread_id = 'forged'",
            "orphan fixture reference",
        ),
        (
            "DELETE FROM draft_accounting_compact_days",
            "orphan fixture reference",
        ),
        (
            "INSERT INTO draft_accounting_compact_days SELECT 'forged', 1, payload FROM draft_accounting_compact_days",
            "invalid thread key",
        ),
        (
            "INSERT INTO draft_accounting_compact_days SELECT '00000000000000000000000000000007', 1, payload FROM draft_accounting_compact_days",
            "noncanonical thread key",
        ),
        (
            "UPDATE draft_accounting_compact_snapshots SET snapshot_id = 'foreign'",
            "orphan fixture reference",
        ),
        (
            "UPDATE draft_accounting_contributions SET thread_id = 'foreign'",
            "attribution mismatch",
        ),
        (
            "UPDATE draft_accounting_contributions SET attempt_id = 'orphan'",
            "orphan fixture reference",
        ),
        (
            "UPDATE draft_accounting_contributions SET utc_day = 0.5",
            "mismatched types",
        ),
        (
            "UPDATE draft_accounting_estimates SET payload = '{}'",
            "corrupt estimate payload",
        ),
        (
            "UPDATE draft_accounting_estimates SET evidence = '['",
            "orphan fixture reference",
        ),
        (
            "UPDATE draft_accounting_attempts SET request_id = 'forged'",
            "identity mismatch",
        ),
        (
            "UPDATE draft_accounting_observations SET source = 'forged'",
            "position mismatch",
        ),
        (
            "UPDATE draft_accounting_observations SET attempt_id = 'orphan'",
            "orphan fixture reference",
        ),
        (
            "DELETE FROM draft_accounting_price_bindings",
            "orphan fixture reference",
        ),
        (
            "UPDATE draft_accounting_price_bindings SET snapshot_id = NULL",
            "corrupt estimate payload",
        ),
        (
            "DELETE FROM draft_accounting_price_snapshots",
            "orphan fixture reference",
        ),
        (
            "UPDATE draft_accounting_price_snapshots SET payload = ' ' || payload",
            "noncanonical snapshot",
        ),
        (
            "INSERT INTO draft_accounting_tombstones SELECT attempt_id, 31536000000 FROM draft_accounting_attempts",
            "raw/tombstone collision",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES ('forged', 31536000000)",
            "UUID",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES ('00000000-0000-0000-0000-000000000099', 0)",
            "invalid replay expiry",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES ('00000000000000000000000000000099', 31536000000)",
            "noncanonical attempt key",
        ),
        (
            "INSERT INTO draft_accounting_tombstones VALUES ('00000000-0000-0000-0000-000000000099', 94608000000)",
            "invalid replay expiry",
        ),
        (
            "UPDATE draft_accounting_price_snapshots SET payload = json_set(payload, '$.observed_at_ms', 31536000001, '$.effective_from_ms', 31536000001)",
            "future snapshot evidence",
        ),
    ] {
        let mut tx = conn.begin().await?;
        sqlx::query(sql).execute(&mut *tx).await?;
        unchanged(&mut tx, REPLAY_MS, Err(error)).await?;
        tx.rollback().await?;
    }
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await?;
    drop(conn);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn checked_times_keys_and_payloads_fail_without_writes() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = install(&runtime).await?;
    save(&store, &attempt(1, 0), &[snapshot()]).await?;
    let mut tx = runtime.pool.begin().await?;
    unchanged(&mut tx, -1, Err("negative as-of")).await?;
    tx.rollback().await?;
    for (bucket, payload, now, error) in [
        (-1, partial(1, 1, "0").encode()?, DETAIL_MS, "negative day"),
        (
            i64::MAX,
            partial(1, 1, "0").encode()?,
            i64::MAX,
            "day start overflow",
        ),
        (
            i64::MAX / DAY_MS,
            partial(1, 1, "0").encode()?,
            i64::MAX,
            "day expiry overflow",
        ),
        (91, partial(1, 1, "0").encode()?, DETAIL_MS, "future day"),
    ] {
        let mut tx = runtime.pool.begin().await?;
        compact(&mut tx, bucket, &payload).await?;
        unchanged(&mut tx, now, Err(error)).await?;
        tx.rollback().await?;
    }
    for (dispatch, as_of, error) in [
        (-1, 0, "negative count or position"),
        (1, 0, "future dispatch"),
        (i64::MAX, i64::MAX, "detail expiry overflow"),
        (i64::MAX - DETAIL_MS, i64::MAX, "replay expiry overflow"),
    ] {
        let mut tx = runtime.pool.begin().await?;
        sqlx::query("UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.dispatched_at_ms', ?)").bind(dispatch).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM draft_accounting_contributions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM draft_accounting_estimates")
            .execute(&mut *tx)
            .await?;
        unchanged(&mut tx, as_of, Err(error)).await?;
        tx.rollback().await?;
    }
    runtime.close().await;
    Ok(())
}
