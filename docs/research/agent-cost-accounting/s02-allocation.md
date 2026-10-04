# PF-60-S02 allocation handoff

Prepared 2026-09-11 as the final PF-60-S01 output and independently reviewed.
Parent has now accepted/archived S01 and activated the matching first S02
increment in the shared sprint record. This handoff is not implementation or
permission beyond that exact record. Product initiative PF-60,
active [plan](../../plans/active/portfolio-agent-cost-accounting.md), product
heading **Product measurement** — “No commercial performance numbers have been
supplied.” The development skill routes this through the existing plan/sprint
and upstream contracts; no additional feature or acceptance exception is proposed.

## Authority and verified source

Travis's supplied “Approve these defaults” resolves the policy questions:
measured usage, estimates, billed amounts, allowance and balance stay distinct;
unknowns stay unknown; exact USD estimates use approved immutable snapshots;
numeric detail lasts 90 days and aggregates 365 days. User deletion removes
both, retaining only minimal opaque deduplication tombstones for the 365-day
replay horizon. Billing and live collection remain disabled. This approval
supersedes pending-policy wording in the older [contract](contract.md), without
changing its arithmetic or granting access to live prices/accounts.

| Source | Locally verified result |
| --- | --- |
| Worker | `/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`, `workstream/accounting-pf60-s01-20260911`, clean starting HEAD `1c5978690ee4cec0f3d2cab60ad13bf447ebdf84` |
| Receiving snapshot | `integrate/management-workstreams-20260911` at `0415a00dc3d3ee55a96662920c3eedbc0f0d4838`; worker HEAD is its merge-base with this worker |
| Canonical upstream | Configured `upstream` URL `https://github.com/openai/codex.git`; cached `upstream/main` is `1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6` |
| Shared upstream baseline | `git merge-base HEAD upstream/main` = `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`; ancestor checks succeed against both HEAD and cached upstream/main |
| Unintegrated remote tip | Parent reports `c210f4c222f8f7a447d0b3a013f3cb9cc3cd43b0`; object unavailable locally. No network verification or integration claim |

Compared worker/receiving source: no differences in `codex-rs/state/`,
`codex-rs/codex-api/`, `codex-rs/codex-client/`,
`codex-rs/core/src/session/turn.rs`, or S01 research/QA. At the common ancestor,
source inspection confirms `ResponseCompletedUsage` and its `TokenUsage`
conversion, and `StateRuntime::{init,pool}`. The endpoint session, retry and
Core turn paths also exist there. This verifies ancestry and identifiable seams,
not upgrade qualification or integration of the newer cached upstream tip.

Read [S01 fix evidence](../../../qa/portfolio/agent-cost-accounting/pf-60-s01/results.md):
48 tests are the recorded prior result, not rerun here. Preserve both corrections:
known cache components cannot exceed known inclusive input even with a missing
sibling; native Anthropic noncached input remains independently priceable.
With input 50/read 10/write absent, known input cost is `0.000153 USD`, inclusive
input and full cost remain unknown. Original root known cost is `0.001084 USD`;
the missing-write counterexample is `0.001009 USD`. These are synthetic estimates.

## First increment: offline native-state observation journal

Recommend one reviewable S02 increment of **at most 500 non-test changed lines**,
including DTOs, validation, SQL and registration even though compiled only in
tests. Target 450: 140 for typed identity/presence, 270 for SQL/store/replay,
40 for registration/docs. Stop and re-slice if implementation exceeds the bound;
do not remove validation to fit. This proves durable observation replay, not
the whole S02 cost/retention acceptance contract.

Use the existing `StateRuntime` SQLite pool in a disposable synthetic test home.
Register the entire private module under `#[cfg(test)]`. Test setup explicitly
creates additive draft tables using SQL held in that module; ordinary
`StateRuntime::init` never creates them. No public export, Cargo feature,
config switch, collector, production caller or migration in this increment.
This is a local feature-OFF boundary: normal builds cannot reference the module.
It avoids migrating a real user's database merely by starting a feature-OFF app.

Exact proposed first-increment write scope (paths relative to repository root):

| File | Change / registration |
| --- | --- |
| Existing `codex-rs/state/src/runtime.rs` | Only `#[cfg(test)] mod accounting;`; serialize this shared module registration with the parent |
| New `codex-rs/state/src/runtime/accounting.rs` | Private store entrypoints, inline draft DDL; `#[path = "accounting_types.rs"] mod types;` and `#[cfg(test)] #[path = "accounting_tests.rs"] mod tests;` |
| New `codex-rs/state/src/runtime/accounting_types.rs` | Typed identity, numeric presence patches, dialect and bounded source-position metadata; no arbitrary provider JSON or event bodies |
| New `codex-rs/state/src/runtime/accounting_tests.rs` | Native SQLite tests and literal synthetic inputs/expectations; independent of the Python reducer |
| New `qa/portfolio/agent-cost-accounting/pf-60-s02/first-increment.md` | Actual candidate digest, commands/counts, line budget, OFF proof, limitations and human-test prerequisites |

