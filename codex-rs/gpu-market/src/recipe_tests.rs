use super::*;

fn verified_recipe() -> GpuRecipe {
    GpuRecipe {
        id: "verified-test".to_string(),
        revision: "manifest-v1".to_string(),
        model_id: "owner/model".to_string(),
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
        launch_command: vec![
            "server".to_string(),
            "nvidia-smi topo -m --enable-p2p-check".to_string(),
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
fn built_in_deepseek_recipe_is_a_validated_runtime_specific_manifest() {
    let catalog = RecipeCatalog::default();
    let recipe = catalog
        .get("deepseek-flash-2xh200")
        .expect("DeepSeek recipe");
    RecipeCatalog::new(vec![recipe.clone()]).expect("valid built-in DeepSeek manifest");

    assert!(recipe.manifest_verified);
    assert_eq!(
        recipe.revision,
        "deepseek-v4-flash-sglang-v0.5.15-post1-2xh200-r2"
    );
    assert_eq!(recipe.runtime, "sglang");
    assert_eq!(recipe.serving_runtime_version, "0.5.15.post1");
    assert_eq!(recipe.tensor_parallel_size, 2);
    assert_eq!(recipe.maximum_context_tokens, 131_072);
    assert_eq!(recipe.maximum_concurrent_requests, 8);
    assert_eq!(recipe.launch_command[0], "bash");
    assert!(recipe.launch_command.iter().any(|part| {
        part.contains("--tool-call-parser deepseekv4")
            && part.contains("--api-key \"$PFT_ENDPOINT_TOKEN\"")
            && part.contains("nvidia-smi topo -m")
            && part.contains("for (i=2; i<=NF; i++)")
            && part.contains("PFTERMINAL_RUNTIME_GATE=nvlink-ok")
            && !part.contains("--disable-cuda-graph")
            && part.contains("--moe-runner-backend marlin")
            && part.contains("--watchdog-timeout 1200")
            && part.contains("SGLANG_JIT_DEEPGEMM_FAST_WARMUP=1")
            && part.contains("SGLANG_OPT_DEEPGEMM_HC_PRENORM=1")
            && part.contains("SGLANG_OPT_USE_TILELANG_MHC_PRE=1")
            && part.contains("--context-length 131072")
            && part.contains("--max-running-requests 8")
            && part.contains("--chunked-prefill-size 16384")
            && part.contains("--speculative-algorithm EAGLE")
            && part.contains("--speculative-num-steps 3")
            && part.contains("--speculative-eagle-topk 1")
            && part.contains("--speculative-num-draft-tokens 4")
    }));

    let tp4 = catalog
        .get("deepseek-flash-4xh200")
        .expect("qualified TP4 recipe");
    assert_eq!(tp4.revision, "deepseek-v4-flash-sglang-v0.5.12-4xh200-r1");
    assert!(
        tp4.launch_command
            .iter()
            .any(|part| part.contains("--disable-cuda-graph"))
    );
}

#[test]
fn built_in_catalog_contains_only_the_three_curated_proven_topologies() {
    let catalog = RecipeCatalog::default();
    let ids = catalog
        .list()
        .iter()
        .map(|recipe| recipe.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        [
            "deepseek-flash-2xh200",
            "deepseek-flash-4xh200",
            "glm-5.2-fp8-8xh200"
        ]
    );
    for recipe in catalog.list() {
        RecipeCatalog::new(vec![recipe.clone()]).expect("valid curated recipe");
        assert!(recipe.manifest_verified);
        assert_eq!(recipe.tensor_parallel_size, recipe.hardware.gpu_count);
        assert!(
            recipe
                .launch_command
                .iter()
                .any(|part| part.contains("SGLANG_JIT_DEEPGEMM_FAST_WARMUP=1"))
        );
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
