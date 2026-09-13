//! Real process interruption/reopen through the normal crate's public facade.
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
async fn accounting_late_import_process_worker() -> anyhow::Result<()> {
    let Some((path, mode)) = child_input() else {
        return Ok(());
    };
    let runtime = open(&path).await?;
    let store = AccountingStore::open(&runtime, 100 * DAY).await?;
    let e = entry(1, 0)?;
    std::fs::write(path.join("ready"), b"store open")?;
    wait_file(&path.join("go")).await?;
    std::fs::write(path.join("calling"), b"import next")?;
    let bundle = [e];
    let import = store.import_retained(bundle[0].attempt.thread_id, &bundle, 100 * DAY);
    tokio::pin!(import);
    if mode == "queued" {
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut import)
                .await
                .is_err()
        );
        std::fs::write(
            path.join("queued"),
            b"public import polled and still blocked",
        )?;
    }
    let outcome = import.await?;
    assert_eq!(outcome, vec![RetainedImportOutcome::Imported]);
    runtime.close().await;
    std::fs::write(path.join("committed"), b"commit and close completed")?;
    if mode == "lost-ack" {
        std::future::pending::<()>().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_late_import_process_queued_cancel_and_lost_ack_replay() -> anyhow::Result<()> {
    for mode in ["queued", "lost-ack", "restart"] {
        let path = home();
        let runtime = open(&path).await?;
        let e = entry(1, 0)?;
        native(&runtime, e.attempt.thread_id).await?;
        AccountingStore::open(&runtime, 100 * DAY).await?;
        let mut conn = connection(&runtime).await?;
        let before = dump(&mut conn).await?;
        let mut child = spawn_worker(&path, "accounting_late_import_process_worker", mode)?;
        wait_file(&path.join("ready")).await?;
        let mut guard = conn.begin_with("BEGIN IMMEDIATE").await?;
        std::fs::write(path.join("go"), b"writer held")?;
        wait_file(&path.join("calling")).await?;
        if mode == "queued" {
            wait_file(&path.join("queued")).await?;
            // The main-state writer lock is still owned here. Import cannot commit.
            assert!(child.try_wait()?.is_none());
            assert!(!path.join("committed").exists());
            child.kill()?;
            assert!(!child.wait()?.success());
            assert_eq!(dump(&mut guard).await?, before);
            guard.rollback().await?;
        } else {
            guard.commit().await?;
            wait_file(&path.join("committed")).await?;
            if mode == "lost-ack" {
                child.kill()?;
                assert!(!child.wait()?.success());
            } else {
                assert!(child.wait()?.success());
            }
        }
        conn.close().await?;
        runtime.close().await;
        for reopen in 0..2 {
            let runtime = open(&path).await?;
            let mut conn = connection(&runtime).await?;
            if mode == "queued" && reopen == 0 {
                assert_eq!(dump(&mut conn).await?, before);
            }
            let store = AccountingStore::open(&runtime, 100 * DAY).await?;
            let outcome = if mode == "queued" && reopen == 0 {
                RetainedImportOutcome::Imported
            } else {
                RetainedImportOutcome::SuppressedByReplayRecord
            };
            assert_eq!(
                store
                    .import_retained(e.attempt.thread_id, &[e.clone()], 100 * DAY)
                    .await?,
                vec![outcome]
            );
            assert_eq!(
                store.read_day(e.attempt.thread_id, 0, 100 * DAY).await?,
                expected_day(true, 100 * DAY, 1)?
            );
            conn.close().await?;
            runtime.close().await;
        }
    }
    Ok(())
}
