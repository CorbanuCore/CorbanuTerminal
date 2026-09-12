# PF-60-S02 — latest quote on the retention transaction

Manager allocation, executable after its review/checks and recorded launch HEAD.
Product: **Product measurement**, “No commercial performance numbers have been
supplied.” Travis approved conservative UTC-day-start+365-day aggregate expiry;
raw detail stays dispatch+90 days, replay protection dispatch+365 days.
The [retention handoff](retention-design-handoff.md) owns the later atomic boundary.

Baseline: `99001e9b79676f78b6bd941125c2a8fcfca6d90e`, including accepted B1
`c8d46d7097d27a75c0cad51049468ea172f277c2`. Use the existing accounting
worktree/branch in active plan/S02; parent fast-forwards the clean idle worker
to this reviewed amendment and records actual launch HEAD before dispatch.

## Literal scope and interface

Only `codex-rs/state/src/runtime/accounting_estimates.rs`, new sibling
`accounting_latest_quote_tests.rs`, and
`qa/portfolio/agent-cost-accounting/pf-60-s02/latest-quote-increment.md`.
Parent delegates just the sibling test registration in estimates to this worker.
No journal/lifecycle/pricing/codec/production/migration/UI/dependency edits.

Add private `EstimateStore::latest_quote_on_connection(conn: &mut
SqliteConnection, id: Uuid) -> anyhow::Result<ObservationQuote>` (async).
Requires a caller-owned transaction across all reads and any later transfer.
No pool, nested transaction, candidates, writes, clock or new DTO. This method
removes the compactor's dependency on transaction-owning `persist_current`.

Validate latest authority even without estimates. Enumerate every retained
evidence version deterministically and validate through `read_on_connection`,
requiring Some. Corrupt old evidence cannot disappear behind a recomputed quote.
Reuse the existing immutable binding via `bound_quote`, including SQL NULL;
when neither binding nor estimates exists, quote current authority with an empty
candidate list. An estimate missing its binding is corruption, not unknown.
Missing/malformed/ineligible snapshots and arithmetic failures return errors.
Never select a fresh catalog, reprice history, mutate old versions or turn honest
unknowns into zero. Return the entire latest quote, not compact storage detail.
The transient observations must not become a retained numeric ledger after90days.

## Proof and budget

Target610 changed lines (100 implementation/registration,470 tests,40 receipt);
hard500 non-test/800 total additions+deletions. No omitted cases or compressed
code to fit. Return actual diff for manager re-slicing if the coherent proof grows.

Use real isolated on-disk fixture tables, full quote equality and complete ordered
table dumps before/after. Cover current/stale/out-of-order observations; bound-null,
unbound intent, bound-with-no-estimates, zero and missing rates; another attempt's
eligible snapshot must never price these. Literal Anthropic50/read10 at3/0.3 is
0.000153; latest60/read10 becomes0.000183 without altering the saved old version.
Cover seven populations and exact24-place amounts, malformed old/current evidence,
identity/position/replay corruption, orphan binding, missing/malformed/ineligible
snapshot and checked overflow. Caller stages a revision on its own transaction,
reader sees it without nesting; rollback restores all rows and prior quote.
Repeat reads after two disk reopens without recreating tables. No mutation proof
is claimed beyond proving this reader leaves storage unchanged.

Use cached Rust1.95.0/offline/auto-install OFF and assigned target. Scoped fix and
`just fmt` before inspecting final bytes; preserve unrelated paths and disclose
environmental formatter failures. Run nonzero selector
`just test -p codex-state runtime::accounting::pricing::storage::latest_tests`,
existing storage/lifecycle/compact/pricing/journal selectors, full
`just test -p codex-state`, and offline locked normal-library cargo check.
No direct cargo test. Root governance/whitespace checks required. Record full
logs, actual counts, errors/LEAK names, hashes and size. No live/native-user QA
or complete-retention claim. Ancestor cfg(test) and feature-OFF stay unchanged.

Parent owns one independent Astra High review, scoped corrections, local commit/
integration and combined-tree affected tests; no worker commits/reviewers/push.
Then allocate atomic transfer with read/delete/admission together. S03 filters
remain draft, not a reason to extend retention or bypass S02 acceptance.
