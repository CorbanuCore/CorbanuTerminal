use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn report() -> TeamContext {
    serde_json::from_value(json!({
        "status": "current", "generatedAt": "2026-09-11T00:00:00Z",
        "includeInPersonalContext": false, "overview": "The team improved delivery reliability.",
        "members": [
            {"displayName": "Alice", "hiveHandle": "alice", "taskHistoryVisible": true,
             "tasksPastDay": 2, "tasksPastWeek": 5, "focus": "Reliable task delivery",
             "completedChanges": ["Added retry coverage.", "Shipped the worker fix."],
             "operationalEffect": "Fewer stalled requests."},
            {"displayName": "Bob", "taskHistoryVisible": false,
             "tasksPastDay": null, "tasksPastWeek": null,
             "recentWork": "unshared details must never render"}
        ]
    }))
    .unwrap()
}

#[test]
fn shared_report_includes_work_and_preserves_unshared_counts() {
    let text = team_context_text(&report());
    assert!(!text.contains("unshared details"));
    insta::assert_snapshot!("team_context_report", text);
}

#[test]
fn states_keep_previous_report_explicit_and_do_not_invent_zero_counts() {
    let mut report = report();
    report.status = "pending".to_string();
    report.showing_previous_report = true;
    report.members[0].tasks_past_day = None;
    let text = team_context_text(&report);
    assert!(text.contains("Showing the previous report"));
    assert!(text.contains("unavailable in 24 hours"));
    for (state, expected) in [
        ("empty", "No teammates"),
        ("failed", "generation failed"),
        ("unavailable", "currently unavailable"),
    ] {
        report.status = state.to_string();
        report.members.clear();
        assert!(team_context_text(&report).contains(expected));
    }
    assert!(parse_tasknode_value::<TeamContext>(Ok(json!({})), "Team Context").is_err());
}

#[tokio::test]
async fn report_menu_and_failure_offer_refresh() {
    let (mut chat, _, _, _) = crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.show_selection_view(team_context_params(Ok(report())));
    insta::assert_snapshot!(
        "team_context_menu",
        crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 84)
    );
    chat.show_or_replace_tasknode_selection(VIEW_ID, || {
        team_context_params(Err("Service temporarily unavailable".to_string()))
    });
    let screen = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 84);
    insta::assert_snapshot!("team_context_failure", screen);
    assert!(!screen.contains("Read full report"));
    assert_eq!(chat.bottom_pane.active_view_id(), Some(VIEW_ID));
    assert!(!chat.tasknode_response_is_current(
        &codex_tasknode_session::SessionScope::for_profile("other-account"),
        None,
        Some(VIEW_ID)
    ));
}
