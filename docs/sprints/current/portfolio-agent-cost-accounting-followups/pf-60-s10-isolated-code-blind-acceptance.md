---
sprint_id: "PF-60-S10"
title: "Isolated code-blind acceptance of the follow-ups"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 5
owner: "Codex coordinator (integrator, proposed); independent executor and reviewer; Travis Good accepts"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-60-S06, PF-60-S07, PF-60-S08, PF-60-S09"
created: 2026-10-10
updated: 2026-10-10
---

# PF-60-S10 — Isolated code-blind acceptance of the follow-ups

## Execution mandate

- Deliver: item (viii) from Travis's 2026-10-10 decision (#404). PF-60-S05 waived the enforced isolation gate
  (waiver (b)) and PF-60-S04's acceptance ran without it too. Run the acceptance of PF-60-S06 to PF-60-S09 inside the
  [isolated execution gate](../../../../qa/code-blind-functional/isolated-execution.md), so waiver (b) is discharged.
- Excludes: fixing defects found (they become issues); new product scope.

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/proposed/portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: all follow-up flows pass on one recorded binary under enforced isolation.

## Code boundaries

- Existing: `qa/code-blind-functional/` (contract, `RECORD_TEMPLATE.md`, `check.py`, executor and designer prompts).
- Planned: `qa/portfolio/agent-cost-accounting/followups-acceptance-<date>/`; reuse P1 security's isolated VM/runner
  if it exists by then rather than building a second one.

## Preconditions

- [ ] Plan is active.
- [ ] PF-60-S06, PF-60-S07, PF-60-S08 and PF-60-S09 completed and archived.
- [ ] Isolated runner available: VM, separate unprivileged account or verified OS sandbox, with mediated provider
      credentials the executor can't read.

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.

## Remaining

- [ ] AC1: a fresh designer's cases for S06–S09 are frozen before any result is disclosed.
- [ ] AC2: a separate fresh executor runs every case on the exact packaged candidate inside the gate: read-only
      package, private state and PTY, no repository, history, prior findings or real credential access.
- [ ] AC3: negative access probes from the executor and its children (repository, home, keychain, vault, network
      outside the mediated path) are recorded as refused; positive package/PTY controls are recorded.
- [ ] AC4: real provider keys reach the product only through the mediated path; the key scan of all evidence is 0.
- [ ] AC5: every case has PASS, FAIL or a disposition accepted by Travis; pay-per-use totals recompute exactly.
- [ ] AC6: an independent code-blind reviewer audits the record; `python3 qa/code-blind-functional/check.py` passes on it.

## Verification

- [ ] Schema-2 execution and isolation receipts linked.
- [ ] Exact candidate commit and binary digest recorded.
- [ ] Integration: `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Code-blind cases independently executed under enforced isolation; separate evidence check linked.

## Exit evidence

- [ ] Acceptance record and review linked.
- [ ] Travis accepts; waiver (b) recorded as discharged in this plan and in PF-60-S05's archived record (dated addendum).
- [ ] Code-blind handoff checker passes; no automatic human acceptance.
- [ ] Parallel handoff, if applicable: commit, contract versions, scope audit and combined-tree test evidence recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality; #404 closed.
- [ ] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/`.
