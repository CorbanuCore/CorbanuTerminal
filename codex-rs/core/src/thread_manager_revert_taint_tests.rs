//! D624 (a): `thread/revert` keeps the session flagged once untrusted content was seen, even when
//! the remaining history no longer contains that content (#2562, S3b row 572).
use super::*;
use pretty_assertions::assert_eq;

/// A manager on an in-memory store and a stopped root thread to reload, as `thread/revert` does.
async fn reverted_root() -> (
    tempfile::TempDir,
    Config,
    ThreadManager,
    Arc<AuthManager>,
    ThreadId,
) {
    let temp_dir = tempdir().expect("tempdir");
    let mut config = test_config().await;
    config.codex_home = temp_dir.path().join("codex-home").abs();
    config.cwd = config.codex_home.abs();
    config.experimental_thread_store = ThreadStoreConfig::InMemory {
        id: format!("revert-taint-{}", uuid::Uuid::new_v4()),
    };
    config
        .features
        .enable(codex_features::Feature::SourceEnvelopes)
        .expect("enable source envelopes");
    // Post-taint checks apply at the protected levels.
    config.security_level = codex_security_policy::SecurityLevel::Moderate;
    std::fs::create_dir_all(&config.codex_home).expect("create codex home");
    let auth_manager =
        AuthManager::from_auth_for_testing(CodexAuth::create_dummy_chatgpt_auth_for_testing());
    let manager = ThreadManager::new(
        &config,
        auth_manager.clone(),
        build_models_manager(&config, auth_manager.clone()),
        crate::CodexAppsToolsCache::default(),
        SessionSource::Exec,
        Arc::new(codex_exec_server::EnvironmentManager::default_for_tests()),
        empty_extension_registry(),
        Arc::new(crate::test_support::EmptyUserInstructionsProvider),
        /*analytics_events_client*/ None,
        passthrough_image_store(),
        thread_store_from_config(&config, /*state_db*/ None),
        /*agent_graph_store*/ None,
        TEST_INSTALLATION_ID.to_string(),
        /*attestation_provider*/ None,
        /*external_time_provider*/ None,
    );
    let source = manager
        .start_thread(StartThreadOptions::new(config.clone()))
        .await
        .expect("start source thread");
    source
        .thread
        .shutdown_and_wait()
        .await
        .expect("shutdown source thread");
    let _ = manager.remove_thread(&source.thread_id).await;
    (temp_dir, config, manager, auth_manager, source.thread_id)
}

/// The history left after reverting past every untrusted item: nothing without standing.
fn remaining_history(thread_id: ThreadId) -> InitialHistory {
    InitialHistory::Resumed(ResumedHistory {
        history_revision: None,
        conversation_id: thread_id,
        history: Arc::new(Vec::new()),
        rollout_path: None,
    })
}

fn post_taint_generation(thread: &NewThread) -> Option<u64> {
    thread
        .thread
        .session
        .services
        .model_client()
        .post_taint_state()
        .map(|state| state.taint_generation)
}

#[tokio::test]
async fn revert_past_untrusted_content_keeps_the_session_flagged() {
    let (_home, config, manager, auth_manager, thread_id) = reverted_root().await;
    let reloaded = manager
        .resume_thread_after_revert(
            config,
            remaining_history(thread_id),
            auth_manager,
            /*parent_trace*/ None,
            ClientMcpExtensions::default(),
            /*untrusted_content_seen*/ true,
        )
        .await
        .expect("reload after revert");
    assert_eq!(reloaded.thread_id, thread_id);
    assert!(reloaded.thread.untrusted_content_seen());
    // Grants and approvals are judged against this generation: post-taint checks stay on and
    // anything recorded before the reload (generation 0) is stale.
    assert!(
        post_taint_generation(&reloaded).is_some_and(|generation| generation > 0),
        "{:?}",
        post_taint_generation(&reloaded)
    );
    reloaded
        .thread
        .shutdown_and_wait()
        .await
        .expect("shutdown reloaded thread");
}

#[tokio::test]
async fn revert_of_a_clean_session_stays_clean() {
    let (_home, config, manager, auth_manager, thread_id) = reverted_root().await;
    let reloaded = manager
        .resume_thread_after_revert(
            config,
            remaining_history(thread_id),
            auth_manager,
            /*parent_trace*/ None,
            ClientMcpExtensions::default(),
            /*untrusted_content_seen*/ false,
        )
        .await
        .expect("reload after revert");
    assert!(!reloaded.thread.untrusted_content_seen());
    assert!(
        !post_taint_generation(&reloaded).is_some_and(|generation| generation > 0),
        "{:?}",
        post_taint_generation(&reloaded)
    );
    reloaded
        .thread
        .shutdown_and_wait()
        .await
        .expect("shutdown reloaded thread");
}
