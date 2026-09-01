---
sprint_id: "PF-42-S01"
title: "Claude authentication source contract"
status: completed
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
- [x] Defined stable source, selection, health, metadata, and resolution result types with no secret-bearing fields.
- [x] Added versioned persistence through the vault's existing encrypted, locked transaction substrate; absence preserves existing installations.
- [x] Added deterministic selected-source, missing-source, conflict, invalid-state, restart, and redaction tests.

## Remaining

- [x] All CSA-01 implementation tasks are complete; provider behavior remains assigned to dependent sprints.

## Verification

- [x] `cd codex-rs && just fix -p codex-vault && just fmt` passed before final tests.
- [x] `cd codex-rs && just test -p codex-vault` passed 41/41; a focused zero-retry rerun confirmed the one transient nextest leak annotation was scheduling noise.
- [x] TUI applicability: none; this sprint adds no user-visible behavior.

## Exit evidence

- [x] Implementation commit `af6ff246754458d7cbbac06e7de673d6b6b005a4` recorded.
- [x] Contract has no raw credential field and serialized state is metadata-only.
- [x] Done and Remaining ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/claude-subscription-auth/`.
