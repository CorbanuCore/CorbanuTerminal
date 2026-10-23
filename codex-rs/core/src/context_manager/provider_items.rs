//! #578: history items another provider produced, prepared for the provider a
//! request goes to.
//!
//! A session can switch providers between turns. Model output items carry
//! provider-bound state: reasoning (encrypted for one organization, signed by
//! one Anthropic-compatible host, or plain text a Chat host streamed), item
//! IDs a stateful host could look up, and hosted-tool records. A Responses
//! request with `store: false` can only read an item ID as a reference to an
//! item that host stored, so another provider's reasoning there fails every
//! later turn with `404 Item with id 'rs_…' not found`.
//!
//! Model output items record the provider (and named account) that produced
//! them ([`ResponseItem::producer`]). Before a request is built:
//! - Items from the same provider are sent as they are, so its requests and
//!   prompt cache are unchanged.
//! - Another provider's items are adapted to the target wire:
//!   - Responses: its reasoning, hosted web searches, image generations and
//!     remote-compaction summaries are dropped (another host's encrypted or
//!     plain reasoning and summaries cannot be sent to a Responses host, only
//!     referenced; Chat and Anthropic requests leave them out too); its other
//!     items keep their content but lose their IDs and encrypted function
//!     arguments, so they are sent inline, never as references. The IDs are
//!     removed by the client ([`strip_foreign_references`]) after source
//!     labelling, which looks items up by their recorded bytes.
//!   - Anthropic: its reasoning (another host's signed thinking) and
//!     web-search blocks are dropped. Messages and tool calls carry no item
//!     IDs on this wire.
//!   - Chat Completions: unchanged. This wire sends no item IDs or encrypted
//!     state; reasoning text is sent inline where the model replays it.
//! - Items with no recorded producer (sessions recorded before #578, or items
//!   the host built itself) are left as they are, except what a
//!   `store: false` Responses request could only send as a reference:
//!   reasoning or a compaction with no encrypted content, or an Anthropic
//!   thinking block. Those are dropped, and so are Anthropic web-search
//!   blocks.
//!
//! The producer is the provider id and named account, not the model: a model
//! switch inside one multi-vendor gateway (Vercel, OpenRouter) is not a
//! provider switch. Encrypted content inside local items (agent mail, tool
//! outputs) carries no producer and is sent as before.

use crate::session::turn_context::TurnContext;
use codex_model_provider_info::NamedProviderAccount;
use codex_model_provider_info::WireApi;
use codex_protocol::models::ResponseItem;

/// The producer recorded on a model output item: `<provider id>`, or
/// `<provider id>:<account>` for a named account (PF-84).
pub(crate) fn producer_key(provider_id: &str, account: Option<&NamedProviderAccount>) -> String {
    match account {
        Some(account) => account.to_string(),
        None => provider_id.to_string(),
    }
}

/// The provider a request is about to be sent to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderTarget {
    /// [`TurnContext::output_producer`] of the turn.
    pub(crate) producer: String,
    pub(crate) wire: WireApi,
    /// Whether a Responses host stores items (`store: true`), so an item ID
    /// can be a valid reference to one of its own items.
    pub(crate) stores_items: bool,
}

impl ProviderTarget {
    pub(crate) fn for_turn(turn_context: &TurnContext) -> Self {
        let info = turn_context.provider.info();
        Self {
            producer: turn_context.output_producer(),
            wire: info.wire_api,
            // Mirrors the `store` flag of `ModelClient::build_responses_request`.
            stores_items: codex_api::is_azure_responses_provider(
                &info.name,
                info.base_url.as_deref(),
            ) || info.is_vercel(),
        }
    }
}

/// Adapts `items` for `target`; see the module docs. Returns whether another
/// provider's compaction summary was left out (the caller logs it once per
/// session).
pub(crate) fn adapt_for_provider(items: &mut Vec<ResponseItem>, target: &ProviderTarget) -> bool {
    let mut dropped_summary = false;
    items.retain(|item| match item.producer() {
        Some(producer) if producer == target.producer => true,
        Some(_) => {
            let keep = adapt_foreign(item, target.wire);
            dropped_summary |= !keep
                && matches!(
                    item,
                    ResponseItem::Compaction { .. } | ResponseItem::ContextCompaction { .. }
                );
            keep
        }
        None => keep_unattributed(item, target),
    });
    dropped_summary
}

/// Another provider's item: returns whether it is kept. Kept items stay as
/// recorded (see [`strip_foreign_references`]).
fn adapt_foreign(item: &ResponseItem, wire: WireApi) -> bool {
    match wire {
        WireApi::Chat => true,
        WireApi::Anthropic => !matches!(
            item,
            ResponseItem::Reasoning { .. } | ResponseItem::WebSearchCall { .. }
        ),
        WireApi::Responses => !matches!(
            item,
            ResponseItem::Reasoning { .. }
                | ResponseItem::WebSearchCall { .. }
                | ResponseItem::ImageGenerationCall { .. }
                | ResponseItem::Compaction { .. }
                | ResponseItem::ContextCompaction { .. }
        ),
    }
}

/// The second step for a Responses request, run by the client once source
/// labelling has looked the items up by their recorded bytes: another
/// producer's items lose their IDs and encrypted function arguments, so they
/// are sent inline, never as references.
pub(crate) fn strip_foreign_references(items: &mut [ResponseItem], producer: &str) {
    for item in items {
        if item
            .producer()
            .is_none_or(|item_producer| item_producer == producer)
        {
            continue;
        }
        item.set_id(/*new_id*/ None);
        if let ResponseItem::FunctionCall {
            encrypted_function_args,
            ..
        } = item
        {
            *encrypted_function_args = None;
        }
    }
}

/// An item with no recorded producer: kept unless a Responses request could
/// only send it as an unresolvable reference, or it is Anthropic-native.
fn keep_unattributed(item: &ResponseItem, target: &ProviderTarget) -> bool {
    if target.wire != WireApi::Responses {
        return true;
    }
    match item {
        ResponseItem::Reasoning {
            encrypted_content,
            anthropic_content_block,
            ..
        } => {
            anthropic_content_block.is_none()
                && (target.stores_items || encrypted_content.is_some())
        }
        ResponseItem::WebSearchCall {
            anthropic_content_block,
            ..
        } => anthropic_content_block.is_none(),
        ResponseItem::ContextCompaction {
            encrypted_content, ..
        } => target.stores_items || encrypted_content.is_some(),
        _ => true,
    }
}

#[cfg(test)]
#[path = "provider_items_tests.rs"]
mod tests;
