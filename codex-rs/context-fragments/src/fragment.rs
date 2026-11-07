use crate::AnnotatedContent;
use crate::message_from_parts;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ContentItemKind;
use codex_protocol::models::ContentItemMetadata;
use codex_protocol::models::ResponseItem;
use codex_protocol::models::input_text;

/// A rendered contextual fragment and the role that owns its annotated content.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderedFragment {
    role: &'static str,
    content: AnnotatedContent,
}

impl RenderedFragment {
    /// Creates a rendered fragment without separating its role and annotated content.
    pub fn new(role: &'static str, content: AnnotatedContent) -> Self {
        Self { role, content }
    }

    /// Returns the response role associated with this fragment.
    pub fn role(&self) -> &'static str {
        self.role
    }

    /// Returns this fragment's model-visible content and classification.
    pub fn annotated_content(&self) -> &AnnotatedContent {
        &self.content
    }

    /// Separates the role and annotated content at an API boundary.
    pub fn into_parts(self) -> (&'static str, AnnotatedContent) {
        (self.role, self.content)
    }
}

impl From<RenderedFragment> for ResponseItem {
    fn from(fragment: RenderedFragment) -> Self {
        let (role, annotated_content) = fragment.into_parts();
        message_from_parts(role, vec![annotated_content])
    }
}

/// Context payload that is injected as a message fragment.
///
/// Implementations own the response role and provide the exact fragment body.
/// Marked fragments also provide start/end markers used to recognize injected
/// context later. `render()` concatenates markers and `InputText` bodies without
/// adding separators, so implementations should include any whitespace they need
/// between tags in `body()`. All other content passes through unchanged. Unmarked
/// fragments should leave both markers empty and never match arbitrary text.
/// Producers must enforce size bounds for all model-visible content before rendering.
/// User-role non-text content is currently classified as ordinary user media because
/// contextual-user-message recognition relies on text markers.
pub trait ContextualUserFragment {
    fn role(&self) -> &'static str;

    /// Attribution is supplied when the fragment is produced, never inferred from rendered text.
    fn content_metadata(&self) -> ContentItemMetadata {
        ContentItemMetadata::harness()
    }

    /// Stable producer ID recorded with the rendered fragment, such as a World State section ID.
    ///
    /// Provider adapters use it to tell apart distinct sources that share markers.
    fn source_id(&self) -> Option<&'static str> {
        None
    }

    /// Binds producer attribution before this fragment is rendered or persisted.
    fn with_metadata(self, metadata: ContentItemMetadata) -> AttributedFragment<Self>
    where
        Self: Sized,
    {
        AttributedFragment {
            fragment: self,
            metadata,
        }
    }

    /// Returns a stable `<feature>.<name>` classification, using `generic` for shared fragments.
    fn content_kind(&self) -> ContentItemKind;

    fn markers(&self) -> (&'static str, &'static str);

    fn body(&self) -> ContentItem;

    fn type_markers() -> (&'static str, &'static str)
    where
        Self: Sized;

    fn matches_text(text: &str) -> bool
    where
        Self: Sized,
    {
        let (start_marker, end_marker) = Self::type_markers();
        matches_marked_text(start_marker, end_marker, text)
    }

    fn render(&self) -> ContentItem {
        let (start_marker, end_marker) = self.markers();
        let body = self.body();
        if start_marker.is_empty() && end_marker.is_empty() {
            return body;
        }

        match body {
            ContentItem::InputText { text } => {
                input_text(format!("{start_marker}{text}{end_marker}"))
            }
            content => content,
        }
    }

    /// Renders the role, model-visible content, and classification together.
    fn render_fragment(&self) -> RenderedFragment {
        RenderedFragment::new(
            self.role(),
            AnnotatedContent::from_parts(
                self.render(),
                self.content_kind(),
                self.content_metadata(),
            ),
        )
    }

    fn into(self) -> ResponseItem
    where
        Self: Sized,
    {
        let source_id = self.source_id();
        attach_context_source(ResponseItem::from(self.render_fragment()), source_id)
    }

    fn into_boxed_response_item(self: Box<Self>) -> ResponseItem {
        let source_id = self.source_id();
        attach_context_source(ResponseItem::from(self.render_fragment()), source_id)
    }
}

