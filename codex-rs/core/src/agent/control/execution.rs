//! Tracks local running capacity and releases reservations through the shared guard.
//! The local permit owns the running count; root and MAv1 turns remain unrestricted.

use super::LocalAgentControl;
use crate::agent::types::AgentExecutionGuard;
use codex_protocol::error::AgentErrorContext;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::MultiAgentVersion;
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

struct LocalExecutionPermit {
    limiter: Arc<AgentExecutionLimiter>,
}

impl Drop for LocalExecutionPermit {
    fn drop(&mut self) {
        self.limiter.active.fetch_sub(1, Ordering::AcqRel);
        self.limiter.capacity_changed.notify_waiters();
    }
}

<<<<<<< HEAD
impl AgentControl {
    pub(crate) async fn ensure_execution_capacity_for_op(
        &self,
        thread_id: ThreadId,
        op: &Op,
    ) -> CodexResult<()> {
        self.ensure_execution_capacity_for_turn_start(thread_id, op_starts_worker_turn(op))
            .await
    }

    pub(super) async fn ensure_execution_capacity_for_turn_start(
        &self,
        thread_id: ThreadId,
        starts_turn: bool,
    ) -> CodexResult<()> {
        if !starts_turn {
            return Ok(());
        }
        let state = self.upgrade()?;
        let thread = state.get_thread(thread_id).await?;
        if thread.session.active_turn.lock().await.is_some() {
            return Ok(());
        }
        let config = thread.session.get_config().await;
        let multi_agent_version = thread
            .multi_agent_version()
            .unwrap_or_else(|| config.multi_agent_version_from_features());
        self.ensure_execution_capacity(multi_agent_version, &thread.session_source)
    }

||||||| 413492cd6c
impl AgentControl {
    pub(crate) async fn ensure_execution_capacity_for_op(
        &self,
        thread_id: ThreadId,
        op: &Op,
    ) -> CodexResult<()> {
        self.ensure_execution_capacity_for_turn_start(thread_id, op_starts_turn(op))
            .await
    }

    pub(super) async fn ensure_execution_capacity_for_turn_start(
        &self,
        thread_id: ThreadId,
        starts_turn: bool,
    ) -> CodexResult<()> {
        if !starts_turn {
            return Ok(());
        }
        let state = self.upgrade()?;
        let thread = state.get_thread(thread_id).await?;
        if thread.session.active_turn.lock().await.is_some() {
            return Ok(());
        }
        let config = thread.session.get_config().await;
        let multi_agent_version = thread
            .multi_agent_version()
            .unwrap_or_else(|| config.multi_agent_version_from_features());
        self.ensure_execution_capacity(multi_agent_version, &thread.session_source)
    }

=======
impl LocalAgentControl {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    pub(crate) fn ensure_execution_capacity(
        &self,
        multi_agent_version: MultiAgentVersion,
        session_source: &SessionSource,
    ) -> CodexResult<()> {
        if !is_execution_limited(multi_agent_version, session_source) {
            return Ok(());
        }
        let max_threads = self.runtime.agent_execution_limiter.max_threads();
        if self.runtime.agent_execution_limiter.has_capacity() {
            Ok(())
        } else {
            Err(
                CodexErr::new(CodexErrorDetails::AgentLimitReached { max_threads })
                    .with_agent_context(AgentErrorContext::ExecutionCapacity),
            )
        }
    }

    pub(crate) fn execution_guard(
        &self,
        multi_agent_version: MultiAgentVersion,
        session_source: &SessionSource,
    ) -> Option<AgentExecutionGuard> {
        is_execution_limited(multi_agent_version, session_source)
            .then(|| Arc::clone(&self.runtime.agent_execution_limiter).guard())
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
            .ok_or_else(|| {
                CodexErr::new(CodexErrorDetails::AgentLimitReached {
                    max_threads: self.agent_execution_limiter.max_threads(),
                })
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
        AgentExecutionGuard::new(LocalExecutionPermit { limiter: self })
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

<<<<<<< HEAD
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

||||||| 413492cd6c
fn op_starts_turn(op: &Op) -> bool {
    matches!(op, Op::UserInput { .. })
        || matches!(op, Op::InterAgentCommunication { communication } if communication.trigger_turn)
}

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
