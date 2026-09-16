//! These tests link the normal library, never include private implementation/fixture DDL.
use anyhow::Context;
use codex_protocol::ThreadId;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::*;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;
use serde_json::json;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

const DAY: i64 = 86_400_000;

fn ready(result: InspectionDay) -> Inspection {
    let InspectionDay::Ready(view) = result else {
        panic!("{result:?}")
    };
    view
}

#[tokio::test]
async fn accounting_inspect_public_reopens_twice() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[snapshot()?], 0).await?;
    store
        .observe(a.thread_id, &a, &[observation(&a, 1, 4)?], 0)
        .await?;
    let expected = AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        assert_eq!(
            AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?,
            expected
        );
        runtime.close().await;
    }
    Ok(())
}

// Read every table/column, including schemas and checkpoint, without fixture DDL.
async fn inspection_tables(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Vec<String>>> {
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
            .fetch_all(&mut *conn)
            .await?;
    let mut rows = vec![
        sqlx::query_scalar("SELECT json_array(name, sql) FROM sqlite_schema ORDER BY name")
            .fetch_all(&mut *conn)
            .await?,
    ];
    for table in tables {
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                .bind(&table)
                .fetch_all(&mut *conn)
                .await?;
        let columns = columns
            .iter()
            .map(|c| format!("quote(\"{}\")", c.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(",");
        let table = table.replace('"', "\"\"");
        rows.push(
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "SELECT json_array({columns}) FROM \"{table}\" ORDER BY 1"
            )))
            .fetch_all(&mut *conn)
            .await?,
        );
    }
    Ok(rows)
}

