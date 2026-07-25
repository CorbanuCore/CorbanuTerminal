use super::AgentControl;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::sync::Notify;

#[derive(Default)]
pub(super) struct AgentExecutionLimiter {
    active: AtomicUsize,
    max_threads: OnceLock<usize>,
    capacity_changed: Notify,
}

pub(crate) struct AgentExecutionGuard {
    limiter: Arc<AgentExecutionLimiter>,
}

impl Drop for AgentExecutionGuard {
    fn drop(&mut self) {
        self.limiter.active.fetch_sub(1, Ordering::AcqRel);
        self.limiter.capacity_changed.notify_waiters();
    }
}

impl AgentControl {
    pub(crate) async fn ensure_execution_capacity_for_op(
        &self,
        thread_id: ThreadId,
        op: &Op,
    ) -> CodexResult<()> {
        if !op_starts_worker_turn(op) {
            return Ok(());
        }
        let state = self.upgrade()?;
        let thread = state.get_thread(thread_id).await?;
        if thread.codex.session.active_turn.lock().await.is_some() {
            return Ok(());
        }
        let config = thread.codex.session.get_config().await;
        let multi_agent_version = thread
            .multi_agent_version()
            .unwrap_or_else(|| config.multi_agent_version_from_features());
        self.ensure_execution_capacity(multi_agent_version, &thread.session_source)
    }

    pub(crate) fn ensure_execution_capacity(
        &self,
        multi_agent_version: MultiAgentVersion,
        session_source: &SessionSource,
    ) -> CodexResult<()> {
        if !is_execution_limited(multi_agent_version, session_source) {
            return Ok(());
        }
        let max_threads = self.agent_execution_limiter.max_threads();
        if self.agent_execution_limiter.has_capacity() {
            Ok(())
        } else {
            Err(CodexErr::AgentLimitReached { max_threads })
        }
    }

    pub(crate) fn execution_guard(
        &self,
        multi_agent_version: MultiAgentVersion,
        session_source: &SessionSource,
    ) -> Option<AgentExecutionGuard> {
        is_execution_limited(multi_agent_version, session_source)
            .then(|| Arc::clone(&self.agent_execution_limiter).guard())
    }

    pub(crate) fn try_execution_guard(
        &self,
        multi_agent_version: MultiAgentVersion,
        session_source: &SessionSource,
    ) -> CodexResult<Option<AgentExecutionGuard>> {
        if !is_execution_limited(multi_agent_version, session_source) {
            return Ok(None);
        }
        Arc::clone(&self.agent_execution_limiter)
            .try_guard()
            .map(Some)
            .ok_or_else(|| CodexErr::AgentLimitReached {
                max_threads: self.agent_execution_limiter.max_threads(),
            })
    }

    pub(crate) async fn wait_for_execution_capacity(&self) {
        self.agent_execution_limiter.wait_for_capacity().await;
    }
}

impl AgentExecutionLimiter {
    pub(super) fn initialize(&self, max_threads: usize) {
        self.max_threads.get_or_init(|| max_threads);
    }

    fn max_threads(&self) -> usize {
        self.max_threads.get().copied().unwrap_or(usize::MAX)
    }

    fn has_capacity(&self) -> bool {
        self.active.load(Ordering::Acquire) < self.max_threads()
    }

    fn guard(self: Arc<Self>) -> AgentExecutionGuard {
        self.active.fetch_add(1, Ordering::AcqRel);
        AgentExecutionGuard { limiter: self }
    }

    fn try_guard(self: Arc<Self>) -> Option<AgentExecutionGuard> {
        let max_threads = self.max_threads();
        let mut active = self.active.load(Ordering::Acquire);
        loop {
            if active >= max_threads {
                return None;
            }
            match self.active.compare_exchange_weak(
                active,
                active + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Some(AgentExecutionGuard { limiter: self }),
                Err(next) => active = next,
            }
        }
    }

    async fn wait_for_capacity(&self) {
        loop {
            let notified = self.capacity_changed.notified();
            if self.has_capacity() {
                return;
            }
            notified.await;
        }
    }
}

/// Returns whether an operation starts autonomous worker execution.
///
/// Human input is deliberately excluded. A user must remain able to address a
/// persistent crew member while autonomous descendants occupy every worker
/// slot. Once admitted, the human-started task still acquires an execution
/// guard, so it remains visible to capacity accounting. Mailbox wakeups and
/// agent-authored work remain bounded here.
fn op_starts_worker_turn(op: &Op) -> bool {
    matches!(op, Op::InterAgentCommunication { communication } if communication.trigger_turn)
        || matches!(op, Op::WakePendingWork)
}

fn is_execution_limited(
    multi_agent_version: MultiAgentVersion,
    session_source: &SessionSource,
) -> bool {
    multi_agent_version == MultiAgentVersion::V2
        && matches!(session_source, SessionSource::SubAgent(_))
}

#[cfg(test)]
#[path = "execution_tests.rs"]
mod tests;
