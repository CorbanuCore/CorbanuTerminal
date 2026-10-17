//! A recorded warm-up is billed even when there are no saved turns to resume.
//!
//! A date-limited cost packet cannot prove the absence of older saved turns;
//! explain the conditional exception without classifying the conversation.
use super::ObservationQuote;
use super::reopen_warmup;

pub(super) fn note(quotes: &[&ObservationQuote]) -> Option<String> {
    reopen_warmup(quotes).then(|| {
        "A conversation with only a reopen warm-up and no saved turns cannot be opened with /resume. The warm-up is still billed and remains included in the recorded cost.".into()
    })
}
