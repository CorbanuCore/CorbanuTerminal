---
sprint_id: "PF-67-S02"
title: "Small adaptation experiment"
status: draft
plan_file: "docs/plans/proposed/portfolio-domain-finetuning.md"
plan_feature: "PF-67"
execution_order: 2
owner: "Domain-model research lead (assignment pending)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-67-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-67-S02 — Small adaptation experiment

## Execution mandate

- Deliver: docs/research/domain-finetuning/experiment.md; All three approaches are scored on the same held-out data with cost and update-time accounting.
- Excludes: Using Thomson Reuters or other restricted data without rights, large training jobs, production model replacement or promised real-time updates.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Real-time domain fine-tuning feasibility](../../../plans/proposed/portfolio-domain-finetuning.md)
- Feature: `PF-67`; acceptance: All three approaches are scored on the same held-out data with cost and update-time accounting.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-domain-finetuning.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/benchmarking/index.md`.
- Planned output: `docs/research/domain-finetuning/experiment.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/domain-finetuning/pf-67-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-67-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Run the approved baseline and one small adaptation method on the same frozen task set in an isolated workspace.
- [ ] Record compute, update duration, inference cost, forgotten capabilities and safety/reliability regressions.
- [ ] Interrupt and resume an experimental run; retain seeds, checkpoints, failures and data provenance.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Reproduce scores from exported predictions; audit dataset overlap independently.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-domain-finetuning/` and update plan backlinks.
