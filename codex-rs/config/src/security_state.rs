use codex_security_policy::SecurityLevel;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub const AUTHORITATIVE_STATE_SCHEMA_VERSION: u32 = 1;
pub const AUTHORITATIVE_COMMIT_SCHEMA_VERSION: u32 = 1;

/// Controller identity and ownership epoch attached to protected state.
///
/// The target ID is the lower-hex SHA-256 identity validated by the PF-27
/// platform-containment contract. The owner epoch prevents an earlier
/// credential or provenance owner from restoring over a later owner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoritativeStateOwner {
    pub controller_target_id: String,
    pub owner_id: String,
    pub owner_generation: u64,
}

impl AuthoritativeStateOwner {
    pub fn new(
        controller_target_id: impl Into<String>,
        owner_id: impl Into<String>,
        owner_generation: u64,
    ) -> Result<Self, AuthoritativeStateValidationError> {
        let owner = Self {
            controller_target_id: controller_target_id.into(),
            owner_id: owner_id.into(),
            owner_generation,
        };
        owner.validate()?;
        Ok(owner)
    }

    pub fn validate(&self) -> Result<(), AuthoritativeStateValidationError> {
        if !is_lower_hex_sha256(&self.controller_target_id) {
            return Err(AuthoritativeStateValidationError::InvalidControllerTargetId);
        }
        if self.owner_id.is_empty() || self.owner_id.len() > 256 {
            return Err(AuthoritativeStateValidationError::InvalidOwnerId);
        }
        if self.owner_id.chars().any(char::is_control) {
            return Err(AuthoritativeStateValidationError::InvalidOwnerId);
        }
        if self.owner_generation == 0 {
            return Err(AuthoritativeStateValidationError::InvalidOwnerGeneration);
        }
        Ok(())
    }
}

/// Immutable protected-state generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoritativeSecurityState {
    pub schema_version: u32,
    pub revision: u64,
    pub owner: AuthoritativeStateOwner,
    pub level: SecurityLevel,
    pub grant_generation: u64,
    pub revocation_generation: u64,
    pub kill_switch_generation: u64,
    pub kill_switch_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovered_from_revision: Option<u64>,
}

impl AuthoritativeSecurityState {
    pub fn new(
        revision: u64,
        owner: AuthoritativeStateOwner,
        level: SecurityLevel,
        grant_generation: u64,
        revocation_generation: u64,
        kill_switch_generation: u64,
        kill_switch_active: bool,
    ) -> Result<Self, AuthoritativeStateValidationError> {
        let state = Self {
            schema_version: AUTHORITATIVE_STATE_SCHEMA_VERSION,
            revision,
            owner,
            level,
            grant_generation,
            revocation_generation,
            kill_switch_generation,
            kill_switch_active,
            recovered_from_revision: None,
        };
        state.validate()?;
        Ok(state)
    }

    pub fn validate(&self) -> Result<(), AuthoritativeStateValidationError> {
        if self.schema_version != AUTHORITATIVE_STATE_SCHEMA_VERSION {
            return Err(
                AuthoritativeStateValidationError::UnsupportedStateSchemaVersion {
                    found: self.schema_version,
                    supported: AUTHORITATIVE_STATE_SCHEMA_VERSION,
                },
            );
        }
        if self.revision == 0 {
            return Err(AuthoritativeStateValidationError::InvalidRevision);
        }
        self.owner.validate()?;
        if self.kill_switch_generation > self.revocation_generation {
            return Err(AuthoritativeStateValidationError::InvalidKillSwitchGeneration);
        }
        if self
            .recovered_from_revision
            .is_some_and(|revision| revision == 0 || revision >= self.revision)
        {
            return Err(AuthoritativeStateValidationError::InvalidRecoveryRevision);
        }
        Ok(())
    }

    pub fn validate_successor(
        &self,
        previous: &Self,
    ) -> Result<(), AuthoritativeStateValidationError> {
        self.validate()?;
        previous.validate()?;
        let expected_revision = previous
            .revision
            .checked_add(1)
            .ok_or(AuthoritativeStateValidationError::RevisionOverflow)?;
        if self.revision != expected_revision {
            return Err(AuthoritativeStateValidationError::NonSequentialRevision);
        }
        if self.owner.controller_target_id != previous.owner.controller_target_id {
            return Err(AuthoritativeStateValidationError::ControllerIdentityChanged);
        }
        match self
            .owner
            .owner_generation
            .cmp(&previous.owner.owner_generation)
        {
            std::cmp::Ordering::Less => {
                return Err(AuthoritativeStateValidationError::OwnerGenerationRollback);
            }
            std::cmp::Ordering::Equal if self.owner.owner_id != previous.owner.owner_id => {
                return Err(AuthoritativeStateValidationError::OwnerChangedWithoutRotation);
            }
            std::cmp::Ordering::Greater => {
                let expected_generation = previous
                    .owner
                    .owner_generation
                    .checked_add(1)
                    .ok_or(AuthoritativeStateValidationError::OwnerGenerationOverflow)?;
                if self.owner.owner_generation != expected_generation {
                    return Err(AuthoritativeStateValidationError::OwnerGenerationSkipped);
                }
            }
            _ => {}
        }
        if self.grant_generation < previous.grant_generation
            || self.revocation_generation < previous.revocation_generation
            || self.kill_switch_generation < previous.kill_switch_generation
        {
            return Err(AuthoritativeStateValidationError::AuthorityGenerationRollback);
        }
        Ok(())
    }

