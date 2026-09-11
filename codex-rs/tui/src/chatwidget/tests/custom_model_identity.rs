use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn custom_model_routes_are_distinguishable_before_selection() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(Some("fixture-model")).await;
    chat.config.model_provider_id = "qa-a".to_string();
    let mut preset = chat.model_catalog.try_list_models().unwrap().remove(0);
    preset.model = "fixture-model".to_string();
    preset.display_name = "fixture-model".to_string();
    preset.description = "Configured runtime provider".to_string();
    preset.supported_reasoning_efforts.clear();
    preset.is_default = false;
    preset.provider_id = Some("qa-a".to_string());
    let mut other = preset.clone();
    other.provider_id = Some("qa-b".to_string());

    chat.open_all_models_popup(vec![preset, other]);
    let popup = render_bottom_popup(&chat, 100);
    assert!(popup.contains("fixture-model via qa-a (current)"));
    assert!(popup.contains("fixture-model via qa-b"));
    assert_eq!(popup.matches("(current)").count(), 1);
    assert_chatwidget_snapshot!("custom_model_provider_identity", popup);

    chat.handle_key_event(KeyEvent::from(KeyCode::Down));
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    let selected = std::iter::from_fn(|| rx.try_recv().ok())
        .find_map(|event| match event {
            AppEvent::OpenReasoningPopup { model, .. } => Some(model),
            _ => None,
        })
        .expect("selecting the labeled route emits its exact provider");
    assert_eq!(selected.model, "fixture-model");
    assert_eq!(selected.provider_id.as_deref(), Some("qa-b"));
    chat.open_reasoning_popup(selected);
    let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
    assert!(events.iter().any(|event| matches!(event,
        AppEvent::UpdateModelSelection { model, provider }
            if model == "fixture-model" && provider.as_deref() == Some("qa-b")
    )));
    assert!(events.iter().any(|event| matches!(event,
        AppEvent::PersistModelSelection { model, provider, .. }
            if model == "fixture-model" && provider.as_deref() == Some("qa-b")
    )));
}