No edits to `state/src/lib.rs`, `state/Cargo.toml`, workspace manifests/lockfiles,
`state/src/migrations.rs` or `state/BUILD.bazel` are needed: existing serde,
serde_json, sqlx, uuid and Tokio dependencies suffice. Root `defs.bzl`'s
`codex_rust_crate` includes `src/**/*.rs` and registers unit tests. Inline SQL
and Rust literals avoid new compile-time resource registration. If a later
increment externalizes SQL/fixtures, explicitly allocate `state/BUILD.bazel`
compile data; a numbered production migration also uses the existing
`STATE_MIGRATOR = sqlx::migrate!("./migrations")`. Reserve no migration number
until the receiving tree and concurrent schema owners are rechecked.

Implement these operations, private to the test module:

1. `begin_attempt`: accept a caller-supplied opaque request/attempt identity,
   native `ThreadId`, turn, retry predecessor, provider/model, opaque scope,
   dialect and dispatch time. Persist identity before simulated dispatch;
   repeat identity is a no-op, changed identity is a conflict. Never construct
   identity from token counts, credentials, provider response ID or retry index.
2. `append_observation`: one transaction verifies/inserts identity, inserts a
   typed patch keyed by `(attempt_id, revision)`, and records its source position.
   Identical duplicates are no-ops; conflicting duplicates roll back the whole
   batch. Compare typed canonical payloads, not map ordering. Source sequence
   comes from the caller; record exact accepted positions rather than treating
   the largest seen revision as a contiguous replay checkpoint.
3. `read_observations`: reopen and return stable identity plus patches sorted
   by source revision. Keep omitted, explicit-null and numeric-zero presence
   distinguishable; reducers preserve earlier known values on null/omission
   and replace on reported numbers. Do not sum cumulative patches. Validate
   nonnegative exact integers and reject overflow, fractional/bool counts and
   impossible known subsets. Native Anthropic noncached input is a disjoint
   field; inclusive input requires all three input components. Unknown
   compatible dialects cannot inherit native Anthropic semantics.

Draft storage shape: separate attempt identity and observation tables, with
source position on each observation (the accepted-position replay checkpoint).
Both are product-owned additions in native state, not the throttle table or a
second database authority. No cost accumulator, aggregate, price lookup or
retention scheduler in this first increment. Store typed numeric patches and
minimal metadata only; no prompt/tool/response bodies or credential fingerprints.

The remaining structural choice is how later daily aggregates support deletion
after the 90-day detail cutoff: (A) per-thread/day contributions, reduced at read
time; (B) shared daily totals plus a per-thread/day subtraction ledger. Recommend
A for atomic deletion and fewer mutable totals. Manager selects A as the
engineering approach to the approved deletion behavior; production schema still
needs its later exact allocation. Global totals alone cannot satisfy
the approved deletion policy. This choice does not block the test-only journal.

## Native adapters for subsequent gated wiring

These are verified source seams and dispositions, **not additional first-increment
write permission**. Parent must allocate later shared registrations and tests
before touching them. Keep dependency direction Core → API/state; provider
parsers must not import SQLite or implement accounting policy.

