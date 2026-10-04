# PF-60-S01 accounting contract — proposal v1

Acceptance update: Travis approved the v1 defaults in this task on September 11.
The [manager decision](../../plans/workstream-manager-handoff-2026-09-11.md#accounting-decision)
supersedes pending-policy wording in the original proposal below. S01 is accepted
and archived; the reviewed S02 handoff allocates only a test-only native journal.
No billing, live collection or production migration is enabled.

Status: proposed for Travis Good; source inspection and synthetic fixture evidence
only. This is an unfinished research contract, not shipped feature documentation,
a migration, a provider billing assertion, or human acceptance. S02 and later
sprints remain blocked until the manager obtains acceptance and archives S01.

Product citation: **Product measurement**, [product specification](../../corbanu-product-spec.md#product-measurement):
“No commercial performance numbers have been supplied.” The active
[PF-60 plan](../../plans/active/portfolio-agent-cost-accounting.md) authorizes this
bounded preparation, with [PF-60-S01](../../sprints/archive/portfolio-agent-cost-accounting/pf-60-s01-accounting-contract-and-golden-fixtures.md)
originally allocated `in_progress`. Change class: product initiative.

Source baseline: `f17e5a54daba11c6da2554f14b62f11ace752142`, verified clean on
`workstream/accounting-pf60-s01-20260911` in
`/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`.
The allocation records retain ancestor `295aed26e53b17f919f7199ae1c9748b1b1250ba`;
the launch explicitly authorized fast-forward to the verified merged main above.
The manager owns those records. No private logs/accounts, credentials, live
pricing, network calls, other worktrees, or additional models were used.

## Observed native boundaries

All paths below are repository-relative source at the baseline. Symbols identify
the mapping durably; none establishes that complete request accounting exists.

| Native source / symbol | Fields and meaning | Accounting disposition / missing evidence |
| --- | --- | --- |
| `codex-rs/protocol/src/protocol.rs`, `TokenUsage`, `TokenUsageInfo`, `TokenCountEvent` | `input_tokens`, `cached_input_tokens`, `cache_write_input_tokens`, `output_tokens`, `reasoning_output_tokens`, `total_tokens`; `last_token_usage` and cumulative `total_token_usage`; optional `info` and `rate_limits` | Retain types and compatibility. Native integers cannot express field absence; cache-write deserialization defaults to zero. Add provenance/presence at a thin future adapter, not inferred zero. A snapshot is never another charge. |
| `codex-rs/codex-api/src/sse/responses.rs`, `From<ResponseCompletedUsage> for TokenUsage` | `input_tokens_details.cached_tokens` and `.cache_write_tokens`; `output_tokens_details.reasoning_tokens`; provider total | Input includes cache components; reasoning is an output subset for this contract. Absent details become native defaults, losing presence. Preserve raw numeric field presence before conversion in S02; never capture prompt/content. |
| `codex-rs/codex-api/src/endpoint/anthropic_messages.rs`, `AnthropicUsage::merge_into` | Optional `input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`, `output_tokens` | Native input = noncached + read + write; start/delta fields merge, not add. Compatible-provider cumulative input heuristic exists. Adapter must identify dialect; ambiguous compatible input stays unknown. Reasoning field is set to zero by native conversion, not separately measured. |
| `codex-rs/codex-api/src/endpoint/chat_completions.rs`, `From<ChatUsage> for TokenUsage` | `prompt_tokens`, `completion_tokens`, `total_tokens`, `prompt_tokens_details.cached_tokens`, `completion_tokens_details.reasoning_tokens` | Retain optional-field provenance; current defaults and fixed cache-write zero do not prove measured zero. The Corbanu API fixture uses Chat, matching `create_pfterminal_plan_provider` in `model-provider-info/src/lib.rs`; its cache-write value remains unknown. |
| `codex-rs/core/src/session/turn.rs`, completion branch in `try_run_sampling_request` | `ResponseEvent::Completed { response_id, token_usage, finish_reason, end_turn }` creates `ModelResponseCompleted` and `RawResponseCompleted`, then records usage | Completion usage is recorded before some finish-reason errors. A failed/retried completion can still consume tokens. Interrupted streams may lack final usage. Preserve consumed attempts regardless of turn success. |
| `codex-rs/protocol/src/protocol.rs`, `ModelResponseCompletedEvent`, `RawResponseCompletedEvent`; `codex-rs/rollout/src/policy.rs`, `should_persist_event_msg` | Model completion persists `turn_id`, `response_id`, `model`, `model_provider_id`, optional `finish_reason`; raw completion has `response_id`, optional `token_usage` | Model completion and TokenCount persist; **RawResponseCompleted is transient**. Durable request identity plus usage cannot be assumed from their adjacency. No historical one-to-one join guarantee or request price snapshot found at this seam. |
| `codex-rs/core/src/session/mod.rs`, `record_token_usage_info`, `send_token_count_event`, `last_token_info_from_rollout`; `codex-rs/state/src/extract.rs`, `apply_event_msg` | Session accumulates usage, emits snapshots, restores latest nonempty info; `threads.tokens_used = total_token_usage.total_tokens.max(0)` | Retain cumulative display/recovery. `TokenUsageInfo::fill_to_context_window` can synthesize context totals; thread totals are not an invoice or authoritative request delta ledger. Never subtract neighboring snapshots to invent request costs. |
| `codex-rs/state/migrations/0040_provider_request_state.sql`, `0041_provider_request_cache_usage.sql`; `codex-rs/state/src/runtime/provider_requests.rs` | PK `(provider_id, model, key_fingerprint)`; latest preflight `last_input_tokens`, `last_cached_input_tokens`, `last_request_bytes`, `last_thread_id`, `last_turn_id`; lease/cooldown/status/error request ID; `last_provider_input_tokens`, `last_provider_cached_input_tokens` | Retain throttle authority. Rows overwrite per key/model, not per request. Lease acquisition clears provider counts to zero; success clears request ID, stores only input/cache via COALESCE. No output, cost, history, retry ledger or completeness. Do not repurpose this table as accounting or export fingerprints. |
| `codex-rs/core/src/session/turn.rs`, `provider_request_key`, `rough_prompt_token_estimate`, lease predicates and result helpers | Config provider ID/model; key fingerprint (sometimes shared placeholder); rough bytes/4 preflight; selected metered subagent leases; failure `status`, `request_id`, `retry_after_ms` | Fingerprints/lease owner are not durable billing-account or request identity. Coverage is conditional; missing row says nothing about free work. Preflight is estimated, distinct from provider measurement. |
| `codex-rs/core/src/session/turn.rs`, sampling retry loop; `codex-rs/codex-client/src/retry.rs`, `run_with_retry`; `codex-rs/codex-api/src/endpoint/session.rs` | `same_turn_attempt_index = retries + 1`; transport has its own attempt counter and retry policy | Retain retry behavior. A turn has multiple sampling calls and nested transport retries. Turn ID plus retry index is insufficient; S02 must assign identity at each actual dispatch, linked to a sampling operation. |
| `codex-rs/protocol/src/protocol.rs`, `SessionMeta`, `TurnContextItem`, `SubAgentSource::ThreadSpawn` | Native `id`/`session_id`, `parent_thread_id`, `forked_from_id`, `source`, `history_base`, `subagent_history_start_ordinal`; turn ID/model/provider; spawn parent/depth/path | Use thread IDs for ownership, parent edges for aggregation. Forked/inherited history is context, not new child consumption. Model/provider may be absent historically; current session configuration cannot fill past values. |
| `codex-rs/state/migrations/0021_thread_spawn_edges.sql`; `codex-rs/state/src/runtime/threads.rs`, spawn-edge methods | `child_thread_id` primary key, `parent_thread_id`, `status`; upsert and source-derived backfill; deletion removes edges | Retain native hierarchy, not a second scheduler. Validate cycles, unknown parents and conflicting lineage; retained accounting must preserve enough lineage after deletion. Campaign membership is not established by these seams and needs explicit scope later. |
| `codex-rs/app-server/src/request_processors/token_usage_replay.rs` | Latest persisted TokenCount attributed to active turn ID, falling back to rebuilt turn position or latest completed/failed turn | Retain connection-scoped notification replay; deliberately bypasses `send_event` to avoid new persisted usage. Fallback turn attribution is for UI shape, not proof of a billable request. |
| `codex-rs/app-server-protocol/src/protocol/v2/thread.rs`, `ThreadTokenUsage`; `codex-rs/tui/src/token_usage.rs`; `codex-rs/tui/src/chatwidget/usage.rs` | v2 includes cache-write; local TUI token model lacks it and displays blended noncached input + output; usage menu includes ChatGPT reset credits | Retain existing behavior in S01. Future S03 must audit all consumers; blended/context metrics and reset credits cannot price requests. No UI qualification here. |
| `codex-rs/protocol/src/protocol.rs`, `RateLimitSnapshot`, `RateLimitWindow`, `CreditsSnapshot` | Limit ID, primary/secondary used percent, window/reset; provider credits balance/unlimited; spend-control state | Provider allowance snapshot only; no conversion to USD or task allocation without an explicit provider contract. Absence means unavailable. |
| `codex-rs/wallet/src/corbanu_api.rs`, `CorbanuApiBalance`, `CorbanuApiPricing`, `CorbanuApiAccount` | Decimal-string `balance_microusd`, `reserved_microusd`, `available_microusd` and USD strings; model input/output/cache-read/cache-write USD strings plus `version` | Canonical Corbanu API account semantics. Keep balance, reservation and availability distinct; do not subtract a local estimate from any of them. No invoice/request-settlement mapping or effective interval in these DTOs. |
| `codex-rs/tui/src/chatwidget/wallet_api.rs`, `load_read_only_account`, menu rendering | Source describes `/v1/models`, keyed `/v1/account`, wallet-account match; per-million-token pricing and in-flight reservation display | Inspected source only: no endpoint invoked. Catalog display says upstream cost/zero markup; that text is not verified pricing evidence. Account absence currently has a UI zero fallback; accounting proposal instead preserves unavailable. |
| `codex-rs/model-provider-info/src/lib.rs`; `codex-rs/tui/src/chatwidget/wallet_usage.rs` | Compatibility ID `pfterminal-plan` remains; old WalletPlan weekly/monthly token entitlements also remain in source | Retain canonical routing identifiers; label that route Corbanu API in this contract. **Exclude legacy Plan entitlements from this accounting design.** No old Plan allowance restoration or conversion of prepaid USDC to measured task spend. |

## Decisions proposed for Travis: vocabulary and unknowns

1. **Measured usage** means provider-reported numeric token fields with a source
   reference, wire dialect, field presence and observation time. It does not
   certify provider billing. Use `null` plus a reason (`not_reported`,
   `interrupted`, `legacy_presence_lost`, `unattributed`, `conflict`) for absent or
   ambiguous fields. Explicit reported zero is valid. Known partial fields remain
   inspectable; absence must not be converted to zero during aggregation.
2. **Estimated usage** (e.g. preflight bytes/4) is a separate quantity with method
   and version, never substituted for measured counts. **Estimated cost** is
   token arithmetic using an immutable, identified price snapshot, even when the
   tokens are measured. State which components are priced and which are missing.
3. **Billed cost** requires an authoritative settlement/invoice line identity,
   currency, amount and request allocation, separately from estimates. A payment
   top-up is funding, not request spend. No provider invoice ingestion or native
   bill-to-response join was verified. Corrections must be explicit revisions or
   adjustments, not overwrite estimates or silently rebill history.
4. **Provider allowance** is the provider's capacity/credit/window snapshot with
   its own unit and freshness. It is not estimated cost, an invoice, or Corbanu
   API money. Unknown allowance stays unknown.
5. **Corbanu API balance** is a gateway account snapshot: balance, reserved and
   available as returned, with account scope and fetch/as-of provenance. Keep
   exact micro-USD strings and USD display strings. Never derive task spend from
   balance deltas: top-ups, concurrent requests and reservation release interfere.
   If `balance - reserved != available` or representations disagree, retain the
   source and flag it rather than repair it. `balance_rate` is qualitative only.

## Proposed request identity and replay contract

These proposed fields are fixture-contract fields, **not existing persisted
native schema**. S02 must adapt native events and state rather than create a
parallel usage authority. No migration number or public schema is reserved here.

- A logical sampling request gets an opaque durable `request_id`, owned by one
  native `thread_id` and `turn_id`. Multiple model calls in a turn get different
  request IDs. A retry chain retains request identity.
- Each actual outbound dispatch gets a durable `attempt_id` before dispatch,
  and optional `retry_of` to its predecessor. Include provider configuration ID,
  endpoint/account scope (opaque local identity, never credential fingerprint),
  exact model, wire dialect, and dispatch timestamp. Retry may consume tokens
  even on 429, cancellation, timeout, or terminal provider error. Local preflight
  denial before dispatch is a separate non-dispatch fact, not a fabricated
  measured zero. Every attempt's observed usage counts once.
- Provider response/request IDs are optional correlation fields scoped to
  provider/account/endpoint. They are not the primary key and do not authorize
  suppressing a distinct attempt. Reused provider IDs across ambiguous attempts
  require reconciliation, not a silent merge. The fixture tests a missing ID and
  reused IDs without collapsing attempts; provider reconciliation remains S02.
- Each observation has `(attempt_id, revision)` plus provenance; revisions are
  durable source sequence, not arrival order or a derived wall-clock rank.
  Identical duplicate keys are no-ops; conflicting payloads for the same key or
  changed immutable identity fail visibly. Sort revisions, merge cumulative
  field patches by replacement, and preserve omitted fields. Never add a stream
  snapshot to its previous snapshot. Null patches mean unavailable, not erase a
  previously reported count; retractions need an explicit future correction type.
- Native raw usage is lost in current rollout persistence. S02 must commit
  identity, observation and replay checkpoint atomically using native state;
  reconnect notification must never create an attempt. Import legacy snapshots
  separately with original lineage/source position and unknown coverage. Do not
  invent retry history from cumulative totals or attach inherited parent usage
  to a child. Reordering requires preserved original revision; absent sequence
  cannot be guessed. Crash between dispatch and receipt remains unknown.
- A subtree is the set of unique native thread IDs reachable through validated
  spawn edges, including the root. Sum each owned attempt once. Never sum a
  parent's displayed inclusive subtotal with its children's subtotals. Reject
  cycles/multiple parents; quarantine orphans with incomplete coverage. Historical
  forks/context copies carry no new charge. A rollback hides context, not spend.
- Expose each field's known subtotal and unknown-attempt count. Full total is
  null when any participating attempt/component is unknown; do not call a known
  subtotal the total. Billed and estimated subtotals never add to one another.
  Absent requests/lineage may make coverage unknown even when observed rows sum.

## Decisions proposed for Travis: pricing, precision and retention

Propose USD-only estimates initially; reject cross-currency addition, no implicit
FX. Store token counts as nonnegative exact integers and money/prices as exact
decimal strings, never binary floats. Keep native Corbanu balance in integer
micro-USD without recasting token estimates to prematurely rounded micro-USD.
Propose exact rational token-price arithmetic (integer coefficient/scale), sum
before display rounding, and display rounding half-even at six USD decimals
with an explicit rounded marker. Nonzero sub-micro amounts must not look free.
Authoritative billed amounts retain issuer precision and currency.

For explicitly supported inclusive-input dialects, price disjoint buckets:
`noncached = input - cache_read - cache_write`; reasoning is already included in
output. Estimate USD = `(noncached * input_rate + cache_read * read_rate +
cache_write * write_rate + output * output_rate) / 1,000,000`.
Reject negative counts, cache sums above input, reasoning above output, malformed
prices and conflicting provider totals for these synthetic dialects. If a cache
split is unknown, input cost is unknown; known output may still be priced. A zero
bucket needs no price. Different cache TTL tiers, tool/search/media fees, regional
or volume rates, separate reasoning billing and taxes need explicit later pricing
contracts; absence must not be silently treated as zero. Synthetic fixtures
declare token-only pricing; this is not a claim that real invoices lack fees.

Propose immutable price records with `price_id`, provider/account scope, exact
model, currency, rates/unit, provenance, capture time, inclusive `effective_from`
and exclusive `effective_to`. Select by dispatch time, not replay time; a gap,
overlap, missing model/account, unsupported currency or unknown dispatch time
leaves price unknown/rejected. Persist selected price identity with each estimate.
Changes to a catalog cannot alter an old estimate; retrospective re-estimation
must create a separate version with explicit consent. The native catalog's
`pricing.version` helps identify provenance but supplies no effective dates:
never backdate it or infer historical rates from today's catalog. Fixtures use
invented rates and a deliberately absent Corbanu model price.

Propose 90 days of local numeric request observations, identity/deduplication and
lineage, and 365 days of daily aggregates plus referenced price versions. This is
a decision proposal, not existing retention or a deletion job. Beyond raw retention,
show an explicit drill-down cutoff. Keep compact deduplication tombstones for the
entire accepted replay horizon; reject older imports after that horizon or they
can double count deleted attempts. Travis must decide horizon, deletion semantics,
user export/control and whether aggregates survive thread deletion. No prompts,
tool bodies, secret fingerprints, wallet addresses, account keys or private logs
belong in accounting provenance. S01 deletes nothing and starts no collector.

## Executable boundary and next sprint dependencies

[Synthetic QA](../../../qa/portfolio/agent-cost-accounting/pf-60-s01/README.md)
contains raw inputs, independently written literal expectations, arithmetic,
reference reducer and nonempty unit tests. It demonstrates the proposed rules
in Python only. No Rust/native adapter, database crash durability, TUI workflow,
live repository, actual billing reconciliation, benchmark or human acceptance is
proved. Even a fixture restart roundtrip is not native persistence qualification.

Manager / Travis decisions required: accept or revise the vocabulary, accounting
unit/unknown policy, retention/replay horizon, precision/display rounding, currency,
approved price source and effective-date policy, historical import scope, lineage
deletion policy, and billed-evidence authority. Independent recomputation and
combined-tree fixture/governance checks remain with the manager.

S02 requires accepted/archived S01, a verified upstream SHA (currently unresolved;
fork HEAD is not upstream), serialized native schema/manifest ownership and exact
adapter allocation. Adapt presence capture at provider conversion, dispatch/retry
identity and durable numeric observation at native Core/state seams; retain
protocol compatibility, throttle state, spawn hierarchy and connection-scoped
replay. Test retries at both layers, crash/restart and duplicate import, historical
unknowns, cancellation, compaction, rollback and inherited history. No upstream
code is changed by S01; upgrade testing is therefore not applicable here, but is
required before those runtime edits. S03 needs S02 persisted attribution and
full cache-write/unknown display support. S04 needs one recorded binary, actual
interactive flows, applicable TensorCash/Isometric qualification, benchmark/release
evidence and named-human acceptance; all remain pending.
