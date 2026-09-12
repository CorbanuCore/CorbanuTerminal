use super::*;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use sqlx::Connection;

fn attempt() -> Attempt {
    serde_json::from_value(json!({
        "attempt_id":"00000000-0000-0000-0000-000000000001",
        "request_id":"00000000-0000-0000-0000-000000000002",
        "thread_id":"00000000-0000-0000-0000-000000000003",
        "turn":"fixture", "retry_of":null, "provider":"synthetic",
        "model":"fixture-model", "scope":"00000000-0000-0000-0000-000000000004",
        "dialect":"NativeAnthropic", "dispatched_at_ms":100
    }))
    .unwrap()
}

fn snapshot(rate: &str) -> Snapshot {
    serde_json::from_value(json!({
        "id":"00000000-0000-0000-0000-000000000010",
        "provider":"synthetic", "model":"fixture-model",
        "scope":"00000000-0000-0000-0000-000000000004",
        "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":rate,"read":"0.3","write":null,"output":null},
        "source_reference":"00000000-0000-0000-0000-000000000011",
        "source_kind":"ProviderPublished", "observed_at_ms":80,
        "approved_at_ms":90,"effective_from_ms":100,"effective_end_ms":200
    }))
    .unwrap()
}

fn row(revision: i64, patch: Value) -> Observation {
    serde_json::from_value(json!({"revision":revision,
        "source":"00000000-0000-0000-0000-000000000005",
        "sequence":revision * 10,"patch":patch}))
    .unwrap()
}

fn home() -> impl std::ops::Deref<Target = std::path::PathBuf> {
    scopeguard::guard(
        super::super::super::super::test_support::unique_temp_dir(),
        |path| {
            let _ = std::fs::remove_dir_all(path);
        },
    )
}

// Every column of all five fixture tables, including SQL NULL and exact payload bytes.
async fn dump(conn: &mut SqliteConnection) -> anyhow::Result<Vec<Vec<String>>> {
    let mut tables = Vec::new();
    for query in [
        "SELECT json_array(attempt_id, request_id, payload) FROM draft_accounting_attempts ORDER BY attempt_id",
        "SELECT json_array(attempt_id, revision, source, sequence, payload) FROM draft_accounting_observations ORDER BY attempt_id, revision",
        "SELECT json_array(snapshot_id, payload) FROM draft_accounting_price_snapshots ORDER BY snapshot_id",
        "SELECT json_array(attempt_id, snapshot_id) FROM draft_accounting_price_bindings ORDER BY attempt_id",
        "SELECT json_array(attempt_id, evidence, payload) FROM draft_accounting_estimates ORDER BY attempt_id, evidence",
    ] {
        tables.push(sqlx::query_scalar(query).fetch_all(&mut *conn).await?);
    }
    Ok(tables)
}

async fn unchanged_quote(conn: &mut SqliteConnection, id: Uuid) -> anyhow::Result<Value> {
    let before = dump(conn).await?;
    let changes: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *conn)
        .await?;
    let quote = EstimateStore::latest_quote_on_connection(conn, id).await?;
    assert_eq!(dump(conn).await?, before);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(conn)
            .await?,
        changes
    );
    Ok(serde_json::to_value(quote)?)
}

async fn unchanged_error(
    conn: &mut SqliteConnection,
    id: Uuid,
    expected_error: &str,
) -> anyhow::Result<()> {
    let before = dump(conn).await?;
    let changes: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *conn)
        .await?;
    let error = EstimateStore::latest_quote_on_connection(conn, id)
        .await
        .unwrap_err();
    assert!(error.to_string().contains(expected_error), "{error:#}");
    assert_eq!(dump(conn).await?, before);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(conn)
            .await?,
        changes
    );
    Ok(())
}

