---
sprint_id: "PF-70-S01"
title: "Index methodology and tradability"
status: draft
plan_file: "docs/plans/proposed/portfolio-acceleration-index.md"
plan_feature: "PF-70"
execution_order: 1
owner: "Alex Good (index owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-68-S03"
created: 2026-09-09
updated: 2026-09-09
---

# PF-70-S01 — Index methodology and tradability

## Execution mandate

- Deliver: docs/research/acceleration-index/methodology.md; All constituent and weight decisions are deterministic given a dated input set.
- Excludes: Trading, launching tokens, claiming neutrality without a defined hedge, or inventing Alex's holdings and weights.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Algorithmic acceleration index paper design](../../../plans/proposed/portfolio-acceleration-index.md)
- Feature: `PF-70`; acceptance: All constituent and weight decisions are deterministic given a dated input set.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-acceleration-index.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/acceleration-index/methodology.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/acceleration-index/pf-70-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Alex chooses objective, universe, weights and whether any hedge is part of the hypothesis; verified stock-access evidence is required.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Obtain Alex's intended thesis/universe; distinguish proposed long-only index and optional hedge hypothesis.
- [ ] Define inclusion, weighting, rebalance, corporate actions, delisting and missing-data rules.
- [ ] Consume PF-68 instrument diligence; record exposure gaps and unavailable names rather than substitute silently.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Two independent calculations reconstruct one sample rebalance exactly.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-acceleration-index/` and update plan backlinks.
