//! D690: process-wide snapshot queries for model credential holders.
//! The registry keeps weak references only. Queries never resolve a capability,
//! consume a route, grant a destination, or return credential material.

use super::CredentialBrokerState;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::Weak;

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

impl SnapshotHolder for RwLock<CredentialBrokerState> {
    fn occurs_in(&self, _text: &str) -> Result<bool, ModelSnapshotScanError> {
        let state = self.read().map_err(|_| ModelSnapshotScanError)?;
        // These legacy capability routes have no inspection operation. Calling
        // resolve/dispatch here would spend authority just to inspect a snapshot.
        // Refuse until their holders can supply a non-consuming query.
        if state.scoped_openai.is_some() || state.isolated_openai.is_some() {
            return Err(ModelSnapshotScanError);
        }
        Ok(false)
    }
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
    let records = records
        .iter()
        .map(|(length, hash)| {
            Ok(super::snapshot_scan::Fingerprint {
                length: *length,
                prefix: key.to_vec(),
                digest: super::isolated::protocol::decode_key(hash)
                    .ok_or(ModelSnapshotScanError)?,
            })
        })
        .collect::<Result<Vec<_>, ModelSnapshotScanError>>()?;
    super::snapshot_scan::scan(text, records, super::snapshot_scan::MAX_WINDOWS)
        .map_err(|()| ModelSnapshotScanError)
}

#[cfg(test)]
#[path = "model_snapshot_tests.rs"]
mod tests;
