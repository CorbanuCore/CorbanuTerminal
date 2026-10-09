//! acct-scope-62: a `/cost` page never implies zero when the evidence is "not
//! in this scope" or "no price", and the audit of its amounts is bound to every
//! page.

use super::super::InspectorPage;
use super::super::inspection_pages;
use crate::app_event::AppEvent;
use crate::chatwidget::tests::helpers::render_bottom_popup_with_height;
use crate::chatwidget::tests::make_chatwidget_manual;
use chrono::NaiveDate;
use codex_protocol::ThreadId;
use codex_state::accounting::Attempt;
use codex_state::accounting::BucketQuote;
use codex_state::accounting::DayTotals;
use codex_state::accounting::Decimal;
use codex_state::accounting::DeletedAttempts;
use codex_state::accounting::Dialect;
use codex_state::accounting::Inspection;
use codex_state::accounting::InspectionDay;
use codex_state::accounting::ObservationQuote;
use codex_state::accounting::OtherConversations;
use codex_state::accounting::RetentionCoverage;
use codex_state::accounting::Usage;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use uuid::Uuid;

const DAY_MS: i64 = 86_400_000;

fn utc_day() -> i64 {
    NaiveDate::from_ymd_opt(2026, 9, 16)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|time| time.and_utc().timestamp() / 86_400)
        .unwrap()
}

fn thread(n: u128) -> ThreadId {
    ThreadId::from_string(&Uuid::from_u128(n).to_string()).unwrap()
}

fn decimal(text: &str) -> Decimal {
    text.to_owned().try_into().unwrap()
}

/// OpenAI-style priced attempt: 80 noncached at $5/M, 20 cache read at
/// $0.50/M, 10 output at $30/M is $0.00071 exactly.
fn priced(id: u128, owner: ThreadId) -> ObservationQuote {
    let mut quote = unpriced(id, owner, "openai", "gpt-5.4");
    quote.usage = Usage {
        input: Some(100),
        noncached: Some(80),
        read: Some(20),
        output: Some(10),
        ..Usage::default()
    };
    quote.buckets = [
        BucketQuote::Priced(decimal("0.0004")),
        BucketQuote::Priced(decimal("0.00001")),
        BucketQuote::MissingUsage,
        BucketQuote::Priced(decimal("0.0003")),
    ];
    quote.known_subtotal = decimal("0.00071");
    quote.all_buckets_priced = Some(quote.known_subtotal);
    quote.subtotal_display = quote.known_subtotal.display();
    quote
}

/// A custom provider's attempt: tokens recorded, no published rate.
fn unpriced(id: u128, owner: ThreadId, provider: &str, model: &str) -> ObservationQuote {
    ObservationQuote {
        attempt: Attempt {
            attempt_id: Uuid::from_u128(id),
            request_id: Uuid::from_u128(1000 + id),
            thread_id: owner,
            turn: "synthetic".into(),
            retry_of: None,
            provider: provider.into(),
            model: model.into(),
            scope: Uuid::from_u128(5),
            dialect: Dialect::Inclusive,
            dispatched_at_ms: (utc_day() * DAY_MS + DAY_MS / 2).try_into().unwrap(),
        },
        observations: vec![],
        snapshot: None,
        usage: Usage {
            input: Some(50),
            noncached: Some(50),
            output: Some(5),
            ..Usage::default()
        },
        buckets: [
            BucketQuote::MissingRate,
            BucketQuote::MissingUsage,
            BucketQuote::MissingUsage,
            BucketQuote::MissingRate,
        ],
        known_subtotal: Decimal::default(),
        all_buckets_priced: None,
        subtotal_display: Decimal::default().display(),
        known_equivalent: Decimal::default(),
        all_buckets_equivalent: None,
        plan_burn_millis: None,
        plan_burn_milli_tokens: None,
        pricing_rules: 1,
    }
}

fn by_request(
    quotes: Vec<ObservationQuote>,
) -> std::collections::BTreeMap<Uuid, Vec<ObservationQuote>> {
    let mut requests = std::collections::BTreeMap::<Uuid, Vec<ObservationQuote>>::new();
    for quote in quotes {
        requests
            .entry(quote.attempt.request_id)
            .or_default()
            .push(quote);
    }
    requests
}

