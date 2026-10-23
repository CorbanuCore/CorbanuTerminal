//! App integration: pane pickers, turn submission, and display synchronization.

use std::sync::Arc;

use anyhow::Result;
use anyhow::anyhow;

use crate::app::App;
use crate::app_command::AppCommand;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionShortcutAction;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::custom_prompt_view::CustomPromptView;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;
use crate::key_hint;
use crate::spawn_orchestration::SpawnRole;
use crate::spawn_orchestration::thread_node_id;
use crate::tui;
use codex_product_brand::MAIN_PANE_TITLE;
use codex_product_brand::native_pane_title;
use codex_protocol::ThreadId;
use crossterm::event::KeyCode;

use super::command_plan::claude_pane_title;
use super::command_plan::compose_claude_pane_prompt;
use super::command_plan::prompt_from_user_turn;
use super::execution::run_prepared_claude_turn;
use super::pane::ClaudePaneStatus;
use super::pane::ClaudePaneUsageStatus;
use super::pane::PaneLayoutState;
use super::progress::truncate_for_display;
use super::provider::ClaudeProviderProfileKind;
use super::registry::CODEX_MAIN_PANE_ID;
use super::registry::ClaudePaneRegistry;
use super::registry::PANE_LAYOUT_VERSION;
use super::registry::load_pane_layout;
use super::registry::persist_pane_layout;
use super::turn_types::ClaudePaneTurnOutput;
use super::turn_types::ClaudePaneTurnProgress;
impl App {
    pub(crate) fn is_operator_owned_codex_user_pane(
        &self,
        thread_id: ThreadId,
        entry: &crate::multi_agents::AgentPickerThreadEntry,
    ) -> bool {
        let is_bound_nazgul = self
            .spawn_nazgul_pane_id
            .as_deref()
            .and_then(crate::spawn_orchestration::node_id_thread)
            == Some(thread_id);
        Some(thread_id) != self.primary_thread_id
            && (!self.is_managed_spawn_crew_thread(thread_id) || is_bound_nazgul)
            && !self.agent_navigation.is_parent_owned(thread_id)
            && entry
                .agent_role
                .as_deref()
                .is_none_or(|role| role == "default")
    }

    /// Reject native thread lifecycle commands when the visible pane is owned by another
    /// lifecycle authority. Falling back from an external Claude pane to the last native thread
    /// can otherwise archive/delete Main, while deleting one managed `/spawn` member directly
    /// leaves the durable crew projection pointing at a removed subtree.
    pub(crate) fn terminal_thread_lifecycle_block_reason(
        &self,
        command: &str,
        thread_id: ThreadId,
    ) -> Option<String> {
        if let Some(title) = self.claude_panes.active_claude_pane_title() {
            return Some(format!(
                "'/{command}' cannot target Claude pane `{title}`. Select Main or an operator-created Corbanu Terminal pane first; Claude pane removal needs its own pane lifecycle action."
            ));
        }
        if Some(thread_id) != self.primary_thread_id && self.is_managed_spawn_crew_thread(thread_id)
        {
            return Some(format!(
                "'/{command}' cannot remove one managed /spawn member independently. Switch to Main; the crew must be removed through its managed lifecycle so the native graph and pane layout stay consistent."
            ));
        }
        if Some(thread_id) != self.primary_thread_id
            && self.agent_navigation.is_parent_owned(thread_id)
        {
            return Some(format!(
                "'/{command}' cannot target a parent-controlled task worker. Switch to Main or an operator-created Corbanu Terminal pane first."
            ));
        }
        None
    }

    pub(crate) async fn open_pane_picker(&mut self, app_server: &mut AppServerSession) {
        self.restore_codex_user_panes_from_saved_state(app_server)
            .await;
        self.backfill_loaded_subagent_threads(app_server).await;
        self.restore_native_spawn_panes_from_saved_state(app_server)
            .await;

        let items = self.pane_picker_items();

        self.chat_widget.show_selection_view(SelectionViewParams {
            title: Some("Panes".to_string()),
            subtitle: Some("Switch user panes or inspect the managed /spawn crew.".to_string()),
            footer_hint: Some("Enter select | F2 rename | type to search".into()),
            items,
            is_searchable: true,
            search_placeholder: Some("Search panes and crew".to_string()),
            ..Default::default()
        });
    }

    pub(crate) fn pane_picker_items(&self) -> Vec<SelectionItem> {
        let mut items = Vec::new();
        items.push(section_item("User Panes"));
        items.extend(self.user_pane_items());
        items.push(section_item("Create User Pane"));
        items.extend(new_pane_items());
        items.push(section_item("Managed Crew (/spawn)"));
        items.extend(self.spawn_tree_items(/*show_task_actions*/ false));
        items
    }

    /// Removes a successfully archived/deleted operator pane from the owning Main layout.
    ///
    /// The app-server owns the rollout lifecycle, while the TUI layout owns the operator-pane
    /// membership projection. A successful terminal lifecycle request must update both; otherwise
    /// the next Main restart tries to resume an intentionally archived thread and renders a ghost
    /// disabled row. Managed crew and parent-controlled workers have separate lifecycle authority
    /// and are deliberately not handled here.
    pub(crate) fn forget_terminal_operator_pane(&mut self, thread_id: ThreadId) -> bool {
        let is_operator_pane = self
            .agent_navigation
            .get(&thread_id)
            .is_some_and(|entry| self.is_operator_owned_codex_user_pane(thread_id, entry));
        if !is_operator_pane {
            return false;
        }

        self.agent_navigation.remove(thread_id);
        if self.active_thread_id == Some(thread_id) {
            self.active_thread_id = self.primary_thread_id;
        }
        true
    }

