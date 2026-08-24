---
sprint_id: "PF-19-S01"
title: "Revocation contract reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-19"
execution_order: 5
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-17-S01, PF-18-S01"
created: 2026-08-24
updated: 2026-08-24
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

## Done

- [x] Sprint record is linked to PF-19.
- [x] Commit `8a3b416c26` added revocation events/state and policy contract tests.
- [x] Reviewed and corrected target, generation, ordering, idempotency, and cache-invalidation semantics.
- [x] Proved revocation dominates active grants and pending mandates, including acting-agent revocation.
- [x] Corrective commit `46e9121ebe` added race convergence, duplicate-event, unknown-target, corrupt-generation, restart, and rollback cases.
- [x] Recorded the final API in evidence commit `655a66c173`.

## Remaining

None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy` passed.
- [x] Format: `cd codex-rs && just fmt`; final diff inspected.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy revocation` passed 2 tests.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy` passed 14 tests.
- [x] TUI applicability: none; PF-25 owns revocation and kill-switch interaction.

## Exit evidence

- [x] Original `8a3b416c26`, corrective `46e9121ebe`, and evidence `655a66c173` commits and paths recorded.
- [x] Test output recorded at `qa/security-levels/sprints/PF-19-S01/evidence.md`.
- [x] Ledgers reflect reality and the completed record is archived.
