use super::*;
use crate::chatwidget::tests::helpers::{render_bottom_popup, render_bottom_popup_with_height};
use crate::chatwidget::tests::make_chatwidget_manual;
use codex_app_server_protocol::AccountTokenUsageSummary;
use codex_state::accounting::{
    Attempt, DayTotals, Dialect, Inspection, Metric, RetentionCoverage, Usage,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pretty_assertions::assert_eq;

fn decimal(text: &str) -> Decimal {
    text.to_owned().try_into().unwrap()
}

fn quote() -> ObservationQuote {
    let known = decimal("0.00018");
    ObservationQuote {
        attempt: Attempt {
            attempt_id: Uuid::from_u128(1),
            request_id: Uuid::from_u128(2),
            thread_id: ThreadId::from_string(&Uuid::from_u128(3).to_string()).unwrap(),
            turn: "synthetic".into(),
            retry_of: Some(Uuid::from_u128(4)),
            provider: "synthetic".into(),
            model: "synthetic-model".into(),
            scope: Uuid::from_u128(5),
            dialect: Dialect::Inclusive,
            dispatched_at_ms: 0.try_into().unwrap(),
        },
        observations: vec![],
        snapshot: None,
        usage: Usage {
            input: Some(100),
            read: Some(20),
            output: Some(40),
            ..Usage::default()
        },
        buckets: [
            BucketQuote::MissingUsage,
            BucketQuote::Priced(decimal("0.00002")),
            BucketQuote::MissingUsage,
            BucketQuote::Priced(decimal("0.00016")),
        ],
        known_subtotal: known,
        all_buckets_priced: None,
        subtotal_display: known.display(),
    }
}

fn packet() -> InspectionDay {
    let q = quote();
    InspectionDay::Ready(Inspection {
        owner: q.attempt.thread_id,
        utc_day: 0,
        read_at_ms: 1,
        coverage: RetentionCoverage {
            completed_as_of_ms: 0,
            detail_expired_through_ms: None,
            aggregate_day_floor: 0,
            oldest_recorded_day: Some(0),
        },
        totals: DayTotals {
            known_usd: q.known_subtotal,
            unknown_estimates: 1,
            attempts: 1,
            measured: [Some(100), None, Some(20), None, Some(40), None, None].map(|n| Metric {
                known: n.unwrap_or(0),
                unknown: i64::from(n.is_none()),
            }),
        },
        own_totals: DayTotals::from_quotes([&q]).unwrap(),
        descendant_totals: DayTotals::default(),
        unknown_parent_totals: DayTotals::default(),
        unknown_parent_unavailable_threads: 0,
        unknown_parent_requests: Default::default(),
        requests: std::collections::BTreeMap::from([(q.attempt.request_id, vec![q])]),
    })
}

fn range_packet(partial: bool, day: InspectionDay) -> InspectionDay {
    InspectionDay::Range {
        requested: InspectionRange {
            start_ms: i64::from(partial),
            end_ms: 3_600_000,
            grouping: InspectionGrouping::Hour,
        },
        oldest_aggregate_day: Some(0),
        read_at_ms: 90 * 86_400_000,
        buckets: vec![codex_state::accounting::InspectionBucket {
            start_ms: 0,
            end_ms: 3_600_000,
            effective: Some((i64::from(partial), 3_600_000)),
            partial,
            days: vec![day],
        }],
    }
}

#[test]
fn accounting_inspect_range_partial_no_amount_and_explicit_coverage() {
    let InspectionDay::Ready(mut view) = packet() else {
        panic!()
    };
    view.read_at_ms = view.coverage.completed_as_of_ms;
    let pages = inspection_pages(Ok(range_packet(true, InspectionDay::Ready(view))));
    let text = pages
        .iter()
        .flat_map(|p| &p.text)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!text.contains('$'));
    assert!(text.contains("Partial bucket — excluded from totals"));
    assert!(text.contains("Range total unavailable"));
    assert!(
        pages
            .iter()
            .all(|p| p.text.iter().any(|t| t.contains("timezone: UTC")))
    );
    insta::assert_snapshot!(pages[0].text.join("\n"), @"
    Requested: [1970-01-01T00:00:00.001Z, 1970-01-01T01:00:00.000Z); timezone: UTC; grouping: Hour
    Oldest retained aggregate day (ledger): Some(0); 90-day drill-down cutoff: Some(0) ms UTC (exclusive)
    Collection coverage: unknown. Range estimate covers root and resolved descendants; unknown ancestry stays separate in bucket breakdowns. Billed cost: unavailable — no settlement evidence.
    Range: Unknown parent population: 0 inspectable attempts, excluded from range total
    Range total unavailable — partial or unavailable buckets excluded; no partial total.
    Effective coverage (requested ∩ aggregate retention ∩ snapshot) for [1970-01-01T00:00:00.000Z, 1970-01-01T01:00:00.000Z): [1970-01-01T00:00:00.001Z, 1970-01-01T01:00:00.000Z)
    ");
}

#[test]
fn accounting_inspect_range_coverage_names_intersection_for_request_and_retention_clips() {
    let day_ms = 86_400_000;
    for (requested_start, aggregate_floor, effective_start) in [(1, 0, 1), (0, 1, day_ms)] {
        let pages = range_pages(
            InspectionRange {
                start_ms: requested_start,
                end_ms: 31 * day_ms,
                grouping: InspectionGrouping::Month,
            },
            Some(aggregate_floor),
            31 * day_ms,
            vec![codex_state::accounting::InspectionBucket {
                start_ms: 0,
                end_ms: 31 * day_ms,
                effective: Some((effective_start, 31 * day_ms)),
                partial: true,
                days: vec![InspectionDay::DetailUnavailable {
                    coverage: RetentionCoverage {
                        completed_as_of_ms: 31 * day_ms,
                        detail_expired_through_ms: None,
                        aggregate_day_floor: aggregate_floor,
                        oldest_recorded_day: Some(aggregate_floor),
                    },
                    read_at_ms: 31 * day_ms,
                    compact: false,
                }],
            }],
        );
        let bounds = interval(0, 31 * day_ms);
        let effective = interval(effective_start, 31 * day_ms);
        assert!(pages[0].text.contains(&format!(
            "Effective coverage (requested ∩ aggregate retention ∩ snapshot) for {bounds}: {effective}"
        )));
        let bucket = &pages[pages[0].links[0].1];
        assert!(bucket.text.contains(&format!(
            "Bucket: {bounds}; effective coverage (requested ∩ aggregate retention ∩ snapshot): {effective}"
        )));
        assert!(
            !pages
                .iter()
                .flat_map(|p| &p.text)
                .any(|s| s.contains("effective aggregate retention coverage")
                    || s.contains("Effective aggregate retention coverage"))
        );
        assert!(
            bucket
                .text
                .iter()
                .any(|s| s == "Partial bucket — excluded from totals")
        );
    }
}

#[test]
fn accounting_inspect_range_bucket_ancestry_counts_have_explicit_scopes() {
    let mut buckets = Vec::new();
    for index in 0..2 {
        let InspectionDay::Ready(mut view) = breakdown_packet() else {
            panic!()
        };
        view.unknown_parent_unavailable_threads = (index + 2) as usize;
        buckets.push(codex_state::accounting::InspectionBucket {
            start_ms: index * 3_600_000,
            end_ms: (index + 1) * 3_600_000,
            effective: Some((index * 3_600_000, (index + 1) * 3_600_000)),
            partial: false,
            days: vec![InspectionDay::Ready(view)],
        });
    }
    let pages = range_pages(
        InspectionRange {
            start_ms: 0,
            end_ms: 7_200_000,
            grouping: InspectionGrouping::Hour,
        },
        None,
        7_200_000,
        buckets,
    );
    let bucket = &pages[pages[0].links[0].1];
    let counts = bucket
        .text
        .iter()
        .filter(|s| s.contains("Unknown parent population:") || s.contains("Unresolved ancestry:"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        counts,
        vec![
            "Range: Unknown parent population: 2 inspectable attempts, excluded from range total",
            "Range: Unresolved ancestry: 5 thread-slice entries have unavailable detail; their costs and retention coverage are unknown and excluded from this root.",
            "Bucket: Unresolved ancestry: 2 thread-day entries have unavailable detail; their costs and retention coverage are unknown and excluded from this root.",
            "Bucket: Unknown parent population: 1 attempts, excluded from root total",
        ]
    );
}

#[test]
fn accounting_inspect_range_aggregate_coverage_does_not_claim_raw_detail() {
    let day = InspectionDay::DetailUnavailable {
        coverage: RetentionCoverage {
            completed_as_of_ms: 90 * 86_400_000 + 1_800_000,
            detail_expired_through_ms: Some(1_800_000),
            aggregate_day_floor: 0,
            oldest_recorded_day: Some(0),
        },
        read_at_ms: 90 * 86_400_000 + 1_800_000,
        compact: true,
    };
    let pages = inspection_pages(Ok(range_packet(false, day)));
    let bucket = &pages[pages[0].links[0].1];
    assert_eq!(bucket.title, "Bucket unavailable");
    assert!(bucket.text.iter().any(|s| s == "Whole bucket within aggregate retention coverage; detail availability checked separately"));
    assert!(!pages.iter().flat_map(|p| &p.text).any(|s| s.contains('$')));
    insta::assert_snapshot!(
        bucket.text.iter().filter(|s| s.starts_with("Bucket:") || s.starts_with("Whole bucket")).cloned().collect::<Vec<_>>().join("\n"),
        @"
    Bucket: [1970-01-01T00:00:00.000Z, 1970-01-01T01:00:00.000Z); effective coverage (requested ∩ aggregate retention ∩ snapshot): [1970-01-01T00:00:00.000Z, 1970-01-01T01:00:00.000Z)
    Whole bucket within aggregate retention coverage; detail availability checked separately
    "
    );
}

#[test]
fn accounting_inspect_range_overview_discloses_unknown_population_before_total() {
    let mut buckets = Vec::new();
    for index in 0..2 {
        let InspectionDay::Ready(mut view) = breakdown_packet() else {
            panic!()
        };
        view.unknown_parent_unavailable_threads = (index + 2) as usize;
        buckets.push(codex_state::accounting::InspectionBucket {
            start_ms: index * 3_600_000,
            end_ms: (index + 1) * 3_600_000,
            effective: Some((index * 3_600_000, (index + 1) * 3_600_000)),
            partial: false,
            days: vec![InspectionDay::Ready(view)],
        });
    }
    let pages = range_pages(
        InspectionRange {
            start_ms: 0,
            end_ms: 2 * 3_600_000,
            grouping: InspectionGrouping::Hour,
        },
        None,
        2 * 3_600_000,
        buckets,
    );
    let text = &pages[0].text;
    let total = text.iter().position(|s| s.contains("$0.000014")).unwrap();
    let count =
        "Range: Unknown parent population: 2 inspectable attempts, excluded from range total";
    let note = "Range: Unresolved ancestry: 5 thread-slice entries have unavailable detail; their costs and retention coverage are unknown and excluded from this root.";
    assert!(text.iter().position(|s| s == count).unwrap() < total);
    assert!(text.iter().position(|s| s == note).unwrap() < total);
    insta::assert_snapshot!(
        text.iter().filter(|s| s.starts_with("Range: Unknown parent population:") || s.starts_with("Range: Unresolved ancestry:")).cloned().collect::<Vec<_>>().join("\n"),
        @"
    Range: Unknown parent population: 2 inspectable attempts, excluded from range total
    Range: Unresolved ancestry: 5 thread-slice entries have unavailable detail; their costs and retention coverage are unknown and excluded from this root.
    "
    );
}

#[test]
fn accounting_inspect_range_breakdowns_and_navigation_reconcile() {
    let pages = inspection_pages(Ok(range_packet(false, breakdown_packet())));
    assert!(pages[0].text.iter().any(|t| t.contains("$0.000007")));
    let bucket = pages[0].links[0].1;
    assert_eq!(pages[bucket].parent, Some(0));
    for p in &pages[1..] {
        assert!(p.text.iter().any(|s| s.starts_with("Requested:")));
        assert!(p.text.iter().any(|s| s.starts_with("Bucket:")
            && s.contains("effective coverage (requested ∩ aggregate retention ∩ snapshot):")));
        assert!(
            !p.text
                .iter()
                .any(|s| s.starts_with("UTC admission interval:"))
        );
        for (_, target) in &p.links {
            assert!(*target > bucket && *target < pages.len());
        }
    }
    for (title, amount) in [
        ("Root's own attempts", "0.000001"),
        ("Descendant attempts", "0.000006"),
        ("Unknown parent population", "$0.000008"),
        (
            "Provider: unknown (attribution absent); Model: unknown (attribution absent)",
            "0.000004",
        ),
    ] {
        let p = pages.iter().find(|p| p.title == title).unwrap();
        assert!(p.text.iter().any(|s| s.contains(amount)));
    }
}

#[test]
fn accounting_inspect_range_week_and_month_merge_exact_attempts() {
    for grouping in [InspectionGrouping::Week, InspectionGrouping::Month] {
        let start = 4 * 86_400_000; // Monday, 1970-01-05.
        let requested = InspectionRange {
            start_ms: start,
            end_ms: start + 1,
            grouping,
        };
        let (start_ms, end_ms) = requested.bucket(start).unwrap();
        let mut days = Vec::new();
        for index in 0..2 {
            let InspectionDay::Ready(mut view) = breakdown_packet() else {
                panic!()
            };
            view.utc_day = start_ms / 86_400_000 + index;
            view.read_at_ms = end_ms;
            view.coverage.completed_as_of_ms = end_ms;
            for q in view
                .requests
                .values_mut()
                .chain(view.unknown_parent_requests.values_mut())
                .flatten()
            {
                q.attempt.attempt_id =
                    Uuid::from_u128(q.attempt.attempt_id.as_u128() + index as u128 * 100);
                q.attempt.dispatched_at_ms = (start_ms + index * 86_400_000).try_into().unwrap();
            }
            days.push(InspectionDay::Ready(view));
        }
        let pages = inspection_pages(Ok(InspectionDay::Range {
            requested: InspectionRange {
                start_ms,
                end_ms,
                grouping,
            },
            oldest_aggregate_day: None,
            read_at_ms: end_ms,
            buckets: vec![codex_state::accounting::InspectionBucket {
                start_ms,
                end_ms,
                effective: Some((start_ms, end_ms)),
                partial: false,
                days,
            }],
        }));
        assert!(pages[0].text.iter().any(|s| s.contains("$0.000014")));
        for (title, amount) in [
            ("Root's own attempts", "0.000002"),
            ("Descendant attempts", "0.000012"),
            ("Unknown parent population", "$0.000016"),
        ] {
            assert!(
                pages
                    .iter()
                    .find(|p| p.title == title)
                    .unwrap()
                    .text
                    .iter()
                    .any(|s| s.contains(amount))
            );
        }
        assert_eq!(
            pages
                .iter()
                .filter(|p| p.title == "Attempt, components and original price")
                .count(),
            6
        );
    }
}

#[test]
fn accounting_inspect_range_unavailable_precision_and_freshness() {
    for state in [
        InspectionDay::CheckpointLag,
        InspectionDay::NeedsRefresh,
        InspectionDay::DetailUnavailable {
            coverage: RetentionCoverage {
                completed_as_of_ms: 90 * 86_400_000,
                detail_expired_through_ms: Some(0),
                aggregate_day_floor: 0,
                oldest_recorded_day: Some(0),
            },
            read_at_ms: 90 * 86_400_000,
            compact: true,
        },
    ] {
        let expected = match state {
            InspectionDay::CheckpointLag => "Snapshot is not current",
            InspectionDay::NeedsRefresh => "stored contributions need refresh",
            _ => {
                "Whole UTC-day bounds offered: [1970-01-01T00:00:00.000Z, 1970-01-02T00:00:00.000Z)"
            }
        };
        let pages = inspection_pages(Ok(range_packet(false, state)));
        let text = pages
            .iter()
            .flat_map(|p| &p.text)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains(expected));
        assert!(!text.contains('$'));
    }
    let pages = inspection_pages(Ok(range_packet(false, packet())));
    assert!(
        pages[1..]
            .iter()
            .all(|p| p.text.iter().any(|s| s.contains("Snapshot is not current")))
    );
}

#[tokio::test]
async fn accounting_inspect_range_command_refresh_and_refusal() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    let today = NaiveDate::from_ymd_opt(2026, 9, 16).unwrap();
    for grouping in ["hour", "day", "week", "month"] {
        chat.open_accounting_command(&format!("requests 2026-09-01 2026-09-02 {grouping}"), today);
        let AppEvent::LoadAccountingInspector {
            generation, range, ..
        } = rx.try_recv().unwrap()
        else {
            panic!()
        };
        let range = range.unwrap();
        assert_eq!(format!("{:?}", range.grouping).to_lowercase(), grouping);
        chat.refresh_accounting_inspector(generation);
        let AppEvent::LoadAccountingInspector {
            range: refreshed,
            generation: next,
            ..
        } = rx.try_recv().unwrap()
        else {
            panic!()
        };
        assert_eq!(refreshed, Some(range));
        assert_ne!(next, generation);
    }
    for (args, message) in [
        ("requests 2026-09-02 2026-09-01 hour", "reversed"),
        ("requests 2026-09-01 2026-09-01 day", "empty"),
        ("requests bad 2026-09-01 day", "Range refused"),
        (
            "requests 2026-09-01T00:00:00.0001Z 2026-09-02 hour",
            "finer than milliseconds",
        ),
        ("requests 2026-09-01T00:00:00-07:00 2026-09-02 day", "UTC Z"),
        ("requests 2026-09-01 2026-09-02 year", "grouping must"),
    ] {
        chat.open_accounting_command(args, today);
        let AppEvent::InsertHistoryCell(cell) = rx.try_recv().unwrap() else {
            panic!()
        };
        let text = cell
            .display_lines(120)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains(message), "{text}");
        assert!(rx.try_recv().is_err());
    }
}

