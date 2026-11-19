//! Pin upstream scenarios to the complete catalog shipped in rust-v0.167.0.
//! Verbatim JSON subsets keep each fixture below the repository's blob limit.

use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelsResponse;

/// Upstream reviewer capabilities/policy on a synthetic model, never the product catalog.
/// D593(b)/D604: exercising the reviewer must not enable background scoring.
pub fn guardian_test_model() -> ModelInfo {
    serde_json::from_str(include_str!("fixtures/guardian-test-model.json"))
        .expect("Guardian test model")
}

pub fn upstream_model_catalog() -> ModelsResponse {
    let mut catalog = ModelsResponse { models: Vec::new() };
    for fixture in [
        include_str!("fixtures/upstream-v0.167.0-models-1.json"),
        include_str!("fixtures/upstream-v0.167.0-models-2.json"),
        include_str!("fixtures/upstream-v0.167.0-models-3.json"),
        include_str!("fixtures/upstream-v0.167.0-models-4.json"),
    ] {
        let subset: ModelsResponse = serde_json::from_str(fixture).expect("upstream model catalog");
        catalog.models.extend(subset.models);
    }
    catalog
}
