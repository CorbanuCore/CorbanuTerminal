//! S01's invented numeric fixture through the normal library, not provider evidence.
use anyhow::Context;
use codex_protocol::ThreadId;
use codex_state::accounting::*;
use codex_state::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use sqlx::Connection;
use sqlx::SqliteConnection;
use std::collections::BTreeSet;
use std::collections::HashSet;
use uuid::Uuid;

#[path = "../src/runtime/accounting_late_import_test_support.rs"]
mod support;
use support::*;

// Original observation positions, not arrival-order revisions.
const CHECKPOINT: [usize; 4] = [0, 1, 3, 4];
const REPLAY: [usize; 13] = [8, 5, 1, 0, 7, 3, 2, 6, 4, 5, 0, 2, 8];

fn at(value: &str) -> anyhow::Result<i64> {
    Ok(chrono::DateTime::parse_from_rfc3339(value)?.timestamp_millis())
}

fn owner(value: u128) -> anyhow::Result<ThreadId> {
    ThreadId::from_string(&Uuid::from_u128(value).to_string()).map_err(Into::into)
}

struct Fixture {
    entries: Vec<RetainedImport>,
    events: Vec<(usize, Observation)>,
    prices: Vec<Snapshot>,
    young: i64,
}

impl Fixture {
    fn new() -> anyhow::Result<Self> {
        let january = at("2026-01-01T00:00:00Z")?;
        let september = at("2026-09-01T00:00:00Z")?;
        let mut prices = Vec::new();
        for (id, provider, model, scope, start, end, rates) in [
            (
                201,
                "openai",
                "synthetic-o",
                301,
                january,
                Some(september),
                ["1", "0.25", "1.25", "2"],
            ),
            (
                202,
                "openai",
                "synthetic-o",
                301,
                september,
                None,
                ["2", "0.5", "2.5", "4"],
            ),
            (
                203,
                "anthropic",
                "synthetic-a",
                302,
                january,
                None,
                ["3", "0.3", "3.75", "6"],
            ),
        ] {
            prices.push(Snapshot {
                id: Uuid::from_u128(id),
                provider: provider.into(),
                model: model.into(),
                scope: Uuid::from_u128(scope),
                currency: Currency::Usd,
                unit: Unit::PerMillionTokens,
                rates: Rates {
                    noncached: Some(rates[0].to_owned().try_into()?),
                    read: Some(rates[1].to_owned().try_into()?),
                    write: Some(rates[2].to_owned().try_into()?),
                    output: Some(rates[3].to_owned().try_into()?),
                },
                // Synthetic catalog simulation; NOT authentic provider approval.
                source_reference: Uuid::from_u128(id + 200),
                source_kind: SourceKind::NativeCatalog,
                observed_at_ms: start.try_into()?,
                approved_at_ms: start.try_into()?,
                effective_from_ms: start.try_into()?,
                effective_end_ms: end.map(Count::try_from).transpose()?,
            });
        }
        let mut entries = Vec::new();
        for (id, request, thread, turn, provider, model, time, price) in [
            (
                1,
                101,
                11,
                "turn-root",
                "openai",
                "synthetic-o",
                "2026-09-11T00:00:00Z",
                Some(1),
            ),
            (
                2,
                102,
                12,
                "turn-child-a",
                "anthropic",
                "synthetic-a",
                "2026-09-11T00:00:01Z",
                Some(2),
            ),
            (
                3,
                102,
                12,
                "turn-child-a",
                "anthropic",
                "synthetic-a",
                "2026-09-11T00:00:02Z",
                Some(2),
            ),
            (
                4,
                103,
                13,
                "turn-child-b",
                "pfterminal-plan",
                "synthetic-corbanu-unpriced",
                "2026-09-11T00:00:03Z",
                None,
            ),
            (
                5,
                104,
                14,
                "historical-turn",
                "openai",
                "synthetic-o",
                "2026-08-31T23:59:59Z",
                Some(0),
            ),
            (
                6,
                105,
                14,
                "historical-turn-2",
                "openai",
                "synthetic-o",
                "2026-09-02T00:00:00Z",
                Some(1),
            ),
        ] {
            let (scope, dialect) = match provider {
                "openai" => (301, Dialect::Inclusive),
                "anthropic" => (302, Dialect::NativeAnthropic),
                "pfterminal-plan" => (303, Dialect::Inclusive),
                _ => anyhow::bail!("unexpected synthetic provider"),
            };
            entries.push(RetainedImport {
                attempt: Attempt {
                    attempt_id: Uuid::from_u128(id),
                    request_id: Uuid::from_u128(request),
                    thread_id: owner(thread)?,
                    turn: turn.into(),
                    retry_of: (id == 3).then_some(Uuid::from_u128(2)),
                    provider: provider.into(),
                    model: model.into(),
                    scope: Uuid::from_u128(scope),
                    dialect,
                    dispatched_at_ms: at(time)?.try_into()?,
                },
                observations: Vec::new(),
                original_price: match price {
                    Some(index) => OriginalPriceEvidence::Bound(prices[index].clone()),
                    None => OriginalPriceEvidence::Unpriced,
                },
            });
        }
        // Transcription of the nine original wire patches, with no default zeros.
        let patches = [
            (
                0,
                1,
                json!({"input":100,"read":40,"write":0,"output":20,"reasoning":8,"total":120}),
            ),
            (1, 1, json!({"input":50,"read":10,"write":20})),
            (1, 2, json!({})),
            (2, 1, json!({"input":100,"read":20,"write":40,"output":0})),
            (2, 2, json!({"output":12})),
            (2, 3, json!({"output":30})),
            (
                3,
                1,
                json!({"input":80,"read":20,"output":10,"reasoning":0,"total":90}),
            ),
            (
                4,
                1,
                json!({"input":100,"read":40,"write":0,"output":20,"reasoning":8,"total":120}),
            ),
            (5, 1, json!({"input":50,"output":10,"total":60})),
        ];
        let mut events = Vec::new();
        for (position, (entry, revision, patch)) in patches.into_iter().enumerate() {
            let observation = Observation {
                revision: revision.try_into()?,
                source: Uuid::from_u128(901),
                sequence: i64::try_from(position)?.try_into()?,
                patch: serde_json::from_value(patch)?,
            };
            entries[entry].observations.push(observation.clone());
            events.push((entry, observation));
        }
        Ok(Self {
            entries,
            events,
            prices,
            young: at("2026-09-11T00:00:03Z")?,
        })
    }