#[tokio::test]
async fn accounting_inspect_public_reads_do_not_write() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    let mut conn = connection(&runtime).await?;
    let before = inspection_tables(&mut conn).await?;
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?,
        InspectionDay::Absent
    );
    assert_eq!(inspection_tables(&mut conn).await?, before);
    AccountingStore::open(&runtime, 0)
        .await?
        .admit(a.thread_id, &a, &[], 0)
        .await?;
    let before = inspection_tables(&mut conn).await?;
    assert_eq!(
        ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 1).await?)
            .totals
            .attempts,
        1
    );
    assert!(
        AccountingStore::inspect_day(&runtime, a.thread_id, -1, 0)
            .await
            .is_err()
    );
    // Drop a polled read at its first asynchronous database boundary.
    let mut read = Box::pin(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0));
    std::future::poll_fn(|cx| {
        assert!(std::future::Future::poll(read.as_mut(), cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(read);
    assert_eq!(inspection_tables(&mut conn).await?, before);
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_public_identity_retry_scope() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, DAY - 1)?;
    let mut retry = attempt(2, DAY)?;
    retry.request_id = a.request_id;
    retry.retry_of = Some(a.attempt_id);
    let separate = attempt(3, DAY)?;
    let mut other = attempt(4, DAY)?;
    other.thread_id = ThreadId::new();
    for owner in [a.thread_id, other.thread_id] {
        native(&runtime, owner).await?;
    }
    let store = AccountingStore::open(&runtime, DAY - 1).await?;
    for a in [&a, &retry, &separate, &other] {
        store
            .admit(a.thread_id, a, &[], i64::from(a.dispatched_at_ms))
            .await?;
    }
    let mut retry2 = retry.clone();
    retry2.attempt_id = Uuid::from_u128(5);
    retry2.retry_of = Some(retry.attempt_id);
    store.admit(a.thread_id, &retry2, &[], DAY).await?;
    let view = ready(AccountingStore::inspect_day(&runtime, a.thread_id, 1, DAY).await?);
    let members: std::collections::BTreeMap<_, Vec<_>> = view
        .requests
        .iter()
        .map(|(id, rows)| (*id, rows.iter().map(|q| q.attempt.attempt_id).collect()))
        .collect();
    assert_eq!(
        members,
        std::collections::BTreeMap::from([
            (a.request_id, vec![retry.attempt_id, retry2.attempt_id]),
            (separate.request_id, vec![separate.attempt_id]),
        ])
    );
    assert_eq!(view.totals.attempts, 3);
    assert_eq!(
        ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, DAY).await?)
            .totals
            .attempts,
        1
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_public_literal_partial_goldens() -> anyhow::Result<()> {
    for (priced, patch, expected_cost, unknown, noncached) in [
        (
            true,
            json!({"input":100,"read":20,"output":40}),
            "0.00018",
            1,
            None,
        ),
        (
            true,
            json!({"input":100,"read":20,"write":0,"output":40}),
            "0.00026",
            0,
            Some(80),
        ),
        (
            false,
            json!({"input":0,"read":0,"write":0,"output":0}),
            "0",
            0,
            Some(0),
        ),
        (
            false,
            json!({"input":100,"read":20,"write":0,"output":40}),
            "0",
            1,
            Some(80),
        ),
    ] {
        let path = home();
        let runtime = open(&path).await?;
        let mut a = attempt(1, 0)?;
        a.dialect = Dialect::Inclusive;
        native(&runtime, a.thread_id).await?;
        let mut price = snapshot()?;
        price.rates =
            serde_json::from_value(json!({"noncached":"1","read":"1","write":"2","output":"4"}))?;
        let store = AccountingStore::open(&runtime, 0).await?;
        store
            .admit(
                a.thread_id,
                &a,
                if priced {
                    std::slice::from_ref(&price)
                } else {
                    &[]
                },
                0,
            )
            .await?;
        let mut o = observation(&a, 1, 0)?;
        o.patch = serde_json::from_value(patch)?;
        store.observe(a.thread_id, &a, &[o], 0).await?;
        let view = ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?);
        assert_eq!(
            serde_json::to_value(view.totals.known_usd)?,
            json!(expected_cost)
        );
        assert_eq!(view.totals.unknown_estimates, unknown);
        let quote = &view.requests[&a.request_id][0];
        assert_eq!(quote.usage.noncached, noncached);
        assert_eq!(quote.snapshot.is_some(), priced);
        assert_eq!(quote.all_buckets_priced.is_none(), unknown == 1);
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_public_delete_and_empty() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let now = chrono::Utc::now().timestamp_millis();
    let a = attempt(1, now)?;
    native(&runtime, a.thread_id).await?;
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, now / DAY, now).await?,
        InspectionDay::Absent
    );
    let store = AccountingStore::open(&runtime, now).await?;
    assert_eq!(
        ready(AccountingStore::inspect_day(&runtime, a.thread_id, now / DAY, now).await?)
            .totals
            .attempts,
        0
    );
    let mut other = attempt(2, now)?;
    other.thread_id = ThreadId::new();
    native(&runtime, other.thread_id).await?;
    for a in [&a, &other] {
        store.admit(a.thread_id, a, &[], now).await?;
    }
    let other_before =
        AccountingStore::inspect_day(&runtime, other.thread_id, now / DAY, now).await?;
    runtime.delete_thread(a.thread_id).await?;
    let read_at = chrono::Utc::now().timestamp_millis();
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, now / DAY, read_at).await?,
        InspectionDay::MissingThread
    );
    let mut other_after =
        ready(AccountingStore::inspect_day(&runtime, other.thread_id, now / DAY, read_at).await?);
    let before = ready(other_before);
    other_after.read_at_ms = before.read_at_ms;
    // Deletion can advance the store checkpoint; retained evidence must remain identical.
    assert_eq!(
        (other_after.owner, other_after.totals, other_after.requests),
        (before.owner, before.totals, before.requests)
    );
    native(&runtime, a.thread_id).await?;
    assert_eq!(
        ready(AccountingStore::inspect_day(&runtime, a.thread_id, now / DAY, read_at).await?)
            .totals
            .attempts,
        0
    );
    runtime.close().await;
    Ok(())
}

