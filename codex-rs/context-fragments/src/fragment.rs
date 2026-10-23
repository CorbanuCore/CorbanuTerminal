use codex_protocol::models::ContentItem;
use codex_protocol::models::InternalChatMessageMetadataPassthrough;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;

/// Type-erased registration for a contextual user fragment.
///
/// Implementations are used by context filtering code to recognize injected
/// fragments without constructing the concrete context payload.
pub trait FragmentRegistration: Sync {
    fn matches_text(&self, text: &str) -> bool;
}

pub struct FragmentRegistrationProxy<T> {
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T> FragmentRegistrationProxy<T> {
    pub const fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for FragmentRegistrationProxy<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ContextualUserFragment> FragmentRegistration for FragmentRegistrationProxy<T> {
    fn matches_text(&self, text: &str) -> bool {
        T::matches_text(text)
    }
}

/// Context payload that is injected as a message fragment.
///
/// Implementations own the response role and provide the exact fragment body.
/// Marked fragments also provide start/end markers used to recognize injected
/// context later. `render()` concatenates markers and body without adding
/// separators, so implementations should include any whitespace they need
/// between tags in `body()`. Unmarked fragments should leave both markers empty,
/// in which case the default helpers render only the body and never match
/// arbitrary text.
pub trait ContextualUserFragment {
    fn role(&self) -> &'static str;

    /// Whether this fragment must be recorded as its own response item.
    fn requires_separate_message(&self) -> bool {
        false
    }

    /// Stable producer ID recorded with the rendered fragment, such as a World State section ID.
    ///
    /// Provider adapters use it to tell apart distinct sources that share markers.
    fn source_id(&self) -> Option<&'static str> {
        None
    }

    fn markers(&self) -> (&'static str, &'static str);

    fn body(&self) -> String;

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

    fn render(&self) -> String {
        let (start_marker, end_marker) = self.markers();
        let body = self.body();
        if start_marker.is_empty() && end_marker.is_empty() {
            return body;
        }

        format!("{start_marker}{body}{end_marker}")
    }

    fn into(self) -> ResponseItem
    where
        Self: Sized,
    {
        context_message(self.role(), [(self.render(), self.source_id())])
    }

    fn into_boxed_response_item(self: Box<Self>) -> ResponseItem {
        context_message(self.role(), [(self.render(), self.source_id())])
    }

    fn into_response_input_item(self) -> ResponseInputItem
    where
        Self: Sized,
    {
        ResponseInputItem::Message {
            role: self.role().to_string(),
            content: vec![ContentItem::InputText {
                text: self.render(),
            }],
            phase: None,
        }
    }
}

/// Builds a context message from rendered sections and their producer IDs.
///
/// Producers are recorded, aligned with the content, for developer messages, the only role whose
/// contextual sections are deduplicated per producer before non-OpenAI requests.
pub fn context_message(
    role: &str,
    sections: impl IntoIterator<Item = (String, Option<&'static str>)>,
) -> ResponseItem {
    let (content, sources): (Vec<_>, Vec<_>) = sections
        .into_iter()
        .map(|(text, source_id)| {
            (
                ContentItem::InputText { text },
                source_id.map(str::to_string),
            )
        })
        .unzip();
    let internal_chat_message_metadata_passthrough = (role == "developer"
        && sources.iter().any(Option::is_some))
    .then(|| InternalChatMessageMetadataPassthrough {
        context_fragment_sources: Some(sources),
        ..Default::default()
    });
    ResponseItem::Message {
        id: None,
        role: role.to_string(),
        content,
        phase: None,
        internal_chat_message_metadata_passthrough,
    }
}

/// A fragment attributed to a stable producer ID, such as the World State section that rendered
/// it.
///
/// Producer IDs share one namespace across World State sections and extension fragments.
pub struct AttributedFragment<F: ?Sized> {
    source_id: &'static str,
    fragment: Box<F>,
}

impl<F: ?Sized> AttributedFragment<F> {
    pub fn new(source_id: &'static str, fragment: Box<F>) -> Self {
        Self {
            source_id,
            fragment,
        }
    }
}

impl<F: ContextualUserFragment + ?Sized> ContextualUserFragment for AttributedFragment<F> {
    fn role(&self) -> &'static str {
        self.fragment.role()
    }

    fn requires_separate_message(&self) -> bool {
        self.fragment.requires_separate_message()
    }

    fn source_id(&self) -> Option<&'static str> {
        Some(self.source_id)
    }

    fn markers(&self) -> (&'static str, &'static str) {
        self.fragment.markers()
    }

    fn body(&self) -> String {
        self.fragment.body()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("", "")
    }

    fn render(&self) -> String {
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
