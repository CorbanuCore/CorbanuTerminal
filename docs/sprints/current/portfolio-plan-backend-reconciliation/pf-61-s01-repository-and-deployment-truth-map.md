---
sprint_id: "PF-61-S01"
title: "Repository and deployment truth map"
status: draft
plan_file: "docs/plans/proposed/portfolio-plan-backend-reconciliation.md"
plan_feature: "PF-61"
execution_order: 1
owner: "Jim Ricketts (proposed)"
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

# PF-61-S01 — Repository and deployment truth map

## Execution mandate

- Deliver: docs/research/plan-backend-reconciliation/current-state.md; Every boundary has an evidence pointer and owner; no branch is treated as deployed merely because code exists.
- Excludes: Deployments, production billing changes, buying inference, merging old worktrees or claiming per-wallet xAPI is live.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Plan and model-serving backend reconciliation](../../../plans/proposed/portfolio-plan-backend-reconciliation.md)
- Feature: `PF-61`; acceptance: Every boundary has an evidence pointer and owner; no branch is treated as deployed merely because code exists.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-plan-backend-reconciliation.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/features/wallet-plan.md`; `codex-rs/tui/src/chatwidget/wallet_http.rs`; `codex-rs/tui/src/chatwidget/wallet_usage.rs`.
- Planned output: `docs/research/plan-backend-reconciliation/current-state.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/plan-backend-reconciliation/pf-61-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; An operator must identify the authoritative deployment and authorize any production inspection; secret values stay outside the report.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Inventory public and private repository branches, deployment references and catalog/privacy labels without reading credentials.
- [ ] Trace Corbanu API top-up, wallet proof, keys, upstream dispatch, usage, settlement/retry and funding boundaries; classify legacy Plan surfaces against the receiving migration decision.
- [ ] Classify each difference as live, built-not-live, obsolete or unverified; identify overlaps with large-image and provider-auth work.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Independent reviewer follows every row to code or dated operator evidence.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-plan-backend-reconciliation/` and update plan backlinks.
