//! Controller-owned authoritative-state contract.
//!
//! Unix uid/mode checks are defense-in-depth, not a same-uid containment
//! boundary. Activation additionally requires PF-27's independently validated
//! process, filesystem, config, and protected-store capabilities plus an
//! external durable anchor. Non-Unix persistence is an explicit activation
//! blocker until equivalent ACL, no-follow, and directory-durability checks are
//! implemented and qualified.

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_config::AuthoritativeSecurityState;
use codex_config::AuthoritativeStateCommit;
use codex_config::AuthoritativeStateOwner;
use codex_config::AuthoritativeStateValidationError;
use codex_secret_broker::platform_contract::PlatformReport;
use codex_secret_broker::platform_contract::ProtectedModeAuthorization;
use codex_secret_broker::platform_contract::ResultRejection;
use codex_secret_broker::platform_contract::validate_protected_mode_report;
use sha2::Digest;
use sha2::Sha256;
use thiserror::Error;

const STATE_PREFIX: &str = "state-";
const INTENT_PREFIX: &str = "intent-";
const COMMIT_PREFIX: &str = "commit-";
const RECORD_SUFFIX: &str = ".json";
const AUTHORITATIVE_ANCHOR_SCHEMA_VERSION: u32 = 1;

/// Opaque permission to mutate one authoritative ownership epoch.
///
/// Construction consumes the PF-27 platform witness returned for target and
/// probe identities that the platform-identity provider must derive
/// independently of the report and ordinary/model-editable configuration.
#[derive(Debug)]
pub(crate) struct TrustedControllerAuthorization {
    _platform_authorization: ProtectedModeAuthorization,
    owner: AuthoritativeStateOwner,
}

/// High-water mark held by a PF-27 protected-store provider outside the
/// append-only record directory.
///
/// The external anchor makes an empty or truncated record directory distinct
/// from a genuine first install. PF-20 deliberately defines this contract
/// without selecting an OS mechanism; protected activation remains unavailable
/// until PF-27 can supply an implementation with its protected-store capability.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoritativeStateAnchor {
    pub(crate) schema_version: u32,
    pub(crate) revision: u64,
    pub(crate) owner: AuthoritativeStateOwner,
    pub(crate) state_sha256: String,
    pub(crate) commit_sha256: String,
}

/// A provider must atomically compare against the exact expected anchor and
/// durably commit `next` before returning success. It must never synthesize a
/// missing anchor, accept rollback, or source values from agent-editable state.
pub(crate) trait AuthoritativeStateAnchorStore: std::fmt::Debug + Send + Sync {
    fn load_anchor(
        &self,
    ) -> Result<Option<AuthoritativeStateAnchor>, AuthoritativeStateAnchorError>;

    fn compare_and_store_anchor(
        &self,
        expected: Option<&AuthoritativeStateAnchor>,
        next: &AuthoritativeStateAnchor,
    ) -> Result<(), AuthoritativeStateAnchorError>;
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum AuthoritativeStateAnchorError {
    #[error("protected authoritative-state anchor is unavailable")]
    Unavailable,
    #[error("protected authoritative-state anchor changed concurrently")]
    Conflict,
    #[error("protected authoritative-state anchor is corrupt or unsupported")]
    Invalid,
}

impl AuthoritativeStateAnchor {
    fn validate(&self) -> Result<(), AuthoritativeStateAnchorError> {
        if self.schema_version != AUTHORITATIVE_ANCHOR_SCHEMA_VERSION
            || self.revision == 0
            || !is_lower_hex_sha256(&self.state_sha256)
            || !is_lower_hex_sha256(&self.commit_sha256)
            || self.owner.validate().is_err()
        {
            return Err(AuthoritativeStateAnchorError::Invalid);
        }
        Ok(())
    }
}

impl TrustedControllerAuthorization {
    pub(crate) fn from_platform_report(
        report: &PlatformReport<'_>,
        expected_target_id: &str,
        expected_probe_sha256: &str,
        now_unix_seconds: u64,
        owner_id: impl Into<String>,
        owner_generation: u64,
    ) -> Result<Self, AuthoritativeStateStoreError> {
        let platform_authorization = validate_protected_mode_report(
            report,
            expected_target_id,
            expected_probe_sha256,
            now_unix_seconds,
        )
        .map_err(AuthoritativeStateStoreError::PlatformAuthorization)?;
        let owner = AuthoritativeStateOwner::new(expected_target_id, owner_id, owner_generation)?;
        Ok(Self {
            _platform_authorization: platform_authorization,
            owner,
        })
    }

