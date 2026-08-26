# GLM-5.3-Flash 2×B300 mixed-concurrency benchmark

This directory contains the reproducible qualification workload for the
qualified native-FP8 2×B300 preset. It measures authenticated streaming
chat inference at 4, 8, 16, 32, 64, 128, and 256 concurrent requests.

## Workload

Each concurrency level runs two closed-loop waves: 2 × concurrency total
requests, with no more than concurrency requests outstanding. The deterministic
mix is:

| Share | Class | Target input | Requested output |
| ---: | --- | ---: | ---: |
| 50% | short | 1,024 | 2,000 |
| 25% | medium | 8,192 | 6,000 |
| 12.5% | large | 32,768 | 8,000 |
| 12.5% | long | 96,000 | 20,000 |

The weighted target is 18,656 input tokens and exactly 6,000 output tokens per
request. This is a controlled stress mix, not a claim that one universal
production distribution exists. It deliberately combines interactive,
document, and long-context traffic while preserving the requested 6,000-token
mean.

The harness calibrates each synthetic prompt through the serving runtime's
/tokenize endpoint. Unique text appears at the start of every prompt so prefix
caching cannot turn the workload into a shared-prefix benchmark. Streaming
requests set ignore_eos and use server-reported usage, so early end-of-sequence
tokens do not invalidate requested output lengths.

For every level, summary.json and summary.csv record:

- completed and failed requests;
- measured mean input and output tokens;
- aggregate output tokens/second;
- effective output tokens/second per configured stream;
- request throughput;
- median, p90, and p99 TTFT, TPOT, and end-to-end latency.

Per-request, content-free timing and token evidence is written to
concurrency-N.json. Prompts, generated text, credentials, and endpoint secrets
are never written.

## Why this load method

vLLM documents request-rate infinity with a maximum concurrency as its
maximum-throughput/closed-loop mode, and recommends ignore-eos for synthetic
fixed-output tests. NVIDIA Perf Analyzer defines concurrency mode the same way:
a fixed number of outstanding requests is maintained. MLPerf's
Llama-3.1-405B text-generation scenario permits outputs up to 20,000 tokens, so
the long bucket remains inside that established large-model benchmark bound.

Primary references:

- https://github.com/vllm-project/vllm/blob/main/docs/benchmarking/cli.md
- https://docs.nvidia.com/deeplearning/triton-inference-server/archives/triton-inference-server-2670/user-guide/docs/perf_analyzer/docs/inference_load_modes.html
- https://github.com/mlcommons/inference_policies/blob/master/inference_rules.adoc

## Run

Validate the matrix without an endpoint:

    python3 qa/gpu-rentals/benchmarks/glm53-b300/run_mixed_sweep.py \
      --validate-only

After the normal /gpu workflow reports READY, export the endpoint token in the
process environment, then run:

    PFT_ENDPOINT_TOKEN='...' \
      python3 qa/gpu-rentals/benchmarks/glm53-b300/run_mixed_sweep.py \
      --base-url http://RENTAL_HOST:PORT/v1 \
      --result-dir qa/gpu-rentals/benchmarks/glm53-b300/results/RUN_ID

Do not put the token in a command-line argument, shell history, result path, or
evidence file. Run levels in the declared order. The harness writes results
after every completed level and stops at the first level with any failed or
short output.

The live rental must be created through the product's bounded /gpu confirmation
flow. Record the approved hourly, total-spend, and duration caps before
confirmation; after the sweep, terminate through /gpu and verify the provider
reports no remaining billable instance.

## Qualified live result

The 2026-08-26 Vast run is preserved under
`results/20260826-vast-48809614/`. All 1,016 requests completed with zero
failures. Aggregate output throughput increased from 245.36 tok/s at four
streams to 2,662.88 tok/s at 256 streams; per-stream throughput declined from
61.34 to 10.40 tok/s. The 256-stream mixed workload reached 99.9–100% KV-cache
occupancy and queued work without OOM, eviction, preemption, or short output.
Treat 256 as a validated stress ceiling rather than a production target with
memory headroom. See `summary.json` for all timing percentiles and
`qa/gpu-rentals/sprints/PF-27-S01/evidence.md` for rental and cleanup evidence.
