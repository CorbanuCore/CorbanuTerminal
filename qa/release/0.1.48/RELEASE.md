# Corbanu Terminal 0.1.48 — authorized rate-limit retry, MiMo and GLM Flash

## Authority and scope

On 2026-09-24 the requesting repository operator was asked whether to cut
0.1.48 with the shared-pool 429 retry fix and answered "yes". This is explicit
human release authorization under the root AGENTS.md release gate. The
requesting operator is the release owner.

Classification: **bounded fix**. The changes add a catalog entry, a per-model
prompt addition for GLM 5.3 Flash routes, and a bounded retry for one class of
gateway error. No credential, financial, data or persistence boundary changes.
Product specification heading: **Shipping MVP — LIVE**, row **Multi-provider
inference**.

Branch: `release/corbanu-0.1.48`, cut from `release/corbanu-0.1.47`
(`72e205fe11`). It contains three commits cherry-picked unchanged from
`fix/responses-synthetic-continue-only-on-prefill` plus this version bump.

## Changes

1. **Transient shared-pool 429s are retried** (`dfc5e57335`). OpenRouter returns
   429 with `error.metadata.limit_source = "upstream_provider_shared_pool"` when
   a shared upstream pool is saturated and asks callers to retry shortly. 0.1.47
   ended the turn on the first such response. Requests now retry these marked
   429s up to four times with 2/4/8/16 s backoff, honoring Retry-After (capped at
   60 s), on the Responses, Chat Completions and Anthropic Messages paths. Every
   other 429, including the caller's own quota and usage limits, keeps the
   existing no-retry behavior.
2. **MiMo V2.6 Pro on OpenRouter** (`c4c3413c67`). Adds `xiaomi/mimo-v2.6-pro`:
   1,048,576-token context, 131,072 max output, text and image input, low/high
   reasoning (default high).
3. **GLM 5.3 Flash work pattern** (`1466896957`). A short work pattern after the
   standard base's opening paragraph for the Z.AI, OpenRouter and Vercel GLM 5.3
   Flash routes. Other models keep the unchanged standard base.

## Evidence

- Rate limits, live: DeepSeek V4.1 Flash over OpenRouter pinned to Together,
  ten concurrent agents, released 0.1.47. Corbanu fully passed 0 of 13 tasks;
  11 runs ended within one second on a single shared-pool 429 that told the
  caller to retry shortly. Hermes rode out the same limits in the same window.
  The same comparison at four concurrent agents saw no 429s.
- Rate limits, tests: `only_shared_pool_rate_limits_are_transient`,
  `transient_rate_limit_delay_backs_off_and_honors_retry_after`,
  `transient_shared_pool_rate_limit_is_retried` and
  `other_rate_limits_are_not_retried` pass. The two integration tests abort
  with a stack overflow under this host's default test-thread stack, as does
  the pre-existing `history_dedupes_streamed_and_final_messages_across_turns`;
  all pass with `RUST_MIN_STACK=16777216`.
- MiMo: `codex-models-manager` tests and the OpenRouter model-picker snapshot
  pass.
- GLM 5.3 Flash, isolated harness, 10 tasks, same host and reasoning level:
  11 of 30 full passes as shipped; 13 of 20 with the work pattern; Hermes 19 of
  30. Disabling reasoning replay instead made it worse (6 of 20).

## Disclosed gaps

- The live rate-limit retry was not re-run against real shared-pool 429s with a
  release binary; the fix is covered by the mock-server tests above.
- GLM 5.3 Flash evidence is 20 runs with the work pattern, not a full 30-run
  cycle.
- Full workspace tests, cross-platform true-TUI qualification and the release
  suite in both default live repositories are not claimed.
- The 0.1.44 cancellation-recovery limitation is carried forward unchanged.
- The competitor/model benchmark cycle remains incomplete; see
  [benchmarks](benchmarks/README.md).
