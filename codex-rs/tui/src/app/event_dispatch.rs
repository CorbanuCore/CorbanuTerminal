//! AppEvent dispatch for the TUI app.
//!
//! This module contains the exhaustive `AppEvent` dispatcher and exit-mode handling. Large domain
//! actions are delegated to focused app submodules so the central match remains the routing layer.

use super::rate_limit_refresh::RateLimitReadStatus;
use super::rate_limit_refresh::RateLimitRefreshOutcome;
use super::resize_reflow::trailing_run_start;
use super::session_lifecycle::ThreadAttachPresentation;
use super::thread_settings::SettingsUpdateOutcome;
use super::*;
<<<<<<< HEAD
use crate::app_event::WalletUnlockContinuation;
||||||| 413492cd6c
=======
use crate::app_event::RecapTrigger;
use crate::app_event::ThreadTitleDestination;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::app_server_session::ForkGoalContinuation;
use crate::app_server_session::ThreadSettingsOwner;
use crate::app_server_session::UnsupportedLegacyPermissionProfile;
use crate::app_server_session::turn_permissions_overrides;
use crate::config_update::format_config_error;
use crate::external_agent_config_migration::flow::ExternalAgentConfigMigrationFlowOutcome;
<<<<<<< HEAD
use chrono::Utc;
use codex_app_server_protocol::ThreadAgentMessageParams;
||||||| 413492cd6c
=======
use crate::pager_overlay::TranscriptHistoryState;
use crate::session_resume::cwds_differ;
use crate::status::StatusAccountDisplay;
use codex_app_server_client::AppServerEvent;
use codex_app_server_protocol::ThreadGoalStatus;
use codex_app_server_protocol::ThreadStatus;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
#[cfg(target_os = "windows")]
<<<<<<< HEAD
use codex_config::types::WindowsSandboxModeToml;
use codex_product_brand::LEGACY_MAIN_PANE_TITLE;
use codex_product_brand::MAIN_PANE_TITLE;
use codex_product_brand::native_pane_name;
use codex_product_brand::native_pane_title;
use codex_protocol::protocol::AgentMessageKind;
use std::collections::HashSet;
||||||| 413492cd6c
use codex_config::types::WindowsSandboxModeToml;
=======
use codex_app_server_protocol::WindowsSandboxSetupMode;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

pub(super) const SHUTDOWN_FIRST_EXIT_TIMEOUT: Duration = Duration::from_secs(/*secs*/ 2);

impl App {
<<<<<<< HEAD
    // This local-only route has no app-server client or account request capability.
    pub(super) fn handle_accounting_inspector_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::OpenAccountingInspector { day } => {
                self.chat_widget.open_accounting_inspector(day)
            }
            AppEvent::LoadAccountingInspector {
                generation,
                thread,
                day,
                range,
            } => {
                let db = self.state_db.clone();
                let embedded = matches!(self.app_server_target, crate::AppServerTarget::Embedded);
                let current = self.current_displayed_thread_id();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = if !embedded {
                        Err(format!(
                            "Unavailable — recorded request inspection is local-only; remote server selected. Next step: run {} in a session started without a remote server.",
                            crate::chatwidget::cost_command()
                        ))
                    } else if thread.is_none() {
                        Err(format!(
                            "Unavailable — no current native thread. Next step: send a turn or open a conversation with /resume, then run {cost}.",
                            cost = crate::chatwidget::cost_command()
                        ))
                    } else if thread != current {
                        Err(format!(
                            "Unavailable — the conversation changed while this view was loading. Next step: run {} again.",
                            crate::chatwidget::cost_command()
                        ))
                    } else if let (Some(db), Some(owner)) = (db, thread) {
                        accounting_inspector_read_result(async {
                            let now = chrono::Utc::now().timestamp_millis();
                            match range {
                                Some(range) => {
                                    codex_state::accounting::AccountingStore::inspect_range(
                                        &db, owner, range, now,
                                    )
                                    .await
                                }
                                None => {
                                    codex_state::accounting::AccountingStore::inspect_day(
                                        &db, owner, day, now,
                                    )
                                    .await
                                }
                            }
                        })
                        .await
                    } else {
                        Err("Unavailable — native state database is not open. Next step: restart Corbanu Terminal; if it stays unavailable, check that its home directory is writable.".into())
                    };
                    tx.send(AppEvent::AccountingInspectorLoaded {
                        generation,
                        thread,
                        day,
                        result,
                    });
                });
            }
            AppEvent::AccountingInspectorLoaded {
                generation,
                thread,
                day,
                result,
            } => {
                if thread == self.current_displayed_thread_id() {
                    self.chat_widget
                        .finish_accounting_inspector(generation, thread, day, result);
                }
            }
            AppEvent::NavigateAccountingInspector { generation, page } => self
                .chat_widget
                .navigate_accounting_inspector(generation, page),
            AppEvent::CloseAccountingInspector { generation } => {
                self.chat_widget.close_accounting_inspector(generation)
            }
            AppEvent::RefreshAccountingInspector { generation } => {
                self.chat_widget.refresh_accounting_inspector(generation)
            }
            _ => unreachable!("non-inspector event"),
        }
    }
}

pub(super) async fn accounting_inspector_read_result(
    read: impl std::future::Future<Output = anyhow::Result<codex_state::accounting::InspectionDay>>,
) -> Result<codex_state::accounting::InspectionDay, String> {
    match tokio::time::timeout(Duration::from_secs(15), read).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => {
            tracing::warn!(error = %format!("{error:#}"), "accounting inspection failed");
            if let Some(clock) = codex_state::accounting::clock_behind_ledger(&error) {
                return Err(clock_behind_ledger_message(clock.behind_ms));
            }
            if let Some(clock) = codex_state::accounting::clock_before_day(&error) {
                return Err(clock_before_day_message(clock.behind_ms));
            }
            Err("Unavailable — accounting evidence is corrupt, incompatible or could not be read. Refresh to retry; no repair performed. Next step: if Refresh fails again, check your provider's bill; the log names the cause.".into())
        }
        Err(_) => Err("Unavailable — inspection timed out. Next step: select Refresh; another Corbanu process may be holding the ledger.".into()),
    }
}

/// #1518: the ledger is intact but newer than the system clock, so the read is
/// refused until the clock catches up. Says so instead of suggesting damage.
pub(super) fn clock_behind_ledger_message(behind_ms: i64) -> String {
    let behind = clock_lag(behind_ms);
    format!(
        "Unavailable — your system clock is behind the newest recorded usage ({behind}). The accounting data is not damaged and nothing was changed. Next step: check your system date and time, then select Refresh; totals show again once the clock passes the newest record."
    )
}

/// #1522: the view's UTC day starts after the system clock (the view was
/// opened while the clock read ahead, then the clock was set back). Refused
/// before the ledger is read, so claim only that, not that the data is sound.
pub(super) fn clock_before_day_message(behind_ms: i64) -> String {
    let behind = clock_lag(behind_ms);
    format!(
        "Unavailable — your system clock reads earlier than the day this view shows ({behind} before it starts). This is not a sign of damage; nothing was read or changed. Next step: check your system date and time, then select Refresh, or close this view and run {cost} again for the clock's current day.",
        cost = crate::chatwidget::cost_command()
    )
}

fn clock_lag(behind_ms: i64) -> String {
    const MINUTE: i64 = 60_000;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    let rounded = |unit: i64| behind_ms.saturating_add(unit / 2) / unit;
    let about = |n: i64, unit: &str| format!("about {n} {unit}{}", if n == 1 { "" } else { "s" });
    if behind_ms < MINUTE {
        "less than a minute".to_string()
    } else if rounded(MINUTE) < 60 {
        about(rounded(MINUTE), "minute")
    } else if rounded(HOUR) < 48 {
        about(rounded(HOUR), "hour")
    } else {
        about(rounded(DAY), "day")
    }
}

fn resolve_shared_provider_selection_model(
    configured_model: Option<String>,
    active_model: &str,
    runtime_provider_id: &str,
) -> Option<String> {
    let model = configured_model
        .filter(|model| !model.trim().is_empty())
        .or_else(|| {
            let active_model = active_model.trim();
            (!active_model.is_empty()).then(|| active_model.to_string())
        });
    codex_model_provider_info::resolve_model_for_provider(model, runtime_provider_id)
}

fn refresh_shared_provider_session_if_idle(
    session: Option<&mut crate::onboarding::provider_setup::ProviderSetupSession>,
    resolve: impl FnOnce() -> Vec<codex_provider_auth::ProviderStatusSnapshot>,
) -> bool {
    let Some(session) = session else {
        return false;
    };
    if !session.can_refresh_statuses() {
        return false;
    }
    let statuses = resolve();
    session.refresh_from_statuses(&statuses)
}

fn wallet_unlock_continuation_is_current(
    active: Option<&crate::onboarding::provider_setup::DeferredProviderSetup>,
    continuation: &WalletUnlockContinuation,
) -> bool {
    match continuation {
        WalletUnlockContinuation::OpenCorbanuApi { deferred }
        | WalletUnlockContinuation::CorbanuApiOperation { deferred, .. } => {
            active == deferred.as_ref()
        }
        _ => true,
    }
}

fn corbanu_api_continuation_is_current(
    active: Option<&crate::onboarding::provider_setup::DeferredProviderSetup>,
    deferred: Option<&crate::onboarding::provider_setup::DeferredProviderSetup>,
) -> bool {
    active == deferred
}

const RESERVED_PANE_DISPLAY_NAMES: &[&str] = &[
    MAIN_PANE_TITLE,
    LEGACY_MAIN_PANE_TITLE,
    "Codex - Main",
    "me",
    "none",
    "root",
    "nazgul",
];

impl App {
    pub(super) async fn handle_resume_picker_event(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        tui_events: &mut (dyn Stream<Item = TuiEvent> + Send + Unpin),
    ) -> Result<AppRunControl> {
        let picker_app_server = match crate::start_app_server_for_picker(
            &self.config,
            &self.app_server_target,
            self.state_db.clone(),
            self.environment_manager.clone(),
        )
        .await
        {
            Ok(app_server) => app_server,
            Err(err) => {
                self.chat_widget
                    .add_error_message(format!("Failed to start TUI session picker: {err}"));
                self.chat_widget.maybe_send_next_queued_input();
                return Ok(AppRunControl::Continue);
            }
        };
        let selection =
            crate::resume_picker::run_resume_picker_from_existing_session_with_app_server(
                tui,
                &self.config,
                /*show_all*/ false,
                /*include_non_interactive*/ false,
                picker_app_server,
                tui_events,
            )
            .await?;
        match selection {
            SessionSelection::Resume(target_session) => {
                match self
                    .resume_target_session_with_events(tui, app_server, target_session, tui_events)
                    .await?
                {
                    AppRunControl::Continue => {}
                    AppRunControl::Exit(reason) => return Ok(AppRunControl::Exit(reason)),
                }
            }
            SessionSelection::Exit
            | SessionSelection::StartFresh
            | SessionSelection::ResumePanesOnly { .. } => {
                self.refresh_in_memory_config_from_disk_best_effort("closing the session picker")
                    .await;
            }
            SessionSelection::Fork(_) => {}
        }
        self.chat_widget.maybe_send_next_queued_input();
        tui.frame_requester().schedule_frame();
        Ok(AppRunControl::Continue)
    }

    pub(super) async fn handle_external_agent_config_migration_event(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        tui_events: &mut mpsc::UnboundedReceiver<TuiEvent>,
    ) {
        match crate::external_agent_config_migration::flow::handle_external_agent_config_migration_prompt(
            tui,
            app_server,
            &self.config,
            tui_events,
        )
        .await
        {
            Ok(ExternalAgentConfigMigrationFlowOutcome::Started(lines)) => {
                self.chat_widget.add_plain_history_lines(lines);
            }
            Ok(ExternalAgentConfigMigrationFlowOutcome::NoItems) => {
                self.chat_widget.add_info_message(
                    crate::external_agent_config_migration::flow::EXTERNAL_AGENT_CONFIG_MIGRATION_NO_ITEMS_MESSAGE
                        .to_string(),
                    /*hint*/ None,
                );
            }
            Ok(ExternalAgentConfigMigrationFlowOutcome::Cancelled) => {}
            Err(error_message) => {
                self.chat_widget.add_error_message(error_message);
            }
        }
        tui.frame_requester().schedule_frame();
    }

    /// PF-84: the model list again, after the session moved to another
    /// account's sign-in (the app server lists with that account now).
    /// Returns whether the list was replaced.
    pub(super) async fn refresh_model_list(&mut self, app_server: &mut AppServerSession) -> bool {
        match app_server.list_models().await {
            Ok(models) if !models.is_empty() => {
                self.model_catalog.replace_listed_models(models);
                let model = self.chat_widget.current_model().to_string();
                let provider_id = self.chat_widget.config_ref().model_provider_id.clone();
                self.model_catalog.sync_runtime_models(
                    [provider_id.as_str()],
                    Some(&model),
                    &provider_id,
                );
                true
            }
            Ok(_) => {
                tracing::warn!("model/list returned no models after an account switch");
                false
            }
            Err(error) => {
                tracing::warn!("model/list failed after an account switch: {error}");
                false
            }
        }
    }

