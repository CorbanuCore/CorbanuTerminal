# GGUF fine-tune qualification — 2026-07-15

Status: experimental, suitable for internal stress testing. Neither recipe is
production-qualified. Both source repositories describe abliterated models;
operators must treat their outputs as reduced-safety, untrusted model output.

## Product changes

- Added immutable experimental recipes for
  `huihui-ai/Huihui-DeepSeek-V4-Flash-abliterated-ds4-GGUF` and
  `huihui-ai/Huihui-GLM-5.2-abliterated-GGUF`.
- Separated the immutable artifact source ID from the model identity exposed by
  a serving runtime. This supports fine-tunes whose repository name cannot be
  used as the runtime's wire-level model alias.
- Made the runtime wire API part of the immutable recipe. Existing manifests
  default to Chat; verified recipes may explicitly select Chat or Responses.
- Preserved the selected wire API through the rental controller, state overlay,
  and core model-provider loader. The previous loader silently forced every GPU
  runtime to Chat.
- Marked maturity independently from manifest integrity. An experimental recipe
  still has pinned artifacts, bounded capacity, scoped auth, and deterministic
  launch behavior, but it does not inherit the claims of a qualified recipe.
- Increased the readiness chat output budget from 32 to 256 tokens so reasoning
  models must reach final answer content instead of passing on reasoning-only
  output.

## Immutable recipes

### Huihui DeepSeek V4 Flash Q4_K

- Source revision: `f06f59bce3c36b3282b75c9fe2621c83c9399d10`.
- File: `Huihui-DeepSeek-V4-Flash-BF16-abliterated-ds4-Q4_K.gguf`.
- Bytes: `164633502304`.
- SHA-256: `39f3e232fc6c4e1a42a60bf7842c86bd3f1db53e7231e07047f430a713a5f97e`.
- Runtime: `agtico/ds4@f17f6fe8758bd4d00439546de2e7904c9ee38fb0`,
  built for `sm_90`, two H200s, tensor parallel 2, 131072-token context.
- Wire identity: `deepseek-v4-flash`; wire API: Responses.
- Disk KV snapshots are intentionally disabled. The Huihui TP fork serializes
  only GPU 0 for that path, so enabling it would claim multi-GPU recovery that
  the runtime does not implement.

The original pinned Huihui TP runtime failed under a real PfTerminal tool loop.
After two successful tool turns it returned HTTP 500 with
`layer-slice KV position mismatch: have 130413 want 73069`. Inspection showed
that its multi-GPU sync appended each full replayed API prompt to the existing
checkpoint. The pinned PfTerminal fork now extends only an exact prefix shared
by every GPU slice; otherwise it resets every slice and rebuilds from token
zero. The CUDA runtime compiled successfully on the rented H200 host.

### Huihui GLM 5.2 UD-IQ1_M

- Source revision: `2f0aff2760627f87faf16f8adc81caca7d7b10f6`.
- Six `UD-IQ1_M` shards, total expected download `231226309536` bytes, each
  verified against its recipe SHA-256.
- Runtime: `ggml-org/llama.cpp@c3d47e696b1187a27e896aa828d48ff9a33fc679`,
  built for CUDA with complete-layer split across two H200s.
- Wire identity: the source repository ID; wire API: Chat; 32768-token context.
- Row splitting failed because this CUDA quant path does not expose split
  buffers. The recipe uses the proven layer split rather than guessing from the
  model name.

Both runtimes bind to loopback. Pinned Caddy `2.11.4` with archive SHA-256
`527fbf917c39189a1e3b31d34fa955601680b2d5c8055d2a87b8b9588dec7bb9`
enforces the scoped bearer token on every externally visible route and proxies
streaming without buffering.

## Live Vast qualification

The final lanes used two H200s with NVLink and exact source-built runtimes.
The official optimized DeepSeek rental that predated this qualification was not
modified.

Both final endpoints passed:

- no token on `/v1/models`: HTTP 401;
- wrong token: HTTP 401;
- exact model identity;
- ordinary final answer content;
- SSE stream ending in `[DONE]`;
- forced `readiness_probe` tool call;
- cancelled long stream followed by a healthy model probe;
- multi-turn nonce recovery (`ORCHID-731`).

Direct 256-token generation samples on the final hosts:

- DeepSeek Q4_K TP2: 38.10, 37.95, and 38.31 tokens/second.
- GLM 5.2 IQ1_M layer split: 36.06, 43.05, and 43.05 tokens/second.

Fresh PfTerminal `0.1.11` debug sessions used isolated homes. The final rebuilt
debug binary has SHA-256
`fc98e2e5c7c35b8661b763457e3fc4963e727821c0f1e350e20de020fe62bc82`.

- DeepSeek Responses: four consecutive turns completed and the endpoint stayed
  healthy. Turns one and two invoked real shell tools. Turn three did not obey
  the requested tool contract: it printed a shell block and fabricated a line;
  turn four still completed normally. This is a model/tool-adherence defect,
  not a transport failure, and keeps the recipe experimental.
- GLM Chat: a fresh bounded turn invoked `pwd`, returned the observed directory,
  and a second user turn completed exactly. A separate long free-form inspection
  remained transport-stable but over-explored until the 32k context repeatedly
  compacted. That planning/convergence limitation also keeps the recipe
  experimental.

Local evidence was captured under `/tmp/pft-gguf-qual-evidence-20260715` and is
not committed because it contains raw session and provider diagnostics.

## Automated evidence

- `just test -p codex-gpu-market`: 61 passed in the final pre-format run.
- `just test -p codex-state gpu_runtime_providers`: 4 passed.
- `just test -p codex-core gpu_runtime_provider`: 3 passed.
- `just test -p codex-tui gpu`: 7 passed in the preceding implementation run.
- A preceding full `just test -p codex-tui` run completed 3258/3264 with six
  unrelated existing flaky failures. No `.snap.new` files remain.

## Remaining limits

- These are reduced-safety fine-tunes and are intentionally hidden behind an
  experimental label.
- DeepSeek TP disk KV restore remains disabled until every GPU slice is saved
  and restored atomically.
- DeepSeek tool adherence was 2/3 in the four-turn TUI stress conversation.
- GLM's 32k recipe is adequate for bounded agent work, but the full PfTerminal
  prompt plus a long exploratory loop can force repeated compaction.
- Model metadata warnings seen in direct-config test homes are not the product
  rental path: the GPU overlay supplies a dynamic model preset and context size.
