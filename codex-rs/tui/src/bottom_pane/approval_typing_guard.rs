//! Keeps typed or pasted text from answering an approval prompt.
//!
//! Someone who believes the composer is focused types a word into the prompt.
//! When characters answered it, the first letter picked an option: typing
//! `/permissions` approved with "don't ask again" through `p`. Characters
//! therefore never answer on their own. A decision key only highlights its
//! option and Enter confirms it, so the prompt is answered by exactly one
//! decision key then Enter, by navigation then Enter, by a chord shortcut for
//! an option that answers only this request, or cancelled with Esc. Any other input (a second character, an unbound
//! character, a paste, editing keys) is typed text: Enter is ignored and the
//! prompt explains how to answer until the user navigates with the arrow,
//! Page, Home or End keys. No timing is involved.
//!
//! The prompt right after a character command key such as open thread needs
//! navigation before Enter confirms, since its following keys may be a word.
//!
//! Options that last beyond this request ("don't ask again", session scope)
//! are never chosen by a key: their key only explains that the arrow keys
//! choose them, and Enter stays blocked until the user navigates.
//!
//! #787: the bottom pane also drops every key but Ctrl+C that reaches a
//! request before it is drawn or in the second after (see
//! `REQUEST_INPUT_GUARD`), since those keys were meant for whatever was on
//! screen before. The prompt then says so and needs navigation before Enter
//! ([`TypingGuard::on_early_input`]). #817: that guard is the only one for a
//! request coming on screen (a new request, the next of a queue, one revealed
//! or held back while the user typed): once its second has passed with no key
//! dropped, a deliberate Enter confirms the highlighted option.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    /// The prompt just opened; Enter confirms the highlighted option.
    #[default]
    Idle,
    /// Enter waits for navigation, and a key is typed text until then: right
    /// after a character command key, whose following keys may be a word, or
    /// after keys were dropped as the request appeared (#787). `told`
    /// records that the prompt is explaining this.
    Fresh { told: bool },
    /// One decision key highlighted this option; Enter confirms it.
    Armed(usize),
    /// The key of an option that lasts beyond this request was pressed. It
    /// chooses nothing, and Enter waits until the user navigates.
    PersistentKey,
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
    /// After a character command key: navigate, then Enter (#523).
    NavigateFirst,
    PersistentNeedsArrows,
    /// #787: keys that arrived as the request appeared were dropped; Enter
    /// waits until the user navigates.
    EarlyKeys,
}

#[derive(Debug, Default)]
pub(super) struct TypingGuard {
    state: State,
    /// #787: keys were dropped as this request appeared; the notice says so
    /// until the user navigates or the request changes.
    early: bool,
}

impl TypingGuard {
    /// Whether single-key commands that are not decisions (open thread,
    /// cancel) may act: no text has been typed into this prompt.
    pub(super) fn accepts_commands(&self) -> bool {
        matches!(self.state, State::Idle | State::Fresh { .. })
    }

    /// #523: whether only navigation may act now: the prompt opened while
    /// the user typed or over another view (or was revealed), and nothing
    /// but typed text came since. A form's text fields take no keys then.
    pub(super) fn awaits_navigation(&self) -> bool {
        matches!(self.state, State::Fresh { .. } | State::Typed)
    }

    pub(super) fn notice(&self) -> Option<Notice> {
        if self.early && matches!(self.state, State::Fresh { .. } | State::Typed) {
            return Some(Notice::EarlyKeys);
        }
        match self.state {
            State::Typed => Some(Notice::TypedText),
            State::PersistentKey => Some(Notice::PersistentNeedsArrows),
            State::Fresh { told: true } => Some(Notice::NavigateFirst),
            State::Idle | State::Fresh { told: false, .. } | State::Armed(_) => None,
        }
    }

    /// Record a plain character. `option` is the option the key is bound to.
    /// Returns the option to highlight when the key armed one.
    pub(super) fn on_text_key(&mut self, option: Option<usize>) -> Option<usize> {
        match (self.state, option) {
            (State::Idle, Some(idx)) => {
                self.state = State::Armed(idx);
                Some(idx)
            }
            (
                State::Idle
                | State::Fresh { .. }
                | State::Armed(_)
                | State::PersistentKey
                | State::Typed,
                _,
            ) => {
                self.state = State::Typed;
                None
            }
        }
    }

    /// Record the key of an option that lasts beyond this request. It never
    /// chooses that option; as a first key it explains how to choose it, and
    /// otherwise it is typed text like any other character.
    pub(super) fn on_persistent_key(&mut self) {
        self.state = match self.state {
            State::Idle => State::PersistentKey,
            State::Fresh { .. } | State::Armed(_) | State::PersistentKey | State::Typed => {
                State::Typed
            }
        };
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
            State::Fresh { .. } => {
                self.state = State::Fresh { told: true };
                Confirm::Blocked
            }
            State::PersistentKey | State::Typed => Confirm::Blocked,
        }
    }

    /// Forget earlier input after deliberate navigation.
    pub(super) fn reset(&mut self) {
        self.state = State::Idle;
        self.early = false;
    }

    /// #787: input reached this request in the moment after it appeared and
    /// was dropped, so it was meant for something else. Enter now waits for
    /// navigation (a key is typed text until then), and the prompt says why.
    pub(super) fn on_early_input(&mut self) {
        self.early = true;
        if self.state != State::Typed {
            self.state = State::Fresh { told: true };
        }
    }

    /// #817: the prompt shows its request anew: revealed, the next screen or
    /// question of it, or back from a screen of its own. Keys that reach it
    /// before it is drawn or in the second after are dropped by the bottom
    /// pane (#787), which is what keeps a double or held Enter from answering
    /// it; nothing else waits, so Enter confirms after that. An armed or
    /// persistent key is forgotten; text typed into this request stays typed
    /// text. A different request uses [`Self::on_new_request`].
    pub(super) fn on_request_changed(&mut self) {
        self.early = false;
        if self.state != State::Typed {
            self.state = State::Idle;
        }
    }

    /// #817 (review r1): a different request now shows in this prompt (the
    /// next of its queue). Nothing typed into the previous one carries over:
    /// keys still typed as it appears are dropped by the bottom pane, which
    /// then calls [`Self::on_early_input`].
    pub(super) fn on_new_request(&mut self) {
        self.early = false;
        self.state = State::Idle;
    }

    /// A character command key (such as open thread) acted. Enter waits for
    /// a choice, and a following character counts as typed text.
    pub(super) fn on_command_key(&mut self) {
        // Typed text stays typed text (#523: a prompt revealed mid-word).
        if self.state != State::Typed {
            self.state = State::Fresh { told: false };
        }
    }

    /// #544: whether nothing waits: no fresh choice, typed text or armed
    /// key. Only then may a key open a text field (notes) of a request.
    pub(super) fn is_idle(&self) -> bool {
        self.state == State::Idle
    }

    /// #523: whether a number key armed an option for Enter.
    pub(super) fn is_armed(&self) -> bool {
        matches!(self.state, State::Armed(_))
    }

    /// #523: whether Enter waits for a choice: keys were dropped as the
    /// prompt appeared, a character command key acted, or text was typed
    /// into it.
    pub(super) fn awaits_choice(&self) -> bool {
        matches!(self.state, State::Fresh { .. } | State::Typed)
    }
}

#[cfg(test)]
#[path = "approval_typing_guard_tests.rs"]
mod tests;
