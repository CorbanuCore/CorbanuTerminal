//! Prospective exact estimates from the bundled authority, never model discovery.
use codex_protocol::openai_models::ModelBilling;
use codex_state::accounting::Basis;
use codex_state::accounting::Currency;
use codex_state::accounting::Decimal;
use codex_state::accounting::LongContext;
use codex_state::accounting::Rates;
use codex_state::accounting::Snapshot;
use codex_state::accounting::SourceKind;
use codex_state::accounting::Unit;
use uuid::Uuid;

/// Providers whose published price sheet bills input only as a cache hit or a
/// cache miss, with no cache-write charge. DeepSeek:
/// api-docs.deepseek.com/quick_start/pricing. Baseten Model APIs:
/// baseten.co/pricing (input, cache input, output). Z.AI API:
/// docs.z.ai/guides/overview/pricing (input, cached input, output; cache
/// storage listed as free). Not Moonshot (the K3 sheet
/// charges cache writes), OpenAI (GPT-5.6+ bills cache writes), or the routers
/// (OpenRouter, Vercel), which pass upstream cache-write charges through.
const NO_CACHE_WRITE_CHARGE: [&str; 3] = [
    codex_model_provider_info::DEEPSEEK_PROVIDER_ID,
    codex_model_provider_info::BASETEN_PROVIDER_ID,
    codex_model_provider_info::ZAI_PROVIDER_ID,
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
    let stated = Stated::of(billing);
    let reference = if let Some(extended) = stated.reference(&v1)? {
        extended
    } else if free_cache_write {
        serde_json::to_vec(&(v1, "cache_write", 0))?
    } else if let Some((lifetime, write)) = anthropic_write {
        serde_json::to_vec(&(v1, lifetime, write))?
    } else {
        serde_json::to_vec(&v1)?
    };
    let write = match stated.write {
        Some(write) => Some(write),
        None if free_cache_write => Some(0),
        None => anthropic_write.map(|(_, write)| write),
    };
    let mut prices = snapshot(
        model,
        provider,
        scope,
        accepted_at,
        Rates {
            noncached: Some(rate(input)?),
            output: Some(rate(output)?),
            read: read.map(rate).transpose()?,
            write: write.map(rate).transpose()?,
        },
        reference,
        Basis::Billed,
        /*plan_burn_millis*/ None,
    )?;
    stated.bind_long_context(&mut prices)?;
    Ok(prices)
}

/// What a catalogue row states beyond input, output and cache-read rates:
/// its cache-write rate and its long-context tier. A row that states neither
/// keeps the price identity it had before the catalogue could state them.
struct Stated {
    write: Option<u32>,
    long_context: Option<codex_protocol::openai_models::LongContextRates>,
}

impl Stated {
    fn of(billing: &ModelBilling) -> Self {
        Self {
            write: billing.api_key_cache_write(),
            long_context: billing.api_key_long_context(),
        }
    }

    /// The price identity extended with the stated rates, or `None` when the
    /// row states neither.
    fn reference(&self, v1: &impl serde::Serialize) -> anyhow::Result<Option<Vec<u8>>> {
        if self.write.is_none() && self.long_context.is_none() {
            return Ok(None);
        }
        let long = self.long_context.map(|tier| {
            (
                tier.above_input_tokens,
                tier.input_milli_usd_per_million_tokens,
                tier.output_milli_usd_per_million_tokens,
                tier.cached_input_milli_usd_per_million_tokens,
                tier.cache_write_milli_usd_per_million_tokens,
            )
        });
        Ok(Some(serde_json::to_vec(&(
            v1,
            "stated-v1",
            self.write,
            long,
        ))?))
    }

    fn bind_long_context(&self, prices: &mut [Snapshot]) -> anyhow::Result<()> {
        let Some(tier) = self.long_context else {
            return Ok(());
        };
        let long = LongContext {
            above_input_tokens: i64::from(tier.above_input_tokens).try_into()?,
            rates: Rates {
                noncached: Some(rate(tier.input_milli_usd_per_million_tokens)?),
                read: tier
                    .cached_input_milli_usd_per_million_tokens
                    .map(rate)
                    .transpose()?,
                write: tier
                    .cache_write_milli_usd_per_million_tokens
                    .map(rate)
                    .transpose()?,
                output: Some(rate(tier.output_milli_usd_per_million_tokens)?),
            },
        };
        for price in prices {
            price.long_context = Some(long.clone());
        }
        Ok(())
    }
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
    // A row priced by its own catalogue entry states its own cache-write rate
    // and long-context tier, if any.
    let stated = match (api_twin, equivalent) {
        (None, Some(_)) => Stated::of(&billing),
        _ => Stated {
            write: None,
            long_context: None,
        },
    };
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
    // Rows priced by their own catalogue entry keep their v1 identity unless
    // they state a cache-write rate or a long-context tier.
    let reference = match (api_twin, write) {
        (Some(api_model), Some((lifetime, write))) => {
            serde_json::to_vec(&(v1, "api_row", api_model, lifetime, write))?
        }
        (Some(api_model), None) => serde_json::to_vec(&(v1, "api_row", api_model))?,
        (None, _) => match stated.reference(&v1)? {
            Some(extended) => extended,
            None => serde_json::to_vec(&v1)?,
        },
    };
    let write = write.map(|(_, write)| write).or(stated.write);
    let mut prices = snapshot(
        model,
        provider,
        scope,
        accepted_at,
        Rates {
            noncached: input.map(rate).transpose()?,
            output: output.map(rate).transpose()?,
            read: read.map(rate).transpose()?,
            write: write.map(rate).transpose()?,
        },
        reference,
        Basis::PlanEquivalent,
        burn,
    )?;
    stated.bind_long_context(&mut prices)?;
    Ok(prices)
}

/// A price record that states only the attempt's billing basis: subscription
/// work the catalogue gives no figure for, local work, or work on a route with
/// no declared basis. It carries no rates, so it never invents a number; it
/// exists so the basis is bound to the attempt even without a price.
pub(super) fn basis_only(
    basis: Basis,
    source: codex_state::accounting::BasisSource,
    model: &str,
    provider: &str,
    scope: Uuid,
    accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    let reference = serde_json::to_vec(&("basis-only-v1", provider, model, basis, source))?;
    snapshot(
        model,
        provider,
        scope,
        accepted_at,
        Rates {
            noncached: None,
            read: None,
            write: None,
            output: None,
        },
        reference,
        basis,
        /*plan_burn_millis*/ None,
    )
}

/// The catalogue's billing for exactly this provider's row, or nothing. A
/// row that is not spawn-eligible still states its published price.
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
    let orchestration = row.orchestration.as_ref()?;
    let billing = orchestration.accounting_billing()?;
    (orchestration.provider_id() == provider).then(|| billing.clone())
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
        basis_source: codex_state::accounting::BasisSource::BuiltIn,
        long_context: None,
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
