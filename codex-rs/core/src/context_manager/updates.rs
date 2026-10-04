use crate::context::ContextualUserFragment;
use codex_protocol::models::ContentItem;
use codex_protocol::models::InternalChatMessageMetadataPassthrough;
use codex_protocol::models::ResponseItem;

/// Model-visible context text and the stable producer it is attributed to, if any.
pub(crate) struct ContextSection {
    text: String,
    source_id: Option<&'static str>,
}

impl ContextSection {
    pub(crate) fn attributed(text: String, source_id: Option<&'static str>) -> Self {
        Self { text, source_id }
    }

    pub(crate) fn from_fragment(fragment: &dyn ContextualUserFragment) -> Self {
        Self::attributed(fragment.render(), fragment.source_id())
    }
}

impl From<String> for ContextSection {
    fn from(text: String) -> Self {
        Self::attributed(text, /*source_id*/ None)
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
        .into_iter()
        .filter_map(|(role, _, sections)| build_text_message(role, sections))
        .collect()
}

fn build_text_message(role: &str, sections: Vec<ContextSection>) -> Option<ResponseItem> {
    if sections.is_empty() {
        return None;
    }

    let (content, sources): (Vec<_>, Vec<_>) = sections
        .into_iter()
        .map(|section| {
            (
                ContentItem::InputText { text: section.text },
                section.source_id.map(str::to_string),
            )
        })
        .unzip();
    // Only developer fragments are deduplicated per source for non-OpenAI providers.
    let internal_chat_message_metadata_passthrough = (role == "developer"
        && sources.iter().any(Option::is_some))
    .then(|| InternalChatMessageMetadataPassthrough {
        context_fragment_sources: Some(sources),
        ..Default::default()
    });

    Some(ResponseItem::Message {
        id: None,
        role: role.to_string(),
        content,
        phase: None,
        internal_chat_message_metadata_passthrough,
    })
}