#[test]
fn accounting_inspect_partial_and_unknown_copy() {
    insta::assert_snapshot!(estimate(decimal("0.00018"), 1, 1).join("\n"), @"
    Known estimated token cost: $0.000180 + unknown costs
    Full recorded estimate: unavailable (1 of 1 attempts incomplete)
    ");
    let pages = inspection_pages(Ok(packet()));
    let summary = pages[0].text.join("\n");
    for label in METRICS {
        assert!(summary.contains(label));
    }
    assert!(summary.contains("100 known + unknown in 0 attempts"));
    assert!(summary.contains("Billed cost: unavailable"));
    assert!(summary.contains("Collection coverage: unknown"));
    assert!(
        attempt_text(&quote())
            .join("\n")
            .contains("Cache write: unknown — no retained numeric evidence")
    );
}

#[test]
fn accounting_inspect_unpriced_zero_and_missing_rate() {
    insta::assert_snapshot!(estimate(Decimal::default(), 1, 1).join("\n"), @"
    Estimated token cost: unknown
    Known estimated token cost: $0.000000 + unknown costs
    Full recorded estimate: unavailable (1 of 1 attempts incomplete)
    ");
    assert_eq!(
        estimate(Decimal::default(), 0, 1),
        vec!["Estimated token cost for recorded attempts: $0.000000"]
    );
    let mut q = quote();
    q.usage.write = Some(0);
    q.buckets[1] = BucketQuote::MissingRate;
    let text = attempt_text(&q).join("\n");
    assert!(text.contains("Cache write: 0"));
    assert!(text.contains("Cache read cost: unknown — rate unavailable"));
    assert!(text.contains("Price: unavailable — no dispatch-time price snapshot"));
}

#[test]
fn accounting_inspect_exact_rounding_reconciliation() {
    for (exact_value, displayed) in [
        (
            "0.0000005",
            "$0.000000 (rounded) (nonzero, less than $0.000001)",
        ),
        (
            "0.0000006",
            "$0.000001 (rounded) (nonzero, less than $0.000001)",
        ),
        ("0.0000015", "$0.000002 (rounded)"),
        ("0.0000025", "$0.000002 (rounded)"),
    ] {
        assert_eq!(money(decimal(exact_value)), displayed);
        assert_eq!(exact(decimal(exact_value)), exact_value);
    }
    // Two exact half-micro components round only after their stored subtotal.
    let mut q = quote();
    q.buckets = [BucketQuote::Priced(decimal("0.0000005")); 4];
    q.known_subtotal = decimal("0.000002");
    let text = attempt_text(&q).join("\n");
    assert_eq!(text.matches("exact USD 0.0000005").count(), 4);
    assert!(text.contains("Known subtotal exact USD: 0.000002"));
}

#[test]
fn accounting_inspect_request_attempt_price_detail() {
    let mut q = quote();
    q.snapshot = Some(serde_json::from_value(serde_json::json!({
        "id":Uuid::from_u128(6),"provider":"synthetic","model":"synthetic-model","scope":Uuid::from_u128(5),
        "currency":"USD","unit":"PerMillionTokens","rates":{"noncached":"1","read":null,"write":"2","output":"4"},
        "source_reference":Uuid::from_u128(7),"source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":1000
    })).unwrap());
    let text = attempt_text(&q).join("\n");
    for id in 1..=7 {
        assert!(text.contains(&Uuid::from_u128(id).to_string()));
    }
    for required in [
        "Turn: synthetic",
        "Provider: synthetic",
        "Model: synthetic-model",
        "Opaque scope:",
        "Price source: ProviderPublished",
        "Price effective interval: [0, 1000)",
        "Rate unavailable for Cache read",
        "Literal wire/endpoint",
        "Completion/billing status: not recorded",
    ] {
        assert!(text.contains(required), "{required}");
    }
}

#[tokio::test]
async fn accounting_inspect_narrow_and_long_fields() {
    for width in [40, 80] {
        let (mut chat, _rx, _ops) = make_chatwidget_manual(None).await;
        let mut q = quote();
        q.attempt.provider = format!("provider\u{1b}[31m{}", "LONG".repeat(30));
        let page = InspectorPage {
            title: "Attempt".into(),
            text: attempt_text(&q),
            links: vec![],
            parent: None,
            selected: Arc::default(),
        };
        let inspector = Inspector {
            generation: Uuid::new_v4(),
            thread: None,
            day: 0,
            range: None,
            pages: vec![page],
            page: 0,
            alive: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        };
        let params = inspector.params();
        let names: Vec<_> = params.items.iter().map(|i| i.name.clone()).collect();
        assert!(names.iter().all(|s| !s.chars().any(char::is_control)));
        chat.bottom_pane.show_selection_view(params);
        // Every selectable fragment can actually be reached in a small viewport.
        for name in names {
            let screen = render_bottom_popup_with_height(&chat, width, 12);
            assert!(
                screen.contains(&name),
                "{name:?} missing at {width}: {screen}"
            );
            chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        }
    }
}

#[tokio::test]
async fn accounting_inspect_empty_day_after_checkpoint_renders_lag() -> anyhow::Result<()> {
    use codex_state::accounting::AccountingStore;

    let home = tempfile::tempdir()?;
    let owner = ThreadId::new();
    let runtime = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path().to_path_buf())?,
        ),
        "synthetic".into(),
    )
    .await?;
    let metadata = codex_state::ThreadMetadataBuilder::new(
        owner,
        home.path().join("synthetic.jsonl"),
        chrono::Utc::now(),
        codex_protocol::protocol::SessionSource::Cli,
    );
    runtime.upsert_thread(&metadata.build("synthetic")).await?;
    AccountingStore::open(&runtime, 0).await?;
    let empty = AccountingStore::inspect_day(&runtime, owner, 0, 0).await?;
    let InspectionDay::Ready(view) = &empty else {
        panic!("{empty:?}");
    };
    assert_eq!(view.totals, DayTotals::default());
    assert!(view.requests.is_empty());
    assert_eq!(view.coverage.completed_as_of_ms, 0);
    assert_eq!(
        inspection_pages(Ok(empty))[0].text[0],
        "No recorded attempts in this day; collection coverage unknown."
    );

    // The next UTC day has no requests and opening the inspector must not maintain it.
    for _ in 0..2 {
        let lagged = AccountingStore::inspect_day(&runtime, owner, 1, 86_400_000).await?;
        let text = inspection_pages(Ok(lagged))[0].text.join("\n");
        assert!(
            text.contains("Snapshot is not current; newer activity is unverified"),
            "{text}"
        );
        assert!(
            !text.contains("stored contributions need refresh"),
            "{text}"
        );
        assert!(!text.contains('$'), "{text}");
    }
    runtime.close().await;
    Ok(())
}

