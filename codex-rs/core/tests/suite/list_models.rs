#![allow(clippy::unwrap_used)]

use anyhow::Result;
use codex_features::Feature;
use codex_models_manager::manager::RefreshStrategy;
use codex_protocol::protocol::MultiAgentVersion;
use core_test_support::responses::*;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use test_case::test_case;

#[test_case(None, false; "all configured providers")]
#[test_case(Some(vec!["openai".to_string()]), false; "explicit openai-only policy")]
#[test_case(Some(Vec::new()), false; "explicit deny-all policy")]
#[test_case(None, true; "hidden models and unconfigured providers")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_models_discovers_full_authorized_catalog(
    allowlist: Option<Vec<String>>,
    restrict_catalog: bool,
) -> Result<()> {
    let server = start_mock_server().await;
    let policy = allowlist.clone();
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_model_info_override("gpt-5.6-luna", |model| {
            model.multi_agent_version = Some(MultiAgentVersion::V1);
        })
        .with_model_info_override("gpt-6.1-sol", move |model| {
            if restrict_catalog {
                model.visibility = codex_protocol::openai_models::ModelVisibility::Hide;
            }
        })
        .with_config(move |config| {
            if restrict_catalog {
                config.model_providers.remove("zai");
            }
            config.features.enable(Feature::MultiAgentV2).unwrap();
            config.agent_provider_allowlist = policy;
            config.multi_agent_v2.expose_spawn_agent_model_overrides = true;
            config.multi_agent_v2.hide_spawn_agent_metadata = restrict_catalog;
        })
        .build_with_auto_env(&server)
        .await?;
    let manager = test.thread_manager.get_models_manager();
    let expected = manager
        .list_models(RefreshStrategy::Offline, test.config.http_client_factory())
        .await
        .into_iter()
        .filter(|model| {
            model.show_in_picker
                && model.provider_id.as_ref().is_some_and(|provider| {
                    test.config.model_providers.contains_key(provider)
                        && allowlist.as_ref().is_none_or(|ids| ids.contains(provider))
                })
        })
        .collect::<Vec<_>>();
    if allowlist.is_none() {
        assert!(expected.len() > 32, "exercise the old description cap");
        for slug in ["minimax/minimax-m3", "gpt-5.6-luna", "zai/glm-5.2-fast"] {
            assert!(
                expected.iter().any(|model| model.model == slug),
                "missing {slug}"
            );
        }
    }
    let mut actual = Vec::new();
    let mut offset = 0;
    loop {
        let page = lookup_page(&test, &server, (offset > 0).then_some(offset)).await?;
        assert!(page.get("oversized_entries").is_none());
        assert_eq!(page["total"], json!(expected.len()));
        let rows = page["models"].as_array().expect("model rows");
        assert!(rows.len() <= 5);
        actual.extend(rows.iter().cloned());
        match page["next_offset"].as_u64() {
            Some(next) => {
                assert!(next as usize > offset, "pagination must advance");
                offset = next as usize;
            }
            None => break,
        }
    }
    assert_eq!(
        actual
            .iter()
            .map(|row| (
                row["model_provider"].as_str().unwrap(),
                row["model"].as_str().unwrap(),
                row["display_name"].as_str().unwrap(),
                row["reasoning_efforts"].clone(),
                row["default_reasoning_effort"].clone(),
            ))
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|model| (
                model.provider_id.as_deref().unwrap(),
                model.model.as_str(),
                model.display_name.as_str(),
                json!(
                    model
                        .supported_reasoning_efforts
                        .iter()
                        .map(|preset| &preset.effort)
                        .collect::<Vec<_>>()
                ),
                json!(model.default_reasoning_effort),
            ))
            .collect::<Vec<_>>(),
    );
    if let Some(astra) = actual.iter().find(|row| row["model"] == "gpt-6-astra") {
        assert_eq!(astra["explicit_choice_only"], true);
    }
    if restrict_catalog {
        assert!(
            actual
                .iter()
                .all(|row| row["model"] != "gpt-6.1-sol" && row["model_provider"] != "zai")
        );
    }
    for (row, model) in actual.iter().zip(&expected) {
        assert_eq!(row["display_name_truncated"], false);
        if restrict_catalog || model.service_tiers.is_empty() {
            assert!(row.get("service_tiers").is_none());
        } else {
            assert_eq!(
                row["service_tiers"],
                json!(
                    model
                        .service_tiers
                        .iter()
                        .map(|tier| &tier.id)
                        .collect::<Vec<_>>()
                )
            );
        }
        let mut config = test.config.clone();
        config.model = Some(model.model.clone());
        config.model_provider_id = model.provider_id.clone().unwrap();
        config.model_provider = config.model_providers[&config.model_provider_id].clone();
        config.model_context_window = None;
        let info = manager
            .get_model_info(&model.model, &config.to_models_manager_config())
            .await;
        assert_eq!(
            row["context_window"],
            json!(
                (!info.used_fallback_model_metadata)
                    .then(|| info.resolved_context_window())
                    .flatten()
            )
        );
    }
    Ok(())
}

async fn lookup_page(
    test: &core_test_support::test_codex::TestCodex,
    server: &wiremock::MockServer,
    offset: Option<usize>,
) -> Result<Value> {
    let call_id = format!("catalog-page-{}", offset.unwrap_or_default());
    let args = offset.map_or_else(|| json!({}), |offset| json!({"offset": offset}));
    mount_sse_once(
        server,
        sse(vec![
            ev_response_created("lookup"),
            ev_function_call(&call_id, "list_models", &args.to_string()),
            ev_completed("lookup"),
        ]),
    )
    .await;
    let result = mount_sse_once(
        server,
        sse(vec![ev_response_created("done"), ev_completed("done")]),
    )
    .await;
    test.submit_turn("List the next page of authorized models.")
        .await?;
    let output = result.single_request().function_call_output(&call_id);
    let text = output["output"].as_str().expect("JSON text output");
    assert!(text.len() <= 3_400, "bounded context item");
    Ok(serde_json::from_str(text)?)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_models_oversized_metadata_does_not_block_later_entries() -> Result<()> {
    use codex_protocol::openai_models::ReasoningEffort;
    use codex_protocol::openai_models::ReasoningEffortPreset;
    let server = start_mock_server().await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_model_info_override("gpt-6-astra", |model| {
            model.priority = -2;
            model.display_name = "🧪".repeat(4096);
        })
        .with_model_info_override("gpt-6-sol", |model| {
            model.priority = -1;
            model.supported_reasoning_levels = vec![ReasoningEffortPreset {
                effort: ReasoningEffort::Custom("x".repeat(4096)),
                description: String::new(),
            }];
        })
        .with_config(|config| {
            config.features.enable(Feature::MultiAgentV2).unwrap();
            config.multi_agent_v2.expose_spawn_agent_model_overrides = true;
        })
        .build_with_auto_env(&server)
        .await?;
    let first = lookup_page(&test, &server, /*offset*/ None).await?;
    assert_eq!(first["models"][0]["model"], "gpt-6-astra");
    assert_eq!(first["models"][0]["display_name"], "🧪".repeat(128));
    assert_eq!(first["models"][0]["display_name_truncated"], true);
    assert_eq!(first["oversized_entries"], json!([1]));
    let offset = first["next_offset"].as_u64().unwrap() as usize;
    assert!(offset > 1);
    let second = lookup_page(&test, &server, Some(offset)).await?;
    assert!(!second["models"].as_array().unwrap().is_empty());
    assert!(second["next_offset"].as_u64().unwrap() as usize > offset);
    Ok(())
}
