use crate::app::App;
use crate::app_server_session::AppServerSession;
use crate::claude_panes::CODEX_MAIN_PANE_ID;
use crate::crew_presets;
use crate::crew_state::CrewCreationStatus;
use crate::crew_state::CrewInstanceState;
use crate::spawn_orchestration::SpawnRole;
use crate::spawn_orchestration::pane_node_id;
use crate::spawn_orchestration::spawn_role_from_agent_type;
use crate::spawn_orchestration::thread_node_id;
use codex_protocol::ThreadId;
use codex_protocol::crew::CrewSpec;
use codex_protocol::crew::RuntimeRequest;
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use std::collections::HashMap;
use std::collections::HashSet;

impl App {
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
                || runtime.reasoning_effort != *reasoning_effort
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

    fn ensure_crew_providers_ready(&self, crew: &CrewSpec) -> Result<()> {
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
                self.ensure_native_spawn_provider_ready(Some(provider_id))?;
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

    async fn create_spawn_crew(
        &mut self,
        app_server: &mut AppServerSession,
        crew: CrewSpec,
    ) -> Result<(ThreadId, ThreadId)> {
        let root_thread_id = self
            .primary_thread_id
            .or(self.active_thread_id)
            .ok_or_else(|| eyre!("Cannot create a crew before Codex Main has started."))?;
        let spawn_config = self.native_spawn_agent_config()?;
        self.ensure_crew_providers_ready(&crew)?;
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
                .spawn_agent_thread(
                    &spawn_config,
                    parent_thread_id,
                    member.role_profile.clone(),
                    nickname.clone(),
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
                false,
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
            || saved_runtime.reasoning_effort.as_ref() != reasoning_effort
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
