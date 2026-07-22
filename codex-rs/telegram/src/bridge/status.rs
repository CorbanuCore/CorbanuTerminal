use std::collections::VecDeque;

use super::BridgeRuntime;
use crate::conversation::ConversationKey;

impl BridgeRuntime {
    pub(super) async fn runtime_status_text(&self, conversation: ConversationKey) -> String {
        let approvals = self.sessions.pending_approval_count(conversation).await;
        let active_turn = self.sessions.turn_id(conversation).await;
        let queued = self
            .pending_inputs
            .get(&conversation)
            .map_or(0, VecDeque::len);
        let last_error = self.last_errors.get(&conversation);
        let state = runtime_state(
            approvals,
            active_turn.is_some(),
            queued,
            last_error.is_some(),
        );
        let (model, provider) = self.active_model_settings(conversation).await;
        let mut text = format!(
            "State: {}\nModel: {} via {provider}\nWorkspace: {}\nConversation: {}\nQueued messages: {queued}",
            state.label(),
            model.as_deref().unwrap_or("server default"),
            self.config.cwd.display(),
            conversation.display_label()
        );
        if let Some(last_contact) = self.last_successful_contact_at.get(&conversation) {
            text.push_str(&format!(
                "\nLast successful Telegram contact: unix {last_contact}"
            ));
        }
        if let Some(last_error) = last_error {
            text.push_str(&format!("\nLast error: {last_error}"));
        }
        text.push_str("\nNext: ");
        text.push_str(state.next_action());
        text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeState {
    Idle,
    Working,
    WorkingQueued,
    AwaitingApproval,
    Recovering,
    Blocked,
}

impl RuntimeState {
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::WorkingQueued => "Working · follow-ups queued",
            Self::AwaitingApproval => "Awaiting approval",
            Self::Recovering => "Recovering queued input",
            Self::Blocked => "Blocked",
        }
    }

    fn next_action(self) -> &'static str {
        match self {
            Self::AwaitingApproval => "use the approval buttons or /cancel.",
            Self::Working | Self::WorkingQueued => "send a follow-up to steer, or /cancel.",
            Self::Recovering => "wait for automatic recovery, or /cancel.",
            Self::Blocked => "review the last error, then retry the task or use /new.",
            Self::Idle => "send a task, or /new for a fresh thread.",
        }
    }
}

fn runtime_state(
    approvals: usize,
    active_turn: bool,
    queued: usize,
    has_error: bool,
) -> RuntimeState {
    if approvals > 0 {
        RuntimeState::AwaitingApproval
    } else if active_turn && queued > 0 {
        RuntimeState::WorkingQueued
    } else if active_turn {
        RuntimeState::Working
    } else if has_error {
        RuntimeState::Blocked
    } else if queued > 0 {
        RuntimeState::Recovering
    } else {
        RuntimeState::Idle
    }
}

#[cfg(test)]
#[path = "status_tests.rs"]
mod tests;
