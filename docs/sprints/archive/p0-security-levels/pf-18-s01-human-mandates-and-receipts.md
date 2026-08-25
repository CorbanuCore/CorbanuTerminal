---
sprint_id: "PF-18-S01"
title: "Human mandates and receipts reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-18"
execution_order: 4
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-16-S01"
created: 2026-08-24
updated: 2026-08-25
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

## Record creation

- [x] Sprint record is linked to PF-18.
- [x] Commit `e22a35ccf2` added human mandates, canonical action binding, replay state, and `ActionReceipt`.

## Done

- [x] Review canonical encoding, human-principal binding, expiry, replay state, and receipt redaction.
- [x] Prove changing any approved action field invalidates the mandate.
- [x] Add or correct duplicate, stale, malformed, clock-failure, and receipt-serialization cases.
- [x] Record any corrective commit and final public API.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy mandate`.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [x] TUI applicability: none; PF-25 owns trusted human interaction.

## Exit evidence

- [x] Commits and changed paths recorded.
- [x] Test output linked under `qa/security-levels/sprints/PF-18-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
