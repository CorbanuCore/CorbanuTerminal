---
sprint_id: "PF-61-S03"
title: "Repair scope and release decision"
status: draft
plan_file: "docs/plans/proposed/portfolio-plan-backend-reconciliation.md"
plan_feature: "PF-61"
execution_order: 3
owner: "Jim Ricketts (proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-61-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-61-S03 — Repair scope and release decision

## Execution mandate

- Deliver: docs/research/plan-backend-reconciliation/decision.md; A named owner can select one repair with evidence and acceptance criteria; unresolved architecture decisions prevent readiness.
- Excludes: Deployments, production billing changes, buying inference, merging old worktrees or claiming per-wallet xAPI is live.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Plan and model-serving backend reconciliation](../../../plans/proposed/portfolio-plan-backend-reconciliation.md)
- Feature: `PF-61`; acceptance: A named owner can select one repair with evidence and acceptance criteria; unresolved architecture decisions prevent readiness.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-plan-backend-reconciliation.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/features/wallet-plan.md`; `codex-rs/tui/src/chatwidget/wallet_http.rs`; `codex-rs/tui/src/chatwidget/wallet_usage.rs`.
- Planned output: `docs/research/plan-backend-reconciliation/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/plan-backend-reconciliation/pf-61-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-61-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Rank reproduced discrepancies by customer impact and security risk; distinguish bounded fixes from new initiatives.
- [ ] For each approved candidate fix, draft exact target files, tests, migration/rollback risks and implementation sprint amendments; do not implement.
- [ ] Reconcile source-of-truth release/worktrees with Jim; obtain Travis's scope decisions and leave rejected/deferred items visible.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Cross-check the gap list against active-plan and sprint ledgers; rerun both governance checkers.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-plan-backend-reconciliation/` and update plan backlinks.