// Populate valid unrelated history without invoking maintenance for every clone.
async fn unrelated_population(runtime: &StateRuntime, count: u128) -> anyhow::Result<()> {
    let store = AccountingStore::open(runtime, 0).await?;
    let owner = attempt(1, 0)?;
    let quote = store.admit(owner.thread_id, &owner, &[], 0).await?;
    let threads: Vec<_> = (0..600)
        .map(|id| ThreadId::from_string(&Uuid::from_u128(100_000 + id).to_string()))
        .collect::<Result<_, _>>()?;
    for thread in &threads {
        native(runtime, *thread).await?;
    }
    let mut conn = connection(runtime).await?;
    sqlx::query("BEGIN").execute(&mut conn).await?;
    for id in 2..=count {
        let mut a = attempt(id, 0)?;
        a.thread_id = threads[(id as usize - 2) % threads.len()];
        let mut q = quote.clone();
        q.attempt = a.clone();
        sqlx::query("INSERT INTO draft_accounting_attempts VALUES (?, ?, ?)")
            .bind(a.attempt_id.to_string())
            .bind(a.request_id.to_string())
            .bind(serde_json::to_string(&a)?)
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, NULL)")
            .bind(a.attempt_id.to_string())
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, '[]', ?)")
            .bind(a.attempt_id.to_string())
            .bind(serde_json::to_string(&q)?)
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_contributions VALUES (?, ?, 0, '[]')")
            .bind(a.attempt_id.to_string())
            .bind(a.thread_id.to_string())
            .execute(&mut conn)
            .await?;
    }
    sqlx::query("COMMIT").execute(&mut conn).await?;
    conn.close().await?;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_small_day_on_busy_host() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    unrelated_population(&runtime, 12_001).await?;
    let view = ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?);
    assert_eq!(
        view.totals,
        DayTotals::from_quotes(view.requests.values().flatten())?
    );
    assert_eq!(
        (
            view.totals.attempts,
            view.unknown_parent_totals.attempts,
            view.totals.unknown_estimates
        ),
        (1, 0, 1)
    );
    assert_eq!(view.totals.known_usd, "0".to_owned().try_into()?);
    assert!(
        view.totals
            .measured
            .iter()
            .all(|metric| metric.known == 0 && metric.unknown == 1)
    );
    let range = AccountingStore::inspect_range(
        &runtime,
        a.thread_id,
        InspectionRange {
            start_ms: 0,
            end_ms: DAY,
            grouping: InspectionGrouping::Day,
        },
        0,
    )
    .await?;
    let InspectionDay::Range { buckets, .. } = range else {
        panic!("{range:?}")
    };
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].days.len(), 1);
    let InspectionDay::Ready(bucket_view) = &buckets[0].days[0] else {
        panic!("{buckets:?}")
    };
    assert_eq!(bucket_view, &view);
    // The unrelated attempt's observation cap must not become the owner's cap.
    let mut other = view.requests[&a.request_id][0].clone();
    other.attempt = attempt(2, 0)?;
    other.attempt.thread_id = ThreadId::from_string(&Uuid::from_u128(100_000).to_string())?;
    other.observations = (1..=4097)
        .map(|revision| {
            let mut row = observation(&other.attempt, revision, 0)?;
            row.patch = Patch::default();
            Ok(row)
        })
        .collect::<anyhow::Result<_>>()?;
    let mut conn = connection(&runtime).await?;
    sqlx::query("BEGIN").execute(&mut conn).await?;
    for row in &other.observations {
        sqlx::query("INSERT INTO draft_accounting_observations VALUES (?, ?, ?, ?, ?)")
            .bind(other.attempt.attempt_id.to_string())
            .bind(i64::from(row.revision))
            .bind(row.source.to_string())
            .bind(i64::from(row.sequence))
            .bind(serde_json::to_string(row)?)
            .execute(&mut conn)
            .await?;
    }
    let evidence = serde_json::to_string(&other.observations)?;
    sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, ?, ?)")
        .bind(other.attempt.attempt_id.to_string())
        .bind(&evidence)
        .bind(serde_json::to_string(&other)?)
        .execute(&mut conn)
        .await?;
    sqlx::query("UPDATE draft_accounting_contributions SET evidence = ? WHERE attempt_id = ?")
        .bind(evidence)
        .bind(other.attempt.attempt_id.to_string())
        .execute(&mut conn)
        .await?;
    sqlx::query("COMMIT").execute(&mut conn).await?;
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?,
        InspectionDay::Ready(view)
    );
    assert_eq!(
        AccountingStore::inspect_day(&runtime, other.attempt.thread_id, 0, 0).await?,
        InspectionDay::TooLarge
    );
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_valid_ten_thousand_retained_rows() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    unrelated_population(&runtime, 10_000).await?;
    let mut conn = connection(&runtime).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_attempts")
            .fetch_one(&mut conn)
            .await?,
        10_000
    );
    let view = ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?);
    assert_eq!(
        (
            view.totals.attempts,
            view.requests.len(),
            view.unknown_parent_totals.attempts
        ),
        (1, 1, 0)
    );
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_maximal_valid_packet() -> anyhow::Result<()> {
    const CEILING: usize = 4 * 1024 * 1024;
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[], 0).await?;
    let template = store
        .observe(a.thread_id, &a, &[observation(&a, 1, 1)?], 0)
        .await?;
    let mut quotes = Vec::new();
    let mut packet = 8192;
    loop {
        let mut q = template.clone();
        q.attempt = attempt(quotes.len() as u128 + 1, 0)?;
        q.observations.clear();
        let mut size = serde_json::to_vec(&q)?.len() + 2048;
        for revision in 1..=4096 {
            let row = observation(&q.attempt, revision, 1)?;
            let extra = serde_json::to_vec(&row)?.len() + usize::from(revision > 1);
            if packet + size + extra > CEILING - 1 {
                break;
            }
            size += extra;
            q.observations.push(row);
        }
        assert!(!q.observations.is_empty());
        packet += size;
        if q.observations.len() < 4096 {
            // Fill the final sub-observation remainder with legal identity metadata.
            let mut remaining = CEILING - 1 - packet;
            for text in [&mut q.attempt.model, &mut q.attempt.provider] {
                let padding = remaining.min(128 - text.len());
                text.push_str(&"x".repeat(padding));
                remaining -= padding;
                packet += padding;
            }
            assert_eq!(remaining, 0);
            quotes.push(q);
            break;
        }
        quotes.push(q);
    }
    assert_eq!(packet, CEILING - 1);
    assert_eq!(
        8192 + quotes
            .iter()
            .map(|q| serde_json::to_vec(q).unwrap().len() + 2048)
            .sum::<usize>(),
        packet
    );
    let mut conn = connection(&runtime).await?;
    sqlx::query("BEGIN").execute(&mut conn).await?;
    sqlx::raw_sql(
        "DELETE FROM draft_accounting_contributions; DELETE FROM draft_accounting_estimates;
        DELETE FROM draft_accounting_observations; DELETE FROM draft_accounting_price_bindings;
        DELETE FROM draft_accounting_attempts",
    )
    .execute(&mut conn)
    .await?;
    for q in &quotes {
        let a = &q.attempt;
        let evidence = serde_json::to_string(&q.observations)?;
        sqlx::query("INSERT INTO draft_accounting_attempts VALUES (?, ?, ?)")
            .bind(a.attempt_id.to_string())
            .bind(a.request_id.to_string())
            .bind(serde_json::to_string(a)?)
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, NULL)")
            .bind(a.attempt_id.to_string())
            .execute(&mut conn)
            .await?;
        for row in &q.observations {
            sqlx::query("INSERT INTO draft_accounting_observations VALUES (?, ?, ?, ?, ?)")
                .bind(a.attempt_id.to_string())
                .bind(i64::from(row.revision))
                .bind(row.source.to_string())
                .bind(i64::from(row.sequence))
                .bind(serde_json::to_string(row)?)
                .execute(&mut conn)
                .await?;
        }
        sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, ?, ?)")
            .bind(a.attempt_id.to_string())
            .bind(&evidence)
            .bind(serde_json::to_string(q)?)
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_contributions VALUES (?, ?, 0, ?)")
            .bind(a.attempt_id.to_string())
            .bind(a.thread_id.to_string())
            .bind(evidence)
            .execute(&mut conn)
            .await?;
    }
    // Valid historical versions together exceed 4 MiB but do not enlarge the packet.
    let mut history_bytes = 0;
    for omitted in 1..=16 {
        let mut old = quotes[0].clone();
        old.observations.truncate(old.observations.len() - omitted);
        let evidence = serde_json::to_string(&old.observations)?;
        history_bytes += evidence.len();
        sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, ?, ?)")
            .bind(old.attempt.attempt_id.to_string())
            .bind(evidence)
            .bind(serde_json::to_string(&old)?)
            .execute(&mut conn)
            .await?;
    }
    assert!(history_bytes > CEILING);
    sqlx::query("COMMIT").execute(&mut conn).await?;
    let view = ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?);
    assert_eq!(view.totals, DayTotals::from_quotes(&quotes)?);
    assert_eq!(
        view.requests
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<_>>(),
        quotes
    );
    // Two more legal bytes cross the same projected packet ceiling.
    let q = quotes.last_mut().unwrap();
    q.attempt.turn.push_str("xx");
    sqlx::query("UPDATE draft_accounting_attempts SET payload = ? WHERE attempt_id = ?")
        .bind(serde_json::to_string(&q.attempt)?)
        .bind(q.attempt.attempt_id.to_string())
        .execute(&mut conn)
        .await?;
    sqlx::query("UPDATE draft_accounting_estimates SET payload = ? WHERE attempt_id = ?")
        .bind(serde_json::to_string(q)?)
        .bind(q.attempt.attempt_id.to_string())
        .execute(&mut conn)
        .await?;
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?,
        InspectionDay::TooLarge
    );
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_public_limits_no_truncation() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[], 0).await?;
    // Clone legitimately admitted, all-unknown intent and its original NULL evidence.
    let mut conn = connection(&runtime).await?;
    let quote = ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?).requests
        [&a.request_id][0]
        .clone();
    for id in 2..=513 {
        let a = attempt(id, 0)?;
        let mut q = quote.clone();
        q.attempt = a.clone();
        sqlx::query("INSERT INTO draft_accounting_attempts VALUES (?, ?, ?)")
            .bind(a.attempt_id.to_string())
            .bind(a.request_id.to_string())
            .bind(serde_json::to_string(&a)?)
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, NULL)")
            .bind(a.attempt_id.to_string())
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_estimates VALUES (?, '[]', ?)")
            .bind(a.attempt_id.to_string())
            .bind(serde_json::to_string(&q)?)
            .execute(&mut conn)
            .await?;
        sqlx::query("INSERT INTO draft_accounting_contributions VALUES (?, ?, 0, '[]')")
            .bind(a.attempt_id.to_string())
            .bind(a.thread_id.to_string())
            .execute(&mut conn)
            .await?;
        if id == 512 {
            assert_eq!(
                ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?)
                    .totals
                    .attempts,
                512
            );
        }
    }
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?,
        InspectionDay::TooLarge
    );
    conn.close().await?;
    runtime.close().await;
    inspection_limit_edges().await?;
    Ok(())
}

