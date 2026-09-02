use super::*;
use color_eyre::eyre::WrapErr;
use pretty_assertions::assert_eq;
use std::path::Path;

#[test]
fn app_scoped_key_path_quotes_dotted_app_ids() {
    assert_eq!(
        app_scoped_key_path("plugin.linear", "enabled"),
        "apps.\"plugin.linear\".enabled"
    );
}

#[test]
fn trusted_project_edit_targets_project_trust_level() {
    assert_eq!(
        trusted_project_edit(Path::new("/workspace/team.project")),
        ConfigEdit {
            key_path: "projects.\"/workspace/team.project\".trust_level".to_string(),
            value: serde_json::json!("trusted"),
            merge_strategy: MergeStrategy::Replace,
        }
    );
}

#[test]
fn onboarding_provider_selection_persists_provider_and_compatible_model() {
    let edits = build_onboarding_provider_selection_edits(
        Some(codex_model_provider_info::AMBIENT_DEFAULT_MODEL),
        codex_model_provider_info::ANTHROPIC_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![
            ConfigEdit {
                key_path: "model".to_string(),
                value: serde_json::json!(codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL),
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_reasoning_effort".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::ANTHROPIC_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            },
        ]
    );
}

#[test]
fn onboarding_openai_without_model_persists_only_provider() {
    let edits = build_onboarding_provider_selection_edits(
        /*current_model*/ None,
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![ConfigEdit {
            key_path: "model_provider".to_string(),
            value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
            merge_strategy: MergeStrategy::Replace,
        }]
    );
}

#[test]
fn onboarding_openai_clears_fresh_inherited_ambient_model() {
    let edits = build_onboarding_provider_selection_edits(
        Some(codex_model_provider_info::AMBIENT_DEFAULT_MODEL),
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![
            ConfigEdit {
                key_path: "model".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_reasoning_effort".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            },
        ]
    );
}

#[test]
fn onboarding_openai_clears_explicit_claude_model_and_effort() {
    let existing_config = serde_json::json!({
        "model": codex_model_provider_info::ANTHROPIC_DEFAULT_MODEL,
        "model_reasoning_effort": "high",
    });
    let edits = build_onboarding_provider_selection_edits(
        existing_config["model"].as_str(),
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(
        edits,
        vec![
            ConfigEdit {
                key_path: "model".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_reasoning_effort".to_string(),
                value: serde_json::Value::Null,
                merge_strategy: MergeStrategy::Replace,
            },
            ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            },
        ]
    );
}

#[test]
fn onboarding_returning_openai_model_preserves_high_reasoning_effort() {
    let existing_config = serde_json::json!({
        "model": "gpt-5.6-sol",
        "model_reasoning_effort": "high",
    });
    let edits = build_onboarding_provider_selection_edits(
        existing_config["model"].as_str(),
        codex_model_provider_info::OPENAI_PROVIDER_ID,
    );

    assert_eq!(existing_config["model_reasoning_effort"], "high");
    assert_eq!(
        edits,
        vec![ConfigEdit {
            key_path: "model_provider".to_string(),
            value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
            merge_strategy: MergeStrategy::Replace,
        }]
    );
}

#[test]
fn onboarding_openai_preserves_catalog_and_custom_compatible_models() {
    for model in ["codex-auto-fast", "o3"] {
        let existing_config = serde_json::json!({
            "model": model,
            "model_reasoning_effort": "high",
        });
        let edits = build_onboarding_provider_selection_edits(
            existing_config["model"].as_str(),
            codex_model_provider_info::OPENAI_PROVIDER_ID,
        );

        assert_eq!(
            edits,
            vec![ConfigEdit {
                key_path: "model_provider".to_string(),
                value: serde_json::json!(codex_model_provider_info::OPENAI_PROVIDER_ID),
                merge_strategy: MergeStrategy::Replace,
            }],
            "unexpected edits for OpenAI-compatible model {model}"
        );
    }
}

#[test]
fn format_config_error_preserves_server_validation_message() {
    let err = Err::<(), _>(color_eyre::eyre::eyre!(
        "config/batchWrite failed: Invalid configuration: features.fast_mode=true violates \
         managed requirements; allowed set [fast_mode=false]"
    ))
    .wrap_err("config/batchWrite failed in TUI")
    .unwrap_err();

    assert_eq!(
        format_config_error(&err),
        "config/batchWrite failed in TUI: config/batchWrite failed: Invalid configuration: \
         features.fast_mode=true violates managed requirements; allowed set [fast_mode=false]"
    );
}