// Expected amounts and all seven usage fields are supplied literally, never repriced.
fn expected(
    a: &Attempt,
    rows: &[Observation],
    selected: Option<&Snapshot>,
    usage: [Option<i64>; 7],
    buckets: Value,
    subtotal: &str,
) -> Value {
    let [input, noncached, read, write, output, reasoning, total] = usage;
    json!({
        "attempt":a, "observations":rows, "snapshot":selected,
        "usage":{"input":input,"noncached":noncached,"read":read,"write":write,
            "output":output,"reasoning":reasoning,"total":total},
        "buckets":buckets, "known_subtotal":subtotal, "all_buckets_priced":null,
        "subtotal_display":{"text":subtotal,"rounded":false,"nonzero_sub_micro":false}
    })
}

#[tokio::test]
async fn current_stale_reordered_and_caller_rollback_survive_two_reopens() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    let price = snapshot("3");
    let first = row(3, json!({"input":50,"read":10}));
    store
        .journal
        .append_observation(&a, &[first.clone()])
        .await?;
    let historical = expected(
        &a,
        &[first.clone()],
        Some(&price),
        [None, Some(50), Some(10), None, None, None, None],
        json!([{"Priced":"0.00015"},{"Priced":"0.000003"},"MissingUsage","MissingUsage"]),
        "0.000153",
    );
    assert_eq!(
        serde_json::to_value(
            store
                .persist_current(a.attempt_id, &[price.clone()])
                .await?
        )?,
        historical
    );
    let mut tx = runtime.pool.begin().await?;
    assert_eq!(unchanged_quote(&mut tx, a.attempt_id).await?, historical);
    tx.commit().await?;
    let earlier = row(1, json!({"input":40}));
    let newer = row(4, json!({"input":60}));
    store
        .journal
        .append_observation(&a, &[newer.clone(), earlier.clone()])
        .await?;
    let rows = vec![earlier, first, newer];
    let latest = expected(
        &a,
        &rows,
        Some(&price),
        [None, Some(60), Some(10), None, None, None, None],
        json!([{"Priced":"0.00018"},{"Priced":"0.000003"},"MissingUsage","MissingUsage"]),
        "0.000183",
    );
    let mut tx = runtime.pool.begin().await?;
    let baseline = dump(&mut tx).await?;
    assert_eq!(unchanged_quote(&mut tx, a.attempt_id).await?, latest);
    let staged = row(5, json!({"input":70}));
    sqlx::query("INSERT INTO draft_accounting_observations VALUES (?, ?, ?, ?, ?)")
        .bind(a.attempt_id.to_string())
        .bind(i64::from(staged.revision))
        .bind(staged.source.to_string())
        .bind(i64::from(staged.sequence))
        .bind(serde_json::to_string(&staged)?)
        .execute(&mut *tx)
        .await?;
    let mut staged_rows = rows.clone();
    staged_rows.push(staged);
    assert_eq!(
        unchanged_quote(&mut tx, a.attempt_id).await?,
        expected(
            &a,
            &staged_rows,
            Some(&price),
            [None, Some(70), Some(10), None, None, None, None],
            json!([{"Priced":"0.00021"},{"Priced":"0.000003"},"MissingUsage","MissingUsage"]),
            "0.000213",
        )
    );
    tx.rollback().await?;
    let mut tx = runtime.pool.begin().await?;
    assert_eq!(dump(&mut tx).await?, baseline);
    assert_eq!(unchanged_quote(&mut tx, a.attempt_id).await?, latest);
    tx.commit().await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let mut tx = runtime.pool.begin().await?;
        assert_eq!(dump(&mut tx).await?, baseline);
        assert_eq!(unchanged_quote(&mut tx, a.attempt_id).await?, latest);
        let old = EstimateStore::read_on_connection(
            &mut tx,
            a.attempt_id,
            &serde_json::to_string(std::slice::from_ref(&rows[1]))?,
        )
        .await?
        .unwrap();
        assert_eq!(serde_json::to_value(old)?, historical);
        assert_eq!(dump(&mut tx).await?, baseline);
        tx.commit().await?;
        runtime.close().await;
        assert!(runtime.pool.is_closed());
    }
    Ok(())
}

