---
sprint_id: "PF-62-S03"
title: "Commercial and delivery handoff"
status: draft
plan_file: "docs/plans/proposed/portfolio-compute-referrals.md"
plan_feature: "PF-62"
execution_order: 3
owner: "Alex Good (commercial lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-62-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-62-S03 — Commercial and delivery handoff

## Execution mandate

- Deliver: docs/research/compute-referrals/decision.md; A no-go is a successful research result; a go names provider, authorized terms and bounded implementation scope.
- Excludes: Opening accounts, accepting partner terms, buying GPUs, distributing API keys, editing the website or enabling referrals.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Compute referrals and provider onboarding discovery](../../../plans/proposed/portfolio-compute-referrals.md)
- Feature: `PF-62`; acceptance: A no-go is a successful research result; a go names provider, authorized terms and bounded implementation scope.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-compute-referrals.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/compute-referrals/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/compute-referrals/pf-62-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-62-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Have Alex validate terms, economics and disclosure questions; mark unresolved legal questions for qualified review.
- [ ] Inspect exact chosen-provider code only after selection; produce an allocated-boundary proposal and implementation test inventory.
- [ ] Record go/no-go, spend cap and receiving owner; leave product work draft until activation.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Independent review checks that no signed agreement, purchase or activation is implied.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-compute-referrals/` and update plan backlinks.
