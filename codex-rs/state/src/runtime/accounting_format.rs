//! The ledger's format: which stored shapes this build reads, and which it refuses.
//!
//! The format is the number of accounting migrations a ledger has applied.
//! Format 1 is the original schema. Format 2 lets a price record state a
//! `Local` or `Undeclared` basis and where its basis came from, and lets a
//! compact day count that work. A build reads every format up to its own and
//! upgrades older ledgers before writing; a ledger written in a newer format is
//! refused with `NewerLedgerFormat`, so the caller can stop collecting rather
//! than misread records it does not understand.
//!
//! Payload bytes are checked for canonical form on every read. Two forms are
//! valid: today's, and the form written before price records gained a basis
//! (2026-09-21, `5bae03414e`), which omits the basis and plan fields. That older
//! form is accepted only for a record whose omitted fields hold their defaults,
//! so no record can be read under a form that would hide a field it states.
use super::Attempt;
use super::Basis;
use super::BasisSource;
use super::BucketQuote;
use super::Count;
use super::Currency;
use super::Decimal;
use super::DisplayAmount;
use super::Observation;
use super::ObservationQuote;
use super::Rates;
use super::Snapshot;
use super::SourceKind;
use super::Unit;
use super::Usage;
use serde::Serialize;
use uuid::Uuid;

/// A ledger whose format is newer than this build's: collection must stop
/// rather than read or write records this build does not understand.
#[derive(Debug)]
pub struct NewerLedgerFormat {
    pub found: i64,
    pub supported: i64,
}

impl std::fmt::Display for NewerLedgerFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "accounting ledger format {} is newer than this build's format {}",
            self.found, self.supported
        )
    }
}

impl std::error::Error for NewerLedgerFormat {}

/// Whether `error` is a refusal of a newer ledger format.
pub fn is_newer_format(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.downcast_ref::<NewerLedgerFormat>().is_some())
}

/// A snapshot as written before the basis existed.
#[derive(Serialize)]
struct SnapshotBeforeBasis<'a> {
    id: &'a Uuid,
    provider: &'a str,
    model: &'a str,
    scope: &'a Uuid,
    currency: &'a Currency,
    unit: &'a Unit,
    rates: &'a Rates,
    source_reference: &'a Uuid,
    source_kind: &'a SourceKind,
    observed_at_ms: &'a Count,
    approved_at_ms: &'a Count,
    effective_from_ms: &'a Count,
    effective_end_ms: &'a Option<Count>,
}

impl<'a> SnapshotBeforeBasis<'a> {
    /// `None` when the snapshot states a field the older form cannot carry.
    fn of(snapshot: &'a Snapshot) -> Option<Self> {
        (snapshot.basis == Basis::Billed
            && snapshot.plan_burn_millis.is_none()
            && snapshot.basis_source == BasisSource::BuiltIn)
            .then_some(Self {
                id: &snapshot.id,
                provider: &snapshot.provider,
                model: &snapshot.model,
                scope: &snapshot.scope,
                currency: &snapshot.currency,
                unit: &snapshot.unit,
                rates: &snapshot.rates,
                source_reference: &snapshot.source_reference,
                source_kind: &snapshot.source_kind,
                observed_at_ms: &snapshot.observed_at_ms,
                approved_at_ms: &snapshot.approved_at_ms,
                effective_from_ms: &snapshot.effective_from_ms,
                effective_end_ms: &snapshot.effective_end_ms,
            })
    }
}

/// An estimate as written before the basis existed.
#[derive(Serialize)]
struct QuoteBeforeBasis<'a> {
    attempt: &'a Attempt,
    observations: &'a [Observation],
    usage: &'a Usage,
    snapshot: Option<SnapshotBeforeBasis<'a>>,
    buckets: &'a [BucketQuote; 4],
    known_subtotal: &'a Decimal,
    all_buckets_priced: &'a Option<Decimal>,
    subtotal_display: &'a DisplayAmount,
}

impl<'a> QuoteBeforeBasis<'a> {
    fn of(quote: &'a ObservationQuote) -> Option<Self> {
        let snapshot = match &quote.snapshot {
            Some(snapshot) => Some(SnapshotBeforeBasis::of(snapshot)?),
            None => None,
        };
        (quote.pricing_rules == 1
            && quote.known_equivalent == Decimal::default()
            && quote.all_buckets_equivalent.is_none()
            && quote.plan_burn_millis.is_none()
            && quote.plan_burn_milli_tokens.is_none())
        .then_some(Self {
            attempt: &quote.attempt,
            observations: &quote.observations,
            usage: &quote.usage,
            snapshot,
            buckets: &quote.buckets,
            known_subtotal: &quote.known_subtotal,
            all_buckets_priced: &quote.all_buckets_priced,
            subtotal_display: &quote.subtotal_display,
        })
    }
}

/// Whether `payload` is `snapshot` in one of its valid stored forms.
pub(super) fn snapshot_is_canonical(snapshot: &Snapshot, payload: &str) -> anyhow::Result<bool> {
    if serde_json::to_string(snapshot)? == payload {
        return Ok(true);
    }
    Ok(match SnapshotBeforeBasis::of(snapshot) {
        Some(older) => serde_json::to_string(&older)? == payload,
        None => false,
    })
}

/// Whether `payload` is `quote` in one of its valid stored forms.
pub(super) fn quote_is_canonical(quote: &ObservationQuote, payload: &str) -> anyhow::Result<bool> {
    if serde_json::to_string(quote)? == payload {
        return Ok(true);
    }
    Ok(match QuoteBeforeBasis::of(quote) {
        Some(older) => serde_json::to_string(&older)? == payload,
        None => false,
    })
}

#[cfg(test)]
#[path = "accounting_format_tests.rs"]
mod tests;
