//! Seeds a disposable Corbanu home's accounting ledger for the PF-60-S03 demo
//! videos (`qa/demos/specs/pf60s03-*.toml`). Synthetic attempts only; never
//! point it at a real home.
//!
//! ```text
//! accounting_demo_seed <sqlite-home> others
//! accounting_demo_seed <sqlite-home> history <owner-thread-id>
//! ```
//!
//! `others` adds two other conversations today, the same shape as the
//! `cost_scope` tmux test: two priced OpenAI requests, and one request on a
//! custom provider with no published price. `history` adds requests owned by
//! an existing conversation on each of the 14 UTC days before today.

use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use codex_protocol::ThreadId;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::StateRuntime;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::Attempt;
use codex_state::accounting::Observation;
use codex_state::accounting::Snapshot;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde_json::json;
use uuid::Uuid;

const DAY_MS: i64 = 86_400_000;

/// A published per-million-token price, in force from the epoch.
fn price(
    provider: &str,
    model: &str,
    noncached: &str,
    read: &str,
    output: &str,
) -> Result<Snapshot> {
    Ok(serde_json::from_value(json!({
        "id": Uuid::new_v4(), "provider": provider, "model": model, "scope": Uuid::nil(),
        "currency": "USD", "unit": "PerMillionTokens",
        "rates": {"noncached": noncached, "read": read, "write": null, "output": output},
        "source_reference": Uuid::new_v4(), "source_kind": "ProviderPublished",
        "observed_at_ms": 0, "approved_at_ms": 0, "effective_from_ms": 0, "effective_end_ms": null
    }))?)
}

struct Seed<'a> {
    owner: ThreadId,
    provider: &'a str,
    model: &'a str,
    prices: Vec<Snapshot>,
    input: i64,
    read: i64,
    output: i64,
    dispatched_ms: i64,
}

async fn record(store: &AccountingStore<'_>, seed: Seed<'_>, now: i64) -> Result<()> {
    let attempt: Attempt = serde_json::from_value(json!({
        "attempt_id": Uuid::new_v4(), "request_id": Uuid::new_v4(), "thread_id": seed.owner,
        "turn": "demo-seed", "retry_of": null, "provider": seed.provider, "model": seed.model,
        "scope": Uuid::nil(), "dialect": "Inclusive", "dispatched_at_ms": seed.dispatched_ms
    }))?;
    let observation: Observation = serde_json::from_value(json!({
        "revision": 1, "source": attempt.attempt_id, "sequence": 1,
        "patch": {"input": seed.input, "read": seed.read, "write": 0, "output": seed.output}
    }))?;
    store.admit(seed.owner, &attempt, &seed.prices, now).await?;
    store
        .observe(seed.owner, &attempt, &[observation], now)
        .await?;
    Ok(())
}

async fn add_thread(runtime: &StateRuntime, home: &std::path::Path) -> Result<ThreadId> {
    let thread = ThreadId::new();
    let mut builder = ThreadMetadataBuilder::new(
        thread,
        home.join(format!("{thread}.jsonl")),
        chrono::Utc::now(),
        SessionSource::Cli,
    );
    builder.cwd = home.to_path_buf();
    runtime.upsert_thread(&builder.build("openai")).await?;
    Ok(thread)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (home, scenario) = match args.as_slice() {
        [home, scenario, ..] => (PathBuf::from(home), scenario.as_str()),
        _ => anyhow::bail!("usage: accounting_demo_seed <sqlite-home> others|history [owner]"),
    };
    let runtime = StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.clone())?),
        "openai".into(),
    )
    .await?;
    let now = chrono::Utc::now().timestamp_millis();
    let today = now / DAY_MS * DAY_MS;
    let store = AccountingStore::open(&runtime, now).await?;
    let openai = || price("openai", "gpt-5.4", "5", "0.5", "30");
    match scenario {
        "others" => {
            let (a, b) = (
                add_thread(&runtime, &home).await?,
                add_thread(&runtime, &home).await?,
            );
            let dispatched_ms = (now - 1_000).max(today);
            for owner in [a, a] {
                let seed = Seed {
                    owner,
                    provider: "openai",
                    model: "gpt-5.4",
                    prices: vec![openai()?],
                    input: 100,
                    read: 20,
                    output: 10,
                    dispatched_ms,
                };
                record(&store, seed, now).await?;
            }
            let seed = Seed {
                owner: b,
                provider: "local-mock",
                model: "mock-model",
                prices: Vec::new(),
                input: 50,
                read: 0,
                output: 5,
                dispatched_ms,
            };
            record(&store, seed, now).await?;
        }
        "history" => {
            let owner = args
                .get(2)
                .context("history needs the owner thread id")
                .and_then(|id| ThreadId::from_string(id.trim()).context("owner thread id"))?;
            for back in 1..=14_i64 {
                let day = today - back * DAY_MS;
                // Two GLM requests every day, an OpenAI request every third day.
                for (hour, input) in [(9, 12_000 + back * 300), (15, 8_000 + back * 150)] {
                    let seed = Seed {
                        owner,
                        provider: "zai",
                        model: "glm-5.2",
                        prices: vec![price("zai", "glm-5.2", "1.4", "0.26", "4.4")?],
                        input,
                        read: input / 4,
                        output: input / 10,
                        dispatched_ms: day + hour * 3_600_000,
                    };
                    record(&store, seed, now).await?;
                }
                if back % 3 == 0 {
                    let seed = Seed {
                        owner,
                        provider: "openai",
                        model: "gpt-5.4",
                        prices: vec![openai()?],
                        input: 20_000,
                        read: 5_000,
                        output: 1_500,
                        dispatched_ms: day + 11 * 3_600_000,
                    };
                    record(&store, seed, now).await?;
                }
            }
        }
        other => anyhow::bail!("unknown scenario {other}; use others or history"),
    }
    runtime.close().await;
    println!("seeded {scenario}");
    Ok(())
}
