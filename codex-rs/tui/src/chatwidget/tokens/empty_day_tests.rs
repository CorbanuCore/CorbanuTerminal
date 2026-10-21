//! FUP-02 (#352): only a day that ends before the conversation started says
//! "no requests recorded"; every other empty day says what is missing.

use super::super::InspectorPage;
use super::super::inspection_pages;
use crate::app_event::AppEvent;
use crate::chatwidget::tests::helpers::render_bottom_popup_with_height;
use crate::chatwidget::tests::make_chatwidget_manual;
use chrono::NaiveDate;
use codex_protocol::ThreadId;
use codex_state::accounting::DayTotals;
use codex_state::accounting::EmptyDay;
use codex_state::accounting::Inspection;
use codex_state::accounting::InspectionDay;
use codex_state::accounting::OtherConversations;
use codex_state::accounting::RetentionCoverage;
use pretty_assertions::assert_eq;
use uuid::Uuid;

const DAY_MS: i64 = 86_400_000;
/// #352 run Z: 2026-01-01, read on 2026-10-20 at noon UTC.
const DAY: i64 = 20_454;
const STARTED: i64 = 20_746;
const READ_AT: i64 = STARTED * DAY_MS + DAY_MS / 2;
const HEADING: &str = "This conversation on 2026-01-01 (UTC):";

fn empty_day(empty_day: Option<EmptyDay>) -> InspectionDay {
    InspectionDay::Ready(Inspection {
        owner: ThreadId::from_string(&Uuid::from_u128(1).to_string()).unwrap(),
        utc_day: DAY,
        read_at_ms: READ_AT,
        coverage: RetentionCoverage {
            completed_as_of_ms: READ_AT,
            detail_expired_through_ms: Some(READ_AT - 90 * DAY_MS),
            // 2025-10-21, as on the run-Z screen.
            aggregate_day_floor: 20_382,
            oldest_recorded_day: Some(STARTED),
        },
        totals: DayTotals::default(),
        own_totals: DayTotals::default(),
        descendant_totals: DayTotals::default(),
        unknown_parent_totals: DayTotals::default(),
        unknown_parent_unavailable_threads: 0,
        unknown_parent_requests: Default::default(),
        requests: Default::default(),
        other_conversations: Some(OtherConversations::default()),
        empty_day,
    })
}

fn first_screen(page: &InspectorPage) -> Vec<String> {
    let details = page.text.iter().position(|s| s == "—— Details ——").unwrap();
    page.text[..details].to_vec()
}

fn details(page: &InspectorPage) -> String {
    let details = page.text.iter().position(|s| s == "—— Details ——").unwrap();
    page.text[details + 1].clone()
}

/// Every page of the view, flattened.
fn all_text(pages: &[InspectorPage]) -> String {
    pages
        .iter()
        .flat_map(|page| page.text.iter().cloned())
        .collect::<Vec<_>>()
        .join("\n")
}

const NOT_PROOF: &str = "That does not show there were no requests: one that failed to record, or one sent from a build without developer accounting, leaves no record.";
const OTHERS: &str = "No other conversation recorded requests on this day.";

#[test]
fn a_day_before_the_conversation_started_says_no_requests_recorded() {
    let pages = inspection_pages(Ok(empty_day(Some(EmptyDay::BeforeConversation {
        started_day: STARTED,
    }))));
    assert_eq!(
        first_screen(&pages[0])[..2],
        [
            format!(
                "{HEADING} no requests recorded — this conversation started on 2026-10-20 (UTC), after this day."
            ),
            OTHERS.to_string(),
        ]
    );
    assert_eq!(
        details(&pages[0]),
        "No requests recorded: this UTC day ends before the conversation started on 2026-10-20 (UTC), the earliest of its and its subagents' id times, stored creation times and recorded requests. No conversation of unknown ancestry has anything on this day."
    );
    // It is a day view, not "detail unavailable".
    assert!(!all_text(&pages).contains("Request detail unavailable"));
}

