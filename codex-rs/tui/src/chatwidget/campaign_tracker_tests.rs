use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn campaign_tracker_overview_shows_recording_state_and_recovery() {
    let (mut chat, _, _, _) = crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.campaign_tracker_result(
        "/status".to_string(),
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
    chat.campaign_tracker_result("/replay".to_string(),None,Ok(json!({"items":[{"id":"event","kind":"human_prompt","accountId":"alice","sessionId":"thread","handleAtExecution":"alice","content":"","promptWithheld":true,"capabilities":["summary","replay"],"repository":{"label":"repo"},"goal":{"active":false}}]})));
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
        Err("Tracker service unavailable".to_string()),
    );
    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
    assert!(rendered.contains("Retry"));
    assert!(rendered.contains("Tracker service unavailable"));
    assert_eq!(chat.bottom_pane.active_view_id(), Some(VIEW));
}
