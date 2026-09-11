---
sprint_id: "PF-71-S01"
title: "Instrument, rights and operating model"
status: draft
plan_file: "docs/plans/proposed/portfolio-options-basket.md"
plan_feature: "PF-71"
execution_order: 1
owner: "Alex Good (financialization lead, proposed)"
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

# PF-71-S01 — Instrument, rights and operating model

## Execution mandate

- Deliver: docs/research/options-basket/structure.md; Every operating responsibility has an owner hypothesis and unresolved questions; no assumed legal structure.
- Excludes: Issuing tokens, purchasing options, customer solicitation, promising liquidity, custody deployment or hiding path-dependent losses.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Tokenized options-basket feasibility](../../../plans/proposed/portfolio-options-basket.md)
- Feature: `PF-71`; acceptance: Every operating responsibility has an owner hypothesis and unresolved questions; no assumed legal structure.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-options-basket.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/options-basket/structure.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/options-basket/pf-71-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Alex supplies target exposure and roll rule; qualified review must cover issuance, derivatives, distribution, custody and redemption before any live scope.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Specify the proposed option exposure, roll policy and holder rights with Alex; do not invent terms.
- [ ] Research primary instrument and infrastructure constraints; identify custody, valuation, issuance and redemption responsibilities.
- [ ] Prepare counsel questions and distinguish this concept from existing Post Fiat NAV capabilities.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Review instrument lifecycle and rights end-to-end before any economics work.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-options-basket/` and update plan backlinks.
