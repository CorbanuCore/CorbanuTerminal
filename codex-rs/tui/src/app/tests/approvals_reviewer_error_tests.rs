//! Verify that approvals persistence failures retain the backend's actionable cause.

use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn permission_preset_without_task_persists_reviewer_only_after_selection() -> Result<()> {
    for valid_profile in [false, true] {
        let (mut app, mut events, _op_rx) = make_test_app_with_channels().await;
        let home = tempdir()?;
        app.config.codex_home = home.path().to_path_buf().abs();
        let config_path = home.path().join("config.toml");
        std::fs::write(&config_path, "approvals_reviewer = \"auto_review\"\n")?;
        app.set_approvals_reviewer_in_app_and_widget(ApprovalsReviewer::AutoReview);
        assert_eq!(app.active_thread_id, None);
        let mut server = start_config_write_test_app_server(&app).await?;
        let mut tui = crate::tui::test_support::make_test_tui()?;
        while events.try_recv().is_ok() {}
        let profile_id = if valid_profile {
            codex_protocol::models::BUILT_IN_PERMISSION_PROFILE_READ_ONLY
        } else {
            "missing-profile"
        };
        Box::pin(app.handle_event(
            &mut tui,
            &mut server,
            AppEvent::SelectPermissionPreset(PermissionProfileSelection {
                profile_id: profile_id.to_string(),
                approval_policy: Some(AskForApproval::OnRequest),
                approvals_reviewer: Some(ApprovalsReviewer::User),
                display_label: "Read Only".to_string(),
            }),
        ))
        .await?;
        let mut persisted = false;
        while let Ok(event) = events.try_recv() {
            if let AppEvent::UpdateApprovalsReviewer(reviewer) = event {
                assert_eq!(reviewer, ApprovalsReviewer::User);
                Box::pin(app.handle_event(&mut tui, &mut server, event)).await?;
                persisted = true;
            }
        }
        assert_eq!(persisted, valid_profile);
        let expected = if valid_profile {
            ApprovalsReviewer::User
        } else {
            ApprovalsReviewer::AutoReview
        };
        assert_eq!(app.config.approvals_reviewer, expected);
        assert_eq!(app.chat_widget.config_ref().approvals_reviewer, expected);
        let saved: toml::Value = toml::from_str(&std::fs::read_to_string(config_path)?)?;
        assert_eq!(
            saved["approvals_reviewer"].as_str(),
            Some(expected.to_string().as_str())
        );
        server.shutdown().await?;
    }
    Ok(())
}

#[tokio::test]
async fn approvals_reviewer_error_retains_config_cause() -> Result<()> {
    let (mut app, mut events, _op_rx) = make_test_app_with_channels().await;
    let codex_home = tempdir()?;
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    let config_path = codex_home.path().join("config.toml");
    std::fs::write(&config_path, "")?;
    let mut app_server = start_config_write_test_app_server(&app).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    while events.try_recv().is_ok() {}

    // Simulate an external config edit after startup so the real batch-write
    // reload fails, rather than injecting an already-formatted UI error.
    std::fs::write(&config_path, "approvals_reviewer = [\n")?;
    Box::pin(app.handle_event(
        &mut tui,
        &mut app_server,
        AppEvent::UpdateApprovalsReviewer(ApprovalsReviewer::User),
    ))
    .await?;
    app_server.shutdown().await?;

    let cell = match events.try_recv() {
        Ok(AppEvent::InsertHistoryCell(cell)) => cell,
        other => panic!("expected the persistence error, got {other:?}"),
    };
    let rendered = lines_to_single_string(&cell.display_lines(/*width*/ 120)).replace(
        &config_path.to_string_lossy().to_string(),
        "<CODEX_HOME>/config.toml",
    );
    assert!(rendered.contains("unclosed array"), "{rendered}");
    insta::assert_snapshot!("approvals_reviewer_config_error", rendered);
    Ok(())
}
