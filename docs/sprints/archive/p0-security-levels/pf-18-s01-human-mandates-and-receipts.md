---
sprint_id: "PF-18-S01"
title: "Human mandates and receipts reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-18"
execution_order: 4
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-16-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-18-S01 — Human mandates and receipts reconciliation

## Execution mandate

- Deliver: verify exact human-action mandates, replay denial, and secret-free receipts.
- Excludes: trusted preview TUI, signing adapters, credential use, and audit persistence.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-18`
- Acceptance advanced: mutation, replay, duplicate submission, or stale approval fails.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{mandate,digest,authorization}.rs`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [x] PF-16-S01 is completed and archived.
- [x] Exact worktree coordinates match the plan.
- [x] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Done

- [x] Sprint record is linked to PF-18.
- [x] Commit `e22a35ccf2` added human mandates, canonical action binding, replay state, and `ActionReceipt`.
- [x] Reviewed canonical encoding, human-principal binding, expiry, replay state, and receipt redaction.
- [x] Proved mutation of every approved action dimension invalidates the mandate.
- [x] Corrective commit `4b438c46bb` closed pre-approval use and added duplicate, stale, malformed, clock-failure, and receipt-serialization cases.
- [x] Recorded the final public API in evidence commit `cbad39a410`.

## Remaining

None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy` passed.
- [x] Format: `cd codex-rs && just fmt`; final diff inspected.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy mandate` passed 2 tests.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy` passed 13 tests.
- [x] TUI applicability: none; PF-25 owns trusted human interaction.

## Exit evidence

- [x] Original `e22a35ccf2`, corrective `4b438c46bb`, and evidence `cbad39a410` commits and paths recorded.
- [x] Test output recorded at `qa/security-levels/sprints/PF-18-S01/evidence.md`.
- [x] Ledgers reflect reality and the completed record is archived.
