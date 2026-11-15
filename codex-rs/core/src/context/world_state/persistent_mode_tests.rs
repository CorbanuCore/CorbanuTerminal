//! Covers persistent-context transitions independently of effort selection.

use super::*;
use crate::context::world_state::WorldState;
use pretty_assertions::assert_eq;

/// Corbanu (S1c): World State stamps each rendered fragment with its section's producer ID
/// (`context_fragment_sources`, used to deduplicate context for non-OpenAI providers), so the
/// expected item carries the same ID.
fn sourced<F: ContextualUserFragment + 'static>(
    source_id: &'static str,
    fragment: F,
) -> codex_protocol::models::ResponseItem {
    ContextualUserFragment::into(codex_context_fragments::SourcedFragment::new(
        source_id,
        Box::new(fragment),
    ))
}

#[test]
fn persistent_instructions_follow_mode_and_catalog_updates_without_duplicates() {
    let mut history = Vec::new();
    let mut previous = None;
    let replacement = format!("{REPLACEMENT_NOTICE}\n\nupdated instructions");

    for (enabled, instructions, expected) in [
        (false, "", None),
        (true, "instructions", Some("instructions")),
        (true, "instructions", None),
        (true, "updated instructions", Some(replacement.as_str())),
        (true, "", Some(REMOVAL_NOTICE)),
        (true, "", None),
        (true, "instructions", Some("instructions")),
        (false, "", Some(REMOVAL_NOTICE)),
        (false, "", None),
    ] {
        let mut world_state = WorldState::default();
        world_state.add_section(PersistentModeState::new(
            enabled,
            instructions,
            /*send_user_message_async_available*/ false,
        ));
        let (snapshot, fragments) =
            world_state.render_history_fragment_diff(previous.as_ref(), &history);
        let updates = fragments
            .into_iter()
            .map(ContextualUserFragment::into_boxed_response_item)
            .collect::<Vec<_>>();
        assert_eq!(
            updates,
            expected
                .map(|instructions| {
                    sourced(
                        <PersistentModeState as WorldStateSection>::ID,
                        PersistentModeState {
                            instructions: instructions.to_string(),
                        },
                    )
                })
                .into_iter()
                .collect::<Vec<_>>()
        );
        history.extend(updates);
        previous = Some(snapshot);
    }
}

#[test]
fn retained_persistent_instructions_are_replaced_or_retired_without_a_snapshot() {
    let retained = ContextualUserFragment::into(PersistentModeState {
        instructions: "previous instructions".to_string(),
    });
    for (enabled, expected) in [
        (
            true,
            format!("{REPLACEMENT_NOTICE}\n\ncurrent instructions"),
        ),
        (false, REMOVAL_NOTICE.to_string()),
    ] {
        let mut world_state = WorldState::default();
        world_state.add_section(PersistentModeState::new(
            enabled,
            "current instructions",
            /*send_user_message_async_available*/ false,
        ));
        assert_eq!(
            world_state
                .render_history_fragment_diff(
                    /*previous*/ None,
                    std::slice::from_ref(&retained)
                )
                .1
                .into_iter()
                .map(ContextualUserFragment::into_boxed_response_item)
                .collect::<Vec<_>>(),
            vec![sourced(
                <PersistentModeState as WorldStateSection>::ID,
                PersistentModeState {
                    instructions: expected,
                },
            )]
        );
    }
}