/// The open conversation's day, as the store returns it.
fn day(own: Vec<ObservationQuote>, others: Option<OtherConversations>) -> InspectionDay {
    let owner = thread(/*n*/ 1);
    let requests = by_request(own);
    let totals = DayTotals::from_quotes(requests.values().flatten()).unwrap();
    let read_at_ms = utc_day() * DAY_MS + DAY_MS * 3 / 4;
    InspectionDay::Ready(Inspection {
        owner,
        utc_day: utc_day(),
        read_at_ms,
        coverage: RetentionCoverage {
            completed_as_of_ms: read_at_ms,
            detail_expired_through_ms: None,
            aggregate_day_floor: 0,
            oldest_recorded_day: (!requests.is_empty()).then(utc_day),
        },
        own_totals: totals.clone(),
        totals,
        descendant_totals: DayTotals::default(),
        unknown_parent_totals: DayTotals::default(),
        unknown_parent_unavailable_threads: 0,
        unknown_parent_requests: Default::default(),
        requests,
        other_conversations: others,
    })
}

/// Two other conversations: one with two priced OpenAI requests, one on a
/// custom provider with no price.
fn two_other_conversations() -> OtherConversations {
    OtherConversations {
        conversations: 2,
        unavailable: 0,
        requests: by_request(vec![
            priced(/*id*/ 11, thread(/*n*/ 2)),
            priced(/*id*/ 12, thread(/*n*/ 2)),
            unpriced(/*id*/ 13, thread(/*n*/ 3), "local-mock", "mock-model"),
        ]),
        deleted_attempts: DeletedAttempts::Counted(0),
    }
}

fn first_screen(page: &InspectorPage) -> Vec<String> {
    let details = page.text.iter().position(|s| s == "—— Details ——").unwrap();
    page.text[..details].to_vec()
}

