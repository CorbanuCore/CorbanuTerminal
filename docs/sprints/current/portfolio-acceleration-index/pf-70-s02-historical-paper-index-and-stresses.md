---
sprint_id: "PF-70-S02"
title: "Replayability, paper index and stresses"
status: draft
plan_file: "docs/plans/proposed/portfolio-acceleration-index.md"
plan_feature: "PF-70"
execution_order: 2
owner: "Alex Good (index owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-70-S01"
created: 2026-09-09
updated: 2026-09-10
---

# PF-70-S02 — Replayability, paper index and stresses

## Execution mandate

- Deliver: docs/research/acceleration-index/paper-results.md; Returns, drawdown, exposures and turnover reproduce; each untradable exposure is visible.
- Excludes: Trading, launching tokens, claiming neutrality without a defined hedge, or inventing Alex's holdings and weights.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Index-creation API, replayability and creator ecosystem](../../../plans/proposed/portfolio-acceleration-index.md)
- Feature: `PF-70`; acceptance: Returns, drawdown, exposures and turnover reproduce; each untradable exposure is visible.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-acceleration-index.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/acceleration-index/paper-results.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/acceleration-index/pf-70-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-70-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Run time-correct approved data through the methodology with fees, turnover, liquidity and instrument-basis costs.
- [ ] Freeze approved cached transcripts, shares, market caps and prices into a hashed local packet; pin model/prompt/runtime/seed/hardware/output provenance and any safely available replay trace.
- [ ] Recompute independently and across restart; distinguish deterministic arithmetic, saved-output replay and fresh inference reproducibility. Report exact matches, tolerances and counterexamples; no automatic hardware rental.
- [ ] Test missing/corrupt packets and changed model/data versions; fail visibly. Confirm the transcript's ambiguous inference-library name with Alex before using it.
- [ ] Compare to an appropriate declared benchmark; include concentration, stale-price and missing-constituent stresses.
- [ ] Evaluate any hedge separately with its own assumptions; do not label the whole strategy market-neutral by assertion.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Audit look-ahead/survivorship and independently recompute sample weights and returns.
- [ ] Focused: Record repeated-run input/output digests and unsupported guarantees; neither reproducibility nor a reasoning trace establishes privacy or regulatory clearance.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-acceleration-index/` and update plan backlinks.
