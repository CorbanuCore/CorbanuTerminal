//! Startup bootstrap, draft handoff, and protected-input orchestration for the TUI app.
//!
//! Owns the main app run loop from app-server bootstrap through terminal shutdown. Startup input
//! remains isolated from protected interactive requests until the initialized composer owns it.
//! Queued resume history replaces the provisional loading frame only when it is ready to render.
//! Explicit local launch permissions remain runtime overrides across new sessions and reconnects.

use super::reconnect::ReconnectState;
use super::startup_bootstrap::bootstrap_server_owned_start;
use super::startup_bootstrap::uses_server_owned_fresh_bootstrap;
use super::*;
use crate::session_start::SessionStartAction;
use crate::session_start::SessionStartConfig;
use crate::session_start::SessionStartOutcome;
use crate::session_start::cancel_session_start;
use crate::session_start::complete_session_start;
use crate::session_start::session_start_refusal_exit;
use crate::unarchive_prompt::run_unarchive_prompt;

/// Upstream's prompt edit reports these after the server already reverted history; the TUI must
/// stop rather than keep accepting input against the old transcript.
fn server_history_changed(err: &color_eyre::eyre::Report) -> bool {
    err.chain().any(|cause| {
        let message = cause.to_string();
        message.starts_with("prompt edit could not be confirmed")
            || message.starts_with("history was reverted, but refreshing the session timed out")
    })
}

fn spawn_startup_thread_start(
    app_server: &AppServerSession,
    local_settings: crate::local_settings::LocalSettings,
    config: Config,
    launch_choices: crate::app_server_session::StartupLaunchChoices,
    app_event_tx: AppEventSender,
    worktree: Option<crate::ManagedTuiWorktree>,
) {
    let backend_id = app_server.backend_id();
    let request_handle = app_server.request_handle();
    let thread_params_mode = app_server.thread_params_mode();
    let remote_cwd_override = app_server.remote_cwd_override().map(Path::to_path_buf);
    let thread_tool_transport = app_server.thread_tool_transport();
    tokio::spawn(async move {
        let result = crate::app_server_session::start_thread_with_request_handle(
            request_handle,
            &local_settings,
            config,
            thread_params_mode,
            remote_cwd_override,
            thread_tool_transport,
            launch_choices,
        )
        .await
        .and_then(|started| {
            if let Some(worktree) = worktree.as_ref() {
                worktree.bind(started.session.thread_id)?;
            }
            Ok(started)
        });
        app_event_tx.send(AppEvent::StartupThreadStarted { backend_id, result });
    });
}

#[derive(Default)]
pub(super) struct FreshStartupDefaults {
    pub(super) server_defaults_read: bool,
    pub(super) prompt_windows_sandbox: bool,
    pub(super) approval_policy: Option<codex_protocol::protocol::AskForApproval>,
}

pub(super) async fn prepare_fresh_startup_config(
    config: &mut Config,
    app_server: &AppServerSession,
    app_server_target: &AppServerTarget,
    cli_kv_overrides: &[(String, TomlValue)],
    harness_overrides: &ConfigOverrides,
    environments: &EnvironmentManager,
) -> Result<FreshStartupDefaults> {
    let defaults_cwd = match app_server_target {
        AppServerTarget::Embedded | AppServerTarget::LocalDaemon { .. } => config.cwd.as_path(),
        AppServerTarget::Remote { .. } => {
            app_server.remote_cwd_override().unwrap_or(Path::new("."))
        }
    };
    let defaults = crate::config_update::read_effective_config_if_supported(
        app_server.request_handle(),
        defaults_cwd,
    )
    .await?;
    let mut projectless = crate::projectless::AppliedDefaults::default();
    if let Some(defaults) = defaults.as_ref() {
        super::new_session::overlay_new_session_defaults(
            config,
            &defaults.config,
            cli_kv_overrides,
            harness_overrides,
        );
        projectless = crate::projectless::apply_defaults(
            config,
            harness_overrides,
            app_server,
            environments,
            defaults,
        );
    }
    apply_managed_new_thread_defaults(
        config,
        app_server.managed_new_thread_defaults(),
        cli_kv_overrides,
        harness_overrides,
    );
    Ok(FreshStartupDefaults {
        server_defaults_read: defaults.is_some(),
        prompt_windows_sandbox: projectless.prompt_windows_sandbox,
        approval_policy: projectless.approval_policy,
    })
}

pub(super) fn startup_model(
    config: &Config,
    bootstrap: &AppServerBootstrap,
    server_defaults_read: bool,
) -> String {
    config.model.clone().unwrap_or_else(|| {
        if server_defaults_read {
            // Legacy bootstrap paths may be seeded with local config, which can differ from a
            // cleared server model. Use the server catalog when config/read returned model: null.
            bootstrap
                .available_models
                .iter()
                .find(|model| model.is_default)
                .or_else(|| bootstrap.available_models.first())
                .map(|model| model.model.clone())
                .unwrap_or_else(|| bootstrap.default_model.clone())
        } else {
            bootstrap.default_model.clone()
        }
    })
}

impl App {
    /// Keep explicit local launch choices available for new sessions and reconnect recovery.
    pub(super) fn remember_launch_permissions(&mut self) {
        if self.app_server_target.thread_params_mode()
            == crate::app_server_session::ThreadParamsMode::Remote
        {
            return;
        }
        let selected = crate::resume_permissions::ResumePermissions::from_overrides(
            &self.config,
            &self.harness_overrides,
        );
        if selected.approval_policy {
            self.runtime_approval_policy_override = Some(RuntimeApprovalPolicyOverride::Explicit(
                self.config.permissions.approval_policy.value().into(),
            ));
        }
        if selected.approvals_reviewer {
            self.runtime_approvals_reviewer_override = Some(self.config.approvals_reviewer);
        }
        if selected.profile {
            let profile = RuntimePermissionProfileOverride::from_config(&self.config);
            // The server owns constrained profiles that legacy turn parameters cannot express.
            self.runtime_permission_profile_override =
                (profile.active_permission_profile.is_some()
                    || crate::app_server_session::turn_permissions_overrides(
                        TurnPermissionsOverride::LegacySandbox(
                            self.config.permissions.effective_permission_profile(),
                        ),
                        self.config.cwd.as_path(),
                    )
                    .is_ok())
                .then_some(profile);
        }
    }

    /// Keep the provisional loading frame until queued history reaches the owned transcript.
    /// Visible startup decisions and the agent overview must still render immediately.
    pub(super) fn render_startup_frame(
        &mut self,
        tui: &mut tui::Tui,
        app_event_rx: &mpsc::UnboundedReceiver<AppEvent>,
    ) -> Result<()> {
        if tui.is_owned_screen() && !app_event_rx.is_empty() && !self.chat_widget.has_active_view()
        {
            return Ok(());
        }
        self.chat_widget.pre_draw_tick();
        self.render_chat_widget_frame(tui, tui.terminal.last_known_screen_size)?;
        if self.chat_widget.has_active_modal() && self.startup_protected_input_boundary {
            self.discard_startup_modal_input(tui)?;
        }
        Ok(())
    }

    /// Keep a visible modal closed to input if the terminal has not settled yet. The next input
    /// event retries the drain instead of acting on the modal; server events can continue meanwhile.
    pub(super) fn discard_startup_modal_input(&mut self, tui: &mut tui::Tui) -> Result<()> {
        match tui.discard_pending_input_before_interactive_screen() {
            Ok(()) => self.startup_pending_protected_request = false,
            Err(err) if err.kind() == std::io::ErrorKind::TimedOut => {
                self.startup_pending_protected_request = true;
                tracing::warn!(%err, "keeping startup modal input quarantined");
            }
            Err(err) => return Err(err.into()),
        }
        Ok(())
    }

    /// Recognizes queued requests before they become visible protected screens.
    pub(super) fn has_queued_startup_protected_request(&self) -> bool {
        self.startup_protected_input_boundary
            && (self
                .active_thread_rx
                .as_ref()
                .is_some_and(|receiver| !receiver.is_empty())
                && self
                    .active_thread_id
                    .and_then(|thread_id| self.thread_event_channels.get(&thread_id))
                    .is_none_or(|channel| {
                        // A bounded drain can leave ordinary notifications queued. Only protect
                        // input for pending requests, or when their state cannot be inspected.
                        channel.store.try_lock().map_or(/*default*/ true, |store| {
                            store.side_parent_pending_status().is_some()
                        })
                    })
                || self
                    .pending_primary_events
                    .iter()
                    .any(|event| matches!(event, ThreadBufferedEvent::Request(_))))
    }