async fn inspection_limit_edges() -> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    native(&runtime, a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[], 0).await?;
    let observations: Vec<_> = (1..=4096)
        .map(|r| observation(&a, r, r))
        .collect::<anyhow::Result<_>>()?;
    let mut conn = connection(&runtime).await?;
    for batch in observations.chunks(256) {
        store.observe(a.thread_id, &a, batch, 0).await?;
        // This fixture retains one estimate version to isolate the observation cap.
        sqlx::query("DELETE FROM draft_accounting_estimates WHERE evidence NOT IN (SELECT evidence FROM draft_accounting_contributions)")
            .execute(&mut conn).await?;
    }
    conn.close().await?;
    assert_eq!(
        ready(AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?).requests
            [&a.request_id][0]
            .observations
            .len(),
        4096
    );
    store
        .observe(a.thread_id, &a, &[observation(&a, 4097, 4097)?], 0)
        .await?;
    assert_eq!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await?,
        InspectionDay::TooLarge
    );
    runtime.close().await;
    let path = home();
    let runtime = open(&path).await?;
    native(&runtime, a.thread_id).await?;
    AccountingStore::open(&runtime, 0).await?;
    let mut conn = connection(&runtime).await?;
    // Deliberately invalid small rows prove the cap precedes materialization.
    sqlx::raw_sql("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<10000)
        INSERT INTO draft_accounting_attempts SELECT CAST(x AS TEXT), CAST(x AS TEXT), 'null' FROM n")
        .execute(&mut conn).await?;
    assert!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0)
            .await
            .is_err()
    );
    sqlx::raw_sql("INSERT INTO draft_accounting_attempts VALUES ('extra','extra','null')")
        .execute(&mut conn)
        .await?;
    // Malformed ownership remains an error at any unrelated population size.
    assert!(
        AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0)
            .await
            .is_err()
    );
    sqlx::raw_sql("DELETE FROM draft_accounting_attempts")
        .execute(&mut conn)
        .await?;
    for size in [4 * 1024 * 1024, 4 * 1024 * 1024 + 1] {
        let payload = format!("null{}", " ".repeat(size - 4));
        sqlx::query("INSERT OR REPLACE INTO draft_accounting_attempts VALUES ('bad','bad',?)")
            .bind(payload)
            .execute(&mut conn)
            .await?;
        let result = AccountingStore::inspect_day(&runtime, a.thread_id, 0, 0).await;
        assert!(result.is_err());
    }
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

