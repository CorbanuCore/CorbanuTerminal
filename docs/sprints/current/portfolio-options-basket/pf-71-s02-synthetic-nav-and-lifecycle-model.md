---
sprint_id: "PF-71-S02"
title: "Synthetic NAV and lifecycle model"
status: draft
plan_file: "docs/plans/proposed/portfolio-options-basket.md"
plan_feature: "PF-71"
execution_order: 2
owner: "Alex Good (financialization lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-71-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-71-S02 — Synthetic NAV and lifecycle model

## Execution mandate

- Deliver: docs/research/options-basket/paper-model.md; All lifecycle states balance; stale or missing quotes prevent a current-value claim.
- Excludes: Issuing tokens, purchasing options, customer solicitation, promising liquidity, custody deployment or hiding path-dependent losses.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Tokenized options-basket feasibility](../../../plans/proposed/portfolio-options-basket.md)
- Feature: `PF-71`; acceptance: All lifecycle states balance; stale or missing quotes prevent a current-value claim.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-options-basket.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/options-basket/paper-model.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/options-basket/pf-71-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-71-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Build a synthetic position/quote ledger and hand-check NAV, fees, rolls, expiry and exercise/assignment.
- [ ] Stress gaps, illiquidity, unavailable quotes, extreme moves and redemption suspension; model losses explicitly.
- [ ] Record valuation timestamps, stale-NAV behavior and restart/reconciliation procedure.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Independent analyst reproduces the golden lifecycle ledger and all stressed cash/position balances.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-options-basket/` and update plan backlinks.
