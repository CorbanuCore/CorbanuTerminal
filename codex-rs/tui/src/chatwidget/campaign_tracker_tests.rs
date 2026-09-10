use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn campaign_tracker_overview_shows_recording_state_and_recovery() {
    let (mut chat, _, _, _) = crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.campaign_tracker_result(
        "/status".to_string(),
        None,
        None,
        Ok(json!({"handle":"alice","usage":{"events":12}})),
    );
    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
    insta::assert_snapshot!("campaign_tracker_overview", rendered);
}
#[tokio::test]
async fn campaign_tracker_refresh_replaces_view_instead_of_growing_the_stack() {
    let (mut chat, _, _, _) = crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    for _ in 0..4 {
        chat.campaign_tracker_result(
            "/status".to_string(),
            None,
            None,
            Ok(json!({"handle":"alice","usage":{"events":0}})),
        );
    }
    chat.bottom_pane
        .handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        ));
    assert_ne!(chat.bottom_pane.active_view_id(), Some(VIEW));
}
#[tokio::test]
async fn campaign_tracker_summary_only_replay_does_not_offer_prompt_review() {
    let (mut chat, _, _, _) = crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.campaign_tracker_result("/replay".to_string(),None,None,Ok(json!({"items":[{"id":"event","kind":"human_prompt","accountId":"alice","sessionId":"thread","handleAtExecution":"alice","content":"","promptWithheld":true,"capabilities":["summary","replay"],"repository":{"label":"repo"},"goal":{"active":false}}]})));
    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
    assert!(rendered.contains("Prompt access not granted"));
    assert!(!rendered.contains("Review this prompt"));
}
#[tokio::test]
async fn campaign_tracker_failed_request_preserves_a_retry_path() {
    let (mut chat, _, _, _) = crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.campaign_tracker_result(
        "/activity".to_string(),
        None,
        None,
        Err("Tracker service unavailable".to_string()),
    );
    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
    assert!(rendered.contains("Retry"));
    assert!(rendered.contains("Tracker service unavailable"));
    assert_eq!(chat.bottom_pane.active_view_id(), Some(VIEW));
}

#[tokio::test]
async fn campaign_tracker_read_retry_preserves_route_and_search_scope() {
    let (mut chat, _, mut rx, _) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    let path = "/activity?accountId=alice&taskId=task-one&search=retry%20queue";
    chat.campaign_tracker_result(
        path.to_string(),
        None,
        None,
        Err("Activity search exceeds 500 UTF-8 bytes.".to_string()),
    );
    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
    insta::assert_snapshot!("campaign_tracker_read_error_recovery", rendered);
    chat.handle_key_event(crossterm::event::KeyCode::Enter.into());
    let request = std::iter::from_fn(|| rx.try_recv().ok()).find_map(|event| match event {
        AppEvent::CampaignTrackerOpen { path, body } => Some((path, body)),
        _ => None,
    });
    assert_eq!(request, Some((path.to_string(), None)));
}

#[tokio::test]
async fn campaign_tracker_failed_forms_restore_drafts_without_repeating_writes() {
    let cases = [
        (
            "/campaigns",
            json!({"title":"Recovery campaign","objective":"Preserve the corrected objective"}),
            "/_new_campaign",
            "Recovery campaign",
            "Preserve the corrected objective",
        ),
        (
            "/annotations",
            json!({"id":"event-one","revision":2,"kind":"mapping","taskIds":["task-one"],"note":"Evidence for the correction"}),
            "/_mapping",
            "task-one",
            "Evidence for the correction",
        ),
        (
            "/annotations",
            json!({"id":"event-one","revision":2,"kind":"review","scores":{"clarity":3,"context":4,"constraints":2,"successCriteria":4,"iteration":null},"note":"Existing review rationale"}),
            "/_review",
            "Prompt review rationale",
            "Existing review rationale",
        ),
    ];
    for (route, body, editor, title, retained) in cases {
        let (mut chat, _, mut rx, _) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        chat.campaign_tracker_result(
            route.to_string(),
            None,
            Some(body.clone()),
            Err(if route == "/campaigns" {
                "Campaign name exceeds 200 UTF-8 bytes. Shorten it and submit again."
            } else {
                "Rationale is required. Enter a value and submit again."
            }
            .to_string()),
        );
        let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
        assert!(rendered.contains("Edit draft"));
        assert!(!rendered.contains("Retry"));
        if route == "/campaigns" {
            insta::assert_snapshot!("campaign_tracker_write_error_recovery", rendered);
        }
        chat.handle_key_event(crossterm::event::KeyCode::Enter.into());
        let (path, draft) = std::iter::from_fn(|| rx.try_recv().ok())
            .find_map(|event| match event {
                AppEvent::CampaignTrackerOpen { path, body } => Some((path, body)),
                _ => None,
            })
            .expect("local draft recovery event");
        assert_eq!((&path, &draft), (&editor.to_string(), &Some(body)));
        chat.open_campaign_tracker(path, draft);
        let form = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 110);
        assert!(form.contains(title), "{form}");
        assert!(form.contains(retained), "{form}");
        chat.handle_key_event(crossterm::event::KeyCode::Esc.into());
        assert!(
            std::iter::from_fn(|| rx.try_recv().ok())
                .all(|event| !matches!(event, AppEvent::CampaignTrackerOpen { .. }))
        );
    }
}

#[tokio::test]
async fn campaign_tracker_enrollment_retry_does_not_retain_a_credential() {
    let (mut chat, _, mut rx, _) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.campaign_tracker_result(
        "/enrollment".to_string(),
        Some(true),
        Some(
            json!({"workspaceId":"workspace-one","enabled":true,"apiKey":"never-retain-this-key"}),
        ),
        Err("Link Corbanu API in Providers, then retry.".to_string()),
    );
    chat.handle_key_event(crossterm::event::KeyCode::Enter.into());
    let request = std::iter::from_fn(|| rx.try_recv().ok()).find_map(|event| match event {
        AppEvent::CampaignTrackerOpen { path, body } => Some((path, body)),
        _ => None,
    });
    assert_eq!(
        request,
        Some((
            "/enrollment".to_string(),
            Some(json!({"workspaceId":"workspace-one","enabled":true}))
        ))
    );
}
