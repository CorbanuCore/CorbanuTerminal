use super::*;
use codex_protocol_test_support::ContentItemTestExt;
use pretty_assertions::assert_eq;

const CATALOG: &str = "\n## Available skills\n- reconnect-skill: Read the reconnect fixture.\n";

#[test]
fn executor_catalog_reconnect_respects_provider_context_mode() {
    for mode in [
        WorldStateContextMode::AppendOnly,
        WorldStateContextMode::ReplaceSections,
    ] {
        let available = executor_skills_world_state_section(
            Some(CATALOG.to_string()),
            /*include_instructions*/ true,
            mode,
            /*previous_snapshot*/ None,
            Box::new(|| {}),
        );
        let (available_snapshot, available_fragment) =
            available.render_diff(PreviousWorldStateSection::Absent);
        let available_snapshot = available_snapshot.expect("available snapshot");
        let available_fragment = available_fragment.expect("available catalog");
        let unavailable = executor_skills_world_state_section(
            /*body*/ None,
            /*include_instructions*/ true,
            mode,
            Some(&available_snapshot),
            Box::new(|| {}),
        );
        let (unavailable_snapshot, unavailable_fragment) =
            unavailable.render_diff(PreviousWorldStateSection::Known(&available_snapshot));
        let unavailable_snapshot = unavailable_snapshot.expect("unavailable snapshot");
        let unavailable_text = unavailable_fragment
            .expect("unavailable notice")
            .body()
            .clone()
            .expect_text();
        assert!(unavailable_text.ends_with(NO_EXECUTOR_SKILLS_BODY));
        assert_eq!(
            unavailable_text.contains(CATALOG),
            mode == WorldStateContextMode::ReplaceSections,
        );
        assert!(unavailable_snapshot["body"].is_null());
        assert!(
            unavailable
                .render_diff(PreviousWorldStateSection::Known(&unavailable_snapshot))
                .1
                .is_none()
        );

        let restored = executor_skills_world_state_section(
            Some(CATALOG.to_string()),
            /*include_instructions*/ true,
            mode,
            Some(&unavailable_snapshot),
            Box::new(|| {}),
        );
        let (restored_snapshot, restored_fragment) =
            restored.render_diff(PreviousWorldStateSection::Known(&unavailable_snapshot));
        assert_eq!(restored_snapshot.as_ref(), Some(&available_snapshot));
        match mode {
            WorldStateContextMode::AppendOnly => assert_eq!(
                restored_fragment
                    .expect("availability update")
                    .body()
                    .clone()
                    .expect_text(),
                "\n## Skills update\nThe previously listed selected-environment skills are available again.\n",
            ),
            WorldStateContextMode::ReplaceSections => {
                assert_eq!(restored_fragment, Some(available_fragment.clone()));
            }
        }
        assert!(restored.matches_retained_fragment("developer", CATALOG));
        assert!(!restored.matches_retained_fragment("developer", NO_EXECUTOR_SKILLS_BODY));
        assert_eq!(
            restored.render_diff(PreviousWorldStateSection::Absent).1,
            Some(available_fragment),
        );
        assert!(
            restored
                .render_diff(PreviousWorldStateSection::Known(&available_snapshot))
                .1
                .is_none()
        );

        let hidden = executor_skills_world_state_section(
            /*body*/ None,
            /*include_instructions*/ false,
            mode,
            Some(&available_snapshot),
            Box::new(|| {}),
        );
        assert_eq!(
            hidden
                .render_diff(PreviousWorldStateSection::Known(&available_snapshot))
                .1
                .expect("hidden catalog notice")
                .body()
                .clone()
                .expect_text(),
            HIDDEN_EXECUTOR_SKILLS_BODY,
        );
    }
}
