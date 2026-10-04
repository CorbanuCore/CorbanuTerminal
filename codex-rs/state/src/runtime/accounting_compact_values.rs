//! Exact compact numeric values only; ownership and expiry require a later envelope.
use super::DayTotals;
use super::Decimal;
use super::Metric;
use anyhow::Context;
use anyhow::ensure;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompactValues {
    totals: DayTotals,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredValues {
    version: u8,
    // Input, noncached, read, write, output, reasoning, total.
    known: [i64; 7],
    unknown: [i64; 7],
    #[serde(deserialize_with = "deserialize_amount")]
    known_usd: Decimal,
    unknown_estimates: i64,
    attempts: i64,
    // Version 1 days predate plan accounting and carry no plan work by construction.
    #[serde(default, deserialize_with = "deserialize_amount")]
    equivalent_usd: Decimal,
    #[serde(default)]
    unknown_equivalents: i64,
    #[serde(default)]
    plan_burn_known: i64,
    #[serde(default)]
    plan_burn_unknown: i64,
    #[serde(default)]
    plan_attempts: i64,
}

fn deserialize_amount<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Decimal, D::Error> {
    let text = String::deserialize(deserializer)?;
    decode_amount(&text).map_err(serde::de::Error::custom)
}

fn decode_amount(text: &str) -> anyhow::Result<Decimal> {
    ensure!(text.len() <= 40, "stored amount length bound");
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    ensure!(
        !whole.is_empty()
            && (!text.contains('.') || !fraction.is_empty())
            && whole
                .bytes()
                .chain(fraction.bytes())
                .all(|digit| digit.is_ascii_digit()),
        "invalid stored amount syntax"
    );
    ensure!(fraction.len() <= 24, "stored amount scale bound");
    let mut coefficient = 0_u128;
    for digit in whole.bytes().chain(fraction.bytes()) {
        coefficient = coefficient
            .checked_mul(10)
            .and_then(|value| value.checked_add(u128::from(digit - b'0')))
            .context("stored amount coefficient overflow")?;
    }
    let amount = Decimal::canonical(coefficient, fraction.len() as u32);
    ensure!(
        serde_json::to_value(amount)? == serde_json::Value::String(text.to_owned()),
        "noncanonical stored amount"
    );
    Ok(amount)
}

impl CompactValues {
    pub(super) fn decode(text: &str) -> anyhow::Result<Self> {
        ensure!(
            text.trim_start().starts_with('{'),
            "expected compact object"
        );
        let stored: StoredValues = serde_json::from_str(text)?;
        ensure!(
            stored.version == 1 || stored.version == 2,
            "unsupported compact version"
        );
        Self::from_day_totals(&DayTotals {
            measured: std::array::from_fn(|index| Metric {
                known: stored.known[index],
                unknown: stored.unknown[index],
            }),
            known_usd: stored.known_usd,
            unknown_estimates: stored.unknown_estimates,
            attempts: stored.attempts,
            equivalent_usd: stored.equivalent_usd,
            unknown_equivalents: stored.unknown_equivalents,
            plan_burn_milli_tokens: Metric {
                known: stored.plan_burn_known,
                unknown: stored.plan_burn_unknown,
            },
            plan_attempts: stored.plan_attempts,
        })
    }

    pub(super) fn encode(&self) -> anyhow::Result<String> {
        self.validate()?;
        serde_json::to_string(&StoredValues {
            version: 2,
            known: std::array::from_fn(|index| self.totals.measured[index].known),
            unknown: std::array::from_fn(|index| self.totals.measured[index].unknown),
            known_usd: self.totals.known_usd,
            unknown_estimates: self.totals.unknown_estimates,
            attempts: self.totals.attempts,
            equivalent_usd: self.totals.equivalent_usd,
            unknown_equivalents: self.totals.unknown_equivalents,
            plan_burn_known: self.totals.plan_burn_milli_tokens.known,
            plan_burn_unknown: self.totals.plan_burn_milli_tokens.unknown,
            plan_attempts: self.totals.plan_attempts,
        })
        .context("encode compact values")
    }

    pub(super) fn from_day_totals(totals: &DayTotals) -> anyhow::Result<Self> {
        let value = Self {
            totals: totals.clone(),
        };
        value.validate()?;
        Ok(value)
    }

    pub(super) fn to_day_totals(&self) -> anyhow::Result<DayTotals> {
        self.validate()?;
        Ok(self.totals.clone())
    }

    pub(super) fn checked_add(&self, other: &Self) -> anyhow::Result<Self> {
        self.validate()?;
        other.validate()?;
        let mut totals = self.totals.clone();
        for (metric, rhs) in totals.measured.iter_mut().zip(&other.totals.measured) {
            metric.known = metric
                .known
                .checked_add(rhs.known)
                .context("known overflow")?;
            metric.unknown = metric
                .unknown
                .checked_add(rhs.unknown)
                .context("unknown overflow")?;
        }
        totals.known_usd = totals.known_usd.add(other.totals.known_usd)?;
        totals.equivalent_usd = totals.equivalent_usd.add(other.totals.equivalent_usd)?;
        totals.unknown_equivalents = totals
            .unknown_equivalents
            .checked_add(other.totals.unknown_equivalents)
            .context("unknown equivalent overflow")?;
        totals.plan_burn_milli_tokens.known = totals
            .plan_burn_milli_tokens
            .known
            .checked_add(other.totals.plan_burn_milli_tokens.known)
            .context("plan burn overflow")?;
        totals.plan_burn_milli_tokens.unknown = totals
            .plan_burn_milli_tokens
            .unknown
            .checked_add(other.totals.plan_burn_milli_tokens.unknown)
            .context("plan burn unknown overflow")?;
        totals.plan_attempts = totals
            .plan_attempts
            .checked_add(other.totals.plan_attempts)
            .context("plan attempt overflow")?;
        totals.unknown_estimates = totals
            .unknown_estimates
            .checked_add(other.totals.unknown_estimates)
            .context("unknown estimate overflow")?;
        totals.attempts = totals
            .attempts
            .checked_add(other.totals.attempts)
            .context("attempt overflow")?;
        Self::from_day_totals(&totals)
    }

    fn validate(&self) -> anyhow::Result<()> {
        let totals = &self.totals;
        ensure!(totals.attempts >= 0, "negative attempts");
        ensure!(
            (0..=totals.attempts).contains(&totals.unknown_estimates),
            "invalid unknown estimate count"
        );
        for metric in &totals.measured {
            ensure!(metric.known >= 0, "negative known count");
            ensure!(
                (0..=totals.attempts).contains(&metric.unknown),
                "invalid unknown count"
            );
            ensure!(
                metric.unknown < totals.attempts || metric.known == 0,
                "known count without known constituents"
            );
        }
        for amount in [totals.known_usd, totals.equivalent_usd] {
            ensure!(amount.scale <= 24, "stored amount scale bound");
            ensure!(
                Decimal::canonical(amount.coefficient, amount.scale) == amount,
                "noncanonical stored amount"
            );
            ensure!(
                totals.attempts > 0 || amount.coefficient == 0,
                "amount without constituents"
            );
        }
        // Plan work is a subset of the day's attempts, and every plan figure is
        // bounded by it: a plan number larger than the plan work it came from
        // would be a reporting error rather than a big bill.
        ensure!(
            (0..=totals.attempts).contains(&totals.plan_attempts),
            "invalid plan attempt count"
        );
        ensure!(
            (0..=totals.plan_attempts).contains(&totals.unknown_equivalents)
                && (0..=totals.plan_attempts).contains(&totals.plan_burn_milli_tokens.unknown),
            "invalid plan unknown count"
        );
        ensure!(
            totals.plan_burn_milli_tokens.known >= 0,
            "negative plan burn"
        );
        ensure!(
            totals.plan_attempts > 0
                || (totals.equivalent_usd.coefficient == 0
                    && totals.plan_burn_milli_tokens.known == 0),
            "plan figures without plan attempts"
        );
        Ok(())
    }
}

#[cfg(test)]
#[path = "accounting_compact_values_tests.rs"]
mod tests;
