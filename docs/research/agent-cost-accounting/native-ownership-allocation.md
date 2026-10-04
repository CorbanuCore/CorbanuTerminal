# PF-60-S02 — Native ownership and deletion bridge allocation

September 12, 2026. Manager-approved next bounded increment in the existing S02
reservation. This condenses the inspected private proposal, not another retention
prerequisite or C3. Only allocation documents are authorized now; parent checks
and integrates them before source implementation dispatch.

## Accepted input and exact coordinates

- Base: `d5608c58d91a75396fea78b447477c44091d4625`.
- Worker: `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`;
  branch: `workstream/accounting-pf60-s01-20260911`.
- C2's exact six files were committed as `b7a3466de` and integrated at that base.
  First independent Astra High `c2-atomic-review.json` exited 0, clean. Parent's
  receiving `just test -p codex-state -p codex-tasknode-session` passed 346/346,
  0 skipped, 13.556s, run `06b633c7-51ae-4dd8-bf96-81a2665c2871`. No LEAK marker
  appeared in the final summary; 11 existing fixture dead-code warnings remain.
  These are parent-supplied results, not commands rerun by this allocation.
- Preserve the [C2 receipt](../../../qa/portfolio/agent-cost-accounting/pf-60-s02/atomic-retention-increment.md),
  its original failures and 1519-total/190-non-test one-candidate disposition in
  `c2-size-disposition.md`, and every prior A/B/C1 review/receipt. C2 is accepted,
  not 800-line compliant; none of that allowance is reset or inherited here.
- [Active plan](../../plans/active/portfolio-agent-cost-accounting.md) and
  [current S02](../../sprints/current/portfolio-agent-cost-accounting/pf-60-s02-idempotent-usage-persistence-and-replay.md)
  own the reservation. Parent verifies disjoint scope, especially sole accounting
  ownership of `threads.rs`, and records source dispatch after integration.

## One concrete output and six future write paths

Connect completed accounting operations to real native thread ownership and the
native state deletion transaction. Keep the existing `runtime.rs` accounting
`#[cfg(test)]` gate intact: executable native integration proof, not normal-build
accounting installation, production activation, a collector or full S02 completion.

| Exact future repository path | Bounded work | Estimated added + deleted |
| --- | --- | ---: |
| `codex-rs/state/src/runtime/accounting.rs` | Register private native adapter; preserve low-level fixture admission | 5 |
| `codex-rs/state/src/runtime/accounting_native.rs` | New native-owner admission facade and installed-fixture deletion hook | 130 |
| `codex-rs/state/src/runtime/accounting_retention_atomic.rs` | Narrow forwarding to complete C2 operations and native test registration; no algorithm rewrite | 28 |
| `codex-rs/state/src/runtime/threads.rs` | Shared internal deletion/time seam and test-gated accounting transaction hook; estimated +95/-60 | 155 |
| `codex-rs/state/src/runtime/accounting_native_tests.rs` | New combined native/accounting failure, lifecycle and concurrency proof | 620 |
| `qa/portfolio/agent-cost-accounting/pf-60-s02/native-ownership-increment.md` | Exact final evidence, counts, digests, original failures and limits | 55 |
| **Total / conservative non-test** | **Estimate, not actual capacity** | **993 / 373** |

Approved target: **1200 total / 500 non-test**, including receipt and counting
moves as both additions and deletions. Above 800 is explicitly justified by one
coherent native/accounting failure and concurrency proof, not C2's exception.
Planning range is 900–1200 total / 330–470 non-test. Count test-gated runtime code
as non-test. Report actual overage to the integrator before expanding; do not
compress proof, silently widen paths or invent another prerequisite chain.
One new code review plus necessary scoped correction is authorized for this
increment; prior original failures and review usage remain recorded. No review
runs during this documents-only allocation pass.

## Existing seams and implementation contract

1. `accounting.rs::Journal::append_on_connection` retains its existing fixture
   semantics. A new internal native entry takes expected `ThreadId`, existing
   Attempt/observations and explicit as-of time. Under one `BEGIN IMMEDIATE`,
   require valid Active installed accounting, compare expected and attempt owner,
   and query native `threads.id`. Missing/mismatched owner fails without writes;
   never synthesize a thread. Complete maintenance and append share this connection,
   including rollback of maintenance-clock advancement on failed admission.
