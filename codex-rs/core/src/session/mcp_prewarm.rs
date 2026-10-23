//! Best-effort MCP prewarming.
//!
//! A bounded channel coalesces refresh requests. The worker only prepares the
//! newest thread state; exact model steps remain the correctness path.

use super::*;

impl Session {
    pub(crate) fn request_mcp_runtime_refresh(&self) {
        self.mark_mcp_runtime_dirty();
        self.schedule_mcp_prewarm();
    }

    pub(super) fn start_mcp_prewarm_worker(
        self: &Arc<Self>,
        requests: async_channel::Receiver<()>,
        mut auth_changes: tokio::sync::watch::Receiver<u64>,
    ) {
        let session = Arc::downgrade(self);
        let shutdown = self.mcp_prewarm_shutdown.clone();
        let mut account_switches = self.services.active_auth.subscribe_switches();
        let worker = self.services.runtime_handle.spawn(async move {
            loop {
                // PF-84: on a mid-session switch to another account's sign-in
                // (checked first: the old manager may be gone by now, which
                // also ends its auth-change channel), watch the new manager.
                let mut resubscribe = false;
                let auth_changed = tokio::select! {
                    biased;
                    _ = shutdown.cancelled() => break,
                    switch = account_switches.changed() => {
                        if switch.is_err() {
                            break;
                        }
                        resubscribe = true;
                        true
                    },
                    request = requests.recv() => {
                        if request.is_err() {
                            break;
                        }
                        false
                    },
                    auth_change = auth_changes.changed() => {
                        // A dropped manager: the session has moved off it.
                        resubscribe = auth_change.is_err();
                        true
                    },
                };
                let Some(session) = session.upgrade() else {
                    break;
                };
                if resubscribe {
                    auth_changes = session.services.auth_manager().auth_change_receiver();
                }
                if auth_changed {
                    session.mark_mcp_runtime_dirty();
                }
                tokio::select! {
                    biased;
                    _ = shutdown.cancelled() => break,
                    _ = session.refresh_mcp_if_dirty() => {},
                }
            }
        });
        *self
            .mcp_prewarm_task
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(worker);
    }

    pub(super) fn schedule_mcp_prewarm(&self) {
        let _ = self.mcp_prewarm_tx.try_send(());
    }

    pub(super) async fn stop_mcp_prewarm_worker(&self) {
        self.mcp_prewarm_shutdown.cancel();
        let worker = self
            .mcp_prewarm_task
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(worker) = worker
            && let Err(error) = worker.await
        {
            warn!(%error, "MCP prewarm worker stopped unexpectedly");
        }
    }
}
