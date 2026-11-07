<<<<<<< HEAD
use crate::context::ContextualUserFragment;
||||||| 413492cd6c
use crate::context::ContextualUserFragment;
use codex_protocol::models::ContentItem;
=======
//! Groups contextual fragments without moving them across explicit response items.

use crate::context::world_state::Placement;
use crate::context::world_state::WorldStateUpdate;
use crate::context::world_state::WorldStateUpdateContent;
use codex_context_fragments::RenderedFragment;
use codex_context_fragments::message_from_parts;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_protocol::models::ResponseItem;

<<<<<<< HEAD
/// Model-visible context text and the stable producer it is attributed to, if any.
pub(crate) struct ContextSection {
    text: String,
    source_id: Option<&'static str>,
}

impl ContextSection {
    pub(crate) fn new(text: String, source_id: Option<&'static str>) -> Self {
        Self { text, source_id }
    }

    pub(crate) fn attributed(text: String, source_id: &'static str) -> Self {
        Self::new(text, Some(source_id))
    }

    pub(crate) fn from_fragment(fragment: &dyn ContextualUserFragment) -> Self {
        Self::new(fragment.render(), fragment.source_id())
    }
}

impl From<String> for ContextSection {
    fn from(text: String) -> Self {
        Self::new(text, /*source_id*/ None)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MessageGroup {
    Standalone,
    Mergeable,
}

pub(crate) fn build_developer_update_item(
    sections: Vec<impl Into<ContextSection>>,
) -> Option<ResponseItem> {
    build_text_message("developer", sections.into_iter().map(Into::into).collect())
}

pub(crate) fn build_contextual_user_message(text_sections: Vec<String>) -> Option<ResponseItem> {
    build_text_message("user", text_sections.into_iter().map(Into::into).collect())
}

pub(crate) fn merge_contextual_fragments(
    fragments: Vec<Box<dyn ContextualUserFragment>>,
) -> Vec<ResponseItem> {
    let mut messages: Vec<(&str, MessageGroup, Vec<ContextSection>)> =
        Vec::with_capacity(fragments.len());
    for fragment in fragments {
        let role = fragment.role();
        let group = if fragment.requires_separate_message() {
            MessageGroup::Standalone
        } else {
            MessageGroup::Mergeable
        };
        let section = ContextSection::from_fragment(fragment.as_ref());
        match messages.last_mut() {
            Some((previous_role, previous_group, sections))
                if *previous_role == role
                    && *previous_group == MessageGroup::Mergeable
                    && group == MessageGroup::Mergeable =>
            {
                sections.push(section);
            }
            _ => messages.push((role, group, vec![section])),
        }
    }
    messages
||||||| 413492cd6c
#[derive(Clone, Copy, PartialEq, Eq)]
enum MessageGroup {
    Standalone,
    Mergeable,
}

pub(crate) fn build_developer_update_item(text_sections: Vec<String>) -> Option<ResponseItem> {
    build_text_message("developer", text_sections)
}

pub(crate) fn build_contextual_user_message(text_sections: Vec<String>) -> Option<ResponseItem> {
    build_text_message("user", text_sections)
}

pub(crate) fn merge_contextual_fragments(
    fragments: Vec<Box<dyn ContextualUserFragment>>,
) -> Vec<ResponseItem> {
    let mut messages: Vec<(&str, MessageGroup, Vec<String>)> = Vec::with_capacity(fragments.len());
    for fragment in fragments {
        let role = fragment.role();
        let group = if fragment.requires_separate_message() {
            MessageGroup::Standalone
        } else {
            MessageGroup::Mergeable
        };
        let text = fragment.render();
        match messages.last_mut() {
            Some((previous_role, previous_group, text_sections))
                if *previous_role == role
                    && *previous_group == MessageGroup::Mergeable
                    && group == MessageGroup::Mergeable =>
            {
                text_sections.push(text);
            }
            _ => messages.push((role, group, vec![text])),
        }
    }
    messages
=======
pub(crate) fn build_rendered_message(fragments: Vec<RenderedFragment>) -> Option<ResponseItem> {
    let role = fragments.first()?.role();
    debug_assert!(fragments.iter().all(|fragment| fragment.role() == role));
    let content = fragments
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .into_iter()
<<<<<<< HEAD
        .filter_map(|(role, _, sections)| build_text_message(role, sections))
        .collect()
}

fn build_text_message(role: &str, sections: Vec<ContextSection>) -> Option<ResponseItem> {
    if sections.is_empty() {
        return None;
    }
    Some(codex_context_fragments::context_message(
        role,
        sections
            .into_iter()
            .map(|section| (section.text, section.source_id)),
    ))
||||||| 413492cd6c
        .filter_map(|(role, _, text_sections)| build_text_message(role, text_sections))
        .collect()
}

fn build_text_message(role: &str, text_sections: Vec<String>) -> Option<ResponseItem> {
    if text_sections.is_empty() {
        return None;
    }

    let content = text_sections
        .into_iter()
        .map(|text| ContentItem::InputText { text })
        .collect();

    Some(ResponseItem::Message {
        id: None,
        role: role.to_string(),
        content,
        phase: None,
        internal_chat_message_metadata_passthrough: None,
    })
=======
        .map(|fragment| fragment.into_parts().1)
        .collect();

    Some(message_from_parts(role, content))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

pub(crate) fn merge_world_state_updates(updates: Vec<WorldStateUpdate>) -> Vec<ResponseItem> {
    let mut items = Vec::new();
    let mut pending = Vec::<RenderedFragment>::new();
    for update in updates {
        let fragment = match update.content {
            WorldStateUpdateContent::Fragment(fragment) => fragment,
            WorldStateUpdateContent::Item(item) => {
                items.extend(build_rendered_message(std::mem::take(&mut pending)));
                items.push(*item);
                continue;
            }
        };
        let rendered = fragment.render_fragment();
        if update.placement != Placement::Mergeable {
            items.extend(build_rendered_message(std::mem::take(&mut pending)));
            items.extend(build_rendered_message(vec![rendered]));
            continue;
        }
        if pending
            .first()
            .is_some_and(|previous| previous.role() != rendered.role())
        {
            items.extend(build_rendered_message(std::mem::take(&mut pending)));
        }
        pending.push(rendered);
    }
    items.extend(build_rendered_message(pending));
    items
}

#[cfg(test)]
#[path = "updates_tests.rs"]
mod tests;