    fn authorizes(&self, state: &AuthoritativeSecurityState) -> bool {
        self.owner == state.owner
    }

    fn authorizes_owner(&self, owner: &AuthoritativeStateOwner) -> bool {
        &self.owner == owner
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AuthoritativeStateLoad {
    LegacyFirstInstall,
    Active(AuthoritativeSecurityState),
}

/// Append-only protected-state store.
///
/// Each revision is written as immutable state, intent and commit files. Intent
/// precedes commit, so a crash cannot make an incomplete generation active and
/// a later call can resume only the byte-identical, controller-authorized state.
#[derive(Clone, Debug)]
pub(crate) struct AuthoritativeStateStore {
    root: PathBuf,
    anchor_store: Arc<dyn AuthoritativeStateAnchorStore>,
}

impl AuthoritativeStateStore {
    pub(crate) fn new(
        root: impl Into<PathBuf>,
        anchor_store: Arc<dyn AuthoritativeStateAnchorStore>,
    ) -> Self {
        Self {
            root: root.into(),
            anchor_store,
        }
    }

    pub(crate) fn load(&self) -> Result<AuthoritativeStateLoad, AuthoritativeStateStoreError> {
        match self.inspect()? {
            StoreInspection::LegacyFirstInstall => Ok(AuthoritativeStateLoad::LegacyFirstInstall),
            StoreInspection::Active { head, .. } => Ok(AuthoritativeStateLoad::Active(head)),
            StoreInspection::Interrupted { revision, .. } => {
                Err(AuthoritativeStateStoreError::InterruptedWrite { revision })
            }
            StoreInspection::Unanchored { revision, .. } => {
                Err(AuthoritativeStateStoreError::UnanchoredRecords { revision })
            }
            StoreInspection::RejectedPending { revision, .. } => {
                Err(AuthoritativeStateStoreError::AnchorMismatch { revision })
            }
        }
    }

    pub(crate) fn compare_and_activate(
        &self,
        expected_revision: u64,
        next: &AuthoritativeSecurityState,
        authorization: &TrustedControllerAuthorization,
    ) -> Result<AuthoritativeSecurityState, AuthoritativeStateStoreError> {
        validate_store_root(&self.root)?;
        next.validate()?;
        if !authorization.authorizes(next) {
            return Err(AuthoritativeStateStoreError::UnauthorizedOwner);
        }

        let inspection = self.inspect()?;
        let (previous_commit_sha256, expected_anchor, advanced_anchor) = match &inspection {
            StoreInspection::LegacyFirstInstall => {
                if expected_revision != 0 || next.revision != 1 {
                    return Err(AuthoritativeStateStoreError::RevisionConflict {
                        expected: expected_revision,
                        actual: 0,
                    });
                }
                (None, None, None)
            }
            StoreInspection::Active {
                head,
                head_commit_sha256,
                anchor,
            } => {
                if expected_revision != head.revision {
                    return Err(AuthoritativeStateStoreError::RevisionConflict {
                        expected: expected_revision,
                        actual: head.revision,
                    });
                }
                next.validate_successor(head)?;
                (Some(head_commit_sha256.clone()), Some(anchor.clone()), None)
            }
            StoreInspection::Interrupted {
                head,
                head_commit_sha256,
                pending,
                revision,
                anchor,
            } => {
                let actual = head.as_ref().map_or(0, |state| state.revision);
                if expected_revision != actual || next.revision != *revision {
                    return Err(AuthoritativeStateStoreError::RevisionConflict {
                        expected: expected_revision,
                        actual,
                    });
                }
                if let Some(previous) = head {
                    next.validate_successor(previous)?;
                } else if next.revision != 1 {
                    return Err(AuthoritativeStateStoreError::NonInitialFirstRevision);
                }
                if let Some(pending) = pending
                    && pending != next
                {
                    return Err(AuthoritativeStateStoreError::InterruptedStateMismatch {
                        revision: *revision,
                    });
                }
                (head_commit_sha256.clone(), None, Some(anchor.clone()))
            }
            StoreInspection::Unanchored { revision, .. } => {
                return Err(AuthoritativeStateStoreError::UnanchoredRecords {
                    revision: *revision,
                });
            }
            StoreInspection::RejectedPending { revision, .. } => {
                return Err(AuthoritativeStateStoreError::AnchorMismatch {
                    revision: *revision,
                });
            }
        };

        let state_bytes = serialize(next)?;
        let state_sha256 = sha256(&state_bytes);
        let commit = AuthoritativeStateCommit::new(
            next.revision,
            state_sha256.clone(),
            previous_commit_sha256,
        )?;
        let commit_bytes = serialize(&commit)?;
        let next_anchor = AuthoritativeStateAnchor {
            schema_version: AUTHORITATIVE_ANCHOR_SCHEMA_VERSION,
            revision: next.revision,
            owner: next.owner.clone(),
            state_sha256,
            commit_sha256: sha256(&commit_bytes),
        };
        if let Some(anchor) = advanced_anchor {
            if anchor != next_anchor {
                return Err(AuthoritativeStateStoreError::InterruptedStateMismatch {
                    revision: next.revision,
                });
            }
        } else {
            self.anchor_store
                .compare_and_store_anchor(expected_anchor.as_ref(), &next_anchor)?;
        }

        write_once_or_verify(&self.root, &state_name(next.revision), &state_bytes)?;
        write_once_or_verify(&self.root, &intent_name(next.revision), &commit_bytes)?;
        write_once_or_verify(&self.root, &commit_name(next.revision), &commit_bytes)?;
        sync_directory(&self.root)?;

        match self.load()? {
            AuthoritativeStateLoad::Active(active) if active == *next => Ok(active),
            AuthoritativeStateLoad::Active(_) => {
                Err(AuthoritativeStateStoreError::CommittedStateMismatch)
            }
            AuthoritativeStateLoad::LegacyFirstInstall => {
                Err(AuthoritativeStateStoreError::CommitDidNotActivate)
            }
        }
    }

    /// Removes an unanchored suffix or a rejected anchored-pending record after
    /// re-validating the controller epoch. Anchored committed history is never
    /// removed by this operation.
    pub(crate) fn discard_unanchored_suffix(
        &self,
        revision: u64,
        authorization: &TrustedControllerAuthorization,
    ) -> Result<(), AuthoritativeStateStoreError> {
        let (head_revision, found, authorized) = match self.inspect()? {
            StoreInspection::Unanchored { head, revision, .. } => {
                (head.revision, revision, authorization.authorizes(&head))
            }
            StoreInspection::RejectedPending {
                head,
                revision,
                anchor,
            } => (
                head.as_ref().map_or(0, |state| state.revision),
                revision,
                authorization.authorizes_owner(&anchor.owner),
            ),
            _ => return Err(AuthoritativeStateStoreError::NoUnanchoredPending),
        };
        if found != revision || revision != head_revision.saturating_add(1) {
            return Err(AuthoritativeStateStoreError::UnanchoredRecords { revision: found });
        }
        if !authorized {
            return Err(AuthoritativeStateStoreError::UnauthorizedOwner);
        }
        let records = scan_records(&self.root)?;
        let highest = records
            .states
            .keys()
            .chain(records.intents.keys())
            .chain(records.commits.keys())
            .copied()
            .max()
            .unwrap_or(revision);
        for discard_revision in revision..=highest {
            for name in [
                state_name(discard_revision),
                intent_name(discard_revision),
                commit_name(discard_revision),
            ] {
                let path = self.root.join(name);
                match fs::symlink_metadata(&path) {
                    Ok(metadata) => {
                        if metadata.file_type().is_symlink() {
                            return Err(AuthoritativeStateStoreError::SymlinkRejected { path });
                        }
                        validate_private_metadata(&path, &metadata, false)?;
                        fs::remove_file(&path).map_err(|source| {
                            io_error("discard unanchored record", &path, source)
                        })?;
                    }
                    Err(source) if source.kind() == io::ErrorKind::NotFound => {}
                    Err(source) => {
                        return Err(io_error("inspect unanchored record", &path, source));
                    }
                }
            }
        }
        sync_directory(&self.root)
    }

    pub(crate) fn recover_from_revision(
        &self,
        expected_revision: u64,
        snapshot_revision: u64,
        authorization: &TrustedControllerAuthorization,
    ) -> Result<AuthoritativeSecurityState, AuthoritativeStateStoreError> {
        let StoreInspection::Active { head, .. } = self.inspect()? else {
            return Err(AuthoritativeStateStoreError::RecoveryRequiresActiveState);
        };
        if head.revision != expected_revision {
            return Err(AuthoritativeStateStoreError::RevisionConflict {
                expected: expected_revision,
                actual: head.revision,
            });
        }
        if !authorization.authorizes(&head) {
            return Err(AuthoritativeStateStoreError::UnauthorizedOwner);
        }
        let snapshot = self.read_committed_state(snapshot_revision)?;
        let recovered = AuthoritativeSecurityState::recovered_successor(&head, &snapshot)?;
        self.compare_and_activate(expected_revision, &recovered, authorization)
    }

    fn read_committed_state(
        &self,
        revision: u64,
    ) -> Result<AuthoritativeSecurityState, AuthoritativeStateStoreError> {
        let commit_bytes = read_private_file(&self.root.join(commit_name(revision)))?;
        let commit: AuthoritativeStateCommit = deserialize(&commit_bytes)?;
        commit.validate()?;
        if commit.revision != revision {
            return Err(AuthoritativeStateStoreError::RecordRevisionMismatch { revision });
        }
        let state_bytes = read_private_file(&self.root.join(state_name(revision)))?;
        if sha256(&state_bytes) != commit.state_sha256 {
            return Err(AuthoritativeStateStoreError::StateDigestMismatch { revision });
        }
        let state: AuthoritativeSecurityState = deserialize(&state_bytes)?;
        state.validate()?;
        if state.revision != revision {
            return Err(AuthoritativeStateStoreError::RecordRevisionMismatch { revision });
        }
        Ok(state)
    }

    fn inspect(&self) -> Result<StoreInspection, AuthoritativeStateStoreError> {
        let anchor = self.anchor_store.load_anchor()?;
        if let Some(anchor) = &anchor {
            anchor.validate()?;
        }
        let records = self.inspect_records()?;
        match (anchor, records) {
            (None, RecordInspection::Empty) => Ok(StoreInspection::LegacyFirstInstall),
            (Some(anchor), RecordInspection::Empty) if anchor.revision == 1 => {
                Ok(StoreInspection::Interrupted {
                    head: None,
                    head_commit_sha256: None,
                    pending: None,
                    revision: 1,
                    anchor,
                })
            }
            (Some(anchor), RecordInspection::Empty) => {
                Err(AuthoritativeStateStoreError::AnchorAheadOfRecords {
                    anchor_revision: anchor.revision,
                    record_revision: 0,
                })
            }
            (
                Some(anchor),
                RecordInspection::Active {
                    head,
                    head_state_sha256,
                    head_commit_sha256,
                },
            ) if anchor.revision == head.revision => {
                if anchor.owner != head.owner
                    || anchor.state_sha256 != head_state_sha256
                    || anchor.commit_sha256 != head_commit_sha256
                {
                    return Err(AuthoritativeStateStoreError::AnchorMismatch {
                        revision: anchor.revision,
                    });
                }
                Ok(StoreInspection::Active {
                    head,
                    head_commit_sha256,
                    anchor,
                })
            }
            (
                Some(anchor),
                RecordInspection::Active {
                    head,
                    head_commit_sha256,
                    ..
                },
            ) if anchor.revision == head.revision.saturating_add(1) => {
                let revision = anchor.revision;
                Ok(StoreInspection::Interrupted {
                    head: Some(head),
                    head_commit_sha256: Some(head_commit_sha256),
                    pending: None,
                    revision,
                    anchor,
                })
            }
            (Some(anchor), RecordInspection::Active { head, .. })
                if anchor.revision < head.revision =>
            {
                let anchored_head = self.read_committed_state(anchor.revision)?;
                let anchored_commit_bytes =
                    read_private_file(&self.root.join(commit_name(anchor.revision)))?;
                let anchored_commit: AuthoritativeStateCommit =
                    deserialize(&anchored_commit_bytes)?;
                let anchored_commit_sha256 = sha256(&anchored_commit_bytes);
                if anchor.owner != anchored_head.owner
                    || anchor.state_sha256 != anchored_commit.state_sha256
                    || anchor.commit_sha256 != anchored_commit_sha256
                {
                    return Err(AuthoritativeStateStoreError::AnchorMismatch {
                        revision: anchor.revision,
                    });
                }
                let revision = anchor.revision.saturating_add(1);
                Ok(StoreInspection::Unanchored {
                    head: anchored_head,
                    head_commit_sha256: anchored_commit_sha256,
                    revision,
                })
            }
            (Some(anchor), RecordInspection::Active { head, .. }) => {
                Err(AuthoritativeStateStoreError::AnchorAheadOfRecords {
                    anchor_revision: anchor.revision,
                    record_revision: head.revision,
                })
            }
            (
                None,
                RecordInspection::Active {
                    head,
                    head_commit_sha256,
                    ..
                },
            ) => {
                let revision = head.revision;
                Ok(StoreInspection::Unanchored {
                    head,
                    head_commit_sha256,
                    revision,
                })
            }
            (
                anchor,
                RecordInspection::Interrupted {
                    head,
                    head_commit_sha256,
                    pending,
                    pending_state_sha256,
                    pending_commit_sha256,
                    revision,
                },
            ) => {
                let head_revision = head.as_ref().map_or(0, |state| state.revision);
                let Some(anchor) = anchor else {
                    let Some(head) = head else {
                        return Err(AuthoritativeStateStoreError::UnanchoredRecords { revision });
                    };
                    return Ok(StoreInspection::Unanchored {
                        head,
                        head_commit_sha256: head_commit_sha256
                            .ok_or(AuthoritativeStateStoreError::MissingProtectedState)?,
                        revision,
                    });
                };
                if anchor.revision == head_revision {
                    let Some(head) = head else {
                        return Err(AuthoritativeStateStoreError::UnanchoredRecords { revision });
                    };
                    return Ok(StoreInspection::Unanchored {
                        head,
                        head_commit_sha256: head_commit_sha256
                            .ok_or(AuthoritativeStateStoreError::MissingProtectedState)?,
                        revision,
                    });
                }
                if anchor.revision != revision || revision != head_revision.saturating_add(1) {
                    return Err(AuthoritativeStateStoreError::AnchorAheadOfRecords {
                        anchor_revision: anchor.revision,
                        record_revision: head_revision,
                    });
                }
                if let Some(pending) = &pending
                    && (anchor.owner != pending.owner
                        || pending_state_sha256.as_deref() != Some(anchor.state_sha256.as_str())
                        || pending_commit_sha256.as_deref() != Some(anchor.commit_sha256.as_str()))
                {
                    return Ok(StoreInspection::RejectedPending {
                        head,
                        revision,
                        anchor,
                    });
                }
                Ok(StoreInspection::Interrupted {
                    head,
                    head_commit_sha256,
                    pending,
                    revision,
                    anchor,
                })
            }
        }
    }

    fn inspect_records(&self) -> Result<RecordInspection, AuthoritativeStateStoreError> {
        validate_store_root(&self.root)?;
        let records = scan_records(&self.root)?;
        if records.states.is_empty() && records.intents.is_empty() && records.commits.is_empty() {
            return Ok(RecordInspection::Empty);
        }

        let highest = records
            .states
            .keys()
            .chain(records.intents.keys())
            .chain(records.commits.keys())
            .copied()
            .max()
            .ok_or(AuthoritativeStateStoreError::MissingProtectedState)?;
        let committed_highest = records.commits.keys().next_back().copied().unwrap_or(0);
        if highest > committed_highest.saturating_add(1) {
            return Err(AuthoritativeStateStoreError::NonSequentialRecords);
        }

        let mut head = None;
        let mut previous_commit_sha256 = None;
        let mut head_state_sha256 = None;
        for revision in 1..=committed_highest {
            if !records.states.contains_key(&revision)
                || !records.intents.contains_key(&revision)
                || !records.commits.contains_key(&revision)
            {
                return Err(AuthoritativeStateStoreError::MissingCommittedRecord { revision });
            }
            let intent_bytes = read_private_file(&self.root.join(intent_name(revision)))?;
            let commit_bytes = read_private_file(&self.root.join(commit_name(revision)))?;
            if intent_bytes != commit_bytes {
                return Err(AuthoritativeStateStoreError::IntentCommitMismatch { revision });
            }
            let commit: AuthoritativeStateCommit = deserialize(&commit_bytes)?;
            commit.validate()?;
            if commit.revision != revision
                || commit.previous_commit_sha256 != previous_commit_sha256
            {
                return Err(AuthoritativeStateStoreError::InvalidCommitChain { revision });
            }
            let state = self.read_committed_state(revision)?;
            if let Some(previous) = &head {
                state.validate_successor(previous)?;
            }
            previous_commit_sha256 = Some(sha256(&commit_bytes));
            head_state_sha256 = Some(commit.state_sha256);
            head = Some(state);
        }

        if highest == committed_highest {
            let Some(head) = head else {
                return Err(AuthoritativeStateStoreError::MissingProtectedState);
            };
            return Ok(RecordInspection::Active {
                head,
                head_state_sha256: head_state_sha256
                    .ok_or(AuthoritativeStateStoreError::MissingProtectedState)?,
                head_commit_sha256: previous_commit_sha256
                    .ok_or(AuthoritativeStateStoreError::MissingProtectedState)?,
            });
        }

        let revision = highest;
        if revision != committed_highest.saturating_add(1)
            || !records.states.contains_key(&revision)
            || records.commits.contains_key(&revision)
        {
            return Err(AuthoritativeStateStoreError::NonSequentialRecords);
        }
        let pending_bytes = read_private_file(&self.root.join(state_name(revision)))?;
        let pending: AuthoritativeSecurityState = deserialize(&pending_bytes)?;
        pending.validate()?;
        if pending.revision != revision {
            return Err(AuthoritativeStateStoreError::RecordRevisionMismatch { revision });
        }
        if let Some(previous) = &head {
            pending.validate_successor(previous)?;
        } else if revision != 1 {
            return Err(AuthoritativeStateStoreError::NonInitialFirstRevision);
        }
        if records.intents.contains_key(&revision) {
            let intent_bytes = read_private_file(&self.root.join(intent_name(revision)))?;
            let intent: AuthoritativeStateCommit = deserialize(&intent_bytes)?;
            intent.validate()?;
            if intent.revision != revision
                || intent.state_sha256 != sha256(&pending_bytes)
                || intent.previous_commit_sha256 != previous_commit_sha256
            {
                return Err(AuthoritativeStateStoreError::InvalidIntent { revision });
            }
        }
        let pending_state_sha256 = sha256(&pending_bytes);
        let pending_commit = AuthoritativeStateCommit::new(
            revision,
            pending_state_sha256.clone(),
            previous_commit_sha256.clone(),
        )?;
        let pending_commit_sha256 = sha256(&serialize(&pending_commit)?);
        Ok(RecordInspection::Interrupted {
            head,
            head_commit_sha256: previous_commit_sha256,
            pending: Some(pending),
            pending_state_sha256: Some(pending_state_sha256),
            pending_commit_sha256: Some(pending_commit_sha256),
            revision,
        })
    }
}

#[derive(Debug)]
enum StoreInspection {
    LegacyFirstInstall,
    Active {
        head: AuthoritativeSecurityState,
        head_commit_sha256: String,
        anchor: AuthoritativeStateAnchor,
    },
    Interrupted {
        head: Option<AuthoritativeSecurityState>,
        head_commit_sha256: Option<String>,
        pending: Option<AuthoritativeSecurityState>,
        revision: u64,
        anchor: AuthoritativeStateAnchor,
    },
    Unanchored {
        head: AuthoritativeSecurityState,
        head_commit_sha256: String,
        revision: u64,
    },
    RejectedPending {
        head: Option<AuthoritativeSecurityState>,
        revision: u64,
        anchor: AuthoritativeStateAnchor,
    },
}

#[derive(Debug)]
enum RecordInspection {
    Empty,
    Active {
        head: AuthoritativeSecurityState,
        head_state_sha256: String,
        head_commit_sha256: String,
    },
    Interrupted {
        head: Option<AuthoritativeSecurityState>,
        head_commit_sha256: Option<String>,
        pending: Option<AuthoritativeSecurityState>,
        pending_state_sha256: Option<String>,
        pending_commit_sha256: Option<String>,
        revision: u64,
    },
}

#[derive(Default)]
struct StoreRecords {
    states: BTreeMap<u64, ()>,
    intents: BTreeMap<u64, ()>,
    commits: BTreeMap<u64, ()>,
}

fn scan_records(root: &Path) -> Result<StoreRecords, AuthoritativeStateStoreError> {
    let mut records = StoreRecords::default();
    for entry in fs::read_dir(root).map_err(|source| io_error("read directory", root, source))? {
        let entry = entry.map_err(|source| io_error("read directory entry", root, source))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|source| io_error("inspect directory entry", &path, source))?;
        if metadata.file_type().is_symlink() {
            return Err(AuthoritativeStateStoreError::SymlinkRejected { path });
        }
        validate_private_metadata(&path, &metadata, /*directory*/ false)?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(AuthoritativeStateStoreError::UnexpectedEntry { path });
        };
        if name.starts_with(".authoritative-state-") {
            continue;
        }
        if let Some(revision) = parse_record_name(name, STATE_PREFIX) {
            records.states.insert(revision, ());
        } else if let Some(revision) = parse_record_name(name, INTENT_PREFIX) {
            records.intents.insert(revision, ());
        } else if let Some(revision) = parse_record_name(name, COMMIT_PREFIX) {
            records.commits.insert(revision, ());
        } else {
            return Err(AuthoritativeStateStoreError::UnexpectedEntry { path });
        }
    }
    Ok(records)
}

fn parse_record_name(name: &str, prefix: &str) -> Option<u64> {
    let revision = name.strip_prefix(prefix)?.strip_suffix(RECORD_SUFFIX)?;
    if revision.len() != 20 || !revision.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    revision.parse().ok()
}

fn state_name(revision: u64) -> String {
    format!("{STATE_PREFIX}{revision:020}{RECORD_SUFFIX}")
}

fn intent_name(revision: u64) -> String {
    format!("{INTENT_PREFIX}{revision:020}{RECORD_SUFFIX}")
}

fn commit_name(revision: u64) -> String {
    format!("{COMMIT_PREFIX}{revision:020}{RECORD_SUFFIX}")
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, AuthoritativeStateStoreError> {
    let mut bytes = serde_json::to_vec(value).map_err(AuthoritativeStateStoreError::Serialize)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn deserialize<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, AuthoritativeStateStoreError> {
    serde_json::from_slice(bytes).map_err(AuthoritativeStateStoreError::Deserialize)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(unix)]
fn validate_store_root(root: &Path) -> Result<(), AuthoritativeStateStoreError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|source| io_error("inspect protected-state root", root, source))?;
    if metadata.file_type().is_symlink() {
        return Err(AuthoritativeStateStoreError::SymlinkRejected {
            path: root.to_path_buf(),
        });
    }
    validate_private_metadata(root, &metadata, /*directory*/ true)
}

#[cfg(not(unix))]
fn validate_store_root(_root: &Path) -> Result<(), AuthoritativeStateStoreError> {
    Err(AuthoritativeStateStoreError::UnsupportedPlatform)
}

fn read_private_file(path: &Path) -> Result<Vec<u8>, AuthoritativeStateStoreError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| io_error("inspect protected-state file", path, source))?;
    if metadata.file_type().is_symlink() {
        return Err(AuthoritativeStateStoreError::SymlinkRejected {
            path: path.to_path_buf(),
        });
    }
    validate_private_metadata(path, &metadata, /*directory*/ false)?;
    fs::read(path).map_err(|source| io_error("read protected-state file", path, source))
}