    #[allow(dead_code)]
    pub(crate) async fn restore_pane_layout_for_thread(
        &mut self,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) {
        let thread_id_string = thread_id.to_string();
        let restored_pane_layout =
            load_pane_layout(self.config.codex_home.as_ref(), Some(&thread_id_string));

        if let Some(layout) = restored_pane_layout.as_ref() {
            for thread_id in layout
                .codex_user_pane_ids
                .iter()
                .filter_map(|thread_id| ThreadId::from_string(thread_id).ok())
            {
                self.upsert_agent_picker_thread(
                    thread_id, /*agent_nickname*/ None, /*agent_role*/ None,
                    /*is_closed*/ false,
                );
            }
        }

        self.spawn_parent_by_node = restored_pane_layout
            .as_ref()
            .map(|layout| {
                layout
                    .spawn_parent_by_node
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .unwrap_or_default();
        self.spawn_parent_by_thread.clear();
        self.claude_panes = ClaudePaneRegistry::restore_from_disk(
            self.config.codex_home.as_ref(),
            restored_pane_layout.as_ref(),
        );
        self.spawn_nazgul_pane_id = restored_pane_layout
            .as_ref()
            .and_then(|layout| layout.spawn_nazgul_pane_id.clone())
            .filter(|pane_id| self.valid_restored_nazgul_binding(pane_id));
        self.spawn_nazgul_rebind_required = restored_pane_layout
            .as_ref()
            .is_some_and(|layout| layout.spawn_nazgul_rebind_required);
        self.claude_pane_transcript_cells.clear();
        self.seed_restored_claude_pane_transcripts();
        self.restore_codex_user_panes_from_saved_state(app_server)
            .await;
        self.restore_native_spawn_panes_from_saved_state(app_server)
            .await;
        self.show_restored_active_claude_pane();
    }

    /// Reattach operator-owned native panes recorded in the owning Main layout.
    ///
    /// `/panes` uses independent `thread/start` sessions so direct input remains enabled. They are
    /// intentionally absent from the Core spawn hierarchy, which means generic descendant
    /// backfill cannot rediscover them after the in-process app server exits. The layout is the
    /// membership authority; each listed thread is resumed through the normal app-server boundary
    /// and keeps its original rollout, model runtime, name, and input ownership.
    pub(crate) async fn restore_codex_user_panes_from_saved_state(
        &mut self,
        app_server: &mut AppServerSession,
    ) {
        let thread_ids = self.restorable_operator_owned_codex_user_pane_ids();

        for thread_id in thread_ids {
            if self.primary_thread_id == Some(thread_id) || self.thread_has_live_session(thread_id)
            {
                continue;
            }

            let existing_nickname = self
                .agent_navigation
                .get(&thread_id)
                .and_then(|entry| entry.agent_nickname.clone());
            let persisted_nickname = match self.state_db.as_ref() {
                Some(state_db) => state_db
                    .get_thread(thread_id)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|metadata| metadata.agent_nickname)
                    .or(existing_nickname.clone()),
                None => existing_nickname.clone(),
            };
            let discovered_nickname = match app_server
                .thread_read(thread_id, /*include_turns*/ false)
                .await
            {
                Ok(thread) => thread
                    .name
                    .or(thread.agent_nickname)
                    .or(persisted_nickname.clone()),
                Err(err) => {
                    tracing::warn!(
                        %thread_id,
                        error = %err,
                        "persisted operator-owned pane was not readable before resume"
                    );
                    persisted_nickname.clone()
                }
            };

            match app_server
                .resume_thread(
                    self.config.clone(),
                    thread_id,
                    crate::app_server_session::ResumeModelSettings::RestoreFromThread,
                    crate::app_server_session::ResumePermissionSettings::RestoreFromThread,
                )
                .await
            {
                Ok(started) if !started.blocks_direct_input => {
                    let nickname = started.session.thread_name.clone().or(discovered_nickname);
                    let model = started.session.model.clone();
                    let channel = self.ensure_thread_channel(thread_id);
                    channel.mark_live();
                    channel.set_session(started.session, started.turns).await;
                    self.upsert_agent_picker_thread(
                        thread_id, nickname, /*agent_role*/ None, /*is_closed*/ false,
                    );
                    self.agent_navigation.set_model(thread_id, Some(model));
                }
                Ok(_) => {
                    tracing::error!(
                        %thread_id,
                        "persisted operator-owned pane resumed as parent-controlled"
                    );
                    self.upsert_agent_picker_thread(
                        thread_id,
                        discovered_nickname,
                        /*agent_role*/ None,
                        /*is_closed*/ true,
                    );
                    self.chat_widget.add_error_message(format!(
                        "Corbanu Terminal pane {thread_id} restored with direct input disabled; refusing to present it as usable."
                    ));
                }
                Err(err) => {
                    tracing::warn!(
                        %thread_id,
                        error = %err,
                        "failed to resume persisted operator-owned pane"
                    );
                    self.upsert_agent_picker_thread(
                        thread_id,
                        discovered_nickname,
                        /*agent_role*/ None,
                        /*is_closed*/ true,
                    );
                    self.chat_widget.add_error_message(format!(
                        "Failed to resume Corbanu Terminal pane {thread_id}: {err}"
                    ));
                }
            }
        }
        self.persist_pane_state();
    }

    /// Returns only persisted operator panes that still need an automatic attach attempt.
    ///
    /// A failed restore is marked closed and remains visible as a disabled row. Retrying it every
    /// time `/panes` opens both repeats an expensive failing app-server request and can keep the
    /// healthy picker behind the same error. A later liveness refresh may explicitly reopen the
    /// entry, while user-driven recovery can operate through the session/resume surfaces.
    pub(crate) fn restorable_operator_owned_codex_user_pane_ids(&self) -> Vec<ThreadId> {
        let mut thread_ids = self
            .agent_navigation
            .ordered_threads()
            .into_iter()
            .filter(|(thread_id, entry)| {
                !entry.is_closed
                    && self.is_operator_owned_codex_user_pane(*thread_id, entry)
                    && self.primary_thread_id != Some(*thread_id)
                    && !self.thread_has_live_session(*thread_id)
            })
            .map(|(thread_id, _)| thread_id)
            .collect::<Vec<_>>();
        thread_ids.sort_by_key(ToString::to_string);
        thread_ids
    }

    #[allow(dead_code)]
    fn valid_restored_nazgul_binding(&self, pane_id: &str) -> bool {
        pane_id == CODEX_MAIN_PANE_ID
            || crate::spawn_orchestration::node_id_thread(pane_id).is_some()
            || self
                .claude_panes
                .panes()
                .iter()
                .any(|pane| pane.id == pane_id)
    }

    pub(crate) fn open_claude_pane_profile_picker(&mut self) {
        if let Some(reason) = crate::security::level::external_agent_block_reason() {
            self.chat_widget.add_error_message(reason);
            return;
        }
        let mut items = Vec::new();
        for profile in ClaudeProviderProfileKind::creation_options() {
            let profile_config = profile.profile();
            let kind = *profile;
            items.push(SelectionItem {
                name: format!("+ {}", profile.status_model_label()),
                description: Some(profile_config.description.to_string()),
                search_value: Some(format!(
                    "{} {}",
                    profile_config.title, profile_config.description
                )),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenClaudePaneNamePrompt { profile: kind });
                })],
                dismiss_on_select: true,
                ..Default::default()
            });
        }

        self.chat_widget.show_selection_view(SelectionViewParams {
            title: Some("New Claude Pane".to_string()),
            subtitle: Some("Choose the provider route for Claude Code headless.".to_string()),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            search_placeholder: Some("Search Claude providers".to_string()),
            ..Default::default()
        });
    }

    pub(crate) fn open_claude_pane_name_prompt(&mut self, profile: ClaudeProviderProfileKind) {
        let tx = self.app_event_tx.clone();
        let initial_name = profile.profile().title.to_string();
        let view = CustomPromptView::new(
            "Name Claude pane".to_string(),
            "Pane display name".to_string(),
            initial_name,
            Some(profile.status_model_label()),
            Box::new(move |name: String| {
                tx.send(AppEvent::CreateClaudePane {
                    profile,
                    display_name: Some(name.trim().to_string()),
                });
            }),
        );
        self.chat_widget.show_custom_prompt_view(view);
    }

    pub(crate) fn open_codex_pane_model_picker(&mut self) {
        let default_model = self.native_spawn_default_model();
        if std::env::var("PFTERMINAL_ORCHESTRATE_QA").as_deref() == Ok("1") {
            self.open_codex_pane_name_prompt(
                default_model,
                Some(self.config.model_provider_id.clone()),
                /*effort*/ None,
            );
            return;
        }
        let presets = self
            .chat_widget
            .model_catalog()
            .try_list_models()
            .unwrap_or_default();
        self.chat_widget.open_all_models_popup_for_purpose(
            presets,
            crate::chatwidget::ModelSelectionPurpose::CodexPane { default_model },
        );
    }

    pub(crate) fn open_codex_pane_name_prompt(
        &mut self,
        model: String,
        provider: Option<String>,
        effort: Option<codex_protocol::openai_models::ReasoningEffort>,
    ) {
        let tx = self.app_event_tx.clone();
        let initial_name = self.next_codex_pane_nickname();
        let view = CustomPromptView::new(
            "Name Corbanu Terminal pane".to_string(),
            "Pane display name".to_string(),
            initial_name,
            Some(format!("Model: {model}")),
            Box::new(move |name: String| {
                tx.send(AppEvent::CreateCodexPane {
                    model: model.clone(),
                    provider: provider.clone(),
                    effort: effort.clone(),
                    display_name: Some(name.trim().to_string()),
                });
            }),
        );
        self.chat_widget.show_custom_prompt_view(view);
    }

    pub(crate) fn open_rename_codex_pane_prompt(&mut self, thread_id: ThreadId) {
        let tx = self.app_event_tx.clone();
        let initial_name = self
            .agent_navigation
            .get(&thread_id)
            .and_then(|entry| entry.agent_nickname.clone())
            .unwrap_or_else(|| {
                if self.primary_thread_id == Some(thread_id) {
                    "Main".to_string()
                } else {
                    short_thread_id(thread_id)
                }
            });
        let view = CustomPromptView::new(
            "Rename Corbanu Terminal pane".to_string(),
            "Pane display name".to_string(),
            initial_name,
            /*context_label*/ None,
            Box::new(move |name: String| {
                tx.send(AppEvent::RenameCodexPane {
                    thread_id,
                    name: name.trim().to_string(),
                });
            }),
        );
        self.chat_widget.show_custom_prompt_view(view);
    }

    pub(crate) fn open_rename_claude_pane_prompt(&mut self, pane_id: String) {
        let Some(initial_name) = self
            .claude_panes
            .panes()
            .iter()
            .find(|pane| pane.id == pane_id)
            .map(|pane| pane.title.clone())
        else {
            self.chat_widget
                .add_error_message(format!("No Claude pane found for `{pane_id}`."));
            return;
        };
        let tx = self.app_event_tx.clone();
        let view = CustomPromptView::new(
            "Rename Claude pane".to_string(),
            "Pane display name".to_string(),
            initial_name,
            /*context_label*/ None,
            Box::new(move |name: String| {
                tx.send(AppEvent::RenameClaudePane {
                    pane_id: pane_id.clone(),
                    name: name.trim().to_string(),
                });
            }),
        );
        self.chat_widget.show_custom_prompt_view(view);
    }

    pub(crate) fn save_active_claude_pane_transcript(&mut self) {
        let Some(active_pane_id) = self
            .claude_panes
            .active_claude_pane_id()
            .map(ToString::to_string)
        else {
            return;
        };
        self.claude_pane_transcript_cells
            .insert(active_pane_id, self.transcript_cells.clone());
    }

    fn restore_claude_pane_transcript(&mut self, tui: &mut tui::Tui, pane_id: &str) -> Result<()> {
        self.reset_for_thread_switch(tui)
            .map_err(|err| anyhow!(err.to_string()))?;
        self.transcript_cells = self
            .claude_pane_transcript_cells
            .get(pane_id)
            .cloned()
            .unwrap_or_default();
        let width = self
            .chat_widget
            .history_wrap_width(tui.terminal.last_known_screen_size.width);
        for cell in self.transcript_cells.clone() {
            self.insert_history_cell_lines(tui, cell.as_ref(), width);
        }
        Ok(())
    }

    pub(crate) fn append_inactive_claude_pane_transcript_cell(
        &mut self,
        pane_id: &str,
        cell: Arc<dyn crate::history_cell::HistoryCell>,
    ) {
        self.claude_pane_transcript_cells
            .entry(pane_id.to_string())
            .or_default()
            .push(cell);
    }

    pub(crate) fn persist_pane_state(&mut self) -> bool {
        let codex_thread_id = self
            .primary_thread_id
            .or_else(|| self.chat_widget.thread_id())
            .map(|thread_id| thread_id.to_string());
        let Some(codex_thread_id) = codex_thread_id else {
            return true;
        };
        let layout = PaneLayoutState {
            version: PANE_LAYOUT_VERSION,
            codex_thread_id: Some(codex_thread_id),
            codex_user_pane_ids: {
                let mut thread_ids = self
                    .agent_navigation
                    .ordered_threads()
                    .into_iter()
                    .filter(|(thread_id, entry)| {
                        self.is_operator_owned_codex_user_pane(*thread_id, entry)
                    })
                    .map(|(thread_id, _)| thread_id.to_string())
                    .collect::<Vec<_>>();
                thread_ids.sort();
                thread_ids
            },
            active_user_pane_id: Some(self.claude_panes.active_user_pane_id().to_string()),
            spawn_nazgul_pane_id: self.spawn_nazgul_pane_id.clone(),
            spawn_nazgul_rebind_required: self.spawn_nazgul_rebind_required,
            claude_pane_ids: self
                .claude_panes
                .panes()
                .iter()
                .map(|pane| pane.id.clone())
                .collect(),
            spawn_parent_by_node: self
                .spawn_parent_by_node
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            spawn_native_runtime_by_node: self
                .spawn_native_runtime_by_node
                .iter()
                .map(|(node, runtime)| (node.clone(), runtime.clone()))
                .collect(),
            spawn_native_endpoint_by_node: self
                .spawn_native_endpoint_by_node
                .iter()
                .map(|(node, endpoint)| (node.clone(), endpoint.to_string()))
                .collect(),
            spawn_crew: self.spawn_crew.clone(),
            orchestrate_whips: self
                .orchestrate_whips
                .iter()
                .filter(|(_, whip)| whip.state != crate::orchestrate::WhipState::Detached)
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            orchestrate_next_whip_seq: self.orchestrate_next_whip_seq,
            // Compatibility input only. New `/spawn` work is admitted to the native mailbox and
            // is never persisted in a second TUI-owned dispatch queue.
            spawn_pending_dispatches: Default::default(),
            spawn_pending_dispatches_by_thread: Default::default(),
            spawn_pending_dispatches_by_pane: Default::default(),
            spawn_next_dispatch_seq: self.spawn_next_dispatch_seq.max(1),
            spawn_processed_dispatch_seq_ids: self.recent_spawn_processed_dispatch_seq_ids(),
            spawn_processed_dispatch_origin_ids: {
                let mut origins = self
                    .spawn_processed_dispatch_origins
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                origins.sort();
                origins
            },
            spawn_accepted_delivery_ids: Vec::new(),
        };
        if let Err(err) = persist_pane_layout(self.config.codex_home.as_ref(), &layout) {
            tracing::warn!(error = %err, "failed to persist pane layout");
            self.chat_widget.add_error_message(format!(
                "Pane layout was not saved; assignments cannot be restored after restart: {err}"
            ));
            return false;
        }
        true
    }

    pub(crate) fn seed_restored_claude_pane_transcripts(&mut self) {
        let cwd = self.config.cwd.clone();
        let restored: Vec<_> = self
            .claude_panes
            .panes()
            .iter()
            .map(|pane| {
                (
                    pane.id.clone(),
                    pane.latest_result_message.clone(),
                    pane.latest_audit_path.clone(),
                    pane.latest_turn_status,
                )
            })
            .collect();
        for (pane_id, result, audit_path, status) in restored {
            let entry = self
                .claude_pane_transcript_cells
                .entry(pane_id)
                .or_default();
            if !entry.is_empty() {
                continue;
            }
            if let Some(result) = result {
                entry.push(Arc::new(crate::history_cell::AgentMarkdownCell::new(
                    result,
                    cwd.as_path(),
                )));
            }
            if let Some(audit_path) = audit_path {
                let status = status
                    .map(super::pane::ClaudePaneTurnStatus::label)
                    .unwrap_or("unknown");
                entry.push(Arc::new(crate::history_cell::new_info_event(
                    "Restored Claude pane state.".to_string(),
                    Some(format!(
                        "latest status: {status}; audit: {}",
                        audit_path.display()
                    )),
                )));
            }
        }
    }

    pub(crate) fn show_restored_active_claude_pane(&mut self) {
        let Some(active_pane_id) = self
            .claude_panes
            .active_claude_pane_id()
            .map(ToString::to_string)
        else {
            self.sync_external_pane_turn_display(CODEX_MAIN_PANE_ID);
            self.sync_active_agent_label();
            return;
        };
        self.transcript_cells = self
            .claude_pane_transcript_cells
            .get(&active_pane_id)
            .cloned()
            .unwrap_or_default();
        self.sync_external_pane_turn_display(&active_pane_id);
        self.sync_active_agent_label();
    }

    pub(crate) async fn select_user_pane(&mut self, tui: &mut tui::Tui, pane_id: String) {
        self.save_active_claude_pane_transcript();
        match self.claude_panes.set_active_user_pane(&pane_id) {
            Ok(()) if pane_id == CODEX_MAIN_PANE_ID => {
                self.sync_external_pane_turn_display(&pane_id);
                self.sync_active_agent_label();
                self.persist_pane_state();
            }
            Ok(()) => {
                self.detach_active_thread_for_external_pane().await;
                if let Err(err) = self.restore_claude_pane_transcript(tui, &pane_id) {
                    self.chat_widget
                        .add_error_message(format!("Failed to switch Claude pane display: {err}"));
                }
                self.sync_external_pane_turn_display(&pane_id);
                self.sync_active_agent_label();
                self.persist_pane_state();
            }
            Err(err) => self.chat_widget.add_error_message(err.to_string()),
        }
    }

    pub(crate) fn sync_external_pane_turn_display(&mut self, pane_id: &str) {
        if pane_id == CODEX_MAIN_PANE_ID || !self.claude_panes.claude_pane_is_running(pane_id) {
            self.chat_widget.suspend_external_pane_turn_display();
            return;
        }
        self.chat_widget.begin_external_pane_turn();
        if let Some(status) = self.claude_panes.live_status_for_pane(pane_id) {
            self.chat_widget
                .update_external_pane_live_status(status.header, status.details);
        }
    }

    pub(crate) async fn create_claude_pane(
        &mut self,
        tui: &mut tui::Tui,
        profile: ClaudeProviderProfileKind,
        display_name: Option<String>,
    ) {
        self.save_active_claude_pane_transcript();
        match self.claude_panes.create_pane(
            profile,
            self.config.cwd.to_path_buf(),
            self.config.codex_home.as_ref(),
        ) {
            Ok(id) => {
                self.detach_active_thread_for_external_pane().await;
                self.claude_pane_transcript_cells
                    .entry(id.clone())
                    .or_default();
                if let Err(err) = self.restore_claude_pane_transcript(tui, &id) {
                    self.chat_widget.add_error_message(format!(
                        "Failed to initialize Claude pane display: {err}"
                    ));
                }
                if let Some(display_name) = display_name
                    && let Err(err) = self.claude_panes.rename_pane(&id, display_name)
                {
                    self.chat_widget
                        .add_error_message(format!("Failed to rename Claude pane: {err}"));
                }
                let title = profile.profile().title;
                self.sync_active_agent_label();
                self.persist_pane_state();
                self.chat_widget.add_info_message(
                    format!("Created and switched to {title}."),
                    Some("Type normally; turns will run through Claude Code headless.".to_string()),
                );
                tracing::info!(pane_id = %id, profile = ?profile, "created Claude headless pane");
                // #554: a contained pane checks its Claude Code now, so an
                // old one is explained before the first task.
                let tx = self.app_event_tx.clone();
                let codex_home = self.config.codex_home.to_path_buf();
                let cwd = self.config.cwd.to_path_buf();
                tokio::spawn(async move {
                    if let Some(problem) =
                        crate::claude_panes::execution::contained_claude_launch_problem(
                            codex_home,
                            id.clone(),
                            cwd,
                        )
                        .await
                    {
                        tx.send(AppEvent::ClaudePaneLaunchProblem {
                            pane_id: id,
                            problem,
                        });
                    }
                });
            }
            Err(err) => self.chat_widget.add_error_message(err.to_string()),
        }
    }

    pub(crate) async fn create_spawn_claude_pane(
        &mut self,
        _tui: &mut tui::Tui,
        role: SpawnRole,
        parent_node_id: Option<String>,
        profile: ClaudeProviderProfileKind,
    ) {
        if role == SpawnRole::Nazgul {
            self.chat_widget.add_error_message(
                "Nazgul is a pane binding, not a spawned Claude worker.".to_string(),
            );
            return;
        }
        let spawn_nickname = self.next_spawn_agent_nickname(role);
        match self.claude_panes.create_pane_with_role(
            profile,
            self.config.cwd.to_path_buf(),
            self.config.codex_home.as_ref(),
            Some(role),
            spawn_nickname.clone(),
        ) {
            Ok(id) => {
                // Spawned workers are background children: the operator's
                // control surface stays where it is. Switching focus into the
                // new pane caused follow-up control-plane input to be routed
                // into the worker (round-3 B5 failure mode).
                self.claude_pane_transcript_cells
                    .entry(id.clone())
                    .or_default();
                let title = claude_pane_title(profile, Some(role), spawn_nickname.as_deref());
                let logical_parent_node_id =
                    self.logical_parent_node_for_spawn(role, parent_node_id.as_deref());
                self.spawn_parent_by_node.insert(
                    crate::spawn_orchestration::pane_node_id(&id),
                    logical_parent_node_id.clone(),
                );
                let profile_config = profile.profile();
                let logical_node_id = crate::spawn_orchestration::pane_node_id(&id);
                if let Err(err) = self.record_custom_spawn_member(
                    &logical_node_id,
                    &logical_parent_node_id,
                    role,
                    spawn_nickname
                        .clone()
                        .unwrap_or_else(|| role.label().to_string()),
                    crate::dispatch_queue::SavedNativeSpawnRuntime {
                        model: profile_config.provider_model.to_string(),
                        provider: profile.crew_runtime_provider_id(),
                        reasoning_effort: None,
                    },
                ) {
                    self.mark_crew_incomplete(err.to_string());
                    self.chat_widget.add_error_message(format!(
                        "Created the pane, but could not persist its crew identity: {err}"
                    ));
                    return;
                }
                self.sync_active_agent_label();
                self.persist_pane_state();
                self.persist_claude_spawn_pane_state(&id, &logical_parent_node_id)
                    .await;
                self.chat_widget.add_info_message(
                    format!("Created {title} as a background worker."),
                    Some(format!(
                        "Harness: Claude Code; role: {}; control stays on the current pane (use /panes to open it); no task was started.",
                        role.label()
                    )),
                );
                tracing::info!(
                    pane_id = %id,
                    profile = ?profile,
                    role = ?role,
                    "created Claude spawn pane"
                );
            }
            Err(err) => self.chat_widget.add_error_message(err.to_string()),
        }
    }

    pub(crate) fn try_submit_active_claude_pane_op(&mut self, op: &AppCommand) -> bool {
        let Some(pane_id) = self
            .claude_panes
            .active_claude_pane_id()
            .map(ToString::to_string)
        else {
            return false;
        };
        if matches!(op, AppCommand::Interrupt) {
            if !self.claude_panes.claude_pane_is_running(&pane_id) {
                self.chat_widget.complete_external_pane_turn(
                    /*last_agent_message*/ None, /*duration_ms*/ None,
                );
                return true;
            }
            match self.claude_panes.interrupt_turn(&pane_id) {
                Ok(()) => {
                    self.chat_widget.update_external_pane_live_status(
                        "Claude interrupting".to_string(),
                        Some("Waiting for the Claude process to stop.".to_string()),
                    );
                }
                Err(err) => self.chat_widget.add_error_message(err.to_string()),
            }
            return true;
        }
        let prompt = match prompt_from_user_turn(op) {
            Ok(Some(prompt)) => prompt,
            Ok(None) => return false,
            Err(err) => {
                self.chat_widget.fail_external_pane_turn(err.to_string());
                return true;
            }
        };
        // Control-plane guard: slash commands act on Corbanu Terminal itself, never
        // on the active worker pane. Without this, a recognized command that
        // reaches the op path while a Claude pane is active is forwarded to
        // the worker as task text (the round-3 B5 failure mode).
        if self.chat_widget.try_dispatch_slash_input(&prompt) {
            return true;
        }
        if let Some(reason) = crate::security::level::external_agent_block_reason() {
            self.chat_widget.fail_external_pane_turn(reason);
            return true;
        }
        self.note_assignment_user_turn(&crate::spawn_orchestration::pane_node_id(&pane_id));
        // #507: a task held while the kill switch was on goes with this turn.
        let held = self.held_claude_task(&pane_id);
        let prompt = with_held_claude_task(held.as_deref(), prompt);
        let prompt_context = self.claude_pane_prompt_context(&pane_id);
        let prompt = compose_claude_pane_prompt(prompt, prompt_context.as_deref());
        let claude_plan_account = self.claude_plan_account();
        let prepared = match self.claude_panes.prepare_turn_for_account(
            &pane_id,
            prompt,
            self.config.codex_home.as_ref(),
            claude_plan_account.as_deref(),
        ) {
            Ok(prepared) => prepared,
            Err(err) => {
                self.chat_widget.fail_external_pane_turn(err.to_string());
                return true;
            }
        };
        if let Some(held) = held {
            // A person's turn: the held task no longer counts as an
            // automatic one for the loop breaker.
            self.abort_spawn_auto_processing_turn(&crate::spawn_orchestration::pane_node_id(
                &pane_id,
            ));
            self.held_claude_task_delivered(&pane_id, &held);
        }

        self.chat_widget.begin_external_pane_turn();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let pane_id = prepared.pane_id.clone();
            let result = run_prepared_claude_turn(prepared, Some(tx.clone())).await;
            tx.send(AppEvent::ClaudePaneTurnFinished { pane_id, result });
        });
        true
    }

    /// #554: shows why a new contained Claude pane cannot run yet, in that
    /// pane.
    pub(crate) fn on_claude_pane_launch_problem(&mut self, pane_id: &str, problem: String) {
        if !self
            .claude_panes
            .panes()
            .iter()
            .any(|pane| pane.id == pane_id)
        {
            return;
        }
        let message = format!("This Claude pane cannot run yet. {problem}");
        if self.claude_panes.active_claude_pane_id() == Some(pane_id) {
            self.chat_widget.add_warning_message(message);
        } else {
            self.append_inactive_claude_pane_transcript_cell(
                pane_id,
                Arc::new(crate::history_cell::new_warning_event(message)),
            );
        }
    }

    /// #507: the kill switch changed, in `/security` here or (`elsewhere`)
    /// in another Corbanu Terminal process on this home. Core has stopped or
    /// released its own sessions; the Claude panes are the app's. Each change
    /// is handled once, whichever report arrives first.
    pub(crate) fn on_kill_switch_changed(&mut self, on: bool, elsewhere: bool) {
        if self.kill_switch_known_on == on {
            return;
        }
        self.kill_switch_known_on = on;
        if elsewhere {
            self.chat_widget.add_info_message(
                if on {
                    KILL_SWITCH_ON_ELSEWHERE
                } else {
                    KILL_SWITCH_OFF_ELSEWHERE
                }
                .to_string(),
                /*hint*/ None,
            );
        }
        // Turned off (here or elsewhere), nothing starts by itself (Travis,
        // 2026-10-12): tasks held for Claude panes go with their next turn.
        if on {
            self.on_kill_switch_turned_on();
        } else {
            self.on_kill_switch_turned_off();
        }
    }

    /// #507 (Travis, 2026-10-12): a kill switch turned on or off by another
    /// Corbanu Terminal process on this home reaches this one while it runs.
    pub(crate) fn watch_kill_switch_elsewhere(&mut self) {
        let mut watched = crate::legacy_core::security_revocation::watch_kill_switch(
            self.config.codex_home.as_ref(),
        );
        // Start from the watch's own value, so no change falls in between.
        self.kill_switch_known_on = *watched.borrow_and_update();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            while watched.changed().await.is_ok() {
                let on = *watched.borrow_and_update();
                tx.send(AppEvent::KillSwitchChangedElsewhere { on });
            }
        });
    }

    /// #842 (Travis, 2026-10-15): Moderate confirmed in another Corbanu
    /// Terminal process on this home reaches this one while it runs; Core
    /// has already raised this process's sessions, and the person is told.
    pub(crate) fn watch_level_raised_elsewhere(&mut self) {
        let mut watched = crate::legacy_core::security_revocation::watch_level_raised_elsewhere(
            self.config.codex_home.as_ref(),
        );
        let _ = watched.borrow_and_update();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            while watched.changed().await.is_ok() {
                if let Some(level) = *watched.borrow_and_update() {
                    tx.send(AppEvent::SecurityLevelRaisedElsewhere { level });
                }
            }
        });
    }

    /// #842/#919: says that Moderate or Aggressive confirmed in another
    /// Corbanu Terminal on this home now applies here, and what waits for
    /// the restart.
    pub(crate) fn on_level_raised_elsewhere(
        &mut self,
        level: crate::legacy_core::security_revocation::SecurityLevel,
    ) {
        use crate::legacy_core::security_revocation::SecurityLevel;
        let message = match level {
            SecurityLevel::Moderate => MODERATE_CONFIRMED_ELSEWHERE.to_string(),
            SecurityLevel::Aggressive => aggressive_confirmed_elsewhere(
                self.config
                    .features
                    .enabled(codex_features::Feature::SourceEnvelopes),
                crate::security::level::context()
                    .map_or(crate::security::level::ChosenLevel::Permissive, |context| {
                        context.active
                    }),
            ),
            SecurityLevel::Permissive => return,
        };
        self.chat_widget.add_info_message(message, /*hint*/ None);
    }

    /// #507: the kill switch was turned on (here, or in another Corbanu
    /// Terminal on this home): running
    /// Claude pane turns stop, and assignments and whips pause at once (not
    /// at the next sweep), so none can fire once the switch is off again.
    pub(crate) fn on_kill_switch_turned_on(&mut self) {
        self.stop_claude_panes_for_kill_switch();
        self.pause_whips_for_kill_switch();
    }

    /// #507: the kill switch was turned on: every running Claude pane turn
    /// is interrupted (its Claude process and the commands it runs end), with
    /// the reason in that pane.
    pub(crate) fn stop_claude_panes_for_kill_switch(&mut self) {
        let running: Vec<String> = self
            .claude_panes
            .panes()
            .iter()
            .map(|pane| pane.id.clone())
            .filter(|pane_id| self.claude_panes.claude_pane_is_running(pane_id))
            .collect();
        for pane_id in running {
            if let Err(err) = self.claude_panes.interrupt_turn_for_kill_switch(&pane_id) {
                tracing::warn!(pane_id, %err, "kill switch: could not interrupt the Claude pane");
                continue;
            }
            let message = crate::legacy_core::security_revocation::KILL_SWITCH_STOPPED;
            if self.claude_panes.active_claude_pane_id() == Some(pane_id.as_str()) {
                self.chat_widget.add_warning_message(message.to_string());
            } else {
                self.append_inactive_claude_pane_transcript_cell(
                    &pane_id,
                    Arc::new(crate::history_cell::new_warning_event(message.to_string())),
                );
            }
        }
    }

    /// PF-84: the named Claude Plan account panes must use, if one is selected.
    fn claude_plan_account(&self) -> Option<String> {
        self.config
            .model_providers
            .get(codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID)?
            .account
            .as_ref()
            .map(|account| account.name.clone())
    }

    /// #613: whether the kill switch is on, as saved or as this process
    /// last learned. After a switch-off in another process this stays true
    /// until that change is handled here (about a second): tasks are held,
    /// never sent, in that window. A switch-off here is handled in order.
    pub(crate) fn kill_switch_on_now(&self) -> bool {
        self.kill_switch_known_on
            || crate::legacy_core::security_revocation::kill_switch_on(
                self.config.codex_home.as_ref(),
            )
    }

    #[cfg(test)]
    pub(crate) fn submit_claude_pane_task(&mut self, pane_id: String, task: String) {
        self.submit_claude_pane_task_queued(
            pane_id, task, /*sent_while_kill_switch_on*/ false,
        );
    }

    /// `sent_while_kill_switch_on` (#613): another agent sent the task while
    /// the switch was on; it waits for the pane's next turn even when the
    /// switch was turned off before it got here (nothing restarts after
    /// switch-off, #507).
    pub(crate) fn submit_claude_pane_task_queued(
        &mut self,
        pane_id: String,
        task: String,
        sent_while_kill_switch_on: bool,
    ) {
        let task = task.trim().to_string();
        let target_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);
        if task.is_empty() {
            self.chat_widget
                .add_error_message("Claude pane task cannot be empty.".to_string());
            return;
        }
        if let Some(reason) = crate::security::level::external_agent_block_reason() {
            let node_key = crate::spawn_orchestration::pane_node_id(&pane_id);
            self.abort_spawn_auto_processing_turn(&node_key);
            // #660: no sender turn for a task sent while the switch was on.
            self.record_spawn_dispatch_failed_for_task_with_turns(
                &target_node_id,
                &task,
                &reason,
                /*start_turns*/ !sent_while_kill_switch_on,
            );
            self.chat_widget.add_error_message(reason);
            return;
        }
        // #507: while the kill switch is on, a task another agent sent waits,
        // as agent messages to Codex panes do, for the pane's next turn after
        // it is off; turning it off does not send it.
        if sent_while_kill_switch_on || self.kill_switch_on_now() {
            // No auto-processing turn starts for it: the task goes with the
            // pane's next turn, which counts as that turn's own kind. Each
            // pane holds one task, the newest: repeated assignment fires and
            // child reports would otherwise pile up (the newest prompt also
            // carries the latest context).
            self.abort_spawn_auto_processing_turn(&target_node_id);
            let Some(title) = self
                .claude_panes
                .panes()
                .iter()
                .find(|pane| pane.id == pane_id)
                .map(|pane| pane.title.clone())
            else {
                let detail = format!("Claude pane `{pane_id}` does not exist");
                // #660: the sender is told; no turn starts for it there.
                self.record_kill_switch_dispatch_failed_for_task(&target_node_id, &task, &detail);
                self.chat_widget.add_error_message(detail);
                return;
            };
            let superseded: Vec<String> = self
                .kill_switch_held_claude_tasks
                .iter()
                .filter(|(held, _)| *held == pane_id)
                .map(|(_, task)| task.clone())
                .collect();
            self.kill_switch_held_claude_tasks
                .retain(|(held, _)| *held != pane_id);
            // The same text again is the same task: its acks stay with it
            // (they are keyed by text) instead of being reported superseded.
            for old in superseded.iter().filter(|old| **old != task) {
                self.record_kill_switch_dispatch_failed_for_task(
                    &target_node_id,
                    old,
                    "superseded by a newer task while the security kill switch was on",
                );
            }
            self.kill_switch_held_claude_tasks.push((pane_id, task));
            if superseded.is_empty() {
                let message = if self.kill_switch_on_now() {
                    format!(
                        "A task for Claude pane `{title}` waits: the security kill switch is on. Turning it off starts nothing: the task goes with this pane's next turn, when someone types in it or it gets new work (a newer task replaces it; it is lost if Corbanu Terminal exits first)."
                    )
                } else {
                    format!(
                        "A task for Claude pane `{title}` waits: it was sent while the security kill switch was on, and turning the switch off starts nothing. It goes with this pane's next turn, when someone types in it or it gets new work (a newer task replaces it; it is lost if Corbanu Terminal exits first)."
                    )
                };
                self.chat_widget.add_info_message(message, /*hint*/ None);
            }
            return;
        }
        let held = self.held_claude_task(&pane_id);
        let is_active = self.claude_panes.active_user_pane_id() == pane_id;
        let user_cell =
            crate::history_cell::new_user_prompt(task.clone(), Vec::new(), Vec::new(), Vec::new());
        if is_active {
            self.app_event_tx
                .send(AppEvent::InsertHistoryCell(Box::new(user_cell)));
        } else {
            self.append_inactive_claude_pane_transcript_cell(&pane_id, Arc::new(user_cell));
        }
        self.claude_panes
            .set_latest_task_message(&pane_id, Some(task.clone()));
        let prompt_context = self.claude_pane_prompt_context(&pane_id);
        let prompt = compose_claude_pane_prompt(
            with_held_claude_task(held.as_deref(), task.clone()),
            prompt_context.as_deref(),
        );
        let node_key = crate::spawn_orchestration::pane_node_id(&pane_id);
        let auto_processing_turn = self
            .spawn_auto_loop_state_by_node
            .get(&node_key)
            .is_some_and(|state| state.pending_auto_turn);
        if !auto_processing_turn {
            self.spawn_operator_input_seen = true;
        }
        let claude_plan_account = self.claude_plan_account();
        let prepared = match self.claude_panes.prepare_turn_for_account(
            &pane_id,
            prompt,
            self.config.codex_home.as_ref(),
            claude_plan_account.as_deref(),
        ) {
            Ok(prepared) => prepared,
            Err(err) => {
                self.abort_spawn_auto_processing_turn(&node_key);
                self.record_spawn_dispatch_failed_for_task(&target_node_id, &task, err.to_string());
                self.chat_widget.add_error_message(err.to_string());
                return;
            }
        };
        if let Some(held) = held {
            self.held_claude_task_delivered(&pane_id, &held);
        }
        self.record_spawn_dispatch_delivered_for_task(&target_node_id, &task);
        self.record_claude_spawn_rollout_task_started(&pane_id, &task, prepared.plan.turn_index);
        // Loop breaker: a turn we auto-triggered (child-report processing) transitions
        // pending -> running; any other submitted task is fresh work and resets the auto chain.
        self.note_spawn_turn_started_for_auto_loop(&node_key);
        self.note_whip_target_started(&node_key);

        if self.claude_panes.active_user_pane_id() == pane_id {
            self.chat_widget.begin_external_pane_turn();
        }
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let pane_id = prepared.pane_id.clone();
            let result = run_prepared_claude_turn(prepared, Some(tx.clone())).await;
            tx.send(AppEvent::ClaudePaneTurnFinished { pane_id, result });
        });
    }

    fn claude_pane_prompt_context(&self, pane_id: &str) -> Option<String> {
        let mut contexts = Vec::new();
        if let Some(role_context) = self
            .claude_panes
            .claude_pane_spawn_role(pane_id)
            .and_then(SpawnRole::claude_pane_context)
        {
            contexts.push(role_context.to_string());
        }
        if let Some(spawn_context) = self.spawn_context_for_user_pane(pane_id) {
            contexts.push(spawn_context);
        }
        (!contexts.is_empty()).then(|| contexts.join("\n\n"))
    }

    pub(crate) fn on_claude_pane_turn_progress(&mut self, progress: ClaudePaneTurnProgress) {
        let is_active = self.claude_panes.active_user_pane_id() == progress.pane_id;
        // Deliberately no dispatch scanning here: spawn task blocks must never dispatch from a
        // streaming turn. Dispatch happens only in on_claude_pane_turn_finished for turns that
        // ended with ClaudePaneTurnStatus::Success, so an interrupted or failed pane turn can
        // never fire a truncated pfterminal_send_task block.
        if let Some(status) = self.claude_panes.update_live_progress(&progress)
            && is_active
        {
            if progress.phase == "assistant-text"
                && let Some(delta) = self
                    .claude_panes
                    .take_visible_assistant_transcript_delta(&progress.pane_id)
            {
                self.chat_widget.stream_external_pane_response_delta(delta);
            }
            self.chat_widget
                .update_external_pane_live_status(status.header, status.details);
        }
    }

    pub(crate) fn on_claude_pane_turn_finished(
        &mut self,
        pane_id: String,
        result: Result<ClaudePaneTurnOutput, String>,
    ) {
        // #554: a turn the kill switch stopped says so once, under the
        // switch's message, instead of "interrupted by user".
        let kill_switch_stopped = self.claude_panes.take_kill_switch_stop(&pane_id);
        match result {
            Ok(mut output) => {
                let is_active = self.claude_panes.active_user_pane_id() == pane_id;
                let failure_message = if kill_switch_stopped
                    && output.status == super::pane::ClaudePaneTurnStatus::Interrupted
                {
                    crate::chatwidget::KILL_SWITCH_TURN_STOPPED.to_string()
                } else {
                    output.failure_message()
                };
                let source_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);
                if output.status.is_success() {
                    self.dispatch_orchestrate_blocks_from_text(&source_node_id, &output.text);
                    let (visible_text, _) =
                        crate::orchestrate::extract_orchestrate_blocks(&output.text);
                    output.text = visible_text;
                }
                let (visible_text, dispatches) =
                    crate::spawn_orchestration::extract_spawn_task_dispatches(&output.text);
                output.text = visible_text;
                if is_active
                    && let Some(delta) = self
                        .claude_panes
                        .take_final_visible_assistant_transcript_delta(&pane_id, &output.text)
                {
                    self.chat_widget.stream_external_pane_response_delta(delta);
                }
                let active_text_streamed = is_active
                    && self
                        .claude_panes
                        .has_emitted_visible_assistant_transcript(&pane_id);
                let dispatches = self
                    .claude_panes
                    .filter_new_spawn_dispatches(&pane_id, dispatches);
                self.claude_panes.finish_turn(&pane_id, &Ok(output.clone()));
                let report_status = output.status.label().to_string();
                let report_text = if output.text.trim().is_empty() {
                    failure_message.clone()
                } else {
                    output.text.clone()
                };
                self.record_claude_spawn_rollout_task_completed(&pane_id, &output);
                // #507: the report of a turn the kill switch stopped starts no
                // turn in the parent (the stop may end after the switch is
                // off); the parent gets it with its next turn.
                self.record_spawn_child_report_for_claude_pane_with(
                    &pane_id,
                    &report_status,
                    Some(&report_text),
                    /*start_turns*/ !kill_switch_stopped,
                );
                // Dispatch only from cleanly completed turns. Interrupted, paused, and failed
                // turns must never dispatch: their text can contain a complete-looking
                // pfterminal_send_task block whose task was truncated mid-thought.
                if output.status.is_success() && !dispatches.is_empty() {
                    self.dispatch_spawn_task_blocks_from_model_turn(
                        &pane_id,
                        &crate::spawn_orchestration::pane_node_id(&pane_id),
                        &format!("claude-artifact:{}", output.artifact_path.display()),
                        dispatches,
                    );
                }
                // Loop breaker: finalize AFTER the dispatch call above so a dispatch emitted by
                // this turn is attributed to it before the auto-turn flags clear.
                self.note_spawn_turn_completed_for_auto_loop(
                    &crate::spawn_orchestration::pane_node_id(&pane_id),
                );
                self.note_whip_target_idle_with_fire_control(
                    &source_node_id,
                    Some(&report_text),
                    /*allow_fire*/ true,
                    output.status.is_success(),
                );
                if !output.text.trim().is_empty() {
                    if is_active && !active_text_streamed {
                        self.chat_widget
                            .append_external_pane_response(output.text.clone());
                    } else if !is_active {
                        self.append_inactive_claude_pane_transcript_cell(
                            &pane_id,
                            Arc::new(crate::history_cell::AgentMarkdownCell::new(
                                output.text.clone(),
                                self.config.cwd.as_path(),
                            )),
                        );
                    }
                }
                let hint = output.audit_hint();
                if output.status.is_success() {
                    if is_active {
                        self.chat_widget.complete_external_pane_turn(
                            Some(output.text),
                            Some(output.duration_ms),
                        );
                        self.chat_widget
                            .add_info_message("Claude pane turn complete.".to_string(), Some(hint));
                    } else {
                        self.append_inactive_claude_pane_transcript_cell(
                            &pane_id,
                            Arc::new(crate::history_cell::new_info_event(
                                "Claude pane turn complete.".to_string(),
                                Some(hint),
                            )),
                        );
                    }
                } else if is_active {
                    self.chat_widget.fail_external_pane_turn(failure_message);
                    self.chat_widget.add_info_message(
                        "Claude pane turn audit recorded.".to_string(),
                        Some(hint),
                    );
                } else {
                    self.append_inactive_claude_pane_transcript_cell(
                        &pane_id,
                        Arc::new(crate::history_cell::new_error_event(failure_message)),
                    );
                    self.append_inactive_claude_pane_transcript_cell(
                        &pane_id,
                        Arc::new(crate::history_cell::new_info_event(
                            "Claude pane turn audit recorded.".to_string(),
                            Some(hint),
                        )),
                    );
                }
            }
            Err(error) => {
                self.claude_panes.finish_turn(&pane_id, &Err(error.clone()));
                let source_node_id = crate::spawn_orchestration::pane_node_id(&pane_id);
                self.note_spawn_turn_completed_for_auto_loop(&source_node_id);
                self.record_spawn_child_report_for_claude_pane_with(
                    &pane_id,
                    "error",
                    Some(&error),
                    /*start_turns*/ !kill_switch_stopped,
                );
                self.note_whip_target_idle_with_fire_control(
                    &source_node_id,
                    Some(&error),
                    /*allow_fire*/ true,
                    /*turn_succeeded*/ false,
                );
                if self.claude_panes.active_user_pane_id() == pane_id {
                    self.chat_widget.fail_external_pane_turn(error);
                } else {
                    self.append_inactive_claude_pane_transcript_cell(
                        &pane_id,
                        Arc::new(crate::history_cell::new_error_event(error)),
                    );
                }
            }
        }
    }

    /// #507: the kill switch was turned off. Nothing starts: each task held
    /// for a Claude pane, or (#660 follow-up) waiting in a managed Codex
    /// pane's mailbox, goes with that pane's next turn. The person is told
    /// which panes have one.
    pub(crate) fn on_kill_switch_turned_off(&mut self) {
        let claude_titles = self
            .kill_switch_held_claude_tasks
            .iter()
            .filter_map(|(pane_id, _)| {
                self.claude_panes
                    .panes()
                    .iter()
                    .find(|pane| pane.id == *pane_id)
                    .map(|pane| pane.title.clone())
            });
        let codex_titles = self
            .kill_switch_held_native_tasks
            .iter()
            .filter(|thread_id| {
                self.agent_navigation
                    .get(thread_id)
                    .is_some_and(|entry| !entry.is_closed)
            })
            .map(|thread_id| self.thread_label(*thread_id));
        let titles: Vec<String> = claude_titles
            .chain(codex_titles)
            .map(|title| format!("`{title}`"))
            .collect();
        if titles.is_empty() {
            return;
        }
        self.chat_widget.add_info_message(
            format!(
                "Nothing restarted. A task waits for each of these panes and goes with that pane's next turn, when someone types in it or it gets new work: {}.",
                titles.join(", ")
            ),
            /*hint*/ None,
        );
    }

    /// #507: a removed Claude pane's waiting task is dropped and its sender
    /// told.
    pub(crate) fn drop_held_claude_task_of_removed_pane(&mut self, pane_id: &str) {
        let Some(task) = self.held_claude_task(pane_id) else {
            return;
        };
        self.kill_switch_held_claude_tasks
            .retain(|(held, _)| held != pane_id);
        self.record_spawn_dispatch_failed_for_task(
            &crate::spawn_orchestration::pane_node_id(pane_id),
            &task,
            "the Claude pane was removed while its task waited (security kill switch)",
        );
    }

    /// #507: the task held for `pane_id` while the kill switch was on.
    fn held_claude_task(&self, pane_id: &str) -> Option<String> {
        self.kill_switch_held_claude_tasks
            .iter()
            .find(|(held, _)| held == pane_id)
            .map(|(_, task)| task.clone())
    }

    /// #507: the held task went with the turn that just started in `pane_id`.
    fn held_claude_task_delivered(&mut self, pane_id: &str, task: &str) {
        self.kill_switch_held_claude_tasks
            .retain(|(held, _)| held != pane_id);
        self.record_spawn_dispatch_delivered_for_task(
            &crate::spawn_orchestration::pane_node_id(pane_id),
            task,
        );
        let summary = HELD_TASK_DELIVERED.to_string();
        let mut hint: String = task.chars().take(HELD_TASK_HINT_CHARS).collect();
        if hint.len() < task.len() {
            hint.push('…');
        }
        let hint = Some(hint);
        if self.claude_panes.active_user_pane_id() == pane_id {
            self.chat_widget.add_info_message(summary, hint);
        } else {
            self.append_inactive_claude_pane_transcript_cell(
                pane_id,
                Arc::new(crate::history_cell::new_info_event(summary, hint)),
            );
        }
    }

    fn user_pane_items(&self) -> Vec<SelectionItem> {
        let mut items = Vec::new();
        // The external-pane registry uses `codex-main` for every native Core
        // thread.  The active Core thread is therefore the authority for which
        // native row is current; treating `codex-main` alone as Main made both
        // Main and a selected operator pane appear current at the same time.
        let is_current = self.claude_panes.active_user_pane_id() == CODEX_MAIN_PANE_ID
            && self.active_thread_id == self.primary_thread_id;
        let main_name = self
            .primary_thread_id
            .and_then(|thread_id| self.agent_navigation.get(&thread_id))
            .and_then(|entry| entry.agent_nickname.as_deref())
            .filter(|nickname| !nickname.trim().is_empty())
            .map(native_pane_title)
            .unwrap_or_else(|| MAIN_PANE_TITLE.to_string());
        let main_rename_shortcuts = self
            .primary_thread_id
            .map(|thread_id| vec![rename_codex_pane_shortcut(thread_id)])
            .unwrap_or_default();
        items.push(SelectionItem {
            name: main_name.clone(),
            description: Some(self.codex_main_pane_description()),
            is_current,
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::SelectUserPane {
                    pane_id: CODEX_MAIN_PANE_ID.to_string(),
                });
            })],
            dismiss_on_select: true,
            selected_shortcuts: main_rename_shortcuts,
            search_value: Some(main_name),
            ..Default::default()
        });
        items.extend(self.codex_user_pane_items());
        for pane in self.claude_panes.panes() {
            let pane_id = pane.id.clone();
            let pane_id_for_action = pane_id.clone();
            let node_id = crate::spawn_orchestration::pane_node_id(&pane.id);
            let mut description = format!(
                "{}; {}",
                pane.profile.profile().provider_model,
                claude_pane_status_label(pane.status.clone())
            );
            description.push_str(&self.whip_status_suffix_for_target(&node_id));
            if let Some(status) = pane.latest_turn_status {
                description.push_str(&format!("; latest status: {}", status.label()));
            }
            if let Some(status) = pane.latest_usage_status {
                match (status, pane.latest_usage_summary.as_deref()) {
                    (ClaudePaneUsageStatus::Reported, Some(usage)) => {
                        description.push_str(&format!("; latest usage: {usage}"));
                    }
                    _ => {
                        description.push_str(&format!("; latest usage: {}", status.label()));
                    }
                }
            }
            if let Some(path) = pane.latest_audit_path.as_ref() {
                description.push_str(&format!("; audit: {}", path.display()));
            }
            if let Some(task) = pane.latest_task_message.as_deref() {
                description.push_str(&format!("; task: {task}"));
            }
            if let Some(result) = pane.latest_result_message.as_deref() {
                description.push_str(&format!("; result: {result}"));
            }
            items.push(SelectionItem {
                name: pane.title.clone(),
                description: Some(description),
                is_current: self.claude_panes.active_user_pane_id() == pane.id,
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::SelectUserPane {
                        pane_id: pane_id_for_action.clone(),
                    });
                })],
                dismiss_on_select: true,
                selected_shortcuts: vec![rename_claude_pane_shortcut(pane_id)],
                search_value: Some(format!("{} {}", pane.title, pane.id)),
                ..Default::default()
            });
        }
        items
    }

    fn codex_user_pane_items(&self) -> Vec<SelectionItem> {
        self.agent_navigation
            .ordered_threads()
            .into_iter()
            .filter(|(thread_id, entry)| self.is_operator_owned_codex_user_pane(*thread_id, entry))
            .map(|(thread_id, entry)| {
                let name = entry
                    .agent_nickname
                    .as_deref()
                    .filter(|nickname| !nickname.trim().is_empty())
                    .map(native_pane_title)
                    .unwrap_or_else(|| native_pane_title(&short_thread_id(thread_id)));
                let description = self.codex_pane_description(thread_id, entry);
                let mut item = SelectionItem {
                    name: name.clone(),
                    name_prefix_spans: crate::multi_agents::agent_picker_status_dot_spans(
                        entry.is_closed,
                    ),
                    description: Some(description),
                    is_current: self.claude_panes.active_user_pane_id() == CODEX_MAIN_PANE_ID
                        && self.active_thread_id == Some(thread_id),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::SelectAgentThread(thread_id));
                    })],
                    dismiss_on_select: true,
                    selected_shortcuts: vec![rename_codex_pane_shortcut(thread_id)],
                    search_value: Some(format!("{name} {thread_id}")),
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
            .collect()
    }

    fn codex_main_pane_description(&self) -> String {
        let Some(thread_id) = self.primary_thread_id else {
            return "model unavailable; loading".to_string();
        };
        let entry = self.agent_navigation.get(&thread_id);
        let model = self
            .primary_session_model(thread_id)
            .or_else(|| entry.and_then(|entry| entry.model.as_deref()))
            .unwrap_or("model unavailable");
        let mut description = format!("{model}; {}", native_thread_status_label(entry));
        append_context_left(
            &mut description,
            self.spawn_context_left_by_thread.get(&thread_id),
        );
        description.push_str(&self.whip_status_suffix_for_target(&thread_node_id(thread_id)));
        description
    }

    pub(crate) fn codex_pane_description(
        &self,
        thread_id: ThreadId,
        entry: &crate::multi_agents::AgentPickerThreadEntry,
    ) -> String {
        let model = entry.model.as_deref().unwrap_or("model unavailable");
        let mut description = format!("{model}; {}", native_thread_status_label(Some(entry)));
        append_context_left(
            &mut description,
            self.spawn_context_left_by_thread.get(&thread_id),
        );
        description.push_str(&self.whip_status_suffix_for_target(&thread_node_id(thread_id)));
        if let Some(task) = entry.last_task_message.as_deref() {
            description.push_str(&format!(
                "; latest task: {}",
                truncate_for_display(task, /*max_chars*/ 80)
            ));
        }
        if let Some(result) = entry.last_result_message.as_deref() {
            description.push_str(&format!(
                "; latest result: {}",
                truncate_for_display(result, /*max_chars*/ 80)
            ));
        }
        description
    }

    pub(crate) fn next_codex_pane_nickname(&self) -> String {
        let count = self
            .agent_navigation
            .ordered_threads()
            .into_iter()
            .filter(|(thread_id, _)| Some(*thread_id) != self.primary_thread_id)
            .filter(|(thread_id, _)| !self.is_managed_spawn_crew_thread(*thread_id))
            .filter(|(_, entry)| {
                entry
                    .agent_role
                    .as_deref()
                    .map(|role| role == "default")
                    .unwrap_or(true)
            })
            .count();
        format!("Corbanu {}", count + 1)
    }
}

