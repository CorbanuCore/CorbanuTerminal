#[cfg(test)]
use std::cell::Cell;
use std::sync::Arc;

use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_utils_absolute_path::AbsolutePathBuf;

use crate::ActionId;
use crate::AuthorityIdentity;
use crate::DispatchResolution;
use crate::EventContext;
use crate::ReservationId;
use crate::SecurityEvent;
use crate::SecurityEventId;
use crate::SecurityEventKind;
#[cfg(test)]
pub(crate) use crate::journal_faults::FaultPoint;
#[cfg(test)]
pub(crate) use crate::journal_faults::InjectedFault;
use crate::journal_support::validate_resolution;
use crate::journal_types::AppendAcknowledgement;
use crate::journal_types::DispatchPermit;
use crate::journal_types::EventChainError;
use crate::journal_types::INTEGRITY_CHECKPOINT_SCHEMA_VERSION;
use crate::journal_types::IntegrityCheckpoint;
use crate::journal_types::IntegrityRootError;
use crate::journal_types::IntegrityRootStore;
use crate::journal_types::JournalConfig;
use crate::journal_types::JournalError;
use crate::journal_types::JournalOwner;
use crate::storage::EventChainState;
use crate::storage::JournalRecord;
use crate::storage::map_blocker;

pub struct ReferenceJournal {
    pub(crate) root: AbsolutePathBuf,
    pub(crate) owner: JournalOwner,
    pub(crate) root_store: Arc<dyn IntegrityRootStore>,
    pub(crate) config: JournalConfig,
    pub(crate) blocked: bool,
    pub(crate) reconciliation_required: bool,
    pub(crate) minimum_policy_generation: u64,
    pub(crate) minimum_run_generation: u64,
    pub(crate) validated: Option<ValidatedJournalState>,
    #[cfg(test)]
    pub(crate) fault: Option<(FaultPoint, InjectedFault)>,
    #[cfg(test)]
    pub(crate) scan_count: Cell<usize>,
}

impl std::fmt::Debug for ReferenceJournal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReferenceJournal")
            .field("owner", &self.owner)
            .field("config", &self.config)
            .field("blocked", &self.blocked)
            .field("reconciliation_required", &self.reconciliation_required)
            .field("minimum_policy_generation", &self.minimum_policy_generation)
            .field("minimum_run_generation", &self.minimum_run_generation)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedJournalState {
    pub(crate) event_count: usize,
    pub(crate) tail_record_sha256: String,
    pub(crate) checkpoint: Option<IntegrityCheckpoint>,
    pub(crate) chain: EventChainState,
}

#[derive(Clone, Debug)]
pub(crate) struct DuplicateDispatch {
    action_id: ActionId,
    reservation_id: ReservationId,
    resolved: bool,
}

impl ReferenceJournal {
    pub fn new(
        root: AbsolutePathBuf,
        owner: JournalOwner,
        root_store: Arc<dyn IntegrityRootStore>,
        config: JournalConfig,
    ) -> Self {
        Self {
            root,
            owner,
            root_store,
            config,
            blocked: true,
            reconciliation_required: false,
            minimum_policy_generation: 0,
            minimum_run_generation: 0,
            validated: None,
            #[cfg(test)]
            fault: None,
            #[cfg(test)]
            scan_count: Cell::new(0),
        }
    }

    pub(crate) fn mark_blocked(&mut self) {
        self.blocked = true;
        self.validated = None;
    }

