use super::super::Journal;
use super::super::StateRuntime;
use super::super::types::Dialect;
use super::*;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

fn attempt() -> Attempt {
    serde_json::from_value(json!({
        "attempt_id": "00000000-0000-0000-0000-000000000001",
        "request_id": "00000000-0000-0000-0000-000000000002",
        "thread_id": "00000000-0000-0000-0000-000000000003",
        "turn": "fixture", "retry_of": null, "provider": "synthetic",
        "model": "fixture-model", "scope": "00000000-0000-0000-0000-000000000004",
        "dialect": "NativeAnthropic", "dispatched_at_ms": 100
    }))
    .unwrap()
}

fn catalog() -> Value {
    json!({
        "id": "00000000-0000-0000-0000-000000000010",
        "provider": "synthetic", "model": "fixture-model",
        "scope": "00000000-0000-0000-0000-000000000004",
        "currency": "USD", "unit": "PerMillionTokens",
        "rates": {"noncached": "3", "read": "0.3", "write": null, "output": null},
        "source_reference": "00000000-0000-0000-0000-000000000011",
        "source_kind": "ProviderPublished", "observed_at_ms": 80,
        "approved_at_ms": 90, "effective_from_ms": 100, "effective_end_ms": 200
    })
}

fn snapshot() -> Snapshot {
    serde_json::from_value(catalog()).unwrap()
}
fn decimal(text: &str) -> Decimal {
    Decimal::try_from(text.to_owned()).unwrap()
}
fn row(revision: i64, patch: Value) -> Observation {
    serde_json::from_value(json!({"revision": revision,
        "source": "00000000-0000-0000-0000-000000000005",
        "sequence": revision * 10, "patch": patch}))
    .unwrap()
}

fn partial_expected(observations: Vec<Observation>) -> ObservationQuote {
    ObservationQuote {
        attempt: attempt(),
        observations,
        usage: Usage {
            noncached: Some(50),
            read: Some(10),
            ..Usage::default()
        },
        snapshot: Some(snapshot()),
        buckets: [
            BucketQuote::Priced(decimal("0.00015")),
            BucketQuote::Priced(decimal("0.000003")),
            BucketQuote::MissingUsage,
            BucketQuote::MissingUsage,
        ],
        known_subtotal: decimal("0.000153"),
        all_buckets_priced: None,
        subtotal_display: DisplayAmount {
            text: "0.000153".into(),
            rounded: false,
            nonzero_sub_micro: false,
        },
    }
}

#[test]
fn native_partial_retains_exact_subtotal_and_provenance() {
    let rows = vec![row(1, json!({"input":50,"read":10}))];
    assert_eq!(
        quote_observations(&attempt(), &rows, &[snapshot()]).unwrap(),
        partial_expected(rows)
    );
}

#[test]
fn inclusive_revisions_replace_and_reasoning_is_not_charged_twice() {
    let mut a = attempt();
    a.dialect = Dialect::Inclusive;
    let rows = vec![
        row(1, json!({"input":100,"read":20,"write":10,"output":5})),
        row(
            3,
            json!({"input":60,"read":10,"write":0,"output":2,"reasoning":2}),
        ),
        row(7, json!({"input":null,"read":null})),
    ];
    let mut s = snapshot();
    s.rates.output = Some(decimal("2"));
    assert_eq!(
        quote_observations(&a, &rows, &[s.clone()]).unwrap(),
        ObservationQuote {
            attempt: a,
            observations: rows,
            usage: Usage {
                input: Some(60),
                noncached: Some(50),
                read: Some(10),
                write: Some(0),
                output: Some(2),
                reasoning: Some(2),
                total: None
            },
            snapshot: Some(s),
            buckets: [
                BucketQuote::Priced(decimal("0.00015")),
                BucketQuote::Priced(decimal("0.000003")),
                BucketQuote::Priced(Decimal::default()),
                BucketQuote::Priced(decimal("0.000004"))
            ],
            known_subtotal: decimal("0.000157"),
            all_buckets_priced: Some(decimal("0.000157")),
            subtotal_display: DisplayAmount {
                text: "0.000157".into(),
                rounded: false,
                nonzero_sub_micro: false
            },
        }
    );
}

