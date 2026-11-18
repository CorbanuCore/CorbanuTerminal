use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn november_models_picker_snapshots() {
    for (slug, provider, snapshot) in [
        ("gpt-6.2-astra", "openai", "november_models_openai_picker"),
        (
            "claude-fable-5-2",
            "anthropic",
            "november_models_anthropic_picker",
        ),
    ] {
        let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some(slug)).await;
        chat.config.model_provider_id = provider.to_string();
        chat.set_model(slug);
        chat.thread_id = Some(ThreadId::new());
        let presets = chat.model_catalog.try_list_models().expect("catalog");
        chat.open_all_models_popup(presets);
        let popup = render_bottom_popup_with_height(&chat, /*width*/ 130, /*height*/ 44);
        assert!(popup.contains(slug), "{popup}");
        insta::assert_snapshot!(snapshot, popup);
        chat.handle_key_event(KeyEvent::from(KeyCode::Esc));
        assert_eq!(chat.current_model(), slug);
    }
}

#[tokio::test]
async fn november_models_catalog_and_fable_effort_snapshots() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("claude-fable-5-2")).await;
    let presets = chat.model_catalog.try_list_models().expect("catalog");
    let models = presets
        .iter()
        .filter(|preset| {
            matches!(
                preset.model.as_str(),
                "gpt-6.2-astra" | "gpt-6.2-sol" | "claude-fable-5-2"
            )
        })
        .map(|preset| {
            (
                &preset.model,
                &preset.provider_id,
                &preset.default_reasoning_effort,
                &preset.supported_reasoning_efforts,
                preset.is_default,
                preset.show_in_picker,
                &preset.service_tiers,
            )
        })
        .collect::<Vec<_>>();
    insta::assert_snapshot!(
        "november_models_catalog",
        serde_json::to_string_pretty(&models).unwrap()
    );
    let fable = presets
        .into_iter()
        .find(|model| model.model == "claude-fable-5-2")
        .expect("Fable 5.2");
    chat.open_reasoning_popup_for_purpose(fable, ModelSelectionPurpose::Session);
    let popup = render_bottom_popup(&chat, /*width*/ 100);
    insta::assert_snapshot!("november_models_fable_effort", popup);
}
