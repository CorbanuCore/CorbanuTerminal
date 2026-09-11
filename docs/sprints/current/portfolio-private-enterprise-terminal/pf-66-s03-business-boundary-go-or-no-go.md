---
sprint_id: "PF-66-S03"
title: "Business boundary go or no-go"
status: draft
plan_file: "docs/plans/proposed/portfolio-private-enterprise-terminal.md"
plan_feature: "PF-66"
execution_order: 3
owner: "Alex Good (commercial lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-66-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-66-S03 — Business boundary go or no-go

## Execution mandate

- Deliver: docs/research/private-enterprise-terminal/decision.md; There is no implied enterprise build commitment; decision and evidence gaps are visible.
- Excludes: Sales outreach, customer promises, multi-tenant infrastructure, procurement commitments or treating Venice/Nebius/Palantir capabilities as verified.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Private enterprise Terminal opportunity qualification](../../../plans/proposed/portfolio-private-enterprise-terminal.md)
- Feature: `PF-66`; acceptance: There is no implied enterprise build commitment; decision and evidence gaps are visible.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-private-enterprise-terminal.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/private-enterprise-terminal/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/private-enterprise-terminal/pf-66-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-66-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Have Alex assess demand evidence and identify whether buyer interviews are needed next.
- [ ] Have Travis resolve trader-first fit versus separate-product ownership; record business and engineering opportunity cost.
- [ ] If approved, draft a narrow paid-pilot proposal with acceptance, security review and explicit next authorization.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Check that all pilot promises are either demonstrated or clearly conditional.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-private-enterprise-terminal/` and update plan backlinks.
