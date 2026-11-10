mod history;
mod image_availability;
mod normalize;
pub(crate) mod provider_items;
pub(crate) mod updates;

pub(crate) use history::ContextManager;
pub(crate) use history::HistoryReplacement;
pub(crate) use history::estimate_image_reference_bytes;
pub(crate) use history::estimate_item_token_count;
pub(crate) use history::is_user_turn_boundary;
pub(crate) use image_availability::replace_unresolvable_image_files;
pub(crate) use normalize::AUDIO_CONTENT_OMITTED_PLACEHOLDER;
pub(crate) use normalize::IMAGE_CONTENT_OMITTED_PLACEHOLDER;

/// [`normalize::strip_images_when_unsupported`] over bare response items, as the security
/// ingress tests hold them.
#[cfg(test)]
pub(crate) fn strip_images_when_unsupported(
    input_modalities: &[codex_protocol::openai_models::InputModality],
    items: &mut [codex_protocol::models::ResponseItem],
) {
    let mut envelopes: Vec<_> = items
        .iter()
        .cloned()
        .map(codex_history::ResponseItemEnvelope::new)
        .collect();
    normalize::strip_images_when_unsupported(input_modalities, &mut envelopes);
    for (item, envelope) in items.iter_mut().zip(envelopes) {
        *item = envelope.into_item();
    }
}
