//! Local registry lookups for startup, legacy context and loaded descendant checks.
//! These use the same local registry and thread manager as lifecycle operations.

use super::LocalAgentRuntime;
use crate::agent::types::AgentMetadata;
use crate::session_prefix::format_subagent_context_line;
use crate::thread_manager::ThreadManagerState;
use codex_protocol::ThreadId;
use codex_protocol::error::AgentErrorContext;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use std::collections::HashMap;
use std::sync::Arc;

impl LocalAgentRuntime {
    pub(crate) fn register_session_root(
        &self,
        current_thread_id: ThreadId,
        current_parent_thread_id: Option<ThreadId>,
    ) {
        if current_parent_thread_id.is_none() {
            self.registry.register_root_thread(current_thread_id);
        }
    }

    pub(crate) fn ensure_agent_known(&self, agent_id: ThreadId) -> CodexResult<AgentMetadata> {
        self.registry
            .agent_metadata_for_thread(agent_id)
            .ok_or_else(|| CodexErr::ThreadNotFound(agent_id))
    }

    pub(crate) async fn list_live_agent_subtree_thread_ids(
        &self,
        agent_id: ThreadId,
    ) -> CodexResult<Vec<ThreadId>> {
        let mut thread_ids = vec![agent_id];
        thread_ids.extend(self.live_thread_spawn_descendants(agent_id).await?);
        Ok(thread_ids)
    }

    pub(crate) async fn format_legacy_environment_context_subagents(
        &self,
        parent_thread_id: ThreadId,
    ) -> String {
        let Ok(agents) = self.open_thread_spawn_children(parent_thread_id).await else {
            return String::new();
        };
        // Corbanu: each line also carries the agent's role, status and last task/result.
        let state = self.upgrade().ok();
        let mut lines = Vec::new();
        for (thread_id, metadata) in agents {
            let status = match state.as_ref() {
                Some(state) => match state.get_thread(thread_id).await {
                    Ok(thread) => Some(thread.agent_status().await),
                    Err(_) => None,
                },
                None => None,
            };
            let last_result_message = metadata
                .last_result_message
                .clone()
                .or_else(|| status.as_ref().and_then(super::result_message_from_status));
            let reference = metadata
                .agent_path
                .as_ref()
                .map(|path| path.name().to_string())
                .unwrap_or_else(|| thread_id.to_string());
            lines.push(format_subagent_context_line(
                &reference,
                metadata.agent_nickname.as_deref(),
                metadata.agent_role.as_deref(),
                status.as_ref(),
                metadata.last_task_message.as_deref(),
                last_result_message.as_deref(),
            ));
        }
        lines
            .into_iter()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(super) async fn open_thread_spawn_children(
        &self,
        parent_thread_id: ThreadId,
    ) -> CodexResult<Vec<(ThreadId, AgentMetadata)>> {
        let mut children_by_parent = self.live_thread_spawn_children().await?;
        Ok(children_by_parent
            .remove(&parent_thread_id)
            .unwrap_or_default())
    }

    pub(super) async fn live_thread_spawn_children(
        &self,
    ) -> CodexResult<HashMap<ThreadId, Vec<(ThreadId, AgentMetadata)>>> {
        let state = self.upgrade()?;
        let mut children_by_parent = HashMap::<ThreadId, Vec<(ThreadId, AgentMetadata)>>::new();

        for (parent_thread_id, child_thread_id) in state.list_live_thread_spawn_edges().await {
            // Corbanu: a child the registry does not know yet still shows its path, nickname
            // and role from its session source.
            let metadata = match self.registry.agent_metadata_for_thread(child_thread_id) {
                Some(metadata) => metadata,
                None => match state.get_thread(child_thread_id).await {
                    Ok(child_thread) => AgentMetadata {
                        agent_id: Some(child_thread_id),
                        agent_path: child_thread.session_source.get_agent_path(),
                        agent_nickname: child_thread.session_source.get_nickname(),
                        agent_role: child_thread.session_source.get_agent_role(),
                        last_task_message: None,
                        last_result_message: None,
                    },
                    Err(_) => AgentMetadata {
                        agent_id: Some(child_thread_id),
                        ..Default::default()
                    },
                },
            };
            children_by_parent
                .entry(parent_thread_id)
                .or_default()
                .push((child_thread_id, metadata));
        }

        for children in children_by_parent.values_mut() {
            children.sort_by(|left, right| {
                left.1
                    .agent_path
                    .as_deref()
                    .unwrap_or_default()
                    .cmp(right.1.agent_path.as_deref().unwrap_or_default())
                    .then_with(|| left.0.to_string().cmp(&right.0.to_string()))
            });
        }

        Ok(children_by_parent)
    }

    pub(super) async fn live_thread_spawn_descendants(
        &self,
        root_thread_id: ThreadId,
    ) -> CodexResult<Vec<ThreadId>> {
        let mut children_by_parent = self.live_thread_spawn_children().await?;
        let mut descendants = Vec::new();
        let mut stack = children_by_parent
            .remove(&root_thread_id)
            .unwrap_or_default()
            .into_iter()
            .map(|(child_thread_id, _)| child_thread_id)
            .rev()
            .collect::<Vec<_>>();

        while let Some(thread_id) = stack.pop() {
            descendants.push(thread_id);
            if let Some(children) = children_by_parent.remove(&thread_id) {
                for (child_thread_id, _) in children.into_iter().rev() {
                    stack.push(child_thread_id);
                }
            }
        }

        Ok(descendants)
    }

    pub(in crate::agent) fn upgrade(&self) -> CodexResult<Arc<ThreadManagerState>> {
        self.manager.upgrade().ok_or_else(|| {
            CodexErr::UnsupportedOperation("thread manager dropped".to_string())
                .with_agent_context(AgentErrorContext::ManagerUnavailable)
        })
    }
}
