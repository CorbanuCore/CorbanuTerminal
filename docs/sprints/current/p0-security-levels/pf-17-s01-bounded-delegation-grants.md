---
sprint_id: "PF-17-S01"
title: "Bounded delegation grants reconciliation"
status: draft
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

- [ ] PF-16-S01 is completed and archived.
- [ ] Exact worktree coordinates match the plan.
- [ ] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Done

- [x] Sprint record is linked to PF-17.
- [x] Commit `d68c4dbc95` added `BoundedGrant`, `GrantScope`, delegation checks, and expiry semantics.

## Remaining

- [ ] Review every grant field against the plan's bounded-grant contract.
- [ ] Prove delegation preserves the human principal and acting-agent chain while narrowing all scopes.
- [ ] Add or correct mutation, expiry, adjacent-resource, quantitative-limit, and child-widening cases.
- [ ] Record any corrective commit and final public API.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy grant`.
- [ ] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [ ] TUI applicability: none; PF-25 tests grant presentation and confirmation.

## Exit evidence

- [ ] Commits and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-17-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