| Adapter / source symbol | Exact boundary and disposition |
| --- | --- |
| Request: `codex-rs/core/src/session/turn.rs`, sampling retry loop and `try_run_sampling_request` | Adapt with one opaque logical request context outside the retry loop; multiple model calls in a turn get distinct IDs. Preserve cancellation and consumed failed attempts. Completion handling is too late to assign dispatch identity. |
| Dispatch: `codex-rs/codex-api/src/endpoint/session.rs`, `EndpointSession::execute_with`, `stream_encoded_json_with` | Adapt each actual `transport.execute`/`transport.stream` invocation after successful auth preparation with a pre-dispatch durable attempt callback. The async callback must finish before sending; a committed intent followed by a crash has unknown dispatch/usage, not measured zero. Allocate later failure behavior explicitly; never silently label unrecorded work complete. |
| Retry: `codex-rs/codex-api/src/telemetry.rs`, `run_with_request_telemetry`; `codex-rs/codex-client/src/retry.rs`, `run_with_retry` | Retain retry policy. Each nested transport invocation receives a distinct attempt ID under the logical request. The local retry counter is not globally unique. HTTP-only seams do not prove WebSocket/other dispatch coverage; leave those paths disabled for accounting until separately mapped and tested. |
| Presence: `codex-rs/codex-api/src/sse/responses.rs`, `ResponseCompletedUsage` → `TokenUsage` | Adapt numeric extraction before defaulting detail fields, including cache-write. Carry an internal typed presence sidecar; preserve existing token events. The current deserialized DTO already loses some absent-field information. |
| Presence: `codex-rs/codex-api/src/endpoint/chat_completions.rs`, `ChatUsage` → `TokenUsage` | Capture optional numeric fields before serde defaults/conversion; fixed cache-write zero is unknown. Corbanu API routing does not imply measured write zero or an approved price. |
| Presence: `codex-rs/codex-api/src/endpoint/anthropic_messages.rs`, `AnthropicUsage::merge_into` | Capture raw optional input/read/write/output patches before merging; distinguish verified native noncached dialect from compatible-provider heuristic. Preserve partial noncached pricing evidence and cumulative replacement semantics. |
| Durable store: `codex-rs/state/src/runtime.rs`, `StateRuntime`; planned `runtime/accounting.rs` | Adapt the same native pool and transaction boundary after schema acceptance. Extend the test-only module in later bounded increments; no API-to-state dependency. Retain `runtime/provider_requests.rs` lease/throttle authority unchanged. |
| Replay/hierarchy: `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/state/src/runtime/threads.rs` | Retain connection-scoped TokenCount notifications and native spawn edges. Neither reconnect nor inherited history creates an attempt. Do not promote transient `RawResponseCompleted` adjacency or cumulative thread totals into request records. |

## Verification and independent human prerequisites

First-increment tests must run nonzero registered cases under
`runtime::accounting::tests`: identity/immutable-owner conflicts; zero versus
omitted/null persistence; duplicate/reordered revisions; whole-batch rollback;
concurrent same-key writers; close/reopen then replay twice; interrupted intent
with no observation; native Anthropic partial input and inclusive cache-subset
validation. Compare complete rows and source positions. Reopen uses a real
on-disk disposable SQLite database and closed connections; do not call it a
process-kill test. The later crash test must use a separate process and an
interruption between transaction boundaries.

After formatting, from `codex-rs`, run `CARGO_NET_OFFLINE=true just test -p
codex-state runtime::accounting::tests` and then `CARGO_NET_OFFLINE=true just
test -p codex-state`. Verify normal compilation with `cargo check --offline
--locked -p codex-state --lib`. Use the pinned, preinstalled toolchain/cache;
missing dependencies are a prerequisite, not authority to fetch. Inspect the
combined diff for the test-only registration and absence of production calls.
Existing app-server token-usage tests remain a full-S02 gate when wiring changes;
an empty filter is not evidence. Record actual counts/results, not these plans.

For the first increment, an independent human can review the pinned diff, use
the registered test commands in an isolated home, and inspect literal expected
rows. Supply toolchain/cache prerequisites and the precise candidate/test paths
before assigning that review. No user accounting control exists yet: do not
claim human product-test readiness from this harness.

Before independent product testing, parent must provide a named tester and
separate setup operator; intent-only test proposal/evidence process under
`qa/code-blind-functional/README.md`; candidate SHA/binary hash; a documented
isolated activation method with default-OFF proof; synthetic/local provider
fixtures; approved test price snapshots; and public user controls for inspection,
restart and deletion. Cases must include root plus two children, consumed retry,
missing price/usage, interruption/restart, replay twice, deletion of detail and
aggregates, 90/365-day cutoffs and tombstone replay rejection. Setup cannot
substitute database repair for user recovery. Actual-key TUI, applicable
TensorCash/Isometric disposable checkouts, PF-13/PF-26 gates, named acceptance,
benchmarks and release evidence remain required where applicable; none is proved
or waived here. Live services, private logs and billing stay out of this allocation.

## Parent activation and completion gates

Accept the combined S01 artifact/fix evidence and this handoff; record the
approved defaults and verified ancestor in the shared plan; complete/archive
S01 and repair backlinks. Allocate S02's exact owner, worktree, branch, current
40-character receiving base and the five literal paths above; serialize shared
registration, reconcile the reserved lane, and run plan/sprint checkers before
making S02 ready/in_progress. Do not reuse S01's stale allocated base as S02's.

The small increment does not complete S02. Subsequent allocations still owe
native dispatch/presence wiring, provider-ID ambiguity handling, immutable exact
price selection/arithmetic, historical unknowns/lineage, atomic aggregate
deletion, opaque tombstones without retained numeric or identifying payloads,
365-day replay rejection, migration/upgrade tests, crash/restart and full golden
totals. Live collection must stay OFF until those dependencies and acceptance
gates pass. S03 remains dependent on completed/archived S02; no partial-journal
pass enables UI totals or billing. Parent owns combined-tree review and evidence.
