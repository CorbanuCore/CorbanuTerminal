use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ActorChain;
use crate::AuthorizationRequest;
use crate::BoundedText;
use crate::PolicyPrincipal;
use crate::PrincipalKind;
use crate::digest::canonical_sha256;

pub const MANDATE_SCHEMA_VERSION: u32 = 1;
pub const REPLAY_LEDGER_SCHEMA_VERSION: u32 = 1;

/// Exact, user-visible protected action that is safe to hash and audit.
///
/// It carries resource identifiers and limits, never raw credentials, private
/// financial data, arbitrary tool payloads, or model output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtectedActionPreview {
    pub schema_version: u32,
    pub request: AuthorizationRequest,
    pub expires_at_unix_seconds: i64,
    pub nonce: BoundedText,
}

impl ProtectedActionPreview {
    pub fn new(
        request: AuthorizationRequest,
        expires_at_unix_seconds: i64,
        nonce: BoundedText,
    ) -> Result<Self, MandateError> {
        let preview = Self {
            schema_version: MANDATE_SCHEMA_VERSION,
            request,
            expires_at_unix_seconds,
            nonce,
        };
        preview.validate()?;
        Ok(preview)
    }

    pub fn validate(&self) -> Result<(), MandateError> {
        if self.schema_version != MANDATE_SCHEMA_VERSION {
            return Err(MandateError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: MANDATE_SCHEMA_VERSION,
            });
        }
        self.request
            .validate()
            .map_err(MandateError::Authorization)?;
        if self.expires_at_unix_seconds <= self.request.context.now_unix_seconds {
            return Err(MandateError::InvalidExpiry);
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<String, MandateError> {
        self.validate()?;
        canonical_sha256(self).map_err(MandateError::Serialization)
    }
}

/// Human approval bound to one exact protected-action preview.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProtectedActionMandate {
    pub schema_version: u32,
    pub mandate_id: BoundedText,
    pub preview_digest: BoundedText,
    pub approver: PolicyPrincipal,
    pub actor_chain: ActorChain,
    pub approved_at_unix_seconds: i64,
    pub expires_at_unix_seconds: i64,
}

#[derive(Serialize)]
struct MandateBinding<'a> {
    schema_version: u32,
    preview_digest: &'a BoundedText,
    approver: &'a PolicyPrincipal,
    actor_chain: &'a ActorChain,
    approved_at_unix_seconds: i64,
    expires_at_unix_seconds: i64,
}

impl ProtectedActionMandate {
    pub fn approve(
        preview: &ProtectedActionPreview,
        approver: PolicyPrincipal,
        approved_at_unix_seconds: i64,
    ) -> Result<Self, MandateError> {
        preview.validate()?;
        if approver.kind != PrincipalKind::Human {
            return Err(MandateError::ApproverMustBeHuman);
        }
        if preview.request.subject.as_slice().first() != Some(&approver) {
            return Err(MandateError::ApproverDoesNotOwnActorChain);
        }
        if approved_at_unix_seconds < preview.request.context.now_unix_seconds
            || approved_at_unix_seconds >= preview.expires_at_unix_seconds
        {
            return Err(MandateError::ApprovalOutsideValidityWindow);
        }
        let preview_digest = BoundedText::new(preview.digest()?)?;
        let mut mandate = Self {
            schema_version: MANDATE_SCHEMA_VERSION,
            mandate_id: BoundedText::new("pending")?,
            preview_digest,
            approver,
            actor_chain: preview.request.subject.clone(),
            approved_at_unix_seconds,
            expires_at_unix_seconds: preview.expires_at_unix_seconds,
        };
        mandate.mandate_id = BoundedText::new(mandate.expected_id()?)?;
        Ok(mandate)
    }

    pub fn validate(&self) -> Result<(), MandateError> {
        if self.schema_version != MANDATE_SCHEMA_VERSION {
            return Err(MandateError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: MANDATE_SCHEMA_VERSION,
            });
        }
        if self.approver.kind != PrincipalKind::Human {
            return Err(MandateError::ApproverMustBeHuman);
        }
        if self.actor_chain.as_slice().first() != Some(&self.approver) {
            return Err(MandateError::ApproverDoesNotOwnActorChain);
        }
        if self.approved_at_unix_seconds < 0
            || self.expires_at_unix_seconds <= self.approved_at_unix_seconds
        {
            return Err(MandateError::ApprovalOutsideValidityWindow);
        }
        if self.mandate_id.as_str() != self.expected_id()? {
            return Err(MandateError::IntegrityMismatch);
        }
        Ok(())
    }

    pub fn matches_preview(
        &self,
        preview: &ProtectedActionPreview,
        now_unix_seconds: i64,
    ) -> Result<bool, MandateError> {
        self.validate()?;
        preview.validate()?;
        if now_unix_seconds < self.approved_at_unix_seconds
            || now_unix_seconds >= self.expires_at_unix_seconds
        {
            return Ok(false);
        }
        if preview.request.subject != self.actor_chain {
            return Ok(false);
        }
        Ok(self.preview_digest.as_str() == preview.digest()?)
    }

    fn expected_id(&self) -> Result<String, MandateError> {
        canonical_sha256(&MandateBinding {
            schema_version: self.schema_version,
            preview_digest: &self.preview_digest,
            approver: &self.approver,
            actor_chain: &self.actor_chain,
            approved_at_unix_seconds: self.approved_at_unix_seconds,
            expires_at_unix_seconds: self.expires_at_unix_seconds,
        })
        .map_err(MandateError::Serialization)
    }
}

