use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
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

/// Opaque permission to mutate one authoritative ownership epoch.
///
/// Construction consumes the PF-27 platform witness returned for the exact
/// expected target and probe identities. Callers cannot replace that witness
/// with a model-supplied role or a value from ordinary configuration.
#[derive(Debug)]
pub(crate) struct TrustedControllerAuthorization {
    _platform_authorization: ProtectedModeAuthorization,
    owner: AuthoritativeStateOwner,
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
}

impl AuthoritativeStateStore {
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) fn load(&self) -> Result<AuthoritativeStateLoad, AuthoritativeStateStoreError> {
        match self.inspect()? {
            StoreInspection::LegacyFirstInstall => Ok(AuthoritativeStateLoad::LegacyFirstInstall),
            StoreInspection::Active { head, .. } => Ok(AuthoritativeStateLoad::Active(head)),
            StoreInspection::Interrupted { revision, .. } => {
                Err(AuthoritativeStateStoreError::InterruptedWrite { revision })
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
        let previous_commit_sha256 = match &inspection {
            StoreInspection::LegacyFirstInstall => {
                if expected_revision != 0 || next.revision != 1 {
                    return Err(AuthoritativeStateStoreError::RevisionConflict {
                        expected: expected_revision,
                        actual: 0,
                    });
                }
                None
            }
            StoreInspection::Active {
                head,
                head_commit_sha256,
            } => {
                if expected_revision != head.revision {
                    return Err(AuthoritativeStateStoreError::RevisionConflict {
                        expected: expected_revision,
                        actual: head.revision,
                    });
                }
                next.validate_successor(head)?;
                Some(head_commit_sha256.clone())
            }
            StoreInspection::Interrupted {
                head,
                head_commit_sha256,
                pending,
                revision,
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
                head_commit_sha256.clone()
            }
        };

        let state_bytes = serialize(next)?;
        let state_sha256 = sha256(&state_bytes);
        let commit =
            AuthoritativeStateCommit::new(next.revision, state_sha256, previous_commit_sha256)?;
        let commit_bytes = serialize(&commit)?;

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
        validate_store_root(&self.root)?;
        let records = scan_records(&self.root)?;
        if records.states.is_empty() && records.intents.is_empty() && records.commits.is_empty() {
            return Ok(StoreInspection::LegacyFirstInstall);
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
            head = Some(state);
        }

        if highest == committed_highest {
            let Some(head) = head else {
                return Err(AuthoritativeStateStoreError::MissingProtectedState);
            };
            return Ok(StoreInspection::Active {
                head,
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
        Ok(StoreInspection::Interrupted {
            head,
            head_commit_sha256: previous_commit_sha256,
            pending: Some(pending),
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
    },
    Interrupted {
        head: Option<AuthoritativeSecurityState>,
        head_commit_sha256: Option<String>,
        pending: Option<AuthoritativeSecurityState>,
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
    if destination.exists() {
        let existing = read_private_file(&destination)?;
        if existing == contents {
            return Ok(());
        }
        return Err(AuthoritativeStateStoreError::ExistingRecordMismatch { path: destination });
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
            return write_once_or_verify(root, name, contents);
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
