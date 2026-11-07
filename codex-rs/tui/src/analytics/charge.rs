//! How `/cost` prices a request on a route that reports its charge (#368).
//!
//! Travis, 2026-10-14: OpenRouter first, then the Vercel AI Gateway and the
//! Corbanu API by the same rule. These routes state what they charged with each
//! response (`usage.cost` or the gateway's own figure, recorded as
//! `billed_usd`), and that charge is exact where a catalogue rate is not:
//! OpenRouter's price depends on which upstream endpoint served the request.
//! So a request is priced from that charge. Without one, the catalogue rate
//! bound at dispatch stands, shown as an estimate; with neither, there is no
//! price.
//!
//! The ledger is unchanged: its quote keeps the catalogue estimate and the
//! charge stays in the attempt's usage. This view puts the charge in as the
//! attempt's figure before anything is summed, so every total, page and link
//! reads the same amount, and each page says which basis it used.

use codex_state::accounting::BucketQuote;
use codex_state::accounting::DayTotals;
use codex_state::accounting::Decimal;
use codex_state::accounting::Inspection;
use codex_state::accounting::InspectionDay;
use codex_state::accounting::ObservationQuote;

use super::joined;
use super::lacks_price;
use super::money;
use super::per_use;
use super::provider_names;

/// Routes that state what they charged with each response, and so are priced
/// from that charge. Only these record `billed_usd` (core `accounting_chat.rs`,
/// `accounting_responses.rs` and `accounting.rs`); keep the lists in step.
pub(super) const CHARGE_REPORTING: [&str; 4] = [
    codex_model_provider_info::OPENROUTER_PROVIDER_ID,
    codex_model_provider_info::VERCEL_PROVIDER_ID,
    codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID,
    codex_model_provider_info::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
];

/// Whether `provider`'s requests are priced from the charge it reports.
pub(super) fn prices_from_charge(provider: &str) -> bool {
    CHARGE_REPORTING.contains(&provider)
}

/// Display names of `providers`, joined ("OpenRouter", "OpenRouter and Vercel
/// AI Gateway"), or "the provider" when there are none.
pub(super) fn named<'a>(providers: impl IntoIterator<Item = &'a str>) -> String {
    let names = provider_names(providers);
    if names.is_empty() {
        "the provider".to_string()
    } else {
        joined(&names)
    }
}

/// Which charge-reporting routes priced some of the attempts summed somewhere:
/// a set over [`CHARGE_REPORTING`], so the summary that carries it stays `Copy`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Chargers(u8);

// One bit per charge-reporting route.
const _: () = assert!(CHARGE_REPORTING.len() <= u8::BITS as usize);

impl Chargers {
    /// This set with `provider` added, if it is a charge-reporting route.
    pub(super) fn with(self, provider: &str) -> Self {
        CHARGE_REPORTING
            .iter()
            .position(|id| *id == provider)
            .map_or(self, |index| Self(self.0 | 1 << index))
    }

    /// The providers whose reported charge priced one of `quotes`.
    pub(super) fn of<'a>(quotes: impl IntoIterator<Item = &'a ObservationQuote>) -> Self {
        quotes
            .into_iter()
            .filter(|quote| reported_charge(quote).is_some())
            .fold(Self::default(), |set, quote| {
                set.with(&quote.attempt.provider)
            })
    }

    fn ids(self) -> impl Iterator<Item = &'static str> {
        CHARGE_REPORTING
            .iter()
            .enumerate()
            .filter(move |(index, _)| self.0 & (1 << index) != 0)
            .map(|(_, id)| *id)
    }

    /// Their display names, joined.
    pub(super) fn names(self) -> String {
        named(self.ids())
    }

    /// Whether they are more than one provider by name.
    pub(super) fn several(self) -> bool {
        provider_names(self.ids()).len() > 1
    }
}

