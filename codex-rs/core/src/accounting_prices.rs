//! Prospective exact estimates from the bundled authority, never model discovery.
use codex_protocol::openai_models::ModelBilling;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use codex_state::accounting::Basis;
use codex_state::accounting::Currency;
use codex_state::accounting::Decimal;
use codex_state::accounting::Rates;
use codex_state::accounting::Snapshot;
use codex_state::accounting::SourceKind;
use codex_state::accounting::Unit;
use uuid::Uuid;

/// Providers whose published price sheet bills input only as a cache hit or a
/// cache miss, with no cache-write charge. DeepSeek:
/// api-docs.deepseek.com/quick_start/pricing. Baseten Model APIs:
/// baseten.co/pricing (input, cache input, output). Not Moonshot (the K3 sheet
/// charges cache writes), OpenAI (GPT-5.6+ bills cache writes), or the routers
/// (OpenRouter, Vercel), which pass upstream cache-write charges through.
const NO_CACHE_WRITE_CHARGE: [&str; 2] = [
    codex_model_provider_info::DEEPSEEK_PROVIDER_ID,
    codex_model_provider_info::BASETEN_PROVIDER_ID,
];

/// Rates the provider charges this route, for a turn it bills per token.
pub(super) fn original(
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
    source: &str,
) -> anyhow::Result<Vec<Snapshot>> {
    let catalog = codex_models_manager::bundled_models_response()?;
    let Some(billing) = billing_for(&catalog.models, model, provider) else {
        return Ok(Vec::new());
    };
    billed(model, provider, &billing, scope, accepted_at, source)
}

/// Rates for a turn billed per token, from an already-resolved catalogue row.
fn billed(
    model: &str,
    provider: &str,
    billing: &ModelBilling,
    scope: Uuid,
    accepted_at: i64,
    source: &str,
) -> anyhow::Result<Vec<Snapshot>> {
    // A plan row states no per-token price, and a plan turn is not billed per
    // token: either way there is nothing here to charge.
    let Some((input, output, read)) = billing.api_key_rates_at(accepted_at) else {
        return Ok(Vec::new());
    };
    // Canonical tuple version is part of provenance. UUIDv5 is a content identity,
    // not an authenticity claim. Null cache-write is deliberate in projection v1;
    // a provider whose price sheet has no cache-write charge states zero, which
    // extends the tuple so every other identity is unchanged.
    let free_cache_write = NO_CACHE_WRITE_CHARGE.contains(&provider);
    let v1 = (
        source,
        provider,
        model,
        "api_key",
        "default",
        "USD/million",
        input,
        output,
        read,
    );
    let reference = if free_cache_write {
        serde_json::to_vec(&(v1, "cache_write", 0))?
    } else {
        serde_json::to_vec(&v1)?
    };
    snapshot(
        model,
        provider,
        scope,
        accepted_at,
        Rates {
            noncached: Some(rate(input)?),
            output: Some(rate(output)?),
            read: read.map(rate).transpose()?,
            write: free_cache_write.then(|| rate(0)).transpose()?,
        },
        reference,
        Basis::Billed,
        None,
    )
}

/// Subscription capacity: the plan rate that applied at dispatch, and the API
/// rates the catalogue states for the same route, if it states any.
///
/// A row with no plan side yields nothing. Inventing a burn for a metered row,
/// or an API equivalent for a row that states no API price, would put a number
/// in the ledger that no catalogue ever stated.
pub(super) fn plan_original(
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
    tier: Option<&str>,
) -> anyhow::Result<Vec<Snapshot>> {
    if !matches!(tier, None | Some("default")) {
        return Ok(Vec::new());
    }
    let catalog = codex_models_manager::bundled_models_response()?;
    let Some(billing) = billing_for(&catalog.models, model, provider) else {
        return Ok(Vec::new());
    };
    let Some(burn) = billing.plan_burn_millis_at(accepted_at) else {
        return Ok(Vec::new());
    };
    let equivalent = billing.api_key_rates_at(accepted_at);
    let input = equivalent.map(|(input, _, _)| input);
    let output = equivalent.map(|(_, output, _)| output);
    let read = equivalent.and_then(|(_, _, read)| read);
    let reference = serde_json::to_vec(&(
        "plan-equivalent-bundled-v1",
        provider,
        model,
        "plan",
        burn,
        "USD/million",
        input,
        output,
        read,
    ))?;
    snapshot(
        model,
        provider,
        scope,
        accepted_at,
        Rates {
            noncached: input.map(rate).transpose()?,
            output: output.map(rate).transpose()?,
            read: read.map(rate).transpose()?,
            write: None,
        },
        reference,
        Basis::PlanEquivalent,
        Some(burn),
    )
}

/// The catalogue's billing for exactly this provider's row, or nothing.
///
/// The slug must be unambiguous and the row must belong to the provider the
/// attempt actually used: a rate from another provider's row would be a guess
/// wearing this provider's name.
fn billing_for(
    rows: &[codex_protocol::openai_models::ModelInfo],
    model: &str,
    provider: &str,
) -> Option<ModelBilling> {
    let mut matches = rows.iter().filter(|row| row.slug == model);
    let row = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let ModelOrchestrationMetadata::Eligible {
        provider_id,
        billing,
        ..
    } = row.orchestration.as_ref()?
    else {
        return None;
    };
    (provider_id == provider).then(|| billing.clone())
}

#[expect(clippy::too_many_arguments)]
fn snapshot(
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
    rates: Rates,
    reference: Vec<u8>,
    basis: Basis,
    plan_burn_millis: Option<u32>,
) -> anyhow::Result<Vec<Snapshot>> {
    let time = accepted_at.try_into()?;
    Ok(vec![Snapshot {
        id: Uuid::new_v4(),
        provider: provider.into(),
        model: model.into(),
        scope,
        currency: Currency::Usd,
        unit: Unit::PerMillionTokens,
        rates,
        source_reference: Uuid::new_v5(&Uuid::NAMESPACE_OID, &reference),
        source_kind: SourceKind::NativeCatalog,
        basis,
        plan_burn_millis,
        observed_at_ms: time,
        approved_at_ms: time,
        effective_from_ms: time,
        effective_end_ms: None,
    }])
}

pub(super) fn responses_original(
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
    tier: Option<&str>,
) -> anyhow::Result<Vec<Snapshot>> {
    if !matches!(tier, None | Some("default")) {
        return Ok(Vec::new());
    }
    original(
        model,
        provider,
        scope,
        accepted_at,
        "openai-responses-api-key-bundled-v1",
    )
}

pub(super) fn chat_original(
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    original(
        model,
        provider,
        scope,
        accepted_at,
        "openai-chat-api-key-bundled-v1",
    )
}

pub(super) fn anthropic_original(
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    original(model, provider, scope, accepted_at, "anthropic-bundled-v1")
}

#[cfg(test)]
fn responses_project(
    model: &str,
    scope: Uuid,
    billing: &ModelBilling,
    accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    billed(
        model,
        "openai",
        billing,
        scope,
        accepted_at,
        "openai-responses-api-key-bundled-v1",
    )
}

fn rate(milli: u32) -> anyhow::Result<Decimal> {
    let whole = milli / 1000;
    let fraction = milli % 1000;
    serde_json::from_value(serde_json::Value::String(format!("{whole}.{fraction:03}")))
        .map_err(Into::into)
}

#[cfg(test)]
#[path = "accounting_prices_tests.rs"]
mod tests;
