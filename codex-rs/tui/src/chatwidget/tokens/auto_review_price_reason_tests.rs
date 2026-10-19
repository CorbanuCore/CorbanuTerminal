//! #913 / D31(c): explain the absent Codex Auto Review price on real inspection
//! pages, while preserving the existing no-price and billing-scope behavior.
//! These source-aware fixtures were frozen before the production change.

use super::DAY_MS;
use super::InspectorPage;
use super::assert_never_zero;
use super::by_request;
use super::day;
use super::first_screen;
use super::inspection_pages;
use super::own_day;
use super::priced;
use super::thread;
use super::unpriced;
use super::utc_day;
use super::with_basis;
use codex_state::accounting::AttemptAccount;
use codex_state::accounting::Basis;
use codex_state::accounting::BasisSource;
use codex_state::accounting::BucketQuote;
use codex_state::accounting::DeletedAttempts;
use codex_state::accounting::InspectionBucket;
use codex_state::accounting::InspectionDay;
use codex_state::accounting::InspectionGrouping;
use codex_state::accounting::InspectionRange;
use codex_state::accounting::ObservationQuote;
use codex_state::accounting::OtherConversations;
use codex_state::accounting::Usage;
use pretty_assertions::assert_eq;
use uuid::Uuid;

const REASON: &str = "OpenAI doesn't publish a price for Codex Auto Review.";
const OPENAI_NEXT_STEP: &str = "Next step for requests with no price: check the bill from OpenAI. No published price covers them, so no cost is shown for them here.";

fn review(id: u128) -> ObservationQuote {
    unpriced(id, thread(/*n*/ 1), "openai", "codex-auto-review")
}

fn reason_count(page: &InspectorPage) -> usize {
    page.text
        .iter()
        .filter(|line| line.as_str() == REASON)
        .count()
}

fn request_page(pages: &[InspectorPage], request_id: Uuid) -> &InspectorPage {
    let request_line = format!("Request: {request_id}");
    pages
        .iter()
        .find(|page| page.title == "Request" && page.text.contains(&request_line))
        .unwrap()
}

