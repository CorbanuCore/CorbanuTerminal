use super::*;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::Attempt;
use codex_state::accounting::Dialect;
use codex_state::accounting::Observation;
use codex_state::accounting::Patch;
use codex_state::accounting::Snapshot;
use pretty_assertions::assert_eq;

const DAY: i64 = 86_400_000;
const HOUR: i64 = 3_600_000;
const START: i64 = 100 * DAY;

// The independent numeric oracle fixes a = 12000 nanoUSD and b = 34000
// nanoUSD. At a synthetic $1/million rate, 12 and 34 tokens cost exactly
// $0.000012 and $0.000034; the expected sum is the literal $0.000046.
async fn ledger(
    rows: &[(i64, i64)],
) -> anyhow::Result<(tempfile::TempDir, Arc<codex_state::StateRuntime>, ThreadId)> {
    let home = tempfile::tempdir()?;
    let runtime = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::from_sqlite_home(
            codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path().to_path_buf())?,
        ),
        "synthetic".into(),
    )
    .await?;
    let owner = ThreadId::from_string(&Uuid::from_u128(7).to_string())?;
    let metadata = codex_state::ThreadMetadataBuilder::new(
        owner,
        home.path().join("synthetic.jsonl"),
        chrono::DateTime::from_timestamp(100 * 86_400, 0).unwrap(),
        codex_protocol::protocol::SessionSource::Cli,
    );
    runtime.upsert_thread(&metadata.build("synthetic")).await?;
    let price: Snapshot = serde_json::from_value(serde_json::json!({
        "id":Uuid::from_u128(1000), "provider":"synthetic", "model":"fixture",
        "scope":Uuid::nil(), "currency":"USD", "unit":"PerMillionTokens",
        "rates":{"noncached":"1","read":"1","write":"1","output":"1"},
        "source_reference":Uuid::from_u128(1001), "source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null
    }))?;
    let store = AccountingStore::open(&runtime, rows[0].0).await?;
    for (index, &(time, input)) in rows.iter().enumerate() {
        let id = index as u128 + 1;
        let attempt = Attempt {
            attempt_id: Uuid::from_u128(id),
            request_id: Uuid::from_u128(id + 100),
            thread_id: owner,
            turn: "synthetic".into(),
            retry_of: None,
            provider: "synthetic".into(),
            model: "fixture".into(),
            scope: Uuid::nil(),
            dialect: Dialect::NativeAnthropic,
            dispatched_at_ms: time.try_into()?,
            account: None,
        };
        store
            .admit(owner, &attempt, std::slice::from_ref(&price), time)
            .await?;
        store
            .observe(
                owner,
                &attempt,
                &[Observation {
                    revision: 1.try_into()?,
                    source: attempt.attempt_id,
                    sequence: 1.try_into()?,
                    patch: serde_json::from_value::<Patch>(serde_json::json!({
                        "input":input,"read":0,"write":0,"output":0
                    }))?,
                }],
                time,
            )
            .await?;
    }
    Ok((home, runtime, owner))
}