    async fn create_owners(&self, runtime: &StateRuntime) -> anyhow::Result<()> {
        for id in 11..=14 {
            native(runtime, owner(id)?).await?;
        }
        for child in [12, 13] {
            runtime
                .upsert_thread_spawn_edge(
                    owner(11)?,
                    owner(child)?,
                    DirectionalThreadSpawnEdgeStatus::Open,
                )
                .await?;
        }
        Ok(())
    }

    async fn admit(&self, runtime: &StateRuntime) -> anyhow::Result<()> {
        self.create_owners(runtime).await?;
        let store = AccountingStore::open(runtime, at("2026-08-31T23:59:59Z")?).await?;
        for index in [4, 5, 0, 1, 2, 3] {
            let entry = &self.entries[index];
            // Both old/new descriptors may be present; dispatch selects the original.
            store
                .admit(
                    entry.attempt.thread_id,
                    &entry.attempt,
                    &self.prices,
                    i64::from(entry.attempt.dispatched_at_ms),
                )
                .await?;
        }
        Ok(())
    }

    async fn deliver(&self, runtime: &StateRuntime, indices: &[usize]) -> anyhow::Result<()> {
        let store = AccountingStore::open(runtime, self.young).await?;
        for index in indices {
            let (entry, observation) = &self.events[*index];
            let attempt = &self.entries[*entry].attempt;
            store
                .observe(
                    attempt.thread_id,
                    attempt,
                    std::slice::from_ref(observation),
                    self.young,
                )
                .await?;
        }
        Ok(())
    }
}

