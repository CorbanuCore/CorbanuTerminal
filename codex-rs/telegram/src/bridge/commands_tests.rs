use super::MODELS_PER_PAGE;
use super::bounded_button_label;
use super::render_model_picker;
use crate::model_selection::CatalogModel;
use crate::model_selection::ModelPickerCallback;
use crate::model_selection::available_models;
use pretty_assertions::assert_eq;

fn catalog(count: usize) -> Vec<CatalogModel> {
    (0..count)
        .map(|index| CatalogModel {
            id: format!("model-{index}"),
            model: format!("provider/model-{index}"),
            display_name: format!("Model {index}"),
        })
        .collect()
}

#[test]
fn model_picker_marks_active_choice_and_emits_selectable_callbacks() {
    let catalog = catalog(/*count*/ 2);
    let page = render_model_picker(
        Some("provider/model-1"),
        "provider",
        &catalog,
        /*requested_page*/ 0,
    );

    assert!(page.text.contains("Active model: provider/model-1"));
    assert_eq!(page.buttons.len(), available_models(&catalog).len());
    let active = page
        .buttons
        .iter()
        .find(|row| row[0].0 == "✓ Model 1")
        .expect("active model should have a selectable button");
    assert!(matches!(
        ModelPickerCallback::decode(&active[0].1),
        Some(ModelPickerCallback::Select { .. })
    ));
}

#[test]
fn model_picker_pages_are_bounded_and_navigable() {
    let catalog = catalog(MODELS_PER_PAGE + 2);
    let first = render_model_picker(/*active_model*/ None, "provider", &catalog, /*requested_page*/ 0);
    let second = render_model_picker(/*active_model*/ None, "provider", &catalog, /*requested_page*/ 1);
    let available_count = available_models(&catalog).len();

    assert_eq!(first.buttons.len(), MODELS_PER_PAGE + 1);
    assert_eq!(second.buttons.len(), available_count - MODELS_PER_PAGE + 1);
    assert_eq!(first.buttons.last().unwrap()[0].0, "Next →");
    assert_eq!(second.buttons.last().unwrap()[0].0, "← Previous");
    assert!(second.text.contains("page 2/2"));
}

#[test]
fn model_picker_clamps_stale_pages_and_button_labels() {
    let catalog = catalog(MODELS_PER_PAGE + 1);
    let page = render_model_picker(/*active_model*/ None, "provider", &catalog, usize::MAX);

    assert!(page.text.contains("page 2/2"));
    assert_eq!(page.buttons[0][0].0, "Model 8");
    assert_eq!(
        bounded_button_label("✓ ", &"x".repeat(100)).chars().count(),
        60
    );
}