/// A contextual fragment with attribution supplied by its producer.
pub struct AttributedFragment<F> {
    fragment: F,
    metadata: ContentItemMetadata,
}

impl<F: ContextualUserFragment> ContextualUserFragment for AttributedFragment<F> {
    fn role(&self) -> &'static str {
        self.fragment.role()
    }
    fn content_kind(&self) -> ContentItemKind {
        self.fragment.content_kind()
    }
    fn markers(&self) -> (&'static str, &'static str) {
        self.fragment.markers()
    }
    fn type_markers() -> (&'static str, &'static str) {
        F::type_markers()
    }
    fn body(&self) -> ContentItem {
        self.fragment.body()
    }
    fn render(&self) -> ContentItem {
        self.fragment.render()
    }
    fn content_metadata(&self) -> ContentItemMetadata {
        self.metadata.clone()
    }
    fn source_id(&self) -> Option<&'static str> {
        self.fragment.source_id()
    }
}

/// Builds a context message from rendered sections and their producer IDs.
///
/// Producers are recorded, aligned with the content, for developer messages, the only role whose
/// contextual sections are deduplicated per producer before non-OpenAI requests.
pub fn context_message(
    role: &'static str,
    sections: impl IntoIterator<Item = (AnnotatedContent, Option<&'static str>)>,
) -> ResponseItem {
    let (parts, sources): (Vec<_>, Vec<_>) = sections.into_iter().unzip();
    with_context_sources(message_from_parts(role, parts), sources)
}

fn attach_context_source(item: ResponseItem, source_id: Option<&'static str>) -> ResponseItem {
    with_context_sources(item, vec![source_id])
}

fn with_context_sources(
    mut item: ResponseItem,
    sources: Vec<Option<&'static str>>,
) -> ResponseItem {
    if let ResponseItem::Message {
        role,
        internal_chat_message_metadata_passthrough,
        ..
    } = &mut item
        && role == "developer"
        && sources.iter().any(Option::is_some)
    {
        internal_chat_message_metadata_passthrough
            .get_or_insert_with(Default::default)
            .context_fragment_sources = Some(
            sources
                .into_iter()
                .map(|source| source.map(str::to_string))
                .collect(),
        );
    }
    item
}

/// A fragment attributed to a stable producer ID, such as the World State section that rendered
/// it.
///
/// Producer IDs share one namespace across World State sections and extension fragments.
pub struct SourcedFragment<F: ?Sized> {
    source_id: &'static str,
    fragment: Box<F>,
}

impl<F: ?Sized> SourcedFragment<F> {
    pub fn new(source_id: &'static str, fragment: Box<F>) -> Self {
        Self {
            source_id,
            fragment,
        }
    }
}

impl<F: ContextualUserFragment + ?Sized> ContextualUserFragment for SourcedFragment<F> {
    fn role(&self) -> &'static str {
        self.fragment.role()
    }

    fn content_metadata(&self) -> ContentItemMetadata {
        self.fragment.content_metadata()
    }

    fn content_kind(&self) -> ContentItemKind {
        self.fragment.content_kind()
    }

    fn source_id(&self) -> Option<&'static str> {
        Some(self.source_id)
    }

    fn markers(&self) -> (&'static str, &'static str) {
        self.fragment.markers()
    }

    fn body(&self) -> ContentItem {
        self.fragment.body()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("", "")
    }

    fn render(&self) -> ContentItem {
        self.fragment.render()
    }
}

pub(crate) fn matches_marked_text(start_marker: &str, end_marker: &str, text: &str) -> bool {
    if start_marker.is_empty() || end_marker.is_empty() {
        return false;
    }

    let trimmed = text.trim_start();
    let starts_with_marker = trimmed
        .get(..start_marker.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(start_marker));
    let trimmed = trimmed.trim_end();
    let ends_with_marker = trimmed
        .get(trimmed.len().saturating_sub(end_marker.len())..)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(end_marker));
    starts_with_marker && ends_with_marker
}
