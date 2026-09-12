use super::*;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

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
        |p| {
            let _ = std::fs::remove_dir_all(p);
        },
    )
}

async fn counts(store: &EstimateStore<'_>) -> anyhow::Result<(i64, i64, i64)> {
    Ok(sqlx::query_as(
        "SELECT (SELECT count(*) FROM draft_accounting_price_snapshots),
        (SELECT count(*) FROM draft_accounting_price_bindings),
        (SELECT count(*) FROM draft_accounting_estimates)",
    )
    .fetch_one(store.journal.runtime.pool.as_ref())
    .await?)
}

#[tokio::test]
async fn exact_cases_survive_two_disk_reopens() -> anyhow::Result<()> {
    for (patch, rate, exact, display, rounded, sub) in [
        (
            json!({"input":50,"read":10}),
            Some("3"),
            "0.000153",
            "0.000153",
            false,
            false,
        ),
        (json!({"input":null}), None, "0", "0.000000", false, false),
        (
            json!({"input":0,"read":0,"write":0,"output":0}),
            None,
            "0",
            "0.000000",
            false,
            false,
        ),
        (
            json!({"input":1}),
            Some("0.5"),
            "0.0000005",
            "0.000000",
            true,
            true,
        ),
        (
            json!({"input":3}),
            Some("0.5"),
            "0.0000015",
            "0.000002",
            true,
            false,
        ),
        (
            json!({"input":1,"read":1}),
            Some("0.4"),
            "0.0000008",
            "0.000001",
            true,
            true,
        ),
        (
            json!({"input":1}),
            Some("0.000000000000000001"),
            "0.000000000000000000000001",
            "0.000000",
            true,
            true,
        ),
    ] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = EstimateStore::create_for_tests(&runtime).await?;
        let rows = vec![row(1, patch)];
        store.journal.append_observation(&attempt(), &rows).await?;
        let candidates: Vec<_> = rate
            .map(|r| {
                let mut s = snapshot(r);
                if r == "0.4" {
                    s.rates.read = s.rates.noncached;
                }
                s
            })
            .into_iter()
            .collect();
        let expected = quote_observations(&attempt(), &rows, &candidates)?;
        assert_eq!(serde_json::to_value(expected.known_subtotal)?, json!(exact));
        assert_eq!(
            expected.subtotal_display,
            DisplayAmount {
                text: display.into(),
                rounded,
                nonzero_sub_micro: sub,
            }
        );
        assert_eq!(
            store
                .persist_current(attempt().attempt_id, &candidates)
                .await?,
            expected
        );
        let evidence = serde_json::to_string(&rows)?;
        runtime.close().await;
        assert!(runtime.pool.is_closed());
        for _ in 0..2 {
            let runtime =
                StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
            let store = EstimateStore {
                journal: Journal { runtime: &runtime },
            };
            assert_eq!(
                store.read_estimate(attempt().attempt_id, &evidence).await?,
                Some(expected.clone())
            );
            assert_eq!(
                store.persist_current(attempt().attempt_id, &[]).await?,
                expected
            );
            assert_eq!(
                store
                    .journal
                    .read_observations(attempt().attempt_id)
                    .await?,
                Some((attempt(), rows.clone()))
            );
            assert_eq!(counts(&store).await?, (i64::from(rate.is_some()), 1, 1));
            runtime.close().await;
            assert!(runtime.pool.is_closed());
        }
    }
    assert!(serde_json::from_value::<Decimal>(json!("0.0000000000000000001")).is_err());
    Ok(())
}

