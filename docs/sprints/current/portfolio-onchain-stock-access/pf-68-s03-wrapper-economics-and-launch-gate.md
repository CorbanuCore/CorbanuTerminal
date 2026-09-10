---
sprint_id: "PF-68-S03"
title: "Wrapper economics and launch gate"
status: draft
plan_file: "docs/plans/proposed/portfolio-onchain-stock-access.md"
plan_feature: "PF-68"
execution_order: 3
owner: "Alex Good (product/commercial lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-68-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-68-S03 — Wrapper economics and launch gate

## Execution mandate

- Deliver: docs/research/onchain-stock-access/decision.md; A go does not authorize live transactions; exact next permissions and blocked questions are listed.
- Excludes: Opening accounts/entities, trading, signing, deposits, evading eligibility or claiming tokenized exposure equals shareholder ownership.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [On-chain stock access and broker-wrapper feasibility](../../../plans/proposed/portfolio-onchain-stock-access.md)
- Feature: `PF-68`; acceptance: A go does not authorize live transactions; exact next permissions and blocked questions are listed.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-onchain-stock-access.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/wallet-plan.md`.
- Planned output: `docs/research/onchain-stock-access/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/onchain-stock-access/pf-68-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-68-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Model fee/revenue and support costs with sourced ranges and pessimistic cases, not return promises.
- [ ] Separate a read-only discovery MVP from any trading/custody/financialization proposal.
- [ ] Have Alex and Travis accept/reject the next scope subject to qualified legal review; draft future sprints only for approved scope.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Recompute economic cases and cross-check legal/eligibility exclusions.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-onchain-stock-access/` and update plan backlinks.
