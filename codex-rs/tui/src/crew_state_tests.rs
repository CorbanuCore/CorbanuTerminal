use super::*;
use crate::crew_presets;
use pretty_assertions::assert_eq;

#[test]
fn crew_instance_round_trip_preserves_logical_to_native_identity() {
    for spec in [
        crew_presets::standard_crew_spec(),
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
