use super::*;
use crate::WireApi;
use crate::built_in_model_providers;
use crate::canonical_catalog_provider;
use crate::corrected_catalog_provider;
use crate::resolve_model_for_provider;
use pretty_assertions::assert_eq;

#[test]
fn glm_models_list_both_zai_connections_openai_style_first() {
    for model in ["glm-5.3-flash", "glm-5.3", "glm-5.2"] {
        let providers = model_routes(model)
            .iter()
            .map(|route| (route.provider_id, route.style))
            .collect::<Vec<_>>();
        assert_eq!(
            providers,
            vec![
                (ZAI_PROVIDER_ID, ModelRouteStyle::OpenAi),
                (ZAI_ANTHROPIC_PROVIDER_ID, ModelRouteStyle::Claude),
            ],
            "{model}"
        );
    }
}

#[test]
fn single_connection_models_have_no_route_choice() {
    for model in [
        "gpt-5.5",
        "z-ai/glm-5.3-flash",
        "zai/glm-5.3-flash",
        "k3",
        "",
    ] {
        assert!(model_routes(model).is_empty(), "{model}");
        assert_eq!(default_model_route(model), None, "{model}");
    }
}

/// #653 default rule: wherever a default connection is picked for a model with
/// several, it is the OpenAI-style one.
#[test]
fn default_connection_is_openai_style_everywhere() {
    for model in ["glm-5.3-flash", "glm-5.3", "glm-5.2"] {
        let default = default_model_route(model).expect("several connections");
        assert_eq!(default.style, ModelRouteStyle::OpenAi);
        assert_eq!(canonical_catalog_provider(model), Some(default.provider_id));
        assert_eq!(
            corrected_catalog_provider(model, "openai"),
            Some(default.provider_id)
        );
    }
}

#[test]
fn every_connection_is_a_built_in_provider_that_serves_the_model_in_its_style() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    for model in ["glm-5.3-flash", "glm-5.3", "glm-5.2"] {
        for route in model_routes(model) {
            let provider = providers
                .get(route.provider_id)
                .unwrap_or_else(|| panic!("{} is built in", route.provider_id));
            let expected = match provider.wire_api {
                WireApi::Anthropic => ModelRouteStyle::Claude,
                WireApi::Chat | WireApi::Responses => ModelRouteStyle::OpenAi,
            };
            assert_eq!(route.style, expected, "{}", route.provider_id);
            assert_eq!(
                resolve_model_for_provider(Some(model.to_string()), route.provider_id).as_deref(),
                Some(model),
                "{} keeps {model}",
                route.provider_id
            );
            assert_eq!(
                corrected_catalog_provider(model, route.provider_id),
                None,
                "{} is not corrected away",
                route.provider_id
            );
        }
    }
}

#[test]
fn labels_are_plain() {
    let claude = model_route("glm-5.3-flash", ZAI_ANTHROPIC_PROVIDER_ID).expect("route");
    let openai = model_route("glm-5.3-flash", ZAI_PROVIDER_ID).expect("route");
    assert_eq!(claude.label(), "Z.AI (Claude-style)");
    assert_eq!(
        openai.model_label("Z.AI GLM 5.3 Flash"),
        "GLM 5.3 Flash · Z.AI (OpenAI-style)"
    );
    assert_eq!(
        claude.model_label("Z.AI GLM 5.3 Flash"),
        "GLM 5.3 Flash · Z.AI (Claude-style)"
    );
    assert_eq!(
        claude.model_label("glm-4.7"),
        "glm-4.7 · Z.AI (Claude-style)"
    );
    assert_eq!(model_route("glm-5.3-flash", "openrouter"), None);
}
