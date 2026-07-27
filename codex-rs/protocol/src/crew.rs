use crate::config_types::ServiceTier;
use crate::openai_models::ReasoningEffort;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use thiserror::Error;
use ts_rs::TS;

pub const CURRENT_CREW_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRef(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTransportKind {
    NativeResponses,
    NativeChatCompletions,
    ExternalPlan,
    OpenAiCompatible,
}

/// A fully resolved runtime persisted with an agent thread.
///
/// `credential_ref` names a vault entry. It never contains credential material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRuntimeSpec {
    pub provider_id: String,
    pub model_id: String,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub service_tier: Option<ServiceTier>,
    pub credential_ref: CredentialRef,
    pub transport: AgentTransportKind,
}

/// The runtime intent stored in a crew definition before credential and transport resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeRequest {
    Exact {
        provider_id: String,
        model_id: String,
        reasoning_effort: Option<ReasoningEffort>,
        service_tier: Option<ServiceTier>,
    },
    Selector {
        allowed_provider_ids: Vec<String>,
        preferred_model_ids: Vec<String>,
        reasoning_effort: Option<ReasoningEffort>,
        service_tier: Option<ServiceTier>,
    },
}

impl RuntimeRequest {
    pub fn exact(
        provider_id: impl Into<String>,
        model_id: impl Into<String>,
        reasoning_effort: Option<ReasoningEffort>,
    ) -> Self {
        Self::Exact {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
            reasoning_effort,
            service_tier: None,
        }
    }

    pub fn exact_parts(&self) -> Option<(&str, &str, Option<ReasoningEffort>)> {
        let Self::Exact {
            provider_id,
            model_id,
            reasoning_effort,
            ..
        } = self
        else {
            return None;
        };
        Some((provider_id, model_id, reasoning_effort.as_ref().cloned()))
    }

    fn validate(&self, member_id: &str) -> Result<(), CrewSpecError> {
        match self {
            Self::Exact {
                provider_id,
                model_id,
                ..
            } => {
                require_nonempty(member_id, "provider_id", provider_id)?;
                require_nonempty(member_id, "model_id", model_id)
            }
            Self::Selector {
                allowed_provider_ids,
                preferred_model_ids,
                ..
            } => {
                if allowed_provider_ids.is_empty() {
                    return Err(CrewSpecError::EmptyRuntimeSelector {
                        member_id: member_id.to_string(),
                    });
                }
                for provider_id in allowed_provider_ids {
                    require_nonempty(member_id, "allowed_provider_ids", provider_id)?;
                }
                for model_id in preferred_model_ids {
                    require_nonempty(member_id, "preferred_model_ids", model_id)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationMode {
    ExplicitOnly,
    Proactive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewPolicy {
    pub delegation_mode: DelegationMode,
    pub allow_ephemeral_descendants: bool,
    pub provider_allowlist: Vec<String>,
    pub maximum_spend_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewMemberSpec {
    pub logical_member_id: String,
    pub display_name: String,
    pub role_profile: String,
    pub parent_member_id: Option<String>,
    pub runtime_request: RuntimeRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrewSpec {
    pub schema_version: u32,
    pub crew_id: String,
    pub preset_id: Option<String>,
    pub members: Vec<CrewMemberSpec>,
    pub policy: CrewPolicy,
}

impl CrewSpec {
    pub fn validate(&self) -> Result<(), CrewSpecError> {
        if self.schema_version != CURRENT_CREW_SCHEMA_VERSION {
            return Err(CrewSpecError::UnsupportedSchemaVersion {
                actual: self.schema_version,
                supported: CURRENT_CREW_SCHEMA_VERSION,
            });
        }
        if self.crew_id.trim().is_empty() {
            return Err(CrewSpecError::EmptyCrewId);
        }
        if self.members.is_empty() {
            return Err(CrewSpecError::EmptyCrew);
        }

        let allowlist = self
            .policy
            .provider_allowlist
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let mut known_members = HashSet::new();
        let mut root_count = 0;
        for member in &self.members {
            let member_id = member.logical_member_id.trim();
            if member_id.is_empty() {
                return Err(CrewSpecError::EmptyMemberId);
            }
            if !known_members.insert(member_id) {
                return Err(CrewSpecError::DuplicateMemberId {
                    member_id: member_id.to_string(),
                });
            }
            require_nonempty(member_id, "display_name", &member.display_name)?;
            require_nonempty(member_id, "role_profile", &member.role_profile)?;
            if let Some(parent_id) = member.parent_member_id.as_deref() {
                if !known_members.contains(parent_id) {
                    return Err(CrewSpecError::ParentMustPrecedeChild {
                        member_id: member_id.to_string(),
                        parent_id: parent_id.to_string(),
                    });
                }
            } else {
                root_count += 1;
            }
            member.runtime_request.validate(member_id)?;
            if !allowlist.is_empty()
                && let Some((provider_id, _, _)) = member.runtime_request.exact_parts()
                && !allowlist.contains(provider_id)
            {
                return Err(CrewSpecError::ProviderNotAllowed {
                    member_id: member_id.to_string(),
                    provider_id: provider_id.to_string(),
                });
            }
        }
        if root_count != 1 {
            return Err(CrewSpecError::RootCount { actual: root_count });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "class", rename_all = "snake_case")]
#[ts(tag = "class", rename_all = "snake_case", export_to = "v2/")]
pub enum AgentClass {
    CrewMember {
        crew_id: String,
        logical_member_id: String,
        human_addressable: bool,
    },
    EphemeralTask {
        assignment_id: String,
        retention: RetentionPolicy,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export_to = "v2/")]
pub enum RetentionPolicy {
    CloseOnCompletion,
    UnloadOnCompletion,
    Retain,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CrewSpecError {
    #[error("crew schema version {actual} is unsupported; expected {supported}")]
    UnsupportedSchemaVersion { actual: u32, supported: u32 },
    #[error("crew_id must not be empty")]
    EmptyCrewId,
    #[error("crew must contain at least one member")]
    EmptyCrew,
    #[error("logical_member_id must not be empty")]
    EmptyMemberId,
    #[error("crew member {member_id} is duplicated")]
    DuplicateMemberId { member_id: String },
    #[error("crew member {member_id} has an empty {field}")]
    EmptyMemberField {
        member_id: String,
        field: &'static str,
    },
    #[error("crew member {member_id} references parent {parent_id} before it exists")]
    ParentMustPrecedeChild {
        member_id: String,
        parent_id: String,
    },
    #[error("crew must have exactly one root member; found {actual}")]
    RootCount { actual: usize },
    #[error("crew member {member_id} has an empty runtime selector")]
    EmptyRuntimeSelector { member_id: String },
    #[error("provider {provider_id} for crew member {member_id} is not allowed by crew policy")]
    ProviderNotAllowed {
        member_id: String,
        provider_id: String,
    },
}

fn require_nonempty(
    member_id: &str,
    field: &'static str,
    value: &str,
) -> Result<(), CrewSpecError> {
    if value.trim().is_empty() {
        return Err(CrewSpecError::EmptyMemberField {
            member_id: member_id.to_string(),
            field,
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "crew_tests.rs"]
mod tests;
