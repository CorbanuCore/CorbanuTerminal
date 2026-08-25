---
sprint_id: "PF-19-S01"
title: "Revocation contract reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-19"
execution_order: 5
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-17-S01, PF-18-S01"
created: 2026-08-24
updated: 2026-08-25
---

# PF-19-S01 — Revocation contract reconciliation

## Execution mandate

- Deliver: verify durable revocation semantics that override grants, mandates, and cached decisions.
- Excludes: persistence wiring, kill-switch TUI, credential broker integration, and runtime adapters.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-19`
- Acceptance advanced: revoked authority cannot start or resume another protected operation.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{revocation,grant,mandate,lib}.rs`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [x] PF-17-S01 and PF-18-S01 are completed and archived.
- [x] Exact worktree coordinates match the plan.
- [x] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Record creation

- [x] Sprint record is linked to PF-19.
- [x] Commit `8a3b416c26` added revocation events/state and policy contract tests.

## Done

- [x] Review target, generation, ordering, idempotency, and cache-invalidation semantics.
- [x] Prove revocation dominates active grants, pending mandates, and replay state.
- [x] Add or correct race, duplicate-event, unknown-target, and rollback cases.
- [x] Record any corrective commit and final public API.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy revocation`.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [x] TUI applicability: none; PF-25 owns revocation and kill-switch interaction.

## Exit evidence

- [x] Commits and changed paths recorded.
- [x] Test output linked under `qa/security-levels/sprints/PF-19-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
