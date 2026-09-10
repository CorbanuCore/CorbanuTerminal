---
sprint_id: "PF-67-S01"
title: "Task, rights and baseline contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-domain-finetuning.md"
plan_feature: "PF-67"
execution_order: 1
owner: "Domain-model research lead (assignment pending)"
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

# PF-67-S01 — Task, rights and baseline contract

## Execution mandate

- Deliver: docs/research/domain-finetuning/evaluation.md; All data is authorized and every proposed gain has a falsifiable metric and baseline.
- Excludes: Using Thomson Reuters or other restricted data without rights, large training jobs, production model replacement or promised real-time updates.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Real-time domain fine-tuning feasibility](../../../plans/proposed/portfolio-domain-finetuning.md)
- Feature: `PF-67`; acceptance: All data is authorized and every proposed gain has a falsifiable metric and baseline.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-domain-finetuning.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/benchmarking/index.md`.
- Planned output: `docs/research/domain-finetuning/evaluation.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/domain-finetuning/pf-67-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Approve data rights, one target task, isolated experiment location and capped compute budget.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Choose a narrow task and licensed/public corpus; record acquisition, use, retention and redistribution rights.
- [ ] Define time-split held-out sets, contamination checks, quality rubric, latency/update target and stop budget.
- [ ] Implement an evaluation specification comparing prompt-only, retrieval and adaptation; no training yet.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Reviewer checks temporal leakage and rubric scoring on deliberately contaminated and clean examples.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-domain-finetuning/` and update plan backlinks.
