# PF-27-S01 implementation and live-qualification evidence

Candidate implementation commits:

- `c0f2e02e4a` — `feat(gpu): add GLM 5.3 Flash H200 preset`
- `64034f2e8a` — `feat(gpu): add experimental GLM B300 benchmark preset`
- `67399f3106` — `fix(gpu): remove obsolete GLM vLLM logging flag`
- `ca22482a8b` — `fix(gpu): run GLM endpoints in text-only mode`
- `e29e769c74` — `fix(gpu): publish GLM endpoint readiness phase`
- `d4b20009f3` — `fix(qa): count provider reasoning stream tokens`
- `a50f24527a` — `feat(gpu): qualify GLM B300 preset`
- `d79f8721a8` — `fix(qa): write benchmark CSV with LF`

Worktree: `/home/pfrpc/repos/CorbanuTerminal-glm53-flash`

Branch: `feat/glm-5-3-flash-vast-preset`

## Pinned deployment contract

| Field | H200 preset | B300 preset |
| --- | --- | --- |
| Recipe | `glm-5.3-flash-4xh200` | `glm-5.3-flash-fp8-2xb300` |
| Model | `zai-org/GLM-5.3-Flash` | `zai-org/GLM-5.3-Flash` |
| Model revision | `3f1971b7b5f7a528c9c4ef6212c8785298a8c24a` | same |
| Runtime | dedicated vLLM GLM-5.3 image | same |
| Image digest | `sha256:2c6da6c6f16ed15c91e412d896dba13701f25fe1861eaec9ddaa4db34d1d21c4` | same |
| Hardware | 4× connected NVIDIA H200 | 2× connected NVIDIA B300 |
| Context / max sequences | 65,536 / 4 | 131,072 / 256 |
| KV cache | Hopper-safe BF16 | FP8 |
| Authentication | generated per-rental token | generated per-rental token |
| Protocol | authenticated OpenAI-compatible chat over controller-owned SSH transport | same |

The billable qualification used the pre-promotion recipe id
`glm-5.3-flash-fp8-2xb300-experimental` with the same immutable `r4` launch.
After the gate passed, `a50f24527a` promoted the catalog entry to the stable id
`glm-5.3-flash-fp8-2xb300` and qualified status without changing that launch.

Both launch recipes run in language-model-only mode, supervise the vLLM child,
forward shutdown signals, and publish `endpoint_probing` only after an
authenticated local `/v1/models` check succeeds. The endpoint token is supplied
to local readiness curl configuration over stdin rather than process arguments.

## Final-tree automated evidence

| Command | Result |
| --- | --- |
| `cd codex-rs && just fix -p codex-gpu-market` | pass |
| `cd codex-rs && just fix -p codex-tui` | pass; pre-existing TUI warnings only |
| `cd codex-rs && just fmt` | pass |
| `cd codex-rs && just test -p codex-gpu-market` | 78 passed |
| `python3 qa/gpu-rentals/benchmarks/glm53-b300/test_run_mixed_sweep.py` | 4 passed |
| `python3 qa/gpu-rentals/benchmarks/glm53-b300/run_mixed_sweep.py --validate-only` | pass; seven levels, 1,016 requests, weighted output exactly 6,000 tokens |
| `cd codex-rs && just test -p codex-tui gpu_menu` | 9 passed; reviewed `r4` catalog snapshot included |
| `CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all tmux_gpu_menu_lists_glm_5_3_h200_and_b300_presets_then_cancels --retries 0` | 1 passed, zero retries |

## Live rental authorization and hardware

The product owner approved maximums of $16/hour, $125 total, and 480 minutes.
The live rental was created only through the normal `/gpu` selection, limits,
offer review, and final confirmation flow.

| Field | Observed value |
| --- | --- |
| Rental id | `gpu-1bf5c8f8-4e81-4bfb-8c7d-3700188d438d` |
| Vast offer id | `48685789` |
| Vast resource id | `48809614` |
| All-in rate | $15.15625/hour |
| Location / verification | Utah / verified |
| GPUs | 2× `NVIDIA B300 SXM6 AC`, 275,040 MiB each |
| Compute capability / driver | 10.3 / 610.57.04 |
| Interconnect | NV18 between GPU 0 and GPU 1 |
| Budget result | accepted rate below $16/hour; final estimate below $125; duration below 480 minutes |

The pinned checkpoint downloaded 62 shards totaling 305.79 GiB. Each worker
loaded 150.23 GiB of model data. vLLM selected DeepGEMM FP8 kernels,
FlashInfer sparse MLA, and FP8 KV cache. Engine initialization took 649.61
seconds after model download, including graph profiling, DeepGEMM warmup, and
FlashInfer autotuning. The runtime reported a 12,694,323-token KV cache and
96.85× concurrency at the full 131,072-token context limit.

