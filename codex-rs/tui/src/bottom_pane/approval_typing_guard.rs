//! Keeps typed or pasted text from answering an approval prompt.
//!
//! Someone who believes the composer is focused types a word into the prompt.
//! When characters answered it, the first letter picked an option: typing
//! `/permissions` approved with "don't ask again" through `p`. Characters
//! therefore never answer on their own. A decision key only highlights its
//! option and Enter confirms it, so the prompt is answered by exactly one
//! decision key then Enter, by navigation then Enter, by a chord shortcut, or
//! cancelled with Esc. Any other input (a second character, an unbound
//! character, a paste, editing keys) is typed text: Enter is ignored and the
//! prompt explains how to answer until the user navigates with the arrow,
//! Page, Home or End keys. No timing is involved.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    /// No input since the prompt opened or the user last navigated.
    #[default]
    Idle,
    /// One decision key highlighted this option; Enter confirms it.
    Armed(usize),
    /// Input looked like typing; Enter must not confirm anything.
    Typed,
}

#[derive(Debug, Default)]
pub(super) struct TypingGuard {
    state: State,
}

impl TypingGuard {
    /// Whether nothing has been typed since the last navigation.
    pub(super) fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    /// Whether typed text was detected; the prompt explains how to answer.
    pub(super) fn typed_text(&self) -> bool {
        self.state == State::Typed
    }

    /// Record a plain character. `option` is the option the key is bound to.
    /// Returns the option to highlight when the key armed one.
    pub(super) fn on_text_key(&mut self, option: Option<usize>) -> Option<usize> {
        match (self.state, option) {
            (State::Idle, Some(idx)) => {
                self.state = State::Armed(idx);
                Some(idx)
            }
            (State::Idle | State::Armed(_) | State::Typed, _) => {
                self.state = State::Typed;
                None
            }
        }
    }

    /// Record input that is not a decision: a paste, editing keys, or chords
    /// that edit text in the composer.
    pub(super) fn on_typed_input(&mut self) {
        self.state = State::Typed;
    }

    /// Whether Enter may confirm the highlighted option.
    pub(super) fn allows_accept(&self) -> bool {
        !self.typed_text()
    }

    /// Forget earlier input after deliberate navigation or an answer.
    pub(super) fn reset(&mut self) {
        self.state = State::Idle;
    }

    /// The prompt now shows another request. A key armed for the previous one
    /// must not let Enter confirm the new one; typed text stays typed text.
    pub(super) fn on_request_changed(&mut self) {
        if let State::Armed(_) = self.state {
            self.state = State::Typed;
        }
    }
}

#[cfg(test)]
#[path = "approval_typing_guard_tests.rs"]
mod tests;
