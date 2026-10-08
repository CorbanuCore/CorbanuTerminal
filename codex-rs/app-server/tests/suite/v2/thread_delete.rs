use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use app_test_support::create_fake_paginated_rollout;
use app_test_support::create_fake_rollout;
use app_test_support::create_mock_responses_server_repeating_assistant;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::JSONRPCError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ThreadDeleteParams;
use codex_app_server_protocol::ThreadDeleteResponse;
use codex_app_server_protocol::ThreadDeletedNotification;
use codex_app_server_protocol::ThreadLoadedListParams;
use codex_app_server_protocol::ThreadLoadedListResponse;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_core::find_thread_path_by_id_str;
use codex_protocol::ThreadId;
use codex_protocol::protocol::HistoryPosition;
use codex_protocol::protocol::SessionSource;
use codex_state::DirectionalThreadSpawnEdgeStatus;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::AsOf;
use codex_state::accounting::Attempt;
use codex_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;
use std::path::Path;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[tokio::test]
async fn thread_delete_rejects_paginated_writer_owned_by_another_process() -> Result<()> {
    let server = create_mock_responses_server_repeating_assistant("Done").await;
    let codex_home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri()).write(codex_home.path())?;
    let thread_id = create_fake_paginated_rollout(
        codex_home.path(),
        "2025-01-01T00-00-00",
        "2025-01-01T00:00:00Z",
        "owned",
        Some("mock_provider"),
        /*git_info*/ None,
    )?;
    let mut owner = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let _: ThreadResumeResponse = owner
        .request(|request_id| ClientRequest::ThreadResume {
            request_id,
            params: ThreadResumeParams {
                thread_id: thread_id.clone(),
                exclude_turns: true,
                ..Default::default()
            },
        })
        .await?;

    let mut other = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;
    let request_id = other
        .send_thread_delete_request(ThreadDeleteParams {
            thread_id: thread_id.clone(),
        })
        .await?;
    let error: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        other.read_stream_until_error_message(RequestId::Integer(request_id)),
    )
    .await??;
    assert_eq!(error.error.code, -32600);
    assert_eq!(
        error.error.message,
        format!("thread {thread_id} already has an active writer")
    );
    timeout(DEFAULT_READ_TIMEOUT, owner.shutdown_gracefully()).await??;
    let _: ThreadDeleteResponse = other
        .request(|request_id| ClientRequest::ThreadDelete {
            request_id,
            params: ThreadDeleteParams { thread_id },
        })
        .await?;
    Ok(())
}

#[tokio::test]
async fn thread_delete_deletes_spawned_descendants() -> Result<()> {
    let codex_home = TempDir::new()?;

    let parent_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 0, "parent")?;
    let child_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 1, "child")?;
    let grandchild_id =
        create_delete_test_rollout(codex_home.path(), /*minute*/ 2, "grandchild")?;

    let state_db = StateRuntime::init(
        codex_state::SqliteConfig::new_for_testing(codex_home.path().abs()),
        "mock_provider".into(),
    )
    .await?;
    let parent_thread_id = ThreadId::from_string(&parent_id)?;
    let child_thread_id = ThreadId::from_string(&child_id)?;
    let grandchild_thread_id = ThreadId::from_string(&grandchild_id)?;

    for (parent, child, status) in [
        (
            parent_thread_id,
            child_thread_id,
            DirectionalThreadSpawnEdgeStatus::Closed,
        ),
        (
            child_thread_id,
            grandchild_thread_id,
            DirectionalThreadSpawnEdgeStatus::Open,
        ),
    ] {
        state_db
            .upsert_thread_spawn_edge(parent, child, status)
            .await?;
    }

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .build_initialized()
        .await?;

    let _: ThreadDeleteResponse = mcp
        .request(|request_id| ClientRequest::ThreadDelete {
            request_id,
            params: ThreadDeleteParams {
                thread_id: parent_id.clone(),
            },
        })
        .await?;

    let mut deleted_ids = Vec::new();
    for _ in 0..3 {
        let deleted_notification: ThreadDeletedNotification = timeout(
            DEFAULT_READ_TIMEOUT,
            mcp.read_notification("thread/deleted"),
        )
        .await??;
        deleted_ids.push(deleted_notification.thread_id);
    }
    assert_eq!(deleted_ids, vec![grandchild_id, child_id, parent_id]);

    for thread_id in [parent_thread_id, child_thread_id, grandchild_thread_id] {
        let rollout_path = find_thread_path_by_id_str(
            codex_home.path(),
            &thread_id.to_string(),
            /*state_db_ctx*/ None,
        )
        .await?;
        assert!(
            rollout_path.is_none(),
            "expected active rollout for {thread_id} to be deleted"
        );
    }
    assert_eq!(
        state_db
            .list_thread_spawn_descendants(parent_thread_id)
            .await?,
        Vec::<ThreadId>::new()
    );
    Ok(())
}

