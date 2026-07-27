use codex_protocol::crew::CrewMemberSpec;
use codex_protocol::crew::CrewSpec;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CrewInstanceState {
    pub(crate) spec: CrewSpec,
    pub(crate) member_node_by_id: BTreeMap<String, String>,
    pub(crate) status: CrewCreationStatus,
}

impl CrewInstanceState {
    pub(crate) fn begin(mut spec: CrewSpec) -> Result<Self, CrewStateError> {
        spec.validate()
            .map_err(|err| CrewStateError::InvalidSpec(err.to_string()))?;
        spec.crew_id = format!("crew-{}", Uuid::now_v7());
        Ok(Self {
            spec,
            member_node_by_id: BTreeMap::new(),
            status: CrewCreationStatus::Creating,
        })
    }

    pub(crate) fn resume_creation(&mut self) {
        self.status = CrewCreationStatus::Creating;
    }

    pub(crate) fn record_member(
        &mut self,
        logical_member_id: &str,
        node_id: &str,
    ) -> Result<(), CrewStateError> {
        if !self
            .spec
            .members
            .iter()
            .any(|member| member.logical_member_id == logical_member_id)
        {
            return Err(CrewStateError::UnknownMember {
                member_id: logical_member_id.to_string(),
            });
        }
        if let Some(existing_node) = self.member_node_by_id.get(logical_member_id) {
            if existing_node == node_id {
                return Ok(());
            }
            return Err(CrewStateError::MemberAlreadyMapped {
                member_id: logical_member_id.to_string(),
                existing_node: existing_node.clone(),
                requested_node: node_id.to_string(),
            });
        }
        if let Some((existing_member, _)) = self
            .member_node_by_id
            .iter()
            .find(|(_, existing_node)| existing_node.as_str() == node_id)
        {
            return Err(CrewStateError::NodeAlreadyMapped {
                node_id: node_id.to_string(),
                existing_member: existing_member.clone(),
                requested_member: logical_member_id.to_string(),
            });
        }
        self.member_node_by_id
            .insert(logical_member_id.to_string(), node_id.to_string());
        Ok(())
    }

    pub(crate) fn mark_ready(&mut self) -> Result<(), CrewStateError> {
        let expected = self
            .spec
            .members
            .iter()
            .map(|member| member.logical_member_id.as_str())
            .collect::<BTreeSet<_>>();
        let actual = self
            .member_node_by_id
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if expected != actual {
            let missing = expected
                .difference(&actual)
                .map(|member_id| (*member_id).to_string())
                .collect();
            return Err(CrewStateError::MissingMembers { missing });
        }
        self.status = CrewCreationStatus::Ready;
        Ok(())
    }

    pub(crate) fn mark_incomplete(&mut self, error: impl Into<String>) {
        self.status = CrewCreationStatus::Incomplete {
            error: error.into(),
        };
    }

    pub(crate) fn add_ready_member(
        &mut self,
        member: CrewMemberSpec,
        node_id: &str,
    ) -> Result<(), CrewStateError> {
        if !matches!(self.status, CrewCreationStatus::Ready) {
            return Err(CrewStateError::NotReady);
        }
        let mut next = self.clone();
        next.spec.preset_id = None;
        next.spec.members.push(member.clone());
        next.spec
            .validate()
            .map_err(|err| CrewStateError::InvalidSpec(err.to_string()))?;
        next.record_member(&member.logical_member_id, node_id)?;
        next.mark_ready()?;
        *self = next;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum CrewCreationStatus {
    Creating,
    Ready,
    Incomplete { error: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum CrewStateError {
    #[error("invalid crew specification: {0}")]
    InvalidSpec(String),
    #[error("crew specification does not contain member {member_id}")]
    UnknownMember { member_id: String },
    #[error("crew member {member_id} is already mapped to {existing_node}, not {requested_node}")]
    MemberAlreadyMapped {
        member_id: String,
        existing_node: String,
        requested_node: String,
    },
    #[error("native node {node_id} is already mapped to {existing_member}, not {requested_member}")]
    NodeAlreadyMapped {
        node_id: String,
        existing_member: String,
        requested_member: String,
    },
    #[error("crew is missing native nodes for members: {missing:?}")]
    MissingMembers { missing: Vec<String> },
    #[error("crew must be ready before a member is added")]
    NotReady,
}

#[cfg(test)]
#[path = "crew_state_tests.rs"]
mod tests;
