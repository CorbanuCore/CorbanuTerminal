//! Host provenance and monotone derived context. Wire metadata is not authority;
//! only trusted ingress/storage adapters may supply it to a runtime context.

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest as _;
use sha2::Sha256;
use thiserror::Error;

pub const MAX_TAINT_SOURCES: usize = 64;

/// Opaque host-generated identifier, never a bearer credential or trust label.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "[u8; 16]", into = "[u8; 16]")]
pub struct SourceId([u8; 16]);

impl TryFrom<[u8; 16]> for SourceId {
    type Error = ProvenanceError;

    fn try_from(bytes: [u8; 16]) -> Result<Self, Self::Error> {
        if bytes == [0; 16] {
            return Err(ProvenanceError::InvalidIdentity);
        }
        Ok(Self(bytes))
    }
}

impl From<SourceId> for [u8; 16] {
    fn from(value: SourceId) -> Self {
        value.0
    }
}

/// None of these source classes grants human authority, even after sanitization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    File,
    Web,
    Document,
    Tool,
    Mcp,
    Connector,
    Email,
    Memory,
    Delegated,
    Unknown,
}

/// Created at host ingress, not deserialized from model/content payloads. The
/// digest binds the exact acquired bytes; derived content retains its ancestors'
/// taint instead of relabeling the original envelope with a new body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceEnvelope {
    schema_version: u32,
    source_id: SourceId,
    kind: SourceKind,
    content_sha256: [u8; 32],
}

impl SourceEnvelope {
    /// `source_id` and `kind` must come from the host adapter, never parsed labels.
    pub fn host_assigned(source_id: SourceId, kind: SourceKind, content: &[u8]) -> Self {
        Self {
            schema_version: 1,
            source_id,
            kind,
            content_sha256: Sha256::digest(content).into(),
        }
    }

    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn matches_content(&self, content: &[u8]) -> bool {
        self.content_sha256 == <[u8; 32]>::from(Sha256::digest(content))
    }
}

/// Bounded ancestry, not permission. Deserialization is for authenticated host
/// checkpoints only; unknown/unverifiable storage restores as `unknown()`.
/// Consumers must compare it to the current host context before protected use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(try_from = "TaintWire", deny_unknown_fields)]
pub struct TaintContext {
    #[schemars(range(min = 1, max = 1))]
    schema_version: u32,
    #[schemars(length(max = 64))]
    sources: BTreeSet<SourceId>,
    unknown_origin: bool,
}

impl TaintContext {
    /// Only a trusted host root with no external ancestors may start here.
    pub fn trusted_input() -> Self {
        Self {
            schema_version: 1,
            sources: BTreeSet::new(),
            unknown_origin: false,
        }
    }

    pub fn unknown() -> Self {
        Self {
            unknown_origin: true,
            ..Self::trusted_input()
        }
    }

    pub fn from_host_source(source: &SourceEnvelope) -> Self {
        Self {
            schema_version: 1,
            sources: BTreeSet::from([source.source_id]),
            unknown_origin: source.kind == SourceKind::Unknown,
        }
    }

    /// The same union applies to summary, compaction, memory, child, and resume
    /// joins. Overflow keeps bounded diagnostics and permanently marks unknown;
    /// dropping ancestry can never turn into a protected-use permission.
    pub fn derive(&self, other: &Self) -> Self {
        let sources: BTreeSet<_> = self.sources.union(&other.sources).copied().collect();
        let unknown_origin =
            self.unknown_origin || other.unknown_origin || sources.len() > MAX_TAINT_SOURCES;
        Self {
            schema_version: 1,
            sources: sources.into_iter().take(MAX_TAINT_SOURCES).collect(),
            unknown_origin,
        }
    }

    pub fn has_unknown_origin(&self) -> bool {
        self.unknown_origin
    }

    pub fn sources(&self) -> &BTreeSet<SourceId> {
        &self.sources
    }
}

impl Default for TaintContext {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaintWire {
    schema_version: u32,
    sources: Vec<SourceId>,
    unknown_origin: bool,
}

impl TryFrom<TaintWire> for TaintContext {
    type Error = ProvenanceError;

    fn try_from(value: TaintWire) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err(ProvenanceError::UnsupportedVersion);
        }
        if value.sources.len() > MAX_TAINT_SOURCES {
            return Err(ProvenanceError::TooManySources);
        }
        Ok(Self {
            schema_version: 1,
            sources: value.sources.into_iter().collect(),
            unknown_origin: value.unknown_origin,
        })
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProvenanceError {
    #[error("source identity must be host-issued and nonzero")]
    InvalidIdentity,
    #[error("unsupported taint schema version")]
    UnsupportedVersion,
    #[error("taint source bound exceeded")]
    TooManySources,
}
