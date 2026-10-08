//! What a `/cost` view covers, what lies outside it, and what to do when a
//! request has no price.
//!
//! The view explains one conversation: its root and resolved subagents. The
//! same UTC day can hold requests from other conversations, and a request can
//! record tokens that no published price covers. Neither may read as zero: the
//! first is stated beside this conversation's figures with its own total, the
//! second with the step that finds the real amount.

use codex_state::accounting::DayTotals;
use codex_state::accounting::DeletedAttempts;
use codex_state::accounting::ObservationQuote;
use codex_state::accounting::OtherConversations;

use super::EstimateGaps;
use super::by_route;
use super::has_no_price;
use super::lower_first;
use super::plain_billing;
use super::provider_name;
use super::request_count;
use super::route_line;

const RESUME_STEP: &str =
    "To see those requests, open that conversation with /resume and run /cost there.";

fn counted(n: usize, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}

/// Same-day requests of other conversations: never part of this conversation's
/// total, and never left out silently. `None` means the view did not read them.
pub(super) fn other_conversations_lines(others: Option<&OtherConversations>) -> Vec<String> {
    let Some(others) = others else {
        return vec![
            "Other conversations are not included in this view; /cost covers only the open conversation.".into(),
        ];
    };
    let deleted = deleted_line(others.deleted_attempts);
    if others.conversations == 0 {
        // Only a day with nothing deleted on it is known to be empty.
        return match deleted {
            None => vec!["No other conversation recorded requests on this day.".into()],
            Some(deleted) => vec![
                "No other saved conversation has recorded requests on this day.".into(),
                deleted,
            ],
        };
    }
    let quotes: Vec<&ObservationQuote> = others.requests.values().flatten().collect();
    let conversations = counted(others.conversations, "conversation");
    let mut lines = vec![if quotes.is_empty() {
        format!("Other conversations on this day, not included above: {conversations}.")
    } else {
        format!(
            "Other conversations on this day, not included above: {} in {conversations}.",
            request_count(&quotes)
        )
    }];
    lines.extend(
        by_route(quotes.iter().copied())
            .values()
            .map(|q| route_line(q)),
    );
    if !quotes.is_empty() {
        lines.push(match DayTotals::from_quotes(quotes.iter().copied()) {
            Ok(totals) => format!(
                "Other conversations' {}.",
                lower_first(&plain_billing(&totals, EstimateGaps::of(quotes.iter().copied())).1)
            ),
            Err(_) => "Other conversations' cost: unavailable — exact arithmetic overflow.".into(),
        });
    }
    if others.unavailable > 0 {
        lines.push(format!(
            "{} could not be read in full; their cost is unknown and not included.",
            counted(others.unavailable, "conversation")
        ));
    }
    lines.push(RESUME_STEP.into());
    // After the resume step: deleted conversations cannot be resumed.
    lines.extend(deleted);
    lines
}

/// Deleted conversations' spend on this day. Deletion removes their recorded
/// requests and cost, but not what the provider charged for them; say so
/// rather than let the day look emptier or cheaper than it was. `None` means
/// none were deleted. A deleted subagent of this conversation counts here too:
/// its replay fence names no conversation.
fn deleted_line(deleted_attempts: DeletedAttempts) -> Option<String> {
    match deleted_attempts {
        DeletedAttempts::Counted(0) => None,
        DeletedAttempts::Counted(n) => Some(format!(
            "Deleted conversations or subagents sent {} on this day. Their recorded cost was deleted with them, so it is not included here; any cost they incurred is on your provider's bill.",
            counted(n, "request attempt")
        )),
        DeletedAttempts::Uncountable => Some(
            "Days older than 90 days keep too little detail to count requests from deleted conversations; any they made are not included here.".into(),
        ),
    }
}

/// What to do about requests whose recorded tokens no published price covers:
/// the amount is on the provider's own bill, not zero.
pub(super) fn no_price_next_step<'a>(
    quotes: impl IntoIterator<Item = &'a ObservationQuote>,
) -> Option<String> {
    let mut providers: Vec<String> = Vec::new();
    for quote in quotes.into_iter().filter(|quote| has_no_price(quote)) {
        let provider = provider_name(&quote.attempt.provider);
        if !providers.contains(&provider) {
            providers.push(provider);
        }
    }
    let providers = match providers.as_slice() {
        [] => return None,
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    };
    Some(format!(
        "Next step for requests with no price: check the bill from {providers}. No published price covers them, so only their tokens are shown here, not a cost."
    ))
}

#[cfg(test)]
#[path = "scope_tests.rs"]
mod tests;