fn write_once_or_verify(
    root: &Path,
    name: &str,
    contents: &[u8],
) -> Result<(), AuthoritativeStateStoreError> {
    let destination = root.join(name);
    match fs::symlink_metadata(&destination) {
        Ok(_) => return verify_existing_record(&destination, contents),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(io_error(
                "inspect immutable protected-state destination",
                &destination,
                source,
            ));
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = root.join(format!(
        ".authoritative-state-{}-{nanos}-{}",
        std::process::id(),
        name
    ));
    let mut file = open_private_create_new(&temporary)?;
    file.write_all(contents)
        .map_err(|source| io_error("write temporary state", &temporary, source))?;
    file.sync_all()
        .map_err(|source| io_error("sync temporary state", &temporary, source))?;
    drop(file);

    match fs::hard_link(&temporary, &destination) {
        Ok(()) => {}
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(&temporary);
            return verify_existing_record(&destination, contents);
        }
        Err(source) => {
            let _ = fs::remove_file(&temporary);
            return Err(io_error(
                "publish immutable protected-state record",
                &destination,
                source,
            ));
        }
    }
    if temporary.exists() {
        fs::remove_file(&temporary)
            .map_err(|source| io_error("remove temporary state", &temporary, source))?;
    }
    sync_directory(root)?;
    Ok(())
}

