//! Keeps typed or pasted text from answering an approval prompt.
//!
//! Approval prompts answer on single keys (`y`, `p`, `1`, ...). Someone who
//! believes the composer is focused types a word, and its first letter would
//! pick an option: typing `/permissions` used to approve with "don't ask
//! again" through `p`. The guard therefore holds a decision key until the
//! keyboard has been quiet for [`SHORTCUT_SETTLE_DELAY`]. A deliberate key
//! press arrives alone and applies after that delay; any other text key in the
//! meantime, an unbound character or a paste marks the input as typed text.
//! Typed text disables decision keys and Enter until the user navigates the
//! list, so only an explicit key press or selection can answer the prompt.

use std::time::Duration;
use std::time::Instant;

/// How long a decision key waits for a following key before it applies.
///
/// Typed words and pastes arrive faster than this; a deliberate single key
/// press is followed by a pause.
pub(crate) const SHORTCUT_SETTLE_DELAY: Duration = Duration::from_millis(250);

/// What a held decision key does once it settles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShortcutTarget {
    /// Select the option at this index.
    Option(usize),
    /// Cancel the request through a character cancel binding.
    Cancel,
}

#[derive(Debug, Default)]
pub(super) struct TypingGuard {
    pending: Option<(ShortcutTarget, Instant)>,
    typed_text: bool,
}

impl TypingGuard {
    /// Whether text keys are currently treated as typing rather than commands.
    pub(super) fn is_engaged(&self) -> bool {
        self.typed_text || self.pending.is_some()
    }

    /// Whether typed text was detected; the prompt explains how to answer.
    pub(super) fn typed_text(&self) -> bool {
        self.typed_text
    }

    /// Record a text key. `target` is the decision the key is bound to, if any.
    pub(super) fn on_text_key(&mut self, target: Option<ShortcutTarget>, now: Instant) {
        if self.typed_text {
            return;
        }
        match (self.pending.take(), target) {
            (None, Some(target)) => self.pending = Some((target, now)),
            (Some(_), _) | (None, None) => self.typed_text = true,
        }
    }

    /// Record pasted text.
    pub(super) fn on_paste(&mut self) {
        self.pending = None;
        self.typed_text = true;
    }

    /// Return true when Enter must not confirm the highlighted option because
    /// it follows typed text (including a key that had not settled yet).
    pub(super) fn blocks_accept(&mut self) -> bool {
        if self.pending.take().is_some() {
            self.typed_text = true;
        }
        self.typed_text
    }

    /// Forget typed text after deliberate navigation or a new request.
    pub(super) fn reset(&mut self) {
        self.pending = None;
        self.typed_text = false;
    }

    /// Take the held decision once the keyboard has been quiet long enough.
    pub(super) fn take_due(&mut self, now: Instant) -> Option<ShortcutTarget> {
        let (target, pressed_at) = self.pending?;
        if now.saturating_duration_since(pressed_at) < SHORTCUT_SETTLE_DELAY {
            return None;
        }
        self.pending = None;
        Some(target)
    }

    /// Time until the held decision settles, used to schedule a redraw.
    pub(super) fn next_delay(&self, now: Instant) -> Option<Duration> {
        self.pending.map(|(_, pressed_at)| {
            SHORTCUT_SETTLE_DELAY.saturating_sub(now.saturating_duration_since(pressed_at))
        })
    }
}

#[cfg(test)]
#[path = "approval_typing_guard_tests.rs"]
mod tests;