pub(crate) fn new_pane_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem {
            name: "+ Corbanu Terminal Pane".to_string(),
            description: Some(
                "Create a persistent native Corbanu Terminal pane; choose model next.".to_string(),
            ),
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::OpenCodexPaneModelPicker);
            })],
            dismiss_on_select: true,
            ..Default::default()
        },
        SelectionItem {
            name: "+ Claude Pane".to_string(),
            description: Some(
                "Create a Claude Code headless pane; choose provider next.".to_string(),
            ),
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::OpenClaudePaneProfilePicker);
            })],
            dismiss_on_select: true,
            ..Default::default()
        },
    ]
}

fn short_thread_id(thread_id: codex_protocol::ThreadId) -> String {
    thread_id.to_string().chars().take(8).collect()
}

fn claude_pane_status_label(status: ClaudePaneStatus) -> &'static str {
    match status {
        ClaudePaneStatus::Idle => "idle",
        ClaudePaneStatus::Running => "running",
    }
}

fn native_thread_status_label(
    entry: Option<&crate::multi_agents::AgentPickerThreadEntry>,
) -> &'static str {
    match entry {
        Some(entry) if entry.is_closed => "done",
        Some(entry) if entry.is_running => "running",
        Some(_) => "idle",
        None => "unknown",
    }
}