#[test]
fn absent_null_zero_and_missing_rates_remain_distinct() {
    let mut s = snapshot();
    s.rates.output = Some(decimal("0"));
    for patch in [
        json!({}),
        json!({"input":null,"read":null,"write":null,"output":null}),
    ] {
        let rows = vec![row(1, patch)];
        assert_eq!(
            quote_observations(&attempt(), &rows, &[s.clone()]).unwrap(),
            ObservationQuote {
                attempt: attempt(),
                observations: rows,
                usage: Usage::default(),
                snapshot: Some(s.clone()),
                buckets: [BucketQuote::MissingUsage; 4],
                known_subtotal: Decimal::default(),
                all_buckets_priced: None,
                subtotal_display: Decimal::default().display(),
            }
        );
    }
    let rows = [row(1, json!({"input":0,"read":0,"write":0,"output":0}))];
    let q = quote_observations(&attempt(), &rows, &[]).unwrap();
    assert_eq!(
        (q.buckets, q.all_buckets_priced, q.snapshot),
        (
            [BucketQuote::Priced(Decimal::default()); 4],
            Some(Decimal::default()),
            None
        )
    );
    let rows = [row(1, json!({"input":1,"read":0,"write":1,"output":2}))];
    assert_eq!(
        quote_observations(&attempt(), &rows, &[s]).unwrap().buckets,
        [
            BucketQuote::Priced(decimal("0.000003")),
            BucketQuote::Priced(Decimal::default()),
            BucketQuote::MissingRate,
            BucketQuote::Priced(Decimal::default())
        ]
    );
    let mut a = attempt();
    a.dialect = Dialect::UnknownCompatible;
    assert_eq!(
        quote_observations(&a, &rows, &[snapshot()])
            .unwrap()
            .buckets,
        [
            BucketQuote::MissingUsage,
            BucketQuote::Priced(Decimal::default()),
            BucketQuote::MissingRate,
            BucketQuote::MissingRate
        ]
    );
}

#[test]
fn validates_attempt_and_replay_before_quoting() {
    let mut a = attempt();
    a.provider = "\n".into();
    assert!(quote_observations(&a, &[], &[]).is_err());
    for rows in [
        vec![row(0, json!({}))],
        vec![row(1, json!({})), row(1, json!({}))],
        vec![row(3, json!({})), row(1, json!({}))],
        vec![row(1, json!({"output":1,"reasoning":2}))],
    ] {
        assert!(quote_observations(&attempt(), &rows, &[snapshot()]).is_err());
    }
}

#[test]
fn exact_identity_and_half_open_dispatch_selection() {
    for (field, value) in [
        ("provider", json!("Synthetic")),
        ("model", json!("other")),
        ("scope", json!("00000000-0000-0000-0000-000000000099")),
    ] {
        let mut raw = catalog();
        raw[field] = value;
        let s = serde_json::from_value(raw).unwrap();
        let q = quote_observations(&attempt(), &[row(1, json!({"input":1}))], &[s]).unwrap();
        assert_eq!(
            (q.snapshot, q.all_buckets_priced, q.buckets[0]),
            (None, None, BucketQuote::MissingRate)
        );
    }
    for (dispatch, found) in [
        (99, false),
        (100, true),
        (199, true),
        (200, false),
        (201, false),
    ] {
        let mut a = attempt();
        a.dispatched_at_ms = Count::try_from(dispatch).unwrap();
        assert_eq!(
            quote_observations(&a, &[], &[snapshot()]).unwrap().snapshot,
            found.then(snapshot)
        );
    }
}