#[tokio::test]
async fn thread_delete_preflights_external_fork_references_for_spawned_subtrees() -> Result<()> {
    let codex_home = TempDir::new()?;

    let parent_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 0, "parent")?;
    let child_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 1, "child")?;
    let external_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 2, "external")?;
    let parent_thread_id = ThreadId::from_string(&parent_id)?;
    let child_thread_id = ThreadId::from_string(&child_id)?;
    let external_thread_id = ThreadId::from_string(&external_id)?;
    let parent_path = find_thread_path_by_id_str(
        codex_home.path(),
        &parent_thread_id.to_string(),
        /*state_db_ctx*/ None,
    )
    .await?
    .expect("parent rollout path");
    let external_path = find_thread_path_by_id_str(
        codex_home.path(),
        &external_thread_id.to_string(),
        /*state_db_ctx*/ None,
    )
    .await?
    .expect("external rollout path");
    let mut external_meta: serde_json::Value = serde_json::from_str(
        std::fs::read_to_string(external_path.as_path())?
            .lines()
            .next()
            .expect("external session metadata"),
    )?;
    external_meta["payload"]["history_base"] = serde_json::to_value(HistoryPosition {
        thread_id: parent_thread_id,
        end_ordinal_exclusive: 1,
        end_byte_offset: std::fs::metadata(parent_path.as_path())?.len(),
    })?;
    std::fs::write(external_path.as_path(), format!("{external_meta}\n"))?;

    let state_db = StateRuntime::init(
        SqliteConfig::new_for_testing(codex_home.path().abs()),
        "mock_provider".into(),
    )
    .await?;
    state_db
        .upsert_thread_spawn_edge(
            parent_thread_id,
            child_thread_id,
            DirectionalThreadSpawnEdgeStatus::Closed,
        )
        .await?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .build_initialized()
        .await?;

    let delete_id = mcp
        .send_thread_delete_request(ThreadDeleteParams {
            thread_id: parent_id.clone(),
        })
        .await?;
    let delete_err: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(delete_id)),
    )
    .await??;
    assert_eq!(
        delete_err.error.message,
        format!("cannot delete thread {parent_thread_id}: forked history still references it")
    );

    for thread_id in [parent_thread_id, child_thread_id, external_thread_id] {
        assert!(
            find_thread_path_by_id_str(
                codex_home.path(),
                &thread_id.to_string(),
                /*state_db_ctx*/ None,
            )
            .await?
            .is_some(),
            "expected rollout for {thread_id} to remain"
        );
    }
    assert_eq!(
        state_db
            .list_thread_spawn_descendants(parent_thread_id)
            .await?,
        vec![child_thread_id]
    );
    Ok(())
}

fn create_delete_test_rollout(codex_home: &Path, minute: u8, preview: &str) -> Result<String> {
    create_fake_rollout(
        codex_home,
        &format!("2025-01-01T00-{minute:02}-00"),
        &format!("2025-01-01T00:{minute:02}:00Z"),
        preview,
        Some("mock_provider"),
        /*git_info*/ None,
    )
}

