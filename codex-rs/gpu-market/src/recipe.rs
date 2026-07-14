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
    pub hardware: HardwareRequirements,
    pub tensor_parallel_size: u16,
    pub maximum_context_tokens: u64,
    pub maximum_concurrent_requests: u16,
    pub expected_download_bytes: u64,
    pub launch_arguments: Vec<String>,
    pub chat_encoding: String,
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

fn qwen_recipe() -> GpuRecipe {
    GpuRecipe {
        id: "qwen-32b-1xh200".to_string(),
        revision: "manifest-pending".to_string(),
        model_id: "Qwen/Qwen3-32B".to_string(),
        model_revision: "main".to_string(),
        image: "vllm/vllm-openai:manifest-pending".to_string(),
        runtime: "vllm".to_string(),
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
        launch_arguments: Vec::new(),
        chat_encoding: "tokenizer-chat-template".to_string(),
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
        launch_arguments: Vec::new(),
        chat_encoding: "deepseek-v4-encoding".to_string(),
        manifest_verified: false,
    }
}
