//! Local provenance of contextual message fragments.
//!
//! Provider adapters use it to keep the newest copy of each producer's context section when
//! several producers share a marker. It never reaches a model provider.

use super::ContentItem;
use super::InternalChatMessageMetadataPassthrough;
use super::ResponseItem;

impl ResponseItem {
    /// Returns the recorded producer of a message's `content[index]`.
    ///
    /// Returns `None` for unattributed content and for provenance that no longer aligns with the
    /// message content.
    pub fn context_fragment_source(&self, index: usize) -> Option<&str> {
        let Self::Message {
            content,
            internal_chat_message_metadata_passthrough: Some(metadata),
            ..
        } = self
        else {
            return None;
        };
        metadata
            .context_fragment_sources
            .as_ref()
            .filter(|sources| sources.len() == content.len())
            .and_then(|sources| sources.get(index)?.as_deref())
    }

    /// Retains message content items, keeping recorded fragment provenance aligned, along with
    /// upstream's per-item kinds and metadata (each kept only while it aligns with the content).
    pub fn retain_message_content(&mut self, mut keep: impl FnMut(&mut ContentItem) -> bool) {
        let Self::Message {
            content,
            internal_chat_message_metadata_passthrough: metadata,
            ..
        } = self
        else {
            return;
        };
        let mut kept = Vec::with_capacity(content.len());
        content.retain_mut(|content_item| {
            let retained = keep(content_item);
            kept.push(retained);
            retained
        });
        let Some(passthrough) = metadata.as_mut() else {
            return;
        };
        fn retain_aligned<T>(values: &mut Option<Vec<T>>, kept: &[bool]) {
            if let Some(items) = values.take()
                && items.len() == kept.len()
            {
                *values = Some(
                    items
                        .into_iter()
                        .zip(kept)
                        .filter_map(|(item, kept)| kept.then_some(item))
                        .collect(),
                );
            } else {
                // Not aligned with the content: dropped rather than mislabel other items.
                *values = None;
            }
        }
        if passthrough.content_item_kinds.is_some() {
            retain_aligned(&mut passthrough.content_item_kinds, &kept);
        }
        if passthrough.content_item_metadata.is_some() {
            retain_aligned(&mut passthrough.content_item_metadata, &kept);
        }
        if passthrough.context_fragment_sources.is_some() {
            retain_aligned(&mut passthrough.context_fragment_sources, &kept);
            if passthrough
                .context_fragment_sources
                .as_ref()
                .is_some_and(|sources| sources.iter().all(Option::is_none))
            {
                passthrough.context_fragment_sources = None;
            }
        }
        clear_empty_passthrough(metadata);
    }

    /// Removes local fragment provenance before serializing the item for a model provider.
    pub fn clear_context_fragment_sources(&mut self) {
        if let Self::Message {
            internal_chat_message_metadata_passthrough: metadata,
            ..
        } = self
            && let Some(passthrough) = metadata.as_mut()
            && passthrough.context_fragment_sources.take().is_some()
        {
            clear_empty_passthrough(metadata);
        }
    }
}

/// Drops passthrough metadata that only carried local fragment provenance, so the provider sees
/// the same payload as for an unattributed message.
fn clear_empty_passthrough(metadata: &mut Option<InternalChatMessageMetadataPassthrough>) {
    if metadata
        .as_ref()
        .is_some_and(|metadata| *metadata == InternalChatMessageMetadataPassthrough::default())
    {
        *metadata = None;
    }
}

#[cfg(test)]
#[path = "context_fragment_sources_tests.rs"]
mod tests;
