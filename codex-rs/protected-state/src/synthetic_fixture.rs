//! Fresh-only Linux/GNU test data. No caller-selected root, path, key or UID.
use super::*;
use crate::PolicyRootStore;
use crate::checkpoint::Binding;
use codex_config::AuthoritativeStateOwner;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

pub struct SyntheticRootFixture {
    roots: [Arc<ControllerRoot>; 2],
    _directory: tempfile::TempDir,
}
impl SyntheticRootFixture {
    pub fn fresh() -> Result<Self, RootError> {
        let directory = tempfile::tempdir().map_err(|_| RootError::Unavailable)?;
        let bindings = [
            Binding::Journal {
                producer: Self::journal_checkpoint()?.producer,
                owner_generation: 1,
                integrity_key_id: BoundedText::new("fixture-key")
                    .map_err(|_| RootError::Invalid)?,
            },
            Binding::Policy {
                owner: Self::policy_checkpoint()?.owner,
            },
        ];
        let mut roots = Vec::new();
        for (index, binding) in bindings.into_iter().enumerate() {
            let registry = directory.path().join(format!("registry-{index}"));
            let storage = directory.path().join(format!("storage-{index}"));
            for path in [&registry, &storage] {
                std::fs::create_dir(path).map_err(|_| RootError::Unavailable)?;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
                    .map_err(|_| RootError::Unavailable)?;
            }
            roots.push(Arc::new(ControllerRoot::enroll(
                &registry, &storage, binding,
            )?));
        }
        Ok(Self {
            roots: roots.try_into().map_err(|_| RootError::Invalid)?,
            _directory: directory,
        })
    }
    pub fn roots(&self) -> [Arc<ControllerRoot>; 2] {
        self.roots.clone()
    }
    pub fn client(stream: UnixStream) -> Result<NativeAnchorClient, RootError> {
        NativeAnchorClient::from_authenticated_stream(stream)
    }
    pub fn journal_checkpoint() -> Result<IntegrityCheckpoint, RootError> {
        Ok(IntegrityCheckpoint {
            schema_version: 1,
            sequence: 1,
            record_sha256: "a".repeat(64),
            producer: PolicyPrincipal::new(PrincipalKind::Service, "native-fixture")
                .map_err(|_| RootError::Invalid)?,
            owner_generation: 1,
            integrity_key_id: BoundedText::new("fixture-key").map_err(|_| RootError::Invalid)?,
            policy_generation: 1,
            run_generation: 1,
        })
    }
    pub fn policy_checkpoint() -> Result<PolicyCheckpoint, RootError> {
        Ok(PolicyCheckpoint {
            schema_version: 1,
            revision: 1,
            owner: AuthoritativeStateOwner::new("1".repeat(64), "controller", 1)
                .map_err(|_| RootError::Invalid)?,
            state_sha256: "a".repeat(64),
            commit_sha256: "b".repeat(64),
        })
    }
    /// Inject a stale authenticated sequence using the actual client framing.
    pub fn replay_load(client: &NativeAnchorClient) -> Result<(), RootError> {
        let mut slot = client.channel.lock().map_err(|_| RootError::Unavailable)?;
        let channel = slot.as_mut().ok_or(RootError::Unavailable)?;
        channel.sequence = 1;
        channel.send(&Request::LoadJournal, b"corbanu-anchor-request/v1")?;
        let result = channel
            .receive::<Reply>(b"corbanu-anchor-reply/v1")
            .map(|_| ());
        *slot = None;
        result
    }
    pub fn stored(&self) -> Result<bool, RootError> {
        Ok(IntegrityRootStore::load(self.roots[0].as_ref())
            .map_err(|_| RootError::Unavailable)?
            .is_some()
            && self.roots[1].load_policy()?.is_some())
    }
}