#[test]
fn prospective_catalog_is_order_independent_and_return_is_owned() {
    let rows = vec![row(1, json!({"input":50,"read":10}))];
    let mut later = snapshot();
    later.id = Uuid::from_u128(20);
    later.source_kind = SourceKind::NativeCatalog;
    later.observed_at_ms = Count::try_from(180).unwrap();
    later.approved_at_ms = Count::try_from(190).unwrap();
    later.effective_from_ms = Count::try_from(200).unwrap();
    later.effective_end_ms = None;
    later.rates.noncached = Some(decimal("99"));
    let mut snapshots = vec![later, snapshot()];
    let q = quote_observations(&attempt(), &rows, &snapshots).unwrap();
    snapshots.reverse();
    assert_eq!(
        quote_observations(&attempt(), &rows, &snapshots).unwrap(),
        partial_expected(rows.clone())
    );
    snapshots[0].rates.noncached = Some(decimal("999"));
    assert_eq!(q, partial_expected(rows));
    let mut a = attempt();
    a.dispatched_at_ms = Count::try_from(200).unwrap();
    assert_eq!(
        quote_observations(&a, &[], &snapshots).unwrap().snapshot,
        Some(snapshots[1].clone())
    );
}

#[test]
fn invalid_catalogs_fail_even_when_not_selected() {
    assert!(quote_observations(&attempt(), &[], &[snapshot(), snapshot()]).is_err());
    let mut overlap = snapshot();
    overlap.id = Uuid::from_u128(20);
    assert!(quote_observations(&attempt(), &[], &[snapshot(), overlap]).is_err());
    let many: Vec<_> = (0..65)
        .map(|id| {
            let mut s = snapshot();
            s.id = Uuid::from_u128(id);
            s.model = "other".into();
            s
        })
        .collect();
    assert!(quote_observations(&attempt(), &[], &many).is_err());
    assert!(quote_observations(&attempt(), &[], &many[..64]).is_ok());
    for (field, value) in [
        ("observed_at_ms", json!(101)),
        ("approved_at_ms", json!(101)),
        ("effective_end_ms", json!(100)),
        ("effective_end_ms", json!(99)),
        ("provider", json!("")),
        ("model", json!(" ")),
        ("model", json!("x".repeat(129))),
        ("provider", json!("bad\nmetadata")),
    ] {
        let mut raw = catalog();
        raw["scope"] = json!("00000000-0000-0000-0000-000000000099");
        raw[field] = value;
        assert!(
            quote_observations(&attempt(), &[], &[serde_json::from_value(raw).unwrap()]).is_err()
        );
    }
}

#[test]
fn snapshot_schema_rejects_unsupported_or_unbounded_inputs() {
    for (field, value) in [
        ("currency", json!("EUR")),
        ("unit", json!("PerToken")),
        ("source_kind", json!("Unapproved")),
        ("id", json!("opaque-but-not-uuid")),
        ("source_reference", json!("x".repeat(129))),
        ("extra", json!(0)),
        ("observed_at_ms", json!(-1)),
        ("approved_at_ms", json!(true)),
    ] {
        let mut raw = catalog();
        raw[field] = value;
        assert!(serde_json::from_value::<Snapshot>(raw).is_err());
    }
    for rate in [json!(1), json!(true), json!(1.5), json!("-1"), json!("1e3")] {
        let mut raw = catalog();
        raw["rates"]["read"] = rate;
        assert!(serde_json::from_value::<Snapshot>(raw).is_err());
    }
    let mut raw = catalog();
    raw["rates"]["reasoning"] = json!("1");
    assert!(serde_json::from_value::<Snapshot>(raw).is_err());
}

#[test]
fn decimal_syntax_and_representation_bounds() {
    for text in [
        "",
        ".",
        ".1",
        "1.",
        "1.2.3",
        "+1",
        "-0",
        "1e0",
        " 1",
        "1 ",
        "NaN",
        "١",
        "0.0000000000000000001",
        "100000000000000000000000000000000000000",
        "340282366920938463463374607431768211456",
    ] {
        assert!(Decimal::try_from(text.to_owned()).is_err(), "{text}");
    }
    assert_eq!(
        decimal("0001.2300"),
        Decimal {
            coefficient: 123,
            scale: 2
        }
    );
    assert_eq!(
        decimal("0.000000000000000001").price(1).unwrap(),
        Decimal {
            coefficient: 1,
            scale: 24
        }
    );
    assert_eq!(
        decimal("99999999999999999999999999999999999999"),
        Decimal {
            coefficient: 10_u128.pow(38) - 1,
            scale: 0
        }
    );
}