fn append_context_left(description: &mut String, context_left: Option<&i64>) {
    if let Some(context_left) = context_left {
        description.push_str(&format!("; ctx {context_left}%"));
    }
}

fn section_item(name: &str) -> SelectionItem {
    SelectionItem {
        name: name.to_string(),
        is_disabled: true,
        ..Default::default()
    }
}

fn rename_codex_pane_shortcut(thread_id: ThreadId) -> SelectionShortcutAction {
    SelectionShortcutAction {
        key: key_hint::plain(KeyCode::F(2)),
        action: Box::new(move |tx| {
            tx.send(AppEvent::OpenRenameCodexPanePrompt { thread_id });
        }),
        dismiss_on_select: true,
    }
}

fn rename_claude_pane_shortcut(pane_id: String) -> SelectionShortcutAction {
    SelectionShortcutAction {
        key: key_hint::plain(KeyCode::F(2)),
        action: Box::new(move |tx| {
            tx.send(AppEvent::OpenRenameClaudePanePrompt {
                pane_id: pane_id.clone(),
            });
        }),
        dismiss_on_select: true,
    }
}

/// #507: shown when another Corbanu Terminal on this home turned the kill
/// switch on.
pub(crate) const KILL_SWITCH_ON_ELSEWHERE: &str = "The security kill switch was turned on in another Corbanu Terminal on this home. Work running here was stopped (Claude panes included), and nothing is sent to the model until it is turned off in /security (here or there). Messages and tasks from other agents wait until then.";

