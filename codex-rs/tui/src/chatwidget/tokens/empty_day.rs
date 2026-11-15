//! FUP-02 (#352): the words for a day with nothing recorded. Only a day that
//! ends before the conversation started says "no requests recorded"; any
//! other empty day says what is missing, never that nothing happened.

use codex_state::accounting::EmptyDay;

use super::utc_date;

/// Why an empty day inside the ledger's life is not proof of no requests.
pub(super) const NOT_PROOF: &str = "That does not show there were no requests: one that failed to record, or one sent from a build without developer accounting, leaves no record.";

/// The first screen's opening lines. `None` is a view that did not classify
/// its emptiness, which proves nothing either.
pub(super) fn overview(heading: &str, empty: Option<EmptyDay>) -> Vec<String> {
    match empty {
        Some(EmptyDay::BeforeConversation { started_day }) => vec![format!(
            "{heading} no requests recorded — this conversation started on {} (UTC), after this day.",
            utc_date(started_day)
        )],
        Some(EmptyDay::BeforeLedger { ledger_started_day }) => vec![format!(
            "{heading} not recorded — this home's accounting ledger was created on {} (UTC), after this day.",
            utc_date(ledger_started_day)
        )],
        Some(EmptyDay::NotRecorded) | None => {
            vec![
                format!("{heading} nothing recorded."),
                NOT_PROOF.to_string(),
            ]
        }
    }
}

/// The details' first line: what the emptiness rests on.
pub(super) fn detail(empty: Option<EmptyDay>) -> String {
    match empty {
        Some(EmptyDay::BeforeConversation { started_day }) => format!(
            "No requests recorded: this UTC day ends before the conversation started on {} (UTC), the earliest of its and its subagents' id times, stored creation times and recorded requests. No conversation of unknown ancestry has anything on this day.",
            utc_date(started_day)
        ),
        Some(EmptyDay::BeforeLedger { ledger_started_day }) => format!(
            "Not recorded: this home's accounting ledger was created on {} (UTC), after this day; collection coverage unknown.",
            utc_date(ledger_started_day)
        ),
        Some(EmptyDay::NotRecorded) | None => {
            "No recorded attempts in this conversation or its subagents; collection coverage unknown.".to_string()
        }
    }
}

#[cfg(test)]
#[path = "empty_day_tests.rs"]
mod tests;