fn home() -> impl std::ops::Deref<Target = std::path::PathBuf> {
    scopeguard::guard(
        std::env::temp_dir().join(format!("accounting-store-{}", Uuid::new_v4())),
        |path| {
            let _ = std::fs::remove_dir_all(path);
        },
    )
}

async fn open(path: &Path) -> anyhow::Result<Arc<StateRuntime>> {
    StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(path.to_path_buf())?),
        "synthetic".into(),
    )
    .await
}

async fn connection(runtime: &StateRuntime) -> anyhow::Result<SqliteConnection> {
    let pool = runtime
        .sqlite()
        .open_read_write_pool(&runtime.sqlite().state_db_path())
        .await?;
    // Detach ownership before closing the pool; callers explicitly close the connection.
    let mut conn = pool.acquire().await?.detach();
    pool.close().await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
            .fetch_one(&mut conn)
            .await?,
        1
    );
    Ok(conn)
}

fn attempt(id: u128, time: i64) -> anyhow::Result<Attempt> {
    Ok(Attempt {
        attempt_id: Uuid::from_u128(id),
        request_id: Uuid::from_u128(id + 100),
        thread_id: ThreadId::from_string(&Uuid::from_u128(7).to_string())?,
        turn: "fixture".into(),
        retry_of: None,
        provider: "synthetic".into(),
        model: "fixture".into(),
        scope: Uuid::nil(),
        dialect: Dialect::NativeAnthropic,
        dispatched_at_ms: time.try_into()?,
    })
}

