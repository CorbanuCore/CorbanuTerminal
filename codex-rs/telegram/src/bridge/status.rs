use std::collections::VecDeque;

use codex_app_server_protocol::ThreadTokenUsage;
use codex_app_server_protocol::TokenUsageBreakdown;

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
        if let Some(usage) = self.last_token_usage.get(&conversation) {
            text.push('\n');
            text.push_str(&token_usage_text(usage));
        }
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

/// What `/status` knows about a thread's token usage.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct StatusTokenUsage {
    /// Tokens in the context window: the last report, or core's estimate.
    context_tokens: i64,
    model_context_window: Option<i64>,
    /// The thread's provider-reported total; `None` until a provider reports.
    thread_total: Option<TokenUsageBreakdown>,
}

impl StatusTokenUsage {
    /// Folds a thread usage update into what `/status` shows.
    ///
    /// Core also sends updates that carry no provider usage:
    /// `recompute_token_usage` after compaction or rollback (zero counters, the
    /// history estimate in `last.total_tokens`) and `fill_to_context_window` when
    /// the window is exceeded (zero counters, the window size in
    /// `total.total_tokens`). Both still say how full the context is, so they
    /// update `Context`, but they are not a cost: the thread total stays at the
    /// last real report, or unknown (#1570).
    pub(super) fn apply(previous: Option<Self>, update: ThreadTokenUsage) -> Self {
        if reports_usage(&update) {
            return Self {
                context_tokens: update.last.total_tokens,
                model_context_window: update.model_context_window,
                thread_total: Some(update.total),
            };
        }
        Self {
            context_tokens: update.last.total_tokens.max(update.total.total_tokens),
            model_context_window: update.model_context_window,
            thread_total: previous.and_then(|usage| usage.thread_total),
        }
    }
}

/// Same rule as exec's JSON output: an update whose counters are all zero but
/// which carries a token total is an estimate, not a provider report. A
/// provider's measured zero has no total either and is kept as reported.
fn reports_usage(usage: &ThreadTokenUsage) -> bool {
    let total = &usage.total;
    let counters_are_zero = [
        total.input_tokens,
        total.cached_input_tokens,
        total.cache_write_input_tokens,
        total.output_tokens,
        total.reasoning_output_tokens,
    ]
    .iter()
    .all(|count| *count == 0);
    !counters_are_zero || (total.total_tokens == 0 && usage.last.total_tokens == 0)
}

/// Renders the two numbers a Telegram operator cannot otherwise see.
///
/// `Context` is the live window occupancy, which auto-compaction keeps bounded.
/// `Thread total` is the number that is not bounded by anything: a Telegram chat
/// resumes the same thread until someone runs `/new`, so this accumulates for
/// the life of the thread and is what a subscription is actually spent on.
fn token_usage_text(usage: &StatusTokenUsage) -> String {
    let context = usage.context_tokens.max(0);
    let mut text = match usage.model_context_window {
        Some(window) if window > 0 => format!(
            "Context: {} of {} tokens ({}%)",
            thousands(context),
            thousands(window),
            (context.saturating_mul(100) / window).min(100)
        ),
        _ => format!("Context: {} tokens", thousands(context)),
    };
    match &usage.thread_total {
        Some(total) => text.push_str(&format!(
            "\nThread total: {} tokens ({} in, {} out)",
            thousands(total.total_tokens.max(0)),
            thousands(total.input_tokens.max(0)),
            thousands(total.output_tokens.max(0)),
        )),
        None => text.push_str("\nThread total: not reported by the provider"),
    }
    text.push_str("\nReset with /new when a thread no longer needs its history.");
    text
}

/// Group digits so a seven-figure total is legible at a glance in a chat client.
fn thousands(value: i64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped
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
