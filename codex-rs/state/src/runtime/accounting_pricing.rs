//! Exact quotation over caller-provided price descriptors; not approval authority.
use super::types::Attempt;
use super::types::Count;
use super::types::Dialect;
use super::types::Observation;
use super::types::Usage;
use super::types::replay;
use anyhow::Context;
use anyhow::ensure;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

/// Canonical coefficient / 10^scale. Rates admit 38 digits and 18 decimal places;
/// exact products/sums admit u128 coefficients and up to 24 decimal places.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Decimal {
    coefficient: u128,
    scale: u32,
}

impl TryFrom<String> for Decimal {
    type Error = anyhow::Error;

    fn try_from(text: String) -> anyhow::Result<Self> {
        ensure!(text.len() <= 39, "decimal length bound");
        let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
        ensure!(
            !whole.is_empty()
                && (!text.contains('.') || !fraction.is_empty())
                && whole
                    .bytes()
                    .chain(fraction.bytes())
                    .all(|b| b.is_ascii_digit()),
            "invalid decimal syntax"
        );
        ensure!(whole.len() + fraction.len() <= 38, "decimal digit bound");
        ensure!(fraction.len() <= 18, "decimal scale bound");
        let mut coefficient = 0_u128;
        for digit in whole.bytes().chain(fraction.bytes()) {
            coefficient = coefficient
                .checked_mul(10)
                .and_then(|n| n.checked_add(u128::from(digit - b'0')))
                .context("decimal coefficient overflow")?;
        }
        Ok(Self::canonical(coefficient, fraction.len() as u32))
    }
}

impl Decimal {
    fn canonical(mut coefficient: u128, mut scale: u32) -> Self {
        while scale > 0 && coefficient.is_multiple_of(10) {
            coefficient /= 10;
            scale -= 1;
        }
        Self { coefficient, scale }
    }

    fn price(self, count: i64) -> anyhow::Result<Self> {
        let count = u128::try_from(count)?;
        let coefficient = self
            .coefficient
            .checked_mul(count)
            .context("price product overflow")?;
        Ok(Self::canonical(coefficient, self.scale + 6))
    }

    /// Exact addition, the only way amounts are ever combined in this ledger.
    pub fn add(self, other: Self) -> anyhow::Result<Self> {
        if self.coefficient == 0 {
            return Ok(other);
        }
        if other.coefficient == 0 {
            return Ok(self);
        }
        let scale = self.scale.max(other.scale);
        let align = |value: Self| {
            value
                .coefficient
                .checked_mul(10_u128.pow(scale - value.scale))
                .context("decimal alignment overflow")
        };
        let coefficient = align(self)?
            .checked_add(align(other)?)
            .context("decimal sum overflow")?;
        Ok(Self::canonical(coefficient, scale))
    }

