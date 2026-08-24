---
sprint_id: "PF-20-S01"
title: "Versioned security persistence reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-20"
execution_order: 6
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-15-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-20-S01 — Versioned security persistence reconciliation

## Execution mandate

- Deliver: verify typed, versioned level persistence with explicit corrupt/unknown-state failure.
- Excludes: TUI confirmation, effective runtime policy, downgrade invalidation, and audit persistence.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-20`
- Acceptance advanced: restart restores a known level without transiently weakening policy.

## Code boundaries

- Existing: `codex-rs/config/src/config_toml.rs`; `codex-rs/core/src/config/{mod,edit}.rs`
- Generated/build: `codex-rs/core/config.schema.json`; Cargo and Bazel dependency files
- Tests: `codex-rs/core/src/config/{config_tests,edit_tests}.rs`

## Preconditions

- [x] PF-15-S01 is completed and archived.
- [x] Exact worktree coordinates match the plan.
- [x] Read `codex-rs/AGENTS.md` and `codex-rs/core/AGENTS.md` before corrective work.

## Done

- [x] Sprint record is linked to PF-20.
- [x] Commit `0e3f2dfd92` added versioned config, schema, editing, and persistence tests.
- [x] Reviewed legacy absence, explicit Permissive, unknown version, corrupt level, and atomic-write behavior; no corrective commit was required.
- [x] Ran `just write-config-schema`; the checked-in schema matched generated types without a diff.
- [x] Confirmed dependency metadata did not change during reconciliation, so no Bazel lock update was required.
- [x] Recorded final paths and environment-qualified test evidence in commit `b1db9c103a`.

## Remaining

None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-config && just fix -p codex-core` passed with pre-existing warnings only.
- [x] Format: `cd codex-rs && just fmt`; final tracked diff remained empty.
- [x] Focused tests: `codex-config` passed 229 tests and `codex-core config::` passed 477 tests under a clean fixed `TMPDIR`.
- [x] Schema: `cd codex-rs && just write-config-schema`; final schema diff was empty.
- [x] TUI applicability: none; PF-24 owns interactive persistence.

## Exit evidence

- [x] Original `0e3f2dfd92` and evidence `b1db9c103a` commits, schema result, and paths recorded; no corrective commit was required.
- [x] Test output recorded at `qa/security-levels/sprints/PF-20-S01/evidence.md`.
- [x] Ledgers reflect reality and the completed record is archived.