2. `accounting_retention_atomic.rs::{maintain_on_connection,delete_on_connection}`
   remain complete coupled operations. Thin inherent Journal forwarding with
   accounting-scoped visibility can expose them to the native child using existing
   inputs. No independently callable partial transfer, prepared-plan apply API,
   nested transaction-owning wrapper or public pricing/storage module chain.
3. `threads.rs::{delete_thread,delete_threads_strict}` retain public signatures,
   empty-input and missing-row cleanup behavior, return counts and last-delete
   ordering. Factor the existing internal body for explicit-time testing; the
   public accounting path captures one time, reused across all owners/operations.
   Acquire the installed-accounting state write reservation before owner checks
   and accounting deletion, and call complete C2 deletion before dynamic tools,
   spawn edges and native thread rows on the same transaction/connection.
   Repeated same-time maintenance per owner is acceptable. No early missing-row
   return may strand compact accounting state.
4. Normal-build/no-schema deletion keeps its existing behavior. The accounting
   hook and accounting-specific schema/activation/time checks stay `cfg(test)`;
   do not add blanket activation failures, new normal-build accounting queries or
   unconditional accounting-driven transaction policy to the absent-schema route.
   Only installed accounting takes the coupled path; partial/malformed schema,
   staging or invalid Active metadata fail closed, never implicitly activate.
5. `threads.rs::{upsert_thread,upsert_thread_spawn_edge,list_thread_spawn_descendants}`
   are native ownership/lineage authorities. Reuse `test_support::test_thread_metadata`
   without widening it. Expected owner is a native caller selection, not a new
   authenticated principal. No parallel thread/campaign registry. Legitimate
   `upsert_thread` can recreate a UUID: this bridge does not establish a permanent
   deleted-thread fence or authorize historical resurrection.
6. Logs through `logs_pool`, memory and goal cleanup precede the main-state
   transaction and use separate stores. Preserve those operations and retry graph.
   Later failure may leave earlier cleanup committed. Only native state tables
   and accounting are transactionally coupled; no cross-database/physical-erasure
   promise, ATTACH redesign or widened lifecycle authority.

No writes to migrations, `runtime.rs`, lib exports, manifests/locks, BUILD files,
Core/API/protocol/TUI, accepted tests/receipts or ledgers in the future code scope.
The two new Rust files use existing source inclusion. `threads.rs` belongs solely
to this accounting allocation while dispatched; manager checks other owners.

## Required new proof, without repeating A/B/C1/C2

Use actual native methods on an on-disk state database. Preserve existing proof
and reuse support read-only. Whole-state comparisons include all ten accounting
tables plus ordered `threads`, `thread_spawn_edges`, `thread_dynamic_tools` rows.
Exercise the shared deletion body with explicit as-of times and the unchanged
public wrapper with recent synthetic data; helper-only coverage is insufficient.

- Active parent/child admission and independent request IDs; expected-owner
  mismatch, missing/deleted owner rejects old replay and fresh UUID without writes
  or native row creation. Archived is not deleted.
- Native raw+compact and compact-only deletion after two actual closes/reopens;
  repeat/missing owner return counts and another owner's shared snapshot preserved.
- Multi-owner failure on second accounting cleanup, then exact-marker failures in
  native dynamic-tool, edge and thread DELETEs after accounting maintenance/deletion.
  Require full rollback, unchanged discoverable owner/graph, two reopens, removal
  of only the injected fault and successful retry. Include combined commit-time
  failure; accounting-only C2 proof does not substitute for wiring proof.
- Real append-held→native-delete and native-delete-held→append contention using
  distinct connections, channels and controlled peer busy_timeout=0. Observe actual
  SQLite BUSY/LOCKED before release, not sleeps. After deletion, replay and fresh
  UUID retries fail absent ownership. Held read snapshot sees complete old joint
  state; fresh snapshot sees joint deletion, not intermediate accounting absence.
- Earlier logs/memory/goals cleanup failure preserves accounting/native owner;
  later main-state failure explicitly verifies that earlier separate-store cleanup
  may persist while accounting/native state rolls back and retry remains possible.
