//! #616/#872: how rollout appends and thread shutdown update what exec
//! waits for at exit. The thread store fails writes with the error the local
//! store reports for EPERM while `denied` is set.

use super::handlers;
use super::tests::make_session_and_context;
use super::tests::open_thread_persistence;
use codex_protocol::ThreadId;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::StreamErrorEvent;
use codex_protocol::protocol::UserMessageEvent;
use codex_thread_store::AppendThreadItemsParams;
use codex_thread_store::ArchiveThreadParams;
use codex_thread_store::CreateThreadParams;
use codex_thread_store::DeleteThreadParams;
use codex_thread_store::ListThreadsParams;
use codex_thread_store::LoadThreadHistoryParams;
use codex_thread_store::LocalThreadStore;
use codex_thread_store::LocalThreadStoreConfig;
use codex_thread_store::ReadThreadByRolloutPathParams;
use codex_thread_store::ReadThreadParams;
use codex_thread_store::ResumeThreadParams;
use codex_thread_store::StoredThread;
use codex_thread_store::StoredThreadHistory;
use codex_thread_store::ThreadPage;
use codex_thread_store::ThreadStore;
use codex_thread_store::ThreadStoreError;
use codex_thread_store::ThreadStoreFuture;
use codex_thread_store::UpdateThreadMetadataParams;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// A local store whose writes fail like an unreadable home while `denied`.
/// `as_any` exposes the local store, so the rollout path still resolves.
struct DeniableStore {
    local: LocalThreadStore,
    denied: AtomicBool,
}

impl DeniableStore {
    /// The trait methods (some inherent ones share their names).
    fn local(&self) -> &dyn ThreadStore {
        &self.local
    }

    /// A failed write, as the local store reports EPERM, while denied.
    fn denial(&self) -> Option<ThreadStoreFuture<'static, ()>> {
        self.denied.load(Ordering::SeqCst).then(|| {
            Box::pin(async {
                Err(ThreadStoreError::Internal {
                    message: "Operation not permitted (os error 1)".to_string(),
                })
            }) as ThreadStoreFuture<'static, ()>
        })
    }
}

impl ThreadStore for DeniableStore {
    fn as_any(&self) -> &dyn std::any::Any {
        self.local().as_any()
    }

    fn create_thread(&self, params: CreateThreadParams) -> ThreadStoreFuture<'_, ()> {
        self.local().create_thread(params)
    }

    fn resume_thread(&self, params: ResumeThreadParams) -> ThreadStoreFuture<'_, ()> {
        self.local().resume_thread(params)
    }

    fn append_items(&self, params: AppendThreadItemsParams) -> ThreadStoreFuture<'_, ()> {
        self.denial()
            .unwrap_or_else(|| self.local().append_items(params))
    }

    fn persist_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.denial()
            .unwrap_or_else(|| self.local().persist_thread(thread_id))
    }

    fn flush_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.denial()
            .unwrap_or_else(|| self.local().flush_thread(thread_id))
    }

    fn shutdown_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.denial()
            .unwrap_or_else(|| self.local().shutdown_thread(thread_id))
    }

    fn discard_thread(&self, thread_id: ThreadId) -> ThreadStoreFuture<'_, ()> {
        self.local().discard_thread(thread_id)
    }

    fn load_history(
        &self,
        params: LoadThreadHistoryParams,
    ) -> ThreadStoreFuture<'_, StoredThreadHistory> {
        self.local().load_history(params)
    }

    fn read_thread(&self, params: ReadThreadParams) -> ThreadStoreFuture<'_, StoredThread> {
        self.local().read_thread(params)
    }

    fn read_thread_by_rollout_path(
        &self,
        params: ReadThreadByRolloutPathParams,
    ) -> ThreadStoreFuture<'_, StoredThread> {
        self.local().read_thread_by_rollout_path(params)
    }

    fn list_threads(&self, params: ListThreadsParams) -> ThreadStoreFuture<'_, ThreadPage> {
        self.local().list_threads(params)
    }

    fn update_thread_metadata(
        &self,
        params: UpdateThreadMetadataParams,
    ) -> ThreadStoreFuture<'_, StoredThread> {
        self.local().update_thread_metadata(params)
    }

    fn archive_thread(&self, params: ArchiveThreadParams) -> ThreadStoreFuture<'_, ()> {
        self.local().archive_thread(params)
    }

    fn unarchive_thread(&self, params: ArchiveThreadParams) -> ThreadStoreFuture<'_, StoredThread> {
        self.local().unarchive_thread(params)
    }

    fn delete_thread(&self, params: DeleteThreadParams) -> ThreadStoreFuture<'_, ()> {
        self.local().delete_thread(params)
    }
}

fn user_message(text: &str) -> RolloutItem {
    RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
        message: text.to_string(),
        ..Default::default()
    }))
}

/// One test, as the failures it observes are process-wide.
#[tokio::test]
async fn unsaved_rollout_tracks_written_appends_and_ends_with_the_thread() {
    let (mut session, _turn_context) = make_session_and_context().await;
    let config = session.get_config().await;
    let store = Arc::new(DeniableStore {
        local: LocalThreadStore::new(
            LocalThreadStoreConfig::from_config(config.as_ref()),
            /*state_db*/ None,
        ),
        denied: AtomicBool::new(false),
    });
    session.services.thread_store = store.clone();
    open_thread_persistence(&mut session).await;
    let thread = session.thread_id;
    let state = || crate::home_access::unsaved_state(thread);
    assert_eq!(state(), (false, None));

    // The home denies the append: the thread is unsaved from now on.
    store.denied.store(true, Ordering::SeqCst);
    session
        .persist_rollout_items(&[user_message("lost?")])
        .await;
    let (unsaved, since) = state();
    assert!(unsaved && since.is_some(), "{:?}", state());

    // Access returns, but an append of events that are never written shows
    // nothing about the home: the thread stays unsaved.
    store.denied.store(false, Ordering::SeqCst);
    session
        .persist_rollout_items(&[RolloutItem::EventMsg(EventMsg::StreamError(
            StreamErrorEvent {
                message: "reconnecting".to_string(),
                codex_error_info: None,
                additional_details: None,
            },
        ))])
        .await;
    assert_eq!(state(), (true, since));

    // A written append saves it and ends the run.
    session
        .persist_rollout_items(&[user_message("saved")])
        .await;
    assert_eq!(state(), (false, None));

    // #872: a thread whose last append failed and that then shuts down
    // (its shutdown write failing too) leaves nothing behind for the exit
    // wait, so a later outage starts its own run.
    store.denied.store(true, Ordering::SeqCst);
    session.persist_rollout_items(&[user_message("lost")]).await;
    assert!(state().0);
    let session = Arc::new(session);
    assert!(handlers::shutdown(&session, "sub-1".to_string()).await);
    assert_eq!(state(), (false, None));
}