/// The response field each charge-reporting route states its charge in.
fn charge_field(provider: &str) -> &'static str {
    match provider {
        codex_model_provider_info::VERCEL_PROVIDER_ID => "gatewayCost",
        // The Chat route falls back to `usage.cost` without the Corbanu field
        // (codex-api `chat_accounting.rs`); the Messages route has only it.
        codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID => {
            "usage.corbanu_charge_usd or usage.cost"
        }
        codex_model_provider_info::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID => {
            "usage.corbanu_charge_usd"
        }
        _ => "usage.cost",
    }
}

/// The charge this attempt is priced from: pay-per-use work on a route priced
/// from its reported charges, not refused, whose response stated one.
pub(super) fn reported_charge(quote: &ObservationQuote) -> Option<Decimal> {
    (prices_from_charge(&quote.attempt.provider)
        && per_use(quote)
        && quote.usage.refused_status.is_none())
    .then_some(quote.usage.billed_usd)
    .flatten()
}

/// What the published prices put the attempt at, the figure a reported charge
/// replaces, or why there is no such figure.
pub(super) fn published_estimate(quote: &ObservationQuote) -> String {
    if quote
        .buckets
        .iter()
        .any(|bucket| matches!(bucket, BucketQuote::MissingRate))
    {
        return "none — no published price covers every component".to_string();
    }
    if quote
        .buckets
        .iter()
        .any(|bucket| matches!(bucket, BucketQuote::MissingUsage))
    {
        return "not available — the response did not report every token count its published price needs"
            .to_string();
    }
    quote
        .buckets
        .iter()
        .try_fold(Decimal::default(), |sum, bucket| match bucket {
            BucketQuote::Priced(amount) => sum.add(*amount).ok(),
            BucketQuote::MissingUsage | BucketQuote::MissingRate => None,
        })
        .map_or_else(
            || "unavailable — exact arithmetic overflow".to_string(),
            money,
        )
}

/// Price the attempt from its reported charge. Returns whether it changed;
/// applying it again changes nothing.
fn reprice(quote: &mut ObservationQuote) -> bool {
    let Some(charge) = reported_charge(quote) else {
        return false;
    };
    if quote.known_subtotal == charge && quote.all_buckets_priced == Some(charge) {
        return false;
    }
    quote.known_subtotal = charge;
    quote.all_buckets_priced = Some(charge);
    quote.subtotal_display = charge.display();
    true
}

/// Price every attempt in `view` from its reported charge where it has one.
/// Returns whether any attempt in the view's own totals changed.
fn reprice_view(view: &mut Inspection) -> bool {
    let mut changed = false;
    for quote in view
        .requests
        .values_mut()
        .chain(view.unknown_parent_requests.values_mut())
        .flatten()
    {
        changed |= reprice(quote);
    }
    if let Some(others) = &mut view.other_conversations {
        // Other conversations have no stored totals here; their figures are
        // summed from these quotes when shown.
        for quote in others.requests.values_mut().flatten() {
            reprice(quote);
        }
    }
    changed
}

/// `day` with every attempt priced from its reported charge where it has one.
/// A single day's totals are summed again where any attempt changed, and fail
/// only when the exact sums overflow. A range's days keep their stored totals:
/// the range view sums every total it shows from the attempts, bucket by
/// bucket, so an overflow there stays confined to its bucket.
pub(super) fn priced_from_charges(day: InspectionDay) -> anyhow::Result<InspectionDay> {
    Ok(match day {
        InspectionDay::Ready(mut view) => {
            if reprice_view(&mut view) {
                let owner = view.owner;
                let all = || view.requests.values().flatten();
                view.totals = DayTotals::from_quotes(all())?;
                view.own_totals =
                    DayTotals::from_quotes(all().filter(|q| q.attempt.thread_id == owner))?;
                view.descendant_totals =
                    DayTotals::from_quotes(all().filter(|q| q.attempt.thread_id != owner))?;
                view.unknown_parent_totals =
                    DayTotals::from_quotes(view.unknown_parent_requests.values().flatten())?;
            }
            InspectionDay::Ready(view)
        }
        InspectionDay::Range {
            requested,
            aggregate_day_floor,
            completed_as_of_ms,
            read_at_ms,
            mut buckets,
        } => {
            for day in buckets.iter_mut().flat_map(|bucket| &mut bucket.days) {
                if let InspectionDay::Ready(view) = day {
                    reprice_view(view);
                }
            }
            InspectionDay::Range {
                requested,
                aggregate_day_floor,
                completed_as_of_ms,
                read_at_ms,
                buckets,
            }
        }
        other => other,
    })
}