fn verify_existing_record(
    destination: &Path,
    contents: &[u8],
) -> Result<(), AuthoritativeStateStoreError> {
    let existing = read_private_file(destination)?;
    if existing == contents {
        Ok(())
    } else {
        Err(AuthoritativeStateStoreError::ExistingRecordMismatch {
            path: destination.to_path_buf(),
        })
    }
}

#[cfg(unix)]
fn open_private_create_new(path: &Path) -> Result<File, AuthoritativeStateStoreError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|source| io_error("create temporary state", path, source))
}

#[cfg(not(unix))]
fn open_private_create_new(path: &Path) -> Result<File, AuthoritativeStateStoreError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| io_error("create temporary state", path, source))
}

#[cfg(unix)]
fn validate_private_metadata(
    path: &Path,
    metadata: &fs::Metadata,
    directory: bool,
) -> Result<(), AuthoritativeStateStoreError> {
    use std::os::unix::fs::MetadataExt;

    if directory != metadata.is_dir() || (!directory && !metadata.is_file()) {
        return Err(AuthoritativeStateStoreError::UnexpectedFileType {
            path: path.to_path_buf(),
        });
    }
    // SAFETY: `geteuid` has no arguments or caller-side safety preconditions.
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(AuthoritativeStateStoreError::WrongOwner {
            path: path.to_path_buf(),
        });
    }
    if metadata.mode() & 0o077 != 0 {
        return Err(AuthoritativeStateStoreError::PermissionsTooOpen {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_metadata(
    path: &Path,
    metadata: &fs::Metadata,
    directory: bool,
) -> Result<(), AuthoritativeStateStoreError> {
    if directory != metadata.is_dir() || (!directory && !metadata.is_file()) {
        return Err(AuthoritativeStateStoreError::UnexpectedFileType {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(not(windows))]
fn sync_directory(root: &Path) -> Result<(), AuthoritativeStateStoreError> {
    File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error("sync protected-state directory", root, source))
}

#[cfg(windows)]
fn sync_directory(_root: &Path) -> Result<(), AuthoritativeStateStoreError> {
    Ok(())
}

fn io_error(
    operation: &'static str,
    path: &Path,
    source: io::Error,
) -> AuthoritativeStateStoreError {
    AuthoritativeStateStoreError::Io {
        operation,
        path: path.to_path_buf(),
        source,
    }
}

#[derive(Debug, Error)]
pub(crate) enum AuthoritativeStateStoreError {
    #[error("platform containment did not authorize protected state: {0}")]
    PlatformAuthorization(ResultRejection),
    #[error(transparent)]
    Validation(#[from] AuthoritativeStateValidationError),
    #[error(transparent)]
    Anchor(#[from] AuthoritativeStateAnchorError),
    #[error("protected authoritative-state persistence is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("authoritative state is not owned by the authorized controller epoch")]
    UnauthorizedOwner,
    #[error("expected authoritative revision {expected}, but current revision is {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("the first protected-state revision must be one")]
    NonInitialFirstRevision,
    #[error("protected-state write for revision {revision} was interrupted")]
    InterruptedWrite { revision: u64 },
    #[error("interrupted revision {revision} does not match the proposed state")]
    InterruptedStateMismatch { revision: u64 },
    #[error("unanchored protected-state records begin at revision {revision}")]
    UnanchoredRecords { revision: u64 },
    #[error("there is no unanchored pending revision to discard")]
    NoUnanchoredPending,
    #[error(
        "protected anchor revision {anchor_revision} is ahead of record revision {record_revision}"
    )]
    AnchorAheadOfRecords {
        anchor_revision: u64,
        record_revision: u64,
    },
    #[error("protected anchor does not match authoritative record {revision}")]
    AnchorMismatch { revision: u64 },
    #[error("protected-state records are not sequential")]
    NonSequentialRecords,
    #[error("protected-state record {revision} is missing")]
    MissingCommittedRecord { revision: u64 },
    #[error("protected state is missing after activation")]
    MissingProtectedState,
    #[error("state or commit revision does not match record {revision}")]
    RecordRevisionMismatch { revision: u64 },
    #[error("state digest does not match commit {revision}")]
    StateDigestMismatch { revision: u64 },
    #[error("intent and commit differ for revision {revision}")]
    IntentCommitMismatch { revision: u64 },
    #[error("commit chain is invalid at revision {revision}")]
    InvalidCommitChain { revision: u64 },
    #[error("pending intent is invalid at revision {revision}")]
    InvalidIntent { revision: u64 },
    #[error("existing protected-state record differs: {path}")]
    ExistingRecordMismatch { path: PathBuf },
    #[error("protected-state path is a symlink: {path}")]
    SymlinkRejected { path: PathBuf },
    #[error("protected-state path has the wrong file type: {path}")]
    UnexpectedFileType { path: PathBuf },
    #[error("protected-state path is not owned by the controller identity: {path}")]
    WrongOwner { path: PathBuf },
    #[error("protected-state permissions are too open: {path}")]
    PermissionsTooOpen { path: PathBuf },
    #[error("unexpected protected-state directory entry: {path}")]
    UnexpectedEntry { path: PathBuf },
    #[error("recovery requires a complete active state")]
    RecoveryRequiresActiveState,
    #[error("committed state did not match the proposed state")]
    CommittedStateMismatch,
    #[error("commit did not activate protected state")]
    CommitDidNotActivate,
    #[error("failed to serialize authoritative state: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize authoritative state: {0}")]
    Deserialize(serde_json::Error),
    #[error("{operation} at {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
