---
sprint_id: "PF-85-S04"
title: "Tier 3: TUI, app-server, exec, CLI"
status: draft
plan_file: "docs/plans/active/upstream-sync-rust-v0.167.0.md"
plan_feature: "PF-85"
execution_order: 5
owner: "Slice owners per ledger; integrator merges"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-85-S02"
created: 2026-11-07
updated: 2026-11-14
---

# PF-85-S04 — Tier 3: TUI, app-server, exec, CLI

## Execution mandate

- Deliver: Workspace compiles everywhere; S4a–S4e resolved; CI mode reverted; true-TUI QA and demo reels.
- Excludes: S3b port.
- Tracking issue: #1631. Details: [program plan §6](../../../management/upstreamSyncPlan.md).

## Plan linkage

- Plan: [Upstream Codex sync to rust-v0.167.0](../../../plans/active/upstream-sync-rust-v0.167.0.md).
- Feature: `PF-85`.
- Product citation: **Product principles** — “Maintain continuous Codex parity without removing Corbanu-specific behavior.”

## Code boundaries

- Write scope: Ledger rows with tier 3.
- Branch target: `sync/upstream-rust-v0.167.0` (PRs only).

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.
- [x] Early start authorized (coordinator, 2026-11-14): S04 starts before the `depends_on` PF-85-S02 exit, in
  parallel with the last S3a rows and main merge #3 (#2176). Order: S4c app-server, then S4d exec; S4e stays
  contributor-friendly. Claim on #1631; status on #1532.
- [x] PF-85-S02's library work (S2a/S2b/S2c) is merged on the sync branch; its tests are deferred to the
  compile pass, so S4 slice PRs carry no S2 test evidence and core-test failures there are expected-red.

## Done

- [x] Sprint record created and linked to one plan feature.

## Remaining

- [ ] Slice PRs S4a–S4e; snapshots re-accepted after reading each diff.
- [ ] Revert the sync-branch CI mode (`voice-host` stays in `.bazelignore` and the Cargo `exclude` list, D316).
- [ ] True-TUI QA and demo reels.
- [ ] Code-blind functional design frozen before test-result disclosure, or non-user-facing N/A reason recorded.

## Verification

- [ ] `blocking-ci`, `tmux-smoke`, `orchestrate-tui-matrix`.
- [ ] TUI applicability: yes, plan §9 list in tmux.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/upstream-sync/`.
