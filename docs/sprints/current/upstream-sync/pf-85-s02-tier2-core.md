---
sprint_id: "PF-85-S02"
title: "Tier 2: core"
status: draft
plan_file: "docs/plans/active/upstream-sync-rust-v0.167.0.md"
plan_feature: "PF-85"
execution_order: 3
owner: "Slice owners per ledger; integrator merges"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-85-S01"
created: 2026-11-07
updated: 2026-11-19
---

# PF-85-S02 — Tier 2: core

## Execution mandate

- Deliver: `codex-core` compiles with tests on macOS, Linux and Windows (S2a–S2c and S3a core paths); config schema regenerated; main merge #3 (S2 exit; numbering in the program plan §4).
- Excludes: S3b port behaviour (listed as S3b ledger rows).
- Tracking issue: #1629. Details: [program plan §6](../../../management/upstreamSyncPlan.md).

## Plan linkage

- Plan: [Upstream Codex sync to rust-v0.167.0](../../../plans/active/upstream-sync-rust-v0.167.0.md).
- Feature: `PF-85`.
- Product citation: **Product principles** — “Maintain continuous Codex parity without removing Corbanu-specific behavior.”

## Code boundaries

- Write scope: Ledger rows with tier 2.
- Branch target: `sync/upstream-rust-v0.167.0` (PRs only).

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] S3a (security resolution, core paths and the Tier 1 sandbox crates) finished 2026-11-14: every S3a ledger
  row is resolved on the sync branch, the last in #2237. Exit summary:
  [#1628](https://github.com/CorbanuCore/CorbanuTerminal/issues/1628#issuecomment-6889180617) (also on #1532).
  Its core tests ran, or were only type-checked, on scratch builds (codex-core tests don't build on the branch
  yet); the compile pass reruns them.

- [x] Reconciled 2026-11-19 at sync `91efec006d` (sync-integrator): slices S2a 389/390, S2b 42/42 and S2c 131/131 ledger rows resolved. The open S2a row is the #2296 snapshot group (`-compile.tsv`), held for the final core run after #2274.
- [x] Main merge #3 (S2 exit) and later carries through #13 (#2791). Main is fully contained in sync at `15415856d7`.
- [x] Core library: Linux 4,876/0 at `51dc00e706` (astra-synccore7) and macOS 5,026/0 at `12d90cce4a` (astra-syncmac2; landing plan tables). Windows `codex-core` integration is down to the three D742 `agent_message_wake` cases ([run](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44594161347/job/135808168280), #2729).

## Remaining

- [ ] S2a #2296 snapshot-group row: close it on a post-#2274 core run (owner: sync core maintainer; #2563).
- [ ] Security-owner approval of `turn.rs`/`client.rs` call sites, or a link to the S3b rows that record it.
- [ ] D742 decision and fix for `agent_message_wake` (#2729).
- [ ] Code-blind functional design frozen before test-result disclosure, or non-user-facing N/A reason recorded.

## Verification

- [ ] `cargo test -p codex-core` on macOS and glitch; Windows via package CI.
- [ ] TUI applicability: none.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/upstream-sync/`.
