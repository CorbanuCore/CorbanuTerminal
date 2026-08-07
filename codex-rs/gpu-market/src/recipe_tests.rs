use super::*;

fn verified_recipe() -> GpuRecipe {
    GpuRecipe {
        id: "verified-test".to_string(),
        revision: "manifest-v1".to_string(),
        model_family: "test".to_string(),
        recommendation_priority: None,
        model_id: "owner/model".to_string(),
        served_model_id: "owner/model".to_string(),
        wire_api: "chat".to_string(),
        model_revision: "1111111111111111111111111111111111111111".to_string(),
        image: "registry/runtime@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        runtime: "vllm".to_string(),
        serving_runtime_version: "1.0.0".to_string(),
        license_id: "apache-2.0".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "550.0".to_string(),
        gpu_architectures: vec!["sm90".to_string()],
        weight_format: "fp8".to_string(),
        hardware: HardwareRequirements {
            gpu_model: "NVIDIA H200".to_string(),
            gpu_count: 2,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 128_000,
            minimum_disk_gib: 400,
            requires_high_bandwidth_interconnect: true,
            allowed_cuda_versions: vec!["12.8".to_string()],
        },
        tensor_parallel_size: 2,
        maximum_context_tokens: 32_768,
        maximum_concurrent_requests: 2,
        expected_download_bytes: 180_000_000_000,
        model_weight_bytes: 180_000_000_000,
        kv_cache_reserve_bytes: 40_000_000_000,
        workspace_reserve_bytes: 20_000_000_000,
        container_entrypoint: Vec::new(),
        launch_command: vec![
            "server".to_string(),
            "owner/model".to_string(),
            "1111111111111111111111111111111111111111".to_string(),
            "nvidia-smi topo -m; printf PFTERMINAL_RUNTIME_GATE=nvlink-ok".to_string(),
            "--api-key".to_string(),
            "$PFT_ENDPOINT_TOKEN".to_string(),
        ],
        environment_allowlist: vec!["PFT_ENDPOINT_TOKEN".to_string()],
        startup_deadline_ms: 120_000,
        download_deadline_ms: 3_600_000,
        probe_deadline_ms: 60_000,
        inference_port: 8000,
        chat_encoding: "encoding-v1".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: RecipeStability::Qualified,
        manifest_verified: true,
    }
}

#[test]
fn verified_recipe_accepts_only_complete_immutable_manifests() {
    RecipeCatalog::new(vec![verified_recipe()]).expect("complete manifest");

    let mut mutable_model = verified_recipe();
    mutable_model.model_revision = "main".to_string();
    assert!(RecipeCatalog::new(vec![mutable_model]).is_err());

    let mut mutable_image = verified_recipe();
    mutable_image.image = "registry/runtime:latest".to_string();
    assert!(RecipeCatalog::new(vec![mutable_image]).is_err());
}

#[test]
fn recommendation_selection_is_unique_and_excludes_unqualified_recipes() {
    let mut qualified = verified_recipe();
    qualified.id = "qualified".to_string();
    qualified.model_family = "deepseek".to_string();
    qualified.recommendation_priority = Some(1);

    let mut experimental = verified_recipe();
    experimental.id = "experimental".to_string();
    experimental.model_family = "deepseek".to_string();
    experimental.recommendation_priority = Some(0);
    experimental.stability = RecipeStability::Experimental;

    let catalog = RecipeCatalog::new(vec![experimental, qualified.clone()])
        .expect("distinct recommendation slots");
    assert_eq!(
        catalog
            .recommended_for_family("deepseek")
            .map(|recipe| recipe.id.as_str()),
        Some("qualified")
    );

    let mut duplicate = qualified.clone();
    duplicate.id = "duplicate".to_string();
    assert!(RecipeCatalog::new(vec![qualified, duplicate]).is_err());
}

