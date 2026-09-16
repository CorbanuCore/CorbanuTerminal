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
        requests: std::collections::BTreeMap::from([(q.attempt.request_id, vec![q])]),
    })
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
            Collection coverage: unknown; recorded attempts only. Descendants excluded.
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
    Collection coverage: unknown; recorded attempts only. Descendants excluded.
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
        "Descendants excluded",
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
