---
sprint_id: "PF-61-S02"
title: "Offline compatibility fixture pack"
status: draft
plan_file: "docs/plans/proposed/portfolio-plan-backend-reconciliation.md"
plan_feature: "PF-61"
execution_order: 2
owner: "Jim Ricketts (proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-61-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-61-S02 — Offline compatibility fixture pack

## Execution mandate

- Deliver: docs/research/plan-backend-reconciliation/contracts.md; All six failure classes map to existing tests or explicitly specified missing tests with expected values.
- Excludes: Deployments, production billing changes, buying inference, merging old worktrees or claiming per-wallet xAPI is live.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Plan and model-serving backend reconciliation](../../../plans/proposed/portfolio-plan-backend-reconciliation.md)
- Feature: `PF-61`; acceptance: All six failure classes map to existing tests or explicitly specified missing tests with expected values.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-plan-backend-reconciliation.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/features/wallet-plan.md`; `codex-rs/tui/src/chatwidget/wallet_http.rs`; `codex-rs/tui/src/chatwidget/wallet_usage.rs`.
- Planned output: `docs/research/plan-backend-reconciliation/contracts.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/plan-backend-reconciliation/pf-61-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-61-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Define offline fixtures for API catalog, revoked/invalid keys, insufficient balance, oversized image, partial stream and retry accounting; legacy Plan fixtures test retirement, not renewed entitlement.
- [ ] Record exact existing public/private tests that cover each fixture and the gaps; run only offline tests with live-purchase paths excluded.
- [ ] Define acceptance and rollback criteria for each confirmed discrepancy, separating auth, billing and transport ownership.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Public from codex-rs: just test -p codex-tui wallet_http; private test commands must be vetted against live hooks before execution.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-plan-backend-reconciliation/` and update plan backlinks.
