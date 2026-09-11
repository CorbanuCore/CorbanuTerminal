---
sprint_id: "PF-62-S01"
title: "Provider terms and economics screen"
status: draft
plan_file: "docs/plans/proposed/portfolio-compute-referrals.md"
plan_feature: "PF-62"
execution_order: 1
owner: "Alex Good (commercial lead, proposed)"
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

# PF-62-S01 — Provider terms and economics screen

## Execution mandate

- Deliver: docs/research/compute-referrals/provider-matrix.md; At least two eligible candidates are compared, or documented evidence explains why fewer exist.
- Excludes: Opening accounts, accepting partner terms, buying GPUs, distributing API keys, editing the website or enabling referrals.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Compute referrals and provider onboarding discovery](../../../plans/proposed/portfolio-compute-referrals.md)
- Feature: `PF-62`; acceptance: At least two eligible candidates are compared, or documented evidence explains why fewer exist.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-compute-referrals.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/compute-referrals/provider-matrix.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/compute-referrals/pf-62-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Alex confirms commercial eligibility and required disclosure; Travis chooses one provider and approves any product-spec expansion.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Research primary provider documentation for existing supported compute routes and referral terms; date every claim.
- [ ] Compare eligibility, attribution window, payouts, support burden and a pessimistic unit-economics model; unknown terms remain unknown.
- [ ] Recommend one candidate or no-go; request Alex's commercial decision without contacting providers.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Check every commercial claim against a dated primary source; recompute the pessimistic model.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-compute-referrals/` and update plan backlinks.
