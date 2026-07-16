use crate::GpuRecipe;
use crate::HardwareRequirements;
use crate::RecipeStability;

// This already-qualified image supplies Python for runtime bootstrap and the CUDA compiler
// needed by the pinned source runtimes.
const GGUF_BUILD_IMAGE: &str =
    "lmsysorg/sglang@sha256:00c53fe4c31bf22d7b37537f28bbdfd924c02de13cdfb4bff7378c9c34d75ab2";
const LLAMA_CPP_REVISION: &str = "c3d47e696b1187a27e896aa828d48ff9a33fc679";
const DS4_TP_REVISION: &str = "f17f6fe8758bd4d00439546de2e7904c9ee38fb0";
const CADDY_VERSION: &str = "2.11.4";
const CADDY_LINUX_AMD64_SHA256: &str =
    "527fbf917c39189a1e3b31d34fa955601680b2d5c8055d2a87b8b9588dec7bb9";
const GLM_CONTEXT_TOKENS: u64 = 300_000;
const GLM_KV_CACHE_RESERVE_BYTES: u64 = 28_000_000_000;

pub(crate) fn huihui_deepseek_v4_flash_recipe() -> GpuRecipe {
    let model_id = "huihui-ai/Huihui-DeepSeek-V4-Flash-abliterated-ds4-GGUF";
    let served_model_id = "deepseek-v4-flash";
    let model_revision = "f06f59bce3c36b3282b75c9fe2621c83c9399d10";
    let filename = "Huihui-DeepSeek-V4-Flash-BF16-abliterated-ds4-Q4_K.gguf";
    let launch = format!(
        concat!(
            "set -euo pipefail; export DEBIAN_FRONTEND=noninteractive; ",
            "test \"$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l)\" -eq 2; ",
            "nvidia-smi --query-gpu=name --format=csv,noheader | awk 'index($0, \"H200\") == 0 {{ exit 1 }}'; ",
            "nvidia-smi topo -m | awk '$1 == \"GPU0\" {{ for (i=2; i<=NF; i++) if ($i ~ /^NV[0-9]+$/) ok=1 }} END {{ exit !ok }}'; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=nvlink-ok\\n'; ",
            "apt-get update; apt-get install -y --no-install-recommends git build-essential ca-certificates python3-pip; ",
            "python3 -m pip install --break-system-packages --no-cache-dir huggingface_hub==1.23.0 hf-xet==1.5.1; ",
            "if [ ! -d /opt/ds4/.git ]; then git clone --branch pfterminal-tp-session-sync https://github.com/agtico/ds4.git /opt/ds4; fi; ",
            "cd /opt/ds4; git checkout {runtime_revision}; ",
            "test \"$(git rev-parse HEAD)\" = {runtime_revision}; ",
            "make -B ds4-server CUDA_ARCH=sm_90 -j \"$(nproc)\"; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=build-ok\\n'; mkdir -p /models; ",
            "HF_XET_HIGH_PERFORMANCE=1 hf download {model_id} {filename} --revision {model_revision} --local-dir /models; ",
            "printf '39f3e232fc6c4e1a42a60bf7842c86bd3f1db53e7231e07047f430a713a5f97e  /models/{filename}\\n' | sha256sum -c -; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=model-ok\\n'; ",
            "python3 -c 'import hashlib,io,sys,tarfile,urllib.request; d=urllib.request.urlopen(sys.argv[1],timeout=120).read(); assert hashlib.sha256(d).hexdigest()==sys.argv[2]; t=tarfile.open(fileobj=io.BytesIO(d),mode=\"r:gz\"); f=t.extractfile(\"caddy\"); assert f is not None; open(sys.argv[3],\"wb\").write(f.read())' ",
            "'https://github.com/caddyserver/caddy/releases/download/v{caddy_version}/caddy_{caddy_version}_linux_amd64.tar.gz' ",
            "'{caddy_sha256}' /opt/caddy; chmod 700 /opt/caddy; ",
            "printf '%s\\n' ':8000 {{' '  @unauthorized not header Authorization \"Bearer {{$PFT_ENDPOINT_TOKEN}}\"' ",
            "'  respond @unauthorized 401' '  reverse_proxy 127.0.0.1:8001 {{' '    flush_interval -1' '  }}' '}}' > /tmp/Caddyfile; ",
            "mkdir -p /models/ds4-kv; cd /opt/ds4; ",
            "CUDA_VISIBLE_DEVICES=0,1 ./ds4-server --cuda -m /models/{filename} --tensor-parallel 2 ",
            "--ctx 131072 --power 100 --warm-weights ",
            "--host 127.0.0.1 --port 8001 >/tmp/ds4-server.log 2>&1 & server_pid=$!; ",
            "for i in $(seq 1 900); do kill -0 \"$server_pid\"; python3 -c 'import urllib.request; urllib.request.urlopen(\"http://127.0.0.1:8001/v1/models\",timeout=2).read()' >/dev/null 2>&1 && break; sleep 2; done; ",
            "kill -0 \"$server_pid\"; python3 -c 'import urllib.request; urllib.request.urlopen(\"http://127.0.0.1:8001/v1/models\",timeout=2).read()' >/dev/null; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=server-ok\\n'; /opt/caddy run --config /tmp/Caddyfile --adapter caddyfile & proxy_pid=$!; ",
            "set +e; wait -n \"$server_pid\" \"$proxy_pid\"; status=$?; kill \"$server_pid\" \"$proxy_pid\" 2>/dev/null; wait 2>/dev/null; exit \"$status\""
        ),
        runtime_revision = DS4_TP_REVISION,
        filename = filename,
        model_id = model_id,
        model_revision = model_revision,
        caddy_version = CADDY_VERSION,
        caddy_sha256 = CADDY_LINUX_AMD64_SHA256,
    );
    GpuRecipe {
        id: "huihui-deepseek-v4-flash-q4k-2xh200-experimental".to_string(),
        revision: "huihui-deepseek-v4-flash-q4k-ds4-tp2-r3".to_string(),
        model_id: model_id.to_string(),
        served_model_id: served_model_id.to_string(),
        wire_api: "responses".to_string(),
        model_revision: model_revision.to_string(),
        image: GGUF_BUILD_IMAGE.to_string(),
        runtime: "ds4".to_string(),
        serving_runtime_version: DS4_TP_REVISION.to_string(),
        license_id: "MIT".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "570.26".to_string(),
        gpu_architectures: vec!["sm_90".to_string()],
        weight_format: "q4_k".to_string(),
        hardware: h200_pair(
            /*minimum_vram_mib_per_gpu*/ 130_000,
            /*minimum_host_ram_mib*/ 384 * 1024,
            /*minimum_disk_gib*/ 350,
        ),
        tensor_parallel_size: 2,
        maximum_context_tokens: 131_072,
        maximum_concurrent_requests: 1,
        expected_download_bytes: 164_633_502_304,
        model_weight_bytes: 164_633_502_304,
        kv_cache_reserve_bytes: 8_000_000_000,
        workspace_reserve_bytes: 50_000_000_000,
        launch_command: vec!["bash".to_string(), "-lc".to_string(), launch],
        environment_allowlist: vec!["PFT_ENDPOINT_TOKEN".to_string()],
        startup_deadline_ms: 90 * 60 * 1_000,
        download_deadline_ms: 75 * 60 * 1_000,
        probe_deadline_ms: 15 * 60 * 1_000,
        inference_port: 8000,
        chat_encoding: "ds4-native-dsml".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: RecipeStability::Experimental,
        manifest_verified: true,
    }
}

