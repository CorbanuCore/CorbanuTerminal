//! Resolves controller targets using the caller's registered identity.
//! Legacy callers can still supply their captured session source for path resolution.

use super::LocalAgentControl;
use super::LocalAgentRuntime;
use crate::agent::api::AgentTarget;
use codex_protocol::AgentPath;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::SessionSource;

impl LocalAgentControl {
    pub(crate) fn resolve_target(
        &self,
        caller: ThreadId,
        target: &AgentTarget,
    ) -> CodexResult<ThreadId> {
        match target {
            AgentTarget::Id(thread_id) => Ok(*thread_id),
            AgentTarget::Reference(reference) => {
                if let Some(thread_id) = self.runtime.resolve_corbanu_reference(reference) {
                    return Ok(thread_id);
                }
                let caller = self.runtime.ensure_agent_known(caller)?;
                self.runtime.resolve_path_reference(
                    &caller.agent_path.unwrap_or_else(AgentPath::root),
                    reference,
                )
            }
        }
    }
}

impl LocalAgentRuntime {
    pub(crate) async fn resolve_agent_reference(
        &self,
        _current_thread_id: ThreadId,
        current_session_source: &SessionSource,
        agent_reference: &str,
    ) -> CodexResult<ThreadId> {
        let agent_reference = agent_reference.trim();
        if let Some(thread_id) = self.resolve_corbanu_reference(agent_reference) {
            return Ok(thread_id);
        }
        let current_agent_path = current_session_source
            .get_agent_path()
            .unwrap_or_else(AgentPath::root);
        self.resolve_path_reference(&current_agent_path, agent_reference)
    }

    /// Corbanu: besides a path, an agent can be named by its thread id, its nickname, or the
    /// `/spawn` picker label `nickname [role]`. Only agents this tree's registry knows resolve;
    /// anything else falls through to path resolution.
    fn resolve_corbanu_reference(&self, agent_reference: &str) -> Option<ThreadId> {
        let agent_reference = agent_reference.trim();
        if let Ok(thread_id) = ThreadId::from_string(agent_reference)
            && self.registry.agent_metadata_for_thread(thread_id).is_some()
        {
            return Some(thread_id);
        }
        if let Some(thread_id) = self.registry.agent_id_for_nickname(agent_reference) {
            return Some(thread_id);
        }
        super::nickname_from_picker_label(agent_reference)
            .and_then(|agent_nickname| self.registry.agent_id_for_nickname(agent_nickname))
    }

    pub(super) fn resolve_path_reference(
        &self,
        current_agent_path: &AgentPath,
        agent_reference: &str,
    ) -> CodexResult<ThreadId> {
        let agent_path = current_agent_path
            .resolve(agent_reference)
            .map_err(CodexErr::UnsupportedOperation)?;
        if let Some(thread_id) = self.registry.agent_id_for_path(&agent_path) {
            return Ok(thread_id);
        }
        Err(CodexErr::UnsupportedOperation(format!(
            "live agent path `{}` not found",
            agent_path.as_str()
        )))
    }
}