// Admit synthetic evidence, then model a late raw import and interrupted
// estimate/contribution persistence without adding a production mutation API.
async fn maintenance_inspection(mutation: &str) -> anyhow::Result<InspectionDay> {
    use codex_state::accounting::{AccountingStore, RetainedDay};

    const CHECKPOINT: i64 = 100 * 86_400_000;
    let home = tempfile::tempdir()?;
    let runtime = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path().to_path_buf())?,
        ),
        "synthetic".into(),
    )
    .await?;
    let mut recent = quote().attempt;
    recent.retry_of = None;
    recent.dispatched_at_ms = CHECKPOINT.try_into()?;
    let owner = recent.thread_id;
    let metadata = codex_state::ThreadMetadataBuilder::new(
        owner,
        home.path().join("synthetic.jsonl"),
        chrono::Utc::now(),
        codex_protocol::protocol::SessionSource::Cli,
    );
    runtime.upsert_thread(&metadata.build("synthetic")).await?;
    let store = AccountingStore::open(&runtime, CHECKPOINT).await?;
    store.admit(owner, &recent, &[], CHECKPOINT).await?;
    let mut late = recent.clone();
    late.attempt_id = Uuid::from_u128(8);
    late.request_id = Uuid::from_u128(9);
    store.admit(owner, &late, &[], CHECKPOINT).await?;
    let mutation = format!(
        "UPDATE draft_accounting_attempts SET payload = json_set(payload, '$.dispatched_at_ms', 0) WHERE attempt_id = '{id}';
         UPDATE draft_accounting_estimates SET payload = json_set(payload, '$.attempt.dispatched_at_ms', 0) WHERE attempt_id = '{id}';
         UPDATE draft_accounting_contributions SET utc_day = 0 WHERE attempt_id = '{id}';
         {mutation}",
        id = late.attempt_id,
    );
    // TUI has no SQL dependency. Python is already required by the guarded test
    // runner; its standard sqlite3 module touches only this disposable fixture.
    let output = tokio::process::Command::new(if cfg!(windows) { "python" } else { "python3" })
        .arg("-c")
        .arg("import sqlite3, sys; c = sqlite3.connect(sys.argv[1]); c.executescript(sys.argv[2]); c.close()")
        .arg(runtime.sqlite().state_db_path())
        .arg(mutation)
        .output()
        .await?;
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        store.read_day(owner, /*utc_day*/ 100, CHECKPOINT).await?,
        RetainedDay::NeedsMaintenance {
            completed_as_of_ms: CHECKPOINT
        }
    );
    let result = AccountingStore::inspect_day(&runtime, owner, /*utc_day*/ 100, CHECKPOINT).await?;
    assert_eq!(
        AccountingStore::inspect_day(&runtime, owner, /*utc_day*/ 100, CHECKPOINT).await?,
        result
    );
    runtime.close().await;
    Ok(result)
}

