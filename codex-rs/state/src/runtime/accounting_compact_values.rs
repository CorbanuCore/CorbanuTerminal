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
        ensure!(stored.version == 1, "unsupported compact version");
        Self::from_day_totals(&DayTotals {
            measured: std::array::from_fn(|index| Metric {
                known: stored.known[index],
                unknown: stored.unknown[index],
            }),
            known_usd: stored.known_usd,
            unknown_estimates: stored.unknown_estimates,
            attempts: stored.attempts,
        })
    }

    pub(super) fn encode(&self) -> anyhow::Result<String> {
        self.validate()?;
        serde_json::to_string(&StoredValues {
            version: 1,
            known: std::array::from_fn(|index| self.totals.measured[index].known),
            unknown: std::array::from_fn(|index| self.totals.measured[index].unknown),
            known_usd: self.totals.known_usd,
            unknown_estimates: self.totals.unknown_estimates,
            attempts: self.totals.attempts,
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
        let amount = totals.known_usd;
        ensure!(amount.scale <= 24, "stored amount scale bound");
        ensure!(
            Decimal::canonical(amount.coefficient, amount.scale) == amount,
            "noncanonical stored amount"
        );
        ensure!(
            totals.attempts > 0 || amount.coefficient == 0,
            "amount without constituents"
        );
        Ok(())
    }
}

#[cfg(test)]
#[path = "accounting_compact_values_tests.rs"]
mod tests;
