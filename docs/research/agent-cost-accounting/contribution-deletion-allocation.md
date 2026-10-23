# PF-60-S02 — retained contributions and recorded-attempt deletion

Manager allocation, September 12 UTC. Product initiative PF-60, same active
PF-60-S02; **Product measurement**, “No commercial performance numbers have been
supplied.” Approved defaults unchanged; no billing/production authority.
Accepted storage d898fbac0 and native adapter d7bf73a52 are combined at
`56295f66825e87dc924705d86fcbcebf258ba7f9`:283 native tests,129 Python/governance
tests and both normal-library checks pass. No leak markers in this run; the
earlier two warnings remain unattributed. Storage review is clean.

## Scope and hard boundary

Reuse accounting worktree/branch in the active plan, fast-forward when clean to
the reviewed allocation commit. Parent records actual launch HEAD/agent identity.
Five literal worker files:

- `codex-rs/state/src/runtime/accounting_estimates.rs`: private lifecycle child
  registration and extract the existing read_estimate validation into a private
  connection-taking reader. Existing read_estimate keeps its transaction wrapper
  and identical results. No persist_current or pricing changes.
- `codex-rs/state/src/runtime/accounting.rs`: only installed-fixture tombstone
  check inside append_observation's existing BEGIN IMMEDIATE, before inserts.
  Check sqlite_schema for explicit table presence; absence preserves journal-only
  fixtures, SQL errors fail closed. No DDL, signature, type or reducer changes.
- `codex-rs/state/src/runtime/accounting_lifecycle.rs`
- `codex-rs/state/src/runtime/accounting_lifecycle_tests.rs`
- `qa/portfolio/agent-cost-accounting/pf-60-s02/contribution-deletion-increment.md`

All code remains under cfg(test) accounting, private child of storage. Parent
serializes the two exact shared seams. No public visibility, dependencies, BUILD,
old tests/receipts, numbered migrations, native delete_threads_strict, provider,
dispatch, UI, auth, shared plans or production promotion. Target <=300 new logic,
<=55 changed seam lines, <=380 tests, <=40 receipt. These are planning targets;
hard <=500 non-test / <=800 total additions AND deletions. Re-slice if hard limits
cannot fit without dropping checks or compressing code. No review-budget reset.

## Concrete artifact

Stage A is retained-detail contribution replacement plus atomic deletion of
already-recorded attempts. It is not full native thread deletion or retention.
Stage B separately adds compaction, retention-aware reads and admission cutoffs;
do not quietly include it here. This stage needs no new human default decision.

Explicit fixture setup adds only draft_accounting_contributions with columns
attempt_id TEXT primary key, thread_id TEXT, utc_day INTEGER nonnegative,
evidence TEXT, all NOT NULL; composite foreign key (attempt_id,evidence) references
draft_accounting_estimates. Index (thread_id,utc_day). Also add
draft_accounting_tombstones(attempt_id TEXT primary key NOT NULL,
expires_at_ms INTEGER NOT NULL CHECK nonnegative). No native thread FK yet.
Contribution evidence copies observations and therefore remains 90-day detail,
not a 365-day compact aggregate. Existing fixture tables are unchanged.

Private lifecycle store wraps EstimateStore. Its descendant module can use
private native pool/Journal/Attempt/Usage/Decimal/authority and connection reader
without expanding visibility or calling ancestor methods in nested transactions.
Use native writer/read transactions, exact canonical data and checked arithmetic.
UTC day = nonnegative dispatch milliseconds /86,400,000, never arrival/local time.

Return typed day totals with seven separate measured-token known sums and unknown
attempt counts, exact USD known subtotal, unknown-full-estimate count and number
of constituent attempts. Full metric is unavailable if its unknown count is
nonzero. Empty set has no contributors, not proof a run was free. Reasoning/input
subsets stay separate, never extra priced buckets. Billed spend, allowance and
API balance are absent. Add exact amounts before six-decimal half-even display.

## Transaction contract

1. refresh_current(id): BEGIN IMMEDIATE; reject tombstone, load validated
   authoritative Attempt/ordered observations and exact evidence key. Verify
   current saved estimate through the connection reader. Missing estimate is
   NeedsEstimate, not use of a stale version or fresh catalog. Validate any old
   reference, then replace this attempt's single contribution reference; equal
   is no-op. Do not silently repair corruption or sum multiple versions.
2. New higher OR lower revisions change exact evidence. Append, persist estimate,
   then refresh replaces one contribution; immutable historical estimates remain.
   These three calls are not atomic together: expose NeedsEstimate/NeedsRefresh.
3. read_day(thread,day): one read transaction. Validate selected references and
   thread/day against typed Attempts, and membership/current evidence against
   the journal. Missing/stale references return NeedsRefresh, not partial current
   totals. Recompute from checked quotes with exact integer/Decimal::add; no
   float/SQL money SUM or rate-parser roundtrip for quoted amounts.
