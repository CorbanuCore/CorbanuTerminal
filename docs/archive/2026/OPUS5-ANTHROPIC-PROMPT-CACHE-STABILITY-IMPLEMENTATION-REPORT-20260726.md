# Opus 5 prompt-cache stability implementation report

Date: 2026-07-26 UTC
Repository baseline: `67aa0522d76224e6b9b94daf52086c54a12df5d5`
Specification:
`OPUS5-ANTHROPIC-PROMPT-CACHE-STABILITY-SPEC-20260726.md`
Scored specification hash: final delivered copy
`4d1a84f58fa2f28273a8934e34bbd396e73e7b755ffceedb1a8885127779cf05`
Text Improvement Harness score: **92.20/100**
Qualification result: **PASS for the cache-stability defect**

## 1. Outcome

The cache-breaking edit fallback is repaired at the tool-schema boundary.

Fallback-capable strict-patch turns now expose `apply_patch`,
`structured_edit`, and `structured_write` from request one. Two consecutive
grammar failures change only turn-local dispatch policy. They do not rebuild,
remove, add, reorder, or rewrite model-visible tools.

Three independent live `claude-opus-5` waves exercised the exact failure
sequence against a 123K-token prefix. The first request after activation read:

| Wave | Transition input | Cached input | Hit rate | Uncached suffix |
|---:|---:|---:|---:|---:|
| 1 | 124,268 | 124,072 | 99.8423% | 196 |
| 2 | 124,265 | 124,069 | 99.8423% | 196 |
| 3 | 124,264 | 124,068 | 99.8423% | 196 |

There was no zero-cache transition in any wave. All three waves used identical
prompt and tool-schema hashes, emitted exactly one activation metric, completed
the structured edit, and left the locally rejected strict-patch target absent.

This directly reverses the incident behavior:

| Evidence | Before | After |
|---|---:|---:|
| Transition cache hit | 0 / 118,536 and 0 / 110,177 | at least 124,068 / 124,264 |
| Two incident transition writes | 228,713 tokens | 392 suffix tokens for two comparable transitions |
| Three post-fix transition writes | not applicable | 588 suffix tokens total |
| Transition schema mutation | yes | no |

## 2. Implementation

### Turn-local state machine

`codex-rs/core/src/session/turn_context.rs` now owns one `AtomicU8` state:

```text
0 -> no grammar failure
1 -> one consecutive grammar failure
2 -> fallback active, absorbing
```

Compare/exchange loops make failure, reset, and activation transitions
linearizable. Only the caller committing `1 -> 2` receives
`PatchFallbackTransition::Activated`. Parse success maps `0|1 -> 0` and cannot
deactivate state 2.

Focused concurrency coverage is in
`codex-rs/core/src/session/turn_context_tests.rs`.

### Immutable schema, mutable dispatch

`codex-rs/core/src/tools/spec_plan.rs` separates native structured-edit
availability from runtime fallback state:

- native Z.AI/GLM, Ambient, and Meta profiles retain structured-only tools;
- fallback-capable strict profiles register all three edit tools in stable
  order;
- mutable fallback state is never read while constructing model-visible specs.

`codex-rs/core/src/tools/handlers/apply_patch.rs`:

- resets the streak immediately after valid strict grammar parsing;
- increments only strict grammar failures;
- rejects direct strict calls locally before parsing or filesystem work when
  fallback is active;
- applies equivalent grammar/reset semantics to shell interception;
- distinguishes strict shell calls from patches generated internally by the
  structured-edit handler, preventing local enforcement from blocking the
  accepted fallback path.

`codex-rs/core/src/tools/handlers/structured_edit.rs` emits the one-time
activation metric:

```text
profile=<profile>
protocol=strict_apply_patch
outcome=fallback_activated
reason=consecutive_grammar_failures
consecutive_failures=2
tool_schema_changed=false
```

The tracked Rust diff plus the new state-machine test is 621 additions and 136
deletions, net +485 lines. Production implementation accounts for 239 added
lines; the remaining additions are generalized concurrency, shell-path,
request-byte, provider-planning, and seven-request integration coverage. Gross
churn therefore exceeds the specification's aspirational 500-line target while
net growth remains below it; no broader tool API refactor was introduced.

### Regression coverage

The seven-request integration sequence in
`codex-rs/core/tests/suite/tool_harness.rs` proves:

