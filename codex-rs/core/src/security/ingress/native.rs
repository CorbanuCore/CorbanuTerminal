//! Exact-item sidecar for provider adapters. This is not stored in text or in
//! provider-owned passthrough fields; forged wrapper text cannot populate it.

use super::AdmittedSource;
use super::IngressError;
use super::MAX_INGRESS_TEXT_BYTES;
use super::NativeScreeningCandidate;
use super::PendingSource;
use super::structural::LabelledCache;
use super::structural::MessageOrigin;
use crate::context::ContextualUserFragment;
use crate::context::ProvenanceContext;
use codex_content_security::ContentDigest;
use codex_content_security::ScreenedContent;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use codex_protocol::provenance::SourceDescriptor;
use codex_protocol::provenance::SourceKind;
use std::collections::HashMap;

const MAX_ADMITTED_ITEMS: usize = 256;
/// Labelled mode keeps only digests and kinds, so it can hold a long session.
const MAX_LABELLED_REGISTRATIONS: usize = 65_536;

// Enforce the raw bound while serializing, before allocating a whole oversized
// history item merely to discover that it cannot enter this bounded carrier.
fn item_bytes(item: &ResponseItem) -> Result<Vec<u8>, IngressError> {
    struct BoundedBytes(Vec<u8>);
    impl std::io::Write for BoundedBytes {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > MAX_INGRESS_TEXT_BYTES.saturating_sub(self.0.len()) {
                return Err(std::io::Error::other("source item exceeds admission bound"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = BoundedBytes(Vec::new());
    serde_json::to_writer(&mut writer, item).map_err(|_| IngressError::TooLarge)?;
    Ok(writer.0)
}

/// Role plus exact text parts. Media is excluded so history normalization
/// that strips images for a text-only model keeps the host registration.
fn message_key(item: &ResponseItem) -> Option<ContentDigest> {
    let ResponseItem::Message { role, content, .. } = item else {
        return None;
    };
    let texts: Vec<&str> = content
        .iter()
        .filter_map(|part| match part {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect();
    serde_json::to_vec(&(role, texts))
        .ok()
        .map(|bytes| ContentDigest::of(&bytes))
}

#[derive(Default)]
pub(crate) struct NativeIngress {
    admitted: HashMap<ContentDigest, AdmittedSource>,
    pending: HashMap<ContentDigest, PendingSource>,
    calls: HashMap<ContentDigest, SourceKind>,
    unavailable: bool,
    /// `source_envelopes`: project labelled data instead of failing closed.
    labelled_mode: bool,
    messages: HashMap<ContentDigest, MessageOrigin>,
    pub(super) labelled: LabelledCache,
    /// Restored history has no recorded origins; reinject host context once.
    host_context_reinjection: bool,
}

impl std::fmt::Debug for NativeIngress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeIngress")
            .field("items", &self.admitted.len())
            .finish()
    }
}

impl NativeIngress {
    pub(crate) fn set_labelled_mode(&mut self, enabled: bool) {
        self.labelled_mode = enabled;
    }

    pub(crate) fn labelled_mode(&self) -> bool {
        self.labelled_mode
    }

    /// Record the host-chosen origin of exact history messages. Called only
    /// from Core record seams; a full registry degrades new messages to
    /// unattributed (labelled) data and never upgrades anything.
    pub(crate) fn register_messages(&mut self, items: &[ResponseItem], origin: MessageOrigin) {
        for item in items {
            if !matches!(item, ResponseItem::Message { .. }) {
                continue;
            }
            let Some(key) = message_key(item) else {
                continue;
            };
            if self.messages.len() >= MAX_LABELLED_REGISTRATIONS
                && !self.messages.contains_key(&key)
            {
                continue;
            }
            // First registration wins: a later external copy of identical
            // bytes cannot downgrade or upgrade the human/host record.
            self.messages.entry(key).or_insert(origin);
        }
    }

    /// Restored or forked history carries no host-recorded origins, so its
    /// messages stay labelled; ask for fresh host context on the next turn.
    pub(crate) fn note_restored_history(&mut self, items: &[ResponseItem]) {
        if !self.labelled_mode {
            return;
        }
        self.host_context_reinjection = true;
        // Keep the real route label for restored tool calls (still untrusted).
        for item in items {
            match item {
                ResponseItem::FunctionCall { call_id, .. }
                | ResponseItem::CustomToolCall { call_id, .. } => {
                    self.register_call(call_id, SourceKind::Tool);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn take_host_context_reinjection(&mut self) -> bool {
        std::mem::take(&mut self.host_context_reinjection)
    }

    pub(super) fn message_origin(&self, item: &ResponseItem) -> Option<MessageOrigin> {
        self.messages.get(&message_key(item)?).copied()
    }

    pub(super) fn call_kind(&self, call_id: &str) -> Option<SourceKind> {
        self.calls
            .get(&ContentDigest::of(call_id.as_bytes()))
            .copied()
    }

    /// Invoked by the host tool dispatcher, never by source labels in output.
    pub(crate) fn register_call(&mut self, call_id: &str, kind: SourceKind) {
        let key = ContentDigest::of(call_id.as_bytes());
        let capacity = if self.labelled_mode {
            MAX_LABELLED_REGISTRATIONS
        } else {
            MAX_ADMITTED_ITEMS
        };
        if self.calls.len() >= capacity && !self.calls.contains_key(&key) {
            self.unavailable = true;
            return;
        }
        match self.calls.get(&key) {
            // The MCP adapter refines the generic router's observed tool route.
            Some(SourceKind::Tool) if kind == SourceKind::Mcp => {
                self.calls.insert(key, kind);
            }
            Some(existing) if *existing != kind => {
                self.unavailable = true;
            }
            Some(_) => {}
            None => {
                self.calls.insert(key, kind);
            }
        }
    }

    /// Observe exact post-preparation history items at the trusted append seam.
    /// No body or role label can supply an admission capability here.
    pub(crate) fn observe(&mut self, items: &[ResponseItem], retrieved_at_unix_ms: u64) {
        for item in items {
            let Ok(bytes) = item_bytes(item) else {
                self.unavailable = true;
                return;
            };
            let key = ContentDigest::of(&bytes);
            if self.pending.contains_key(&key) || self.admitted.contains_key(&key) {
                continue;
            }
            if self.pending.len() + self.admitted.len() >= MAX_ADMITTED_ITEMS {
                self.unavailable = true;
                return;
            }
            let route = match item {
                ResponseItem::Message { .. } => "transcript",
                ResponseItem::AgentMessage { .. } => "child",
                ResponseItem::FunctionCallOutput { call_id, .. }
                | ResponseItem::CustomToolCallOutput { call_id, .. } => {
                    match self.calls.get(&ContentDigest::of(call_id.as_bytes())) {
                        Some(SourceKind::Mcp) => "mcp",
                        Some(SourceKind::Tool) => "tool",
                        _ => continue, // Unregistered output remains absent and cannot be projected.
                    }
                }
                _ => continue,
            };
            let descriptor = SourceDescriptor {
                kind: SourceKind::Unknown,
                origin_id: key.to_hex(),
                actor_id: "native-context".into(),
                retrieved_at_unix_ms,
            };
            let Ok(text) = std::str::from_utf8(&bytes) else {
                self.unavailable = true;
                return;
            };
            match PendingSource::prepare(route, descriptor, text, &[]) {
                Ok((pending, _)) => {
                    self.pending.insert(key, pending);
                }
                Err(_) => {
                    self.unavailable = true;
                    return;
                }
            }
        }
    }

    /// Preserve the exact bounded payload and host identity for a producer.
    pub(crate) fn screening_candidate(
        &self,
        item: &ResponseItem,
    ) -> Result<NativeScreeningCandidate, IngressError> {
        if self.unavailable {
            return Err(IngressError::RegistryUnavailable);
        }
        let bytes = item_bytes(item)?;
        self.pending
            .get(&ContentDigest::of(&bytes))
            .map(PendingSource::screening_candidate)
            .ok_or(IngressError::NativeAdmissionUnavailable)
    }

    /// Only a producer's complete matching screening result can advance pending
    /// context. A failed match consumes the candidate and never restores raw data.
    pub(crate) fn admit_screened(
        &mut self,
        item: &ResponseItem,
        screened: ScreenedContent,
    ) -> Result<(), IngressError> {
        let bytes = item_bytes(item)?;
        let pending = self
            .pending
            .remove(&ContentDigest::of(&bytes))
            .ok_or(IngressError::NativeAdmissionUnavailable)?;
        let source = pending.admit(screened)?;
        self.insert(item, source)
    }

    /// Install a trusted producer's screened exact item. Replacements and
    /// capacity pressure fail; dropping an old binding must never admit raw data.
    pub(crate) fn insert(
        &mut self,
        item: &ResponseItem,
        source: AdmittedSource,
    ) -> Result<(), IngressError> {
        let bytes = item_bytes(item)?;
        let digest = ContentDigest::of(&bytes);
        if digest.as_bytes() != &source.raw_digest {
            return Err(IngressError::BindingMismatch);
        }
        if self.admitted.len() >= MAX_ADMITTED_ITEMS || self.admitted.contains_key(&digest) {
            return Err(IngressError::RegistryUnavailable);
        }
        self.admitted.insert(digest, source);
        Ok(())
    }

    /// Projection is append-stable: no timestamps/IDs are regenerated for retries.
    /// Every item requires the exact host-held binding, including user messages;
    /// a human transport does not turn quoted content into an action grant.
    pub(crate) fn project(
        &self,
        items: &[ResponseItem],
    ) -> Result<Vec<ResponseItem>, IngressError> {
        if self.unavailable {
            return Err(IngressError::RegistryUnavailable);
        }
        if items.is_empty() {
            return Err(IngressError::NativeAdmissionUnavailable);
        }
        items
            .iter()
            .map(|item| {
                let bytes = item_bytes(item)?;
                let source = self
                    .admitted
                    .get(&ContentDigest::of(&bytes))
                    .ok_or(IngressError::NativeAdmissionUnavailable)?;
                let fragment = ProvenanceContext::from_admitted(source.clone());
                let (start, end) = fragment.markers();
                let text = format!("{start}\n{}\n{end}", fragment.body());
                match item {
                    ResponseItem::Message { id, .. } | ResponseItem::AgentMessage { id, .. } => {
                        Ok(ResponseItem::Message {
                            id: id.clone(),
                            role: "user".into(),
                            content: vec![ContentItem::InputText { text }],
                            phase: None,
                            internal_chat_message_metadata_passthrough: None,
                        })
                    }
                    ResponseItem::FunctionCallOutput { id, call_id, .. } => {
                        Ok(ResponseItem::FunctionCallOutput {
                            id: id.clone(),
                            call_id: call_id.clone(),
                            output: FunctionCallOutputPayload::from_text(text),
                            internal_chat_message_metadata_passthrough: None,
                        })
                    }
                    ResponseItem::CustomToolCallOutput {
                        id, call_id, name, ..
                    } => Ok(ResponseItem::CustomToolCallOutput {
                        id: id.clone(),
                        call_id: call_id.clone(),
                        name: name.clone(),
                        output: FunctionCallOutputPayload::from_text(text),
                        internal_chat_message_metadata_passthrough: None,
                    }),
                    _ => Err(IngressError::UnregisteredSource),
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "native_tests.rs"]
mod tests;
