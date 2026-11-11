use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn restored_openai_appears_in_already_open_model_picker() -> Result<()> {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let app_server = start_config_write_test_app_server(&app).await?;
    let metadata = crate::provider_status_host::ProviderAccountMetadata {
        claude: codex_provider_auth::ClaudeCredentialMetadata::Configured {
            source: codex_provider_auth::ClaudeCredentialSource::Managed,
        },
        ..Default::default()
    };
    let host = crate::provider_status_host::ProviderStatusHost::from_config(&app.config, metadata);
    app.model_catalog.set_provider_policy(
        crate::chatwidget::provider_model_policy::ProviderModelPolicy::new(
            host.clone(),
            codex_provider_auth::ProviderRuntimeAuthorizations::default(),
        ),
    );
    app.model_catalog.sync_runtime_models(
        app.config.model_providers.keys().map(String::as_str),
        app.config.model.as_deref(),
        &app.config.model_provider_id,
    );
    app.provider_management_generation = 1;
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
        &app_server,
    );
    app.chat_widget
        .handle_key_event(KeyEvent::from(KeyCode::Esc));
    let selection = (
        app.chat_widget.current_model().to_string(),
        app.chat_widget.config_ref().model_provider_id.clone(),
    );
    app.chat_widget
        .open_all_models_popup(app.model_catalog.try_list_models().unwrap());
    let before = render_bottom_popup(&app.chat_widget, /*width*/ 120);
    assert!(!before.contains("OpenAI"), "{before}");
    assert!(before.contains("claude"), "{before}");

    // The operator restores OpenAI, but opens /model before discovery finishes.
    host.mark_openai_account();
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
        &app_server,
    );
    let restored = render_bottom_popup(&app.chat_widget, /*width*/ 120);
    assert!(restored.contains("OpenAI"), "{restored}");
    assert!(restored.contains("Claude"), "{restored}");
    assert_eq!(
        (
            app.chat_widget.current_model().to_string(),
            app.chat_widget.config_ref().model_provider_id.clone(),
        ),
        selection,
    );
    insta::assert_snapshot!("model_picker_openai_restored_while_open", restored);

    // Adding a provider before the original group must retain the highlighted
    // model, not transfer its numeric row index to OpenAI.
    while rx.try_recv().is_ok() {}
    app.chat_widget
        .handle_key_event(KeyEvent::from(KeyCode::Enter));
    let AppEvent::OpenReasoningPopup { model, .. } = rx.try_recv().unwrap() else {
        panic!("expected model selection");
    };
    assert_eq!(model.provider_id.as_deref(), Some("claude-plan"));
    app.chat_widget.open_reasoning_popup(model);
    let reasoning = render_bottom_popup(&app.chat_widget, /*width*/ 120);
    host.update_account_metadata(metadata);
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
        &app_server,
    );
    assert_eq!(
        render_bottom_popup(&app.chat_widget, /*width*/ 120),
        reasoning
    );
    app.chat_widget
        .handle_key_event(KeyEvent::from(KeyCode::Esc));
    let returned = render_bottom_popup(&app.chat_widget, /*width*/ 120);
    assert!(!returned.contains("OpenAI"), "{returned}");
    assert!(returned.contains("claude"), "{returned}");

    host.mark_openai_account();
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
        &app_server,
    );

    // Refresh again on a different tab: keep the tab and do not stack another popup.
    app.chat_widget
        .handle_key_event(KeyEvent::from(KeyCode::Right));
    let on_claude = render_bottom_popup(&app.chat_widget, /*width*/ 120);
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
        &app_server,
    );
    assert_eq!(
        render_bottom_popup(&app.chat_widget, /*width*/ 120),
        on_claude
    );

    // Removing OpenAI withdraws it from the open picker, too.
    host.update_account_metadata(metadata);
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
        &app_server,
    );
    let removed = render_bottom_popup(&app.chat_widget, /*width*/ 120);
    assert!(!removed.contains("OpenAI"), "{removed}");
    assert!(removed.contains("claude"), "{removed}");
    app.chat_widget
        .handle_key_event(KeyEvent::from(KeyCode::Esc));
    assert!(!render_bottom_popup(&app.chat_widget, /*width*/ 120).contains("Select Model"));
    Ok(())
}

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
        /*generation*/ 1,
        host.clone(),
        host.resolve().entries().to_vec(),
        /*accounts*/ None,
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
    app.provider_manager_statuses_resolved(
        /*generation*/ 2,
        host.clone(),
        recovered.clone(),
        /*accounts*/ None,
        &app_server,
    );
    assert!(
        !app.model_catalog
            .provider_is_selectable("openai", "gpt-6-astra")
    );
    app.provider_manager_statuses_resolved(
        /*generation*/ 1,
        host,
        recovered,
        /*accounts*/ None,
        &app_server,
    );
    assert!(
        app.model_catalog
            .provider_is_selectable("openai", "gpt-6-astra")
    );
    assert_eq!(app.chat_widget.provider_manager_selected_index(), None);
    Ok(())
}
