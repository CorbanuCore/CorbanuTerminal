//! Descriptive source metadata, never an authorization capability.
//!
//! Deserializing an envelope does not authenticate its producer. Core must bind
//! it to the exact source and bytes at trusted ingress before model admission.

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

pub const SOURCE_ENVELOPE_VERSION: u32 = 1;
pub const MAX_SOURCE_LINEAGE: usize = 32;
pub const MAX_SOURCE_TRANSFORMATIONS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Web,
    Search,
    File,
    Transcript,
    Social,
    Trollbox,
    Email,
    Mcp,
    Tool,
    Plugin,
    Hook,
    ChildAgent,
    Unknown,
}

/// Intentionally has no human/system/approved variant. Neither a classifier
/// verdict nor externally supplied metadata can manufacture authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAuthority {
    Untrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDescriptor {
    pub kind: SourceKind,
    /// Opaque adapter-owned identity, not an unredacted URL/path or display name.
    pub origin_id: String,
    /// Opaque observed actor identity; not an authenticated human principal.
    pub actor_id: String,
    pub retrieved_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceTransformation {
    pub id: String,
    pub input_digest: [u8; 32],
    pub output_digest: [u8; 32],
}

/// Immutable, bounded metadata retained separately from the content it describes.
/// Wire values are validated but remain unauthenticated until Core admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "EnvelopeWire")]
pub struct SourceEnvelope {
    schema_version: u32,
    source_id: Uuid,
    source: SourceDescriptor,
    authority: SourceAuthority,
    raw_digest: [u8; 32],
    content_digest: [u8; 32],
    transformations: Vec<SourceTransformation>,
    taint_lineage: Vec<Uuid>,
}

impl SourceEnvelope {
    pub fn new(
        source_id: Uuid,
        source: SourceDescriptor,
        raw_digest: [u8; 32],
        transformations: Vec<SourceTransformation>,
        mut taint_lineage: Vec<Uuid>,
    ) -> Result<Self, ProvenanceError> {
        if source_id.is_nil()
            || !valid_id(&source.origin_id)
            || !valid_id(&source.actor_id)
            || source.retrieved_at_unix_ms == 0
            || raw_digest == [0; 32]
            || transformations.len() > MAX_SOURCE_TRANSFORMATIONS
            || taint_lineage.len() >= MAX_SOURCE_LINEAGE
            || taint_lineage.iter().any(Uuid::is_nil)
        {
            return Err(ProvenanceError::InvalidEnvelope);
        }
        let mut content_digest = raw_digest;
        for transformation in &transformations {
            if !valid_id(&transformation.id)
                || transformation.input_digest != content_digest
                || transformation.output_digest == [0; 32]
            {
                return Err(ProvenanceError::InvalidTransformation);
            }
            content_digest = transformation.output_digest;
        }
        taint_lineage.push(source_id);
        taint_lineage.sort_unstable();
        taint_lineage.dedup();
        Ok(Self {
            schema_version: SOURCE_ENVELOPE_VERSION,
            source_id,
            source,
            authority: SourceAuthority::Untrusted,
            raw_digest,
            content_digest,
            transformations,
            taint_lineage,
        })
    }

    pub fn source_id(&self) -> Uuid {
        self.source_id
    }

    pub fn source(&self) -> &SourceDescriptor {
        &self.source
    }

    pub fn authority(&self) -> SourceAuthority {
        self.authority
    }

    pub fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }

    pub fn taint_lineage(&self) -> &[Uuid] {
        &self.taint_lineage
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeWire {
    schema_version: u32,
    source_id: Uuid,
    source: SourceDescriptor,
    authority: SourceAuthority,
    raw_digest: [u8; 32],
    content_digest: [u8; 32],
    transformations: Vec<SourceTransformation>,
    taint_lineage: Vec<Uuid>,
}

impl TryFrom<EnvelopeWire> for SourceEnvelope {
    type Error = ProvenanceError;

    fn try_from(mut wire: EnvelopeWire) -> Result<Self, Self::Error> {
        if wire.schema_version != SOURCE_ENVELOPE_VERSION
            || wire.authority != SourceAuthority::Untrusted
            || !wire.taint_lineage.contains(&wire.source_id)
        {
            return Err(ProvenanceError::InvalidEnvelope);
        }
        let original_lineage = wire.taint_lineage.clone();
        wire.taint_lineage.retain(|id| *id != wire.source_id);
        let envelope = Self::new(
            wire.source_id,
            wire.source,
            wire.raw_digest,
            wire.transformations,
            wire.taint_lineage,
        )?;
        if envelope.content_digest != wire.content_digest
            || envelope.taint_lineage != original_lineage
        {
            return Err(ProvenanceError::InvalidEnvelope);
        }
        Ok(envelope)
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProvenanceError {
    #[error("invalid source envelope")]
    InvalidEnvelope,
    #[error("invalid source transformation chain")]
    InvalidTransformation,
}

#[cfg(test)]
#[path = "provenance_tests.rs"]
mod tests;
