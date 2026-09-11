---
sprint_id: "PF-69-S01"
title: "Strategy and risk contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-cash-basis-product.md"
plan_feature: "PF-69"
execution_order: 1
owner: "Alex Good (strategy owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-09-09
updated: 2026-09-09
---

# PF-69-S01 — Strategy and risk contract

## Execution mandate

- Deliver: docs/research/cash-basis-product/strategy.md; A second analyst can implement the same strategy without guessing instrument or leverage choices.
- Excludes: Executing Alex's trade, allocating user capital, yield guarantees, borrowing, leverage activation, deposits or pooled customer funds.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Cash management and basis-strategy paper validation](../../../plans/proposed/portfolio-cash-basis-product.md)
- Feature: `PF-69`; acceptance: A second analyst can implement the same strategy without guessing instrument or leverage choices.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-cash-basis-product.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/benchmarking/index.md`.
- Planned output: `docs/research/cash-basis-product/strategy.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/cash-basis-product/pf-69-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Alex supplies the intended strategy and venue assumptions; no live trial without explicit separate limits and qualified review.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Ask Alex to specify instruments, venues, rebalance rule, collateral, horizon and proposed constraints; unresolved choices block evaluation.
- [ ] Research primary venue mechanics and fees; define custody, margin, funding, settlement and exit assumptions.
- [ ] Create a no-trade paper ledger and predeclare stress cases and rejection thresholds.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Review assumptions for cash-versus-risk mislabeling and missing leg/withdrawal exposures.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-cash-basis-product/` and update plan backlinks.
