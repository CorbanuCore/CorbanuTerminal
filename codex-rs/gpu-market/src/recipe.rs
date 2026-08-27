use crate::HardwareRequirements;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RecipeStability {
    #[default]
    Qualified,
    Experimental,
}

impl RecipeStability {
    pub fn label(self) -> &'static str {
        match self {
            Self::Qualified => "qualified",
            Self::Experimental => "experimental",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuRecipe {
    pub id: String,
    pub revision: String,
    /// Immutable artifact source, normally a Hugging Face repository ID.
    pub model_id: String,
    /// Model identity exposed by the serving runtime. Empty values from older
    /// manifests retain the source model ID for backward compatibility.
    #[serde(default)]
    pub served_model_id: String,
    /// OpenAI-compatible protocol used by Corbanu Terminal after the runtime is
    /// registered. Older manifests used chat completions exclusively.
    #[serde(default = "default_wire_api")]
    pub wire_api: String,
    pub model_revision: String,
    pub image: String,
    pub runtime: String,
    pub serving_runtime_version: String,
    pub license_id: String,
    pub requires_huggingface_token: bool,
    pub minimum_driver_version: String,
    pub gpu_architectures: Vec<String>,
    pub weight_format: String,
    pub hardware: HardwareRequirements,
    pub tensor_parallel_size: u16,
    pub maximum_context_tokens: u64,
    pub maximum_concurrent_requests: u16,
    pub expected_download_bytes: u64,
    pub model_weight_bytes: u64,
    pub kv_cache_reserve_bytes: u64,
    pub workspace_reserve_bytes: u64,
    /// Complete argv supplied to the pinned image after its entrypoint.
    ///
    /// Runtime-specific launch behavior belongs to the immutable recipe. The
    /// controller must not infer a vLLM/SGLang command from the model name.
    pub launch_command: Vec<String>,
    pub environment_allowlist: Vec<String>,
    pub startup_deadline_ms: u64,
    pub download_deadline_ms: u64,
    pub probe_deadline_ms: u64,
    pub inference_port: u16,
    pub chat_encoding: String,
    pub probe_contract: String,
    /// Deployment maturity, separate from manifest verification. An
    /// experimental recipe is immutable and safe to launch but has not yet
    /// earned the product-quality claims of a qualified recipe.
    #[serde(default)]
    pub stability: RecipeStability,
    pub manifest_verified: bool,
}

#[derive(Debug, Clone)]
pub struct RecipeCatalog {
    recipes: Vec<GpuRecipe>,
}

impl Default for RecipeCatalog {
    fn default() -> Self {
        Self {
            recipes: vec![
                deepseek_flash_recipe(/*gpu_count*/ 2),
                crate::glm_recipes::glm_5_3_flash_recipe(),
                crate::glm_recipes::glm_5_3_flash_b300_recipe(),
                crate::glm_recipes::glm_5_3_flash_nvfp4_b200_recipe(),
                crate::qwen_recipes::qwen_3_8_27b_recipe(),
                glm_5_2_recipe(),
                crate::gguf_recipes::huihui_deepseek_v4_flash_recipe(),
                crate::gguf_recipes::huihui_glm_5_2_recipe(),
            ],
        }
    }
}

impl RecipeCatalog {
    pub fn new(recipes: Vec<GpuRecipe>) -> anyhow::Result<Self> {
        let mut ids = std::collections::HashSet::new();
        for recipe in &recipes {
            if recipe.id.trim().is_empty() || !ids.insert(recipe.id.as_str()) {
                return Err(anyhow::anyhow!("recipe ids must be non-empty and unique"));
            }
            if recipe.hardware.gpu_count == 0 || recipe.tensor_parallel_size == 0 {
                return Err(anyhow::anyhow!("recipe GPU counts must be positive"));
            }
            if recipe.manifest_verified {
                validate_verified_recipe(recipe)?;
            }
        }
        Ok(Self { recipes })
    }

    pub fn list(&self) -> &[GpuRecipe] {
        self.recipes.as_slice()
    }

    pub fn get(&self, id: &str) -> Option<&GpuRecipe> {
        self.recipes.iter().find(|recipe| recipe.id == id)
    }
}

impl GpuRecipe {
    pub fn served_model_id(&self) -> &str {
        if self.served_model_id.trim().is_empty() {
            self.model_id.as_str()
        } else {
            self.served_model_id.as_str()
        }
    }
}

fn default_wire_api() -> String {
    "chat".to_string()
}

fn validate_verified_recipe(recipe: &GpuRecipe) -> anyhow::Result<()> {
    anyhow::ensure!(
        matches!(recipe.wire_api.as_str(), "chat" | "responses"),
        "verified recipe wire API must be chat or responses"
    );
    anyhow::ensure!(
        recipe.tensor_parallel_size == recipe.hardware.gpu_count,
        "verified recipe tensor parallel size must match its allocated GPU count"
    );
    anyhow::ensure!(
        is_immutable_revision(recipe.model_revision.as_str()),
        "verified recipe model revision must be an immutable commit digest"
    );
    anyhow::ensure!(
        has_image_digest(recipe.image.as_str()),
        "verified recipe image must use an immutable sha256 digest"
    );
    anyhow::ensure!(
        !recipe.revision.contains("pending")
            && [
                recipe.runtime.as_str(),
                recipe.serving_runtime_version.as_str(),
                recipe.license_id.as_str(),
                recipe.minimum_driver_version.as_str(),
                recipe.weight_format.as_str(),
                recipe.chat_encoding.as_str(),
            ]
            .iter()
            .all(|value| !value.trim().is_empty()),
        "verified recipe manifest metadata is incomplete"
    );
    anyhow::ensure!(
        !recipe.gpu_architectures.is_empty()
            && recipe.maximum_context_tokens > 0
            && recipe.maximum_concurrent_requests > 0
            && recipe.expected_download_bytes > 0
            && recipe.model_weight_bytes > 0
            && recipe.startup_deadline_ms > 0
            && recipe.download_deadline_ms > 0
            && recipe.probe_deadline_ms > 0
            && recipe.inference_port > 0,
        "verified recipe capacity and deadline fields must be complete"
    );
    let total_vram_bytes = recipe
        .hardware
        .minimum_vram_mib_per_gpu
        .checked_mul(u64::from(recipe.hardware.gpu_count))
        .and_then(|value| value.checked_mul(1024 * 1024))
        .ok_or_else(|| anyhow::anyhow!("verified recipe VRAM calculation overflowed"))?;
    let required_vram_bytes = recipe
        .model_weight_bytes
        .checked_add(recipe.kv_cache_reserve_bytes)
        .and_then(|value| value.checked_add(recipe.workspace_reserve_bytes))
        .ok_or_else(|| anyhow::anyhow!("verified recipe memory calculation overflowed"))?;
    anyhow::ensure!(
        required_vram_bytes <= total_vram_bytes,
        "verified recipe exceeds its minimum aggregate VRAM"
    );
    let disk_bytes = recipe
        .hardware
        .minimum_disk_gib
        .checked_mul(1024 * 1024 * 1024)
        .ok_or_else(|| anyhow::anyhow!("verified recipe disk calculation overflowed"))?;
    anyhow::ensure!(
        recipe
            .expected_download_bytes
            .checked_add(recipe.workspace_reserve_bytes)
            .is_some_and(|required| required <= disk_bytes),
        "verified recipe exceeds its minimum disk capacity"
    );
    let allowed_environment = ["PFT_ENDPOINT_TOKEN", "HF_TOKEN"];
    anyhow::ensure!(
        recipe
            .environment_allowlist
            .iter()
            .all(|name| allowed_environment.contains(&name.as_str()))
            && recipe
                .environment_allowlist
                .iter()
                .any(|name| name == "PFT_ENDPOINT_TOKEN")
            && (!recipe.requires_huggingface_token
                || recipe
                    .environment_allowlist
                    .iter()
                    .any(|name| name == "HF_TOKEN")),
        "verified recipe environment allowlist is unsafe or incomplete"
    );
    let launch = recipe.launch_command.join(" ");
    anyhow::ensure!(
        launch.contains(recipe.model_id.as_str())
            && launch.contains(recipe.model_revision.as_str()),
        "verified recipe launch must consume its pinned model source and revision"
    );
    let scoped_auth = recipe
        .launch_command
        .windows(2)
        .any(|arguments| arguments == ["--api-key", "$PFT_ENDPOINT_TOKEN"])
        || launch.contains("--api-key \"$PFT_ENDPOINT_TOKEN\"")
        || (launch.contains("Bearer {$PFT_ENDPOINT_TOKEN}") && launch.contains("reverse_proxy"));
    anyhow::ensure!(
        !recipe.launch_command.is_empty() && !launch.contains("--api-key=") && scoped_auth,
        "verified recipe must launch an authenticated endpoint using its scoped token"
    );
    if recipe.hardware.requires_high_bandwidth_interconnect {
        anyhow::ensure!(
            launch.contains("nvidia-smi topo -m")
                && launch.contains("PFTERMINAL_RUNTIME_GATE=nvlink-ok"),
            "verified multi-GPU recipe must gate serving on allocation-local topology checks"
        );
    }
    if matches!(recipe.runtime.as_str(), "llama.cpp" | "ds4") {
        anyhow::ensure!(
            is_immutable_revision(recipe.serving_runtime_version.as_str())
                && launch.contains(recipe.serving_runtime_version.as_str()),
            "verified source-built recipes must pin and launch their runtime revision"
        );
    }
    anyhow::ensure!(
        recipe.probe_contract == "pfterminal-openai-v1",
        "verified recipe readiness contract is unsupported"
    );
    Ok(())
}

fn is_immutable_revision(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn has_image_digest(value: &str) -> bool {
    value.rsplit_once("@sha256:").is_some_and(|(name, digest)| {
        !name.is_empty()
            && digest.len() == 64
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn deepseek_flash_recipe(gpu_count: u16) -> GpuRecipe {
    let (
        recipe_revision,
        image,
        serving_runtime_version,
        runtime_environment,
        runtime_flags,
        maximum_context_tokens,
        maximum_concurrent_requests,
    ) = match gpu_count {
        2 => (
            "deepseek-v4-flash-0731-sglang-v0.5.15-post1-2xh200-r3",
            "lmsysorg/sglang@sha256:00c53fe4c31bf22d7b37537f28bbdfd924c02de13cdfb4bff7378c9c34d75ab2",
            "0.5.15.post1",
            concat!(
                "SGLANG_JIT_DEEPGEMM_FAST_WARMUP=1 ",
                "SGLANG_OPT_DEEPGEMM_HC_PRENORM=1 ",
                "SGLANG_OPT_USE_TILELANG_MHC_PRE=1"
            ),
            concat!(
                "--context-length 131072 --max-running-requests 8 ",
                "--chunked-prefill-size 16384 --mem-fraction-static 0.82 ",
                "--speculative-algorithm EAGLE --speculative-num-steps 3 ",
                "--speculative-eagle-topk 1 --speculative-num-draft-tokens 4"
            ),
            131_072,
            8,
        ),
        4 => (
            "deepseek-v4-flash-sglang-v0.5.12-4xh200-r1",
            "lmsysorg/sglang@sha256:015f39a45844be5a7b35270c56dc4d9ebcfe9b0c21a3b4f877a4ee22e795bd7a",
            "0.5.12+127b9e3283f7c2a43234b852ff5c9f1796d53624",
            "SGLANG_JIT_DEEPGEMM_FAST_WARMUP=1",
            concat!(
                "--context-length 65536 --max-running-requests 2 --disable-cuda-graph ",
                "--chunked-prefill-size 8192 --mem-fraction-static 0.82"
            ),
            65_536,
            2,
        ),
        _ => unreachable!("the curated DeepSeek catalog contains only TP2 and TP4 recipes"),
    };
    let launch = concat!(
        "set -euo pipefail; printf 'PFTERMINAL_RUNTIME_GATE=begin\\n'; ",
        "test \"$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l)\" -eq {GPU_COUNT}; ",
        "nvidia-smi --query-gpu=name --format=csv,noheader | ",
        "awk 'index($0, \"H200\") == 0 { exit 1 }'; ",
        "printf 'PFTERMINAL_RUNTIME_GATE=gpu-identity-ok\\n'; ",
        "driver=$(nvidia-smi --query-gpu=driver_version --format=csv,noheader | head -1); ",
        "test \"$(printf '%s\\n' 570.26 \"$driver\" | sort -V | head -1)\" = 570.26; ",
        "printf 'PFTERMINAL_RUNTIME_GATE=driver-ok\\n'; ",
        "nvidia-smi topo -m | awk '$1 == \"GPU0\" { for (i=2; i<=NF; i++) if ($i ~ /^NV[0-9]+$/) ok=1 } END { exit !ok }'; ",
        "printf 'PFTERMINAL_RUNTIME_GATE=nvlink-ok\\n'; ",
        "{RUNTIME_ENVIRONMENT} exec python3 -m sglang.launch_server ",
        "--model-path deepseek-ai/DeepSeek-V4-Flash-0731 ",
        "--revision 7872f01b1d1fe23eabc4c98b48bffcef5a386062 ",
        "--served-model-name deepseek-ai/DeepSeek-V4-Flash-0731 ",
        "--host 0.0.0.0 --port 8000 --tp {GPU_COUNT} --enable-p2p-check ",
        "{RUNTIME_FLAGS} ",
        "--watchdog-timeout 1200 --trust-remote-code --moe-runner-backend marlin ",
        "--tool-call-parser deepseekv4 --reasoning-parser deepseek-v4 ",
        "--api-key \"$PFT_ENDPOINT_TOKEN\""
    )
    .replace("{GPU_COUNT}", gpu_count.to_string().as_str())
    .replace("{RUNTIME_ENVIRONMENT}", runtime_environment)
    .replace("{RUNTIME_FLAGS}", runtime_flags);
    GpuRecipe {
        id: format!("deepseek-flash-{gpu_count}xh200"),
        revision: recipe_revision.to_string(),
        model_id: "deepseek-ai/DeepSeek-V4-Flash-0731".to_string(),
        served_model_id: "deepseek-ai/DeepSeek-V4-Flash-0731".to_string(),
        wire_api: "chat".to_string(),
        model_revision: "7872f01b1d1fe23eabc4c98b48bffcef5a386062".to_string(),
        image: image.to_string(),
        runtime: "sglang".to_string(),
        serving_runtime_version: serving_runtime_version.to_string(),
        license_id: "MIT".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "570.26".to_string(),
        gpu_architectures: vec!["sm_90".to_string()],
        weight_format: "mixed-fp4-fp8".to_string(),
        hardware: HardwareRequirements {
            gpu_model: "NVIDIA H200".to_string(),
            gpu_count,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 256 * 1024,
            minimum_disk_gib: 400,
            requires_high_bandwidth_interconnect: true,
            allowed_cuda_versions: vec!["13.0".to_string()],
        },
        tensor_parallel_size: gpu_count,
        maximum_context_tokens,
        maximum_concurrent_requests,
        expected_download_bytes: 166_898_661_074,
        model_weight_bytes: 166_886_535_336,
        kv_cache_reserve_bytes: 48_000_000_000,
        workspace_reserve_bytes: 48_000_000_000,
        launch_command: vec!["bash".to_string(), "-lc".to_string(), launch],
        environment_allowlist: vec!["PFT_ENDPOINT_TOKEN".to_string()],
        startup_deadline_ms: 45 * 60 * 1_000,
        download_deadline_ms: 35 * 60 * 1_000,
        probe_deadline_ms: 10 * 60 * 1_000,
        inference_port: 8000,
        chat_encoding: "deepseek-v4-encoding".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: RecipeStability::Qualified,
        manifest_verified: true,
    }
}

fn glm_5_2_recipe() -> GpuRecipe {
    GpuRecipe {
        id: "glm-5.2-fp8-8xh200".to_string(),
        revision: "glm-5.2-fp8-sglang-v0.5.15-post1-r1".to_string(),
        model_id: "zai-org/GLM-5.2-FP8".to_string(),
        served_model_id: "zai-org/GLM-5.2-FP8".to_string(),
        wire_api: "chat".to_string(),
        model_revision: "ba978f7d347eaf65d22f1a86833408afdb953541".to_string(),
        image: "lmsysorg/sglang@sha256:00c53fe4c31bf22d7b37537f28bbdfd924c02de13cdfb4bff7378c9c34d75ab2".to_string(),
        runtime: "sglang".to_string(),
        serving_runtime_version: "0.5.15.post1".to_string(),
        license_id: "MIT".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "570.26".to_string(),
        gpu_architectures: vec!["sm_90".to_string()],
        weight_format: "fp8".to_string(),
        hardware: HardwareRequirements {
            gpu_model: "NVIDIA H200".to_string(),
            gpu_count: 8,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 768 * 1024,
            minimum_disk_gib: 900,
            requires_high_bandwidth_interconnect: true,
            allowed_cuda_versions: vec!["13.0".to_string()],
        },
        tensor_parallel_size: 8,
        maximum_context_tokens: 131_072,
        maximum_concurrent_requests: 4,
        expected_download_bytes: 753_500_000_000,
        model_weight_bytes: 753_375_793_584,
        kv_cache_reserve_bytes: 160_000_000_000,
        workspace_reserve_bytes: 100_000_000_000,
        launch_command: vec![
            "bash".to_string(),
            "-lc".to_string(),
            concat!(
                "set -euo pipefail; printf 'PFTERMINAL_RUNTIME_GATE=begin\\n'; ",
                "test \"$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l)\" -eq 8; ",
                "nvidia-smi --query-gpu=name --format=csv,noheader | awk 'index($0, \"H200\") == 0 { exit 1 }'; ",
                "nvidia-smi topo -m | awk '$1 == \"GPU0\" { for (i=2; i<=NF; i++) if ($i ~ /^NV[0-9]+$/) ok=1 } END { exit !ok }'; ",
                "printf 'PFTERMINAL_RUNTIME_GATE=nvlink-ok\\n'; ",
                "SGLANG_JIT_DEEPGEMM_FAST_WARMUP=1 exec python3 -m sglang.launch_server ",
                "--model-path zai-org/GLM-5.2-FP8 ",
                "--revision ba978f7d347eaf65d22f1a86833408afdb953541 ",
                "--served-model-name zai-org/GLM-5.2-FP8 ",
                "--host 0.0.0.0 --port 8000 --tp 8 --enable-p2p-check ",
                "--context-length 131072 --max-running-requests 4 ",
                "--mem-fraction-static 0.85 --watchdog-timeout 1200 ",
                "--tool-call-parser glm47 --reasoning-parser glm45 ",
                "--trust-remote-code --api-key \"$PFT_ENDPOINT_TOKEN\""
            )
            .to_string(),
        ],
        environment_allowlist: vec!["PFT_ENDPOINT_TOKEN".to_string()],
        startup_deadline_ms: 75 * 60 * 1_000,
        download_deadline_ms: 60 * 60 * 1_000,
        probe_deadline_ms: 15 * 60 * 1_000,
        inference_port: 8000,
        chat_encoding: "glm-5.2-tokenizer-template".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: RecipeStability::Qualified,
        manifest_verified: true,
    }
}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
