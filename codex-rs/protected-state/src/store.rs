use crate::PolicyCheckpoint;
use crate::RootError;
use crate::checkpoint::Binding;
use crate::checkpoint::Checkpoint;
use crate::linux::Directory;
use codex_config::AuthoritativeStateOwner;
use codex_security_audit::IntegrityCheckpoint;
use codex_security_audit::IntegrityRootError;
use codex_security_audit::IntegrityRootStore;
use codex_security_audit::JournalOwner;
use hmac::Hmac;
use hmac::Mac;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use zeroize::Zeroizing;

const REGISTRY: &str = "/etc/corbanu-protected-state";
const STORAGE: &str = "/var/lib/corbanu-protected-state";

/// Administrator-selected namespace, not authority. Only the root-only system
/// enrollment operation may commit it; normal startup never enrolls implicitly.
pub struct Enrollment(Binding);

impl Enrollment {
    pub fn journal(owner: &JournalOwner) -> Self {
        Self(Binding::Journal {
            producer: owner.producer().clone(),
            owner_generation: owner.owner_generation(),
            integrity_key_id: owner.integrity_key_id().clone(),
        })
    }

    pub fn policy(owner: AuthoritativeStateOwner) -> Result<Self, RootError> {
        owner.validate().map_err(|_| RootError::Invalid)?;
        Ok(Self(Binding::Policy { owner }))
    }

