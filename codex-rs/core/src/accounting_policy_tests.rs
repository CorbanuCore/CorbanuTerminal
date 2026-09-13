use super::*;
use anyhow::Result;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::ThreadMetadataBuilder;
use codex_utils_absolute_path::AbsolutePathBuf;
use futures::poll;
use pretty_assertions::assert_eq;
use sqlx::Connection;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::time::Duration;

const MODEL: &str = "claude-opus-5";
const ENDPOINT: &str = "http://127.0.0.1:1/v1/messages";

struct Fixture {
    home: tempfile::TempDir,
    db: Arc<StateRuntime>,
    sampling: Arc<Sampling>,
}

impl Fixture {
    async fn new() -> Result<Self> {
        let home = tempfile::tempdir()?;
        let db = StateRuntime::init(
            SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path())?),
            "anthropic".into(),
        )
        .await?;
        let owner = ThreadId::new();
        db.upsert_thread(
            &ThreadMetadataBuilder::new(
                owner,
                home.path().join("policy.jsonl"),
                chrono::Utc::now(),
                SessionSource::Cli,
            )
            .build("anthropic"),
        )
        .await?;
        let sampling = Sampling::start(db.clone(), owner, "policy-turn".into(), &mode()).await?;
        Ok(Self { home, db, sampling })
    }

    async fn connection(&self) -> Result<sqlx::SqliteConnection> {
        let pool = self
            .db
            .sqlite()
            .open_read_write_pool(&self.db.sqlite().state_db_path())
            .await?;
        let connection = pool.acquire().await?.detach();
        pool.close().await;
        Ok(connection)
    }

    async fn attempts(&self) -> Result<Vec<Attempt>> {
        let mut connection = self.connection().await?;
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts ORDER BY rowid")
                .fetch_all(&mut connection)
                .await?;
        connection.close().await?;
        rows.into_iter()
            .map(|row| Ok(serde_json::from_str(&row)?))
            .collect()
    }

    async fn two_reopens(mut self, expected: Vec<Attempt>) -> Result<()> {
        self.db.close().await;
        for _ in 0..2 {
            self.db = StateRuntime::init(
                SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(self.home.path())?),
                "anthropic".into(),
            )
            .await?;
            AccountingStore::open(&self.db, now()).await?;
            assert_eq!(self.attempts().await?, expected);
            self.db.close().await;
        }
        Ok(())
    }
}

fn mode() -> AccountingMode {
    AccountingMode::DirectAnthropic {
        scope: Uuid::new_v4(),
        approved_endpoint: "http://127.0.0.1:1/v1".into(),
    }
}

fn poison<T>(mutex: &Mutex<T>) {
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("deliberate policy fixture poison");
        }))
        .is_err()
    );
}

#[tokio::test]
async fn accounting_policy_wait_cancellation_and_post_wait_failure_have_no_effect() -> Result<()> {
    let fixture = Fixture::new().await?;
    let permit = WRITES.acquire().await?;
    let mut waiting = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    assert!(poll!(&mut waiting).is_pending());
    drop(waiting);
    fixture.sampling.check()?;
    let mut start = Box::pin(Sampling::start(
        fixture.db.clone(),
        fixture.sampling.owner,
        "cancelled-start".into(),
        &AccountingMode::Disabled,
    ));
    // OFF fails before the gate; a valid start waits and can be dropped without a Slot.
    assert!(start.as_mut().await.is_err());
    let enabled = mode();
    let mut start = Box::pin(Sampling::start(
        fixture.db.clone(),
        fixture.sampling.owner,
        "cancelled-start".into(),
        &enabled,
    ));
    assert!(poll!(&mut start).is_pending());
    drop(start);
    let mut rejected = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    assert!(poll!(&mut rejected).is_pending());
    fixture.sampling.reject();
    drop(permit);
    assert!(rejected.await.is_err());
    assert!(fixture.attempts().await?.is_empty());
    let fresh = Sampling::start(
        fixture.db.clone(),
        fixture.sampling.owner,
        "fresh".into(),
        &enabled,
    )
    .await?;
    let attempt = fresh.admit(MODEL, ENDPOINT).await?;
    assert_eq!(attempt.retry_of, None);
    fixture.two_reopens(vec![attempt]).await
}

