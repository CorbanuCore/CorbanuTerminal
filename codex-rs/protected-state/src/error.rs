use codex_security_audit::IntegrityRootError;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RootError {
    #[error("controller root is unavailable")]
    Unavailable,
    #[error("controller integrity key is missing")]
    MissingKey,
    #[error("controller root is invalid")]
    Invalid,
    #[error("controller root changed concurrently")]
    Conflict,
    #[error("controller commit outcome is unknown; reconcile before reuse")]
    Ambiguous,
    #[error("native controller operation is unsupported")]
    Unsupported,
}

impl From<RootError> for IntegrityRootError {
    fn from(error: RootError) -> Self {
        match error {
            RootError::MissingKey => Self::MissingKey,
            RootError::Invalid => Self::Invalid,
            RootError::Conflict => Self::Conflict,
            // PF-41's existing ambiguity representation. This includes lost
            // acknowledgement and post-publication sync failure, not just time.
            RootError::Ambiguous => Self::Timeout,
            RootError::Unavailable | RootError::Unsupported => Self::Unavailable,
        }
    }
}
