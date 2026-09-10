---
sprint_id: "PF-66-S01"
title: "Buyer and competitor evidence map"
status: draft
plan_file: "docs/plans/proposed/portfolio-private-enterprise-terminal.md"
plan_feature: "PF-66"
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

# PF-66-S01 — Buyer and competitor evidence map

## Execution mandate

- Deliver: docs/research/private-enterprise-terminal/opportunity.md; Each claimed differentiator has evidence or is explicitly an untested hypothesis.
- Excludes: Sales outreach, customer promises, multi-tenant infrastructure, procurement commitments or treating Venice/Nebius/Palantir capabilities as verified.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Private enterprise Terminal opportunity qualification](../../../plans/proposed/portfolio-private-enterprise-terminal.md)
- Feature: `PF-66`; acceptance: Each claimed differentiator has evidence or is explicitly an untested hypothesis.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-private-enterprise-terminal.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/private-enterprise-terminal/opportunity.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/private-enterprise-terminal/pf-66-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Travis must explicitly decide whether this fits Corbanu or belongs in a separate business; Alex owns buyer validation.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Research current primary vendor materials for Venice, Nebius and Palantir as hypotheses, not endorsements.
- [ ] Compare buyer, data locality, operator access, retention, deployment, procurement and economics with Corbanu's actual capabilities.
- [ ] Draft interview questions and identify required commercial validation; do not contact anyone.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Review claim/source/date coverage and identify at least one credible reason the opportunity could fail.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-private-enterprise-terminal/` and update plan backlinks.
