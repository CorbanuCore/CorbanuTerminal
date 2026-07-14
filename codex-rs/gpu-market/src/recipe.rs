use crate::HardwareRequirements;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuRecipe {
    pub id: String,
    pub revision: String,
    pub model_id: String,
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
    pub launch_arguments: Vec<String>,
    pub environment_allowlist: Vec<String>,
    pub startup_deadline_ms: u64,
    pub download_deadline_ms: u64,
    pub probe_deadline_ms: u64,
    pub inference_port: u16,
    pub chat_encoding: String,
    pub probe_contract: String,
    pub manifest_verified: bool,
}

#[derive(Debug, Clone)]
pub struct RecipeCatalog {
    recipes: Vec<GpuRecipe>,
}

impl Default for RecipeCatalog {
    fn default() -> Self {
        Self {
            recipes: vec![qwen_recipe(), deepseek_flash_recipe()],
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

fn validate_verified_recipe(recipe: &GpuRecipe) -> anyhow::Result<()> {
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
    let allowed_environment = ["VLLM_API_KEY", "HF_TOKEN"];
    anyhow::ensure!(
        recipe
            .environment_allowlist
            .iter()
            .all(|name| allowed_environment.contains(&name.as_str()))
            && recipe
                .environment_allowlist
                .iter()
                .any(|name| name == "VLLM_API_KEY")
            && (!recipe.requires_huggingface_token
                || recipe
                    .environment_allowlist
                    .iter()
                    .any(|name| name == "HF_TOKEN")),
        "verified recipe environment allowlist is unsafe or incomplete"
    );
    anyhow::ensure!(
        recipe
            .launch_arguments
            .iter()
            .all(|argument| argument != "--api-key" && !argument.starts_with("--api-key=")),
        "verified recipe cannot embed endpoint credentials in launch arguments"
    );
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

fn qwen_recipe() -> GpuRecipe {
    GpuRecipe {
        id: "qwen-32b-1xh200".to_string(),
        revision: "manifest-pending".to_string(),
        model_id: "Qwen/Qwen3-32B".to_string(),
        model_revision: "main".to_string(),
        image: "vllm/vllm-openai:manifest-pending".to_string(),
        runtime: "vllm".to_string(),
        serving_runtime_version: "manifest-pending".to_string(),
        license_id: "manifest-pending".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "manifest-pending".to_string(),
        gpu_architectures: Vec::new(),
        weight_format: "manifest-pending".to_string(),
        hardware: HardwareRequirements {
            gpu_model: "NVIDIA H200".to_string(),
            gpu_count: 1,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 64 * 1024,
            minimum_disk_gib: 160,
            requires_high_bandwidth_interconnect: false,
            allowed_cuda_versions: Vec::new(),
        },
        tensor_parallel_size: 1,
        maximum_context_tokens: 131_072,
        maximum_concurrent_requests: 4,
        expected_download_bytes: 80_000_000_000,
        model_weight_bytes: 0,
        kv_cache_reserve_bytes: 0,
        workspace_reserve_bytes: 0,
        launch_arguments: Vec::new(),
        environment_allowlist: vec!["VLLM_API_KEY".to_string()],
        startup_deadline_ms: 0,
        download_deadline_ms: 0,
        probe_deadline_ms: 0,
        inference_port: 8000,
        chat_encoding: "tokenizer-chat-template".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        manifest_verified: false,
    }
}

fn deepseek_flash_recipe() -> GpuRecipe {
    GpuRecipe {
        id: "deepseek-flash-2xh200".to_string(),
        revision: "prior-run-validated-manifest-pending".to_string(),
        model_id: "deepseek-ai/DeepSeek-V4-Flash".to_string(),
        model_revision: "main".to_string(),
        image: "vllm/vllm-openai:manifest-pending".to_string(),
        runtime: "vllm".to_string(),
        serving_runtime_version: "manifest-pending".to_string(),
        license_id: "manifest-pending".to_string(),
        requires_huggingface_token: true,
        minimum_driver_version: "manifest-pending".to_string(),
        gpu_architectures: Vec::new(),
        weight_format: "manifest-pending".to_string(),
        hardware: HardwareRequirements {
            gpu_model: "NVIDIA H200".to_string(),
            gpu_count: 2,
            minimum_vram_mib_per_gpu: 130_000,
            minimum_host_ram_mib: 256 * 1024,
            minimum_disk_gib: 400,
            requires_high_bandwidth_interconnect: true,
            allowed_cuda_versions: Vec::new(),
        },
        tensor_parallel_size: 2,
        maximum_context_tokens: 384_000,
        maximum_concurrent_requests: 2,
        expected_download_bytes: 180_000_000_000,
        model_weight_bytes: 0,
        kv_cache_reserve_bytes: 0,
        workspace_reserve_bytes: 0,
        launch_arguments: Vec::new(),
        environment_allowlist: vec!["VLLM_API_KEY".to_string(), "HF_TOKEN".to_string()],
        startup_deadline_ms: 0,
        download_deadline_ms: 0,
        probe_deadline_ms: 0,
        inference_port: 8000,
        chat_encoding: "deepseek-v4-encoding".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        manifest_verified: false,
    }
}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