## Live canary corrections

The first live launch identified three generalized deployment-boundary defects:

1. The pinned vLLM build rejected an obsolete request-logging flag. Both GLM
   recipes removed it and received immutable revision bumps.
2. The multimodal checkpoint attempted to initialize a native processor even
   though this preset exposes text inference. Both recipes now use vLLM's
   language-model-only mode.
3. `exec vllm serve` prevented the wrapper from publishing endpoint readiness.
   Both recipes now supervise vLLM, perform authenticated local readiness, then
   publish `endpoint_probing`, with signal forwarding and child-status
   propagation.

A diagnostic rendered the original endpoint credential, so it was treated as
compromised and rotated through the masked Vault UI. The server and controller
were restarted with the replacement credential, the retired credential was
rejected, and the replacement passed the full readiness contract. No credential
value is present in repository artifacts. The benchmark warmup also exposed the
provider's `reasoning` stream field; the harness now recognizes generated delta
fields independently of one OpenAI SDK schema revision, with regression tests.

## READY and authenticated contract

The replacement controller created a fresh loopback SSH transport and the
endpoint reached READY. The full readiness sequence proved:

- missing and intentionally wrong endpoint credentials returned 401;
- the Vault-backed rental credential returned 200 from `/v1/models`;
- model identity matched `zai-org/GLM-5.3-Flash`;
- normal chat, streaming completion, cancellation recovery, and forced tool
  call requests returned 200;
- periodic authenticated health checks continued to pass during load.

## Mixed-context benchmark

Results are committed under
`qa/gpu-rentals/benchmarks/glm53-b300/results/20260826-vast-48809614/`.
The deterministic closed-loop workload used two waves per level and the
50/25/12.5/12.5 input/output mix described in the benchmark README. All 1,016
requests completed with exactly 6,000 mean output tokens and zero failures.
Prompts, completions, credentials, and endpoint tokens were not written.

| Streams | Requests | Duration (s) | Aggregate output tok/s | Tok/s per stream | Median TTFT (s) | p90 TTFT (s) | Median TPOT (ms) | p90 TPOT (ms) | Median E2E (s) | p90 E2E (s) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 8 | 195.63 | 245.36 | 61.34 | 7.65 | 13.75 | 10.14 | 12.16 | 56.13 | 120.79 |
| 8 | 16 | 212.91 | 450.89 | 56.36 | 0.92 | 3.96 | 12.09 | 13.29 | 43.70 | 143.04 |
| 16 | 32 | 283.93 | 676.22 | 42.26 | 3.18 | 14.90 | 17.25 | 18.50 | 68.31 | 236.88 |
| 32 | 64 | 380.38 | 1,009.53 | 31.55 | 1.95 | 7.94 | 21.67 | 29.65 | 79.18 | 275.51 |
| 64 | 128 | 525.12 | 1,462.52 | 22.85 | 4.61 | 12.99 | 32.11 | 40.49 | 103.33 | 331.74 |
| 128 | 256 | 731.50 | 2,099.80 | 16.40 | 24.77 | 43.72 | 44.04 | 62.29 | 161.25 | 459.93 |
| 256 | 512 | 1,153.64 | 2,662.88 | 10.40 | 20.62 | 148.55 | 73.71 | 108.89 | 339.44 | 728.94 |

Aggregate throughput increased at every level. At 256 streams the engine
reached 6.5K–7.1K aggregate generation tok/s during fully loaded intervals, but
mixed-context refill drove KV-cache occupancy to 99.9–100% and queued up to 67
requests. No OOM, eviction, preemption, timeout, short output, or request failure
occurred. Consequently, 256 streams is a validated stress ceiling for this mix,
not a recommended no-headroom production target. The closed-loop aggregate
metric includes prefill, refill stalls, and the 20K-token long-request tail.

## Spend and provider-confirmed cleanup

The `/gpu` termination detail showed an estimated $28.3518 immediately before
termination, about 112.24 minutes at the exact $15.15625/hour rate. This was
below all approved limits. Termination was requested through `/gpu`, which kept
billing unresolved until the controller confirmed provider absence.

Cleanup evidence:

- Corbanu Terminal displayed `termination is provider-confirmed` and removed the
  rental from the active/potentially billable list.
- A read-only Vast instance-list check returned no resource `48809614`.
- Unrelated Vast resources `48790553` and `48790554` remained running and were
  untouched.
- The controller-owned SSH tunnel exited.
- Vault lookup for the per-rental endpoint-token label returned absent.

This completes PF-27-S01 live endpoint, benchmark, spend, and cleanup evidence.
