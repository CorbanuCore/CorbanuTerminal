# GPU market implementation report — 2026-07-15

## Outcome

The GPU rental market is implemented on `feat/gpu-market-v2` and was qualified
with paid H200 capacity through both provider adapters. The product offers only
three curated, pinned recipes:

- `deepseek-flash-2xh200` on 2×H200;
- `deepseek-flash-4xh200` on 4×H200;
- `glm-5.2-fp8-8xh200` on 8×H200.

The exact final candidate binary has SHA-256
`a3aa2bee3fb3d5a4e9ab44a114f84f73ca837ee411a31e1d9c18e4b0af82caaa`.
The final free-form TUI qualification ran from `2026-07-15T06:48:58Z` through
`2026-07-15T07:19:09Z`. The rental then reached provider-confirmed termination,
its runtime-provider overlay was removed, and Vast reported zero active
instances.

## Live provider evidence

### DeepSeek V4 Flash

- Vast 4×H200 manual manifest proof: resource `44936108`; SGLang TP4 with
  Marlin reached READY and served authenticated inference; terminated.
- RunPod 2×H200 adapter proof: pod `48amrfr8ryfb5x`; the pinned SGLang TP2
  recipe passed chat, streaming through `[DONE]`, a forced tool call, five
  dependent turns, client cancellation/recovery, and wrong-token rejection;
  terminated with HTTP 204 and absent from subsequent inventory.
- Vast product qualification: rental
  `gpu-6875ba78-0da1-4b36-bf1b-e3f69a92f04b`, resource `44950014`, accepted
  hourly price `$7.4358`. Two simultaneous PfTerminal processes selected the
  runtime model and completed multiple ordinary, long, tool-backed repository
  tasks from the exact final binary.
- The final product session exercised concurrent generation, client
  cancellation/recovery, dependent follow-ups, repeated automatic context
  compaction, and continued inference after compaction. One captured compaction
  occurred at `total_usage_tokens=59108` / `estimated=57991` with
  `reason=ContextLimit phase=MidTurn`; the subsequent turn completed normally.
- A preceding Vast host stopped before readiness. Rental
  `gpu-1930768c-e6c5-4837-acca-064b44708f0a`, resource `44944774`, was detected
  as a provider-side failure, entered cleanup, and reached provider-confirmed
  termination without being reported READY.

The DeepSeek runtime is pinned to model revision
`60d8d70770c6776ff598c94bb586a859a38244f1` and SGLang image digest
`sha256:015f39a45844be5a7b35270c56dc4d9ebcfe9b0c21a3b4f877a4ee22e795bd7a`.
The qualified runtime cap is 65,536 tokens. PfTerminal applies its normal 95%
safety margin, giving an effective context limit of 62,259 tokens. READY
requires authenticated model identity, chat, streaming, cancellation,
recovery, and structured tool-call probes—not merely an open port.

### GLM 5.2 FP8

- Vast resource `44939217` ran the official `zai-org/GLM-5.2-FP8` weights on
  8×H200 with SGLang TP8 and a 131,072-token product context.
- NVLink/P2P gates, authenticated chat, streaming, a forced tool call,
  dependent turns, cancellation/recovery, and wrong-token rejection passed.
- The resource was terminated after proof.

## Product behavior proved live

- `/gpu` performs bounded price/total/TTL authorization and shows provider,
  accepted hourly price, lifecycle, and global accrued-spend state.
- Vast and RunPod are presented as infrastructure providers; the model picker
  presents the rented model rather than leaking provider implementation detail.
- A newly READY runtime provider appears in an already-running session and in a
  fresh process. No restart or static `config.toml` mutation is required.
- Starting with a stale persisted `gpu-*` provider falls back to the static
  provider with a visible warning instead of bricking startup.
- Multiple processes share the durable ledger while controller leases
  serialize remote effects.
- Vast provisions a token-authenticated SGLang endpoint behind a verified HTTPS
  tunnel. RunPod uses its authenticated HTTPS proxy boundary.
- SGLang stream payloads that encode optional collections as JSON `null` are
  decoded as empty collections by a generalized deserializer. The regression
  covers both `tool_calls` and `reasoning_details`.
- Runtime endpoint context capacity is stored with the provider overlay and
  clamps the per-turn context limit. This prevents a model-card maximum from
  exceeding the capacity of the actual rented recipe.

## Live defects found and repaired

The free-form runs found four boundary failures that formulaic readiness probes
did not expose. Each was fixed generally and covered by regression tests:

1. Terminal rentals left a stale runtime-provider row. Termination now removes
   the overlay in the same transaction as terminal state, and reconciliation
   prunes historical stale rows.
2. A full readiness conversation used as a health poll could fail while the
   endpoint was legitimately saturated by user generation. READY endpoints now
   receive lightweight authenticated health/model probes; the full contract is
   reserved for readiness and recovery.