/// How one attempt without a reported charge is priced.
enum Fallback {
    /// Every component priced from published prices.
    Estimate,
    /// Only some components priced: a lower bound.
    Partial,
    /// No figure at all.
    None,
}

fn fallback(quote: &ObservationQuote) -> Fallback {
    if quote.all_buckets_priced.is_some() {
        Fallback::Estimate
    } else if quote.known_subtotal != Decimal::default() {
        Fallback::Partial
    } else {
        Fallback::None
    }
}

/// "Price basis: …" for pages holding work on a route priced from reported
/// charges: whether each figure is the provider's charge, an estimate from
/// published prices, or no price. `None` for any other page, whose figures
/// already say they are estimates.
pub(super) fn price_basis(quotes: &[&ObservationQuote]) -> Option<String> {
    let billable: Vec<&ObservationQuote> = quotes
        .iter()
        .copied()
        .filter(|quote| per_use(quote) && quote.usage.refused_status.is_none())
        .collect();
    if !billable
        .iter()
        .any(|quote| prices_from_charge(&quote.attempt.provider))
    {
        return None;
    }
    // A single attempt here is on a charge-priced route.
    if let [quote] = billable.as_slice() {
        let provider = named([quote.attempt.provider.as_str()]);
        if reported_charge(quote).is_some() {
            return Some(format!(
                "Price basis: the charge {provider} reported with the response ({})",
                charge_field(&quote.attempt.provider)
            ));
        }
        return Some(match fallback(quote) {
            Fallback::Estimate => format!(
                "Price basis: estimate from published prices — {provider} reported no charge for this response"
            ),
            Fallback::Partial => format!(
                "Price basis: partial estimate from published prices, a lower bound — {provider} reported no charge for this response"
            ),
            Fallback::None => format!(
                "Price basis: none — {provider} reported no charge, and {}",
                if lacks_price(quote) {
                    "no published price covers this response"
                } else {
                    "the response did not report every token count its published price needs"
                }
            ),
        });
    }
    let (mut charged, mut estimated, mut partial, mut silent, mut none) = (0, 0, 0, 0, 0);
    let mut silent_providers = Vec::new();
    for quote in &billable {
        if reported_charge(quote).is_some() {
            charged += 1;
            continue;
        }
        match fallback(quote) {
            Fallback::Estimate => estimated += 1,
            Fallback::Partial => partial += 1,
            Fallback::None => none += 1,
        }
        if prices_from_charge(&quote.attempt.provider) {
            silent += 1;
            silent_providers.push(quote.attempt.provider.as_str());
        }
    }
    let attempts = |n: usize| format!("{n} {}", if n == 1 { "attempt" } else { "attempts" });
    let mut parts = Vec::new();
    if charged > 0 {
        parts.push(format!(
            "{} priced from the charge {} reported",
            attempts(charged),
            Chargers::of(billable.iter().copied()).names()
        ));
    }
    if estimated > 0 {
        parts.push(format!(
            "{} estimated from published prices",
            attempts(estimated)
        ));
    }
    if partial > 0 {
        parts.push(format!(
            "{} estimated in part from published prices (a lower bound)",
            attempts(partial)
        ));
    }
    if none > 0 {
        parts.push(format!("{} with no price", attempts(none)));
    }
    let mut basis = format!("Price basis: {}", parts.join("; "));
    if silent > 0 {
        basis.push_str(&format!(
            " — {} reported no charge for {}",
            named(silent_providers),
            attempts(silent)
        ));
    }
    Some(basis)
}

#[cfg(test)]
#[path = "charge_tests.rs"]
mod tests;
