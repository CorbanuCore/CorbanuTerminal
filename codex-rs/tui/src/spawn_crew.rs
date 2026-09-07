use crate::app::App;
use crate::app_server_session::AppServerSession;
use crate::claude_panes::CODEX_MAIN_PANE_ID;
use crate::crew_presets;
use crate::crew_state::CrewCreationStatus;
use crate::crew_state::CrewInstanceState;
use crate::spawn_orchestration::SpawnRole;
use crate::spawn_orchestration::node_id_pane;
use crate::spawn_orchestration::node_id_thread;
use crate::spawn_orchestration::pane_node_id;
use crate::spawn_orchestration::spawn_role_from_agent_type;
use crate::spawn_orchestration::thread_node_id;
use crate::tui;
use codex_protocol::ThreadId;
use codex_protocol::crew::AgentClass;
use codex_protocol::crew::CrewSpec;
use codex_protocol::crew::RuntimeRequest;
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use std::collections::HashMap;
use std::collections::HashSet;

impl App {
    /// Permanently remove the one CrewSpec-owned hierarchy while preserving a user-owned root
    /// that was only bound as Nazgul. This action is intentionally whole-crew-only: member-level
    /// lifecycle commands remain rejected so Core graph and pane-layout ownership cannot diverge.
    pub(crate) async fn remove_spawn_crew(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
    ) -> Result<String> {
        if self.spawn_legacy_read_only {
            return Err(eyre!(
                "The restored legacy hierarchy has no verified CrewSpec ownership and cannot be removed automatically."
            ));
        }
        let crew = self
            .spawn_crew
            .clone()
            .ok_or_else(|| eyre!("No managed crew exists."))?;
        let root_node_id = crew
            .spec
            .members
            .iter()
            .find(|member| member.parent_member_id.is_none())
            .and_then(|member| crew.member_node_by_id.get(&member.logical_member_id))
            .cloned();
        let preserved_bound_root = root_node_id.as_ref().filter(|node_id| {
            node_id_thread(node_id).is_none()
                || !self.spawn_parent_by_node.contains_key(node_id.as_str())
        });

        let mut ordered_nodes = crew
            .spec
            .members
            .iter()
            .filter_map(|member| {
                crew.member_node_by_id
                    .get(&member.logical_member_id)
                    .cloned()
            })
            .collect::<Vec<_>>();
        for node_id in crew.member_node_by_id.values() {
            if !ordered_nodes.contains(node_id) {
                ordered_nodes.push(node_id.clone());
            }
        }
        let member_nodes = ordered_nodes.iter().cloned().collect::<HashSet<_>>();
        let mut native_thread_ids = Vec::new();
        let mut claude_pane_ids = Vec::new();
        for node_id in &ordered_nodes {
            if preserved_bound_root.is_some_and(|root| root == node_id) {
                continue;
            }
            if let Some(thread_id) = node_id_thread(node_id) {
                native_thread_ids.push(thread_id);
            } else if let Some(pane_id) = node_id_pane(node_id) {
                claude_pane_ids.push(pane_id.to_string());
            } else {
                return Err(eyre!(
                    "Crew member node `{node_id}` is neither a native thread nor a managed Claude pane; no data was removed."
                ));
            }
        }

        // Validate every external artifact boundary before the first destructive mutation.
        for pane_id in &claude_pane_ids {
            let pane = self
                .claude_panes
                .panes()
                .iter()
                .find(|pane| pane.id == *pane_id)
                .ok_or_else(|| {
                    eyre!(
                        "Managed Claude pane `{pane_id}` is missing from the live registry; no data was removed."
                    )
                })?;
            if pane.spawn_role.is_none() {
                return Err(eyre!(
                    "Claude pane `{}` is not marked as a managed crew member; no data was removed.",
                    pane.title
                ));
            }
            let expected = self.config.codex_home.join("panes").join(pane_id);
            if pane.artifact_dir.as_path() != expected.as_path() {
                return Err(eyre!(
                    "Managed Claude pane `{}` has artifact path `{}` outside `{}`; no data was removed.",
                    pane.title,
                    pane.artifact_dir.display(),
                    expected.display()
                ));
            }
        }

        let primary_thread_id = self.primary_thread_id.ok_or_else(|| {
            eyre!("Corbanu Terminal Main is unavailable; no crew data was removed.")
        })?;
        self.save_active_claude_pane_transcript();
        self.claude_panes
            .set_active_user_pane(CODEX_MAIN_PANE_ID)
            .map_err(|err| eyre!(err.to_string()))?;
        if self.active_thread_id != Some(primary_thread_id) {
            self.select_agent_thread_and_discard_side(tui, app_server, primary_thread_id)
                .await?;
        }

        // CrewSpec requires parents to precede children. Delete in reverse order so each explicit
        // child is reconciled before its parent; app-server deletion also shuts down that member's
        // non-CrewSpec descendants.
        for thread_id in native_thread_ids.iter().rev().copied() {
            app_server.thread_delete(thread_id).await.map_err(|err| {
                eyre!(
                    "Failed to remove native crew member {thread_id}; CrewSpec ownership was retained for recovery: {err:#}"
                )
            })?;
        }
        for pane_id in claude_pane_ids.iter().rev() {
            self.claude_panes
                .remove_managed_crew_pane(pane_id, self.config.codex_home.as_ref())
                .map_err(|err| {
                    eyre!(
                        "Failed to remove managed Claude pane {pane_id}; CrewSpec ownership was retained for recovery: {err:#}"
                    )
                })?;
            self.claude_pane_transcript_cells.remove(pane_id);
        }

        let native_thread_set = native_thread_ids.iter().copied().collect::<HashSet<_>>();
        self.spawn_crew = None;
        self.spawn_legacy_read_only = false;
        self.spawn_nazgul_pane_id = None;
        self.spawn_nazgul_rebind_required = false;
        self.spawn_parent_by_thread.retain(|child, parent| {
            !native_thread_set.contains(child) && !native_thread_set.contains(parent)
        });
        self.spawn_parent_by_node.retain(|child, parent| {
            !member_nodes.contains(child) && !member_nodes.contains(parent)
        });
        self.spawn_native_runtime_by_node
            .retain(|node_id, _| !member_nodes.contains(node_id));
        self.spawn_native_endpoint_by_node
            .retain(|node_id, thread_id| {
                !member_nodes.contains(node_id) && !native_thread_set.contains(thread_id)
            });
        self.spawn_status_by_thread
            .retain(|thread_id, _| !native_thread_set.contains(thread_id));
        self.spawn_waiting_for_agents_by_thread
            .retain(|thread_id, _| !native_thread_set.contains(thread_id));
        self.spawn_parent_reports_by_node
            .retain(|node_id, _| !member_nodes.contains(node_id));
        self.spawn_dispatch_acks_by_target_task
            .retain(|(target, _), _| !member_nodes.contains(target));
        self.spawn_processed_terminal_turns
            .retain(|(thread_id, _)| !native_thread_set.contains(thread_id));
        self.spawn_auto_loop_state_by_node
            .retain(|node_id, _| !member_nodes.contains(node_id));
        self.spawn_quarantine_notified_by_node
            .retain(|node_id| !member_nodes.contains(node_id));
        self.spawn_context_left_by_thread
            .retain(|thread_id, _| !native_thread_set.contains(thread_id));
        self.spawn_last_report_seq_by_node
            .retain(|node_id, _| !member_nodes.contains(node_id));
        self.spawn_last_dispatch_seq_by_node
            .retain(|node_id, _| !member_nodes.contains(node_id));
        self.spawn_last_event_at_by_node
            .retain(|node_id, _| !member_nodes.contains(node_id));
        self.orchestrate_whips.retain(|_, whip| {
            !member_nodes.contains(&whip.target)
                && whip
                    .holder
                    .as_ref()
                    .is_none_or(|holder| !member_nodes.contains(holder))
        });
        self.orchestrate_idle_generation_by_target
            .retain(|node_id, _| !member_nodes.contains(node_id));

        for thread_id in native_thread_ids.iter().copied() {
            self.discard_thread_local_state(thread_id).await;
            // Whole-crew removal is permanent, unlike a normal agent shutdown. Do not leave a
            // closed navigation/status projection behind after the backend thread and CrewSpec
            // member have both been deleted.
            self.agent_navigation.remove(thread_id);
            self.spawn_status_by_thread.remove(&thread_id);
        }
        self.sync_active_agent_label();
        self.persist_pane_state();
        Ok(format!(
            "Removed managed crew {}: {} native member(s), {} Claude member(s){}.",
            crew.spec.crew_id,
            native_thread_ids.len(),
            claude_pane_ids.len(),
            preserved_bound_root
                .map(|_| "; preserved the bound user-owned root")
                .unwrap_or_default()
        ))
    }