    pub(super) async fn refresh_gpu_runtime_overlay(&mut self) {
        let Some(state_db) = self.state_db.as_ref() else {
            self.model_catalog.replace_gpu_models(Vec::new());
            self.refresh_gpu_spend_indicator().await;
            return;
        };
        let providers = match state_db.list_gpu_runtime_providers().await {
            Ok(providers) => providers,
            Err(error) => {
                tracing::warn!(%error, "failed to refresh GPU runtime provider overlay");
                self.refresh_gpu_spend_indicator().await;
                return;
            }
        };
        let models = providers
            .iter()
            .filter_map(super::gpu_runtime_model_preset)
            .collect();
        let runtime_providers = providers
            .into_iter()
            .filter_map(|provider| {
                crate::legacy_core::config::gpu_runtime_model_provider(
                    provider,
                    &self.config.codex_home,
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        self.model_catalog.replace_gpu_models(models);
        self.config
            .model_providers
            .retain(|provider_id, _| !provider_id.starts_with("gpu-"));
        self.config
            .model_providers
            .extend(runtime_providers.clone());
        self.chat_widget
            .replace_gpu_model_providers(&runtime_providers);
        self.refresh_gpu_notifications().await;
        self.refresh_gpu_spend_indicator().await;
    }

    async fn refresh_gpu_notifications(&mut self) {
        let Some(state_db) = self.state_db.clone() else {
            return;
        };
        let Ok(rentals) = state_db.list_gpu_rentals(/*limit*/ 1_000).await else {
            return;
        };
        let now_ms = chrono::Utc::now().timestamp_millis();
        for rental in rentals {
            let Some((kind, message, is_error)) = gpu_notification(&rental) else {
                continue;
            };
            match state_db
                .record_gpu_notification_once(
                    rental.rental_id.as_str(),
                    rental.state_sequence,
                    kind.as_str(),
                    now_ms,
                )
                .await
            {
                Ok(true) if is_error => self.chat_widget.add_error_message(message),
                Ok(true) => self.chat_widget.add_info_message(message, /*hint*/ None),
                Ok(false) => {}
                Err(error) => {
                    tracing::warn!(%error, "failed to deduplicate GPU rental notification")
                }
            }
        }
    }

    fn normalize_pane_display_name(&self, name: &str) -> Result<String> {
        let normalized = name.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty() {
            color_eyre::eyre::bail!("Pane name cannot be empty.");
        }
        if RESERVED_PANE_DISPLAY_NAMES
            .iter()
            .any(|reserved| reserved.eq_ignore_ascii_case(&normalized))
        {
            color_eyre::eyre::bail!("`{normalized}` is reserved; choose a different pane name.");
        }
        Ok(normalized)
    }

    fn occupied_pane_name_keys(&self, exclude_node_id: Option<&str>) -> HashSet<String> {
        let mut occupied = HashSet::new();
        let mut insert = |value: &str| {
            let folded = value.trim().to_ascii_lowercase();
            if !folded.is_empty() {
                occupied.insert(folded);
            }
        };

        for (thread_id, entry) in self.agent_navigation.ordered_threads() {
            let node_id = crate::spawn_orchestration::thread_node_id(thread_id);
            if exclude_node_id == Some(node_id.as_str()) {
                continue;
            }
            let label = self.thread_label(thread_id);
            insert(&label);
            if let Some(nickname) = entry.agent_nickname.as_deref() {
                insert(nickname);
                if Some(thread_id) != self.primary_thread_id
                    && entry
                        .agent_role
                        .as_deref()
                        .map(|role| role == "default")
                        .unwrap_or(true)
                {
                    insert(&native_pane_title(nickname));
                    insert(&format!("PFTerminal - {nickname}"));
                    insert(&format!("Codex - {nickname}"));
                }
            }
        }
        for pane in self.claude_panes.panes() {
            let node_id = crate::spawn_orchestration::pane_node_id(&pane.id);
            if exclude_node_id == Some(node_id.as_str()) {
                continue;
            }
            insert(&pane.title);
        }
        occupied
    }

    fn unique_pane_display_name(
        &self,
        requested: &str,
        exclude_node_id: Option<&str>,
    ) -> Result<String> {
        let base = self.normalize_pane_display_name(requested)?;
        let occupied = self.occupied_pane_name_keys(exclude_node_id);
        let mut candidate = base.clone();
        for suffix in 2..1000 {
            if !occupied.contains(&candidate.to_ascii_lowercase()) {
                return Ok(candidate);
            }
            candidate = format!("{base} ({suffix})");
        }
        color_eyre::eyre::bail!("Could not find an unused pane name for `{base}`.");
    }

    fn unique_native_pane_nickname(
        &self,
        requested: &str,
        exclude_thread_id: Option<ThreadId>,
    ) -> Result<String> {
        let mut base = self.normalize_pane_display_name(requested)?;
        if let Some(stripped) = base.strip_prefix("Codex - ") {
            base = self.normalize_pane_display_name(stripped)?;
        }
        if let Some(stripped) = native_pane_name(&base).map(str::to_owned) {
            base = self.normalize_pane_display_name(&stripped)?;
        }
        let exclude_node_id = exclude_thread_id.map(crate::spawn_orchestration::thread_node_id);
        let occupied = self.occupied_pane_name_keys(exclude_node_id.as_deref());
        let mut candidate = base.clone();
        for suffix in 2..1000 {
            let candidate_key = candidate.to_ascii_lowercase();
            let corbanu_display_key =
                format!("corbanu terminal - {}", candidate.to_ascii_lowercase());
            let pfterminal_display_key = format!("pfterminal - {}", candidate.to_ascii_lowercase());
            let legacy_display_key = format!("codex - {}", candidate.to_ascii_lowercase());
            if !occupied.contains(&candidate_key)
                && !occupied.contains(&corbanu_display_key)
                && !occupied.contains(&pfterminal_display_key)
                && !occupied.contains(&legacy_display_key)
            {
                return Ok(candidate);
            }
            candidate = format!("{base} ({suffix})");
        }
        color_eyre::eyre::bail!("Could not find an unused pane name for `{base}`.");
    }

    async fn rename_current_pane_display_name(
        &mut self,
        app_server: &mut AppServerSession,
        name: String,
    ) {
        if let Some(pane_id) = self
            .claude_panes
            .active_claude_pane_id()
            .map(ToString::to_string)
        {
            self.rename_claude_pane_display_name(pane_id, name);
            return;
        };

        let Some(thread_id) = self.current_displayed_thread_id() else {
            self.chat_widget
                .add_error_message("No active Corbanu Terminal pane to rename.".to_string());
            return;
        };
        self.rename_codex_pane_display_name(app_server, thread_id, name)
            .await;
    }

    fn rename_claude_pane_display_name(&mut self, pane_id: String, name: String) {
        let exclude = crate::spawn_orchestration::pane_node_id(&pane_id);
        let title = match self.unique_pane_display_name(&name, Some(exclude.as_str())) {
            Ok(title) => title,
            Err(err) => {
                self.chat_widget.add_error_message(err.to_string());
                return;
            }
        };
        match self.claude_panes.rename_pane(&pane_id, title.clone()) {
            Ok(()) => {
                self.sync_active_agent_label();
                self.persist_pane_state();
                self.chat_widget
                    .add_info_message(format!("Renamed pane to {title}."), /*hint*/ None);
            }
            Err(err) => self
                .chat_widget
                .add_error_message(format!("Failed to rename pane: {err}")),
        }
    }

    async fn rename_codex_pane_display_name(
        &mut self,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
        name: String,
    ) {
        let nickname = match self.unique_native_pane_nickname(&name, Some(thread_id)) {
            Ok(nickname) => nickname,
            Err(err) => {
                self.chat_widget.add_error_message(err.to_string());
                return;
            }
        };
        let backend_name = nickname.clone();
        if let Err(err) = app_server.thread_set_name(thread_id, backend_name).await {
            self.chat_widget
                .add_error_message(format!("Failed to rename thread: {err}"));
            return;
        }
        self.agent_navigation
            .set_agent_nickname(thread_id, Some(nickname.clone()));
        self.persist_existing_thread_agent_nickname(thread_id, Some(nickname.clone()))
            .await;
        self.sync_active_agent_label();
        self.persist_pane_state();
        self.chat_widget
            .add_info_message(format!("Renamed pane to {nickname}."), /*hint*/ None);
    }

    async fn persist_existing_thread_agent_nickname(
        &self,
        thread_id: ThreadId,
        agent_nickname: Option<String>,
    ) {
        let Some(state_db) = self.state_db.as_ref() else {
            return;
        };
        let Ok(Some(mut metadata)) = state_db.get_thread(thread_id).await else {
            return;
        };
        let now = Utc::now();
        metadata.updated_at = now;
        metadata.recency_at = now;
        metadata.agent_nickname = agent_nickname;
        if let Err(err) = state_db.upsert_thread(&metadata).await {
            tracing::warn!(
                thread_id = %thread_id,
                error = %err,
                "failed to persist renamed pane nickname"
            );
        }
    }
}

fn gpu_notification(rental: &codex_state::GpuRental) -> Option<(String, String, bool)> {
    use codex_state::GpuRentalState;
    match rental.observed_state {
        GpuRentalState::Ready => Some((
            "ready".to_string(),
            format!(
                "GPU rental {} is READY and its model is available in the picker.",
                rental.rental_id
            ),
            false,
        )),
        GpuRentalState::Degraded => Some((
            "degraded".to_string(),
            format!(
                "GPU rental {} is DEGRADED and has been disabled for new selections.",
                rental.rental_id
            ),
            true,
        )),
        GpuRentalState::Failed => Some((
            "failed".to_string(),
            failed_gpu_notification(
                rental.rental_id.as_str(),
                rental.last_error_message.as_deref(),
            ),
            true,
        )),
        GpuRentalState::TerminationUnconfirmed => Some((
            "termination-unconfirmed".to_string(),
            format!(
                "GPU rental {} cleanup is UNCONFIRMED; it remains visible as a billing risk.",
                rental.rental_id
            ),
            true,
        )),
        GpuRentalState::TerminatedConfirmed => Some((
            "terminated".to_string(),
            format!(
                "GPU rental {} termination is provider-confirmed.",
                rental.rental_id
            ),
            false,
        )),
        _ => rental.provision_step.as_deref().and_then(|step| {
            crate::chatwidget::gpu_menu::gpu_provision_phase_label(step)?;
            let progress = crate::chatwidget::gpu_menu::gpu_progress_summary(
                rental,
                chrono::Utc::now().timestamp_millis(),
            );
            Some((
                format!("progress-{step}"),
                format!(
                    "GPU rental {}: {progress}. Provider billing is active; /gpu shows current spend and termination controls.",
                    rental.rental_id
                ),
                false,
            ))
        }),
    }
}

fn failed_gpu_notification(rental_id: &str, reason: Option<&str>) -> String {
    reason.map_or_else(
        || format!("GPU rental {rental_id} failed before a billable resource was confirmed."),
        |reason| {
            format!(
                "GPU rental {rental_id} failed before a billable resource was confirmed: {reason}"
            )
        },
    )
}

impl App {
    #[cfg(test)]
    pub(super) fn start_gpu_controller(&mut self) {}

    #[cfg(not(test))]
    pub(super) fn start_gpu_controller(&mut self) {
        let executable = match std::env::current_exe() {
            Ok(executable) => executable,
            Err(error) => {
                self.chat_widget.add_error_message(format!(
                    "GPU rental state was saved, but the independent controller could not be located: {error}"
                ));
                return;
            }
        };
        let mut command = tokio::process::Command::new(executable);
        command
            .arg("internal-gpu-controller")
            .env("CODEX_HOME", self.config.codex_home.as_path())
            .env(codex_state::SQLITE_HOME_ENV, self.config.sqlite.home())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        match super::background_child::spawn_with_reaper(&mut command) {
            Ok(_) => {}
            Err(error) => self.chat_widget.add_error_message(format!(
                "GPU rental state was saved, but the independent controller did not start: {error}. Run `corbanu internal-gpu-controller` before relying on local TTL or spend enforcement."
            )),
        }
    }

    pub(super) async fn refresh_gpu_spend_indicator(&mut self) {
        let Some(state_db) = self.state_db.as_ref() else {
            self.chat_widget.set_gpu_spend_status(/*status*/ None);
            return;
        };
        let status = state_db
            .list_gpu_rentals(/*limit*/ 1_000)
            .await
            .ok()
            .and_then(|rentals| {
                let billable = rentals
                    .into_iter()
                    .filter(codex_state::GpuRental::may_be_billable)
                    .collect::<Vec<_>>();
                if billable.is_empty() {
                    return None;
                }
                let estimated = billable
                    .iter()
                    .map(|rental| rental.estimated_accrued_microusd)
                    .sum::<i64>() as f64
                    / 1_000_000.0;
                let hourly_cap = billable
                    .iter()
                    .map(|rental| rental.max_hourly_microusd)
                    .sum::<i64>() as f64
                    / 1_000_000.0;
                Some(format!(
                    "GPU SPEND {} active · ${estimated:.2} est · ≤${hourly_cap:.2}/hr",
                    billable.len()
                ))
            });
        self.chat_widget.set_gpu_spend_status(status);
    }

    fn render_shared_provider_setup(&mut self) {
        let (Some(host), Some(session)) = (
            self.shared_provider_status_host.as_ref(),
            self.shared_provider_setup_session.as_ref(),
        ) else {
            return;
        };
        self.chat_widget.open_shared_provider_setup(
            host,
            session.snapshot().queued_corbanu,
            session.can_finish(),
        );
    }

    fn finish_shared_provider_api_key(
        &mut self,
        target: codex_provider_auth::ApiKeyAuthTarget,
        status: Option<codex_provider_auth::ProviderStatusSnapshot>,
    ) {
        let Some(session) = self.shared_provider_setup_session.as_mut() else {
            return;
        };
        let Some(persisted_status) = status.filter(|status| {
            status.id == target.provider_id
                && status.configuration
                    == codex_provider_auth::ProviderConfigurationState::Configured
        }) else {
            session.dispatch(crate::onboarding::provider_setup::ProviderSetupAction::AuthFailed);
            self.chat_widget
                .add_error_message("Provider credential could not be stored.".to_string());
            return;
        };
        let runtime_provider_id = target.runtime_provider_id.clone();
        let transition = session.dispatch(
            crate::onboarding::provider_setup::ProviderSetupAction::AuthConfigured {
                provider_id: target.provider_id.clone(),
                runtime_provider_id: runtime_provider_id.clone(),
            },
        );
        if !transition.applied {
            return;
        }
        let Some(host) = self.shared_provider_status_host.as_ref() else {
            return;
        };
        if !host.activate(target.provider_id.as_str()) {
            session.dispatch(crate::onboarding::provider_setup::ProviderSetupAction::AuthFailed);
            self.chat_widget
                .add_error_message("Provider activation could not be persisted.".to_string());
            return;
        }
        let status = host.resolve_target(&target);
        self.model_catalog.refresh_provider_policy();
        let usable = persisted_status.availability
            == codex_provider_auth::ProviderAvailabilityState::Ready
            && status.configuration == codex_provider_auth::ProviderConfigurationState::Configured
            && status.eligibility == codex_provider_auth::ProviderEligibilityState::Active
            && status.availability == codex_provider_auth::ProviderAvailabilityState::Ready;
        let resolved = session.dispatch(
            crate::onboarding::provider_setup::ProviderSetupAction::ActivationResolved {
                provider_id: target.provider_id,
                runtime_provider_id,
                usable,
            },
        );
        for effect in resolved.effects {
            if let crate::onboarding::provider_setup::ProviderSetupEffect::PersistInitialSelection(
                runtime,
            ) = effect
            {
                let Some(model) = resolve_shared_provider_selection_model(
                    self.config.model.clone(),
                    self.chat_widget.current_model(),
                    runtime.as_str(),
                ) else {
                    self.chat_widget.add_error_message(
                        "No compatible model is available for the configured provider.".to_string(),
                    );
                    continue;
                };
                self.app_event_tx.send(AppEvent::UpdateModelSelection {
                    model: model.clone(),
                    provider: Some(runtime.to_string()),
                });
                self.app_event_tx.send(AppEvent::PersistModelSelection {
                    model,
                    provider: Some(runtime.to_string()),
                    effort: None,
                });
            }
        }
        if !usable {
            self.chat_widget.add_error_message(
                "Provider is configured and active, but is not currently usable.".to_string(),
            );
        }
    }

    fn finish_or_defer_shared_provider_setup(&mut self) {
        let Some(session) = self.shared_provider_setup_session.as_mut() else {
            return;
        };
        let transition =
            session.dispatch(crate::onboarding::provider_setup::ProviderSetupAction::Done);
        if !transition.applied {
            self.render_shared_provider_setup();
            return;
        }
        for effect in transition.effects {
            match effect {
                crate::onboarding::provider_setup::ProviderSetupEffect::BeginDeferred(deferred) => {
                    self.active_deferred_provider_setup = Some(deferred.clone());
                    self.app_event_tx
                        .send(AppEvent::BeginDeferredCorbanuPlan { deferred });
                }
                crate::onboarding::provider_setup::ProviderSetupEffect::Finish => {
                    self.shared_provider_setup_session = None;
                    self.shared_provider_status_host = None;
                }
                _ => {}
            }
        }
    }

    fn handle_shared_provider_auth_action(
        &mut self,
        action: codex_provider_auth::ProviderAuthAction,
    ) {
        let Some(host) = self.shared_provider_account_auth_host.as_mut() else {
            return;
        };
        let presentations = host.dispatch(action);
        for presentation in presentations {
            match presentation {
                crate::provider_account_auth_host::ProviderAccountPresentation::Pending(kind) => {
                    self.chat_widget.open_shared_account_pending(kind);
                }
                crate::provider_account_auth_host::ProviderAccountPresentation::OpenAiChallenge {
                    challenge,
                } => self.chat_widget.open_shared_openai_challenge(challenge),
                crate::provider_account_auth_host::ProviderAccountPresentation::ClaudeMethodChoice { recovery } => {
                    self.chat_widget.open_shared_claude_method_choice(recovery);
                }
                crate::provider_account_auth_host::ProviderAccountPresentation::ClaudeManagedTokenEntry => {
                    self.chat_widget.open_shared_claude_managed_token_entry();
                }
                crate::provider_account_auth_host::ProviderAccountPresentation::ClaudeChallenge {
                    challenge,
                } => self.chat_widget.open_shared_claude_challenge(challenge),
                crate::provider_account_auth_host::ProviderAccountPresentation::Completion(
                    completion,
                ) => {
                    self.chat_widget.dismiss_shared_account_auth();
                    self.apply_shared_account_completion(completion);
                }
                crate::provider_account_auth_host::ProviderAccountPresentation::Failed(failure) => {
                    self.chat_widget.open_shared_account_failure(failure);
                }
            }
        }
    }

    fn apply_shared_account_completion(
        &mut self,
        completion: codex_provider_auth::ProviderAuthCompletion,
    ) {
        let (provider_id, runtime_provider_id, status) = match completion {
            codex_provider_auth::ProviderAuthCompletion::OpenAiAccount(
                codex_provider_auth::OpenAiAccountCompletion::Configured { target, status },
            ) => (target.provider_id, target.runtime_provider_id, Some(status)),
            codex_provider_auth::ProviderAuthCompletion::ClaudeAccount(
                codex_provider_auth::claude_account_flow::ClaudeAccountCompletion::Configured {
                    target,
                    status,
                },
            ) => (target.provider_id, target.runtime_provider_id, Some(status)),
            codex_provider_auth::ProviderAuthCompletion::OpenAiAccount(
                codex_provider_auth::OpenAiAccountCompletion::Cancelled { .. },
            )
            | codex_provider_auth::ProviderAuthCompletion::ClaudeAccount(
                codex_provider_auth::claude_account_flow::ClaudeAccountCompletion::Cancelled {
                    ..
                },
            ) => {
                if let Some(session) = self.shared_provider_setup_session.as_mut() {
                    session.dispatch(
                        crate::onboarding::provider_setup::ProviderSetupAction::AuthCancelled,
                    );
                }
                self.render_shared_provider_setup();
                return;
            }
            _ => return,
        };
        let Some(status) = status.filter(|status| {
            status.id == provider_id
                && status.configuration
                    == codex_provider_auth::ProviderConfigurationState::Configured
        }) else {
            return;
        };
        let Some(session) = self.shared_provider_setup_session.as_mut() else {
            return;
        };
        let configured = session.dispatch(
            crate::onboarding::provider_setup::ProviderSetupAction::AuthConfigured {
                provider_id: provider_id.clone(),
                runtime_provider_id: runtime_provider_id.clone(),
            },
        );
        let Some(host) = self.shared_provider_status_host.as_ref() else {
            return;
        };
        if !configured.applied || !host.activate(provider_id.as_str()) {
            session.dispatch(crate::onboarding::provider_setup::ProviderSetupAction::AuthFailed);
            return;
        }
        let active = host.resolve_provider(provider_id.as_str());
        self.model_catalog.refresh_provider_policy();
        let usable = status.availability == codex_provider_auth::ProviderAvailabilityState::Ready
            && active.as_ref().is_some_and(|status| {
                status.eligibility == codex_provider_auth::ProviderEligibilityState::Active
                    && status.availability == codex_provider_auth::ProviderAvailabilityState::Ready
            });
        let resolved = session.dispatch(
            crate::onboarding::provider_setup::ProviderSetupAction::ActivationResolved {
                provider_id,
                runtime_provider_id,
                usable,
            },
        );
        for effect in resolved.effects {
            if let crate::onboarding::provider_setup::ProviderSetupEffect::PersistInitialSelection(
                runtime,
            ) = effect
                && let Some(model) = resolve_shared_provider_selection_model(
                    self.config.model.clone(),
                    self.chat_widget.current_model(),
                    runtime.as_str(),
                )
            {
                self.app_event_tx.send(AppEvent::UpdateModelSelection {
                    model: model.clone(),
                    provider: Some(runtime.to_string()),
                });
                self.app_event_tx.send(AppEvent::PersistModelSelection {
                    model,
                    provider: Some(runtime.to_string()),
                    effort: None,
                });
            }
        }
        self.render_shared_provider_setup();
    }

    pub(super) async fn handle_event(
||||||| 413492cd6c
    pub(super) async fn handle_event(
=======
    // Keep the existing dispatcher layout until it can be formatted separately.
    #[rustfmt::skip]
    pub(crate) async fn handle_event(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        event: AppEvent,
    ) -> Result<AppRunControl> {
<<<<<<< HEAD
        let event = match event {
            AppEvent::TaskNodeScopedResult {
                label,
                generation,
                scope,
                identity,
                view,
                event,
            } => {
                if !self
                    .chat_widget
                    .tasknode_response_is_latest(label, generation)
                    || !self.chat_widget.tasknode_response_is_current(
                        &scope,
                        identity.as_deref(),
                        view.as_deref(),
                    )
                {
                    return Ok(AppRunControl::Continue);
                }
                *event
            }
            event => event,
||||||| 413492cd6c
=======
        let from_agents_overview = matches!(event, AppEvent::ForkAgentsOverviewThreadReady { .. });
        let event = if let AppEvent::ForkAgentsOverviewThreadReady { thread_id } = event {
            if self.current_displayed_thread_id() != Some(thread_id)
                || (self.thread_unavailable(thread_id)
                    && !self.chat_widget.is_external_writer_view())
            {
                self.chat_widget.fork_in_progress = false;
                return Ok(AppRunControl::Continue);
            }
            AppEvent::ForkCurrentSession { name: None }
        } else {
            event
        };
        // Release the shortcut's input guard even when a fork is rejected below.
        if matches!(event, AppEvent::ForkCurrentSession { .. }) {
            self.chat_widget.fork_in_progress = false;
        }
        if self.reconnect.offline && matches!(event, AppEvent::Exit(ExitMode::ShutdownFirst)) {
            return Ok(AppRunControl::Exit(ExitReason::UserRequested));
        }
        if self.reconnect.offline
            && !matches!(
                &event,
                AppEvent::OpenDaemonMenu
                    | AppEvent::OpenWarnings
                    | AppEvent::StatusLineSetup { .. }
                    | AppEvent::StatusLineSetupCancelled
                    | AppEvent::TerminalTitleSetup { .. }
                    | AppEvent::TerminalTitleSetupPreview { .. }
                    | AppEvent::TerminalTitleSetupCancelled
                    | AppEvent::OpenConfigEditor(_)
                    | AppEvent::SyntaxThemeSelected { .. }
                    | AppEvent::SyntaxThemePreviewed
                    | AppEvent::OpenKeymapActionMenu { .. }
                    | AppEvent::OpenKeymapReplaceBindingMenu { .. }
                    | AppEvent::OpenKeymapCapture { .. }
                    | AppEvent::OpenKeymapDebug
                    | AppEvent::KeymapCaptured { .. }
                    | AppEvent::KeymapCleared { .. }
                    | AppEvent::TestConfigNotification
                    | AppEvent::SendConfigTestNotification { .. }
                    | AppEvent::SaveConfigPreference(_)
                    | AppEvent::CopyWarning(_)
                    | AppEvent::UpdateWarnings { .. }
                    | AppEvent::CopySelection { .. }
                    | AppEvent::SelectTranscriptCopy { .. }
                    | AppEvent::TranscriptCopyClosed
                    | AppEvent::ConfirmDaemonUpdate(_)
                    | AppEvent::RunDaemonUpdate(_)
                    | AppEvent::InsertHistoryCell(_)
                    | AppEvent::CommitRealtimeTranscriptHistory
                    | AppEvent::ResetTranscriptForThreadSwitch
                    | AppEvent::ResetTranscriptForThreadSwitchPreservingScreen
                    | AppEvent::FinishPromptRevert { .. }
                    | AppEvent::ManagedWorktreeCreated(_)
                    | AppEvent::AgentsOverviewWorktreeCreated(_)
                    | AppEvent::AgentsOverviewPinToggled { .. }
                    | AppEvent::AppendMessageHistoryEntry { .. }
                    | AppEvent::BeginInitialHistoryReplayBuffer
                    | AppEvent::BeginThreadSwitchHistoryReplayBuffer
                    | AppEvent::EndInitialHistoryReplayBuffer
                    | AppEvent::FatalExitRequest(_)
            )
        {
            return Ok(AppRunControl::Continue);
        }
        if matches!(
            &event,
            AppEvent::OpenWindowsSandboxEnablePrompt { .. }
                | AppEvent::OpenWindowsSandboxFallbackPrompt { .. }
                | AppEvent::BeginWindowsSandboxElevatedSetup { .. }
                | AppEvent::BeginWindowsSandboxLegacySetup { .. }
                | AppEvent::EnableWindowsSandboxForAgentMode { .. }
        ) && !self.windows_sandbox_setup_is_local()
        {
            if matches!(
                &event,
                AppEvent::OpenWindowsSandboxFallbackPrompt { .. }
                    | AppEvent::EnableWindowsSandboxForAgentMode { .. }
            ) {
                #[cfg(any(target_os = "windows", test))]
                {
                    self.windows_sandbox.pending_setup = None;
                }
                self.chat_widget.clear_windows_sandbox_setup_status();
            }
            self.chat_widget.add_info_message(
                "Windows sandbox setup requires local connections and executors.".to_string(),
                /*hint*/ None,
            );
            return Ok(AppRunControl::Continue);
        }
        if self.chat_widget.has_misalignment_policy_violation()
            && matches!(
                event,
                AppEvent::OpenAgentPicker
                    | AppEvent::SelectAgentThread(_)
                    | AppEvent::StartSide { .. }
                    | AppEvent::ForkCurrentSession { .. }
                    | AppEvent::StartManagedWorktree {
                        mode: crate::app_event::ManagedWorktreeMode::Fork,
                        ..
                    }
                    | AppEvent::RevertSessionForPromptEdit { .. }
                    | AppEvent::SetThreadGoalDraft { .. }
                    | AppEvent::SetThreadGoalStatus {
                        status: ThreadGoalStatus::Active,
                        ..
                    }
            )
        {
            return Ok(AppRunControl::Continue);
        }

        if matches!(event, AppEvent::OpenDaemonMenu | AppEvent::OpenWarnings) {
            self.verify_managed_daemon().await;
        }

        // Keep the picker check and model update in one event without recursively polling this
        // large dispatcher on the TUI thread's stack.
        let (event, sparkle_model) = match event {
            AppEvent::AstraSelectedFromModelPicker {
                thread_id,
                model,
                action,
            } => {
                let should_offer = self.chat_widget.current_model() != model
                    && self.chat_widget.sparkle_thread_for_picker_action(&model) == Some(thread_id);
                let next_event = action.into_app_event(model.clone());
                (next_event, should_offer.then_some(model))
            }
            event => (event, None),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };
        match event {
<<<<<<< HEAD
            AppEvent::AgentControlTick { directory } => {
                self.chat_widget.handle_agent_control_tick(&directory)
            }
            AppEvent::TaskNodeScopedResult { .. } => return Ok(AppRunControl::Continue),
||||||| 413492cd6c
=======
            AppEvent::OpenDaemonMenu => self.open_daemon_menu(),
            AppEvent::ConfirmDaemonRestart => self.confirm_daemon_restart(),
            AppEvent::RunDaemonRestart => {
                if self.chat_widget.remote_connection.as_ref().is_some_and(|connection| connection.is_managed_daemon)
                    && self.daemon_restart.is_none()
                {
                    self.daemon_restart = Some(tokio::spawn(codex_app_server_daemon::run(
                        codex_app_server_daemon::LifecycleCommand::Restart {
                            feature_overrides: Default::default(),
                        },
                    )));
                }
            }
            AppEvent::ConfirmDaemonUpdate(source) => self.confirm_daemon_update(source),
            AppEvent::RunDaemonUpdate(source) => {
                self.pending_update_action = Some(UpdateAction::Daemon(source));
                return Ok(self.handle_exit_mode(app_server, ExitMode::Immediate).await);
            }
            AppEvent::UserVerificationApproved { thread_id, server_name, request_id } => {
                Box::pin(self.start_user_verification(app_server, thread_id, server_name, request_id)).await?;
            }
            AppEvent::UserVerificationFinished { thread_id, server_name, request_id, attempt_id, result } => {
                // Keep this RPC future out of the event loop's stack frame.
                Box::pin(self.finish_user_verification(app_server, thread_id, server_name, request_id, attempt_id, result)).await?;
            }
            AppEvent::ReviewMisalignment(review) => {
                self.open_misalignment_review(tui, review);
            }
            AppEvent::ContinueMisalignment(review) => {
                self.continue_misalignment(app_server, review).await;
            }
            AppEvent::CloseMisalignmentReview => self.chat_widget.show_misalignment_policy_precaution(),
            AppEvent::SkillsListLoaded { ref cwd, .. }
                if cwds_differ(cwd, self.config.cwd.as_path()) =>
            {
                self.skill_load_warnings.startup_complete = true;
            }
            AppEvent::PluginMentionsLoaded { ref cwd, .. }
                if cwds_differ(cwd, self.config.cwd.as_path()) => {}
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            AppEvent::NewSession { name } => {
                self.start_fresh_session(
                    tui, app_server, /*session_start_source*/ None,
                    /*initial_user_message*/ None, name,
                )
                .await;
                if self.chat_widget.has_misalignment_policy_violation() {
                    self.chat_widget.show_misalignment_policy_precaution();
                }
            }
            AppEvent::ContinueProviderChat { thread_id } => {
                // A queued shortcut must still refer to the locked chat on screen.
                if self.chat_widget.thread_id() == Some(thread_id)
                    && self.chat_widget.has_provider_error()
                    && let Some(path) = self.chat_widget.rollout_path()
                {
                    let name = self
                        .chat_widget
                        .thread_name()
                        .filter(|name| !name.trim().is_empty())
                        .map_or_else(
                            || "our previous conversation".to_string(),
                            |name| format!("“{name}”"),
                        );
                    let message = format!(
                        "Read {name} and let’s continue where we left off.\n\nSaved conversation: {}",
                        path.display(),
                    );
                    self.start_fresh_session(
                        tui,
                        app_server,
                        /*session_start_source*/ None,
                        /*initial_user_message*/ None,
                        /*new_thread_name*/ None,
                    )
                    .await;
                    if self
                        .chat_widget
                        .thread_id()
                        .is_some_and(|new_id| new_id != thread_id)
                    {
                        self.chat_widget
                            .restore_user_message_to_composer(message.into());
                    }
                }
            }
            AppEvent::StartManagedWorktree { mode, name } => {
                if self.pending_start_managed_worktree.is_some() {
                    self.chat_widget
                        .add_error_message("A worktree is already being created.".to_string());
                } else {
                    self.pending_start_managed_worktree = Some((mode, name));
                }
            }
            AppEvent::ManagedWorktreeCreated(created) => {
                self.pending_managed_worktree_created = Some(created);
            }
            AppEvent::BrowseManagedWorktrees => {
                if let Some(request) = self.chat_widget.request_managed_worktrees() {
                    crate::worktree_browser::fetch(
                        request,
                        self.config.codex_home.to_path_buf(),
                        app_server.request_handle(),
                        self.app_event_tx.clone(),
                    );
                }
            }
            AppEvent::ManagedWorktreesLoaded { request, result } => {
                self.chat_widget.on_managed_worktrees_loaded(request, result);
            }
            AppEvent::ManagedWorktreeAction { request, action } => {
                if let Some(event) = self.chat_widget.managed_worktree_action(&request, action) {
                    self.app_event_tx.send(event);
                }
            }
            AppEvent::ShowManagedWorktreeActions { request, entry } => {
                self.chat_widget.show_managed_worktree_actions(request, entry);
            }
            AppEvent::ConfirmManagedWorktreeRemoval { request, root } => {
                self.chat_widget.confirm_managed_worktree_removal(request, root);
            }
            AppEvent::RemoveManagedWorktree { request, root } => {
                if self.chat_widget.worktree_request_is_current(&request)
                    && !request.cwd.starts_with(&root)
                {
                    let codex_home = self.config.codex_home.to_path_buf();
                    let tx = self.app_event_tx.clone();
                    tokio::spawn(async move {
                        let result = crate::worktree_browser::remove(
                            codex_home,
                            request.cwd,
                            root.clone(),
                        )
                        .await
                        .map_err(|error| error.to_string());
                        tx.send(AppEvent::ManagedWorktreeRemoved { root, result });
                    });
                }
            }
            AppEvent::ManagedWorktreeRemoved { root, result } => match result {
                Ok(()) => self.chat_widget.add_info_message(
                    format!("Removed worktree at {}. Thread history was kept.", root.display()),
                    /*hint*/ None,
                ),
                Err(error) => self.chat_widget.add_error_message(format!(
                    "Could not remove worktree at {}: {error}",
                    root.display()
                )),
            },
            AppEvent::ChangeWorkingDirectory {
                thread_id,
                requested_cwd,
            } => {
                if self.pending_working_directory_change.is_some()
                    || self.primary_thread_id != Some(thread_id)
                    || !self.chat_widget.can_change_working_directory(thread_id)
                {
                    self.chat_widget.add_error_message(
                        "Changing directories requires an idle primary session without queued input."
                            .to_string(),
                    );
                } else if crate::uses_remote_workspace_or_environment(
                    &self.app_server_target,
                    self.environment_manager.as_ref(),
                ) {
                    self.chat_widget.add_error_message(
                        "Changing directories is not supported for remote workspaces or remote execution environments."
                            .to_string(),
                    );
                } else {
                    let cwd = AbsolutePathBuf::resolve_path_against_base(
                        requested_cwd.as_path(),
                        self.chat_widget.config_ref().cwd.as_path(),
                    );
                    match std::fs::metadata(cwd.as_path()) {
                        Ok(metadata) if metadata.is_dir() => {
                            self.pending_working_directory_change =
                                Some(working_directory::PendingWorkingDirectoryChange {
                                    source_thread_id: thread_id,
                                    source_cwd: self.config.cwd.clone(),
                                    destination: cwd,
                                });
                        }
                        Ok(_) => self
                            .chat_widget
                            .add_error_message(format!("Not a directory: {}", cwd.display())),
                        Err(error) => self.chat_widget.add_error_message(format!(
                            "Cannot access directory {}: {error}",
                            cwd.display()
                        )),
                    }
                }
            }
            AppEvent::StartupThreadStarted { result, .. } => {
                self.handle_startup_thread_started(app_server, result)
                    .await?;
            }
            AppEvent::DynamicToolThreadStarted {
                thread,
                task_tools_available,
                registered,
            } => {
                let Ok(thread_id) = ThreadId::from_string(&thread.id) else {
                    return Ok(AppRunControl::Continue);
                };
                self.agents_overview
                    .dispatched_requests
                    .entry(thread_id)
                    .or_default();
                if task_tools_available {
                    app_server.remember_task_tool_thread(thread_id);
                }
                // Fallback metadata must not replay after fresh reads or newer notifications.
                if !thread.ephemeral
                    && !self.agents_overview.removed_threads.contains(&thread_id)
                    && !self.agents_overview.threads.get(&thread_id).is_some_and(Option::is_some) {
                    self.agents_overview.threads.insert(thread_id, Some(thread));
                    self.agents_overview.refresh_thread_ids.insert(thread_id);
                }
                self.refresh_changed_agents_overview_threads(app_server);
                self.repaint_agents_overview();
                let _ = registered.send(());
            }
            AppEvent::DynamicToolCallCompleted {
                request_id,
                response,
            } => {
                self.dynamic_tool_tasks.remove(&request_id);
                match serde_json::to_value(response) {
                    Ok(result) => {
                        if let Err(error) = app_server
                            .resolve_server_request(request_id.clone(), result)
                            .await
                        {
                            tracing::warn!(?request_id, %error, "failed to resolve dynamic tool call");
                        }
                    }
                    Err(error) => {
                        tracing::warn!(?request_id, %error, "failed to serialize dynamic tool response");
                    }
                }
            }
            AppEvent::TaskToolsAvailable { thread_id } => {
                app_server.remember_task_tool_thread(thread_id);
            }
            AppEvent::RequestOlderScrollbackHistory { thread_id } => {
                if self.chat_widget.thread_id() == Some(thread_id)
                    && self.overlay.is_none()
                    && self.scrollback_has_older_history
                {
                    self.request_older_history_page(app_server, thread_id);
                }
            }
            AppEvent::OlderThreadHistoryLoaded {
                thread_id,
                cursor,
                result,
            } => {
                if let Err(err) = self
                    .handle_older_history_page(tui, app_server, thread_id, &cursor, result)
                    .await
                {
                    app_server.cancel_older_history_page(thread_id, &cursor);
                    if self.chat_widget.thread_id() == Some(thread_id) {
                        self.transcript_view.history = TranscriptHistoryState::Failed;
                        if let Some(Overlay::Transcript(overlay)) = self.overlay.as_mut() {
                            overlay.set_history_state(TranscriptHistoryState::Failed);
                        }
                        tui.frame_requester().schedule_frame();
                    }
                    tracing::warn!(%thread_id, error = %err, "failed to load older transcript history");
                }
            }
            AppEvent::OpenWarnings => self.chat_widget.open_warnings(&self.transcript_cells),
            AppEvent::UpdateWarnings { transcript, dismissed, kept } => {
                if !Arc::ptr_eq(&transcript, &self.chat_widget.warning_display_state.transcript) {
                    return Ok(AppRunControl::Continue);
                }
                let state = &mut self.chat_widget.warning_display_state.dismissed;
                state.extend(dismissed.into_iter().map(|entry| (entry.id, entry.details)));
                for entry in kept {
                    if state.get(&entry.id) == Some(&entry.details) {
                        state.remove(&entry.id);
                    }
                }
                self.chat_widget.warning_display_state.synced_cells = None;
                tui.frame_requester().schedule_frame();
            }
            AppEvent::CopyWarning(text) => {
                let result = tui.copy_transcript_selection(&text, crate::clipboard_copy::CopyFormat::PlainText);
                self.chat_widget.show_selection_copy_result(result);
            }
            AppEvent::OpenTranscriptExportFilePrompt => {
                self.chat_widget.show_transcript_export_file_prompt();
            }
            AppEvent::ExportTranscript { destination } => {
                if let Err(error) = self.export_transcript(tui, app_server, destination).await {
                    self.chat_widget
                        .add_error_message(format!("Export failed: {error}"));
                }
                if self.chat_widget.no_modal_or_popup_active() {
                    self.chat_widget
                        .set_queue_autosend_suppressed(/*suppressed*/ false);
                    self.chat_widget.maybe_send_next_queued_input();
                }
            }
            AppEvent::TranscriptCopyClosed => {
                self.chat_widget.maybe_send_next_queued_input();
            }
            AppEvent::SelectTranscriptCopy { guard } => {
                self.refresh_owned_transcript(tui)?;
                if !self.transcript_view.begin_copy_mode(&self.transcript_cells, Some(guard)) {
                    self.chat_widget.add_info_message("Nothing to copy".into(), /*hint*/ None);
                }
                tui.frame_requester().schedule_frame();
            }
            AppEvent::CopySelection { text, label, format } => {
                let result = tui.clipboard.copy(text, format, tui.frame_requester());
                self.chat_widget.show_copy_result(&label, result);
            }
            AppEvent::ClearUi { name } => {
                if self.reject_pending_permission_root_switch() {
                    return Ok(AppRunControl::Continue);
                }
                self.clear_terminal_ui(tui, /*redraw_header*/ false)?;
                self.reset_app_ui_state_after_clear();

                self.start_fresh_session(
                    tui,
                    app_server,
                    Some(ThreadStartSource::Clear),
                    /*initial_user_message*/ None,
                    name,
                )
                .await;
            }
            AppEvent::RawOutputModeChanged { enabled } => {
                self.apply_raw_output_mode(tui, enabled, /*notify*/ false);
            }
            AppEvent::ClearUiAndSubmitUserMessage { text } => {
                if self.reject_pending_permission_root_switch() {
                    self.chat_widget.restore_user_message_to_composer(text.into());
                    return Ok(AppRunControl::Continue);
                }
                self.clear_terminal_ui(tui, /*redraw_header*/ false)?;
                self.reset_app_ui_state_after_clear();

                self.start_fresh_session(
                    tui,
                    app_server,
                    Some(ThreadStartSource::Clear),
                    crate::chatwidget::create_initial_user_message(
                        Some(text),
                        Vec::new(),
                        Vec::new(),
                    ),
                    /*new_thread_name*/ None,
                )
                .await;
            }
            AppEvent::OpenResumePicker => {
<<<<<<< HEAD
                // The main loop must lend its drained input queue to modal screens.
                self.chat_widget.add_error_message(
                    "Resume could not acquire terminal input. Retry /resume.".to_string(),
                );
||||||| 413492cd6c
                let picker_app_server = match crate::start_app_server_for_picker(
                    &self.config,
                    &self.app_server_target,
                    self.state_db.clone(),
                    self.environment_manager.clone(),
                )
                .await
                {
                    Ok(app_server) => app_server,
                    Err(err) => {
                        self.chat_widget.add_error_message(format!(
                            "Failed to start TUI session picker: {err}"
                        ));
                        self.chat_widget.maybe_send_next_queued_input();
                        return Ok(AppRunControl::Continue);
                    }
                };
                match crate::resume_picker::run_resume_picker_from_existing_session_with_app_server(
                    tui,
                    &self.config,
                    /*show_all*/ false,
                    /*include_non_interactive*/ false,
                    picker_app_server,
                )
                .await?
                {
                    SessionSelection::Resume(target_session) => {
                        match self
                            .resume_target_session(tui, app_server, target_session)
                            .await?
                        {
                            AppRunControl::Continue => {}
                            AppRunControl::Exit(reason) => {
                                return Ok(AppRunControl::Exit(reason));
                            }
                        }
                    }
                    SessionSelection::Exit | SessionSelection::StartFresh => {
                        self.refresh_in_memory_config_from_disk_best_effort(
                            "closing the session picker",
                        )
                        .await;
                    }
                    SessionSelection::Fork(_) => {}
                }

                self.chat_widget.maybe_send_next_queued_input();
                // Leaving alt-screen may blank the inline viewport; force a redraw either way.
                tui.frame_requester().schedule_frame();
=======
                self.pending_open_resume_picker = true;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            AppEvent::OpenExternalAgentConfigMigration => {
<<<<<<< HEAD
                self.chat_widget.add_error_message(
                    "Import could not acquire terminal input. Retry /import.".to_string(),
                );
||||||| 413492cd6c
                match crate::external_agent_config_migration::flow::handle_external_agent_config_migration_prompt(
                    tui,
                    app_server,
                    &self.config,
                )
                .await
                {
                    Ok(ExternalAgentConfigMigrationFlowOutcome::Started(lines)) => {
                        self.chat_widget.add_plain_history_lines(lines);
                    }
                    Ok(ExternalAgentConfigMigrationFlowOutcome::NoItems) => {
                        self.chat_widget.add_info_message(
                            crate::external_agent_config_migration::flow::EXTERNAL_AGENT_CONFIG_MIGRATION_NO_ITEMS_MESSAGE
                                .to_string(),
                            /*hint*/ None,
                        );
                    }
                    Ok(ExternalAgentConfigMigrationFlowOutcome::Cancelled) => {}
                    Err(error_message) => {
                        self.chat_widget.add_error_message(error_message);
                    }
                }
                tui.frame_requester().schedule_frame();
=======
                let cwd = if self.chat_widget.thread_id().is_some()
                    || !app_server.uses_remote_workspace()
                {
                    Some(self.chat_widget.config_ref().cwd.to_path_buf())
                } else {
                    app_server.remote_cwd_override().map(Path::to_path_buf)
                };
                match crate::external_agent_config_migration::flow::handle_external_agent_config_migration_prompt(
                    tui,
                    app_server,
                    cwd.as_deref(),
                )
                .await
                {
                    Ok(ExternalAgentConfigMigrationFlowOutcome::Started(lines)) => {
                        self.chat_widget.add_plain_history_lines(lines);
                    }
                    Ok(ExternalAgentConfigMigrationFlowOutcome::NoItems) => {
                        self.chat_widget.add_info_message(
                            crate::external_agent_config_migration::flow::EXTERNAL_AGENT_CONFIG_MIGRATION_NO_ITEMS_MESSAGE
                                .to_string(),
                            /*hint*/ None,
                        );
                    }
                    Ok(ExternalAgentConfigMigrationFlowOutcome::Cancelled) => {}
                    Ok(ExternalAgentConfigMigrationFlowOutcome::TerminalError(err)) => {
                        return Err(err.into());
                    }
                    Err(error_message) => {
                        self.chat_widget.add_error_message(error_message);
                    }
                }
                tui.frame_requester().schedule_frame();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            AppEvent::ResumeSessionByIdOrName(id_or_name) => {
                match crate::lookup_session_target_with_app_server(
                    crate::resume_picker::SessionPickerAction::Resume,
                    app_server,
                    &self.config,
                    &id_or_name,
                )
                .await
                {
                    Ok(Some(target_session)) => {
                        return self
                            .resume_target_session(tui, app_server, target_session)
                            .await;
                    }
                    Ok(None) => {
                        self.chat_widget.add_error_message(format!(
                            "No saved chat found matching '{id_or_name}'."
                        ));
                    }
                    Err(err)
                        if err
                            .downcast_ref::<crate::named_session_lookup::AmbiguousSessionName>()
                            .is_some() =>
                    {
                        self.chat_widget.add_error_message(err.to_string());
                    }
                    Err(err) => return Err(err),
                }
            }
            AppEvent::ResumeBackendSession(_) => {
                unreachable!("backend selection is handled by the session manager")
            }
            AppEvent::ArchiveCurrentThread => {
                return self.archive_current_thread(tui, app_server).await;
            }
            AppEvent::DeleteCurrentThread => {
                return self.delete_current_thread(tui, app_server).await;
            }
            AppEvent::ForkCurrentSession { name } => {
                let from_locked_thread = self.chat_widget.is_external_writer_view();
                let source = if from_agents_overview {
                    "agents_overview_shortcut"
                } else if from_locked_thread {
                    "locked_thread_shortcut"
                } else {
                    "slash_command"
                };
                self.session_telemetry.counter(
                    "codex.thread.fork",
                    /*inc*/ 1,
                    &[("source", source)],
                );
                self.chat_widget
                    .add_plain_history_lines(vec!["/fork".magenta().into()]);
                if let Some(thread_id) = self.chat_widget.thread_id() {
                    if self.pending_server_profiles.contains_key(&thread_id) {
                        self.chat_widget.add_error_message(
                            "Wait for permissions to update before forking.".into(),
                        );
                        return Ok(AppRunControl::Continue);
                    }
                    self.chat_widget.fork_in_progress = true;
                    // This handler awaits the fork outside the draw loop. Paint before waiting.
                    let screen_size = tui.terminal.last_known_screen_size;
                    self.handle_draw_pre_render(tui, screen_size)?;
                    self.chat_widget.pre_draw_tick();
                    self.render_chat_widget_frame(tui, screen_size)?;
                    self.refresh_in_memory_config_from_disk_best_effort("forking the thread")
                        .await;
<<<<<<< HEAD
                    // The active thread can use a different provider, cwd, permissions, or
                    // service tier from the freshly reloaded app defaults. Fork from the
                    // widget's thread-scoped runtime snapshot so model/provider pairs remain
                    // coherent across resumed and switched threads.
                    let mut fork_config = self.chat_widget.config_ref().clone();
||||||| 413492cd6c
                    let mut fork_config = self.config.clone();
=======
                    let mut fork_config = self.config.clone();
                    fork_config.daybreak_enabled = self.chat_widget.daybreak_enabled;
                    if app_server.uses_remote_workspace() {
                        fork_config.workspace_roots.clone_from(
                            &self.chat_widget.config_ref().workspace_roots,
                        );
                    }
                    fork_config.model_provider_id.clone_from(&self.chat_widget.config_ref().model_provider_id);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    fork_config.model = Some(self.chat_widget.current_model().to_string());
<<<<<<< HEAD
                    let fork_reasoning_effort = self.chat_widget.current_reasoning_effort();
                    fork_config.model_reasoning_effort = fork_reasoning_effort.clone();
                    fork_config.service_tier = self.chat_widget.configured_service_tier();
                    match app_server.fork_thread(fork_config, thread_id).await {
||||||| 413492cd6c
                    fork_config.model_reasoning_effort =
                        self.chat_widget.current_reasoning_effort();
                    match app_server.fork_thread(fork_config, thread_id).await {
=======
                    fork_config.model_reasoning_effort =
                        self.chat_widget.current_reasoning_effort();
                    let selected_profile = self.selected_server_profile(thread_id);
                    match app_server.fork_thread_at(
                        &self.local_settings,
                        fork_config,
                        thread_id,
                        /*last_turn_id*/ None,
                        /*before_turn_id*/ None,
                        ForkGoalContinuation::StartIfIdle,
                        selected_profile.as_ref(),
                    ).await {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                        Ok(mut forked) => {
<<<<<<< HEAD
                            // Ultra is a Corbanu Terminal UI/runtime mode. The app server may project it
                            // onto the nearest provider-facing effort, but a fork must retain the
                            // user's selected mode in the attached session.
                            forked.session.reasoning_effort = fork_reasoning_effort;
||||||| 413492cd6c
=======
                            let retained_input = from_locked_thread
                                .then(|| self.chat_widget.capture_thread_input_state())
                                .flatten();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                            let name_error = if let Some(name) = name {
                                match app_server
                                    .thread_set_name(forked.session.thread_id, name.clone())
                                    .await
                                {
                                    Ok(()) => {
                                        forked.session.thread_name = Some(name);
                                        None
                                    }
                                    Err(err) => {
                                        Some(format!("Failed to name the forked session: {err}"))
                                    }
                                }
                            } else {
                                None
                            };
                            self.retain_subscriptions_for_fork();
                            match self
                                .replace_chat_widget_with_app_server_thread(
                                    tui,
                                    forked,
                                    ThreadAttachPresentation::SessionLineage,
                                    /*initial_user_message*/ None,
                                )
                                .await
                            {
                                Ok(()) => {
                                    if selected_profile.is_some() {
                                        self.adopt_inherited_server_selection();
                                    }
                                    // Keep local input without replacing the fork's running state.
                                    self.chat_widget.restore_reconnected_input(retained_input, &[]);
                                    if let Some(err) = name_error {
                                        self.chat_widget.add_error_message(err);
                                    }
                                    self.chat_widget.add_info_message(
                                        "Fork created. You can continue here.".to_string(),
                                        /*hint*/ None,
                                    );
                                }
                                Err(err) => {
                                    self.chat_widget.add_error_message(format!(
                                        "Failed to attach to forked app-server thread: {err}"
                                    ));
                                }
                            }
                        }
                        Err(err) => {
                            self.chat_widget.add_error_message(format!(
                                "Failed to fork current session through the app server: {err}"
                            ));
                        }
                    }
                    if from_locked_thread || from_agents_overview {
                        // Repeated fork shortcuts must not act on the resulting view.
                        if let Err(err) = tui.discard_pending_input_before_interactive_screen() {
                            tracing::warn!(%err, "failed to discard input after forking");
                        }
                        tui.schedule_screen_size_recheck(Duration::ZERO);
                    }
                } else {
                    self.chat_widget.add_error_message(
                        "A thread must contain at least one turn before it can be forked."
                            .to_string(),
                    );
                }

                self.chat_widget.fork_in_progress = false;
                if !from_agents_overview {
                    self.chat_widget.maybe_send_next_queued_input();
                }
                tui.frame_requester().schedule_frame();
            }
            AppEvent::RevertSessionForPromptEdit {
                thread_id,
                selected_cell,
                mut prompt,
            } => {
                if self.chat_widget.thread_id() != Some(thread_id) {
                    return Ok(AppRunControl::Continue);
                }
                if self.app_server_target.uses_remote_workspace()
                    && (!prompt.local_images.is_empty()
                        || prompt.text.trim_start().starts_with(['/', '!']))
                {
                    self.chat_widget.add_error_message(
                        "This remote prompt contains local image paths or command syntax that cannot be restored safely. Write a new message and reattach any images.".into(),
                    );
                    tui.frame_requester().schedule_frame();
                    return Ok(AppRunControl::Continue);
                }
                if self.pending_server_profiles.contains_key(&thread_id) {
                    self.chat_widget.restore_user_message_to_composer(prompt);
                    self.chat_widget.add_error_message(
                        "Wait for permissions to update before editing this prompt.".into(),
                    );
                    tui.frame_requester().schedule_frame();
                    return Ok(AppRunControl::Continue);
                }
                let Some(index) = self.transcript_cells.iter().position(|cell| Arc::ptr_eq(cell, &selected_cell)) else {
                    self.restore_backtrack_prompt_after_revert_error(prompt, "the selected prompt is no longer visible");
                    tui.frame_requester().schedule_frame();
                    return Ok(AppRunControl::Continue);
                };
                let nth_user_message = crate::app_backtrack::user_count(&self.transcript_cells[..index]);
                let selection: Result<(String, Vec<Turn>)> = async {
                    let channel = self.thread_event_channels.get(&thread_id)
                        .ok_or_else(|| color_eyre::eyre::eyre!("the selected thread is no longer available"))?;
                    let (start_item, loaded_tail, latest_turn_id) = {
                        let store = channel.store.lock().await;
                        (
                            store.turns.iter().find_map(|turn| turn.items.first()
                                .map(|item| (turn.id.clone(), item.id().to_string()))),
                            store.turns.last().map(|turn| turn.id.clone()),
                            store.latest_turn_id.clone(),
                        )
                    };
                    let mut thread = app_server.thread_read(thread_id, /*include_turns*/ false).await?;
                    app_server.hydrate_initial_thread_history(
                        &mut thread,
                        /*turn_cursor*/ None,
                        /*item_cursor*/ None,
                        /*config*/ None,
                        /*local_settings*/ None,
                        start_item.as_ref().map(|(turn_id, _)| turn_id).or(loaded_tail.as_ref()).map_or(
                            crate::app_server_session::HistoryHydrationScope::Complete,
                            |turn_id| crate::app_server_session::HistoryHydrationScope::ThroughTurn(turn_id),
                        ),
                    ).await?;
                    if thread.turns.last().map(|turn| &turn.id) != latest_turn_id.as_ref() {
                        color_eyre::eyre::bail!("thread history changed; reload the session before editing this prompt");
                    }
                    // With no retained visible items, the next prompt follows the metadata-only tail.
                    let start_item = start_item.or_else(|| loaded_tail.as_ref().and_then(|tail| {
                        thread.turns.iter().position(|turn| &turn.id == tail).and_then(|index| {
                            thread.turns[index + 1..].iter().find_map(|turn| turn.items.first()
                                .map(|item| (turn.id.clone(), item.id().to_string())))
                        })
                    }));
                    let before_turn_id = crate::app_backtrack::backtrack_revert_before_turn_id(
                        &thread.turns,
                        start_item.as_ref(),
                        nth_user_message,
                        &mut prompt,
<<<<<<< HEAD
                    ) {
                        Ok(Some(before_turn_id)) => {
                            app_server
                                .fork_thread_at(
                                    config.clone(),
                                    thread_id,
                                    /*last_turn_id*/ None,
                                    /*before_turn_id*/ Some(before_turn_id),
                                    ForkGoalContinuation::StartIfIdle,
                                )
                                .await
                        }
                        // #1038: the first message: a new thread, still a reopen.
                        Ok(None) => app_server.start_thread_for_first_prompt_edit(&config).await,
                        Err(err) => Err(err),
                    },
                    Err(err) => Err(err),
                };
                match started {
                    Ok(forked) => {
                        self.shutdown_current_thread(app_server).await;
                        match self
                            .replace_chat_widget_with_app_server_thread(
                                tui,
                                forked,
                                ThreadAttachPresentation::PromptEdit,
                                /*initial_user_message*/ None,
                            )
                            .await
                        {
                            Ok(()) => self.chat_widget.restore_user_message_to_composer(prompt),
                            Err(err) => {
                                self.restore_backtrack_prompt_after_branch_error(prompt, err);
||||||| 413492cd6c
                    ) {
                        Ok(Some(before_turn_id)) => {
                            app_server
                                .fork_thread_at(
                                    config.clone(),
                                    thread_id,
                                    /*last_turn_id*/ None,
                                    /*before_turn_id*/ Some(before_turn_id),
                                    ForkGoalContinuation::StartIfIdle,
                                )
                                .await
                        }
                        Ok(None) => {
                            app_server
                                .start_thread_with_session_start_source(
                                    &config, /*session_start_source*/ None,
                                )
                                .await
                        }
                        Err(err) => Err(err),
                    },
                    Err(err) => Err(err),
                };
                match started {
                    Ok(forked) => {
                        self.shutdown_current_thread(app_server).await;
                        match self
                            .replace_chat_widget_with_app_server_thread(
                                tui,
                                forked,
                                ThreadAttachPresentation::PromptEdit,
                                /*initial_user_message*/ None,
                            )
                            .await
                        {
                            Ok(()) => self.chat_widget.restore_user_message_to_composer(prompt),
                            Err(err) => {
                                self.restore_backtrack_prompt_after_branch_error(prompt, err);
=======
                    )?;
                    // Keep the store aligned with the displayed prefix, including retained live turns.
                    let cut = thread.turns.iter().position(|turn| turn.id == before_turn_id)
                        .ok_or_else(|| color_eyre::eyre::eyre!("selected turn disappeared"))?;
                    thread.turns.truncate(cut);
                    if let Some((turn_id, item_id)) = &start_item {
                        for turn in &mut thread.turns {
                            if &turn.id == turn_id {
                                if let Some(index) = turn.items.iter().position(|item| item.id() == item_id) {
                                    turn.items.drain(..index);
                                }
                                break;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                            }
                            turn.items.clear();
                        }
                    }
                    Ok((before_turn_id, thread.turns))
                }.await;
                let (before_turn_id, retained_turns) = match selection {
                    Ok(selection) => selection,
                    Err(err) => {
                        self.restore_backtrack_prompt_after_revert_error(prompt, err);
                        tui.frame_requester().schedule_frame();
                        return Ok(AppRunControl::Continue);
                    }
                };
                let reverted = match app_server.revert_thread(thread_id, before_turn_id, &retained_turns).await {
                    Ok(reverted) => reverted,
                    Err(err) => {
                    // Validation and unsupported-method errors leave history unchanged. Other
                    // failures can arrive after the server has already committed the revert.
                    if matches!(&err, codex_app_server_client::TypedRequestError::Server { source, .. }
                        if matches!(source.code, -32602..=-32600)) {
                        self.restore_backtrack_prompt_after_revert_error(prompt, err);
                        tui.frame_requester().schedule_frame();
                        return Ok(AppRunControl::Continue);
                    }
                    self.chat_widget.restore_user_message_to_composer(prompt);
                    return Err(color_eyre::Report::new(err).wrap_err("prompt edit could not be confirmed; resume this session to reload its history"));
                    }
                };
                self.chat_widget.restore_user_message_to_composer(prompt.clone());
                // Stop on any post-mutation failure: accepting input against the old displayed
                // transcript would hide the fact that server history has already changed.
                tokio::time::timeout(Duration::from_secs(/*secs*/ 10), async {
                    while let Some(event) = app_server.next_event().await {
                        let is_reverted = event.backend_id == app_server.backend_id() && matches!(
                            &event.event,
                            AppServerEvent::ServerNotification(notification)
                                if matches!(notification.as_ref(), ServerNotification::ThreadReverted(notification)
                                    if notification.thread_id == thread_id.to_string())
                        );
                        self.handle_backend_event(app_server, event).await;
                        if is_reverted {
                            return Ok(());
                        }
                    }
                    Err(color_eyre::eyre::eyre!("app-server disconnected"))
                }).await
                    .wrap_err("history was reverted, but refreshing the session timed out; resume this session to reload it")??;
                // Preserve the widget and unrelated threads; only replace this replay store.
                self.chat_widget.restore_thread_input_state(
                    /*input_state*/ None,
                    crate::chatwidget::ThreadInputStateRestoreMode { preserve_in_flight_turn: false },
                );
                self.chat_widget.restore_user_message_to_composer(prompt);
                self.chat_widget.set_queue_autosend_suppressed(/*suppressed*/ true);
                self.abort_thread_event_listener(thread_id);
                if let Some(mut rx) = self.active_thread_rx.take() {
                    while let Ok(event) = rx.try_recv() {
                        if let ThreadBufferedEvent::Notification(notification) = event {
                            self.chat_widget.handle_server_notification(*notification, Some(ReplayKind::ThreadSnapshot));
                        }
                    }
                }
                let mut session = self.thread_event_channels.get(&thread_id)
                    .ok_or_else(|| color_eyre::eyre::eyre!("reverted thread is no longer available"))?
                    .store.lock().await.session.clone()
                    .ok_or_else(|| color_eyre::eyre::eyre!("reverted thread has no session"))?;
                session.rollout_path = reverted.thread.path.clone();
                if self.primary_thread_id == Some(thread_id) {
                    self.primary_session_configured = Some(session.clone());
                }
                self.thread_event_channels.remove(&thread_id);
                self.active_thread_id = None;
                self.recap.reset_for_new_thread(Instant::now());
                self.recap.seed_from_turns(&retained_turns, Instant::now());
                self.retain_realtime_replay_state_before_replace();
                self.forget_realtime_replay_thread(thread_id);
                self.chat_widget.reset_after_prompt_revert(reverted.thread.path, &retained_turns);
                self.ensure_thread_channel(thread_id).store.lock().await
                    .set_session(session, retained_turns);
                self.activate_thread_channel(thread_id).await;
                // Apply the trim after any transcript inserts produced by shutdown notifications.
                // The existing reset barrier keeps terminal input blocked until then.
                self.pending_thread_switch_resets += 1;
                self.app_event_tx.send(AppEvent::FinishPromptRevert {
                    thread_id, nth_user_message,
                });
            }
            AppEvent::FinishPromptRevert { thread_id, nth_user_message } => {
                self.pending_thread_switch_resets -= 1;
                if self.chat_widget.thread_id() == Some(thread_id) {
                    if let Some(index) = crate::app_backtrack::nth_user_position(&self.transcript_cells, nth_user_message) {
                        self.transcript_cells.truncate(index);
                self.native_history.retain(&self.transcript_cells);
                    }
                    self.transcript_view = Default::default();
                    self.scrollback_has_older_history = app_server.has_older_history(thread_id);
                    self.deferred_history_lines.clear();
                    self.last_rendered_history_tail = None;
                    self.last_thread_usage_status_cell = None;
                    self.pending_thread_usage_history_refresh = false;
                    self.backtrack_render_pending = !tui.is_owned_screen();
                    self.chat_widget.set_queue_autosend_suppressed(/*suppressed*/ false);
                    self.chat_widget.emit_prompt_edit_thread_event();
                    tui.frame_requester().schedule_frame();
                }
            }
            AppEvent::BeginInitialHistoryReplayBuffer => {
                self.begin_initial_history_replay_buffer();
            }
            AppEvent::BeginThreadSwitchHistoryReplayBuffer => {
                self.begin_thread_switch_history_replay_buffer();
            }
            AppEvent::ResetTranscriptForThreadSwitch => {
                self.reset_for_thread_switch(tui)?;
                self.pending_thread_switch_resets -= 1;
            }
            AppEvent::ResetTranscriptForThreadSwitchPreservingScreen => {
                self.reset_transcript_state_after_clear();
                tui.clear_pending_history_lines();
                tui.defer_thread_switch_clear();
                self.pending_thread_switch_resets -= 1;
            }
            AppEvent::CommitRealtimeTranscriptHistory => {
                for cell in self.chat_widget.take_realtime_transcript_history() {
                    self.insert_history_cell(tui, cell);
                }
            }
            AppEvent::FollowTranscript => {
                self.transcript_view.jump_to_latest();
                tui.frame_requester().schedule_frame();
            }
            AppEvent::InsertHistoryCell(cell) => {
                self.insert_history_cell(tui, cell);
            }
            AppEvent::TurnTipReady { thread_id, turn_id } => {
                self.turn_tips.ready(thread_id, &turn_id, self.transcript_cells.last());
                tui.frame_requester().schedule_frame();
            }
            AppEvent::EndInitialHistoryReplayBuffer => {
                self.scrollback_has_older_history = self
                    .chat_widget
                    .thread_id()
                    .is_some_and(|thread_id| app_server.has_older_history(thread_id));
                if tui.is_owned_screen() {
                    self.transcript_view.history = if self.scrollback_has_older_history { TranscriptHistoryState::Partial } else { TranscriptHistoryState::Complete };
                }
                self.finish_initial_history_replay_buffer(tui);
            }
            AppEvent::ConsolidateAgentMessage {
                source,
                copy_source,
                cwd,
                inline_visualization_context,
                scrollback_reflow,
                deferred_history_cell,
            } => {
                self.handle_consolidate_agent_message(
                    tui,
                    history_cell::AgentMarkdownCell::new_with_inline_visualizations(source, &cwd, inline_visualization_context).with_copy_source(copy_source),
                    scrollback_reflow,
                    deferred_history_cell,
                )?;
                self.chat_widget.note_stream_consolidation_completed();
                self.insert_pending_usage_output_after_stream_shutdown(tui);
            }
            AppEvent::ConsolidateProposedPlan(source) => {
                let end = self.transcript_cells.len();
                let start = trailing_run_start::<history_cell::ProposedPlanStreamCell>(
                    &self.transcript_cells,
                );
                let consolidated: Arc<dyn HistoryCell> =
                    Arc::new(history_cell::new_proposed_plan(source, &self.config.cwd));

                if start < end {
                    self.native_history.consolidate(&self.transcript_cells[start..end], &consolidated);
                    if tui.is_owned_screen() {
                        self.transcript_view.replace_group(&self.transcript_cells, start..end, &consolidated);
                    } else {
                        self.transcript_view.replace_range(&self.transcript_cells, start..end, &consolidated);
                    }
                    self.transcript_cells
                        .splice(start..end, std::iter::once(consolidated.clone()));

                    if let Some(Overlay::Transcript(t)) = &mut self.overlay {
                        t.regroup_cells(start..end, consolidated.clone());
                        tui.frame_requester().schedule_frame();
                    }

                    self.finish_required_stream_reflow(tui)?;
                } else {
                    let deferred = tui.is_owned_screen() || self.native_history.insert(&consolidated);
                    self.transcript_cells.push(consolidated.clone());
                    if let Some(Overlay::Transcript(t)) = &mut self.overlay {
                        t.insert_cell(consolidated.clone());
                        tui.frame_requester().schedule_frame();
                    }
                    self.render_inserted_history_cell(tui, &consolidated, deferred);

                    self.maybe_finish_stream_reflow(tui)?;
                }
                self.chat_widget.note_stream_consolidation_completed();
                self.insert_pending_usage_output_after_stream_shutdown(tui);
            }
            AppEvent::StartCommitAnimation => {
                self.commit_animation.get_or_insert_with(|| {
                    let mut interval = tokio::time::interval_at(
                        tokio::time::Instant::now() + COMMIT_ANIMATION_TICK,
                        COMMIT_ANIMATION_TICK,
                    );
                    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                    interval
                });
            }
            AppEvent::StopCommitAnimation => {
                self.commit_animation = None;
            }
            AppEvent::Exit(mode)
                if matches!(self.app_server_target, AppServerTarget::Embedded)
                    && (self.agents_overview.threads.iter().any(|(id, thread)| {
                        (mode == ExitMode::Immediate
                            || Some(*id) != self.current_displayed_thread_id()
                            || self.chat_widget.active_view_is(AGENTS_OVERVIEW_VIEW_ID))
                            && matches!(thread.as_ref().map(|t| &t.status), Some(ThreadStatus::Active { .. }))
                    }) || self.running_side_thread_id().await.is_some()) =>
            {
                self.chat_widget.show_selection_view(SelectionViewParams {
                    title: Some("Quit and stop running sessions?".into()),
                    subtitle: Some("Quitting this TUI will stop its running sessions.".into()),
                    items: vec![
                        SelectionItem {
                            name: "Stay in Codex".into(),
                            dismiss_on_select: true,
                            ..Default::default()
                        },
                        SelectionItem {
                            name: "Stop sessions and quit".into(),
                            actions: vec![Box::new(move |tx| {
                                tx.send(AppEvent::ConfirmedEmbeddedExit(mode))
                            })],
                            dismiss_on_select: true,
                            ..Default::default()
                        },
                    ],
                    ..SelectionViewParams::confirmation()
                });
            }
            AppEvent::Exit(mode) | AppEvent::ConfirmedEmbeddedExit(mode) => {
                if matches!(mode, ExitMode::ShutdownFirst | ExitMode::ShutdownAfterInterrupt) {
                    self.show_shutdown_feedback(tui)?;
                }
                return Ok(self.handle_exit_mode(app_server, mode).await);
            }
            AppEvent::RunningTaskExit { action, thread_id } => match action {
                RunningTaskExitAction::RunInBackground => {
                    return Ok(self.handle_exit_mode(app_server, ExitMode::Immediate).await);
                }
                RunningTaskExitAction::CancelTask => {
                    if self.chat_widget.thread_id() == Some(thread_id)
                        && self.chat_widget.is_agent_turn_running()
                        && self.chat_widget.submit_op(AppCommand::interrupt())
                    {
                        self.chat_widget.pause_active_goal_for_interrupt();
                    } else if self.side_threads.contains_key(&thread_id)
                        && let Err(error) = self
                            .try_submit_active_thread_op_via_app_server(
                                app_server,
                                thread_id,
                                &AppCommand::interrupt(),
                            )
                            .await
                    {
                        self.chat_widget
                            .add_error_message(format!("Failed to interrupt task: {error}"));
                    }
                }
                RunningTaskExitAction::Exit => {
                    if self.chat_widget.thread_id() == Some(thread_id)
                        && self.chat_widget.is_active_goal_turn_running()
                        && let Err(error) = app_server
                            .thread_goal_set(
                                thread_id,
                                /*objective*/ None,
                                Some(codex_app_server_protocol::ThreadGoalStatus::Paused),
                                /*token_budget*/ None,
                            )
                            .await
                    {
                        self.chat_widget
                            .add_error_message(format!("Failed to pause task goal: {error}"));
                        return Ok(AppRunControl::Continue);
                    }
                    let turn_id = self
                        .active_turn_id_for_thread(thread_id)
                        .await
                        .unwrap_or_default();
                    match app_server.turn_interrupt(thread_id, turn_id).await {
                        Ok(()) => {
                            self.app_event_tx
                                .send(AppEvent::Exit(ExitMode::ShutdownAfterInterrupt));
                        }
                        Err(error) => {
                            self.chat_widget
                                .add_error_message(format!("Failed to interrupt task: {error}"));
                        }
                    }
                }
            },
            AppEvent::Logout => match app_server.logout_account().await {
                Ok(()) => {
                    self.show_shutdown_feedback(tui)?;
                    return Ok(self
                        .handle_exit_mode(app_server, ExitMode::ShutdownFirst)
                        .await);
                }
                Err(err) => {
                    tracing::error!("failed to logout: {err}");
                    self.chat_widget
                        .add_error_message(format!("Logout failed: {err}"));
                }
            },
            AppEvent::RestartForSecurityLevel => {
                // #436: never restart into a session without its provider key.
                if let Some(blocked) = crate::security::restart::restart_blocker() {
                    self.chat_widget.add_error_message(blocked);
                    return Ok(AppRunControl::Continue);
                }
                self.show_shutdown_feedback(tui)?;
                let _ = self
                    .handle_exit_mode(app_server, ExitMode::ShutdownFirst)
                    .await;
                return Ok(AppRunControl::Exit(ExitReason::Restart));
            }
            AppEvent::KillSwitchTurnedOn => {
                self.on_kill_switch_changed(/*on*/ true, /*elsewhere*/ false);
            }
            AppEvent::KillSwitchTurnedOff => {
                self.on_kill_switch_changed(/*on*/ false, /*elsewhere*/ false);
            }
            AppEvent::KillSwitchChangedElsewhere { on } => {
                self.on_kill_switch_changed(on, /*elsewhere*/ true);
            }
            AppEvent::SecurityLevelRaisedElsewhere { level } => {
                self.on_level_raised_elsewhere(level);
            }
            AppEvent::FatalExitRequest(message) => {
                return Ok(AppRunControl::Exit(ExitReason::Fatal(message)));
            }
            AppEvent::ImagesPrepared(id) => {
                self.chat_widget.on_images_prepared(id);
            }
            AppEvent::CodexOp(mut op) => {
                if let AppCommand::OverrideTurnContext {
                    cwd,
                    approval_policy,
                    approvals_reviewer,
                    permission_profile,
                    active_permission_profile,
                    ..
                } = &op
                    && (cwd.is_some()
                        || approval_policy.is_some()
                        || approvals_reviewer.is_some()
                        || permission_profile.is_some()
                        || active_permission_profile.is_some())
                    && self.reject_pending_permission_change()
                {
                    return Ok(AppRunControl::Continue);
                }
                if self.active_thread_id == self.chat_widget.thread_id() {
                    if matches!(&op, AppCommand::UserTurn { .. })
                        && self.chat_widget.defer_pending_turn_for_luna_reserve()
                    {
                        return Ok(AppRunControl::Continue);
                    }
                    self.chat_widget
                        .apply_reserve_fallback_to_pending_turn(&mut op);
                }
                let is_user_turn = matches!(&op, AppCommand::UserTurn { .. });
                let is_realtime_stop = matches!(&op, AppCommand::RealtimeConversationStop { .. });
                let realtime_stop_thread_id = match &op {
                    AppCommand::RealtimeConversationStop { thread_id } => Some(*thread_id),
                    _ => None,
                };
                let realtime_speech_delivery_id = match &op {
                    AppCommand::RealtimeConversationSpeech { delivery_id, .. } => {
                        Some(*delivery_id)
                    }
                    _ => None,
                };
                let is_realtime_conversation = matches!(
                    &op,
                    AppCommand::RealtimeConversationStart { .. }
                        | AppCommand::RealtimeConversationStop { .. }
                        | AppCommand::RealtimeConversationSpeech { .. }
                );
                if is_user_turn {
<<<<<<< HEAD
                    if let Some(thread_id) = self.active_thread_id {
                        self.note_assignment_user_turn(
                            &crate::spawn_orchestration::thread_node_id(thread_id),
                        );
                    }
                    self.handle_draw_pre_render(tui)?;
||||||| 413492cd6c
                    self.handle_draw_pre_render(tui)?;
=======
                    let screen_size = tui.terminal.last_known_screen_size;
                    self.handle_draw_pre_render(tui, screen_size)?;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    if self.transcript_reflow.has_pending_reflow() {
                        self.transcript_reflow.schedule_immediate();
                        self.maybe_run_resize_reflow(tui, screen_size)?;
                    }
                    self.chat_widget.pre_draw_tick();
                    self.render_chat_widget_frame(tui, screen_size)?;
                }
                let parked_voice = match &op {
                    AppCommand::RealtimeConversationStart { thread_id, .. }
                    | AppCommand::RealtimeConversationStop { thread_id }
                    | AppCommand::RealtimeConversationSpeech { thread_id, .. } => self
                        .background_voice
                        .as_ref()
                        .is_some_and(|owner| owner.thread_id() == Some(*thread_id)),
                    _ => false,
                };
                let visible_thread = self.active_thread_id;
                if parked_voice
                    && let Some(owner) = self.background_voice.as_mut()
                {
                    std::mem::swap(&mut self.chat_widget, owner);
                    self.active_thread_id = self.chat_widget.thread_id();
                }
                self.chat_widget.prepare_local_op_submission(&op);
                let result = self.submit_active_thread_op(app_server, op).await;
                if result.is_err()
                    && let Some(delivery_id) = realtime_speech_delivery_id
                {
                    self.chat_widget.restore_undelivered_realtime_speech(delivery_id);
                }
                if parked_voice
                    && let Some(owner) = self.background_voice.as_mut()
                {
                    std::mem::swap(&mut self.chat_widget, owner);
                    self.active_thread_id = visible_thread;
                }
                if let Err(err) = result {
                    if is_realtime_stop {
                        let chat_widget = match self.background_voice.as_deref_mut() {
                            Some(owner) if parked_voice => owner,
                            _ => &mut self.chat_widget,
                        };
                        if chat_widget.thread_id() == realtime_stop_thread_id {
                            chat_widget.record_realtime_failure(
                                crate::chatwidget::RealtimeFailureCause::AppServerRequest,
                            );
                        }
                    }
                    if self.recover_transport_error(&err) {
                        return Ok(AppRunControl::Continue);
                    }
                    let chat_widget = match self.background_voice.as_deref_mut() {
                        Some(owner) if parked_voice => owner,
                        _ => &mut self.chat_widget,
                    };
                    let unsupported_permissions = err
                        .downcast_ref::<UnsupportedLegacyPermissionProfile>()
                        .is_some();
                    if unsupported_permissions {
                        chat_widget
                            .set_queue_autosend_suppressed(/*suppressed*/ true);
                    }
                    let handled = is_user_turn
                        && (matches!(
                            err.downcast_ref::<TypedRequestError>(),
                            Some(TypedRequestError::Server { method, .. })
                                if method == "turn/start"
                        ) || unsupported_permissions)
                        && chat_widget
                            .handle_turn_start_rejection(format!("Failed to start turn: {err:#}"));
                    if is_realtime_conversation {
                        let message = format!("Voice conversation failed: {err:#}");
                        if is_realtime_stop {
                            if chat_widget.thread_id() == realtime_stop_thread_id {
                                chat_widget.reset_realtime_conversation();
                                chat_widget.add_realtime_error(message);
                            }
                        } else {
                            chat_widget.on_realtime_error(
                                message,
                                crate::chatwidget::RealtimeFailureCause::AppServerRequest,
                            );
                        }
                        tracing::error!(error = ?err, "realtime conversation request failed");
                    } else if handled {
                        tracing::error!(error = ?err, "failed to start turn through app server");
                    } else {
                        return Err(err);
                    }
                }
            }
            AppEvent::ConfirmSafetyBufferedRetry {
                thread_id,
                turn_id,
                model,
                turn,
                prompt,
            } => {
                self.chat_widget
                    .confirm_safety_buffered_retry(thread_id, turn_id, model, turn, prompt);
            }
            AppEvent::RetrySafetyBufferedTurn {
                thread_id,
                turn_id,
                model,
                turn,
                prompt,
            } => {
                self.retry_safety_buffered_turn(
                    tui,
                    app_server,
                    super::safety_buffering::SafetyBufferedRetry {
                        thread_id,
                        turn_id,
                        model,
                        turn,
                        prompt,
                    },
                )
                .await;
            }
            AppEvent::ContinueSafetyRecourse {
                thread_id,
                turn_id,
                prompt,
            } => {
                super::safety_recourse::continue_with_rewrite(
                    self, tui, app_server, thread_id, turn_id, prompt,
                )
                .await;
            }
            AppEvent::AppendMessageHistoryEntry { thread_id, text } => {
                self.append_message_history_entry(thread_id, text);
            }
            AppEvent::SyncThreadGitBranch {
                thread_id,
                branch,
                cwd: _cwd,
            } => {
                if let Err(err) = app_server
                    .thread_metadata_update_branch(thread_id, branch)
                    .await
                {
                    tracing::warn!("failed to sync thread git branch from directive: {err}");
                }
            }
            AppEvent::LookupMessageHistoryEntry {
                thread_id,
                offset,
                log_id,
            } => {
                self.lookup_message_history_entry(thread_id, offset, log_id)
                    .await?;
            }
            AppEvent::LookupMessageHistoryBatch {
                thread_id,
                cursor,
                log_id,
            } => {
                self.lookup_message_history_batch(thread_id, cursor, log_id)
                    .await?;
            }
            AppEvent::ApproveRecentAutoReviewDenial { thread_id, id } => {
                self.chat_widget
                    .approve_recent_auto_review_denial(thread_id, id);
            }
            AppEvent::SubmitThreadOp { thread_id, op } => {
                self.submit_thread_op(app_server, thread_id, op).await?;
            }
            AppEvent::ThreadHistoryEntryResponse { thread_id, event } => {
                self.enqueue_thread_history_entry_response(thread_id, event)
                    .await?;
            }
            AppEvent::DiffResult(cwd, text) => {
                if cwds_differ(&cwd, self.chat_widget.config_ref().cwd.as_path()) {
                    return Ok(AppRunControl::Continue);
                }
                // Clear the in-progress state in the bottom pane
                self.chat_widget.on_diff_complete();
                // Enter alternate screen using TUI helper and build pager lines
                let _ = tui.enter_alt_screen();
                let pager_lines: Vec<ratatui::text::Line<'static>> = if text.trim().is_empty() {
                    vec!["No changes detected.".italic().into()]
                } else {
                    text.lines().map(ansi_escape_line).collect()
                };
                self.overlay = Some(Overlay::new_static_with_lines(
                    pager_lines,
                    "D I F F".to_string(),
                    self.keymap.pager.clone(),
                ));
                tui.frame_requester().schedule_frame();
            }
            AppEvent::MkDocsResult(result) => match result {
                Ok(site) => {
                    let _ = tui.enter_alt_screen();
                    self.overlay = Some(Overlay::new_mkdocs(
                        site,
                        self.keymap.pager.clone(),
                        self.keymap.list.clone(),
                    ));
                    tui.frame_requester().schedule_frame();
                }
                Err(error) => {
                    self.chat_widget.add_error_message(error);
                    tui.frame_requester().schedule_frame();
                }
            },
            AppEvent::OpenAppLink {
                app_id,
                title,
                description,
                instructions,
                url,
                is_installed,
                is_enabled,
            } => {
                self.chat_widget
                    .open_app_link_view(crate::bottom_pane::AppLinkViewParams {
                        app_id,
                        title,
                        description,
                        instructions,
                        url,
                        is_installed,
                        is_enabled,
                        suggest_reason: None,
                        suggestion_type: None,
                        elicitation_target: None,
                    });
            }
            AppEvent::AccountEmailLoaded { request_id, email, workspace } => {
                if self.account_email_request_id == Some(request_id) {
                    self.account_email_request_id = None;
                    self.chat_widget.on_account_email_loaded(email, workspace);
                }
            }
            AppEvent::SecuritySetupLoaded { request_id, identity, notice } => {
                tracing::debug!(current = request_id == self.chat_widget.security_setup_request_id, "handling security setup notice");
                if request_id == self.chat_widget.security_setup_request_id {
                    self.chat_widget.show_security_setup(identity, notice);
                }
            }
            AppEvent::OpenUrlInBrowser { url } => {
                self.open_url_in_browser(url);
            }
            AppEvent::OpenDesktopThread { thread_id } => {
                self.open_desktop_thread(thread_id);
            }
            AppEvent::PetSelected { pet_id } => {
                self.handle_pet_selected(tui, pet_id);
            }
            AppEvent::PetDisabled => {
                self.handle_pet_disabled(tui).await;
            }
            AppEvent::PetPreviewRequested { pet_id } => {
                self.chat_widget.start_pet_picker_preview(pet_id);
            }
            AppEvent::PetPreviewLoaded { request_id, result } => {
                self.handle_pet_preview_loaded(tui, request_id, result);
            }
            AppEvent::PetSelectionLoaded {
                request_id,
                pet_id,
                result,
            } => {
                return self
                    .handle_pet_selection_loaded(tui, request_id, pet_id, result)
                    .await;
            }
            AppEvent::ConfiguredPetLoaded {
                generation,
                pet_id,
                result,
            } => {
                self.handle_configured_pet_loaded(tui, generation, pet_id, result);
            }
            AppEvent::RefreshConnectors { force_refetch } => {
                self.chat_widget.refresh_connectors(force_refetch);
            }
            AppEvent::FetchConnectorsList {
                force_refetch,
                generation,
            } => {
                if generation == self.chat_widget.connector_scope_generation() {
                    self.fetch_connectors_list(app_server, force_refetch);
                }
            }
            AppEvent::FetchInstalledConnectorMentions {
                force_refresh,
                generation,
            } => {
                if generation == self.chat_widget.connector_scope_generation() {
                    self.fetch_installed_connector_mentions(app_server, force_refresh, generation);
                }
            }
            AppEvent::PluginInstallAuthAdvance { refresh_connectors } => {
                if refresh_connectors {
                    self.chat_widget.refresh_connectors(/*force_refetch*/ true);
                }
                self.chat_widget.advance_plugin_install_auth_flow();
            }
            AppEvent::PluginInstallAuthAbandon => {
                self.chat_widget.abandon_plugin_install_auth_flow();
            }
            AppEvent::FetchPluginsList { cwd } => {
                self.fetch_plugins_list(app_server, cwd);
            }
            AppEvent::FetchHooksList { target } => {
                self.fetch_hooks_list(app_server, target);
            }
            AppEvent::OpenMarketplaceAddPrompt => {
                self.chat_widget.open_marketplace_add_prompt();
            }
            AppEvent::OpenMarketplaceAddLoading { source } => {
                self.chat_widget.open_marketplace_add_loading_popup(&source);
            }
            AppEvent::OpenMarketplaceRemoveConfirm {
                marketplace_name,
                marketplace_display_name,
            } => {
                self.chat_widget.open_marketplace_remove_confirmation(
                    marketplace_name,
                    marketplace_display_name,
                );
            }
            AppEvent::OpenMarketplaceRemoveLoading {
                marketplace_display_name,
            } => {
                self.chat_widget
                    .open_marketplace_remove_loading_popup(&marketplace_display_name);
            }
            AppEvent::OpenMarketplaceUpgradeLoading { marketplace_name } => {
                self.chat_widget
                    .open_marketplace_upgrade_loading_popup(marketplace_name.as_deref());
            }
            AppEvent::OpenPluginDetailLoading {
                plugin_display_name,
            } => {
                self.chat_widget
                    .open_plugin_detail_loading_popup(&plugin_display_name);
            }
            AppEvent::OpenPluginInstallLoading {
                plugin_display_name,
            } => {
                self.chat_widget
                    .open_plugin_install_loading_popup(&plugin_display_name);
            }
            AppEvent::OpenPluginUninstallLoading {
                plugin_display_name,
            } => {
                self.chat_widget
                    .open_plugin_uninstall_loading_popup(&plugin_display_name);
            }
            AppEvent::PluginsLoaded { cwd, result } => {
                self.chat_widget.on_plugins_loaded(cwd, result);
            }
            AppEvent::OpenPluginsList { cwd, response } => {
                self.chat_widget.open_plugins_list(cwd, response);
            }
            AppEvent::PluginRemoteSectionsLoaded {
                cwd,
                marketplaces,
                section_errors,
            } => {
                self.chat_widget.on_plugin_remote_sections_loaded(
                    cwd,
                    marketplaces,
                    section_errors,
                );
            }
            AppEvent::HooksLoaded { target, result } => {
                self.chat_widget.on_hooks_loaded(target, result);
            }
            AppEvent::FetchMarketplaceAdd { cwd, source } => {
                self.fetch_marketplace_add(app_server, cwd, source);
            }
            AppEvent::FetchMarketplaceUpgrade {
                cwd,
                marketplace_name,
            } => {
                self.fetch_marketplace_upgrade(app_server, cwd, marketplace_name);
            }
            AppEvent::MarketplaceAddLoaded {
                cwd,
                source,
                result,
            } => {
                let add_succeeded = result.is_ok();
                self.chat_widget
                    .on_marketplace_add_loaded(cwd.clone(), source, result);
                if add_succeeded && self.chat_widget.config_ref().cwd.as_path() == cwd.as_path() {
                    self.fetch_plugins_list(app_server, cwd);
                }
            }
            AppEvent::MarketplaceUpgradeLoaded { cwd, result } => {
                let marketplace_contents_changed =
                    matches!(&result, Ok(response) if !response.upgraded_roots.is_empty());
                if marketplace_contents_changed {
                    self.refresh_plugin_mentions_after_config_write();
                }
                self.chat_widget
                    .on_marketplace_upgrade_loaded(cwd.clone(), result);
                if self.chat_widget.config_ref().cwd.as_path() == cwd.as_path() {
                    self.fetch_plugins_list(app_server, cwd);
                }
            }
            AppEvent::FetchMarketplaceRemove {
                cwd,
                marketplace_name,
                marketplace_display_name,
            } => {
                self.fetch_marketplace_remove(
                    app_server,
                    cwd,
                    marketplace_name,
                    marketplace_display_name,
                );
            }
            AppEvent::MarketplaceRemoveLoaded {
                cwd,
                marketplace_name,
                marketplace_display_name,
                result,
            } => {
                let remove_succeeded = result.is_ok();
                self.chat_widget.on_marketplace_remove_loaded(
                    cwd.clone(),
                    marketplace_name,
                    marketplace_display_name,
                    result,
                );
                if remove_succeeded && self.chat_widget.config_ref().cwd.as_path() == cwd.as_path()
                {
                    self.refresh_plugin_mentions_after_config_write();
                    self.fetch_plugins_list(app_server, cwd);
                }
            }
            AppEvent::FetchPluginDetail { cwd, params } => {
                self.fetch_plugin_detail(app_server, cwd, params);
            }
            AppEvent::PluginDetailLoaded { cwd, result } => {
                self.chat_widget.on_plugin_detail_loaded(cwd, result);
            }
            AppEvent::FetchPluginInstall {
                cwd,
                location,
                plugin_name,
                plugin_display_name,
            } => {
                self.fetch_plugin_install(
                    app_server,
                    cwd,
                    location,
                    plugin_name,
                    plugin_display_name,
                );
            }
            AppEvent::FetchPluginUninstall {
                cwd,
                plugin_id,
                plugin_display_name,
            } => {
                self.fetch_plugin_uninstall(app_server, cwd, plugin_id, plugin_display_name);
            }
            AppEvent::SetPluginEnabled {
                cwd,
                plugin_id,
                enabled,
            } => {
                self.set_plugin_enabled(app_server, cwd, plugin_id, enabled);
            }
            AppEvent::PluginInstallLoaded {
                cwd,
                location,
                plugin_name,
                plugin_display_name,
                result,
            } => {
                let install_succeeded = result.is_ok();
                if install_succeeded {
                    self.refresh_plugin_mentions_after_config_write();
                }
                let should_refresh_plugin_detail = self.chat_widget.on_plugin_install_loaded(
                    cwd.clone(),
                    location.clone(),
                    plugin_name.clone(),
                    plugin_display_name,
                    result,
                );
                if install_succeeded && self.chat_widget.config_ref().cwd.as_path() == cwd.as_path()
                {
                    self.fetch_plugins_list(app_server, cwd.clone());
                    if should_refresh_plugin_detail {
                        let (marketplace_path, remote_marketplace_name) =
                            location.into_request_params();
                        self.fetch_plugin_detail(
                            app_server,
                            cwd,
                            PluginReadParams {
                                marketplace_path,
                                remote_marketplace_name,
                                plugin_name,
                            },
                        );
                    }
                }
            }
            AppEvent::PluginEnabledSet {
                cwd,
                plugin_id,
                enabled,
                result,
            } => {
                let queued_enabled = self
                    .pending_plugin_enabled_writes
                    .get_mut(&plugin_id)
                    .and_then(Option::take);
                let should_apply_result = if let Some(queued_enabled) = queued_enabled
                    && (result.is_err() || queued_enabled != enabled)
                {
                    self.spawn_plugin_enabled_write(
                        app_server,
                        cwd.clone(),
                        plugin_id.clone(),
                        queued_enabled,
                    );
                    false
                } else {
                    true
                };
                if should_apply_result {
                    self.pending_plugin_enabled_writes.remove(&plugin_id);
                    let update_succeeded = result.is_ok();
                    if update_succeeded {
                        self.refresh_plugin_mentions_after_config_write();
                    }
                    self.chat_widget
                        .on_plugin_enabled_set(cwd, plugin_id, enabled, result);
                }
            }
            AppEvent::FetchMcpInventory { detail, thread_id } => {
                self.fetch_mcp_inventory(app_server, detail, thread_id);
            }
            AppEvent::StartMcpLogin { name, thread_id } => {
                if self.pending_mcp_login_start.is_some() {
                    self.chat_widget.add_info_message(
                        "MCP sign-in is starting. Wait for it to finish before trying again.".to_string(),
                        /*hint*/ None,
                    );
                    return Ok(AppRunControl::Continue);
                }
                let request_id = format!("mcp-login-{}", uuid::Uuid::new_v4());
                self.pending_mcp_login_start = Some(PendingMcpLoginStart {
                    request_id: request_id.clone(),
                    name: name.clone(),
                    thread_id,
                    completions: Vec::new(),
                });
                self.start_mcp_login(app_server, request_id, name, thread_id);
            }
            AppEvent::McpLoginStarted { request_id, result } => {
                if let Some(pending) = self
                    .pending_mcp_login_start
                    .take_if(|pending| pending.request_id == request_id)
                {
                    match result {
                        Ok(response) => {
                            // Track the latest attempt per server: unrelated OAuth logins can
                            // remain open while a user retries this server.
                            if let Some(login_id) = response.login_id.as_ref() {
                                self.active_mcp_login_ids
                                    .insert(pending.name.clone(), login_id.clone());
                            }
                            let completed = pending.completions.iter().any(|completion| {
                                completion.login_id == response.login_id
                                    && completion.name == pending.name
                            });
                            if !completed {
                                self.open_url_in_browser(response.authorization_url);
                            }
                        }
                        Err(error) => {
                            self.enqueue_thread_notification(
                                pending.thread_id,
                                ServerNotification::McpServerOauthLoginCompleted(
                                    codex_app_server_protocol::McpServerOauthLoginCompletedNotification {
                                        name: pending.name,
                                        thread_id: Some(pending.thread_id.to_string()),
                                        login_id: None,
                                        success: false,
                                        error: Some(error),
                                    },
                                ),
                            ).await?;
                        }
                    }
                    // A rejected start leaves the old attempt current. A successful replacement
                    // has changed its ID, so the old cancellation is discarded here.
                    for completion in pending.completions {
                        if completion.login_id.is_some() {
                            if completion.login_id.as_ref()
                                != self.active_mcp_login_ids.get(&completion.name)
                            {
                                continue;
                            }
                            self.active_mcp_login_ids.remove(&completion.name);
                        }
                        if let Some(thread_id) = completion.thread_id.as_deref()
                            .and_then(|id| ThreadId::from_string(id).ok())
                        {
                            self.enqueue_thread_notification(
                                thread_id,
                                ServerNotification::McpServerOauthLoginCompleted(completion),
                            ).await?;
                        }
                    }
                }
            }
            AppEvent::McpInventoryLoaded {
                result,
                detail,
                thread_id,
            } => {
                self.handle_mcp_inventory_result(result, detail, thread_id);
            }
            AppEvent::SkillsListLoaded { result, .. } => {
                self.handle_skills_list_result(
                    result.map_err(|err| color_eyre::eyre::eyre!(err)),
                    "failed to load skills on startup",
                );
                self.skill_load_warnings.startup_complete = true;
            }
            AppEvent::StartFileSearch(query) => {
                self.file_search.on_user_query(query.clone());
                if let Some(thread_id) = self.active_thread_id
                    && app_server.task_tools_available(thread_id)
                    && self.config.features.enabled(Feature::MentionsV2)
                {
                    let cwd = self
                        .thread_cwd(thread_id)
                        .await
                        .map(|cwd| cwd.to_path_buf())
                        .or_else(|| {
                            app_server
                                .remote_cwd_override()
                                .map(std::path::Path::to_path_buf)
                        })
                        .unwrap_or_else(|| self.config.cwd.to_path_buf());
                    crate::task_mentions::spawn_search(
                        app_server.request_handle(),
                        query,
                        thread_id,
                        cwd,
                        app_server.task_search_generation(),
                        self.app_event_tx.clone(),
                    );
                }
            }
            AppEvent::FileSearchResult { query, matches } => {
                self.chat_widget.apply_file_search_result(query, matches);
            }
            AppEvent::TaskSearchResult {
                thread_id,
                query,
                matches,
            } => {
                if self.active_thread_id == Some(thread_id)
                    && app_server.task_tools_available(thread_id)
                {
                    self.chat_widget.on_task_search_result(&query, matches);
                }
            }
            AppEvent::RefreshRateLimits { origin } => {
                self.refresh_rate_limits(app_server, origin);
            }
<<<<<<< HEAD
            event @ (AppEvent::OpenAccountingInspector { .. }
            | AppEvent::LoadAccountingInspector { .. }
            | AppEvent::AccountingInspectorLoaded { .. }
            | AppEvent::NavigateAccountingInspector { .. }
            | AppEvent::CloseAccountingInspector { .. }
            | AppEvent::RefreshAccountingInspector { .. }) => self.handle_accounting_inspector_event(event),
            AppEvent::RefreshTokenActivity { request_id } => {
                self.refresh_token_activity(app_server, request_id);
||||||| 413492cd6c
            AppEvent::RefreshTokenActivity { request_id } => {
                self.refresh_token_activity(app_server, request_id);
=======
            AppEvent::ApplyBackendBannerFallback { thread_id } => {
                if self.active_thread_id == Some(thread_id)
                    && self.chat_widget.thread_id() == Some(thread_id)
                {
                    self.apply_backend_banner_fallback(app_server).await;
                    if !self.rate_limit_refresh_state.has_pending_recovery() {
                        self.chat_widget.finish_rate_limit_recovery();
                    }
                    self.refresh_rate_limits(app_server, RateLimitRefreshOrigin::Periodic);
                }
            }
            AppEvent::RefreshThreadUsage {
                thread_id,
                request_id,
            } => {
                self.refresh_thread_usage(app_server, thread_id, request_id);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            AppEvent::RefreshStatusLineWorkspaceHeadline { request_id } => {
                self.refresh_status_line_workspace_headline(app_server, request_id);
            }
            AppEvent::OpenThreadGoalMenu { thread_id } => {
                self.open_thread_goal_menu(app_server, thread_id).await;
            }
            AppEvent::OpenThreadGoalEditor { thread_id } => {
                self.open_thread_goal_editor(app_server, thread_id).await;
            }
            AppEvent::SetThreadGoalDraft {
                thread_id,
                draft,
                mode,
            } => {
                self.set_thread_goal_draft(app_server, thread_id, draft, mode)
                    .await;
            }
            AppEvent::SetThreadGoalStatus { thread_id, status } => {
                self.set_thread_goal_status(app_server, thread_id, status)
                    .await;
            }
            AppEvent::ClearThreadGoal { thread_id } => {
                self.clear_thread_goal(app_server, thread_id).await;
            }
            AppEvent::SendAddCreditsNudgeEmail { credit_type } => {
                if let Some(request_id) = self
                    .chat_widget
                    .start_add_credits_nudge_email_request(credit_type)
                {
                    self.send_add_credits_nudge_email(app_server, request_id, credit_type);
                }
            }
            AppEvent::AddCreditsNudgeEmailFinished { request_id, result } => {
                self.chat_widget
                    .finish_add_credits_nudge_email_request(request_id, result);
            }
            AppEvent::RateLimitsLoaded {
                request_id,
                origin,
                hard_stop_generation,
                result,
            } => {
                let accepted = match self.rate_limit_refresh_state.finish(
                    request_id,
                    hard_stop_generation,
                    self.rate_limit_hard_stop_generation,
                    if result.is_ok() {
                        RateLimitReadStatus::Succeeded
                    } else {
                        RateLimitReadStatus::Failed
                    },
                ) {
                    RateLimitRefreshOutcome::Apply => true,
                    RateLimitRefreshOutcome::Ignore => false,
                    RateLimitRefreshOutcome::RefreshRecovery => {
                        // Start in this account's event turn; a queued refresh could cross an account change.
                        self.refresh_rate_limits(app_server, RateLimitRefreshOrigin::Recovery);
                        false
                    }
                };
                match result {
                Ok(response) => {
                    let rate_limit_reset_credits = response.rate_limit_reset_credits.clone();
                    let snapshots = if accepted
                    {
                        self.chat_widget.apply_usage_notice_read(request_id);
                        self.chat_widget.update_backend_banner(&response);
                        self.apply_backend_banner_fallback(app_server).await;
                        app_server_rate_limit_snapshots(response)
                    } else {
                        Vec::new()
                    };
                    match origin {
                        RateLimitRefreshOrigin::Recovery | RateLimitRefreshOrigin::Periodic => {
                            for snapshot in snapshots {
                                self.chat_widget.on_rate_limit_snapshot(Some(snapshot));
                            }
                        }
                        RateLimitRefreshOrigin::StartupPrefetch {
                            reset_hint_request_id,
                        } => {
                            if self.chat_widget.finish_rate_limit_reset_hint_refresh(
                                reset_hint_request_id,
                                snapshots,
                                rate_limit_reset_credits.ok_or_else(|| {
                                    "account/rateLimits/read response did not include rateLimitResetCredits"
                                        .to_string()
                                }),
                            ) {
                                self.insert_pending_usage_output_if_ready(tui);
                            }
                            tui.frame_requester().schedule_frame();
                        }
                        RateLimitRefreshOrigin::ResetConsume { request_id } => {
                            self.chat_widget.finish_post_consume_reset_credits_refresh(
                                request_id,
                                snapshots,
                                rate_limit_reset_credits.ok_or_else(|| {
                                    "account/rateLimits/read response did not include rateLimitResetCredits"
                                        .to_string()
                                }),
                            );
                            tui.frame_requester().schedule_frame();
                        }
                        RateLimitRefreshOrigin::StatusCommand { request_id } => {
                            self.chat_widget
                                .finish_status_rate_limit_refresh(request_id, snapshots);
                        }
                        RateLimitRefreshOrigin::UsageMenu { request_id } => {
                            self.chat_widget.finish_usage_menu_rate_limit_refresh(
                                request_id,
                                snapshots,
                                rate_limit_reset_credits.ok_or_else(|| {
                                    "account/rateLimits/read response did not include rateLimitResetCredits"
                                    .to_string()
                                }),
                            );
                        }
                        RateLimitRefreshOrigin::ResetPicker { request_id } => {
                            self.chat_widget.finish_rate_limit_reset_credits_refresh(
                                request_id,
                                snapshots,
                                rate_limit_reset_credits.ok_or_else(|| {
                                    "account/rateLimits/read response did not include rateLimitResetCredits"
                                        .to_string()
                                }),
                            );
                        }
                    }
                }
                Err(err) => {
                    // A failed read is not authoritative recovery. Keep the last valid banner.
                    tracing::warn!("account/rateLimits/read failed during TUI refresh: {err}");
                    match origin {
                        RateLimitRefreshOrigin::Recovery | RateLimitRefreshOrigin::Periodic => {
                            // Re-evaluate snapshot age even when the backend cannot refresh it.
                            // This updates display freshness without authorizing model recovery.
                            self.chat_widget.refresh_status_surfaces();
                        },
                        RateLimitRefreshOrigin::StartupPrefetch {
                            reset_hint_request_id,
                        } => {
                            self.chat_widget.finish_rate_limit_reset_hint_refresh(
                                reset_hint_request_id,
                                Vec::new(),
                                Err(err),
                            );
                        }
                        RateLimitRefreshOrigin::ResetConsume { request_id } => {
                            self.chat_widget.finish_post_consume_reset_credits_refresh(
                                request_id,
                                Vec::new(),
                                Err(err),
                            );
                        }
                        RateLimitRefreshOrigin::StatusCommand { request_id } => {
                            self.chat_widget
                                .finish_status_rate_limit_refresh(request_id, Vec::new());
                        }
                        RateLimitRefreshOrigin::UsageMenu { request_id } => {
                            self.chat_widget.finish_usage_menu_rate_limit_refresh(
                                request_id,
                                Vec::new(),
                                Err(err),
                            );
                        }
                        RateLimitRefreshOrigin::ResetPicker { request_id } => {
                            self.chat_widget.finish_rate_limit_reset_credits_refresh(
                                request_id,
                                Vec::new(),
                                Err(err),
                            );
                        }
                    }
                }
                }
                if (accepted || matches!(
                    origin,
                    RateLimitRefreshOrigin::Recovery | RateLimitRefreshOrigin::ResetConsume { .. }
                )) && !self.rate_limit_refresh_state.has_pending_recovery()
                {
                    self.chat_widget.finish_rate_limit_recovery();
                }
            },
            AppEvent::OpenAnalytics { view: summary_view } => {
                tui.enter_alt_screen()?;
                let mut view = self.retained_analytics.take().unwrap_or_else(|| {
                    Box::new(crate::analytics::AnalyticsView::new(self.keymap.list.clone()))
                });
                view.keymap = self.keymap.list.clone();
                if summary_view.is_some() {
                    view.select_summary(summary_view);
                }
                view.open(
                    app_server.request_handle(),
                    tui.frame_requester(),
                    self.model_catalog.try_list_models()?,
                    std::sync::Arc::new(self.config.clone()),
                );
                self.overlay = Some(Overlay::Analytics(view));
                tui.frame_requester().schedule_frame();
            }
            AppEvent::OpenRateLimitResetCredits => {
                let request_id = self.chat_widget.show_rate_limit_reset_loading_popup();
                self.refresh_rate_limits(
                    app_server,
                    RateLimitRefreshOrigin::ResetPicker { request_id },
                );
            }
            AppEvent::OpenRateLimitResetConfirmation {
                picker_request_id,
                confirmation_gate,
                credit_id,
                reset_title,
                reset_detail,
                reset_description,
            } => {
                self.chat_widget.show_rate_limit_reset_confirmation(
                    picker_request_id,
                    confirmation_gate,
                    credit_id,
                    reset_title,
                    reset_detail,
                    reset_description,
                );
            }
            AppEvent::ConsumeRateLimitResetCredit {
                idempotency_key,
                credit_id,
            } => {
                if let Some(request_id) = self
                    .chat_widget
                    .start_rate_limit_reset_consumption(&idempotency_key)
                {
                    self.consume_rate_limit_reset_credit(
                        app_server,
                        request_id,
                        idempotency_key,
                        credit_id,
                    );
                }
            }
            AppEvent::RateLimitResetCreditConsumed {
                request_id,
                idempotency_key,
                credit_id,
                result,
            } => {
                if let Err(err) = &result {
                    tracing::warn!(
                        "account/rateLimitResetCredit/consume failed during TUI request: {err}"
                    );
                }
                if self.chat_widget.finish_rate_limit_reset_consume(
                    request_id,
                    idempotency_key,
                    credit_id,
                    result,
                ) {
                    // Reads started before redemption must not restore the pre-reset banner.
                    self.rate_limit_hard_stop_generation =
                        self.rate_limit_hard_stop_generation.wrapping_add(1);
                    self.rate_limit_refresh_state.invalidate_recovery();
                    self.chat_widget.clear_backend_banner();
                    self.refresh_rate_limits(
                        app_server,
                        RateLimitRefreshOrigin::ResetConsume { request_id },
                    );
                }
            }
            AppEvent::ThreadUsageLoaded {
                thread_id,
                request_id,
                result,
            } => {
                self.finish_thread_usage_refresh(tui, thread_id, request_id, result)?;
            }
            AppEvent::AgentsOverviewUsageLoaded { thread_id, request_id, result } => {
                self.finish_agents_overview_usage(thread_id, request_id, result);
            }
            AppEvent::CommitPendingUsageOutput => {
                self.insert_pending_usage_output_if_ready(tui);
            }
            AppEvent::CommitPendingUsageOutputAfterStreamShutdown => {
                self.insert_pending_usage_output_after_stream_shutdown(tui);
            }
            AppEvent::ConnectorsLoaded {
                thread_id,
                cwd,
                generation,
                result,
                is_final,
            } => {
                if thread_id == self.current_displayed_thread_id()
                    && cwd.as_path() == self.chat_widget.config_ref().cwd.as_path()
                    && generation == self.chat_widget.connector_scope_generation()
                {
                    self.chat_widget.on_connectors_loaded(result, is_final);
                }
            }
            AppEvent::InstalledConnectorMentionsLoaded {
                thread_id,
                cwd,
                generation,
                result,
            } => {
                if thread_id == self.current_displayed_thread_id()
                    && cwd.as_path() == self.chat_widget.config_ref().cwd.as_path()
                    && generation == self.chat_widget.connector_scope_generation()
                {
                    self.chat_widget
                        .on_connector_mentions_loaded(generation, result);
                }
            }
            AppEvent::UpdateReasoningEffort(effort) => {
                self.on_update_reasoning_effort(effort.clone());
                self.refresh_session_header(tui)?;
                self.sync_active_thread_reasoning_setting(app_server, effort)
                    .await;
            }
<<<<<<< HEAD
            AppEvent::UpdateModel(model) => {
                self.chat_widget.set_model(&model);
                self.refresh_session_header(tui)?;
                self.sync_active_thread_model_setting(app_server, model)
                    .await;
                self.sync_active_thread_service_tier_to_cached_session()
||||||| 413492cd6c
            AppEvent::UpdateModel(model) => {
                self.chat_widget.set_model(&model);
                self.sync_active_thread_model_setting(app_server, model)
                    .await;
                self.sync_active_thread_service_tier_to_cached_session()
=======
            AppEvent::UpdateLunaReserveReasoning { thread_id, effort } => {
                self.update_luna_reserve_reasoning(app_server, thread_id, effort)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    .await;
            }
<<<<<<< HEAD
            AppEvent::UpdateModelSelection { model, provider } => {
                let selected_provider = provider
                    .as_deref()
                    .unwrap_or(self.config.model_provider_id.as_str());
                if !self
                    .model_catalog
                    .provider_is_selectable(selected_provider, &model)
                {
                    // Selection events can be queued behind credential-entry or secure-reveal
                    // overlays. Re-resolve the provider snapshot once before rejecting them so a
                    // credential that was stored and activated earlier in this event batch is
                    // immediately usable without a restart.
                    self.model_catalog.refresh_provider_policy();
                    if !self
                        .model_catalog
                        .provider_is_selectable(selected_provider, &model)
                    {
                        self.chat_widget.add_error_message(format!(
                            "Model provider `{selected_provider}` is inactive or unavailable. Repair or reactivate it in /providers."
                        ));
                        return Ok(AppRunControl::Continue);
                    }
                }
                // PF-84: a switch to another sign-in account waits until the
                // session has moved to it, then fetches that account's list.
                let mut account_switch = None;
                if let Some(model_provider) = provider.as_ref() {
                    let Some(provider_info) =
                        self.config.model_providers.get(model_provider).cloned()
                    else {
                        self.chat_widget.add_error_message(format!(
                            "Model provider `{model_provider}` is not configured."
                        ));
                        return Ok(AppRunControl::Continue);
                    };
                    account_switch =
                        sign_in_account_switch(&self.config.model_provider, &provider_info);
                    self.config.model_provider_id = model_provider.clone();
                    self.config.model_provider = provider_info.clone();
                    self.chat_widget
                        .set_model_provider(model_provider.clone(), provider_info);
                    self.model_catalog
                        .update_current_provider(model_provider.as_str());
                }
                self.chat_widget.set_model(&model);
                self.refresh_session_header(tui)?;
                let outcome = self
                    .sync_active_thread_model_selection(
                        app_server,
                        model,
                        provider,
                        account_switch.is_some(),
                    )
                    .await;
                // Only once core has moved: otherwise the list (and the
                // message) could still be the old account's.
                if let Some(account) = account_switch {
                    // #1251: the new account's sign-in summary, for `/status`.
                    self.check_named_accounts();
                    let message = match outcome {
                        SettingsUpdateOutcome::Confirmed => {
                            if self.refresh_model_list(app_server).await {
                                Some(account_switch_message(
                                    account.as_ref(),
                                    &self.config,
                                ))
                            } else {
                                Some(SWITCH_LIST_NOT_REFRESHED.to_string())
                            }
                        }
                        SettingsUpdateOutcome::Accepted => Some(SWITCH_NOT_CONFIRMED.to_string()),
                        // A failure already says so; nothing was sent otherwise.
                        SettingsUpdateOutcome::Failed | SettingsUpdateOutcome::NotSent => None,
                    };
                    if let Some(message) = message {
                        self.chat_widget.add_info_message(message, /*hint*/ None);
                    }
                }
                self.sync_active_thread_service_tier_to_cached_session()
                    .await;
            }
            AppEvent::UpdatePersonality(personality) => {
                self.on_update_personality(personality);
                self.sync_active_thread_personality_setting(app_server, personality)
||||||| 413492cd6c
            AppEvent::UpdatePersonality(personality) => {
                self.on_update_personality(personality);
                self.sync_active_thread_personality_setting(app_server, personality)
=======
            AppEvent::UpdateModel(model) => {
                if self
                    .active_thread_model_setting_update_params(model.clone())
                    .is_some_and(|params| params.permissions.is_some())
                    && self.reject_pending_permission_change()
                {
                    return Ok(AppRunControl::Continue);
                }
                let model_changed = self.chat_widget.current_model() != model
                    || self.chat_widget.current_collaboration_mode().model() != model;
                if model_changed {
                    self.chat_widget.set_model(&model);
                    self.sync_active_thread_model_setting(app_server, model, /*effort*/ None)
                        .await;
                    self.sync_active_thread_service_tier_to_cached_session()
                        .await;
                }
            }
            AppEvent::ApplyModelSelection { model, effort } => {
                if app_server.thread_params_mode().settings_owner()
                    == ThreadSettingsOwner::Server
                {
                    self.select_session_model(
                        app_server,
                        model,
                        effort,
                        /*service_tier*/ None,
                    )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    .await;
                } else {
                    self.apply_default_model_selection(app_server, model, effort)
                        .await;
                }
            }
            AppEvent::AstraSelectedFromModelPicker { .. } => unreachable!("picker event unwrapped"),
            AppEvent::BackgroundVoiceError { thread_id, message } => {
                if self.chat_widget.thread_id() == Some(thread_id) {
                    self.chat_widget.add_error_message(message);
                } else {
                    self.background_voice_error = Some((thread_id, message));
                }
            }
            AppEvent::RealtimeConversationStateChanged => {
                self.repaint_agents_overview();
            }
            AppEvent::VoiceControl { thread_id, control } => {
                if thread_id == self.chat_widget.thread_id() || self.voice_owner_thread_id().is_some() {
                    self.control_voice(control);
                }
            }
            AppEvent::RealtimeWebrtcOfferCreated {
                thread_id,
                attempt_id,
                result,
            } => {
                if let Some(owner) = self.voice_widget_for_thread(thread_id) {
                    owner.on_realtime_webrtc_offer_created(thread_id, attempt_id, result);
                } else if let Ok(offer) = result {
                    offer.handle.close();
                }
            }
            AppEvent::RealtimeWebrtcConnected {
                thread_id,
                attempt_id,
                result,
            } => {
                if let Some(owner) = self.voice_widget_for_thread(thread_id) {
                    owner.on_realtime_webrtc_connected(attempt_id, result);
                }
            }
            AppEvent::StopRealtimeConversation { thread_id } => {
                match tokio::time::timeout(
                    SHUTDOWN_FIRST_EXIT_TIMEOUT,
                    app_server.thread_realtime_stop(thread_id),
                )
                .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => tracing::warn!(
                        %thread_id,
                        %error,
                        "failed to stop voice conversation after switching threads"
                    ),
                    Err(_) => tracing::warn!(
                        %thread_id,
                        "timed out stopping voice conversation after switching threads"
                    ),
                }
            }
            AppEvent::SettingsSelectionClosed => {
                #[cfg(any(target_os = "windows", test))]
                let windows_sandbox_setup_pending =
                    self.windows_sandbox.pending_setup.is_some();
                #[cfg(not(any(target_os = "windows", test)))]
                let windows_sandbox_setup_pending = false;
                if self.chat_widget.no_modal_or_popup_active()
                    && !windows_sandbox_setup_pending
                {
                    self.chat_widget
                        .recover_inline_permission_profile_selection();
                }
                self.app_event_tx.send(AppEvent::SettingsSelectionSettled);
            }
            AppEvent::SettingsSelectionSettled => {
                if !self.chat_widget.permission_popup_pending
                    && self.chat_widget.no_modal_or_popup_active()
                    && !self
                        .chat_widget
                        .inline_permission_profile_selection_pending()
                    && !self
                        .chat_widget
                        .thread_id()
                        .is_some_and(|thread_id| {
                            self.pending_server_profiles.contains_key(&thread_id)
                                || self
                                    .agents_overview
                                    .requested_permission_profiles
                                    .get(&thread_id)
                                    .is_some_and(|selection| {
                                        let config = self.chat_widget.config_ref();
                                        config
                                            .permissions
                                            .active_permission_profile()
                                            .is_none_or(|active| {
                                                active.id != selection.profile_id
                                            })
                                            || selection.approval_policy.is_some_and(|policy| {
                                                config.permissions.approval_policy.value()
                                                    != policy.to_core()
                                            })
                                            || selection
                                                .approvals_reviewer
                                                .is_some_and(|reviewer| {
                                                    config.approvals_reviewer != reviewer
                                                })
                                    })
                        })
                {
                    let config = self.chat_widget.config_ref();
                    let permissions_override = Self::turn_permissions_override_from_config(
                        config,
                        config.permissions.active_permission_profile().as_ref(),
                        self.runtime_permission_profile_override
                            .as_ref()
                            .and_then(RuntimePermissionProfileOverride::turn_permission_profile),
                    );
                    if turn_permissions_overrides(permissions_override, config.cwd.as_path())
                        .is_ok()
                    {
                        self.chat_widget
                            .set_queue_autosend_suppressed(/*suppressed*/ false);
                        self.chat_widget.maybe_send_next_queued_input();
                    }
                }
            }
            AppEvent::FetchPermissionProfiles { request_id, thread_cwd } => {
                if self.chat_widget.permission_popup_request_is_current(request_id) {
                    crate::permission_discovery::fetch(
                        app_server,
                        request_id,
                        self.chat_widget.config_ref(),
                        thread_cwd.as_deref(),
                        self.app_event_tx.clone(),
                    );
                }
            }
            AppEvent::PermissionProfilesLoaded { request_id, result } => {
                self.chat_widget.on_permission_profiles_loaded(request_id, result);
            }
            AppEvent::FetchModels { request_id } => {
                if self.chat_widget.model_popup_request_is_current(request_id) {
                    app_server.fetch_models(request_id, self.app_event_tx.clone());
                }
            }
            AppEvent::ModelsLoaded { request_id, result: Ok(models) }
                if self.chat_widget.model_popup_request_is_current(request_id)
                    && self.chat_widget.status_account_display() == Some(&StatusAccountDisplay::ApiKey) =>
            {
                let checked = app_server.constrain_api_key_models(
                    self.chat_widget.config_ref(),
                    self.chat_widget.thread_id(),
                    /*requirements*/ None,
                    models,
                );
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    tx.send(AppEvent::ApiKeyCompatModelsLoaded { request_id, result: Ok(checked.await) });
                });
            }
            AppEvent::ModelsLoaded { request_id, result }
            | AppEvent::ApiKeyCompatModelsLoaded { request_id, result } => {
                self.chat_widget.on_models_loaded(request_id, result);
                let catalog = self.chat_widget.model_catalog();
                if !Arc::ptr_eq(&self.model_catalog, &catalog) {
                    self.model_catalog = catalog;
                    app_server.set_available_models(self.model_catalog.try_list_models()?);
                    self.sync_active_thread_service_tier_to_cached_session().await;
                }
            }
            AppEvent::OpenReasoningPopup { model, purpose } => {
                self.chat_widget
                    .open_reasoning_popup_for_purpose(model, purpose);
            }
            AppEvent::OpenAdvancedReasoningPopup { model, purpose } => {
                self.chat_widget
                    .open_advanced_reasoning_popup_for_purpose(model, purpose);
            }
            AppEvent::ApplyAdvancedReasoning { model, effort } => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                if app_server.thread_params_mode().settings_owner()
                    == ThreadSettingsOwner::Server
                {
                    self.select_session_model(
                        app_server,
                        model,
                        Some(effort),
                        /*service_tier*/ None,
                    )
                    .await;
                    if let Some(model) = sparkle_model.as_ref() {
                        self.chat_widget
                            .on_sparkle_model_selected_from_picker(model);
                    }
                    return Ok(AppRunControl::Continue);
                }
                if self
                    .active_thread_model_setting_update_params(model.clone())
                    .is_some_and(|params| params.permissions.is_some())
                    && self.reject_pending_permission_change()
                {
                    return Ok(AppRunControl::Continue);
                }
                let model_changed = self.chat_widget.current_model() != model
                    || self.chat_widget.current_collaboration_mode().model() != model;
                let default_effort =
                    self.on_apply_advanced_reasoning(model.as_str(), effort.clone());
<<<<<<< HEAD
                self.refresh_session_header(tui)?;
                if let Some(mut params) =
                    self.active_thread_model_setting_update_params(model.clone())
||||||| 413492cd6c
                if let Some(mut params) =
                    self.active_thread_model_setting_update_params(model.clone())
=======
                if model_changed {
                    self.sync_active_thread_model_setting(
                        app_server,
                        model.clone(),
                        Some(effort.clone()),
                    )
                    .await;
                } else if let Some(mut params) =
                    self.active_thread_reasoning_setting_update_params(Some(effort.clone()))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                {
                    params.collaboration_mode =
                        Some(self.chat_widget.effective_collaboration_mode());
                    self.send_thread_settings_update(app_server, params).await;
                }
                self.sync_active_thread_service_tier_to_cached_session()
                    .await;

                if let Some(default_effort) = default_effort.as_ref()
                    && let Err(err) = self.persist_model_defaults(
                        app_server.request_handle(),
                        crate::config_update::build_model_selection_edits(
                            model.as_str(),
                            /*provider*/ None,
                            Some(default_effort),
                        ),
                        "default model and reasoning effort",
                    )
                    .await
                {
                    let error = format_config_error(&err);
                    tracing::error!(error = %error, "failed to persist conversation model");
                    self.chat_widget
                        .add_error_message(format!("Failed to save default model: {error}"));
                } else {
                    self.chat_widget.add_info_message(
                        format!("Model changed to {model} {effort} for this conversation"),
                        /*hint*/ None,
                    );
                }
            }
            AppEvent::OpenPlanReasoningScopePrompt {
                model,
                provider,
                effort,
            } => {
                self.chat_widget
                    .open_plan_reasoning_scope_prompt(model, provider, effort);
            }
            AppEvent::OpenAllModelsPopup => {
                self.chat_widget.open_all_models_popup();
            }
            AppEvent::OpenGpuMenu => {
                self.refresh_gpu_spend_indicator().await;
                let Some(state_db) = self.state_db.as_ref() else {
                    self.chat_widget.add_error_message(
                        "GPU rental state is unavailable in this session.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                match state_db.list_gpu_rentals(/*limit*/ 100).await {
                    Ok(rentals) => self.chat_widget.open_gpu_menu(rentals),
                    Err(error) => self
                        .chat_widget
                        .add_error_message(format!("Unable to read GPU rentals: {error}")),
                }
            }
            AppEvent::OpenGpuRental { rental_id } => {
                let Some(state_db) = self.state_db.as_ref() else {
                    self.chat_widget.add_error_message(
                        "GPU rental state is unavailable in this session.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                match state_db.get_gpu_rental(rental_id.as_str()).await {
                    Ok(Some(rental)) => self.chat_widget.open_gpu_rental(rental),
                    Ok(None) => self
                        .chat_widget
                        .add_error_message(format!("GPU rental {rental_id} was not found.")),
                    Err(error) => self
                        .chat_widget
                        .add_error_message(format!("Unable to read GPU rental: {error}")),
                }
            }
            AppEvent::DisableGpuServing { rental_id } => {
                let Some(state_db) = self.state_db.as_ref() else {
                    self.chat_widget.add_error_message(
                        "GPU rental state is unavailable in this session.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                let now_ms = chrono::Utc::now().timestamp_millis();
                match state_db
                    .set_gpu_runtime_provider_health(rental_id.as_str(), "degraded", now_ms)
                    .await
                {
                    Ok(true) => self.chat_widget.add_info_message(
                        format!(
                            "Stopped serving GPU rental {rental_id}. Provider billing may continue."
                        ),
                        /*hint*/ None,
                    ),
                    Ok(false) => self.chat_widget.add_error_message(format!(
                        "GPU rental {rental_id} has no active runtime provider."
                    )),
                    Err(error) => self
                        .chat_widget
                        .add_error_message(format!("Unable to stop GPU serving: {error}")),
                }
                self.refresh_gpu_spend_indicator().await;
            }
            AppEvent::TerminateGpuRental { rental_id } => {
                let Some(state_db) = self.state_db.as_ref() else {
                    self.chat_widget.add_error_message(
                        "GPU rental state is unavailable in this session.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                let now_ms = chrono::Utc::now().timestamp_millis();
                let _ = state_db
                    .set_gpu_runtime_provider_health(rental_id.as_str(), "degraded", now_ms)
                    .await;
                match state_db
                    .request_gpu_rental_termination(rental_id.as_str(), now_ms)
                    .await
                {
                    Ok(true) => {
                        self.chat_widget.add_info_message(
                            format!(
                                "Termination requested for GPU rental {rental_id}; billing remains unresolved until the provider confirms absence."
                            ),
                            /*hint*/ None,
                        );
                        self.start_gpu_controller();
                    }
                    Ok(false) => self.chat_widget.add_error_message(format!(
                        "GPU rental {rental_id} is already terminal or cannot be terminated."
                    )),
                    Err(error) => self
                        .chat_widget
                        .add_error_message(format!("Unable to terminate GPU rental: {error}")),
                }
                self.refresh_gpu_spend_indicator().await;
            }
            AppEvent::OpenGpuAuthorizationPrompt { recipe_id, state } => self
                .chat_widget
                .open_gpu_authorization_prompt(recipe_id, state),
            AppEvent::SearchGpuOffers {
                recipe_id,
                maximum_hourly_microusd,
                maximum_total_microusd,
                ttl_minutes,
            } => {
                let Some(state_db) = self.state_db.clone() else {
                    self.chat_widget.add_error_message(
                        "GPU rental state is unavailable in this session.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                let tx = self.app_event_tx.clone();
                let codex_home = self.config.codex_home.clone();
                let now_ms = chrono::Utc::now().timestamp_millis();
                let authorization = codex_gpu_market::RentalAuthorization {
                    client_operation_id: Uuid::new_v4().to_string(),
                    maximum_hourly_microusd,
                    maximum_total_microusd,
                    terminate_at_ms: now_ms.saturating_add(ttl_minutes.saturating_mul(60_000)),
                    acknowledged_local_enforcement: true,
                };
                self.chat_widget.add_info_message(
                    format!("Searching verified capacity for {recipe_id}…"),
                    /*hint*/ None,
                );
                tokio::spawn(async move {
                    let result = async {
                        let installation_id =
                            crate::legacy_core::resolve_installation_id(&codex_home)
                                .await
                                .map_err(|error| error.to_string())?;
                        let credentials =
                            Arc::new(codex_gpu_market::VaultGpuCredentialResolver::new(Arc::new(
                                codex_vault::Vault::new(codex_home.to_path_buf()),
                            )));
                        let service = codex_gpu_market::GpuMarketService::new(
                            state_db,
                            codex_gpu_market::RecipeCatalog::default(),
                            installation_id,
                        );
                        service
                            .search(
                                recipe_id.as_str(),
                                maximum_hourly_microusd,
                                &codex_gpu_market::VastProvider::new(credentials.clone()),
                                &codex_gpu_market::RunpodProvider::new(credentials),
                            )
                            .await
                            .map_err(|error| error.safe_message)
                    }
                    .await;
                    tx.send(AppEvent::GpuOffersLoaded {
                        recipe_id,
                        authorization,
                        offers: result,
                    });
                });
            }
            AppEvent::GpuOffersLoaded {
                recipe_id,
                authorization,
                offers,
            } => match offers {
                Ok(offers) => {
                    self.chat_widget
                        .open_gpu_offers(recipe_id, authorization, offers);
                }
                Err(message) => self.chat_widget.add_error_message(message),
            },
            AppEvent::OpenGpuConfirmation {
                recipe_id,
                authorization,
                offer,
            } => self
                .chat_widget
                .open_gpu_confirmation(recipe_id, authorization, offer),
            AppEvent::ConfirmGpuRental {
                recipe_id,
                authorization,
                offer,
            } => {
                let Some(state_db) = self.state_db.clone() else {
                    self.chat_widget.add_error_message(
                        "GPU rental state is unavailable in this session.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                let tx = self.app_event_tx.clone();
                let codex_home = self.config.codex_home.clone();
                tokio::spawn(async move {
                    let result = async {
                        let installation_id =
                            crate::legacy_core::resolve_installation_id(&codex_home)
                                .await
                                .map_err(|error| error.to_string())?;
                        let credentials =
                            Arc::new(codex_gpu_market::VaultGpuCredentialResolver::new(Arc::new(
                                codex_vault::Vault::new(codex_home.to_path_buf()),
                            )));
                        let rental_id = format!("gpu-{}", authorization.client_operation_id);
                        credentials
                            .ensure_rental_endpoint_token(rental_id.as_str())
                            .map_err(|_| {
                                "Could not create the scoped GPU endpoint credential. No rental was started."
                                    .to_string()
                            })?;
                        let service = codex_gpu_market::GpuMarketService::new(
                            state_db,
                            codex_gpu_market::RecipeCatalog::default(),
                            installation_id,
                        );
                        let now_ms = chrono::Utc::now().timestamp_millis();
                        match offer.provider.as_str() {
                            "vast" => {
                                service
                                    .confirm(
                                        recipe_id.as_str(),
                                        &offer,
                                        &authorization,
                                        &codex_gpu_market::VastProvider::new(credentials.clone()),
                                        now_ms,
                                    )
                                    .await
                            }
                            "runpod" => {
                                service
                                    .confirm(
                                        recipe_id.as_str(),
                                        &offer,
                                        &authorization,
                                        &codex_gpu_market::RunpodProvider::new(credentials.clone()),
                                        now_ms,
                                    )
                                    .await
                            }
                            _ => Err(codex_gpu_market::ProviderError::new(
                                codex_gpu_market::ProviderErrorKind::InvalidRequest,
                                "Unsupported GPU provider.",
                            )),
                        }
                        .map_err(|error| error.safe_message)
                    }
                    .await;
                    tx.send(AppEvent::GpuRentalConfirmationFinished { result });
                });
            }
            AppEvent::GpuRentalConfirmationFinished { result } => match result {
                Ok(rental) => {
                    if rental.observed_state == codex_state::GpuRentalState::Failed {
                        let reason = rental.last_error_message.as_deref().unwrap_or(
                            "The earlier allocation attempt failed before a resource was created.",
                        );
                        self.chat_widget.add_error_message(format!(
                            "That confirmation already produced GPU rental {}, which failed: {reason} Choose another current offer from /gpu.",
                            rental.rental_id
                        ));
                    } else {
                        self.chat_widget.add_info_message(
                            format!(
                                "GPU rental {} was authorized. The independent controller is starting; /gpu remains authoritative for billing state.",
                                rental.rental_id
                            ),
                            /*hint*/ None,
                        );
                        self.start_gpu_controller();
                    }
                    self.refresh_gpu_spend_indicator().await;
                }
                Err(message) => self.chat_widget.add_error_message(message),
            },
            AppEvent::OpenGpuProviderCredential { provider } => {
                self.chat_widget.open_gpu_provider_credential(provider);
            }
            AppEvent::SaveGpuProviderCredential { provider, api_key } => {
                let (label, display_name) = match provider.as_str() {
                    "runpod" => (codex_gpu_market::RUNPOD_API_KEY_LABEL, "RunPod"),
                    "vast" => (codex_gpu_market::VAST_API_KEY_LABEL, "Vast.ai"),
                    _ => {
                        self.chat_widget
                            .add_error_message("Unsupported GPU provider credential.".to_string());
                        return Ok(AppRunControl::Continue);
                    }
                };
                let vault = codex_vault::Vault::new(self.config.codex_home.clone().to_path_buf());
                let secret = api_key.into_inner();
                let result = if vault.exists(label).unwrap_or(false) {
                    vault
                        .update(
                            label,
                            Some(secret),
                            /*provider*/ None,
                            /*notes*/ None,
                            /*revocation_notes*/ None,
                        )
                        .map(|_| ())
                } else {
                    vault.add(codex_vault::AddCredential {
                        label: label.to_string(),
                        credential_type: codex_vault::CredentialType::ApiKey,
                        provider: Some(provider),
                        notes: Some("Corbanu Terminal GPU rental provider credential".to_string()),
                        revocation_notes: Some(
                            "Revoke at the provider and delete from /vault when retired."
                                .to_string(),
                        ),
                        secret,
                    })
                };
                match result {
                    Ok(()) => self.chat_widget.add_info_message(
                        format!("Stored {display_name} API key in the vault."),
                        /*hint*/ None,
                    ),
                    Err(_) => self.chat_widget.add_error_message(format!(
                        "Could not store {display_name} API key in the vault."
                    )),
                }
            }
            AppEvent::OpenProviderApiKeyAdd {
                provider_id,
                provider_name,
                env_key,
            } => {
                self.chat_widget
                    .open_provider_api_key_add(provider_id, provider_name, env_key);
            }
            AppEvent::SaveProviderApiKey {
                provider_id,
                display_name,
                api_key,
            } => {
                let save_id = Uuid::new_v4().to_string();
                self.chat_widget.open_provider_api_key_save_pending(
                    save_id.clone(),
                    display_name.clone(),
                );
                let request_handle = app_server.request_handle();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = request_handle
                        .request_typed::<codex_app_server_protocol::LoginAccountResponse>(
                            ClientRequest::LoginAccount {
                                request_id: codex_app_server_protocol::RequestId::String(format!(
                                    "provider-api-key-login-{save_id}"
                                )),
                                params:
                                    codex_app_server_protocol::LoginAccountParams::ProviderApiKey {
                                        provider: provider_id,
                                        api_key: api_key.into_inner(),
                                    },
                            },
                        )
                        .await;

                    let result = match result {
                        Ok(codex_app_server_protocol::LoginAccountResponse::ApiKey {}) => Ok(()),
                        Ok(other) => Err(format!(
                            "unexpected account/login/start response: {other:?}"
                        )),
                        Err(err) => Err(err.to_string()),
                    };
                    tx.send(AppEvent::ProviderApiKeySaveFinished {
                        save_id,
                        display_name,
                        result,
                    });
                });
            }
            AppEvent::OpenSharedProviderSetup => {
                let host = self.reusable_provider_status_host().unwrap_or_else(|| {
                    crate::provider_status_host::ProviderStatusHost::from_config(
                        &self.config,
                        crate::provider_status_host::ProviderAccountMetadata {
                            claude: codex_provider_auth::ClaudeCredentialMetadata::Checking,
                            ..Default::default()
                        },
                    )
                });
                let session = crate::onboarding::provider_setup::ProviderSetupSession::from_statuses(
                    host.resolve().entries(),
                );
                self.shared_provider_status_host = Some(host.clone());
                self.shared_provider_setup_session = Some(session);
                self.shared_provider_account_auth_host = Some(
                    crate::provider_account_auth_host::ProviderAccountAuthHost::new(
                        app_server.request_handle(),
                        self.app_event_tx.clone(),
                        host,
                        self.config.clone(),
                    ),
                );
                self.render_shared_provider_setup();
                let config = self.config.clone();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let metadata =
                        crate::provider_status_host::ProviderAccountMetadata::discover(&config)
                            .await;
                    tx.send(AppEvent::ProviderAccountMetadataResolved { metadata });
                });
            }
            AppEvent::ProviderAccountMetadataResolved { metadata } => {
                let refreshed = if let Some(host) = self.shared_provider_status_host.as_ref() {
                    host.update_account_metadata(metadata);
                    refresh_shared_provider_session_if_idle(
                        self.shared_provider_setup_session.as_mut(),
                        || host.resolve().entries().to_vec(),
                    )
                } else {
                    false
                };
                if refreshed {
                    self.render_shared_provider_setup();
                }
                self.model_catalog.refresh_provider_policy();
            }
            AppEvent::SharedProviderSetupBegin {
                provider_id,
                capability,
            } => {
                let Some(host) = self.shared_provider_status_host.as_ref() else {
                    return Ok(AppRunControl::Continue);
                };
                let Some(entry) = host.catalog().get(provider_id.as_str()) else {
                    return Ok(AppRunControl::Continue);
                };
                let Some(session) = self.shared_provider_setup_session.as_mut() else {
                    return Ok(AppRunControl::Continue);
                };
                if !session
                    .dispatch(crate::onboarding::provider_setup::ProviderSetupAction::Begin {
                        provider_id: provider_id.clone(),
                    })
                    .applied
                {
                    return Ok(AppRunControl::Continue);
                }
                match capability {
                    codex_provider_auth::ProviderSetupCapability::ApiKey { .. } => {
                        match codex_provider_auth::ApiKeyAuthTarget::from_catalog_capability(
                            entry,
                            &capability,
                        ) {
                            Ok(target) => self.chat_widget.open_shared_provider_api_key(
                                target,
                                entry.display_name.clone(),
                            ),
                            Err(_) => {
                                session.dispatch(
                                    crate::onboarding::provider_setup::ProviderSetupAction::AuthFailed,
                                );
                                self.render_shared_provider_setup();
                            }
                        }
                    }
                    codex_provider_auth::ProviderSetupCapability::OpenAiAccount => {
                        let Ok(target) = codex_provider_auth::OpenAiAccountTarget::from_catalog_entry(entry) else {
                            return Ok(AppRunControl::Continue);
                        };
                        let status = host.resolve_provider(provider_id.as_str());
                        if let Some(status) = status {
                            self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
                                codex_provider_auth::OpenAiAccountAction::Start(
                                    codex_provider_auth::OpenAiAccountFlowStart {
                                        target,
                                        method: codex_provider_auth::OpenAiAccountMethod::DeviceCode,
                                        context: codex_provider_auth::OpenAiAccountLoginContext::ProviderEnrollment,
                                        status,
                                    },
                                ).into(),
                            ));
                        }
                    }
                    codex_provider_auth::ProviderSetupCapability::ClaudeAccount => {
                        let Ok(target) = codex_provider_auth::claude_account_flow::ClaudeAccountTarget::from_catalog_entry(entry) else {
                            return Ok(AppRunControl::Continue);
                        };
                        if let Some(status) = host.resolve_provider(provider_id.as_str()) {
                            self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
                                codex_provider_auth::claude_account_flow::ClaudeAccountAction::Start(
                                    codex_provider_auth::claude_account_flow::ClaudeAccountFlowStart {
                                        target,
                                        intent: codex_provider_auth::claude_account_flow::ClaudeAccountIntent::Add,
                                        status,
                                    },
                                ).into(),
                            ));
                        }
                    }
                    _ => {
                        session.dispatch(
                            crate::onboarding::provider_setup::ProviderSetupAction::AuthFailed,
                        );
                        self.render_shared_provider_setup();
                    }
                }
            }
            AppEvent::SharedProviderSetupSelectExisting {
                provider_id,
                runtime_provider_id,
            } => {
                let Some(model) = resolve_shared_provider_selection_model(
                    self.config.model.clone(),
                    self.chat_widget.current_model(),
                    runtime_provider_id.as_str(),
                ) else {
                    self.chat_widget.add_error_message(
                        "No compatible model is available for the selected provider.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                let Some(session) = self.shared_provider_setup_session.as_mut() else {
                    return Ok(AppRunControl::Continue);
                };
                let transition = session.dispatch(
                    crate::onboarding::provider_setup::ProviderSetupAction::SelectExisting {
                        provider_id,
                        runtime_provider_id,
                    },
                );
                for effect in transition.effects {
                    if let crate::onboarding::provider_setup::ProviderSetupEffect::PersistInitialSelection(
                        runtime,
                    ) = effect
                    {
                        self.app_event_tx.send(AppEvent::UpdateModelSelection {
                            model: model.clone(),
                            provider: Some(runtime.to_string()),
                        });
                        self.app_event_tx.send(AppEvent::PersistModelSelection {
                            model: model.clone(),
                            provider: Some(runtime.to_string()),
                            effort: None,
                        });
                    }
                }
                self.render_shared_provider_setup();
            }
            AppEvent::SharedProviderSetupQueueCorbanu(queued) => {
                if let Some(session) = self.shared_provider_setup_session.as_mut() {
                    session.dispatch(
                        crate::onboarding::provider_setup::ProviderSetupAction::QueueCorbanu(queued),
                    );
                    self.render_shared_provider_setup();
                }
            }
            AppEvent::SaveSharedProviderApiKey {
                target,
                api_key,
            } => {
                let request_handle = app_server.request_handle();
                let tx = self.app_event_tx.clone();
                let Some(host) = self.shared_provider_status_host.clone() else {
                    return Ok(AppRunControl::Continue);
                };
                let completed_target = target.clone();
                tokio::spawn(async move {
                    let result = crate::provider_auth_effect_executor::ProviderAuthEffectExecutor::persist_api_key(
                        request_handle,
                        host,
                        target,
                        codex_provider_auth::ApiKeySecret::new(api_key.into_inner()),
                    )
                    .await
                    .map_err(|error| format!("{error:?}"));
                    tx.send(AppEvent::SharedProviderApiKeyFinished {
                        target: completed_target,
                        result,
                    });
                });
            }
            AppEvent::SharedProviderApiKeyFinished { target, result } => {
                self.finish_shared_provider_api_key(target, result.ok());
                self.render_shared_provider_setup();
            }
            AppEvent::SharedProviderSetupDone => {
                self.finish_or_defer_shared_provider_setup();
            }
            AppEvent::SharedProviderSetupCancelled => {
                let can_finish = self
                    .shared_provider_setup_session
                    .as_ref()
                    .is_some_and(super::super::onboarding::provider_setup::ProviderSetupSession::can_finish);
                if can_finish {
                    self.finish_or_defer_shared_provider_setup();
                } else {
                    self.chat_widget.add_error_message(
                        "Configure a usable provider before leaving provider setup.".to_string(),
                    );
                    self.render_shared_provider_setup();
                }
            }
            AppEvent::SharedProviderAuthAction(action) => {
                if self
                    .provider_management_host
                    .as_ref()
                    .is_some_and(|host| host.authenticating_provider().is_some())
                {
                    self.handle_provider_manager_auth_action(action);
                } else {
                    self.handle_shared_provider_auth_action(action);
                }
            }
            AppEvent::OpenProviderManager => self.open_provider_manager(app_server),
            AppEvent::ProviderManagerStatusesResolved {
                generation,
                status_host,
                statuses,
                accounts,
            } => self.provider_manager_statuses_resolved(
                generation,
                status_host,
                statuses,
                accounts,
                app_server,
            ),
            AppEvent::ProviderAccount(event) => {
                self.handle_provider_account_event(tui, app_server, event)
                    .await;
            }
            AppEvent::NamedBedrockAccountChecked {
                generation,
                offered,
            } => self.named_bedrock_account_checked(generation, offered),
            AppEvent::ProviderManagerMetadataResolved {
                generation,
                metadata,
            } => {
                self.provider_manager_metadata_resolved(generation, metadata);
            }
            AppEvent::OpenProviderManagerActions { provider_id } => {
                self.open_provider_manager_actions(provider_id);
            }
            AppEvent::OpenProviderManagerRecovery { provider_id } => {
                self.open_provider_manager_recovery(provider_id);
            }
            AppEvent::ProviderManagerApiKeyCancelled { attempt_id, provider_id } => {
                if self.provider_management_host.as_ref()
                    .and_then(crate::provider_management_host::ProviderManagementHost::authenticating_attempt)
                    == Some((attempt_id, provider_id.clone()))
                {
                    self.provider_manager_authentication_cancelled(provider_id);
                }
            }
            AppEvent::ProviderManagerBeginAuthentication {
                provider_id,
                capability,
            } => self.provider_manager_begin_authentication(provider_id, capability),
            AppEvent::ProviderManagerClaudeRecoverySourceResolved {
                attempt_id,
                target,
                status,
                source,
            } => self.provider_manager_claude_recovery_source_resolved(
                attempt_id, target, status, source,
            ),
            AppEvent::SaveProviderManagerApiKey {
                attempt_id,
                target,
                api_key,
            } => {
                let Some(host) = self.provider_management_host.as_ref() else {
                    return Ok(AppRunControl::Continue);
                };
                if host.authenticating_attempt()
                    != Some((attempt_id, target.provider_id.clone()))
                {
                    return Ok(AppRunControl::Continue);
                }
                let request_handle = app_server.request_handle();
                let status_host = host.status_host().clone();
                let tx = self.app_event_tx.clone();
                let completed_target = target.clone();
                tokio::spawn(async move {
                    let status = crate::provider_auth_effect_executor::ProviderAuthEffectExecutor::persist_api_key(
                        request_handle,
                        status_host,
                        target,
                        codex_provider_auth::ApiKeySecret::new(api_key.into_inner()),
                    )
                    .await
                    .ok();
                    tx.send(AppEvent::ProviderManagerApiKeyFinished {
                        attempt_id,
                        target: completed_target,
                        status,
                    });
                });
            }
            AppEvent::ProviderManagerApiKeyFinished {
                attempt_id,
                target,
                status,
            } => self.provider_manager_api_key_finished(attempt_id, target, status),
            AppEvent::ProviderManagerRequestPolicy {
                provider_id,
                policy,
            } => self.provider_manager_request_policy(provider_id, policy),
            AppEvent::ProviderManagerChooseReplacement {
                target_provider_id,
                replacement,
            } => self.provider_manager_choose_replacement(
                target_provider_id,
                replacement,
                app_server,
            ),
            AppEvent::ProviderManagerCancelReplacement { target_provider_id } => {
                self.provider_manager_cancel_replacement(target_provider_id);
            }
            AppEvent::ProviderManagerPersistenceFinished { attempt_id, result } => {
                let policy_may_have_changed = !matches!(
                    result,
                    codex_provider_auth::ProviderManagementPersistenceResult::Failed
                );
                self.provider_manager_persistence_finished(attempt_id, result);
                if policy_may_have_changed {
                    self.model_catalog.sync_runtime_models(
                        self.config.model_providers.keys().map(String::as_str),
                        self.config.model.as_deref(),
                        &self.config.model_provider_id,
                    );
                    self.model_catalog.refresh_provider_policy();
                }
            }
            AppEvent::ProviderManagerReplacementPersisted {
                attempt_id,
                target_provider_id,
                replacement,
                success,
            } => self.provider_manager_replacement_persisted(
                attempt_id,
                target_provider_id,
                replacement,
                success,
            ),
            AppEvent::ProviderManagerDeactivateAfterReplacement {
                attempt_id,
                target_provider_id,
            } => self.provider_manager_deactivate_after_replacement(
                attempt_id,
                target_provider_id,
            ),
            AppEvent::ProviderApiKeySaveDismissed { save_id } => {
                self.chat_widget.on_provider_api_key_save_dismissed(save_id);
            }
            AppEvent::ProviderApiKeySaveFinished {
                save_id,
                display_name,
                result,
            } => {
                self.chat_widget
                    .on_provider_api_key_save_finished(save_id, display_name, result);
            }
            AppEvent::OpenTelegram => {
                self.chat_widget.open_telegram_menu();
            }
            AppEvent::OpenTelegramTokenEntry => {
                self.chat_widget.open_telegram_token_entry();
            }
            AppEvent::ValidateTelegramToken { token } => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = crate::chatwidget::telegram_setup::validate_and_store_token(
                        codex_home, token,
                    )
                    .await;
                    tx.send(AppEvent::TelegramTokenValidated { result });
                });
            }
            AppEvent::TelegramTokenValidated { result } => match result {
                Ok(identity) => {
                    let generation = self.chat_widget.begin_telegram_discovery(Some(identity));
                    self.app_event_tx
                        .send(AppEvent::PollTelegramChats { generation });
                }
                Err(error) => {
                    self.chat_widget.add_error_message(error);
                    self.chat_widget.open_telegram_token_entry();
                }
            },
            AppEvent::DiscoverTelegramChats => {
                let generation = self.chat_widget.begin_telegram_discovery(/*identity*/ None);
                self.app_event_tx
                    .send(AppEvent::PollTelegramChats { generation });
            }
            AppEvent::PollTelegramChats { generation } => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result =
                        crate::chatwidget::telegram_setup::discover_chats(codex_home).await;
                    tx.send(AppEvent::TelegramChatsDiscovered { generation, result });
                });
            }
            AppEvent::TelegramChatsDiscovered { generation, result } => match result {
                Ok(discovery) => {
                    if self
                        .chat_widget
                        .apply_telegram_discovery(generation, discovery)
                    {
                        let tx = self.app_event_tx.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                            tx.send(AppEvent::PollTelegramChats { generation });
                        });
                    }
                }
                Err(error) => self
                    .chat_widget
                    .telegram_discovery_failed(generation, error),
            },
            AppEvent::ConfirmTelegramChat { candidate } => {
                self.chat_widget.confirm_telegram_chat(candidate);
            }
            AppEvent::ConnectTelegramChat {
                candidate,
                defaults,
            } => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::chatwidget::telegram_setup::connect_chat(
                            &codex_home,
                            candidate,
                            defaults,
                        )
                    })
                    .await
                    .map_err(|error| format!("Telegram connection task failed: {error}"))
                    .and_then(|result| result);
                    tx.send(AppEvent::TelegramOperationFinished { result });
                });
            }
            AppEvent::StartTelegramConnector => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::chatwidget::telegram_setup::start_connector(&codex_home)
                    })
                    .await
                    .map_err(|error| format!("Telegram start task failed: {error}"))
                    .and_then(|result| result);
                    tx.send(AppEvent::TelegramOperationFinished { result });
                });
            }
            AppEvent::StopTelegramConnector => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::chatwidget::telegram_setup::stop_connector(&codex_home)
                    })
                    .await
                    .map_err(|error| format!("Telegram stop task failed: {error}"))
                    .and_then(|result| result);
                    tx.send(AppEvent::TelegramOperationFinished { result });
                });
            }
            AppEvent::ReplaceTelegramBot => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::chatwidget::telegram_setup::stop_connector(&codex_home)
                    })
                    .await
                    .map_err(|error| format!("Telegram stop task failed: {error}"))
                    .and_then(|result| result);
                    if result.is_ok() {
                        tx.send(AppEvent::OpenTelegramTokenEntry);
                    } else {
                        tx.send(AppEvent::TelegramOperationFinished { result });
                    }
                });
            }
            AppEvent::ConfirmTelegramDisconnect => {
                self.chat_widget.confirm_telegram_disconnect();
            }
            AppEvent::DisconnectTelegram => {
                let codex_home = self.config.codex_home.clone().to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        crate::chatwidget::telegram_setup::disconnect(&codex_home)
                    })
                    .await
                    .map_err(|error| format!("Telegram disconnect task failed: {error}"))
                    .and_then(|result| result);
                    tx.send(AppEvent::TelegramOperationFinished { result });
                });
            }
            AppEvent::TelegramOperationFinished { result } => {
                match result {
                    Ok(message) => self.chat_widget.add_info_message(message, /*hint*/ None),
                    Err(error) => self.chat_widget.add_error_message(error),
                }
                self.chat_widget.open_telegram_menu();
            }
            AppEvent::TelegramStatusReady { result } => {
                self.chat_widget.refresh_telegram_menu(result);
            }
            AppEvent::OpenCodexAccountDeviceLogin => {
                self.chat_widget.open_codex_account_device_login_pending();
                let request_handle = app_server.request_handle();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = request_handle
                        .request_typed::<codex_app_server_protocol::LoginAccountResponse>(
                            ClientRequest::LoginAccount {
                                request_id: codex_app_server_protocol::RequestId::String(format!(
                                    "codex-account-device-login-{}",
                                    Uuid::new_v4()
                                )),
                                params: codex_app_server_protocol::LoginAccountParams::OpenaiProviderDeviceCode,
                            },
                        )
                        .await;

                    match result {
                        Ok(
                            codex_app_server_protocol::LoginAccountResponse::ChatgptDeviceCode {
                                login_id,
                                verification_url,
                                user_code,
                            },
                        ) => {
                            tx.send(AppEvent::CodexAccountDeviceLoginReady {
                                login_id,
                                verification_url,
                                user_code,
                            });
                        }
                        Ok(other) => {
                            tx.send(AppEvent::CodexAccountDeviceLoginFailed {
                                message: format!(
                                    "Unexpected account/login/start response: {other:?}"
                                ),
                            });
                        }
                        Err(err) => {
                            tx.send(AppEvent::CodexAccountDeviceLoginFailed {
                                message: err.to_string(),
                            });
                        }
                    }
                });
            }
            AppEvent::OpenClaudeCodePlanLogin => {
                self.chat_widget.open_claude_auth_method_choice();
            }
            AppEvent::RunClaudeSetupToken => {
                self.chat_widget.open_claude_subscription_token_entry();
                tui.frame_requester().schedule_frame();
            }
            AppEvent::UseClaudeCodePlanLogin => {
                let codex_home = self.config.codex_home.clone();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result =
                        crate::chatwidget::claude_code_login::select_existing_claude_code_login(
                            std::path::Path::new("claude"),
                            /*health_executable*/ None,
                            codex_home.as_path(),
                            Duration::from_secs(10),
                        )
                        .await;
                    tx.send(AppEvent::ClaudeCodePlanLoginSelectionChecked { result });
                });
            }
            AppEvent::UseLegacyClaudeEnvironmentToken => {
                let codex_home = self.config.codex_home.clone();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        let token = std::env::var("CLAUDE_CODE_OAUTH_TOKEN")
                            .ok()
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                "CLAUDE_CODE_OAUTH_TOKEN is missing or blank; the current method was not changed."
                                    .to_string()
                            })?;
                        let selection = codex_vault::ClaudeAuthSelection::new_environment_token(
                            &token,
                        )
                        .map_err(|error| {
                            format!("Could not identify CLAUDE_CODE_OAUTH_TOKEN: {error}")
                        })?;
                        codex_vault::Vault::new(codex_home.to_path_buf())
                            .save_claude_auth_selection(&selection)
                            .map_err(|error| {
                                format!(
                                    "Could not select CLAUDE_CODE_OAUTH_TOKEN: {error}. The current method was not changed."
                                )
                            })?;
                        Ok(
                            "Legacy CLAUDE_CODE_OAUTH_TOKEN selected. Retry the Claude Plan request or choose a model from /model."
                                .to_string(),
                        )
                    })
                    .await
                    .unwrap_or_else(|error| {
                        Err(format!(
                            "Could not select CLAUDE_CODE_OAUTH_TOKEN: {error}. The current method was not changed."
                        ))
                    });
                    tx.send(AppEvent::LegacyClaudeEnvironmentTokenSelected { result });
                });
            }
            AppEvent::LegacyClaudeEnvironmentTokenSelected { result } => match result {
                Ok(message) => self.chat_widget.add_info_message(message, /*hint*/ None),
                Err(message) => {
                    self.chat_widget.add_error_message(message.clone());
                    self.chat_widget.open_claude_auth_recovery(message);
                }
            },
            AppEvent::ClaudeCodePlanLoginSelectionChecked { result } => match result {
                    Ok(true) => self.chat_widget.add_info_message(
                        "Existing Claude Code login selected. Retry the Claude Plan request or choose a model from /model."
                            .to_string(),
                        /*hint*/ None,
                    ),
                    Ok(false) => {
                        let input_tx = crate::chatwidget::claude_code_login::start(
                            self.app_event_tx.clone(),
                            self.config.codex_home.to_path_buf(),
                        );
                        self.chat_widget
                            .open_claude_code_plan_login_pending(input_tx);
                    }
                    Err(message) => {
                        self.chat_widget.add_error_message(message.clone());
                        self.chat_widget.open_claude_auth_recovery(message);
                    }
                },
            AppEvent::SaveClaudeManagedSubscriptionToken { token } => {
                let codex_home = self.config.codex_home.clone();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = crate::chatwidget::claude_code_login::enroll_managed_subscription_token(
                        codex_home.to_path_buf(),
                        token,
                    )
                    .await;
                    tx.send(AppEvent::ClaudeManagedSubscriptionTokenSaved { result });
                });
            }
            AppEvent::ClaudeManagedSubscriptionTokenSaved { result } => {
                self.chat_widget
                    .on_claude_managed_subscription_token_saved(result);
            }
            AppEvent::ClaudeCodePlanLoginReady {
                verification_url,
                input_tx,
            } => {
                self.chat_widget
                    .open_claude_code_plan_login_ready(verification_url, input_tx);
            }
            AppEvent::OpenClaudeCodePlanLoginCodeEntry { input_tx } => {
                self.chat_widget
                    .open_claude_code_plan_login_code_entry(input_tx);
            }
            AppEvent::ClaudeCodePlanLoginFinished { result } => {
                self.chat_widget.on_claude_code_plan_login_finished(result);
            }
            AppEvent::ProviderCredentialStatusesReady {
                claude_status,
                pfterminal_plan_status,
                api_key_statuses,
            } => {
                self.chat_widget.refresh_provider_credentials_status(
                    claude_status,
                    pfterminal_plan_status,
                    api_key_statuses,
                );
            }
            AppEvent::CodexAccountDeviceLoginReady {
                login_id,
                verification_url,
                user_code,
            } => {
                self.chat_widget.open_codex_account_device_login_ready(
                    login_id,
                    verification_url,
                    user_code,
                );
            }
            AppEvent::CodexAccountDeviceLoginFailed { message } => {
                self.chat_widget
                    .on_codex_account_device_login_failed(message);
            }
            AppEvent::ProviderAccountLoginCompleted { login_id, success } => {
                if let Some(login_id) = login_id {
                    if let Some(host) = self.provider_management_host.as_ref() {
                        host.account_auth_host_ref()
                            .openai_login_completed(login_id.clone(), success);
                    }
                    if let Some(host) = self.shared_provider_account_auth_host.as_ref() {
                        host.openai_login_completed(login_id.clone(), success);
                    }
                    self.chat_widget.on_codex_account_login_result(
                        login_id,
                        success,
                        /*failure_message*/ None,
                    );
                }
            }
            AppEvent::CancelCodexAccountDeviceLogin { login_id } => {
                self.chat_widget.add_info_message(
                    "OpenAI Codex account login canceled.".to_string(),
                    /*hint*/ None,
                );
                let request_handle = app_server.request_handle();
                tokio::spawn(async move {
                    let _ = request_handle
                        .request_typed::<codex_app_server_protocol::CancelLoginAccountResponse>(
                            ClientRequest::CancelLoginAccount {
                                request_id: codex_app_server_protocol::RequestId::String(format!(
                                    "cancel-codex-account-device-login-{}",
                                    Uuid::new_v4()
                                )),
                                params: codex_app_server_protocol::CancelLoginAccountParams {
                                    login_id,
                                },
                            },
                        )
                        .await;
                });
            }
            AppEvent::OpenVaultCredentialAdd => {
                self.chat_widget.open_vault_credential_add();
            }
            AppEvent::VaultCredentialAddRequested { label, secret } => {
                self.chat_widget.add_vault_credential(label, secret);
            }
            AppEvent::VaultCredentialAdded { label, result } => {
                self.chat_widget.on_vault_credential_added(label, result);
            }
            AppEvent::OpenWallet => {
                self.chat_widget.open_wallet_menu();
            }
            AppEvent::OpenWalletPlanUsage => {
                self.chat_widget.open_wallet_plan_usage();
            }
            AppEvent::WalletPlanUsageReady { result } => {
                self.chat_widget.on_wallet_plan_usage_ready(result);
            }
            AppEvent::OpenWalletCreate => {
                self.chat_widget.open_wallet_create();
            }
            AppEvent::OpenWalletRestore => {
                self.chat_widget.open_wallet_restore();
            }
            AppEvent::OpenWalletRestorePasscode { recovery } => {
                self.chat_widget.open_wallet_restore_passcode(recovery);
            }
            AppEvent::OpenWalletRecoveryBackup => {
                self.chat_widget.open_wallet_recovery_backup();
            }
            AppEvent::WalletRecoveryBackupFinished { result } => {
                self.chat_widget.on_wallet_recovery_backup_finished(result);
            }
            AppEvent::OpenWalletUnlock {
                policy,
                continuation,
            } => {
                if wallet_unlock_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    &continuation,
                ) {
                    self.chat_widget.open_wallet_unlock(policy, continuation);
                }
            }
            AppEvent::WalletUnlockPreflightFinished {
                policy,
                continuation,
                result,
            } => {
                if wallet_unlock_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    &continuation,
                ) {
                    self.chat_widget
                        .on_wallet_unlock_preflight_finished(policy, continuation, result);
                }
            }
            AppEvent::OpenWalletCustomUnlock {
                validation_error,
                continuation,
            } => {
                if wallet_unlock_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    &continuation,
                ) {
                    self.chat_widget
                        .open_wallet_custom_unlock(validation_error, continuation);
                }
            }
            AppEvent::WalletLockRequested => {
                self.chat_widget.lock_wallet();
            }
            AppEvent::ConfirmWalletPlanDisconnect => {
                self.chat_widget.confirm_wallet_plan_disconnect();
            }
            AppEvent::WalletPlanDisconnectRequested => {
                self.chat_widget.disconnect_wallet_plan();
            }
            AppEvent::WalletPlanDisconnected { result } => {
                self.chat_widget.on_wallet_plan_disconnected(result);
            }
            AppEvent::ConfirmWalletRemoval { address } => {
                self.chat_widget.confirm_wallet_removal(address);
            }
            AppEvent::WalletRemoveRequested { address } => {
                self.chat_widget.remove_wallet_from_device(address);
            }
            AppEvent::WalletRemoved { result } => {
                self.chat_widget.on_wallet_removed(result);
            }
            AppEvent::WalletStatusReady { generation, result } => {
                self.chat_widget.on_wallet_status_ready(generation, result);
            }
            AppEvent::OpenCorbanuApi { deferred } => {
                if corbanu_api_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    deferred.as_ref(),
                ) {
                    if let Some(deferred) = deferred {
                        self.chat_widget.open_corbanu_api_for_deferred(deferred);
                    } else {
                        self.chat_widget.open_corbanu_api();
                    }
                }
            }
            AppEvent::CorbanuApiLoaded {
                result,
                deferred,
                capability_lapsed,
            } => {
                if capability_lapsed {
                    self.chat_widget.forget_lapsed_wallet_capability();
                }
                if corbanu_api_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    deferred.as_ref(),
                ) {
                    self.chat_widget
                        .on_corbanu_api_loaded_with_deferred(result, deferred);
                }
            }
            AppEvent::OpenCorbanuApiTopUp { deferred } => {
                if corbanu_api_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    deferred.as_ref(),
                ) {
                    self.chat_widget.open_corbanu_api_top_up(deferred);
                }
            }
            AppEvent::ConfirmCorbanuApiTopUp {
                amount_usd,
                deferred,
            } => {
                if corbanu_api_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    deferred.as_ref(),
                ) {
                    self.chat_widget
                        .confirm_corbanu_api_top_up(amount_usd, deferred);
                }
            }
            AppEvent::ConfirmCorbanuApiKeyRevocation {
                key_id,
                display_prefix,
                deferred,
            } => {
                if corbanu_api_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    deferred.as_ref(),
                ) {
                    self.chat_widget.confirm_corbanu_api_key_revocation(
                        key_id,
                        display_prefix,
                        deferred,
                    );
                }
            }
            AppEvent::CorbanuApiOperationRequested {
                operation,
                deferred,
            } => {
                if corbanu_api_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    deferred.as_ref(),
                ) {
                    self.chat_widget
                        .request_corbanu_api_operation(operation, deferred);
                }
            }
            AppEvent::CorbanuApiOperationFinished { result, deferred } => {
                // A submitted wallet operation cannot be cancelled. Always surface its result so a
                // one-time generated key is stored and revealed instead of being lost. A stale
                // deferred continuation is stripped below, which prevents provider activation and
                // model selection.
                let was_deferred = deferred.is_some();
                let active_deferred = deferred.filter(|deferred| {
                    self.active_deferred_provider_setup.as_ref() == Some(deferred)
                });
                let ordinary_operation_is_current =
                    !was_deferred && self.active_deferred_provider_setup.is_none();
                let refresh_surface = ordinary_operation_is_current || active_deferred.is_some();
                let configured = self
                    .chat_widget
                    .on_corbanu_api_operation_finished(
                        result,
                        active_deferred.clone(),
                        refresh_surface,
                    );
                if configured && ordinary_operation_is_current {
                    // The credential write is synchronous. Activate the still-current ordinary
                    // operation, then rebuild the policy from durable storage before queueing its
                    // model selection. Deferred activation remains owned by the exact deferred
                    // continuation below; stale submitted completions only store/reveal their key.
                    let activated = if let Some(policy) = self.model_catalog.provider_policy() {
                        let host = policy.host();
                        if !host.activate(codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID) {
                            self.chat_widget.add_error_message(
                                "Corbanu API key was stored, but provider activation could not be persisted."
                                    .to_string(),
                            );
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    };
                    self.model_catalog.refresh_provider_policy();
                    if activated {
                        let model = "corbanu/glm-5.3-flash";
                        if self.model_catalog.provider_is_selectable(
                            codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID,
                            model,
                        ) {
                            self.chat_widget.select_corbanu_api_model(model);
                        } else {
                            self.chat_widget.add_error_message(
                                "Corbanu API key was stored, but provider status reconciliation is still incomplete."
                                    .to_string(),
                            );
                        }
                    }
                }
                if configured && let Some(deferred) = active_deferred {
                    self.app_event_tx
                        .send(AppEvent::DeferredCorbanuPlanConfigured { deferred });
                }
            }
            AppEvent::WalletCreateFinished { operation, result } => {
                if let Some(deferred) = self.pending_wallet_create_deferred.take() {
                    self.chat_widget
                        .on_deferred_wallet_create_finished(operation, result, deferred);
                } else {
                    self.chat_widget
                        .on_wallet_create_finished(operation, result);
                }
            }
            AppEvent::WalletUnlockFinished {
                policy,
                continuation,
                result,
            } => {
                if !wallet_unlock_continuation_is_current(
                    self.active_deferred_provider_setup.as_ref(),
                    &continuation,
                ) {
                    return Ok(AppRunControl::Continue);
                }
                self.chat_widget
                    .on_wallet_unlock_finished(policy, continuation, result);
            }
            AppEvent::OpenWalletPlans { mode } => {
                self.chat_widget.open_wallet_plans(mode);
            }
            AppEvent::BeginDeferredCorbanuPlan { deferred } => {
                if self.active_deferred_provider_setup.as_ref() != Some(&deferred) {
                    return Ok(AppRunControl::Continue);
                }
                if self.chat_widget.has_wallet_signing_capability() {
                    self.chat_widget.open_corbanu_api_for_deferred(deferred);
                } else {
                    let home = self.config.codex_home.to_path_buf();
                    let tx = self.app_event_tx.clone();
                    tokio::spawn(async move {
                        let result = codex_wallet_daemon::WalletDaemonClient::new(home)
                            .status()
                            .await
                            .map(|status| {
                                if !status.wallet_exists {
                                    crate::app_event::DeferredWalletPreflight::Missing
                                } else if status.locked {
                                    crate::app_event::DeferredWalletPreflight::Locked
                                } else {
                                    crate::app_event::DeferredWalletPreflight::Unlocked
                                }
                            })
                            .map_err(|error| error.to_string());
                        tx.send(AppEvent::DeferredWalletPreflightFinished { deferred, result });
                    });
                }
            }
            AppEvent::DeferredWalletPreflightFinished { deferred, result } => {
                if self.active_deferred_provider_setup.as_ref() != Some(&deferred) {
                    return Ok(AppRunControl::Continue);
                }
                match result {
                    Ok(crate::app_event::DeferredWalletPreflight::Missing) => {
                        self.pending_wallet_create_deferred = Some(deferred.clone());
                        self.chat_widget.open_deferred_wallet_create(deferred);
                    }
                    Ok(crate::app_event::DeferredWalletPreflight::Locked)
                    | Ok(crate::app_event::DeferredWalletPreflight::Unlocked) => {
                        self.app_event_tx.send(AppEvent::OpenWalletUnlock {
                            policy: codex_wallet_daemon::UnlockPolicy::Timed {
                                duration_seconds: 300,
                            },
                            continuation:
                                crate::app_event::WalletUnlockContinuation::OpenCorbanuApi {
                                    deferred: Some(deferred),
                                },
                        });
                    }
                    Err(error) => {
                        self.chat_widget.add_error_message(format!(
                            "Wallet status could not be checked: {error}"
                        ));
                        self.chat_widget.show_deferred_wallet_preflight_retry(deferred);
                    }
                }
            }
            AppEvent::WalletPlansReady { mode, result } => {
                self.chat_widget.on_wallet_plans_ready(mode, result);
            }
            AppEvent::ConfirmWalletPlanPurchase { plan } => {
                self.chat_widget.confirm_wallet_plan_purchase(plan);
            }
            AppEvent::WalletPlanPurchaseRequested { plan } => {
                self.chat_widget.purchase_wallet_plan(plan);
            }
            AppEvent::WalletPlanProvisioned {
                operation,
                result,
                deferred_setup,
            } => {
                self.chat_widget
                    .on_wallet_plan_provisioned(operation, result, deferred_setup);
            }
            AppEvent::WalletPlanCredentialPersistenceFinished {
                attempt_id,
                operation,
                plan_id,
                purchase,
                deferred_setup,
                api_key,
                result,
            } => {
                self.chat_widget
                    .on_wallet_plan_credential_persistence_finished(
                        attempt_id,
                        operation,
                        plan_id,
                        purchase,
                        deferred_setup,
                        api_key,
                        result,
                    );
            }
            AppEvent::DeferredCorbanuPlanCancelled { deferred } => {
                if self.active_deferred_provider_setup.as_ref() == Some(&deferred) {
                    self.chat_widget.dismiss_deferred_wallet_plan_views();
                    self.pending_wallet_create_deferred = None;
                    self.active_deferred_provider_setup = None;
                    if let Some(session) = self.shared_provider_setup_session.as_mut() {
                        session.dispatch(
                            crate::onboarding::provider_setup::ProviderSetupAction::DeferredPlanCancelled {
                                continuation_id: deferred.continuation_id(),
                            },
                        );
                        if deferred.has_usable_fallback() {
                            self.shared_provider_setup_session = None;
                            self.shared_provider_status_host = None;
                        } else {
                            self.render_shared_provider_setup();
                        }
                    } else if !deferred.has_usable_fallback() {
                        self.app_event_tx.send(AppEvent::OpenSharedProviderSetup);
                    }
                }
            }
            AppEvent::DeferredCorbanuPlanConfigured { deferred } => {
                if self.active_deferred_provider_setup.as_ref() == Some(&deferred)
                    && self
                        .chat_widget
                        .on_deferred_corbanu_plan_configured(&deferred)
                        .is_ok()
                    {
                        // The wallet adapter queued an exact provider selection. Refresh the
                        // shared policy before that event is handled so the newly stored and
                        // activated plan is selectable in this process.
                        self.model_catalog.refresh_provider_policy();
                        if let Some(session) = self.shared_provider_setup_session.as_mut() {
                            session.dispatch(
                                crate::onboarding::provider_setup::ProviderSetupAction::DeferredPlanConfigured {
                                    continuation_id: deferred.continuation_id(),
                                },
                            );
                        }
                        self.chat_widget.dismiss_deferred_wallet_plan_views();
                        self.active_deferred_provider_setup = None;
                        self.shared_provider_setup_session = None;
                        self.shared_provider_status_host = None;
                        self.shared_provider_account_auth_host = None;
                    }
            }
            AppEvent::WalletPlanReceiptReady {
                attempt_id,
                selection_policy,
                receipt,
            } => {
                self.chat_widget.on_wallet_plan_receipt_reconciled(
                    attempt_id,
                    selection_policy,
                    receipt,
                );
            }
            AppEvent::OpenWalletPlanReceipt { receipt } => {
                self.chat_widget.open_wallet_plan_receipt(receipt);
            }
            AppEvent::CloseWalletPlanReceipt => {
                self.chat_widget.close_wallet_plan_receipt();
            }
            AppEvent::WalletRecoverPlanRequested => {
                self.chat_widget.recover_wallet_plan_access();
            }
            AppEvent::OpenVaultCredentialsList => {
                self.chat_widget.open_vault_credentials_list();
            }
            AppEvent::OpenVaultCredentialActions { label } => {
                self.chat_widget.open_vault_credential_actions(label);
            }
            AppEvent::OpenVaultCopySecret { label } => {
                self.chat_widget.copy_vault_secret_to_clipboard(label);
            }
            AppEvent::OpenVaultRevealSecret { label } => {
                self.chat_widget.reveal_vault_secret(label);
            }
            AppEvent::VaultRevealSecretFinished { label, result } => {
                self.chat_widget
                    .on_vault_reveal_secret_finished(label, result);
            }
            AppEvent::OpenVaultReplaceSecret { label } => {
                self.chat_widget.open_vault_replace_secret(label);
            }
            AppEvent::VaultCredentialReplaceRequested { label, secret } => {
                self.chat_widget.replace_vault_credential(label, secret);
            }
            AppEvent::VaultCredentialReplaced { label, result } => {
                self.chat_widget.on_vault_credential_replaced(label, result);
            }
            AppEvent::ConfirmVaultCredentialDelete { label } => {
                self.chat_widget.confirm_vault_credential_delete(label);
            }
            AppEvent::VaultCredentialDeleteRequested { label } => {
                self.chat_widget.delete_vault_credential(label);
            }
            AppEvent::VaultCredentialDeleted { label, result } => {
                self.chat_widget.on_vault_credential_deleted(label, result);
                // #1193/#1251: the deleted entry may have been a named
                // account or a named sign-in.
                self.check_named_accounts();
            }
            AppEvent::VaultMenuCredentialsReady { result } => {
                self.chat_widget.on_vault_menu_credentials_ready(result);
            }
            AppEvent::VaultCredentialsReady { result } => {
                self.chat_widget.on_vault_credentials_ready(result);
            }
            AppEvent::VaultCopySecretFinished { label, result } => {
                self.chat_widget
                    .on_vault_copy_secret_finished(label, result);
            }
            AppEvent::CampaignTrackerDocument { text } => {
                let _ = tui.enter_alt_screen();
                let lines = text
                    .lines()
                    .map(|line| ratatui::text::Line::from(line.to_string()))
                    .collect();
                self.overlay = Some(Overlay::new_static_with_lines(
                    lines,
                    "Campaign Tracker · read-only replay".to_string(),
                    self.keymap.pager.clone(),
                ));
                tui.frame_requester().schedule_frame();
            }
            AppEvent::CampaignTrackerTick => {
                self.chat_widget.campaign_tracker_sync();
            }
            AppEvent::CampaignTrackerOpen { path, body } => {
                self.chat_widget.open_campaign_tracker(path, body);
            }
            AppEvent::CampaignTrackerResult {
                path,
                enrollment,
                body,
                result,
            } => {
                self.chat_widget
                    .campaign_tracker_result(path, enrollment, body, result);
            }
            AppEvent::CampaignTrackerSync { identity, result } => {
                self.chat_widget
                    .campaign_tracker_sync_result(identity, result);
            }
            AppEvent::OpenTaskNodeMenu => {
                self.chat_widget.open_tasknode_menu();
            }
            AppEvent::TaskNodeMenuStatusResult { result } => {
                self.chat_widget.handle_tasknode_menu_status_result(result);
            }
            AppEvent::TaskNodeMenuRequestsResult { result } => {
                self.chat_widget
                    .handle_tasknode_menu_requests_result(result);
            }
            AppEvent::TaskNodeMenuPoll { generation } => {
                self.chat_widget.handle_tasknode_menu_poll(generation);
            }
            AppEvent::OpenTaskNodeLink => {
                self.chat_widget.open_tasknode_link();
            }
            AppEvent::TaskNodeLinkResult { result } => {
                self.chat_widget.handle_tasknode_link_result(result);
            }
            AppEvent::OpenTaskNodeStatus => {
                self.chat_widget.open_tasknode_status();
            }
            AppEvent::TaskNodeStatusResult { result } => {
                self.chat_widget.handle_tasknode_status_result(result);
            }
            AppEvent::OpenTaskNodeTaskList { tab } => {
                self.chat_widget.open_tasknode_task_list(tab);
            }
            AppEvent::TaskNodeTaskListResult { tab, result } => {
                self.chat_widget
                    .handle_tasknode_task_list_result(tab, result);
            }
            AppEvent::OpenTaskNodeTaskActions { task_id } => {
                self.chat_widget.open_tasknode_task_actions(task_id);
            }
            AppEvent::TaskNodeTaskActionsResult { task_id, result } => {
                self.chat_widget
                    .handle_tasknode_task_actions_result(task_id, result);
            }
            AppEvent::CopyTaskNodeTaskBrief { task_id } => {
                self.chat_widget.copy_tasknode_task_brief(task_id);
            }
            AppEvent::CopyTaskNodeTaskBriefResult { task_id, result } => {
                self.chat_widget
                    .handle_copy_tasknode_task_brief_result(task_id, result);
            }
            AppEvent::SubmitTaskNodeTaskAction { task_id, action } => {
                self.chat_widget
                    .submit_tasknode_task_action(task_id, action);
            }
            AppEvent::SubmitTaskNodeTaskActionResult { action, result } => {
                self.chat_widget
                    .handle_submit_tasknode_task_action_result(action, result);
            }
            AppEvent::OpenTaskNodeEvidencePrompt { task_id } => {
                self.chat_widget.open_tasknode_evidence_prompt(task_id);
            }
            AppEvent::OpenTaskNodeEvidencePromptResult { task_id, result } => {
                self.chat_widget
                    .handle_open_tasknode_evidence_prompt_result(task_id, result);
            }
            AppEvent::SubmitTaskNodeEvidence { task_id, summary } => {
                self.chat_widget.submit_tasknode_evidence(task_id, summary);
            }
            AppEvent::SubmitTaskNodeEvidenceResult { result } => {
                self.chat_widget
                    .handle_submit_tasknode_evidence_result(result);
            }
            AppEvent::OpenTaskNodeTaskRequestPrompt => {
                self.chat_widget.open_tasknode_task_request_prompt();
            }
            AppEvent::SubmitTaskNodeTaskRequest { detail } => {
                self.chat_widget.submit_tasknode_task_request(detail);
            }
            AppEvent::SubmitTaskNodeTaskRequestResult { result } => {
                self.chat_widget
                    .handle_submit_tasknode_task_request_result(result);
            }
            AppEvent::TaskNodeTeamContextDocument { text } => {
                let _ = tui.enter_alt_screen();
                let lines = text.lines().flat_map(|line| {
                    textwrap::wrap(line, 100).into_iter().map(|line| ratatui::text::Line::from(line.into_owned()))
                }).collect();
                self.overlay = Some(Overlay::new_static_with_lines(
                    lines,
                    "Task Node Team Context · read only".to_string(),
                    self.keymap.pager.clone(),
                ));
                tui.frame_requester().schedule_frame();
            }
            AppEvent::OpenTaskNodeTeamContext => {
                self.chat_widget.open_tasknode_team_context();
            }
            AppEvent::OpenTaskNodeTeamContextResult { result } => {
                self.chat_widget.handle_tasknode_team_context_result(result);
            }
            AppEvent::OpenTaskNodeContext => {
                self.chat_widget.open_tasknode_context();
            }
            AppEvent::OpenTaskNodeContextResult { result } => {
                self.chat_widget.handle_open_tasknode_context_result(result);
            }
            AppEvent::OpenTaskNodeContextEdit {
                title,
                body,
                revision,
                body_format,
            } => {
                self.chat_widget
                    .open_tasknode_context_edit(title, body, revision, body_format);
            }
            AppEvent::SubmitTaskNodeContextEdit {
                title,
                body,
                revision,
            } => {
                self.chat_widget
                    .submit_tasknode_context_edit(title, body, revision);
            }
            AppEvent::SubmitTaskNodeContextEditResult { result } => {
                self.chat_widget
                    .handle_submit_tasknode_context_edit_result(result);
            }
            AppEvent::OpenTaskNodeRequestPage { cursor } => {
                self.chat_widget.open_tasknode_request_page(cursor)
            }
            AppEvent::RetryTaskNodeRequest {
                request_id,
                attempt,
            } => {
                self.chat_widget.retry_tasknode_request(request_id, attempt);
            }
            AppEvent::RetryTaskNodeRequestResult { result } => {
                self.chat_widget
                    .handle_retry_tasknode_request_result(result);
            }
            AppEvent::OpenTaskNodeRequestList => {
                self.chat_widget.open_tasknode_request_list();
            }
            AppEvent::OpenTaskNodeRequestListResult { result } => {
                self.chat_widget
                    .handle_open_tasknode_request_list_result(result);
            }
            AppEvent::OpenTaskNodeBalance => {
                self.chat_widget.open_tasknode_balance();
            }
            AppEvent::OpenTaskNodeBalanceResult { result } => {
                self.chat_widget.handle_open_tasknode_balance_result(result);
            }
            AppEvent::OpenTaskNodeRewards => {
                self.chat_widget.open_tasknode_rewards();
            }
            AppEvent::OpenTaskNodeRewardsResult { result } => {
                self.chat_widget.handle_open_tasknode_rewards_result(result);
            }
            AppEvent::OpenTaskNodeChat => {
                self.chat_widget.open_tasknode_chat();
            }
            AppEvent::OpenTaskNodeChatConversationsResult { result } => {
                self.chat_widget
                    .handle_open_tasknode_chat_conversations_result(result);
            }
            AppEvent::OpenTaskNodeChatHistory {
                conversation_id,
                title,
            } => {
                self.chat_widget
                    .open_tasknode_chat_history(conversation_id, title);
            }
            AppEvent::OpenTaskNodeChatHistoryResult {
                conversation_id,
                title,
                result,
            } => {
                self.chat_widget.handle_open_tasknode_chat_history_result(
                    conversation_id,
                    title,
                    result,
                );
            }
            AppEvent::OpenTaskNodeChatPrompt {
                conversation_id,
                title,
            } => {
                self.chat_widget
                    .open_tasknode_chat_prompt(conversation_id, title);
            }
            AppEvent::SubmitTaskNodeChat {
                conversation_id,
                title,
                message,
            } => {
                self.chat_widget
                    .submit_tasknode_chat(conversation_id, title, message);
            }
            AppEvent::TaskNodeChatStreamDelta {
                stream_id,
                conversation_id,
                title,
                text,
            } => {
                self.chat_widget.handle_tasknode_chat_stream_delta(
                    stream_id,
                    conversation_id,
                    title,
                    text,
                );
            }
            AppEvent::TaskNodeChatStreamDone {
                stream_id,
                conversation_id,
                title,
                result,
            } => {
                self.chat_widget.handle_tasknode_chat_stream_done(
                    stream_id,
                    conversation_id,
                    title,
                    result,
                );
            }
            AppEvent::LogoutTaskNode => {
                self.chat_widget.logout_tasknode();
            }
            AppEvent::TaskNodeLogoutResult { result } => {
                self.chat_widget.handle_tasknode_logout_result(result);
            }
            AppEvent::OpenFullAccessConfirmation {
                preset,
                return_to_permissions,
                profile_selection,
            } => {
                self.chat_widget.open_full_access_confirmation(
                    preset,
                    return_to_permissions,
                    profile_selection,
                );
            }
            AppEvent::ApplyPermissionShortcut { thread_id, selection } => {
                self.apply_permission_shortcut(app_server, thread_id, selection).await;
            }
            AppEvent::OpenFeedbackNote {
                category,
                include_logs,
            } => {
                self.chat_widget.open_feedback_note(category, include_logs, self.feedback_audience);
            }
            AppEvent::OpenFeedbackConsent { category } => {
                self.chat_widget.open_feedback_consent(category);
            }
            AppEvent::SubmitFeedback {
                category,
                reason,
                turn_id,
                include_logs,
            } => {
                self.submit_feedback(app_server, category, reason, turn_id, include_logs);
            }
            AppEvent::FeedbackSubmitted {
                origin_thread_id,
                category,
                include_logs,
                result,
            } => {
                self.handle_feedback_submitted(origin_thread_id, category, include_logs, result)
                    .await;
            }
            AppEvent::LaunchExternalEditor => {
                if self.chat_widget.external_editor_state() == ExternalEditorState::Active {
                    self.launch_external_editor(tui).await;
                }
            }
            AppEvent::RefreshWindowsSandbox { thread_id } => {
                #[cfg(any(target_os = "windows", test))]
                self.refresh_windows_sandbox_for_thread(app_server, thread_id).await;
                #[cfg(not(any(target_os = "windows", test)))]
                let _ = thread_id;
            }
            AppEvent::OpenWindowsSandboxEnablePrompt {
                preset,
                profile_selection,
            } => {
                self.chat_widget
                    .open_windows_sandbox_enable_prompt(preset, profile_selection);
            }
            AppEvent::OpenWindowsSandboxFallbackPrompt {
                preset,
                profile_selection,
            } => {
                #[cfg(any(target_os = "windows", test))]
                {
                    self.windows_sandbox.pending_setup = None;
                }
                self.session_telemetry.counter(
                    "codex.windows_sandbox.fallback_prompt_shown",
                    /*inc*/ 1,
                    &[],
                );
                self.chat_widget.clear_windows_sandbox_setup_status();
                if let Some(started_at) = self.windows_sandbox.setup_started_at.take() {
                    self.session_telemetry.record_duration(
                        "codex.windows_sandbox.elevated_setup_duration_ms",
                        started_at.elapsed(),
                        &[("result", "failure")],
                    );
                }
                self.chat_widget
                    .open_windows_sandbox_fallback_prompt(preset, profile_selection);
            }
            AppEvent::BeginWindowsSandboxElevatedSetup {
                preset,
                profile_selection,
            } => {
                #[cfg(target_os = "windows")]
                self.begin_windows_sandbox_setup(
                    app_server,
                    preset,
                    profile_selection,
                    WindowsSandboxEnableMode::Elevated,
                )
                .await;
                #[cfg(not(target_os = "windows"))]
                let _ = (preset, profile_selection);
            }
            AppEvent::BeginWindowsSandboxLegacySetup {
                preset,
                profile_selection,
            } => {
                #[cfg(target_os = "windows")]
<<<<<<< HEAD
                {
                    let setup_permissions = match self
                        .windows_setup_permissions(&preset, profile_selection.as_ref())
                        .await
                    {
                        Ok(setup_permissions) => setup_permissions,
                        Err(err) => {
                            tracing::warn!(
                                error = %err,
                                "failed to resolve permission profile for legacy Windows sandbox setup"
                            );
                            self.chat_widget.add_error_message(format!(
                                "Failed to prepare Windows sandbox for the selected permission profile: {err}"
                            ));
                            return Ok(AppRunControl::Continue);
                        }
                    };
                    let permission_profile = setup_permissions.permission_profile;
                    let workspace_roots = setup_permissions.workspace_roots;
                    let command_cwd = self.config.cwd.clone();
                    let env_map: std::collections::HashMap<String, String> =
                        std::env::vars().collect();
                    let codex_home = self.config.codex_home.clone();
                    let tx = self.app_event_tx.clone();
                    let session_telemetry = self.session_telemetry.clone();

                    self.chat_widget.show_windows_sandbox_setup_status();
                    tokio::task::spawn_blocking(move || {
                        if let Err(err) =
                            codex_windows_sandbox::run_windows_sandbox_legacy_preflight(
                                &permission_profile,
                                workspace_roots.as_slice(),
                                codex_home.as_path(),
                                command_cwd.as_path(),
                                &env_map,
                            )
                        {
                            session_telemetry.counter(
                                "codex.windows_sandbox.legacy_setup_preflight_failed",
                                /*inc*/ 1,
                                &[],
                            );
                            tracing::warn!(
                                error = %format!("{err:#}"),
                                "failed to preflight non-admin Windows sandbox setup"
                            );
                        }
                        tx.send(AppEvent::EnableWindowsSandboxForAgentMode {
                            preset,
                            mode: WindowsSandboxEnableMode::Legacy,
                            profile_selection,
                        });
                    });
                }
||||||| 413492cd6c
                {
                    let setup_permissions = match self
                        .windows_setup_permissions(&preset, profile_selection.as_ref())
                        .await
                    {
                        Ok(setup_permissions) => setup_permissions,
                        Err(err) => {
                            tracing::warn!(
                                error = %err,
                                "failed to resolve permission profile for legacy Windows sandbox setup"
                            );
                            self.chat_widget.add_error_message(format!(
                                "Failed to prepare Windows sandbox for the selected permission profile: {err}"
                            ));
                            return Ok(AppRunControl::Continue);
                        }
                    };
                    let permission_profile = setup_permissions.permission_profile;
                    let workspace_roots = setup_permissions.workspace_roots;
                    let command_cwd = self.config.cwd.clone();
                    let env_map: std::collections::HashMap<String, String> =
                        std::env::vars().collect();
                    let codex_home = self.config.codex_home.clone();
                    let tx = self.app_event_tx.clone();
                    let session_telemetry = self.session_telemetry.clone();

                    self.chat_widget.show_windows_sandbox_setup_status();
                    tokio::task::spawn_blocking(move || {
                        if let Err(err) =
                            codex_windows_sandbox::run_windows_sandbox_legacy_preflight(
                                &permission_profile,
                                workspace_roots.as_slice(),
                                codex_home.as_path(),
                                command_cwd.as_path(),
                                &env_map,
                            )
                        {
                            session_telemetry.counter(
                                "codex.windows_sandbox.legacy_setup_preflight_failed",
                                /*inc*/ 1,
                                &[],
                            );
                            tracing::warn!(
                                error = %err,
                                "failed to preflight non-admin Windows sandbox setup"
                            );
                        }
                        tx.send(AppEvent::EnableWindowsSandboxForAgentMode {
                            preset,
                            mode: WindowsSandboxEnableMode::Legacy,
                            profile_selection,
                        });
                    });
                }
=======
                self.begin_windows_sandbox_setup(
                    app_server,
                    preset,
                    profile_selection,
                    WindowsSandboxEnableMode::Legacy,
                )
                .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                #[cfg(not(target_os = "windows"))]
                let _ = (preset, profile_selection);
            }
            AppEvent::EnableWindowsSandboxForAgentMode {
                preset,
                mode,
                profile_selection,
            } => {
                #[cfg(any(target_os = "windows", test))]
                {
                    self.windows_sandbox.pending_setup = None;
                }
                #[cfg(target_os = "windows")]
                {
                    self.chat_widget.clear_windows_sandbox_setup_status();
                    if let Some(started_at) = self.windows_sandbox.setup_started_at.take()
                        && mode == WindowsSandboxEnableMode::Elevated
                    {
                        self.session_telemetry.record_duration(
                            "codex.windows_sandbox.elevated_setup_duration_ms",
                            started_at.elapsed(),
                            &[("result", "success")],
                        );
                    }
                    let selected_mode = match mode {
                        WindowsSandboxEnableMode::Elevated => WindowsSandboxSetupMode::Elevated,
                        WindowsSandboxEnableMode::Legacy => WindowsSandboxSetupMode::Unelevated,
                    };
                    let elevated_enabled = selected_mode == WindowsSandboxSetupMode::Elevated;
                    if self
                        .verify_windows_sandbox_mode_after_setup(app_server, selected_mode)
                        .await
                    {
<<<<<<< HEAD
                        Ok(response) if response.status == WriteStatus::OkOverridden => {
                            self.sync_windows_sandbox_after_overridden_write(app_server, &response)
                                .await;
                        }
                        Ok(_) => {
                            if elevated_enabled {
                                self.config.set_windows_sandbox_enabled(/*value*/ false);
                                self.config
                                    .set_windows_elevated_sandbox_enabled(/*value*/ true);
                            } else {
                                self.config.set_windows_sandbox_enabled(/*value*/ true);
                                self.config
                                    .set_windows_elevated_sandbox_enabled(/*value*/ false);
                            }
                            self.chat_widget.set_windows_sandbox_mode(
                                self.config.permissions.windows_sandbox_mode,
                            );
                            let windows_sandbox_level =
                                crate::windows_sandbox::level_from_config(&self.config);
                            if let Some((sample_paths, extra_count, failed_scan)) =
                                self.chat_widget.world_writable_warning_details()
                            {
                                self.app_event_tx.send(AppEvent::CodexOp(
                                    AppCommand::override_turn_context(
                                        /*cwd*/ None,
                                        /*approval_policy*/ None,
                                        /*approvals_reviewer*/ None,
                                        /*permission_profile*/ None,
                                        /*active_permission_profile*/ None,
                                        #[cfg(target_os = "windows")]
                                        Some(windows_sandbox_level),
                                        /*model*/ None,
                                        /*effort*/ None,
                                        /*summary*/ None,
                                        /*service_tier*/ None,
                                        /*collaboration_mode*/ None,
                                        /*personality*/ None,
                                    ),
                                ));
                                self.app_event_tx.send(
                                    AppEvent::OpenWorldWritableWarningConfirmation {
                                        preset: Some(preset.clone()),
                                        profile_selection: profile_selection.clone(),
                                        sample_paths,
                                        extra_count,
                                        failed_scan,
                                    },
                                );
                            } else if let Some(selection) = profile_selection {
                                self.app_event_tx.send(AppEvent::CodexOp(
                                    AppCommand::override_turn_context(
                                        /*cwd*/ None,
                                        /*approval_policy*/ None,
                                        /*approvals_reviewer*/ None,
                                        /*permission_profile*/ None,
                                        /*active_permission_profile*/ None,
                                        #[cfg(target_os = "windows")]
                                        Some(windows_sandbox_level),
                                        /*model*/ None,
                                        /*effort*/ None,
                                        /*summary*/ None,
                                        /*service_tier*/ None,
                                        /*collaboration_mode*/ None,
                                        /*personality*/ None,
                                    ),
                                ));
                                if self.apply_permission_profile_selection(app_server, selection).await {
                                    self.chat_widget.submit_initial_user_message_if_pending();
||||||| 413492cd6c
                        Ok(response) if response.status == WriteStatus::OkOverridden => {
                            self.sync_windows_sandbox_after_overridden_write(app_server, &response)
                                .await;
                        }
                        Ok(_) => {
                            if elevated_enabled {
                                self.config.set_windows_sandbox_enabled(/*value*/ false);
                                self.config
                                    .set_windows_elevated_sandbox_enabled(/*value*/ true);
                            } else {
                                self.config.set_windows_sandbox_enabled(/*value*/ true);
                                self.config
                                    .set_windows_elevated_sandbox_enabled(/*value*/ false);
                            }
                            self.chat_widget.set_windows_sandbox_mode(
                                self.config.permissions.windows_sandbox_mode,
                            );
                            let windows_sandbox_level =
                                crate::windows_sandbox::level_from_config(&self.config);
                            if let Some((sample_paths, extra_count, failed_scan)) =
                                self.chat_widget.world_writable_warning_details()
                            {
                                self.app_event_tx.send(AppEvent::CodexOp(
                                    AppCommand::override_turn_context(
                                        /*cwd*/ None,
                                        /*approval_policy*/ None,
                                        /*approvals_reviewer*/ None,
                                        /*permission_profile*/ None,
                                        /*active_permission_profile*/ None,
                                        #[cfg(target_os = "windows")]
                                        Some(windows_sandbox_level),
                                        /*model*/ None,
                                        /*effort*/ None,
                                        /*summary*/ None,
                                        /*service_tier*/ None,
                                        /*collaboration_mode*/ None,
                                        /*personality*/ None,
                                    ),
                                ));
                                self.app_event_tx.send(
                                    AppEvent::OpenWorldWritableWarningConfirmation {
                                        preset: Some(preset.clone()),
                                        profile_selection: profile_selection.clone(),
                                        sample_paths,
                                        extra_count,
                                        failed_scan,
                                    },
                                );
                            } else if let Some(selection) = profile_selection {
                                self.app_event_tx.send(AppEvent::CodexOp(
                                    AppCommand::override_turn_context(
                                        /*cwd*/ None,
                                        /*approval_policy*/ None,
                                        /*approvals_reviewer*/ None,
                                        /*permission_profile*/ None,
                                        /*active_permission_profile*/ None,
                                        #[cfg(target_os = "windows")]
                                        Some(windows_sandbox_level),
                                        /*model*/ None,
                                        /*effort*/ None,
                                        /*summary*/ None,
                                        /*service_tier*/ None,
                                        /*collaboration_mode*/ None,
                                        /*personality*/ None,
                                    ),
                                ));
                                if self.apply_permission_profile_selection(selection).await {
                                    self.chat_widget.submit_initial_user_message_if_pending();
=======
                            self.chat_widget.windows_sandbox_elevated_setup_complete =
                                elevated_enabled;
                            if let Some(selection) = profile_selection {
                                let inline = self.chat_widget.complete_inline_permission_profile_selection();
                                self.select_permission_profile(app_server, selection).await;
                                if inline {
                                    self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                                }
                            } else {
                                self.app_event_tx.send(AppEvent::CodexOp(
                                    AppCommand::override_turn_context(
                                        /*cwd*/ None,
<<<<<<< HEAD
                                        /*approval_policy*/ None,
                                        /*approvals_reviewer*/ None,
                                        /*permission_profile*/ None,
                                        /*active_permission_profile*/ None,
                                        #[cfg(target_os = "windows")]
                                        Some(windows_sandbox_level),
||||||| 413492cd6c
                                        Some(AskForApproval::from(preset.approval)),
                                        Some(self.config.approvals_reviewer),
                                        Some(preset.permission_profile.clone()),
                                        Some(preset.active_permission_profile.clone()),
                                        #[cfg(target_os = "windows")]
                                        Some(windows_sandbox_level),
=======
                                        Some(AskForApproval::from(preset.approval)),
                                        Some(self.config.approvals_reviewer),
                                        Some(preset.permission_profile.clone()),
                                        Some(preset.active_permission_profile.clone()),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                                        /*model*/ None,
                                        /*effort*/ None,
                                        /*summary*/ None,
                                        /*service_tier*/ None,
                                        /*collaboration_mode*/ None,
                                    ),
                                ));
<<<<<<< HEAD
                                self.app_event_tx.send(AppEvent::SelectPermissionProfile(PermissionProfileSelection {
                                    profile_id: preset.active_permission_profile.id.clone(),
                                    approval_policy: Some(AskForApproval::from(preset.approval)),
                                    approvals_reviewer: Some(self.config.approvals_reviewer),
                                    display_label: preset.active_permission_profile.id.clone(),
                                }));
||||||| 413492cd6c
                                self.app_event_tx.send(AppEvent::UpdateAskForApprovalPolicy(
                                    AskForApproval::from(preset.approval),
                                ));
                                self.app_event_tx
                                    .send(AppEvent::UpdateActivePermissionProfile(
                                        preset.active_permission_profile.clone(),
                                    ));
=======
                                self.app_event_tx.send(AppEvent::UpdateAskForApprovalPolicy(
                                    AskForApproval::from(preset.approval),
                                ));
                                self.app_event_tx
                                    .send(AppEvent::UpdateActivePermissionProfile(
                                        preset.active_permission_profile.clone(),
                                    ));
                            }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                                self.chat_widget.add_plain_history_lines(vec![
                                    Line::from(vec!["• ".dim(), "Sandbox ready".into()]),
                                    Line::from(vec![
                                        "  ".into(),
                                        "Codex can now safely edit files and execute commands in your computer"
                                            .dark_gray(),
                                    ]),
                                ]);
<<<<<<< HEAD
                            }
                            if let Some(message) = codex_sandboxing::legacy_windows_sandbox_warning(
                                windows_sandbox_level,
                                &preset.permission_profile,
                            ) {
                                self.chat_widget
                                    .add_to_history(history_cell::new_warning_event(message));
                            }
                        }
                        Err(err) => {
                            tracing::error!(
                                error = %err,
                                "failed to enable Windows sandbox feature"
                            );
                            self.chat_widget.add_error_message(format!(
                                "Failed to enable the Windows sandbox feature: {err}"
                            ));
                        }
||||||| 413492cd6c
                            }
                        }
                        Err(err) => {
                            tracing::error!(
                                error = %err,
                                "failed to enable Windows sandbox feature"
                            );
                            self.chat_widget.add_error_message(format!(
                                "Failed to enable the Windows sandbox feature: {err}"
                            ));
                        }
=======
                    } else if self
                        .chat_widget
                        .recover_inline_permission_profile_selection()
                    {
                        self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
                    } else {
                        self.chat_widget
                            .retain_input_after_failed_permission_selection();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = (preset, mode, profile_selection);
                }
            }
<<<<<<< HEAD
            AppEvent::PersistModelSelection {
                model,
                provider,
                effort,
            } => {
                let selected_provider = provider
                    .as_deref()
                    .unwrap_or(self.config.model_provider_id.as_str());
                if !self
                    .model_catalog
                    .provider_is_selectable(selected_provider, &model)
                {
                    // Match the in-session update path above. A queued persistence event must not
                    // retain the snapshot from before its provider credential was stored.
                    self.model_catalog.refresh_provider_policy();
                    if !self
                        .model_catalog
                        .provider_is_selectable(selected_provider, &model)
                    {
                        self.chat_widget.add_error_message(format!(
                            "Cannot save `{model}` because provider `{selected_provider}` is inactive or unavailable."
                        ));
                        return Ok(AppRunControl::Continue);
                    }
                }
                if self.model_catalog.take_session_recovery_only() {
                    self.chat_widget.add_info_message(
                        "Provider recovery applied to this resumed session only.".to_string(),
                        Some("Use /model again to change the global default.".to_string()),
                    );
                    return Ok(AppRunControl::Continue);
                }
                match crate::config_update::write_config_batch(
                    app_server.request_handle(),
                    crate::config_update::build_model_selection_edits(
                        model.as_str(),
                        provider.as_deref(),
                        effort.as_ref(),
                    ),
                )
                .await
                {
                    Ok(_) => {
                        let effort_label = effort
                            .as_ref()
                            .map(std::string::ToString::to_string)
                            .unwrap_or_else(|| "default".to_string());
                        tracing::info!(
                            "Selected model: {model}, Selected provider: {:?}, Selected effort: {effort_label}",
                            provider
                        );
                        let model_label = self
                            .model_catalog
                            .try_list_models()
                            .ok()
                            .and_then(|models| {
                                models
                                    .into_iter()
                                    .find(|preset| preset.model == model)
                                    .map(|preset| preset.display_name)
                            })
                            .filter(|display_name| !display_name.trim().is_empty())
                            .unwrap_or_else(|| model.clone());
                        let provider_label = provider.as_deref().map(|provider_id| {
                            if provider_id
                                == codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID
                            {
                                "Corbanu API".to_string()
                            } else {
                                self.config
                                    .model_providers
                                    .get(provider_id)
                                    .map(|provider| provider.name.clone())
                                    .filter(|name| !name.trim().is_empty())
                                    .unwrap_or_else(|| provider_id.to_string())
                            }
                        });
                        // #653: a model served over several connections names the one chosen.
                        let route = provider.as_deref().and_then(|provider_id| {
                            codex_model_provider_info::model_route(&model, provider_id)
                        });
                        let mut message = if let Some(route) = route {
                            format!("Model changed to {}", route.model_label(&model_label))
                        } else if let Some(provider_label) = provider_label {
                            format!("Model changed to {model_label} via {provider_label}")
                        } else {
                            format!("Model changed to {model_label}")
                        };
                        if let Some(label) = Self::reasoning_label_for(&model, effort.as_ref()) {
                            message.push(' ');
                            message.push_str(&label);
                        }
                        self.chat_widget.add_info_message(message, /*hint*/ None);
                    }
                    Err(err) => {
                        let error = format_config_error(&err);
                        tracing::error!(
                            error = %error,
                            "failed to persist model selection"
                        );
                        self.chat_widget
                            .add_error_message(format!("Failed to save default model: {error}"));
                    }
||||||| 413492cd6c
            AppEvent::PersistModelSelection { model, effort } => {
                match crate::config_update::write_config_batch(
                    app_server.request_handle(),
                    crate::config_update::build_model_selection_edits(
                        model.as_str(),
                        effort.as_ref(),
                    ),
                )
                .await
                {
                    Ok(_) => {
                        let effort_label = effort
                            .as_ref()
                            .map(std::string::ToString::to_string)
                            .unwrap_or_else(|| "default".to_string());
                        tracing::info!("Selected model: {model}, Selected effort: {effort_label}");
                        let mut message = format!("Model changed to {model}");
                        if let Some(label) = Self::reasoning_label_for(&model, effort.as_ref()) {
                            message.push(' ');
                            message.push_str(&label);
                        }
                        self.chat_widget.add_info_message(message, /*hint*/ None);
                    }
                    Err(err) => {
                        let error = format_config_error(&err);
                        tracing::error!(
                            error = %error,
                            "failed to persist model selection"
                        );
                        self.chat_widget
                            .add_error_message(format!("Failed to save default model: {error}"));
                    }
=======
            AppEvent::PersistModelSelection { model, effort } => {
                self.persist_default_model_selection(app_server.request_handle(), model, effort)
                    .await;
            }
            AppEvent::PersistDaybreakSelection { thread_id, enabled } => {
                self.persist_daybreak_selection(app_server, thread_id, enabled).await;
            }
            AppEvent::SelectSessionModel { model, effort, service_tier } => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.select_session_model(app_server, model, effort, service_tier).await;
            }
            AppEvent::OpenQuickDialEditor { target, dial } => {
                self.chat_widget.open_quick_dial_editor(target, dial);
            }
            AppEvent::OpenQuickDialSavePicker(dial) => {
                self.chat_widget.open_quick_dial_save_picker(dial);
            }
            AppEvent::ConfirmQuickDial { slot, dial } => self.chat_widget.confirm_quick_dial(slot, dial),
            AppEvent::SaveQuickDial { slot, dial } => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.save_quick_dial(slot, dial).await;
            }
            AppEvent::RecallQuickDial { slot, thread_id } => {
                if self.chat_widget.thread_id() == thread_id {
                    self.chat_widget.recall_quick_dial(slot);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                }
            }
            AppEvent::CyberModelAutoReviewNotice => {
                self.chat_widget.add_warning_message(
                    "Cyber models default to \"Approve for me\" for safety reasons.".to_string(),
                );
            }
            AppEvent::PluginUninstallLoaded {
                cwd,
                plugin_id: _plugin_id,
                plugin_display_name,
                result,
            } => {
                let uninstall_succeeded = result.is_ok();
                if uninstall_succeeded {
                    self.refresh_plugin_mentions_after_config_write();
                }
                self.chat_widget.on_plugin_uninstall_loaded(
                    cwd.clone(),
                    plugin_display_name,
                    result,
                );
                if uninstall_succeeded
                    && self.chat_widget.config_ref().cwd.as_path() == cwd.as_path()
                {
                    self.fetch_plugins_list(app_server, cwd);
                }
            }
            AppEvent::RefreshPluginMentions => {
                self.refresh_plugin_mentions(app_server);
            }
            AppEvent::PluginMentionsLoaded { mut plugins, .. } => {
                if !self.config.features.enabled(Feature::Plugins) {
                    plugins = None;
                }
                self.chat_widget.on_plugin_mentions_loaded(plugins);
            }
            AppEvent::OpenRealtimeSettings => self.chat_widget.open_realtime_settings(),
            AppEvent::OpenRealtimeSoundDevices => self.chat_widget.open_realtime_sound_devices(),
            AppEvent::OpenRealtimeVoices => self.open_realtime_voices(app_server).await,
            AppEvent::OpenRealtimeDevicePicker { kind } => self.list_realtime_devices(kind),
            AppEvent::OpenRealtimeInputChannels { device } => self.chat_widget.open_realtime_input_channels(device),
            AppEvent::RealtimeDevicesListed { origin, kind, result } => {
                if origin == self.active_thread_id {
                    match result {
                        Ok(devices) => self.chat_widget.open_realtime_device_picker(kind, devices),
                        Err(error) => self.chat_widget.add_error_message(error),
                    }
                }
            }
            AppEvent::PersistRealtimeDevice { kind, name } => {
                self.persist_realtime_device(kind, name).await;
            }
            AppEvent::PersistRealtimeInputChannel { channel } => {
                self.persist_realtime_input_channel(channel).await;
            }
            AppEvent::PersistRealtimeVoiceSelection { voice } => {
                self.persist_realtime_voice(app_server, voice).await;
            }
            AppEvent::PersistServiceTierSelection { service_tier } => {
                self.refresh_status_line();
                self.refresh_session_header(tui)?;
                self.config.service_tier = service_tier.clone();
                self.sync_active_thread_service_tier_to_cached_session()
                    .await;
                let edits = crate::config_update::build_service_tier_selection_edits(
                    service_tier.as_deref(),
                );
                match self.persist_model_defaults(app_server.request_handle(), edits, "default service tier")
                    .await
                {
                    Ok(()) => {
                        let message = if let Some(service_tier) = service_tier {
                            format!("Service tier set to {service_tier}")
                        } else {
                            "Service tier cleared".to_string()
                        };
                        self.chat_widget.add_info_message(message, /*hint*/ None);
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "failed to persist service tier selection");
                        let error = format_config_error(&err);
                        self.chat_widget.add_error_message(format!(
                            "Failed to save default service tier: {error}\n\
                             You can continue this task. To save the default, resolve the error above, then switch to a different tier and back to the desired tier."
                        ));
                    }
                }
            }
            AppEvent::UpdateAskForApprovalPolicy(policy) => {
                if self.reject_pending_permission_change() {
                    return Ok(AppRunControl::Continue);
                }
                let mut config = self.config.clone();
                if !self.try_set_approval_policy_on_config(
                    &mut config,
                    policy,
                    "Failed to set approval policy",
                    "failed to set approval policy on app config",
                ) {
                    return Ok(AppRunControl::Continue);
                }
                self.config = config;
                let approval_policy =
                    AskForApproval::from(self.config.permissions.approval_policy.value());
                self.runtime_approval_policy_override =
                    Some(RuntimeApprovalPolicyOverride::Explicit(approval_policy));
                self.chat_widget.set_approval_policy(approval_policy);
                self.sync_active_thread_permission_settings_to_cached_session()
                    .await;
            }
            AppEvent::UpdateActivePermissionProfile(active_permission_profile) => {
                if self.reject_pending_permission_change() {
                    return Ok(AppRunControl::Continue);
                }
                let mut config = self.config.clone();
                let Some(permission_profile) = self
                    .try_set_builtin_active_permission_profile_on_config(
                        &mut config,
                        active_permission_profile.clone(),
                        "Failed to set permission profile",
                        "failed to set active permission profile on app config",
                    )
                else {
                    return Ok(AppRunControl::Continue);
                };
                let permission_profile_for_chat = permission_profile.clone();

                self.config = config;
                if let Err(err) = self
                    .chat_widget
                    .set_permission_profile_from_session_snapshot(
                        PermissionProfileSnapshot::active(
                            permission_profile_for_chat,
                            active_permission_profile,
                        ),
                    )
                {
                    tracing::warn!(%err, "failed to set permission profile on chat config");
                    self.chat_widget
                        .add_error_message(format!("Failed to set permission profile: {err}"));
                    return Ok(AppRunControl::Continue);
                }
                self.runtime_approvals_reviewer_override = Some(self.config.approvals_reviewer);
                self.runtime_permission_profile_override =
                    Some(RuntimePermissionProfileOverride::from_config(&self.config));
                self.sync_active_thread_permission_settings_to_cached_session()
                    .await;
                self.chat_widget.submit_initial_user_message_if_pending();
            }
            AppEvent::SelectPermissionProfile(selection) => {
<<<<<<< HEAD
                if self.apply_permission_profile_selection(app_server, selection).await {
                    self.chat_widget.submit_initial_user_message_if_pending();
                }
            }
            AppEvent::PermissionConfirmationCompleted { selection_id, result } => {
                // Completion also carries confirmed settings into fresh sessions and
                // submits or restores held initial input after checking correlation.
                if let Some(reviewer) = self.finish_permission_confirmation(selection_id, result)
                    && let Err(error) = crate::config_update::write_config_batch(
                        app_server.request_handle(),
                        vec![crate::config_update::replace_config_value("approvals_reviewer", serde_json::json!(reviewer.to_string()))],
                    ).await
                {
                    self.chat_widget.add_error_message(format!("Permissions applied, but the default reviewer could not be saved: {error}"));
                }
            }
            AppEvent::SelectPermissionPreset(selection) => {
                let reviewer = selection.approvals_reviewer;
                if let Some(thread_id) = self.active_thread_id {
                    self.request_permission_confirmation(app_server, codex_app_server_protocol::ThreadSettingsUpdateParams {
                        thread_id: thread_id.to_string(),
                        permissions: Some(selection.profile_id),
                        approval_policy: selection.approval_policy,
                        approvals_reviewer: reviewer.map(Into::into),
                        ..Default::default()
                    }, selection.display_label, reviewer);
                } else if self.apply_permission_profile_selection(app_server, selection).await {
                    if let Some(reviewer) = reviewer {
                        self.app_event_tx.send(AppEvent::UpdateApprovalsReviewer(reviewer));
                    }
                    self.chat_widget.submit_initial_user_message_if_pending();
||||||| 413492cd6c
                if self.apply_permission_profile_selection(selection).await {
                    self.chat_widget.submit_initial_user_message_if_pending();
=======
                let inline_selection = self
                    .chat_widget
                    .complete_inline_permission_profile_selection();
                self.select_permission_profile(app_server, selection).await;
                if inline_selection {
                    self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                }
            }
            AppEvent::UpdateApprovalsReviewer(policy) => {
                if self.reject_pending_permission_change() {
                    return Ok(AppRunControl::Continue);
                }
                self.runtime_approvals_reviewer_override = Some(policy);
                self.config.approvals_reviewer = policy;
                self.chat_widget.set_approvals_reviewer(policy);
                self.sync_active_thread_permission_settings_to_cached_session()
                    .await;
                if let Err(err) = crate::config_update::write_config_batch(
                    app_server.request_handle(),
                    vec![crate::config_update::replace_config_value(
                        "approvals_reviewer",
                        serde_json::json!(policy.to_string()),
                    )],
                )
                .await
                {
                    tracing::error!(
                        error = %err,
                        "failed to persist approvals reviewer update"
                    );
                    self.chat_widget.add_error_message(format!(
                        "Failed to save approvals reviewer: {}",
                        format_config_error(&err)
                    ));
                }
            }
            AppEvent::FetchExperimentalFeatures { thread_id, response_tx } => {
                self.fetch_experimental_features(app_server, thread_id, response_tx);
            }
            AppEvent::SaveExperimentalFeatures { thread_id, updates, response_tx } => {
                self.save_experimental_features(app_server, thread_id, updates, response_tx);
            }
            AppEvent::EnableFeatureForNewThreads(feature) => {
                self.enable_feature_for_new_threads(tui, app_server, feature)
                    .await;
            }
            AppEvent::UpdateFeatureFlags { updates } => {
                self.update_feature_flags(app_server, updates).await;
            }
            AppEvent::UpdateMemorySettings {
                use_memories,
                generate_memories,
            } => {
                self.update_memory_settings_with_app_server(
                    app_server,
                    use_memories,
                    generate_memories,
                )
                .await;
            }
            AppEvent::ResetMemories => {
                self.reset_memories_with_app_server(app_server).await;
            }
            AppEvent::UpdateRateLimitSwitchPromptHidden(hidden) => {
                self.chat_widget.set_rate_limit_switch_prompt_hidden(hidden);
            }
            AppEvent::UpdatePlanModeReasoningEffort(effort) => {
                self.on_update_plan_mode_reasoning_effort(effort);
                self.sync_active_thread_plan_mode_reasoning_setting(app_server)
                    .await;
            }
            AppEvent::PersistRateLimitSwitchPromptHidden => {
                self.local_settings.notices.hide_rate_limit_model_nudge = Some(true);
                if let Err(err) = ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
                    .set_hide_rate_limit_model_nudge(/*acknowledged*/ true)
                    .apply()
                    .await
                {
                    tracing::error!(
                        error = %err,
                        "failed to persist rate limit switch prompt preference"
                    );
                    self.chat_widget.add_error_message(format!(
                        "Failed to save rate limit reminder preference: {err}"
                    ));
                }
            }
            AppEvent::PersistPlanModeReasoningEffort(effort) => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                let key_path = "plan_mode_reasoning_effort";
                let edit = if let Some(effort) = effort {
                    crate::config_update::replace_config_value(
                        key_path,
                        serde_json::json!(effort.to_string()),
                    )
                } else {
                    crate::config_update::clear_config_value(key_path)
                };
                if let Err(err) = self.persist_model_defaults(
                    app_server.request_handle(),
                    vec![edit],
                    "Plan mode reasoning effort",
                )
                .await
                {
                    tracing::error!(
                        error = %err,
                        "failed to persist plan mode reasoning effort"
                    );
                    self.chat_widget.add_error_message(format!(
                        "Failed to save Plan mode reasoning effort: {err}"
                    ));
                }
            }
            AppEvent::PersistModelMigrationPromptAcknowledged {
                from_model,
                to_model,
            } => {
                if let Err(err) = ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
                    .record_model_migration_seen(from_model.as_str(), to_model.as_str())
                    .apply()
                    .await
                {
                    tracing::error!(
                        error = %err,
                        "failed to persist model migration prompt acknowledgement"
                    );
                    self.chat_widget.add_error_message(format!(
                        "Failed to save model migration prompt preference: {err}"
                    ));
                }
            }
            AppEvent::OpenAgentsOverview => self.open_agents_overview(app_server),
            AppEvent::ShowMoreAgentsOverview => self.show_more_agents_overview(app_server),
            AppEvent::RefreshViewedThread { backend_id, thread_id, revision } => {
                if app_server.backend_id() == backend_id {
                    app_server.frozen_receipt.take_if(|(id, rev)| *id == thread_id && *rev == revision);
                    self.agents_overview.refresh_thread_ids.insert(thread_id);
                    self.refresh_changed_agents_overview_threads(app_server);
                }
            }
            AppEvent::NewAgentsOverviewSession { cwd } => {
                return Box::pin(self.new_agents_overview_session(tui, app_server, cwd)).await;
            }
            AppEvent::AgentsOverviewThreadsLoaded { request_id, result } => {
                self.apply_agents_overview_thread_refresh(app_server, request_id, result);
            }
            AppEvent::SelectAgentsOverviewThread { thread_id } => {
                match self
                    .select_agents_overview_thread(tui, app_server, thread_id)
                    .await?
                {
                    AppRunControl::Continue
                        if self.primary_thread_id.is_none()
                            && self.chat_widget.selected_index_for_present_view(AGENTS_OVERVIEW_VIEW_ID).is_none() => {
                        self.open_agents_overview(app_server);
                    }
                    AppRunControl::Continue => {}
                    AppRunControl::Exit(reason) => return Ok(AppRunControl::Exit(reason)),
                }
            }
            AppEvent::ForkAgentsOverviewThread { thread_id } => {
                if self.reconnect.offline {
                    return Ok(AppRunControl::Continue);
                }
                if !self.side_threads.is_empty()
                    && self.current_displayed_thread_id() != Some(thread_id)
                {
                    self.add_agents_overview_error(
                        "Close the side conversation before forking another task.".into(),
                    );
                    return Ok(AppRunControl::Continue);
                }
                // Attachment replay must not submit queued input on the source task.
                self.chat_widget.fork_in_progress = true;
                let control = self.select_agents_overview_thread(tui, app_server, thread_id).await;
                if matches!(&control, Ok(AppRunControl::Continue))
                    && self.current_displayed_thread_id() == Some(thread_id)
                {
                    if let Some(input) = self.chat_widget.capture_thread_input_state() {
                        self.agents_overview.input_states.insert(thread_id, input);
                    }
                    self.app_event_tx.send(AppEvent::ForkAgentsOverviewThreadReady { thread_id });
                } else {
                    self.chat_widget.fork_in_progress = false;
                }
                return control;
            }
            AppEvent::ForkAgentsOverviewThreadReady { .. } => unreachable!("normalized above"),
            AppEvent::NewAgentsOverviewWorktree { cwd } => {
                Box::pin(self.new_agents_overview_worktree(tui, app_server, cwd)).await;
            }
            AppEvent::AgentsOverviewWorktreeCreated(result) => {
                self.pending_managed_worktree_creation = false;
                self.agents_overview.view_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).creating_worktree = false;
                match result {
                    Ok(mut pending) => {
                        let Some(checkout) = pending.checkout.take() else {
                            return Ok(AppRunControl::Continue);
                        };
                        let manager = pending.manager.clone();
                        let cwd = AbsolutePathBuf::try_from(checkout.cwd.clone())?;
                        return Box::pin(self.start_agents_overview_session(tui, app_server, Some(cwd), Some((manager, checkout)), /*startup_draft*/ None)).await;
                    }
                    Err(error) => self.add_agents_overview_error(error),
                }
            }
            AppEvent::RenameAgentsOverviewThread { thread_id, name } => {
                match app_server.thread_set_name(thread_id, name.clone()).await {
                    Ok(()) => {
                        self.chat_widget.expect_manual_thread_name(thread_id, name);
                        self.cancel_thread_title_generation(thread_id);
                    }
                    Err(error) => {
                        if let Ok(mut state) = self.agents_overview.view_state.lock() {
                            state.set_rename_input(&name, &self.keymap);
                            state.rename_target = Some(thread_id);
                        }
                        self.repaint_agents_overview();
                        self.add_agents_overview_error(format!("Failed to rename task: {error}"));
                    }
                }
            }
            AppEvent::ToggleAgentsOverviewPin { thread_id, pinned } => {
                self.toggle_agents_overview_pin(app_server, thread_id, pinned);
            }
            AppEvent::AgentsOverviewPinToggled {
                request_id,
                thread_id,
                pinned,
                result,
            } => {
                self.complete_agents_overview_pin(
                    app_server, request_id, thread_id, pinned, result,
                );
            }
            AppEvent::SuggestThreadName {
                thread_id,
                request_id,
            } => {
                self.suggest_thread_name(app_server, thread_id, request_id)
                    .await;
            }
            AppEvent::ThreadTitleStarted {
                cancellation,
                thread_id,
                destination,
                prompt,
                effort,
                result,
            } => {
                self.on_thread_title_started(
                    app_server,
                    thread_id,
                    destination,
                    prompt,
                    effort,
                    result,
                    cancellation,
                );
            }
            AppEvent::GeneratedThreadTitle {
                cancellation,
                thread_id,
                temporary_thread_id,
                destination,
                result,
            } => {
                self.temporary_structured_requests
                    .remove(&temporary_thread_id);

                if cancellation.is_cancelled() {
                    return Ok(AppRunControl::Continue);
                }
                self.finish_thread_title_generation(thread_id, destination);
                match destination {
                    ThreadTitleDestination::Automatic => {
                        if let Ok(response) = result
                            && let Some(title) = super::thread_title::parse_thread_title(&response)
                            && let Ok(thread) = app_server
                                .thread_read(thread_id, /*include_turns*/ false)
                                .await
                            && thread.name.is_none()
                        {
                            match app_server.thread_set_name(thread_id, title.clone()).await {
                                Ok(()) => self
                                    .chat_widget
                                    .on_thread_name_updated(thread_id, Some(title)),
                                Err(error) => {
                                    tracing::debug!(%error, "failed to apply generated thread title");
                                }
                            }
                        }
                    }
                    ThreadTitleDestination::RenameSuggestion { request_id } => {
                        let suggestion = result
                            .ok()
                            .and_then(|response| super::thread_title::parse_thread_title(&response));

                        self.chat_widget.apply_thread_name_suggestion(
                            thread_id,
                            request_id,
                            suggestion.as_deref(),
                        );
                    }
                }
            }
            AppEvent::HideAgentsOverviewThread { thread_id } => {
                self.prepare_agents_overview_removal(&HashSet::from([thread_id]));
                self.agents_overview.hidden_threads.insert(thread_id);
                self.repaint_agents_overview();
            }
            AppEvent::ConfirmAgentsOverviewAction { thread_id, action } => {
                self.confirm_agents_overview_action(thread_id, action);
            }
            AppEvent::RunAgentsOverviewAction { thread_id, action } => {
                self.run_agents_overview_action(tui, app_server, thread_id, action).await?;
            }
            AppEvent::StopAgentsOverviewThread { thread_id } => {
                self.stop_agents_overview_thread(app_server, thread_id)
                    .await;
            }
            AppEvent::OpenAgentPicker => {
                self.open_agent_picker(app_server).await;
            }
            AppEvent::AgentPickerThreadsLoaded {
                primary_thread_id,
                request_id,
                result,
            } => {
                self.apply_agent_picker_thread_refresh(
                    app_server,
                    primary_thread_id,
                    request_id,
                    result,
                );
            }
            AppEvent::SelectAgentThread(thread_id) => {
<<<<<<< HEAD
                self.save_active_claude_pane_transcript();
                let _ = self
                    .claude_panes
                    .set_active_user_pane(crate::claude_panes::CODEX_MAIN_PANE_ID);
                self.select_agent_thread_and_discard_side(tui, app_server, thread_id)
||||||| 413492cd6c
                self.select_agent_thread_and_discard_side(tui, app_server, thread_id)
=======
                self.select_agent_thread(tui, app_server, thread_id)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    .await?;
            }
            AppEvent::OpenPanePicker => {
                self.open_pane_picker(app_server).await;
            }
            AppEvent::OpenCodexPaneModelPicker => {
                self.open_codex_pane_model_picker();
            }
            AppEvent::OpenClaudePaneProfilePicker => {
                self.open_claude_pane_profile_picker();
            }
            AppEvent::OpenSpawnRolePicker => {
                self.open_spawn_role_picker();
            }
            AppEvent::OpenSpawnNazgulPanePicker => {
                self.open_spawn_nazgul_pane_picker();
            }
            AppEvent::OpenSpawnNazgulPicker => {
                self.open_spawn_nazgul_picker();
            }
            AppEvent::BindSpawnNazgulPane { pane_id } => {
                self.bind_spawn_nazgul_pane(pane_id);
            }
            AppEvent::OpenSpawnParentPicker { role } => {
                self.open_spawn_parent_picker(role);
            }
            AppEvent::OpenSpawnHarnessPicker {
                role,
                parent_node_id,
            } => {
                self.open_spawn_harness_picker(role, parent_node_id);
            }
            AppEvent::OpenSpawnModelPicker {
                role,
                parent_node_id,
            } => {
                self.open_spawn_model_picker(role, parent_node_id);
            }
            AppEvent::OpenSpawnClaudeProfilePicker {
                role,
                parent_node_id,
            } => {
                self.open_spawn_claude_profile_picker(role, parent_node_id);
            }
            AppEvent::CreateSpawnAgent {
                role,
                parent_node_id,
                agent_nickname,
                model,
                provider,
                effort,
            } => {
                let Some(agent_type) = role.agent_type() else {
                    self.chat_widget.add_error_message(
                        "Nazgul is a pane binding, not a spawned worker.".to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                if role == crate::spawn_orchestration::SpawnRole::Nazgul
                    && let Err(err) = self.preflight_new_custom_spawn_root()
                {
                    self.chat_widget.add_error_message(err.to_string());
                    return Ok(AppRunControl::Continue);
                }
                let Some(parent_thread_id) =
                    self.backend_parent_thread_for_spawn(role, parent_node_id.as_deref())
                else {
                    self.chat_widget.add_error_message(
                        "Cannot spawn a native agent before Corbanu Terminal Main has started."
                            .to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                if role == crate::spawn_orchestration::SpawnRole::Troll {
                    self.persist_bound_nazgul_root_thread_metadata().await;
                }
                let logical_parent_node_id =
                    self.logical_parent_node_for_spawn(role, parent_node_id.as_deref());
                let agent_nickname =
                    agent_nickname.or_else(|| self.next_spawn_agent_nickname(role));
                if let Err(err) = self
                    .ensure_native_spawn_provider_ready(provider.as_deref())
                    .await
                {
                    self.chat_widget.add_error_message(err.to_string());
                    return Ok(AppRunControl::Continue);
                }
                let spawn_config = match self.native_spawn_agent_config() {
                    Ok(config) => config,
                    Err(err) => {
                        self.chat_widget.add_error_message(err.to_string());
                        return Ok(AppRunControl::Continue);
                    }
                };
                let mut started_crew_root = false;
                let (agent_class, prepared_root_crew_id) = if role
                    == crate::spawn_orchestration::SpawnRole::Nazgul
                {
                    let runtime = crate::dispatch_queue::SavedNativeSpawnRuntime {
                        model: model.clone(),
                        provider: provider
                            .clone()
                            .unwrap_or_else(|| spawn_config.model_provider_id.clone()),
                        reasoning_effort: effort.clone(),
                    };
                    let display_name = agent_nickname
                        .clone()
                        .unwrap_or_else(|| role.label().to_string());
                    match self.prepare_custom_spawn_root(display_name, runtime) {
                        Ok(agent_class) => {
                            let crew_id = match &agent_class {
                                codex_protocol::crew::AgentClass::CrewMember {
                                    crew_id, ..
                                } => Some(crew_id.clone()),
                                codex_protocol::crew::AgentClass::EphemeralTask { .. } => None,
                            };
                            (agent_class, crew_id)
                        }
                        Err(err) => {
                            self.chat_widget.add_error_message(err.to_string());
                            return Ok(AppRunControl::Continue);
                        }
                    }
                } else {
                    match self.custom_spawn_member_agent_class_under(role, &logical_parent_node_id) {
                        Ok((agent_class, started)) => {
                            started_crew_root = started;
                            (agent_class, None)
                        }
                        Err(err) => {
                            self.chat_widget.add_error_message(err.to_string());
                            return Ok(AppRunControl::Continue);
                        }
                    }
                };
                match app_server
                    .spawn_agent_thread_with_class(
                        &spawn_config,
                        parent_thread_id,
                        agent_type.to_string(),
                        agent_nickname.clone(),
                        agent_class,
                        model.clone(),
                        provider.clone(),
                        effort.clone(),
                        /*base_instructions*/ None,
                    )
                    .await
                {
                    Ok(started) => {
                        let thread_id = started.session.thread_id;
                        self.register_spawn_agent_pane(
                            thread_id,
                            parent_thread_id,
                            logical_parent_node_id.clone(),
                            agent_nickname.clone(),
                            agent_type,
                            started,
                            /*persist_layout*/ true,
                        )
                        .await;
                        // When spawning a Nazgul pane, bind it as the visible root so subsequent
                        // Troll spawns and "Nazgul" dispatches route to this pane.
                        let bound_as_nazgul = role == crate::spawn_orchestration::SpawnRole::Nazgul;
                        if bound_as_nazgul {
                            self.set_spawn_nazgul_pane_binding(
                                crate::spawn_orchestration::thread_node_id(thread_id),
                            );
                            self.persist_bound_nazgul_root_thread_metadata().await;
                        }
                        let logical_node_id = crate::spawn_orchestration::thread_node_id(thread_id);
                        let crew_result = if bound_as_nazgul {
                            self.ensure_custom_spawn_root(&logical_node_id)
                        } else {
                            let runtime = self
                                .spawn_native_runtime_by_node
                                .get(&logical_node_id)
                                .cloned()
                                .ok_or_else(|| {
                                    color_eyre::eyre::eyre!(
                                        "spawned pane {logical_node_id} has no persisted runtime"
                                    )
                                });
                            runtime.and_then(|runtime| {
                                self.record_custom_spawn_member(
                                    &logical_node_id,
                                    &logical_parent_node_id,
                                    role,
                                    agent_nickname
                                        .clone()
                                        .unwrap_or_else(|| role.label().to_string()),
                                    runtime,
                                )
                            })
                        };
                        if let Err(err) = crew_result {
                            self.mark_crew_incomplete(err.to_string());
                            self.chat_widget.add_error_message(format!(
                                "Spawned the pane, but could not persist its crew identity: {err}"
                            ));
                            return Ok(AppRunControl::Continue);
                        }
                        self.persist_pane_state();
                        if self.active_thread_id.is_none() {
                            self.select_agent_thread_and_discard_side(tui, app_server, thread_id)
                                .await?;
                        }
                        let binding_suffix = if bound_as_nazgul {
                            " and bound it as the Nazgul root"
                        } else {
                            ""
                        };
                        self.chat_widget.add_info_message(
                            format!(
                                "Spawned Corbanu Terminal {} pane{}{binding_suffix}.",
                                role.label(),
                                agent_nickname
                                    .as_deref()
                                    .map(|nickname| format!(" {nickname}"))
                                    .unwrap_or_default()
                            ),
                            Some(format!("{model}; no task was started.")),
                        );
                    }
                    Err(err) => {
                        if let Some(crew_id) = prepared_root_crew_id.as_deref() {
                            self.abort_prepared_custom_spawn_root(crew_id);
                        }
                        if started_crew_root {
                            self.abort_started_custom_spawn_root();
                        }
                        tracing::error!(
                            error = ?err,
                            error_chain = %format!("{err:#}"),
                            role = role.label(),
                            "thread/spawnAgent retry limit reached; keeping the TUI alive"
                        );
                        self.chat_widget.add_error_message(format!(
                            "Failed to spawn Corbanu Terminal {} pane: {err:#}",
                            role.label()
                        ));
                    }
                }
            }
            AppEvent::CreateSpawnStandardCrew => {
                match self.create_spawn_standard_crew(app_server).await {
                    Ok((nazgul_thread_id, troll_thread_id)) => {
                        self.open_spawn_status();
                        self.chat_widget.add_info_message(
                            "Created standard crew: Nazgul + Troll + 3 Orcs.".to_string(),
                            Some(format!(
                                "Nazgul: {nazgul_thread_id}. Troll: {troll_thread_id}. No task was started. Send work explicitly from /spawn status or by dispatch block."
                            )),
                        );
                    }
                    Err(err) => {
                        tracing::error!(
                            error = ?err,
                            error_chain = %format!("{err:#}"),
                            "standard crew spawn failed; keeping all live panes available"
                        );
                        self.chat_widget
                            .add_error_message(format!("Failed to create standard crew: {err:#}"));
                    }
                }
            }
            AppEvent::CreateSpawnCorbanuApiCrew => {
                match self.create_spawn_corbanu_api_crew(app_server).await {
                    Ok((nazgul_thread_id, troll_thread_id)) => {
                        self.open_spawn_status();
                        self.chat_widget.add_info_message(
                            "Created Corbanu API crew: Kimi K3 Nazgul + Luna Troll + 3 Flash Orcs."
                                .to_string(),
                            Some(format!(
                                "Nazgul: {nazgul_thread_id}. Troll: {troll_thread_id}. No task was started. Send work explicitly from /spawn status or by dispatch block."
                            )),
                        );
                    }
                    Err(err) => {
                        tracing::error!(
                            error = ?err,
                            error_chain = %format!("{err:#}"),
                            "Corbanu API crew spawn failed; keeping all live panes available"
                        );
                        self.chat_widget.add_error_message(format!(
                            "Failed to create Corbanu API crew: {err:#}"
                        ));
                    }
                }
            }
            AppEvent::OpenSpawnAgentTaskPrompt { thread_id } => {
                self.open_spawn_agent_task_prompt(thread_id);
            }
            AppEvent::OpenSpawnClaudePaneTaskPrompt { pane_id } => {
                self.open_spawn_claude_pane_task_prompt(pane_id);
            }
            AppEvent::SubmitSpawnAgentTask {
                thread_id,
                task,
                sent_while_kill_switch_on,
            } => {
                // D106 (#1176): a pane the person opened takes agents' tasks
                // from its mailbox too; the /spawn crew's state does not stop
                // them, as it did not when they arrived as its input.
                let is_operator_pane = self
                    .agent_navigation
                    .get(&thread_id)
                    .is_some_and(|entry| self.is_operator_owned_codex_user_pane(thread_id, entry));
                if !is_operator_pane
                    && (self.spawn_legacy_read_only
                        || self.spawn_crew.as_ref().is_some_and(|crew| {
                            !matches!(crew.status, crate::crew_state::CrewCreationStatus::Ready)
                        }))
                {
                    let target_node_id = self.logical_native_node_for_thread(thread_id);
                    let acks = self.take_spawn_dispatch_acks_for_task(&target_node_id, task.trim());
                    self.release_spawn_dispatch_origins(&acks);
                    // #660: a task sent while the switch was on starts no turn
                    // in its sender, also when it fails after switch-off.
                    self.record_spawn_dispatch_acks_with_turns(
                        &acks,
                        "failed",
                        "crew identity or creation state is not reconciled",
                        /*notify*/ true,
                        /*start_turns*/ !sent_while_kill_switch_on,
                    );
                    self.chat_widget.add_error_message(
                        "This /spawn hierarchy is read-only until its crew identity and creation \
                         state are reconciled."
                            .to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                }
                let task = task.trim().to_string();
                if task.is_empty() {
                    // The sender is told, as before for a pane the person opened.
                    let target_node_id = self.logical_native_node_for_thread(thread_id);
                    self.record_spawn_dispatch_failed_for_task_with_turns(
                        &target_node_id,
                        &task,
                        "spawn task cannot be empty",
                        /*start_turns*/ !sent_while_kill_switch_on,
                    );
                    self.chat_widget
                        .add_error_message("Spawn task cannot be empty.".to_string());
                    return Ok(AppRunControl::Continue);
                }
                let task = self.spawn_agent_task_for_submission(thread_id, &task);
                let label = self.thread_label(thread_id);
                let target_node_id = self.logical_native_node_for_thread(thread_id);
                // The TUI does not queue native work. It consumes any edge-adapter acknowledgement
                // metadata, derives one stable identity, and admits the assignment directly to the
                // Core mailbox. Core owns ordering, deduplication, wake-up, and turn lifecycle.
                let acks = self.take_spawn_dispatch_acks_for_task(&target_node_id, task.as_str());
                let source_node_id = acks
                    .first()
                    .map(|ack| ack.source_node_id.clone())
                    .unwrap_or_else(|| self.spawn_root_node_id());
                let source_thread_id = self
                    .spawn_node_backing_thread_id(&source_node_id)
                    .or(self.primary_thread_id)
                    .unwrap_or(thread_id);
                let seq = acks
                    .first()
                    .map(|ack| ack.seq)
                    .unwrap_or_else(|| self.reserve_spawn_dispatch_seq_without_persist());
                let origin_id = acks
                    .first()
                    .and_then(|ack| ack.origin_id.as_deref())
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("host-seq-{seq:020}"));
                let message_id = crate::dispatch_queue::native_mailbox_message_id(
                    &origin_id,
                    &source_node_id,
                    &target_node_id,
                );
                // #507: with the kill switch on the task waits in the mailbox
                // for that pane's next turn; nothing runs, before or after the
                // switch is turned off. #660: also when another agent sent it
                // while the switch was on and it is handled only after
                // switch-off; it asks for no turn.
                // `trigger_turn` is part of a mailbox message's identity. The ID
                // comes from the dispatch origin, admitted once, so the same ID
                // is not sent again on the other side of a switch change.
                let hold = sent_while_kill_switch_on || self.kill_switch_on_now();
                let params = ThreadAgentMessageParams {
                    source_thread_id: source_thread_id.to_string(),
                    target_thread_id: thread_id.to_string(),
                    message_id: Some(message_id.clone()),
                    assignment_id: Some(message_id.clone()),
                    kind: AgentMessageKind::Assignment,
                    content: task.clone(),
                    trigger_turn: !hold,
                };
                match app_server.send_agent_message(params).await {
                    Ok(_) => {
                        self.spawn_processed_dispatch_origins.insert(origin_id);
                        self.spawn_processed_dispatch_seq_ids
                            .extend(acks.iter().map(|ack| ack.seq));
                        self.evict_spawn_processed_dispatch_seq_ids();
                        for ack in &acks {
                            self.note_assignment_dispatch_delivered(
                                &ack.source_node_id,
                                &ack.target_node_id,
                            );
                        }
                        self.record_spawn_dispatch_acks(
                            &acks,
                            "queued",
                            "durably admitted to the native mailbox",
                            /*notify*/ false,
                        );
                        if hold {
                            self.note_kill_switch_held_native_task(thread_id);
                            self.chat_widget.add_info_message(
                                self.kill_switch_held_native_task_message(&label),
                                /*hint*/ None,
                            );
                        } else {
                            self.spawn_status_by_thread.insert(
                                thread_id,
                                codex_app_server_protocol::CollabAgentState {
                                    status: codex_app_server_protocol::CollabAgentStatus::Running,
                                    message: None,
                                },
                            );
                            self.agent_navigation
                                .set_running(thread_id, /*is_running*/ true);
                        }
                        self.agent_navigation.set_last_task_message(
                            thread_id,
                            Some(task.chars().take(240).collect()),
                        );
                        self.persist_pane_state();
                    }
                    // D106: a pane the person opened is never re-created as a
                    // saved agent; if Core has not loaded it, the task fails
                    // (below) as it did when it had no live session.
                    Err(error)
                        if !is_operator_pane
                            && AppServerSession::agent_message_target_not_found(
                                &error, thread_id,
                            ) =>
                    {
                        match self
                            .materialize_saved_native_spawn_thread_for_task(app_server, thread_id)
                            .await
                        {
                            Ok(materialized_thread_id) => {
                                // The switch may have gone on (in another
                                // process) while the agent was materialized.
                                let hold = hold || self.kill_switch_on_now();
                                let materialized_source_thread_id = self
                                    .spawn_node_backing_thread_id(&source_node_id)
                                    .or(self.primary_thread_id)
                                    .unwrap_or(materialized_thread_id);
                                let retry_params = ThreadAgentMessageParams {
                                    source_thread_id: materialized_source_thread_id.to_string(),
                                    target_thread_id: materialized_thread_id.to_string(),
                                    message_id: Some(message_id.clone()),
                                    assignment_id: Some(message_id),
                                    kind: AgentMessageKind::Assignment,
                                    content: task.clone(),
                                    trigger_turn: !hold,
                                };
                                match app_server.send_agent_message(retry_params).await {
                                    Ok(_) => {
                                        self.spawn_processed_dispatch_origins.insert(origin_id);
                                        self.spawn_processed_dispatch_seq_ids
                                            .extend(acks.iter().map(|ack| ack.seq));
                                        self.evict_spawn_processed_dispatch_seq_ids();
                                        for ack in &acks {
                                            self.note_assignment_dispatch_delivered(
                                                &ack.source_node_id,
                                                &ack.target_node_id,
                                            );
                                        }
                                        self.record_spawn_dispatch_acks(
                                            &acks,
                                            "queued",
                                            "cold-restored agent was materialized and the assignment was durably admitted to its native mailbox",
                                            /*notify*/ false,
                                        );
                                        if hold {
                                            self.note_kill_switch_held_native_task(
                                                materialized_thread_id,
                                            );
                                            self.chat_widget.add_info_message(
                                                self.kill_switch_held_native_task_message(&label),
                                                /*hint*/ None,
                                            );
                                        } else {
                                            self.spawn_status_by_thread.insert(
                                                materialized_thread_id,
                                                codex_app_server_protocol::CollabAgentState {
                                                    status: codex_app_server_protocol::CollabAgentStatus::Running,
                                                    message: None,
                                                },
                                            );
                                            self.agent_navigation.set_running(
                                                materialized_thread_id,
                                                /*is_running*/ true,
                                            );
                                        }
                                        self.agent_navigation.set_last_task_message(
                                            materialized_thread_id,
                                            Some(task.chars().take(240).collect()),
                                        );
                                        self.persist_pane_state();
                                    }
                                    Err(retry_error) => {
                                        self.release_spawn_dispatch_origins(&acks);
                                        self.record_spawn_dispatch_acks_with_turns(
                                            &acks,
                                            "failed",
                                            &format!(
                                                "native mailbox admission still failed after Core materialized the saved agent: {retry_error:#}"
                                            ),
                                            /*notify*/ true,
                                            /*start_turns*/ !hold,
                                        );
                                        self.chat_widget.add_error_message(format!(
                                            "Could not admit task for {label}: {retry_error:#}"
                                        ));
                                    }
                                }
                            }
                            Err(materialization_error) => {
                                self.release_spawn_dispatch_origins(&acks);
                                self.record_spawn_dispatch_acks_with_turns(
                                    &acks,
                                    "failed",
                                    &format!(
                                        "native mailbox target was unavailable and Core could not materialize the saved agent: {materialization_error:#}"
                                    ),
                                    /*notify*/ true,
                                    /*start_turns*/ !hold,
                                );
                                self.chat_widget.add_error_message(format!(
                                    "Could not admit task for {label}: {materialization_error:#}"
                                ));
                            }
                        }
                    }
                    Err(error) => {
                        self.release_spawn_dispatch_origins(&acks);
                        self.record_spawn_dispatch_acks_with_turns(
                            &acks,
                            "failed",
                            &format!("native mailbox admission failed: {error:#}"),
                            /*notify*/ true,
                            /*start_turns*/ !hold,
                        );
                        self.chat_widget.add_error_message(format!(
                            "Could not admit task for {label}; no automatic replay was attempted: {error:#}"
                        ));
                    }
                }
            }
            AppEvent::SubmitSpawnAgentTaskFromPerson { thread_id, task } => {
                // D95 (#1146): the person's own task from Spawn status reaches
                // the pane as user input, as if typed there. No agent's
                // dispatch acknowledgement is consumed or attributed to it.
                if self.spawn_legacy_read_only
                    || self.spawn_crew.as_ref().is_some_and(|crew| {
                        !matches!(crew.status, crate::crew_state::CrewCreationStatus::Ready)
                    })
                {
                    self.chat_widget.add_error_message(
                        "This /spawn hierarchy is read-only until its crew identity and creation \
                         state are reconciled."
                            .to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                }
                let task = task.trim().to_string();
                if task.is_empty() {
                    self.chat_widget
                        .add_error_message("Spawn task cannot be empty.".to_string());
                    return Ok(AppRunControl::Continue);
                }
                if task.len() > crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES {
                    self.chat_widget.add_error_message(format!(
                        "task is {} bytes; maximum is {} bytes",
                        task.len(),
                        crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES
                    ));
                    return Ok(AppRunControl::Continue);
                }
                let label = self.thread_label(thread_id);
                // A pane its parent controls takes no typed input; the task
                // goes to it as before, as a message, and the person is told.
                if self.agent_navigation.is_parent_owned(thread_id) {
                    self.chat_widget.add_info_message(
                        person_task_to_parent_owned_pane_message(&label),
                        /*hint*/ None,
                    );
                    self.app_event_tx.send(AppEvent::SubmitSpawnAgentTask {
                        thread_id,
                        task,
                        sent_while_kill_switch_on: false,
                    });
                    return Ok(AppRunControl::Continue);
                }
                // #507: a typed message is not kept while the kill switch is
                // on; neither is this one (checked again before it is sent).
                if self.kill_switch_on_now() {
                    self.chat_widget
                        .add_error_message(person_task_kill_switch_message(&label));
                    return Ok(AppRunControl::Continue);
                }
                let thread_id = match self
                    .materialize_saved_native_spawn_thread_for_task(app_server, thread_id)
                    .await
                {
                    Ok(thread_id) => thread_id,
                    Err(error) => {
                        self.chat_widget
                            .add_error_message(format!("Not sent to {label}: {error:#}"));
                        return Ok(AppRunControl::Continue);
                    }
                };
                if self.agent_navigation.is_parent_owned(thread_id) {
                    self.chat_widget.add_info_message(
                        person_task_to_parent_owned_pane_message(&label),
                        /*hint*/ None,
                    );
                    self.app_event_tx.send(AppEvent::SubmitSpawnAgentTask {
                        thread_id,
                        task,
                        sent_while_kill_switch_on: false,
                    });
                    return Ok(AppRunControl::Continue);
                }
                if self.kill_switch_on_now() {
                    self.chat_widget
                        .add_error_message(person_task_kill_switch_message(&label));
                    return Ok(AppRunControl::Continue);
                }
                // User input, as typing sends it: added to a running turn, or
                // a new turn. It sends no settings, so the pane keeps the
                // approval, sandbox, model and mode Core holds for it.
                let items = vec![codex_app_server_protocol::UserInput::Text {
                    text: task.clone(),
                    text_elements: Vec::new(),
                }];
                let sent = match self.active_turn_id_for_submission(thread_id).await {
                    Some(turn_id) => app_server
                        .turn_steer(thread_id, turn_id, items, /*client_user_message_id*/ None)
                        .await
                        .map(|_| ())
                        .map_err(|error| {
                            format!(
                                "Not sent to {label}: its running turn did not take it ({error}). \
                                 Send it again when that turn ends."
                            )
                        }),
                    None => match app_server
                        .turn_start_keeping_thread_settings(
                            thread_id,
                            items,
                            self.spawn_additional_context_for_thread(thread_id),
                        )
                        .await
                    {
                        Ok(crate::app_server_session::TurnStartOutcome::Started { .. }) => Ok(()),
                        Ok(crate::app_server_session::TurnStartOutcome::CapacityUnavailable {
                            max_threads,
                        }) => Err(format!(
                            "Not sent to {label}: all {max_threads} execution slots are in use."
                        )),
                        Err(error) => {
                            Err(format!("Not sent to {label}: the turn could not start: {error:#}"))
                        }
                    },
                };
                match sent {
                    Ok(()) => {
                        self.agent_navigation.set_last_task_message(
                            thread_id,
                            Some(task.chars().take(240).collect()),
                        );
                        self.persist_pane_state();
                    }
                    Err(detail) => self.chat_widget.add_error_message(detail),
                }
            }
            AppEvent::SendSpawnAgentMailboxMessage { params } => {
                if let Err(err) = app_server.send_agent_message(params).await {
                    tracing::error!(
                        error = ?err,
                        error_chain = %format!("{err:#}"),
                        "failed to deliver edge-adapter report through native mailbox"
                    );
                    self.chat_widget.add_error_message(format!(
                        "A child report could not be durably delivered: {err:#}"
                    ));
                }
            }
            AppEvent::SubmitSpawnClaudePaneTask {
                pane_id,
                task,
                sent_while_kill_switch_on,
            } => {
                if self.spawn_legacy_read_only {
                    self.chat_widget.add_error_message(
                        "This restored legacy /spawn hierarchy is read-only. Existing Claude panes \
                         can be inspected but not mutated by the new control plane."
                            .to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                }
                let target = crate::spawn_orchestration::pane_node_id(&pane_id);
                if task.len() > crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES {
                    let acks = self.take_spawn_dispatch_acks_for_task(&target, &task);
                    self.release_spawn_dispatch_origins(&acks);
                    let detail = format!(
                        "task is {} bytes; maximum is {} bytes",
                        task.len(),
                        crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES
                    );
                    // #660: no sender turn for a task sent while on.
                    self.record_spawn_dispatch_acks_with_turns(
                        &acks,
                        "failed",
                        &detail,
                        /*notify*/ true,
                        /*start_turns*/ !sent_while_kill_switch_on,
                    );
                    self.chat_widget.add_error_message(detail);
                    return Ok(AppRunControl::Continue);
                }
                // #507: with the kill switch on the task waits, also for a
                // pane still stopping (#613: or when it was sent while on).
                let hold = sent_while_kill_switch_on || self.kill_switch_on_now();
                if self.claude_panes.claude_pane_is_running(&pane_id) && !hold {
                    let acks = self.take_spawn_dispatch_acks_for_task(&target, &task);
                    self.release_spawn_dispatch_origins(&acks);
                    self.record_spawn_dispatch_acks(
                        &acks,
                        "failed",
                        "legacy external Claude pane is busy; no secondary queue or automatic retry exists",
                        /*notify*/ true,
                    );
                    self.chat_widget.add_error_message(format!(
                        "Cannot send task to {pane_id}: the legacy external Claude pane is busy. \
                         Wait for it to finish or use a native Claude Plan member in /spawn."
                    ));
                    return Ok(AppRunControl::Continue);
                }
                self.submit_claude_pane_task_queued(pane_id, task, sent_while_kill_switch_on);
            }
            AppEvent::OpenSpawnStatus => {
                self.open_spawn_status();
            }
            AppEvent::OpenRemoveSpawnCrewConfirmation => {
                self.open_remove_spawn_crew_confirmation();
            }
            AppEvent::RemoveSpawnCrew => {
                match self.remove_spawn_crew(tui, app_server).await {
                    Ok(message) => self.chat_widget.add_info_message(message, /*hint*/ None),
                    Err(err) => self.chat_widget.add_error_message(err.to_string()),
                }
            }
            AppEvent::HandleOrchestrateCommand { args } => {
                self.handle_orchestrate_command(args);
            }
            AppEvent::OpenOrchestrateTargetPicker => {
                self.open_orchestrate_target_picker();
            }
            AppEvent::OpenOrchestrateFastTargetPicker => {
                self.open_orchestrate_fast_target_picker();
            }
            AppEvent::OpenOrchestrateFastManagerPicker { target } => {
                self.open_orchestrate_fast_manager_picker(target);
            }
            AppEvent::AttachOrchestrateFastManager {
                target,
                manager_node_id,
            } => {
                let args = crate::orchestrate::orchestrate_guided_attach_args(
                    &target,
                    "8h",
                    crate::orchestrate::DRAFT_WITH_MANAGER_SPEC,
                    &manager_node_id,
                );
                match self.attach_guided_assignment(&args) {
                    Ok(message) => self.chat_widget.add_info_message(message, /*hint*/ None),
                    Err(err) => self.chat_widget.add_error_message(err),
                }
            }
            AppEvent::OpenOrchestrateDurationPicker { target } => {
                self.open_orchestrate_duration_picker(target);
            }
            AppEvent::OpenOrchestrateWhipPicker {
                target,
                duration_arg,
                duration_label,
            } => {
                self.open_orchestrate_whip_picker(target, duration_arg, duration_label);
            }
            AppEvent::OpenOrchestrateWriteWhipPrompt {
                target,
                duration_arg,
                duration_label,
            } => {
                self.open_orchestrate_write_whip_prompt(target, duration_arg, duration_label);
            }
            AppEvent::OpenOrchestrateSaveWhipPrompt {
                target,
                duration_arg,
                duration_label,
                instructions,
            } => {
                self.open_orchestrate_save_whip_prompt(
                    target,
                    duration_arg,
                    duration_label,
                    instructions,
                );
            }
            AppEvent::SaveOrchestrateWhipAndConfirm {
                target,
                duration_arg,
                duration_label,
                requested_name,
                instructions,
            } => {
                self.save_orchestrate_whip_and_open_confirm(
                    target,
                    duration_arg,
                    duration_label,
                    requested_name,
                    instructions,
                );
            }
            AppEvent::OpenOrchestrateConfirm {
                target,
                duration_arg,
                duration_label,
                whip_name,
                manager_node_id,
            } => {
                self.open_orchestrate_confirm(
                    target,
                    duration_arg,
                    duration_label,
                    whip_name,
                    manager_node_id,
                );
            }
            AppEvent::OpenOrchestrateManagerPicker {
                target,
                duration_arg,
                duration_label,
                whip_name,
            } => {
                self.open_orchestrate_manager_picker(
                    target,
                    duration_arg,
                    duration_label,
                    whip_name,
                );
            }
            AppEvent::CreateOrchestrateManager {
                target,
                duration_arg,
                whip_name,
            } => {
                let Some(parent_thread_id) = self.primary_thread_id.or(self.active_thread_id)
                else {
                    self.chat_widget.add_error_message(
                        "Cannot create a Manager before Corbanu Terminal Main has started."
                            .to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                };
                let spawn_config = match self.native_spawn_agent_config() {
                    Ok(config) => config,
                    Err(err) => {
                        self.chat_widget.add_error_message(err.to_string());
                        return Ok(AppRunControl::Continue);
                    }
                };
                let model = self.native_spawn_default_model();
                let provider = self.native_spawn_default_provider(&model);
                let nickname = match self
                    .unique_native_pane_nickname("Manager", /*exclude_thread_id*/ None)
                {
                    Ok(nickname) => nickname,
                    Err(err) => {
                        self.chat_widget.add_error_message(err.to_string());
                        return Ok(AppRunControl::Continue);
                    }
                };
                match app_server
                    .spawn_agent_thread(
                        &spawn_config,
                        parent_thread_id,
                        "default".to_string(),
                        Some(nickname.clone()),
                        model,
                        provider,
                        /*reasoning_effort*/ None,
                        /*base_instructions*/ None,
                    )
                    .await
                {
                    Ok(started) => {
                        let thread_id = started.session.thread_id;
                        self.register_codex_user_pane(
                            app_server,
                            thread_id,
                            Some(nickname.clone()),
                            started,
                        )
                        .await;
                        let manager_node_id = crate::spawn_orchestration::thread_node_id(thread_id);
                        let args = crate::orchestrate::orchestrate_guided_attach_args(
                            &target,
                            &duration_arg,
                            &whip_name,
                            &manager_node_id,
                        );
                        match self.attach_guided_assignment(&args) {
                            Ok(message) => {
                                self.chat_widget.add_info_message(message, /*hint*/ None);
                                self.chat_widget.add_info_message(
                                    format!("Created Manager {nickname}."),
                                    Some(
                                        "The assignment brief was sent to the new pane."
                                            .to_string(),
                                    ),
                                );
                            }
                            Err(err) => self.chat_widget.add_error_message(format!(
                                "Manager pane {nickname} created but not bound: {err}"
                            )),
                        }
                    }
                    Err(err) => self
                        .chat_widget
                        .add_error_message(format!("Failed to create Manager: {err}")),
                }
            }
            AppEvent::OpenOrchestrateWhipDetails { whip_id } => {
                self.open_orchestrate_whip_details(whip_id);
            }
            AppEvent::OpenOrchestrateExtendDurationPicker { whip_id } => {
                self.open_orchestrate_extend_duration_picker(whip_id);
            }
            AppEvent::WhipSweepTick => {
                self.sweep_orchestrate_whips();
            }
            AppEvent::SelectUserPane { pane_id } => {
                let is_codex_main = pane_id == crate::claude_panes::CODEX_MAIN_PANE_ID;
                self.select_user_pane(tui, pane_id).await;
                if is_codex_main && let Some(primary_thread_id) = self.primary_thread_id {
                    self.select_agent_thread_and_discard_side(tui, app_server, primary_thread_id)
                        .await?;
                }
            }
            AppEvent::CreateCodexPane {
                model,
                provider,
                effort,
                display_name,
            } => {
                if self.primary_thread_id.or(self.active_thread_id).is_none() {
                    self.chat_widget.add_error_message(
                        "Cannot create a Corbanu Terminal pane before Corbanu Terminal Main has started."
                            .to_string(),
                    );
                    return Ok(AppRunControl::Continue);
                }
                let pane_config = match self.native_spawn_agent_config() {
                    Ok(config) => config,
                    Err(err) => {
                        self.chat_widget.add_error_message(err.to_string());
                        return Ok(AppRunControl::Continue);
                    }
                };
                let requested_name =
                    display_name.unwrap_or_else(|| self.next_codex_pane_nickname());
                let nickname = match self
                    .unique_native_pane_nickname(&requested_name, /*exclude_thread_id*/ None)
                {
                    Ok(nickname) => nickname,
                    Err(err) => {
                        self.chat_widget.add_error_message(err.to_string());
                        return Ok(AppRunControl::Continue);
                    }
                };
                match app_server
                    .start_user_pane_thread(
                        &pane_config,
                        model.clone(),
                        provider.clone(),
                        effort.clone(),
                    )
                    .await
                {
                    Ok(started) => {
                        let thread_id = started.session.thread_id;
                        self.register_codex_user_pane(
                            app_server,
                            thread_id,
                            Some(nickname.clone()),
                            started,
                        )
                        .await;
                        self.persist_pane_state();
                        self.save_active_claude_pane_transcript();
                        let _ = self
                            .claude_panes
                            .set_active_user_pane(crate::claude_panes::CODEX_MAIN_PANE_ID);
                        self.select_agent_thread_and_discard_side(tui, app_server, thread_id)
                            .await?;
                        self.chat_widget.add_info_message(
                            format!("Created and switched to Corbanu Terminal pane {nickname}."),
                            Some(format!("{model}; no task was started.")),
                        );
                    }
                    Err(err) => {
                        self.chat_widget.add_error_message(format!(
                            "Failed to create Corbanu Terminal pane: {err}"
                        ));
                    }
                }
            }
            AppEvent::OpenCodexPaneNamePrompt {
                model,
                provider,
                effort,
            } => {
                self.open_codex_pane_name_prompt(model, provider, effort);
            }
            AppEvent::OpenClaudePaneNamePrompt { profile } => {
                self.open_claude_pane_name_prompt(profile);
            }
            AppEvent::CreateClaudePane {
                profile,
                display_name,
            } => {
                let display_name = match display_name {
                    Some(display_name) => {
                        match self
                            .unique_pane_display_name(&display_name, /*exclude_node_id*/ None)
                        {
                            Ok(name) => Some(name),
                            Err(err) => {
                                self.chat_widget.add_error_message(err.to_string());
                                return Ok(AppRunControl::Continue);
                            }
                        }
                    }
                    None => None,
                };
                self.create_claude_pane(tui, profile, display_name).await;
            }
            AppEvent::RenameCurrentPane { name } => {
                self.rename_current_pane_display_name(app_server, name)
                    .await;
            }
            AppEvent::OpenRenameCodexPanePrompt { thread_id } => {
                self.open_rename_codex_pane_prompt(thread_id);
            }
            AppEvent::OpenRenameClaudePanePrompt { pane_id } => {
                self.open_rename_claude_pane_prompt(pane_id);
            }
            AppEvent::RenameCodexPane { thread_id, name } => {
                self.rename_codex_pane_display_name(app_server, thread_id, name)
                    .await;
            }
            AppEvent::RenameClaudePane { pane_id, name } => {
                self.rename_claude_pane_display_name(pane_id, name);
            }
            AppEvent::CreateSpawnClaudePane {
                role,
                parent_node_id,
                profile,
            } => {
                self.create_spawn_claude_pane(tui, role, parent_node_id, profile)
                    .await;
            }
            AppEvent::ClaudePaneTurnFinished { pane_id, result } => {
                self.on_claude_pane_turn_finished(pane_id, result);
            }
            AppEvent::ClaudePaneTurnProgress { progress } => {
                self.on_claude_pane_turn_progress(progress);
            }
            AppEvent::ClaudePaneApprovalRequested(request) => {
                self.chat_widget.show_claude_approval(*request);
            }
            AppEvent::ClaudePaneLaunchProblem { pane_id, problem } => {
                self.on_claude_pane_launch_problem(&pane_id, problem);
            }
            AppEvent::ClaudePaneApprovalsSettled => {
                self.chat_widget.remove_settled_claude_approvals();
            }
            AppEvent::PaneBridgeModelRequestSent {
                provider_id,
                base_url,
                path,
                model,
                usage,
            } => {
                self.record_pane_bridge_model_request(
                    app_server,
                    provider_id,
                    base_url,
                    path,
                    model,
                    usage,
                );
            }
            AppEvent::StartSide {
                parent_thread_id,
                user_message,
            } => {
                return self
                    .handle_start_side(tui, app_server, parent_thread_id, user_message)
                    .await;
            }
            AppEvent::OpenSkillsList => {
                self.chat_widget.open_skills_list();
            }
            AppEvent::OpenManageSkillsPopup => {
                self.chat_widget.open_manage_skills_popup();
            }
            AppEvent::SetSkillEnabled { path, enabled } => {
                match crate::config_update::write_skill_enabled(
                    app_server.request_handle(),
                    path.clone(),
                    enabled,
                )
                .await
                {
                    Ok(()) => {
                        self.chat_widget.update_skill_enabled(path, enabled);
                    }
                    Err(err) => {
                        let path_display = path.inferred_native_path_string();
                        self.chat_widget.add_error_message(format!(
                            "Failed to update skill config for {path_display}: {err}"
                        ));
                    }
                }
            }
            AppEvent::SetAppEnabled { id, enabled } => {
                let edits = if enabled {
                    vec![
                        crate::config_update::clear_config_value(
                            crate::config_update::app_scoped_key_path(&id, "enabled"),
                        ),
                        crate::config_update::clear_config_value(
                            crate::config_update::app_scoped_key_path(&id, "disabled_reason"),
                        ),
                    ]
                } else {
                    vec![
                        crate::config_update::replace_config_value(
                            crate::config_update::app_scoped_key_path(&id, "enabled"),
                            serde_json::json!(false),
                        ),
                        crate::config_update::replace_config_value(
                            crate::config_update::app_scoped_key_path(&id, "disabled_reason"),
                            serde_json::json!("user"),
                        ),
                    ]
                };
                match crate::config_update::write_config_batch(app_server.request_handle(), edits)
                    .await
                {
                    Ok(_) => {
                        self.chat_widget.update_connector_enabled(&id, enabled);
                    }
                    Err(err) => {
                        self.chat_widget.add_error_message(format!(
                            "Failed to update app config for {id}: {err}"
                        ));
                    }
                }
            }
            AppEvent::SetHookEnabled { key, enabled } => {
                self.set_hook_enabled(app_server, key, enabled);
            }
            AppEvent::TrustHook { key, current_hash } => {
                self.trust_hook(app_server, key, current_hash);
            }
            AppEvent::TrustHooks { updates } => {
                self.trust_hooks(app_server, updates);
            }
            AppEvent::HookEnabledSet {
                key,
                enabled,
                result,
            } => {
                let queued_enabled = self
                    .pending_hook_enabled_writes
                    .get_mut(&key)
                    .and_then(Option::take);
                let should_apply_result = if let Some(queued_enabled) = queued_enabled
                    && (result.is_err() || queued_enabled != enabled)
                {
                    self.spawn_hook_enabled_write(app_server, key.clone(), queued_enabled);
                    false
                } else {
                    true
                };
                if should_apply_result {
                    self.pending_hook_enabled_writes.remove(&key);
                    if let Err(err) = result {
                        self.chat_widget.add_error_message(err);
                    }
                }
            }
            AppEvent::HookTrusted { result } => {
                if let Err(err) = result {
                    self.chat_widget.add_error_message(err);
                }
            }
            AppEvent::ResolvePermissionProfile { profile_id } => {
                if self.reject_pending_permission_change() {
                    self.chat_widget
                        .retain_input_after_failed_permission_selection();
                    self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
                    return Ok(AppRunControl::Continue);
                }
                #[cfg(any(target_os = "windows", test))]
                if self.chat_widget.windows_sandbox_local_server
                    && self.windows_sandbox_host() != WindowsSandboxHost::Remote
                    && self.chat_widget.windows_sandbox_config.requirements.is_none()
                    && !self.refresh_windows_sandbox_config(app_server).await
                {
                    self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
                    return Ok(AppRunControl::Continue);
                }
                self.chat_widget
                    .select_permission_profile(&profile_id);
            }
            AppEvent::OpenPermissionsPopup | AppEvent::OpenApprovalsPopup => {
                self.chat_widget.permission_popup_pending = false;
                if self.reject_pending_permission_change() {
                    self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
                    return Ok(AppRunControl::Continue);
                }
                #[cfg(any(target_os = "windows", test))]
                if self.chat_widget.windows_sandbox_local_server
                    && self.windows_sandbox_host() != WindowsSandboxHost::Remote
                    && self.chat_widget.windows_sandbox_config.requirements.is_none()
                    && !self.refresh_windows_sandbox_config(app_server).await
                {
                    self.app_event_tx.send(AppEvent::SettingsSelectionClosed);
                    return Ok(AppRunControl::Continue);
                }
                if app_server.uses_remote_workspace() {
                    self.chat_widget.request_permission_profiles();
                } else {
                    self.chat_widget.open_approvals_popup();
                }
            }
            AppEvent::OpenReviewBranchPicker(cwd) => {
                self.chat_widget.show_review_branch_picker(&cwd).await;
            }
            AppEvent::OpenReviewCommitPicker(cwd) => {
                self.chat_widget.show_review_commit_picker(&cwd).await;
            }
            AppEvent::OpenReviewCustomPrompt => {
                self.chat_widget.show_review_custom_prompt();
            }
            AppEvent::SubmitUserMessageWithMode {
                text,
                collaboration_mode,
            } => {
                if let Some(thread_id) = self.active_thread_id {
                    self.note_assignment_user_turn(&crate::spawn_orchestration::thread_node_id(
                        thread_id,
                    ));
                }
                self.chat_widget
                    .submit_user_message_with_mode(text, collaboration_mode);
            }
            AppEvent::ManageSkillsClosed => {
                self.chat_widget.handle_manage_skills_closed();
            }
            AppEvent::FullScreenUserVerificationRequest(request) => {
                let _ = tui.enter_alt_screen();
                self.overlay = Some(Overlay::new_static_with_renderables(
                    vec![crate::bottom_pane::user_verification::prompt_header(&request)],
                    "U S E R  V E R I F I C A T I O N".to_string(),
                    self.keymap.pager.clone(),
                ));
            }
            AppEvent::FullScreenApprovalRequest(request) => match request {
                ApprovalRequest::ApplyPatch(request) => {
                    let _ = tui.enter_alt_screen();
                    let diff_summary = DiffSummary::new(request.changes, request.cwd);
                    self.overlay = Some(Overlay::new_static_with_renderables(
                        vec![diff_summary.into()],
                        "P A T C H".to_string(),
                        self.keymap.pager.clone(),
                    ));
                }
                ApprovalRequest::Exec(request) => {
                    let _ = tui.enter_alt_screen();
                    let full_cmd = strip_bash_lc_and_escape(&request.command);
                    let full_cmd_lines = highlight_bash_to_lines(&full_cmd);
                    self.overlay = Some(Overlay::new_static_with_lines(
                        full_cmd_lines,
                        "E X E C".to_string(),
                        self.keymap.pager.clone(),
                    ));
                }
                ApprovalRequest::Permissions(request) => {
                    let _ = tui.enter_alt_screen();
                    let mut lines = Vec::new();
                    if let Some(environment_id) = request.environment_id {
                        lines.push(Line::from(vec![
                            "Environment: ".into(),
                            environment_id.bold(),
                        ]));
                        lines.push(Line::from(""));
                    }
                    if let Some(reason) = request.reason {
                        lines.push(Line::from(vec!["Reason: ".into(), reason.italic()]));
                        lines.push(Line::from(""));
                    }
                    if let Some(rule_line) =
                        crate::bottom_pane::format_requested_permissions_rule(&request.permissions)
                    {
                        lines.push(Line::from(vec![
                            "Permission rule: ".into(),
                            rule_line.cyan(),
                        ]));
                    }
                    self.overlay = Some(Overlay::new_static_with_renderables(
                        vec![Box::new(Paragraph::new(lines).wrap(Wrap { trim: false }))],
                        "P E R M I S S I O N S".to_string(),
                        self.keymap.pager.clone(),
                    ));
                }
                ApprovalRequest::McpElicitation(request) => {
                    let _ = tui.enter_alt_screen();
                    let paragraph = Paragraph::new(vec![
                        Line::from(vec!["Server: ".into(), request.server_name.bold()]),
                        Line::from(""),
                        Line::from(request.message),
                    ])
                    .wrap(Wrap { trim: false });
                    self.overlay = Some(Overlay::new_static_with_renderables(
                        vec![Box::new(paragraph)],
                        "E L I C I T A T I O N".to_string(),
                        self.keymap.pager.clone(),
                    ));
                }
            },
            AppEvent::FullscreenTranscriptSelected { enabled } => {
                self.save_fullscreen_transcript(enabled).await;
            }
            AppEvent::StatusLineSetup {
                items,
                use_theme_colors,
            } => {
                let ids = items.iter().map(ToString::to_string).collect::<Vec<_>>();
                let items_edit = crate::legacy_core::config::edit::status_line_items_edit(&ids);
                let colors_edit =
                    crate::legacy_core::config::edit::status_line_use_colors_edit(use_theme_colors);
                let apply_result = ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
                    .with_edits([items_edit, colors_edit])
                    .apply()
                    .await;
                match apply_result {
                    Ok(()) => {
                        self.local_settings.tui.status_line = Some(ids.clone());
                        self.local_settings.tui.status_line_use_colors = use_theme_colors;
                        self.chat_widget.setup_status_line(items, use_theme_colors);
                        self.chat_widget.refresh_config_panel(/*notice*/ None);
                    }
                    Err(err) => {
                        let error = format_config_error(&err);
                        tracing::error!(error = %error, "failed to persist status line settings; keeping previous selection");
                        self.app_event_tx.send(AppEvent::FollowTranscript);
                        let message = format!("Failed to save status line settings: {error}");
                        self.chat_widget.report_config_save_error(message);
                    }
                }
            }
            AppEvent::StatusLineBranchUpdated { cwd, branch } => {
                self.chat_widget.set_status_line_branch(cwd, branch);
                self.refresh_status_line();
            }
            AppEvent::StatusLineGitSummaryUpdated { cwd, summary } => {
                self.chat_widget.set_status_line_git_summary(cwd, summary);
                self.refresh_status_line();
            }
            AppEvent::StatusLineWorkspaceHeadlineUpdated { request_id, result } => {
                if self
                    .chat_widget
                    .set_status_line_workspace_headline(request_id, result)
                {
                    tui.frame_requester().schedule_frame();
                }
            }
            AppEvent::StatusLineSetupCancelled => {
                self.chat_widget.cancel_status_line_setup();
                self.chat_widget.refresh_config_panel(/*notice*/ None);
            }
            AppEvent::TerminalTitleSetup { items } => {
                let ids = items.iter().map(ToString::to_string).collect::<Vec<_>>();
                let edit = crate::legacy_core::config::edit::terminal_title_items_edit(&ids);
                let apply_result = ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
                    .with_edits([edit])
                    .apply()
                    .await;
                match apply_result {
                    Ok(()) => {
                        self.local_settings.tui.terminal_title = Some(ids.clone());
                        self.chat_widget.setup_terminal_title(items);
                        self.chat_widget.refresh_config_panel(/*notice*/ None);
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "failed to persist terminal title items; keeping previous selection");
                        self.app_event_tx.send(AppEvent::FollowTranscript);
                        self.chat_widget.revert_terminal_title_setup_preview();
                        let message = format!("Failed to save terminal title items: {err}");
                        self.chat_widget.report_config_save_error(message);
                    }
                }
            }
            AppEvent::TerminalTitleSetupPreview { items } => {
                self.chat_widget.preview_terminal_title(items);
            }
            AppEvent::TerminalTitleSetupCancelled => {
                self.chat_widget.cancel_terminal_title_setup();
                self.chat_widget.refresh_config_panel(/*notice*/ None);
            }
            AppEvent::OpenConfigEditor(handoff) => {
                self.cancel_pending_key_chord();
                if let Err(err) = tui.discard_pending_input_before_interactive_screen() {
                    tracing::warn!(%err, "failed to discard input before opening config editor");
                    self.chat_widget.config_editor_open_failed(format!(
                        "Failed to open setting because terminal input could not be cleared: {err}"
                    ));
                } else {
                    let resume_current_availability =
                        super::resume_config::resume_current_directory_availability(
                            &self.app_server_target,
                            self.environment_manager.as_ref(),
                            self.runtime_working_directory_override
                                .as_deref()
                                .or(self.harness_overrides.cwd.as_deref())
                                .or_else(|| app_server.remote_cwd_override()),
                        );
                    self.chat_widget
                        .open_config_editor(handoff, resume_current_availability);
                    self.chat_widget.refresh_config_panel(/*notice*/ None);
                }
            }
            AppEvent::TestConfigNotification => self.schedule_config_notification_test(),
            AppEvent::SendConfigTestNotification { generation } => {
                self.send_config_notification_test(tui, generation)
            }
            AppEvent::SaveConfigPreference(preference) => {
                self.save_config_preference(tui, preference).await;
            }
            AppEvent::PersistAgentsOverviewGrouping(grouping) => {
                self.persist_agents_overview_grouping(grouping).await;
            }
            AppEvent::SyntaxThemeSelected { name } => {
                let edit = crate::legacy_core::config::edit::syntax_theme_edit(&name);
                let apply_result = ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
                    .with_edits([edit])
                    .apply()
                    .await;
                match apply_result {
                    Ok(()) => {
                        // Ensure the selected theme is active in the current
                        // session.  The preview callback covers arrow-key
                        // navigation, but if the user presses Enter without
                        // navigating, the runtime theme must still be applied.
                        if let Some(theme) = crate::render::highlight::resolve_theme_by_name(
                            &name,
                            Some(&self.local_settings.codex_home),
                        ) {
                            crate::render::highlight::set_syntax_theme(theme);
                        }
                        self.sync_tui_theme_selection(name);
                        self.chat_widget.refresh_config_panel(/*notice*/ None);
                        self.refresh_status_line();
                        tui.frame_requester().schedule_frame();
                    }
                    Err(err) => {
                        self.restore_runtime_theme_from_config();
                        self.refresh_status_line();
                        tracing::error!(error = %err, "failed to persist theme selection");
                        self.app_event_tx.send(AppEvent::FollowTranscript);
                        let message = format!("Failed to save theme: {err}");
                        self.chat_widget.report_config_save_error(message);
                    }
                }
            }
            AppEvent::SyntaxThemePreviewed => {
                self.refresh_status_line();
                tui.frame_requester().schedule_frame();
            }
            AppEvent::OpenKeymapActionMenu { context, action } => {
                self.chat_widget
                    .open_keymap_action_menu(context, action, &self.keymap);
            }
            AppEvent::OpenKeymapReplaceBindingMenu { context, action } => {
                self.chat_widget
                    .open_keymap_replace_binding_menu(context, action, &self.keymap);
            }
            AppEvent::OpenKeymapCapture {
                context,
                action,
                intent,
                capture_mode,
            } => {
                self.chat_widget.open_keymap_capture(
                    context,
                    action,
                    intent,
                    capture_mode,
                    &self.keymap,
                );
            }
            AppEvent::OpenKeymapDebug => {
                self.chat_widget.open_keymap_debug(&self.keymap);
            }
            AppEvent::KeymapCaptured {
                context,
                action,
                key,
                intent,
            } => {
                self.apply_keymap_capture(app_server, context, action, key, intent)
                    .await;
                self.merge_startup_warnings(tui, &history_cell::StartupWarningsCell::default());
            }
            AppEvent::KeymapCleared { context, action } => {
                self.apply_keymap_clear(context, action).await;
                self.merge_startup_warnings(tui, &history_cell::StartupWarningsCell::default());
            }
            AppEvent::GenerateRecap { thread_id } => {
                if self.current_displayed_thread_id() == Some(thread_id) {
                    if self.chat_widget.is_user_turn_pending_or_running() {
                        self.chat_widget.add_error_message(
                            "Wait for the current task to finish before running /recap.".to_string(),
                        );
                    } else {
                        self.request_recap(app_server, thread_id, RecapTrigger::Manual);
                    }
                }
            }
            AppEvent::CheckRecap { thread_id } => {
                if self.current_displayed_thread_id() == Some(thread_id)
                    && !self.chat_widget.is_user_turn_pending_or_running()
                    && self
                        .recap
                        .should_generate(tokio::time::Instant::now().into_std())
                {
                    self.request_recap(app_server, thread_id, RecapTrigger::Automatic);
                }
            }
            AppEvent::RecapStarted {
                thread_id,
                request_id,
                trigger,
                completed_turn_count,
                turn_revision,
                history,
                result,
            } => {
                self.handle_recap_started(
                    app_server,
                    recap::RecapRequest {
                        thread_id,
                        request_id,
                        trigger,
                        completed_turn_count,
                        turn_revision,
                    },
                    history,
                    result,
                );
            }
            AppEvent::RecapGenerated {
                thread_id,
                request_id,
                trigger,
                temporary_thread_id,
                completed_turn_count,
                turn_revision,
                result,
            } => {
                self.temporary_structured_requests.remove(&temporary_thread_id);
                if let Some(cell) = self.handle_generated_recap(
                    recap::RecapRequest {
                        thread_id,
                        request_id,
                        trigger,
                        completed_turn_count,
                        turn_revision,
                    },
                    temporary_thread_id,
                    result,
                ) {
                    self.insert_history_cell(tui, Box::new(cell));
                }
            }
        }
        if let Some(model) = sparkle_model {
            self.chat_widget
                .on_sparkle_model_selected_from_picker(&model);
        }
        Ok(AppRunControl::Continue)
    }

    async fn apply_keymap_capture(
        &mut self,
        app_server: &mut AppServerSession,
        context: String,
        action: String,
        key: String,
        intent: crate::app_event::KeymapEditIntent,
    ) {
        let outcome = match crate::keymap_setup::keymap_with_edit(
            &self.local_settings.tui.keymap,
            &self.keymap,
            &context,
            &action,
            &key,
            &intent,
        ) {
            Ok(outcome) => outcome,
            Err(err) => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget.add_error_message(err);
                return;
            }
        };
        let (keymap_config, bindings, message) = match outcome {
            crate::keymap_setup::KeymapEditOutcome::Updated {
                keymap_config,
                bindings,
                message,
            } => (*keymap_config, bindings, message),
            crate::keymap_setup::KeymapEditOutcome::Unchanged { message } => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget.add_info_message(message, /*hint*/ None);
                return;
            }
        };

        let runtime_keymap = match RuntimeKeymap::from_config(&keymap_config) {
            Ok(runtime_keymap) => runtime_keymap,
            Err(err) => {
                let params = crate::keymap_setup::build_keymap_conflict_params(
                    context,
                    action,
                    key,
                    intent,
                    err,
                    &self.keymap,
                );
                self.chat_widget.show_selection_view(params);
                return;
            }
        };

        let save = if matches!(self.app_server_target, AppServerTarget::LocalDaemon { .. }) {
            // Let the daemon validate shared config before writing: older daemons may
            // not understand bindings introduced by this TUI, including new defaults.
            app_server
                .request_handle()
                .request_typed::<codex_app_server_protocol::ConfigWriteResponse>(
                    codex_app_server_protocol::ClientRequest::ConfigValueWrite {
                        request_id: app_server.next_request_id(),
                        params: codex_app_server_protocol::ConfigValueWriteParams {
                            key_path: format!("tui.keymap.{context}.{action}"),
                            value: match bindings.as_slice() {
                                [binding] => serde_json::json!(binding),
                                bindings => serde_json::json!(bindings),
                            },
                            merge_strategy: codex_app_server_protocol::MergeStrategy::Replace,
                            file_path: Some(
                                self.local_settings.user_config_path.display().to_string(),
                            ),
                            expected_version: None,
                        },
                    },
                )
                .await
                .map(|_| ())
                .map_err(anyhow::Error::new)
        } else {
            let edit = crate::legacy_core::config::edit::keymap_bindings_edit(
                &context, &action, &bindings,
            );
            ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
                .with_edits([edit])
                .apply()
                .await
        };
        match save {
            Ok(()) => {
                self.cancel_pending_key_chord();
                self.local_settings.tui.keymap = keymap_config.clone();
                self.keymap = runtime_keymap.clone();
                self.chat_widget
                    .apply_keymap_update(keymap_config, &runtime_keymap);
                self.sync_side_thread_ui();
                self.chat_widget
                    .return_to_keymap_picker(&context, &action, &runtime_keymap);
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget.add_info_message(message, /*hint*/ None);
            }
            Err(err) => {
                tracing::error!(error = %err, "failed to persist keymap binding");
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget
                    .report_config_save_error(format!("Failed to save shortcut: {err}"));
            }
        }
    }

    fn refresh_plugin_mentions_after_config_write(&mut self) {
        self.chat_widget.refresh_plugin_mentions();
        self.chat_widget.submit_op(AppCommand::reload_user_config());
        self.chat_widget
            .refresh_connector_mentions(/*force_refresh*/ true);
    }

    async fn apply_keymap_clear(&mut self, context: String, action: String) {
        let keymap_config = match crate::keymap_setup::keymap_without_custom_binding(
            &self.local_settings.tui.keymap,
            &context,
            &action,
        ) {
            Ok(keymap_config) => keymap_config,
            Err(err) => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget.add_error_message(err);
                return;
            }
        };

        let runtime_keymap = match RuntimeKeymap::from_config(&keymap_config) {
            Ok(runtime_keymap) => runtime_keymap,
            Err(err) => {
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget
                    .add_error_message(format!("Failed to refresh shortcuts: {err}"));
                return;
            }
        };

        let edit = crate::legacy_core::config::edit::keymap_binding_clear_edit(&context, &action);
        match ConfigEditsBuilder::for_config_path(self.local_settings.user_config_path.as_path())
            .with_edits([edit])
            .apply()
            .await
        {
            Ok(()) => {
                self.cancel_pending_key_chord();
                self.local_settings.tui.keymap = keymap_config.clone();
                self.keymap = runtime_keymap.clone();
                self.chat_widget
                    .apply_keymap_update(keymap_config, &runtime_keymap);
                self.sync_side_thread_ui();
                self.chat_widget
                    .return_to_keymap_picker(&context, &action, &runtime_keymap);
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget.add_info_message(
                    format!("Removed custom shortcut for `{context}.{action}`."),
                    /*hint*/ None,
                );
            }
            Err(err) => {
                tracing::error!(error = %err, "failed to clear keymap binding");
                self.app_event_tx.send(AppEvent::FollowTranscript);
                self.chat_widget
                    .report_config_save_error(format!("Failed to remove shortcut: {err}"));
            }
        }
    }

    pub(super) async fn handle_exit_mode(
        &mut self,
        app_server: &mut AppServerSession,
        mode: ExitMode,
    ) -> AppRunControl {
        for (request_id, (_, task)) in self.dynamic_tool_tasks.drain() {
            task.abort();
            let response = crate::dynamic_tools::failure_response(
                "TUI disconnected while handling a dynamic tool call",
            );
            match serde_json::to_value(response) {
                Ok(result) => {
                    if let Err(error) = app_server
                        .resolve_server_request(request_id.clone(), result)
                        .await
                    {
                        tracing::warn!(?request_id, %error, "failed to cancel dynamic tool call");
                    }
                }
                Err(error) => {
                    tracing::warn!(?request_id, %error, "failed to serialize dynamic tool response")
                }
            }
        }
        match mode {
            ExitMode::ShutdownFirst | ExitMode::ShutdownAfterInterrupt => {
                // Mark the thread we are explicitly shutting down for exit so
                // its shutdown completion does not trigger agent failover.
                self.pending_shutdown_exit_thread_id =
                    self.active_thread_id.or(self.chat_widget.thread_id());
                if !self.side_threads.is_empty()
                    && tokio::time::timeout(
                        SHUTDOWN_FIRST_EXIT_TIMEOUT,
                        self.shutdown_side_threads(app_server, /*root*/ None),
                    )
                    .await
                    .is_err()
                {
                    tracing::warn!("timed out waiting for side-conversation shutdown");
                }
                if self.pending_shutdown_exit_thread_id.is_some()
                    || self.voice_owner_thread_id().is_some()
                {
                    // This is a UI escape-hatch budget, not a protocol
                    // deadline. A healthy local thread/unsubscribe round trip
                    // should finish comfortably inside two seconds, while a
                    // longer wait makes Ctrl+C feel broken when the app-server
                    // is already wedged.
                    if tokio::time::timeout(
                        SHUTDOWN_FIRST_EXIT_TIMEOUT,
                        self.shutdown_current_thread(app_server),
                    )
                    .await
                    .is_err()
                    {
                        tracing::warn!("timed out waiting for app-server thread shutdown");
                    }
                }
                self.pending_shutdown_exit_thread_id = None;
                AppRunControl::Exit(if mode == ExitMode::ShutdownAfterInterrupt {
                    ExitReason::TurnInterrupted
                } else {
                    ExitReason::UserRequested
                })
            }
            ExitMode::Immediate => {
                self.stop_realtime_conversation(app_server).await;
                self.pending_shutdown_exit_thread_id = None;
                AppRunControl::Exit(ExitReason::UserRequested)
            }
        }
    }

    async fn return_to_agents_after_removal(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) -> Result<()> {
        self.discard_thread_local_state(thread_id).await;
        self.agents_overview.input_states.remove(&thread_id);
        self.agents_overview.dispatched_requests.remove(&thread_id);
        self.reset_for_thread_switch(tui)?;
        self.pending_thread_switch_resets += 1;
        self.app_event_tx
            .send(AppEvent::ResetTranscriptForThreadSwitch);
        self.detach_current_thread_for_navigation(app_server, /*destination*/ None)
            .await;
        self.reset_thread_event_state().await;
        let init = self.chatwidget_init_for_forked_or_resumed_thread(
            tui,
            self.config.clone(),
            /*initial_user_message*/ None,
        );
        self.replace_chat_widget(ChatWidget::new_with_app_event(init));
        self.open_agents_overview(app_server);
        Ok(())
    }

    pub(super) async fn archive_current_thread(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
    ) -> Result<AppRunControl> {
        let Some(thread_id) = self.active_thread_id.or(self.chat_widget.thread_id()) else {
            self.chat_widget
                .add_error_message("A thread must start before it can be archived.".to_string());
            return Ok(AppRunControl::Continue);
        };
        if let Some(reason) = self.terminal_thread_lifecycle_block_reason("archive", thread_id) {
            self.chat_widget.add_error_message(reason);
            return AppRunControl::Continue;
        }
        if self.side_threads.contains_key(&thread_id) {
            self.chat_widget.add_error_message(
                "'/archive' is unavailable in side conversations. Press Ctrl+C to return to the main thread first."
                    .to_string(),
            );
            return Ok(AppRunControl::Continue);
        }

<<<<<<< HEAD
        match app_server.thread_archive(thread_id).await {
            Ok(()) => {
                if self.forget_terminal_operator_pane(thread_id) {
                    self.persist_pane_state();
                }
                AppRunControl::Exit(ExitReason::UserRequested)
||||||| 413492cd6c
        match app_server.thread_archive(thread_id).await {
            Ok(()) => AppRunControl::Exit(ExitReason::UserRequested),
=======
        if !self
            .shutdown_side_threads(app_server, Some(thread_id))
            .await
        {
            return Ok(AppRunControl::Continue);
        }

        let result = async {
            self.stop_voice_for_removed_thread(app_server, thread_id)
                .await?;
            app_server.thread_archive(thread_id).await
        }
        .await;
        Ok(match result {
            Ok(()) => {
                self.track_agents_overview_notification(&ServerNotification::ThreadArchived(
                    codex_app_server_protocol::ThreadArchivedNotification {
                        thread_id: thread_id.to_string(),
                    },
                ));
                self.return_to_agents_after_removal(tui, app_server, thread_id)
                    .await?;
                AppRunControl::Continue
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            Err(err) => {
                self.chat_widget
                    .add_error_message(format!("Failed to archive current thread: {err}"));
                AppRunControl::Continue
            }
        })
    }

    pub(super) async fn delete_current_thread(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
<<<<<<< HEAD
    ) -> AppRunControl {
        if let Some(pane_id) = self
            .claude_panes
            .active_claude_pane_id()
            .map(ToString::to_string)
        {
            match self
                .claude_panes
                .remove_operator_pane(&pane_id, self.config.codex_home.as_ref())
            {
                Ok(removed) => {
                    self.claude_pane_transcript_cells.remove(&pane_id);
                    self.drop_held_claude_task_of_removed_pane(&pane_id);
                    self.persist_pane_state();
                    tracing::info!(
                        pane_id,
                        pane_title = %removed.title,
                        interrupted_running_turn = removed.interrupted_running_turn,
                        "deleted operator-created Claude pane"
                    );
                    return AppRunControl::Exit(ExitReason::UserRequested);
                }
                Err(err) => {
                    self.chat_widget
                        .add_error_message(format!("Failed to delete Claude pane: {err}"));
                    return AppRunControl::Continue;
                }
            }
        }
||||||| 413492cd6c
    ) -> AppRunControl {
=======
    ) -> Result<AppRunControl> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let Some(thread_id) = self.active_thread_id.or(self.chat_widget.thread_id()) else {
            self.chat_widget
                .add_error_message("A thread must start before it can be deleted.".to_string());
            return Ok(AppRunControl::Continue);
        };
        if let Some(reason) = self.terminal_thread_lifecycle_block_reason("delete", thread_id) {
            self.chat_widget.add_error_message(reason);
            return AppRunControl::Continue;
        }
        if self.side_threads.contains_key(&thread_id) {
            self.chat_widget.add_error_message(
                "'/delete' is unavailable in side conversations. Press Ctrl+C to return to the main thread first."
                    .to_string(),
            );
            return Ok(AppRunControl::Continue);
        }

<<<<<<< HEAD
        match app_server.thread_delete(thread_id).await {
            Ok(()) => {
                if self.forget_terminal_operator_pane(thread_id) {
                    self.persist_pane_state();
                }
                AppRunControl::Exit(ExitReason::UserRequested)
||||||| 413492cd6c
        match app_server.thread_delete(thread_id).await {
            Ok(()) => AppRunControl::Exit(ExitReason::UserRequested),
=======
        if !self
            .shutdown_side_threads(app_server, Some(thread_id))
            .await
        {
            return Ok(AppRunControl::Continue);
        }

        let result = async {
            self.stop_voice_for_removed_thread(app_server, thread_id)
                .await?;
            app_server.thread_delete(thread_id).await
        }
        .await;
        Ok(match result {
            Ok(()) => {
                self.track_agents_overview_notification(&ServerNotification::ThreadDeleted(
                    codex_app_server_protocol::ThreadDeletedNotification {
                        thread_id: thread_id.to_string(),
                    },
                ));
                self.return_to_agents_after_removal(tui, app_server, thread_id)
                    .await?;
                AppRunControl::Continue
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            Err(err) => {
                self.chat_widget
                    .add_error_message(format!("Failed to delete current thread: {err}"));
                AppRunControl::Continue
            }
        })
    }
}

const SWITCH_LIST_NOT_REFRESHED: &str = "The account switch applied, but the model list could not be refreshed; switch accounts again or restart to refresh it.";
const SWITCH_NOT_CONFIRMED: &str = "The account switch was not confirmed, so the model list was not refreshed; switch accounts again or restart to refresh it.";
/// #1205: appended to the switch message for the default login when this
/// process left it in place and a login restriction is configured.
pub(super) const DEFAULT_LOGIN_RESTRICTION_CAVEAT: &str = "If your login restrictions (`forced_login_method`, `forced_chatgpt_workspace_id`) rule out the default login, this session cannot use it and its next request is refused with the reason; sign in with `corbanu login`, or pick a named account in /providers.";

/// PF-84: `Some(account)` when moving from `current` to `next` changes the
/// sign-in account the session uses (`Some(None)`: to the default login).
pub(super) fn sign_in_account_switch(
    current: &codex_model_provider_info::ModelProviderInfo,
    next: &codex_model_provider_info::ModelProviderInfo,
) -> Option<Option<codex_login::NamedChatgptAccount>> {
    let account = codex_model_provider::named_sign_in_account(next);
    (codex_model_provider::named_sign_in_account(current) != account).then_some(account)
}

pub(super) fn account_switch_message(
    account: Option<&codex_login::NamedChatgptAccount>,
    config: &Config,
) -> String {
    account_switch_message_for(
        account,
        codex_login::default_login_left_unenforced(),
        login_restrictions_configured(config),
    )
}

/// #1205: whether `forced_login_method` or `forced_chatgpt_workspace_id` is
/// configured; only these can rule out the default login (the login method
/// implied by an API-key provider such as Z.AI does not count).
pub(super) fn login_restrictions_configured(config: &Config) -> bool {
    config.configured_forced_login_method.is_some() || config.forced_chatgpt_workspace_id.is_some()
}

/// `default_login_left_in_place`: this process skipped the startup
/// enforcement of the login restrictions on the default login (#570), so a
/// default login they rule out is hidden rather than used (#609). The caveat
/// is shown only when `login_restrictions_configured` too (#1205).
pub(super) fn account_switch_message_for(
    account: Option<&codex_login::NamedChatgptAccount>,
    default_login_left_in_place: bool,
    login_restrictions_configured: bool,
) -> String {
    let label = match account {
        Some(account) => format!("{} account `{}`", account.provider_id, account.name),
        None => "the default OpenAI login".to_string(),
    };
    let message =
        format!("Connectors, usage and the model list now use {label}; model list refreshed.");
    if account.is_none() && default_login_left_in_place && login_restrictions_configured {
        format!("{message} {DEFAULT_LOGIN_RESTRICTION_CAVEAT}")
    } else {
        message
    }
}

/// D95: why a task the person sent from Spawn status to a pane its parent
/// controls arrives as a message, not as their own input.
fn person_task_to_parent_owned_pane_message(label: &str) -> String {
    format!(
        "{label} is controlled by its parent and takes no direct input, so the task was sent to \
         it as a message (it arrives as a message from another agent, not as your own input)."
    )
}

/// D95: a task the person sends from Spawn status while the kill switch is on.
fn person_task_kill_switch_message(label: &str) -> String {
    format!(
        "Not sent to {label}: the security kill switch is on. A task you send from Spawn status \
         is your own input, as if you typed it in that pane, and typed input is not kept while \
         the switch is on. Send it again after turning the switch off."
    )
}

#[cfg(test)]
mod gpu_notification_tests {
    use std::collections::HashMap;

    use codex_model_provider_info::ModelProviderInfo;
    use codex_provider_auth::ProviderCatalog;

    use super::corbanu_api_continuation_is_current;
    use super::failed_gpu_notification;
    use super::refresh_shared_provider_session_if_idle;
    use super::resolve_shared_provider_selection_model;
    use super::wallet_unlock_continuation_is_current;
    use crate::app_event::WalletUnlockContinuation;
    use crate::onboarding::provider_setup::ProviderSetupAction;
    use crate::onboarding::provider_setup::ProviderSetupEffect;
    use crate::onboarding::provider_setup::ProviderSetupSession;

    #[test]
    fn late_metadata_during_authentication_does_not_resolve_or_present_the_list() {
        let catalog = ProviderCatalog::from_runtime_providers(&HashMap::from([(
            "late-provider".to_string(),
            ModelProviderInfo {
                name: "Late Provider".to_string(),
                env_key: Some("LATE_PROVIDER_KEY".to_string()),
                ..Default::default()
            },
        )]));
        let provider_id = catalog.entries()[0].id.clone();
        let mut session = ProviderSetupSession::from_statuses(&[]);
        session.dispatch(ProviderSetupAction::Begin { provider_id });
        let mut resolved = false;

        let refreshed = refresh_shared_provider_session_if_idle(Some(&mut session), || {
            resolved = true;
            Vec::new()
        });

        assert!(!refreshed);
        assert!(
            !resolved,
            "in-flight metadata must not trigger a full status resolve"
        );
    }

    #[test]
    fn failed_rental_notification_includes_actionable_provider_reason() {
        let message = failed_gpu_notification(
            "gpu-test",
            Some("The selected capacity was claimed. Search again from /gpu."),
        );

        assert!(message.contains("selected capacity was claimed"));
        assert!(message.contains("/gpu"));
    }

    #[test]
    fn shared_provider_selection_uses_active_model_when_config_has_no_model() {
        assert_eq!(
            resolve_shared_provider_selection_model(
                /*configured_model*/ None,
                "active-custom-model",
                "custom"
            ),
            Some("active-custom-model".to_string())
        );
        assert_eq!(
            resolve_shared_provider_selection_model(/*configured_model*/ None, "", "custom"),
            None
        );
    }

    #[test]
    fn deferred_wallet_unlock_events_require_the_exact_active_continuation() {
        let mut session = ProviderSetupSession::from_statuses(&[]);
        session.dispatch(ProviderSetupAction::QueueCorbanu(true));
        let deferred = session
            .dispatch(ProviderSetupAction::Done)
            .effects
            .into_iter()
            .find_map(|effect| match effect {
                ProviderSetupEffect::BeginDeferred(deferred) => Some(deferred),
                _ => None,
            })
            .expect("queued Corbanu setup should defer");
        let continuation = WalletUnlockContinuation::OpenCorbanuApi {
            deferred: Some(deferred.clone()),
        };

        assert!(wallet_unlock_continuation_is_current(
            Some(&deferred),
            &continuation
        ));
        assert!(!wallet_unlock_continuation_is_current(
            /*active*/ None,
            &continuation
        ));
        assert!(corbanu_api_continuation_is_current(
            Some(&deferred),
            Some(&deferred)
        ));
        assert!(!corbanu_api_continuation_is_current(
            Some(&deferred),
            /*deferred*/ None
        ));
        assert!(corbanu_api_continuation_is_current(
            /*active*/ None, /*deferred*/ None
        ));
        let ordinary_continuation = WalletUnlockContinuation::OpenCorbanuApi { deferred: None };
        assert!(!wallet_unlock_continuation_is_current(
            Some(&deferred),
            &ordinary_continuation
        ));
        assert!(wallet_unlock_continuation_is_current(
            /*active*/ None,
            &ordinary_continuation
        ));
    }
}