pub(crate) fn huihui_glm_5_2_recipe() -> GpuRecipe {
    let model_id = "huihui-ai/Huihui-GLM-5.2-abliterated-GGUF";
    let model_revision = "2f0aff2760627f87faf16f8adc81caca7d7b10f6";
    let launch = format!(
        concat!(
            "set -euo pipefail; export DEBIAN_FRONTEND=noninteractive; ",
            "test \"$(nvidia-smi --query-gpu=name --format=csv,noheader | wc -l)\" -eq 2; ",
            "nvidia-smi --query-gpu=name --format=csv,noheader | awk 'index($0, \"H200\") == 0 {{ exit 1 }}'; ",
            "nvidia-smi topo -m | awk '$1 == \"GPU0\" {{ for (i=2; i<=NF; i++) if ($i ~ /^NV[0-9]+$/) ok=1 }} END {{ exit !ok }}'; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=nvlink-ok\\n'; ",
            "apt-get update; apt-get install -y --no-install-recommends git cmake ninja-build build-essential ca-certificates python3-pip; ",
            "python3 -m pip install --break-system-packages --no-cache-dir huggingface_hub==1.23.0 hf-xet==1.5.1; ",
            "if [ ! -d /opt/llama.cpp/.git ]; then git clone https://github.com/ggml-org/llama.cpp.git /opt/llama.cpp; fi; ",
            "cd /opt/llama.cpp; git checkout {runtime_revision}; ",
            "test \"$(git rev-parse HEAD)\" = {runtime_revision}; ",
            "cmake -S . -B build -G Ninja -DGGML_CUDA=ON -DGGML_CUDA_FA_ALL_QUANTS=ON -DCMAKE_BUILD_TYPE=Release; ",
            "cmake --build build --target llama-server -j \"$(nproc)\"; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=build-ok\\n'; mkdir -p /models/glm52; ",
            "HF_XET_HIGH_PERFORMANCE=1 hf download {model_id} --revision {model_revision} --include 'UD-IQ1_M/*.gguf' --local-dir /models/glm52; ",
            "printf '%s\\n' ",
            "'e16a262199ae650398bd154d7dc108a7bd03da07bfafc51dc410dc0b68d9a258  /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00001-of-00006.gguf' ",
            "'60cb22b0610081ddcd71b915a1ba27d680fcc0c55213c7a264c500b11006dd99  /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00002-of-00006.gguf' ",
            "'4bc04b128ecd9b6589787afe3373529465b3ed3a7554f51273c0c94e9cf1abe2  /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00003-of-00006.gguf' ",
            "'341f28c0884b22c9f39b6feacef0b84cdfe92f744174e2d7fe693b6ac7c3fe00  /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00004-of-00006.gguf' ",
            "'b5d77a9d61d725a2edd6464f7414036e54a1a16da017c4236071c8cb74f83e6b  /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00005-of-00006.gguf' ",
            "'dbad0161633006c7351d9299e7fd9372e3be79edf597ea630a29f5b204275207  /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00006-of-00006.gguf' | sha256sum -c -; ",
            "printf 'PFTERMINAL_RUNTIME_GATE=model-ok\\n'; ",
            "python3 -c 'import hashlib,io,sys,tarfile,urllib.request; d=urllib.request.urlopen(sys.argv[1],timeout=120).read(); assert hashlib.sha256(d).hexdigest()==sys.argv[2]; t=tarfile.open(fileobj=io.BytesIO(d),mode=\"r:gz\"); f=t.extractfile(\"caddy\"); assert f is not None; open(sys.argv[3],\"wb\").write(f.read())' ",
            "'https://github.com/caddyserver/caddy/releases/download/v{caddy_version}/caddy_{caddy_version}_linux_amd64.tar.gz' ",
            "'{caddy_sha256}' /opt/caddy; chmod 700 /opt/caddy; ",
            "printf '%s\\n' ':8000 {{' '  @unauthorized not header Authorization \"Bearer {{$PFT_ENDPOINT_TOKEN}}\"' ",
            "'  respond @unauthorized 401' '  reverse_proxy 127.0.0.1:8001 {{' '    flush_interval -1' '  }}' '}}' > /tmp/Caddyfile; ",
            "/opt/llama.cpp/build/bin/llama-server ",
            "--model /models/glm52/UD-IQ1_M/GLM-5.2-UD-IQ1_M-00001-of-00006.gguf ",
            "--alias {model_id} --host 127.0.0.1 --port 8001 ",
            "--api-key \"$PFT_ENDPOINT_TOKEN\" --ctx-size {context_tokens} --parallel 1 ",
            "--n-gpu-layers 999 --split-mode layer --tensor-split 1,1 --flash-attn on --jinja ",
            ">/tmp/llama-server.log 2>&1 & server_pid=$!; ",
            "for i in $(seq 1 900); do kill -0 \"$server_pid\"; python3 -c 'import os,urllib.request; r=urllib.request.Request(\"http://127.0.0.1:8001/v1/models\",headers={{\"Authorization\":\"Bearer \"+os.environ[\"PFT_ENDPOINT_TOKEN\"]}}); urllib.request.urlopen(r,timeout=2).read()' >/dev/null 2>&1 && break; sleep 2; done; ",
            "kill -0 \"$server_pid\"; printf 'PFTERMINAL_RUNTIME_GATE=server-ok\\n'; ",
            "/opt/caddy run --config /tmp/Caddyfile --adapter caddyfile & proxy_pid=$!; ",
            "set +e; wait -n \"$server_pid\" \"$proxy_pid\"; status=$?; kill \"$server_pid\" \"$proxy_pid\" 2>/dev/null; wait 2>/dev/null; exit \"$status\""
        ),
        runtime_revision = LLAMA_CPP_REVISION,
        model_id = model_id,
        model_revision = model_revision,
        context_tokens = GLM_CONTEXT_TOKENS,
        caddy_version = CADDY_VERSION,
        caddy_sha256 = CADDY_LINUX_AMD64_SHA256,
    );
    GpuRecipe {
        id: "huihui-glm-5.2-iq1m-2xh200-experimental".to_string(),
        revision: "huihui-glm-5.2-iq1m-llamacpp-2xh200-r3".to_string(),
        model_id: model_id.to_string(),
        served_model_id: model_id.to_string(),
        wire_api: "chat".to_string(),
        model_revision: model_revision.to_string(),
        image: GGUF_BUILD_IMAGE.to_string(),
        runtime: "llama.cpp".to_string(),
        serving_runtime_version: LLAMA_CPP_REVISION.to_string(),
        license_id: "MIT".to_string(),
        requires_huggingface_token: false,
        minimum_driver_version: "570.26".to_string(),
        gpu_architectures: vec!["sm_90".to_string()],
        weight_format: "ud-iq1_m".to_string(),
        hardware: h200_pair(
            /*minimum_vram_mib_per_gpu*/ 132_000,
            /*minimum_host_ram_mib*/ 480 * 1024,
            /*minimum_disk_gib*/ 500,
        ),
        tensor_parallel_size: 2,
        maximum_context_tokens: GLM_CONTEXT_TOKENS,
        maximum_concurrent_requests: 1,
        expected_download_bytes: 231_226_309_536,
        model_weight_bytes: 231_226_309_536,
        kv_cache_reserve_bytes: GLM_KV_CACHE_RESERVE_BYTES,
        workspace_reserve_bytes: 16_000_000_000,
        launch_command: vec!["bash".to_string(), "-lc".to_string(), launch],
        environment_allowlist: vec!["PFT_ENDPOINT_TOKEN".to_string()],
        startup_deadline_ms: 120 * 60 * 1_000,
        download_deadline_ms: 90 * 60 * 1_000,
        probe_deadline_ms: 20 * 60 * 1_000,
        inference_port: 8000,
        chat_encoding: "gguf-embedded-jinja".to_string(),
        probe_contract: "pfterminal-openai-v1".to_string(),
        stability: RecipeStability::Experimental,
        manifest_verified: true,
    }
}

fn h200_pair(
    minimum_vram_mib_per_gpu: u64,
    minimum_host_ram_mib: u64,
    minimum_disk_gib: u64,
) -> HardwareRequirements {
    HardwareRequirements {
        gpu_model: "NVIDIA H200".to_string(),
        gpu_count: 2,
        minimum_vram_mib_per_gpu,
        minimum_host_ram_mib,
        minimum_disk_gib,
        requires_high_bandwidth_interconnect: true,
        allowed_cuda_versions: vec!["13.0".to_string(), "13.1".to_string(), "13.2".to_string()],
    }
}