    /// Wait until visible and queued startup decisions cannot consume a delayed OSC response.
    #[cfg(any(windows, test))]
    pub(super) fn ready_for_terminal_color_probe(&self, has_pending_app_events: bool) -> bool {
        !has_pending_app_events
            && !self.chat_widget.has_active_view()
            && !self.startup_pending_protected_request
            && !self.has_queued_startup_protected_request()
            && !self.chat_widget.has_pending_protected_request()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn run(
        tui: &mut tui::Tui,
        initial_backends: super::backend_sessions::InitialBackendSessions,
        mut config: Config,
        launch_cwd: PathBuf,
        cli_kv_overrides: Vec<(String, TomlValue)>,
        harness_overrides: ConfigOverrides,
        loader_overrides: LoaderOverrides,
        cloud_config_bundle: CloudConfigBundleLoader,
        initial_prompt: Option<String>,
        initial_images: Vec<PathBuf>,
        session_selection: SessionSelection,
        resume_fallback_notice: Option<String>,
        startup_resume_model_override: Option<ResumeModelOverride>,
        feedback: codex_feedback::CodexFeedback,
        is_first_run: bool,
        should_prompt_windows_sandbox_nux_at_startup: bool,
        app_server_target: AppServerTarget,
        state_db: Option<StateDbHandle>,
        environment_manager: Arc<EnvironmentManager>,
        startup_elapsed_before_app: Duration,
        startup_bootstrap: Option<AppServerBootstrap>,
        startup_hooks_browser: Option<HooksListEntry>,
        daemon_startup_warning: Option<String>,
        managed_daemon: bool,
        mut startup_draft: StartupDraftPump,
        managed_worktree: Option<crate::ManagedTuiWorktree>,
        daemon_cli_executable: Option<AbsolutePathBuf>,
        deferred_provider_setup: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
        startup_provider_resolution: Option<crate::startup_provider::StartupProviderResolution>,
    ) -> Result<AppExitInfo> {
        use tokio_stream::StreamExt;

        async fn shutdown_startup_sessions(sessions: impl IntoIterator<Item = AppServerSession>) {
            let shutdowns = sessions.into_iter().map(AppServerSession::shutdown);
            let shutdown_result: std::io::Result<Vec<()>> = futures::future::join_all(shutdowns)
                .await
                .into_iter()
                .collect();
            if let Err(shutdown_error) = shutdown_result {
                tracing::warn!("app-server shutdown failed: {shutdown_error}");
            }
        }

        async fn shutdown_on_startup_error(
            app_server: AppServerSession,
            other_backends: &mut Vec<AppServerSession>,
            error: impl Into<color_eyre::eyre::Report>,
        ) -> Result<AppExitInfo> {
            shutdown_startup_sessions(std::iter::once(app_server).chain(other_backends.drain(..)))
                .await;
            Err(error.into())
        }

        async fn cancel_startup_sessions(
            app_server: AppServerSession,
            other_backends: &mut Vec<AppServerSession>,
        ) -> AppExitInfo {
            let exit_info = crate::boxed_future!(cancel_session_start(app_server)).await;
            shutdown_startup_sessions(other_backends.drain(..)).await;
            exit_info
        }

        let (mut app_server, mut other_backends) = initial_backends.into_parts();
        // Preserve singleton error propagation while releasing all sessions on multi-backend exits.
        macro_rules! startup_try {
            ($result:expr) => {
                match $result {
                    Ok(value) => value,
                    Err(error) if other_backends.is_empty() => return Err(error.into()),
                    Err(error) => {
                        return shutdown_on_startup_error(app_server, &mut other_backends, error)
                            .await;
                    }
                }
            };
        }
        // Adopt actual launch ownership before constructing session-local preferences.
        startup_try!(tui.prepare_owned_screen(config.tui_fullscreen_transcript));
        let mut local_settings = crate::local_settings::LocalSettings::for_tui(&config, tui);
        let startup_started_at = Instant::now();
        if let SessionSelection::Resume(target) | SessionSelection::Fork(target) =
            &session_selection
        {
            startup_try!(target.ensure_backend(app_server.backend_id()));
        }
        let (app_event_tx, mut app_event_rx) = unbounded_channel();
        let app_event_tx = AppEventSender::new(app_event_tx);
        crate::chatwidget::agent_control::start(&app_event_tx);
        crate::chatwidget::campaign_tracker::start(&app_event_tx);
        if let Some(message) = project_config_warning(&config) {
            app_event_tx.send(AppEvent::InsertHistoryCell(Box::new(
                history_cell::StartupWarningsCell::new(vec![message]),
            )));
        }
        emit_system_bwrap_warning(&app_event_tx, &config);
        emit_legacy_windows_sandbox_warning(&app_event_tx, &config);
        tui.set_notification_settings(
            local_settings.tui.notification_settings.method,
            local_settings.tui.notification_settings.condition,
        );

        // A resumed session keeps the model, provider and effort it was saved with.
        let _startup_resume_model_override = match (
            startup_resume_model_override,
            &session_selection,
            state_db.as_ref(),
        ) {
            (Some(explicit), _, _) => Some(explicit),
            (None, SessionSelection::Resume(target), Some(state_db)) => {
                match state_db.get_thread(target.thread_id).await {
                    Ok(Some(metadata)) => apply_persisted_resume_runtime(
                        &mut config,
                        metadata.model.as_deref(),
                        metadata.model_provider.as_str(),
                        metadata.reasoning_effort.clone(),
                    ),
                    Ok(None) => None,
                    Err(error) => {
                        tracing::warn!(
                            thread_id = %target.thread_id,
                            %error,
                            "failed to load persisted runtime for startup resume"
                        );
                        None
                    }
                }
            }
            (None, _, _) => None,
        };

        let harness_overrides = startup_try!(normalize_harness_overrides_for_cwd(
            harness_overrides,
            &config.cwd
        ));
        app_server.model_provider_override = harness_overrides.model_provider.clone();
        let fresh_start = matches!(
            &session_selection,
            SessionSelection::StartFresh
                | SessionSelection::ResumePanesOnly { .. }
                | SessionSelection::Exit
        );
        let server_owned_fresh_bootstrap = uses_server_owned_fresh_bootstrap(
            &app_server_target,
            &session_selection,
            &loader_overrides,
        );
        let mut bootstrap = startup_bootstrap;
        if bootstrap.is_none() && !server_owned_fresh_bootstrap {
            bootstrap = Some(
                match crate::boxed_future!(
                    startup_draft.run_until(tui, app_server.bootstrap(&config))
                )
                .await
                {
                    Ok(bootstrap) => startup_try!(bootstrap),
                    Err(err) => {
                        return shutdown_on_startup_error(app_server, &mut other_backends, err)
                            .await;
                    }
                },
            );
        }
        if matches!(&session_selection, SessionSelection::Fork(_)) {
            // The app server resolves omitted overrides from the fork destination's config.
            if harness_overrides.model.is_none()
                && !super::new_session::has_launch_setting(&config, &cli_kv_overrides, "model")
                && !super::new_session::has_launch_setting(
                    &config,
                    &cli_kv_overrides,
                    "model_reasoning_effort",
                )
            {
                config.model = None;
                config.model_reasoning_effort = None;
            }
        }
        let startup_defaults = if fresh_start {
            match crate::boxed_future!(startup_draft.run_until(
                tui,
                prepare_fresh_startup_config(
                    &mut config,
                    &app_server,
                    &app_server_target,
                    &cli_kv_overrides,
                    &harness_overrides,
                    &environment_manager,
                ),
            ))
            .await
            {
                Ok(Ok(defaults)) => defaults,
                Ok(Err(err)) => {
                    return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
                }
                Err(err) => {
                    return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
                }
            }
        } else {
            FreshStartupDefaults::default()
        };
        let mut launch_choices = crate::app_server_session::StartupLaunchChoices::from_launch(
            &cli_kv_overrides,
            &harness_overrides,
            &loader_overrides,
        );
        let bootstrap = match bootstrap {
            Some(bootstrap) => bootstrap,
            None => match crate::boxed_future!(startup_draft.run_until(
                tui,
                bootstrap_server_owned_start(
                    &mut app_server,
                    &mut config,
                    &mut launch_choices,
                    startup_defaults.server_defaults_read,
                    &cli_kv_overrides,
                    &harness_overrides,
                ),
            ))
            .await
            {
                Ok(bootstrap) => startup_try!(bootstrap),
                Err(err) => {
                    return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
                }
            },
        };
        launch_choices.apply_projectless_approval_policy(
            startup_defaults.approval_policy,
            bootstrap.allowed_approval_policies.as_deref(),
        );
        tracing::debug!(
            has_platform_family = app_server.app_server_platform_family().is_some(),
            has_platform_os = app_server.app_server_platform_os().is_some(),
            "connected app-server platform"
        );
        let bootstrap_ms = bootstrap.duration.as_millis();
        if !server_owned_fresh_bootstrap
            && matches!(&session_selection, SessionSelection::AgentsOverview)
        {
            apply_managed_new_thread_defaults(
                &mut config,
                app_server.managed_new_thread_defaults(),
                &cli_kv_overrides,
                &harness_overrides,
            );
        }
        let mut model = startup_model(&config, &bootstrap, startup_defaults.server_defaults_read);
        let mut available_models = bootstrap.available_models;
        let listed_models = available_models.clone();
        if let Some(state_db) = state_db.as_ref() {
            match state_db.list_gpu_runtime_providers().await {
                Ok(providers) => {
                    available_models.extend(providers.iter().filter_map(gpu_runtime_model_preset))
                }
                Err(error) => tracing::warn!(%error, "failed to load GPU rental model catalog"),
            }
        }
        let mut remote_connection =
            crate::status::remote_connection::remote_connection_status_value(
                &app_server_target,
                app_server.server_version(),
            );
        if let Some(connection) = &mut remote_connection {
            connection.is_managed_daemon = connection.is_local_daemon && managed_daemon;
        }
        let initial_server_version_notice =
            if !matches!(app_server_target, AppServerTarget::Embedded) {
                crate::status::remote_connection::pending_server_version_notice(
                    &local_settings.tui,
                    &app_server_target,
                    app_server.server_codex_home(),
                    CODEX_CLI_VERSION,
                    app_server.server_version(),
                    /*last_shown*/ None,
                )
            } else {
                None
            };
        if let Err(err) = startup_draft.flush_pending_events(tui).await {
            return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
        }
        let exit_info =
            if matches!(&session_selection, SessionSelection::Fork(_)) && config.model.is_none() {
                None
            } else {
                let result = handle_model_migration_prompt_if_needed(
                    tui,
                    &mut config,
                    &local_settings,
                    &mut launch_choices,
                    model.as_str(),
                    &app_event_tx,
                    &available_models,
                )
                .await;
                startup_try!(result)
            };
        if let Some(exit_info) = exit_info {
            shutdown_startup_sessions(std::iter::once(app_server).chain(other_backends)).await;
            return Ok(exit_info);
        }
        if let Some(updated_model) = config.model.clone() {
            model = updated_model;
        }
        let mut source_overrides = harness_overrides.clone();
        source_overrides.cwd = None;
        app_server.worktree_source_config_builder = Some(Box::new(
            crate::legacy_core::config::ConfigBuilder::default()
                .codex_home(config.codex_home.to_path_buf())
                .cli_overrides(cli_kv_overrides.clone())
                .harness_overrides(source_overrides)
                .loader_overrides(loader_overrides.clone())
                .cloud_config_bundle(cloud_config_bundle.clone()),
        ));
        let dynamic_tool_status_updates = tokio::sync::broadcast::channel(/*capacity*/ 64).0;
        if matches!(
            &app_server_target,
            AppServerTarget::LocalDaemon { .. } | AppServerTarget::Embedded
        ) && !crate::uses_remote_workspace_or_environment(
            &app_server_target,
            environment_manager.as_ref(),
        ) && let Err(error) = crate::boxed_future!(app_server.start_dynamic_tool_mcp(
            config.clone(),
            app_event_tx.clone(),
            dynamic_tool_status_updates.clone(),
        ))
        .await
        {
            tracing::warn!(%error, "TUI task delegation is unavailable without its MCP server");
        }
        let provider_runtime = crate::boxed_future!(crate::startup_provider::resolve_for_app(
            &config,
            startup_provider_resolution
        ))
        .await;
        crate::model_catalog::include_runtime_model(
            &mut available_models,
            &model,
            &config.model_provider_id,
        );
        let session_recovery_only = matches!(
            provider_runtime.current,
            codex_provider_auth::CurrentSelectionDecision::RequireExplicitRecovery { .. }
        ) && matches!(&session_selection, SessionSelection::Resume(_));
        let shared_provider_status_host = Some(provider_runtime.policy.host());
        let model_catalog = Arc::new(
            ModelCatalog::new(available_models.clone())
                .with_collaboration_modes(bootstrap.collaboration_modes),
        );
        model_catalog.mark_listed(&listed_models);
        model_catalog.set_provider_policy(provider_runtime.policy);
        // #572: Bedrock models for a session that may switch to a named Bedrock account.
        provider_management_status::check_named_bedrock_account(
            &model_catalog,
            &config,
            &app_event_tx,
        );
        // D86/#1193: which providers have named accounts, for `/status`.
        crate::provider_named_accounts::check_named_account_providers(&model_catalog, &config);
        model_catalog.set_session_recovery_only(session_recovery_only);
        let feedback_audience = bootstrap.feedback_audience;
        let auth_mode = bootstrap.auth_mode;
        let has_chatgpt_account = bootstrap.has_chatgpt_account;
        let has_codex_backend_auth = bootstrap.has_codex_backend_auth;
        let requires_openai_auth = bootstrap.requires_openai_auth;
        let status_account_display = bootstrap.status_account_display.clone();
        let initial_plan_type = bootstrap.plan_type;
        let session_telemetry = SessionTelemetry::new(
            ThreadId::new(),
            model.as_str(),
            model.as_str(),
            /*account_id*/ None,
            bootstrap.account_email.clone(),
            auth_mode,
            codex_login::default_client::originator().value,
            config.otel.log_user_prompt,
            user_agent(),
            serde_json::from_value(serde_json::json!("cli"))
                .unwrap_or_else(|err| panic!("cli session source should deserialize: {err}")),
        );
        if local_settings
            .tui
            .status_line
            .as_ref()
            .is_some_and(|cmd| !cmd.is_empty())
        {
            session_telemetry.counter("codex.status_line", /*inc*/ 1, &[]);
        }

        let status_line_invalid_items_warned = Arc::new(AtomicBool::new(false));
        let terminal_title_invalid_items_warned = Arc::new(AtomicBool::new(false));
        let workspace_command_runner: WorkspaceCommandRunner = Arc::new(
            AppServerWorkspaceCommandRunner::new(app_server.request_handle()),
        );
        let enhanced_keys_supported = tui.enhanced_keys_supported();
        let wait_for_initial_session_configured =
            Self::should_wait_for_initial_session(&session_selection);
        let should_prompt_for_paused_goal_after_startup_resume =
            Self::should_prompt_for_paused_goal_after_startup_resume(
                &session_selection,
                &initial_prompt,
                &initial_images,
            );
        let thread_and_widget_started_at = Instant::now();
        let pending_startup_thread_start = matches!(
            &session_selection,
            SessionSelection::StartFresh
                | SessionSelection::ResumePanesOnly { .. }
                | SessionSelection::Exit
        );
        let restore_pane_layout_for_thread_id = match &session_selection {
            SessionSelection::Resume(target_session) => Some(target_session.thread_id.to_string()),
            SessionSelection::ResumePanesOnly { codex_thread_id } => Some(codex_thread_id.clone()),
            SessionSelection::Fork(_)
            | SessionSelection::StartFresh
            | SessionSelection::Exit
            | SessionSelection::AgentsOverview => None,
        };
        let mut start_in_agents_overview =
            matches!(&session_selection, SessionSelection::AgentsOverview);
        let mut read_only_thread = false;
        let mut provider_error = None;
        let mut history_notice = None;
        let (mut chat_widget, initial_started_thread) = match session_selection {
            SessionSelection::StartFresh
            | SessionSelection::ResumePanesOnly { .. }
            | SessionSelection::Exit
            | SessionSelection::AgentsOverview => {
                if !start_in_agents_overview {
                    spawn_startup_thread_start(
                        &app_server,
                        local_settings.clone(),
                        config.clone(),
                        launch_choices.clone(),
                        app_event_tx.clone(),
                        managed_worktree.clone(),
                    );
                }
                // Count a startup tooltip once the initial chat widget can render it.
                let startup_tooltip_override = if start_in_agents_overview {
                    None
                } else {
                    match crate::boxed_future!(startup_draft.run_until(
                        tui,
                        prepare_startup_tooltip_override(
                            &mut local_settings,
                            &available_models,
                            is_first_run,
                        ),
                    ))
                    .await
                    {
                        Ok(tooltip_override) => tooltip_override,
                        Err(err) => {
                            return shutdown_on_startup_error(app_server, &mut other_backends, err)
                                .await;
                        }
                    }
                };
                let init = crate::chatwidget::ChatWidgetInit {
                    local_settings: local_settings.clone(),
                    config: config.clone(),
                    frame_requester: tui.frame_requester(),
                    app_event_tx: app_event_tx.clone(),
                    workspace_command_runner: Some(workspace_command_runner.clone()),
                    initial_user_message: crate::chatwidget::create_initial_user_message(
                        initial_prompt.clone(),
                        initial_images.clone(),
                        // CLI prompt args are plain strings, so they don't provide element ranges.
                        Vec::new(),
                    ),
                    enhanced_keys_supported,
                    has_chatgpt_account,
                    requires_openai_auth,
                    has_codex_backend_auth,
                    model_catalog: model_catalog.clone(),
                    feedback: feedback.clone(),
                    is_first_run,
                    status_account_display: status_account_display.clone(),
                    initial_plan_type,
                    model: Some(model.clone()),
                    startup_tooltip_override,
                    status_line_invalid_items_warned: status_line_invalid_items_warned.clone(),
                    terminal_title_invalid_items_warned: terminal_title_invalid_items_warned
                        .clone(),
                    session_telemetry: session_telemetry.clone(),
                };
                let mut chat_widget = ChatWidget::new_with_app_event(init);
                chat_widget.set_queue_submissions_until_session_configured(
                    /*queue*/ !start_in_agents_overview,
                );
                (chat_widget, None)
            }
            SessionSelection::Resume(target_session) => {
                if app_server_target.thread_params_mode()
                    == crate::app_server_session::ThreadParamsMode::Remote
                    && config_persistence::has_explicit_resume_permission_override(
                        &config,
                        &harness_overrides,
                    )
                {
                    return shutdown_on_startup_error(
                        app_server,
                        &mut other_backends,
                        color_eyre::eyre::eyre!(
                            "Permission overrides are not supported when resuming a remote task."
                        ),
                    )
                    .await;
                }
                if let Some(history_mode) = target_session.history_mode {
                    app_server.remember_thread_history_mode(target_session.thread_id, history_mode);
                }
                let model_settings = config_persistence::resume_model_settings_for_overrides(
                    &config,
                    &harness_overrides,
                );
                let resumed = match crate::boxed_future!(startup_draft.run_until(
                    tui,
                    app_server.resume_thread_with_permission_overrides(
                        &local_settings,
                        config.clone(),
                        target_session.thread_id,
                        model_settings,
                        crate::resume_permissions::ResumePermissions::from_overrides(
                            &config,
                            &harness_overrides,
                        ),
                    ),
                ))
                .await
                {
                    Ok(Ok(resumed)) => Ok(resumed),
                    Ok(Err(err))
                        if crate::app_server_session::is_active_writer_error(&err)
                            || crate::app_server_session::model_provider_error(&err).is_some() =>
                    {
                        provider_error = crate::app_server_session::model_provider_error(&err);
                        read_only_thread = true;
                        match crate::boxed_future!(startup_draft.run_until(
                            tui,
                            app_server.read_thread_for_viewing(
                                &config,
                                &local_settings,
                                target_session.thread_id,
                            ),
                        ))
                        .await
                        {
                            Ok(result) => result.map(|(thread, notice)| {
                                history_notice = notice;
                                thread
                            }),
                            Err(err) => {
                                return shutdown_on_startup_error(
                                    app_server,
                                    &mut other_backends,
                                    err,
                                )
                                .await;
                            }
                        }
                    }
                    Ok(resumed) => resumed,
                    Err(err) => {
                        return shutdown_on_startup_error(app_server, &mut other_backends, err)
                            .await;
                    }
                };
                let resumed = if read_only_thread {
                    match resumed {
                        Ok(resumed) => Some(resumed),
                        Err(err) => {
                            return shutdown_on_startup_error(app_server, &mut other_backends, err)
                                .await;
                        }
                    }
                } else {
                    let action = SessionStartAction::Resume(
                        model_settings,
                        crate::resume_permissions::ResumePermissions::from_overrides(
                            &config,
                            &harness_overrides,
                        ),
                    );
                    let resumed = crate::boxed_future!(complete_session_start(
                        &mut app_server,
                        SessionStartConfig {
                            config: &config,
                            local_settings: &local_settings,
                        },
                        &target_session,
                        action,
                        resumed,
                        async || {
                            startup_draft.flush_pending_events(tui).await?;
                            run_unarchive_prompt(tui, target_session.thread_id, action).await
                        },
                    ))
                    .await;
                    // #656: a resume the server refused ends the launch with a plain message.
                    let resumed = match resumed {
                        Err(err) => match session_start_refusal_exit(
                            "resume",
                            target_session.thread_id,
                            &err,
                        ) {
                            Some(exit_info) => {
                                let _ = tui.terminal.clear();
                                shutdown_startup_sessions(
                                    std::iter::once(app_server).chain(other_backends.drain(..)),
                                )
                                .await;
                                return Ok(exit_info);
                            }
                            None => Err(err),
                        },
                        resumed => resumed,
                    };
                    match startup_try!(resumed) {
                        SessionStartOutcome::Started(started) => Some(*started),
                        SessionStartOutcome::CommandCenter => None,
                        SessionStartOutcome::Exit => {
                            return Ok(crate::boxed_future!(cancel_startup_sessions(
                                app_server,
                                &mut other_backends
                            ))
                            .await);
                        }
                    }
                };
                if let Some(resumed) = resumed.as_ref() {
                    apply_persisted_resume_runtime(
                        &mut config,
                        Some(&resumed.session.model),
                        &resumed.session.model_provider_id,
                        resumed.session.reasoning_effort.clone(),
                    );
                    config.service_tier = resumed.session.service_tier.clone();
                    if !read_only_thread
                        && !resumed.blocks_direct_input
                        && let Err(error) =
                            crate::boxed_future!(resume_defaults::remember_resumed_model(
                                &app_server,
                                &config,
                                model_settings,
                                &resumed.session,
                            ))
                            .await
                    {
                        app_event_tx.send(AppEvent::InsertHistoryCell(Box::new(
                            history_cell::new_error_event(format!(
                                "Session restored, but failed to save its startup model: {error}"
                            )),
                        )));
                    }
                }
                let init = crate::chatwidget::ChatWidgetInit {
                    local_settings: local_settings.clone(),
                    config: config.clone(),
                    frame_requester: tui.frame_requester(),
                    app_event_tx: app_event_tx.clone(),
                    workspace_command_runner: Some(workspace_command_runner.clone()),
                    initial_user_message: crate::chatwidget::create_initial_user_message(
                        initial_prompt.clone(),
                        initial_images.clone(),
                        // CLI prompt args are plain strings, so they don't provide element ranges.
                        Vec::new(),
                    )
                    .filter(|_| resumed.is_some()),
                    enhanced_keys_supported,
                    has_chatgpt_account,
                    requires_openai_auth,
                    has_codex_backend_auth,
                    model_catalog: model_catalog.clone(),
                    feedback: feedback.clone(),
                    is_first_run,
                    status_account_display: status_account_display.clone(),
                    initial_plan_type,
                    model: config.model.clone(),
                    startup_tooltip_override: None,
                    status_line_invalid_items_warned: status_line_invalid_items_warned.clone(),
                    terminal_title_invalid_items_warned: terminal_title_invalid_items_warned
                        .clone(),
                    session_telemetry: session_telemetry.clone(),
                };
                (ChatWidget::new_with_app_event(init), resumed)
            }
            SessionSelection::Fork(target_session) => {
                let explicit_permission_override =
                    config_persistence::has_explicit_resume_permission_override(
                        &config,
                        &harness_overrides,
                    );
                if explicit_permission_override
                    && app_server_target.thread_params_mode()
                        == crate::app_server_session::ThreadParamsMode::Remote
                {
                    return shutdown_on_startup_error(
                        app_server,
                        &mut other_backends,
                        color_eyre::eyre::eyre!(
                            "Permission overrides are not supported when forking a remote task."
                        ),
                    )
                    .await;
                }
                let permission_mode = if explicit_permission_override {
                    crate::app_server_session::ForkPermissionMode::OverrideFromCurrentConfig
                } else {
                    crate::app_server_session::ForkPermissionMode::InheritSaved
                };
                session_telemetry.counter(
                    "codex.thread.fork",
                    /*inc*/ 1,
                    &[("source", "cli_subcommand")],
                );
                let forked = match crate::boxed_future!(startup_draft.run_until(
                    tui,
                    app_server.fork_thread_with_permission_mode(
                        &local_settings,
                        config.clone(),
                        target_session.thread_id,
                        permission_mode,
                    ),
                ))
                .await
                {
                    Ok(forked) => forked,
                    Err(err) => {
                        return shutdown_on_startup_error(app_server, &mut other_backends, err)
                            .await;
                    }
                };
                let action = SessionStartAction::Fork(permission_mode);
                let forked = crate::boxed_future!(complete_session_start(
                    &mut app_server,
                    SessionStartConfig {
                        config: &config,
                        local_settings: &local_settings,
                    },
                    &target_session,
                    action,
                    forked,
                    async || {
                        startup_draft.flush_pending_events(tui).await?;
                        run_unarchive_prompt(tui, target_session.thread_id, action).await
                    },
                ))
                .await;
                // #656: a fork the server refused ends the launch with a plain message.
                let forked = match forked {
                    Err(err) => {
                        match session_start_refusal_exit("fork", target_session.thread_id, &err) {
                            Some(exit_info) => {
                                let _ = tui.terminal.clear();
                                shutdown_startup_sessions(
                                    std::iter::once(app_server).chain(other_backends.drain(..)),
                                )
                                .await;
                                return Ok(exit_info);
                            }
                            None => Err(err),
                        }
                    }
                    forked => forked,
                };
                let forked = match startup_try!(forked) {
                    SessionStartOutcome::Started(started) => Some(*started),
                    SessionStartOutcome::CommandCenter => None,
                    SessionStartOutcome::Exit => {
                        return Ok(crate::boxed_future!(cancel_startup_sessions(
                            app_server,
                            &mut other_backends
                        ))
                        .await);
                    }
                };
                if let Some(forked) = &forked {
                    if let Some(worktree) = managed_worktree.as_ref()
                        && let Err(err) = worktree.bind(forked.session.thread_id)
                    {
                        return shutdown_on_startup_error(app_server, &mut other_backends, err)
                            .await;
                    }
                    if config.model_reasoning_effort.is_none() {
                        config.model_reasoning_effort = forked.session.reasoning_effort.clone();
                    }
                }
                let init = crate::chatwidget::ChatWidgetInit {
                    local_settings: local_settings.clone(),
                    config: config.clone(),
                    frame_requester: tui.frame_requester(),
                    app_event_tx: app_event_tx.clone(),
                    workspace_command_runner: Some(workspace_command_runner.clone()),
                    initial_user_message: crate::chatwidget::create_initial_user_message(
                        initial_prompt.clone(),
                        initial_images.clone(),
                        // CLI prompt args are plain strings, so they don't provide element ranges.
                        Vec::new(),
                    )
                    .filter(|_| forked.is_some()),
                    enhanced_keys_supported,
                    has_chatgpt_account,
                    requires_openai_auth,
                    has_codex_backend_auth,
                    model_catalog: model_catalog.clone(),
                    feedback: feedback.clone(),
                    is_first_run,
                    status_account_display: status_account_display.clone(),
                    initial_plan_type,
                    model: config.model.clone(),
                    startup_tooltip_override: None,
                    status_line_invalid_items_warned: status_line_invalid_items_warned.clone(),
                    terminal_title_invalid_items_warned: terminal_title_invalid_items_warned
                        .clone(),
                    session_telemetry: session_telemetry.clone(),
                };
                (ChatWidget::new_with_app_event(init), forked)
            }
        };
        let startup_session_cancelled = initial_started_thread.is_none()
            && !pending_startup_thread_start
            && !start_in_agents_overview;
        if startup_session_cancelled {
            start_in_agents_overview = true;
        }
        chat_widget.note_rendered_width(tui.terminal.last_known_screen_size.width);
        if pending_startup_thread_start && !start_in_agents_overview {
            chat_widget
                .empty_state_animation
                .borrow_mut()
                .continue_from(&mut startup_draft.blossom.borrow_mut());
        }
        if let Some(notice) = resume_fallback_notice {
            app_event_tx.send(AppEvent::InsertHistoryCell(Box::new(
                // Sync: plain warnings stay out of the transcript upstream; this one must show.
                crate::history_cell::new_transcript_warning_event(notice),
            )));
        }
        chat_widget.remote_connection = remote_connection;
        chat_widget.snapshot_local_images = app_server_target.uses_remote_workspace();
        chat_widget.set_local_worktree_operations(!crate::uses_remote_workspace_or_environment(
            &app_server_target,
            environment_manager.as_ref(),
        ));
        let thread_and_widget_ms = thread_and_widget_started_at.elapsed().as_millis();
        chat_widget.maybe_prompt_planted_project_metadata();
        chat_widget.windows_sandbox_local_server = !app_server_target.uses_remote_workspace()
            && app_server.app_server_platform_os() == Some("windows");
        chat_widget.windows_sandbox_host = if app_server_target.uses_remote_workspace() {
            WindowsSandboxHost::Remote
        } else {
            initial_started_thread
                .as_ref()
                .map_or(WindowsSandboxHost::Unknown, |started| {
                    started.session.windows_sandbox_host
                })
        };
        // This launch warning belongs to the TUI, independent of picker/trust client replacement.
        if let Some(warning) = daemon_startup_warning {
            app_event_tx.send(AppEvent::InsertHistoryCell(Box::new(
                history_cell::StartupWarningsCell::new(vec![warning]),
            )));
        }
        // Automatic NFS detection is Linux-only; manual migration supports Unix.
        #[cfg(target_os = "linux")]
        if !app_server_target.uses_remote_workspace() {
            tokio::spawn(startup_warnings::warn_about_nfs_migration(
                config.sqlite_config().clone(),
                app_event_tx.clone(),
            ));
        }
        let file_search = FileSearchManager::new(config.cwd.to_path_buf(), app_event_tx.clone());
        let runtime_keymap =
            RuntimeKeymap::from_config(&local_settings.tui.keymap).map_err(|err| {
                color_eyre::eyre::eyre!(
                    "Invalid `tui.keymap` configuration: {err}\n\
Fix the config and retry.\n\
See the Corbanu Terminal keymap documentation for supported actions and examples."
                )
            });
        let runtime_keymap = startup_try!(runtime_keymap);
        #[cfg(not(debug_assertions))]
        let upgrade_version = crate::updates::get_upgrade_version(&config);

        let restored_pane_layout =
            restore_pane_layout_for_thread_id
                .as_deref()
                .and_then(|thread_id| {
                    crate::claude_panes::load_pane_layout(
                        config.codex_home.as_ref(),
                        Some(thread_id),
                    )
                });
        let restored_spawn_parent_by_node = restored_pane_layout
            .as_ref()
            .map(|layout| {
                layout
                    .spawn_parent_by_node
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let restored_claude_panes = crate::claude_panes::ClaudePaneRegistry::restore_from_disk(
            config.codex_home.as_ref(),
            restored_pane_layout.as_ref(),
        );
        let restored_codex_user_pane_ids = restored_pane_layout
            .as_ref()
            .map(|layout| {
                layout
                    .codex_user_pane_ids
                    .iter()
                    .filter_map(|thread_id| ThreadId::from_string(thread_id).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let restored_spawn_nazgul_pane_id = restored_pane_layout
            .as_ref()
            .and_then(|layout| layout.spawn_nazgul_pane_id.clone())
            .filter(|pane_id| {
                // `codex-main`, any restored Claude pane, or a native Codex agent thread node id
                // (`thread:<uuid>`) are all valid Nazgul root bindings.
                if pane_id == crate::claude_panes::CODEX_MAIN_PANE_ID {
                    return true;
                }
                if crate::spawn_orchestration::node_id_thread(pane_id).is_some() {
                    return true;
                }
                restored_claude_panes
                    .panes()
                    .iter()
                    .any(|pane| pane.id == *pane_id)
            });
        let restored_spawn_nazgul_rebind_required = restored_pane_layout
            .as_ref()
            .is_some_and(|layout| layout.spawn_nazgul_rebind_required);
        let restored_spawn_legacy_read_only = restored_pane_layout
            .as_ref()
            .is_some_and(crate::spawn_orchestration::pane_layout_has_legacy_spawn_hierarchy);
        let mut restored_orchestrate_whips: HashMap<_, _> = restored_pane_layout
            .as_ref()
            .map(|layout| {
                layout
                    .orchestrate_whips
                    .iter()
                    .filter(|(_, whip)| whip.state != crate::orchestrate::WhipState::Detached)
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default();
        for whip in restored_orchestrate_whips.values_mut() {
            if matches!(
                whip.kind,
                crate::orchestrate::WhipKind::Assignment {
                    phase: crate::orchestrate::AssignmentPhase::Executing,
                    ..
                }
            ) {
                whip.last_fire_utc = Some(chrono::Utc::now());
                // Generation zero represents the state restored from disk. Treat any saved
                // Worker output as already observed so restart honors one full cadence; a fresh
                // completion increments the generation and can still trigger immediately.
                whip.last_idle_generation_fired = Some(0);
            } else {
                whip.last_idle_generation_fired = Some(0);
            }
        }
        let restored_orchestrate_next_whip_seq = restored_pane_layout
            .as_ref()
            .map(|layout| layout.orchestrate_next_whip_seq)
            .unwrap_or_else(|| {
                restored_orchestrate_whips
                    .keys()
                    .filter_map(|id| {
                        id.strip_prefix("whip-")
                            .or_else(|| id.strip_prefix("assignment-"))?
                            .parse::<u64>()
                            .ok()
                    })
                    .max()
                    .unwrap_or(0)
            });
        let restored_orchestrate_idle_generation_by_target = restored_orchestrate_whips
            .values()
            .map(|whip| (whip.target.clone(), 0))
            .collect();
        let restored_spawn_next_dispatch_seq = restored_pane_layout
            .as_ref()
            .map(|layout| layout.spawn_next_dispatch_seq.max(1))
            .unwrap_or(1);
        let restored_spawn_processed_dispatch_seq_ids = restored_pane_layout
            .as_ref()
            .map(|layout| {
                crate::spawn_orchestration::bounded_spawn_processed_dispatch_seq_ids(
                    layout.spawn_processed_dispatch_seq_ids.iter().copied(),
                    restored_spawn_next_dispatch_seq,
                )
            })
            .unwrap_or_default();
        let restored_spawn_processed_dispatch_origins = restored_pane_layout
            .as_ref()
            .map(|layout| {
                layout
                    .spawn_processed_dispatch_origin_ids
                    .iter()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        let agents_overview =
            agents_overview::AgentsOverviewState::new(local_settings.tui.agents_overview_grouping);
        let mut app = Self {
            feature_write_lock: Arc::default(),
            config_notification_test_generation: 0,
            config_notification_test_pending: None,
            model_catalog,
            session_telemetry: session_telemetry.clone(),
            terminal_size_tags: tui.terminal_size_tags.clone(),
            app_event_tx,
            chat_widget,
            workspace_command_runner: Some(workspace_command_runner),
            config,
            local_settings,
            active_deferred_provider_setup: deferred_provider_setup.clone(),
            shared_provider_setup_session: None,
            shared_provider_status_host,
            shared_provider_account_auth_host: None,
            provider_management_host: None,
            provider_management_generation: 0,
            pending_wallet_create_deferred: None,
            launch_cwd,
            runtime_working_directory_override: None,
            state_db,
            cli_kv_overrides,
            harness_overrides,
            loader_overrides,
            cloud_config_bundle,
            runtime_approval_policy_override: None,
            runtime_approvals_reviewer_override: None,
            runtime_permission_profile_override: None,
            pending_permission_confirmation: None,
            pending_server_profiles: HashMap::new(),
            file_search,
            enhanced_keys_supported,
            keymap: runtime_keymap,
            key_chord_matcher: KeyChordMatcher::default(),
            routed_key_activation: None,
            transcript_cells: Vec::new(),
            claude_pane_transcript_cells: HashMap::new(),
            native_history: Default::default(),
            turn_tips: Default::default(),
            transcript_view: Default::default(),
            conversation_frame: Default::default(),
            last_rendered_history_tail: None,
            last_thread_usage_status_cell: None,
            pending_thread_usage_history_refresh: false,
            overlay: None,
            retained_analytics: None,
            deferred_history_lines: Vec::new(),
            has_emitted_history_lines: false,
            transcript_reflow: TranscriptReflowState::default(),
            initial_history_replay_buffer: None,
            pending_thread_switch_resets: 0,
            scrollback_has_older_history: false,
            commit_animation: None,
            status_line_invalid_items_warned: status_line_invalid_items_warned.clone(),
            terminal_title_invalid_items_warned: terminal_title_invalid_items_warned.clone(),
            skill_load_warnings: SkillLoadWarningState::default(),
            skills_refresh_generation: Arc::new(AtomicU64::new(0)),
            backtrack: BacktrackState::default(),
            backtrack_render_pending: false,
            feedback: feedback.clone(),
            feedback_audience,
            environment_manager,
            app_server_target,
            pending_right_click_paste: None,
            right_click_paste_environment: super::right_click_paste::PasteEnvironment::detect(),
            reconnect: ReconnectState {
                seen_version_notice: initial_server_version_notice
                    .as_ref()
                    .map(|(_, key)| key.clone()),
                ..Default::default()
            },
            daemon_cli_executable,
            pending_update_action: None,
            daemon_restart: None,
            pending_shutdown_exit_thread_id: None,
            windows_sandbox: WindowsSandboxState {
                prompt_after_trust: should_prompt_windows_sandbox_nux_at_startup
                    || startup_defaults.prompt_windows_sandbox,
                ..Default::default()
            },
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
            agents_overview,
            spawn_parent_by_thread: HashMap::new(),
            spawn_parent_by_node: restored_spawn_parent_by_node,
            spawn_native_runtime_by_node: restored_pane_layout
                .as_ref()
                .map(|layout| {
                    layout
                        .spawn_native_runtime_by_node
                        .clone()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default(),
            spawn_native_endpoint_by_node: restored_pane_layout
                .as_ref()
                .map(|layout| {
                    layout
                        .spawn_native_endpoint_by_node
                        .iter()
                        .filter_map(|(node, endpoint)| {
                            ThreadId::from_string(endpoint)
                                .ok()
                                .map(|endpoint| (node.clone(), endpoint))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            spawn_crew: restored_pane_layout
                .as_ref()
                .and_then(|layout| layout.spawn_crew.clone()),
            spawn_legacy_read_only: restored_spawn_legacy_read_only,
            spawn_status_by_thread: HashMap::new(),
            spawn_waiting_for_agents_by_thread: HashMap::new(),
            spawn_parent_reports_by_node: HashMap::new(),
            spawn_dispatch_acks_by_target_task: HashMap::new(),
            spawn_next_dispatch_seq: restored_spawn_next_dispatch_seq,
            spawn_processed_dispatch_seq_ids: restored_spawn_processed_dispatch_seq_ids,
            spawn_processed_dispatch_origins: restored_spawn_processed_dispatch_origins,
            spawn_processed_terminal_turns: HashSet::new(),
            spawn_auto_loop_state_by_node: HashMap::new(),
            spawn_operator_input_seen: false,
            kill_switch_held_claude_tasks: Vec::new(),
            kill_switch_held_native_tasks: Vec::new(),
            // Set from the home's watch at start (`watch_kill_switch_elsewhere`).
            kill_switch_known_on: false,
            spawn_quarantine_notified_by_node: HashSet::new(),
            spawn_context_left_by_thread: HashMap::new(),
            spawn_last_report_seq_by_node: HashMap::new(),
            spawn_last_dispatch_seq_by_node: HashMap::new(),
            spawn_last_event_at_by_node: HashMap::new(),
            spawn_nazgul_pane_id: restored_spawn_nazgul_pane_id,
            spawn_nazgul_rebind_required: restored_spawn_nazgul_rebind_required,
            orchestrate_whips: Box::new(restored_orchestrate_whips),
            orchestrate_next_whip_seq: restored_orchestrate_next_whip_seq,
            #[cfg(test)]
            orchestrate_now_override: None,
            orchestrate_idle_generation_by_target: Box::new(
                restored_orchestrate_idle_generation_by_target,
            ),
            side_threads: HashMap::new(),
            claude_panes: restored_claude_panes,
            abandoned_side_threads: HashSet::new(),
            active_thread_id: None,
            active_thread_rx: None,
            primary_thread_id: None,
            last_subagent_backfill_attempt: None,
            primary_session_configured: None,
            pending_primary_events: VecDeque::new(),
            pending_app_server_requests: PendingAppServerRequests::default(),
            dynamic_tool_status_updates,
            dynamic_tool_tasks: HashMap::new(),
            pending_startup_thread_start,
            pending_server_version_notice: if pending_startup_thread_start {
                initial_server_version_notice
                    .as_ref()
                    .map(|(notice, _)| notice.clone())
            } else {
                None
            },
            pending_open_resume_picker: false,
            pending_working_directory_change: None,
            pending_start_managed_worktree: None,
            pending_managed_worktree_creation: false,
            pending_managed_worktree_created: None,
            pending_managed_worktree_transition: None,
            pending_managed_worktree_attach: None,
            startup_protected_input_boundary: true,
            startup_pending_protected_request: false,
            account_email_request_id: None,
            rate_limit_hard_stop_generation: 0,
            rate_limit_refresh_state: Default::default(),
            pending_mcp_login_start: None,
            active_mcp_login_ids: HashMap::new(),
            pending_plugin_enabled_writes: HashMap::new(),
            pending_hook_enabled_writes: HashMap::new(),
            recap: recap::RecapState::default(),
            #[cfg(test)]
            _test_codex_home: None,
        };
        if tui.is_owned_screen()
            && app.right_click_paste_environment.wsl
            && app.right_click_paste_environment.allows(
                app.local_settings.tui.right_click_paste,
                app.local_settings.transcript_mode,
            )
        {
            // Prewarm WSL interop off the UI thread, without accessing clipboard contents.
            let _ = tui.clipboard.warm_text_reader(tui.frame_requester());
        }
        app.remember_launch_permissions();
        if !tui.is_terminal_focused() {
            app.recap.note_focus_lost(Instant::now());
        }
        let _ =
            app.initialize_server_version_notice(CODEX_CLI_VERSION, app_server.server_version());
        if initial_server_version_notice.is_none() {
            app.update_server_version_overview_notice(
                CODEX_CLI_VERSION,
                /*server_version*/ None,
            );
        }
        if start_in_agents_overview {
            app.open_agents_overview(&app_server);
        } else {
            app.refresh_agents_overview_threads(&app_server);
        }
        if let Some(deferred) = deferred_provider_setup {
            app.app_event_tx
                .send(AppEvent::BeginDeferredCorbanuPlan { deferred });
        }
        for thread_id in restored_codex_user_pane_ids {
            app.upsert_agent_picker_thread(
                thread_id, /*agent_nickname*/ None, /*agent_role*/ None,
                /*is_closed*/ false,
            );
        }
        app.refresh_gpu_spend_indicator().await;
        // A rental outlives the TUI that created it. Reattach the independent reconciler on
        // every startup; its process lock deduplicates concurrent Corbanu Terminal processes and it
        // exits immediately when there is no potentially billable work.
        app.start_gpu_controller();
        app.watch_kill_switch_elsewhere();
        app.watch_level_raised_elsewhere();
        let telegram_home = app.config.codex_home.clone().to_path_buf();
        tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                crate::chatwidget::telegram_setup::ensure_connector(&telegram_home)
            })
            .await;
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    tracing::warn!(%error, "configured Telegram connector did not start")
                }
                Err(error) => {
                    tracing::warn!(%error, "Telegram connector startup task failed")
                }
            }
        });
        if let Some(entry) = startup_hooks_browser {
            app.chat_widget.open_hooks_browser(entry);
        }
        app.seed_restored_claude_pane_transcripts();
        app.show_restored_active_claude_pane();
        app.update_visible_history_rows(tui.terminal.last_known_screen_size);
        let initial_session_started_at = Instant::now();
        if let Some(started) = initial_started_thread {
            let thread_id = started.session.thread_id;
            app.chat_widget
                .set_task_mentions_enabled(started.task_tools_available);
            if started.blocks_direct_input {
                app.mark_primary_thread_parent_owned(thread_id);
            }
            if let Some(error) = provider_error.clone() {
                app.chat_widget
                    .show_provider_error(error, /*rejected_message_id*/ None);
            } else if read_only_thread {
                app.chat_widget.show_external_writer_thread();
            }
            match crate::boxed_future!(startup_draft.run_until(
                tui,
                app.enqueue_primary_thread_session(started.session, started.turns),
            ))
            .await
            {
                Ok(result) => startup_try!(result),
                Err(err) => {
                    return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
                }
            }
            crate::boxed_future!(app.restore_codex_user_panes_from_saved_state(&mut app_server))
                .await;
            crate::boxed_future!(app.restore_native_spawn_panes_from_saved_state(&mut app_server))
                .await;
            app.audit_restored_assignments();
            if let Some(error) = provider_error {
                app.chat_widget
                    .show_provider_error(error, /*rejected_message_id*/ None);
            } else if read_only_thread {
                app.ensure_thread_channel(thread_id).mark_external_writer();
                app.chat_widget.show_external_writer_thread();
            }
            if let Some(notice) = history_notice {
                app.chat_widget
                    .add_info_message(notice.to_string(), /*hint*/ None);
            }
            if !read_only_thread
                && should_prompt_for_paused_goal_after_startup_resume
                && let Err(err) = crate::boxed_future!(startup_draft.run_until(
                    tui,
                    app.maybe_prompt_resume_paused_goal_after_resume(&mut app_server, thread_id,),
                ))
                .await
            {
                return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
            }
        }
        if !start_in_agents_overview
            && !pending_startup_thread_start
            && let Some((notice, _)) = &initial_server_version_notice
        {
            app.app_event_tx.send(AppEvent::InsertHistoryCell(Box::new(
                history_cell::new_server_version_warning(notice.clone()),
            )));
        }
        let initial_session_ms = initial_session_started_at.elapsed().as_millis();

        if let Err(err) = startup_draft.flush_pending_events(tui).await {
            return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
        }
        if Self::should_handle_active_thread_events(
            wait_for_initial_session_configured,
            app.active_thread_rx.is_some(),
        ) && let Err(err) = app.drain_active_thread_events(tui).await
        {
            return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
        }
        if app_event_rx.is_empty()
            && !app.has_queued_startup_protected_request()
            && !app.startup_pending_protected_request
            && !app.chat_widget.has_active_view()
            && !app.chat_widget.has_pending_protected_request()
            && let Err(err) = startup_draft.flush_pending_paste_newline(tui).await
        {
            return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
        }
        // Keep cancelled resume/fork text editable, but never carry confirmation to another session.
        let mut pending_startup_submission =
            startup_draft.take_submission_intent() && !startup_session_cancelled;
        let mut pending_startup_draft = Some(startup_draft.into_draft());
        if app_event_rx.is_empty() && !app.has_queued_startup_protected_request() {
            app.chat_widget.restore_startup_input_when_ready(
                &mut pending_startup_draft,
                &mut pending_startup_submission,
            );
        }

        #[cfg(windows)]
        let mut terminal_color_probe_pending = true;
        #[cfg(windows)]
        if app.ready_for_terminal_color_probe(!app_event_rx.is_empty()) {
            tui.probe_default_colors_after_protected_startup();
            terminal_color_probe_pending = false;
        }

        let event_stream_started_at = Instant::now();
        tui.schedule_screen_size_recheck(Duration::ZERO);
        if let Err(err) = app.render_startup_frame(tui, &app_event_rx) {
            return shutdown_on_startup_error(app_server, &mut other_backends, err).await;
        }
        let tui_events = tui.event_stream();
        tokio::pin!(tui_events);
        let _whip_sweep_tick = spawn_whip_sweep_tick(app.app_event_tx.clone());
        tracing::info!(
            duration_ms = %(startup_elapsed_before_app + startup_started_at.elapsed()).as_millis(),
            bootstrap_ms = %bootstrap_ms,
            thread_and_widget_ms = %thread_and_widget_ms,
            initial_session_ms = %initial_session_ms,
            event_stream_ms = %event_stream_started_at.elapsed().as_millis(),
            "tui startup initial frame scheduled"
        );
        app.refresh_startup_skills(&app_server);
        // Kick off a non-blocking rate-limit prefetch so the first `/status`
        // already has data and available reset credits can be surfaced, without
        // delaying the initial frame render.
        if requires_openai_auth && has_chatgpt_account {
            crate::security_setup::prefetch(
                &app.config,
                &app_server,
                app.app_event_tx.clone(),
                app.chat_widget.security_setup_request_id,
            );
            let reset_hint_request_id = app.chat_widget.start_rate_limit_reset_startup_check();
            app.refresh_rate_limits(
                &app_server,
                RateLimitRefreshOrigin::StartupPrefetch {
                    reset_hint_request_id,
                },
            );
        }

        let mut last_screen_size = tui.terminal.last_known_screen_size;
        let mut settled_size_tags = crate::terminal_size_telemetry::tags(last_screen_size);
        let mut pending_resize: Option<(ratatui::layout::Size, Instant)> = None;
        let mut reconnect = None;
        let mut embedded_stream_closed_notified = false;
        let mut gpu_runtime_overlay_tick = tokio::time::interval(Duration::from_secs(5));
        gpu_runtime_overlay_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut waiting_for_initial_session_configured = wait_for_initial_session_configured;
        let mut waiting_for_initial_session_header = true;

        #[cfg(not(debug_assertions))]
        let pre_loop_exit_reason = if let Some(latest_version) = upgrade_version {
            let control = Box::pin(app.handle_event(
                tui,
                &mut app_server,
                AppEvent::InsertHistoryCell(Box::new(UpdateAvailableHistoryCell::new(
                    latest_version,
                    crate::update_action::get_update_action(),
                ))),
            ))
            .await;
            match startup_try!(control) {
                AppRunControl::Continue => None,
                AppRunControl::Exit(exit_reason) => Some(exit_reason),
            }
        } else {
            None
        };
        #[cfg(debug_assertions)]
        let pre_loop_exit_reason: Option<ExitReason> = None;

        let mut backends =
            super::backend_sessions::BackendSessions::from_initial(app_server, other_backends);

        let exit_reason_result = if let Some(exit_reason) = pre_loop_exit_reason {
            Ok(exit_reason)
        } else {
            loop {
                let displayed_session = backends.displayed;
                let displayed_owner = backends.displayed_owner();
                if !backends.is_listening(displayed_owner)
                    && !app.pending_startup_thread_start
                    && !app.reconnect.offline
                {
                    app.begin_reconnect();
                }
                // Reconnect can dismiss an overlay from the server-event path.
                if app.overlay.is_none() {
                    if !tui.is_owned_screen() && tui.is_alt_screen_active() {
                        app.close_transcript_overlay(tui);
                    } else if let Err(err) =
                        tui.set_mouse_capture(app.chat_widget.mouse_capture(tui.is_owned_screen()))
                    {
                        break Err(err.into());
                    }
                }
                if app.pending_open_resume_picker {
                    app.pending_open_resume_picker = false;
                    match Box::pin(app.open_resume_picker_for_backends(tui, &mut backends)).await {
                        Ok(AppRunControl::Continue) => {}
                        Ok(AppRunControl::Exit(reason)) => break Ok(reason),
                        Err(err) if app.recover_transport_error(&err) => {}
                        Err(err) => break Err(err),
                    }
                    continue;
                }
                if let Some(pending) = app.pending_working_directory_change.take() {
                    Box::pin(app.finish_working_directory_change(
                        tui,
                        backends.session_at_mut(displayed_session),
                        pending,
                    ))
                    .await;
                    continue;
                }
                if let Some((mode, name)) = app.pending_start_managed_worktree.take() {
                    Box::pin(app.start_managed_worktree(
                        backends.session_at_mut(displayed_session),
                        mode,
                        name,
                    ))
                    .await;
                    continue;
                }
                // Complete the fork and widget attachment on separate fresh loop iterations.
                if let Some(attach) = app.pending_managed_worktree_attach.take() {
                    Box::pin(app.attach_working_directory(
                        tui,
                        backends.session_at_mut(displayed_session),
                        *attach,
                    ))
                    .await;
                    continue;
                }
                if let Some(transition) = app.pending_managed_worktree_transition.take() {
                    if let Err(error) = Box::pin(app.switch_to_managed_worktree(
                        tui,
                        backends.session_at_mut(displayed_session),
                        *transition,
                    ))
                    .await
                    {
                        app.chat_widget.add_error_message(error.to_string());
                    }
                    continue;
                }
                if let Some(created) = app.pending_managed_worktree_created.take() {
                    Box::pin(app.finish_managed_worktree(*created)).await;
                    continue;
                }
                if app.reconnect.offline && !app.reconnect.failed && reconnect.is_none() {
                    let presentation = app.reconnect.presentation;
                    let wait_for_drain =
                        app.pending_startup_thread_start && backends.is_listening(displayed_owner);
                    let attempt = reconnect::reconnect(
                        displayed_owner,
                        app.app_server_target.clone(),
                        app.config.clone(),
                        app.local_settings.clone(),
                        app.launch_approval_resume_permissions(),
                        app.current_displayed_thread_id(),
                        backends
                            .session_at(displayed_session)
                            .remote_cwd_override()
                            .map(Path::to_path_buf),
                        backends
                            .session_at(displayed_session)
                            .thread_tool_transport(),
                        presentation,
                    );
                    let future = Box::pin(async move {
                        if let reconnect::ReconnectPresentation::Startup(deadline) = presentation
                            && wait_for_drain
                        {
                            tokio::time::sleep_until(deadline).await;
                            color_eyre::eyre::bail!(
                                "Automatic reconnect timed out waiting for the server to finish shutting down."
                            );
                        }
                        attempt.await
                    });
                    reconnect = Some((displayed_owner, future));
                }
                // Replay queues history and operations. A buffered closure must not switch
                // widgets before those app events have been applied.
                let has_pending_app_events = !app_event_rx.is_empty();
                let paused_backend = ((app.reconnect.offline && !app.pending_startup_thread_start)
                    || (!matches!(app.app_server_target, AppServerTarget::Embedded)
                        && has_pending_app_events))
                    .then_some(displayed_owner);
                let initial_session_header_pending = waiting_for_initial_session_header
                    && app.primary_session_configured.is_some()
                    && has_pending_app_events;
                let block_terminal_input_for_pending_startup_events =
                    (!matches!(app.app_server_target, AppServerTarget::Embedded)
                        && has_pending_app_events)
                        || initial_session_header_pending
                        || (pending_startup_draft.is_some()
                            || app.startup_protected_input_boundary)
                            && has_pending_app_events
                        || (!waiting_for_initial_session_configured
                            && app.has_queued_startup_protected_request());
                // The screen cache is physical geometry, not the inline viewport. This also
                // notices the final size after returning from a standalone nested screen.
                let size = tui.terminal.last_known_screen_size;
                if !crate::terminal_size_telemetry::enabled(&app.config)
                    || app.terminal_size_tags.is_empty()
                {
                    pending_resize = None;
                    settled_size_tags = crate::terminal_size_telemetry::tags(size);
                } else if size != last_screen_size {
                    pending_resize =
                        Some((size, Instant::now() + Duration::from_secs(/*secs*/ 15)));
                }
                last_screen_size = size;
                let rate_limit_poll_deadline = app
                    .chat_widget
                    .rate_limit_refresh_interval()
                    .and_then(|interval| app.rate_limit_refresh_state.poll_deadline(interval));
                let control = select! {
                    result = async {
                        match app.daemon_restart.as_mut() {
                            Some(task) => task.await,
                            None => std::future::pending().await,
                        }
                    }, if app.daemon_restart.is_some() => {
                        app.daemon_restart = None;
                        if let Err(err) = result.unwrap_or_else(|err| Err(err.into())) {
                            app.chat_widget.add_error_message(format!("Could not restart the background server: {err:#}"));
                        }
                        AppRunControl::Continue
                    }
                    Some(event) = app_event_rx.recv() => {
                        if let AppEvent::ResumeBackendSession(target) = event {
                            match app.resume_backend_session(tui, &mut backends, target).await {
                                Ok(AppRunControl::Continue) => continue,
                                Ok(AppRunControl::Exit(reason)) => break Ok(reason),
                                Err(error) => break Err(error),
                            }
                        }
                        let is_initial_session_header = matches!(
                            &event,
                            AppEvent::InsertHistoryCell(cell)
                                if cell.as_any().is::<history_cell::SessionInfoCell>()
                        );
                        let event_session = match &event {
                            AppEvent::StartupThreadStarted { backend_id, .. } => {
                                backends.slot(*backend_id).unwrap_or(displayed_session)
                            }
                            _ => displayed_session,
                        };
                        let had_active_modal = app.chat_widget.has_active_modal();
                        match Box::pin(app.handle_event(tui, backends.session_at_mut(event_session), event)).await {
                            Ok(AppRunControl::Continue) => {
                                if is_initial_session_header {
                                    waiting_for_initial_session_header = false;
                                }
                                if !had_active_modal
                                    && app.chat_widget.has_active_modal()
                                    && let Err(err) = app.render_startup_frame(tui, &app_event_rx)
                                {
                                    break Err(err);
                                }
                                AppRunControl::Continue
                            }
                            Ok(AppRunControl::Exit(reason)) => AppRunControl::Exit(reason),
                            Err(err) if app.recover_transport_error(&err) => AppRunControl::Continue,
                            // Server history already changed: accepting input would hide it.
                            Err(err) if server_history_changed(&err) => break Err(err),
                            Err(err) => {
                                tracing::error!(error = ?err, "contained app event handler failure");
                                app.chat_widget.add_error_message(format!(
                                    "A command failed but Corbanu Terminal is still running: {err:#}"
                                ));
                                AppRunControl::Continue
                            }
                        }
                    }
                    active = async {
                        if let Some(rx) = app.active_thread_rx.as_mut() {
                            rx.recv().await
                        } else {
                            None
                        }
                    }, if App::should_handle_active_thread_events(
                        waiting_for_initial_session_configured,
                        app.active_thread_rx.is_some()
                    ) && !has_pending_app_events && !app.reconnect.offline => {
                        if let Some(event) = active {
                            if let Err(err) = crate::boxed_future!(app.handle_active_thread_event(tui, backends.session_at_mut(displayed_session), event)).await {
                                tracing::error!(error = ?err, "contained active-thread event failure");
                                app.chat_widget.add_error_message(format!(
                                    "A pane event failed but Corbanu Terminal is still running: {err:#}"
                                ));
                            }
                        } else {
                            app.clear_active_thread().await;
                        }
                        AppRunControl::Continue
                    }
                    event = tui_events.next(), if app.pending_thread_switch_resets == 0
                        && (app.reconnect.offline || !block_terminal_input_for_pending_startup_events) => {
                        if let Some(event) = event {
                            if (matches!(
                                &event,
                                TuiEvent::Key(key)
                                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
                            ) || matches!(&event, TuiEvent::Paste(_)))
                                && !app.reconnect.offline
                                && pending_startup_draft.is_none()
                                && !waiting_for_initial_session_configured
                                && app_event_rx.is_empty()
                                && !app.startup_pending_protected_request
                                && app
                                    .active_thread_rx
                                    .as_ref()
                                    .is_none_or(tokio::sync::mpsc::Receiver::is_empty)
                                && !app.pending_primary_events.iter().any(|event| {
                                    matches!(event, ThreadBufferedEvent::Request(_))
                                })
                            {
                                app.startup_protected_input_boundary = false;
                            }
                            match crate::boxed_future!(app.handle_tui_event(tui, backends.session_at_mut(displayed_session), event)).await {
                                Ok(control) => control,
                                Err(err) if app.recover_transport_error(&err) => AppRunControl::Continue,
                                Err(err) => {
                                    tracing::error!(error = ?err, "contained terminal input handler failure");
                                    app.chat_widget.add_error_message(format!(
                                        "An input command failed but Corbanu Terminal is still running: {err:#}"
                                    ));
                                    AppRunControl::Continue
                                }
                            }
                        } else {
                            tracing::warn!("terminal input stream closed; shutting down active thread");
                            crate::boxed_future!(app.handle_exit_mode(backends.session_at_mut(displayed_session), ExitMode::ShutdownFirst)).await
                        }
                    }
                    signal = tokio::signal::ctrl_c() => {
                        // SIGINT from outside the terminal (raw mode turns Ctrl+C into a key).
                        if let Err(err) = signal {
                            tracing::warn!(error = %err, "failed to listen for SIGINT");
                            AppRunControl::Continue
                        } else if app.active_thread_id.is_some() && !app.reconnect.offline {
                            match Box::pin(app.handle_event(
                                tui,
                                backends.session_at_mut(displayed_session),
                                AppEvent::CodexOp(
                                    AppCommand::interrupt_and_restore_prompt_if_no_output(),
                                ),
                            ))
                            .await
                            {
                                Ok(control) => control,
                                Err(err) => {
                                    tracing::error!(error = ?err, "contained interrupt handler failure");
                                    app.chat_widget.add_error_message(format!(
                                        "Interrupt failed but Corbanu Terminal is still running: {err:#}"
                                    ));
                                    AppRunControl::Continue
                                }
                            }
                        } else {
                            crate::boxed_future!(app.handle_exit_mode(backends.session_at_mut(displayed_session), ExitMode::ShutdownFirst)).await
                        }
                    }
                    _ = gpu_runtime_overlay_tick.tick() => {
                        app.refresh_gpu_runtime_overlay().await;
                        AppRunControl::Continue
                    }
                    (owner, app_server_event) = backends.next_event(paused_backend) => {
                        match app_server_event {
                            Some(event) if owner != displayed_owner || !app.reconnect.offline => {
                                if let Some(session) = backends.session(owner) {
                                    crate::boxed_future!(app.handle_polled_backend_event(session, displayed_owner, event)).await;
                                }
                            }
                            Some(_) => {}
                            None => {
                                // Corbanu: an embedded server is not reconnected; say so once.
                                if owner == displayed_owner
                                    && matches!(app.app_server_target, AppServerTarget::Embedded)
                                    && !embedded_stream_closed_notified
                                {
                                    embedded_stream_closed_notified = true;
                                    app.chat_widget.add_error_message(
                                        "The app server stopped; restart Corbanu Terminal to continue."
                                            .to_string(),
                                    );
                                }
                                if owner == displayed_owner {
                                    if app.reconnect.offline && app.pending_startup_thread_start {
                                        reconnect = None;
                                    } else if !app.pending_startup_thread_start {
                                        app.begin_reconnect();
                                    }
                                }
                                tracing::warn!("app-server event stream closed");
                            }
                        }
                        AppRunControl::Continue
                    }
                    (reconnect_owner, result) = async { match reconnect.as_mut() { Some((owner, future)) => (*owner, future.await), None => std::future::pending().await } }, if reconnect.is_some() && !has_pending_app_events => {
                        reconnect = None;
                        match result {
                            Ok(connected) if reconnect_owner != displayed_owner => {
                                let _ = crate::boxed_future!(connected.session.shutdown()).await;
                            }
                            Err(_) if reconnect_owner != displayed_owner => {}
                            Ok(connected) => {
                                if let Err(err) = crate::boxed_future!(app.finish_reconnect(tui, backends.session_at_mut(displayed_session), &mut app_event_rx, connected, CODEX_CLI_VERSION)).await {
                                    break Err(err);
                                }
                                backends.resume_events(displayed_owner);
                                crate::boxed_future!(app.restore_codex_user_panes_from_saved_state(backends.session_at_mut(displayed_session))).await;
                                crate::boxed_future!(app.restore_native_spawn_panes_from_saved_state(backends.session_at_mut(displayed_session))).await;
                                waiting_for_initial_session_configured = app.pending_startup_thread_start;
                                if app.pending_startup_thread_start {
                                    spawn_startup_thread_start(
                                        backends.session_at(displayed_session),
                                        app.local_settings.clone(),
                                        app.config.clone(),
                                        launch_choices.clone(),
                                        app.app_event_tx.clone(),
                                        managed_worktree.clone(),
                                    );
                                }
                            }
                            Err(error) => {
                                if app.pending_startup_thread_start {
                                    app.pending_startup_thread_start = false;
                                    app.chat_widget.cancel_startup_submission();
                                    app.chat_widget
                                        .set_initial_user_message_submit_suppressed(
                                            /*suppressed*/ true,
                                        );
                                    if let Some(message) =
                                        app.chat_widget.initial_user_message.take()
                                    {
                                        app.chat_widget.restore_user_message_to_composer(message);
                                    }
                                }
                                app.reconnect.failed = true;
                                app.chat_widget.reconnect_failed();
                                app.chat_widget.add_error_message(error.to_string());
                                if let Ok(mut state) = app.agents_overview.view_state.lock() {
                                    state.connection_notice = Some("Reconnect failed — agent list is stale; relaunch to retry");
                                }
                            }
                        }
                        AppRunControl::Continue
                    }
                    () = async {
                        match pending_resize {
                            Some((_, deadline)) => tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await,
                            None => std::future::pending().await,
                        }
                    } => {
                        if let Some((expected, _)) = pending_resize.take()
                            && crate::terminal_size_telemetry::enabled(&app.config)
                        {
                            let actual = crate::terminal_size_telemetry::read_size();
                            if actual != expected {
                                pending_resize = Some((actual, Instant::now() + Duration::from_secs(/*secs*/ 15)));
                            } else {
                                let tags = crate::terminal_size_telemetry::tags(actual);
                                if tags != settled_size_tags {
                                    crate::terminal_size_telemetry::record(&app.terminal_size_tags, &app.config, "resize", actual);
                                }
                                settled_size_tags = tags;
                            }
                        }
                        AppRunControl::Continue
                    }
                    () = async {
                        match rate_limit_poll_deadline {
                            Some(deadline) => {
                                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                            }
                            None => std::future::pending().await,
                        }
                    }, if backends.is_listening(displayed_owner) => {
                        app.refresh_rate_limits(backends.session_at(displayed_session), RateLimitRefreshOrigin::Periodic);
                        AppRunControl::Continue
                    }
                    () = async {
                        match app.chat_widget.terminal_title_next_refresh {
                            Some(deadline) => {
                                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                            }
                            None => std::future::pending().await,
                        }
                    } => {
                        app.chat_widget.refresh_goal_status_indicator_for_time_tick();
                        app.chat_widget.refresh_terminal_title();
                        AppRunControl::Continue
                    }
                    () = async {
                        match app.commit_animation.as_mut() {
                            Some(interval) => {
                                interval.tick().await;
                            }
                            None => std::future::pending().await,
                        }
                    }, if !has_pending_app_events => {
                        crate::session_log::log_commit_tick();
                        app.chat_widget.on_commit_tick();
                        AppRunControl::Continue
                    }
                };
                if App::should_stop_waiting_for_initial_session(
                    waiting_for_initial_session_configured,
                    app.primary_thread_id,
                ) {
                    waiting_for_initial_session_configured = false;
                    let had_active_modal = app.chat_widget.has_active_modal();
                    if let Err(err) = app.drain_active_thread_events(tui).await {
                        break Err(err);
                    }
                    if !had_active_modal
                        && app.chat_widget.has_active_modal()
                        && let Err(err) = app.render_startup_frame(tui, &app_event_rx)
                    {
                        break Err(err);
                    }
                }
                match control {
                    AppRunControl::Continue => {
                        if app.reconnect.offline && !app.pending_startup_thread_start {
                            pending_startup_submission = false;
                            app.chat_widget.cancel_startup_submission();
                        }
                        if app_event_rx.is_empty() && !app.has_queued_startup_protected_request() {
                            app.chat_widget.restore_startup_input_when_ready(
                                &mut pending_startup_draft,
                                &mut pending_startup_submission,
                            );
                        }
                        #[cfg(windows)]
                        if terminal_color_probe_pending
                            && app.ready_for_terminal_color_probe(!app_event_rx.is_empty())
                        {
                            tui.probe_default_colors_after_protected_startup();
                            terminal_color_probe_pending = false;
                        }
                    }
                    AppRunControl::Exit(reason) => break Ok(reason),
                }
            }
        };
        // Let the pending terminal poll unwind while the app server shuts down.
        tui.pause_events();
        app.chat_widget.mention_usage_mut().flush().await;
        let shutdown_detail = match &exit_reason_result {
            Ok(_) => "user or terminal exit",
            Err(_) => "TUI event loop failure",
        };
        if let Err(error) =
            crate::boxed_future!(app.show_backend_shutdown_control_events(tui, shutdown_detail))
                .await
        {
            tracing::error!(
                error = ?error,
                error_chain = %format!("{error:#}"),
                "failed to render backend shutdown control events"
            );
        }
        app.abort_all_thread_event_listeners();
        if let Err(err) = crate::boxed_future!(backends.shutdown()).await {
            tracing::error!(error = ?err, "failed to shut down embedded app server");
        }
        let clear_pet_result = tui.clear_ambient_pet_image();
        let clear_images = tui.terminal.clear_frame_images(&mut tui.retained_images);
        let clear_result = clear_images.and(tui.terminal.clear());
        if let Some(task) = app.daemon_restart.take()
            && let Err(err) = task.await.unwrap_or_else(|err| Err(err.into()))
        {
            tracing::warn!(error = %err, "failed to finish background server restart during exit");
        }
        // Keep the alternate screen active until the outer guard restores both keyboard stacks.
        let exit_reason = match exit_reason_result {
            Ok(exit_reason) => {
                clear_pet_result?;
                clear_result?;
                exit_reason
            }
            Err(err) => {
                if let Err(clear_pet_err) = clear_pet_result {
                    tracing::warn!(error = %clear_pet_err, "failed to clear ambient pet image");
                }
                if let Err(clear_err) = clear_result {
                    tracing::warn!(error = %clear_err, "failed to clear terminal UI");
                }
                return Err(err);
            }
        };
        // D243 (#1412): the conversation on screen at quit is the one `resume --last` reopens.
        if matches!(exit_reason, ExitReason::UserRequested)
            && let Some(thread) = app.exit_resumable_thread()
        {
            crate::quit_thread_record::record_quit_thread(
                app.config.codex_home.as_ref(),
                thread.thread_id,
                chrono::Utc::now().timestamp(),
            );
        }
        Ok(app.exit_info(exit_reason))
    }
}