#[tokio::test]
async fn thread_delete_handles_live_threads_before_rollout_exists() -> Result<()> {
    let codex_home = TempDir::new()?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build_initialized()
        .await?;

    let persisted_thread = mcp.start_thread(ThreadStartParams::default()).await?.thread;
    let rollout_path = find_thread_path_by_id_str(
        codex_home.path(),
        &persisted_thread.id,
        /*state_db_ctx*/ None,
    )
    .await?;
    assert_eq!(rollout_path, None);

    let _: ThreadDeleteResponse = mcp
        .request(|request_id| ClientRequest::ThreadDelete {
            request_id,
            params: ThreadDeleteParams {
                thread_id: persisted_thread.id,
            },
        })
        .await?;

    let ThreadStartResponse { thread, .. } = mcp
        .start_thread(ThreadStartParams {
            ephemeral: Some(true),
            ..Default::default()
        })
        .await?;

    let delete_id = mcp
        .send_thread_delete_request(ThreadDeleteParams {
            thread_id: thread.id.clone(),
        })
        .await?;
    let delete_err: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(delete_id)),
    )
    .await??;
    let expected_message = format!(
        "thread is not persisted and cannot be deleted: {}",
        thread.id
    );
    assert_eq!(delete_err.error.message, expected_message);

    let ThreadLoadedListResponse { mut data, .. } = mcp
        .request(|request_id| ClientRequest::ThreadLoadedList {
            request_id,
            params: ThreadLoadedListParams::default(),
        })
        .await?;
    data.sort();
    assert_eq!(data, vec![thread.id]);

    Ok(())
}

/// A ledger attempt owned by `thread_id`, recorded now with the ledger's
/// checkpoint `behind_ms` ahead of the clock (a clock that stepped back).
async fn record_accounting_attempt(
    state_db: &StateRuntime,
    thread_id: ThreadId,
    behind_ms: i64,
) -> Result<()> {
    let rollout_path = find_thread_path_by_id_str(
        state_db.sqlite().home(),
        &thread_id.to_string(),
        /*state_db_ctx*/ None,
    )
    .await?
    .expect("rollout path");
    state_db
        .upsert_thread(
            &ThreadMetadataBuilder::new(
                thread_id,
                rollout_path,
                chrono::Utc::now(),
                SessionSource::Cli,
            )
            .build("mock_provider"),
        )
        .await?;
    let now = chrono::Utc::now().timestamp_millis();
    let store = AccountingStore::open(state_db, now + behind_ms).await?;
    let attempt: Attempt = serde_json::from_value(serde_json::json!({
        "attempt_id": uuid::Uuid::new_v4(), "request_id": uuid::Uuid::new_v4(),
        "thread_id": thread_id, "turn": "fixture", "retry_of": null,
        "provider": "synthetic", "model": "fixture", "scope": uuid::Uuid::nil(),
        "dialect": "NativeAnthropic", "dispatched_at_ms": now
    }))?;
    store.admit(thread_id, &attempt, &[], AsOf::Now).await?;
    Ok(())
}

async fn ledger_counts(state_db: &StateRuntime) -> Result<(i64, i64, i64)> {
    let pool = state_db
        .sqlite()
        .open_read_write_pool(&state_db.sqlite().state_db_path())
        .await?;
    let counts = sqlx::query_as(
        "SELECT (SELECT count(*) FROM draft_accounting_attempts),
                (SELECT count(*) FROM draft_accounting_tombstones),
                (SELECT completed_as_of_ms FROM draft_accounting_retention_checkpoint)",
    )
    .fetch_one(&pool)
    .await?;
    pool.close().await;
    Ok(counts)
}

