//! Prospective exact estimates from the bundled authority, never model discovery.
use codex_protocol::openai_models::ModelBilling;
use codex_protocol::openai_models::ModelOrchestrationMetadata;
use codex_state::accounting::Currency;
use codex_state::accounting::Decimal;
use codex_state::accounting::Rates;
use codex_state::accounting::Snapshot;
use codex_state::accounting::SourceKind;
use codex_state::accounting::Unit;
use uuid::Uuid;

pub(super) fn original(
    model: &str,
    scope: Uuid,
    accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    let catalog = codex_models_manager::bundled_models_response()?;
    let mut matches = catalog.models.iter().filter(|row| row.slug == model);
    let Some(row) = matches.next() else {
        return Ok(Vec::new());
    };
    if matches.next().is_some() {
        return Ok(Vec::new());
    }
    let Some(ModelOrchestrationMetadata::Eligible {
        provider_id,
        billing,
        ..
    }) = &row.orchestration
    else {
        return Ok(Vec::new());
    };
    if provider_id != "anthropic" {
        return Ok(Vec::new());
    }
    project(model, scope, billing, accepted_at)
}

fn project(
    model: &str,
    scope: Uuid,
    billing: &ModelBilling,
    accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    let ModelBilling::Metered {
        input_milli_usd_per_million_tokens: input,
        output_milli_usd_per_million_tokens: output,
        cached_input_milli_usd_per_million_tokens: read,
    } = billing
    else {
        return Ok(Vec::new());
    };
    // Canonical tuple version is part of provenance. UUIDv5 is a content identity,
    // not an authenticity claim. Null cache-write is deliberate in projection v1.
    let source = serde_json::to_vec(&(
        "anthropic-bundled-v1",
        "anthropic",
        model,
        "USD/million",
        input,
        output,
        read,
    ))?;
    let time = accepted_at.try_into()?;
    Ok(vec![Snapshot {
        id: Uuid::new_v4(),
        provider: "anthropic".into(),
        model: model.into(),
        scope,
        currency: Currency::Usd,
        unit: Unit::PerMillionTokens,
        rates: Rates {
            noncached: Some(rate(*input)?),
            output: Some(rate(*output)?),
            read: read.map(rate).transpose()?,
            write: None,
        },
        source_reference: Uuid::new_v5(&Uuid::NAMESPACE_OID, &source),
        source_kind: SourceKind::NativeCatalog,
        observed_at_ms: time,
        approved_at_ms: time,
        effective_from_ms: time,
        effective_end_ms: None,
    }])
}

pub(super) fn responses_original(
    model: &str,
    scope: Uuid,
    accepted_at: i64,
    tier: Option<&str>,
) -> anyhow::Result<Vec<Snapshot>> {
    if !matches!(tier, None | Some("default")) {
        return Ok(Vec::new());
    }
    openai_original(model, scope, accepted_at, "openai-responses-api-key-bundled-v1")
}

pub(super) fn chat_original(model: &str, scope: Uuid, accepted_at: i64) -> anyhow::Result<Vec<Snapshot>> {
    openai_original(model, scope, accepted_at, "openai-chat-api-key-bundled-v1")
}

fn openai_original(model: &str, scope: Uuid, accepted_at: i64, source: &str) -> anyhow::Result<Vec<Snapshot>> {
    let catalog = codex_models_manager::bundled_models_response()?;
    openai_rows(&catalog.models, model, scope, accepted_at, source)
}

fn openai_rows(rows: &[codex_protocol::openai_models::ModelInfo], model: &str, scope: Uuid,
    accepted_at: i64, source: &str) -> anyhow::Result<Vec<Snapshot>> {
    let mut rows = rows.iter().filter(|row| row.slug == model);
    let Some(row) = rows.next() else {
        return Ok(Vec::new());
    };
    if rows.next().is_some() {
        return Ok(Vec::new());
    }
    let Some(ModelOrchestrationMetadata::Eligible {
        provider_id,
        billing,
        ..
    }) = &row.orchestration
    else {
        return Ok(Vec::new());
    };
    if provider_id != "openai" {
        return Ok(Vec::new());
    }
    openai_project(model, scope, billing, accepted_at, source)
}

#[cfg(test)]
fn responses_project(
    model: &str, scope: Uuid, billing: &ModelBilling, accepted_at: i64,
) -> anyhow::Result<Vec<Snapshot>> {
    openai_project(model, scope, billing, accepted_at, "openai-responses-api-key-bundled-v1")
}

fn openai_project(
    model: &str, scope: Uuid, billing: &ModelBilling, accepted_at: i64, source: &str,
) -> anyhow::Result<Vec<Snapshot>> {
    let (input, output, read) = match billing {
        ModelBilling::Metered {
            input_milli_usd_per_million_tokens,
            output_milli_usd_per_million_tokens,
            cached_input_milli_usd_per_million_tokens,
        } => (
            *input_milli_usd_per_million_tokens,
            *output_milli_usd_per_million_tokens,
            *cached_input_milli_usd_per_million_tokens,
        ),
        ModelBilling::AuthDependent {
            api_key_input_milli_usd_per_million_tokens,
            api_key_output_milli_usd_per_million_tokens,
            api_key_cached_input_milli_usd_per_million_tokens,
            ..
        } => (
            *api_key_input_milli_usd_per_million_tokens,
            *api_key_output_milli_usd_per_million_tokens,
            *api_key_cached_input_milli_usd_per_million_tokens,
        ),
        ModelBilling::Plan { .. } | ModelBilling::PlanSchedule { .. } | ModelBilling::Local => {
            return Ok(Vec::new());
        }
    };
    let mut snapshots = project(
        model,
        scope,
        &ModelBilling::Metered {
            input_milli_usd_per_million_tokens: input,
            output_milli_usd_per_million_tokens: output,
            cached_input_milli_usd_per_million_tokens: read,
        },
        accepted_at,
    )?;
    let source = serde_json::to_vec(&(
        source,
        "openai",
        model,
        "api_key",
        "default",
        "USD/million",
        input,
        output,
        read,
    ))?;
    snapshots[0].provider = "openai".into();
    snapshots[0].source_reference = Uuid::new_v5(&Uuid::NAMESPACE_OID, &source);
    Ok(snapshots)
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
