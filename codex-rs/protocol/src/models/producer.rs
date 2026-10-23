//! Local provenance of model output items (#578).
//!
//! A session can switch providers between turns. Items one provider produced (reasoning, its
//! item IDs, encrypted or signed state) mean nothing to another one, and a stateless provider
//! can only read an item ID as a reference to an item it stored. Request building compares the
//! recorded producer with the provider it is about to call. It is removed from every provider
//! request payload and from text built from items for a model (labelled history, memories).

use super::InternalChatMessageMetadataPassthrough;
use super::ResponseItem;

impl ResponseItem {
    /// Returns the provider (and named account) whose response produced this item, if recorded.
    pub fn producer(&self) -> Option<&str> {
        self.internal_chat_message_metadata_passthrough()
            .and_then(|metadata| metadata.producer.as_deref())
            .filter(|producer| !producer.is_empty())
    }

    /// Records the provider whose response produced this item.
    pub fn set_producer(&mut self, producer: &str) {
        if producer.is_empty() {
            return;
        }
        if let Some(metadata) = self.internal_chat_message_metadata_passthrough_mut() {
            metadata
                .get_or_insert_with(InternalChatMessageMetadataPassthrough::default)
                .producer = Some(producer.to_string());
        }
    }

    /// Passthrough metadata carrying only this item's producer, for an item
    /// rebuilt from this one (labelled history), so request building still
    /// knows whose it was. `None` when no producer is recorded.
    pub fn producer_only_metadata(&self) -> Option<InternalChatMessageMetadataPassthrough> {
        self.producer()
            .map(|producer| InternalChatMessageMetadataPassthrough {
                producer: Some(producer.to_string()),
                ..Default::default()
            })
    }

    /// Removes all local-only provenance (fragment sources and producer) before the item is
    /// serialized for a model provider or a client, leaving the payload it had without it.
    pub fn clear_local_provenance(&mut self) {
        self.clear_context_fragment_sources();
        let Some(metadata) = self.internal_chat_message_metadata_passthrough_mut() else {
            return;
        };
        if let Some(passthrough) = metadata.as_mut()
            && passthrough.producer.take().is_some()
            && *passthrough == InternalChatMessageMetadataPassthrough::default()
        {
            *metadata = None;
        }
    }
}

#[cfg(test)]
#[path = "producer_tests.rs"]
mod tests;