    fn name(&self) -> &'static str {
        match &self.0 {
            Binding::Journal { .. } => "journal",
            Binding::Policy { .. } => "policy",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Registration {
    schema: u32,
    installation: [u8; 32],
    binding: Binding,
    key_digest: [u8; 32],
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Head {
    registration: Registration,
    checkpoint: Option<Checkpoint>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthenticatedHead {
    payload: Vec<u8>,
    tag: Vec<u8>,
}

struct State {
    directory: Directory,
    registry: Directory,
    registration: Registration,
    key: Zeroizing<[u8; 32]>,
    _lock: File,
    failed: bool,
}

/// Durable root owned by the native controller. Public construction is fixed-
/// location and kernel-root-only, not a caller-selected path, file or UID. This
/// narrow storage readiness does NOT imply PF-27 protected-mode authorization.
pub struct ControllerRoot {
    state: Mutex<State>,
}

impl std::fmt::Debug for ControllerRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControllerRoot").finish_non_exhaustive()
    }
}

fn system_authority() -> Result<(), RootError> {
    // SAFETY: geteuid has no caller-side preconditions. No caller UID is accepted.
    if unsafe { libc::geteuid() } == 0 {
        Directory::verify_system_path(Path::new(REGISTRY))?;
        Directory::verify_system_path(Path::new(STORAGE))?;
        Ok(())
    } else {
        Err(RootError::Unavailable)
    }
}

impl ControllerRoot {
    /// Explicit first enrollment. The administrator must already have installed
    /// the two root-only directories. No directory setup, reset, overwrite or
    /// recovery is hidden here. Partial enrollment blocks subsequent enrollment.
    pub fn enroll_system(enrollment: Enrollment) -> Result<Self, RootError> {
        system_authority()?;
        let name = enrollment.name();
        Self::enroll(
            &Path::new(REGISTRY).join(name),
            &Path::new(STORAGE).join(name),
            enrollment.0,
        )
    }

    /// Existing journal only; ENOENT never means a new protected installation.
    pub fn open_journal_system() -> Result<Self, RootError> {
        system_authority()?;
        Self::open(
            &Path::new(REGISTRY).join("journal"),
            &Path::new(STORAGE).join("journal"),
        )
    }

    pub fn open_policy_system() -> Result<Self, RootError> {
        system_authority()?;
        Self::open(
            &Path::new(REGISTRY).join("policy"),
            &Path::new(STORAGE).join("policy"),
        )
    }

    pub(crate) fn enroll(
        registry: &Path,
        storage: &Path,
        binding: Binding,
    ) -> Result<Self, RootError> {
        use sha2::Digest;
        let registry = Directory::open(registry)?;
        let directory = Directory::open(storage)?;
        // create_new is the persistent one-shot fence. Keep it even on failure.
        registry.create("enrollment", b"corbanu-enrollment/v1")?;
        let key = Zeroizing::new(rand::random::<[u8; 32]>());
        let registration = Registration {
            schema: 1,
            installation: rand::random(),
            binding,
            key_digest: Sha256::digest(*key).into(),
        };
        directory.create("key", key.as_ref())?;
        directory.create("lock", b"controller-lock/v1")?;
        let lock = directory.lock()?;
        let mut state = State {
            directory,
            registry,
            registration,
            key,
            _lock: lock,
            failed: false,
        };
        state.directory.create("head", &state.encode(None)?)?;
        state.registry.create(
            "complete",
            &serde_json::to_vec(&state.registration).map_err(|_| RootError::Invalid)?,
        )?;
        state.load()?;
        Ok(Self {
            state: Mutex::new(state),
        })
    }

    fn open(registry: &Path, storage: &Path) -> Result<Self, RootError> {
        use sha2::Digest;
        let registry = Directory::open(registry)?;
        if registry.read("enrollment")? != b"corbanu-enrollment/v1" {
            return Err(RootError::Invalid);
        }
        let registration: Registration =
            serde_json::from_slice(&registry.read("complete")?).map_err(|_| RootError::Invalid)?;
        if registration.schema != 1 {
            return Err(RootError::Invalid);
        }
        let directory = Directory::open(storage)?;
        let lock = directory.lock()?;
        let key = Zeroizing::new(
            <[u8; 32]>::try_from(directory.read("key").map_err(|_| RootError::MissingKey)?)
                .map_err(|_| RootError::MissingKey)?,
        );
        if registration.key_digest != <[u8; 32]>::from(Sha256::digest(*key)) {
            return Err(RootError::MissingKey);
        }
        let mut state = State {
            directory,
            registry,
            registration,
            key,
            _lock: lock,
            failed: false,
        };
        state.load()?;
        Ok(Self {
            state: Mutex::new(state),
        })
    }

    pub(crate) fn load(&self) -> Result<Option<Checkpoint>, RootError> {
        self.state
            .lock()
            .map_err(|_| RootError::Unavailable)?
            .load()
    }

    pub(crate) fn compare(
        &self,
        expected: Option<&Checkpoint>,
        next: &Checkpoint,
    ) -> Result<(), RootError> {
        let mut state = self.state.lock().map_err(|_| RootError::Unavailable)?;
        if state.load()?.as_ref() != expected {
            return Err(RootError::Conflict);
        }
        next.validate_successor(expected, &state.registration.binding)?;
        let encoded = state.encode(Some(next.clone()))?;
        let result = state
            .directory
            .create("pending", &encoded)
            .and_then(|()| state.directory.publish());
        if result.is_err() {
            state.failed = true;
        }
        result
    }

    pub(crate) fn require_policy(&self) -> Result<(), RootError> {
        if matches!(
            self.state
                .lock()
                .map_err(|_| RootError::Unavailable)?
                .registration
                .binding,
            Binding::Policy { .. }
        ) {
            Ok(())
        } else {
            Err(RootError::Invalid)
        }
    }

    pub(crate) fn require_journal(&self) -> Result<(), RootError> {
        if matches!(
            self.state
                .lock()
                .map_err(|_| RootError::Unavailable)?
                .registration
                .binding,
            Binding::Journal { .. }
        ) {
            Ok(())
        } else {
            Err(RootError::Invalid)
        }
    }
}

impl crate::PolicyRootStore for ControllerRoot {
    fn load_policy(&self) -> Result<Option<PolicyCheckpoint>, RootError> {
        self.require_policy()?;
        match self.load()? {
            None => Ok(None),
            Some(Checkpoint::Policy(value)) => Ok(Some(value)),
            Some(Checkpoint::Journal(_)) => Err(RootError::Invalid),
        }
    }

    fn compare_policy(
        &self,
        expected: Option<&PolicyCheckpoint>,
        next: &PolicyCheckpoint,
    ) -> Result<(), RootError> {
        self.compare(
            expected.cloned().map(Checkpoint::Policy).as_ref(),
            &Checkpoint::Policy(next.clone()),
        )
    }
}

impl State {
    fn encode(&self, checkpoint: Option<Checkpoint>) -> Result<Vec<u8>, RootError> {
        let payload = serde_json::to_vec(&Head {
            registration: self.registration.clone(),
            checkpoint,
        })
        .map_err(|_| RootError::Invalid)?;
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.key.as_ref()).map_err(|_| RootError::Invalid)?;
        mac.update(b"corbanu-controller-head/v1\0");
        mac.update(&payload);
        serde_json::to_vec(&AuthenticatedHead {
            payload,
            tag: mac.finalize().into_bytes().to_vec(),
        })
        .map_err(|_| RootError::Invalid)
    }

    fn load(&mut self) -> Result<Option<Checkpoint>, RootError> {
        if self.failed {
            return Err(RootError::Unavailable);
        }
        let result = self.read_verified();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn read_verified(&self) -> Result<Option<Checkpoint>, RootError> {
        if self.directory.has_pending()? {
            return Err(RootError::Ambiguous);
        }
        // Loss or substitution of the independent enrollment/key is fatal even
        // while the controller process still holds its previous valid key.
        if self.registry.read("enrollment")? != b"corbanu-enrollment/v1" {
            return Err(RootError::Invalid);
        }
        let registration: Registration = serde_json::from_slice(&self.registry.read("complete")?)
            .map_err(|_| RootError::Invalid)?;
        if registration != self.registration
            || self.directory.read("key")?.as_slice() != self.key.as_ref()
        {
            return Err(RootError::Invalid);
        }
        let encoded: AuthenticatedHead = serde_json::from_slice(&self.directory.read("head")?)
            .map_err(|_| RootError::Invalid)?;
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.key.as_ref()).map_err(|_| RootError::Invalid)?;
        mac.update(b"corbanu-controller-head/v1\0");
        mac.update(&encoded.payload);
        mac.verify_slice(&encoded.tag)
            .map_err(|_| RootError::Invalid)?;
        let head: Head =
            serde_json::from_slice(&encoded.payload).map_err(|_| RootError::Invalid)?;
        if head.registration != self.registration {
            return Err(RootError::Invalid);
        }
        if let Some(checkpoint) = &head.checkpoint {
            checkpoint.validate(&self.registration.binding)?;
        }
        Ok(head.checkpoint)
    }
}

impl IntegrityRootStore for ControllerRoot {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        self.require_journal()?;
        match ControllerRoot::load(self)? {
            None => Ok(None),
            Some(Checkpoint::Journal(value)) => Ok(Some(value)),
            Some(Checkpoint::Policy(_)) => Err(IntegrityRootError::Invalid),
        }
    }

    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        self.compare(
            expected.cloned().map(Checkpoint::Journal).as_ref(),
            &Checkpoint::Journal(next.clone()),
        )
        .map_err(Into::into)
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
