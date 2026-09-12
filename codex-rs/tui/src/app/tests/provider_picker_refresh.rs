use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn provider_manager_late_refresh_updates_picker_without_reopening_dismissed_menu()
-> Result<()> {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let app_server = start_config_write_test_app_server(&app).await?;
    let host = crate::provider_status_host::ProviderStatusHost::from_config(
        &app.config,
        crate::provider_status_host::ProviderAccountMetadata::default(),
    );
    app.model_catalog.set_provider_policy(
        crate::chatwidget::provider_model_policy::ProviderModelPolicy::new(
            host.clone(),
            codex_provider_auth::ProviderRuntimeAuthorizations::default(),
        ),
    );
    app.provider_management_generation = 1;
    app.provider_manager_statuses_resolved(
        1,
        host.clone(),
        host.resolve().entries().to_vec(),
        &app_server,
    );
    assert!(app.chat_widget.provider_manager_selected_index().is_some());
    app.chat_widget
        .handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            KeyModifiers::NONE,
        ));
    assert_eq!(app.chat_widget.provider_manager_selected_index(), None);
    assert!(
        !app.model_catalog
            .provider_is_selectable("openai", "gpt-6-astra")
    );
    host.update_account_metadata(crate::provider_status_host::ProviderAccountMetadata {
        openai: codex_login::OpenAiAuthMetadata::Account,
        ..Default::default()
    });
    let recovered = host.resolve().entries().to_vec();
    app.provider_manager_statuses_resolved(2, host.clone(), recovered.clone(), &app_server);
    assert!(
        !app.model_catalog
            .provider_is_selectable("openai", "gpt-6-astra")
    );
    app.provider_manager_statuses_resolved(1, host, recovered, &app_server);
    assert!(
        app.model_catalog
            .provider_is_selectable("openai", "gpt-6-astra")
    );
    assert_eq!(app.chat_widget.provider_manager_selected_index(), None);
    Ok(())
}
