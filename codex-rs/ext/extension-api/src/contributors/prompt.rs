//! Extension prompt placement with content and producer attribution kept together.

use codex_context_fragments::AnnotatedContent;
use codex_context_fragments::ContextualUserFragment;
use codex_context_fragments::RenderedFragment;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ContentItemKind;
use codex_protocol::models::ContentItemMetadata;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PromptSlot {
    DeveloperPolicy,
    DeveloperCapabilities,
    /// Content inside the context-window message, supplied by `contribute_thread_context`.
    ContextWindow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PromptFragment {
    slot: PromptSlot,
<<<<<<< HEAD
    text: String,
    source_id: Option<&'static str>,
    stored_data: bool,
||||||| 413492cd6c
    text: String,
=======
    content: AnnotatedContent,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

impl PromptFragment {
    /// Places a contextual fragment in the prompt, preserving its content, kind, and source.
    /// The slot determines placement within the developer message, regardless of its original role.
    pub fn from_fragment(slot: PromptSlot, fragment: &impl ContextualUserFragment) -> Self {
        Self {
            slot,
<<<<<<< HEAD
            text: text.into(),
            source_id: None,
            stored_data: false,
||||||| 413492cd6c
            text: text.into(),
=======
            content: fragment.render_fragment().into_parts().1,
        }
    }

    /// Creates harness-provided context for the given slot.
    pub fn new(slot: PromptSlot, text: impl Into<String>, content_kind: ContentItemKind) -> Self {
        Self {
            slot,
            content: AnnotatedContent::text(text, content_kind, ContentItemMetadata::harness()),
        }
    }

    /// Creates tool-supplied context without changing its prompt placement.
    pub fn tool(
        slot: PromptSlot,
        text: impl Into<String>,
        content_kind: ContentItemKind,
        namespace: String,
    ) -> Self {
        Self {
            slot,
            content: AnnotatedContent::text(
                text,
                content_kind,
                ContentItemMetadata::tool(Some(namespace.into())),
            ),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
    }

    /// Marks text derived from stored content the host did not write, such as
    /// memories summarised from earlier sessions. With `source_envelopes`, a
    /// protected request receives it as labelled untrusted data, not policy:
    /// Core then sends the fragment as its own developer message whatever its slot.
    pub fn with_stored_data(mut self) -> Self {
        self.stored_data = true;
        self
    }

    /// Attributes the fragment to a stable producer ID.
    ///
    /// Use this when other producers may emit context with the same markers, so the harness can
    /// keep one current copy per producer instead of one per marker. Producer IDs share one
    /// namespace with World State section IDs.
    pub fn with_source_id(mut self, source_id: &'static str) -> Self {
        self.source_id = Some(source_id);
        self
    }

    /// Creates a developer-policy prompt fragment.
    pub fn developer_policy(text: impl Into<String>, content_kind: ContentItemKind) -> Self {
        Self::new(PromptSlot::DeveloperPolicy, text, content_kind)
    }

    /// Creates a developer-capabilities prompt fragment.
    pub fn developer_capability(text: impl Into<String>, content_kind: ContentItemKind) -> Self {
        Self::new(PromptSlot::DeveloperCapabilities, text, content_kind)
    }

    /// Returns the target prompt slot.
    pub fn slot(&self) -> PromptSlot {
        self.slot
    }

    /// Returns the model-visible content.
    pub fn content(&self) -> &ContentItem {
        self.content.content()
    }

    /// Returns the producer-owned classification of the model-visible content.
    pub fn content_kind(&self) -> &ContentItemKind {
        self.content.kind()
    }

    /// Consumes the placement wrapper when assembling a shared context message.
    pub fn into_content(self) -> AnnotatedContent {
        self.content
    }
}

impl From<PromptFragment> for RenderedFragment {
    fn from(fragment: PromptFragment) -> Self {
        Self::new("developer", fragment.content)
    }

    /// Returns the stable producer ID, if the fragment is attributed.
    pub fn source_id(&self) -> Option<&'static str> {
        self.source_id
    }

    /// Whether the text is derived from stored content (see [`Self::with_stored_data`]).
    pub fn is_stored_data(&self) -> bool {
        self.stored_data
    }
}
