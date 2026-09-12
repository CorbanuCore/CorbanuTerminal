//! Exact quotation over caller-provided price descriptors; not approval authority.
use super::types::Attempt;
use super::types::Count;
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

    fn add(self, other: Self) -> anyhow::Result<Self> {
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

    fn display(self) -> DisplayAmount {
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
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum BucketQuote {
    Priced(Decimal),
    MissingUsage,
    MissingRate,
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
    pub known_subtotal: Decimal,
    pub all_buckets_priced: Option<Decimal>,
    pub subtotal_display: DisplayAmount,
}

fn quote_observations(
    attempt: &Attempt,
    observations: &[Observation],
    snapshots: &[Snapshot],
) -> anyhow::Result<ObservationQuote> {
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
        .zip([usage.noncached, usage.read, usage.write, usage.output])
        .zip(rates)
    {
        *bucket = match (count, rate) {
            (None, _) => BucketQuote::MissingUsage,
            (Some(0), _) => BucketQuote::Priced(Decimal::default()),
            (Some(n), Some(rate)) => BucketQuote::Priced(rate.price(n)?),
            (Some(_), None) => BucketQuote::MissingRate,
        };
        if let BucketQuote::Priced(amount) = bucket {
            known_subtotal = known_subtotal.add(*amount)?;
        } else {
            complete = false;
        }
    }
    Ok(ObservationQuote {
        attempt: attempt.clone(),
        observations: observations.to_vec(),
        usage,
        snapshot: selected.cloned(),
        buckets,
        known_subtotal,
        all_buckets_priced: complete.then_some(known_subtotal),
        subtotal_display: known_subtotal.display(),
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