fn text(pages: &[InspectorPage]) -> String {
    pages
        .iter()
        .flat_map(|page| std::iter::once(&page.title).chain(&page.text))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

fn overview(pages: &[InspectorPage]) -> String {
    pages[0].text.join("\n")
}

async fn finished_bucket(
    grouping: InspectionGrouping,
    end: i64,
    read_at: i64,
) -> anyhow::Result<()> {
    let checkpoint = START + HOUR / 2;
    let (_home, runtime, owner) = ledger(&[(START, 12), (checkpoint, 34)]).await?;
    let requested = InspectionRange {
        start_ms: START,
        end_ms: end,
        grouping,
    };
    for read_at in [read_at, read_at + HOUR] {
        let result = AccountingStore::inspect_range(&runtime, owner, requested, read_at).await?;
        let InspectionDay::Range { buckets, .. } = &result else {
            panic!("expected native range: {result:#?}");
        };
        let InspectionDay::Ready(view) = &buckets[0].days[0] else {
            panic!("expected native ready detail: {result:#?}");
        };
        assert_eq!(
            (
                buckets[0].effective,
                view.coverage.completed_as_of_ms,
                view.requests
                    .values()
                    .flatten()
                    .map(|q| {
                        (
                            q.attempt.attempt_id,
                            serde_json::to_value(q.known_subtotal).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>(),
                serde_json::to_value(view.totals.known_usd)?,
                view.totals.unknown_estimates,
            ),
            (
                Some((START, checkpoint + 1)),
                checkpoint,
                vec![
                    (Uuid::from_u128(1), serde_json::json!("0.000012")),
                    (Uuid::from_u128(2), serde_json::json!("0.000034")),
                ],
                serde_json::json!("0.000046"),
                0,
            ),
            "native fixture must reconcile before testing presentation"
        );
        let pages = inspection_pages(Ok(result));
        let all = text(&pages);
        assert!(
            overview(&pages).contains("Estimated token cost for recorded attempts: $0.000046"),
            "{all}"
        );
        assert!(
            all.contains("Recorded through ledger checkpoint: 1970-04-11T00:30:00.000Z"),
            "{all}"
        );
        for absent in [
            "Partial bucket",
            "Range total unavailable",
            "In progress",
            NOT_CURRENT,
        ] {
            assert!(!all.contains(absent), "{absent}: {all}");
        }
        assert!(all.contains("Collection coverage: unknown"), "{all}");
        if grouping == InspectionGrouping::Day && read_at == START + DAY + HOUR {
            let bucket = &pages[pages[0].links[0].1];
            insta::assert_snapshot!(
                "accounting_checkpoint_finished_day",
                format!(
                    "{}\n{}\n\n{}\n{}",
                    pages[0].title,
                    overview(&pages),
                    bucket.title,
                    bucket.text.join("\n")
                )
            );
        }
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_finished_day_total_native_rows() -> anyhow::Result<()> {
    finished_bucket(InspectionGrouping::Day, START + DAY, START + DAY + HOUR).await
}

#[tokio::test]
async fn accounting_checkpoint_finished_hour_total_native_rows() -> anyhow::Result<()> {
    finished_bucket(InspectionGrouping::Hour, START + HOUR, START + 2 * HOUR).await
}

#[tokio::test]
async fn accounting_checkpoint_current_hour_before_new_request_native() -> anyhow::Result<()> {
    let checkpoint = START + HOUR / 2;
    let (_home, runtime, owner) = ledger(&[(START, 12), (checkpoint, 34)]).await?;
    let result = AccountingStore::inspect_range(
        &runtime,
        owner,
        InspectionRange {
            start_ms: START + HOUR,
            end_ms: START + 2 * HOUR,
            grouping: InspectionGrouping::Hour,
        },
        START + HOUR + HOUR / 2,
    )
    .await?;
    let InspectionDay::Range { buckets, .. } = &result else {
        panic!("expected native range: {result:#?}");
    };
    let InspectionDay::Ready(view) = &buckets[0].days[0] else {
        panic!("expected checkpoint-backed empty detail: {result:#?}");
    };
    assert_eq!(
        (
            buckets[0].effective,
            view.coverage.completed_as_of_ms,
            view.totals.attempts
        ),
        (None, checkpoint, 0)
    );
    let pages = inspection_pages(Ok(result));
    let all = text(&pages);
    assert!(
        all.contains("Recorded through ledger checkpoint: 1970-04-11T00:30:00.000Z"),
        "{all}"
    );
    for absent in [
        "Partial bucket",
        "Range total unavailable",
        "In progress",
        NOT_CURRENT,
    ] {
        assert!(!all.contains(absent), "{absent}: {all}");
    }
    assert!(all.contains("Collection coverage: unknown"), "{all}");
    insta::assert_snapshot!("accounting_checkpoint_current_hour", all);
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_ordinary_complete_bucket_control_native() -> anyhow::Result<()> {
    // A later, separate request puts the checkpoint after the selected day.
    let (_home, runtime, owner) =
        ledger(&[(START, 12), (START + HOUR / 2, 34), (START + DAY, 0)]).await?;
    let result = AccountingStore::inspect_range(
        &runtime,
        owner,
        InspectionRange {
            start_ms: START,
            end_ms: START + DAY,
            grouping: InspectionGrouping::Day,
        },
        START + DAY + HOUR,
    )
    .await?;
    let InspectionDay::Range { buckets, .. } = &result else {
        panic!("expected native range: {result:#?}");
    };
    assert!(!buckets[0].partial, "{result:#?}");
    let pages = inspection_pages(Ok(result));
    assert!(
        overview(&pages).contains("Estimated token cost for recorded attempts: $0.000046"),
        "{}",
        text(&pages)
    );
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_current_hour_after_midnight_native() -> anyhow::Result<()> {
    let checkpoint = START + HOUR / 2;
    let (_home, runtime, owner) = ledger(&[(START, 12), (checkpoint, 34)]).await?;
    let result = AccountingStore::inspect_range(
        &runtime,
        owner,
        InspectionRange {
            start_ms: START + DAY,
            end_ms: START + DAY + HOUR,
            grouping: InspectionGrouping::Hour,
        },
        START + DAY + HOUR / 2,
    )
    .await?;
    let InspectionDay::Range {
        buckets,
        completed_as_of_ms,
        ..
    } = &result
    else {
        panic!("expected native range: {result:#?}");
    };
    assert_eq!(&buckets[0].days, &[InspectionDay::CheckpointLag]);
    assert_eq!(*completed_as_of_ms, Some(checkpoint));
    let pages = inspection_pages(Ok(result));
    let all = text(&pages);
    assert!(
        all.contains("Recorded through ledger checkpoint: 1970-04-11T00:30:00.000Z"),
        "{all}"
    );
    for absent in [
        "Partial bucket",
        "Range total unavailable",
        "In progress",
        NOT_CURRENT,
    ] {
        assert!(!all.contains(absent), "{absent}: {all}");
    }
    assert!(all.contains("Collection coverage: unknown"), "{all}");
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_calendar_membership_native() -> anyhow::Result<()> {
    let checkpoint = START + HOUR / 2;
    for (rows, first_cost, total_cost, first_count) in [
        (vec![(START, 12)], "0.000012", "0.000012", 1),
        (
            vec![(START, 12), (checkpoint, 34)],
            "0.000046",
            "0.000046",
            2,
        ),
        (
            vec![(START, 12), (checkpoint, 34), (checkpoint + 1, 56)],
            "0.000102",
            "0.000102",
            3,
        ),
        (
            vec![
                (START, 12),
                (checkpoint, 34),
                (checkpoint + 1, 56),
                (START + HOUR, 78),
            ],
            "0.000102",
            "0.000180",
            3,
        ),
        (
            vec![(START, 12), (checkpoint, 34), (checkpoint, 56)],
            "0.000102",
            "0.000102",
            3,
        ),
    ] {
        let (_home, runtime, owner) = ledger(&rows).await?;
        let result = AccountingStore::inspect_range(
            &runtime,
            owner,
            InspectionRange {
                start_ms: START,
                end_ms: START + 2 * HOUR,
                grouping: InspectionGrouping::Hour,
            },
            START + 3 * HOUR,
        )
        .await?;
        let InspectionDay::Range { buckets, .. } = &result else {
            panic!("expected native range: {result:#?}");
        };
        let InspectionDay::Ready(first) = &buckets[0].days[0] else {
            panic!("expected native ready detail: {result:#?}");
        };
        assert_eq!(
            (
                first.totals.attempts,
                serde_json::to_value(first.totals.known_usd)?
            ),
            (first_count, serde_json::json!(first_cost))
        );
        let pages = inspection_pages(Ok(result));
        let expected = format!("Estimated token cost for recorded attempts: ${total_cost}");
        assert!(overview(&pages).contains(&expected), "{}", text(&pages));
        assert!(!text(&pages).contains("Range total unavailable"));
        runtime.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_user_clips_remain_excluded_native() -> anyhow::Result<()> {
    let (_home, runtime, owner) = ledger(&[(START, 12), (START + HOUR / 2, 34)]).await?;
    for (start_ms, end_ms) in [
        (START + 1, START + DAY),
        (START, START + DAY - 1),
        (START + 1, START + DAY - 1),
    ] {
        let result = AccountingStore::inspect_range(
            &runtime,
            owner,
            InspectionRange {
                start_ms,
                end_ms,
                grouping: InspectionGrouping::Day,
            },
            START + DAY + HOUR,
        )
        .await?;
        let pages = inspection_pages(Ok(result));
        let all = text(&pages);
        assert!(
            all.contains("Partial bucket — excluded from totals"),
            "{all}"
        );
        assert!(
            overview(&pages).contains("Range total unavailable"),
            "{all}"
        );
        assert!(!overview(&pages).contains("Estimated token cost"), "{all}");
        assert!(
            !all.contains("Recorded through ledger checkpoint:"),
            "{all}"
        );
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_retained_away_detail_stays_unavailable_native() -> anyhow::Result<()>
{
    let checkpoint = START + 90 * DAY + HOUR;
    let (_home, runtime, owner) = ledger(&[(START, 12), (checkpoint, 34)]).await?;
    let result = AccountingStore::inspect_range(
        &runtime,
        owner,
        InspectionRange {
            start_ms: START,
            end_ms: START + DAY,
            grouping: InspectionGrouping::Day,
        },
        checkpoint + HOUR,
    )
    .await?;
    let InspectionDay::Range { buckets, .. } = &result else {
        panic!("expected native range: {result:#?}");
    };
    assert!(matches!(
        buckets[0].days[0],
        InspectionDay::DetailUnavailable { .. }
    ));
    let pages = inspection_pages(Ok(result));
    assert!(overview(&pages).contains("Range total unavailable"));
    assert!(text(&pages).contains("Request detail unavailable"));
    assert!(!overview(&pages).contains("Estimated token cost"));
    runtime.close().await;
    Ok(())
}

// Supporting renderer controls replace one native packet's status. The actual
// native transitions to NeedsRefresh/CheckpointLag/DetailUnavailable are also
// exercised by codex-state's inspection tests; these checks guard presentation.
#[tokio::test]
async fn accounting_checkpoint_mixed_unavailable_keeps_whole_range_withheld() -> anyhow::Result<()>
{
    let checkpoint = START + HOUR + HOUR / 2;
    let read_at = START + 3 * HOUR;
    let (_home, runtime, owner) = ledger(&[(START, 12), (checkpoint, 34)]).await?;
    for bad in [
        InspectionDay::NeedsRefresh,
        InspectionDay::CheckpointLag,
        InspectionDay::DetailUnavailable {
            coverage: codex_state::accounting::RetentionCoverage {
                completed_as_of_ms: checkpoint,
                detail_expired_through_ms: Some(START),
                aggregate_day_floor: 0,
                oldest_recorded_day: Some(100),
            },
            read_at_ms: read_at,
            compact: true,
        },
    ] {
        let mut result = AccountingStore::inspect_range(
            &runtime,
            owner,
            InspectionRange {
                start_ms: START,
                end_ms: START + 3 * HOUR,
                grouping: InspectionGrouping::Hour,
            },
            read_at,
        )
        .await?;
        let InspectionDay::Range { buckets, .. } = &mut result else {
            panic!()
        };
        buckets[2].days = vec![bad];
        let pages = inspection_pages(Ok(result));
        let range = overview(&pages);
        assert!(range.contains("Range total unavailable"), "{range}");
        assert!(!range.contains("Estimated token cost"), "{range}");
        let bucket_texts: Vec<_> = pages[0]
            .links
            .iter()
            .map(|(_, target)| pages[*target].text.join("\n"))
            .collect();
        assert!(bucket_texts[0].contains("Estimated token cost for recorded attempts: $0.000012"));
        assert!(bucket_texts[1].contains("Estimated token cost for recorded attempts: $0.000034"));
        assert!(bucket_texts[2].contains("Partial bucket — excluded from totals"));
    }
    runtime.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_checkpoint_unknown_or_mismatched_coverage_is_not_promoted() -> anyhow::Result<()>
{
    let checkpoint = START + HOUR / 2;
    let (_home, runtime, owner) = ledger(&[(START, 12), (checkpoint, 34)]).await?;
    for (reported, missing_days) in [
        (None, false),
        (Some(checkpoint - 1), false),
        (Some(checkpoint + 1), false),
        (Some(checkpoint), true),
    ] {
        let mut result = AccountingStore::inspect_range(
            &runtime,
            owner,
            InspectionRange {
                start_ms: START,
                end_ms: START + DAY,
                grouping: InspectionGrouping::Day,
            },
            START + DAY + HOUR,
        )
        .await?;
        let InspectionDay::Range {
            completed_as_of_ms,
            buckets,
            ..
        } = &mut result
        else {
            panic!()
        };
        *completed_as_of_ms = reported;
        if missing_days {
            buckets[0].days.clear();
        }
        let pages = inspection_pages(Ok(result));
        assert!(overview(&pages).contains("Range total unavailable"));
        assert!(!text(&pages).contains("Recorded through ledger checkpoint:"));
    }
    runtime.close().await;
    Ok(())
}

// #1009: a checkpoint-limited bucket next to a user-clipped one leaves the
// range incomplete. The not-current notice is left out for it, so the
// overview must name the ledger checkpoint instead of saying neither.
#[tokio::test]
async fn accounting_checkpoint_incomplete_range_names_the_checkpoint() -> anyhow::Result<()> {
    for ((rows, requested, read_at), checkpoint) in [
        // Recorded-through bucket, then a bucket clipped by the requested end.
        (
            [(START, 12), (START + HOUR / 2, 34)],
            (START, START + 2 * HOUR - 1),
            START + 3 * HOUR,
        ),
        // A bucket clipped by the requested start, then one in progress.
        (
            [(START, 12), (START + 2 * HOUR + HOUR / 4, 34)],
            (START + 1, START + 3 * HOUR),
            START + 2 * HOUR + HOUR / 2,
        ),
    ]
    .into_iter()
    .zip(["1970-04-11T00:30:00.000Z", "1970-04-11T02:15:00.000Z"])
    {
        let (_home, runtime, owner) = ledger(&rows).await?;
        let result = AccountingStore::inspect_range(
            &runtime,
            owner,
            InspectionRange {
                start_ms: requested.0,
                end_ms: requested.1,
                grouping: InspectionGrouping::Hour,
            },
            read_at,
        )
        .await?;
        let pages = inspection_pages(Ok(result));
        let all = text(&pages);
        let range = overview(&pages);
        assert!(
            all.contains("Recorded through ledger checkpoint:") || all.contains("In progress"),
            "fixture must have a checkpoint-limited bucket: {all}"
        );
        assert!(range.contains("Range total unavailable"), "{all}");
        assert!(
            range.contains(&format!(
                "Recorded through ledger checkpoint: {checkpoint} (as of this read)"
            )),
            "{all}"
        );
        assert!(!range.contains(NOT_CURRENT), "{all}");
        runtime.close().await;
    }
    Ok(())
}
