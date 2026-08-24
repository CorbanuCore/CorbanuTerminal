---
sprint_id: "PF-18-S01"
title: "Human mandates and receipts reconciliation"
status: draft
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

- [ ] PF-16-S01 is completed and archived.
- [ ] Exact worktree coordinates match the plan.
- [ ] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Done

- [x] Sprint record is linked to PF-18.
- [x] Commit `e22a35ccf2` added human mandates, canonical action binding, replay state, and `ActionReceipt`.

## Remaining

- [ ] Review canonical encoding, human-principal binding, expiry, replay state, and receipt redaction.
- [ ] Prove changing any approved action field invalidates the mandate.
- [ ] Add or correct duplicate, stale, malformed, clock-failure, and receipt-serialization cases.
- [ ] Record any corrective commit and final public API.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy mandate`.
- [ ] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [ ] TUI applicability: none; PF-25 owns trusted human interaction.

## Exit evidence

- [ ] Commits and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-18-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
