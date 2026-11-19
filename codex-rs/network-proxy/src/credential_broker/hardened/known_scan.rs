//! D685: check copied credentials without returning raw values or authority.
//! Isolated and unavailable records retain their existing keyed fingerprints.
//! The shared scanner groups lengths and memoizes exact nonmatching text
//! windows; no additional credential-derived metadata crosses a boundary.

use super::HardenedBroker;
use super::Isolation;
use super::RecordSecret;
use codex_secret_broker::output_gate;
use zeroize::Zeroizing;

/// Values at least this long match anywhere; shorter values match whole words.
pub(crate) const MIN_SUBSTRING_CREDENTIAL_BYTES: usize = 8;
/// Eight distinct lengths over 3 MiB use at most 24 Mi of the 32 Mi windows.
pub(crate) const MAX_FINGERPRINT_WINDOWS: usize =
    crate::credential_broker::snapshot_scan::MAX_WINDOWS;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum KnownCredentialScanError {
    #[error("the hardened credential engine does not own brokering")]
    NotArmed,
    #[error("a brokered credential is too short to recognize in text")]
    Unrecognizable,
    #[error("text is too large to check against brokered credentials")]
    BudgetExceeded,
}

impl HardenedBroker {
    /// Whether text contains any registered credential, including isolated and
    /// unavailable records. An incomplete check fails closed.
    pub(crate) fn known_credential_occurs_in(
        &self,
        text: &str,
    ) -> Result<bool, KnownCredentialScanError> {
        self.known_credential_occurs_in_with_budget(text, MAX_FINGERPRINT_WINDOWS)
    }

    pub(super) fn known_credential_occurs_in_with_budget(
        &self,
        text: &str,
        max_windows: usize,
    ) -> Result<bool, KnownCredentialScanError> {
        if !self.armed() {
            return Err(KnownCredentialScanError::NotArmed);
        }
        if text.len() > output_gate::MAX_SCAN_BYTES {
            return Err(KnownCredentialScanError::BudgetExceeded);
        }
        let key = self.fingerprint_key();
        let state = self.read_state();
        if state
            .credentials
            .iter()
            .any(|record| record.value_len < output_gate::MIN_VALUE_BYTES)
        {
            return Err(KnownCredentialScanError::Unrecognizable);
        }
        scan_records(state.credentials.iter(), key.as_deref(), text, max_windows)
    }

    /// D732: which of `texts` contain a registered credential of at least
    /// [`MIN_SUBSTRING_CREDENTIAL_BYTES`] (shorter ones are not looked for),
    /// with the same D685 bounds as [`Self::known_credential_occurs_in`].
    /// Takes the state the caller already holds. A text that cannot be
    /// checked is an error; with nothing to look for every text is clean.
    pub(super) fn env_copy_occurrences(
        &self,
        state: &super::HardenedState,
        texts: &[&str],
    ) -> Vec<Result<bool, KnownCredentialScanError>> {
        let key = self.fingerprint_key();
        let records: Vec<&super::HardenedRecord> = state
            .credentials
            .iter()
            .filter(|record| record.value_len >= MIN_SUBSTRING_CREDENTIAL_BYTES)
            .collect();
        texts
            .iter()
            .map(|text| {
                if records.is_empty() {
                    return Ok(false);
                }
                if text.len() > output_gate::MAX_SCAN_BYTES {
                    return Err(KnownCredentialScanError::BudgetExceeded);
                }
                scan_records(
                    records.iter().copied(),
                    key.as_deref(),
                    text,
                    MAX_FINGERPRINT_WINDOWS,
                )
            })
            .collect()
    }

    fn fingerprint_key(&self) -> Option<Zeroizing<[u8; 32]>> {
        match &self.isolation {
            Isolation::InProcess => None,
            #[cfg(any(unix, windows))]
            Isolation::Process(mode) => Some(mode.fingerprint_key.clone()),
            #[cfg(not(any(unix, windows)))]
            Isolation::Unsupported => Some(Zeroizing::new([0; 32])),
        }
    }
}

/// Raw records match directly; isolated and unavailable ones by keyed
/// fingerprint through the shared bounded scanner.
fn scan_records<'a>(
    credentials: impl Iterator<Item = &'a super::HardenedRecord>,
    key: Option<&[u8; 32]>,
    text: &str,
    max_windows: usize,
) -> Result<bool, KnownCredentialScanError> {
    let mut records = Vec::new();
    let mut found = false;
    for record in credentials {
        let fingerprint = match &record.secret {
            RecordSecret::Raw(value) => {
                found |= raw_occurs(text.as_bytes(), value.as_bytes());
                continue;
            }
            #[cfg(any(unix, windows))]
            RecordSecret::Brokered { fingerprint, .. } => fingerprint,
            RecordSecret::Unavailable { fingerprint } => fingerprint,
        };
        let key = key.ok_or(KnownCredentialScanError::Unrecognizable)?;
        let mut prefix = key.to_vec();
        prefix.extend_from_slice(record.env_var.as_bytes());
        prefix.push(0);
        records.push(crate::credential_broker::snapshot_scan::Fingerprint {
            length: record.value_len,
            prefix,
            digest: *fingerprint,
        });
    }
    let isolated = crate::credential_broker::snapshot_scan::scan(text, records, max_windows)
        .map_err(|()| KnownCredentialScanError::BudgetExceeded)?;
    Ok(found || isolated)
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_whole_word(text: &[u8], start: usize, end: usize) -> bool {
    !(start > 0 && is_word_byte(text[start - 1]))
        && !text.get(end).copied().is_some_and(is_word_byte)
}

fn raw_occurs(text: &[u8], value: &[u8]) -> bool {
    let whole_word = value.len() < MIN_SUBSTRING_CREDENTIAL_BYTES;
    if value.is_empty() || value.len() > text.len() {
        return false;
    }
    text.windows(value.len())
        .enumerate()
        .any(|(start, window)| {
            window == value && (!whole_word || is_whole_word(text, start, start + value.len()))
        })
}

impl crate::credential_broker::model_snapshot::SnapshotHolder
    for std::sync::RwLock<super::HardenedState>
{
    fn occurs_in(
        &self,
        _text: &str,
    ) -> Result<bool, crate::credential_broker::model_snapshot::ModelSnapshotScanError> {
        use crate::credential_broker::model_snapshot::ModelSnapshotScanError;
        let state = self.read().map_err(|_| ModelSnapshotScanError)?;
        // Do not consume a capability through resolve/dispatch for inspection.
        if state.scoped_openai.is_some() || state.isolated_openai.is_some() {
            return Err(ModelSnapshotScanError);
        }
        Ok(false)
    }
}

#[cfg(test)]
#[path = "known_scan_tests.rs"]
mod tests;