- Negative/backward times, staging/partial/Active-NULL metadata and unsupported
  90..365-day admission do not advance maintenance/native metadata. Normal init
  installs no accounting schema; absent-schema native deletion behavior is unchanged.

Future verification only: recorded scoped `just fix -p codex-state` and literal
file formatting, then nonzero `just test -p codex-state retention_plan::reduction::atomic::native_tests`
(register `#[cfg(test)] #[path = "accounting_native_tests.rs"] mod native_tests;`
in the atomic child), `just test -p codex-state runtime::accounting`, exact native
selectors `delete_thread_cleans_associated_state` and
`delete_thread_keeps_retry_graph_on_cleanup_failure`, full `just test -p codex-state`,
and normal-library `cargo check --offline --locked -p codex-state --lib`.
Reuse cached Rust 1.95.0, offline/autoinstall OFF and assigned target; no direct
cargo test or environment repair. Record exact commands/exits/nonzero counts,
original failures, warnings, LEAK markers, literal diff/digests and limitations.
Parent owns independent review and combined state/TaskNode receiving validation.
No builds, tests, source formatting, reviews, children, commits/pushes or live
actions are authorized in this allocation-documents turn.

## Remaining S02 integration gaps, not new bridge prerequisites

- Normal-build store: `state/src/runtime.rs` still excludes accounting;
  `state/src/migrations.rs::STATE_MIGRATOR` installs numbered migrations at startup,
  and `state/BUILD.bazel` includes `migrations/**`. Manager must allocate actual
  receiving-tree numbering and OFF installation policy before promotion. No number
  is reserved and test fixture installation is not a user-data migration.
- Real durable dispatch: `core/src/session/turn.rs` sampling retries and
  `try_run_sampling_request`, plus `codex-api/src/endpoint/session.rs::{execute_with,stream_encoded_json_with}`
  actual auth/transport retries need awaited pre-send persistence and stable
  request context without an API→SQLite dependency. Core attempt index alone
  misses transport retries; WebSocket/chat-completions coverage remains required.
- Presence/provenance: `codex-api/src/sse/responses.rs::ResponseCompletedUsage`
  defaults cache/reasoning details; `endpoint/anthropic_messages.rs::AnthropicUsage::merge_into`
  collapses optional values and applies compatibility heuristics. Preserve
  omitted/null/zero before conversion and dialect provenance; cumulative token
  snapshots cannot recover request identity or lost presence.
- Native production lifecycle: `core/src/state_db_bridge.rs` delegates to
  `rollout/src/state_db.rs`; unavailable StateDbHandle and pre-send failure policy
  need explicit integration qualification. Native row recreation remains possible.
  Reuse immutable original Snapshot/binding and exact quotation; no supplied live
  catalog/effective-time authority, no API-balance-as-invoice, unknown stays unknown.
- **90..365-day late import remains unqualified, not waived.** Keep the current
  Journal rejection guard. A real adapter needs authoritative request/dispatch
  identity, raw presence and original-price provenance with atomic deduped
  compact-only import, without temporarily persisting aged raw detail. Revisions
  cannot be subtracted after per-attempt amounts are erased; sealed final imports
  are an option only if authority/finality is actually guaranteed. Do not shorten
  replay to 90 days or silently call historical observations final.
- `app-server/src/request_processors/token_usage_replay.rs` replays display
  snapshots, not billable requests; native spawn edges remain hierarchy authority.
  Root plus two children, consumed retries, ambiguous provider requests and legacy
  unknowns still need complete native end-to-end goldens. S03 stays draft; range/
  interval UI, full S02/S03, human, live and release readiness are not claimed.

Manager engineering decisions: internal clock/visibility seams, exact allocation,
size/review allowance, receiving migration numbering and integration sequencing.
No new product decision is needed for this approved bridge or accepted 90/365-day
and conservative-day-expiry policy. New authority would be required to change
those horizons, retain identifying/numeric detail longer, permanently drop required
imports, label ambiguous usage known, promise cross-database erasure or enable new
user-visible/live behavior. This allocation grants none of those changes.
