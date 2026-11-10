//! Reopens an agent only when its runtime is absent, then returns its effective state.
//! Rollout and descendant restoration remain in the existing local startup path.

use super::LocalAgentControl;
use crate::agent::types::LiveAgent;
use crate::codex_thread::ThreadConfigSnapshot;
use crate::config::Config;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::SessionSource;

impl LocalAgentControl {
    pub(crate) async fn resume_agent(
        &self,
        config: Config,
        thread_id: ThreadId,
        source: SessionSource,
        start_refusal: Option<String>,
    ) -> CodexResult<(LiveAgent, ThreadConfigSnapshot, Vec<(ThreadId, String)>)> {
        let manager = self.runtime.upgrade()?;
        // Corbanu (#1144): open V1 descendants that could not be resumed are returned with why.
        let mut refused_descendants = Vec::new();
        let thread = match manager.get_thread(thread_id).await {
            Ok(thread) => thread,
            Err(err) if matches!(err.details(), CodexErrorDetails::ThreadNotFound(_)) => {
                // Corbanu (#1074): a caller may refuse a start (an agent whose connection has
                // no credential of its own), decided here where a start is actually needed.
                if let Some(message) = start_refusal {
                    return Err(CodexErr::UnsupportedOperation(message));
                }
                let (_, refused) =
                    Box::pin(self.resume_agent_tree_from_rollout(config, thread_id, source))
                        .await?;
                refused_descendants = refused;
                manager.get_thread(thread_id).await?
            }
            Err(err) => return Err(err),
        };
        let agent = LiveAgent {
            thread_id,
            metadata: self.get_agent_metadata(thread_id).unwrap_or_default(),
            status: thread.agent_status().await,
        };
        let config = thread.config_snapshot().await;
        Ok((agent, config, refused_descendants))
    }
}