fn totals(
    known: [i64; 7],
    unknown: [i64; 7],
    usd: &str,
    unknown_estimates: i64,
    attempts: i64,
) -> anyhow::Result<DayTotals> {
    Ok(DayTotals {
        measured: std::array::from_fn(|index| Metric {
            known: known[index],
            unknown: unknown[index],
        }),
        known_usd: usd.to_owned().try_into()?,
        unknown_estimates,
        attempts,
    })
}

// Independent expected owner/day values, not an invocation of the usage reducer.
fn expected_days() -> anyhow::Result<Vec<(ThreadId, i64, DayTotals)>> {
    let day = at("2026-09-11T00:00:00Z")? / DAY;
    Ok(vec![
        (
            owner(11)?,
            day,
            totals([100, 60, 40, 0, 20, 8, 120], [0; 7], "0.000220", 0, 1)?,
        ),
        (
            owner(12)?,
            day,
            totals(
                [240, 150, 30, 60, 30, 0, 190],
                [0, 0, 0, 0, 1, 2, 1],
                "0.000864",
                1,
                2,
            )?,
        ),
        (
            owner(13)?,
            day,
            totals([80, 0, 20, 0, 10, 0, 90], [0, 1, 0, 1, 0, 0, 0], "0", 1, 1)?,
        ),
        (
            owner(14)?,
            at("2026-08-31T00:00:00Z")? / DAY,
            totals([100, 60, 40, 0, 20, 8, 120], [0; 7], "0.000110", 0, 1)?,
        ),
        (
            owner(14)?,
            at("2026-09-02T00:00:00Z")? / DAY,
            totals(
                [50, 0, 0, 0, 10, 0, 60],
                [0, 1, 1, 1, 0, 1, 0],
                "0.000040",
                1,
                1,
            )?,
        ),
    ])
}

async fn assert_days(runtime: &StateRuntime, time: i64) -> anyhow::Result<Vec<DayTotals>> {
    let store = AccountingStore::open(runtime, time).await?;
    let mut actual = Vec::new();
    for (thread, day, expected) in expected_days()? {
        let oldest = if thread == owner(14)? {
            at("2026-08-31T00:00:00Z")? / DAY
        } else {
            at("2026-09-11T00:00:00Z")? / DAY
        };
        let value = store.read_day(thread, day, time).await?;
        assert_eq!(
            value,
            RetainedDay::Available {
                coverage: RetentionCoverage {
                    completed_as_of_ms: time,
                    detail_expired_through_ms: Some(time - 90 * DAY),
                    aggregate_day_floor: (time - 365 * DAY) / DAY + 1,
                    oldest_recorded_day: Some(oldest),
                },
                totals: Current::Ready(expected),
            }
        );
        let RetainedDay::Available {
            totals: Current::Ready(total),
            ..
        } = value
        else {
            anyhow::bail!("golden day was not ready");
        };
        actual.push(total);
    }
    Ok(actual)
}

async fn assert_prices(conn: &mut SqliteConnection, fixture: &Fixture) -> anyhow::Result<()> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT snapshot_id, payload FROM draft_accounting_price_snapshots ORDER BY snapshot_id",
    )
    .fetch_all(&mut *conn)
    .await?;
    let expected = fixture
        .prices
        .iter()
        .map(|price| Ok((price.id.to_string(), serde_json::to_string(price)?)))
        .collect::<anyhow::Result<Vec<_>>>()?;
    assert_eq!(rows, expected);
    Ok(())
}

