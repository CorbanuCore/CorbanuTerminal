use super::*;
use codex_config::ConfigLayerEntry;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

#[test]
fn issue_2150_project_role_provenance_reaches_api_layers_and_metadata() {
    let role_file =
        AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("issue-2150-role.toml"))
            .expect("absolute role path");
    let layer = ConfigLayerEntry::new(
        ConfigLayerSource::SessionFlags,
        toml::from_str("model = 'gpt-6-sol'").expect("role config"),
    );
    let flags = serde_json::json!({"type": "sessionFlags"});
    assert_eq!(
        serde_json::to_value(config_layer_to_api(layer.as_layer()).name).unwrap(),
        flags
    );

    let layer = layer.with_project_role_file(role_file.clone());
    let expected = serde_json::json!({"type": "projectRole", "file": role_file});
    assert_eq!(
        serde_json::to_value(config_layer_to_api(layer.as_layer()).name).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::to_value(config_layer_metadata_to_api(layer.metadata()).name).unwrap(),
        expected
    );
    assert_eq!(layer.name, ConfigLayerSource::SessionFlags);
    assert_eq!(
        config_layer_to_api(layer.as_layer()).name.precedence(),
        ApiConfigLayerSource::SessionFlags.precedence()
    );
}