    /// Creates a forward-only recovery generation from an older snapshot.
    ///
    /// Recovery may restore a stricter level, but never weakens the current
    /// level, clears a kill switch, reduces an authority generation, or changes
    /// ownership. The returned revision is later than the current head.
    pub fn recovered_successor(
        current: &Self,
        snapshot: &Self,
    ) -> Result<Self, AuthoritativeStateValidationError> {
        current.validate()?;
        snapshot.validate()?;
        if snapshot.revision >= current.revision {
            return Err(AuthoritativeStateValidationError::InvalidRecoveryRevision);
        }
        if snapshot.owner != current.owner {
            return Err(AuthoritativeStateValidationError::RecoveryOwnerMismatch);
        }
        let mut recovered = Self {
            schema_version: AUTHORITATIVE_STATE_SCHEMA_VERSION,
            revision: current
                .revision
                .checked_add(1)
                .ok_or(AuthoritativeStateValidationError::RevisionOverflow)?,
            owner: current.owner.clone(),
            level: current.level.max(snapshot.level),
            grant_generation: current.grant_generation.max(snapshot.grant_generation),
            revocation_generation: current
                .revocation_generation
                .max(snapshot.revocation_generation),
            kill_switch_generation: current
                .kill_switch_generation
                .max(snapshot.kill_switch_generation),
            kill_switch_active: current.kill_switch_active || snapshot.kill_switch_active,
            recovered_from_revision: Some(snapshot.revision),
        };
        if recovered.kill_switch_active
            && recovered.kill_switch_generation < recovered.revocation_generation
        {
            recovered.kill_switch_generation = recovered.revocation_generation;
        }
        recovered.validate_successor(current)?;
        Ok(recovered)
    }
}

/// Commit record for one immutable state generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoritativeStateCommit {
    pub schema_version: u32,
    pub revision: u64,
    pub state_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_commit_sha256: Option<String>,
}

impl AuthoritativeStateCommit {
    pub fn new(
        revision: u64,
        state_sha256: impl Into<String>,
        previous_commit_sha256: Option<String>,
    ) -> Result<Self, AuthoritativeStateValidationError> {
        let commit = Self {
            schema_version: AUTHORITATIVE_COMMIT_SCHEMA_VERSION,
            revision,
            state_sha256: state_sha256.into(),
            previous_commit_sha256,
        };
        commit.validate()?;
        Ok(commit)
    }

    pub fn validate(&self) -> Result<(), AuthoritativeStateValidationError> {
        if self.schema_version != AUTHORITATIVE_COMMIT_SCHEMA_VERSION {
            return Err(
                AuthoritativeStateValidationError::UnsupportedCommitSchemaVersion {
                    found: self.schema_version,
                    supported: AUTHORITATIVE_COMMIT_SCHEMA_VERSION,
                },
            );
        }
        if self.revision == 0 {
            return Err(AuthoritativeStateValidationError::InvalidRevision);
        }
        if !is_lower_hex_sha256(&self.state_sha256)
            || self
                .previous_commit_sha256
                .as_deref()
                .is_some_and(|digest| !is_lower_hex_sha256(digest))
        {
            return Err(AuthoritativeStateValidationError::InvalidDigest);
        }
        if (self.revision == 1) != self.previous_commit_sha256.is_none() {
            return Err(AuthoritativeStateValidationError::InvalidCommitChain);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AuthoritativeStateValidationError {
    #[error(
        "unsupported authoritative-state schema version {found}; supported version is {supported}"
    )]
    UnsupportedStateSchemaVersion { found: u32, supported: u32 },
    #[error(
        "unsupported authoritative commit schema version {found}; supported version is {supported}"
    )]
    UnsupportedCommitSchemaVersion { found: u32, supported: u32 },
    #[error("controller target ID must be a lower-hex SHA-256 digest")]
    InvalidControllerTargetId,
    #[error("owner ID must be nonempty, bounded, and free of control characters")]
    InvalidOwnerId,
    #[error("owner generation must be nonzero")]
    InvalidOwnerGeneration,
    #[error("state revision must be nonzero")]
    InvalidRevision,
    #[error("state revision overflow")]
    RevisionOverflow,
    #[error("kill-switch generation cannot exceed revocation generation")]
    InvalidKillSwitchGeneration,
    #[error("recovery revision must name an earlier nonzero generation")]
    InvalidRecoveryRevision,
    #[error("successor revision must advance exactly once")]
    NonSequentialRevision,
    #[error("controller identity cannot change in an authoritative store")]
    ControllerIdentityChanged,
    #[error("owner generation cannot roll back")]
    OwnerGenerationRollback,
    #[error("owner generation overflow")]
    OwnerGenerationOverflow,
    #[error("owner identity changed without rotating its generation")]
    OwnerChangedWithoutRotation,
    #[error("owner rotation must advance exactly one generation")]
    OwnerGenerationSkipped,
    #[error("grant, revocation, and kill-switch generations cannot roll back")]
    AuthorityGenerationRollback,
    #[error("recovery snapshot belongs to another owner generation")]
    RecoveryOwnerMismatch,
    #[error("state or commit digest must be a lower-hex SHA-256 value")]
    InvalidDigest,
    #[error("commit chain predecessor does not match its revision")]
    InvalidCommitChain,
}

fn is_lower_hex_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
