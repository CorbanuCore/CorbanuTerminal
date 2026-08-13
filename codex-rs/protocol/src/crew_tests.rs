use super::*;
use pretty_assertions::assert_eq;

fn valid_crew() -> CrewSpec {
    CrewSpec {
        schema_version: CURRENT_CREW_SCHEMA_VERSION,
        crew_id: "crew-1".to_string(),
        preset_id: Some("standard".to_string()),
        members: vec![
            CrewMemberSpec {
                logical_member_id: "manager".to_string(),
                display_name: "Manager".to_string(),
                role_profile: "manager".to_string(),
                parent_member_id: None,
                runtime_request: RuntimeRequest::exact(
                    "provider-a",
                    "model-a",
                    /*reasoning_effort*/ None,
                ),
            },
            CrewMemberSpec {
                logical_member_id: "worker".to_string(),
                display_name: "Worker".to_string(),
                role_profile: "worker".to_string(),
                parent_member_id: Some("manager".to_string()),
                runtime_request: RuntimeRequest::exact(
                    "provider-b",
                    "model-b",
                    Some(ReasoningEffort::High),
                ),
            },
        ],
        policy: CrewPolicy {
            delegation_mode: DelegationMode::Proactive,
            allow_ephemeral_descendants: true,
            provider_allowlist: vec!["provider-a".to_string(), "provider-b".to_string()],
            maximum_spend_usd: Some(10.0),
        },
    }
}

#[test]
fn valid_crew_round_trips_without_losing_runtime_or_identity() {
    let crew = valid_crew();
    crew.validate().expect("valid crew");
    let encoded = serde_json::to_string(&crew).expect("serialize crew");
    let decoded = serde_json::from_str::<CrewSpec>(&encoded).expect("deserialize crew");

    assert_eq!(decoded, crew);
}

#[test]
fn validation_rejects_ambiguous_identity_topology_and_policy() {
    let mut duplicate = valid_crew();
    duplicate.members[1].logical_member_id = "manager".to_string();
    assert_eq!(
        duplicate.validate(),
        Err(CrewSpecError::DuplicateMemberId {
            member_id: "manager".to_string(),
        })
    );

    let mut missing_parent = valid_crew();
    missing_parent.members[1].parent_member_id = Some("missing".to_string());
    assert_eq!(
        missing_parent.validate(),
        Err(CrewSpecError::ParentMustPrecedeChild {
            member_id: "worker".to_string(),
            parent_id: "missing".to_string(),
        })
    );

    let mut disallowed = valid_crew();
    disallowed.policy.provider_allowlist = vec!["provider-a".to_string()];
    assert_eq!(
        disallowed.validate(),
        Err(CrewSpecError::ProviderNotAllowed {
            member_id: "worker".to_string(),
            provider_id: "provider-b".to_string(),
        })
    );
}
