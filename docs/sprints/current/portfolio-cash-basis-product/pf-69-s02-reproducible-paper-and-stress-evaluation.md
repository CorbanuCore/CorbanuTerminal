---
sprint_id: "PF-69-S02"
title: "Reproducible paper and stress evaluation"
status: draft
plan_file: "docs/plans/proposed/portfolio-cash-basis-product.md"
plan_feature: "PF-69"
execution_order: 2
owner: "Alex Good (strategy owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-69-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-69-S02 — Reproducible paper and stress evaluation

## Execution mandate

- Deliver: docs/research/cash-basis-product/paper-results.md; Both favorable and loss scenarios are reported; all results reproduce from the same data and code.
- Excludes: Executing Alex's trade, allocating user capital, yield guarantees, borrowing, leverage activation, deposits or pooled customer funds.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Cash management and basis-strategy paper validation](../../../plans/proposed/portfolio-cash-basis-product.md)
- Feature: `PF-69`; acceptance: Both favorable and loss scenarios are reported; all results reproduce from the same data and code.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-cash-basis-product.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/benchmarking/index.md`.
- Planned output: `docs/research/cash-basis-product/paper-results.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/cash-basis-product/pf-69-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-69-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Use authorized timestamped data with fees, spread, funding, collateral and execution delay recorded.
- [ ] Run baseline and adverse scenarios including funding reversal, orphan leg, liquidation and unavailable exit.
- [ ] Retain equity, drawdown, exposure, PNL attribution and replay manifest; do not submit orders.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Check no look-ahead or survivorship leakage; independently reconcile cash and both strategy legs.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-cash-basis-product/` and update plan backlinks.
