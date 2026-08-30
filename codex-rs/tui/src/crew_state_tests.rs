use super::*;
use crate::crew_presets;
use pretty_assertions::assert_eq;

#[test]
fn crew_instance_round_trip_preserves_logical_to_native_identity() {
    for spec in [
        crew_presets::standard_crew_spec(),
        crew_presets::corbanu_api_crew_spec(),
        crew_presets::multimodel_qualification_crew_spec(),
    ] {
        let mut state = CrewInstanceState::begin(spec).expect("valid crew");
        let members = state
            .spec
            .members
            .iter()
            .map(|member| member.logical_member_id.clone())
            .collect::<Vec<_>>();
        for (index, member_id) in members.iter().enumerate() {
            state
                .record_member(
                    member_id,
                    &format!("thread:00000000-0000-7000-8000-{index:012}"),
                )
                .expect("record member");
        }
        state.mark_ready().expect("complete crew");

        let encoded = serde_json::to_string(&state).expect("serialize state");
        let decoded =
            serde_json::from_str::<CrewInstanceState>(&encoded).expect("deserialize state");
        assert_eq!(decoded, state);
    }
}

#[test]
fn crew_instance_rejects_identity_reassignment_and_incomplete_ready_state() {
    let mut state =
        CrewInstanceState::begin(crew_presets::standard_crew_spec()).expect("valid crew");
    state
        .record_member("nazgul", "thread:nazgul")
        .expect("record nazgul");

    assert_eq!(
        state.record_member("nazgul", "thread:replacement"),
        Err(CrewStateError::MemberAlreadyMapped {
            member_id: "nazgul".to_string(),
            existing_node: "thread:nazgul".to_string(),
            requested_node: "thread:replacement".to_string(),
        })
    );
    assert_eq!(
        state.record_member("troll", "thread:nazgul"),
        Err(CrewStateError::NodeAlreadyMapped {
            node_id: "thread:nazgul".to_string(),
            existing_member: "nazgul".to_string(),
            requested_member: "troll".to_string(),
        })
    );
    assert_eq!(
        state.mark_ready(),
        Err(CrewStateError::MissingMembers {
            missing: vec![
                "orc-1".to_string(),
                "orc-2".to_string(),
                "orc-3".to_string(),
                "troll".to_string(),
            ],
        })
    );
}

#[test]
fn ready_custom_crew_can_add_heterogeneous_members_without_changing_existing_identity() {
    let mut spec = crew_presets::standard_crew_spec();
    spec.preset_id = None;
    spec.members.truncate(1);
    spec.policy.provider_allowlist = vec!["claude-plan".to_string(), "kimi-code".to_string()];
    let mut state = CrewInstanceState::begin(spec).expect("valid root crew");
    state
        .record_member("nazgul", "thread:nazgul")
        .expect("record root");
    state.mark_ready().expect("root ready");

    state
        .add_ready_member(
            codex_protocol::crew::CrewMemberSpec {
                logical_member_id: "orc-1".to_string(),
                display_name: "Kimi reviewer".to_string(),
                role_profile: "orc".to_string(),
                parent_member_id: Some("nazgul".to_string()),
                runtime_request: codex_protocol::crew::RuntimeRequest::exact(
                    "kimi-code",
                    "k3",
                    /*reasoning_effort*/ None,
                ),
            },
            "thread:kimi",
        )
        .expect("add Kimi member");

    assert_eq!(
        state.member_node_by_id.get("nazgul").map(String::as_str),
        Some("thread:nazgul")
    );
    assert_eq!(
        state.member_node_by_id.get("orc-1").map(String::as_str),
        Some("thread:kimi")
    );
    // The operator-authorized ceiling is unchanged by adding a member. A crew
    // policy is authorization, not a record of what was requested.
    assert_eq!(
        state.spec.policy.provider_allowlist,
        vec!["claude-plan".to_string(), "kimi-code".to_string()]
    );
    state.spec.validate().expect("expanded crew remains valid");
    let restored: CrewInstanceState =
        serde_json::from_str(&serde_json::to_string(&state).expect("serialize expanded crew"))
            .expect("restore expanded crew");
    assert_eq!(restored, state);
}

#[test]
fn adding_a_member_cannot_broaden_the_crew_provider_allowlist() {
    let mut spec = crew_presets::standard_crew_spec();
    spec.preset_id = None;
    spec.members.truncate(1);
    spec.policy.provider_allowlist = vec!["claude-plan".to_string()];
    let mut state = CrewInstanceState::begin(spec).expect("valid root crew");
    state
        .record_member("nazgul", "thread:nazgul")
        .expect("record root");
    state.mark_ready().expect("root ready");
    let before = state.clone();

    // Every unauthorized runtime is refused, not one example. A model chooses a
    // runtime; only operator policy authorizes one.
    for (member_id, provider, model, node) in [
        ("orc-1", "kimi-code", "k3", "thread:kimi"),
        ("orc-2", "anthropic", "claude-opus-5", "thread:opus"),
        ("orc-3", "openrouter", "x-ai/grok-4.6", "thread:grok"),
    ] {
        let error = state
            .add_ready_member(
                codex_protocol::crew::CrewMemberSpec {
                    logical_member_id: member_id.to_string(),
                    display_name: format!("{provider} member"),
                    role_profile: "orc".to_string(),
                    parent_member_id: Some("nazgul".to_string()),
                    runtime_request: codex_protocol::crew::RuntimeRequest::exact(
                        provider, model, /*reasoning_effort*/ None,
                    ),
                },
                node,
            )
            .expect_err("unauthorized provider must not join the crew");
        assert!(
            error.to_string().contains(provider),
            "rejection should name {provider}, got: {error}"
        );
    }

    // A refused addition leaves crew identity, membership, and policy untouched.
    assert_eq!(state, before);
}
