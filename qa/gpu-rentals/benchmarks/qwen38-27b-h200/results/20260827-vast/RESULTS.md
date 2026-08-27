# Qwen3.8-27B-FP8 2×H200 live results — 2026-08-27 Vast 48834637

## Rental and termination

| Field | Value |
| --- | --- |
| Rental id | `gpu-7491fa47-b2cd-403f-b928-9433e9e8d7e0` |
| Recipe | `qwen3.8-27b-fp8-2xh200-experimental` r1 |
| Provider / offer | Vast offer `36326138`, resource `48834637` |
| Hardware | 2× NVIDIA H200, United States, verified |
| Authorized caps | $12.00/hour, $75.00 total, 360 minutes |
| All-in rate | $8.5033/hour |
| READY after | about 8.5 minutes |
| Estimated final spend | $19.5587 |
| Cleanup | `/gpu` termination requested; provider-confirmed `terminated_confirmed` at 2026-08-27T01:39:27Z; no remaining billable instance |

The endpoint served `Qwen/Qwen3.8-27B-FP8` through the controller's authenticated
loopback transport and passed the standard model, chat, stream, and cancellation
readiness checks before benchmarking.

## Mixed-concurrency sweep

The sweep used the deterministic workload in this directory. All 1,016 requests
completed with exactly 6,000 mean output tokens and zero failures.

The coding benchmark ran concurrently at the product owner's explicit request,
so these levels are deliberately contaminated by agent traffic. Treat them as a
loaded system measurement, not an isolated serving benchmark.

| Streams | Requests | Duration (s) | Aggregate output tok/s | Tok/s per stream |
| ---: | ---: | ---: | ---: | ---: |
| 4 | 8 | 208.38 | 230.35 | 57.59 |
| 8 | 16 | 235.91 | 406.93 | 50.87 |
| 16 | 32 | 315.39 | 608.77 | 38.05 |
| 32 | 64 | 507.98 | 755.93 | 23.62 |
| 64 | 128 | 877.05 | 875.66 | 13.68 |
| 128 | 256 | 1,469.27 | 1,045.42 | 8.17 |
| 256 | 512 | 3,395.87 | 904.63 | 3.53 |

Measured aggregate throughput peaked at 128 streams. The 256-stream level
completed but declined to 904.63 aggregate tok/s (3.53 tok/s per stream), so the
useful loaded-system capacity boundary is between 128 and 256 streams for this
hardware and workload. A clean isolated rerun is required before assigning a
qualified capacity claim.

Timing percentiles for every level are preserved in `summary.json`,
`summary.csv`, and `concurrency-N.json`. Prompts, completions, endpoint
addresses, and credentials are not included.

## Contaminated coding-benchmark observation

Corbanu Terminal ran EventForge, LogTriage, and QueueCraft against this
endpoint concurrently with the sweep. This was an exploratory observation, not
an isolated coding benchmark:

| Task | Pass | Wall seconds |
| --- | ---: | ---: |
| EventForge | no | 1,800.064 (timeout) |
| LogTriage | yes | 1,771.262 |
| QueueCraft | no | 278.06 |

The first attempt exposed a wire-boundary defect: Corbanu serialized
mid-conversation developer context as multiple `system` messages, while the
Qwen3.8 chat template accepts only one leading system message. The Chat
Completions adapter now coalesces all text system messages into one leading
message for non-OpenAI providers, with a regression test. Coding scores after
that fix remain contaminated by sweep contention and must not be compared to
isolated model scores.

## Conclusion

The recipe launched and served the pinned checkpoint cleanly under load. It
remains `experimental` because (1) throughput peaked before the configured
256-stream maximum, (2) coding capability under load was weak, and (3) no clean
isolated rerun or capacity-headroom claim has been recorded.
