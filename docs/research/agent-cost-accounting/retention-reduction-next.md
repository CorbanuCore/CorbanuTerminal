# PF-60-S02 — allocation B: pure retention reduction

Accepted September12: candidateac5a22d67a6ffd9ffb56783754c55a15eb26a4f4,
integrated7514be8ec7b5fc92288bcaf3aff2559bfe23337b. One clean AstraHigh
code review; parent combined330 tests pass9.928s. Original allocation below
is retained; it does not authorize the next coupled mutation increment.

Status: allocated September12 after reviewed input8eca814f84b6005c25cbf85d4eb6a88dd353ebf9
was integrated atfd46c5897c3c3a1c146a97f544e5bacf4e996f9d. Combined state/
TaskNode325passed0skipped9.100s, run0ab6b70f-e645-44eb-8e4d-9a3adaadd41a.
Product heading **Product measurement**, “No commercial performance numbers
have been supplied.” Same approved PF60 feature/S02; S03 remains draft.

One Astra High worker, existing accounting-pf60-s01-20260911 worktree and
workstream/accounting-pf60-s01-20260911 branch. Base is the receiving commit
above; manager records actual clean launch descendant after governance.

## Exact writes and boundary

- codex-rs/state/src/runtime/accounting_retention_plan.rs: private sibling
  reduction module registration only; accepted reader/types remain frozen.
- codex-rs/state/src/runtime/accounting_retention_reduction.rs: new private
  module, ephemeral plan types and read-then-reduce entry.
- codex-rs/state/src/runtime/accounting_retention_reduction_tests.rs: new tests.
- codex-rs/state/src/runtime/accounting_retention_test_support.rs: minimal
  exclusively test-only helper reuse if needed; do not alter A assertions.
- qa/portfolio/agent-cost-accounting/pf-60-s02/retention-reduction-increment.md.

Read input through read_validated_input_on_connection in caller-owned
transaction. Child reduction module can access parent-private fields; no public
exports or transferable serialized authority. Return previous/computed as-of,
surviving compact values/reference unions, expired existing compact keys,
raw UUID/replay-expiry removals and untransferred raw day totals/freshness.
Detail expires atdispatch+90days, aggregate atUTC-day-start+365days, replay
atdispatch+365days; equality expires. Never resurrect expired raw-only days.
Preserve exact checked decimals/populations, absent-price unknowns and original
immutable binding; no repricing. Missing/stale unexpired contribution means
NeedsRefresh. Never write/delete/commit/rollback/open transaction or apply plan.

Preserved draft and all B cases live in private directory
/Volumes/CorbanuDrive/Corbanu/.codex-work/manager-accounting-retention-reslice-20260912.32IaBj/
with PRESERVATION.md hashes. Reuse all relevant mixed-day/shared snapshots,
exact90/365/dispatch0/one-millisecond loss, null/unbound/stale/original-price,
two-reopen and merge-overflow cases. Compare complete literal plans and all ten
tables plus total_changes for success/failure; A reader tests must remain green.
Do not treat preserved draft tests as evidence until run against this candidate.

Target under600 total changed lines; hard800 total/500non-test including receipt.
Count early; preserve proof and request manager reslice if needed. No manifests,
locks, migration, production runtime/native registration, live collection,
network/credentials, commits/pushes/reviews/children or other shared-file edits.
Use cached Rust1.95, auto-install OFF, Cargo/UV offline, assigned worker target.
Scoped fix/format before focused nonempty reduction tests via just test,
full state tests and normal offline locked library check. Preserve known fix
TCP-listener restriction; no environment repairs. Parent owns review/integration.
Integrator adds one B code-review pass; A used1 ofprior+2, unused correction
remains available. No reset/repeat clean A review. TUI/code-blind/live-repo proof
N/A for this private reduction only. Coupled mutation/read/delete/admission,
replay-floor advancement, runtime and actual crash/erasure proof remain later.
