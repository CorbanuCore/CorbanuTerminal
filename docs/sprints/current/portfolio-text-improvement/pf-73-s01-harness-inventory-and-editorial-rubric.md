---
sprint_id: "PF-73-S01"
title: "Harness inventory and editorial rubric"
status: draft
plan_file: "docs/plans/proposed/portfolio-text-improvement.md"
plan_feature: "PF-73"
execution_order: 1
owner: "Editorial workflow owner (assignment pending)"
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

# PF-73-S01 — Harness inventory and editorial rubric

## Execution mandate

- Deliver: docs/research/text-improvement/rubric.md; Every sample has immutable original text and pass/fail fidelity criteria.
- Excludes: Rebuilding a missing harness from guesswork, publishing text, sending private drafts to unapproved providers or automatic model switching.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Text-improvement harness qualification](../../../plans/proposed/portfolio-text-improvement.md)
- Feature: `PF-73`; acceptance: Every sample has immutable original text and pass/fail fidelity criteria.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-text-improvement.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/text-improvement/rubric.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/text-improvement/pf-73-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Identify the real harness, sample permissions, target voice and exact approved model/provider before any remote evaluation.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Locate the existing harness with its owner and document inputs, model route, prompts, output and rollback.
- [ ] Select consented/public sample texts and create a rubric for clarity, fidelity, voice and factual preservation.
- [ ] Seed factual drift, quotation changes and hostile embedded instructions into controlled tests.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Two evaluators score seeded bad examples; all critical factual/quotation mutations must be detected.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-text-improvement/` and update plan backlinks.
