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

    /// Retains message content items, keeping recorded fragment provenance aligned.
    pub fn retain_message_content(&mut self, mut keep: impl FnMut(&mut ContentItem) -> bool) {
        let Self::Message {
            content,
            internal_chat_message_metadata_passthrough: metadata,
            ..
        } = self
        else {
            return;
        };
        let Some(sources) = metadata
            .as_mut()
            .and_then(|metadata| metadata.context_fragment_sources.take())
        else {
            content.retain_mut(keep);
            return;
        };
        let mut sources = (sources.len() == content.len()).then(|| sources.into_iter());
        let mut retained_sources = Vec::with_capacity(content.len());
        content.retain_mut(|content_item| {
            let source = sources.as_mut().and_then(Iterator::next).flatten();
            let retained = keep(content_item);
            if retained {
                retained_sources.push(source);
            }
            retained
        });
        if sources.is_some()
            && retained_sources.iter().any(Option::is_some)
            && let Some(metadata) = metadata.as_mut()
        {
            metadata.context_fragment_sources = Some(retained_sources);
        } else {
            clear_empty_passthrough(metadata);
        }
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
