//! Cancel retained question keys when app-level routing takes input away.

#[cfg(test)]
#[path = "question_burst_tests.rs"]
mod tests;

use super::*;

impl App {
    pub(crate) fn cancel_question_burst_if_unfocused(&mut self) {
        if self.overlay.is_some()
            || self.reconnect.offline
            || self.claude_panes.active_claude_pane_id().is_some()
            || self.chat_widget.fork_in_progress
            || self.chat_widget.is_external_writer_view()
            || self.chat_widget.has_provider_error()
            || self
                .current_displayed_thread_id()
                .is_some_and(|id| self.thread_unavailable(id))
        {
            self.chat_widget.cancel_question_burst_enter();
        }
    }
}
