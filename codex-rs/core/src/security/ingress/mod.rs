//! Trusted source admission. Descriptive wire metadata is not a capability.
//! Native protected requests remain closed until producer/screening adapters
//! supply admitted context; Permissive is deliberately untouched.
#![cfg_attr(not(test), allow(dead_code))]

use codex_content_security::ContentDigest;
use codex_content_security::ScreenedContent;
use codex_content_security::SourceBinding;
use codex_protocol::models::ResponseItem;
use codex_protocol::provenance::SOURCE_ENVELOPE_VERSION;
use codex_protocol::provenance::SourceDescriptor;
use codex_protocol::provenance::SourceEnvelope;
use codex_protocol::provenance::SourceKind;
use codex_protocol::provenance::SourceTransformation;
use codex_security_policy::SecurityLevel;
use serde_json::json;
use std::fmt::Write;
use thiserror::Error;
use uuid::Uuid;

pub(crate) const MAX_INGRESS_TEXT_BYTES: usize = 2_048;
const MAX_PROJECTION_BYTES: usize = 8_192;

#[derive(Clone)]
pub(crate) struct BoundIngressPolicy(pub(crate) super::EffectivePolicyView);

impl std::fmt::Debug for BoundIngressPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BoundIngressPolicy")
    }
}

/// Closed registry of producer contracts, not names supplied by tool content.
fn route_kind(route: &str) -> Result<SourceKind, IngressError> {
    match route {
        "web" => Ok(SourceKind::Web),
        "search" => Ok(SourceKind::Search),
        "file" => Ok(SourceKind::File),
        "transcript" => Ok(SourceKind::Transcript),
        "social" => Ok(SourceKind::Social),
        "trollbox" => Ok(SourceKind::Trollbox),
        "email" => Ok(SourceKind::Email),
        "mcp" => Ok(SourceKind::Mcp),
        "tool" => Ok(SourceKind::Tool),
        "plugin" => Ok(SourceKind::Plugin),
        "hook" => Ok(SourceKind::Hook),
        "child" => Ok(SourceKind::ChildAgent),
        _ => Err(IngressError::UnregisteredSource),
    }
}

/// Host-created pending binding. It is not deserializable or model-visible.
#[derive(Debug)]
pub(crate) struct PendingSource {
    envelope: SourceEnvelope,
    screening_binding: SourceBinding,
}

impl PendingSource {
    /// Prepare one complete bounded segment. Do not clip a prefix and admit it;
    /// the owning producer must segment and screen complete input first.
    pub(crate) fn prepare(
        route: &str,
        mut descriptor: SourceDescriptor,
        raw: &str,
        parents: &[SourceEnvelope],
    ) -> Result<(Self, String), IngressError> {
        descriptor.kind = route_kind(route)?;
        if raw.len() > MAX_INGRESS_TEXT_BYTES { return Err(IngressError::TooLarge); }
        let normalized = normalize(raw);
        if normalized.len() > MAX_INGRESS_TEXT_BYTES { return Err(IngressError::TooLarge); }
        let raw_digest = *ContentDigest::of(raw.as_bytes()).as_bytes();
        let content_digest = *ContentDigest::of(normalized.as_bytes()).as_bytes();
        let mut lineage: Vec<_> = parents.iter().flat_map(|parent| parent.taint_lineage().iter().copied()).collect();
        lineage.sort_unstable();
        lineage.dedup();
        let envelope = SourceEnvelope::new(
            Uuid::new_v4(), descriptor, raw_digest,
            vec![SourceTransformation { id: "model-data-escape-v1".into(), input_digest: raw_digest, output_digest: content_digest }],
            lineage,
        ).map_err(|_| IngressError::InvalidEnvelope)?;
        let envelope_bytes = serde_json::to_vec(&envelope).map_err(|_| IngressError::InvalidEnvelope)?;
        let screening_binding = SourceBinding::from_trusted_provenance(ContentDigest::of(&envelope_bytes), SOURCE_ENVELOPE_VERSION)
            .map_err(|_| IngressError::InvalidEnvelope)?;
        Ok((Self { envelope, screening_binding }, normalized))
    }

    pub(crate) fn envelope(&self) -> &SourceEnvelope { &self.envelope }
    pub(crate) fn screening_binding(&self) -> SourceBinding { self.screening_binding }

    /// An Allow verdict still only releases untrusted data. Match the exact
    /// host-issued source identity AND content, not a self-reported wire label.
    pub(crate) fn admit(self, screened: ScreenedContent) -> Result<AdmittedSource, IngressError> {
        let bytes = screened.bytes().into_raw_untrusted();
        if screened.target().binding().source() != self.screening_binding
            || ContentDigest::of(bytes).as_bytes() != &self.envelope.content_digest()
        { return Err(IngressError::BindingMismatch); }
        let data = std::str::from_utf8(bytes).map_err(|_| IngressError::InvalidEnvelope)?;
        let projection = serde_json::to_string(&json!({ "source": self.envelope, "data": data }))
            .map_err(|_| IngressError::InvalidEnvelope)?;
        if projection.len() > MAX_PROJECTION_BYTES { return Err(IngressError::TooLarge); }
        Ok(AdmittedSource { projection, raw_digest: self.envelope.raw_digest() })
    }
}

/// No constructor or Deserialize implementation outside this admission module.
#[derive(Clone)]
pub(crate) struct AdmittedSource { projection: String, raw_digest: [u8; 32] }

impl AdmittedSource {
    pub(crate) fn into_projection(self) -> String { self.projection }
}

// Escape complete text before any caller performs truncation. Escaping all
// non-ASCII also makes bidi/zero-width/confusable wrapper characters explicit.
fn normalize(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if !ch.is_ascii() || ch.is_control() || matches!(ch, '<' | '>') {
            let _ = write!(normalized, "\\u{{{:x}}}", u32::from(ch));
        } else { normalized.push(ch); }
    }
    normalized
}

/// Final native request gate shared by every provider wire adapter. Configured
/// intent is only a restrictive floor, not proof that protected mode is ready.
/// There is no native admitted-context carrier yet: raw/legacy history, forged
/// wrapper text and new provider/tool variants must all fail before networking.
pub(crate) fn check_native_request(
    level: SecurityLevel,
    _items: &[ResponseItem],
) -> Result<(), IngressError> {
    match level {
        SecurityLevel::Permissive => Ok(()),
        SecurityLevel::Moderate | SecurityLevel::Aggressive => Err(IngressError::NativeAdmissionUnavailable),
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum IngressError {
    #[error("unregistered source ingress")]
    UnregisteredSource,
    #[error("invalid source envelope")]
    InvalidEnvelope,
    #[error("source screening binding mismatch")]
    BindingMismatch,
    #[error("external content exceeds the admission bound")]
    TooLarge,
    #[error("protected source admission is unavailable; external context was not sent")]
    NativeAdmissionUnavailable,
    #[error("source admission registry is full or poisoned")]
    RegistryUnavailable,
}

mod native;
pub(crate) use native::NativeIngress;

#[cfg(test)]
#[path = "ingress_tests.rs"]
pub(crate) mod tests;
