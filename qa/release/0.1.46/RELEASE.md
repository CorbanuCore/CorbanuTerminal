# Corbanu Terminal 0.1.46 — authorized provider and agent-loop fixes

## Authority and scope

On 2026-09-24 the requesting repository operator reviewed the defects below and
their benchmark evidence, called the tool-call loop "catastrophic", and said:
“I think we need to kick off the release in the background and then resume this
workflow as the other issues are super high urgency”. This is explicit human
release authorization under the root AGENTS.md release gate. The requesting
operator is the release owner.

Classification: **bounded fix**. Every change restores intended behavior of
existing provider routes or the existing tool-call guard. No user goal,
credential, financial, data or persistence boundary changes. Product
specification heading: **Shipping MVP — LIVE**, row **Multi-provider
inference**.

Branch: `release/corbanu-0.1.46`, cut from published `release/corbanu-0.1.45`
(`bff0fb7f30`). It contains seven fix commits plus this version bump. Main has
not absorbed 0.1.44 or 0.1.45 (PRs 123 and 124 open); the fixes are also on
PR 126.

## Defects fixed

1. **Vercel server-state continuations retried every turn**
   (`1152cb5d04`). Vercel rejects every `previous_response_id` continuation for
   some upstream models. The client retried the incremental form on every turn,
   so 44 of 98 Corbanu requests in a Kimi K3 benchmark were failed round trips.
   After the first rejection the session now stays on full-context requests.
2. **Synthetic `Continue.` user turn** (`96721e0bfc`). It was appended whenever
   history ended in model output, so the model read it as the user and re-checked
   finished work. It is now added only for a real assistant prefill, with a
   one-retry legacy fallback per session. Eight Vercel runs afterwards: 0
   `Continue.` requests, 0 rejected requests.
3. **Code-mode `exec` unusable over Chat Completions** (`e4eaec72d6`). GPT-6 Sol
   on OpenRouter calls the freeform `exec` tool as a function with an `input`
   string. The handler accepted only custom payloads, so every call was aborted:
   Queryforge scored 1/12 and one run looped for 235 requests. Both payload
   shapes now resolve to the same source (shared with `apply_patch`). Afterwards:
   4 of 4 runs passed, 38/38 hidden tests.
4. **Unbounded whitespace from strict freeform wrappers** (`bc661dd868`). The
   `{"input": string}` wrapper was sent with `strict: true`. One unescaped quote
   closed the string, after which constrained decoding allowed only whitespace:
   GPT-6 Sol streamed 65,536 whitespace tokens ($0.66, about 11 minutes) in one
   call, in 2 of 3 runs. The wrapper is no longer strict. Chat replay now keeps
   malformed calls and their error outputs so the model sees that its call
   failed. Afterwards: 4 of 4 runs passed; largest single completion 7,857
   tokens.
5. **GLM on Vercel failed for zero-data-retention accounts** (`333ae35e3f`).
   Corbanu pins `zai/*` models to Z.AI's own upstream so thinking controls take
   effect. Z.AI is not ZDR-eligible, so every GLM 5.3 and GLM 5.3 Flash request
   failed with `no_zdr_providers_available`. When the gateway's routing metadata
   shows it skipped every provider without attempting one, the session retries
   once without the pin and keeps gateway routing.
6. **Identical tool-call loops ran unbounded** (`af7dfff301`, `4cf5cee84f`).
   The identical-call guard counted repeats in the tool runtime, which is rebuilt
   for every model request, so a model re-issuing one call per request was never
   stopped. GLM 5.3 Flash on Vercel ran one command 156–250 times in a row in 4
   of 20 runs ($0.63–$1.35 per run against about $0.03 normally). The turn now
   tracks consecutive identical direct calls: after three in a row further
   repeats are refused without running; after eight the turn stops. Polling tools
   (`write_stdin`, code-mode `wait`, agent `wait`/`list_agents`, `sleep`,
   `wait_for_environment`, `current_time`) are exempt. A fatal tool error now ends
   the turn instead of being logged and ignored, which also makes the existing
   malformed-call stop take effect.

## Evidence

- Regression tests added for each fix, including an end-to-end test in which the
  model issues the same call in consecutive responses (calls 4–7 refused, turn
  stopped at the eighth).
- Full `codex-core` nextest run on the fix commits (8 MiB stack, local profile):
  3,521 tests. Every failure also fails with the fixes reverted or passes in
  isolation: 18 environment-dependent tests (missing `codex` test binary for
  `cli_stream`, sandbox and permission suites, timing-sensitive code-mode and
  unified-exec tests) and `spawn_agent_allows_depth_up_to_configured_max_depth`,
  which predates 0.1.45.
- Leak-isolated benchmarks (`benchmarks/coding`, per-run containers, relay with
  recorded cost; Corbanu and Hermes at the same explicit reasoning level):

| Model and route | Hidden tests, Corbanu vs Hermes | Cost, Corbanu vs Hermes |
| --- | ---: | ---: |
| Opus 5.5, OpenRouter | 36 vs 36 of 38 | $1.96 vs $2.14 |
| GPT-6 Sol, OpenRouter | 38 vs 38 | $1.20 vs $1.30 |
| Kimi K3, OpenRouter | 33 vs 35 | $1.21 vs $2.11 |
| GLM 5.3, OpenRouter (same host pinned) | 37 vs 36 | $1.74 vs $1.59 |
| GLM 5.3 Flash, OpenRouter (same host pinned) | 36 vs 35 | $0.11 vs $0.16 |

Two tasks, two waves each. These are diagnostics, not a qualifying benchmark.

## Disclosed gaps

- The loop guard stops a model that repeats one call. A model that cycles
  through several calls with unchanged results, or toggles an edit back and
  forth, can still run long: in one post-fix GLM 5.3 Flash run the guard refused
  14 repeats but the model kept cycling. A result-aware guard is in progress
  and is not in this release.
- Candidate binaries used for the benchmarks were built locally; release
  binaries are built by the release workflow.
- Full workspace tests, cross-platform true-TUI qualification and the release
  suite in both default live repositories are not claimed.
- The 0.1.44 cancellation-recovery limitation is carried forward unchanged.
- The competitor/model benchmark cycle remains incomplete; see
  [benchmarks](benchmarks/README.md).
