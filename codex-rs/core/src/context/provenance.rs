//! Provider-independent data fragment. Construction requires Core admission;
//! external content cannot choose a role or supply its own wrapper metadata.

use super::ContextualUserFragment;
use crate::security::ingress::AdmittedSource;

#[derive(Clone)]
pub(crate) struct ProvenanceContext {
    rendered: String,
}

impl std::fmt::Debug for ProvenanceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvenanceContext").field("bytes", &self.rendered.len()).finish()
    }
}

impl ProvenanceContext {
    pub(crate) fn from_admitted(source: AdmittedSource) -> Self {
        Self { rendered: source.into_projection() }
    }
}

impl ContextualUserFragment for ProvenanceContext {
    fn role(&self) -> &'static str { "user" }

    fn markers(&self) -> (&'static str, &'static str) { Self::type_markers() }

    fn type_markers() -> (&'static str, &'static str) {
        ("<corbanu_untrusted_data>", "</corbanu_untrusted_data>")
    }

    fn body(&self) -> String { self.rendered.clone() }
}
