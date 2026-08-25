---
sprint_id: "PF-17-S01"
title: "Bounded delegation grants reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-17"
execution_order: 3
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-16-S01"
created: 2026-08-24
updated: 2026-08-25
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

## Record creation

- [x] Sprint record is linked to PF-17.
- [x] Commit `d68c4dbc95` added `BoundedGrant`, `GrantScope`, delegation checks, and expiry semantics.

## Done

- [x] Review every grant field against the plan's bounded-grant contract.
- [x] Prove delegation preserves the human principal and acting-agent chain while narrowing all scopes.
- [x] Add or correct mutation, expiry, adjacent-resource, quantitative-limit, and child-widening cases.
- [x] Record any corrective commit and final public API.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy grant`.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [x] TUI applicability: none; PF-25 tests grant presentation and confirmation.

## Exit evidence

- [x] Commits and changed paths recorded.
- [x] Test output linked under `qa/security-levels/sprints/PF-17-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
