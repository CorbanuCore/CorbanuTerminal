---
sprint_id: "PF-65-S02"
title: "One bounded reproducible experiment"
status: draft
plan_file: "docs/plans/proposed/portfolio-private-inference-research.md"
plan_feature: "PF-65"
execution_order: 2
owner: "Privacy research lead (assignment pending)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-65-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-65-S02 — One bounded reproducible experiment

## Execution mandate

- Deliver: docs/research/private-inference-research/experiment.md; Results can be rerun from a manifest; no private customer data was used.
- Excludes: Privacy guarantees, proprietary training data, production deployment or assuming a named technology works from the conversation alone.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Private inference feasibility for Ambient and Corbanu](../../../plans/proposed/portfolio-private-inference-research.md)
- Feature: `PF-65`; acceptance: Results can be rerun from a manifest; no private customer data was used.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-private-inference-research.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/private-inference-research/experiment.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/private-inference-research/pf-65-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-65-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Select one approach after S01 and record environment, public/synthetic dataset, baseline and spend/time cap.
- [ ] Implement only an isolated research fixture in a separately approved research workspace; do not alter Terminal providers.
- [ ] Measure latency, throughput, task quality, cost and observable leakage; retain negative results.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Repeat the baseline and candidate with the same inputs; record variance and raw result hashes.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-private-inference-research/` and update plan backlinks.