#[test]
fn legacy_manifests_default_the_served_alias_to_the_source_model() {
    let mut json = serde_json::to_value(verified_recipe()).expect("serialize recipe");
    json.as_object_mut()
        .expect("recipe object")
        .remove("served_model_id");
    let recipe: GpuRecipe = serde_json::from_value(json).expect("legacy recipe");

    assert_eq!(recipe.served_model_id(), "owner/model");
}

#[test]
fn legacy_manifests_default_to_chat_and_verified_protocols_are_bounded() {
    let mut json = serde_json::to_value(verified_recipe()).expect("serialize recipe");
    json.as_object_mut()
        .expect("recipe object")
        .remove("wire_api");
    let recipe: GpuRecipe = serde_json::from_value(json).expect("legacy recipe");
    assert_eq!(recipe.wire_api, "chat");

    let mut invalid = verified_recipe();
    invalid.wire_api = "model-name-guessed-protocol".to_string();
    assert!(RecipeCatalog::new(vec![invalid]).is_err());
}

#[test]
fn recommended_deepseek_recipe_resolves_the_0731_vllm_dspark_manifest() {
    let catalog = RecipeCatalog::default();
    let recipe = catalog
        .recommended_for_family("DEEPSEEK")
        .expect("recommended DeepSeek recipe");
    RecipeCatalog::new(vec![recipe.clone()]).expect("valid built-in DeepSeek manifest");

    assert!(recipe.manifest_verified);
    assert_eq!(
        recipe.revision,
        "deepseek-v4-flash-0731-vllm-v0.26.0-2xh200-r1"
    );
    assert_eq!(recipe.model_id, "deepseek-ai/DeepSeek-V4-Flash-0731");
    assert_eq!(
        recipe.model_revision,
        "9e165c30e2704aec5d9d593cce3eebd58bbef1cb"
    );
    assert_eq!(recipe.runtime, "vllm");
    assert_eq!(recipe.serving_runtime_version, "0.26.0");
    assert_eq!(recipe.tensor_parallel_size, 2);
    assert_eq!(recipe.maximum_context_tokens, 131_072);
    assert_eq!(recipe.maximum_concurrent_requests, 8);
    assert_eq!(recipe.container_entrypoint, ["bash", "-lc"]);
    assert!(recipe.launch_command.iter().any(|part| {
        part.contains("vllm serve deepseek-ai/DeepSeek-V4-Flash-0731")
            && part.contains("--tool-call-parser deepseek_v4")
            && part.contains("--reasoning-parser deepseek_v4")
            && part.contains("--api-key \"$PFT_ENDPOINT_TOKEN\"")
            && part.contains("nvidia-smi topo -m")
            && part.contains("for (i=2; i<=NF; i++)")
            && part.contains("PFTERMINAL_RUNTIME_GATE=nvlink-ok")
            && part.contains("--max-model-len 131072")
            && part.contains("--speculative-config")
            && part.contains("\"method\":\"dspark\"")
    }));

    assert!(catalog.recommended_for_family("unknown").is_none());
}

#[test]
fn built_in_catalog_distinguishes_qualified_and_experimental_recipes() {
    let catalog = RecipeCatalog::default();
    let ids = catalog
        .list()
        .iter()
        .map(|recipe| recipe.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        [
            "deepseek-flash-0731-2xh200",
            "glm-5.2-fp8-8xh200",
            "huihui-glm-5.2-iq1m-2xh200-experimental"
        ]
    );
    for recipe in catalog.list() {
        RecipeCatalog::new(vec![recipe.clone()]).expect("valid curated recipe");
        assert!(recipe.manifest_verified);
        assert_eq!(recipe.tensor_parallel_size, recipe.hardware.gpu_count);
    }
    assert_eq!(
        catalog
            .list()
            .iter()
            .map(|recipe| recipe.stability)
            .collect::<Vec<_>>(),
        [
            RecipeStability::Qualified,
            RecipeStability::Qualified,
            RecipeStability::Experimental,
        ]
    );

    let deepseek_recipes = catalog
        .list()
        .iter()
        .filter(|recipe| recipe.model_family == "deepseek")
        .collect::<Vec<_>>();
    assert_eq!(deepseek_recipes.len(), 1);
    assert_eq!(
        deepseek_recipes[0].model_id,
        "deepseek-ai/DeepSeek-V4-Flash-0731"
    );
}

