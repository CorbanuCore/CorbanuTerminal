//! Permission requests never optimistically alter the settings used by a turn.
use super::*;
use crate::app_server_session::PermissionConfirmationResult;
use codex_app_server_protocol::ThreadSettingsUpdateParams;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PendingPermissionConfirmation {
    pub selection_id: uuid::Uuid,
    pub thread_id: ThreadId,
    pub label: String,
    pub persist_reviewer: Option<ApprovalsReviewer>,
    pub requested: ThreadSettingsUpdateParams,
    pub observed: bool,
    pub applied: bool,
}

impl PendingPermissionConfirmation {
    fn matches(&self, config: &Config) -> bool {
        self.requested.permissions.as_ref().is_some_and(|id| {
            config
                .permissions
                .active_permission_profile()
                .as_ref()
                .map(|p| &p.id)
                == Some(id)
        }) && self.requested.approval_policy.is_none_or(|policy| {
            policy == AskForApproval::from(config.permissions.approval_policy.value())
        }) && self
            .requested
            .approvals_reviewer
            .as_ref()
            .is_none_or(|reviewer| reviewer.to_core() == config.approvals_reviewer)
    }
}

impl App {
    pub(super) fn request_permission_confirmation(
        &mut self,
        app_server: &mut AppServerSession,
        params: ThreadSettingsUpdateParams,
        label: String,
        persist_reviewer: Option<ApprovalsReviewer>,
    ) {
        if self.pending_permission_confirmation.is_some() {
            self.chat_widget.add_error_message("A permission selection is still pending. Wait for its result before choosing again.".into());
            return;
        }
        let Ok(thread_id) = ThreadId::from_string(&params.thread_id) else {
            return;
        };
        let selection_id = uuid::Uuid::new_v4();
        let mut pending = PendingPermissionConfirmation {
            selection_id,
            thread_id,
            label: label.clone(),
            persist_reviewer,
            requested: params.clone(),
            observed: false,
            applied: false,
        };
        pending.observed = pending.matches(self.chat_widget.config_ref());
        self.pending_permission_confirmation = Some(pending);
        self.chat_widget.add_info_message(
            format!("Permissions requested: {label}. Waiting for confirmation."),
            Some("Wait for the result before starting a new turn. Running command authority and pending approvals are unchanged.".into()),
        );
        app_server.confirm_permissions(params, selection_id, self.app_event_tx.clone());
        // Bound the notification half too: a successful reply alone does not
        // provide an effective profile snapshot (including on older servers).
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            tx.send(AppEvent::PermissionConfirmationCompleted {
                selection_id,
                result: PermissionConfirmationResult::Uncertain(
                    "Effective settings were not observed before the deadline.".into(),
                ),
            });
        });
    }

    pub(super) fn observe_permission_confirmation(&mut self) {
        let Some(pending) = self.pending_permission_confirmation.as_mut() else {
            return;
        };
        if self.active_thread_id != Some(pending.thread_id) {
            return;
        }
        pending.observed |= pending.matches(self.chat_widget.config_ref());
        if pending.observed && pending.applied {
            self.app_event_tx
                .send(AppEvent::PermissionConfirmationCompleted {
                    selection_id: pending.selection_id,
                    result: PermissionConfirmationResult::Applied,
                });
        }
    }

    pub(super) fn finish_permission_confirmation(
        &mut self,
        selection_id: uuid::Uuid,
        result: PermissionConfirmationResult,
    ) -> Option<ApprovalsReviewer> {
        if !self
            .pending_permission_confirmation
            .as_ref()
            .is_some_and(|pending| pending.selection_id == selection_id)
        {
            return None;
        }
        let mut pending = self.pending_permission_confirmation.take()?;
        if self.active_thread_id != Some(pending.thread_id) {
            return None;
        }
        let mut persist_reviewer = None;
        if matches!(result, PermissionConfirmationResult::Applied) {
            pending.applied = true;
            if !pending.observed {
                self.pending_permission_confirmation = Some(pending);
                return None;
            }
            // Adopt the latest observation, never the requested values. This
            // also replaces any older runtime override used by /new's reload.
            let effective = self.chat_widget.config_ref();
            self.config.permissions = effective.permissions.clone();
            self.config.approvals_reviewer = effective.approvals_reviewer;
            self.runtime_approval_policy_override = Some(AskForApproval::from(
                self.config.permissions.approval_policy.value(),
            ));
            self.runtime_permission_profile_override =
                Some(RuntimePermissionProfileOverride::from_config(&self.config));
            if pending.matches(effective) {
                persist_reviewer = pending.persist_reviewer;
                self.chat_widget
                    .submit_initial_user_message_after_permission_confirmation();
            } else {
                self.chat_widget.restore_initial_user_message();
            }
        }
        if !matches!(result, PermissionConfirmationResult::Applied) {
            self.chat_widget.restore_initial_user_message();
        }
        let (message, hint) = if matches!(result, PermissionConfirmationResult::Applied)
            && !pending.matches(self.chat_widget.config_ref())
        {
            (
                format!(
                    "Permission request applied: {}. Newer settings govern new turns.",
                    pending.label
                ),
                format!(
                    "{} Held initial input was restored without submission. Check /status for the latest next-turn settings.",
                    permission_continuation_hint()
                ),
            )
        } else {
            permission_confirmation_message(&pending.label, &result)
        };
        self.chat_widget.add_info_message(message, Some(hint));
        persist_reviewer
    }
}

fn permission_continuation_hint() -> &'static str {
    "If a running turn has different permissions, a prompt sent here is held until it finishes, then runs with the latest permissions. Running work, granted approvals and pending approvals are unchanged. Shared services keep their existing refresh behavior."
}

fn permission_confirmation_message(
    label: &str,
    result: &PermissionConfirmationResult,
) -> (String, String) {
    match result {
        PermissionConfirmationResult::Applied => (
            format!("Permissions confirmed for new turns: {label}."),
            permission_continuation_hint().into(),
        ),
        PermissionConfirmationResult::Unsupported => (
            format!("Permissions unconfirmed: {label}. This server does not provide application confirmation."),
            "The request may have been accepted. No automatic retry was made; reconnect to a server that supports confirmation to verify a new selection.".into(),
        ),
        PermissionConfirmationResult::Failed(error) => (
            format!("Permission selection failed: {label}. {error}"),
            "The rejected selection was not saved as a future-turn override. Choose again after resolving the error.".into(),
        ),
        PermissionConfirmationResult::Uncertain(error) => (
            format!("Permission outcome uncertain: {label}."),
            format!("No automatic retry was made; running authority is unchanged. Reconnect and inspect session settings before choosing again. {error}"),
        ),
    }
}

#[cfg(test)]
#[path = "permission_confirmation_tests.rs"]
mod tests;