    pub(crate) fn validate_restored_crew_state(&self) -> Result<()> {
        let Some(state) = self.spawn_crew.as_ref() else {
            return Ok(());
        };
        state
            .spec
            .validate()
            .map_err(|err| eyre!("restored crew specification is invalid: {err}"))?;
        if !matches!(state.status, CrewCreationStatus::Ready) {
            return Err(eyre!(
                "restored crew {} is not ready; resume its explicit creation flow",
                state.spec.crew_id
            ));
        }
        if state.member_node_by_id.len() != state.spec.members.len() {
            return Err(eyre!(
                "restored crew {} has {} member mappings for {} members",
                state.spec.crew_id,
                state.member_node_by_id.len(),
                state.spec.members.len()
            ));
        }

        for member in &state.spec.members {
            let node_id = state
                .member_node_by_id
                .get(&member.logical_member_id)
                .ok_or_else(|| {
                    eyre!(
                        "restored crew member {} has no native identity",
                        member.logical_member_id
                    )
                })?;
            let thread_id = self.spawn_node_backing_thread_id(node_id).ok_or_else(|| {
                eyre!(
                    "restored crew member {} maps to stale native node {}",
                    member.logical_member_id,
                    node_id
                )
            })?;
            if self.agent_navigation.get(&thread_id).is_none() {
                return Err(eyre!(
                    "restored crew member {} maps to unavailable thread {}",
                    member.logical_member_id,
                    thread_id
                ));
            }

            let expected_parent = match member.parent_member_id.as_deref() {
                Some(parent_member_id) => state
                    .member_node_by_id
                    .get(parent_member_id)
                    .cloned()
                    .ok_or_else(|| {
                        eyre!(
                            "restored crew member {} references unmapped parent {}",
                            member.logical_member_id,
                            parent_member_id
                        )
                    })?,
                None => pane_node_id(CODEX_MAIN_PANE_ID),
            };
            if self.spawn_parent_by_node.get(node_id) != Some(&expected_parent) {
                return Err(eyre!(
                    "restored crew member {} has a stale or changed parent edge",
                    member.logical_member_id
                ));
            }

            let RuntimeRequest::Exact {
                provider_id,
                model_id,
                reasoning_effort,
                ..
            } = &member.runtime_request
            else {
                return Err(eyre!(
                    "restored crew member {} does not have an exact runtime",
                    member.logical_member_id
                ));
            };
            let runtime = self
                .spawn_native_runtime_by_node
                .get(node_id)
                .ok_or_else(|| {
                    eyre!(
                        "restored crew member {} has no persisted runtime",
                        member.logical_member_id
                    )
                })?;
            if runtime.provider != *provider_id
                || runtime.model != *model_id
                || reasoning_effort
                    .as_ref()
                    .is_some_and(|expected| runtime.reasoning_effort.as_ref() != Some(expected))
            {
                return Err(eyre!(
                    "restored crew member {} runtime changed from {}/{} {:?} to {}/{} {:?}",
                    member.logical_member_id,
                    provider_id,
                    model_id,
                    reasoning_effort,
                    runtime.provider,
                    runtime.model,
                    runtime.reasoning_effort
                ));
            }
        }
        Ok(())
    }

