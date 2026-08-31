---
sprint_id: "PF-42-S01"
title: "Claude authentication source contract"
status: in_progress
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-42"
execution_order: 1
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/vault/src/claude_auth.rs, codex-rs/vault/src/claude_auth_tests.rs, codex-rs/vault/src/lib.rs, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/"
integration_gate: "Jim Ricketts freezes the typed contract on feat/claude-subscription-auth-isolated, runs vault plus governance tests, archives CSA-01, and only then allocates CSA-02 or CSA-03 against the same base."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
branch: "feat/claude-subscription-auth-isolated"
base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
depends_on: "none"
created: 2026-08-30
updated: 2026-08-30
---

# CSA-01 — Claude authentication source contract

## Execution mandate

- Deliver: typed metadata-only source, selection, health, deterministic resolution, and atomic persistence contracts.
- Excludes: secret enrollment, Claude store discovery, provider behavior, and user-visible TUI changes.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/claude-subscription-auth.md).
- Feature: `PF-42` (plan alias `CSA-01`).
- Acceptance advanced: persist an explicit source choice and fail visibly instead of silently changing identity or billing path.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs` encrypted credential boundary.
- Planned: `codex-rs/vault/src/claude_auth.rs` and sibling tests.

## Preconditions

- [x] Plan is active.
- [x] Dependencies are completed; CSA-01 has none.
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Serial owner/lane/scope and receiving integration gate are recorded.

## Done

- [x] Sprint record created and linked to CSA-01.

## Remaining

- [ ] Define stable source, selection, health, metadata, and resolution result types with no secret-bearing fields.
- [ ] Add versioned atomic metadata persistence that preserves absence for existing installations.
- [ ] Add deterministic selected-source, missing-source, conflict, invalid-state, restart, and redaction tests.

## Verification

- [ ] `cd codex-rs && just fix -p codex-vault && just fmt` precedes final tests.
- [ ] `cd codex-rs && just test -p codex-vault` passes on the final CSA-01 tree.
- [ ] TUI applicability: none; this sprint adds no user-visible behavior.

## Exit evidence

- [ ] Implementation commit and exact test output recorded.
- [ ] Contract has no raw credential field and serialized state is metadata-only.
- [ ] Done and Remaining ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/claude-subscription-auth/`.