/// #308: with the clock behind the ledger checkpoint, deleting failed as a
/// backward checkpoint after the rollout was already gone. It now succeeds
/// and the deleted spend is left as a deletion tombstone.
#[tokio::test]
async fn thread_delete_tolerates_a_clock_behind_the_accounting_checkpoint() -> Result<()> {
    for behind_ms in [58_000, 6 * 86_400_000] {
        let codex_home = TempDir::new()?;
        let thread_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 0, "skewed")?;
        let owner = ThreadId::from_string(&thread_id)?;
        let state_db = StateRuntime::init(
            SqliteConfig::new_for_testing(codex_home.path().abs()),
            "mock_provider".into(),
        )
        .await?;
        record_accounting_attempt(&state_db, owner, behind_ms).await?;
        let (_, _, checkpoint) = ledger_counts(&state_db).await?;
        assert!(checkpoint > chrono::Utc::now().timestamp_millis());

        let mut mcp = TestAppServer::builder()
            .with_codex_home(codex_home.path())
            .without_auto_env()
            .build_initialized()
            .await?;
        assert!(
            ledger_counts(&state_db).await?.2 > chrono::Utc::now().timestamp_millis(),
            "the clock is still behind the checkpoint when the delete is sent"
        );
        let _: ThreadDeleteResponse = mcp
            .request(|request_id| ClientRequest::ThreadDelete {
                request_id,
                params: ThreadDeleteParams {
                    thread_id: thread_id.clone(),
                },
            })
            .await?;

        assert_eq!(
            find_thread_path_by_id_str(codex_home.path(), &thread_id, /*state_db_ctx*/ None)
                .await?,
            None
        );
        let (attempts, tombstones, after) = ledger_counts(&state_db).await?;
        assert_eq!((attempts, tombstones), (0, 1));
        assert!(after >= checkpoint, "the checkpoint never moves backward");
        assert!(state_db.get_thread(owner).await?.is_none());
    }
    Ok(())
}

/// #308: when the ledger cannot be updated, nothing is deleted and the error
/// says so; once it can, the same delete finishes.
#[tokio::test]
async fn thread_delete_leaves_the_rollout_when_accounting_fails() -> Result<()> {
    let codex_home = TempDir::new()?;
    let thread_id = create_delete_test_rollout(codex_home.path(), /*minute*/ 0, "kept")?;
    let owner = ThreadId::from_string(&thread_id)?;
    let state_db = StateRuntime::init(
        SqliteConfig::new_for_testing(codex_home.path().abs()),
        "mock_provider".into(),
    )
    .await?;
    record_accounting_attempt(&state_db, owner, /*behind_ms*/ 0).await?;
    let pool = state_db
        .sqlite()
        .open_read_write_pool(&state_db.sqlite().state_db_path())
        .await?;
    sqlx::query(
        "CREATE TRIGGER fixture_reject_tombstone BEFORE INSERT ON draft_accounting_tombstones \
         BEGIN SELECT RAISE(ABORT, 'fixture-ledger-failure'); END",
    )
    .execute(&pool)
    .await?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .without_auto_env()
        .build_initialized()
        .await?;
    let delete_id = mcp
        .send_thread_delete_request(ThreadDeleteParams {
            thread_id: thread_id.clone(),
        })
        .await?;
    let error: JSONRPCError = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(delete_id)),
    )
    .await??;
    let message = error.error.message;
    assert!(
        message.starts_with(&format!(
            "could not delete conversation {thread_id}: its local records cannot be updated right now, \
             so it was left as it is. Try again;"
        )),
        "{message}"
    );
    assert!(message.contains("fixture-ledger-failure"), "{message}");
    assert!(
        find_thread_path_by_id_str(codex_home.path(), &thread_id, /*state_db_ctx*/ None)
            .await?
            .is_some(),
        "the rollout stays"
    );
    assert_eq!(
        ledger_counts(&state_db).await?.0,
        1,
        "the ledger is unchanged"
    );

    sqlx::query("DROP TRIGGER fixture_reject_tombstone")
        .execute(&pool)
        .await?;
    pool.close().await;
    let _: ThreadDeleteResponse = mcp
        .request(|request_id| ClientRequest::ThreadDelete {
            request_id,
            params: ThreadDeleteParams {
                thread_id: thread_id.clone(),
            },
        })
        .await?;
    assert_eq!(
        find_thread_path_by_id_str(codex_home.path(), &thread_id, /*state_db_ctx*/ None).await?,
        None
    );
    assert_eq!(ledger_counts(&state_db).await?.0, 0);
    Ok(())
}
