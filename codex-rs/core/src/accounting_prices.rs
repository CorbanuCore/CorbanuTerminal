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
    let anthropic_write = anthropic_cache_write_milli(provider, input)
        .filter(|_| provider == codex_model_provider_info::ANTHROPIC_PROVIDER_ID);
    let reference = if free_cache_write {
        serde_json::to_vec(&(v1, "cache_write", 0))?
    } else if let Some((lifetime, write)) = anthropic_write {
        serde_json::to_vec(&(v1, lifetime, write))?
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
            write: if free_cache_write {
                Some(rate(0)?)
            } else {
                anthropic_write.map(|(_, write)| rate(write)).transpose()?
            },
        },
        reference,
        Basis::Billed,
        None,
    )
}

/// Subscription capacity: the plan rate that applied at dispatch, if the
/// catalogue states one, and the API rates it states for the same route, if any.
///
/// An OpenAI metered row reached through a ChatGPT subscription (the vendor
/// published API rates and no plan figure, e.g. GPT-6 Sol) records its API
/// equivalent with no plan rate. That is limited to OpenAI because a
/// subscription-style Codex login classifies every built-in provider's route as
/// plan-priced, and another vendor's metered row there may be paid by that
/// vendor's own API key: recording it as plan work would hide real spend.
/// A row that states neither yields nothing: inventing a burn, or an equivalent
/// for a row with no API price, would put a number in the ledger that no
/// catalogue ever stated.
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
    let burn = billing.plan_burn_millis_at(accepted_at);
    let mut equivalent = billing.api_key_rates_at(accepted_at);
    // A Claude subscription row states only its plan rate. The API price of
    // the model it serves is the Anthropic API row for that model, which is
    // the vendor's own published price, not a guess; this route is always
    // subscription work, so no API-key spend can hide behind it.
    let mut api_twin = None;
    if equivalent.is_none()
        && provider == codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID
        && let Some(api_model) = codex_model_provider_info::claude_plan_api_model(model)
        && let Some(api_billing) = billing_for(
            &catalog.models,
            api_model,
            codex_model_provider_info::ANTHROPIC_PROVIDER_ID,
        )
    {
        equivalent = api_billing.api_key_rates_at(accepted_at);
        api_twin = equivalent.map(|_| api_model);
    }
    if burn.is_none()
        && (equivalent.is_none() || provider != codex_model_provider_info::OPENAI_PROVIDER_ID)
    {
        return Ok(Vec::new());
    }
    let input = equivalent.map(|(input, _, _)| input);
    let output = equivalent.map(|(_, output, _)| output);
    let read = equivalent.and_then(|(_, _, read)| read);
    let write = api_twin
        .and(input)
        .and_then(|input| anthropic_cache_write_milli(provider, input));
    let v1 = (
        "plan-equivalent-bundled-v1",
        provider,
        model,
        "plan",
        burn,
        "USD/million",
        input,
        output,
        read,
    );
    // Rows priced by their own catalogue entry keep their v1 identity.
    let reference = match (api_twin, write) {
        (Some(api_model), Some((lifetime, write))) => {
            serde_json::to_vec(&(v1, "api_row", api_model, lifetime, write))?
        }
        (Some(api_model), None) => serde_json::to_vec(&(v1, "api_row", api_model))?,
        (None, _) => serde_json::to_vec(&v1)?,
    };
    snapshot(
        model,
        provider,
        scope,
        accepted_at,
        Rates {
            noncached: input.map(rate).transpose()?,
            output: output.map(rate).transpose()?,
            read: read.map(rate).transpose()?,
            write: write.map(|(_, write)| rate(write)).transpose()?,
        },
        reference,
        Basis::PlanEquivalent,
        burn,
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

/// Anthropic's published cache-write price, as a multiple of the input price,
/// for the cache lifetime this client requests on the route: five minutes
/// (1.25x) on an API key, one hour (2x) on a Claude subscription. Source:
/// platform.claude.com/docs/en/about-claude/pricing. `None` when the multiple
/// is not exact in the catalogue's milli-USD unit.
fn anthropic_cache_write_milli(provider: &str, input: u32) -> Option<(&'static str, u32)> {
    match provider {
        codex_model_provider_info::ANTHROPIC_PROVIDER_ID if input.is_multiple_of(4) => {
            Some(("cache_write_5m", input / 4 * 5))
        }
        codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID => {
            input.checked_mul(2).map(|write| ("cache_write_1h", write))
        }
        _ => None,
    }
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
