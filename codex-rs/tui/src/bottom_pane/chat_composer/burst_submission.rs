//! Resolves the ambiguous Enter after a long, unbracketed single-line burst.
//! Poll before input and on redraw ticks. Explicit pastes and draft replacement
//! clear PasteBurst, so they cannot inherit a delayed submission.

use super::*;

impl ChatComposer {
    pub(super) fn handle_submission_key(
        &mut self,
        key: KeyEvent,
        should_queue: bool,
    ) -> (InputResult, bool) {
        // Tab and explicitly modified submit bindings are unambiguous. Preserve
        // their existing immediate long-burst submission instead of holding them.
        if (key.code != KeyCode::Enter || !key.modifiers.is_empty())
            && self
                .draft
                .paste_burst
                .single_line_burst_enter_should_submit(self.draft.textarea.text())
            && let Some(pasted) = self.draft.paste_burst.flush_before_modified_input()
        {
            self.apply_paste(pasted);
        }
        self.handle_submission(should_queue)
    }

    pub(crate) fn take_burst_submission(&mut self, now: Instant) -> Option<InputResult> {
        if !self.draft.input_enabled || !self.draft.paste_burst.take_due_enter(now) {
            return None;
        }
        let pasted = self.draft.paste_burst.flush_before_modified_input()?;
        self.apply_paste(pasted);
        let result = self
            .handle_submission_with_time(self.queue_submissions, now)
            .0;
        self.reset_vim_mode_after_successful_dispatch(&result);
        self.sync_popups();
        Some(result)
    }

    pub(crate) fn cancel_pending_burst_submission(&mut self) {
        if self.draft.paste_burst.cancel_pending_enter()
            && let Some(pasted) = self.draft.paste_burst.flush_before_modified_input()
        {
            // Cancellation must see a real draft, not an apparently empty
            // composer that Ctrl+C/escape could interpret as quit/backtrack.
            self.apply_paste(pasted);
        }
    }
}
