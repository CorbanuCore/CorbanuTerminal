//! Thin PF-20 adapter. Native controller construction and containment remain
//! separate gates; accepting this trait object never authorizes a transition.
use std::sync::Arc;
use codex_protected_state::PolicyCheckpoint;
use codex_protected_state::PolicyRootStore;
use codex_protected_state::RootError;
use super::authoritative_state::AuthoritativeStateAnchor;
use super::authoritative_state::AuthoritativeStateAnchorError;
use super::authoritative_state::AuthoritativeStateAnchorStore;

#[derive(Debug)]
pub(crate) struct NativeAuthoritativeStateAnchor(pub(crate) Arc<dyn PolicyRootStore>);

impl From<&AuthoritativeStateAnchor> for PolicyCheckpoint {
    fn from(anchor: &AuthoritativeStateAnchor) -> Self {
        Self { schema_version: anchor.schema_version, revision: anchor.revision, owner: anchor.owner.clone(), state_sha256: anchor.state_sha256.clone(), commit_sha256: anchor.commit_sha256.clone() }
    }
}

impl From<PolicyCheckpoint> for AuthoritativeStateAnchor {
    fn from(anchor: PolicyCheckpoint) -> Self {
        Self { schema_version: anchor.schema_version, revision: anchor.revision, owner: anchor.owner, state_sha256: anchor.state_sha256, commit_sha256: anchor.commit_sha256 }
    }
}

fn error(error: RootError) -> AuthoritativeStateAnchorError {
    match error {
        RootError::Conflict => AuthoritativeStateAnchorError::Conflict,
        RootError::Invalid => AuthoritativeStateAnchorError::Invalid,
        RootError::MissingKey | RootError::Unavailable | RootError::Unsupported | RootError::Ambiguous => AuthoritativeStateAnchorError::Unavailable,
    }
}

impl AuthoritativeStateAnchorStore for NativeAuthoritativeStateAnchor {
    fn load_anchor(&self) -> Result<Option<AuthoritativeStateAnchor>, AuthoritativeStateAnchorError> {
        self.0.load_policy().map(|anchor| anchor.map(Into::into)).map_err(error)
    }

    fn compare_and_store_anchor(&self, expected: Option<&AuthoritativeStateAnchor>, next: &AuthoritativeStateAnchor) -> Result<(), AuthoritativeStateAnchorError> {
        self.0.compare_policy(expected.map(Into::into).as_ref(), &next.into()).map_err(error)
    }
}
