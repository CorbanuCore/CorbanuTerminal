---
sprint_id: "PF-65-S01"
title: "Threat model and evidence review"
status: draft
plan_file: "docs/plans/proposed/portfolio-private-inference-research.md"
plan_feature: "PF-65"
execution_order: 1
owner: "Privacy research lead (assignment pending)"
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

# PF-65-S01 — Threat model and evidence review

## Execution mandate

- Deliver: docs/research/private-inference-research/threat-model.md; Every privacy claim names its adversary and evidence; uncertainty is explicit.
- Excludes: Privacy guarantees, proprietary training data, production deployment or assuming a named technology works from the conversation alone.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Private inference feasibility for Ambient and Corbanu](../../../plans/proposed/portfolio-private-inference-research.md)
- Feature: `PF-65`; acceptance: Every privacy claim names its adversary and evidence; uncertainty is explicit.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-private-inference-research.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/private-inference-research/threat-model.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/private-inference-research/pf-65-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Choose adversary, trust boundary, latency/cost tolerances and authorized experiment budget before experimentation.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Extract testable privacy requirements, distinguishing transport, storage, operator access and model-input protection.
- [ ] Research primary papers and implementation documentation; separate demonstrated capability from vendor claims.
- [ ] Define baseline, attacker capabilities, data-retention assumptions and disqualifying leaks.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Independent reviewer attempts to falsify each claimed boundary using the stated adversary.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-private-inference-research/` and update plan backlinks.