    pub fn record_decision(
        &mut self,
        event: SecurityEvent,
    ) -> Result<AppendAcknowledgement, JournalError> {
        if !matches!(event.kind, SecurityEventKind::Decision { .. }) {
            return Err(JournalError::WrongEventKind);
        }
        self.append(event)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub fn reserve_dispatch(
        &mut self,
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        request: &AuthorizationRequest,
        authority: AuthorityIdentity,
        deduplication_key: BoundedText,
        occurred_at_unix_seconds: i64,
    ) -> Result<(DispatchPermit, AppendAcknowledgement), JournalError> {
        if self.reconciliation_required {
            return Err(JournalError::ReconciliationRequired);
        }
        let event = SecurityEvent::dispatch_intent(
            context.clone(),
            causal_parent,
            request,
            authority.clone(),
            deduplication_key,
            occurred_at_unix_seconds,
        )?;
        let SecurityEventKind::DispatchIntent {
            action_id,
            reservation_id,
            ..
        } = &event.kind
        else {
            return Err(JournalError::WrongEventKind);
        };
        let permit = DispatchPermit {
            context,
            intent_event_id: event.event_id.clone(),
            action_id: action_id.clone(),
            reservation_id: reservation_id.clone(),
            authority,
        };
        let (acknowledgement, duplicate) = self.append(event)?;
        if let Some(duplicate) = duplicate {
            return Err(if duplicate.resolved {
                JournalError::AlreadyResolved {
                    event_id: acknowledgement.event_id,
                    action_id: duplicate.action_id,
                    reservation_id: duplicate.reservation_id,
                    sequence: acknowledgement.sequence,
                }
            } else {
                JournalError::AlreadyReserved {
                    event_id: acknowledgement.event_id,
                    action_id: duplicate.action_id,
                    reservation_id: duplicate.reservation_id,
                    sequence: acknowledgement.sequence,
                }
            });
        }
        Ok((permit, acknowledgement))
    }

    pub fn resolve_dispatch(
        &mut self,
        permit: DispatchPermit,
        current_context: EventContext,
        resolution: DispatchResolution,
        occurred_at_unix_seconds: i64,
    ) -> Result<AppendAcknowledgement, JournalError> {
        if current_context.producer != permit.context.producer
            || current_context.policy_generation < permit.context.policy_generation
            || current_context.run_generation < permit.context.run_generation
        {
            return Err(JournalError::InvalidResolution);
        }
        validate_resolution(&permit.authority, &resolution)?;
        let event = SecurityEvent::dispatch_resolution(
            current_context,
            permit.intent_event_id,
            permit.action_id,
            permit.reservation_id,
            resolution,
            occurred_at_unix_seconds,
        )?;
        self.append(event)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub fn record_restriction(
        &mut self,
        context: EventContext,
        causal_parent: Option<SecurityEventId>,
        event: codex_security_policy::RevocationEvent,
    ) -> Result<AppendAcknowledgement, JournalError> {
        self.append(SecurityEvent::restriction(context, causal_parent, event)?)
            .map(|(acknowledgement, _)| acknowledgement)
    }

    pub(crate) fn append(
        &mut self,
        event: SecurityEvent,
    ) -> Result<(AppendAcknowledgement, Option<DuplicateDispatch>), JournalError> {
        if self.blocked {
            return Err(JournalError::RecoveryRequired);
        }
        event.validate()?;
        if event.context.producer != self.owner.producer {
            return Err(JournalError::ProducerMismatch);
        }
        if event.context.policy_generation < self.minimum_policy_generation {
            return Err(EventChainError::PolicyGenerationRegression.into());
        }
        if event.context.run_generation < self.minimum_run_generation {
            return Err(EventChainError::RunGenerationRegression.into());
        }
        let _lock = self.writer_lock()?;
        let validated = self
            .validated
            .clone()
            .ok_or(JournalError::RecoveryRequired)?;
        let checkpoint = self.load_checkpoint().map_err(|blocker| {
            self.mark_blocked();
            map_blocker(blocker)
        })?;
        if checkpoint != validated.checkpoint {
            self.mark_blocked();
            return Err(JournalError::RecoveryRequired);
        }

        if let Some(sequence) = validated.chain.event_sequence(&event.event_id) {
            let checkpoint = checkpoint.ok_or(JournalError::RecoveryRequired)?;
            let duplicate = match &event.kind {
                SecurityEventKind::DispatchIntent {
                    action_id,
                    reservation_id,
                    ..
                } => Some(DuplicateDispatch {
                    action_id: action_id.clone(),
                    reservation_id: reservation_id.clone(),
                    resolved: validated.chain.reservation_is_resolved(reservation_id),
                }),
                _ => None,
            };
            return Ok((
                AppendAcknowledgement {
                    event_id: event.event_id,
                    sequence,
                    checkpoint,
                    duplicate: true,
                },
                duplicate,
            ));
        }
        if let SecurityEventKind::DispatchIntent {
            action_id,
            deduplication_digest,
            ..
        } = &event.kind
            && let Some((existing_event_id, existing_action_id, reservation_id, sequence, resolved)) =
                validated
                    .chain
                    .matching_dispatch(action_id, deduplication_digest)
        {
            let checkpoint = checkpoint.ok_or(JournalError::RecoveryRequired)?;
            return Ok((
                AppendAcknowledgement {
                    event_id: existing_event_id,
                    sequence,
                    checkpoint,
                    duplicate: true,
                },
                Some(DuplicateDispatch {
                    action_id: existing_action_id,
                    reservation_id,
                    resolved,
                }),
            ));
        }
        if matches!(event.kind, SecurityEventKind::DispatchIntent { .. })
            && !validated.chain.pending_dispatches().is_empty()
        {
            return Err(JournalError::ReconciliationRequired);
        }
        if validated.event_count >= self.config.max_records {
            self.mark_blocked();
            return Err(JournalError::CapacityExceeded);
        }

        let sequence = u64::try_from(validated.event_count)
            .map_err(|_| JournalError::CapacityExceeded)?
            .checked_add(1)
            .ok_or(JournalError::CapacityExceeded)?;
        let next_event_count = validated
            .event_count
            .checked_add(1)
            .ok_or(JournalError::CapacityExceeded)?;
        let previous = validated.tail_record_sha256;
        let record = JournalRecord::new(sequence, previous, event)?;
        let mut candidate_chain = validated.chain;
        candidate_chain
            .apply(&record.event, sequence, &self.owner)
            .map_err(JournalError::from)?;

        #[cfg(test)]
        self.maybe_fault(FaultPoint::BeforeRecordWrite, &record.event.event_id)?;
        self.write_record(&record)?;
        #[cfg(test)]
        self.maybe_fault(FaultPoint::AfterRecordSync, &record.event.event_id)?;

        self.publish_record(&record)?;
        #[cfg(test)]
        self.maybe_fault(FaultPoint::AfterRecordRename, &record.event.event_id)?;

        let next_checkpoint = IntegrityCheckpoint {
            schema_version: INTEGRITY_CHECKPOINT_SCHEMA_VERSION,
            sequence,
            record_sha256: record.record_sha256,
            producer: self.owner.producer.clone(),
            owner_generation: self.owner.owner_generation,
            integrity_key_id: self.owner.integrity_key_id.clone(),
            policy_generation: record.event.context.policy_generation,
            run_generation: record.event.context.run_generation,
        };
        if let Err(error) = self
            .root_store
            .compare_and_store(checkpoint.as_ref(), &next_checkpoint)
        {
            self.mark_blocked();
            return Err(match error {
                IntegrityRootError::Timeout => JournalError::CommitUnknown {
                    event_id: record.event.event_id.clone(),
                },
                IntegrityRootError::MissingKey | IntegrityRootError::Unavailable => {
                    JournalError::IntegrityRootUnavailable
                }
                IntegrityRootError::Conflict => JournalError::IntegrityRootConflict,
                IntegrityRootError::Invalid => JournalError::IntegrityRootInvalid,
            });
        }
        if self.reconciliation_required && candidate_chain.pending_dispatches().is_empty() {
            self.reconciliation_required = false;
        }
        self.validated = Some(ValidatedJournalState {
            event_count: next_event_count,
            tail_record_sha256: next_checkpoint.record_sha256.clone(),
            checkpoint: Some(next_checkpoint.clone()),
            chain: candidate_chain,
        });
        Ok((
            AppendAcknowledgement {
                event_id: record.event.event_id,
                sequence,
                checkpoint: next_checkpoint,
                duplicate: false,
            },
            None,
        ))
    }
}
