//! D690: process-wide snapshot queries for model credential holders.
//! The registry keeps weak references only. Queries never resolve a capability,
//! consume a route, grant a destination, or return credential material.

use std::sync::{Arc, Mutex, Weak};

/// A holder cannot safely complete a snapshot credential check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("model credential snapshot protection is unavailable")]
pub struct ModelSnapshotScanError;

/// Owners inspect their own records, returning no key material or authority.
pub(super) trait SnapshotHolder: Send + Sync {
    fn occurs_in(&self, text: &str) -> Result<bool, ModelSnapshotScanError>;
}

static HOLDERS: Mutex<Vec<Weak<dyn SnapshotHolder>>> = Mutex::new(Vec::new());

pub(super) fn register(holder: Arc<dyn SnapshotHolder>) {
    // Preserve poison: scans must refuse if registration cannot be trusted.
    if let Ok(mut holders) = HOLDERS.lock() {
        holders.retain(|holder| holder.strong_count() != 0);
        if !holders
            .iter()
            .any(|entry| entry.ptr_eq(&Arc::downgrade(&holder)))
        {
            holders.push(Arc::downgrade(&holder));
        }
    }
}

/// Whether text contains a key held by a model credential owner in this process.
/// Unsupported scoped resolvers and unavailable isolated holders refuse the
/// query. Only a yes/no result crosses the holder boundary.
pub fn model_credential_occurs_in(text: &str) -> Result<bool, ModelSnapshotScanError> {
    if text.len() > codex_secret_broker::output_gate::MAX_SCAN_BYTES {
        return Err(ModelSnapshotScanError);
    }
    let holders: Vec<_> = HOLDERS
        .lock()
        .map_err(|_| ModelSnapshotScanError)?
        .iter()
        .filter_map(Weak::upgrade)
        .collect();
    let mut found = false;
    for holder in holders {
        found |= holder.occurs_in(text)?;
    }
    Ok(found)
}

#[cfg(any(unix, windows))]
impl SnapshotHolder for super::isolated::IsolatedBrokerClient {
    fn occurs_in(&self, text: &str) -> Result<bool, ModelSnapshotScanError> {
        self.model_credential_occurs_in(text)
            .map_err(|_| ModelSnapshotScanError)
    }
}

#[cfg(any(unix, windows))]
pub(super) fn fingerprint(key: &[u8; 32], value: &[u8]) -> String {
    use sha2::Digest;
    super::isolated::protocol::encode_hex(
        &sha2::Sha256::new()
            .chain_update(key)
            .chain_update(value)
            .finalize(),
    )
}

/// Same D685 bounds: >=8 bytes anywhere, 3-7 as whole words, <3 refuses;
/// isolated records retain only a keyed fingerprint plus length.
#[cfg(any(unix, windows))]
pub(super) fn scan_fingerprints(
    text: &str,
    key: &[u8; 32],
    records: &[(usize, String)],
) -> Result<bool, ModelSnapshotScanError> {
    use sha2::Digest;
    if text.len() > codex_secret_broker::output_gate::MAX_SCAN_BYTES
        || records.iter().any(|(len, hash)| {
            *len < 3 || *len > 8192 || super::isolated::protocol::decode_key(hash).is_none()
        })
    {
        return Err(ModelSnapshotScanError);
    }
    let text = text.as_bytes();
    let prefix = sha2::Sha256::new().chain_update(key);
    let mut windows_left = 1usize << 21;
    for (len, hash) in records {
        if *len > text.len() {
            continue;
        }
        let expected = super::isolated::protocol::decode_key(hash).ok_or(ModelSnapshotScanError)?;
        for start in 0..=text.len() - len {
            let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
            if *len < 8
                && ((start > 0 && word(text[start - 1]))
                    || text.get(start + len).copied().is_some_and(word))
            {
                continue;
            }
            if windows_left == 0 {
                return Err(ModelSnapshotScanError);
            }
            windows_left -= 1;
            let candidate: [u8; 32] = prefix
                .clone()
                .chain_update(&text[start..start + len])
                .finalize()
                .into();
            if candidate == expected {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
#[path = "model_snapshot_tests.rs"]
mod tests;
