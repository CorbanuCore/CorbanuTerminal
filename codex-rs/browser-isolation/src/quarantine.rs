use crate::BrowserError;
use codex_security_policy::AuthorityEpoch;
use codex_utils_absolute_path::AbsolutePathBuf;
use sha2::Digest;
use sha2::Sha256;

/// Opaque, in-memory download. No path or payload is written to the workspace.
/// Deliberately not Clone/Deserialize: promotion consumes this exact artifact.
pub struct QuarantinedArtifact {
    id: uuid::Uuid,
    digest: String,
    bytes: Vec<u8>,
    epoch: AuthorityEpoch,
}

/// Host-produced approval facts, not a model-supplied "approved" flag. The path
/// is selected by the host; HTTP filenames never become filesystem paths.
pub struct PromotionRequest {
    pub artifact_id: uuid::Uuid,
    pub sha256: String,
    pub byte_length: usize,
    pub destination: AbsolutePathBuf,
    pub epoch: AuthorityEpoch,
}

pub struct PromotedArtifact {
    pub destination: AbsolutePathBuf,
    /// Still untrusted bytes, never executable instructions or sanitized content.
    pub bytes: Vec<u8>,
}

impl QuarantinedArtifact {
    pub(crate) fn new(bytes: Vec<u8>, epoch: AuthorityEpoch) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            digest: format!("{:x}", Sha256::digest(&bytes)),
            bytes,
            epoch,
        }
    }

    pub fn byte_length(&self) -> usize {
        self.bytes.len()
    }
    pub fn sha256(&self) -> &str {
        &self.digest
    }

    /// Trusted host integration must run its native file-write policy AND an
    /// explicit human confirmation for these exact facts inside `approve`.
    /// This synchronous callback and handoff must share the caller's authority
    /// guard. S02 owns that UI/dispatch integration; this API itself writes nothing.
    /// A denial consumes/drops the quarantined payload; stale approval cannot replay.
    pub fn promote(
        self,
        destination: AbsolutePathBuf,
        current_epoch: AuthorityEpoch,
        approve: impl FnOnce(&PromotionRequest) -> Result<(), BrowserError>,
    ) -> Result<PromotedArtifact, BrowserError> {
        if self.epoch != current_epoch {
            return Err(BrowserError::StaleAuthority);
        }
        let request = PromotionRequest {
            artifact_id: self.id,
            sha256: self.digest,
            byte_length: self.bytes.len(),
            destination,
            epoch: self.epoch,
        };
        approve(&request).map_err(|_| BrowserError::PromotionDenied)?;
        Ok(PromotedArtifact {
            destination: request.destination,
            bytes: self.bytes,
        })
    }
}

#[cfg(test)]
#[path = "quarantine_tests.rs"]
mod tests;