#[tokio::test]
async fn accounting_inspect_maintenance_with_stale_raw_renders_refresh() -> anyhow::Result<()> {
    for mutation in [
        "DELETE FROM draft_accounting_contributions WHERE utc_day = 100",
        "DELETE FROM draft_accounting_contributions WHERE utc_day = 100; DELETE FROM draft_accounting_estimates WHERE attempt_id = '00000000-0000-0000-0000-000000000001'",
    ] {
        let result = maintenance_inspection(mutation).await?;
        let text = inspection_pages(Ok(result))[0].text.join("\n");
        insta::allow_duplicates! {
            insta::assert_snapshot!(text, @"
            Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
            Billed cost: unavailable — no settlement evidence
            Logical requests may have attempts on other days; this UTC day is not their complete lifetime.
            Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.
            ");
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_maintenance_with_healthy_raw_renders_lag() -> anyhow::Result<()> {
    let result = maintenance_inspection("").await?;
    let text = inspection_pages(Ok(result))[0].text.join("\n");
    insta::assert_snapshot!(text, @"
    Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
    Billed cost: unavailable — no settlement evidence
    Logical requests may have attempts on other days; this UTC day is not their complete lifetime.
    Snapshot is not current; newer activity is unverified
    ");
    Ok(())
}

#[test]
fn accounting_inspect_availability_state_snapshots() {
    let states = [
        InspectionDay::Absent,
        InspectionDay::MissingThread,
        InspectionDay::CheckpointLag,
        InspectionDay::NeedsRefresh,
        InspectionDay::TooLarge,
    ];
    let text: Vec<_> = states
        .into_iter()
        .map(|s| inspection_pages(Ok(s))[0].text.last().unwrap().clone())
        .collect();
    insta::assert_snapshot!(text.join("\n"), @"
    Unavailable — accounting ledger not installed. Collection remains off.
    Unavailable — native thread no longer exists.
    Snapshot is not current; newer activity is unverified
    Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.
    Range too large for this inspector. No total shown.
    ");
    let InspectionDay::Ready(mut empty) = packet() else {
        unreachable!()
    };
    empty.totals = DayTotals::default();
    empty.requests.clear();
    assert!(
        inspection_pages(Ok(InspectionDay::Ready(empty)))[0].text[0]
            .contains("No recorded attempts")
    );
    let InspectionDay::Ready(view) = packet() else {
        unreachable!()
    };
    let page = inspection_pages(Ok(InspectionDay::DetailUnavailable {
        coverage: view.coverage,
        read_at_ms: 90 * 86_400_000,
        compact: true,
    }));
    let text = page[0].text.join("\n");
    assert!(text.contains("compacted history lost request/provider attribution"));
    assert!(!text.contains('$'));
    for error in [
        "remote connection",
        "corrupt evidence",
        "inspection timed out",
    ] {
        assert!(
            inspection_pages(Err(error.into()))[0]
                .text
                .contains(&error.to_owned())
        );
    }
}

#[test]
fn accounting_inspect_coverage_never_claims_run_complete() {
    let InspectionDay::Ready(mut view) = packet() else {
        unreachable!()
    };
    view.totals.unknown_estimates = 0;
    let text = inspection_pages(Ok(InspectionDay::Ready(view)))[0]
        .text
        .join("\n");
    assert!(text.starts_with("Estimated token cost for recorded attempts:"));
    for caveat in [
        "Collection coverage: unknown",
        "resolved descendants only",
        "Billed cost: unavailable",
        "other days",
        "Snapshot is not current; newer activity is unverified",
    ] {
        assert!(text.contains(caveat));
    }
}

#[tokio::test]
async fn accounting_inspect_snapshot_navigation_roundtrip() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.open_accounting_inspector(0);
    let AppEvent::LoadAccountingInspector {
        generation,
        thread,
        day,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!()
    };
    chat.finish_accounting_inspector(generation, thread, day, Ok(packet()));
    let original = chat.accounting_inspector.as_ref().unwrap().pages[0]
        .text
        .clone();
    for page in [1, 2, 1, 0] {
        chat.navigate_accounting_inspector(generation, page);
        assert_eq!(chat.accounting_inspector.as_ref().unwrap().page, page);
        chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }
    assert_eq!(
        chat.accounting_inspector.as_ref().unwrap().pages[0].text,
        original
    );
    chat.navigate_accounting_inspector(generation, 2);
    chat.navigate_accounting_inspector(generation, 0);
    assert_eq!(
        chat.bottom_pane
            .selected_index_for_active_view(INSPECTOR_VIEW),
        Some(1)
    );
    assert!(render_bottom_popup(&chat, 40).contains("Recorded requests"));
}

fn breakdown_packet() -> InspectionDay {
    let InspectionDay::Ready(mut view) = packet() else {
        unreachable!()
    };
    view.requests.clear();
    // Renderer boundary fixture: the installed ledger currently requires nonempty
    // attribution; missing strings here exercise the explicit unknown fallback.
    for (id, micros, provider, model) in [
        (1, 1, "alpha", "one"),
        (2, 2, "alpha", "two"),
        (3, 4, "", ""),
        (4, 8, "beta", "three"),
    ] {
        let mut q = quote();
        q.attempt.attempt_id = Uuid::from_u128(id);
        q.attempt.request_id = Uuid::from_u128(100 + id);
        q.attempt.thread_id = if id == 1 {
            view.owner
        } else {
            ThreadId::from_string(&Uuid::from_u128(200 + id).to_string()).unwrap()
        };
        q.attempt.provider = provider.into();
        q.attempt.model = model.into();
        q.known_subtotal = decimal(&format!("0.00000{micros}"));
        q.all_buckets_priced = Some(q.known_subtotal);
        if id == 4 {
            view.unknown_parent_requests
                .insert(q.attempt.request_id, vec![q]);
        } else {
            view.requests.insert(q.attempt.request_id, vec![q]);
        }
    }
    view.totals = DayTotals::from_quotes(view.requests.values().flatten()).unwrap();
    view.own_totals = DayTotals::from_quotes(
        view.requests
            .values()
            .flatten()
            .filter(|q| q.attempt.thread_id == view.owner),
    )
    .unwrap();
    view.descendant_totals = DayTotals::from_quotes(
        view.requests
            .values()
            .flatten()
            .filter(|q| q.attempt.thread_id != view.owner),
    )
    .unwrap();
    view.unknown_parent_totals =
        DayTotals::from_quotes(view.unknown_parent_requests.values().flatten()).unwrap();
    InspectionDay::Ready(view)
}

#[test]
fn accounting_inspect_rendered_tree_and_provider_model_reconcile() {
    let pages = inspection_pages(Ok(breakdown_packet()));
    let group = |title: &str| pages.iter().find(|p| p.title == title).unwrap();
    let exact_micros = |p: &InspectorPage| -> u64 {
        let line = p
            .text
            .iter()
            .find_map(|s| s.strip_prefix("Known estimate exact USD: "))
            .unwrap();
        // Independent integer parsing of the rendered fixture values.
        line.strip_prefix("0.").unwrap().parse::<u64>().unwrap() * 10_u64.pow(8 - line.len() as u32)
    };
    assert_eq!(exact_micros(group("Root's own attempts")), 1);
    assert_eq!(exact_micros(group("Descendant attempts")), 6);
    assert!(
        pages[0]
            .text
            .contains(&"Known subtotal exact USD: 0.000007".into())
    );
    let provider_pages: Vec<_> = pages
        .iter()
        .filter(|p| p.title.starts_with("Provider:"))
        .collect();
    assert_eq!(
        provider_pages.iter().map(|p| exact_micros(p)).sum::<u64>(),
        7
    );
    assert_eq!(
        provider_pages.iter().map(|p| p.links.len()).sum::<usize>(),
        3
    );
    assert_eq!(
        exact_micros(group(
            "Provider: unknown (attribution absent); Model: unknown (attribution absent)"
        )),
        4
    );
    let unknown = group("Unknown parent population");
    assert_eq!(unknown.links.len(), 1);
    assert!(unknown.text.iter().any(|s| s.contains("$0.000008")));
    assert!(
        pages[unknown.links[0].1]
            .text
            .contains(&format!("Attempt: {}", Uuid::from_u128(4)))
    );
    for group in provider_pages {
        for (_, request) in &group.links {
            assert_eq!(pages[*request].title, "Logical request");
            assert_eq!(
                pages[pages[*request].links[0].1].title,
                "Attempt, components and original price"
            );
        }
    }
    insta::assert_snapshot!(pages.iter().filter(|p| p.title.starts_with("Provider:") || p.title.ends_with("attempts")).map(|p| format!("{}\n{}", p.title, p.text.iter().filter(|s| s.starts_with("Estimated token cost") || s.starts_with("Known estimate exact") || s.starts_with("Recorded attempts:")).cloned().collect::<Vec<_>>().join("\n"))).collect::<Vec<_>>().join("\n"), @"Root's own attempts\nEstimated token cost for recorded attempts: $0.000001\nRecorded attempts: 1\nKnown estimate exact USD: 0.000001\nDescendant attempts\nEstimated token cost for recorded attempts: $0.000006\nRecorded attempts: 2\nKnown estimate exact USD: 0.000006\nProvider: unknown (attribution absent); Model: unknown (attribution absent)\nEstimated token cost for recorded attempts: $0.000004\nRecorded attempts: 1\nKnown estimate exact USD: 0.000004\nProvider: alpha; Model: one\nEstimated token cost for recorded attempts: $0.000001\nRecorded attempts: 1\nKnown estimate exact USD: 0.000001\nProvider: alpha; Model: two\nEstimated token cost for recorded attempts: $0.000002\nRecorded attempts: 1\nKnown estimate exact USD: 0.000002");
}

#[test]
fn accounting_inspect_group_partial_price_stays_unknown() {
    let InspectionDay::Ready(mut view) = breakdown_packet() else {
        unreachable!()
    };
    view.requests.get_mut(&Uuid::from_u128(102)).unwrap()[0].all_buckets_priced = None;
    view.totals = DayTotals::from_quotes(view.requests.values().flatten()).unwrap();
    view.descendant_totals.unknown_estimates = 1;
    let pages = inspection_pages(Ok(InspectionDay::Ready(view)));
    for title in ["Descendant attempts", "Provider: alpha; Model: two"] {
        let page = pages.iter().find(|p| p.title == title).unwrap();
        assert!(
            page.text.iter().any(|s| s.contains("+ unknown costs")),
            "{title}"
        );
        assert!(
            page.text
                .iter()
                .any(|s| s.starts_with("Full recorded estimate: unavailable")),
            "{title}"
        );
    }
}

#[test]
fn accounting_inspect_unknown_unavailable_note() {
    let InspectionDay::Ready(mut view) = packet() else {
        unreachable!()
    };
    view.unknown_parent_unavailable_threads = 3;
    let pages = inspection_pages(Ok(InspectionDay::Ready(view)));
    let unknown = pages
        .iter()
        .find(|p| p.title == "Unknown parent population")
        .unwrap();
    let note = "Unresolved ancestry: 3 threads have unavailable day detail; their costs and retention coverage are unknown and excluded from this root.";
    assert!(pages[0].text.iter().any(|s| s == note));
    assert!(unknown.text.iter().any(|s| s == note));
    insta::assert_snapshot!(unknown.text.iter().filter(|s| s.starts_with("Unresolved ancestry:") || s.starts_with("Estimates below")).cloned().collect::<Vec<_>>().join("\n"), @"
    Unresolved ancestry: 3 threads have unavailable day detail; their costs and retention coverage are unknown and excluded from this root.
    Estimates below cover inspectable attempts only; unavailable threads may have additional unknown costs.
    ");
}

#[test]
fn accounting_inspect_estimate_only_never_invents_billed_or_difference() {
    let pages = inspection_pages(Ok(breakdown_packet()));
    for page in &pages {
        assert!(
            page.text
                .contains(&"Billed cost: unavailable — no settlement evidence".into())
        );
        assert!(page.text.contains(
            &"Estimate versus billed difference: unknown — no settlement evidence".into()
        ));
        assert!(!page.text.iter().any(|s| s.starts_with("Billed cost: $")));
    }
    assert!(
        pages[0]
            .text
            .contains(&"Estimated token cost for recorded attempts: $0.000007".into())
    );
    let mut empty = quote();
    empty.known_subtotal = Decimal::default();
    empty.all_buckets_priced = None;
    assert!(attempt_text(&empty).contains(&"Estimated token cost: unknown".into()));
}

#[test]
fn accounting_inspect_breakdown_freshness_and_empty_attribution() {
    let pages = inspection_pages(Ok(packet()));
    for page in &pages {
        assert!(
            page.text
                .contains(&"Snapshot is not current; newer activity is unverified".into()),
            "{}",
            page.title
        );
    }
    let unknown = pages
        .iter()
        .find(|p| p.title == "Unknown provider/model attribution")
        .unwrap();
    assert!(
        unknown
            .text
            .iter()
            .any(|s| s.contains("No recorded attempts"))
    );
    assert!(!unknown.text.iter().any(|s| s.contains('$')));
    for state in [
        InspectionDay::NeedsRefresh,
        InspectionDay::CheckpointLag,
        InspectionDay::TooLarge,
    ] {
        let pages = inspection_pages(Ok(state));
        assert_eq!(pages.len(), 1);
        assert!(pages[0].links.is_empty());
        assert!(!pages[0].text.iter().any(|s| s.contains('$')));
    }
}

#[tokio::test]
async fn accounting_inspect_breakdown_navigation_reuses_packet() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.open_accounting_inspector(0);
    let AppEvent::LoadAccountingInspector {
        generation,
        thread,
        day,
        ..
    } = rx.try_recv().unwrap()
    else {
        panic!()
    };
    chat.finish_accounting_inspector(generation, thread, day, Ok(breakdown_packet()));
    let links = chat.accounting_inspector.as_ref().unwrap().pages[0]
        .links
        .clone();
    for (_, target) in links {
        chat.navigate_accounting_inspector(generation, target);
        assert_eq!(chat.accounting_inspector.as_ref().unwrap().page, target);
        chat.navigate_accounting_inspector(generation, 0);
    }
    assert!(!matches!(
        rx.try_recv(),
        Ok(AppEvent::LoadAccountingInspector { .. })
    ));
}

#[test]
fn loaded_state_freezes_chart_anchor_date_at_completion() {
    let state = Arc::new(RwLock::new(TokenActivityState::Loading));
    let handle = TokenActivityHandle {
        state: Arc::clone(&state),
    };
    let today =
        NaiveDate::from_ymd_opt(/*year*/ 2026, /*month*/ 5, /*day*/ 29).expect("valid date");

    handle.finish_with_today(
        Ok(GetAccountTokenUsageResponse {
            summary: AccountTokenUsageSummary {
                lifetime_tokens: None,
                peak_daily_tokens: None,
                longest_running_turn_sec: None,
                current_streak_days: None,
                longest_streak_days: None,
            },
            daily_usage_buckets: None,
        }),
        today,
    );

    let state = state.read().expect("token activity state poisoned");
    match &*state {
        TokenActivityState::Loaded {
            today: loaded_today,
            ..
        } => {
            assert_eq!(*loaded_today, today);
        }
        other => panic!("expected loaded state, got {other:?}"),
    }
}
