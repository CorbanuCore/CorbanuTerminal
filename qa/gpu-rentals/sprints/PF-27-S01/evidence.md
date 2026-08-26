# PF-27-S01 implementation evidence

Candidate commits: `c0f2e02e4a` (`feat(gpu): add GLM 5.3 Flash H200 preset`) and `64034f2e8a` (`feat(gpu): add experimental GLM B300 benchmark preset`)

Worktree: `/home/pfrpc/repos/CorbanuTerminal-glm53-flash`

Branch: `feat/glm-5-3-flash-vast-preset`

## Pinned deployment contract

| Field | Value |
| --- | --- |
| Recipe | `glm-5.3-flash-4xh200` |
| Model | `zai-org/GLM-5.3-Flash` |
| Model revision | `3f1971b7b5f7a528c9c4ef6212c8785298a8c24a` |
| Runtime | vLLM dedicated GLM-5.3 image |
| Image digest | `sha256:2c6da6c6f16ed15c91e412d896dba13701f25fe1861eaec9ddaa4db34d1d21c4` |
| Hardware | 4× NVIDIA H200, allocation-local NVLink topology required |
| Context / concurrency | 65,536 tokens / 4 requests |
| Authentication | generated per-rental `PFT_ENDPOINT_TOKEN`; Vast key remains Vault-backed |
| Inference protocol | authenticated OpenAI-compatible `/v1/chat/completions` over controller-owned SSH transport |

The H200 manifest reserves 328,326,771,576 bytes for pinned model weights,
120,000,000,000 bytes for Hopper BF16 KV cache, and 64,000,000,000 bytes for
runtime workspace. It does not request FP8 KV cache on Hopper.

The experimental `glm-5.3-flash-fp8-2xb300-experimental` manifest uses the same
pinned model and image, TP2 on exactly two NVLink-connected B300s, CUDA 13,
compute capability 10.3, driver 580.65.06 or newer, 131,072 maximum context,
256 maximum sequences, FP8 KV cache, and a 32,768-token scheduler batch bound.
It remains experimental until the bounded live sweep succeeds.

The benchmark artifact under `qa/gpu-rentals/benchmarks/glm53-b300/` runs two
closed-loop waves at 4, 8, 16, 32, 64, 128, and 256 streams. Its deterministic
50/25/12.5/12.5 mix targets 1K/2K, 8K/6K, 32K/8K, and 96K/20K input/output
buckets. That is a weighted 18,656 input tokens and exactly 6,000 requested
output tokens per request. It records aggregate and per-stream output throughput
plus TTFT, TPOT, and end-to-end percentiles without prompts, completions, endpoint
tokens, or provider credentials.

## Final-tree automated evidence

| Command | Result |
| --- | --- |
| `cd codex-rs && just fix -p codex-gpu-market` | pass |
| `cd codex-rs && just fix -p codex-tui` | pass; pre-existing TUI warnings only |
| `cd codex-rs && just fmt` | pass |
| `cd codex-rs && just test -p codex-gpu-market recipe` | 12 passed |
| `cd codex-rs && just test -p codex-gpu-market` | 77 passed |
| `python3 qa/gpu-rentals/benchmarks/glm53-b300/run_mixed_sweep.py --validate-only` | pass; seven levels, 1,016 total requests, weighted output exactly 6,000 tokens |
| `cd codex-rs && just test -p codex-tui gpu_menu` | 9 passed; reviewed catalog snapshot included |
| `python3 docs/plans/check.py` | active 2/2; pass |
| `python3 docs/sprints/check.py` | 22 current, 81 archived; pass |

## True-TUI evidence

Command:

```bash
CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all \
  tmux_gpu_menu_lists_glm_5_3_h200_and_b300_presets_then_cancels --retries 0
```

Result: one test passed in 1.464 seconds with zero retries (nextest run
`681b8e49-19c3-4dd9-a3cd-5f98e7aed0d9`).

The typed harness launched the real candidate in a private tmux server with an
isolated `CODEX_HOME`, `RUST_LOG=trace`, an isolated `log_dir`, and a 140×44
terminal. It sent `/gpu` as literal text, sent Enter separately, waited for the
stable visible checkpoint `Rent zai-org/GLM-5.3-Flash · 2× NVIDIA B300`,
confirmed both H200 and B300 recipe ids plus their qualified/experimental
states, sent Escape, and waited for the menu to disappear. It did not enter
spend limits, search providers, or create a billable resource.

## Remaining live evidence

The live Vast search and rental require the product owner to supply the exact
maximum hourly USD price, maximum total USD spend, and duration in minutes.
After final billable confirmation, evidence must record READY, the secret-free
4–256 benchmark results, observed cost, and provider-confirmed termination.
