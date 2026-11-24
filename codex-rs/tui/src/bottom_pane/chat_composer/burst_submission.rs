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
            self.handle_paste(pasted);
        }
        self.handle_submission(should_queue)
    }

    pub(crate) fn take_burst_submission(&mut self, now: Instant) -> Option<InputResult> {
        if !self.draft.input_enabled || !self.draft.paste_burst.take_due_enter(now) {
            return None;
        }
        let pasted = self.draft.paste_burst.flush_before_modified_input()?;
        self.handle_paste(pasted);
        let result = self
            .handle_submission_with_time(self.queue_submissions, now)
            .0;
        self.reset_vim_mode_after_successful_dispatch(&result);
        self.sync_popups();
        Some(result)
    }

    /// Types `text` as a 1 ms-spaced burst starting at `now`, then presses Enter
    /// at the burst's last instant, leaving that Enter held for the lookahead.
    #[cfg(test)]
    pub(crate) fn arm_burst_enter_for_tests(&mut self, text: &str, now: Instant) {
        let mut at = now;
        for ch in text.chars() {
            self.handle_input_basic_with_time(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                at,
            );
            at += std::time::Duration::from_millis(1);
        }
        let (result, _) = self.handle_submission_with_time(/*should_queue*/ false, at);
        assert_eq!(
            result,
            InputResult::None,
            "the burst Enter must be held, not submitted"
        );
        assert!(
            self.draft.paste_burst.has_pending_enter(),
            "the burst Enter must be held"
        );
    }

    #[cfg(test)]
    pub(crate) fn has_pending_burst_enter_for_tests(&self) -> bool {
        self.draft.paste_burst.has_pending_enter()
    }

    pub(crate) fn cancel_pending_burst_submission(&mut self) {
        if self.draft.paste_burst.cancel_pending_enter()
            && let Some(pasted) = self.draft.paste_burst.flush_before_modified_input()
        {
            // Cancellation must see a real draft, not an apparently empty
            // composer that Ctrl+C/escape could interpret as quit/backtrack.
            self.handle_paste(pasted);
        }
    }
}
