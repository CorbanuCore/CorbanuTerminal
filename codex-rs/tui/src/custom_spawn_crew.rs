use crate::app::App;
use crate::claude_panes::CODEX_MAIN_PANE_ID;
use crate::crew_state::CrewCreationStatus;
use crate::crew_state::CrewInstanceState;
use crate::dispatch_queue::SavedNativeSpawnRuntime;
use crate::spawn_orchestration::SpawnRole;
use codex_protocol::crew::AgentClass;
use codex_protocol::crew::CURRENT_CREW_SCHEMA_VERSION;
use codex_protocol::crew::CrewMemberSpec;
use codex_protocol::crew::CrewPolicy;
use codex_protocol::crew::CrewSpec;
use codex_protocol::crew::DelegationMode;
use codex_protocol::crew::RuntimeRequest;
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;

impl App {
    pub(crate) fn prepare_custom_spawn_root(
        &mut self,
        display_name: String,
        runtime: SavedNativeSpawnRuntime,
    ) -> Result<AgentClass> {
        self.preflight_new_custom_spawn_root()?;
        let member = CrewMemberSpec {
            logical_member_id: "nazgul".to_string(),
            display_name,
            role_profile: "nazgul".to_string(),
            parent_member_id: None,
            runtime_request: RuntimeRequest::exact(
                runtime.provider,
                runtime.model,
                runtime.reasoning_effort,
            ),
        };
        let spec = CrewSpec {
            schema_version: CURRENT_CREW_SCHEMA_VERSION,
            crew_id: "custom".to_string(),
            preset_id: None,
            members: vec![member],
            policy: CrewPolicy {
                delegation_mode: DelegationMode::Proactive,
                allow_ephemeral_descendants: true,
                provider_allowlist: self.authorized_spawn_providers(),
                maximum_spend_usd: None,
            },
        };
        let state = CrewInstanceState::begin(spec)
            .map_err(|err| eyre!("Cannot prepare custom crew metadata: {err}"))?;
        let agent_class = AgentClass::CrewMember {
            crew_id: state.spec.crew_id.clone(),
            logical_member_id: "nazgul".to_string(),
            human_addressable: true,
        };
        self.spawn_crew = Some(state);
        self.spawn_legacy_read_only = false;
        Ok(agent_class)
    }

    pub(crate) fn abort_prepared_custom_spawn_root(&mut self, crew_id: &str) {
        let should_abort = self.spawn_crew.as_ref().is_some_and(|crew| {
            crew.spec.crew_id == crew_id
                && matches!(crew.status, CrewCreationStatus::Creating)
                && crew.member_node_by_id.is_empty()
        });
        if should_abort {
            self.spawn_crew = None;
        }
    }

    pub(crate) fn custom_spawn_member_agent_class(&self, role: SpawnRole) -> Result<AgentClass> {
        let state = self
            .spawn_crew
            .as_ref()
            .ok_or_else(|| eyre!("Custom crew root was not created."))?;
        if !matches!(state.status, CrewCreationStatus::Ready) {
            return Err(eyre!("Custom crew is not ready for another member."));
        }
        let logical_member_id = Self::next_custom_spawn_member_id(role, state)?;
        Ok(AgentClass::CrewMember {
            crew_id: state.spec.crew_id.clone(),
            logical_member_id,
            human_addressable: true,
        })
    }

    /// Reject creating a second root before any app-server thread is started.
    ///
    /// Root creation used to spawn/register the native pane first and only then discover that the
    /// existing CrewSpec already owned a different root. That left a live orphan pane after a
    /// failed product action. Keep this preflight read-only so callers can make creation atomic.
    pub(crate) fn preflight_new_custom_spawn_root(&self) -> Result<()> {
        let Some(existing) = self.spawn_crew.as_ref() else {
            return Ok(());
        };
        let root_node_id = existing
            .spec
            .members
            .iter()
            .find(|member| member.parent_member_id.is_none())
            .and_then(|member| existing.member_node_by_id.get(&member.logical_member_id));
        let root_label = root_node_id
            .map(|node_id| self.nazgul_bound_display_title(node_id))
            .unwrap_or_else(|| "the existing Nazgul root".to_string());
        Err(eyre!(
            "Crew {} already has {root_label}. Select it in /panes or remove the existing crew before creating another Nazgul. No pane was created.",
            existing.spec.crew_id
        ))
    }