#[tokio::test]
async fn accounting_policy_serial_retry_identity_and_time_sample_after_gate() -> Result<()> {
    let fixture = Fixture::new().await?;
    let permit = WRITES.acquire().await?;
    let mut first = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    let mut second = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
    assert!(poll!(&mut first).is_pending());
    assert!(poll!(&mut second).is_pending());
    let queued_at = now();
    tokio::time::timeout(Duration::from_secs(1), async {
        while now() <= queued_at {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    let barrier_time = now();
    drop(permit);
    let (first, second) = tokio::try_join!(first, second)?;
    assert_eq!(first.retry_of, None);
    assert_eq!(second.retry_of, Some(first.attempt_id));
    assert_ne!(first.attempt_id, second.attempt_id);
    for attempt in [&first, &second] {
        assert_eq!(attempt.request_id, fixture.sampling.request);
        assert_eq!(attempt.thread_id, fixture.sampling.owner);
        assert_eq!(attempt.turn, "policy-turn");
        assert!(i64::from(attempt.dispatched_at_ms) >= barrier_time);
    }
    assert!(i64::from(second.dispatched_at_ms) >= i64::from(first.dispatched_at_ms));
    fixture.two_reopens(vec![first, second]).await
}

#[tokio::test]
async fn accounting_policy_observe_wait_cancel_and_start_cancel_publish_nothing() -> Result<()> {
    let fixture = Fixture::new().await?;
    let attempt = fixture.sampling.admit(MODEL, ENDPOINT).await?;
    let permit = WRITES.acquire().await?;
    let mut waiting = Box::pin(fixture.sampling.observe(
        &attempt,
        Uuid::new_v4(),
        0,
        AnthropicUsagePatch::default(),
    ));
    assert!(poll!(&mut waiting).is_pending());
    drop(waiting);
    fixture.sampling.check()?;
    drop(permit);
    let mut connection = fixture.connection().await?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut connection)
        .await?;
    let slot = Slot::default();
    let enabled = mode();
    let mut start = Box::pin(async {
        let sampling = Sampling::start(
            fixture.db.clone(),
            fixture.sampling.owner,
            "cancelled".into(),
            &enabled,
        )
        .await?;
        SamplingScope::attach(slot.clone(), Some(sampling))
    });
    assert!(poll!(&mut start).is_pending());
    assert_eq!(WRITES.available_permits(), 0);
    drop(start);
    assert_eq!(WRITES.available_permits(), 1);
    assert!(read_slot(&slot)?.is_none());
    sqlx::query("ROLLBACK").execute(&mut connection).await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM draft_accounting_observations")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(count, 0);
    connection.close().await?;
    fixture.two_reopens(vec![attempt]).await
}

#[tokio::test]
async fn accounting_policy_cancel_in_operation_rejects_and_releases_capacity() -> Result<()> {
    for operation in ["admit", "observe"] {
        let fixture = Fixture::new().await?;
        let attempt = fixture.sampling.admit(MODEL, ENDPOINT).await?;
        let mut connection = fixture.connection().await?;
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut connection)
            .await?;
        let sampling = &fixture.sampling;
        let mut pending = Box::pin(async {
            if operation == "admit" {
                sampling.admit(MODEL, ENDPOINT).await.map(|_| ())
            } else {
                sampling
                    .observe(
                        &attempt,
                        Uuid::new_v4(),
                        0,
                        AnthropicUsagePatch {
                            input_tokens: AnthropicTokenPresence::Number(7),
                            ..Default::default()
                        },
                    )
                    .await
            }
        });
        assert!(poll!(&mut pending).is_pending());
        assert_eq!(
            WRITES.available_permits(),
            0,
            "inside the operation, not queued"
        );
        drop(pending);
        assert!(fixture.sampling.check().is_err());
        assert_eq!(WRITES.available_permits(), 1);
        sqlx::query("ROLLBACK").execute(&mut connection).await?;
        connection.close().await?;
        assert!(fixture.sampling.admit(MODEL, ENDPOINT).await.is_err());
        let mut connection = fixture.connection().await?;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM draft_accounting_observations")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(count, 0);
        connection.close().await?;
        let fresh = tokio::time::timeout(
            Duration::from_secs(10),
            Sampling::start(
                fixture.db.clone(),
                fixture.sampling.owner,
                "fresh".into(),
                &mode(),
            ),
        )
        .await??;
        let next = fresh.admit(MODEL, ENDPOINT).await?;
        assert_eq!(next.retry_of, None);
        assert_ne!(next.request_id, attempt.request_id);
        // This barrier blocked the first DB operation. It does not establish
        // rollback for arbitrary cancellation after SQLite has committed.
        fixture.two_reopens(vec![attempt, next]).await?;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_policy_completion_guard_never_repairs_possible_committed_rows() -> Result<()> {
    let fixture = Fixture::new().await?;
    let attempt = fixture.sampling.admit(MODEL, ENDPOINT).await?;
    // Directly exercise the uncertain-completion guard with durable data.
    // This is not a claim of a process kill at an unobservable SQL instruction.
    drop(Completion {
        sampling: &fixture.sampling,
        complete: false,
    });
    assert!(fixture.sampling.check().is_err());
    assert!(fixture.sampling.admit(MODEL, ENDPOINT).await.is_err());
    fixture.two_reopens(vec![attempt]).await
}

#[tokio::test]
async fn accounting_policy_previous_poison_before_and_during_admission_is_sticky() -> Result<()> {
    for phase in ["before", "during"] {
        let fixture = Fixture::new().await?;
        let mut connection = fixture.connection().await?;
        if phase == "during" {
            sqlx::query("BEGIN IMMEDIATE")
                .execute(&mut connection)
                .await?;
        } else {
            poison(&fixture.sampling.previous);
        }
        let mut pending = Box::pin(fixture.sampling.admit(MODEL, ENDPOINT));
        if phase == "during" {
            assert!(poll!(&mut pending).is_pending());
            assert_eq!(WRITES.available_permits(), 0);
            poison(&fixture.sampling.previous);
            sqlx::query("ROLLBACK").execute(&mut connection).await?;
        }
        assert!(pending.await.is_err());
        assert!(fixture.sampling.check().is_err());
        assert!(fixture.sampling.admit(MODEL, ENDPOINT).await.is_err());
        assert_eq!(WRITES.available_permits(), 1);
        connection.close().await?;
        let rows = fixture.attempts().await?;
        assert_eq!(rows.len(), usize::from(phase == "during"));
        if let Some(attempt) = rows.first() {
            assert_eq!(attempt.retry_of, None);
            assert_eq!(attempt.request_id, fixture.sampling.request);
        }
        fixture.two_reopens(rows).await?;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_policy_slot_poison_rejects_stale_and_incoming_without_disabling() -> Result<()>
{
    for operation in ["read", "attach", "drop"] {
        let fixture = Fixture::new().await?;
        let incoming = Sampling::start(
            fixture.db.clone(),
            fixture.sampling.owner,
            "incoming".into(),
            &mode(),
        )
        .await?;
        let slot = Slot::default();
        let scope = SamplingScope::attach(slot.clone(), Some(fixture.sampling.clone()))?;
        poison(&slot);
        match operation {
            "read" => assert!(read_slot(&slot).is_err()),
            "attach" => {
                assert!(SamplingScope::attach(slot.clone(), Some(incoming.clone())).is_err());
                assert!(incoming.check().is_err());
            }
            _ => {}
        }
        drop(scope);
        assert!(fixture.sampling.check().is_err());
        assert!(slot.is_poisoned());
        assert!(slot.lock().err().unwrap().into_inner().is_none());
        assert!(read_slot(&slot).is_err(), "empty poison must not look OFF");
        assert!(SamplingScope::attach(slot.clone(), None).is_err());
        assert!(fixture.attempts().await?.is_empty());
        fixture.db.close().await;
    }
    Ok(())
}