fn snapshot() -> anyhow::Result<Snapshot> {
    serde_json::from_value(json!({
        "id":Uuid::from_u128(1000), "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"0.000000000000000001","read":null,"write":null,"output":null},
        "source_reference":Uuid::from_u128(1001), "source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null
    }))
    .context("synthetic original-price snapshot")
}

fn observation(a: &Attempt, revision: i64, input: i64) -> anyhow::Result<Observation> {
    Ok(Observation {
        revision: revision.try_into()?,
        source: a.attempt_id,
        sequence: revision.try_into()?,
        patch: Patch {
            input: Presence::Number(input.try_into()?),
            ..Patch::default()
        },
    })
}

async fn native(runtime: &StateRuntime, owner: ThreadId) -> anyhow::Result<()> {
    let path = runtime.sqlite().home();
    let mut builder = ThreadMetadataBuilder::new(
        owner,
        path.join("synthetic.jsonl"),
        chrono::DateTime::from_timestamp(1_700_000_000, 0).context("synthetic native timestamp")?,
        SessionSource::Cli,
    );
    builder.cwd = path.to_path_buf();
    runtime.upsert_thread(&builder.build("synthetic")).await
}

async fn ordinary(conn: &mut SqliteConnection) -> anyhow::Result<Vec<(i64, Vec<u8>)>> {
    Ok(
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations ORDER BY version")
            .fetch_all(conn)
            .await?,
    )
}