#[test]
fn fine_tune_recipes_pin_source_runtime_artifacts_auth_and_topology() {
    let catalog = RecipeCatalog::default();
    for id in ["huihui-glm-5.2-iq1m-2xh200-experimental"] {
        let recipe = catalog.get(id).expect("fine-tune recipe");
        let launch = recipe.launch_command.join(" ");
        assert_eq!(recipe.stability, RecipeStability::Experimental);
        assert!(launch.contains(recipe.model_revision.as_str()));
        assert!(launch.contains(recipe.serving_runtime_version.as_str()));
        assert!(launch.contains("sha256sum -c -"));
        assert!(launch.contains("huggingface_hub==1.23.0"));
        assert!(launch.contains("hf-xet==1.5.1"));
        assert!(launch.contains("HF_XET_HIGH_PERFORMANCE=1 hf download"));
        assert!(launch.contains(recipe.model_id.as_str()));
        assert!(launch.contains("PFTERMINAL_RUNTIME_GATE=nvlink-ok"));
        for phase in [
            "hardware_check",
            "runtime_setup",
            "runtime_build",
            "model_download",
            "model_verification",
            "model_loading",
            "endpoint_probing",
        ] {
            assert!(launch.contains(&format!("pft_phase {phase}")));
        }
        assert!(!launch.contains("--enable-p2p-check"));
        assert!(launch.contains("Bearer {$PFT_ENDPOINT_TOKEN}"));
        assert!(launch.contains("reverse_proxy 127.0.0.1:8001"));
        assert!(
            launch.contains("527fbf917c39189a1e3b31d34fa955601680b2d5c8055d2a87b8b9588dec7bb9")
        );
        assert_eq!(recipe.runtime, "llama.cpp");
        assert_eq!(recipe.served_model_id(), recipe.model_id);
        assert_eq!(recipe.wire_api, "chat");
        assert_eq!(recipe.maximum_context_tokens, 300_000);
        assert_eq!(recipe.kv_cache_reserve_bytes, 28_000_000_000);
        assert!(launch.contains("--alias"));
        assert!(launch.contains("--api-key \"$PFT_ENDPOINT_TOKEN\""));
        assert!(launch.contains("--ctx-size 300000"));
        assert!(launch.contains("for gpu in 0 1"));
        assert!(launch.contains("CUDA_VISIBLE_DEVICES=\"$gpu\""));
        assert!(launch.contains("$0 !~ /\\(0 MiB, 0 MiB free\\)$/"));
        assert!(launch.contains("PFTERMINAL_RUNTIME_GATE=cuda-ok"));
        // Upstream CUDA does not expose split buffers for this GLM IQ1_M
        // path, so distribute complete layers rather than tensor rows.
        assert!(launch.contains("--split-mode layer"));
    }
}

#[test]
fn verified_recipe_fails_capacity_and_secret_boundaries() {
    let mut over_capacity = verified_recipe();
    over_capacity.kv_cache_reserve_bytes = u64::MAX;
    assert!(RecipeCatalog::new(vec![over_capacity]).is_err());

    let mut embedded_secret = verified_recipe();
    embedded_secret.launch_command = vec!["--api-key=plaintext".to_string()];
    assert!(RecipeCatalog::new(vec![embedded_secret]).is_err());

    let mut unknown_environment = verified_recipe();
    unknown_environment
        .environment_allowlist
        .push("AWS_SECRET_ACCESS_KEY".to_string());
    assert!(RecipeCatalog::new(vec![unknown_environment]).is_err());
}