    pub(crate) fn ensure_custom_spawn_root(&mut self, node_id: &str) -> Result<()> {
        if let Some(existing) = self.spawn_crew.as_mut() {
            let root = existing
                .spec
                .members
                .iter()
                .find(|member| member.parent_member_id.is_none())
                .and_then(|member| existing.member_node_by_id.get(&member.logical_member_id));
            if root.is_some_and(|root| root == node_id) {
                return Ok(());
            }
            if root.is_none()
                && matches!(existing.status, CrewCreationStatus::Creating)
                && existing.member_node_by_id.is_empty()
            {
                let logical_member_id = existing
                    .spec
                    .members
                    .iter()
                    .find(|member| member.parent_member_id.is_none())
                    .map(|member| member.logical_member_id.clone())
                    .ok_or_else(|| eyre!("Prepared custom crew has no root member."))?;
                existing
                    .record_member(&logical_member_id, node_id)
                    .map_err(|err| eyre!("Cannot map custom Nazgul root: {err}"))?;
                existing
                    .mark_ready()
                    .map_err(|err| eyre!("Cannot finalize custom crew root: {err}"))?;
                return Ok(());
            }
            return Err(eyre!(
                "Crew {} already has a different Nazgul root. Remove that crew before rebinding.",
                existing.spec.crew_id
            ));
        }

        let runtime = self.custom_spawn_runtime_for_node(node_id)?;
        let member = CrewMemberSpec {
            logical_member_id: "nazgul".to_string(),
            display_name: self.nazgul_bound_display_title(node_id),
            role_profile: "nazgul".to_string(),
            parent_member_id: None,
            runtime_request: RuntimeRequest::exact(
                runtime.provider.clone(),
                runtime.model,
                runtime.reasoning_effort,
            ),
        };
        let spec = CrewSpec {
            schema_version: CURRENT_CREW_SCHEMA_VERSION,
            crew_id: "custom".to_string(),
            preset_id: None,
            members: vec![member],
            policy: CrewPolicy {
                delegation_mode: DelegationMode::Proactive,
                allow_ephemeral_descendants: true,
                provider_allowlist: self.authorized_spawn_providers(),
                maximum_spend_usd: None,
            },
        };
        let mut state = CrewInstanceState::begin(spec)
            .map_err(|err| eyre!("Cannot create custom crew metadata: {err}"))?;
        state
            .record_member("nazgul", node_id)
            .map_err(|err| eyre!("Cannot map custom Nazgul root: {err}"))?;
        state
            .mark_ready()
            .map_err(|err| eyre!("Cannot finalize custom crew root: {err}"))?;
        self.spawn_crew = Some(state);
        self.spawn_legacy_read_only = false;
        Ok(())
    }

    pub(crate) fn record_custom_spawn_member(
        &mut self,
        node_id: &str,
        parent_node_id: &str,
        role: SpawnRole,
        display_name: String,
        runtime: SavedNativeSpawnRuntime,
    ) -> Result<()> {
        if self.spawn_crew.is_none() {
            let root_node_id = self
                .spawn_nazgul_pane_id
                .clone()
                .unwrap_or_else(|| self.spawn_root_node_id());
            self.ensure_custom_spawn_root(&root_node_id)?;
        }
        let state = self
            .spawn_crew
            .as_mut()
            .ok_or_else(|| eyre!("Custom crew root was not created."))?;
        let parent_member_id = state
            .member_node_by_id
            .iter()
            .find_map(|(member_id, mapped_node)| {
                (mapped_node == parent_node_id).then(|| member_id.clone())
            })
            .ok_or_else(|| {
                eyre!(
                    "Cannot add {}: parent node {parent_node_id} is not a crew member.",
                    role.label()
                )
            })?;
        let prefix = role.agent_type().unwrap_or("member");
        let logical_member_id = Self::next_custom_spawn_member_id(role, state)?;
        state
            .add_ready_member(
                CrewMemberSpec {
                    logical_member_id,
                    display_name,
                    role_profile: prefix.to_string(),
                    parent_member_id: Some(parent_member_id),
                    runtime_request: RuntimeRequest::exact(
                        runtime.provider,
                        runtime.model,
                        runtime.reasoning_effort,
                    ),
                },
                node_id,
            )
            .map_err(|err| eyre!("Cannot persist custom crew member: {err}"))?;
        self.spawn_legacy_read_only = false;
        Ok(())
    }

    fn next_custom_spawn_member_id(role: SpawnRole, state: &CrewInstanceState) -> Result<String> {
        let prefix = role.agent_type().unwrap_or("member");
        (1_u64..=u64::MAX)
            .map(|index| format!("{prefix}-{index}"))
            .find(|candidate| {
                !state
                    .spec
                    .members
                    .iter()
                    .any(|member| member.logical_member_id == *candidate)
            })
            .ok_or_else(|| eyre!("Crew has exhausted the {prefix} member identifier space."))
    }

    fn custom_spawn_runtime_for_node(&self, node_id: &str) -> Result<SavedNativeSpawnRuntime> {
        if let Some(runtime) = self.spawn_native_runtime_by_node.get(node_id) {
            return Ok(runtime.clone());
        }
        if node_id == CODEX_MAIN_PANE_ID {
            return Ok(SavedNativeSpawnRuntime {
                model: self.chat_widget.current_model().to_string(),
                provider: self.config.model_provider_id.clone(),
                reasoning_effort: self.config.model_reasoning_effort.clone(),
            });
        }
        let pane_id = node_id
            .strip_prefix("pane:")
            .ok_or_else(|| eyre!("Nazgul root {node_id} has no persisted runtime."))?;
        let pane = self
            .claude_panes
            .panes()
            .iter()
            .find(|pane| pane.id == pane_id)
            .ok_or_else(|| eyre!("Nazgul root {node_id} is unavailable."))?;
        let profile = pane.profile.profile();
        Ok(SavedNativeSpawnRuntime {
            model: profile.provider_model.to_string(),
            provider: format!("external-claude:{:?}", pane.profile).to_ascii_lowercase(),
            reasoning_effort: None,
        })
    }
}