#[tokio::test]
async fn normal_default_profiles_do_not_install_and_opt_in_keeps_ordinary_history()
-> anyhow::Result<()> {
    let path = home();
    let a = attempt(1, 0)?;
    let runtime = open(&path).await?;
    let mut conn = connection(&runtime).await?;
    let history = ordinary(&mut conn).await?;
    for populated in [false, true] {
        if populated {
            native(&runtime, a.thread_id).await?;
        }
        assert_eq!(sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM sqlite_schema WHERE name GLOB 'draft_accounting_*' OR name = '_accounting_migrations'",
        ).fetch_one(&mut conn).await?, 0);
    }
    let native_before = runtime.get_thread(a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    assert_eq!(runtime.get_thread(a.thread_id).await?, native_before);
    assert_eq!(ordinary(&mut conn).await?, history);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM sqlite_schema WHERE name GLOB 'draft_accounting_*'",
        )
        .fetch_one(&mut conn)
        .await?,
        12
    );
    assert_eq!(
        sqlx::query_as::<_, (i64, bool)>("SELECT version, success FROM _accounting_migrations",)
            .fetch_all(&mut conn)
            .await?,
        vec![(1, true)]
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
            .fetch_one(&mut conn)
            .await?,
        "ok"
    );
    let intent = store.admit(a.thread_id, &a, &[snapshot()?], 0).await?;
    assert_eq!(intent.usage, Usage::default());
    assert_eq!(intent.all_buckets_priced, None);
    assert_eq!(intent.snapshot, Some(snapshot()?));
    let totals = store.read_day(a.thread_id, 0, 0).await?;
    assert_eq!(
        totals,
        RetainedDay::Available {
            coverage: RetentionCoverage {
                completed_as_of_ms: 0,
                detail_expired_through_ms: None,
                aggregate_day_floor: 0,
                oldest_recorded_day: Some(0),
            },
            totals: Current::Ready(DayTotals {
                measured: std::array::from_fn(|_| Metric {
                    known: 0,
                    unknown: 1
                }),
                known_usd: Decimal::default(),
                unknown_estimates: 1,
                attempts: 1,
            }),
        }
    );
    conn.close().await?;
    runtime.close().await;
    for _ in 0..2 {
        let reopened = open(&path).await?;
        let store = AccountingStore::open(&reopened, 0).await?;
        assert_eq!(store.read_day(a.thread_id, 0, 0).await?, totals);
        let mut conn = connection(&reopened).await?;
        assert_eq!(ordinary(&mut conn).await?, history);
        conn.close().await?;
        reopened.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn original_price_null_unknown_zero_and_reordered_replay_survive_restart()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let a = attempt(1, 0)?;
    let missing = attempt(2, 0)?;
    native(&runtime, a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[snapshot()?], 0).await?;
    store.admit(missing.thread_id, &missing, &[], 0).await?;
    let patch = observation(&a, 2, 1)?;
    let quote = store.observe(a.thread_id, &a, &[patch.clone()], 0).await?;
    assert_eq!(
        serde_json::to_value(quote.known_subtotal)?,
        json!("0.000000000000000000000001")
    );
    assert_eq!(
        quote.usage,
        Usage {
            noncached: Some(1),
            ..Usage::default()
        }
    );
    assert_eq!(quote.all_buckets_priced, None);
    store
        .observe(a.thread_id, &a, &[observation(&a, 1, 0)?], 0)
        .await?;
    let expected = store.observe(a.thread_id, &a, &[patch], 0).await?;
    let null = store
        .admit(missing.thread_id, &missing, &[snapshot()?], 0)
        .await?;
    assert_eq!(null.snapshot, None);
    let mut zero = observation(&missing, 1, 0)?;
    zero.patch = serde_json::from_value(
        json!({"input":0,"read":0,"write":0,"output":0,"reasoning":0,"total":0}),
    )?;
    let zero_quote = store
        .observe(missing.thread_id, &missing, &[zero.clone()], 0)
        .await?;
    assert_eq!(zero_quote.all_buckets_priced, Some(Decimal::default()));
    assert_eq!(zero_quote.snapshot, None);
    let expected_day = store.read_day(a.thread_id, 0, 0).await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let store = AccountingStore::open(&runtime, 0).await?;
        let mut later = snapshot()?;
        later.id = Uuid::from_u128(2000);
        later.rates.noncached = Some("99".to_owned().try_into()?);
        assert_eq!(
            store.admit(a.thread_id, &a, &[later.clone()], 0).await?,
            expected
        );
        assert_eq!(
            store
                .admit(missing.thread_id, &missing, &[later], 0)
                .await?,
            zero_quote
        );
        assert_eq!(
            store
                .observe(missing.thread_id, &missing, &[zero.clone()], 0)
                .await?,
            zero_quote
        );
        assert_eq!(store.read_day(a.thread_id, 0, 0).await?, expected_day);
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn installed_deletion_without_store_handle_removes_raw_and_compact_in_normal_library()
-> anyhow::Result<()> {
    let path = home();
    let runtime = open(&path).await?;
    let now = chrono::Utc::now().timestamp_millis();
    let a = attempt(1, now - 90 * DAY)?;
    let mut other = attempt(2, now - 1)?;
    other.thread_id = ThreadId::new();
    for owner in [a.thread_id, other.thread_id] {
        native(&runtime, owner).await?;
    }
    {
        let store = AccountingStore::open(&runtime, i64::from(a.dispatched_at_ms)).await?;
        store
            .admit(
                a.thread_id,
                &a,
                &[snapshot()?],
                i64::from(a.dispatched_at_ms),
            )
            .await?;
        store
            .observe(
                a.thread_id,
                &a,
                &[observation(&a, 1, 1)?],
                i64::from(a.dispatched_at_ms),
            )
            .await?;
        store.maintain(now - 1).await?;
        store
            .admit(other.thread_id, &other, &[snapshot()?], now - 1)
            .await?;
    }
    runtime.close().await;
    let runtime = open(&path).await?; // Collection OFF: never open an accounting handle here.
    assert_eq!(runtime.delete_thread(a.thread_id).await?, 1);
    assert_eq!(runtime.delete_threads_strict(&[other.thread_id]).await?, 1);
    assert_eq!(runtime.delete_thread(a.thread_id).await?, 0);
    assert_eq!(runtime.delete_threads_strict(&[]).await?, 0);
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        let mut conn = connection(&runtime).await?;
        for table in [
            "draft_accounting_attempts",
            "draft_accounting_price_snapshots",
            "draft_accounting_price_bindings",
            "draft_accounting_estimates",
            "draft_accounting_contributions",
            "draft_accounting_observations",
            "draft_accounting_compact_days",
            "draft_accounting_compact_snapshots",
        ] {
            assert_eq!(
                sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
                    "SELECT count(*) FROM {table}"
                )))
                .fetch_one(&mut conn)
                .await?,
                0
            );
        }
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM draft_accounting_tombstones")
                .fetch_one(&mut conn)
                .await?,
            2
        );
        assert!(runtime.get_thread(a.thread_id).await?.is_none());
        assert!(runtime.get_thread(other.thread_id).await?.is_none());
        conn.close().await?;
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn normal_open_rejects_unversioned_partial_failed_checksum_and_newer_without_repair()
-> anyhow::Result<()> {
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
        AccountingStore::open(&runtime, 0).await?;
        let mut conn = connection(&runtime).await?;
        sqlx::raw_sql(mutation).execute(&mut conn).await?;
        let before: Vec<(String, Option<String>)> =
            sqlx::query_as("SELECT name, sql FROM sqlite_schema ORDER BY name")
                .fetch_all(&mut conn)
                .await?;
        let history = ordinary(&mut conn).await?;
        for _ in 0..2 {
            assert!(
                AccountingStore::open(&runtime, 1).await.is_err(),
                "{mutation}"
            );
            assert_eq!(
                sqlx::query_as::<_, (String, Option<String>)>(
                    "SELECT name, sql FROM sqlite_schema ORDER BY name",
                )
                .fetch_all(&mut conn)
                .await?,
                before
            );
            assert_eq!(ordinary(&mut conn).await?, history);
        }
        conn.close().await?;
        runtime.close().await;
        for _ in 0..2 {
            let runtime = open(&path).await?; // Ordinary migrator ignores independent ledger.
            assert!(
                AccountingStore::open(&runtime, 1).await.is_err(),
                "{mutation}"
            );
            runtime.close().await;
        }
    }
    Ok(())
}
