//! Session, resume, fork, and subagent selection lifecycle for the TUI app.
//!
//! This module owns the high-level transitions between app-server threads: starting fresh sessions,
//! resuming/forking saved sessions, replacing ChatWidget instances, and maintaining the agent picker
//! cache used for multi-agent navigation.

use super::agent_picker::AGENT_PICKER_VIEW_ID;
use super::app_server_event_targets::ServerNotificationThreadTarget;
use super::app_server_event_targets::server_notification_thread_target;
use super::app_server_event_targets::server_request_thread_id;
use super::*;
use crate::app_server_session::is_server_shutting_down_error;
use crate::app_server_session::source_agent_path;
use crate::app_server_session::thread_blocks_direct_input;
<<<<<<< HEAD
use crate::session_resume::RecordedProviderAccount;
use codex_config::types::ResumeCwdMode;
||||||| 413492cd6c
use codex_config::types::ResumeCwdMode;
=======
use crate::chatwidget::ThreadInputStateRestoreMode;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) enum ThreadAttachPresentation {
    /// A primary thread created by thread/start, without inherited history.
    Fresh,
    /// A fresh thread whose startup composer is already visible.
    FreshWithDraft,
    SessionLineage,
}

pub(super) struct PreparedFreshSession {
    pub(super) started: AppServerStartedThread,
    pub(super) config: Config,
    pub(super) local_settings: crate::local_settings::LocalSettings,
}

/// Reports whether a loaded-thread backfill completed and which descendants already had their
/// liveness metadata refreshed, allowing the picker to skip duplicate `thread/read` requests.
#[derive(Default)]
pub(crate) struct LoadedSubagentBackfill {
    pub(super) completed: bool,
    pub(super) refreshed_thread_ids: HashSet<ThreadId>,
}

impl App {
    pub(super) async fn open_agent_picker(&mut self, app_server: &mut AppServerSession) {
        let backfill = if self.primary_thread_id.is_none() {
            self.backfill_loaded_subagent_threads(app_server).await
        } else {
            LoadedSubagentBackfill::default()
        };
        // V2 subagents are identified by canonical paths observed from activity events or loaded
        // thread metadata. A buffered active turn is positive liveness evidence; a completed
        // snapshot is terminal evidence. An empty store does not clear a successful spawn hint.
        let path_backed_thread_ids: Vec<_> = self
            .agent_navigation
            .ordered_path_backed_subagent_threads(self.primary_thread_id)
            .into_iter()
            .map(|(thread_id, _)| thread_id)
            .collect();
        for thread_id in path_backed_thread_ids.iter().copied() {
            if let Some(channel) = self.thread_event_channels.get(&thread_id)
                && channel.attachment() == ThreadEventAttachment::Live
            {
                let Ok(store) = channel.store.try_lock() else {
                    continue;
                };
                let has_active_turn = store.active_turn_id().is_some();
                let has_terminal_snapshot = store
                    .turns
                    .last()
                    .is_some_and(|turn| !matches!(turn.status, TurnStatus::InProgress));
                drop(store);
                if has_active_turn {
                    self.agent_navigation.mark_running(thread_id);
                } else if has_terminal_snapshot {
                    self.agent_navigation.mark_stopped(thread_id);
                }
            } else if self.primary_thread_id.is_none()
                && !backfill.refreshed_thread_ids.contains(&thread_id)
            {
                self.refresh_agent_picker_thread_liveness(app_server, thread_id)
                    .await;
            }
        }
        let path_backed_threads = self
            .agent_navigation
            .ordered_path_backed_subagent_threads(self.primary_thread_id);
        if !path_backed_threads.is_empty() {
            let running_threads: Vec<_> = path_backed_threads
                .into_iter()
                .filter_map(|(thread_id, entry)| {
                    if !entry.is_running || entry.is_closed {
                        return None;
                    }
                    Some((thread_id, entry.agent_path.as_deref()?.trim().to_string()))
                })
                .collect();
            let mut entries = Vec::new();
            for (thread_id, agent_path) in running_threads {
                let preview = if let Some(channel) = self.thread_event_channels.get(&thread_id) {
                    match channel.store.try_lock() {
                        Ok(store) => {
                            super::agent_status_feed::AgentStatusThreadPreview::from_store(
                                agent_path, &store,
                            )
                        }
                        Err(_) => {
                            super::agent_status_feed::AgentStatusThreadPreview::empty(agent_path)
                        }
                    }
                } else {
                    super::agent_status_feed::AgentStatusThreadPreview::empty(agent_path)
                };
                entries.push(preview);
            }

            self.chat_widget
                .add_to_history(super::agent_status_feed::AgentStatusHistoryCell::new(
                    entries,
                ));
        }

        let mut thread_ids = self.agent_navigation.tracked_thread_ids();
        for thread_id in self.thread_event_channels.keys().copied() {
            if !thread_ids.contains(&thread_id) {
                thread_ids.push(thread_id);
            }
        }
        for thread_id in thread_ids {
            if path_backed_thread_ids.contains(&thread_id)
                || self.side_threads.contains_key(&thread_id)
                || backfill.refreshed_thread_ids.contains(&thread_id)
            {
                continue;
            }
            if self.primary_thread_id.is_none() {
                self.refresh_agent_picker_thread_liveness(app_server, thread_id)
                    .await;
            } else if self.agent_navigation.get(&thread_id).is_none() {
                self.upsert_agent_picker_thread(
                    thread_id, /*agent_nickname*/ None, /*agent_role*/ None,
                    /*is_closed*/ false,
                );
            }
        }

        self.sync_active_agent_label();
        let has_non_primary_agent_thread = self
            .agent_navigation
            .has_non_primary_thread(self.primary_thread_id);
        if !self.config.features.enabled(Feature::Collab) && !has_non_primary_agent_thread {
            if let Some(primary_thread_id) = self.primary_thread_id {
                self.refresh_agent_picker_threads(app_server, primary_thread_id);
            }
            self.chat_widget.open_feature_enable_prompt(Feature::Collab);
            return;
        }

        if self.agent_navigation.is_empty() {
            self.chat_widget
                .add_info_message("No agents available yet.".to_string(), /*hint*/ None);
            return;
        }

        if !self.refresh_agent_picker() {
            let params = self.agent_picker_selection_view_params(/*selected*/ None);
            self.chat_widget.show_selection_view(params);
        }
        if let Some(primary_thread_id) = self.primary_thread_id {
            self.refresh_agent_picker_threads(app_server, primary_thread_id);
        }
    }

    pub(super) fn agent_picker_selection_view_params(
        &mut self,
        selected: Option<ThreadId>,
    ) -> SelectionViewParams {
        let mut threads = self.agent_navigation.visible_threads();
        let pending = &self.agent_navigation.pending_approvals;
        threads.sort_by_key(|(id, _)| !pending.contains(id));
        let mut row_ids = Vec::with_capacity(threads.len());
        let mut initial_selected_idx = None;
        let items: Vec<SelectionItem> = threads
            .into_iter()
            .enumerate()
            .map(|(idx, (thread_id, entry))| {
                row_ids.push(thread_id);
                let is_pending = pending.contains(&thread_id);
                let is_current = self.active_thread_id == Some(thread_id);
                if selected == Some(thread_id)
                    || initial_selected_idx.is_none() && (is_pending || is_current)
                {
                    initial_selected_idx = Some(idx);
                }
                let name = self.thread_label(thread_id);
                let uuid = thread_id.to_string();
<<<<<<< HEAD
                let mut item = SelectionItem {
                    name: name.clone(),
||||||| 413492cd6c
                SelectionItem {
                    name: name.clone(),
=======
                SelectionItem {
                    name: if is_pending {
                        format!("{name} (needs approval)")
                    } else {
                        name.clone()
                    },
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    name_prefix_spans: agent_picker_status_dot_spans(entry.is_closed),
<<<<<<< HEAD
                    // #867: a sub-agent the security kill switch stopped says so.
                    description: Some(if self.agent_navigation.is_kill_switch_stopped(thread_id) {
                        format!("{uuid} · stopped by the security kill switch")
                    } else {
                        uuid.clone()
                    }),
                    is_current: self.active_thread_id == Some(thread_id),
||||||| 413492cd6c
                    description: Some(uuid.clone()),
                    is_current: self.active_thread_id == Some(thread_id),
=======
                    description: Some(uuid.clone()),
                    is_current,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::SelectAgentThread(thread_id));
                    })],
                    dismiss_on_select: true,
                    search_value: Some(format!("{name} {uuid}")),
                    ..Default::default()
                };
                if let Some(reason) = self.unloaded_agent_thread_reason(thread_id) {
                    item.actions.clear();
                    item.is_disabled = true;
                    item.disabled_reason = Some(reason);
                    item.dismiss_on_select = false;
                }
                item
            })
            .collect();
        self.agent_navigation.picker_row_ids = row_ids;

        SelectionViewParams {
            view_id: Some(AGENT_PICKER_VIEW_ID),
            title: Some("Subagents".to_string()),
            subtitle: Some(AgentNavigationState::picker_subtitle()),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            initial_selected_idx,
            ..SelectionViewParams::picker()
        }
    }

    pub(super) fn is_terminal_thread_read_error(err: &color_eyre::Report) -> bool {
        err.chain()
            .any(|cause| cause.to_string().contains("thread not loaded:"))
    }

    pub(super) fn closed_state_for_thread_read_error(
        err: &color_eyre::Report,
        existing_is_closed: Option<bool>,
    ) -> bool {
        Self::is_terminal_thread_read_error(err) || existing_is_closed.unwrap_or(false)
    }

    pub(super) fn can_fallback_from_include_turns_error(err: &color_eyre::Report) -> bool {
        err.chain().any(|cause| {
            let message = cause.to_string();
            message.contains("includeTurns is unavailable before first user message")
                || message.contains("thread/turns/list is unavailable before first user message")
                || message.contains("ephemeral threads do not support includeTurns")
        })
    }