1. malformed patch;
2. valid patch resets the streak;
3. one later malformed patch does not activate;
4. the next consecutive malformed patch activates;
5. strict retry is locally rejected without mutation;
6. structured edit succeeds;
7. every complete parsed tool array and production-serialized tool byte string
   is identical.

The Anthropic request-construction test in
`codex-rs/core/src/client_tests.rs` separately appends history, rebuilds the
request, and compares the production-serialized `/tools` bytes including cache
control placement.

## 3. Deterministic verification

All test commands were run through `just test`, never direct `cargo test`.

| Command/scope | Result |
|---|---:|
| Seven named state, planning, integration, and Anthropic request tests | 7 / 7 passed |
| `test(=suite::prompt_caching::prompt_tools_are_consistent_across_requests)` | 1 / 1 passed |
| `test(/(structured_edit\|apply_patch\|prompt_caching)/)` with 8 threads | 108 / 108 passed |
| `git diff --check` | passed |
| `just fix -p codex-core` | completed; one task-local test cleanup applied |
| `just fmt` | passed |

The full `codex-core` crate suite was also attempted twice:

- default concurrency: 2,838 passed, 149 failed;
- bounded to eight test threads: 2,874 passed, 113 failed.

The remaining failures are not in `apply_patch`, `structured_edit`, prompt
caching, or the new state machine. Reproduced environmental/baseline groups
include missing `target/debug/codex` and `test_stdio_server` binaries, machine
catalog expectations that differ from the checked-in tests, missing registered
multi-agent threads, and shell/image/MCP timeout fixtures. The focused
cache-stability surface is green, but this report does not misrepresent the
unrelated full-crate suite as green.

Per repository instructions, no tests were rerun after `just fix` and
`just fmt`.

## 4. Live Anthropic proof

Frozen candidate:

```text
pfterminal 0.1.22
binary SHA-256 04516d4dba01be3d7ed0b63b6ca1052d197ba0eabc1fc9454bd2739e3c501cc1
prompt SHA-256 4e67a92ea51bcd6a0b75c5ebbf92e14e925c7a17deb882a05eda9a030cead82a
tool fingerprint 131a45b5c4b9e8e6f7d955797d2d28892e689d3a1d879e31c030da58f7c4b369
```

The prompt was 265,094 bytes and produced a 123K-token first request. Each
wave used a fresh workspace and `CODEX_HOME`. Waves alternated the two
previously attributed benchmark keys so a live five-minute cache from one wave
could not establish the next wave's full prompt.

Every wave:

- issued two consecutive malformed `apply_patch` calls;
- activated fallback exactly once;
- kept the same complete tool fingerprint;
- changed `proof.txt` from `ALPHA` to `OMEGA`;
- proved `forbidden.txt` did not exist;
- ended `PROBE_DONE`.

Wave 3 also issued a third, valid-looking `apply_patch` after activation. The
call was present in the rollout, was rejected locally, did not create its file,
and was followed by a successful `structured_edit`.

Settled Anthropic Admin Usage:

| Key ID | 5m cache creation | Cache read | Output | Uncached input | Cost |
|---|---:|---:|---:|---:|---:|
| `apikey_01ETPF8aNWXS17BDVpWkzPQD` | 244,234 | 1,373,162 | 1,950 | 26 | $2.2619235 |
| `apikey_01SkUWKM1VvMXAt2CR4H3Mk6` | 124,888 | 620,834 | 793 | 12 | $1.1108520 |
| **Campaign** | **369,122** | **1,993,996** | **2,743** | **38** | **$3.3727755** |

The preflight Admin window contained zero organization traffic. The settled
rows contain only the two known dedicated IDs and reconcile to the campaign
total.

## 5. Kimi and non-caching overhead

### Kimi Code K3

The original fresh-home TUI harness timed out before creating a session or
making a provider request. It is retained as a harness failure, not counted as
a Kimi failure or paid call.

The noninteractive core route then completed two real turns on the same Kimi
thread:

| Kimi request | Input | Cached | Hit rate |
|---|---:|---:|---:|
| First primary | 7,156 | 1,024 | 14.31% |
| Resumed primary | 7,326 | 6,912 | **94.35%** |

Kimi therefore exposes measurable cache reuse on the plan-backed K3 route; no
OpenRouter substitute was needed for that claim.

### Added stable-schema cost

Anthropic's `count_tokens` endpoint measured the Opus request with and without
the two pre-registered structured tools:

```text
baseline 122,088
stable   122,881
delta        793 Opus input tokens
```