#[tokio::test]
async fn late_revisions_preserve_versions_and_unavailable_binding() -> anyhow::Result<()> {
    for candidates in [vec![snapshot("3")], vec![]] {
        let home = home();
        let runtime =
            StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
        let store = EstimateStore::create_for_tests(&runtime).await?;
        let a = attempt();
        store
            .journal
            .append_observation(&a, &[row(3, json!({"input":50,"read":10}))])
            .await?;
        let first = store.persist_current(a.attempt_id, &candidates).await?;
        store
            .journal
            .append_observation(&a, &[row(1, json!({"input":40}))])
            .await?;
        let mut later = snapshot("9");
        later.id = Uuid::from_u128(90);
        let second = store
            .persist_current(a.attempt_id, &[later.clone()])
            .await?;
        assert_eq!(second.known_subtotal, first.known_subtotal);
        assert_eq!(second.snapshot, first.snapshot);
        later.effective_from_ms = Count::try_from(200)?;
        later.effective_end_ms = None;
        store
            .journal
            .append_observation(&a, &[row(4, json!({"input":60,"write":0,"output":0}))])
            .await?;
        let third = store.persist_current(a.attempt_id, &[later]).await?;
        assert_eq!(third.snapshot, first.snapshot);
        assert_eq!(
            serde_json::to_value(third.known_subtotal)?,
            json!(if candidates.is_empty() {
                "0"
            } else {
                "0.000183"
            })
        );
        for expected in [first, second, third] {
            assert_eq!(
                store
                    .read_estimate(
                        a.attempt_id,
                        &serde_json::to_string(&expected.observations)?
                    )
                    .await?,
                Some(expected)
            );
        }
        assert_eq!(
            counts(&store).await?,
            (i64::from(!candidates.is_empty()), 1, 3)
        );
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn concurrent_idempotence_and_snapshot_conflicts() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    store.journal.begin_attempt(&a).await?;
    let candidates = [snapshot("3")];
    let (left, right) = tokio::join!(
        store.persist_current(a.attempt_id, &candidates),
        store.persist_current(a.attempt_id, &candidates)
    );
    assert_eq!(left?, right?);
    assert_eq!(counts(&store).await?, (1, 1, 1));
    let mut other = a.clone();
    other.attempt_id = Uuid::from_u128(91);
    store.journal.begin_attempt(&other).await?;
    let mut changed = snapshot("7");
    changed.model = "unselected-model".into();
    assert!(
        store
            .persist_current(other.attempt_id, &[changed])
            .await
            .is_err()
    );
    assert_eq!(counts(&store).await?, (1, 1, 1));
    let mut first = snapshot("5");
    first.id = Uuid::from_u128(92);
    let mut second = first.clone();
    second.rates.noncached = Some(Decimal::try_from("6".to_owned())?);
    let first = [first];
    let second = [second];
    let (left, right) = tokio::join!(
        store.persist_current(other.attempt_id, &first),
        store.persist_current(other.attempt_id, &second)
    );
    assert_ne!(left.is_ok(), right.is_ok());
    assert_eq!(counts(&store).await?, (2, 2, 2));
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn validation_and_sql_failure_roll_back_all_estimate_writes() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    assert!(store.persist_current(a.attempt_id, &[]).await.is_err());
    assert_eq!(store.read_estimate(a.attempt_id, "unknown").await?, None);
    store
        .journal
        .append_observation(&a, &[row(1, json!({"input":4}))])
        .await?;
    let before = store.journal.read_observations(a.attempt_id).await?;
    let mut invalid = snapshot("3");
    invalid.provider = "".into();
    for candidates in [
        vec![invalid],
        vec![snapshot("3"); 2],
        vec![snapshot("3"); 65],
        vec![snapshot("99999999999999999999999999999999999999")],
    ] {
        assert!(
            store
                .persist_current(a.attempt_id, &candidates)
                .await
                .is_err()
        );
        assert_eq!(counts(&store).await?, (0, 0, 0));
    }
    sqlx::raw_sql("CREATE TRIGGER fail_estimate BEFORE INSERT ON draft_accounting_estimates BEGIN SELECT RAISE(ABORT, 'injected'); END;")
        .execute(runtime.pool.as_ref()).await?;
    assert!(
        store
            .persist_current(a.attempt_id, &[snapshot("3")])
            .await
            .is_err()
    );
    assert_eq!(counts(&store).await?, (0, 0, 0));
    sqlx::raw_sql("DROP TRIGGER fail_estimate")
        .execute(runtime.pool.as_ref())
        .await?;
    let expected = store
        .persist_current(a.attempt_id, &[snapshot("3")])
        .await?;
    sqlx::query("UPDATE draft_accounting_estimates SET payload = '{}'")
        .execute(runtime.pool.as_ref())
        .await?;
    assert!(store.persist_current(a.attempt_id, &[]).await.is_err());
    assert!(
        store
            .read_estimate(
                a.attempt_id,
                &serde_json::to_string(&expected.observations)?
            )
            .await
            .is_err()
    );
    assert_eq!(store.journal.read_observations(a.attempt_id).await?, before);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn forged_quote_fields_and_snapshot_content_are_rejected() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    store
        .journal
        .append_observation(&a, &[row(1, json!({"input":50,"read":10}))])
        .await?;
    let quote = store
        .persist_current(a.attempt_id, &[snapshot("3")])
        .await?;
    let original = serde_json::to_value(&quote)?;
    let evidence = serde_json::to_string(&quote.observations)?;
    for (pointer, value) in [
        ("/known_subtotal", json!("0.000154")),
        ("/subtotal_display/rounded", json!(true)),
        ("/subtotal_display/nonzero_sub_micro", json!(true)),
        ("/snapshot/currency", json!("EUR")),
        ("/attempt/turn", json!("forged")),
        ("/observations/0/sequence", json!(99)),
        ("/usage/noncached", json!(51)),
        ("/buckets/0/Priced", json!(0.00015)),
    ] {
        let mut forged = original.clone();
        *forged.pointer_mut(pointer).unwrap() = value;
        sqlx::query("UPDATE draft_accounting_estimates SET payload = ?")
            .bind(serde_json::to_string(&forged)?)
            .execute(runtime.pool.as_ref())
            .await?;
        assert!(
            store.read_estimate(a.attempt_id, &evidence).await.is_err(),
            "{pointer}"
        );
    }
    for remove in [false, true] {
        let mut forged = original.clone();
        if remove {
            forged.as_object_mut().unwrap().remove("all_buckets_priced");
        } else {
            forged["extra"] = json!(0);
        }
        sqlx::query("UPDATE draft_accounting_estimates SET payload = ?")
            .bind(serde_json::to_string(&forged)?)
            .execute(runtime.pool.as_ref())
            .await?;
        assert!(store.read_estimate(a.attempt_id, &evidence).await.is_err());
    }
    sqlx::query("UPDATE draft_accounting_estimates SET payload = ?")
        .bind(serde_json::to_string(&quote)?)
        .execute(runtime.pool.as_ref())
        .await?;
    for (field, value) in [
        ("id", json!(Uuid::from_u128(99))),
        ("currency", json!("EUR")),
        ("extra", json!(0)),
        ("rates", json!({"noncached":"0.0000000000000000001"})),
    ] {
        let mut forged = serde_json::to_value(snapshot("3"))?;
        forged[field] = value;
        sqlx::query("UPDATE draft_accounting_price_snapshots SET payload = ?")
            .bind(serde_json::to_string(&forged)?)
            .execute(runtime.pool.as_ref())
            .await?;
        assert!(store.read_estimate(a.attempt_id, &evidence).await.is_err());
        assert!(store.persist_current(a.attempt_id, &[]).await.is_err());
    }
    sqlx::query("UPDATE draft_accounting_price_snapshots SET payload = ?")
        .bind(serde_json::to_string(&snapshot("3"))?)
        .execute(runtime.pool.as_ref())
        .await?;
    assert_eq!(
        store.read_estimate(a.attempt_id, &evidence).await?,
        Some(quote)
    );
    sqlx::query("UPDATE draft_accounting_price_bindings SET snapshot_id = NULL")
        .execute(runtime.pool.as_ref())
        .await?;
    assert!(store.read_estimate(a.attempt_id, &evidence).await.is_err());
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn retained_journal_keys_payload_and_evidence_are_checked() -> anyhow::Result<()> {
    let home = home();
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let store = EstimateStore::create_for_tests(&runtime).await?;
    let a = attempt();
    let rows = vec![row(1, json!({"input":50,"read":10}))];
    store.journal.append_observation(&a, &rows).await?;
    let quote = store
        .persist_current(a.attempt_id, &[snapshot("3")])
        .await?;
    let evidence = serde_json::to_string(&rows)?;
    for (change, restore) in [
        (
            "UPDATE draft_accounting_observations SET sequence = 99",
            "UPDATE draft_accounting_observations SET sequence = 10",
        ),
        (
            "UPDATE draft_accounting_observations SET revision = 2",
            "UPDATE draft_accounting_observations SET revision = 1",
        ),
        (
            "UPDATE draft_accounting_observations SET source = 'forged'",
            "UPDATE draft_accounting_observations SET source = '00000000-0000-0000-0000-000000000005'",
        ),
        (
            "UPDATE draft_accounting_attempts SET request_id = 'forged'",
            "UPDATE draft_accounting_attempts SET request_id = '00000000-0000-0000-0000-000000000002'",
        ),
    ] {
        sqlx::query(change).execute(runtime.pool.as_ref()).await?;
        assert!(store.read_estimate(a.attempt_id, &evidence).await.is_err());
        assert!(store.persist_current(a.attempt_id, &[]).await.is_err());
        sqlx::query(restore).execute(runtime.pool.as_ref()).await?;
    }
    let mut forged = rows.clone();
    forged[0].sequence = Count::try_from(99)?;
    let forged_evidence = serde_json::to_string(&forged)?;
    sqlx::query("UPDATE draft_accounting_estimates SET evidence = ?")
        .bind(&forged_evidence)
        .execute(runtime.pool.as_ref())
        .await?;
    assert!(
        store
            .read_estimate(a.attempt_id, &forged_evidence)
            .await
            .is_err()
    );
    sqlx::query("UPDATE draft_accounting_estimates SET evidence = ?")
        .bind(&evidence)
        .execute(runtime.pool.as_ref())
        .await?;
    for patch in [
        json!({"input":51,"read":10}),
        json!({"input":null,"read":10}),
    ] {
        sqlx::query("UPDATE draft_accounting_observations SET payload = ?")
            .bind(serde_json::to_string(&row(1, patch))?)
            .execute(runtime.pool.as_ref())
            .await?;
        assert!(store.read_estimate(a.attempt_id, &evidence).await.is_err());
    }
    sqlx::query("UPDATE draft_accounting_observations SET payload = ?")
        .bind(serde_json::to_string(&rows[0])?)
        .execute(runtime.pool.as_ref())
        .await?;
    assert_eq!(
        store.read_estimate(a.attempt_id, &evidence).await?,
        Some(quote)
    );
    sqlx::query("DELETE FROM draft_accounting_observations")
        .execute(runtime.pool.as_ref())
        .await?;
    assert!(store.read_estimate(a.attempt_id, &evidence).await.is_err());
    runtime.close().await;
    Ok(())
}