#[tokio::test]
async fn null_unbound_missing_estimates_and_exact_quotes_survive_two_reopens() -> anyhow::Result<()>
{
    // Binding is explicit fixture state; another attempt installs an eligible catalog row.
    for mode in [
        "null",
        "unbound",
        "bound_without_estimate",
        "intent",
        "zero",
        "unknown",
        "tiny",
    ] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = EstimateStore::create_for_tests(&runtime).await?;
        let mut other = attempt();
        other.attempt_id = Uuid::from_u128(90);
        store.journal.begin_attempt(&other).await?;
        store
            .persist_current(other.attempt_id, &[snapshot("3")])
            .await?;
        let mut a = attempt();
        if mode == "unknown" {
            a.dialect = super::super::super::types::Dialect::UnknownCompatible;
        }
        let rows = match mode {
            "intent" => vec![],
            "zero" => vec![row(
                1,
                json!({"input":0,"read":0,"write":0,"output":0,"reasoning":0}),
            )],
            "unknown" => vec![row(
                1,
                json!({"input":50,"read":10,"reasoning":2,"total":80}),
            )],
            "tiny" => vec![row(1, json!({"input":1}))],
            _ => vec![row(1, json!({"input":50,"read":10}))],
        };
        store.journal.append_observation(&a, &rows).await?;
        let mut price = snapshot("3");
        match mode {
            "null" => {
                store.persist_current(a.attempt_id, &[]).await?;
            }
            "bound_without_estimate" => {
                sqlx::query("INSERT INTO draft_accounting_price_bindings VALUES (?, ?)")
                    .bind(a.attempt_id.to_string())
                    .bind(price.id.to_string())
                    .execute(runtime.pool.as_ref())
                    .await?;
            }
            "tiny" => {
                price = snapshot("0.000000000000000001");
                price.id = Uuid::from_u128(91);
                store
                    .persist_current(a.attempt_id, &[price.clone()])
                    .await?;
            }
            _ => {}
        }
        let mut want = expected(
            &a,
            &rows,
            /*selected*/ None,
            [None, Some(50), Some(10), None, None, None, None],
            json!(["MissingRate", "MissingRate", "MissingUsage", "MissingUsage"]),
            "0",
        );
        want["subtotal_display"]["text"] = json!("0.000000");
        match mode {
            "intent" => {
                want["usage"] = json!({"input":null,"noncached":null,"read":null,"write":null,"output":null,"reasoning":null,"total":null});
                want["buckets"] = json!([
                    "MissingUsage",
                    "MissingUsage",
                    "MissingUsage",
                    "MissingUsage"
                ]);
            }
            "zero" => {
                want["usage"] = json!({"input":0,"noncached":0,"read":0,"write":0,"output":0,"reasoning":0,"total":0});
                want["buckets"] =
                    json!([{"Priced":"0"},{"Priced":"0"},{"Priced":"0"},{"Priced":"0"}]);
                want["all_buckets_priced"] = json!("0");
            }
            "unknown" => {
                want["usage"]["noncached"] = Value::Null;
                want["buckets"][0] = json!("MissingUsage");
            }
            "bound_without_estimate" => {
                want = expected(
                    &a,
                    &rows,
                    Some(&price),
                    [None, Some(50), Some(10), None, None, None, None],
                    json!([{"Priced":"0.00015"},{"Priced":"0.000003"},"MissingUsage","MissingUsage"]),
                    "0.000153",
                );
            }
            "tiny" => {
                want["snapshot"] = serde_json::to_value(&price)?;
                want["usage"]["noncached"] = json!(1);
                want["usage"]["read"] = Value::Null;
                want["buckets"] = json!([{"Priced":"0.000000000000000000000001"},"MissingUsage","MissingUsage","MissingUsage"]);
                want["known_subtotal"] = json!("0.000000000000000000000001");
                want["subtotal_display"] =
                    json!({"text":"0.000000","rounded":true,"nonzero_sub_micro":true});
            }
            _ => {}
        }
        let mut tx = runtime.pool.begin().await?;
        let baseline = dump(&mut tx).await?;
        assert_eq!(
            unchanged_quote(&mut tx, a.attempt_id).await?,
            want,
            "{mode}"
        );
        tx.commit().await?;
        runtime.close().await;
        for _ in 0..2 {
            let runtime =
                StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
            let mut tx = runtime.pool.begin().await?;
            assert_eq!(dump(&mut tx).await?, baseline);
            assert_eq!(
                unchanged_quote(&mut tx, a.attempt_id).await?,
                want,
                "{mode}"
            );
            tx.commit().await?;
            runtime.close().await;
            assert!(runtime.pool.is_closed());
        }
    }
    Ok(())
}

