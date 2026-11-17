//! D685: lets a shell snapshot check its text against credentials this engine
//! already brokers, so a copy survives no longer than its source variable.
//!
//! Matching happens here, against the engine's own records; no raw value is
//! returned or copied, and no destination, injection or registration changes.
//! In-process records are compared in place. Isolated records (PF-27-S04) keep
//! only a keyed fingerprint of their value, so a candidate window of the
//! recorded length is fingerprinted and compared instead; the work is bounded
//! by [`MAX_FINGERPRINT_WINDOWS`].
//!
//! Short values follow the masking rules: below
//! [`output_gate::MIN_VALUE_BYTES`] a value cannot be told apart from ordinary
//! text and the scan fails closed; below [`MIN_SUBSTRING_CREDENTIAL_BYTES`] (the
//! engine's minimum for copied values) a value matches only as a whole word, so
//! a short credential does not flag every variable that happens to contain it.

use super::HardenedBroker;
use super::HardenedRecord;
use super::Isolation;
use super::RecordSecret;
use codex_secret_broker::output_gate;
use zeroize::Zeroizing;

/// Values at least this long match anywhere in text; shorter ones match only
/// as a whole word (same minimum as copied-value detection).
pub(crate) const MIN_SUBSTRING_CREDENTIAL_BYTES: usize = 8;
/// Fingerprints computed per scan before it fails closed. Each hashes one
/// window of at most a few hundred bytes; 2^21 keeps a scan well under a
/// second while covering a multi-megabyte snapshot with a few isolated records.
pub(crate) const MAX_FINGERPRINT_WINDOWS: usize = 1 << 21;

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
    /// Whether `text` contains the value of any credential this engine has
    /// registered, including isolated and unavailable records. Fails closed
    /// when the engine does not own brokering or a check cannot be completed.
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
        let mut windows_left = max_windows;
        let state = self.read_state();
        for record in &state.credentials {
            if record.value_len < output_gate::MIN_VALUE_BYTES {
                return Err(KnownCredentialScanError::Unrecognizable);
            }
            let found = match &record.secret {
                RecordSecret::Raw(value) => raw_occurs(text.as_bytes(), value.as_bytes()),
                #[cfg(any(unix, windows))]
                RecordSecret::Brokered { fingerprint, .. } => fingerprint_occurs(
                    key.as_deref(),
                    record,
                    fingerprint,
                    text,
                    &mut windows_left,
                )?,
                RecordSecret::Unavailable { fingerprint } => fingerprint_occurs(
                    key.as_deref(),
                    record,
                    fingerprint,
                    text,
                    &mut windows_left,
                )?,
            };
            if found {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The key isolated fingerprints were computed with (all zero where the
    /// broker process is unsupported; see `virtualize_env_var`).
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

fn fingerprint_occurs(
    key: Option<&[u8; 32]>,
    record: &HardenedRecord,
    fingerprint: &[u8; 32],
    text: &str,
    windows_left: &mut usize,
) -> Result<bool, KnownCredentialScanError> {
    use sha2::Digest as _;
    // A fingerprinted record without a key cannot be checked.
    let Some(key) = key else {
        return Err(KnownCredentialScanError::Unrecognizable);
    };
    let text = text.as_bytes();
    let len = record.value_len;
    if len > text.len() {
        return Ok(false);
    }
    let whole_word = len < MIN_SUBSTRING_CREDENTIAL_BYTES;
    let prefix = sha2::Sha256::new()
        .chain_update(key.as_slice())
        .chain_update(record.env_var.as_bytes())
        .chain_update([0]);
    for start in 0..=text.len() - len {
        if whole_word && !is_whole_word(text, start, start + len) {
            continue;
        }
        if *windows_left == 0 {
            return Err(KnownCredentialScanError::BudgetExceeded);
        }
        *windows_left -= 1;
        let candidate: [u8; 32] = prefix
            .clone()
            .chain_update(&text[start..start + len])
            .finalize()
            .into();
        if &candidate == fingerprint {
            return Ok(true);
        }
    }
    Ok(false)
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
