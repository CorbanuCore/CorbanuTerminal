//! These tests link the normal library, never include private implementation/fixture DDL.
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
    SqliteConnection::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new()
            .filename(runtime.sqlite().state_db_path())
            .foreign_keys(true),
    )
    .await
    .map_err(Into::into)
}

fn attempt(id: u128, time: i64) -> Attempt {
    Attempt {
        attempt_id: Uuid::from_u128(id),
        request_id: Uuid::from_u128(id + 100),
        thread_id: ThreadId::from_string(&Uuid::from_u128(7).to_string()).unwrap(),
        turn: "fixture".into(),
        retry_of: None,
        provider: "synthetic".into(),
        model: "fixture".into(),
        scope: Uuid::nil(),
        dialect: Dialect::NativeAnthropic,
        dispatched_at_ms: time.try_into().unwrap(),
    }
}

fn snapshot() -> Snapshot {
    serde_json::from_value(json!({
        "id":Uuid::from_u128(1000), "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"0.000000000000000001","read":null,"write":null,"output":null},
        "source_reference":Uuid::from_u128(1001), "source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null
    }))
    .unwrap()
}

fn observation(a: &Attempt, revision: i64, input: i64) -> Observation {
    Observation {
        revision: revision.try_into().unwrap(),
        source: a.attempt_id,
        sequence: revision.try_into().unwrap(),
        patch: Patch {
            input: Presence::Number(input.try_into().unwrap()),
            ..Patch::default()
        },
    }
}

async fn native(runtime: &StateRuntime, owner: ThreadId) -> anyhow::Result<()> {
    let path = runtime.sqlite().home();
    let mut builder = ThreadMetadataBuilder::new(
        owner,
        path.join("synthetic.jsonl"),
        chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
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
    let a = attempt(1, 0);
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
    let intent = store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
    assert_eq!(intent.usage, Usage::default());
    assert_eq!(intent.all_buckets_priced, None);
    assert_eq!(intent.snapshot, Some(snapshot()));
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
    let a = attempt(1, 0);
    let missing = attempt(2, 0);
    native(&runtime, a.thread_id).await?;
    let store = AccountingStore::open(&runtime, 0).await?;
    store.admit(a.thread_id, &a, &[snapshot()], 0).await?;
    store.admit(missing.thread_id, &missing, &[], 0).await?;
    let patch = observation(&a, 2, 1);
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
        .observe(a.thread_id, &a, &[observation(&a, 1, 0)], 0)
        .await?;
    let expected = store.observe(a.thread_id, &a, &[patch], 0).await?;
    let null = store
        .admit(missing.thread_id, &missing, &[snapshot()], 0)
        .await?;
    assert_eq!(null.snapshot, None);
    let mut zero = observation(&missing, 1, 0);
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
        let mut later = snapshot();
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
    let a = attempt(1, now - 90 * DAY);
    let mut other = attempt(2, now - 1);
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
                &[snapshot()],
                i64::from(a.dispatched_at_ms),
            )
            .await?;
        store
            .observe(
                a.thread_id,
                &a,
                &[observation(&a, 1, 1)],
                i64::from(a.dispatched_at_ms),
            )
            .await?;
        store.maintain(now - 1).await?;
        store
            .admit(other.thread_id, &other, &[snapshot()], now - 1)
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