    async fn ensure_crew_providers_ready(&self, crew: &CrewSpec) -> Result<()> {
        crew.validate()
            .map_err(|err| eyre!("The requested crew is invalid: {err}"))?;
        let mut checked_providers = HashSet::new();
        for member in &crew.members {
            let RuntimeRequest::Exact { provider_id, .. } = &member.runtime_request else {
                return Err(eyre!(
                    "Crew member {} does not have a resolved exact runtime.",
                    member.logical_member_id
                ));
            };
            if checked_providers.insert(provider_id.clone()) {
                self.ensure_native_spawn_provider_ready(Some(provider_id))
                    .await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn create_spawn_standard_crew(
        &mut self,
        app_server: &mut AppServerSession,
    ) -> Result<(ThreadId, ThreadId)> {
        self.create_spawn_crew(app_server, crew_presets::standard_crew_spec())
            .await
    }

    pub(crate) async fn create_spawn_corbanu_api_crew(
        &mut self,
        app_server: &mut AppServerSession,
    ) -> Result<(ThreadId, ThreadId)> {
        self.create_spawn_crew(app_server, crew_presets::corbanu_api_crew_spec())
            .await
    }

    async fn create_spawn_crew(
        &mut self,
        app_server: &mut AppServerSession,
        crew: CrewSpec,
    ) -> Result<(ThreadId, ThreadId)> {
        let root_thread_id = self
            .primary_thread_id
            .or(self.active_thread_id)
            .ok_or_else(|| {
                eyre!("Cannot create a crew before Corbanu Terminal Main has started.")
            })?;
        let spawn_config = self.native_spawn_agent_config()?;
        self.ensure_crew_providers_ready(&crew).await?;
        match self.spawn_crew.as_mut() {
            Some(existing)
                if existing.spec.schema_version == crew.schema_version
                    && existing.spec.preset_id == crew.preset_id
                    && existing.spec.members == crew.members
                    && existing.spec.policy == crew.policy =>
            {
                if !matches!(existing.status, CrewCreationStatus::Ready) {
                    existing.resume_creation();
                }
            }
            Some(existing) => {
                return Err(eyre!(
                    "Cannot create preset {:?}: crew {} using preset {:?} already owns this /spawn hierarchy.",
                    crew.preset_id,
                    existing.spec.crew_id,
                    existing.spec.preset_id
                ));
            }
            None => {
                self.spawn_crew = Some(
                    CrewInstanceState::begin(crew.clone())
                        .map_err(|err| eyre!("Cannot create the crew intent: {err}"))?,
                );
            }
        }
        self.persist_pane_state();

        let mut spawned_members = HashMap::<String, (ThreadId, String)>::new();
        let mut nazgul_thread_id = None;
        let mut troll_thread_id = None;
        for member in crew.members {
            let role = spawn_role_from_agent_type(&member.role_profile).ok_or_else(|| {
                eyre!(
                    "Crew member {} has unsupported role profile {}.",
                    member.logical_member_id,
                    member.role_profile
                )
            })?;
            let RuntimeRequest::Exact {
                provider_id,
                model_id,
                reasoning_effort,
                ..
            } = member.runtime_request
            else {
                return Err(eyre!(
                    "Crew member {} does not have a resolved exact runtime.",
                    member.logical_member_id
                ));
            };
            let (parent_thread_id, parent_node_id) =
                if let Some(parent_member_id) = member.parent_member_id.as_deref() {
                    spawned_members
                        .get(parent_member_id)
                        .cloned()
                        .ok_or_else(|| {
                            eyre!(
                                "Crew member {} references unavailable parent {parent_member_id}.",
                                member.logical_member_id
                            )
                        })?
                } else {
                    (root_thread_id, self.spawn_root_node_id())
                };

            if let Some((thread_id, node_id)) = self.validate_existing_crew_member(
                &member.logical_member_id,
                &parent_node_id,
                &provider_id,
                &model_id,
                reasoning_effort.as_ref(),
            )? {
                spawned_members.insert(member.logical_member_id, (thread_id, node_id));
                match role {
                    SpawnRole::Nazgul => nazgul_thread_id = Some(thread_id),
                    SpawnRole::Troll => troll_thread_id = Some(thread_id),
                    SpawnRole::Orc => {}
                }
                continue;
            }

            let nickname = self.next_spawn_agent_nickname(role);
            let started = match app_server
                .spawn_agent_thread_with_class(
                    &spawn_config,
                    parent_thread_id,
                    member.role_profile.clone(),
                    nickname.clone(),
                    AgentClass::CrewMember {
                        crew_id: crew.crew_id.clone(),
                        logical_member_id: member.logical_member_id.clone(),
                        human_addressable: true,
                    },
                    model_id,
                    Some(provider_id),
                    reasoning_effort,
                    /*base_instructions*/ None,
                )
                .await
            {
                Ok(started) => started,
                Err(err) => {
                    self.mark_crew_incomplete(format!("{err:#}"));
                    return Err(err);
                }
            };
            let thread_id = started.session.thread_id;
            self.register_spawn_agent_pane(
                thread_id,
                parent_thread_id,
                parent_node_id,
                nickname,
                &member.role_profile,
                started,
                /*persist_layout*/ false,
            )
            .await;
            let node_id = thread_node_id(thread_id);
            let logical_member_id = member.logical_member_id;
            if let Err(err) = self
                .spawn_crew
                .as_mut()
                .ok_or_else(|| eyre!("The crew intent disappeared during creation."))?
                .record_member(&logical_member_id, &node_id)
            {
                self.mark_crew_incomplete(err.to_string());
                return Err(eyre!("Cannot persist crew member identity: {err}"));
            }
            spawned_members.insert(logical_member_id, (thread_id, node_id.clone()));
            match role {
                SpawnRole::Nazgul => {
                    nazgul_thread_id = Some(thread_id);
                    self.set_spawn_nazgul_pane_binding(node_id);
                    self.persist_bound_nazgul_root_thread_metadata().await;
                }
                SpawnRole::Troll => troll_thread_id = Some(thread_id),
                SpawnRole::Orc => {}
            }
            self.persist_pane_state();
        }

        self.spawn_crew
            .as_mut()
            .ok_or_else(|| eyre!("The crew intent disappeared before completion."))?
            .mark_ready()
            .map_err(|err| eyre!("Cannot mark the crew ready: {err}"))?;
        self.persist_pane_state();
        Ok((
            nazgul_thread_id.ok_or_else(|| eyre!("The crew did not create a Nazgul."))?,
            troll_thread_id.ok_or_else(|| eyre!("The crew did not create a Troll."))?,
        ))
    }

    fn validate_existing_crew_member(
        &self,
        logical_member_id: &str,
        parent_node_id: &str,
        provider_id: &str,
        model_id: &str,
        reasoning_effort: Option<&codex_protocol::openai_models::ReasoningEffort>,
    ) -> Result<Option<(ThreadId, String)>> {
        let Some(existing_node_id) = self
            .spawn_crew
            .as_ref()
            .and_then(|state| state.member_node_by_id.get(logical_member_id))
            .cloned()
        else {
            return Ok(None);
        };
        let thread_id = self
            .spawn_node_backing_thread_id(&existing_node_id)
            .ok_or_else(|| {
                eyre!(
                    "Crew member {logical_member_id} is mapped to {existing_node_id}, but that native thread is not live or resumable. Recover the saved thread instead of creating a duplicate."
                )
            })?;
        let saved_runtime = self
            .spawn_native_runtime_by_node
            .get(&existing_node_id)
            .ok_or_else(|| {
                eyre!("Crew member {logical_member_id} is missing its persisted runtime.")
            })?;
        if saved_runtime.provider != provider_id
            || saved_runtime.model != model_id
            || reasoning_effort
                .is_some_and(|expected| saved_runtime.reasoning_effort.as_ref() != Some(expected))
        {
            return Err(eyre!(
                "Crew member {logical_member_id} was persisted as {}/{} {:?}, not the requested {provider_id}/{model_id} {:?}. Explicitly migrate the runtime instead of silently changing it.",
                saved_runtime.provider,
                saved_runtime.model,
                saved_runtime.reasoning_effort,
                reasoning_effort
            ));
        }
        if self
            .spawn_parent_by_node
            .get(&existing_node_id)
            .map(String::as_str)
            != Some(parent_node_id)
        {
            return Err(eyre!(
                "Crew member {logical_member_id} no longer has its persisted parent. Repair the crew mapping instead of silently reparenting it."
            ));
        }
        Ok(Some((thread_id, existing_node_id)))
    }

    pub(crate) fn mark_crew_incomplete(&mut self, error: String) {
        if let Some(state) = self.spawn_crew.as_mut() {
            state.mark_incomplete(error);
        }
        self.persist_pane_state();
    }
}
