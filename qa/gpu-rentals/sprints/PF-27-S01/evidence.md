# PF-27-S01 implementation evidence

Candidate commit: `c0f2e02e4a` (`feat(gpu): add GLM 5.3 Flash H200 preset`)

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

The manifest reserves 328,326,771,576 bytes for pinned model weights,
120,000,000,000 bytes for Hopper BF16 KV cache, and 64,000,000,000 bytes for
runtime workspace. It does not request FP8 KV cache on Hopper.

## Final-tree automated evidence

| Command | Result |
| --- | --- |
| `cd codex-rs && just fix -p codex-gpu-market` | pass |
| `cd codex-rs && just fix -p codex-tui` | pass; pre-existing TUI warnings only |
| `cd codex-rs && just fmt` | pass |
| `cd codex-rs && just test -p codex-gpu-market recipe` | 11 passed |
| `cd codex-rs && just test -p codex-gpu-market` | 76 passed |
| `cd codex-rs && just test -p codex-tui gpu_menu` | 9 passed; reviewed catalog snapshot included |
| `python3 docs/plans/check.py` | active 2/2; pass |
| `python3 docs/sprints/check.py` | 22 current, 81 archived; pass |

## True-TUI evidence

Command:

```bash
CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all \
  tmux_gpu_menu_lists_glm_5_3_flash_tp4_and_cancels_without_renting --retries 0
```

Result: one test passed in 1.462 seconds with zero retries (nextest run
`13fee981-1a25-47dc-ac16-cbf43b62f9ff`).

The typed harness launched the real candidate in a private tmux server with an
isolated `CODEX_HOME`, `RUST_LOG=trace`, an isolated `log_dir`, and a 140×44
terminal. It sent `/gpu` as literal text, sent Enter separately, waited for the
stable visible checkpoint `Rent zai-org/GLM-5.3-Flash · 4× NVIDIA H200`,
confirmed the recipe id and qualified state, sent Escape, and waited for the
menu to disappear. It did not enter spend limits, search providers, or create a
billable resource.

## Remaining live evidence

The live Vast search and rental require the product owner to supply the exact
maximum hourly USD price, maximum total USD spend, and duration in minutes.
After final billable confirmation, evidence must record READY, an authenticated
chat completion, the secret-free endpoint base URL, and provider-confirmed
termination.
