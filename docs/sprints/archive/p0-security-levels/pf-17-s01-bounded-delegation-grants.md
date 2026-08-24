---
sprint_id: "PF-17-S01"
title: "Bounded delegation grants reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-17"
execution_order: 3
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-16-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-17-S01 — Bounded delegation grants reconciliation

## Execution mandate

- Deliver: verify narrow, expiring grants whose delegation can only reduce authority.
- Excludes: grant TUI, credential resolution, runtime enforcement, and persistence.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-17`
- Acceptance advanced: adjacent action, actor, destination, limit, and post-expiry use fail.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{grant,bounded,authorization}.rs`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [x] PF-16-S01 is completed and archived.
- [x] Exact worktree coordinates match the plan.
- [x] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Done

- [x] Sprint record is linked to PF-17.
- [x] Commit `d68c4dbc95` added `BoundedGrant`, `GrantScope`, delegation checks, and expiry semantics.
- [x] Reviewed every grant field against the bounded-grant contract.
- [x] Proved delegation preserves the human principal and acting-agent chain while narrowing every scope dimension.
- [x] Corrective commit `5a03e1e0ec` closed extra-asset and pre-issuance widening and added mutation, expiry, adjacent-resource, quantitative-limit, and child-widening cases.
- [x] Recorded the final API and paths in evidence commit `f6cba75068`.

## Remaining

None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy` passed.
- [x] Format: `cd codex-rs && just fmt`; final diff inspected.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy grant` passed 2 tests.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy` passed 12 tests.
- [x] TUI applicability: none; PF-25 tests grant presentation and confirmation.

## Exit evidence

- [x] Original `d68c4dbc95`, corrective `5a03e1e0ec`, and evidence `f6cba75068` commits and paths recorded.
- [x] Test output recorded at `qa/security-levels/sprints/PF-17-S01/evidence.md`.
- [x] Ledgers reflect reality and the completed record is archived.