4. delete_recorded_thread(thread,as_of_ms): BEGIN IMMEDIATE. Enumerate/validate
   matching authoritative Attempts including unprojected/unquoted intentions.
   Malformed ownership must fail, not prove absence. Reject future dispatch
   relative to the supplied nonnegative fixture as_of. Insert only opaque attempt
   UUID + checked dispatched_at_ms +365*86,400,000 as tombstone expiry. Do not
   extend expiry on duplicate/delete. This exact rolling replay timestamp avoids
   silently rounding a horizon to the beginning of a day; no expiry removal yet.
5. Same transaction removes contributions, ALL estimate versions and copied
   provenance, bindings, observations and Attempts in FK-safe order. Delete only
   affected snapshots no longer referenced by a surviving binding. Shared
   snapshots survive until their last owner is removed. Repeated deletion is
   idempotent. SQL/validation/arithmetic failure rolls back everything, including
   tombstones; no ownership/numeric/provenance payload may survive deletion.
6. Journal tombstone guard shares its existing writer transaction. Replay-first
   commits then deletion removes it; deletion-first makes replay fail. Refresh
   and estimate writes likewise either precede removal or see missing authority.
   Readers observe complete before/after states, never partial deletion.

Tombstones block while present, even after their recorded expiry; time alone does
not waive deduplication. Stage B must remove them only atomically with a durable
old-import admission cutoff. No pruning API/service is allocated here. This does
not reject unknown/re-keyed attempts for a deleted native thread; future native
ownership/admission integration is required before full deletion or production
claims. Never retain thread/request/model/provider/source/sequence/amount in
tombstones as a workaround for that later gate.

## Human-style technical proof and return

Use an isolated synthetic disk database; no credentials, account, provider or
product UI. Give the reviewer diff/hashes, literal inputs, expected full objects
and exact table inspection. Required cases:

- Two attempts in one thread/day and another owner; partial Anthropic50/read10
  at3/0.3 gives0.000153, unknown-price and explicit zero remain distinct. Include
  two0.0000004 quotes summed before rounding and a24-place amount.
- Duplicate refresh, newer and late-lower revisions yield one current reference;
  absent/stale estimates/references are visibly held. Test corrupt attribution,
  exact count/amount overflow and unchanged historical quotes.
- Delete multiple versions plus an unquoted intention. Inspect every source,
  estimate and contribution table for removed copied provenance. Shared snapshot
  survives unrelated owner, disappears with last owner. Tombstones only twofields.
- Deterministic replay-first/deletion-first and refresh/deletion races using
  barriers or held writer transactions, not sleeps. No resurrection/partial state.
- Inject deletion SQL abort, invalid/future/overflow times and corrupt source;
  whole pre/post rows prove rollback. Repeat deletion and two actual disk reopens
  prove retained reads/replay rejection; no process-kill proof claimed.

Guard just fmt to literal allocated Rust paths before final tests; disclose
unrelated Python formatting/fix limitations, no environment repairs or bypasses.
Cached1.95.0, RUSTUP_AUTO_INSTALL=0, CARGO_NET_OFFLINE=true, own cached target.
Use just test -p codex-state selectors for
runtime::accounting::pricing::storage::lifecycle::tests, storage/pricing/journal,
then full state; normal cargo check --offline --locked -p codex-state --lib.
Both governance checkers and whitespace checks. Require nonzero tests.
Return uncommitted paths/hashes/counts/failures/limits/OFF proof. Parent owns one
independent Astra High review, scoped corrections and combined-tree tests.

## Successor and qualification gates

Next same-S02 slice joins compact independently deletable thread/day totals,
shared snapshot references,90-day detail/365-day aggregate and replay cutoffs,
strict exact-amount decoding, stale detail handling and monotonic admission.
Define rolling cutoff versus day-bucket batching explicitly: no numeric detail
outlives its approved horizon, partial-day compaction cannot double count, and
tombstone removal never admits expired imports. Required boundary/prune/replay/
delete races and retention-aware reads must be allocated together from accepted A.
No global-only aggregates, per-attempt numeric365-day retention or silent reprice.
Only later wire native delete_threads_strict before owner removal, admission,
production migration/promotion, provider presence and durable pre-dispatch intent.
S02 stays in_progress and S03 gated; full goldens/crash/lineage, user inspection/
export/deletion, true-TUI/live repositories/human acceptance/benchmarks remain.
Upstream https://github.com/openai/codex.git, common ancestor
413492cd6c3a4d4f8dff6f406247ccda5a9d88aa, cached1d74c3ba1ee98be2025ab066dcc3fd654fe8a3b6;
product-owned private modules/tiny seams only. No upstream upgrade/release claim.