/// Durable replay guard for one-time mandates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayLedger {
    pub schema_version: u32,
    consumed_mandate_ids: BTreeSet<BoundedText>,
}

impl ReplayLedger {
    pub fn new() -> Self {
        Self {
            schema_version: REPLAY_LEDGER_SCHEMA_VERSION,
            consumed_mandate_ids: BTreeSet::new(),
        }
    }

    pub fn consume(
        &mut self,
        mandate: &ProtectedActionMandate,
        preview: &ProtectedActionPreview,
        now_unix_seconds: i64,
    ) -> Result<(), MandateError> {
        self.validate()?;
        if !mandate.matches_preview(preview, now_unix_seconds)? {
            return Err(MandateError::PreviewMismatchOrExpired);
        }
        if !self.consumed_mandate_ids.insert(mandate.mandate_id.clone()) {
            return Err(MandateError::Replay);
        }
        Ok(())
    }

    pub fn contains(&self, mandate_id: &BoundedText) -> bool {
        self.consumed_mandate_ids.contains(mandate_id)
    }

    pub fn validate(&self) -> Result<(), MandateError> {
        if self.schema_version != REPLAY_LEDGER_SCHEMA_VERSION {
            return Err(MandateError::UnsupportedReplayLedgerVersion {
                found: self.schema_version,
                supported: REPLAY_LEDGER_SCHEMA_VERSION,
            });
        }
        Ok(())
    }
}

impl Default for ReplayLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MandateOutcome {
    Executed,
    Denied,
    Cancelled,
    Failed,
}

/// Secret-free result of a protected action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActionReceipt {
    pub schema_version: u32,
    pub receipt_id: BoundedText,
    pub mandate_id: BoundedText,
    pub preview_digest: BoundedText,
    pub outcome: MandateOutcome,
    pub completed_at_unix_seconds: i64,
}

#[derive(Serialize)]
struct ReceiptBinding<'a> {
    schema_version: u32,
    mandate_id: &'a BoundedText,
    preview_digest: &'a BoundedText,
    outcome: MandateOutcome,
    completed_at_unix_seconds: i64,
}

impl ActionReceipt {
    pub fn complete(
        mandate: &ProtectedActionMandate,
        preview: &ProtectedActionPreview,
        outcome: MandateOutcome,
        completed_at_unix_seconds: i64,
    ) -> Result<Self, MandateError> {
        if !mandate.matches_preview(preview, completed_at_unix_seconds)? {
            return Err(MandateError::PreviewMismatchOrExpired);
        }
        let mut receipt = Self {
            schema_version: MANDATE_SCHEMA_VERSION,
            receipt_id: BoundedText::new("pending")?,
            mandate_id: mandate.mandate_id.clone(),
            preview_digest: mandate.preview_digest.clone(),
            outcome,
            completed_at_unix_seconds,
        };
        receipt.receipt_id = BoundedText::new(receipt.expected_id()?)?;
        Ok(receipt)
    }

    pub fn validate(&self) -> Result<(), MandateError> {
        if self.schema_version != MANDATE_SCHEMA_VERSION {
            return Err(MandateError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: MANDATE_SCHEMA_VERSION,
            });
        }
        if self.completed_at_unix_seconds < 0 {
            return Err(MandateError::NegativeTimestamp);
        }
        if self.receipt_id.as_str() != self.expected_id()? {
            return Err(MandateError::ReceiptIntegrityMismatch);
        }
        Ok(())
    }

    fn expected_id(&self) -> Result<String, MandateError> {
        canonical_sha256(&ReceiptBinding {
            schema_version: self.schema_version,
            mandate_id: &self.mandate_id,
            preview_digest: &self.preview_digest,
            outcome: self.outcome,
            completed_at_unix_seconds: self.completed_at_unix_seconds,
        })
        .map_err(MandateError::Serialization)
    }
}

#[derive(Debug, Error)]
pub enum MandateError {
    #[error("mandate approver must be a human")]
    ApproverMustBeHuman,
    #[error("mandate approver must own the request actor chain")]
    ApproverDoesNotOwnActorChain,
    #[error("protected-action preview must expire after its request time")]
    InvalidExpiry,
    #[error("approval is outside the preview validity window")]
    ApprovalOutsideValidityWindow,
    #[error("mandate or preview is expired or does not match")]
    PreviewMismatchOrExpired,
    #[error("mandate has already been consumed")]
    Replay,
    #[error("mandate integrity digest does not match its bound fields")]
    IntegrityMismatch,
    #[error("receipt integrity digest does not match its bound fields")]
    ReceiptIntegrityMismatch,
    #[error("timestamps must be non-negative")]
    NegativeTimestamp,
    #[error("unsupported mandate schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("unsupported replay ledger version {found}; supported version is {supported}")]
    UnsupportedReplayLedgerVersion { found: u32, supported: u32 },
    #[error(transparent)]
    BoundedText(#[from] crate::BoundedTextError),
    #[error(transparent)]
    Authorization(#[from] crate::AuthorizationError),
    #[error("failed to serialize canonical mandate object: {0}")]
    Serialization(serde_json::Error),
}
