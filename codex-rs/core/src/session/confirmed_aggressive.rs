//! #505: a person's Aggressive confirmation in `/security` reaches a running
//! session's stored settings (see `security::confirmed_approvals`).

use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;

use super::session::Session;
use super::session::SessionSettingsUpdate;
use crate::security::confirmed_approvals::ApprovalsOutcome;
use crate::security::confirmed_approvals::kept_warning;

impl Session {
    /// From the next turn, every command that is not known read-only needs a
    /// person's approval, through the same stored-settings update a thread
    /// settings change uses, so steering, child-agent messages, MCP approvals
    /// and clients see it. A managed requirement that rules it out keeps the
    /// session's own settings, and the person is told here.
    pub(crate) async fn require_approval_for_every_command(&self) -> ApprovalsOutcome {
        let outcome = {
            let state = self.state.lock().await;
            if state.session_configuration.aggressive_approvals {
                return ApprovalsOutcome::Held;
            }
            state
                .session_configuration
                .clone()
                .require_approval_for_every_command()
        };
        let outcome = match outcome {
            ApprovalsOutcome::Held => match self
                .update_settings(SessionSettingsUpdate {
                    aggressive_approvals: true,
                    ..Default::default()
                })
                .await
            {
                // Reported only once the stored settings hold it.
                Ok(())
                    if !self
                        .state
                        .lock()
                        .await
                        .session_configuration
                        .aggressive_approvals =>
                {
                    ApprovalsOutcome::Kept("the settings update did not take it".to_string())
                }
                Ok(()) => {
                    let msg = super::handlers::thread_settings_applied_event(self).await;
                    self.send_event_raw_without_materializing_rollout(Event {
                        id: self.next_internal_sub_id(),
                        msg,
                    })
                    .await;
                    ApprovalsOutcome::Held
                }
                Err(err) => ApprovalsOutcome::Kept(err.to_string()),
            },
            outcome => outcome,
        };
        if let ApprovalsOutcome::Kept(reason) = &outcome {
            tracing::warn!(%reason, "Aggressive: approval for every command not applied");
            self.send_event_raw(Event {
                id: self.next_internal_sub_id(),
                msg: EventMsg::Warning(WarningEvent {
                    message: kept_warning(reason),
                }),
            })
            .await;
        }
        outcome
    }
}

#[cfg(test)]
#[path = "confirmed_aggressive_tests.rs"]
mod tests;