At $6.25 per million five-minute cache-write tokens, the one-time Opus delta is
approximately **$0.004956**.

A controlled OpenRouter `meta-llama/llama-3.3-70b-instruct` pair reported zero
cached tokens in both arms:

```text
baseline 7,852
stable   8,503
delta      651 input tokens / 2,307 serialized tool bytes
```

This satisfies the specification's non-caching-provider measurement.

## 6. Economics

Historical defect:

```text
228,713 full-miss tokens * $6.25/M = $1.42945625
```

Post-fix three-wave transitions:

```text
588 uncached suffix tokens * $6.25/M = $0.003675
one-time added Opus schema = $0.00495625
```

The repair exchanges a sub-cent up-front schema cost for elimination of a
six-figure-token rewrite whenever fallback activates in a long turn.

Metered qualification spend:

| Service | Spend |
|---|---:|
| Anthropic three-wave campaign | $3.3727755 |
| OpenRouter PFTerminal Grok capture | $0.0195880 |
| OpenRouter Grok controlled pair | $0.0301208 |
| OpenRouter Llama controlled pair | $0.0021270 |
| **Known metered total** | **$3.4246113** |

Kimi used the existing plan and had no separately attributed marginal API bill.
The campaign stayed far below the authorized $200 ceiling.

## 7. Qualification artifacts

Run root:

```text
/home/pfrpc/repos/pfterminal-perf-probe/runs/opus5-cache-stability-fix-20260726T134520Z
```

Important files:

- `runbook.md`: frozen protocol, commands, pass criteria, and executed result;
- `campaign_summary.json`: normalized three-wave arithmetic;
- `admin_usage/campaign_settled.summary.json`: authoritative settled Anthropic
  usage;
- `live/anthropic-opus5-*/result.json`: per-wave tool calls, cache events,
  hashes, metrics, and filesystem proof;
- `live/anthropic-opus5-stdin/schema_delta_count.json`: Opus tokenizer delta;
- `live/kimi-k3-exec/cache_summary.json`: resumed Kimi cache evidence;
- `live/openrouter-grok-schema/schema_delta_llama33.json`: uncached-model token
  delta;
- `key_scan.json`: exact-key scan.

The exact-key scan loaded six raw credentials in memory and found zero hits
across the final artifact tree. Raw credentials do not appear in this report,
the specification, tests, requests, rollouts, scripts, or benchmark artifacts.

## 8. Requirement audit

### State and planning

- [x] One linearizable, absorbing turn-local state machine.
- [x] Consecutive grammar semantics with parse-success reset.
- [x] Exactly one activation transition under concurrency.
- [x] Stable initial registration of all fallback-capable edit tools.
- [x] Native structured-only providers retain their prior inventory.
- [x] Mutable fallback policy is absent from tool-plan generation.

### Enforcement and telemetry

- [x] Direct strict retry is rejected before parsing/filesystem work.
- [x] Shell strict parsing uses the same streak semantics.
- [x] Correctness, permissions, sandbox, and filesystem outcomes are not grammar
      failures.
- [x] Structured edits remain executable after activation.
- [x] Exactly one activation metric reports `tool_schema_changed=false`.
- [x] Existing provider cache-pressure warning remains present.

### Verification and economics

- [x] Parsed and production-serialized tool arrays remain identical.
- [x] Anthropic cache-control placement is covered by request construction.
- [x] State concurrency and full fallback sequence are covered.
- [x] Relevant 108-test editing/cache suite is green.
- [x] Three matched long-context Opus waves pass above the 90% gate.
- [x] Settled Admin Usage and per-key attribution are retained.
- [x] Kimi resumed cache reuse is measured.
- [x] First-request schema delta is measured on Opus and an uncached model.
- [x] Exact-key scan is clean.
- [!] Gross Rust churn is 621 additions/136 deletions (net +485), above the
      500-line target because both strict entry paths and concurrency required
      generalized tests; production code is 239 added lines.
- [!] Full `codex-core` remains red for documented unrelated
      baseline/environment fixtures; no cache-stability test is among the
      failures.

## 9. Release assessment

The specified defect is fixed and proven through deterministic request-body
tests, real local enforcement, three live Opus fallback waves, provider usage,
Kimi cache reuse, and an uncached-model overhead measurement.

The implementation is ready for review as a focused `codex-core` change. A
separate repository-health task should repair the pre-existing full-suite
environment/catalog failures; they should not be hidden inside or block
understanding of this cache-boundary remediation.
