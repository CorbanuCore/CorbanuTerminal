---
sprint_id: "PF-68-S02"
title: "Read-only user and risk journey"
status: draft
plan_file: "docs/plans/proposed/portfolio-onchain-stock-access.md"
plan_feature: "PF-68"
execution_order: 2
owner: "Alex Good (product/commercial lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-68-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-68-S02 — Read-only user and risk journey

## Execution mandate

- Deliver: docs/research/onchain-stock-access/paper-journey.md; The user can distinguish market exposure from ownership and identify all unresolved exit risks.
- Excludes: Opening accounts/entities, trading, signing, deposits, evading eligibility or claiming tokenized exposure equals shareholder ownership.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [On-chain stock access and broker-wrapper feasibility](../../../plans/proposed/portfolio-onchain-stock-access.md)
- Feature: `PF-68`; acceptance: The user can distinguish market exposure from ownership and identify all unresolved exit risks.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-onchain-stock-access.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/features/wallet-plan.md`.
- Planned output: `docs/research/onchain-stock-access/paper-journey.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/onchain-stock-access/pf-68-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-68-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Specify discovery, quote inspection, rights disclosure, permissions and hypothetical order lifecycle using synthetic balances.
- [ ] Test closed market, stale quote, unsupported region, unavailable redemption and venue outage as paper scenarios.
- [ ] Map any future integration to existing brokerage/security ownership; prohibit a second signing/credential path.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Tabletop all five failures; verify no real order or credential access is needed.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-onchain-stock-access/` and update plan backlinks.