#[test]
fn checked_product_alignment_and_sum_overflow_are_errors() {
    let huge = decimal("99999999999999999999999999999999999999");
    assert_eq!(
        huge.price(4).unwrap_err().to_string(),
        "price product overflow"
    );
    assert_eq!(
        huge.add(decimal("0.1")).unwrap_err().to_string(),
        "decimal alignment overflow"
    );
    let mut s = snapshot();
    s.rates.noncached = Some(huge);
    s.rates.read = Some(huge);
    let rows = [row(1, json!({"input":2,"read":2}))];
    assert_eq!(
        quote_observations(&attempt(), &rows, &[s])
            .unwrap_err()
            .to_string(),
        "decimal sum overflow"
    );
    assert_eq!(huge.add(Decimal::default()).unwrap(), huge);
}

#[test]
fn display_rounds_half_even_and_preserves_exact_values() {
    for (exact, text, rounded, sub) in [
        ("0", "0.000000", false, false),
        ("0.0000005", "0.000000", true, true),
        ("0.0000015", "0.000002", true, false),
        ("0.0000004", "0.000000", true, true),
        ("0.0000006", "0.000001", true, true),
        ("0.9999995", "1.000000", true, false),
        ("1.123456", "1.123456", false, false),
    ] {
        let amount = decimal(exact);
        assert_eq!(
            amount.display(),
            DisplayAmount {
                text: text.into(),
                rounded,
                nonzero_sub_micro: sub
            }
        );
        assert_eq!(amount, decimal(exact));
    }
    assert_eq!(
        Decimal {
            coefficient: u128::MAX,
            scale: 0
        }
        .display()
        .text,
        "340282366920938463463374607431768211455.000000"
    );
    let mut s = snapshot();
    s.rates.noncached = Some(decimal("0.4"));
    s.rates.read = Some(decimal("0.4"));
    let q = quote_observations(
        &attempt(),
        &[row(1, json!({"input":1,"read":1,"write":0,"output":0}))],
        &[s],
    )
    .unwrap();
    assert_eq!(
        (q.known_subtotal, q.all_buckets_priced, q.subtotal_display),
        (
            decimal("0.0000008"),
            Some(decimal("0.0000008")),
            DisplayAmount {
                text: "0.000001".into(),
                rounded: true,
                nonzero_sub_micro: true
            }
        )
    );
}

#[tokio::test]
async fn closed_journal_reopens_and_quotes_without_changing_rows_or_positions() -> anyhow::Result<()>
{
    let home = super::super::super::test_support::unique_temp_dir();
    let home = scopeguard::guard(home, |path| {
        let _ = std::fs::remove_dir_all(path);
    });
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let journal = Journal::create_for_tests(&runtime).await?;
    let rows = vec![
        row(1, json!({"input":40})),
        row(3, json!({"input":50,"read":10})),
    ];
    journal.append_observation(&attempt(), &rows).await?;
    runtime.close().await;
    assert!(runtime.pool.is_closed());
    let runtime = StateRuntime::init_for_testing(home.to_path_buf(), "synthetic".into()).await?;
    let journal = Journal { runtime: &runtime };
    for _ in 0..2 {
        let (a, stored) = journal
            .read_observations(attempt().attempt_id)
            .await?
            .unwrap();
        assert_eq!((&a, &stored), (&attempt(), &rows));
        assert_eq!(
            quote_observations(&a, &stored, &[snapshot()])?,
            partial_expected(rows.clone())
        );
        assert_eq!(
            journal.read_observations(a.attempt_id).await?,
            Some((a, rows.clone()))
        );
    }
    runtime.close().await;
    Ok(())
}
