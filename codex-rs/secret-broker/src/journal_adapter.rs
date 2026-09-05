//! PF-41 durable intent adapter. This is audit correlation, never authorization.
//!
//! The trusted service supplies a recovered journal and already-authorized
//! request. Its protected root and live PF-16–20 checks remain mandatory. Exact
//! operation/path semantics are digest-bound, not copied into shared audit data.

use crate::BrokerAuditError;
use crate::BrokerAuditIntent;
use crate::BrokerAuditResolution;
use crate::BrokerBinding;
use crate::CredentialReference;
use crate::DurableBrokerAudit;
use crate::OpenAiResponsesOperation;
use codex_security_audit::AuthorityIdentity;
use codex_security_audit::DispatchPermit;
use codex_security_audit::DispatchResolution;
use codex_security_audit::EventContext;
use codex_security_audit::JournalError;
use codex_security_audit::ReferenceJournal;
use codex_security_audit::UnknownOutcomeReason;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::MandateOutcome;
use sha2::Digest;
use sha2::Sha256;
use std::sync::Mutex;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Immutable correlation supplied by the authenticated controller. Constructing
/// it does not mint a grant; the broker backend still checks live authority.
pub struct BrokerJournalBinding {
    pub binding: BrokerBinding,
    pub credential: CredentialReference,
    pub request: AuthorizationRequest,
    pub authority: AuthorityIdentity,
    pub operation: OpenAiResponsesOperation,
}

/// Clock failures deny audit and therefore dispatch; tests inject fixed time.
pub trait BrokerJournalClock: Send + Sync + 'static {
    fn now_unix_seconds(&self) -> Result<i64, BrokerAuditError>;
}

pub struct SystemBrokerJournalClock;

impl BrokerJournalClock for SystemBrokerJournalClock {
    fn now_unix_seconds(&self) -> Result<i64, BrokerAuditError> {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BrokerAuditError::Unavailable)?
            .as_secs();
        i64::try_from(seconds).map_err(|_| BrokerAuditError::Unavailable)
    }
}

/// One controller/run/credential/operation-bound journal producer. There is no
/// filesystem fallback or automatic recovery from an ambiguous prior dispatch.
pub struct JournalBrokerAudit<C = SystemBrokerJournalClock> {
    journal: Mutex<ReferenceJournal>,
    binding: BrokerJournalBinding,
    context: EventContext,
    clock: C,
}

impl<C: BrokerJournalClock> JournalBrokerAudit<C> {
    pub fn new(
        journal: ReferenceJournal,
        binding: BrokerJournalBinding,
        context: EventContext,
        clock: C,
    ) -> Result<Self, BrokerAuditError> {
        binding.binding.validate().map_err(unavailable)?;
        binding.request.validate().map_err(unavailable)?;
        if context.run_generation != binding.binding.run_generation
            || binding.request.context.session_id.as_str() != binding.binding.session_id
            || binding.request.context.task_id.as_str() != binding.binding.task_id
        {
            return Err(BrokerAuditError::Unavailable);
        }
        Ok(Self {
            journal: Mutex::new(journal),
            binding,
            context,
            clock,
        })
    }

    fn matches(&self, intent: &BrokerAuditIntent) -> bool {
        let bound = &self.binding;
        intent.controller_instance == bound.binding.controller_instance
            && intent.session_id == bound.binding.session_id
            && intent.task_id == bound.binding.task_id
            && intent.run_id == bound.binding.run_id
            && intent.run_generation == bound.binding.run_generation
            && intent.sequence > 0
            && intent.credential_reference == bound.credential
            && intent.operation == "openai.responses.create"
            && intent.destination == "https://api.openai.com:443"
            && intent.path == bound.operation.path()
    }
}

impl<C: BrokerJournalClock> DurableBrokerAudit for JournalBrokerAudit<C> {
    type Permit = DispatchPermit;

    fn reserve(&self, intent: &BrokerAuditIntent) -> Result<Self::Permit, BrokerAuditError> {
        if !self.matches(intent) {
            return Err(BrokerAuditError::Unavailable);
        }
        let now = self.clock.now_unix_seconds()?;
        // This projection is used only for audit identity. It binds the entire
        // original authorization plus exact transport semantics without adding
        // raw paths/destinations to PF-41's deliberately minimized schema.
        let semantics = digest(&(
            "broker-audit-semantics-v1",
            &self.binding.request,
            &self.binding.credential,
            &self.binding.operation,
        ))?;
        let mut request = self.binding.request.clone();
        request.context.operation = semantics;
        let deduplication = digest(&(
            "broker-audit-dispatch-v1",
            &self.binding.binding,
            intent.sequence,
        ))?;
        self.journal
            .lock()
            .map_err(unavailable)?
            .reserve_dispatch(
                self.context.clone(),
                None,
                &request,
                self.binding.authority.clone(),
                deduplication,
                now,
            )
            .map(|(permit, _)| permit)
            .map_err(map_journal_error)
    }

    fn resolve(
        &self,
        permit: Self::Permit,
        resolution: BrokerAuditResolution,
    ) -> Result<(), BrokerAuditError> {
        let resolution = match resolution {
            BrokerAuditResolution::Completed => completed(MandateOutcome::Executed),
            BrokerAuditResolution::Failed => completed(MandateOutcome::Failed),
            BrokerAuditResolution::Cancelled => completed(MandateOutcome::Cancelled),
            BrokerAuditResolution::Unknown => DispatchResolution::Unknown {
                reason: UnknownOutcomeReason::TransportLost,
            },
        };
        self.journal
            .lock()
            .map_err(unavailable)?
            .resolve_dispatch(
                &permit,
                self.context.clone(),
                resolution,
                self.clock.now_unix_seconds()?,
            )
            .map(|_| ())
            .map_err(map_journal_error)
    }
}

fn completed(outcome: MandateOutcome) -> DispatchResolution {
    DispatchResolution::Completed {
        outcome,
        mandate_receipt: None,
    }
}

fn digest(value: &impl serde::Serialize) -> Result<BoundedText, BrokerAuditError> {
    let bytes = serde_json::to_vec(value).map_err(unavailable)?;
    BoundedText::new(format!("{:x}", Sha256::digest(bytes))).map_err(unavailable)
}

fn unavailable<T>(_: T) -> BrokerAuditError {
    BrokerAuditError::Unavailable
}

fn map_journal_error(error: JournalError) -> BrokerAuditError {
    match error {
        JournalError::CommitUnknown { .. } => BrokerAuditError::CommitUnknown,
        _ => BrokerAuditError::Unavailable,
    }
}

#[cfg(test)]
#[path = "journal_adapter_tests.rs"]
mod tests;
