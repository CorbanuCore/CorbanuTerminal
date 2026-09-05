use codex_config::AuthoritativeStateOwner;
use codex_security_audit::IntegrityCheckpoint;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use serde::Deserialize;
use serde::Serialize;

use crate::RootError;

/// The existing PF-20 policy anchor payload. Values are data, not authorization;
/// a controller namespace must independently bind the owner before accepting it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCheckpoint {
    pub schema_version: u32,
    pub revision: u64,
    pub owner: AuthoritativeStateOwner,
    pub state_sha256: String,
    pub commit_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum Checkpoint {
    Journal(IntegrityCheckpoint),
    Policy(PolicyCheckpoint),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum Binding {
    Journal {
        producer: PolicyPrincipal,
        owner_generation: u64,
        integrity_key_id: BoundedText,
    },
    Policy { owner: AuthoritativeStateOwner },
}

pub(crate) fn hash_valid(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl Checkpoint {
    pub(crate) fn validate_successor(
        &self,
        previous: Option<&Self>,
        binding: &Binding,
    ) -> Result<(), RootError> {
        let valid = match (self, binding) {
            (Self::Journal(next), Binding::Journal { producer, owner_generation, integrity_key_id }) => {
                next.schema_version == 1 && next.sequence > 0 && next.run_generation > 0
                    && next.owner_generation > 0 && hash_valid(&next.record_sha256)
                    && &next.producer == producer && &next.owner_generation == owner_generation
                    && &next.integrity_key_id == integrity_key_id
            }
            (Self::Policy(next), Binding::Policy { owner }) => {
                next.schema_version == 1 && next.revision > 0 && next.owner.validate().is_ok()
                    && &next.owner == owner && hash_valid(&next.state_sha256)
                    && hash_valid(&next.commit_sha256)
            }
            _ => false,
        };
        if !valid { return Err(RootError::Invalid); }
        let sequence = self.sequence();
        if sequence != previous.map_or(Some(1), |old| old.sequence().checked_add(1)).ok_or(RootError::Invalid)? {
            return Err(RootError::Invalid);
        }
        match (self, previous) {
            (Self::Journal(next), Some(Self::Journal(old)))
                if next.policy_generation < old.policy_generation || next.run_generation < old.run_generation => Err(RootError::Invalid),
            (Self::Journal(_), Some(Self::Policy(_))) | (Self::Policy(_), Some(Self::Journal(_))) => Err(RootError::Invalid),
            _ => Ok(()),
        }
    }

    pub(crate) fn sequence(&self) -> u64 {
        match self { Self::Journal(value) => value.sequence, Self::Policy(value) => value.revision }
    }
}