#[test]
fn unpriced_auto_review_explains_each_displayed_scope_without_a_zero_price() {
    let pages = own_day(vec![review(/*id*/ 1)]);
    // Snapshot the actual explanation text and its page topology. Model display
    // names belong to #352 and are deliberately outside this regression.
    let explanation_pages: Vec<String> = pages
        .iter()
        .filter_map(|page| {
            let mut lines: Vec<&str> = page
                .text
                .iter()
                .map(String::as_str)
                .filter(|line| *line == REASON || *line == OPENAI_NEXT_STEP)
                .collect();
            if lines.is_empty() {
                return None;
            }
            assert!(
                page.text
                    .iter()
                    .any(|line| line.contains("no price available"))
            );
            lines.sort_unstable();
            let title = if page.title.starts_with("Provider: ") {
                "Provider group"
            } else {
                page.title.as_str()
            };
            Some(format!("{title}\n{}", lines.join("\n")))
        })
        .collect();
    insta::assert_snapshot!(explanation_pages.join("\n\n"), @r#"
    Cost — this conversation
    Next step for requests with no price: check the bill from OpenAI. No published price covers them, so no cost is shown for them here.
    OpenAI doesn't publish a price for Codex Auto Review.

    Request
    Next step for requests with no price: check the bill from OpenAI. No published price covers them, so no cost is shown for them here.
    OpenAI doesn't publish a price for Codex Auto Review.

    Attempt, components and original price
    Next step for requests with no price: check the bill from OpenAI. No published price covers them, so no cost is shown for them here.
    OpenAI doesn't publish a price for Codex Auto Review.

    Provider group
    Next step for requests with no price: check the bill from OpenAI. No published price covers them, so no cost is shown for them here.
    OpenAI doesn't publish a price for Codex Auto Review.
    "#);
    assert_never_zero(&pages);
}

#[test]
fn repeated_reviews_and_other_providers_keep_one_reason_per_applicable_scope() {
    let first = review(/*id*/ 1);
    let second = review(/*id*/ 2);
    let other = unpriced(/*id*/ 3, thread(/*n*/ 1), "local-mock", "mock-model");
    let known = priced(/*id*/ 4, thread(/*n*/ 1));
    let requests = [
        (first.attempt.request_id, 1),
        (second.attempt.request_id, 1),
        (other.attempt.request_id, 0),
        (known.attempt.request_id, 0),
    ];
    let pages = own_day(vec![first, second, other, known]);
    assert_eq!(reason_count(&pages[0]), 1);
    for (request_id, expected) in requests {
        assert_eq!(reason_count(request_page(&pages, request_id)), expected);
    }
    let provider_counts: Vec<usize> = pages
        .iter()
        .filter(|page| page.title.starts_with("Provider: "))
        .map(reason_count)
        .collect();
    assert_eq!(provider_counts.len(), 3);
    assert_eq!(provider_counts.iter().sum::<usize>(), 1);
    assert!(
        pages[0]
            .text
            .iter()
            .any(|line| line == "Known estimated token cost: $0.000710 + unknown costs")
    );
    assert!(
        pages[0].text.iter().any(|line| line
            == "Full recorded estimate: unavailable (3 of 4 billed attempts incomplete)")
    );
    assert!(pages[0].text.iter().any(|line| line == "Next step for requests with no price: check the bill from OpenAI and local-mock. No published price covers them, so no cost is shown for them here."));
    assert_never_zero(&pages);
}

#[test]
fn same_slug_other_routes_priced_and_non_billed_work_do_not_gain_the_reason() {
    let mut fully_priced = priced(/*id*/ 1, thread(/*n*/ 1));
    fully_priced.attempt.model = "codex-auto-review".into();
    let mut only_zero = review(/*id*/ 1);
    only_zero.usage = Usage {
        input: Some(0),
        noncached: Some(0),
        read: Some(0),
        write: Some(0),
        output: Some(0),
        ..Usage::default()
    };
    only_zero.buckets = [BucketQuote::MissingRate; 4];
    let cases = [
        (
            "another OpenAI model",
            vec![unpriced(
                /*id*/ 1,
                thread(/*n*/ 1),
                "openai",
                "unknown-model",
            )],
        ),
        (
            "same slug on custom route",
            vec![unpriced(
                /*id*/ 1,
                thread(/*n*/ 1),
                "my-gateway",
                "codex-auto-review",
            )],
        ),
        (
            "subscription",
            vec![with_basis(
                review(/*id*/ 1),
                Basis::PlanEquivalent,
                BasisSource::UserConfig,
            )],
        ),
        (
            "local",
            vec![with_basis(
                review(/*id*/ 1),
                Basis::Local,
                BasisSource::UserConfig,
            )],
        ),
        (
            "undeclared basis",
            vec![with_basis(
                review(/*id*/ 1),
                Basis::Undeclared,
                BasisSource::UserConfig,
            )],
        ),
        ("fully priced", vec![fully_priced]),
        ("zero tokens", vec![only_zero]),
        (
            "missing model",
            vec![unpriced(/*id*/ 1, thread(/*n*/ 1), "openai", "")],
        ),
        ("empty day", Vec::new()),
    ];
    for (label, quotes) in cases {
        let pages = own_day(quotes);
        assert!(!pages.is_empty(), "{label}");
        assert_eq!(pages.iter().map(reason_count).sum::<usize>(), 0, "{label}");
    }
}

#[test]
fn named_account_review_keeps_its_account_and_model_price_reason() {
    let mut quote = review(/*id*/ 1);
    quote.attempt.account = Some(AttemptAccount {
        name: "work".into(),
        fingerprint: Some("0123456789ab".into()),
    });
    let request_id = quote.attempt.request_id;
    let pages = own_day(vec![quote]);
    let request = request_page(&pages, request_id);
    assert!(request.text.iter().any(|line| line == "Account: work"));
    assert_eq!(reason_count(request), 1);
    assert_eq!(reason_count(&pages[0]), 1);
    let provider = pages
        .iter()
        .find(|page| page.title.starts_with("Provider: "))
        .unwrap();
    assert!(provider.title.contains("Account: work"));
    assert_eq!(reason_count(provider), 1);
    assert_never_zero(&pages);
}

#[test]
fn other_conversation_review_explains_the_overview_without_leaking_into_own_request() {
    let own = priced(/*id*/ 1, thread(/*n*/ 1));
    let request_id = own.attempt.request_id;
    let mut outside = review(/*id*/ 2);
    outside.attempt.thread_id = thread(/*n*/ 2);
    let pages = inspection_pages(Ok(day(
        vec![own],
        Some(OtherConversations {
            conversations: 1,
            unavailable: 0,
            requests: by_request(vec![outside]),
            deleted_attempts: DeletedAttempts::Counted(0),
        }),
    )));
    assert_eq!(reason_count(&pages[0]), 1);
    assert_eq!(reason_count(request_page(&pages, request_id)), 0);
    assert_eq!(pages.iter().map(reason_count).sum::<usize>(), 1);
    assert!(
        first_screen(&pages[0])
            .iter()
            .any(|line| line == "Other conversations' estimated cost: no price available.")
    );
    assert!(
        request_page(&pages, request_id)
            .text
            .iter()
            .any(|line| line == "Estimated cost: $0.000710")
    );
    assert_never_zero(&pages);
}

#[test]
fn complete_range_explains_the_model_once_without_changing_unknown_totals() {
    let start_ms = utc_day() * DAY_MS;
    let end_ms = start_ms + DAY_MS;
    let InspectionDay::Ready(mut view) = day(
        vec![review(/*id*/ 1), review(/*id*/ 2)],
        /*others*/ None,
    ) else {
        panic!("ready synthetic day");
    };
    view.read_at_ms = end_ms;
    view.coverage.completed_as_of_ms = end_ms;
    let pages = inspection_pages(Ok(InspectionDay::Range {
        requested: InspectionRange {
            start_ms,
            end_ms,
            grouping: InspectionGrouping::Day,
        },
        aggregate_day_floor: Some(0),
        read_at_ms: end_ms,
        buckets: vec![InspectionBucket {
            start_ms,
            end_ms,
            effective: Some((start_ms, end_ms)),
            partial: false,
            days: vec![InspectionDay::Ready(view)],
        }],
    }));
    assert_eq!(pages[0].title, "Recorded request range");
    assert_eq!(reason_count(&pages[0]), 1);
    assert!(pages[0].text.iter().any(|line| line == OPENAI_NEXT_STEP));
    assert!(
        pages[0]
            .text
            .iter()
            .any(|line| line.contains("unavailable") || line.contains("no price"))
    );
    assert_never_zero(&pages);
}
