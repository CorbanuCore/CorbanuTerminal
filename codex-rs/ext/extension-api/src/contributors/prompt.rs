// All this file should be replaced by the existing fragment implementation ofc

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PromptSlot {
    DeveloperPolicy,
    DeveloperCapabilities,
    ContextualUser,
    SeparateDeveloper,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptFragment {
    slot: PromptSlot,
    text: String,
    source_id: Option<&'static str>,
    stored_data: bool,
}

impl PromptFragment {
    /// Creates a prompt fragment for the given slot.
    pub fn new(slot: PromptSlot, text: impl Into<String>) -> Self {
        Self {
            slot,
            text: text.into(),
            source_id: None,
            stored_data: false,
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
    pub fn developer_policy(text: impl Into<String>) -> Self {
        Self::new(PromptSlot::DeveloperPolicy, text)
    }

    /// Creates a developer-capabilities prompt fragment.
    pub fn developer_capability(text: impl Into<String>) -> Self {
        Self::new(PromptSlot::DeveloperCapabilities, text)
    }

    /// Creates a separate top-level developer prompt fragment.
    pub fn separate_developer(text: impl Into<String>) -> Self {
        Self::new(PromptSlot::SeparateDeveloper, text)
    }

    /// Returns the target prompt slot.
    pub fn slot(&self) -> PromptSlot {
        self.slot
    }

    /// Returns the model-visible text.
    pub fn text(&self) -> &str {
        &self.text
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
