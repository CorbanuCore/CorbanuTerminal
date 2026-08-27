# Qwen3.8-27B-FP8 2×H200 mixed-concurrency benchmark

This directory contains the reproducible qualification workload for the
experimental native-FP8 2×H200 preset. It measures authenticated streaming
chat inference at 4, 8, 16, 32, 64, 128, and 256 concurrent requests.

## Workload

The workload, evidence contract, and closed-loop method are identical to the
qualified GLM-5.3-Flash B300 benchmark in `../glm53-b300/`. Each concurrency
level runs two closed-loop waves: 2 × concurrency total requests, with no more
than concurrency requests outstanding. The deterministic mix is:

| Share | Class | Target input | Requested output |
| ---: | --- | ---: | ---: |
| 50% | short | 1,024 | 2,000 |
| 25% | medium | 8,192 | 6,000 |
| 12.5% | large | 32,768 | 8,000 |
| 12.5% | long | 96,000 | 20,000 |

The weighted target is 18,656 input tokens and exactly 6,000 output tokens per
request. Prompts are uniquely prefixed to defeat shared-prefix caching; the
harness calibrates through the runtime `/tokenize` endpoint and uses
`ignore_eos` with server-reported usage so requested output lengths hold.

The only model-specific differences from the GLM benchmark are the pinned
served identity (`Qwen/Qwen3.8-27B-FP8`), the immutable model revision
`017b9c7af6b5689d5dd426a76e0bc077eb5ca20a`, and the 262,144-token context
bound exposed by this preset.

## Run

Validate the matrix without an endpoint:

    python3 qa/gpu-rentals/benchmarks/qwen38-27b-h200/run_mixed_sweep.py \
      --validate-only

After the normal `/gpu` workflow reports READY, export the endpoint token in
the process environment, then run:

    PFT_ENDPOINT_TOKEN='...' \
      python3 qa/gpu-rentals/benchmarks/qwen38-27b-h200/run_mixed_sweep.py \
      --base-url http://RENTAL_HOST:PORT/v1 \
      --result-dir qa/gpu-rentals/benchmarks/qwen38-27b-h200/results/RUN_ID

Do not put the token in a command-line argument, shell history, result path, or
evidence file. Run levels in the declared order. The harness writes results
after every completed level and stops at the first level with any failed or
short output.

The live rental must be created through the product's bounded `/gpu`
confirmation flow. Record the approved hourly, total-spend, and duration caps
before confirmation; after the sweep, terminate through `/gpu` and verify the
provider reports no remaining billable instance.