    pub fn display(self) -> DisplayAmount {
        let (whole, fraction, rounded, nonzero_sub_micro) = if self.scale <= 6 {
            let divisor = 10_u128.pow(self.scale);
            (
                self.coefficient / divisor,
                self.coefficient % divisor * 10_u128.pow(6 - self.scale),
                false,
                false,
            )
        } else {
            let divisor = 10_u128.pow(self.scale - 6);
            let quotient = self.coefficient / divisor;
            let remainder = self.coefficient % divisor;
            // Compare against the complementary remainder: never double a u128.
            let up = remainder > divisor - remainder
                || (remainder == divisor - remainder && quotient % 2 == 1);
            // divisor >= 10, so quotient + 1 cannot overflow.
            let micros = quotient + u128::from(up);
            (
                micros / 1_000_000,
                micros % 1_000_000,
                remainder != 0,
                self.coefficient != 0 && quotient == 0,
            )
        };
        DisplayAmount {
            text: format!("{whole}.{fraction:06}"),
            rounded,
            nonzero_sub_micro,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DisplayAmount {
    pub text: String,
    pub rounded: bool,
    pub nonzero_sub_micro: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Currency {
    #[serde(rename = "USD")]
    Usd,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Unit {
    PerMillionTokens,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum SourceKind {
    ProviderPublished,
    NativeCatalog,
}

/// What a snapshot's rates are a statement about.
///
/// A subscription turn is not billed per token, so pricing it with API rates and
/// adding the result to spend would claim money that was never charged. Those
/// rates are kept under `PlanEquivalent`, which states what the same tokens would
/// have cost on the API side, alongside the plan rate that actually applied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Basis {
    /// Rates the provider charges for this request. Their sum is money spent.
    #[default]
    Billed,
    /// Subscription capacity: the plan rate applied, and the API rates, if the
    /// catalogue states any, are a counterfactual rather than a charge.
    PlanEquivalent,
}

/// Caller-supplied synthetic approval evidence, not production authorization.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub id: Uuid,
    pub provider: String,
    pub model: String,
    pub scope: Uuid,
    pub currency: Currency,
    pub unit: Unit,
    pub rates: Rates,
    pub source_reference: Uuid,
    pub source_kind: SourceKind,
    /// Older payloads predate plan accounting and are billed rates by construction.
    #[serde(default)]
    pub basis: Basis,
    /// Relative subscription-pool burn that applied at dispatch, 1000 meaning 1.0x.
    #[serde(default)]
    pub plan_burn_millis: Option<u32>,
    pub observed_at_ms: Count,
    pub approved_at_ms: Count,
    pub effective_from_ms: Count,
    pub effective_end_ms: Option<Count>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rates {
    pub noncached: Option<Decimal>,
    pub read: Option<Decimal>,
    pub write: Option<Decimal>,
    pub output: Option<Decimal>,
}

impl Snapshot {
    fn validate(&self) -> anyhow::Result<()> {
        for text in [&self.provider, &self.model] {
            ensure!(
                !text.trim().is_empty() && text.len() <= 128 && !text.chars().any(char::is_control),
                "invalid snapshot metadata"
            );
        }
        let start = i64::from(self.effective_from_ms);
        ensure!(
            start >= i64::from(self.observed_at_ms) && start >= i64::from(self.approved_at_ms),
            "backdated snapshot"
        );
        ensure!(
            self.effective_end_ms
                .is_none_or(|end| i64::from(end) > start),
            "invalid snapshot interval"
        );
        // A plan rate has no meaning under billed rates: keep the two from
        // being mistaken for each other. A plan snapshot may state no plan rate
        // when the vendor published API rates but no subscription figure.
        ensure!(
            self.plan_burn_millis.is_none() || matches!(self.basis, Basis::PlanEquivalent),
            "plan rate does not match price basis"
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum BucketQuote {
    Priced(Decimal),
    MissingUsage,
    MissingRate,
}

/// The pricing rules new estimates are computed under.
///
/// A recorded estimate is history: it is re-verified by recomputing it under
/// the rules that produced it, never under whatever this build would compute
/// today. So any change to how a quote is computed or serialized must add a
/// version here, keep every earlier version reproducible in
/// `quote_observations_under`, and leave the version-1 payload bytes pinned by
/// `recorded_version_one_payloads_stay_byte_identical` untouched. Changing a
/// rule in place would make every recorded estimate fail verification, and a
/// failed verification stops accounting - which stops every accounted turn.
pub const PRICING_RULES: u16 = 1;

// serde's `skip_serializing_if` passes the field by reference.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_first_rules(rules: &u16) -> bool {
    *rules == 1
}

/// An observation-bound estimate has no terminal-coverage or billing claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObservationQuote {
    pub attempt: Attempt,
    pub observations: Vec<Observation>,
    pub usage: Usage,
    pub snapshot: Option<Snapshot>,
    // Disjoint noncached input, cache read, cache write, output, in that order.
    pub buckets: [BucketQuote; 4],
    /// Money charged for this attempt. Zero under a plan, where nothing is.
    pub known_subtotal: Decimal,
    pub all_buckets_priced: Option<Decimal>,
    pub subtotal_display: DisplayAmount,
    /// What these tokens would have cost at the catalogue's API rates, when the
    /// attempt ran on subscription capacity. Never added to money spent.
    pub known_equivalent: Decimal,
    pub all_buckets_equivalent: Option<Decimal>,
    /// The plan rate that applied at dispatch, 1000 meaning 1.0x, and the plan
    /// consumption it implies: total tokens scaled by that rate.
    pub plan_burn_millis: Option<u32>,
    pub plan_burn_milli_tokens: Option<i64>,
    /// The rules this quote was computed under. Version 1 is not serialized,
    /// so every estimate recorded before versioning keeps its exact bytes.
    #[serde(skip_serializing_if = "is_first_rules")]
    pub pricing_rules: u16,
}

impl ObservationQuote {
    /// Whether the attempt ran on subscription capacity, whether or not a plan
    /// rate was stated for it.
    pub fn is_plan(&self) -> bool {
        self.plan_burn_millis.is_some()
            || self
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| matches!(snapshot.basis, Basis::PlanEquivalent))
    }
}

/// Token counts for the four priced buckets. Usage stays exactly as observed -
/// an unreported cache-write count is still unknown there. Pricing alone treats
/// it as nothing to charge, and only when the price snapshot bound at dispatch
/// states cache writes cost zero (a price sheet with cache hits and misses and
/// no write charge, such as DeepSeek's) and the dialect is inclusive, so input
/// is exactly miss + hit. Snapshots are immutable, so attempts priced under an
/// earlier sheet keep the quote they were given.
fn priced_counts(usage: &Usage, dialect: Dialect, snapshot: Option<&Snapshot>) -> [Option<i64>; 4] {
    let write_is_free = snapshot.is_some_and(|s| s.rates.write == Some(Decimal::default()));
    if dialect == Dialect::Inclusive && usage.write.is_none() && write_is_free {
        // Replay has already checked the cache read does not exceed input.
        let miss = usage
            .input
            .zip(usage.read)
            .and_then(|(input, read)| input.checked_sub(read));
        [miss, usage.read, Some(0), usage.output]
    } else {
        [usage.noncached, usage.read, usage.write, usage.output]
    }
}

fn quote_observations(
    attempt: &Attempt,
    observations: &[Observation],
    snapshots: &[Snapshot],
) -> anyhow::Result<ObservationQuote> {
    quote_observations_under(PRICING_RULES, attempt, observations, snapshots)
}

/// Quote under a named version of the pricing rules. Every version ever
/// recorded must stay reproducible here; see `PRICING_RULES`.
fn quote_observations_under(
    rules: u16,
    attempt: &Attempt,
    observations: &[Observation],
    snapshots: &[Snapshot],
) -> anyhow::Result<ObservationQuote> {
    ensure!(
        (1..=PRICING_RULES).contains(&rules),
        "estimate recorded under pricing rules {rules}, newer than this build's {PRICING_RULES}"
    );
    attempt.validate()?;
    let usage = replay(attempt.dialect, observations)?;
    ensure!(snapshots.len() <= 64, "snapshot candidate bound");
    let mut ids = HashSet::new();
    let mut selected = None;
    let dispatch = i64::from(attempt.dispatched_at_ms);
    for snapshot in snapshots {
        snapshot.validate()?;
        ensure!(ids.insert(snapshot.id), "duplicate snapshot identity");
        if snapshot.provider == attempt.provider
            && snapshot.model == attempt.model
            && snapshot.scope == attempt.scope
            && dispatch >= i64::from(snapshot.effective_from_ms)
            && snapshot
                .effective_end_ms
                .is_none_or(|end| dispatch < i64::from(end))
        {
            ensure!(selected.is_none(), "overlapping snapshots at dispatch");
            selected = Some(snapshot);
        }
    }
    let rates = selected.map_or([None; 4], |s| {
        [
            s.rates.noncached,
            s.rates.read,
            s.rates.write,
            s.rates.output,
        ]
    });
    let mut buckets = [BucketQuote::MissingUsage; 4];
    let mut known_subtotal = Decimal::default();
    let mut complete = true;
    for ((bucket, count), rate) in buckets
        .iter_mut()
        .zip(priced_counts(&usage, attempt.dialect, selected))
        .zip(rates)
    {
        *bucket = match (count, rate) {
            (None, _) => BucketQuote::MissingUsage,
            (Some(0), _) if selected.is_some() => BucketQuote::Priced(Decimal::default()),
            (Some(n), Some(rate)) => BucketQuote::Priced(rate.price(n)?),
            (Some(_), None) => BucketQuote::MissingRate,
        };
        if let BucketQuote::Priced(amount) = bucket {
            known_subtotal = known_subtotal.add(*amount)?;
        } else {
            complete = false;
        }
    }
    // Under a plan the same arithmetic answers a different question, so the total
    // moves to the equivalent field and money spent stays exactly zero.
    let plan = selected.is_some_and(|snapshot| matches!(snapshot.basis, Basis::PlanEquivalent));
    let plan_burn_millis = selected.and_then(|snapshot| snapshot.plan_burn_millis);
    let plan_burn_milli_tokens = plan_burn_millis
        .zip(usage.total)
        .map(|(burn, total)| i64::from(burn).checked_mul(total).context("burn overflow"))
        .transpose()?;
    let (known_subtotal, known_equivalent) = if plan {
        (Decimal::default(), known_subtotal)
    } else {
        (known_subtotal, Decimal::default())
    };
    Ok(ObservationQuote {
        attempt: attempt.clone(),
        observations: observations.to_vec(),
        usage,
        snapshot: selected.cloned(),
        buckets,
        known_subtotal,
        all_buckets_priced: (complete && !plan).then_some(known_subtotal),
        subtotal_display: known_subtotal.display(),
        known_equivalent,
        all_buckets_equivalent: (complete && plan).then_some(known_equivalent),
        plan_burn_millis,
        plan_burn_milli_tokens,
        pricing_rules: rules,
    })
}

#[cfg(test)]
#[path = "accounting_pricing_tests.rs"]
mod tests;

#[path = "accounting_estimates.rs"]
mod storage;
pub use storage::Current;
pub use storage::DayTotals;
pub use storage::Metric;
pub use storage::RetainedDay;
pub use storage::RetentionCoverage;
