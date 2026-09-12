# PF-60-S02 — next retention preparation allocation

Status: **allocated for same-sprint execution; launch ID/HEAD in manager receipt**. September 12, 2026.
This is continuation of PF-60-S02, not a new sprint or product contract.
Product: **Product measurement**, “No commercial performance numbers have been
supplied.” The [active plan](../../plans/active/portfolio-agent-cost-accounting.md)
and [approved retention handoff](retention-design-handoff.md) retain authority.

## Launch gate and ownership

Latest quote `8b6d629b6` and accepted main `1b7e9f3df` are now reconciled in
canonical receiving at `87e31f521672e627e6230d48fc16a4cfaa7ff44c`. Overlapping Facilities edits
were preserved recoverably before reconciliation. The old publisher was declined;
no overlapping sync is authorized. Local work is no longer held by publication.

Intended worker: one native Astra High agent in
`/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911`, branch
`workstream/accounting-pf60-s01-20260911`. Before dispatch the manager must:

- Integrate the accepted candidate in the declared canonical receiving checkout.
- Verify the previous worker is closed and this worker checkout is clean.
- Record the actual new 40-character base in both active plan and sprint, replace
  the frozen latest-quote scope with the five paths below, and record the launch
  HEAD after a safe fast-forward. This document's staging SHA is not a launch SHA.
- Preserve the review ledger and run both governance checkers. The integrator
  authorizes two additional passes: one new candidate review and one correction
  only for substantive findings, under Travis's September12 delegation. Do not
  rerun old clean reviews; further extensions are manager decisions.

No changes to pricing, production admission or retention policy are authorized.
Detail expires at dispatch+90 days, daily aggregates at UTC-day-start+365 days,
and opaque replay records at dispatch+365 days; equality expires. No reapproval.

## Literal allocated worker scope

1. `codex-rs/state/src/runtime/accounting_lifecycle.rs`: one private sibling
   registration only, delegated by the manager.
2. `codex-rs/state/src/runtime/accounting_retention_plan.rs`: read-only planner.
3. `codex-rs/state/src/runtime/accounting_retention_plan_tests.rs`: new tests.
4. `codex-rs/state/src/runtime/accounting_retention_test_support.rs`: synthetic
   setup, explicit fixture tables, reopen and complete-table dump support only.
5. `qa/portfolio/agent-cost-accounting/pf-60-s02/retention-plan-increment.md`:
   exact candidate, commands/results, limitations and handoff receipt.

All remain under the existing test-only accounting ancestor. No production
migration, dependencies, manifests, runtime registration, collector, clock,
network, credential access, code review, commit, push or child-agent dispatch.
Existing latest-quote, codec and their test/receipt files remain frozen.

## Read-only contract

Add private `prepare_on_connection(conn: &mut SqliteConnection, as_of_ms: i64)`.
Its caller owns the transaction; the planner never begins, commits, rolls back
or writes. It returns an in-memory whole-store plan with previous/computed
as-of, surviving compact days, expired day keys, raw removal UUID/expiry pairs
and untransferred raw day totals/freshness. It is not a serialized capability
or permission to apply a plan later on a different snapshot.

Fixture support explicitly installs, atomically, compact-day payloads keyed by
canonical thread/day, compact-day immutable snapshot references, and a singleton
completed-as-of checkpoint. Initially NULL means maintenance never completed,
not time zero. Reopen attaches without reinstalling. No production setup caller.
Compact payloads contain only the accepted exact aggregate representation and
snapshot references; no per-attempt detail, provider/model or ownership ledger.

Validate all stored keys, payloads and references, including expired destinations,
orphan contributions, raw/tombstone collisions and checked expiry arithmetic.
Reject negative/backward/future times and malformed checkpoints. Preserve full
latest-quote validation and original immutable binding, including absent prices.
Never reprice from a newer catalog or call a transaction-owning quote persister.

Unexpired raw contributions preserve current missing/stale-result semantics.
Expired raw attempts contribute their latest quote once to an eligible compact
day; never resurrect an already expired day. Merge with existing compact values
using checked exact arithmetic and union immutable snapshot references. A missing
price stays unknown. Return proposed removals without executing any of them.

## Size and evidence

Provisional target: 290 planner/registration + 310 tests + 120 synthetic support
+ 35 receipt = 755 changed lines. Hard limits remain 800 total and 500 non-test,
counting additions and deletions and new files. Count support as test-only only
when it is exclusively fixture code. Return actual size for manager re-slicing
if proof does not fit; never remove required tests to meet an estimate.

Compare literal complete plans, all ten ordered fixture tables, and connection
`total_changes()` before/after every success and failure. Cover stale and absent
quotes, null bindings, existing destinations, shared snapshots, foreign/orphan
references, malformed/overflow values, mixed raw/compact days, exact90/exact365,
dispatch0, one-millisecond conservative-day loss, and two disk reopens. These
tests prove preparation only, not deletion, contention or crash recovery.

Use pinned cached Rust 1.95.0, auto-install OFF, offline Cargo/UV and the assigned
worker target. Scoped fix/format must precede final tests; preserve and disclose
environment restrictions without repair. Run the nonempty `retention_plan::tests`
selector through `just test -p codex-state`, full state tests, and normal-library
`cargo check --offline --locked -p codex-state --lib`. Parent owns independent
review, literal scope/hash audit, governance and receiving-tree affected tests.

No interactive entry changes: code-blind/TUI/live-repository evidence is N/A for
this private preparation increment only. Runtime qualification remains pending.

## Following mutation must remain coherent

Only after this plan is accepted, allocate atomic transfer **together with**
retained reads, compact-only deletion and admission fencing. Prepare and apply
within the same `BEGIN IMMEDIATE`; update replay floor/checkpoint with deletion.
Readers cannot silently use raw-only totals after compaction. Shared snapshots
survive until no raw or compact owner remains. Rollback, reopen and actual writer
contention need explicit proof before acceptance.

Rejecting previously unseen 90–365-day imports may be a bounded fixture
limitation, never a declaration that the approved import/replay contract is
finished. Native ownership, runtime clock/maintenance, late imports and real
crash/physical-erasure qualification remain open. S03 custom ranges/intervals
stay draft until S02 is genuinely completed.
