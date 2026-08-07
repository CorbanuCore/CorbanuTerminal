use super::*;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn default_search_text_uses_model_visible_namespace_metadata_once() {
    let mut schedule_schema = JsonSchema::object(
        BTreeMap::from([(
            "timezone".to_string(),
            JsonSchema::string(Some("IANA timezone.".to_string())),
        )]),
        /*required*/ None,
        /*additional_properties*/ None,
    );
    schedule_schema.description = Some("Schedule settings.".to_string());
    let mut parameters = JsonSchema::object(
        BTreeMap::from([
            (
                "mode".to_string(),
                JsonSchema::string(Some("Update mode.".to_string())),
            ),
            ("schedule".to_string(), schedule_schema),
        ]),
        /*required*/ None,
        /*additional_properties*/ None,
    );
    parameters.description = Some("Automation options.".to_string());
    let spec = ToolSpec::Namespace(crate::ResponsesApiNamespace {
        name: "codex_app".to_string(),
        description: "Manage Codex automations.".to_string(),
        tools: vec![ResponsesApiNamespaceTool::Function(ResponsesApiTool {
            name: "automation_update".to_string(),
            description: "Create or update automations.".to_string(),
            strict: false,
            defer_loading: None,
            parameters,
            output_schema: None,
        })],
    });

    let search_info = ToolSearchInfo::from_tool_spec(spec, /*source_info*/ None)
        .expect("namespace should be searchable");

    assert_eq!(
        search_info.entry.search_text,
        "codex_app Manage Codex automations. automation_update automation update Create or update automations. Automation options. mode Update mode. schedule Schedule settings. timezone IANA timezone."
    );
}

#[test]
fn custom_search_text_is_augmented_with_spec_metadata() {
    let spec = ToolSpec::Namespace(crate::ResponsesApiNamespace {
        name: "multi_agent_v1".to_string(),
        description: "Spawn and manage sub-agents.".to_string(),
        tools: vec![ResponsesApiNamespaceTool::Function(ResponsesApiTool {
            name: "spawn_agent".to_string(),
            description: "Delegate work to a sub-agent.".to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                BTreeMap::from([(
                    "agent_type".to_string(),
                    JsonSchema::string(Some(
                        "custom: Custom role with locked model settings.".to_string(),
                    )),
                )]),
                /*required*/ None,
                /*additional_properties*/ None,
            ),
            output_schema: None,
        })],
    });

    let search_info = ToolSearchInfo::from_spec(
        "spawn agent delegate".to_string(),
        spec,
        /*source_info*/ None,
    )
    .expect("namespace should be searchable");

    assert!(
        search_info
            .entry
            .search_text
            .contains("spawn agent delegate")
    );
    assert!(search_info.entry.search_text.contains("multi_agent_v1"));
    assert!(search_info.entry.search_text.contains("agent_type"));
    assert!(
        search_info
            .entry
            .search_text
            .contains("custom: Custom role")
    );
}

#[test]
fn custom_search_text_does_not_repeat_normalized_spec_fragments() {
    let spec = ToolSpec::Function(ResponsesApiTool {
        name: "create_event".to_string(),
        description: "Create a calendar event.".to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            BTreeMap::from([(
                "start_time".to_string(),
                JsonSchema::string(/*description*/ None),
            )]),
            /*required*/ None,
            /*additional_properties*/ None,
        ),
        output_schema: None,
    });

    let search_info = ToolSearchInfo::from_spec(
        "create-event Create a calendar event. start time".to_string(),
        spec,
        /*source_info*/ None,
    )
    .expect("function should be searchable");

    assert_eq!(
        search_info.entry.search_text,
        "create-event Create a calendar event. start time"
    );
}
