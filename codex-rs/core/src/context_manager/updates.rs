//! Groups contextual fragments without moving them across explicit response items.

use crate::context::world_state::Placement;
use crate::context::world_state::WorldStateUpdate;
use crate::context::world_state::WorldStateUpdateContent;
use codex_context_fragments::RenderedFragment;
use codex_context_fragments::context_message;
use codex_protocol::models::ResponseItem;

/// A rendered fragment and the stable producer it is attributed to, if any (#578).
pub(crate) type SourcedRenderedFragment = (RenderedFragment, Option<&'static str>);

pub(crate) fn build_rendered_message(fragments: Vec<RenderedFragment>) -> Option<ResponseItem> {
    build_sourced_message(
        fragments
            .into_iter()
            .map(|fragment| (fragment, /*source_id*/ None))
            .collect(),
    )
}

/// [`build_rendered_message`] that records each fragment's producer ID with the message,
/// so provider adapters can deduplicate developer sections per producer (#578).
pub(crate) fn build_sourced_message(
    fragments: Vec<SourcedRenderedFragment>,
) -> Option<ResponseItem> {
    let role = fragments.first()?.0.role();
    debug_assert!(
        fragments
            .iter()
            .all(|(fragment, _)| fragment.role() == role)
    );
    Some(context_message(
        role,
        fragments
            .into_iter()
            .map(|(fragment, source_id)| (fragment.into_parts().1, source_id)),
    ))
}

pub(crate) fn merge_world_state_updates(updates: Vec<WorldStateUpdate>) -> Vec<ResponseItem> {
    let mut items = Vec::new();
    let mut pending = Vec::<SourcedRenderedFragment>::new();
    for update in updates {
        let fragment = match update.content {
            WorldStateUpdateContent::Fragment(fragment) => fragment,
            WorldStateUpdateContent::Item(item) => {
                items.extend(build_sourced_message(std::mem::take(&mut pending)));
                items.push(*item);
                continue;
            }
        };
        let rendered = (fragment.render_fragment(), fragment.source_id());
        if update.placement != Placement::Mergeable {
            items.extend(build_sourced_message(std::mem::take(&mut pending)));
            items.extend(build_sourced_message(vec![rendered]));
            continue;
        }
        if pending
            .first()
            .is_some_and(|(previous, _)| previous.role() != rendered.0.role())
        {
            items.extend(build_sourced_message(std::mem::take(&mut pending)));
        }
        pending.push(rendered);
    }
    items.extend(build_sourced_message(pending));
    items
}

#[cfg(test)]
#[path = "updates_tests.rs"]
mod tests;