/// #842: shown when Moderate confirmed in another Corbanu Terminal on this
/// home raised this one's sessions.
pub(crate) const MODERATE_CONFIRMED_ELSEWHERE: &str = "Moderate was confirmed in another Corbanu Terminal on this home, so Core's Moderate rules apply here now: agent commands always stay in the sandbox, credential and Corbanu files are protected from them, and after untrusted content protected actions need your fresh approval. Grants and \"for session\" approvals here ended. This session is shown as Permissive until you restart Corbanu Terminal; model keys through the credential broker start then. Commands already running keep the permissions they started with.";

/// #919 (Travis, 2026-10-16, D20): shown when Aggressive confirmed in
/// another Corbanu Terminal on this home raised this one's sessions;
/// `untrusted_content` when `source_envelopes` is on (Core's labels hold),
/// `active` the level this session is shown as until the restart.
pub(crate) fn aggressive_confirmed_elsewhere(
    untrusted_content: bool,
    active: crate::security::level::ChosenLevel,
) -> String {
    let effects = if untrusted_content {
        crate::bottom_pane::security_level_picker::CORE_EFFECTS
    } else {
        crate::bottom_pane::security_level_picker::CORE_EFFECTS_VAULT_ONLY
    };
    // A session shown as Moderate started with Moderate's credential broker
    // (D2), so the model keys are not among what waits for the restart.
    let broker = if active == crate::security::level::ChosenLevel::Moderate {
        ""
    } else {
        " model keys held by the credential broker,"
    };
    format!(
        "Aggressive was confirmed in another Corbanu Terminal on this home, so Core's Aggressive protections apply here now: {effects}. From each pane's next turn you approve every command that is not known read-only (reviewer: you), in every Corbanu pane and child agent of this Corbanu Terminal, and in those you start before you restart; a pane whose managed configuration rules this out keeps its own approval settings and says so. Grants and \"for session\" approvals here ended. The rest of Aggressive starts when you restart Corbanu Terminal: its sandbox (writes only in the current folder), the project's credential files (.env, .env.* and .envrc) denied to agent commands, network and web search off, secret-like environment variables removed and login profiles and shell snapshots unused,{broker} and the child-agent rules. Until then this session is shown as {} and keeps your own settings for these, and commands already running keep the permissions they started with.",
        active.name()
    )
}

/// #507: shown when another Corbanu Terminal on this home turned it off.
pub(crate) const KILL_SWITCH_OFF_ELSEWHERE: &str = "The security kill switch was turned off in another Corbanu Terminal on this home. Turns are sent to the model again. Nothing restarts by itself here: stopped replies and commands do not restart, and a task that waited for a Claude pane goes with that pane's next turn.";

/// #507: shown in a Claude pane when its turn carries the task held while the
/// kill switch was on.
const HELD_TASK_DELIVERED: &str =
    "Sent with this turn: the task that waited while the security kill switch was on.";

/// #507: how much of the delivered task the pane shows.
const HELD_TASK_HINT_CHARS: usize = 200;

/// #507: the prompt of a pane's first turn after the kill switch, with the
/// task held for it first (it arrived first), as a Codex pane gets its waiting
/// agent messages with its next turn.
pub(crate) fn with_held_claude_task(held: Option<&str>, prompt: String) -> String {
    match held {
        Some(held) => format!(
            "A task sent to you while the security kill switch was on (it waited for this turn):\n\n{held}\n\nThis turn's message:\n\n{prompt}"
        ),
        None => prompt,
    }
}