#[test]
fn an_unproven_empty_day_never_claims_no_requests() {
    for (empty, opening) in [
        (
            Some(EmptyDay::NotRecorded),
            vec![
                format!("{HEADING} nothing recorded."),
                NOT_PROOF.to_string(),
            ],
        ),
        // A view that did not classify its emptiness proves nothing either.
        (
            None,
            vec![
                format!("{HEADING} nothing recorded."),
                NOT_PROOF.to_string(),
            ],
        ),
        (
            Some(EmptyDay::BeforeLedger {
                ledger_started_day: 20_697,
            }),
            vec![format!(
                "{HEADING} not recorded — this home's accounting ledger was created on 2026-09-01 (UTC), after this day."
            )],
        ),
    ] {
        let pages = inspection_pages(Ok(empty_day(empty)));
        assert_eq!(
            first_screen(&pages[0])[..opening.len()],
            opening[..],
            "{empty:?}"
        );
        let text = all_text(&pages).to_lowercase();
        for claim in [
            "no requests recorded",
            "no recorded requests",
            "no requests.",
        ] {
            assert!(!text.contains(claim), "{empty:?} claims {claim:?}:\n{text}");
        }
        // Nothing is stated as a zero amount or a zero token count; only
        // record counts may be zero.
        for line in text.lines() {
            assert!(
                !line.contains("$0") && !line.contains("usd: 0"),
                "{empty:?}: {line}"
            );
            assert!(
                !(line.ends_with(": 0") || line.contains(": 0 "))
                    || line.starts_with("recorded attempts")
                    || line.starts_with("unknown parent population: 0 attempts"),
                "{empty:?}: {line}"
            );
        }
    }
    assert_eq!(
        details(
            &inspection_pages(Ok(empty_day(Some(EmptyDay::BeforeLedger {
                ledger_started_day: 20_697,
            }))))[0]
        ),
        "Not recorded: this home's accounting ledger was created on 2026-09-01 (UTC), after this day; collection coverage unknown."
    );
    assert_eq!(
        details(&inspection_pages(Ok(empty_day(Some(EmptyDay::NotRecorded))))[0]),
        "No recorded attempts in this conversation or its subagents; collection coverage unknown."
    );
}

/// The started day is a UTC date: the last millisecond of a UTC day and the
/// first of the next name different days.
#[test]
fn the_started_date_is_the_utc_date() {
    let opening = |started_day| {
        first_screen(
            &inspection_pages(Ok(empty_day(Some(EmptyDay::BeforeConversation {
                started_day,
            }))))[0],
        )[0]
        .clone()
    };
    assert!(opening(/*started_day*/ 20_745).contains("started on 2026-10-19 (UTC)"));
    assert!(opening(/*started_day*/ 20_746).contains("started on 2026-10-20 (UTC)"));
}

async fn opened(day: InspectionDay) -> crate::chatwidget::ChatWidget {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.on_terminal_resize(/*width*/ 100);
    chat.open_accounting_command(
        "requests 2026-01-01",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    );
    let AppEvent::LoadAccountingInspector {
        generation,
        thread,
        day: requested,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!("expected inspector load");
    };
    chat.finish_accounting_inspector(generation, thread, requested, Ok(day));
    chat
}

#[tokio::test]
async fn empty_day_snapshots() {
    for (name, empty) in [
        (
            "cost_empty_day_before_conversation",
            Some(EmptyDay::BeforeConversation {
                started_day: STARTED,
            }),
        ),
        (
            "cost_empty_day_before_ledger",
            Some(EmptyDay::BeforeLedger {
                ledger_started_day: 20_697,
            }),
        ),
        ("cost_empty_day_not_recorded", Some(EmptyDay::NotRecorded)),
    ] {
        let chat = opened(empty_day(empty)).await;
        insta::assert_snapshot!(
            name,
            render_bottom_popup_with_height(&chat, /*width*/ 100, /*height*/ 24)
        );
    }
}
