# Workstream 1: PF-13 acceptance handoff

Status: blocked, not human-test ready. This is a readiness checklist, not a pass.

1. Confirm the intended candidate and branch owner. The observed Linux checkout
   is 28 commits behind the inspected remote PF-13 branch; no update was applied.
2. Reconcile the old S05 label with the receiving ledger's archived S01–S06.
   Do not repeat completed work or claim old evidence covers a new candidate.
3. Resolve PF-35-S01's existing security reservation before allocating a new
   security sprint. No running agent has been inferred from a branch name.
4. For S07, require its full dependency graph, exact binary/source hashes and
   independent protected-boundary evidence before human qualification.
5. Human qualification must include expired/revoked credentials, interruption,
   visible cancellation and recovery using only supported user controls. No
   hidden file edits, undeclared permissions or raw secret exposure count as
   recovery. Record actual results and missing platforms separately.

Owner decision and signed acceptance: pending.

Plan: [PF-13/security](../../../docs/plans/active/p0-security-levels.md).
Allocation: [three workstreams](../../../docs/plans/workstreams-2026-09-11.md).
