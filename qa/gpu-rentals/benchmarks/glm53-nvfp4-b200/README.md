# GLM-5.3-Flash NVFP4 two-B200 matched evaluation

This directory owns secret-free evidence for the experimental
`LibertAIDAI/GLM-5.3-Flash-NVFP4` route. The evaluation deliberately reuses
the existing GLM-5.3-Flash workloads so quantized-route observations remain
comparable without changing task packets after seeing results.

## Frozen workloads

- Coding: EventForge, LogTriage, and QueueCraft from `benchmarks/coding/tasks/`.
- Website: `benchmarks/website-builder/task_prompt.md` and its independent verifier.
- Serving load: the exact deterministic 4, 8, 16, 32, 64, 128, and 256 stream
  workload in `../glm53-b300/run_mixed_sweep.py`.

The serving sweep writes content-free per-request timings and token counts.
Prompts, completions, endpoint addresses, marketplace credentials, and rental
tokens must not enter this directory.

## Immutable route

| Field | Value |
| --- | --- |
| Checkpoint | `LibertAIDAI/GLM-5.3-Flash-NVFP4` |
| Revision | `aa28e1f54130286c95fee10d0705c74ce8743734` |
| Safetensors | 194,660,206,040 bytes (181.291 GiB) |
| Runtime image | `vllm/vllm-openai@sha256:2e771fa615452282cc331eb418b3ef21636fce355bea0491fca89e6d362ab703` |
| Candidate hardware | 2× NVLinked NVIDIA B200, TP2 |
| Recipe | `glm-5.3-flash-nvfp4-2xb200-experimental` r1 |

## Run after READY

Resolve the rental token through the Vault only in the benchmark process
environment, then execute:

```bash
PFT_ENDPOINT_TOKEN='resolved-without-printing' \
python3 qa/gpu-rentals/benchmarks/glm53-b300/run_mixed_sweep.py \
  --base-url http://RENTAL_LOOPBACK/v1 \
  --result-dir qa/gpu-rentals/benchmarks/glm53-nvfp4-b200/results/RUN_ID
```

Run coding and website workloads separately from the serving sweep. Do not
interpret contaminated concurrent traffic as a matched quality or capacity
comparison. The rental must be created through `/gpu` after fresh exact
hourly, total-spend, duration, offer, and final confirmation, then terminated
through `/gpu` until provider-confirmed absence.