#[tokio::test]
async fn every_current_and_old_version_is_validated_even_without_latest_estimate()
-> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    store
        .journal
        .append_observation(&a, &[row(1, json!({"input":50,"read":10}))])
        .await?;
    let first = store
        .persist_current(a.attempt_id, &[snapshot("3")])
        .await?;
    store
        .journal
        .append_observation(&a, &[row(2, json!({"input":60}))])
        .await?;
    let second = store.persist_current(a.attempt_id, &[]).await?;
    for saved in [&first, &second] {
        for (column, corruption, message) in [
            ("payload", "{}".to_owned(), "corrupt estimate payload"),
            ("evidence", "[".to_owned(), "EOF"),
            (
                "evidence",
                format!(" {}", serde_json::to_string(&saved.observations)?),
                "noncanonical evidence",
            ),
            (
                "evidence",
                serde_json::to_string(&vec![row(9, json!({"input":50}))])?,
                "missing retained observation",
            ),
            (
                "evidence",
                serde_json::to_string(&vec![row(1, json!({"input":51,"read":10}))])?,
                "changed retained observation",
            ),
        ] {
            let mut tx = runtime.pool.begin().await?;
            let query = match column {
                "payload" => "UPDATE draft_accounting_estimates SET payload = ? WHERE evidence = ?",
                "evidence" => {
                    "UPDATE draft_accounting_estimates SET evidence = ? WHERE evidence = ?"
                }
                _ => unreachable!("fixture column"),
            };
            sqlx::query(query)
                .bind(corruption)
                .bind(serde_json::to_string(&saved.observations)?)
                .execute(&mut *tx)
                .await?;
            unchanged_error(&mut tx, a.attempt_id, message).await?;
            tx.rollback().await?;
        }
    }
    // The old version is now the only saved one; a plausible fresh quote must not hide it.
    let mut tx = runtime.pool.begin().await?;
    sqlx::query("DELETE FROM draft_accounting_estimates WHERE evidence = ?")
        .bind(serde_json::to_string(&second.observations)?)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE draft_accounting_estimates SET payload = '{}'")
        .execute(&mut *tx)
        .await?;
    unchanged_error(&mut tx, a.attempt_id, "corrupt estimate payload").await?;
    tx.rollback().await?;
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn source_binding_and_snapshot_corruption_cannot_fall_back_to_unknown() -> anyhow::Result<()>
{
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    store
        .journal
        .append_observation(&a, &[row(1, json!({"input":50,"read":10}))])
        .await?;
    store
        .persist_current(a.attempt_id, &[snapshot("3")])
        .await?;
    // Only this fixture connection disables FK enforcement, solely to inject impossible rows.
    let mut conn = runtime.pool.acquire().await?;
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await?;
    for (query, message) in [
        ("DELETE FROM draft_accounting_attempts", "missing attempt"),
        (
            "UPDATE draft_accounting_attempts SET request_id = 'forged'",
            "attempt identity mismatch",
        ),
        (
            "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.attempt_id', '00000000-0000-0000-0000-000000000099')",
            "attempt identity mismatch",
        ),
        (
            "UPDATE draft_accounting_observations SET sequence = 99",
            "journal position mismatch",
        ),
        (
            "UPDATE draft_accounting_observations SET revision = 9",
            "journal position mismatch",
        ),
        (
            "UPDATE draft_accounting_observations SET source = 'forged'",
            "journal position mismatch",
        ),
        (
            "DELETE FROM draft_accounting_price_bindings",
            "missing binding",
        ),
        (
            "UPDATE draft_accounting_price_bindings SET snapshot_id = NULL",
            "corrupt estimate payload",
        ),
        (
            "DELETE FROM draft_accounting_price_snapshots",
            "missing snapshot",
        ),
        (
            "UPDATE draft_accounting_price_snapshots SET payload = '{}'",
            "missing field",
        ),
        (
            "UPDATE draft_accounting_price_snapshots SET payload = ' ' || payload",
            "noncanonical snapshot",
        ),
    ] {
        let mut tx = conn.begin().await?;
        sqlx::query(query).execute(&mut *tx).await?;
        unchanged_error(&mut tx, a.attempt_id, message).await?;
        tx.rollback().await?;
    }
    // No saved estimate: authority and bound snapshot validation must still run.
    for (query, message) in [
        (
            "UPDATE draft_accounting_attempts SET request_id = 'forged'",
            "attempt identity mismatch",
        ),
        (
            "DELETE FROM draft_accounting_price_snapshots",
            "missing snapshot",
        ),
        (
            "UPDATE draft_accounting_price_snapshots SET payload = '{}'",
            "missing field",
        ),
        (
            "UPDATE draft_accounting_price_snapshots SET payload = ' ' || payload",
            "noncanonical snapshot",
        ),
    ] {
        let mut tx = conn.begin().await?;
        sqlx::query("DELETE FROM draft_accounting_estimates")
            .execute(&mut *tx)
            .await?;
        sqlx::query(query).execute(&mut *tx).await?;
        unchanged_error(&mut tx, a.attempt_id, message).await?;
        tx.rollback().await?;
    }
    for (field, value, message) in [
        (
            "model",
            json!("ineligible-model"),
            "ineligible bound snapshot",
        ),
        (
            "id",
            json!(Uuid::from_u128(99)),
            "snapshot identity mismatch",
        ),
        ("provider", json!(""), "invalid snapshot metadata"),
    ] {
        let mut price = snapshot("3");
        let mut payload = serde_json::to_value(&price)?;
        payload[field] = value;
        price = serde_json::from_value(payload)?;
        let mut tx = conn.begin().await?;
        sqlx::query("DELETE FROM draft_accounting_estimates")
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE draft_accounting_price_snapshots SET payload = ?")
            .bind(serde_json::to_string(&price)?)
            .execute(&mut *tx)
            .await?;
        unchanged_error(&mut tx, a.attempt_id, message).await?;
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
async fn invalid_replay_prefix_and_checked_arithmetic_fail_without_saved_estimates()
-> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    store
        .journal
        .append_observation(
            &a,
            &[
                row(1, json!({"input":50,"read":10})),
                row(2, json!({"output":10})),
            ],
        )
        .await?;
    store
        .persist_current(a.attempt_id, &[snapshot("3")])
        .await?;
    for (patch, message) in [
        (
            json!({"reasoning":9,"output":8}),
            "reasoning exceeds output",
        ),
        (json!({"input":i64::MAX,"read":1}), "input overflow"),
        (json!({"read":i64::MAX,"write":1}), "cache overflow"),
        (
            json!({"input":i64::MAX,"read":0,"write":0}),
            "total overflow",
        ),
    ] {
        let mut tx = runtime.pool.begin().await?;
        sqlx::query("DELETE FROM draft_accounting_estimates")
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE draft_accounting_observations SET payload = ? WHERE revision = 1")
            .bind(serde_json::to_string(&row(1, patch))?)
            .execute(&mut *tx)
            .await?;
        unchanged_error(&mut tx, a.attempt_id, message).await?;
        tx.rollback().await?;
    }
    let mut tx = runtime.pool.begin().await?;
    sqlx::query("DELETE FROM draft_accounting_estimates")
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE draft_accounting_price_snapshots SET payload = ?")
        .bind(serde_json::to_string(&snapshot(
            "99999999999999999999999999999999999999",
        ))?)
        .execute(&mut *tx)
        .await?;
    unchanged_error(&mut tx, a.attempt_id, "price product overflow").await?;
    tx.rollback().await?;
    runtime.close().await;
    Ok(())
}
