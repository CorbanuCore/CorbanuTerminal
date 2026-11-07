//! Shared App fixtures for app submodule unit tests.
//!
//! This module keeps heavyweight `App` construction and config-inspection helpers available to
//! focused sibling test modules without making `app/tests.rs` the only practical place to test
//! app-owned behavior.
//! Return the large fixture on the heap so async scenarios do not embed it in their stack frames.

use super::*;
use crate::chatwidget::tests::make_chatwidget_manual_with_sender;
use crate::test_support::boxed_future;
use codex_models_manager::test_support::construct_model_info_offline_for_tests;
use codex_models_manager::test_support::get_model_offline_for_tests;

<<<<<<< HEAD
pub(super) fn pending_confirmation(
    thread_id: ThreadId,
) -> permission_confirmation::PendingPermissionConfirmation {
    permission_confirmation::PendingPermissionConfirmation {
        selection_id: uuid::Uuid::new_v4(),
        thread_id,
        label: "pending".into(),
        persist_reviewer: None,
        requested: Default::default(),
        observed: false,
        applied: false,
    }
}

pub(super) async fn make_test_app() -> App {
    let (chat_widget, app_event_tx, _rx, _op_rx) = make_chatwidget_manual_with_sender().await;
||||||| 413492cd6c
pub(super) async fn make_test_app() -> App {
    let (chat_widget, app_event_tx, _rx, _op_rx) = make_chatwidget_manual_with_sender().await;
=======
pub(crate) async fn make_test_app() -> Box<App> {
    let (mut chat_widget, app_event_tx, _rx, _op_rx) =
        boxed_future!(make_chatwidget_manual_with_sender()).await;
    let test_codex_home = chat_widget.test_codex_home.take();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let config = chat_widget.config_ref().clone();
    let file_search = FileSearchManager::new(config.cwd.to_path_buf(), app_event_tx.clone());
    let model = get_model_offline_for_tests(config.model.as_deref());
    let session_telemetry = test_session_telemetry(&config, model.as_str());

    Box::new(App {
        feature_write_lock: Arc::default(),
        config_notification_test_generation: 0,
        config_notification_test_pending: None,
        model_catalog: chat_widget.model_catalog(),
        session_telemetry,
        terminal_size_tags: Vec::new(),
        app_event_tx,
        chat_widget,
        workspace_command_runner: None,
        launch_cwd: config.cwd.to_path_buf(),
        runtime_working_directory_override: None,
        local_settings: crate::local_settings::LocalSettings::from(&config),
        config,
        active_deferred_provider_setup: None,
        shared_provider_setup_session: None,
        shared_provider_status_host: None,
        shared_provider_account_auth_host: None,
        provider_management_host: None,
        provider_management_generation: 0,
        pending_wallet_create_deferred: None,
        state_db: None,
        cli_kv_overrides: Vec::new(),
        harness_overrides: ConfigOverrides::default(),
        loader_overrides: LoaderOverrides::without_managed_config_for_tests(),
        cloud_config_bundle: CloudConfigBundleLoader::default(),
        runtime_approval_policy_override: None,
        runtime_approvals_reviewer_override: None,
        runtime_permission_profile_override: None,
<<<<<<< HEAD
        pending_permission_confirmation: None,
||||||| 413492cd6c
=======
        pending_server_profiles: HashMap::new(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        file_search,
        transcript_cells: Vec::new(),
<<<<<<< HEAD
        claude_pane_transcript_cells: HashMap::new(),
||||||| 413492cd6c
=======
        native_history: Default::default(),
        turn_tips: Default::default(),
        transcript_view: Default::default(),
        conversation_frame: Default::default(),
        last_rendered_history_tail: None,
        last_thread_usage_status_cell: None,
        pending_thread_usage_history_refresh: false,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        overlay: None,
        retained_analytics: None,
        deferred_history_lines: Vec::new(),
        has_emitted_history_lines: false,
        transcript_reflow: TranscriptReflowState::default(),
        initial_history_replay_buffer: None,
        pending_thread_switch_resets: 0,
        scrollback_has_older_history: false,
        enhanced_keys_supported: false,
        keymap: crate::keymap::RuntimeKeymap::defaults(),
        key_chord_matcher: crate::keymap::KeyChordMatcher::default(),
        routed_key_activation: None,
        commit_animation: None,
        status_line_invalid_items_warned: Arc::new(AtomicBool::new(false)),
        terminal_title_invalid_items_warned: Arc::new(AtomicBool::new(false)),
        skill_load_warnings: SkillLoadWarningState::default(),
        skills_refresh_generation: Arc::new(AtomicU64::new(0)),
        backtrack: BacktrackState::default(),
        backtrack_render_pending: false,
        feedback: codex_feedback::CodexFeedback::new(),
        feedback_audience: FeedbackAudience::External,
        environment_manager: Arc::new(EnvironmentManager::default_for_tests()),
        app_server_target: crate::AppServerTarget::Embedded,
        pending_right_click_paste: None,
        right_click_paste_environment: super::right_click_paste::PasteEnvironment {
            primary: false,
            platform_default: true,
            ssh: false,
            wsl: false,
            vscode: crate::tui::VscodeDetection::Other,
        },
        reconnect: Default::default(),
        daemon_cli_executable: None,
        pending_update_action: None,
        daemon_restart: None,
        pending_shutdown_exit_thread_id: None,
        windows_sandbox: WindowsSandboxState::default(),
        thread_event_channels: HashMap::new(),
        pending_realtime_speech_replay: HashMap::new(),
        pending_realtime_transcript_replay: HashMap::new(),
        realtime_replay_order: VecDeque::new(),
        background_voice: None,
        background_voice_error: None,
        temporary_structured_requests: HashMap::new(),
        pending_thread_titles: HashMap::new(),
        thread_event_listener_tasks: HashMap::new(),
        agent_navigation: AgentNavigationState::default(),
<<<<<<< HEAD
        spawn_parent_by_thread: HashMap::new(),
        spawn_parent_by_node: HashMap::new(),
        spawn_native_runtime_by_node: HashMap::new(),
        spawn_native_endpoint_by_node: HashMap::new(),
        spawn_crew: None,
        spawn_legacy_read_only: false,
        spawn_status_by_thread: HashMap::new(),
        spawn_waiting_for_agents_by_thread: HashMap::new(),
        spawn_parent_reports_by_node: HashMap::new(),
        spawn_dispatch_acks_by_target_task: HashMap::new(),
        spawn_next_dispatch_seq: 1,
        spawn_processed_dispatch_seq_ids: HashSet::new(),
        spawn_processed_dispatch_origins: HashSet::new(),
        spawn_processed_terminal_turns: HashSet::new(),
        spawn_auto_loop_state_by_node: HashMap::new(),
        spawn_operator_input_seen: false,
        kill_switch_held_claude_tasks: Vec::new(),
        kill_switch_held_native_tasks: Vec::new(),
        kill_switch_known_on: false,
        spawn_quarantine_notified_by_node: HashSet::new(),
        spawn_context_left_by_thread: HashMap::new(),
        spawn_last_report_seq_by_node: HashMap::new(),
        spawn_last_dispatch_seq_by_node: HashMap::new(),
        spawn_last_event_at_by_node: HashMap::new(),
        spawn_nazgul_pane_id: None,
        spawn_nazgul_rebind_required: false,
        orchestrate_whips: Box::new(HashMap::new()),
        orchestrate_next_whip_seq: 0,
        orchestrate_now_override: None,
        orchestrate_idle_generation_by_target: Box::new(HashMap::new()),
||||||| 413492cd6c
=======
        agents_overview: Default::default(),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        side_threads: HashMap::new(),
        claude_panes: Default::default(),
        abandoned_side_threads: HashSet::new(),
        active_thread_id: None,
        active_thread_rx: None,
        primary_thread_id: None,
        last_subagent_backfill_attempt: None,
        primary_session_configured: None,
        pending_primary_events: VecDeque::new(),
        pending_app_server_requests: PendingAppServerRequests::default(),
        dynamic_tool_status_updates: tokio::sync::broadcast::channel(/*capacity*/ 64).0,
        dynamic_tool_tasks: HashMap::new(),
        pending_startup_thread_start: false,
        pending_server_version_notice: None,
        pending_open_resume_picker: false,
        pending_working_directory_change: None,
        pending_start_managed_worktree: None,
        pending_managed_worktree_creation: false,
        pending_managed_worktree_created: None,
        pending_managed_worktree_transition: None,
        pending_managed_worktree_attach: None,
        startup_protected_input_boundary: false,
        startup_pending_protected_request: false,
        account_email_request_id: None,
        rate_limit_hard_stop_generation: 0,
        rate_limit_refresh_state: Default::default(),
        pending_mcp_login_start: None,
        active_mcp_login_ids: HashMap::new(),
        pending_plugin_enabled_writes: HashMap::new(),
        pending_hook_enabled_writes: HashMap::new(),
        recap: recap::RecapState::default(),
        _test_codex_home: test_codex_home,
    })
}

fn test_session_telemetry(config: &Config, model: &str) -> SessionTelemetry {
    let model_info =
        construct_model_info_offline_for_tests(model, &config.to_models_manager_config());
    SessionTelemetry::new(
        ThreadId::new(),
        model,
        model_info.slug.as_str(),
        /*account_id*/ None,
        /*account_email*/ None,
        /*auth_mode*/ None,
        "test_originator".to_string(),
        /*log_user_prompts*/ false,
        "test".to_string(),
        serde_json::from_value(serde_json::json!("cli"))
            .expect("cli session source should deserialize"),
    )
}

pub(super) fn app_enabled_in_effective_config(config: &Config, app_id: &str) -> Option<bool> {
    config
        .config_layer_stack
        .effective_config()
        .as_table()
        .and_then(|table| table.get("apps"))
        .and_then(TomlValue::as_table)
        .and_then(|apps| apps.get(app_id))
        .and_then(TomlValue::as_table)
        .and_then(|app| app.get("enabled"))
        .and_then(TomlValue::as_bool)
}