<<<<<<< HEAD
    /// Updates cached picker metadata and then mirrors any visible-label change into the footer.
    ///
    /// These two writes stay paired so the picker rows and contextual footer continue to describe
    /// the same displayed thread after nickname or role updates.
    pub(crate) fn upsert_agent_picker_thread(
||||||| 413492cd6c
    /// Updates cached picker metadata and then mirrors any visible-label change into the footer.
    ///
    /// These two writes stay paired so the picker rows and contextual footer continue to describe
    /// the same displayed thread after nickname or role updates.
    pub(super) fn upsert_agent_picker_thread(
=======
    /// Cache metadata in both the picker and transcript. Callers refresh the UI after a batch.
    pub(super) fn upsert_agent_picker_thread(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        &mut self,
        thread_id: ThreadId,
        agent_nickname: Option<String>,
        agent_role: Option<String>,
        is_closed: bool,
    ) {
        let existing = self.agent_navigation.get(&thread_id);
        let agent_nickname =
            agent_nickname.or_else(|| existing.and_then(|entry| entry.agent_nickname.clone()));
        let agent_role = agent_role.or_else(|| existing.and_then(|entry| entry.agent_role.clone()));
        self.chat_widget.set_collab_agent_metadata(
            thread_id,
            agent_nickname.clone(),
            agent_role.clone(),
        );
        self.agent_navigation
            .upsert(thread_id, agent_nickname, agent_role, is_closed);
    }

    pub(super) fn sync_agent_picker_thread(&mut self, thread_id: ThreadId) {
        if self.agent_navigation.pending_approvals.contains(&thread_id) {
            self.refresh_agent_picker();
        } else {
            self.sync_active_agent_label();
        }
    }

    /// Persists the app-server's authoritative ownership flag and updates the active composer.
    pub(super) fn mark_primary_thread_parent_owned(&mut self, thread_id: ThreadId) {
        self.agent_navigation.mark_parent_owned(thread_id);
        self.chat_widget.set_parent_owned_thread();
    }

    /// Marks a cached picker thread closed and recomputes the contextual footer label.
    ///
    /// Closing a thread is not the same as removing it: users can still inspect finished agent
    /// transcripts, and the stable next/previous traversal order should not collapse around them.
    pub(crate) fn mark_agent_picker_thread_closed(&mut self, thread_id: ThreadId) {
        self.agent_navigation.mark_closed(thread_id);
        self.note_assignment_node_gone(&crate::spawn_orchestration::thread_node_id(thread_id));
        self.sync_active_agent_label();
    }

    pub(super) async fn refresh_agent_picker_thread_liveness(
        &mut self,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) -> bool {
        let existing_entry = self.agent_navigation.get(&thread_id).cloned();
        let has_replay_channel = self.thread_event_channels.contains_key(&thread_id);
        match app_server
            .thread_read(thread_id, /*include_turns*/ false)
            .await
        {
            Ok(thread) => {
                let is_parent_owned = thread_blocks_direct_input(&thread);
                let agent_path = source_agent_path(&thread.source);
                let is_running = matches!(
                    thread.status,
                    codex_app_server_protocol::ThreadStatus::Active { .. }
                );
                let is_closed = matches!(
                    thread.status,
                    codex_app_server_protocol::ThreadStatus::NotLoaded
                );
                self.upsert_agent_picker_thread(
                    thread_id,
                    thread.name.or(thread.agent_nickname).or_else(|| {
                        existing_entry
                            .as_ref()
                            .and_then(|entry| entry.agent_nickname.clone())
                    }),
                    thread.agent_role.or_else(|| {
                        existing_entry
                            .as_ref()
                            .and_then(|entry| entry.agent_role.clone())
                    }),
                    is_closed,
                );
                if is_parent_owned {
                    self.agent_navigation.mark_parent_owned(thread_id);
                }
                self.agent_navigation.set_agent_path(thread_id, agent_path);
                self.sync_agent_picker_thread(thread_id);
                if is_running {
                    self.agent_navigation.mark_running(thread_id);
                } else {
                    self.agent_navigation
                        .set_running(thread_id, /*is_running*/ false);
                }
                true
            }
            Err(err) => {
                if Self::is_terminal_thread_read_error(&err) && !has_replay_channel {
                    self.agent_navigation.remove(thread_id);
                    self.spawn_parent_by_thread.remove(&thread_id);
                    self.spawn_parent_by_node
                        .remove(&crate::spawn_orchestration::thread_node_id(thread_id));
                    self.spawn_parent_reports_by_node
                        .remove(&crate::spawn_orchestration::thread_node_id(thread_id));
                    return false;
                }
                let is_closed = Self::closed_state_for_thread_read_error(
                    &err,
                    existing_entry.as_ref().map(|entry| entry.is_closed),
                );
                if let Some(entry) = existing_entry {
                    self.upsert_agent_picker_thread(
                        thread_id,
                        entry.agent_nickname,
                        entry.agent_role,
                        is_closed,
                    );
                } else {
                    self.upsert_agent_picker_thread(
                        thread_id, /*agent_nickname*/ None, /*agent_role*/ None,
                        is_closed,
                    );
                }
                self.agent_navigation
                    .set_running(thread_id, /*is_running*/ false);
                self.sync_agent_picker_thread(thread_id);
                true
            }
        }
    }

    /// Materializes a live thread into local replay state when the picker knows about it but the
    /// TUI has not cached a local event channel yet.
    ///
    /// Resume-time backfill intentionally avoids creating empty placeholder channels, because those
    /// placeholders make stale `/subagents` entries open blank transcripts. When a user later
    /// selects a still-live discovered thread, attach it on demand with a real resumed snapshot.
    pub(super) async fn attach_live_thread_for_selection(
        &mut self,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) -> Result<bool> {
        let was_external_writer = self
            .thread_event_channels
            .get(&thread_id)
            .is_some_and(|channel| channel.attachment() == ThreadEventAttachment::ExternalWriter);
        if !self.should_attach_live_thread_for_selection(thread_id) {
            return Ok(true);
        }

        let (session, turns, live_attached) = match app_server
            .resume_thread(
<<<<<<< HEAD
                self.config.clone(),
                thread_id,
                self.resume_model_settings(),
                self.resume_permission_settings(),
||||||| 413492cd6c
            .resume_thread(self.config.clone(), thread_id, self.resume_model_settings())
=======
                &self.local_settings,
                self.config.clone(),
                thread_id,
                crate::app_server_session::ResumeModelSettings::PreserveExistingThread,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            )
            .await
        {
            Ok(started) => {
                if let Some(entry) = self.agent_navigation.get(&thread_id).cloned() {
                    self.upsert_agent_picker_thread(
                        thread_id,
                        entry.agent_nickname,
                        entry.agent_role,
                        /*is_closed*/ false,
                    );
                    self.sync_agent_picker_thread(thread_id);
                }
                if started.blocks_direct_input {
                    self.agent_navigation.mark_parent_owned(thread_id);
                }
                (started.session, started.turns, true)
            }
            Err(resume_err) => {
                tracing::warn!(
                    thread_id = %thread_id,
                    error = %resume_err,
                    "failed to resume live thread for selection; falling back to thread/read"
                );
                let mut thread = app_server
                    .thread_read(thread_id, /*include_turns*/ false)
                    .await?;
                if thread.ephemeral {
                    return Err(color_eyre::eyre::eyre!(
                        "Agent thread {thread_id} is not yet available for replay or live attach."
                    ));
                }
                match app_server
                    .hydrate_initial_thread_history(
                        &mut thread,
                        /*turn_cursor*/ None,
                        /*item_cursor*/ None,
                        Some(&self.config),
                        Some(&self.local_settings),
                        crate::app_server_session::HistoryHydrationScope::Initial,
                    )
                    .await
                {
                    Ok(()) => {}
                    Err(err) if Self::can_fallback_from_include_turns_error(&err) => {}
                    Err(err) => return Err(err),
                }
                let turns = thread.turns.clone();
                if turns.is_empty() {
                    // A `thread/read` fallback without turns would create a blank local replay
                    // channel with no live listener attached, which blocks later real re-attach.
                    return Err(color_eyre::eyre::eyre!(
                        "Agent thread {thread_id} is not yet available for replay or live attach."
                    ));
                }
                let mut session = self.session_state_for_thread_read(thread_id, &thread).await;
                // Reads have no settings. Keep this cached thread's permissions rather than
                // inferring them from the conversation that is currently displayed.
                if let Some(channel) = self.thread_event_channels.get(&thread_id)
                    && let Some(cached) = channel.store.lock().await.session.as_ref()
                {
                    session.approval_policy = cached.approval_policy;
                    session.permission_profile = cached.permission_profile.clone();
                    session.active_permission_profile = cached.active_permission_profile.clone();
                    session.approvals_reviewer = cached.approvals_reviewer;
                }
                // `thread/read` can seed replay state, but it does not attach the app-server
                // listener that `thread/resume` establishes, so treat this path as replay-only.
                session.model.clear();
                if was_external_writer {
                    app_server.freeze_viewed_receipt(thread_id);
                }
                (session, turns, false)
            }
        };
        self.agents_overview.activity.remove(&thread_id);
        let recap_progress =
            if live_attached && let Some(channel) = self.thread_event_channels.remove(&thread_id) {
                let store = channel.store.lock().await;
                if let Some(input) = store.input_state.clone() {
                    self.agents_overview.input_states.insert(thread_id, input);
                }
                store.recap_progress()
            } else {
                Default::default()
            };
        let channel = self.ensure_thread_channel(thread_id);
        if !live_attached {
            if was_external_writer {
                channel.mark_external_writer();
            } else {
                channel.mark_replay_only();
            }
        }
        let mut store = channel.store.lock().await;
        store.set_session(session, turns);
        store.merge_recap_progress(recap_progress);
        store.rebase_buffer_after_session_refresh();
        Ok(live_attached)
    }

    /// Replaces the chat widget, carrying composer preferences and the editor yank, and
    /// re-seeding collab metadata from the navigation cache.
    ///
    /// Thread switches reconstruct the `ChatWidget`, which loses the `collab_agent_metadata` map.
    /// This helper copies every known nickname/role from `AgentNavigationState` into the
    /// replacement widget so that replayed collab items render agent names immediately.
    pub(super) fn replace_chat_widget(&mut self, mut chat_widget: ChatWidget) {
        chat_widget.fork_in_progress = self.chat_widget.fork_in_progress;
        self.pending_right_click_paste = None;
        self.config_notification_test_pending = None;
        if !self.chat_widget.realtime_conversation_is_running() {
            self.retain_realtime_replay_state_before_replace();
        }
        self.commit_animation = None;
        // Transfer the last-written terminal title to the replacement widget
        // so it knows what OSC title is currently displayed. Without this, the
        // new widget would redundantly clear and rewrite the same title, causing
        // a visible flicker in some terminals.
        let previous_terminal_title = self.chat_widget.last_terminal_title.take();
        if chat_widget.last_terminal_title.is_none() {
            chat_widget.last_terminal_title = previous_terminal_title;
        }
        chat_widget.config_panel_tui = self.chat_widget.config_panel_tui.clone();
        chat_widget.remote_connection = self.chat_widget.remote_connection.clone();
<<<<<<< HEAD
        // PF-84: a thread switch keeps the session's per-account usage.
        chat_widget.session_account_usage =
            std::mem::take(&mut self.chat_widget.session_account_usage);
||||||| 413492cd6c
=======
        chat_widget.snapshot_local_images = self.app_server_target.uses_remote_workspace();
        chat_widget.set_local_worktree_operations(self.chat_widget.local_worktree_operations);
        chat_widget.windows_sandbox_local_server = self.chat_widget.windows_sandbox_local_server;
        chat_widget.windows_sandbox_host = WindowsSandboxHost::Unknown;
        #[cfg(any(target_os = "windows", test))]
        {
            chat_widget.windows_sandbox_elevated_setup_complete =
                self.chat_widget.windows_sandbox_elevated_setup_complete;
        }
        chat_widget.inherit_backend_banner_state(&mut self.chat_widget);
        chat_widget.inherit_security_setup(&mut self.chat_widget);
        chat_widget
            .mention_usage_mut()
            .inherit(self.chat_widget.mention_usage_mut());
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        for (thread_id, entry) in self.agent_navigation.ordered_threads() {
            chat_widget.set_collab_agent_metadata(
                thread_id,
                entry.agent_nickname.clone(),
                entry.agent_role.clone(),
            );
        }
        chat_widget.restore_kill_buffer_snapshot(self.chat_widget.take_kill_buffer_snapshot());
        crate::markdown_render::preferences::init(chat_widget.local_settings.tui.rendering);
        let mut previous = std::mem::replace(&mut self.chat_widget, chat_widget);
        if previous.realtime_conversation_is_running() {
            previous.park_voice();
            self.background_voice = Some(Box::new(previous));
        }
        self.refresh_agent_picker();
    }

    pub(super) async fn select_agent_thread(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) -> Result<()> {
<<<<<<< HEAD
        self.save_active_claude_pane_transcript();
        let _ = self
            .claude_panes
            .set_active_user_pane(crate::claude_panes::CODEX_MAIN_PANE_ID);
        if self.active_thread_id == Some(thread_id) {
||||||| 413492cd6c
        if self.active_thread_id == Some(thread_id) {
=======
        let pending_profile_on_unavailable_thread = self.thread_unavailable(thread_id)
            && self.pending_server_profiles.contains_key(&thread_id);
        let cached_session = if self.thread_unavailable(thread_id) {
            self.thread_event_channels[&thread_id]
                .store
                .lock()
                .await
                .session
                .clone()
        } else {
            None
        };
        if self.active_thread_id == Some(thread_id) && !self.thread_unavailable(thread_id) {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            return Ok(());
        }
        if self.reject_pending_permission_root_switch() {
            return Ok(());
        }
        if self.windows_sandbox_blocks_thread_switch() {
            self.chat_widget.add_info_message(
                "Finish Windows sandbox setup before switching threads.".to_string(),
                /*hint*/ None,
            );
            return Ok(());
        }
        if self.active_thread_id == Some(thread_id) {
            // Detach the cached receiver before a successful attachment replaces its channel.
            self.store_active_thread_receiver().await;
        }

        // A tracked side thread stays loaded until it is explicitly discarded and already has a
        // replay channel, so another liveness read cannot add anything before selection.
        if !(self.side_threads.contains_key(&thread_id)
            && self.thread_event_channels.contains_key(&thread_id)
            || self
                .refresh_agent_picker_thread_liveness(app_server, thread_id)
                .await)
        {
            self.add_agents_overview_error(format!(
                "Agent thread {thread_id} is no longer available."
            ));
            return Ok(());
        }
        let mut is_replay_only = self
            .agent_navigation
            .get(&thread_id)
            .is_some_and(|entry| entry.is_closed);
        let mut attached_replay_only = false;
        if self.should_attach_live_thread_for_selection(thread_id) {
            match self
                .attach_live_thread_for_selection(app_server, thread_id)
                .await
            {
                Ok(live_attached) => {
                    attached_replay_only = !live_attached;
                    is_replay_only = attached_replay_only;
                }
                Err(_) if self.thread_event_channels.contains_key(&thread_id) => {
                    is_replay_only = true;
                    attached_replay_only = true;
                }
                Err(err) => {
                    self.add_agents_overview_error(format!(
                        "Failed to attach to agent thread {thread_id}: {err}"
                    ));
                    return Ok(());
                }
            }
<<<<<<< HEAD
        } else if !self.thread_event_channels.contains_key(&thread_id) && is_replay_only {
            let message = self
                .unloaded_agent_thread_reason(thread_id)
                .unwrap_or_else(|| format!("Agent thread {thread_id} is no longer available."));
            self.chat_widget.add_error_message(message);
            return Ok(());
||||||| 413492cd6c
        } else if !self.thread_event_channels.contains_key(&thread_id) && is_replay_only {
            self.chat_widget
                .add_error_message(format!("Agent thread {thread_id} is no longer available."));
            return Ok(());
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        let previous_thread_id = self.active_thread_id;
        // Notifications already routed to the active channel must reach the old
        // widget before its transcript state is transferred. Events routed after
        // deactivation are retained separately for that thread.
        if let Some(mut receiver) = self.active_thread_rx.take() {
            while let Ok(event) = receiver.try_recv() {
                self.handle_thread_event_now_recovering_file_changes(event)
                    .await;
            }
            self.active_thread_rx = Some(receiver);
        }
        self.store_active_thread_receiver().await;
        self.active_thread_id = None;
        let Some((receiver, mut snapshot)) = self.activate_thread_for_replay(thread_id).await
        else {
            self.add_agents_overview_error(format!("Agent thread {thread_id} is already active."));
            if let Some(previous_thread_id) = previous_thread_id {
                self.activate_thread_channel(previous_thread_id).await;
            }
            return Ok(());
        };

        let session_verified = self
            .refresh_snapshot_session_if_needed(
                app_server,
                thread_id,
                is_replay_only,
                &mut snapshot,
            )
            .await;
        if !is_replay_only
            && !self.side_threads.contains_key(&thread_id)
            && snapshot.session.as_ref().is_some_and(|session| {
                session
                    .active_permission_profile
                    .as_ref()
                    .is_some_and(|profile| !profile.id.starts_with(':'))
            })
            && !session_verified
        {
            is_replay_only = true;
            attached_replay_only = true;
        }
        if pending_profile_on_unavailable_thread && !is_replay_only && session_verified {
            self.pending_server_profiles.remove(&thread_id);
        }
        // Refreshing can merge restored turns into the store, so recap progress must be read only
        // after the refresh while the activated thread channel is still retained.
        let Some(channel) = self.thread_event_channels.get(&thread_id) else {
            self.add_agents_overview_error(format!(
                "Agent thread {thread_id} is no longer available."
            ));
            return Ok(());
        };
        let recap_progress = {
            let mut store = channel.store.lock().await;
            if let (Some(cached), Some(session)) =
                (cached_session.as_ref(), snapshot.session.as_mut())
            {
                if !pending_profile_on_unavailable_thread {
                    self.restore_runtime_permissions(session, cached);
                }
                store.session = Some(session.clone());
                if !is_replay_only && self.primary_thread_id == Some(thread_id) {
                    self.primary_session_configured = Some(session.clone());
                }
            }
            store.recap_progress()
        };
        snapshot.input_state = snapshot
            .input_state
            .or(self.agents_overview.input_states.remove(&thread_id));

        self.active_thread_id = Some(thread_id);
        self.active_thread_rx = Some(receiver);

        self.recap.note_focus_gained();
        self.recap = recap::RecapState::default();

        if !tui.is_terminal_focused() {
            self.recap.note_focus_lost(Instant::now());
        }
        let now = Instant::now();
        self.recap.seed_from_progress(recap_progress, now);
        self.schedule_recap_check(thread_id, now);

        self.render_thread_snapshot(tui, app_server, thread_id, snapshot, !is_replay_only)?;
        if is_replay_only
            && self
                .thread_event_channels
                .get(&thread_id)
                .is_none_or(|channel| channel.attachment() != ThreadEventAttachment::ExternalWriter)
        {
            self.chat_widget.pause_unavailable_thread();
            let message = if attached_replay_only {
                format!(
                    "Agent thread {thread_id} could not be resumed live. Replaying saved transcript."
                )
            } else {
                format!("Agent thread {thread_id} is closed. Replaying saved transcript.")
            };
            self.chat_widget.add_info_message(message, /*hint*/ None);
        }
        self.refresh_pending_thread_approvals().await;

        Ok(())
    }

    pub(super) fn render_thread_snapshot(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &AppServerSession,
        thread_id: ThreadId,
        snapshot: ThreadEventSnapshot,
        resume_restored_queue: bool,
    ) -> Result<()> {
        let init = self.chatwidget_init_for_forked_or_resumed_thread(
            tui,
            self.config.clone(),
            /*initial_user_message*/ None,
        );
        self.replace_chat_widget(ChatWidget::new_with_app_event(init));
        self.chat_widget
            .set_task_mentions_enabled(app_server.task_tools_available(thread_id));
        self.chat_widget
            .note_rendered_width(tui.terminal.last_known_screen_size.width);
        let external_writer = self
            .thread_event_channels
            .get(&thread_id)
            .is_some_and(|channel| channel.attachment() == ThreadEventAttachment::ExternalWriter);
        if external_writer {
            self.chat_widget.show_external_writer_thread();
        }
        if self.agent_navigation.is_parent_owned(thread_id) {
            self.chat_widget.set_parent_owned_thread();
        }
        self.reset_for_thread_switch(tui)?;
        self.pending_thread_switch_resets += 1;
        self.app_event_tx
            .send(AppEvent::ResetTranscriptForThreadSwitch);
        self.replay_thread_snapshot(snapshot, resume_restored_queue);
        if let Some(thread_id) = self.chat_widget.thread_id()
            && let Some(active) = self
                .chat_widget
                .config_ref()
                .permissions
                .active_permission_profile()
            && self
                .agents_overview
                .selected_permission_profiles
                .get(&thread_id)
                == Some(&active.id)
        {
            self.adopt_server_permissions();
        }
        if external_writer {
            self.chat_widget.show_external_writer_thread();
        }
        Ok(())
    }

    pub(super) fn should_attach_live_thread_for_selection(&self, thread_id: ThreadId) -> bool {
        self.agent_navigation
            .get(&thread_id)
            .is_some_and(|entry| entry.is_closed)
            || self
                .thread_event_channels
                .get(&thread_id)
                .is_none_or(|channel| channel.attachment() != ThreadEventAttachment::Live)
    }

    pub(crate) fn reset_for_thread_switch(&mut self, tui: &mut tui::Tui) -> Result<()> {
        self.reset_transcript_state_after_clear();
        tui.clear_pending_history_lines();
        Ok(tui.clear_for_thread_switch()?)
    }

    pub(super) async fn reset_thread_event_state(&mut self) {
        let voice_owner = self.voice_owner_thread_id();
        let mut previous_pending = std::mem::take(&mut self.pending_app_server_requests);
        // Move retained tasks' approvals to background routing before clearing request bookkeeping.
        for (thread_id, requests) in &mut self.agents_overview.dispatched_requests {
            if let Some(channel) = self.thread_event_channels.get(thread_id) {
                for request in channel.store.lock().await.pending_replay_requests() {
                    if !requests
                        .iter()
                        .any(|pending| pending.request.id() == request.id())
                        && let Some(request) = previous_pending.buffered_request(request)
                    {
                        requests.push(request);
                    }
                }
            }
        }
        if voice_owner.is_some() {
            for (thread_id, channel) in &self.thread_event_channels {
                if Some(*thread_id) != voice_owner && !self.side_threads.contains_key(thread_id) {
                    for request in channel.store.lock().await.pending_replay_requests() {
                        previous_pending.resolve_notification(&thread_id.to_string(), request.id());
                    }
                }
            }
        }
        self.thread_event_listener_tasks.retain(|id, task| {
            if Some(*id) == voice_owner
                || self.side_threads.contains_key(id)
                || self.agents_overview.dispatched_requests.contains_key(id)
            {
                true
            } else {
                task.abort();
                false
            }
        });
        self.thread_event_channels
            .retain(|id, _| Some(*id) == voice_owner || self.side_threads.contains_key(id));
        self.pending_realtime_speech_replay.clear();
        self.pending_realtime_transcript_replay.clear();
        self.realtime_replay_order.clear();
        self.pending_server_profiles.clear();
        self.agents_overview.activity.clear();
        self.agent_navigation.clear();
<<<<<<< HEAD
        self.spawn_parent_by_thread.clear();
        self.spawn_parent_by_node.clear();
        self.spawn_crew = None;
        self.spawn_status_by_thread.clear();
        self.spawn_parent_reports_by_node.clear();
        // #660 follow-up: the waiting tasks belonged to this session's panes.
        self.kill_switch_held_native_tasks.clear();
        self.side_threads.clear();
        // PF-84: a new session starts its per-account usage afresh.
        self.chat_widget.session_account_usage.clear();
||||||| 413492cd6c
        self.side_threads.clear();
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        self.active_thread_id = None;
        self.active_thread_rx = None;
        self.primary_thread_id = None;
        self.last_subagent_backfill_attempt = None;
        self.primary_session_configured = None;
        self.pending_primary_events.clear();
        if voice_owner.is_none() {
            let side_thread_ids: Vec<_> = self.side_threads.keys().copied().collect();
            for thread_id in side_thread_ids {
                if let Some(channel) = self.thread_event_channels.get(&thread_id) {
                    for request in channel.store.lock().await.pending_replay_requests() {
                        if let Some(request) = previous_pending.buffered_request(request) {
                            let _ = self
                                .pending_app_server_requests
                                .note_buffered_request(&request);
                        }
                    }
                }
            }
        } else {
            self.pending_app_server_requests = previous_pending;
        }
        self.pending_startup_thread_start = false;
        self.pending_server_version_notice = None;
        self.chat_widget.set_pending_thread_approvals(Vec::new());
        self.sync_active_agent_label();
    }

    pub(super) async fn handle_startup_thread_started(
        &mut self,
        app_server: &mut AppServerSession,
        result: Result<AppServerStartedThread>,
    ) -> Result<()> {
        if !self.pending_startup_thread_start {
            if let Ok(started) = result {
                let thread_id = started.session.thread_id;
                if let Err(err) = app_server.thread_unsubscribe(thread_id).await {
                    tracing::warn!(
                        thread_id = %thread_id,
                        "failed to unsubscribe stale startup thread: {err}"
                    );
                }
                self.discard_thread_local_state(thread_id).await;
            }
            return Ok(());
        }

        // A refusal proves no thread was created.
        if let Err(error) = &result
            && matches!(
                error.downcast_ref::<TypedRequestError>(),
                Some(TypedRequestError::Server { method, source })
                    if method == "thread/start" && is_server_shutting_down_error(source)
            )
            && self.begin_reconnect_for_rejected_startup()
        {
            return Ok(());
        }

        self.pending_startup_thread_start = false;
        self.chat_widget
            .set_queue_submissions_until_session_configured(/*queue*/ false);
        match result {
            Ok(started) => {
                self.chat_widget.mark_fresh_task_for_sparkle(&started);
                let thread_id = started.session.thread_id;
                if started.task_tools_available {
                    app_server.remember_task_tool_thread(thread_id);
                    self.chat_widget.set_task_mentions_enabled(/*enabled*/ true);
                }
                self.pending_primary_events.retain(|event| match event {
                    ThreadBufferedEvent::Notification(notification) => matches!(
                        server_notification_thread_target(notification),
                        ServerNotificationThreadTarget::Thread(event_thread_id)
                            if event_thread_id == thread_id
                    ),
                    ThreadBufferedEvent::Request(request) => {
                        server_request_thread_id(request) == Some(thread_id)
                    }
                    ThreadBufferedEvent::HistoryEntryResponse(_)
                    | ThreadBufferedEvent::FeedbackSubmission(_) => true,
                });
                let previous_pending = std::mem::take(&mut self.pending_app_server_requests);
                for event in &self.pending_primary_events {
                    if let ThreadBufferedEvent::Request(request) = event {
                        let retained = previous_pending.buffered_request((**request).clone());
                        #[cfg(test)]
                        let retained = retained.or_else(|| {
                            Some(super::app_server_requests::BufferedServerRequest::new(
                                (**request).clone(),
                                &app_server.request_handle(),
                            ))
                        });
                        if let Some(request) = retained {
                            let _ = self
                                .pending_app_server_requests
                                .note_buffered_request(&request);
                        }
                    }
                }
                if started.blocks_direct_input {
                    self.mark_primary_thread_parent_owned(thread_id);
                }
                // Lifecycle notifications may arrive before the thread/start response.
                if !self.config.ephemeral
                    && !self.pending_primary_events.iter().any(|event| {
                        matches!(event, ThreadBufferedEvent::Notification(notification)
                            if matches!(notification.as_ref(),
                                ServerNotification::TurnStarted(_)
                                    | ServerNotification::ThreadClosed(_)
                                    | ServerNotification::ThreadArchived(_)
                                    | ServerNotification::ThreadDeleted(_)))
                    })
                {
                    self.agents_overview
                        .blank_sessions
                        .insert(thread_id, started.clone());
                }
                // A full usage read can finish before thread/start. Apply its cached fallback
                // after attachment but before the initial prompt or queued draft is submitted.
                let recovery_was_pending = self.chat_widget.hold_rate_limit_recovery();
                if self.loader_overrides.user_config_profile.is_none() {
                    self.chat_widget.accept_startup_approval_policy(
                        &mut self.config.permissions.approval_policy,
                        started.session.approval_policy,
                    )?;
                }
                self.enqueue_primary_thread_session(started.session, started.turns)
                    .await?;
<<<<<<< HEAD
                self.restore_codex_user_panes_from_saved_state(app_server)
                    .await;
                self.restore_native_spawn_panes_from_saved_state(app_server)
                    .await;
                self.audit_restored_assignments();
||||||| 413492cd6c
=======
                self.apply_backend_banner_fallback(app_server).await;
                if let Some(notice) = self.pending_server_version_notice.take() {
                    self.chat_widget.add_server_version_warning(notice);
                }
                if !recovery_was_pending {
                    self.chat_widget.finish_rate_limit_recovery();
                }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                self.chat_widget.maybe_send_next_queued_input();
            }
            Err(err) if self.recover_transport_error(&err) => {}
            Err(err) => {
                let warnings = self
                    .transcript_cells
                    .iter()
                    .find_map(|cell| {
                        cell.as_any()
                            .downcast_ref::<history_cell::StartupWarningsCell>()
                    })
                    .filter(|cell| cell.pending_header)
                    .map(|cell| format!("\n\nStartup warnings:\n{}", cell.messages.join("\n")))
                    .unwrap_or_default();
                return Err(color_eyre::eyre::eyre!(
                    "Failed to start a fresh session through the app server: {err}{warnings}"
                ));
            }
        }
        Ok(())
    }

    pub(super) async fn start_fresh_session(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        session_start_source: Option<ThreadStartSource>,
        initial_user_message: Option<crate::chatwidget::UserMessage>,
        new_thread_name: Option<String>,
<<<<<<< HEAD
    ) -> bool {
||||||| 413492cd6c
    ) {
=======
    ) {
        if self.reject_pending_permission_root_switch() {
            if let Some(message) = initial_user_message {
                self.chat_widget.restore_user_message_to_composer(message);
            }
            return;
        }
        // Dismissing the initial agents overview can expose /new before any folder consent.
        // Resolve trust before loading defaults and applying explicit permission choices.
        if self.current_displayed_thread_id().is_none() {
            let mut config = self.chat_widget.config_ref().clone();
            let cwd = config.cwd.to_path_buf();
            if let Err(control) = self
                .confirm_directory_trust(
                    tui,
                    app_server,
                    &mut config,
                    &cwd,
                    crate::onboarding::DirectoryTrustOptions::default(),
                    /*startup_draft*/ None,
                )
                .await
            {
                if matches!(control, AppRunControl::Exit(_)) {
                    self.app_event_tx.send(AppEvent::Exit(ExitMode::Immediate));
                }
                return;
            }
        }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        // Start a fresh in-memory session while preserving resumability via persisted rollout
        // history. If an initial message is provided, `enqueue_primary_thread_session` suppresses it
        // until the new session is configured and any replayed turns have been rendered.
<<<<<<< HEAD
        // A new thread starts without the account a resumed thread recorded.
        self.harness_overrides.inherited_provider_account = None;
        self.refresh_in_memory_config_from_disk_best_effort("starting a new thread")
            .await;
        let model = self.chat_widget.current_model().to_string();
        let mut config = self.fresh_session_config();
||||||| 413492cd6c
        self.refresh_in_memory_config_from_disk_best_effort("starting a new thread")
            .await;
        let model = self.chat_widget.current_model().to_string();
        let mut config = self.fresh_session_config();
=======
        let (mut config, local_settings) = match self.load_new_session_config(app_server).await {
            Ok(config) => config,
            Err(err) => {
                if let Some(message) = initial_user_message {
                    self.chat_widget.restore_user_message_to_composer(message);
                }
                self.chat_widget
                    .add_error_message(format!("Failed to read new session defaults: {err}"));
                tui.frame_requester().schedule_frame();
                return;
            }
        };
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        apply_managed_new_thread_defaults(
            &mut config,
            app_server.managed_new_thread_defaults(),
            &self.cli_kv_overrides,
            &self.harness_overrides,
        );
<<<<<<< HEAD
        let summary = session_summary(
            self.chat_widget.token_usage(),
            self.chat_widget.thread_id(),
            self.chat_widget.thread_name(),
            self.chat_widget.rollout_path().as_deref(),
        );
        self.shutdown_current_thread(app_server).await;
        let tracked_thread_ids: Vec<ThreadId> =
            self.thread_event_channels.keys().copied().collect();
        for thread_id in tracked_thread_ids {
            if let Err(err) = app_server.thread_unsubscribe(thread_id).await {
                tracing::warn!("failed to unsubscribe tracked thread {thread_id}: {err}");
            }
        }
        self.config = config.clone();
        let started_ok = match app_server
            .start_thread_with_session_start_source(&config, session_start_source)
||||||| 413492cd6c
        let summary = session_summary(
            self.chat_widget.token_usage(),
            self.chat_widget.thread_id(),
            self.chat_widget.thread_name(),
            self.chat_widget.rollout_path().as_deref(),
        );
        self.shutdown_current_thread(app_server).await;
        let tracked_thread_ids: Vec<ThreadId> =
            self.thread_event_channels.keys().copied().collect();
        for thread_id in tracked_thread_ids {
            if let Err(err) = app_server.thread_unsubscribe(thread_id).await {
                tracing::warn!("failed to unsubscribe tracked thread {thread_id}: {err}");
            }
        }
        self.config = config.clone();
        match app_server
            .start_thread_with_session_start_source(&config, session_start_source)
=======
        match app_server
            .start_thread_with_session_start_source(
                &self.local_settings,
                &config,
                session_start_source,
                /*remote_cwd_override*/ None,
                /*selected_profile*/ None,
            )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            .await
        {
            Ok(started) => {
                self.adopt_fresh_session(
                    tui,
                    app_server,
                    PreparedFreshSession {
                        started,
<<<<<<< HEAD
                        ThreadAttachPresentation::SessionLineage,
                        initial_user_message,
                    )
                    .await
                {
                    self.chat_widget.add_error_message(format!(
                        "Failed to attach to fresh app-server thread: {err}"
                    ));
                    false
                } else {
                    if let Some(err) = name_error {
                        self.chat_widget.add_error_message(err);
                    }
                    if let Some(summary) = summary {
                        let mut lines: Vec<Line<'static>> = Vec::new();
                        if let Some(usage_line) = summary.usage_line {
                            lines.push(usage_line.into());
                        }
                        if let Some(command) = summary.resume_hint {
                            let spans =
                                vec!["To continue this session, run ".into(), command.cyan()];
                            lines.push(spans.into());
                        }
                        self.chat_widget.add_plain_history_lines(lines);
                    }
                    true
                }
||||||| 413492cd6c
                        ThreadAttachPresentation::SessionLineage,
                        initial_user_message,
                    )
                    .await
                {
                    self.chat_widget.add_error_message(format!(
                        "Failed to attach to fresh app-server thread: {err}"
                    ));
                } else {
                    if let Some(err) = name_error {
                        self.chat_widget.add_error_message(err);
                    }
                    if let Some(summary) = summary {
                        let mut lines: Vec<Line<'static>> = Vec::new();
                        if let Some(usage_line) = summary.usage_line {
                            lines.push(usage_line.into());
                        }
                        if let Some(command) = summary.resume_hint {
                            let spans =
                                vec!["To continue this session, run ".into(), command.cyan()];
                            lines.push(spans.into());
                        }
                        self.chat_widget.add_plain_history_lines(lines);
                    }
                }
=======
                        config,
                        local_settings,
                    },
                    ThreadAttachPresentation::Fresh,
                    initial_user_message,
                    new_thread_name,
                )
                .await;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            Err(err) => {
                self.chat_widget.add_error_message(format!(
                    "Failed to start a fresh session through the app server: {err}"
                ));
<<<<<<< HEAD
                self.config.model = Some(model);
                false
||||||| 413492cd6c
                self.config.model = Some(model);
=======
                if let Some(message) = initial_user_message {
                    self.chat_widget.restore_user_message_to_composer(message);
                }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
        };
        tui.frame_requester().schedule_frame();
        started_ok
    }

    pub(super) async fn adopt_fresh_session(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        prepared: PreparedFreshSession,
        presentation: ThreadAttachPresentation,
        initial_user_message: Option<crate::chatwidget::UserMessage>,
        new_thread_name: Option<String>,
    ) {
        let PreparedFreshSession {
            mut started,
            config,
            local_settings,
        } = prepared;
        if let Some(thread_id) = self.current_displayed_thread_id()
            && let Some(blank) = self.agents_overview.blank_sessions.get_mut(&thread_id)
        {
            if let Some(channel) = self.thread_event_channels.get(&thread_id)
                && let Some(session) = channel.store.lock().await.session.as_ref()
            {
                blank.session = session.clone();
            }
            if let Some(input) = self.chat_widget.capture_thread_input_state() {
                self.agents_overview.input_states.insert(thread_id, input);
            }
        }
        self.detach_current_thread_for_navigation(app_server, Some(started.session.thread_id))
            .await;
        self.local_settings = local_settings;
        self.refresh_server_version_overview_notice(CODEX_CLI_VERSION);
        self.config = config;
        self.remember_launch_permissions();

        let name_error = if let Some(name) = new_thread_name {
            match app_server
                .thread_set_name(started.session.thread_id, name.clone())
                .await
            {
                Ok(()) => {
                    started.session.thread_name = Some(name);
                    None
                }
                Err(err) => Some(format!("Failed to name the new session: {err}")),
            }
        } else {
            None
        };
        let thread_id = started.session.thread_id;
        if !self.config.ephemeral {
            self.agents_overview
                .blank_sessions
                .insert(thread_id, started.clone());
        }
        if let Err(err) = self
            .replace_chat_widget_with_app_server_thread(
                tui,
                started,
                presentation,
                initial_user_message,
            )
            .await
        {
            self.agents_overview.blank_sessions.remove(&thread_id);
            self.chat_widget.add_error_message(format!(
                "Failed to attach to fresh app-server thread: {err}"
            ));
        } else if let Some(err) = name_error {
            self.chat_widget.add_error_message(err);
        }
    }

    pub(super) async fn replace_chat_widget_with_app_server_thread(
        &mut self,
        tui: &mut tui::Tui,
        started: AppServerStartedThread,
        presentation: ThreadAttachPresentation,
        initial_user_message: Option<crate::chatwidget::UserMessage>,
    ) -> Result<()> {
        // Initial messages are for freshly attached primary threads only. Thread switches and
        // resume/fork flows pass `None` so they cannot replay old history and then auto-submit a new
        // user turn by accident.
        if let Some(mut receiver) = self.active_thread_rx.take() {
            while let Ok(event) = receiver.try_recv() {
                self.handle_thread_event_now_recovering_file_changes(event)
                    .await;
            }
            self.active_thread_rx = Some(receiver);
        }
        self.store_active_thread_receiver().await;
        if matches!(presentation, ThreadAttachPresentation::FreshWithDraft) {
            self.reset_transcript_state_after_clear();
            tui.clear_pending_history_lines();
            tui.defer_thread_switch_clear();
        } else {
            self.reset_for_thread_switch(tui)?;
        }
        self.pending_thread_switch_resets += 1;
        self.app_event_tx.send(
            if matches!(presentation, ThreadAttachPresentation::FreshWithDraft) {
                AppEvent::ResetTranscriptForThreadSwitchPreservingScreen
            } else {
                AppEvent::ResetTranscriptForThreadSwitch
            },
        );
        self.reset_thread_event_state().await;
        let init = self.chatwidget_init_for_forked_or_resumed_thread(
            tui,
            self.config.clone(),
            initial_user_message,
        );
        self.replace_chat_widget(ChatWidget::new_with_app_event(init));
        if matches!(
            presentation,
            ThreadAttachPresentation::Fresh | ThreadAttachPresentation::FreshWithDraft
        ) {
            self.chat_widget.mark_fresh_task_for_sparkle(&started);
            // FreshWithDraft inherits its provisional replay at the handoff.
            if matches!(presentation, ThreadAttachPresentation::Fresh) {
                self.chat_widget
                    .empty_state_animation
                    .borrow_mut()
                    .start_fresh();
            }
        }
        self.chat_widget
            .set_task_mentions_enabled(started.task_tools_available);
        self.chat_widget
            .note_rendered_width(tui.terminal.last_known_screen_size.width);
        if started.blocks_direct_input {
            self.mark_primary_thread_parent_owned(started.session.thread_id);
        }
        self.enqueue_primary_thread_session_with_presentation(
            started.session,
            started.turns,
            presentation,
        )
        .await?;
        Ok(())
    }

    /// Fetches all loaded threads from the app server and registers descendants of the primary
    /// thread in the navigation cache and chat widget metadata.
    ///
    /// Called when opening the `/subagents` picker and after resuming a thread so that the picker and
    /// keyboard navigation are pre-populated even if the TUI did not witness the original spawn
    /// events. Fresh and forked threads cannot have pre-existing descendants.
    ///
    /// The loaded-thread list is fetched in full (no pagination) and the spawn tree is walked
    /// by `find_loaded_subagent_threads_for_primary`. Each discovered subagent is registered via
    /// `upsert_agent_picker_thread`, which writes to both `AgentNavigationState` and the
    /// `ChatWidget` metadata map.
    pub(crate) async fn backfill_loaded_subagent_threads(
        &mut self,
        app_server: &mut AppServerSession,
    ) -> LoadedSubagentBackfill {
        let Some(primary_thread_id) = self.primary_thread_id else {
            return LoadedSubagentBackfill::default();
        };

        let loaded_thread_ids = match app_server
            .thread_loaded_list(ThreadLoadedListParams {
                cursor: None,
                limit: None,
            })
            .await
        {
            Ok(response) => response.data,
            Err(err) => {
                tracing::warn!(%err, "failed to list loaded threads for subagent backfill");
                return LoadedSubagentBackfill::default();
            }
        };

        let mut threads = Vec::new();
        let mut had_read_error = false;
        for thread_id in loaded_thread_ids {
            let Ok(thread_id) = ThreadId::from_string(&thread_id) else {
                tracing::warn!("ignoring loaded thread with invalid id during subagent backfill");
                continue;
            };

            if thread_id == primary_thread_id {
                continue;
            }

            match app_server
                .thread_read(thread_id, /*include_turns*/ false)
                .await
            {
                Ok(thread) => threads.push(thread),
                Err(err) => {
                    had_read_error = true;
                    tracing::warn!(thread_id = %thread_id, %err, "failed to read loaded thread");
                }
            }
        }

        let mut refreshed_thread_ids = HashSet::new();
        for thread in find_loaded_subagent_threads_for_primary(threads, primary_thread_id) {
            let agent_path = thread.agent_path;
            let has_live_channel = self
                .thread_event_channels
                .get(&thread.thread_id)
                .is_some_and(|channel| channel.attachment() == ThreadEventAttachment::Live);
            let is_closed = !has_live_channel && thread.is_closed;
            if thread.blocks_direct_input {
                self.agent_navigation.mark_parent_owned(thread.thread_id);
            }
            self.upsert_agent_picker_thread(
                thread.thread_id,
                thread.agent_nickname,
                thread.agent_role,
                is_closed,
            );
            self.agent_navigation
                .set_agent_path(thread.thread_id, agent_path);
            // A live channel can have an empty store after a successful spawn. Only apply server
            // status for channels that would otherwise need another liveness read.
            if !has_live_channel {
                if thread.is_running {
                    self.agent_navigation.mark_running(thread.thread_id);
                } else {
                    self.agent_navigation
                        .set_running(thread.thread_id, /*is_running*/ false);
                }
                refreshed_thread_ids.insert(thread.thread_id);
            }
        }
        self.refresh_agent_picker();

        LoadedSubagentBackfill {
            completed: !had_read_error,
            refreshed_thread_ids,
        }
    }

    /// Returns the adjacent thread id for keyboard navigation, backfilling from the server if the
    /// local cache has no neighbor.
    ///
    /// Tries the fast path first: ask `AgentNavigationState` directly. If it returns `None` (no
    /// adjacent entry exists, typically because the cache was never populated with remote
    /// subagents), performs a full `backfill_loaded_subagent_threads` and retries. This ensures the
    /// first next/previous keypress in a resumed remote session discovers subagents on demand
    /// without requiring the user to wait for a proactive fetch.
    pub(super) async fn adjacent_thread_id_with_backfill(
        &mut self,
        app_server: &mut AppServerSession,
        direction: AgentNavigationDirection,
    ) -> Option<ThreadId> {
        let current_thread = self.current_displayed_thread_id();
        if let Some(thread_id) = self
            .agent_navigation
            .adjacent_thread_id(current_thread, direction)
        {
            return Some(thread_id);
        }

        let primary_thread_id = self.primary_thread_id?;
        if self.last_subagent_backfill_attempt == Some(primary_thread_id) {
            return None;
        }

        if self
            .backfill_loaded_subagent_threads(app_server)
            .await
            .completed
        {
            self.last_subagent_backfill_attempt = Some(primary_thread_id);
        }
        self.agent_navigation
            .adjacent_thread_id(self.current_displayed_thread_id(), direction)
    }

    pub(super) async fn resume_target_session(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        target_session: SessionTarget,
    ) -> Result<AppRunControl> {
<<<<<<< HEAD
        let mut events = tui.event_stream();
        self.resume_target_session_with_events(tui, app_server, target_session, &mut events)
            .await
    }

    pub(super) async fn resume_target_session_with_events(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        target_session: SessionTarget,
        events: &mut (dyn Stream<Item = TuiEvent> + Send + Unpin),
    ) -> Result<AppRunControl> {
||||||| 413492cd6c
=======
        if let Err(error) = target_session.ensure_backend(app_server.backend_id()) {
            self.add_session_picker_error(error.to_string());
            return Ok(AppRunControl::Continue);
        }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        if self.ignore_same_thread_resume(&target_session) {
            self.agents_overview
                .hidden_threads
                .remove(&target_session.thread_id);
            self.repaint_agents_overview();
            tui.frame_requester().schedule_frame();
            return Ok(AppRunControl::Continue);
        }
<<<<<<< HEAD
        if let Some(message) = crate::security::preflight::resume_refusal(&target_session.thread_id)
        {
            self.chat_widget.add_error_message(message);
            tui.frame_requester().schedule_frame();
||||||| 413492cd6c
=======
        if self.windows_sandbox_blocks_thread_switch() {
            self.chat_widget.add_info_message(
                "Finish Windows sandbox setup before switching threads.".to_string(),
                /*hint*/ None,
            );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            return Ok(AppRunControl::Continue);
        }

        if self.reject_pending_permission_root_switch() {
            return Ok(AppRunControl::Continue);
        }
        let prepared = match self
            .prepare_target_session(
                tui,
                app_server,
                &target_session,
                super::resume_preparation::ResumeSource::Active,
            )
            .await
        {
            Ok(prepared) => prepared,
            Err(control) => return Ok(control),
        };
        self.activate_prepared_session(tui, app_server, prepared)
            .await
    }

    pub(super) async fn activate_prepared_session(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        prepared: super::resume_preparation::PreparedResume,
    ) -> Result<AppRunControl> {
        let super::resume_preparation::PreparedResume {
            backend_id,
            config: resume_config,
            local_settings,
            thread: resumed,
            read_only,
            history_notice,
            provider_error,
        } = prepared;
        if backend_id != app_server.backend_id() {
            self.add_session_picker_error(
                "This session belongs to a different backend. Open the session picker again."
                    .to_string(),
            );
            return Ok(AppRunControl::Continue);
        }
<<<<<<< HEAD
        let resume_cwd = if self.app_server_target.uses_remote_workspace() {
            current_cwd.clone()
        } else {
            let outcome = crate::session_resume::resolve_cwd_for_resume_or_fork_with_events(
                tui,
                &self.config,
                self.state_db.as_deref(),
                &target_session,
                CwdPromptAction::Resume,
                crate::session_resume::ResumeCwdContext {
                    current_cwd: &current_cwd,
                    remembered_current_cwd,
                    allow_remember_current: !uses_remote_workspace_or_environment
                        || cwd_override.is_some(),
                    mode: resume_cwd_mode,
                },
                events,
            )
||||||| 413492cd6c
        let resume_cwd = if self.app_server_target.uses_remote_workspace() {
            current_cwd.clone()
        } else {
            let outcome = crate::session_resume::resolve_cwd_for_resume_or_fork(
                tui,
                &self.config,
                self.state_db.as_deref(),
                &target_session,
                CwdPromptAction::Resume,
                crate::session_resume::ResumeCwdContext {
                    current_cwd: &current_cwd,
                    remembered_current_cwd,
                    allow_remember_current: !uses_remote_workspace_or_environment
                        || cwd_override.is_some(),
                    mode: resume_cwd_mode,
                },
            )
=======
        let resumed_thread_id = resumed.session.thread_id;
        let retained_input = ((self.chat_widget.is_external_writer_view()
            || self.chat_widget.has_provider_error())
            && self.chat_widget.thread_id() == Some(resumed_thread_id))
        .then(|| self.chat_widget.capture_thread_input_state())
        .flatten();
        self.detach_current_thread_for_navigation(app_server, Some(resumed_thread_id))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            .await;
<<<<<<< HEAD
            match outcome {
                Err(err) => {
                    self.chat_widget.add_error_message(format!(
                        "Failed to determine working directory for resume: {err}"
                    ));
                    return Ok(AppRunControl::Continue);
                }
                Ok(crate::session_resume::ResolveCwdOutcome::Continue(Some(cwd))) => cwd,
                Ok(crate::session_resume::ResolveCwdOutcome::Continue(None)) => current_cwd.clone(),
                Ok(crate::session_resume::ResolveCwdOutcome::Exit) => {
                    return Ok(AppRunControl::Exit(ExitReason::UserRequested));
                }
            }
        };

        let (config_current_cwd, config_resume_cwd) =
            if self.app_server_target.uses_remote_workspace() {
                let local_config_cwd = self.config.cwd.to_path_buf();
                (local_config_cwd.clone(), local_config_cwd)
            } else {
                (current_cwd, resume_cwd)
            };
        // PF-84 (#545): the thread resumes on the account it recorded, so the
        // config this session keeps for it names that account too.
        let recorded_account = self.resume_target_recorded_account(&target_session).await;
        let previous_thread_account = std::mem::replace(
            &mut self.harness_overrides.inherited_provider_account,
            recorded_account
                .as_ref()
                .map(RecordedProviderAccount::selection),
        );
        let mut resume_config = match self
            .rebuild_config_for_resume_or_fallback(&config_current_cwd, config_resume_cwd)
            .await
        {
            Ok(cfg) => cfg,
            Err(err) => {
                self.harness_overrides.inherited_provider_account = previous_thread_account;
                self.chat_widget.add_error_message(format!(
                    "Failed to rebuild configuration for resume: {err}"
                ));
                return Ok(AppRunControl::Continue);
            }
        };
        if let Some(recorded) = recorded_account.as_ref()
            && let Some(message) = crate::resume_launch::resume_account_error(
                &resume_config,
                recorded,
                codex_model_provider::AccountRecovery::Providers,
                crate::resume_launch::ThreadOpening::Resume,
            )
        {
            self.harness_overrides.inherited_provider_account = previous_thread_account;
            self.chat_widget.add_error_message(message);
            tui.frame_requester().schedule_frame();
            return Ok(AppRunControl::Continue);
        }
        self.apply_runtime_policy_overrides(&mut resume_config);

        let summary = session_summary(
            self.chat_widget.token_usage(),
            self.chat_widget.thread_id(),
            self.chat_widget.thread_name(),
            self.chat_widget.rollout_path().as_deref(),
||||||| 413492cd6c
            match outcome {
                Err(err) => {
                    self.chat_widget.add_error_message(format!(
                        "Failed to determine working directory for resume: {err}"
                    ));
                    return Ok(AppRunControl::Continue);
                }
                Ok(crate::session_resume::ResolveCwdOutcome::Continue(Some(cwd))) => cwd,
                Ok(crate::session_resume::ResolveCwdOutcome::Continue(None)) => current_cwd.clone(),
                Ok(crate::session_resume::ResolveCwdOutcome::Exit) => {
                    return Ok(AppRunControl::Exit(ExitReason::UserRequested));
                }
            }
        };

        let (config_current_cwd, config_resume_cwd) =
            if self.app_server_target.uses_remote_workspace() {
                let local_config_cwd = self.config.cwd.to_path_buf();
                (local_config_cwd.clone(), local_config_cwd)
            } else {
                (current_cwd, resume_cwd)
            };
        let mut resume_config = match self
            .rebuild_config_for_resume_or_fallback(&config_current_cwd, config_resume_cwd)
            .await
        {
            Ok(cfg) => cfg,
            Err(err) => {
                self.chat_widget.add_error_message(format!(
                    "Failed to rebuild configuration for resume: {err}"
                ));
                return Ok(AppRunControl::Continue);
            }
        };
        self.apply_runtime_policy_overrides(&mut resume_config);

        let summary = session_summary(
            self.chat_widget.token_usage(),
            self.chat_widget.thread_id(),
            self.chat_widget.thread_name(),
            self.chat_widget.rollout_path().as_deref(),
=======
        self.local_settings = local_settings;
        self.refresh_server_version_overview_notice(CODEX_CLI_VERSION);
        self.config = resume_config;
        tui.set_notification_settings(
            self.local_settings.tui.notification_settings.method,
            self.local_settings.tui.notification_settings.condition,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        );
<<<<<<< HEAD
        let model_settings = self.resume_model_settings();
        match app_server
            .resume_thread(
                resume_config.clone(),
                target_session.thread_id,
                model_settings,
                self.resume_permission_settings(),
||||||| 413492cd6c
        match app_server
            .resume_thread(
                resume_config.clone(),
                target_session.thread_id,
                self.resume_model_settings(),
=======
        self.file_search
            .update_search_dir(self.config.cwd.to_path_buf());
        match self
            .replace_chat_widget_with_app_server_thread(
                tui,
                resumed,
                ThreadAttachPresentation::SessionLineage,
                /*initial_user_message*/ None,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            )
            .await
        {
<<<<<<< HEAD
            Ok(resumed) => {
                let resumed_thread_id = resumed.session.thread_id;
                let resumed_session = resumed.session.clone();
                // D172 (#1287): an account selected for this session wins
                // over the recorded one; say so, from the account the app
                // server resumed the thread on.
                let account_notice = recorded_account.as_ref().and_then(|recorded| {
                    crate::resume_launch::resume_account_notice(
                        &resume_config,
                        recorded,
                        resumed_session.provider_account.as_deref(),
                    )
                });
                let remember_model = !resumed.blocks_direct_input;
                self.shutdown_current_thread(app_server).await;
                apply_persisted_resume_runtime(
                    &mut resume_config,
                    Some(resumed.session.model.as_str()),
                    resumed.session.model_provider_id.as_str(),
                    resumed.session.reasoning_effort.clone(),
                );
                resume_config.service_tier = resumed.session.service_tier.clone();
                self.config = resume_config;
                tui.set_notification_settings(
                    self.config.tui_notifications.method,
                    self.config.tui_notifications.condition,
                );
                self.file_search
                    .update_search_dir(self.config.cwd.to_path_buf());
                match self
                    .replace_chat_widget_with_app_server_thread(
                        tui,
                        resumed,
                        ThreadAttachPresentation::SessionLineage,
                        /*initial_user_message*/ None,
                    )
                    .await
||||||| 413492cd6c
            Ok(resumed) => {
                let resumed_thread_id = resumed.session.thread_id;
                self.shutdown_current_thread(app_server).await;
                self.config = resume_config;
                tui.set_notification_settings(
                    self.config.tui_notifications.method,
                    self.config.tui_notifications.condition,
                );
                self.file_search
                    .update_search_dir(self.config.cwd.to_path_buf());
                match self
                    .replace_chat_widget_with_app_server_thread(
                        tui,
                        resumed,
                        ThreadAttachPresentation::SessionLineage,
                        /*initial_user_message*/ None,
                    )
                    .await
=======
            Ok(()) => {
                if let Some(mut input) = retained_input {
                    // A successful resume is the authority for clearing the old rejection.
                    input.provider_error = None;
                    self.chat_widget.restore_thread_input_state(
                        Some(input),
                        ThreadInputStateRestoreMode {
                            preserve_in_flight_turn: false,
                        },
                    );
                }
                if let Some(error) = provider_error {
                    self.chat_widget
                        .show_provider_error(error, /*rejected_message_id*/ None);
                } else if read_only {
                    self.ensure_thread_channel(resumed_thread_id)
                        .mark_external_writer();
                    self.chat_widget.show_external_writer_thread();
                }
                if let Some(notice) = history_notice {
                    self.chat_widget
                        .add_info_message(notice.to_string(), /*hint*/ None);
                }
                self.backfill_loaded_subagent_threads(app_server).await;
                if matches!(
                    self.runtime_approval_policy_override,
                    Some(RuntimeApprovalPolicyOverride::Restored(_))
                ) {
                    self.runtime_approval_policy_override = None;
                }
                if self
                    .runtime_permission_profile_override
                    .as_ref()
                    .is_some_and(|profile| {
                        profile.turn_override == RuntimePermissionProfileTurnOverride::Preserve
                    })
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                {
<<<<<<< HEAD
                    Ok(()) => {
                        if remember_model
                            && let Err(error) = super::resume_defaults::remember_resumed_model(
                                app_server,
                                &self.config,
                                model_settings,
                                &resumed_session,
                            )
                            .await
                        {
                            self.chat_widget.add_error_message(format!(
                                "Session restored, but failed to save its startup model: {error}"
                            ));
                        }
                        self.backfill_loaded_subagent_threads(app_server).await;
                        if let Some(notice) = account_notice {
                            self.chat_widget.add_info_message(notice, /*hint*/ None);
                        }
                        if let Some(summary) = summary {
                            let mut lines: Vec<Line<'static>> = Vec::new();
                            if let Some(usage_line) = summary.usage_line {
                                lines.push(usage_line.into());
                            }
                            if let Some(command) = summary.resume_hint {
                                let spans =
                                    vec!["To continue this session, run ".into(), command.cyan()];
                                lines.push(spans.into());
                            }
                            self.chat_widget.add_plain_history_lines(lines);
                        }
                        self.maybe_prompt_resume_paused_goal_after_resume(
                            app_server,
                            resumed_thread_id,
                        )
||||||| 413492cd6c
                    Ok(()) => {
                        self.backfill_loaded_subagent_threads(app_server).await;
                        if let Some(summary) = summary {
                            let mut lines: Vec<Line<'static>> = Vec::new();
                            if let Some(usage_line) = summary.usage_line {
                                lines.push(usage_line.into());
                            }
                            if let Some(command) = summary.resume_hint {
                                let spans =
                                    vec!["To continue this session, run ".into(), command.cyan()];
                                lines.push(spans.into());
                            }
                            self.chat_widget.add_plain_history_lines(lines);
                        }
                        self.maybe_prompt_resume_paused_goal_after_resume(
                            app_server,
                            resumed_thread_id,
                        )
=======
                    self.runtime_permission_profile_override = None;
                }
                if !read_only {
                    self.replay_agents_overview_requests(app_server, resumed_thread_id)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                        .await;
                }
                if !read_only {
                    self.maybe_prompt_resume_paused_goal_after_resume(
                        app_server,
                        resumed_thread_id,
                    )
                    .await;
                }
            }
<<<<<<< HEAD
            Err(err) => {
                self.harness_overrides.inherited_provider_account = previous_thread_account;
                let path_display = target_session.display_label();
                self.chat_widget.add_error_message(format!(
                    "Failed to resume session from {path_display}: {err}"
                ));
            }
||||||| 413492cd6c
            Err(err) => {
                let path_display = target_session.display_label();
                self.chat_widget.add_error_message(format!(
                    "Failed to resume session from {path_display}: {err}"
                ));
            }
=======
            Err(err) => return Err(err),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }

        Ok(AppRunControl::Continue)
    }

    /// PF-84 (#545): the named account `target` ran on (see
    /// `crate::resume_launch::resume_recorded_provider_account`).
    async fn resume_target_recorded_account(
        &self,
        target: &SessionTarget,
    ) -> Option<RecordedProviderAccount> {
        crate::resume_launch::resume_recorded_provider_account(
            &crate::resume_picker::SessionSelection::Resume(target.clone()),
            &self.config,
            self.app_server_target.uses_remote_workspace(),
            self.resume_model_settings(),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_thread_read_error_detection_matches_not_loaded_errors() {
        let err = color_eyre::eyre::eyre!(
            "thread/read failed during TUI session lookup: thread/read failed: thread not loaded: thr_123"
        );

        assert!(App::is_terminal_thread_read_error(&err));
    }

    #[test]
    fn terminal_thread_read_error_detection_ignores_transient_failures() {
        let err = color_eyre::eyre::eyre!(
            "thread/read failed during TUI session lookup: thread/read transport error: broken pipe"
        );

        assert!(!App::is_terminal_thread_read_error(&err));
    }

    #[test]
    fn closed_state_for_thread_read_error_preserves_live_state_without_cache_on_transient_error() {
        let err = color_eyre::eyre::eyre!(
            "thread/read failed during TUI session lookup: thread/read transport error: broken pipe"
        );

        assert!(!App::closed_state_for_thread_read_error(
            &err, /*existing_is_closed*/ None
        ));
    }

    #[test]
    fn closed_state_for_thread_read_error_marks_terminal_uncached_threads_closed() {
        let err = color_eyre::eyre::eyre!(
            "thread/read failed during TUI session lookup: thread/read failed: thread not loaded: thr_123"
        );

        assert!(App::closed_state_for_thread_read_error(
            &err, /*existing_is_closed*/ None
        ));
    }

    #[test]
    fn include_turns_fallback_detection_handles_unmaterialized_and_ephemeral_threads() {
        let unmaterialized = color_eyre::eyre::eyre!(
            "thread/read failed during TUI session lookup: thread/read failed: thread thr_123 is not materialized yet; includeTurns is unavailable before first user message"
        );
        let ephemeral = color_eyre::eyre::eyre!(
            "thread/read failed during TUI session lookup: thread/read failed: ephemeral threads do not support includeTurns"
        );

        assert!(App::can_fallback_from_include_turns_error(&unmaterialized));
        assert!(App::can_fallback_from_include_turns_error(&ephemeral));
    }
}
