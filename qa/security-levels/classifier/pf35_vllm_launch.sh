#!/usr/bin/env bash
set -euo pipefail

pf35_root="${PF35_QUALIFICATION_ROOT:-/home/travis/pf35-qualification}"
pf35_venv="$pf35_root/runtime/vllm-0.27.1"
pf35_cuda="$pf35_venv/lib/python3.12/site-packages/nvidia/cu13"
pf35_model="$pf35_root/models/preetpatel-Qwen3.8-27B-Uncensored-NVFP4-37b5130"

export CUDA_HOME="$pf35_cuda"
export PATH="$pf35_cuda/bin:$pf35_venv/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
export LD_LIBRARY_PATH="$pf35_cuda/lib:$pf35_cuda/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export VLLM_CACHE_ROOT="$pf35_root/cache/vllm"
export FLASHINFER_WORKSPACE_BASE="$pf35_root/cache/flashinfer"
export TORCH_EXTENSIONS_DIR="$pf35_root/cache/torch-extensions"
export XDG_CACHE_HOME="$pf35_root/cache"
export MAX_JOBS=1
export CMAKE_BUILD_PARALLEL_LEVEL=1
export NVCC_THREADS=1
export PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True

exec "$pf35_venv/bin/vllm" serve "$pf35_model" \
  --served-model-name pf35-qwen38-preet-nvfp4 \
  --host 127.0.0.1 \
  --port 8000 \
  --max-model-len 4096 \
  --kv-cache-dtype fp8 \
  --kv-cache-memory-bytes 48G \
  --max-num-seqs 64 \
  --max-num-batched-tokens 8192 \
  --enable-prefix-caching \
  --enable-chunked-prefill \
  --language-model-only \
  --generation-config vllm \
  --reasoning-parser qwen3 \
  --default-chat-template-kwargs '{"enable_thinking":false}' \
  --optimization-level 2 \
  --cudagraph-capture-sizes 1 2 4 8 16 32 64 \
  --no-enable-log-requests \
  --no-enable-log-outputs
