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
//!
//! Some prompts need a fresh choice (a decision key or navigation) before
//! Enter confirms: a request reached by answering or replacing the previous
//! one, so a double or held Enter cannot answer a request the user has not
//! seen; a prompt that was held back while the user typed, whose next Enter
//! may be meant for their draft; and the prompt right after a character
//! command key such as open thread, whose following keys may be a word.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    /// The prompt just opened; Enter confirms the highlighted option.
    #[default]
    Idle,
    /// Enter waits for a choice (see the module docs for when); `told`
    /// records that the prompt is explaining this. `keys_arm` is false right
    /// after a character command key, whose following keys may be a word.
    Fresh { told: bool, keys_arm: bool },
    /// One decision key highlighted this option; Enter confirms it.
    Armed(usize),
    /// Input looked like typing; Enter must not confirm anything.
    Typed,
}

/// What Enter does now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Confirm {
    /// Confirm the highlighted row.
    Highlighted,
    /// Confirm this option, armed by its key.
    Option(usize),
    /// Ignore Enter; the prompt explains how to answer.
    Blocked,
}

/// Explanation the prompt shows while Enter is ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Notice {
    TypedText,
    ChooseFirst,
}

#[derive(Debug, Default)]
pub(super) struct TypingGuard {
    state: State,
}

impl TypingGuard {
    /// Whether single-key commands that are not decisions (open thread,
    /// cancel) may act: no text has been typed into this prompt.
    pub(super) fn accepts_commands(&self) -> bool {
        matches!(self.state, State::Idle | State::Fresh { .. })
    }

    pub(super) fn notice(&self) -> Option<Notice> {
        match self.state {
            State::Typed => Some(Notice::TypedText),
            State::Fresh { told: true, .. } => Some(Notice::ChooseFirst),
            State::Idle | State::Fresh { told: false, .. } | State::Armed(_) => None,
        }
    }

    /// Record a plain character. `option` is the option the key is bound to.
    /// Returns the option to highlight when the key armed one.
    pub(super) fn on_text_key(&mut self, option: Option<usize>) -> Option<usize> {
        match (self.state, option) {
            (State::Idle | State::Fresh { keys_arm: true, .. }, Some(idx)) => {
                self.state = State::Armed(idx);
                Some(idx)
            }
            (State::Idle | State::Fresh { .. } | State::Armed(_) | State::Typed, _) => {
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

    /// Enter was pressed. Returns what it confirms; a confirmation resets the
    /// guard for the next request.
    pub(super) fn on_confirm(&mut self) -> Confirm {
        match self.state {
            State::Idle => Confirm::Highlighted,
            State::Armed(idx) => {
                self.state = State::Idle;
                Confirm::Option(idx)
            }
            State::Fresh { keys_arm, .. } => {
                self.state = State::Fresh {
                    told: true,
                    keys_arm,
                };
                Confirm::Blocked
            }
            State::Typed => Confirm::Blocked,
        }
    }

    /// Forget earlier input after deliberate navigation.
    pub(super) fn reset(&mut self) {
        self.state = State::Idle;
    }

    /// The prompt now shows another request. It needs a fresh choice before
    /// Enter confirms; typed text stays typed text.
    pub(super) fn on_request_changed(&mut self) {
        if self.state != State::Typed {
            self.state = State::Fresh {
                told: false,
                keys_arm: true,
            };
        }
    }

    /// A character command key (such as open thread) acted. Enter waits for
    /// a choice, and a following character counts as typed text.
    pub(super) fn on_command_key(&mut self) {
        self.state = State::Fresh {
            told: false,
            keys_arm: false,
        };
    }
}

#[cfg(test)]
#[path = "approval_typing_guard_tests.rs"]
mod tests;
