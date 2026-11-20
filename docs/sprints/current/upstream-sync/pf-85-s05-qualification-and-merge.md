---
sprint_id: "PF-85-S05"
title: "Qualification and merge to main"
status: draft
plan_file: "docs/plans/active/upstream-sync-rust-v0.167.0.md"
plan_feature: "PF-85"
execution_order: 6
owner: "Upstream-sync integrator (maintainer)"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-85-S03, PF-85-S04"
created: 2026-11-07
updated: 2026-11-19
---

# PF-85-S05 — Qualification and merge to main

## Execution mandate

- Deliver: Final PR merged to `main` with all lanes green, SG3 and the qualification record; canary.
- Excludes: New upstream tags.
- Tracking issue: #1632. Details: [program plan §6](../../../management/upstreamSyncPlan.md).

## Plan linkage

- Plan: [Upstream Codex sync to rust-v0.167.0](../../../plans/active/upstream-sync-rust-v0.167.0.md).
- Feature: `PF-85`.
- Product citation: **Product principles** — “Maintain continuous Codex parity without removing Corbanu-specific behavior.”

## Code boundaries

- Write scope: Whole workspace (integrator).
- Branch target: the sync branch, then the final PR `sync/upstream-rust-v0.167.0` → `main`.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.

- [x] 2026-11-19 at sync `91efec006d` (sync-integrator): SG1 structural checks recorded in [sg1-supply-chain.md](../../../management/upstreamSync/sg1-supply-chain.md). RUSTSEC-2026-0544 (`libz-sys`, #2799) is still open. CI-mode and CODEOWNERS restoration is prepared as held draft #2801.

## Remaining

Pre-merge gates (all on the exact final candidate):

- [ ] Final main carry no more than 48 hours before landing; apply #2801 (CI mode and CODEOWNERS) with coordinator approval; full run with no expected-red exception; SG1 re-run, #2799 fixed; SG3 security-path diff from the SG2-signed SHA.
- [ ] Rollback/compatibility: vault files open in the last released binary (D319). State DB: 0.1.49 refuses the upgraded DB (`MIGRATIONS.md`), so restore from backup unless Travis decides otherwise; backup/upgrade/restore evidence on the candidate.
- [ ] Functional qualification, Windows human checklist and release dry run (landing plan gates 4–6); code-blind design frozen first.
- [ ] Final independent review (up to 8 rounds, D307), matching corbanu-security approval and Travis's final approval.

After Travis's approval:

- [ ] Travis's guarded merge; 48-hour merge hold on `main` (D306) and Mac/Windows canary; then complete and archive this record.

## Verification

- [ ] Full CI, `just test` on macOS, Windows PC gate, release dry run.
- [ ] TUI applicability: yes, TensorCash and Isometric Game.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/upstream-sync/`.