3. A stale third-party request lease could block a hot runtime `gpu-*` provider.
   Runtime GPU providers use their rental/controller ownership boundary instead
   of an unrelated static-provider request lease.
4. The advertised model context (384K) exceeded the qualified recipe runtime
   (65,536), causing a real HTTP 400 at 65,606 tokens. Runtime-provider state now
   carries endpoint capacity and clamps each turn before request construction.
   The exact live reproduction then auto-compacted and continued successfully.

## Invariant matrix

| Invariant | Automated evidence | Live evidence |
| --- | --- | --- |
| One authorization creates at most one resource | Durable client operation IDs, ownership tags, controller leases, replay/concurrency and ambiguous-create tests | Each product rental had at most one Vast resource; failed cleanup did not duplicate creation |
| Potentially billable resources remain discoverable | Owned-inventory adoption and unrelated-resource rejection tests | RunPod and Vast resources reconciled by provider inventory and durable rental IDs |
| Termination requires provider proof | Termination timeout, ambiguous-delete, inventory-absence, and atomic-overlay tests | Final product rental reached `terminated_confirmed`; Vast inventory returned zero active instances |
| No unauthenticated public inference | HTTPS-only runtime overlay plus no-token/wrong-token readiness probes | RunPod and Vast endpoints rejected wrong credentials and accepted scoped credentials |
| Secrets stay out of ordinary state and logs | Redacted secret types, narrow vault labels, sanitized adapter errors | Qualification logs contained no endpoint or provider credential |
| Price is authorization | Expired quote, price drift, exact-offer, hourly/total/TTL tests | Final rental recorded `$7.4358/h`, `$15` cap, and TTL |
| Rental control is process-independent | Durable controller leases and two-runtime serialization tests | Two TUIs and one controller used the same ready rental concurrently |
| READY means PfTerminal-compatible | Full authenticated readiness-contract tests | Real chat, stream, tool, cancellation, recovery, model identity, and post-compaction continuation passed |
| Runtime rentals never mutate static config | Runtime-selection and stale-config regressions | Static provider remained Ambient while two sessions used rented DeepSeek |
| Retries are bounded and persisted | Backoff, retry-after, step-digest, notification-dedup tests | Provider host loss converged to cleanup; no flood or invisible retry loop occurred |

## Automated qualification

- `just test -p codex-gpu-market`: 57/57 passed.
- `just test -p codex-state gpu`: 10/10 passed.
- `just test -p codex-tui gpu`: 7/7 passed.
- Focused `codex-api` SGLang null-collection regression passed.
- Focused core regressions for hot runtime-provider injection, runtime context
  capacity/clearing, GPU lease bypass, and lease release passed.
- `cargo clippy -p codex-gpu-market -p codex-state --tests -- -D warnings`
  passed.
- `git diff --check` passed.

The complete shared-workspace suite was not run because repository instructions
require explicit approval. A broader core clippy invocation was blocked by the
pre-existing `collapsible_match` warning in
`codex-api/src/endpoint/anthropic_messages.rs:413`, outside this change.

## Final live qualification

- Exact candidate: SHA-256
  `a3aa2bee3fb3d5a4e9ab44a114f84f73ca837ee411a31e1d9c18e4b0af82caaa`.
- Window: `2026-07-15T06:48:58Z`–`2026-07-15T07:19:09Z`.
- Two simultaneous TUI processes plus the durable controller remained alive.
- Both sessions completed multiple long, generic repository tasks and exercised
  automatic compaction. One session also passed cancellation/recovery.
- The endpoint remained `ready` throughout the window under concurrent load.
- Final log query found zero non-manifest WARN/ERROR records in either TUI
  process. The only warnings were pre-existing plugin-manifest validation
  warnings about oversized `defaultPrompt` values.
- Termination was requested through the `/gpu` TUI. State reached
  `terminated_confirmed`, the runtime-provider row count became zero, and Vast
  inventory reported `active_count=0` and `target_present=false`.

DeepSeek did occasionally emit a raw `<tool_calls>` fragment and one inaccurate
piece of prose after compaction. Those are model-output quality findings, not
hidden transport successes: subsequent tool use continued, the TUI stayed
responsive, and no request/compaction loop or crash occurred.

## Spend and cleanup

The final product rental accrued an estimated `$11.741651`. Durable product
rental estimates total `$35.914201`, including all successful and failed
lifecycle attempts. Manual provider proofs consumed less than `$40`, so the
entire implementation run remained below `$76`, safely inside the authorized
`$200` cap. No paid resource remains active on RunPod or Vast.

## Security incident requiring user action

During masked RunPod credential entry, the modal closed unexpectedly and the
credential was pasted into a TUI conversation. It was removed from the
PfTerminal vault and is not present in this patch, but exposure cannot be
undone. The RunPod key must be revoked and replaced in the RunPod console before
future RunPod use.

## Release disposition

The implementation and machine-driven qualification are complete. Human UAT
and any release/merge decision remain separate gates. The branch is pushed for
review; this report makes no release claim.