async fn assert_raw(runtime: &StateRuntime, fixture: &Fixture) -> anyhow::Result<()> {
    let mut conn = connection(runtime).await?;
    let attempts: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT attempt_id, request_id, payload FROM draft_accounting_attempts ORDER BY attempt_id",
    )
    .fetch_all(&mut conn)
    .await?;
    let expected = fixture
        .entries
        .iter()
        .map(|entry| {
            let a = &entry.attempt;
            Ok((
                a.attempt_id.to_string(),
                a.request_id.to_string(),
                serde_json::to_string(a)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    assert_eq!(attempts, expected);
    let observations: Vec<(String, i64, String, i64, String)> = sqlx::query_as(
        "SELECT attempt_id, revision, source, sequence, payload FROM draft_accounting_observations ORDER BY attempt_id, revision",
    ).fetch_all(&mut conn).await?;
    let expected = fixture
        .events
        .iter()
        .map(|(index, observation)| {
            Ok((
                fixture.entries[*index].attempt.attempt_id.to_string(),
                i64::from(observation.revision),
                observation.source.to_string(),
                i64::from(observation.sequence),
                serde_json::to_string(observation)?,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    assert_eq!(observations, expected);
    let bindings: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT attempt_id, snapshot_id FROM draft_accounting_price_bindings ORDER BY attempt_id",
    )
    .fetch_all(&mut conn)
    .await?;
    assert_eq!(
        bindings,
        [Some(202), Some(203), Some(203), None, Some(201), Some(202)]
            .into_iter()
            .enumerate()
            .map(|(i, price)| (
                Uuid::from_u128(i as u128 + 1).to_string(),
                price.map(|id| Uuid::from_u128(id).to_string()),
            ))
            .collect::<Vec<_>>()
    );
    assert_prices(&mut conn, fixture).await?;
    conn.close().await?;
    Ok(())
}

#[tokio::test]
async fn accounting_contract_golden_raw_replay_and_two_reopens() -> anyhow::Result<()> {
    let path = home();
    let fixture = Fixture::new()?;
    let runtime = open(&path).await?;
    fixture.admit(&runtime).await?;
    fixture.deliver(&runtime, &CHECKPOINT).await?;
    let mut conn = connection(&runtime).await?;
    let checkpoint = dump(&mut conn).await?;
    conn.close().await?;
    runtime.close().await;
    let runtime = open(&path).await?;
    let mut conn = connection(&runtime).await?;
    assert_eq!(dump(&mut conn).await?, checkpoint);
    conn.close().await?;
    for _ in 0..2 {
        fixture.deliver(&runtime, &REPLAY).await?;
    }
    assert_raw(&runtime, &fixture).await?;
    assert_days(&runtime, fixture.young).await?;
    let mut conn = connection(&runtime).await?;
    let complete = dump(&mut conn).await?;
    conn.close().await?;
    runtime.close().await;
    for _ in 0..2 {
        let runtime = open(&path).await?;
        fixture.deliver(&runtime, &REPLAY).await?;
        assert_raw(&runtime, &fixture).await?;
        assert_days(&runtime, fixture.young).await?;
        let mut conn = connection(&runtime).await?;
        assert_eq!(dump(&mut conn).await?, complete);
        conn.close().await?;
        runtime.close().await;
    }
    Ok(())
}

// These fixture amounts have at most six places. No float or production sum API.
fn fixture_microusd(value: Decimal) -> anyhow::Result<i64> {
    let text = serde_json::to_value(value)?;
    let text = text
        .as_str()
        .context("decimal must serialize as exact text")?;
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    anyhow::ensure!(fraction.len() <= 6, "fixture precision changed");
    Ok(whole.parse::<i64>()? * 1_000_000 + format!("{fraction:0<6}").parse::<i64>()?)
}

#[tokio::test]
async fn accounting_contract_golden_original_price_and_native_family() -> anyhow::Result<()> {
    let path = home();
    let fixture = Fixture::new()?;
    let runtime = open(&path).await?;
    fixture.admit(&runtime).await?;
    fixture.deliver(&runtime, &REPLAY).await?;
    assert_raw(&runtime, &fixture).await?;
    let descendants = runtime.list_thread_spawn_descendants(owner(11)?).await?;
    assert_eq!(descendants, vec![owner(12)?, owner(13)?]);
    let mut family = HashSet::from([owner(11)?]);
    family.extend(descendants);
    let mut conn = connection(&runtime).await?;
    let payloads: Vec<String> =
        sqlx::query_scalar("SELECT payload FROM draft_accounting_attempts ORDER BY attempt_id")
            .fetch_all(&mut conn)
            .await?;
    let members = payloads
        .iter()
        .map(|text| serde_json::from_str::<Attempt>(text))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|attempt| family.contains(&attempt.thread_id))
        .collect::<Vec<_>>();
    assert_eq!(
        members,
        fixture.entries[..4]
            .iter()
            .map(|e| e.attempt.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        members
            .iter()
            .map(|a| a.request_id)
            .collect::<BTreeSet<_>>(),
        [101, 102, 103].map(Uuid::from_u128).into_iter().collect()
    );
    let store = AccountingStore::open(&runtime, fixture.young).await?;
    let expected_amounts = [
        ("0.000220", Some("0.000220")),
        ("0.000228", None),
        ("0.000636", Some("0.000636")),
        ("0", None),
        ("0.000110", Some("0.000110")),
        ("0.000040", None),
    ];
    for (entry, (subtotal, complete)) in fixture.entries.iter().zip(expected_amounts) {
        let quote = store
            .observe(
                entry.attempt.thread_id,
                &entry.attempt,
                &entry.observations,
                fixture.young,
            )
            .await?;
        assert_eq!(quote.attempt, entry.attempt);
        assert_eq!(quote.observations, entry.observations);
        assert_eq!(quote.known_subtotal, subtotal.to_owned().try_into()?);
        assert_eq!(
            quote.all_buckets_priced,
            complete
                .map(|text| Decimal::try_from(text.to_owned()))
                .transpose()?
        );
        let expected = match &entry.original_price {
            OriginalPriceEvidence::Bound(price) => Some(price.clone()),
            OriginalPriceEvidence::Unpriced => None,
        };
        assert_eq!(quote.snapshot, expected);
    }
    let days = assert_days(&runtime, fixture.young).await?;
    let mut measured: [Metric; 7] = std::array::from_fn(|_| Metric::default());
    let mut micros = 0;
    let mut unknown = 0;
    let mut attempts = 0;
    for day in &days[..3] {
        for (sum, part) in measured.iter_mut().zip(&day.measured) {
            sum.known += part.known;
            sum.unknown += part.unknown;
        }
        micros += fixture_microusd(day.known_usd)?;
        unknown += day.unknown_estimates;
        attempts += day.attempts;
    }
    assert_eq!(
        (measured, micros, unknown, attempts),
        (
            totals(
                [420, 210, 90, 60, 60, 8, 400],
                [0, 1, 0, 1, 1, 2, 1],
                "0.001084",
                2,
                4
            )?
            .measured,
            1084,
            2,
            4,
        )
    );
    conn.close().await?;
    runtime.close().await;
    Ok(())
}

async fn assert_compact(
    runtime: &StateRuntime,
    fixture: &Fixture,
    time: i64,
) -> anyhow::Result<Vec<(String, Vec<String>)>> {
    assert_days(runtime, time).await?;
    let mut conn = connection(runtime).await?;
    assert_prices(&mut conn, fixture).await?;
    let fences: Vec<(String, i64)> = sqlx::query_as(
        "SELECT attempt_id, expires_at_ms FROM draft_accounting_tombstones ORDER BY attempt_id",
    )
    .fetch_all(&mut conn)
    .await?;
    assert_eq!(
        fences,
        fixture
            .entries
            .iter()
            .map(|entry| (
                entry.attempt.attempt_id.to_string(),
                i64::from(entry.attempt.dispatched_at_ms) + 365 * DAY,
            ))
            .collect::<Vec<_>>()
    );
    let refs: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT thread_id, utc_day, snapshot_id FROM draft_accounting_compact_snapshots ORDER BY thread_id, utc_day, snapshot_id",
    ).fetch_all(&mut conn).await?;
    let day = at("2026-09-11T00:00:00Z")? / DAY;
    assert_eq!(
        refs,
        vec![
            (
                owner(11)?.to_string(),
                day,
                Uuid::from_u128(202).to_string()
            ),
            (
                owner(12)?.to_string(),
                day,
                Uuid::from_u128(203).to_string()
            ),
            (
                owner(14)?.to_string(),
                at("2026-08-31T00:00:00Z")? / DAY,
                Uuid::from_u128(201).to_string()
            ),
            (
                owner(14)?.to_string(),
                at("2026-09-02T00:00:00Z")? / DAY,
                Uuid::from_u128(202).to_string()
            ),
        ]
    );
    let compact_keys: Vec<(String, i64)> = sqlx::query_as(
        "SELECT thread_id, utc_day FROM draft_accounting_compact_days ORDER BY thread_id, utc_day",
    )
    .fetch_all(&mut conn)
    .await?;
    assert_eq!(
        compact_keys,
        expected_days()?
            .into_iter()
            .map(|(thread, day, _)| (thread.to_string(), day))
            .collect::<Vec<_>>()
    );
    let accounting = dump(&mut conn)
        .await?
        .into_iter()
        .filter(|(table, _)| table.starts_with("draft_accounting_"))
        .collect::<Vec<_>>();
    assert_eq!(accounting.len(), 10);
    for table in [
        "attempts",
        "observations",
        "price_bindings",
        "estimates",
        "contributions",
    ] {
        let rows = &accounting
            .iter()
            .find(|(name, _)| name == &format!("draft_accounting_{table}"))
            .context("raw table missing")?
            .1;
        assert!(rows.is_empty(), "{table} retained raw detail");
    }
    conn.close().await?;
    Ok(accounting)
}

#[tokio::test]
async fn accounting_contract_golden_compact_original_bundle_parity() -> anyhow::Result<()> {
    let raw_home = home();
    let direct_home = home();
    let fixture = Fixture::new()?;
    let time = fixture.young + 100 * DAY;
    let raw = open(&raw_home).await?;
    fixture.admit(&raw).await?;
    fixture.deliver(&raw, &REPLAY).await?;
    AccountingStore::open(&raw, fixture.young)
        .await?
        .maintain(time)
        .await?;
    let expected = assert_compact(&raw, &fixture, time).await?;
    raw.close().await;
    let direct = open(&direct_home).await?;
    fixture.create_owners(&direct).await?;
    let store = AccountingStore::open(&direct, time).await?;
    for id in 11..=14 {
        let thread = owner(id)?;
        let bundle = fixture
            .entries
            .iter()
            .filter(|e| e.attempt.thread_id == thread)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            store.import_retained(thread, &bundle, time).await?,
            vec![RetainedImportOutcome::Imported; bundle.len()]
        );
    }
    assert_eq!(assert_compact(&direct, &fixture, time).await?, expected);
    direct.close().await;
    for _ in 0..2 {
        for path in [&*raw_home, &*direct_home] {
            let runtime = open(path).await?;
            let store = AccountingStore::open(&runtime, time).await?;
            let mut conn = connection(&runtime).await?;
            let before = dump(&mut conn).await?;
            for id in 11..=14 {
                let thread = owner(id)?;
                let bundle = fixture
                    .entries
                    .iter()
                    .filter(|e| e.attempt.thread_id == thread)
                    .cloned()
                    .collect::<Vec<_>>();
                assert_eq!(
                    store.import_retained(thread, &bundle, time + DAY).await?,
                    vec![RetainedImportOutcome::SuppressedByReplayRecord; bundle.len()]
                );
            }
            // All-suppressed replay must leave even the checkpoint unchanged.
            assert_eq!(dump(&mut conn).await?, before);
            assert_eq!(
                store
                    .read_day(owner(11)?, fixture.young / DAY, time + DAY)
                    .await?,
                RetainedDay::NeedsMaintenance {
                    completed_as_of_ms: time
                }
            );
            assert_eq!(assert_compact(&runtime, &fixture, time).await?, expected);
            conn.close().await?;
            runtime.close().await;
        }
    }
    Ok(())
}