async fn opened(day: InspectionDay, width: u16) -> crate::chatwidget::ChatWidget {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.on_terminal_resize(width);
    chat.open_accounting_command(
        "requests 2026-09-16",
        NaiveDate::from_ymd_opt(2026, 9, 16).unwrap(),
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

/// Every `$` or `USD:` amount stated on a line.
fn stated_amounts(line: &str) -> Vec<String> {
    let mut amounts = Vec::new();
    for marker in ["$", "USD: "] {
        for (at, _) in line.match_indices(marker) {
            let amount: String = line[at + marker.len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            // A sentence's closing period is not part of the amount.
            let amount = amount.trim_end_matches('.');
            if !amount.is_empty() {
                amounts.push(amount.to_string());
            }
        }
    }
    amounts
}

/// No page states a zero amount: every fixture here has unknown or nonzero cost.
fn assert_never_zero(pages: &[InspectorPage]) {
    for page in pages {
        for line in &page.text {
            for amount in stated_amounts(line) {
                assert!(
                    amount.chars().any(|c| c.is_ascii_digit() && c != '0'),
                    "{:?} states a zero amount: {line}",
                    page.title
                );
            }
        }
    }
}

const NEXT_STEP: &str = "Next step for requests with no price: check the bill from local-mock. No published price covers them, so no cost is shown for them here.";
const OTHERS_BLOCK: [&str; 5] = [
    "Other conversations on this day, not included above: 3 requests in 2 conversations.",
    "• local-mock · mock-model — Pay per use. 1 request, 55 tokens. Estimated cost: no price available.",
    "• OpenAI · gpt-5.4 — Pay per use. 2 requests, 220 tokens. Estimated cost: $0.001420.",
    "Other conversations' estimated cost: at least $0.001420 (1 attempt had no price).",
    "To see those requests, open that conversation with /resume and run /cost there.",
];

/// The overview's one line on billing for a day of the two other
/// conversations' requests: neither provider states its charge.
const NOT_STATED: &str = "Costs are estimates from published prices; local-mock and OpenAI don't state their actual charge, so check the bill from local-mock and OpenAI.";

// Point 1: an empty conversation on a day other conversations spent on says
// what it covers, what it leaves out and what that cost, never a bare zero.
#[test]
fn empty_conversation_states_its_scope_and_the_days_other_conversations() {
    let pages = inspection_pages(Ok(day(Vec::new(), Some(two_other_conversations()))));
    let mut expected =
        vec!["This conversation on 2026-09-16 (UTC): no recorded requests.".to_string()];
    expected.extend(OTHERS_BLOCK.map(String::from));
    expected.extend([NEXT_STEP.to_string(), NOT_STATED.to_string()]);
    assert_eq!(first_screen(&pages[0]), expected);
    let details = &pages[0].text[expected.len() + 1..];
    assert_eq!(
        details[0],
        "No recorded attempts in this conversation or its subagents; collection coverage unknown."
    );
    assert!(details.contains(
        &"Known subtotal exact USD: none — no recorded attempts in this conversation".to_string()
    ));
    for page in &pages {
        for line in &page.text {
            assert!(!line.contains("in this day"), "{line}");
        }
    }
    assert_never_zero(&pages);
}

#[test]
fn every_day_view_says_whether_other_conversations_were_read() {
    let first = |others| first_screen(&inspection_pages(Ok(day(Vec::new(), others)))[0]);
    let empty = |deleted_attempts| OtherConversations {
        deleted_attempts,
        ..Default::default()
    };
    assert_eq!(
        first(Some(empty(DeletedAttempts::Counted(0)))),
        vec![
            "This conversation on 2026-09-16 (UTC): no recorded requests.",
            "No other conversation recorded requests on this day.",
            "Costs are estimates from published prices; your provider's bill is the final amount.",
        ]
    );
    // #286: a day whose only other spend was deleted never reads as empty.
    assert_eq!(
        first(Some(empty(DeletedAttempts::Counted(7)))),
        vec![
            "This conversation on 2026-09-16 (UTC): no recorded requests.",
            "No other saved conversation has recorded requests on this day.",
            "Deleted conversations or subagents sent 7 request attempts on this day. Their recorded cost was deleted with them, so it is not included here; any cost they incurred is on your provider's bill.",
            "Costs are estimates from published prices; your provider's bill is the final amount.",
        ]
    );
    assert_eq!(
        first(Some(empty(DeletedAttempts::PastDetailWindow)))[1..3].to_vec(),
        vec![
            "No other saved conversation has recorded requests on this day.",
            "Days older than 90 days keep too little detail to count requests from deleted conversations; any they made are not included here.",
        ]
    );
    assert_eq!(
        first(Some(empty(DeletedAttempts::Unread)))[2],
        "Requests from deleted conversations could not be counted for this day; any they made are not included here."
    );
    assert_eq!(
        first(None)[1],
        "Other conversations are not included in this view; /cost covers only the open conversation."
    );
    // A conversation the store could not read is stated as unread, not as zero.
    let partly = OtherConversations {
        conversations: 2,
        unavailable: 1,
        requests: by_request(vec![priced(/*id*/ 11, thread(/*n*/ 2))]),
        deleted_attempts: DeletedAttempts::Counted(0),
    };
    assert_eq!(
        first(Some(partly))[1..6].to_vec(),
        vec![
            "Other conversations on this day, not included above: 1 request in 2 conversations.",
            "• OpenAI · gpt-5.4 — Pay per use. 1 request, 110 tokens. Estimated cost: $0.000710.",
            "Other conversations' estimated cost: $0.000710.",
            "1 conversation could not be read in full; their cost is unknown and not included.",
            "To see those requests, open that conversation with /resume and run /cost there.",
        ]
    );
    // Deleted spend is disclosed beside the open conversations' total too.
    let mut deleted = two_other_conversations();
    deleted.deleted_attempts = DeletedAttempts::Counted(1);
    let screen = first(Some(deleted));
    let resume = screen
        .iter()
        .position(|line| line.starts_with("To see those requests"))
        .unwrap();
    assert_eq!(
        screen[resume + 1],
        "Deleted conversations or subagents sent 1 request attempt on this day. Their recorded cost was deleted with them, so it is not included here; any cost they incurred is on your provider's bill."
    );
}

// Point 2: a mixed-provider day names the custom provider wherever it ran, in
// this conversation and outside it, and its missing price is never folded away.
#[test]
fn mixed_provider_day_names_every_route_in_and_outside_the_conversation() {
    let own = vec![
        priced(/*id*/ 1, thread(/*n*/ 1)),
        unpriced(/*id*/ 2, thread(/*n*/ 1), "local-mock", "mock-model"),
    ];
    let pages = inspection_pages(Ok(day(own, Some(two_other_conversations()))));
    let mut expected = vec![
        "This conversation on 2026-09-16 (UTC):".to_string(),
        "• local-mock · mock-model — Pay per use. 1 request, 55 tokens. Estimated cost: no price available.".to_string(),
        "• OpenAI · gpt-5.4 — Pay per use. 1 request, 110 tokens. Estimated cost: $0.000710.".to_string(),
    ];
    expected.extend(OTHERS_BLOCK.map(String::from));
    expected.extend([
        NEXT_STEP.to_string(),
        NOT_STATED.to_string(),
        "Select a provider below to see its requests.".to_string(),
    ]);
    assert_eq!(first_screen(&pages[0]), expected);
    let details = &pages[0].text[expected.len() + 1..];
    assert_eq!(
        details[..2].to_vec(),
        vec![
            "Known estimated token cost: $0.000710 + unknown costs",
            "Full recorded estimate: unavailable (1 of 2 billed attempts incomplete)",
        ]
    );
    assert_never_zero(&pages);
}

// Point 3: a request with no price names the next step on every page that
// shows it, and no subtotal reads as zero.
#[test]
fn missing_price_pages_state_the_next_step_and_never_zero() {
    let own = vec![unpriced(
        /*id*/ 2,
        thread(/*n*/ 1),
        "local-mock",
        "mock-model",
    )];
    let pages = inspection_pages(Ok(day(own, Some(OtherConversations::default()))));
    let titles: Vec<&str> = pages
        .iter()
        .filter(|page| page.text.iter().any(|line| line == NEXT_STEP))
        .map(|page| page.title.as_str())
        .collect();
    assert_eq!(
        titles,
        vec![
            "Cost — this conversation",
            "Request",
            "Attempt, components and original price",
            "Provider: local-mock; Model: mock-model",
        ]
    );
    let subtotals: Vec<&String> = pages
        .iter()
        .flat_map(|page| &page.text)
        .filter(|line| line.contains("exact USD:"))
        .collect();
    assert_eq!(
        subtotals,
        vec![
            "Known subtotal exact USD: none — no price for these attempts",
            "Known subtotal exact USD: none — no price for these attempts",
            "Known estimate exact USD: none — no price for these attempts",
            "Known estimate exact USD: none — no price for these attempts",
        ]
    );
    assert_never_zero(&pages);
    insta::assert_snapshot!("cost_missing_price_request_page", pages[1].text.join("\n"));
}

/// Each page, in order, and the monetary statements it must carry.
const AUDIT: [(&str, &[&str]); 11] = [
    (
        "Cost — this conversation",
        &[
            "Estimated cost: $0.000710.",
            "Other conversations' estimated cost: at least $0.001420",
            "Known estimated token cost: $0.000710 + unknown costs",
            "Known subtotal exact USD: 0.00071",
        ],
    ),
    ("Request", &["Estimated cost: $0.000710"]),
    (
        "Attempt, components and original price",
        &[
            "Estimated cost: $0.000710",
            "Known subtotal exact USD: 0.00071",
        ],
    ),
    ("Request", &["Estimated cost: no price available"]),
    (
        "Attempt, components and original price",
        &["Known subtotal exact USD: none — no price for these attempts"],
    ),
    (
        "Root's own attempts",
        &["Known estimate exact USD: 0.00071"],
    ),
    (
        "Descendant attempts",
        &["No recorded attempts; collection coverage unknown."],
    ),
    (
        "Provider: local-mock; Model: mock-model",
        &["Known estimate exact USD: none — no price for these attempts"],
    ),
    (
        "Provider: openai; Model: gpt-5.4",
        &["Known estimate exact USD: 0.00071"],
    ),
    (
        "Unknown provider/model attribution",
        &["No recorded attempts; collection coverage unknown."],
    ),
    (
        "Unknown parent population",
        &["No recorded attempts; collection coverage unknown."],
    ),
];

/// Bound audit: the exact page identities, and on each its own statements.
fn audit(pages: &[InspectorPage]) -> Result<(), String> {
    let titles: Vec<&str> = pages.iter().map(|page| page.title.as_str()).collect();
    let expected: Vec<&str> = AUDIT.iter().map(|(title, _)| *title).collect();
    if titles != expected {
        return Err(format!("page identities {titles:?}"));
    }
    for (index, (page, (_, statements))) in pages.iter().zip(AUDIT).enumerate() {
        for statement in statements {
            if !page.text.iter().any(|line| line.contains(statement)) {
                return Err(format!("page {index} lost {statement:?}"));
            }
        }
    }
    Ok(())
}

fn audited_pages() -> Vec<InspectorPage> {
    let own = vec![
        priced(/*id*/ 1, thread(/*n*/ 1)),
        unpriced(/*id*/ 2, thread(/*n*/ 1), "local-mock", "mock-model"),
    ];
    inspection_pages(Ok(day(own, Some(two_other_conversations()))))
}

// Point 4: the audit is bound to every page, so a page that loses its amount,
// or a missing page, fails even when the same figure survives elsewhere.
#[test]
fn monetary_audit_is_bound_to_every_page() {
    assert_eq!(audit(&audited_pages()), Ok(()));
    for (index, (_, statements)) in AUDIT.iter().enumerate() {
        for statement in *statements {
            let mut pages = audited_pages();
            pages[index].text.retain(|line| !line.contains(statement));
            assert!(
                audit(&pages).is_err(),
                "losing {statement:?} on page {index} passed"
            );
            // An unbound check, "the figure appears somewhere", misses most of these.
            if let Some(amount) = stated_amounts(statement).first() {
                let anywhere = pages
                    .iter()
                    .flat_map(|page| &page.text)
                    .any(|line| line.contains(amount.as_str()));
                assert!(anywhere, "{statement:?} is the figure's only occurrence");
            }
        }
    }
    let mut pages = audited_pages();
    pages.pop();
    assert!(audit(&pages).is_err(), "a missing page passed");
}

#[tokio::test]
async fn cost_scope_snapshots_wide_and_narrow() {
    let empty = opened(
        day(Vec::new(), Some(two_other_conversations())),
        /*width*/ 100,
    )
    .await;
    insta::assert_snapshot!(
        "cost_empty_conversation_other_conversations_wide",
        render_bottom_popup_with_height(&empty, /*width*/ 100, /*height*/ 30)
    );
    let own = vec![
        priced(/*id*/ 1, thread(/*n*/ 1)),
        unpriced(/*id*/ 2, thread(/*n*/ 1), "local-mock", "mock-model"),
    ];
    let mut narrow = opened(day(own, Some(two_other_conversations())), /*width*/ 40).await;
    insta::assert_snapshot!(
        "cost_mixed_provider_day_narrow",
        render_bottom_popup_with_height(&narrow, /*width*/ 40, /*height*/ 60)
    );
    // Scroll to the other conversations' block on the same narrow screen.
    for _ in 0..12 {
        narrow.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }
    insta::assert_snapshot!(
        "cost_mixed_provider_day_narrow_other_conversations",
        render_bottom_popup_with_height(&narrow, /*width*/ 40, /*height*/ 60)
    );
}

/// The OpenAI price bound at dispatch: $5/M noncached, $0.50/M cache read,
/// $30/M output, no stated cache-write rate.
fn openai_price() -> codex_state::accounting::Snapshot {
    serde_json::from_value(serde_json::json!({
        "id": Uuid::from_u128(77), "provider": "openai", "model": "gpt-5.4", "scope": Uuid::from_u128(5),
        "currency": "USD", "unit": "PerMillionTokens",
        "rates": {"noncached": "5", "read": "0.5", "write": null, "output": "30"},
        "source_reference": Uuid::from_u128(78), "source_kind": "ProviderPublished",
        "observed_at_ms": 0, "approved_at_ms": 0, "effective_from_ms": 0, "effective_end_ms": null
    }))
    .unwrap()
}

/// An OpenAI attempt priced at dispatch whose provider reported no token
/// counts at all: a failed or refused request.
fn no_usage(id: u128, owner: ThreadId) -> ObservationQuote {
    let mut quote = unpriced(id, owner, "openai", "gpt-5.4");
    quote.snapshot = Some(openai_price());
    quote.usage = Usage::default();
    quote.buckets = [BucketQuote::MissingUsage; 4];
    quote
}

/// The priced OpenAI attempt with its output count missing: $0.00041 of input
/// is known, the output's cost is not.
fn output_unreported(id: u128, owner: ThreadId) -> ObservationQuote {
    let mut quote = priced(id, owner);
    quote.snapshot = Some(openai_price());
    quote.usage.output = None;
    quote.buckets[3] = BucketQuote::MissingUsage;
    quote.known_subtotal = decimal("0.00041");
    quote.all_buckets_priced = None;
    quote.subtotal_display = quote.known_subtotal.display();
    quote
}

fn own_day(own: Vec<ObservationQuote>) -> Vec<InspectorPage> {
    inspection_pages(Ok(day(own, Some(OtherConversations::default()))))
}

fn link_labels(pages: &[InspectorPage]) -> Vec<String> {
    pages[0]
        .links
        .iter()
        .map(|(label, _)| label.clone())
        .collect()
}

// No-usage: a request whose provider did not report the counts its price
// needs reads as incomplete usage, not as a missing price, and never as zero.
#[test]
fn missing_usage_is_labelled_apart_from_missing_price() {
    let silent = own_day(vec![no_usage(/*id*/ 1, thread(/*n*/ 1))]);
    assert_eq!(
        first_screen(&silent[0])[1],
        "• OpenAI · gpt-5.4 — Pay per use. 1 request, tokens not reported. Estimated cost: not available — usage incomplete."
    );
    let labels = link_labels(&silent);
    for label in [
        "Request 1 · OpenAI · gpt-5.4 · usage incomplete",
        "OpenAI · gpt-5.4 — usage incomplete (1 request)",
    ] {
        assert!(labels.contains(&label.to_string()), "{label}: {labels:#?}");
    }
    assert!(
        !silent
            .iter()
            .flat_map(|page| &page.text)
            .any(|line| line.contains("no price")),
        "a priced route must not be told to look for a missing price"
    );
    // #288: it still gets a next step, naming its own provider, on its own
    // first screen and request page.
    const INCOMPLETE: &str = "Next step for requests with incomplete usage: check the bill from OpenAI. Their provider did not report every token count, so the estimate is missing or only a lower bound.";
    for title in ["Cost — this conversation", "Request"] {
        let page = silent.iter().find(|page| page.title == title).unwrap();
        assert!(
            page.text.iter().any(|line| line == INCOMPLETE),
            "{title}: {:#?}",
            page.text
        );
    }
    assert_never_zero(&silent);

    let partial = own_day(vec![output_unreported(/*id*/ 1, thread(/*n*/ 1))]);
    assert_eq!(
        first_screen(&partial[0])[1],
        "• OpenAI · gpt-5.4 — Pay per use. 1 request, 100+ tokens. Estimated cost: at least $0.000410 (1 attempt had incomplete usage)."
    );
    assert_never_zero(&partial);

    // Both causes on one route are each counted, and each gets its own next
    // step, the missing price first.
    let mixed = own_day(vec![
        no_usage(/*id*/ 1, thread(/*n*/ 1)),
        unpriced(/*id*/ 2, thread(/*n*/ 1), "openai", "gpt-5.4"),
    ]);
    let first = first_screen(&mixed[0]);
    assert_eq!(
        first[1],
        "• OpenAI · gpt-5.4 — Pay per use. 2 requests, 55+ tokens. Estimated cost: not available (1 attempt had no price, 1 had incomplete usage)."
    );
    let steps: Vec<&String> = first
        .iter()
        .filter(|line| line.starts_with("Next step"))
        .collect();
    assert_eq!(
        steps,
        vec![
            "Next step for requests with no price: check the bill from OpenAI. No published price covers them, so no cost is shown for them here.",
            "Next step for requests with incomplete usage: check the bill from OpenAI. Their provider did not report every token count, so the estimate is missing or only a lower bound.",
        ]
    );
    assert!(link_labels(&mixed).contains(
        &"OpenAI · gpt-5.4 — cost not available (1 attempt had no price, 1 had incomplete usage) (2 requests)".to_string()
    ));
    assert_never_zero(&mixed);
}

// Stale estimate: figures read ahead of the ledger's checkpoint say so on every
// page that states an amount, and a reply to a superseded read never replaces
// the refreshed view.
#[tokio::test]
async fn stale_estimate_is_marked_and_never_replaces_a_refresh() {
    const STALE: &str = "Snapshot is not current; newer activity is unverified";
    let stale = || {
        let mut view = day(
            vec![priced(/*id*/ 1, thread(/*n*/ 1))],
            Some(OtherConversations::default()),
        );
        if let InspectionDay::Ready(inspection) = &mut view {
            inspection.coverage.completed_as_of_ms -= 60_000;
        }
        view
    };
    let pages = inspection_pages(Ok(stale()));
    let priced_pages: Vec<&InspectorPage> = pages
        .iter()
        .filter(|page| page.text.iter().any(|line| line.contains('$')))
        .collect();
    assert!(
        priced_pages.len() >= 3,
        "fixture states amounts on several pages"
    );
    for page in priced_pages {
        assert!(
            page.text.iter().any(|line| line == STALE),
            "{:?} states an amount without saying it may be behind",
            page.title
        );
    }

    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.on_terminal_resize(/*width*/ 120);
    chat.open_accounting_command(
        "requests 2026-09-16",
        NaiveDate::from_ymd_opt(2026, 9, 16).unwrap(),
    );
    let AppEvent::LoadAccountingInspector {
        generation: superseded,
        thread: owner,
        day: requested,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!("expected inspector load");
    };
    chat.finish_accounting_inspector(superseded, owner, requested, Ok(stale()));
    chat.refresh_accounting_inspector(superseded);
    let AppEvent::LoadAccountingInspector {
        generation: current,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!("expected refresh load");
    };
    assert_ne!(current, superseded);
    let fresh = day(
        vec![
            priced(/*id*/ 1, thread(/*n*/ 1)),
            priced(/*id*/ 2, thread(/*n*/ 1)),
        ],
        Some(OtherConversations::default()),
    );
    chat.finish_accounting_inspector(current, owner, requested, Ok(fresh));
    chat.finish_accounting_inspector(superseded, owner, requested, Ok(stale()));
    let screen = render_bottom_popup_with_height(&chat, /*width*/ 120, /*height*/ 30);
    assert!(screen.contains("2 requests, 220 tokens"), "{screen}");
    assert!(!screen.contains(STALE), "{screen}");
}

// Unavailable backend: every state with no figures says what to do next,
// directly under the reason.
#[test]
fn every_unavailable_state_names_a_next_step() {
    let states = [
        InspectionDay::Absent,
        InspectionDay::MissingThread,
        InspectionDay::CheckpointLag,
        InspectionDay::NeedsRefresh,
        InspectionDay::TooLarge,
        InspectionDay::DetailUnavailable {
            coverage: RetentionCoverage {
                completed_as_of_ms: 200 * DAY_MS,
                detail_expired_through_ms: Some(100 * DAY_MS),
                aggregate_day_floor: 0,
                oldest_recorded_day: Some(0),
            },
            read_at_ms: 200 * DAY_MS,
            compact: true,
        },
    ];
    for state in states {
        let label = format!("{state:?}");
        let pages = inspection_pages(Ok(state));
        assert_eq!(pages.len(), 1, "{label}");
        assert!(
            pages[0].text[1].starts_with("Next step: "),
            "{label}: {:#?}",
            pages[0].text
        );
        assert!(
            !pages[0].text.iter().any(|line| line.contains('$')),
            "{label}"
        );
        // A build without `/cost` never tells the user to run it.
        if !cfg!(feature = "developer-accounting") {
            assert!(
                !pages[0].text.iter().any(|line| line.contains("/cost")),
                "{label}: {:#?}",
                pages[0].text
            );
        }
    }
    assert_eq!(
        inspection_pages(Ok(InspectionDay::CheckpointLag))[0].text[1],
        "Next step: send a turn in this conversation to bring the ledger up to date, then select Refresh."
    );

    // A load error's next step gets its own line too.
    let pages = inspection_pages(Err(
        "Unavailable — inspection timed out. Next step: select Refresh.".into(),
    ));
    assert_eq!(
        pages[0].text[..2].to_vec(),
        vec![
            "Unavailable — inspection timed out.",
            "Next step: select Refresh.",
        ]
    );
}

/// The priced OpenAI attempt, run on subscription capacity: no money spent.
fn plan_work(id: u128, owner: ThreadId) -> ObservationQuote {
    let mut quote = priced(id, owner);
    quote.plan_burn_millis = Some(1000);
    quote.known_equivalent = quote.known_subtotal;
    quote.all_buckets_equivalent = Some(quote.known_subtotal);
    quote.known_subtotal = Decimal::default();
    quote.all_buckets_priced = None;
    quote.subtotal_display = quote.known_subtotal.display();
    quote
}

// A day of only subscription work points at no bill, unless pay-per-use
// figures from other conversations share the screen.
#[test]
fn subscription_only_day_points_at_no_bill() {
    const BILL: &str = NOT_STATED;
    const PLAN: &str = "Subscription work is not billed per request; any figure here is what it would cost at API prices.";
    let own = || vec![plan_work(/*id*/ 1, thread(/*n*/ 1))];
    let alone = first_screen(&own_day(own())[0]);
    assert!(alone.contains(&PLAN.to_string()), "{alone:#?}");
    assert!(!alone.contains(&BILL.to_string()), "{alone:#?}");
    let beside =
        first_screen(&inspection_pages(Ok(day(own(), Some(two_other_conversations()))))[0]);
    assert!(beside.contains(&BILL.to_string()), "{beside:#?}");
    assert!(!beside.contains(&PLAN.to_string()), "{beside:#?}");

    // A route with both kinds of work rewords only its pay-per-use figure.
    let mixed = own_day(vec![
        plan_work(/*id*/ 1, thread(/*n*/ 1)),
        unpriced(/*id*/ 2, thread(/*n*/ 1), "openai", "gpt-5.4"),
    ]);
    let labels = link_labels(&mixed);
    assert!(
        labels.iter().any(|label| label
            .starts_with("OpenAI · gpt-5.4 — no price available; subscription part — ")),
        "{labels:#?}"
    );
}

// #288: a request on a provider with no price that also reported no tokens
// gets the missing-price next step for its own provider.
#[test]
fn unpriced_request_without_usage_names_its_own_provider() {
    let mut silent = unpriced(/*id*/ 1, thread(/*n*/ 1), "zainousage", "glm-5.2");
    silent.usage = Usage::default();
    silent.buckets = [BucketQuote::MissingUsage; 4];
    let pages = own_day(vec![silent]);
    let first = first_screen(&pages[0]);
    assert_eq!(
        first[1],
        "• zainousage · Z.AI GLM 5.2 — Pay per use. 1 request, tokens not reported. Estimated cost: no price available."
    );
    let step = "Next step for requests with no price: check the bill from zainousage. No published price covers them, so no cost is shown for them here.";
    assert!(first.contains(&step.to_string()), "{first:#?}");
    let request = pages.iter().find(|page| page.title == "Request").unwrap();
    assert!(
        request.text.iter().any(|line| line == step),
        "{:#?}",
        request.text
    );
    assert_never_zero(&pages);
}

// #288: an unpriced request that reported some counts but no priced one (here
// input only, output missing) still gets the missing-price step, and a request
// priced only in part gets its own.
#[test]
fn unpriced_and_partly_priced_requests_get_next_steps() {
    let mut input_only = unpriced(/*id*/ 1, thread(/*n*/ 1), "local-mock", "mock-model");
    input_only.usage = Usage {
        input: Some(100),
        read: Some(0),
        ..Usage::default()
    };
    input_only.buckets = [
        BucketQuote::MissingUsage,
        BucketQuote::MissingRate,
        BucketQuote::MissingUsage,
        BucketQuote::MissingUsage,
    ];
    let pages = own_day(vec![input_only]);
    let step = "Next step for requests with no price: check the bill from local-mock. No published price covers them, so no cost is shown for them here.";
    for title in ["Cost — this conversation", "Request"] {
        let page = pages.iter().find(|page| page.title == title).unwrap();
        assert!(
            page.text.iter().any(|line| line == step),
            "{title}: {:#?}",
            page.text
        );
    }

    let mut in_part = priced(/*id*/ 2, thread(/*n*/ 1));
    in_part.snapshot = Some(openai_price());
    in_part.buckets[3] = BucketQuote::MissingRate;
    in_part.known_subtotal = decimal("0.00041");
    in_part.all_buckets_priced = None;
    in_part.subtotal_display = in_part.known_subtotal.display();
    let in_part_pages = own_day(vec![in_part]);
    let first = first_screen(&in_part_pages[0]);
    assert_eq!(
        first[1],
        "• OpenAI · gpt-5.4 — Pay per use. 1 request, 110 tokens. Estimated cost: at least $0.000410 (1 attempt had no price)."
    );
    let step = "Next step for requests priced only in part: check the bill from OpenAI. Some of their tokens have no published price, so the estimate is only a lower bound.";
    for page in in_part_pages
        .iter()
        .filter(|page| ["Cost — this conversation", "Request"].contains(&page.title.as_str()))
    {
        let steps: Vec<&String> = page
            .text
            .iter()
            .filter(|line| line.starts_with("Next step"))
            .collect();
        assert_eq!(steps, vec![step], "{}", page.title);
    }
    assert_never_zero(&pages);
    assert_never_zero(&in_part_pages);
}
